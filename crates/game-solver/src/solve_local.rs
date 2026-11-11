//! 【求解器批2 C1】单层求解骨架 `solve_local`（提案 §5 C1；计划 :316-335）。
//!
//! **纯度边界**（`lib.rs` 契约 + 计划 :456 A9）：本模块=纯函数——输入 `SolveInput`
//! （由 bin/测试装配），输出 `SolveResult`；**禁文件/网络/时钟/进程 I/O**、禁 GameRng/Clock。
//! 搜索本体在 C2-C6 逐切片接入；C1 只立骨架与 JSON 出口（字段序固定=确定性要求，提案 §3.3）。
use crate::state::s0;

/// 单层求解输入（lib 纯数据；I/O 只在 `bin/solver.rs` 与测试侧）。
/// C1 只承载楼层与难度；夹具面（地形/实体/敌表/脚本）自 C2 起按切片增字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveInput {
    /// 楼层号（计划 :327 判据硬编码 `--floor 1`）
    pub floor: i32,
    /// 难度钉 0（提案 §4.2 BR-4；case67 分支门 `scene.rs:1032`）
    pub difficulty: i32,
}

impl SolveInput {
    /// C1 占位输入：只带 floor/difficulty，其余夹具字段待 C2+ 接入。
    pub fn placeholder(floor: i32) -> Self {
        SolveInput { floor, difficulty: 0 }
    }
}

/// 单层求解结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveResult {
    pub floor: i32,
    pub difficulty: i32,
    /// 入口状态（计划 R10 / 提案 §4.1：s₀=`state.rs s0()`=(hp300, atk10, def10) 注证据）
    pub s0: (i32, i32, i32, i32, i32, i32, i32), // hp, atk, def, yellow, blue, red, gold
    /// 任一出口可达（Exit#1 步行楼梯 或 Exit#2 脚本换层，提案 §1.4）
    pub reachable: bool,
    /// ① max_hp：全部合法路上的最大终态 hp（含 Exit#2；并列取步数少者→字典序最小）
    pub max_hp: Option<i32>,
    /// ② min_kills：**量化域=Exit#1**（提案 §1.4 ② / §8 勘误注 9）
    pub min_kills: Option<i32>,
    /// 三态置信（计划 :448 A1）：UseItem 裁出 ⇒ 批2 恒 `bounded-best`（BR-7）
    pub confidence: &'static str,
    /// 路线（route_gen.py 航点语法，`route_gen.py:12-15`）
    pub route: Vec<String>,
}

impl SolveResult {
    /// JSON 序列化（手写，字段序固定；lib 无 serde 依赖）。
    /// 字段序：floor, difficulty, s0, reachable, max_hp, min_kills, confidence, route。
    pub fn to_json(&self) -> String {
        let (hp, atk, def, y, b, r, g) = self.s0;
        let s0 = format!(
            "{{\"hp\":{hp},\"atk\":{atk},\"def\":{def},\"yellow\":{y},\"blue\":{b},\"red\":{r},\"gold\":{g}}}"
        );
        let route = self
            .route
            .iter()
            .map(|s| format!("\"{}\"", esc(s)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"floor\":{},\"difficulty\":{},\"s0\":{},\"reachable\":{},\"max_hp\":{},\"min_kills\":{},\"confidence\":\"{}\",\"route\":[{}]}}",
            self.floor,
            self.difficulty,
            s0,
            self.reachable,
            opt(self.max_hp),
            opt(self.min_kills),
            esc(self.confidence),
            route
        )
    }
}

fn opt(v: Option<i32>) -> String {
    match v {
        Some(x) => x.to_string(),
        None => "null".to_string(),
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 单层求解入口（批2 MVP）。
///
/// **C1 阶段=占位**：搜索骨架未接入（C2 可达性/战斗 → C3 双查询 → C4 开门 → C5 脚本 →
/// C6 floor1 端到端），恒 `reachable=false, max_hp=None, min_kills=None`。
/// 真实语义见提案 §1 形式化与 §5 切片表。
pub fn solve_local(input: &SolveInput) -> SolveResult {
    let (hp, atk, def, floor) = s0();
    let _ = floor; // s0 的 floor 字段与求解楼层不同号（真源 a.java:3482-3491），输出只注 hp/atk/def
    SolveResult {
        floor: input.floor,
        difficulty: input.difficulty,
        s0: (hp, atk, def, 0, 0, 0, 0),
        reachable: false,
        max_hp: None,
        min_kills: None,
        confidence: "feasible",
        route: Vec::new(),
    }
}
