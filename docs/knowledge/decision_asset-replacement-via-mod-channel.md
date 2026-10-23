---
name: asset-replacement-via-mod-channel
description: 换素材标准方法（2026-10-23 用户裁定）——改 JSON+中间态换 PNG+repack 回原格式，高清素材挂 mod 侧带 density 声明，禁直接覆盖保真面；素材高清暂缓，机制已就绪
type: decision
created: 2026-10-23
sources: [a284203, 2d27c47, 7e2cce3, 9247706, 592fe65]
---

## 裁定（2026-10-23 用户认可）

**素材高清暂缓（生产批未启动）；替换机制=OK，按本方法执行。**
"高清 mod"= 两份并存、开关切换：保真表+原素材（classic 金样基准，永不污染）∥
mod 表+HD 素材（id≥1000 或 mod: 前缀，density 声明）。直接替换原素材被否决
（污染金样帧哈希/oracle 对拍=自毁验证体系，回不去原版对照）。

## 替换方法（三步，零 Rust 零编译）

| 换什么 | 操作 | 工具/文件 |
|---|---|---|
| 像素/美术 | unpack 原容器 → 中间态换 PNG 帧 → repack 回原格式 | `crates/game-data/src/asset_pipeline/`（恒等门禁：未换帧逐字节不变） |
| 资源属性（尺寸/动画行/图集索引/密度/状态条件臂） | 直接编辑 JSON | `data/tables/{sprite_defs,anim_defs,sheet_index}.json`（V2 表驱动，220 逐值锚） |
| 高清素材 | 两者结合：换 HD PNG 帧 + 表声明 `density: N`（像素=逻辑×N，渲染层直采样） | density 通路已验（d=1 恒等/零漂移/perf≤3×） |

## 红线

1. **保真表（data/tables）= 原版事实**，table-content-hash 闸护航——改=勘误流程+金样重过；
   你的新素材/高清替换**挂 mod 侧**（金样运行禁 mod 表、mod 独立基线）。
2. **必须 repack 回原格式**：运行时吃原格式二进制（差分验证的基础）；PNG 直载
   （--hd-assets）只是探针旁路，量产不走。
3. **域外三界才改码**（登记在案）：新动画行超 89 索引/新图集容器超 16/战斗表扩列。
4. 验证链：`python3 gates/cli.py check`（12 闸含哈希/对照器）+ `trace`；换素材后
   classic 金样必须照绿（mod 面不进金样）。

## 状态

- 机制：✅ 全就绪（asset_pipeline 恒等 repack + V2 配置语义 + mod 骨架四规则 +
  零改码链演练 15/15 + density 渲染通路）
- 素材高清生产：⏸ 暂缓（美术面路线已调研：AI 初稿+画师收尾是商用公式；
  形状合规校验器待建，量产路径待拍板）
