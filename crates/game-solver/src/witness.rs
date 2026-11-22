//! P4 witness builder：把 P0 层图 + P3 出层接口（[`solve_local_exit`]）串成一条
//! **可回放**的通关见证（`run-witness.json`）。
//!
//! # 为什么要这个模块
//!
//! `solve_global`（批3-6）是**搜索**：它在全局状态空间里做 BFS，能回答「有没有解 / 覆盖多完整」，
//! 但它吐出来的是一条**结论**（`RunStatus` + `GlobalGoal`），不是一条**逐步可核对的路线**。
//! 本模块是它的对偶面：**不做搜索**，按塔序一层一层往前推，把每层的
//! `(入场格, 入场态) → (出层格, 出层态, 出层路线)` 原样钉下来——每步都能被独立思考复核。
//!
//! 命名：`floorgraph.rs` 的模块 doc 早就写着「可行性由 `run_witness` 拿着这张表去搜」，
//! 故模块名 `witness`、入口函数名 [`run_witness`]（对上那处既有引用，不留悬空名字）。
//!
//! # 硬约束（比「跑通」重要）
//!
//! **绝不许造 witness**：任一层 `ok == false` ⇒ `status` 必须是 `blocked`（截断则是 `capped`），
//! `floors` **只保留成功前缀**，`final.ending_done = false`。宁可报「卡在第 N 层」，
//! 不许把半成品包装成通关。这条是判据 `witness_refuses_to_fake_success` 与
//! `witness_truncated_floor_is_not_consumed` 的对象。
//!
//! # 只信 `ok`（P3 复核 R2 的教训）
//!
//! 旧入口 `solve_local()` 的 `max_hp/route` **不带截断语义**（P3 收口1 之后才补 `truncated`）。
//! 本模块一律走 [`solve_local_exit`]，只认它的 `ok`/`truncated` 二元 ——
//! 截断层的 `out_*` 只作诊断记录，**绝不**拿来接力（见 [`LayerState`] 注释）。
//!
//! # 状态接力
//!
//! 跨层携带的只有两样：`out_player`（玩家全态）、`out_flags`（脚本旗标）。
//! **实体表按层归属**（`out_ents` 是**本层**的活体表，不是下一层的入场表）——
//! 与 `solve_global` 的 `st.ents: BTreeMap<floor, Vec<EntRec>>` 同模型：
//! 新层首入时位图里放的是**该层** `sprite_full` 全表（不是上一层的残留），本层终态记进
//! `WitnessFloor.ents_out` 供回放。把上一层实体表硬塞进下一层是**语义错误**
//! （下一层会以为自己有上一层的怪），故本模块不做；`witness_chains_real_floors` 用
//! 「f2 的 `ents_in` 必须逐条 == `sprite_full(f2)`，且 ≠ f1 的 `ents_out`」钉死这条。
//!
//! # 有限回溯：**故意没做**（写进文档，不是漏了）
//!
//! `solve_local_exit` 每次只回**一条**规范层目标（HP 降序→步数升序→路线字典序，
//! 见 `solve_local::HpLoss`），备用出口/备用目标**不在这个接口面上**。要在本模块做
//! 「换一条出层路重试」，只能绕开 `solve_local_exit` 直接调 `solve_layer_with` 并自行
//! 复刻目标选择契约 —— 那会让 P3 定死的「出层成败契约」出现第二份实现（漂移风险）。
//! 所以本模块的回溯上限是**显式的 0 次**，换成两道确定性护栏：
//!
//! - `max_floors`（默认 [`DEFAULT_MAX_FLOORS`]）：层数上限，防无限推进；
//! - 环检测：同一层在一次见证里被推进第二次 ⇒ `blocked / cycle`。
//!
//! 封顶或环都被如实报成 `blocked`（带 `reason`），不会被当成通关。
//!
//! # 确定性
//!
//! 无 `HashMap` 遍历、无时钟、无随机：同输入 ⇒ JSON 逐字节相同（`LAYER_MEMO` 是
//! thread_local 纯加速，缓存里连截断标记一起存/回放，不改变输出）。
//!
//! # schema 版本（v1 → v2，2026-11-19 / 复核 F7）
//!
//! **v1 的缺口**：`WitnessFloor` 只有 `ok/truncated` 两个成败位，`ExitSolve::pure_pickup`
//! （P5 闭式解采用标记）在 `record_floor` 里被**丢弃** ⇒ `solver run` 的 JSON 下游
//! 无法区分「闭式命中」与「通用搜索」。f54（闭式，`+2400`）与 f1（通用搜索）在 v1
//! JSON 里长得一样，P5 回放与外部审计只能靠猜。
//!
//! **v2 新增（只追加，既有字段序与语义一律不动）**：
//!
//! | 字段 | 类型 | 语义 |
//! |---|---|---|
//! | `pure_pickup` | `bool` | 本层出层解是否来自闭式（`ExitSolve::pure_pickup` 原值，不重算） |
//! | `pure_pickup_reason` | `string \| null` | 命中=`"adopted"`；回退=具名拒绝原因（`reason/ty`，如 `monster_ents/41`）；`null`=本层没问过闭式 |
//!
//! 判据：`witness_floor_records_pure_pickup_flag`（真数据 f54/f1）+
//! `witness_json_has_no_unknown_schema_drift`（标签与字段序）。

use crate::solve_global::{landing, stair_target};
use crate::solve_local::{
    solve_local_exit_toward_shops, EntRec, ExitKind, ExitSolve, LevelSpec, PlayerSpec, SolveInput,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// witness JSON 的 schema 标签（消费者按它判版本；字段语义变更必须换标签）。
///
/// v1 → v2（2026-11-19，复核 F7）：`WitnessFloor` 追加 `pure_pickup` /
/// `pure_pickup_reason` 两字段（闭式解可观测面）。既有字段与顺序零改动，
/// 但「新增字段」就是契约变更 ⇒ 必须换标签（禁静默改契约）。
///
/// v2 → v3（2026-11-20，P10）：`WitnessFloor` 追加 `demand` / `demand_undefined` /
/// `keys_out` / `key_shortfall` 四个**诊断**字段（钥匙需求观测面；不参与比较）。
pub const WITNESS_SCHEMA: &str = "xiyou/witness/v3";

/// 一次见证的层数上限（防无限推进；塔序真实长度 ~54 ⇒ 64 足够且仍是硬护栏）。
pub const DEFAULT_MAX_FLOORS: usize = 64;

/// 结论状态（词表与 `solve_global::RunStatus::as_str` 逐字对齐，见
/// `witness_status_vocabulary_matches_run_status`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessStatus {
    /// 从起点推到了终局（`kind=Ending` 或 f50 终局脚本）。
    Witness,
    /// 某层 `ok=false`（或模型缺口/护栏触界）：**如实卡住**。
    Blocked,
    /// 某层 `truncated=true`：该层解不可信，不许接力。
    Capped,
    /// 目标层缺规格（`per_floor` 缺口）⇒ 模型不完整，结论不可用。
    ModelBroken,
}

impl WitnessStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            WitnessStatus::Witness => "witness",
            WitnessStatus::Blocked => "blocked",
            WitnessStatus::Capped => "capped",
            WitnessStatus::ModelBroken => "model_broken",
        }
    }
}

/// 玩家态在 witness 里的投影（显式字段，不复用 `PlayerSpec` 的 serde derive：
/// `solve_local` 的 `to_json` 是手写零 serde 面，witness 自带一套不互相牵连）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WPlayer {
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub yellow: i32,
    pub blue: i32,
    pub red: i32,
    pub gold: i32,
    pub item_types: Vec<u8>,
    pub item_uses: Vec<u8>,
    pub item_stack_size: i32,
}

impl From<&PlayerSpec> for WPlayer {
    fn from(p: &PlayerSpec) -> Self {
        WPlayer {
            hp: p.hp,
            atk: p.atk,
            def: p.def,
            yellow: p.yellow,
            blue: p.blue,
            red: p.red,
            gold: p.gold,
            item_types: p.item_types.clone(),
            item_uses: p.item_uses.clone(),
            item_stack_size: p.item_stack_size,
        }
    }
}

/// 活体实体记录在 witness 里的投影（`[ty,x,y,param]` 语义同 `EntRec`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WEnt {
    pub ty: i32,
    pub x: i32,
    pub y: i32,
    pub param: i32,
}

pub(crate) fn wents(v: &[EntRec]) -> Vec<WEnt> {
    v.iter()
        .map(|e| WEnt { ty: e.ty, x: e.x, y: e.y, param: e.param })
        .collect()
}

pub(crate) fn kind_str(k: ExitKind) -> &'static str {
    match k {
        ExitKind::Stair => "Stair",
        ExitKind::Script => "Script",
        ExitKind::Ending => "Ending",
        ExitKind::Cloud => "Cloud",
        ExitKind::UseItem => "UseItem",
    }
}

/// 成功推进的一层。
///
/// 不变式（判据 `witness_json_schema_stable` 逐条钉）：`ok == true`、`truncated == false`、
/// `in_player/out_player` 齐全——`floors` 里**只有**可消费的层。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessFloor {
    pub floor: i32,
    /// 入场格。
    pub entry: [i32; 2],
    /// 出层格（`Ending` 层可能为 `null`：终局不以「走到某格」为界）。
    pub exit: Option<[i32; 2]>,
    /// `Stair|Script|Ending|Cloud|UseItem`（`ExitKind` 词表原样）。
    pub kind: String,
    /// 本层走位航点（`solve_local_exit.route` 原样：`"x,y"` / `"door x,y"` / `"bump 41@x,y"` …）。
    pub route: Vec<String>,
    /// 入场血量（= 上一层 `out_hp`；首层 = 新游戏 300）。
    pub in_hp: i32,
    /// 出层血量（`ok=true` 时即真实可达的最大血）。
    pub out_hp: i32,
    pub in_player: WPlayer,
    pub out_player: WPlayer,
    /// 入场脚本旗标（`u128` 位图，`0x` 十六进制字符串——JSON 数字装不下 128 位）。
    pub in_flags: String,
    pub out_flags: String,
    /// 入场活体表：`None` = 该层从未被推进过（求解侧展开成 `sprite_full`）；
    /// `Some` = 位图里记的该层全表/终态（**按层归属**，绝不是上一层的残留）。
    pub ents_in: Option<Vec<WEnt>>,
    /// 本层终态活体表（`ok=true` 才写；供回放/复访）。
    pub ents_out: Vec<WEnt>,
    pub ok: bool,
    pub truncated: bool,
    /// 【v2】本层出层解是否来自 P5 纯拾取闭式（`ExitSolve::pure_pickup` 原值）。
    /// `true` ⇒ `out_hp` 是「入场 hp + Σ 认证单调拾取增量」的闭式上界且路线已引擎回放。
    pub pure_pickup: bool,
    /// 【v2】闭式命中/回退的具名标签（`PurePickupStat::reason_label`）：
    /// 命中=`"adopted"`；回退=拒绝原因（带触发实体型）；`null`=本层没问过闭式。
    pub pure_pickup_reason: Option<String>,
    /// 【v3 诊断】本层的后继钥匙需求 `objective::next_demand`（(y,b,r) 逐色最大）。
    /// **纯诊断字段**：绝不参与出层比较/可行性（objective.rs 模块 doc 契约——钥匙是
    /// 跨层动态资源，单层硬约束会制造假不可行）。
    pub demand: [u8; 3],
    /// 【v3 诊断】需求未定义原因（后继缺规格/后继 keydemand 非 Ok）；`null`=Known。
    pub demand_undefined: Option<String>,
    /// 【v3 诊断】本层出层时携带钥匙 (y,b,r)（`out_player` 的钥匙字段）。
    pub keys_out: [u8; 3],
    /// 【v3 诊断】`keys_out` 逐色小于 `demand` 的缺口。**不阻断、不改 ok**——
    /// 只如实落账供审计与下一片「链路层回拾/补钥」使用。
    pub key_shortfall: Option<KeyShortfall>,
}

/// 【v3 诊断】钥匙缺口（逐色）：`need` 来自 `demand`，`have` 来自 `keys_out`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyShortfall {
    pub need: [u8; 3],
    pub have: [u8; 3],
}

/// 终局块。`ending_done` **只有**真的推进到 `Ending` 才为 true。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessFinal {
    pub ending_done: bool,
    /// 走到哪一层（blocked 时=卡住那层，witness 时=终局层）。
    pub floor: i32,
}

/// 阻断诊断：卡在哪一层、为什么、该层 `ok/truncated` 各是什么（P4 汇报必须项）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessBlocked {
    pub floor: i32,
    /// `no_exit` / `truncated` / `model_missing` / `stair_edge_missing` /
    /// `landing_missing` / `cycle` / `max_floors` / `cloud_not_supported`。
    pub reason: String,
    pub detail: String,
    /// 该层 `ExitSolve.ok` 原值（没调到求解的护栏则不适用 ⇒ `null`）。
    pub layer_ok: Option<bool>,
    /// 该层 `ExitSolve.truncated` 原值（同上）。
    pub layer_truncated: Option<bool>,
}

/// 元信息：让 witness 自解释（schema 版本 / 起点 / 层数 / 度量口径）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessMeta {
    pub schema: String,
    pub start_floor: i32,
    pub start_entry: [i32; 2],
    /// 真正求解过的层数（= `floors.len()`，单列便于消费方不用数数组）。
    pub layers: usize,
    /// `total_steps` 的**口径声明**（见 `total_steps` 字段注）。
    pub steps_metric: String,
}

/// 一份见证（`run-witness.json` 的根）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Witness {
    /// `witness|blocked|capped|model_broken`（= [`WitnessStatus`] 词表）。
    pub status: String,
    /// 成功前缀（**只有** `ok && !truncated` 的层）。
    pub floors: Vec<WitnessFloor>,
    #[serde(rename = "final")]
    pub final_state: WitnessFinal,
    pub blocked: Option<WitnessBlocked>,
    /// Σ 各层 `route.len()`。
    ///
    /// **口径诚实声明**：`ExitSolve` 不暴露引擎步数/击杀数（P3 接口面只有
    /// `route/out_*/exit/kind`），故这里数的是**航点数**（上限意义：一步位移 ≥ 一个航点，
    /// 加上 bump/kill 的无位移动作，真实 tick 步数 ≥ 本值）。不是伪造的 tick 数。
    pub total_steps: i32,
    pub meta: WitnessMeta,
}

impl Witness {
    /// 序列化成 JSON（`serde_json` 结构体字段序 = 声明序 ⇒ 逐字节确定）。
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Witness 序列化不会失败（全为普通字段）")
    }

    /// 序列化成缩进 JSON（`--out` 落盘用；同样确定）。
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("Witness 序列化不会失败（全为普通字段）")
    }
}

/// witness 构造配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitnessCfg {
    /// 层数上限（护栏；默认 [`DEFAULT_MAX_FLOORS`]）。
    pub max_floors: usize,
}

impl Default for WitnessCfg {
    fn default() -> Self {
        WitnessCfg { max_floors: DEFAULT_MAX_FLOORS }
    }
}

/// 单层求解器签名（[`solve_local_exit_toward`] 的同形状投影）——测试注入缝。
///
/// 末参 `Option<i32>` = 本步的**优先目标层**（P0 层图的上行主线后继）；
/// 返回值第二项 = 该优先目标是否被满足（`false` ⇒ 解是回落解，**不得**当前进消费）。
/// 【P9.1 更正（复核 F3/F8）】末组参数带**真实商品表**（`SolverInput.shops`，17 行）：
/// 旧签名的生产实现走 `solve_local_exit_toward`，而它内部把商品表硬编码成 `&[]`，
/// witness 链于是永远在「空商品表口径」下求层解——f3 的 `ty77 param=0x116` 商人格
/// 在真表口径下才判得出「引擎根本不弹购买面」。商品表必须由调用方一路传下来。
pub type LayerSolver<'a> = &'a dyn Fn(
    &SolveInput,
    (i32, i32),
    &PlayerSpec,
    u128,
    Option<&[EntRec]>,
    Option<i32>,
    &[game_data::defs::ShopGoodDef],
) -> (ExitSolve, bool);

/// 生产入口：按 P0 层图的**上行主线**逐层推进 [`solve_local_exit_toward`]，产出可回放见证。
///
/// - `levels`：层号 → 层规格（`solve_global` 的 `per_floor` 同源）；
/// - `start_floor`/`start_entry`：起手层与入场格（塔序起点= f51 / (1,11)）；
/// - `player`/`flags`：起手玩家态与脚本旗标（新游戏 = hp300/atk10/def10/三色钥0/金0/空包/flags0）；
/// - `shops`：**真实商品表**（`SolverInput.shops`）。不许传 `&[]`：空表会让「商店格能不能
///   买」失真（旧版正是这样把 f3 口径打偏的，见复核 F3/F8）；没有商品表的夹具显式传 `&[]`。
/// - 实体表按层归属：新层首入传 `None`（见模块 doc「状态接力」）。
///
/// 「上行主线」= 从当前层出发的 `StairUp` 边目标（见 [`up_line_next`]）：
/// 这就是「P0 层图 + P3 出层接口」的接法——层图给**去哪**，层求解器给**怎么去**。
pub fn run_witness(
    levels: &BTreeMap<i32, SolveInput>,
    start_floor: i32,
    start_entry: (i32, i32),
    player: PlayerSpec,
    flags: u128,
    shops: &[game_data::defs::ShopGoodDef],
    cfg: &WitnessCfg,
) -> Witness {
    run_witness_with(
        levels,
        start_floor,
        start_entry,
        player,
        flags,
        shops,
        cfg,
        &|i, e, p, f, ents, toward, sh| solve_local_exit_toward_shops(i, e, p, f, ents, toward, sh),
    )
}

/// P0 层图的「上行主线」后继：本层 `StairUp` 边的目标层。
///
/// 塔序实测（`solver floors`，P0 层图）：`f51→f52→f53→f54 --StairUp@(7,1)--> f1 → f2 → … → f50`，
/// 边全由 `floorgraph::edges_of` 从 `ty7` 实体 `param&0xff` 取，**不硬编码塔序**。
/// 同层多条 `StairUp`（当前数据里没有）⇒ 取 `(to, cell)` 最小者，保证确定性。
/// `None` = 本层没有上行边 ⇒ 主线终点（终局应在 f50 的 `Ending`）。
pub fn up_line_next(floor: i32, level: &LevelSpec, lays: &BTreeMap<i32, i32>) -> Option<i32> {
    let mut ups: Vec<(i32, i32, i32)> = crate::floorgraph::edges_of(floor, level, lays)
        .into_iter()
        .filter(|e| e.kind == crate::floorgraph::EdgeKind::StairUp)
        .map(|e| (e.to, e.cell.0, e.cell.1))
        .collect();
    ups.sort();
    ups.first().map(|(to, _, _)| *to)
}

/// 一次出层解对应的换层边（`Stair` 取出口梯 `param&0xff`，其余取 `cross_edge`）。
pub(crate) fn exit_edge(level: &LevelSpec, got: &ExitSolve) -> Option<(i32, bool)> {
    match (got.kind, got.exit) {
        (Some(ExitKind::Stair), Some(cell)) => stair_target(level, cell),
        (Some(ExitKind::Ending), _) | (None, _) => None,
        _ => got.cross_edge,
    }
}

/// 把一层成功解记进 `floors`（**唯一**落账入口：`Ending` 层也要落账——
/// 终局层同样是「解出来的成功层」，只是它没有下一层）。
pub(crate) fn record_floor(
    w: &mut Witness,
    floor: i32,
    entry: (i32, i32),
    got: &ExitSolve,
    cur_player: &PlayerSpec,
    cur_flags: u128,
    ents_here: Option<&[EntRec]>,
    demand: &crate::objective::Demand,
) {
    // ok=true ⇒ kind 必为 Some（`materialize_exit` 契约：只有 Some(g) 才回填目标态）
    let kind = got.kind.expect("ok=true ⇒ kind 必为 Some（solve_local_exit 契约）");
    w.total_steps += got.route.len() as i32;
    // 【v3 诊断】出层钥匙（PlayerSpec 钥匙字段；负值防御性钳 0）。
    let clamp = |v: i32| v.clamp(0, u8::MAX as i32) as u8;
    let keys_out = [clamp(got.out_player.yellow), clamp(got.out_player.blue), clamp(got.out_player.red)];
    // 【v3 诊断】缺口 = 逐色 have < need。**只落账，不阻断、不改 ok**（P10 用户裁定：
    // 钥匙可跨层回拾/商店补，单层缺口不是可行性结论）。
    let key_shortfall = match demand.state {
        crate::objective::DemandState::Known => {
            if keys_out.iter().zip(demand.keys.iter()).any(|(h, n)| h < n) {
                Some(KeyShortfall { need: demand.keys, have: keys_out })
            } else {
                None
            }
        }
        crate::objective::DemandState::Undefined(_) => None,
    };
    w.floors.push(WitnessFloor {
        floor,
        entry: [entry.0, entry.1],
        exit: got.exit.map(|(x, y)| [x, y]),
        kind: kind_str(kind).to_string(),
        route: got.route.clone(),
        in_hp: cur_player.hp,
        out_hp: got.out_hp,
        in_player: WPlayer::from(cur_player),
        out_player: WPlayer::from(&got.out_player),
        in_flags: format!("0x{cur_flags:x}"),
        out_flags: format!("0x{:x}", got.out_flags),
        ents_in: ents_here.map(wents),
        ents_out: wents(&got.out_ents),
        ok: got.ok,
        truncated: got.truncated,
        // F7：`ExitSolve` 携带的闭式判定**原样**落盘，不再丢弃（v1 就是丢在这里）。
        pure_pickup: got.pure_pickup,
        pure_pickup_reason: got.pure_pickup_reason.clone(),
        demand: demand.keys,
        demand_undefined: match &demand.state {
            crate::objective::DemandState::Known => None,
            crate::objective::DemandState::Undefined(r) => Some(r.clone()),
        },
        keys_out,
        key_shortfall,
    });
}

/// 同 [`run_witness`]，但允许注入层求解器（**测试缝**：注入「必失败/必截断」的壳层，
/// 用于证明「失败层绝不会被接力」而不必污染进程环境变量）。
pub fn run_witness_with(
    levels: &BTreeMap<i32, SolveInput>,
    start_floor: i32,
    start_entry: (i32, i32),
    player: PlayerSpec,
    flags: u128,
    shops: &[game_data::defs::ShopGoodDef],
    cfg: &WitnessCfg,
    layer: LayerSolver<'_>,
) -> Witness {
    let mut w = Witness {
        status: WitnessStatus::Blocked.as_str().to_string(),
        floors: Vec::new(),
        final_state: WitnessFinal { ending_done: false, floor: start_floor },
        blocked: None,
        total_steps: 0,
        meta: WitnessMeta {
            schema: WITNESS_SCHEMA.to_string(),
            start_floor,
            start_entry: [start_entry.0, start_entry.1],
            layers: 0,
            steps_metric: "sum(route.len()) over floors; ExitSolve 不暴露 tick 步数".to_string(),
        },
    };
    // 跨层携带面：玩家全态 + 脚本旗标 + **按层归属**的活体位图（与 solve_global 同模型）。
    let mut cur_player = player;
    let mut cur_flags = flags;
    let mut ents_by_floor: BTreeMap<i32, Vec<EntRec>> = BTreeMap::new();
    let mut visited: BTreeSet<i32> = BTreeSet::new();
    // P0 层图（脚本槽→LAY 目标）只解析一次；`up_line_next` 用它判 `Script` 边。
    let lays = crate::floorgraph::lay_targets();
    let mut floor = start_floor;
    let mut entry = start_entry;

    loop {
        // ---- 护栏 1：层数上限（不 backtrack、不无限推进）----
        if w.floors.len() >= cfg.max_floors {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "max_floors".to_string(),
                detail: format!("已推进 {} 层，触 max_floors={}", w.floors.len(), cfg.max_floors),
                layer_ok: None,
                layer_truncated: None,
            });
            break;
        }
        // ---- 护栏 2：环检测（同一层推进第二次=闭环/回环，不是通关）----
        if !visited.insert(floor) {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "cycle".to_string(),
                detail: format!("层 {floor} 在此见证里被推进第二次"),
                layer_ok: None,
                layer_truncated: None,
            });
            break;
        }
        // ---- 模型缺口：目标层没有层规格 ⇒ 结论不可用（不许猜）----
        let Some(input) = levels.get(&floor) else {
            w.status = WitnessStatus::ModelBroken.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "model_missing".to_string(),
                detail: format!("层 {floor} 不在 per_floor 规格表里"),
                layer_ok: None,
                layer_truncated: None,
            });
            break;
        };
        let Some(level) = input.level.as_ref() else {
            w.status = WitnessStatus::ModelBroken.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "model_missing".to_string(),
                detail: format!("层 {floor} 的规格是占位（level=None）"),
                layer_ok: None,
                layer_truncated: None,
            });
            break;
        };

        // 入场活体表：新层首入=None（solve_local 侧展开成该层 sprite 全表）；
        // 复访=上次离开本层时的终态位图。
        let ents_here = ents_by_floor.get(&floor).cloned();
        // P0 ⇒ P3 的接缝：层图说「上一层楼是 f{next_up}」，层求解器就在目标集里挑
        // 「出层边指向 f{next_up}」的那条；`hit=false` 表示它挑不到（回落解可能往回走）。
        let next_up = up_line_next(floor, level, &lays);
        let (got, hit) =
            layer(input, entry, &cur_player, cur_flags, ents_here.as_deref(), next_up, shops);

        // ---- 硬约束 A：截断层不可消费（也不许接力）----
        if got.truncated {
            w.status = WitnessStatus::Capped.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "truncated".to_string(),
                detail: format!(
                    "层 {floor} 搜索撞层界（truncated=true）；out_hp={} 只作参考，不可消费",
                    got.out_hp
                ),
                layer_ok: Some(got.ok),
                layer_truncated: Some(true),
            });
            break;
        }
        // ---- 硬约束 B：ok=false ⇒ 卡住，绝不包装成通关 ----
        if !got.ok {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "no_exit".to_string(),
                detail: format!(
                    "层 {floor} 从入场格 {entry:?} 无出层通道（reachable=false）；\
                     in_player hp={} gold={}",
                    cur_player.hp, cur_player.gold
                ),
                layer_ok: Some(false),
                layer_truncated: Some(false),
            });
            break;
        }

        // ok=true ⇒ kind 必为 Some（`materialize_exit` 契约：只有 Some(g) 才回填目标态）
        let kind = got.kind.expect("ok=true ⇒ kind 必为 Some");

        // 【v3 诊断】本层后继钥匙需求（一步前瞻；纯诊断面，绝不参与比较/可行性）。
        let demand = crate::objective::next_demand(levels, floor);

        // 终局：`Ending` 层=通关。终局层本身也是「解出来的成功层」，**照样落账**
        // （它的 kind=Ending、exit 可能为 null；`floors` 因此是完整的逐层路线）。
        if kind == ExitKind::Ending {
            record_floor(&mut w, floor, entry, &got, &cur_player, cur_flags, ents_here.as_deref(), &demand);
            w.status = WitnessStatus::Witness.as_str().to_string();
            w.final_state = WitnessFinal { ending_done: true, floor };
            break;
        }
        // 非终局层必须能接上「上行主线」：没有上行边 ⇒ 本层不是主线中继（如实报）。
        let Some(expected) = next_up else {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "no_up_line".to_string(),
                detail: format!(
                    "层 {floor} 在 P0 层图里没有 StairUp 边（非终局层），但层解 kind={} exit={:?}，\
                     无法接上塔序",
                    kind_str(kind),
                    got.exit
                ),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        };
        // 筋斗云：区间门（target∈[已访层 min,max]）——线性塔序没有可判区间，不消费。
        if kind == ExitKind::Cloud {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "cloud_not_supported".to_string(),
                detail: format!(
                    "层 {floor} 唯一可行出层是筋斗云（区间门）；本版本不消费云边（expected f{expected}）"
                ),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        }
        let edge = exit_edge(level, &got);
        // 硬约束 C：优先目标没命中 ⇒ 手上这条是**回落解**（可能往回走），绝不许当前进。
        if !hit {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "no_exit_toward".to_string(),
                detail: format!(
                    "层 {floor} 的目标集里没有「出层边指向 f{expected}」的解；\
                     规范回落解 kind={} exit={:?} edge={:?}",
                    kind_str(kind),
                    got.exit,
                    edge
                ),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        }
        // ---- 换层边：Stair=出口梯 param；Script/UseItem=换层两元组 ----
        let Some(exit_cell) = got.exit else {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "no_exit".to_string(),
                detail: format!("层 {floor} ok=true 但 exit=None（契约违例）"),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        };
        let Some((target, went_down)) = edge else {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "stair_edge_missing".to_string(),
                detail: format!(
                    "层 {floor} 出层格 {exit_cell:?} 找不到换层边（kind={}）；\
                     缺实体 param 或 cross_edge",
                    kind_str(kind)
                ),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        };
        // 不变式：`hit=true` ⇒ 边目标必等于层图给的上一层（否则是挑选/层图不一致，不许吞）。
        if target != expected {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "edge_mismatch".to_string(),
                detail: format!(
                    "层 {floor} 优先目标 f{expected} 命中，但出层边指向 f{target}（层图/挑选不一致）"
                ),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        }
        // ---- 成功层：出层边已核验通往 f{expected} 才落账（floors=成功前缀）----
        record_floor(&mut w, floor, entry, &got, &cur_player, cur_flags, ents_here.as_deref(), &demand);

        // 接力：本层终态入位图（按层归属），玩家/旗标带入下一层。
        ents_by_floor.insert(floor, got.out_ents.clone());
        let next_player = got.out_player.clone();
        let next_flags = got.out_flags;
        // 落点（同一份 `landing` 语义：与 solve_global 单源）
        let Some(tinput) = levels.get(&target) else {
            w.status = WitnessStatus::ModelBroken.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "model_missing".to_string(),
                detail: format!("层 {floor} 的出层边指向层 {target}，但该层无规格"),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        };
        let Some(tlevel) = tinput.level.as_ref() else {
            w.status = WitnessStatus::ModelBroken.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor,
                reason: "model_missing".to_string(),
                detail: format!("层 {target} 的规格是占位（level=None）"),
                layer_ok: Some(true),
                layer_truncated: Some(false),
            });
            break;
        };
        let Some(lpos) = landing(tlevel, target, went_down) else {
            w.status = WitnessStatus::Blocked.as_str().to_string();
            w.blocked = Some(WitnessBlocked {
                floor: target,
                reason: "landing_missing".to_string(),
                detail: format!(
                    "从层 {floor} 落到层 {target}（went_down={went_down}）找不到落点梯"
                ),
                layer_ok: None,
                layer_truncated: None,
            });
            break;
        };

        // 新层首入：把该层 sprite 全表塞进位图（= 传 None 的显式等价，且供复访一致）。
        if !ents_by_floor.contains_key(&target) {
            ents_by_floor.insert(target, crate::solve_local::sprite_full(tlevel));
        }
        cur_player = next_player;
        cur_flags = next_flags;
        floor = target;
        entry = lpos;
    }

    w.meta.layers = w.floors.len();
    if w.blocked.is_some() && w.final_state.ending_done {
        // 不变式（防御）：blocked/capped/model_broken 永不带 ending_done=true。
        w.final_state.ending_done = false;
    }
    if w.blocked.is_some() {
        w.final_state.floor = w.blocked.as_ref().map(|b| b.floor).unwrap_or(start_floor);
    }
    w
}
