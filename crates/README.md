# crates/ — Rust workspace（P3 建立）

| crate | 职责 | 红线 |
|---|---|---|
| `game-data` | 资源解析（packed PNG / maplv / sprite / script DSL） | 纯解析无渲染；过 L1 golden test |
| `game-core` | 游戏逻辑（状态机/脚本解释器/战斗/道具） | 无 I/O、确定性（虚拟时钟 + GameRng）；禁引平台 crate |
| `game-platform` | 平台 trait（Clock/GameRng/Renderer/Audio/Storage/Input/Font） | 只放 trait 与类型 |
| `game-oracle` | 差分测试宿主 | 比对阈值不得为通过测试而放宽 |
| `game-desktop` | winit + pixels 实现 | 平台细节只在此层 |
| `game-wasm` | wasm-bindgen 实现 | 同上 |

扩展缝（D2）按 `docs/master-plan.md` §2 实现为 trait/常量，Stage A 就要过缝。
