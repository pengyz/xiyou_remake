---
name: android-key-feedback-jni-thread-and-switches
description: NativeActivity 壳做按键反馈的坑——View.playSoundEffect 须 UI 线程且受系统"触摸提示音"（实机关）⇒ 改 AudioManager.playSoundEffect(FX_KEY_CLICK,-1f)；performHapticFeedback 无线程校验、无需权限；听不见摸不着也能用 dumpsys/logcat 核对
type: gotcha
created: 2026-10-26
sources: ["f766313 feat(android): 虚拟键按下反馈", "AOSP android-16.0.0_r1：View/ViewRootImpl/AudioManager/AudioService/VibratorManagerService 源码"]
---

纯 native 线程（android_main / 自建 JNI 线程）里，按键音不能调 `View.playSoundEffect`：
`ViewRootImpl.playSoundEffect`（ViewRootImpl.java:9830）先 `checkThread()`（非 UI 线程抛
CalledFromWrongThreadException），且最终走 `AudioManager.playSoundEffect(int)`，服务端
（AudioService.java:7034）查系统"触摸提示音" `SOUND_EFFECTS_ENABLED`——Xiaomi 14 实机为 0 ⇒ 必静音。
可用的是 `AudioManager.playSoundEffect(FX_KEY_CLICK, -1f)`：服务端 `playSoundEffectVolume`（:7049）
只查 STREAM_SYSTEM 是否静音。触感 `View.performHapticFeedback(VIRTUAL_KEY)` 全链（View.java:28912 →
ViewRootImpl.java:9882 → Vibrator）无 `checkThread`，服务端不查 VIBRATE 权限，可从任意已 attach 线程调。

**为什么：** 两个 View 方法看着对称，线程约束与系统开关却不同；照"系统按钮同款"直觉选
View.playSoundEffect 会在 debug APK 崩（或在 UI 线程上调也静音，因为小米默认关触摸提示音）。
CheckJNI（debug APK 默认开）下带挂起异常再调 JNI 直接 abort——每次调用后必须 ExceptionCheck/Clear。

**何时使用：** Android 壳加任何 Java 侧反馈/系统服务调用（震动、音效、Toast 类）。实现见
`shell/android/src/feedback.rs`（worker 线程 attach 一次、缓存全局引用与方法 ID）。实机核对
不靠耳朵：`adb logcat -v time | grep "trigger keyboard vibrate for <包名>"`（HyperOS 触感日志，
effectId=1 即 VIRTUAL_KEY）、`adb shell dumpsys vibrator_manager`（Recent vibrations 含包名/usage=TOUCH）、
`adb shell dumpsys audio | grep "piid:<系统 SoundPool> event:started"`（系统 SoundPool = uid 1000、
USAGE_ASSISTANCE_SONIFICATION）。核对时以**自己注入点按的时刻**对齐，用户同时段的触控不算验证。
