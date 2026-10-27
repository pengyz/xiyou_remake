//! 虚拟键按下反馈：触感 + 按键音（2026-10-26 用户："虚拟按键按的时候触发震动和按键声音，
//! 现在操作无反馈，体验不好"）。**纯壳层**：键码照常进 game-core，core 不知情。
//!
//! # 策略
//!
//! | 项 | 口径 |
//! |---|---|
//! | 触发 | 触控翻译器（TouchMapper：竖屏控制面板 / 横屏画布键区）产出的每个 `Press`——含十字键拖动换向；按住不重复；死区、视口点按/滑动、硬件键不触发（硬件键自带反馈） |
//! | 合并 | 同一输入事件内多个 `Press` 合并为一次反馈 |
//! | 时机 | 输入回调内立即派发（不等 75ms 逻辑拍）；JNI 在专用工作线程，主循环零阻塞 |
//! | 触感 | `View.performHapticFeedback(VIRTUAL_KEY)`：系统"触感反馈"设置由系统侧裁决；无需 VIBRATE 权限 |
//! | 按键音 | `AudioManager.playSoundEffect(FX_KEY_CLICK, -1)`：随**游戏内"声音"开关**（与 BGM 同一开关）；不受系统"触摸提示音"约束；系统音量静音时仍静 |
//!
//! # 证据（A 级：AOSP `android-16.0.0_r1` 源码 + 实机只读查询；Xiaomi 14 = Android 16 / API 36）
//!
//! - 触感可在非 UI 线程调用：`View.performHapticFeedback`（View.java:28912）→
//!   `ViewRootImpl.performHapticFeedback`（ViewRootImpl.java:9882）→ `Vibrator.performHapticFeedback`，
//!   全程无 `checkThread()`；服务端 `VibratorManagerService.performHapticFeedbackInternal`（:529）
//!   不查 VIBRATE 权限，受限常量仅 BIOMETRIC_*（HapticFeedbackVibrationProvider.java:201）。
//! - 按键音不能走 `View.playSoundEffect`：`ViewRootImpl.playSoundEffect`（ViewRootImpl.java:9830）先
//!   `checkThread()`（须 UI 线程），再调 `AudioManager.playSoundEffect(int)`——服务端
//!   `AudioService.playSoundEffect(int, int)`（AudioService.java:7034）查系统"触摸提示音"
//!   `SOUND_EFFECTS_ENABLED`，实机该项为 **0**（`adb shell settings get system sound_effects_enabled`）
//!   ⇒ 必静音。`AudioManager.playSoundEffect(int, float)`（AudioManager.java:4186）→
//!   `playSoundEffectVolume`（AudioService.java:7049）只查 `isStreamMute(STREAM_SYSTEM)`；
//!   音量 <0 ⇒ 系统默认衰减（SoundEffectsHelper.onPlaySoundEffect）。
//! - 常量：`HapticFeedbackConstants.VIRTUAL_KEY = 1`（HapticFeedbackConstants.java:62）、
//!   `AudioManager.FX_KEY_CLICK = 0`（AudioManager.java:3938）。实机 `haptic_feedback_enabled` = 1。

use crate::input::KeyAction;

/// `HapticFeedbackConstants.VIRTUAL_KEY`（屏上虚拟键按下）。
pub const HAPTIC_VIRTUAL_KEY: i32 = 1;
/// `AudioManager.FX_KEY_CLICK`（系统按键音）。
pub const FX_KEY_CLICK: i32 = 0;
/// `playSoundEffect(int, float)` 音量：<0 ⇒ 系统默认音效衰减（与系统触摸音同响度）。
pub const CLICK_VOLUME_SYSTEM_DEFAULT: f32 = -1.0;

/// 一次按键反馈。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cue {
    /// 触感（系统"触感反馈"设置在系统侧裁决）。
    pub haptic: bool,
    /// 按键音（游戏内"声音"开关）。
    pub click: bool,
}

/// 键动作流中的按下数（`Release` 不计）。
pub fn presses_in(keys: &[KeyAction]) -> u32 {
    keys.iter().filter(|k| matches!(k, KeyAction::Press(_))).count() as u32
}

/// 策略：本输入事件的虚拟键按下数 → 至多一次反馈；触感恒发、按键音随游戏内"声音"。
pub fn cue_for(presses: u32, sound_enabled: bool) -> Option<Cue> {
    (presses > 0).then_some(Cue { haptic: true, click: sound_enabled })
}

/// 反馈出口（Android = JNI 工作线程 [`android::JniFeedback`]；宿主 = [`RecordingFeedback`]）。
pub trait FeedbackOut {
    fn play(&mut self, cue: Cue);
}

/// 宿主记录桩。
#[derive(Debug, Default)]
pub struct RecordingFeedback {
    pub cues: Vec<Cue>,
}

impl FeedbackOut for RecordingFeedback {
    fn play(&mut self, cue: Cue) {
        self.cues.push(cue);
    }
}

/// 派发：按策略得到反馈才交出口。
pub fn emit(out: &mut dyn FeedbackOut, presses: u32, sound_enabled: bool) {
    if let Some(cue) = cue_for(presses, sound_enabled) {
        out.play(cue);
    }
}

/// Android 出口：JNI 工作线程（attach 一次、启动时解析并缓存目标对象与方法 ID）。
///
/// CheckJNI（debug APK 默认开）纪律：每次调用后查异常并清除（带挂起异常再调 JNI = abort）；
/// 解析期局部引用经 Push/PopLocalFrame 一次释放；只保留两个全局引用，线程退出前删除并 detach。
#[cfg(target_os = "android")]
pub mod android {
    use super::{Cue, FeedbackOut, CLICK_VOLUME_SYSTEM_DEFAULT, FX_KEY_CLICK, HAPTIC_VIRTUAL_KEY};
    use jni_sys::{jmethodID, jobject, jvalue, JNIEnv, JavaVM, JNI_OK};
    use std::ffi::{c_void, CStr};
    use std::sync::mpsc;

    /// ANativeActivity 的 `vm` / `clazz`（框架持有的全局引用，活动存活期有效）——只读跨线程传递。
    struct Handles {
        vm: *mut JavaVM,
        activity: jobject,
    }
    unsafe impl Send for Handles {}

    /// 反馈出口：投递给 JNI 工作线程（输入回调零阻塞）。
    pub struct JniFeedback {
        tx: mpsc::Sender<Cue>,
    }

    impl FeedbackOut for JniFeedback {
        fn play(&mut self, cue: Cue) {
            let _ = self.tx.send(cue);
        }
    }

    impl JniFeedback {
        /// 起反馈线程。发送端随本值 drop ⇒ 线程收尾（删全局引用 + DetachCurrentThread）。
        ///
        /// # Safety
        /// `vm` / `activity` 须为 ANativeActivity 的 `vm` / `clazz`。
        pub unsafe fn spawn(vm: *mut c_void, activity: *mut c_void, log: fn(&str)) -> JniFeedback {
            let (tx, rx) = mpsc::channel::<Cue>();
            let h = Handles { vm: vm as *mut JavaVM, activity: activity as jobject };
            let spawned = std::thread::Builder::new()
                .name("jjxy-feedback".into())
                .spawn(move || unsafe { worker(h, rx, log) });
            if let Err(e) = spawned {
                log(&format!("feedback thread spawn err {e}"));
            }
            JniFeedback { tx }
        }
    }

    unsafe fn worker(h: Handles, rx: mpsc::Receiver<Cue>, log: fn(&str)) {
        let vm = h.vm;
        let Some(attach) = (**vm).AttachCurrentThread else { return };
        let mut env: *mut JNIEnv = std::ptr::null_mut();
        let rc = attach(vm, &mut env as *mut *mut JNIEnv as *mut *mut c_void, std::ptr::null_mut());
        if rc != JNI_OK || env.is_null() {
            log(&format!("feedback jni attach failed rc={rc}"));
            return;
        }
        let api = Api::resolve(env, h.activity);
        log(&format!("feedback jni init haptic={} click={}", api.haptic.is_some(), api.click.is_some()));
        let mut n = 0u32;
        while let Ok(cue) = rx.recv() {
            let r = api.play(env, cue);
            if n < 5 {
                log(&format!("feedback cue={cue:?} → haptic={:?} click={:?}", r.0, r.1));
            }
            n += 1;
        }
        api.release(env);
        if let Some(detach) = (**vm).DetachCurrentThread {
            detach(vm);
        }
    }

    /// 调用目标（全局引用 + 方法 ID）。
    struct Target {
        obj: jobject,
        mid: jmethodID,
    }

    struct Api {
        /// decorView.performHapticFeedback(I)Z
        haptic: Option<Target>,
        /// audioManager.playSoundEffect(IF)V
        click: Option<Target>,
    }

    impl Api {
        unsafe fn resolve(env: *mut JNIEnv, act: jobject) -> Api {
            let (Some(push), Some(pop)) = ((**env).PushLocalFrame, (**env).PopLocalFrame) else {
                return Api { haptic: None, click: None };
            };
            if push(env, 16) != JNI_OK {
                exception(env);
                return Api { haptic: None, click: None };
            }
            let api = Api { haptic: resolve_haptic(env, act), click: resolve_click(env, act) };
            pop(env, std::ptr::null_mut());
            api
        }

        /// 返回（触感结果, 按键音结果）：None = 未请求/不可用；Some(false) = 调用异常或系统拒绝。
        unsafe fn play(&self, env: *mut JNIEnv, cue: Cue) -> (Option<bool>, Option<bool>) {
            let haptic = match (&self.haptic, cue.haptic, (**env).CallBooleanMethodA) {
                (Some(t), true, Some(call)) => {
                    let r = call(env, t.obj, t.mid, [jvalue { i: HAPTIC_VIRTUAL_KEY }].as_ptr());
                    Some(!exception(env) && r != 0)
                }
                _ => None,
            };
            let click = match (&self.click, cue.click, (**env).CallVoidMethodA) {
                (Some(t), true, Some(call)) => {
                    let args = [jvalue { i: FX_KEY_CLICK }, jvalue { f: CLICK_VOLUME_SYSTEM_DEFAULT }];
                    call(env, t.obj, t.mid, args.as_ptr());
                    Some(!exception(env))
                }
                _ => None,
            };
            (haptic, click)
        }

        unsafe fn release(self, env: *mut JNIEnv) {
            if let Some(del) = (**env).DeleteGlobalRef {
                for t in [self.haptic, self.click].into_iter().flatten() {
                    del(env, t.obj);
                }
            }
        }
    }

    /// decorView = activity.getWindow().getDecorView()；方法 performHapticFeedback(int)。
    unsafe fn resolve_haptic(env: *mut JNIEnv, act: jobject) -> Option<Target> {
        let window = call_object(env, act, c"getWindow", c"()Landroid/view/Window;", &[])?;
        let decor = call_object(env, window, c"getDecorView", c"()Landroid/view/View;", &[])?;
        let mid = method(env, decor, c"performHapticFeedback", c"(I)Z")?;
        global(env, decor).map(|obj| Target { obj, mid })
    }

    /// audioManager = activity.getSystemService("audio")；方法 playSoundEffect(int, float)。
    unsafe fn resolve_click(env: *mut JNIEnv, act: jobject) -> Option<Target> {
        let name = ((**env).NewStringUTF?)(env, c"audio".as_ptr());
        if name.is_null() || exception(env) {
            return None;
        }
        let sig = c"(Ljava/lang/String;)Ljava/lang/Object;";
        let am = call_object(env, act, c"getSystemService", sig, &[jvalue { l: name }])?;
        let mid = method(env, am, c"playSoundEffect", c"(IF)V")?;
        global(env, am).map(|obj| Target { obj, mid })
    }

    /// obj 的实例方法 ID（GetObjectClass + GetMethodID；失败清异常返回 None）。
    /// `pub(crate)`：exit_dialog.rs（STAGE-B-005 系统对话框）同口径复用。
    pub(crate) unsafe fn method(env: *mut JNIEnv, obj: jobject, name: &CStr, sig: &CStr) -> Option<jmethodID> {
        let cls = ((**env).GetObjectClass?)(env, obj);
        if cls.is_null() || exception(env) {
            return None;
        }
        let mid = ((**env).GetMethodID?)(env, cls, name.as_ptr(), sig.as_ptr());
        if mid.is_null() || exception(env) {
            None
        } else {
            Some(mid)
        }
    }

    /// 调对象返回值实例方法（返回局部引用；null 或异常 ⇒ None）。
    pub(crate) unsafe fn call_object(env: *mut JNIEnv, obj: jobject, name: &CStr, sig: &CStr, args: &[jvalue]) -> Option<jobject> {
        let mid = method(env, obj, name, sig)?;
        let r = ((**env).CallObjectMethodA?)(env, obj, mid, args.as_ptr());
        if r.is_null() || exception(env) {
            None
        } else {
            Some(r)
        }
    }

    pub(crate) unsafe fn global(env: *mut JNIEnv, obj: jobject) -> Option<jobject> {
        let g = ((**env).NewGlobalRef?)(env, obj);
        (!g.is_null()).then_some(g)
    }

    /// 有挂起异常 ⇒ 打印到 logcat 并清除，返回 true。
    pub(crate) unsafe fn exception(env: *mut JNIEnv) -> bool {
        let (Some(check), Some(describe), Some(clear)) =
            ((**env).ExceptionCheck, (**env).ExceptionDescribe, (**env).ExceptionClear)
        else {
            return false;
        };
        if check(env) != 0 {
            describe(env);
            clear(env);
            true
        } else {
            false
        }
    }
}
