//! 规则层：以纯函数 / trait 形式表达的**游戏规则**，引擎与求解器共用同一实现。
//!
//! 规矩（见 `docs/findings/2026-11-17-solver-map-vs-mechanism.md`）：机制常量与判定逻辑住在这里
//! （game-core）；图/题面差异走输入；求解器只允许消费，不得另写一份。

pub mod boss;
pub mod gate;
pub mod sensitivity;
pub mod shop;
