---
name: global-replace-hits-newly-added-method
description: 机械全局替换会命中"刚插入的方法体"——同批先插后换，新方法的委托调用被换成自递归，尾位置优化成静默死循环（100% CPU 永不结束）
type: gotcha
created: 2026-10-13
sources: [迭代56 scene.rs entity_category（先加方法再全局换 entity::render_category(t)→self.entity_category(t)，方法体自身被换，t544 用例 48 分钟 100% CPU）]
---

"插入新方法 A（内部调用旧函数 B）→ 全局把 B 替换为 A" 这两步若在同一批
脚本里**先插后换**，A 自己方法体里的 B 也被换成 A ⇒ 无限自递归。若递归调用
处于尾位置，rustc TCO 优化成跳转 ⇒ **不栈溢出、不 panic，纯 100% CPU 死循环**，
测试-binary 级静默挂死（本例 48 分钟仍 99.9% CPU，State=S 的采样假象——
实际是单线程计算线程在跑，主线程在等）。

**防御：** 全局替换前先数命中数并**核对每一处上下文**（`grep -n` 逐处看）；
替换脚本的锚点必须排除刚插入的方法体（如按行号区间或先替换再加方法）。
怀疑死循环时：`ps -eo pid,etime,time,pcpu` 看 CPU time≈etime 且 pcpu≈100
即计算型死循环（非阻塞）；gdb attach 受 yama 限制时直接 `kill` 后
`timeout N cargo test <name> -- --nocapture` 二分定位挂死用例。
