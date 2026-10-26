---
name: sprite-sheet-clip-blit-cost
description: J2ME 精灵表取格=setClip 小窗+整张表 drawImage；软渲染若全表逐点 set_pixel 再裁剪，97% 时间耗在被裁像素——门禁 10min 的真因；裁剪窗快路径+SHA-NI 后 26s
type: gotcha
created: 2026-10-26
sources: ["84e5cda 门禁提速（gates check ≈10min→26s）", "op() 计时探针：drawImage 3.36s/drawImageT 1.32s 占 preset_floor2 绘制 4.71s"]
---

原版取精灵/图块的惯用法是 `setClip(格) + drawImage(整张精灵表)`。SoftGraphics 旧 blit/transformed
遍历**整张源图**逐点 `set_pixel`，靠 clip 判定丢弃 ⇒ 每帧 ~7.8ms，金样回放（preset_cases/
fixture_frames，数万帧）占门禁 95% 时间。修法：只遍历与 clip 相交的源格/目的格（`clip_cells`）。

**为什么：** 门禁慢的根因不在门禁脚本（非 cargo 闸 11 项合计 <0.4s），而在被测代码的渲染热路径；
被裁像素 set_pixel 本就拒写、变换为双射 ⇒ 跳过它们逐字节等价，金样不受影响。第二热点是逐帧
SHA-256（307KB/帧）——x86 SHA-NI 路径运行时分派，可移植实现留作参照与非 x86 回退。

**何时使用：** 门禁/测试再变慢时先分层计时（闸逐项 → 测试二进制 → 单用例 tick/paint/hash 分段 →
op 类型分摊；无 perf 权限用临时插桩+还原）。改渲染热路径必须带"旧算法逐字复制作参照"的随机等价钉
（`clip_window_equivalence`）并做变异实验确认钉有牙；改哈希必须保 FIPS 向量 + 加速/可移植等价钉。
