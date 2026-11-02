---
name: adb-install-signature-and-step-verify
description: adb 静默装机失败坑（签名不匹配只显示 Streamed Install）——装机必查 EXIT+lastUpdateTime；导航探针每步验证制；2026-11-02 三机验收实锤
type: gotcha
created: 2026-11-02
sources: [c88a575, 9c698e1]
---

2026-11-02 三机验收实锤的两个设备侧坑：

1. **签名不匹配静默装机失败**：`adb install -r` 输出只显示 `Performing Streamed Install`
   就断掉（无 Success 行）=**可能失败**。`INSTALL_FAILED_UPDATE_INCOMPATIBLE`（别的
   构建会话的 debug keystore 装过同包名）时安装**根本没生效**——设备跑旧码而冒烟结论
   却记在新码头上（dash 实测：跑着 11-01 旧包当今日版验收了半小时）。**装机纪律**：
   ①`echo EXIT=$?` 捕获返回码；②`dumpsys package <id> | grep lastUpdateTime` 核对时间；
   ③签名冲突=`adb uninstall` 后干净装（丢存档，测试机可接受）。
2. **导航探针每步验证制**：注入点击/滑动的坐标随分辨率/游戏状态（boot logo 期点击全被
   吞、轮播栏位置随布局算）——盲点三连空是常态。**先截屏确认当前状态再操作下一步**
   （每步一截图一轮廓），比连发坐标高效得多。
