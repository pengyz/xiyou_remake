//! STAGE-B-005（系统对话框版）：游戏内右滑退出确认 = Android 标准 `AlertDialog`（壳层实现，游戏核心零改动）。
//!
//! 用户 2026-10-27：游戏内样式评审后改裁"不用游戏内UI，直接用Android标准关闭UI，作为shell层实现，提醒效果更明确"；
//! 文案 = 标题"退出游戏" / 正文"是否确定退出游戏？未保存的进度将会丢失。" / 按钮"取消""退出"；框开时音乐继续。
//!
//! # 结构
//!
//! | 层 | 职责 |
//! |---|---|
//! | Java `com.jjxy.game.ExitConfirm`（android/app/src/main/java） | UI 线程弹框（主题 DeviceDefault 随系统深浅色）；结果存静态位，`take()` 取走 |
//! | 本模块 `android::ExitDialog`（JNI 工作线程，feedback.rs 同式） | attach 一次；**经 `activity.getClassLoader().loadClass` 解析辅助类**（原生线程 `FindClass` 走系统类加载器、找不到应用类）；show 请求经通道投递；未决期每 50 ms 轮询 `take()`；结果经通道回送 |
//! | runtime.rs `back_requests_exit_dialog` / `on_exit_dialog_result` | 状态机：地图态返回 ⇒ 未决（暂停）⇒ 退出 = 进程退出请求 / 取消 = 继续（宿主钉 tests/exit_dialog.rs） |
//!
//! 失效保底：辅助类未就绪 ⇒ 不请求（BACK 照旧 -7 开游戏内菜单）；弹出调用异常 / 轮询异常 ⇒ 按"取消"回送（不卡暂停）。

/// 对话框结果（Java `ExitConfirm.EXIT` / `CANCEL` 映射）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogResult {
    /// 选"退出"。
    Exit,
    /// 取消（"取消"按钮 / 返回 / 点框外 / 无法弹出）。
    Cancel,
}

/// Java `ExitConfirm.take()` 返回码 → 结果（0 = 未决或已取走；未知码按未决处理）。
pub fn result_from_code(code: i32) -> Option<DialogResult> {
    match code {
        1 => Some(DialogResult::Exit),
        2 => Some(DialogResult::Cancel),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{result_from_code, DialogResult};

    /// 与 Java 常量 NONE=0 / EXIT=1 / CANCEL=2 一一对应（ExitConfirm.java）。
    #[test]
    fn take_codes_map_to_results() {
        assert_eq!(result_from_code(0), None, "NONE = 未决");
        assert_eq!(result_from_code(1), Some(DialogResult::Exit));
        assert_eq!(result_from_code(2), Some(DialogResult::Cancel));
        assert_eq!(result_from_code(7), None, "未知码不当结果");
    }
}

/// Android 出口：JNI 工作线程（主循环零 JNI、零阻塞）。
#[cfg(target_os = "android")]
pub mod android {
    use super::{result_from_code, DialogResult};
    use crate::feedback::android::{call_object, exception, global};
    use jni_sys::{jmethodID, jobject, jvalue, JNIEnv, JavaVM, JNI_OK};
    use std::ffi::{c_void, CStr};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    /// 未决期轮询 `take()` 的间隔（< 逻辑拍 75 ms：结果下一拍内生效）。
    const POLL: Duration = Duration::from_millis(50);

    /// ANativeActivity 的 `vm` / `clazz`（框架持有的全局引用，活动存活期有效）——只读跨线程传递。
    struct Handles {
        vm: *mut JavaVM,
        activity: jobject,
    }
    unsafe impl Send for Handles {}

    /// 退出确认对话框出口：show 请求 → 工作线程；结果 ← 工作线程。发送端随本值 drop ⇒ 线程收尾
    /// （删全局引用 + DetachCurrentThread）。
    pub struct ExitDialog {
        tx: mpsc::Sender<()>,
        rx: mpsc::Receiver<DialogResult>,
        ready: Arc<AtomicBool>,
    }

    impl ExitDialog {
        /// # Safety
        /// `vm` / `activity` 须为 ANativeActivity 的 `vm` / `clazz`。
        pub unsafe fn spawn(vm: *mut c_void, activity: *mut c_void, log: fn(&str)) -> ExitDialog {
            let (tx, cmd) = mpsc::channel::<()>();
            let (out, rx) = mpsc::channel::<DialogResult>();
            let ready = Arc::new(AtomicBool::new(false));
            let h = Handles { vm: vm as *mut JavaVM, activity: activity as jobject };
            let flag = ready.clone();
            let spawned = std::thread::Builder::new()
                .name("jjxy-exit-dialog".into())
                .spawn(move || unsafe { worker(h, cmd, out, flag, log) });
            if let Err(e) = spawned {
                log(&format!("exit dialog thread spawn err {e}"));
            }
            ExitDialog { tx, rx, ready }
        }

        /// 辅助类已解析可用（未就绪 ⇒ 调用方不请求，BACK 照旧 -7）。
        pub fn available(&self) -> bool {
            self.ready.load(Ordering::Acquire)
        }

        /// 请求弹出（非阻塞）。
        pub fn show(&self) {
            let _ = self.tx.send(());
        }

        /// 取结果（非阻塞；None = 未决或无）。
        pub fn try_result(&self) -> Option<DialogResult> {
            self.rx.try_recv().ok()
        }
    }

    /// 辅助类（全局引用）+ 静态方法 ID。
    struct Api {
        cls: jobject,
        show: jmethodID,
        take: jmethodID,
    }

    unsafe fn worker(
        h: Handles,
        cmd: mpsc::Receiver<()>,
        out: mpsc::Sender<DialogResult>,
        ready: Arc<AtomicBool>,
        log: fn(&str),
    ) {
        let vm = h.vm;
        let Some(attach) = (**vm).AttachCurrentThread else { return };
        let mut env: *mut JNIEnv = std::ptr::null_mut();
        let rc = attach(vm, &mut env as *mut *mut JNIEnv as *mut *mut c_void, std::ptr::null_mut());
        if rc != JNI_OK || env.is_null() {
            log(&format!("exit dialog jni attach failed rc={rc}"));
            return;
        }
        let api = resolve(env, h.activity);
        log(&format!("exit dialog jni init ok={}", api.is_some()));
        if let Some(api) = api {
            ready.store(true, Ordering::Release);
            let mut pending = false;
            loop {
                let disconnected = if pending {
                    // 未决期重复请求忽略（Runtime 已防重）；超时 = 轮询时机
                    matches!(cmd.recv_timeout(POLL), Err(mpsc::RecvTimeoutError::Disconnected))
                } else {
                    match cmd.recv() {
                        Ok(()) => {
                            if call_show(env, &api, h.activity) {
                                pending = true;
                            } else {
                                log("exit dialog show failed → cancel");
                                let _ = out.send(DialogResult::Cancel);
                            }
                            false
                        }
                        Err(_) => true,
                    }
                };
                if disconnected {
                    break;
                }
                if pending {
                    if let Some(r) = call_take(env, &api) {
                        pending = false;
                        let _ = out.send(r);
                    }
                }
            }
            if let Some(del) = (**env).DeleteGlobalRef {
                del(env, api.cls);
            }
        }
        if let Some(detach) = (**vm).DetachCurrentThread {
            detach(vm);
        }
    }

    /// 解析辅助类：activity.getClassLoader().loadClass("com.jjxy.game.ExitConfirm") + 静态方法 ID
    /// （局部引用经 Push/PopLocalFrame 一次释放，只留类的全局引用）。
    unsafe fn resolve(env: *mut JNIEnv, act: jobject) -> Option<Api> {
        let (Some(push), Some(pop)) = ((**env).PushLocalFrame, (**env).PopLocalFrame) else {
            return None;
        };
        if push(env, 16) != JNI_OK {
            exception(env);
            return None;
        }
        let api = resolve_in_frame(env, act);
        pop(env, std::ptr::null_mut());
        api
    }

    unsafe fn resolve_in_frame(env: *mut JNIEnv, act: jobject) -> Option<Api> {
        let loader = call_object(env, act, c"getClassLoader", c"()Ljava/lang/ClassLoader;", &[])?;
        let name = ((**env).NewStringUTF?)(env, c"com.jjxy.game.ExitConfirm".as_ptr());
        if name.is_null() || exception(env) {
            return None;
        }
        let sig = c"(Ljava/lang/String;)Ljava/lang/Class;";
        let cls = call_object(env, loader, c"loadClass", sig, &[jvalue { l: name }])?;
        let show = static_method(env, cls, c"show", c"(Landroid/app/Activity;)V")?;
        let take = static_method(env, cls, c"take", c"()I")?;
        let cls = global(env, cls)?;
        Some(Api { cls, show, take })
    }

    unsafe fn static_method(env: *mut JNIEnv, cls: jobject, name: &CStr, sig: &CStr) -> Option<jmethodID> {
        let mid = ((**env).GetStaticMethodID?)(env, cls, name.as_ptr(), sig.as_ptr());
        if mid.is_null() || exception(env) {
            None
        } else {
            Some(mid)
        }
    }

    /// ExitConfirm.show(activity)：成功投递到 UI 线程 = true。
    unsafe fn call_show(env: *mut JNIEnv, api: &Api, act: jobject) -> bool {
        let Some(call) = (**env).CallStaticVoidMethodA else { return false };
        call(env, api.cls, api.show, [jvalue { l: act }].as_ptr());
        !exception(env)
    }

    /// ExitConfirm.take()：Some = 有结果；调用异常 ⇒ 按取消（不卡暂停）。
    unsafe fn call_take(env: *mut JNIEnv, api: &Api) -> Option<DialogResult> {
        let Some(call) = (**env).CallStaticIntMethodA else { return Some(DialogResult::Cancel) };
        let code = call(env, api.cls, api.take, std::ptr::null());
        if exception(env) {
            return Some(DialogResult::Cancel);
        }
        result_from_code(code)
    }
}
