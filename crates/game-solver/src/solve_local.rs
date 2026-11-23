//! 【求解器批2 C1-C5】单层求解 `solve_local`（提案 §5：C1 骨架+子命令壳 / C2 可达性+战斗 /
//! C3 目标双查询 / C4 开门+钥匙经济 / C5 脚本触发转移 StepGate）。
//!
//! **纯度边界**（`lib.rs` 契约 + 计划 :456 A9）：本模块=纯函数——输入 `SolveInput`
//! （由 bin/测试装配），输出 `SolveResult`；**禁文件/网络/时钟/进程 I/O**、禁 GameRng/Clock。
//! 后续切片：C6 floor1 端到端 → C7 对拍回归。
//!
//! **C2 转移分工（提案 §1.3 直调红线）**：
//! - 战斗 Refused/Engage = 直调 `BattleResolver::resolve_bump`（判据 `battle.rs:121` 在其内部，
//!   ❌不抄成本地 if），solver 自备 `BattleCtx` 10 写槽（`:59-75` 全 pub）；
//! - 战斗损失 = 直调 `predict_engagement`（`battle.rs:85`，bump 面 `no_counter=true`），
//!   ❌不自算 hits/loss；
//! - 可达子图 = 直调 `find_path`（`pathfind.rs:55`）+ `Walkability`（`:21`）；
//! - 类目分类 = 直调 `entity_category_with_floor`（`script_host.rs:96`，层覆写单源）；
//! - 开门（C4）= 直调 `consume_key_for_door`（`combat.rs:170`，门映射同 `scene.rs:3397-3399`），
//!   ❌不自写「有钥减一」；钥匙拾取（type26）留 C6 接入面；
//! - 脚本触发（C5）= 判门直调 `try_run_scene(param,true)`（`scene.rs:927`）→ 载入
//!   `load_level_script`（`scene_persist.rs:501`）→ **有拍驱动** `tick`（`scene.rs:2834`，
//!   引擎节拍机自排序 phase0 指令→phase3 行走）→ 终止边读 `persist.pending_floor`
//!   （`scene_persist.rs:386`，§2.2 ④），❌禁自建 23 枚举转移表（§1.3）。
//!   **偏离提案 C5 行「驱动复用 side_a::drive」字面**：无拍 drive 实证无法产生终止边
//!   （行尾收尾 phase4 不再走 dp3、全部 `MOV_0_*` 从原点重武装成孤儿）；读回仍按
//!   §1.3 Obs/Snapshot/pending_floor 三源，报审注明该偏离。
//! - 搜索编排/去重支配/目标/路线序列化 = 本模块自算（§1.3 白名单）。
use crate::state::s0;
use game_core::combat::PlayerCombat;
use game_core::enums::GameMode;
use game_core::pathfind::{find_path, Walkability};
use game_core::scene::GameScene;
use game_core::scene_test_support::{scene_setup, SceneSetup};
use game_core::subsystem::battle::{
    predict_engagement, BattleCtx, BattleEvent, BattleOutcome, BattleResolver, BattleSubsystem,
    EnemyTypeId,
};
use game_core::subsystem::script_host::entity_category_with_floor;
use game_data::defs::{EnemyDef, ShopEffect, ShopGoodDef};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

/// 方向序=上、下、左、右（与 `tools/route_gen.py:29` `KEY` 表同序，提案 §3.3）。
const DIRS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

/// 单层求解输入（lib 纯数据；I/O 只在 `bin/solver.rs` 与测试侧）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveInput {
    /// 楼层号（计划 :327 判据硬编码 `--floor 1`）
    pub floor: i32,
    /// 难度钉 0（提案 §4.2 BR-4；case67 分支门 `scene.rs:1032`）
    pub difficulty: i32,
    /// 单层夹具规格；`None` ⇒ C1 占位分支（`placeholder`，T1 恒绿）
    pub level: Option<LevelSpec>,
}

impl SolveInput {
    /// C1 占位输入：只带 floor/difficulty（无夹具 ⇒ 占位结果）。
    pub fn placeholder(floor: i32) -> Self {
        SolveInput {
            floor,
            difficulty: 0,
            level: None,
        }
    }
}

/// 单层夹具规格（提案 §1.2 动作空间的格/实体/出口载体）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelSpec {
    pub cols: i32,
    pub rows: i32,
    /// 行主序 `terrain[y*cols+x]`：true=地形可走（maplv `tile_walkability` 同义）
    pub terrain: Vec<bool>,
    /// 入口（floor1=(6,11)，提案 §2）
    pub start: (i32, i32),
    /// 实体槽位=本 Vec 下标（遍历恒升序，提案 §3.3）
    pub entities: Vec<EntSpec>,
    /// 出口集（Exit#1 步行楼梯；C5 增 Exit#2 脚本换层）
    pub exits: Vec<ExitSpec>,
    /// 入口玩家状态 s₀
    pub player: PlayerSpec,
    /// 缩放表（difficulty=0 ⇒ base×1，BR-4）
    pub enemies: Vec<EnemyDef>,
    /// 基础表（trait_flags / 撞击面 hp 源）
    pub enemy_base: Vec<EnemyDef>,
    /// `script_line_flags` 位图初值（bit i = `script_line_flags[i]`；68 槽 ⇒ u128，
    /// 提案 §4.1 flags 进 K₀）。`load_floor` ⇒ 0；T9 夹具显式置位（重入断言）
    pub flags: u128,
}

/// 实体夹具记录（type_id + 格坐标 + 场景参数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntSpec {
    pub ty: i32,
    pub x: i32,
    pub y: i32,
    pub param: i32,
}

/// 出口类别（批2=Exit#1 步行楼梯；Exit#2 脚本换层边 C5 接入）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExitKind {
    Stair,
    /// Exit#2 脚本换层边（C5 StepGate 终止边，§2.2 ④）。§1.4 ② 不入 min_kills
    /// 量化域——`Kills::eligible` 只认 `Stair`；`HpLoss` 全收。
    Script,
    /// 批3 C2 终局边（通关终点）：floor50 击败型72 → `load_level_script(65)`
    /// （scene.rs:3891-3894）→ script65=`TAK_127_128 END_0` → `op_end`
    /// （scene.rs:269）→ `GameMode::Ending`（计划 §2.6 通关判定）。
    /// `Cost` 两实现不动：`HpLoss::eligible` 全收天然含本变体；`Kills::eligible`
    /// 仍只认 `Stair`（批2 量化域零变化；全局 Q1/Q2 取解在 solve_global 自做）。
    Ending,
    /// 批3 C3b 筋斗云换层边（QoL 件15，计划 R11 独立换层通道）：tutorialSeen
    /// （≡背包含15，唯一写点 pickup case15 a.java:7539、use=-1 永不消耗）∧
    /// f_bool_13（与层内锚点四邻非同格 a.java:5767-5806）∧ 首个 `param>>9==0`
    /// 的 type7/8 → `changeFloor(..., strict=true)`（a.java:5295-5306/7115-7178）。
    /// 与 Script 同族：`Kills::eligible` 只认 Stair ⇒ 不入 min_kills 量化域
    /// （裁定 Exit#1）。区间门（min/maxFloorReached=全局史）在 solve_global 落闸。
    Cloud,
    /// 批3 F3 UseItem 符21/22 换层边（lead 2026-11-12 裁定；a.java case21/22 =
    /// `changeFloor(currentFloor+1, false, false)` / `(currentFloor-1, true,
    /// false)`，var3=false **无区间门**，边界 `var1<0 || >floorCountMax` 拒
    /// ⇒ 目标层缺规格由 solve_global 丢边）。`itemUseCounts=1`（a.java:944：
    /// 21→idx8、22→idx9）⇒ 成功使用即消耗、post 扣一枚。`Kills::eligible` 只认
    /// Stair ⇒ 不入 min_kills 量化域（裁定 Exit#1）；起跳不耗步。
    UseItem,
    /// S5 段分治锚击杀边（outer_chain 段末任务）：`bump` 撞杀锚 boss 即达段末
    /// 目标，非换层、非通关。仅 `boss_target` 注入链路
    /// （`solve_layer_boss_shops`/`solve_local_boss_toward_shops`）产出；
    /// 默认链路不产此目标 ⇒ 既有出层集零变化。不入任何 solve_global 量化域。
    BossKill,
}

/// 出口定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitSpec {
    pub x: i32,
    pub y: i32,
    pub kind: ExitKind,
    /// 出口序号（1-based；本批 Exit#1=步行楼梯，提案 §1.4 ②量化域标签。
    /// 与 type7 `param` 无关——原 doc「route_gen 楼层号」为误注，2026-11-11 更正）。
    pub index: u8,
}

/// 玩家状态纯数据镜像（`PlayerCombat` 无 derive，`combat.rs:51-63`；
/// `from_combat`/`to_combat`=逐字段机械搬运，非语义重写）。
/// 批3 C5b：+`Hash`（gate_memo 键分量；game-solver 自有类型，非 game-core 面）。
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerSpec {
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

impl PlayerSpec {
    pub fn from_combat(p: PlayerCombat) -> Self {
        PlayerSpec {
            hp: p.hp,
            atk: p.atk,
            def: p.def,
            yellow: p.yellow_keys,
            blue: p.blue_keys,
            red: p.red_keys,
            gold: p.gold,
            item_types: p.item_types,
            item_uses: p.item_uses,
            item_stack_size: p.item_stack_size,
        }
    }

    pub fn to_combat(&self) -> PlayerCombat {
        PlayerCombat {
            hp: self.hp,
            atk: self.atk,
            def: self.def,
            yellow_keys: self.yellow,
            blue_keys: self.blue,
            red_keys: self.red,
            gold: self.gold,
            item_types: self.item_types.clone(),
            item_uses: self.item_uses.clone(),
            item_stack_size: self.item_stack_size,
        }
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
    /// **本次搜索是否撞层界被截断**（P3 收口1 / 复核 R2 黄）。
    ///
    /// 与 [`ExitSolve::truncated`] **同源**：都读同一个 `take_layer_trunc()`，
    /// 语义也同一件事（`run_root` 撞 `layer_node_limit()` ⇒ `true`）。
    /// 旧实现缺这一位 ⇒ `SOLVER_LAYER_LIMIT=10` 与 `200_000` 的输出逐字段相同，
    /// 调用方无法区分「完整解」与「截断解」（复核 A 级实测）。
    /// **截断⇒`max_hp`/`route` 只作参考，不可当完整解消费**（与 `ExitSolve.ok` 同契约）。
    pub truncated: bool,
    /// 三态置信（计划 :448 A1）：UseItem 裁出 ⇒ 批2 恒 `bounded-best`（BR-7）；
    /// **撞层界 ⇒ `capped-search`**（P3 收口1：截断不许冒充 `bounded-best`，
    /// 用词与 `solve_global::RunStatus::Capped` 一致）
    pub confidence: &'static str,
    /// Q1（max_hp）路线：T6 JSON 单 `route` 字段取自本字段（提案 §1.4 ① / §3.1 T6 形状）
    pub route: Vec<String>,
    /// Q2（min_kills）路线：量化域=Exit#1（§1.4 ②）；T3(iii) `route_min_kills` 断言面。
    /// 不入 JSON——T6 形状只有单一 `route` 字段。
    pub route_min_kills: Vec<String>,
    /// Q1 路线钥匙消耗 `(yellow, blue, red)` = 入口 − 终态；T4 钥匙数变化断言面。
    /// 不入 JSON（T6 形状无此字段）。
    pub keys_used: (i32, i32, i32),
}

impl SolveResult {
    /// JSON 序列化（手写，字段序固定；本结构保持零 serde 依赖）。
    /// 字段序：floor, difficulty, s0, reachable, max_hp, min_kills, **truncated**,
    /// confidence, route。`truncated` 插在 `confidence` 之前（P3 收口1）——
    /// `route` 必须仍是**末字段**（tests/solve_local.rs T1 形状判据）。
    /// 置信词表随截断扩一档：`capped-search`（见字段注）。
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
            "{{\"floor\":{},\"difficulty\":{},\"s0\":{},\"reachable\":{},\"max_hp\":{},\"min_kills\":{},\"truncated\":{},\"confidence\":\"{}\",\"route\":[{}]}}",
            self.floor,
            self.difficulty,
            s0,
            self.reachable,
            opt(self.max_hp),
            opt(self.min_kills),
            self.truncated,
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
/// - `level=None`（C1 占位）⇒ `reachable=false, max_hp/min_kills=None`；
/// - `level=Some(..)` ⇒ C2 搜索：可达性+路线（`max_hp`/`min_kills` 自 C3 起填，
///   防 T3 写时即绿）。
/// 真实语义见提案 §1 形式化与 §5 切片表。
pub fn solve_local(input: &SolveInput) -> SolveResult {
    let Some(level) = input.level.as_ref() else {
        let (hp, atk, def, floor) = s0();
        let _ = floor; // s0 的 floor 与求解楼层不同号（真源 a.java:3482-3491），输出只注 hp/atk/def
        return SolveResult {
            floor: input.floor,
            difficulty: input.difficulty,
            s0: (hp, atk, def, 0, 0, 0, 0),
            reachable: false,
            max_hp: None,
            min_kills: None,
            // 占位（无 level）=根本没搜 ⇒ 不存在「被截断的搜索」，不是恒 false 的伪装。
            truncated: false,
            confidence: "feasible",
            route: Vec::new(),
            route_min_kills: Vec::new(),
            keys_used: (0, 0, 0),
        };
    };
    // P3 收口1：入口清截断标记（与 `solve_layer_with` 同式）——否则本次会读到
    // 同一线程上一次（被截断的）搜索残留的 `LAYER_TRUNC`，把「上次截断」错记成
    // 「本次截断」（方向 A 要求 truncated 只描述本次搜索）。
    LAYER_TRUNC.with(|f| f.set(None));
    // 【P7 撞门】计数同式清零（与 `solve_layer_with` 一致）：`solve_local` 直连
    // `Search::run`，不经层缓存 ⇒ 计数必须只反映**本次**搜索。
    GATE_BUMP_STAT.with(|c| c.set(GateBumpStat::CLEARED));
    let mut search = Search::new(input, level);
    // 【批8】hp 上界剪枝：批2 同时要「最优 hp」与「最少击杀」两个查询 ⇒ 打开双目标保护。
    search.hp_prune = true;
    search.kill_objective = true;
    let (reachable, goals) = search.run();
    // 与 `ExitSolve.truncated` 同源：同一个 `take_layer_trunc()`（只读不清）。
    let truncated = take_layer_trunc().is_some();
    let hp_goal = pick::<HpLoss, _>(&goals);
    let kill_goal = pick::<Kills, _>(&goals);
    SolveResult {
        floor: input.floor,
        difficulty: input.difficulty,
        s0: (
            level.player.hp,
            level.player.atk,
            level.player.def,
            level.player.yellow,
            level.player.blue,
            level.player.red,
            level.player.gold,
        ),
        reachable,
        max_hp: hp_goal.map(|g| g.hp),
        min_kills: kill_goal.map(|g| g.kills),
        truncated,
        // BR-7/§64：批2 单层版策略空间受限但空间内最优 ⇒ bounded-best，不冒充 optimal。
        // P3 收口1：本次撞层界 ⇒ 连「空间内最优」都不成立（搜索没走完）⇒ capped-search。
        confidence: if truncated { "capped-search" } else { "bounded-best" },
        route: hp_goal.map(|g| g.route.clone()).unwrap_or_default(),
        route_min_kills: kill_goal.map(|g| g.route.clone()).unwrap_or_default(),
        keys_used: hp_goal
            .map(|g| {
                (
                    level.player.yellow - g.keys.0,
                    level.player.blue - g.keys.1,
                    level.player.red - g.keys.2,
                )
            })
            .unwrap_or((0, 0, 0)),
    }
}

/// 批3 C2 层目标（`solve_global` 层间传播的单层结果镜像；`Goal` 私有面的 pub 投影）。
///
/// 字段=单层求解终态的**层间耦合量**（计划 §3.2/§3.3：hp、钥匙、金币、装备、
/// 背包、脚本旗标、存活位图）——`post.*` 是跨层携带的全部状态，全局层不读层内私有态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerGoal {
    /// 终态 hp（Q1 目标面）
    pub hp: i32,
    /// 本层击杀数（Q2 拼接面：全局 kills=逐层和）
    pub kills: i32,
    /// 本层步数（确定性/tie-break 面：全局 steps=逐层和）
    pub steps: i32,
    /// 终态三色钥匙 `(yellow, blue, red)`
    pub keys: (i32, i32, i32),
    /// 出口类别（`Stair`=层间边 / `Script`=StepGate 终止边 / `Ending`=通关终局边）
    pub kind: ExitKind,
    /// 本层路线（route_gen token：`x,y` / `bump ty@x,y` / `door x,y`，首 token=入场格）
    pub route: Vec<String>,
    /// 终态玩家全状态（hp/atk/def/三色钥/金币/背包——下一层入场 `player` 的注入源）
    pub post: PlayerSpec,
    /// 终态格（出口格=楼梯换层边的取参面：出口梯实体查找）
    pub post_pos: (i32, i32),
    /// 终态存活位图（本层；下一层注入同层重入或本层 post 直传）
    pub post_ents: Vec<EntRec>,
    /// 终态 `script_line_flags` 位图（下一层注入 `flags`）
    pub post_flags: u128,
    /// 批3 C3a/C3b：换层边两元组 `(target, went_down)`（`Script`/`Cloud` 臂非 None；
    /// `solve_global` 换层落点的取参源；Stair 由出口梯实体 param 取参）
    pub cross_edge: Option<(i32, bool)>,
}

/// 批3 C2 单层求解门面（`solve_local`↔`solve_global` 唯一接口；`Search` 保持私有）。
///
/// - `entry`：入场格（批2 特例=`level.start`；层间边=上一层出场梯落点，
///   `swap_floor` 语义 `scene_persist.rs:410-433`）；
/// - `ents`：初始活体实体列表（C6a A-lite）；`None` ⇒ `sprite_full`（新层首入）；
/// - `player`/`flags`：注入的入场玩家状态与脚本旗标（层间携带上一层 `post`；
///   `run()` 特例=`level.player`/`level.flags`）。
///
/// 返回 `(任一层目标可达, 全部非支配层目标)`；`level=None` ⇒ `(false, vec![])`。
pub fn solve_layer(
    input: &SolveInput,
    entry: (i32, i32),
    ents: Option<&[EntRec]>,
    player: PlayerSpec,
    flags: u128,
) -> (bool, Vec<LayerGoal>) {
    solve_layer_with(input, entry, ents, player, flags, ClosurePolicy::OFF, None, &[])
}

/// 【P9.1 更正（复核 F3/F8）】[`solve_layer`] 的**带商品表**变体。
///
/// 旧版 [`solve_layer`] 硬编码 `&[]`（空商品表），witness 链因此永远在「空商品表口径」下
/// 求层解——f3 那层实际有 17 行货表中的商人格（`ty77 param=0x116`），旧链把它当
/// 「无商店」，与真链路口径不同。本函数把**真实商品表**（`SolverInput.shops`）一路带进
/// [`solve_layer_with`]：`hp_bound::plan`、`exact_layer_view`、`Search.shops`、memo 键
/// 全都按真表判。
///
/// 保留 [`solve_layer`] 的空表入口只为老夹具/单测（它们没有商品表）；生产路径
/// （`solver run` → witness → `solve_local_exit_toward_shops`）**必须**走本函数。
pub fn solve_layer_shops(
    input: &SolveInput,
    entry: (i32, i32),
    ents: Option<&[EntRec]>,
    player: PlayerSpec,
    flags: u128,
    shops: &[game_data::defs::ShopGoodDef],
) -> (bool, Vec<LayerGoal>) {
    solve_layer_with(input, entry, ents, player, flags, ClosurePolicy::OFF, None, shops)
}

/// S5 段分治：**锚击杀**层解——`boss_ty` 的撞杀（`bump` 杀）即记
/// `ExitKind::BossKill` 候选。与 [`solve_layer_shops`] 同一搜索内核，仅多注入
/// `boss_target`；默认链路（`boss_target=None`）不产 BossKill 目标 ⇒ 零变化。
pub fn solve_layer_boss_shops(
    input: &SolveInput,
    entry: (i32, i32),
    ents: Option<&[EntRec]>,
    player: PlayerSpec,
    flags: u128,
    shops: &[game_data::defs::ShopGoodDef],
    boss_ty: i32,
) -> (bool, Vec<LayerGoal>) {
    solve_layer_with(input, entry, ents, player, flags, ClosurePolicy::OFF, Some(boss_ty), shops)
}

/// 【P3】单层「入场 → 出层」可续跑求解结果。
///
/// 与 [`solve_local`] 的分工：那个回答「本层能不能到出口」（`reachable`/`max_hp`），
/// 本结构回答「**带着什么状态出层**」——`out_player`/`out_flags`/`out_ents` 就是下一层
/// [`solve_layer`] 的三个注入参数，`route` 与 `SolveResult.route` 同一套 route_gen token，
/// 可直接回放。全局求解原先只能拿「残缺层状态」接力（`coverage.truncated` 的 5 层），
/// 根因是层解不导出出层态；本结构补齐这一环。
///
/// 语义铁律：出层状态**只来自引擎读回**（`pickup_step`/`gate_arrive` 内的
/// `readback_script`），本层不做任何「结算」推演；掉落只生成实体、**不凭空入账**
/// （同 P2 判据 `boss_kill_spawns_drop_entity_from_core_rules_only`）——
/// `out_player.item_types` 里有东西 ⇔ 路上真的踩过那一格。
///
/// 【契约】`ok` 与 `truncated` **不正交**（2026-11-18 P3 复核红2 改判）：
/// `truncated=true ⇒ ok=false`，且此时 `out_*` 只作参考、不是可消费的出层态。
/// 消费方只判 `ok` 一个开关即可，不必（也不该）另查 `truncated` 才能定成败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitSolve {
    /// 是否求出**可信**的「能出层」解 = `reached_goal && !truncated`
    /// （`level=None` / 被怪封死 / 本层无出口 / 搜索撞层界 ⇒ false）。
    /// 唯一成败开关：不要用「`out_*` 非空」推断 ok，也不要在 `ok=true` 时忽略 `truncated`。
    pub ok: bool,
    /// 出层落点（楼梯/门/脚本出口的格子）
    pub exit: Option<(i32, i32)>,
    /// 出层后的玩家状态（hp/atk/def/三色钥/金币/背包）
    pub out_player: PlayerSpec,
    /// 出层后的脚本域状态（`script_line_flags` 位图）
    pub out_flags: u128,
    /// 出层后仍在场的实体（`EntRec` 列表，按引擎口径 `readback_ents`）
    pub out_ents: Vec<EntRec>,
    /// 路线（与既有 `SolveResult.route` 同格式，便于回放）
    pub route: Vec<String>,
    /// 出层时剩余血量（= 目标函数）；`truncated=true` 时只是**部分搜索**的最优参考值，
    /// 不得当「出层最大血」消费（同 `out_*` 口径）。无任何目标时回显入场 hp。
    pub out_hp: i32,
    /// 【附加】出层类别（`LayerGoal.kind` 直传：Stair/Script/Cloud/UseItem/Ending）。
    /// 任务书的 7 字段不够消费方判「这条边通到哪」——与 `solve_global` 同口径补回。
    pub kind: Option<ExitKind>,
    /// 【附加】换层边两元组 `(target, went_down)`（Script/Cloud/UseItem 臂非 None；
    /// Stair 由出口梯实体 `param` 取参，与 `solve_global::stair_target` 同口径）
    pub cross_edge: Option<(i32, bool)>,
    /// 本次搜索是否撞层界（`layer_node_limit()`）：`true` ⇒ **本次结果不可信**
    /// （`ok` 必为 `false`；`out_player`/`out_flags`/`out_ents`/`route` 仅作参考——
    /// 是「部分搜索里找到的最优一条」，不能当作「出层最大血」或下一层入场态消费）。
    /// `false` ⇒ 空间内穷尽：`ok=false` 即真无解，`ok=true` 即空间内最优（可当契约）。
    /// 依据 `LAYER_TRUNC`（`solve_layer_with` 与 `solve_global` 观测同一标记）。
    pub truncated: bool,
    /// 【P5 纯拾取闭式】本条结果是否来自 `pure_pickup.rs` 的**层闭式解**：
    /// `true` ⇒ `out_hp` 是 `entry_hp + Σ认证纯加血` 的闭式上界，且 `route` 已**逐步
    /// 引擎回放验证**（每步 `pickup_step` 成功 + 终态血量/位置/零副作用逐项相符）。
    /// `false` ⇒ 通用搜索（闭式未采用或未尝试；**原因必在**
    /// [`crate::pure_pickup::pure_pickup_stat`]`.reason` 里，禁静默）。
    pub pure_pickup: bool,
    /// 【P5·F7 修复】闭式判定的**可消费标签**（`PurePickupStat::reason_label`）：
    /// `None` = 本层没问过闭式（早退/memo 之外未到闭式分支）；
    /// `Some("adopted")` = 命中；其余 = fail-closed 具名拒绝原因（`reason/ty`，如
    /// `monster_ents/41`、`step_arm_hazard_reachable/83`）。
    /// 为什么要它：`solver run` 消费的是 JSON，而 `pure_pickup` 布尔位只够判
    /// 「是不是闭式」，判不了「为什么不是」——复核 F7 的另一半。
    pub pure_pickup_reason: Option<String>,
}

impl ExitSolve {
    /// 无有效目标时的回显形态：入场状态原样返回（`ok=false` ⇒ 调用方只看 `ok` 一个开关）。
    fn failed(
        player_in: &PlayerSpec,
        flags_in: u128,
        ents_in: Vec<EntRec>,
        truncated: bool,
    ) -> Self {
        ExitSolve {
            ok: false,
            exit: None,
            out_player: player_in.clone(),
            out_flags: flags_in,
            out_ents: ents_in,
            route: Vec::new(),
            out_hp: player_in.hp,
            kind: None,
            cross_edge: None,
            truncated,
            pure_pickup: false,
            pure_pickup_reason: None,
        }
    }
}

/// 【P3】单层「可续跑」求解：给定入场状态，求一条**能出层**的路线，并导出出层状态。
///
/// 首参是 [`SolveInput`] 而非 `&LevelSpec`（任务书原型的 `level: &LevelSpec` 无法提供
/// 楼层号）：`entity_category_with_floor`、`apply_kill_drops`、floor50 `Ending` 判定
/// 全部以 `input.floor` 为轴，而 `LevelSpec` 不含楼层号。`SolveInput` = 层规格 + 楼层 +
/// 难度标签，正是既有层门面 [`solve_layer`] 的首参——本函数是它的**同形投影**，
/// 不新增第二套入口约定。
///
/// - `entry`：入场格；`player_in`/`flags_in`/`ents_in`：入场三态注入（层间携带上一层 `ExitSolve`）；
/// - `ents_in = None` ⇒ 用 `level` 自带实体（`sprite_full`，新层首入语义）；
///   `ents_in = Some(v)` ⇒ **v 就是全部活体**（`Some(&[])` = 层内实体全清——这是 P3 复核
///   F1 缺口的一行式补口：空切片不再被并入 sprite，语义无歧义）；
/// - 目标 = **出层血量最大**（tie-break：步数少 → 路线字典序），复用既有 [`HpLoss`]
///   代价序；选解发生在**已收集的目标集**上，不改搜索空间 ⇒ 不引入新的状态爆炸面
///   （搜索仍受 `weakly_ge`/`dominates` 剪枝 + `layer_node_limit()` 层界管辖）；
/// - 注入路径：`solve_layer`（已支持四注入）→ `search.run_root(entry, ents, player, flags)`；
///   出层读回复用 `pickup_step`/`gate_arrive` 内的 `readback_script`/`readback_ents`。
/// - 成败：`ok = 有目标 && !truncated`（截断即不可信，见 [`ExitSolve::truncated`]）。
pub fn solve_local_exit(
    input: &SolveInput,
    entry: (i32, i32),
    player_in: &PlayerSpec,
    flags_in: u128,
    ents_in: Option<&[EntRec]>,
) -> ExitSolve {
    let (reachable, goals) = solve_layer(input, entry, ents_in, player_in.clone(), flags_in);
    let truncated = take_layer_trunc().is_some();
    // 【P10】出层择优走分层软契约（objective::cmp_exit：hp 大 → gold 多 → steps 少），
    // 不再是旧 HpLoss（hp→steps）——gold 软层加入；demand 绝不参与（objective.rs doc）。
    let best = pick_exit(&goals);
    materialize_exit(input, reachable, best, player_in, flags_in, ents_in, truncated)
}

/// 【P4】[`solve_local_exit`] 的**定向变体**：优先取「出层边指向 `toward` 层」的解。
///
/// # 为什么需要（实证，不是猜）
///
/// [`solve_local_exit`] 的目标挑选是 [`HpLoss`]（出层血最大 → 步数少 → 路线字典序），
/// **不带方向语义**。塔里带下行梯的层（f52 `StairDown@(1,11)→f51`）上，若「往下走」
/// 恰好挤到更多血（血瓶/商店换血），规范挑选就会选**下行**梯 —— P4 首跑实测
/// f51→f52→f51，`blocked/cycle`，`floors=2`。要线性推进，必须能把「P0 层图给的下一层」
/// 作为优先目标层传进来。
///
/// # 语义（零变化面）
///
/// 搜索空间/剪枝/`ok`/`truncated`/`LAYER_MEMO` 键全部沿用 [`solve_local_exit`]；
/// 本函数只在**已收集目标集**上改挑选：命中「边指向 `toward`」的子集 ⇒ 仍按 [`HpLoss`]
/// 在该子集取最优；否则回落规范挑选。第二个返回值 `hit` **显式**告诉调用方
/// 「优先目标是否被满足」——消费方必须看它：`hit=false` 时的解是**回落解**（可能往回走），
/// 把它当「前进」消费就是造假（witness 侧按 blocked 如实上报）。
pub fn solve_local_exit_toward(
    input: &SolveInput,
    entry: (i32, i32),
    player_in: &PlayerSpec,
    flags_in: u128,
    ents_in: Option<&[EntRec]>,
    toward: Option<i32>,
) -> (ExitSolve, bool) {
    solve_local_exit_toward_shops(input, entry, player_in, flags_in, ents_in, toward, &[])
}

/// 【P9.1 更正（复核 F3/F8）】[`solve_local_exit_toward`] 的**带真实商品表**变体。
///
/// 挑选/物化语义与 [`solve_local_exit_toward`] 完全一致，唯一差别是层解走
/// [`solve_layer_shops`]——这样商店格在 `hp_bound`/精确 DP/`Search` 三处都按真表判，
/// 而不是一律当「没有商店」。生产链路（witness）必须走本函数；空表入口只留给没有
/// 商品表的老夹具。
#[allow(clippy::too_many_arguments)]
pub fn solve_local_exit_toward_shops(
    input: &SolveInput,
    entry: (i32, i32),
    player_in: &PlayerSpec,
    flags_in: u128,
    ents_in: Option<&[EntRec]>,
    toward: Option<i32>,
    shops: &[game_data::defs::ShopGoodDef],
) -> (ExitSolve, bool) {
    let (reachable, goals) =
        solve_layer_shops(input, entry, ents_in, player_in.clone(), flags_in, shops);
    let truncated = take_layer_trunc().is_some();
    let prefer: Vec<&LayerGoal> = match (toward, input.level.as_ref()) {
        (Some(t), Some(lv)) => goals
            .iter()
            .filter(|g| goal_edge(lv, g).map(|(to, _)| to) == Some(t))
            .collect(),
        _ => Vec::new(),
    };
    let hit = !prefer.is_empty();
    // 【P10】定向子集上的择优同样走分层软契约（同一份比较器，不写第二份）。
    let best = if hit {
        best_goal(prefer.into_iter())
    } else {
        pick_exit(&goals)
    };
    (materialize_exit(input, reachable, best, player_in, flags_in, ents_in, truncated), hit)
}

/// S5 段分治：**锚击杀**面（outer_chain 段末任务消费）。
///
/// 语义与 [`solve_local_exit_toward_shops`] 同构，唯一差别是目标集 =
/// `ExitKind::BossKill` 候选（`bump` 撞杀 `boss_ty` 即达段末目标；非换层非通关）。
/// 无 BossKill 候选 ⇒ `hit=false` 且 `ExitSolve.ok=false`（不回退到普通出层解——
/// 段末任务的成功判据**只有**杀锚，走到出口不算）。
#[allow(clippy::too_many_arguments)]
pub fn solve_local_boss_toward_shops(
    input: &SolveInput,
    entry: (i32, i32),
    player_in: &PlayerSpec,
    flags_in: u128,
    ents_in: Option<&[EntRec]>,
    boss_ty: i32,
    shops: &[game_data::defs::ShopGoodDef],
) -> (ExitSolve, bool) {
    let (reachable, goals) =
        solve_layer_boss_shops(input, entry, ents_in, player_in.clone(), flags_in, shops, boss_ty);
    let truncated = take_layer_trunc().is_some();
    let prefer: Vec<&LayerGoal> =
        goals.iter().filter(|g| g.kind == ExitKind::BossKill).collect();
    let hit = !prefer.is_empty();
    let best = if hit { best_goal(prefer.into_iter()) } else { None };
    (materialize_exit(input, reachable, best, player_in, flags_in, ents_in, truncated), hit)
}

/// 目标的换层边 `(target, went_down)`：`Stair` 取出口梯实体 `param&0xff`（同
/// [`crate::solve_global::stair_target`] 口径），`Script`/`Cloud`/`UseItem` 取
/// `cross_edge`，`Ending` 非换层 ⇒ `None`。
pub fn goal_edge(level: &LevelSpec, g: &LayerGoal) -> Option<(i32, bool)> {
    match g.kind {
        ExitKind::Stair => crate::solve_global::stair_target(level, g.post_pos),
        ExitKind::Ending => None,
        _ => g.cross_edge,
    }
}

/// 【P10】在**已筛子集**上按分层软契约取最优（与 [`pick_exit`] 同一比较器，
/// 不写第二份 tie-break；demand 绝不参与——objective.rs 模块 doc 契约）。
fn best_goal<'g>(goals: impl Iterator<Item = &'g LayerGoal>) -> Option<&'g LayerGoal> {
    goals.fold(None, |best: Option<&'g LayerGoal>, g| match best {
        None => Some(g),
        Some(b) if better_exit(g, b) => Some(g),
        some => some,
    })
}

/// 【P10】目标集上的出层择优：[`crate::objective::cmp_exit`] 分层软序
/// （hp 大 → gold 多 → steps 少），Equal 时路线字典序做**决定性** tie-break
/// （契约外，保证同输入同输出；demand 字段不参与比较）。
fn better_exit(a: &LayerGoal, b: &LayerGoal) -> bool {
    match crate::objective::cmp_exit(
        &crate::objective::score_of_goal(a),
        &crate::objective::score_of_goal(b),
    ) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => a.route < b.route,
    }
}

/// [`better_exit`] 的 argbest 面（替代旧 `pick::<HpLoss,_>` 在出层口的用途；
/// 旧比较器仍归 [`solve_local`]（Q1 遗留面）专用）。
fn pick_exit(goals: &[LayerGoal]) -> Option<&LayerGoal> {
    let mut best: Option<&LayerGoal> = None;
    for g in goals {
        best = Some(match best {
            None => g,
            Some(b) if better_exit(g, b) => g,
            Some(b) => b,
        });
    }
    best
}

/// 层目标 → [`ExitSolve`] 的物化（[`solve_local_exit`] 与 [`solve_local_exit_toward`]
/// 共用同一份 `ok/truncated` 契约与回显逻辑，禁止第二份）。
fn materialize_exit(
    input: &SolveInput,
    reachable: bool,
    best: Option<&LayerGoal>,
    player_in: &PlayerSpec,
    flags_in: u128,
    ents_in: Option<&[EntRec]>,
    truncated: bool,
) -> ExitSolve {
    // 闭式判定统计只在本线程最近一次 `solve_layer_with` 里有效（thread_local，层内
    // 单写）——memo 命中时它由 `pure_pickup_stat_set(hit_pure)` 回放 ⇒ 同一份可观测面。
    let closed = crate::pure_pickup::pure_pickup_stat();
    let (closed_flag, closed_reason) = (closed.adopted, closed.reason_label());
    match best {
        // `Some(g) if reachable`：有目标且搜索确认可达才回填目标态。
        // 截断 ⇒ 结果不可信 ⇒ ok=false，但 out_* 仍回填「部分搜索里最优的一条」作参考
        // （红2-A 契约：ok 只保证「可信出层」，不保证「截断时无任何路线」）。
        Some(g) if reachable => ExitSolve {
            ok: !truncated,
            exit: Some(g.post_pos),
            out_player: g.post.clone(),
            out_flags: g.post_flags,
            out_ents: g.post_ents.clone(),
            route: g.route.clone(),
            out_hp: g.hp,
            kind: Some(g.kind),
            cross_edge: g.cross_edge,
            truncated,
            // 闭式解采用标记直取统计（`solve_layer_with` 在 memo 命中/闭式命中时都写）
            pure_pickup: closed_flag,
            pure_pickup_reason: closed_reason,
        },
        _ => {
            // 无目标（或目标未确认可达）：回显「入场态」——注入则原样，未注入则本层
            // sprite（下一层首入态），不是空表。
            let echo = match ents_in {
                Some(v) => v.to_vec(),
                None => input.level.as_ref().map(sprite_full).unwrap_or_default(),
            };
            let mut r = ExitSolve::failed(player_in, flags_in, echo, truncated);
            // 无解也保留闭式判定标签（「闭式拒了、通用搜索也没解出」是两种不同的坏）
            r.pure_pickup = closed_flag;
            r.pure_pickup_reason = closed_reason;
            r
        }
    }
}

/// 商品表在 `LAYER_MEMO` 键里的规范行：`(id, src, idx, price, 效果编码)`。
///
/// **为什么是摊平元组而不是 hash 摘要**（P3 收口2）：摘要要接受碰撞概率，而这里是
/// 缓存键——碰撞=静默串味（复核 R3 抓到的正是这一类 bug）。元组同时参与 `Hash` 与
/// `Eq`，哈希撞了也被 `Eq` 兜住 ⇒ 精确。为什么不用 `ShopGoodDef` 本体：它没有
/// `Hash`/`Eq`（`game-data` 不为求解器缓存改 derive），摊平后就是纯整数，零耦合。
///
/// 效果编码 `(tag, a, b, c)` 逐臂定长（`ShopEffect` 七臂，见 `game-data/src/defs.rs`）：
/// 1=GiveGold(amount) 2=AddStack(item) 3=StatPct(atk,def) 4=AddKeys(blue,yellow,red)
/// 5=SwapYellowKey(gold) 6=AddHp(amount) 7=PickupItem(item)；未用槽补 0。
pub(crate) type ShopKeyRow = (i32, i32, i32, i32, (u8, i32, i32, i32));

/// `shops` → [`ShopKeyRow`] 序列（顺序敏感：搜索按 `find` 首配取行，顺序是输入的一部分）。
///
/// **不进编码的字段**：`ShopGoodDef.fail_hint`（只用于失败提示文案，不参与任何
/// 走位/买臂/资源判定 —— `shop_offer` 只读 `src`/`idx`/`price`/`effect`）。
pub(crate) fn shops_key(shops: &[ShopGoodDef]) -> Vec<ShopKeyRow> {
    shops
        .iter()
        .map(|g| {
            let eff = match &g.effect {
                ShopEffect::GiveGold { amount } => (1u8, *amount, 0, 0),
                ShopEffect::AddStack { item } => (2, *item, 0, 0),
                ShopEffect::StatPct { atk_pct, def_pct } => (3, *atk_pct, *def_pct, 0),
                ShopEffect::AddKeys { blue, yellow, red } => (4, *blue, *yellow, *red),
                ShopEffect::SwapYellowKey { gold } => (5, *gold, 0, 0),
                ShopEffect::AddHp { amount } => (6, *amount, 0, 0),
                ShopEffect::PickupItem { item } => (7, *item, 0, 0),
            };
            (g.id, g.src, g.idx, g.price, eff)
        })
        .collect()
}

/// 批4 C4-a：带贪心零决策闭包谓词的层求解（`solve_layer`=谓词 `OFF` 的转发，
/// 批0-3 回归判据「逐层拼接==全局解」在 OFF 下逐字节不变）。
/// 缓存键含 `policy`——同层不同谓词是不同纯函数，不许串味。
pub fn solve_layer_with(
    input: &SolveInput,
    entry: (i32, i32),
    ents: Option<&[EntRec]>,
    player: PlayerSpec,
    flags: u128,
    policy: ClosurePolicy,
    boss_target: Option<i32>,
    shops: &[ShopGoodDef],
) -> (bool, Vec<LayerGoal>) {
    LAYER_TRUNC.with(|f| f.set(None));
    GOLD_PEAK.with(|c| c.set(0));
    SHOP_BUYS.with(|c| c.set((0, 0)));
    // 【P7 撞门】同法清零（层缓存命中时不重放该计数——只作「本层撞门臂确实跑过」的信号，
    // 不参与任何判定，故不写进 `LAYER_MEMO` 值与键）。
    GATE_BUMP_STAT.with(|c| c.set(GateBumpStat::CLEARED));
    // 【D 面】统计入口清零（未跑到 run_root 的早退路径也保持「本层已测」语义：
    // `reason="not_run"` 可见，不是静默空白）。
    pill_dom_stat_set(PillDomStat::CLEARED);
    // 【批8】hp 上界统计同式清零（早退路径也保持「本层已测」语义）。
    crate::hp_bound::hp_bound_stat_clear();
    // 【P9】精确 DP 统计入口清零（同体例：早退路径也「本层已测」，reason=not_run）。
    crate::exact_pickup::exact_stat_clear();
    let Some(level) = input.level.as_ref() else {
        return (false, Vec::new());
    };
    let root_ents = match ents {
        Some(v) => v.to_vec(),
        None => sprite_full(level),
    };
    // C5d″：层结果缓存 lookup——全局队列里同 K₀ 不同 kills/steps 的变体会以完全
    // 相同输入反复 pop 同一层（f52 实测同输入 8 连 pop、单层状态 80 万）。
    //
    // **键实际包含什么（2026-11-18 P3 收口2 逐项核对，别再写「键=全部输入」这种不查就写的话）**：
    //   1. `input.floor`            —— 层的编号（判定面 `entity_category_with_floor` 用它）
    //   2. `entry`                  —— 入场格（层间边=上一层出场梯落点）
    //   3. `player.clone()`         —— 入场玩家全态（hp/atk/def/三色钥/金/背包）
    //   4. `flags`                  —— 入场 `script_line_flags`
    //   5. `root_ents.clone()`      —— 入场活体实体表（`ents` 为 None 时已展开成 `sprite_full`）
    //   6. `level_digest(input)`    —— 层规格内容摘要（difficulty+terrain/start/entities/
    //                                 exits/player/enemies/enemy_base/flags；见 `level_digest`）
    //   7. `policy`                 —— 闭包谓词（同层不同谓词=不同纯函数）
    //   8. `node_limit`             —— `layer_node_limit()`（**环境派生输入**：撞界与否是
    //                                 结果的组成部分，缺它则「先小界后大界」回放截断结果）
    //   9. `shops_key(shops)`       —— **商品表内容**的规范编码（P3 收口2 补；缺它则空表
    //                                 会回放带买臂的解，复核 R3 A 级探针实证）
    // 于是「键≡`solve_layer_with` 的全部实参（`input` 经 `level_digest` 压成内容摘要）」
    // 成立——**这是本次核对后才敢写的等式**，不是自称。
    // 唯一没进键的输入字段：`ShopGoodDef.fail_hint`（只喂 `show_overlay_message` 文案，
    // 不参与任何走位/买臂判定 ⇒ 对搜索纯函数无影响，剥离安全）。
    let node_limit = layer_node_limit();
    let memo_key = (
        input.floor,
        entry,
        player.clone(),
        flags,
        root_ents.clone(),
        level_digest(input),
        policy,
        // S5：boss_target 改变目标集 ⇒ 必须进键（不同目标不同解，缺它=跨目标串味）
        boss_target,
        node_limit,
        shops_key(shops),
    );
    if let Some((
        hit_ok,
        hit_goals,
        hit_trunc,
        hit_peak,
        hit_buys,
        hit_pill,
        hit_pure,
        hit_exact,
    )) = LAYER_MEMO.with(|m| m.borrow().get(&memo_key).cloned())
    {
        LAYER_TRUNC.with(|f| f.set(hit_trunc)); // memo 命中回放截断标记（可观测性不变）
        GOLD_PEAK.with(|c| c.set(hit_peak)); // 批6：金币峰值同样按缓存回放
        SHOP_BUYS.with(|c| c.set(hit_buys)); // 批6：买臂计数同样按缓存回放
        // 【D 面】子集支配统计同样按缓存回放——**禁静默**：memo 命中时也必须能
        // 看到 enabled/reason/cmp（否则「规则没生效」在热路径上不可见）。
        pill_dom_stat_set(hit_pill);
        // 【P5】闭式解统计同样按缓存回放——memo 命中时「本层是否闭式命中」也必须可见。
        crate::pure_pickup::pure_pickup_stat_set(hit_pure);
        // 【P9】精确 DP 统计同式回放（memo 命中 ⇒ 本层「是否精确命中」必须仍可见）。
        crate::exact_pickup::exact_stat_set(hit_exact);
        // 【批8】memo 命中=本层没重跑 ⇒ 显式标注，不把上一层的统计冒充本层。
        crate::hp_bound::stat_mark_memo_hit();
        return (hit_ok, hit_goals);
    }
    // ---------------------------------------------------------------------
    // 【P5 本片】纯拾取层闭式解：memo 之后、`Search` 构造之前先试。
    // 命中 ⇒ `out_hp = entry_hp + Σ认证纯加血`（全捡最优：无怪/无危险步臂/拾取不挡路，
    // 步数只是 tie-break），路线已逐步引擎回放验证 ⇒ 直接短路返回，绕开 20 万节点层界。
    // 未命中 ⇒ fail-closed 落到下面的通用搜索，**原因写进 `pure_pickup_stat()`**
    // （`reason`/`reason_ty`），禁静默。
    // S5：boss_target 模式跳过两档闭式/DP 短路（它们不产 BossKill 目标，命中会
    // 丢锚击杀解）——fail-closed 落通用 Search，锚层本就含怪，零性能顾虑。
    // ---------------------------------------------------------------------
    let closed_attempt = if boss_target.is_none() {
        crate::pure_pickup::try_closed_form(input, entry, &root_ents, &player, flags, policy)
    } else {
        None
    };
    if let Some(mut goals) = closed_attempt {
        // 出口序确定化（每出口一条 `LayerGoal`，`post_pos` 互异 ⇒ 无需 merge）：
        // 不许依赖 Vec 生成序（BFS/策略序是内部细节，出口序必须由内容定）。
        goals.sort_by(|a, b| (a.kind, a.post_pos).cmp(&(b.kind, b.post_pos)));
        LAYER_TRUNC.with(|f| f.set(None)); // 闭式=精确完整解，从未截断
        GOLD_PEAK.with(|c| c.set(player.gold)); // 纯层金币恒定（无买臂、无金币拾取）
        SHOP_BUYS.with(|c| c.set((0, 0)));
        LAYER_MEMO.with(|m| {
            m.borrow_mut().insert(
                memo_key,
                (
                    true,
                    goals.clone(),
                    None,
                    player.gold,
                    (0, 0),
                    pill_dom_stat(),
                    crate::pure_pickup::pure_pickup_stat(),
                    crate::exact_pickup::exact_stat(),
                ),
            );
        });
        return (true, goals);
    }
    // ---------------------------------------------------------------------
    // 【P9 本片】精确拾取 DP：闭式解之后、通用 `Search` 之前再试一档。
    // 定理与闸门见 `crate::exact_pickup` 模块头。适用面比闭式解宽一档
    // （允许零伤 cat8 怪、允许门钥消耗、允许恒等场景门），代价是状态数 2^拾取物。
    // 命中 ⇒ 层内 hp **精确最优**（不是上界、不是近似），路线已逐步引擎回放核对；
    // 未命中 ⇒ fail-closed 落通用搜索，理由写进 `exact_pickup_stat()`（禁静默）。
    // ---------------------------------------------------------------------
    let exact_attempt = if boss_target.is_none() {
        Some(crate::exact_pickup::try_exact_dp(
            input, entry, &root_ents, &player, flags, policy, shops,
        ))
    } else {
        None
    };
    if let Some(Some(mut goals)) = exact_attempt {
        goals.sort_by(|a, b| (a.kind, a.post_pos).cmp(&(b.kind, b.post_pos)));
        LAYER_TRUNC.with(|f| f.set(None)); // 精确=完整解，从未截断
        // 金币峰值：怪有赏金 ⇒ 层内峰值可能高于入场金（取各出口终态最大者）。
        let peak = goals.iter().map(|g| g.post.gold).max().unwrap_or(player.gold);
        GOLD_PEAK.with(|c| c.set(peak));
        SHOP_BUYS.with(|c| c.set((0, 0)));
        LAYER_MEMO.with(|m| {
            m.borrow_mut().insert(
                memo_key,
                (
                    true,
                    goals.clone(),
                    None,
                    peak,
                    (0, 0),
                    pill_dom_stat(),
                    crate::pure_pickup::pure_pickup_stat(),
                    crate::exact_pickup::exact_stat(),
                ),
            );
        });
        return (true, goals);
    }
    let mut search = Search::from_entry(input, level, entry);
    search.cloud = true; // 批3 C3b：层间路径开筋斗云候选（批2 solve_local/run 关）
    search.policy = policy; // 批4 C4-a：闭包谓词注入
    search.shops = shops; // 批5 H4：商品表注入（`&[]` ⇒ 批0-3 零行为变化）
    search.boss_target = boss_target; // S5 段分治：锚击杀目标注入（None=零变化）
    // 【批8】层路径**不启用** hp 剪枝：`solve_layer_with` 的消费者是 witness 的
    // 「朝目标层的出口」查询（`up_line_next`），目标不是「全层 hp 最大」——按全层
    // best_hp 剪枝会把「hp 较低但通向目标出口」的路径剪掉（实测 2026-11-19：
    // 一度启用后 `solver run` 由 floors=6 退化为 f52 `no_exit_toward`）。
    // 同层若要让剪枝生效，须把 best 限定到该查询的目标出口集合（未做，不猜）。
    let (reachable, goals) = search.run_root(entry, root_ents, player, flags, node_limit);
    // C5d′：层出口同态归并——同 (kind, cross_edge, post 全态) 组内 (kills,steps)
    // 支配剪枝（组内 hp 在键里=相等）。全局 push_state 对同 K₀ 子态做同样弱支配
    // ⇒ 本归并=同一结果前移，省 route 克隆与逐 goal GSt/GKey 克隆（f52 单层 3.2 万
    // goals 爆炸源）。仅 solve_layer（批3 面）——批2 solve_local 走 run() 不受影响
    // （其 Cost tie-break 依赖原始 goals 序）。BTreeMap 键序=确定性（禁 HashMap 序）。
    let goals = merge_layer_goals(goals);
    let mapped: Vec<LayerGoal> = goals
        .into_iter()
        .map(|g| LayerGoal {
            hp: g.hp,
            kills: g.kills,
            steps: g.steps,
            keys: g.keys,
            kind: g.kind,
            route: g.route,
            post: g.post.player,
            post_pos: g.post.pos,
            post_ents: g.post.ents,
            post_flags: g.post.flags,
            cross_edge: g.cross_edge,
        })
        .collect();
    // C5d″：纯函数结果入缓存（键清单见 lookup 处九项；gate memo 只提速不改输出）；
    // 截断标记随结果入缓存（同输入⇒同截断⇒确定性）
    LAYER_MEMO.with(|m| {
        m.borrow_mut().insert(
            memo_key,
            (
                reachable,
                mapped.clone(),
                LAYER_TRUNC.with(|f| f.get()),
                GOLD_PEAK.with(|c| c.get()),
                SHOP_BUYS.with(|c| c.get()),
                // 【D 面】统计（诊断面；不是搜索输入，进缓存只为 memo 命中可观测）
                pill_dom_stat(),
                // 【P5】闭式解统计（同上：诊断面，进缓存只为 memo 命中可观测）
                crate::pure_pickup::pure_pickup_stat(),
                // 【P9】精确 DP 统计（同上）
                crate::exact_pickup::exact_stat(),
            ),
        );
    });
    (reachable, mapped)
}

thread_local! {
    /// C5d″：层求解结果缓存——`solve_layer` 是（层规格+入场态）的纯函数 ⇒ 同键同
    /// 输出。全局队列中同 K₀ 不同 kills/steps 的状态以相同输入反复 pop 同层
    /// （f52 实测同输入 8 连 pop、单层 80 万状态、单层秒级）——缓存=跳过重复层搜索。
    /// 键含 `level_digest`（FNV-1a 层规格内容摘要）防「同 floor 不同夹具」串味
    /// （玩具测试同 floor 多套 LevelSpec）；键含 `node_limit`（层界也是输入之一——
    /// 否则同线程先小界后大界会回放旧的截断结果，见 `solve_layer_with` 调用点注）；
    /// 键含 `shops_key`（商品表内容——否则空表回放真表买臂解，P3 复核 R3）；
    /// 完整 9 项清单见 `solve_layer_with` 的 lookup 注（那里是唯一真相源）。
    /// thread_local=多线程测试隔离；缓存纯加速、不改输出与遍历序（确定性不受影响）。
    static LAYER_MEMO: RefCell<
        HashMap<
            (
                i32,
                (i32, i32),
                PlayerSpec,
                u128,
                Vec<EntRec>,
                u64,
                ClosurePolicy,
                // S5：boss_target 进键（见 solve_layer_with memo_key 注）
                Option<i32>,
                usize,
                Vec<ShopKeyRow>,
            ),
            (
                bool,
                Vec<LayerGoal>,
                Option<(i32, usize)>,
                i32,
                (u64, u64),
                PillDomStat,
                crate::pure_pickup::PurePickupStat,
                crate::exact_pickup::ExactStat,
            ),
        >,
    > = RefCell::new(HashMap::new());
    /// R1 层界截断标记（lead 裁定 A 2026-11-13 条件1「截断可观测」）：
    /// `Some((floor, nodes))`=该层在 nodes 达上限时截断。solve_layer 入口清零、
    /// run_root 触界置位、memo 命中按缓存值回放 ⇒ 全局层每调一次都可观测。
    static LAYER_TRUNC: std::cell::Cell<Option<(i32, usize)>> = std::cell::Cell::new(None);
    /// 批6 C6c：本层搜索**生成态金币峰值**（诊断量，不参与搜索决策、不进键/支配）。
    /// `solve_layer` 入口清零、`expand` 汇聚每个生成态、memo 命中按缓存回放 ⇒
    /// 全局层每调一次可观测，`solve_global` 取跨层最大值写入 `GlobalResult.gold_peak`。
    /// 用途：回答「击杀金币补边后金币是否仍是瓶颈」（批5 §4.5 的 210 峰值口径重测）。
    static GOLD_PEAK: std::cell::Cell<i32> = std::cell::Cell::new(0);
    /// 批6 C6c：本层搜索的商店买臂计数 `(买臂总数, 其中授出红钥匙的买臂数)`。
    /// 诊断量（批5 复核用临时打点测过 5716 次买臂 / f12 红 0 次，本指标把它固化成
    /// 可复跑证据）。`solve_layer` 入口清零、memo 命中按缓存回放、`solve_global`
    /// 跨层累加写 `GlobalResult.shop_buys / red_buys`。
    static SHOP_BUYS: std::cell::Cell<(u64, u64)> = std::cell::Cell::new((0, 0));
}

/// R1 层节点上限（计划 §5 R1「离散桶+canonical+**有界可配**+不承诺全局最优」；
/// lead 裁定 A 2026-11-13：默认 200_000、env `SOLVER_LAYER_LIMIT` 可调、
/// **确定性=同 env 同结果**（截断点=节点计数=遍历序确定函数）。层截断⇒该层
/// 结果不完整⇒GlobalResult.confidence 降级 bounded-best/feasible、
/// `truncated_layers` 记 (floor, nodes 实数)——判据③/报审可回答
/// 「结果是否被界污染」）。
fn layer_node_limit() -> usize {
    std::env::var("SOLVER_LAYER_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(200_000)
}

/// 读层截断标记（`solve_global`/`solve_local_exit` 消费；见 LAYER_TRUNC doc）。
/// **只读、不清零**：清零由 `solve_layer_with` 入口负责（每次层求解开始重设）。
pub(crate) fn take_layer_trunc() -> Option<(i32, usize)> {
    LAYER_TRUNC.with(|f| f.get())
}

/// 批6 C6c：本层搜索金币峰值（`solve_global` 聚合消费；见 GOLD_PEAK doc）。
pub(crate) fn take_gold_peak() -> i32 {
    GOLD_PEAK.with(|c| c.get())
}

/// 批6 C6c：本层买臂计数 `(总数, 授红数)`（`solve_global` 聚合消费；见 SHOP_BUYS）。
pub(crate) fn take_shop_buys() -> (u64, u64) {
    SHOP_BUYS.with(|c| c.get())
}

/// C5d″：层规格内容摘要（FNV-1a，`LAYER_MEMO` 键分量）。覆盖 `SolveInput` 全
/// 字段面（difficulty/level 全成员——地形/实体/出口/入场玩家/敌表双份/flags），
/// 内容同⇒缓存共享=正确（同内容同输出）。
fn level_digest(si: &SolveInput) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    {
        let mut mix = |bytes: &[u8]| {
            for &b in bytes {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        };
        mix(&si.difficulty.to_le_bytes());
        let Some(l) = si.level.as_ref() else {
            return h;
        };
        mix(&l.cols.to_le_bytes());
        mix(&l.rows.to_le_bytes());
        for &t in &l.terrain {
            mix(&[t as u8]);
        }
        mix(&l.start.0.to_le_bytes());
        mix(&l.start.1.to_le_bytes());
        for e in &l.entities {
            mix(&e.ty.to_le_bytes());
            mix(&e.x.to_le_bytes());
            mix(&e.y.to_le_bytes());
            mix(&e.param.to_le_bytes());
        }
        for x in &l.exits {
            mix(&x.x.to_le_bytes());
            mix(&x.y.to_le_bytes());
            mix(&[x.kind as u8]);
            mix(&[x.index]);
        }
        let p = &l.player;
        for v in [
            p.hp, p.atk, p.def, p.yellow, p.blue, p.red, p.gold, p.item_stack_size,
        ] {
            mix(&v.to_le_bytes());
        }
        for &v in &p.item_types {
            mix(&[v]);
        }
        for &v in &p.item_uses {
            mix(&[v]);
        }
        for d in l.enemies.iter().chain(l.enemy_base.iter()) {
            mix(&d.id.to_le_bytes());
            mix(&d.hp.to_le_bytes());
            mix(&d.atk.to_le_bytes());
            mix(&d.def.to_le_bytes());
            mix(&d.gold.to_le_bytes());
            mix(&[d.trait_flags]);
        }
        mix(&l.flags.to_le_bytes());
    }
    h
}

/// C5d′：层出口归并（见 `solve_layer` 调用点注）。组键=子态 K₀ 全分量
/// （kind/cross_edge/player/flags/pos/ents）；组内 (kills,steps) Pareto：
/// 已有不劣于新⇒丢新（先到胜=确定性）、新支配旧⇒剔旧、互有优劣⇒并存。
fn merge_layer_goals(goals: Vec<Goal>) -> Vec<Goal> {
    let mut groups: BTreeMap<
        (ExitKind, Option<(i32, bool)>, PlayerSpec, u128, (i32, i32), Vec<EntRec>),
        Vec<Goal>,
    > = BTreeMap::new();
    for g in goals {
        let key = (
            g.kind,
            g.cross_edge,
            g.post.player.clone(),
            g.post.flags,
            g.post.pos,
            g.post.ents.clone(),
        );
        let slot = groups.entry(key).or_default();
        if slot.iter().any(|e| e.kills <= g.kills && e.steps <= g.steps) {
            continue;
        }
        slot.retain(|e| !(g.kills <= e.kills && g.steps <= e.steps));
        slot.push(g);
    }
    groups.into_values().flatten().collect()
}

/// C4 fixture 装载路径：ingest 产物 → 单层 `SolveInput`（提案 §5 C4 / T5 断言面）。
///
/// - 地形：`tile_walkability` 判据（`scene_persist.rs:297-301` 同式 `idx = y*cols*4 + x*2`、
///   `tile_walk[tile]==1`；A 级 oracle FLD 134）；
/// - 实体：该层全部 spawn 记录（含 hidden——type83 场景门需在内，T5）；
/// - 出口：type7|type8 ⇒ `ExitSpec{kind: Stair, index}`（index=出口序号 1-based）；
/// - `floor` 字段 = `floor_idx`（maplv1 ⇒ 1，preset `current_floor` 同号先例）、difficulty=0。
///
/// I/O 归测试/bin 侧（数据已由 `ingest` 解析进内存，本函数纯变换）。
#[allow(clippy::too_many_arguments)]
pub fn load_floor(
    si: &crate::ingest::SolverInput,
    floor_idx: usize,
    start: (i32, i32),
    player: PlayerSpec,
    enemies: Vec<EnemyDef>,
    enemy_base: Vec<EnemyDef>,
    tile_walk: &[i32],
) -> Result<SolveInput, String> {
    let floor = si
        .floors
        .get(floor_idx)
        .ok_or_else(|| format!("floor_idx={floor_idx} 越界（共 {} 层）", si.floors.len()))?;
    let cols = floor.width as i32;
    let rows = floor.height as i32;
    if cols <= 0 || rows <= 0 {
        return Err(format!("floor{floor_idx} 尺寸非法 {cols}x{rows}"));
    }
    // 地形（行主序；索引式与 route_gen.py:54-55 `wk` 同源）：
    // Maplv.terrain=headerless 网格（maplv.rs take(data,4,grid_bytes)，13×13=676B），
    // 行走判定读法=行距 wide·4、列距 2（a.java:6973 rebuildWalkability 内层 +2×wide
    // + 外层 var3<<1；route_gen.py:54-55 `g1[y * wide * 4 + x * 2]`；scene_persist.rs:300 同式）
    let mut terrain = vec![false; (cols * rows) as usize];
    for y in 0..rows {
        for x in 0..cols {
            let idx = (y * cols * 4 + x * 2) as usize;
            let Some(&tile) = floor.terrain.get(idx) else {
                return Err(format!("floor{floor_idx} terrain 缺 ({x},{y}) idx={idx}"));
            };
            terrain[(y * cols + x) as usize] = (tile as usize) < tile_walk.len()
                && tile_walk[tile as usize] == 1;
        }
    }
    // 实体：层 i 区间 = per_floor_records[..i] 前缀和（ingest.rs 平铺约定）
    let rec_off: usize = si.per_floor_records.iter().take(floor_idx).sum();
    let rec_n = si
        .per_floor_records
        .get(floor_idx)
        .copied()
        .unwrap_or(0);
    let recs = si
        .spawns
        .get(rec_off..rec_off + rec_n)
        .ok_or_else(|| format!("floor{floor_idx} spawn 区间越界 off={rec_off} n={rec_n}"))?;
    let mut entities = Vec::with_capacity(recs.len());
    let mut exits = Vec::new();
    let mut exit_no = 0u8;
    for sp in recs {
        let (x, y) = ((sp.record.x >> 5) as i32, (sp.record.y >> 5) as i32);
        entities.push(EntSpec {
            ty: sp.record.type_code as i32,
            x,
            y,
            param: sp.param as i32,
        });
        if matches!(sp.record.type_code, 7 | 8) {
            exit_no = exit_no.saturating_add(1);
            exits.push(ExitSpec {
                x,
                y,
                kind: ExitKind::Stair,
                index: exit_no,
            });
        }
    }
    Ok(SolveInput {
        floor: floor_idx as i32,
        difficulty: 0,
        level: Some(LevelSpec {
            cols,
            rows,
            terrain,
            start,
            entities,
            exits,
            player,
            enemies,
            enemy_base,
            flags: 0,
        }),
    })
}

/// C2 探针：BumpEnemy 转移判定（搜索与本探针**共用 `bump_outcome`**，非自证）。
///
/// `None` = `resolve_bump` 判 Refused（`T(s,a)=⊥`：不产生状态、不计杀，提案 §1.1）；
/// `Some(loss)` = Engage，loss 取自直调 `predict_engagement`（bump 面 `no_counter=true`）。
pub fn probe_bump(input: &SolveInput, ty: i32) -> Option<i32> {
    let level = input.level.as_ref()?;
    let slot = level.entities.iter().position(|e| e.ty == ty)?;
    let player = level.player.to_combat();
    bump_outcome(level, &player, ty, slot)
}

/// BumpEnemy 转移核心（§1.3 直调面）：自备 `BattleCtx` 10 写槽 →
/// `BattleResolver::resolve_bump(BumpEngage)`（Refused 判据在其内部）→ Engage 时
/// 直调 `predict_engagement(..., no_counter=true)` 取损失。
fn bump_outcome(level: &LevelSpec, player: &PlayerCombat, ty: i32, slot: usize) -> Option<i32> {
    let mut battle_active = false;
    let mut battle_anim_tick = 0i32;
    let mut battle_digit_by_type: Vec<i32> = Vec::new();
    let mut battle_enemy_hp = 0i32;
    let mut battle_finishing = false;
    let mut battle_no_counter = false;
    let mut battle_rerun_scene = false;
    let mut battle_target = 0i32;
    let mut battle_tick = 0i32;
    let mut show_battle_digits = false;
    let mut ctx = BattleCtx {
        battle_active: &mut battle_active,
        battle_anim_tick: &mut battle_anim_tick,
        battle_digit_by_type: &mut battle_digit_by_type,
        battle_enemy_hp: &mut battle_enemy_hp,
        battle_finishing: &mut battle_finishing,
        battle_no_counter: &mut battle_no_counter,
        battle_rerun_scene: &mut battle_rerun_scene,
        battle_target: &mut battle_target,
        battle_tick: &mut battle_tick,
        show_battle_digits: &mut show_battle_digits,
        player,
        enemies: &level.enemies,
        enemy_base: &level.enemy_base,
    };
    let mut resolver = BattleResolver;
    match resolver.resolve_bump(
        &mut ctx,
        EnemyTypeId(ty),
        BattleEvent::BumpEngage {
            entity_slot: slot as i32,
        },
    ) {
        BattleOutcome::Engage { .. } => Some(predict_engagement(
            player,
            &level.enemies,
            &level.enemy_base,
            ty,
            true,
        )),
        BattleOutcome::Refused => None,
        // BumpEngage 臂不产 Exchange（`battle.rs:124-133`）
        BattleOutcome::Exchange { .. } => unreachable!("BumpEngage 不进入 Exchange 臂"),
    }
}


// ---------------------------------------------------------------- C6a 实体表（A-lite）
/// 活体实体记录（C6a A-lite 表示统一：引擎 `EntityTable` 活体槽的 canonical 快照）。
/// canonical 序=**引擎槽序**（sprite 记录序=引擎载入序，幸存者按序保位；脚本
/// CES/ADD 新实体按 `spawnEntity` append 序尾追——entity.rs:330-346 `idx = count++`）。
/// **不排序**：评估成稿原拟「ty,x,y,param 字典序」改为引擎槽序——槽序是引擎 append
/// 演化的 canonical，字典序会把 append 压成集合、丢顺序信息（去重面变弱）。
/// 活体谓词=`serialize_floor_state` 逐字 `!removed && state != 1`（entity.rs:86-88）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntRec {
    pub ty: i32,
    pub x: i32,
    pub y: i32,
    pub param: i32,
}

/// sprite 载入序全活列表（`LevelSpec.entities` 序=scene_setup 注入序=引擎 sprite
/// 加载序——即初始 `EntityTable`）。
pub fn sprite_full(level: &LevelSpec) -> Vec<EntRec> {
    level
        .entities
        .iter()
        .map(|e| EntRec { ty: e.ty, x: e.x, y: e.y, param: e.param })
        .collect()
}

/// 从跑完脚本的 harness 场景读回活体实体表（C6a A-lite）：谓词同
/// `serialize_floor_state`（entity.rs:86-88）；param 已由 `build_scene` 按
/// (type,格) 首配回灌 sprite 值、CES 新实体 param=0——与引擎读值同源。
fn readback_ents(sc: &GameScene) -> Vec<EntRec> {
    sc.entities
        .items
        .iter()
        .enumerate()
        .filter(|(i, e)| *i < sc.entities.count && !e.removed && e.state != 1)
        .map(|(_, e)| EntRec { ty: e.type_, x: e.px >> 5, y: e.py >> 5, param: e.param as i32 })
        .collect()
}

/// 击杀掉落：与引擎**同表**（`game_core::rules::boss`），只**生成实体**。
///
/// 语义依据（A 级）：
/// - `a.java:8912-8921`：`spawnBossEvent(idx)` = 逐 4 元组 `spawnEntity(...)` +
///   `markEntityDesRemoved(...)` —— 落的是**实体**（DES 态），不是直接进背包或钥匙计数；
/// - 拾取在引擎里是「踩上去才 pickup」（`applyStepCellEffects` cat2/cat4 →
///   `pickupItemType` + `markEntityRemoved`，a.java:5719-5722）⇒ 本函数**不发钥匙、
///   不写 `player` 背包**；这些掉落在后续 `Cell::Pickup` 转移里由引擎拾取臂入账；
/// - 组号同样只来自 game-core：`event_of`（击杀拍掉落臂）+ `post_victory_event_of`
///   （69@35 的 post-anim 收尾臂，a.java:10144-10149）。本文件**不复制任何表值**，
///   满足「求解器与引擎同表」的硬约束。
///
/// 序：调用方先 `ents.remove(slot)`（boss 出表）再追加掉落实体——与引擎读回序一致：
/// `entity::spawn` 恒取 `table.count` 追加（`crates/game-core/src/entity.rs:330`），
/// 故引擎里掉落槽号必然大于 boss 槽；boss 到胜利动画 `battle_anim_tick==0` 才
/// `entity::remove`（`scene.rs:3709-3711`），活体读回按槽序跳过 removed ⇒ 两个
/// 掉落实体的相对序与「先 69 击杀拍组4、后收尾组7」的 append 序一致。
fn apply_kill_drops(ns: &mut St, ety: i32, floor: i32) {
    let events = [
        game_core::rules::boss::event_of(ety, floor),
        game_core::rules::boss::post_victory_event_of(ety, floor),
    ];
    for ev in events.into_iter().flatten() {
        for s in game_core::rules::boss::event_spawns(ev) {
            ns.ents.push(EntRec {
                ty: s.ty,
                x: s.x,
                y: s.y,
                param: s.extra,
            });
        }
    }
}

// ---------------------------------------------------------------- 搜索态

/// 搜索态（提案 §1 状态 s 的批2 切面：hp/kills 为支配维度，不进去重键）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct St {
    pos: (i32, i32),
    player: PlayerSpec,
    /// 当前层活体实体列表（C6a A-lite：引擎 `EntityTable` 活体槽 canonical
    /// 快照——sprite 序幸存者保位+脚本新增 append 序尾追；层内单调：杀/拾/DES
    /// 压实移除、CES/ADD 尾追加）
    ents: Vec<EntRec>,
    /// `script_line_flags` 位图（C5 脚本门；68 槽 ⇒ u128，初值 `LevelSpec.flags`）
    flags: u128,
    kills: i32,
    steps: i32,
}

/// 去重键 K₀（提案 §4.1：pos, ents_digest, flags, keys, bag, atk, def；
/// hp/kills/steps/gold **不进键**）。
///
/// `Hash` 只服务于 D 面子集支配的「药维桶」（`HashMap<PillKey>`，桶内才是
/// 线性支配扫描；桶本身按内容全等，绝不按哈希相等）——既有走位/去重逻辑仍用
/// 派生 `Ord` 的 `BTreeMap`，键语义零变化。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Key {
    pos: (i32, i32),
    ents: Vec<EntRec>,
    flags: u128,
    keys: (i32, i32, i32),
    bag: (Vec<u8>, Vec<u8>, i32),
    atk: i32,
    def: i32,
}

/// 键 = `st` 的全套精确维度 + **指定 ents**（D 面把药维实体剥掉再算键：
/// `key_of` 是 `ents = st.ents` 的特例，既有语义逐字不变）。
fn key_of_ents(st: &St, ents: Vec<EntRec>) -> Key {
    Key {
        pos: st.pos,
        ents,
        flags: st.flags,
        keys: (st.player.yellow, st.player.blue, st.player.red),
        bag: (
            st.player.item_types.clone(),
            st.player.item_uses.clone(),
            st.player.item_stack_size,
        ),
        atk: st.player.atk,
        def: st.player.def,
    }
}

fn key_of(st: &St) -> Key {
    key_of_ents(st, st.ents.clone())
}

/// 支配判定（§4.1：hp₁≥hp₂ ∧ kills₁≤kills₂ ∧ steps₁≤steps₂ ∧ gold₁≥gold₂
/// 且至少一严格 ⇒ 丢 s₂）。
/// 弱支配（含相等）：同 K₀ 下 e 各指标不劣于 ns 即丢 ns（等价态=丢）。
/// 与 solve_global::push_state 的 weakly_ge 同口径（C3b bounce 环修）。
///
/// 【2026-11-17 F2 修】gold 进支配维度。此前只比 hp/kills/steps：同 K₀（同格、
/// 同实体、同旗标、同钥匙、同背包、同攻防）下「钱多但血少」的态会被「钱少血多」的态
/// 判为受支配而丢弃 —— 而金只能花不会招灾（唯一用途是商店购买），金多在任何续局里
/// 都不劣。8xx 金的红钥匙正是靠金才够得着，丢富态等于把可行解剪成不可行解，
/// 且不产生任何报错（只会表现为「界内无解」）。单调性依据：金只在购买时减少，
/// 不参与战斗结算、不改变任何格子的可通行性。
fn weakly_ge(a: &St, b: &St) -> bool {
    a.player.hp >= b.player.hp
        && a.kills <= b.kills
        && a.steps <= b.steps
        && a.player.gold >= b.player.gold
}

fn dominates(a: &St, b: &St) -> bool {
    let ge_hp = a.player.hp >= b.player.hp;
    let le_kills = a.kills <= b.kills;
    let le_steps = a.steps <= b.steps;
    let ge_gold = a.player.gold >= b.player.gold;
    let strict = a.player.hp > b.player.hp
        || a.kills < b.kills
        || a.steps < b.steps
        || a.player.gold > b.player.gold;
    ge_hp && le_kills && le_steps && ge_gold && strict
}

// ──────────────────────────────────────────────────────────────────────────
// 【D 面】纯加血拾取的**子集支配**（证据与证明：docs/findings/2026-11-19-
// pill-subset-dominance.md）
//
// 问题：f54 的 `ents` 维度唯一差异是「捡了哪几颗药」（ty31 +50 / ty32 +200），
// 而 `Key` 要求 ents **精确相等** ⇒ 同一格上按「已捡药集合」分叉出成千上万个
// 互不比较的键（200k 档 unique_keys=195174，队列去重命中率 0%）。
//
// 本节的规则把键**放宽到「非药实体 + 剩余药集合」**，只在一条可证明的性质上
// 放宽：`s1` 比 `s2` 血多、且 `s1` 还没捡的药是 `s2` 还没捡的药的**超集**
// （⇒ `s1` 已捡的 ⊆ `s2` 已捡的 ⇒ `s2` 沿途还会去捡 `s1` 已经吃掉的东西时，
// 模拟链就断——所以反方向一律不许剪，见 `pill_dominates` 注）。
//
// 健全性依赖的闸门（任一不满足 ⇒ `PillDom::Off(reason)` 退回既有精确键搜索，
// **不静默**：`pill_dom_stat()` 暴露 enabled/reason）：
//   G1 药型经 `certified_pure_hp_gain` 背书（本层增益固定>0、除 hp 外零副作用）；
//   G2 药型不在 `policy.ban`（ban ⇒ 该格 `Blocked`，s1 进不去而 s2 能 Move）；
//   G3 药实体**独格**（同格无其它实体；否则 `classify` 可能改判门/梯/怪/阻挡，
//      或拾取臂连带触发 cat1 臂）；
//   G4 层内无「踩格副作用臂」实体（5/60/61/62/76/83）落在**可达格**上——这些臂
//      会改实体表/跑关卡脚本，无法静态证明两状态触发结果只差一个单调项；
//   G5 掉落不会生成上述危险型、也不会落在药格上（掉落位置固定，见 boss 表）。
// 另有**比较预算**（`PILL_CMP_BUDGET`）：超出即停止咨询（只损失剪枝，不损健全性）。

/// 药型 → 本层固定增益（`certified_pure_hp_gain` 的产物；>0）。
type PillGains = BTreeMap<i32, i32>;

/// 本层「纯加血拾取」的支配计划：开（带身份表）/ 关（带可观测原因）。
enum PillDom {
    On(PillOrd),
    /// fail-closed：`(原因, 触发类型)`——`reason_ty=0` 表示与具体类型无关。
    Off(&'static str, i32),
}

/// 药维实体身份表：`EntRec`（`(ty,x,y,param)` 全等）→ 位号。
///
/// **静态**成立的前提是 G3（独格）+ G4（无实体表变更臂）+ G5（掉落不落药格）：
/// 实体不移动、只减不增 ⇒ 药格永远只有那颗药 ⇒ 「这一位还在不在」就等价于
/// 「这颗药还没被捡」。位宽 32（>32 颗即 fail-closed，不上位图以外的花活）。
struct PillOrd {
    bits: BTreeMap<EntRec, u32>,
    gains: PillGains,
}

impl PillOrd {
    /// 把实体表拆成（非药实体, 剩余药位图）——非药部分进桶键、位图进支配判定。
    fn split(&self, ents: &[EntRec]) -> (Vec<EntRec>, u32) {
        let mut rest = Vec::with_capacity(ents.len());
        let mut bits = 0u32;
        for e in ents {
            match self.bits.get(e) {
                Some(b) => bits |= 1 << *b,
                None => rest.push(e.clone()),
            }
        }
        (rest, bits)
    }
}

/// 子集支配判定：`a` 不劣于 `b`（`b` 可剪）。
///
/// 调用前提=两者**同桶**（`Key` 去掉药维后全等 ⇒ pos/flags/非药实体/三钥/
/// 背包/atk/def 全等）。四条不等式与 `weakly_ge` 同口径：
/// - `hp(a) ≥ hp(b)`：血是终局目标，且血多⇒拒战面更宽（`battle.rs:121-123`
///   `Refused ⟺ loss ≥ hp`；`predict_engagement` 不用 hp ⇒ 同一动作 loss 相同）；
/// - `kills(a) ≤ kills(b)` / `steps(a) ≤ steps(b)`：与 `weakly_ge` 同（`steps` 参与
///   `Cost::HpLoss` 的 tie-break ⇒ 少步确实更优，不能省这条）；
/// - `gold(a) ≥ gold(b)`：与 `weakly_ge` 同（F2 修）。
///
/// 第五条是本规则的全部新意：`rem(a) ⊇ rem(b)`（bitset：`b & !a == 0`）。
/// **方向不能反**：`rem(a) ⊇ rem(b)` 等价于「a 已捡的 ⊆ b 已捡的」——只有这个
/// 方向，b 的任意后续路线在 a 都能照走（沿途每颗药两边同时可捡、净收益相同）。
/// 反方向（a 剩得少但血多）**不成立**：b 还能捡 a 已经吃掉的药，a 不能。
fn pill_dominates(a: &St, a_rem: u32, b: &St, b_rem: u32) -> bool {
    a.player.hp >= b.player.hp
        && a.kills <= b.kills
        && a.steps <= b.steps
        && a.player.gold >= b.player.gold
        && (b_rem & !a_rem) == 0
}

/// 药维桶扫描：本规则的**全部剪枝决策**都在这里（纯函数，便于单测钉方向）。
///
/// `cands` 产出 `(节点 id, 旧态, 旧态剩余药位图)`；`budget` 是比较预算余量
/// （每比较一对扣 1，扣到 0 即停——只损失剪枝、不损健全性）。
/// 返回 `(新态是否被支配, 可剪的旧态 id 列表)`。
fn pill_scan<'a, I>(cands: I, ns: &St, ns_rem: u32, budget: &mut u64) -> (bool, Vec<u32>)
where
    I: IntoIterator<Item = (u32, &'a St, u32)>,
{
    let mut dominated = false;
    let mut kill: Vec<u32> = Vec::new();
    for (eid, e, e_rem) in cands {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        // 方向不可反：旧态血不少且剩余药是超集 ⇒ 新态可剪
        if pill_dominates(e, e_rem, ns, ns_rem) {
            dominated = true;
            break;
        }
        // 新态血不少且剩余药是超集 ⇒ 旧态可剪
        if pill_dominates(ns, ns_rem, e, e_rem) {
            kill.push(eid);
        }
    }
    (dominated, kill)
}

/// 踩格副作用臂实体型（`scene.rs apply_step_cell_effects`）：5 宝箱 / 60-62 陷阱 /
/// 76 组门 / 83 关卡脚本。这些臂能改实体表（60 火焰上爬、76 清组格）、能按 hp 算
/// 伤害、能跑任意脚本 ⇒ 一律 fail-closed（G4）。
pub(crate) const STEP_ARM_HAZARD_TYPES: [i32; 6] = [5, 60, 61, 62, 76, 83];

/// 子集支配比较预算（每层）。超出即停止咨询：只损失剪枝机会、不损健全性；
/// 计数与遍历序确定 ⇒ 结果仍确定（同输入同输出）。
const PILL_CMP_BUDGET: u64 = 4_000_000;

/// 本层子集支配的可观测统计（`pill_dom_stat()`）。**禁静默**：规则没启用时
/// `enabled=false` + `reason`/`reason_ty` 说明卡在哪条闸门。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PillDomStat {
    /// 本层是否启用（`false` ⇒ 退回既有精确键搜索，行为与原版逐字节相同）
    pub enabled: bool,
    /// 被背书的药型数（0 = 本层没有纯加血拾取）
    pub pill_types: usize,
    /// 本层药实体数（进位图的颗数）
    pub pills: usize,
    /// `enabled=false` 的原因；启用时为 `"on"`；未跑过层=`"not_run"`
    pub reason: &'static str,
    /// 触发 fail-closed 的实体/药型；0 = 与具体类型无关
    pub reason_ty: i32,
    /// 进桶的态数（预算耗尽后不再入桶）
    pub states: u64,
    /// 实际比较次数（同桶内 (新态, 旧态) 对）
    pub cmp: u64,
    /// 因本规则被剪的新态数
    pub pruned_new: u64,
    /// 因本规则被剪的旧桶态数
    pub killed_old: u64,
    /// 比较预算是否耗尽（耗尽后本层不再咨询本规则）
    pub budget_exhausted: bool,
}

impl PillDomStat {
    const CLEARED: PillDomStat = PillDomStat {
        enabled: false,
        pill_types: 0,
        pills: 0,
        reason: "not_run",
        reason_ty: 0,
        states: 0,
        cmp: 0,
        pruned_new: 0,
        killed_old: 0,
        budget_exhausted: false,
    };
}

thread_local! {
    /// 本层子集支配统计（thread_local=多线程测试隔离；`solve_layer_with` 入口清零、
    /// memo 命中按缓存回放、`run_root` 末尾写回）。
    static PILL_DOM: std::cell::Cell<PillDomStat> = std::cell::Cell::new(PillDomStat::CLEARED);
}

/// 最近一次层求解的子集支配统计（诊断面；不参与搜索决策）。
///
/// 用途：让「规则没生效」**可见**——`enabled=false` 时 `reason`/`reason_ty` 指出
/// 卡在哪条闸门（例如 `hp_gain_not_certified` / `step_arm_hazard_reachable`）。
pub fn pill_dom_stat() -> PillDomStat {
    PILL_DOM.with(|c| c.get())
}

fn pill_dom_stat_set(s: PillDomStat) {
    PILL_DOM.with(|c| c.set(s));
}

/// 动作（路线重建用）。`Move(x,y)`=**到达格**（C6a：门格节点的 `st.pos` 被
/// 读回改写成脚本走位落点，route token 必须用到达格而非读回 pos——否则
/// 「gate 航点」与 Move token 漂到脚本落点，C3a 断言与 replay 语义双坏）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Act {
    Start,
    Move(i32, i32), // x, y 到达格

    Bump(i32, i32, i32), // type, x, y
    /// 开门走入（route_gen `door x,y` 航点，route_gen.py:14；C4）
    Door(i32, i32), // x, y
    /// P7 撞门（`interact_with_cell` 的 bump 面，`from_walk=false`）：门格 x,y。
    /// **玩家不移动**（bump 面 `allowed=false`）、不计步——与怪物 `Bump` 的
    /// 「撞完进格」不同，故独立变体，避免路线航点误解成到达。
    GateBump(i32, i32), // x, y
}

/// 终态候选（双查询共用；提案 §1.4「一次搜索维护 Pareto 前沿即可双查询取解」）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct Goal {
    hp: i32,
    kills: i32,
    steps: i32,
    /// 终态三色钥匙数（`keys_used` 断言面：入口 − 本值，T4）
    keys: (i32, i32, i32),
    kind: ExitKind,
    route: Vec<String>,
    /// 批3 C1：终态全态（层间传播用——pos/ents 位图/背包/atk,def/flags 继承到下一层）
    post: St,
    /// 批3 C3a/C3b：换层边两元组 `(target, went_down)`——`ExitKind::Script`/`Cloud`/`UseItem`（F3）——原仅 Script/
    /// `Cloud` 臂非 None（Stair 由出口梯实体 param 取参、Ending 非换层）
    cross_edge: Option<(i32, bool)>,
}

/// 取解比较面：私有 `Goal` 与公开 [`LayerGoal`] 的同构投影。
///
/// 为什么要有这一层：`solve_local` 在私有 `Goal` 上取解，而层门面 [`solve_layer`]
/// 交出去的是投影后的 `LayerGoal`。P3 的 [`solve_local_exit`] 必须与既有查询在
/// **同一套 tie-break** 上选解（hp 大 → steps 少 → 路线字典序），否则「同层多解取
/// 血量最大」会随入口路径不同而漂移——漂移即静默改解。一个比较器 + 两个投影，
/// 禁止第二份 `hp/steps/route` 比较。
trait GoalFace {
    fn hp(&self) -> i32;
    fn kills(&self) -> i32;
    fn steps(&self) -> i32;
    fn route(&self) -> &[String];
    fn kind(&self) -> ExitKind;
}

impl GoalFace for Goal {
    fn hp(&self) -> i32 {
        self.hp
    }
    fn kills(&self) -> i32 {
        self.kills
    }
    fn steps(&self) -> i32 {
        self.steps
    }
    fn route(&self) -> &[String] {
        &self.route
    }
    fn kind(&self) -> ExitKind {
        self.kind
    }
}

impl GoalFace for LayerGoal {
    fn hp(&self) -> i32 {
        self.hp
    }
    fn kills(&self) -> i32 {
        self.kills
    }
    fn steps(&self) -> i32 {
        self.steps
    }
    fn route(&self) -> &[String] {
        &self.route
    }
    fn kind(&self) -> ExitKind {
        self.kind
    }
}

/// 代价泛型（§5 C3「同骨架两代价泛型」+ §1.4 双查询独立出解）。
/// 支配剪枝与代价无关 ⇒ 搜索骨架只跑一次，代价只在取解时分叉。
trait Cost {
    /// 量化域（HpLoss：全部出口；Kills：仅 Exit#1 步行楼梯，§1.4 ②）
    fn eligible(kind: ExitKind) -> bool;
    /// 优劣：true = a 优于 b（并列取步数少者、再并列字典序最小路线，§3.3）
    fn better<G: GoalFace>(a: &G, b: &G) -> bool;
}

/// Q1：max_hp（域=全部合法路，含 Exit#2；§1.4 ①）
struct HpLoss;

impl Cost for HpLoss {
    fn eligible(_kind: ExitKind) -> bool {
        true
    }
    fn better<G: GoalFace>(a: &G, b: &G) -> bool {
        if a.hp() != b.hp() {
            a.hp() > b.hp()
        } else if a.steps() != b.steps() {
            a.steps() < b.steps()
        } else {
            a.route() < b.route()
        }
    }
}

/// Q2：min_kills（域=终态 ∈ Exit#1；§1.4 ②）
struct Kills;

impl Cost for Kills {
    fn eligible(kind: ExitKind) -> bool {
        matches!(kind, ExitKind::Stair)
    }
    fn better<G: GoalFace>(a: &G, b: &G) -> bool {
        if a.kills() != b.kills() {
            a.kills() < b.kills()
        } else if a.steps() != b.steps() {
            a.steps() < b.steps()
        } else {
            a.route() < b.route()
        }
    }
}

/// 取解：域内 argbest（None = 该域无解；`reachable` 仍由「任一出口」决定）。
fn pick<C: Cost, G: GoalFace>(goals: &[G]) -> Option<&G> {
    let mut best: Option<&G> = None;
    for g in goals.iter().filter(|g| C::eligible(g.kind())) {
        best = match best {
            None => Some(g),
            Some(b) if C::better(g, b) => Some(g),
            some => some,
        };
    }
    best
}

struct Node {
    st: St,
    parent: u32,
    act: Act,
}

/// 格分类（§1.3 Move 判定=类目+type 分类+地形终审）。
/// 批5 H4：购买效果落地。**只做资源四类**（`AddKeys`/`GiveGold`/`AddHp`/`SwapYellowKey`）。
///
/// ⚠️ `AddStack`/`StatPct`/`PickupItem` **不出臂 = 模型内无此效果边，非世界无此效果**
/// （原版 a.java:7379-7384 有真实现；这是求解器建模缺口，不是考证结论——H4 的教训就是
/// 缺边被读成「无解」）。若三查询卡在这些效果上，按报审列的出臂优先级补。
/// 返回 `false` = 效果无法实施（如 `SwapYellowKey` 黄不足）⇒ 不出买臂。
/// 批6 C6a：**击杀给金**（引擎单源 `crates/game-core/src/scene.rs:2301-2309`
/// `kill_adjacent_at`）：
///
/// ```ignore
/// let gold = if self.player.find_item(25) >= 0 {
///     self.enemy_base[(t - 41) as usize].gold << 1
/// } else {
///     self.enemy_base[(t - 41) as usize].gold
/// };
/// self.player.gold += gold;
/// ```
///
/// - 表源同源：`enemy_base[i]` 的 `id == 41 + i`（`CombatTables::to_records`
///   `combat.rs:24-35`）⇒ `idx = t - 41` 与引擎逐字一致（非另立映射）；
/// - item25 双倍判定走 `PlayerCombat::find_item`（`combat.rs:67-74`，`to_combat()`
///   出来的就是引擎同一实现）⇒ 判定面零自写；
/// - 表缺行/零赏金 ⇒ 0（引擎侧同一行只会给出 0，等价）。
fn kill_bounty(level: &LevelSpec, player: &PlayerCombat, ety: i32) -> i32 {
    // C6d：唯一源在 game-core（引擎 `kill_adjacent_at` 同调此函数），求解器不再自写。
    game_core::combat::kill_reward(&level.enemy_base, ety, player)
}

fn apply_shop_effect(effect: &ShopEffect, p: &mut PlayerSpec) -> bool {
    match effect {
        ShopEffect::AddKeys { blue, yellow, red } => {
            p.blue += blue;
            p.yellow += yellow;
            p.red += red;
            true
        }
        ShopEffect::GiveGold { amount } => {
            p.gold += amount;
            true
        }
        ShopEffect::AddHp { amount } => {
            p.hp += amount;
            true
        }
        ShopEffect::SwapYellowKey { gold } => {
            if p.yellow < 1 {
                return false;
            }
            p.yellow -= 1;
            p.gold += gold;
            true
        }
        ShopEffect::AddStack { .. }
        | ShopEffect::StatPct { .. }
        | ShopEffect::PickupItem { .. } => false, // 见上：不出臂=模型内无此效果边
    }
}

enum Cell {
    Blocked,
    Move,
    Monster(usize, i32), // 槽位, type_id
    /// 黄/红/蓝门（C4：expand 内直调 `consume_key_for_door`；未消耗 ⇒ ⊥）
    Door(usize, i32), // 槽位, type_id
    /// 拾取实体（C6：cat 2|4 道物，`apply_step_cell_effects` `scene.rs:4093-4096`
    /// 拾取+移除同拍——expand 内 harness `try_step` 落格触发，读回 player keys/实体移除）
    Pickup, // cat 2|4 道物（拾取面=该格全部存活 cat2|4，expand 内 harness 直调）
    /// 商人（批5 H4：cat16，`render_category` `77|78=>16`，a.java:6268-6298）。
    /// expand 出**两臂**：买（开价门过 ∧ 买得起 ∧ 效果可实施 ⇒ 扣金施效 + 实体移除
    /// `markEntityRemoved`）/ 走（地形可走 ⇒ 原 Move 臂）——保守取超集，买不过也能过
    /// ⇒ 永不误剪（批0-3 里 shop 走臂=原行为，零回归）。
    Shop(usize, i32), // 槽位, type_id(77|78)
}

/// 到达格的 StepGate 结果（§1.3 StepGate=踩场景门 `from_walk=true`；bump 面
/// by construction 不触发——`interact_with_cell` 用 `try_run_scene(param,false)`）。
enum GateArrive {
    /// 无门实体 / 门未命中（gate=false：flags 置位 / case66 / case67@d0）——正常走格
    Pass,
    /// 门命中且脚本换层终止边（harness 读 `persist.pending_floor`≠0，§2.2 ④）。
    /// 批3 C3a 线程化两元组：`pending`=目标层（swap_floor 的 pending_floor）、
    /// `went_down`=落点链入参（scene_persist.rs:410 find_floor_gate 方向）
    Terminal {
        pending: i32,
        went_down: bool,
    },
    /// 门命中但非终态完成或驱动停摆 ⇒ ⊥（C5 口径：批2 夹具不触发，报审注明；
    /// T6 对 floor1 reachable=true 兜底——若真触发会即刻暴露）
    Dropped,
    /// 批3 C2 终局边（通关）：floor50 击败型72 的 bump 后继——游戏到此结束
    /// （Ending 非换层、非 StepGate，由 expand 的怪臂直接产出，不经 gate_arrive）。
    Ending,
    /// S5 段分治：`boss_target` 命中的 bump 杀后继（同 Ending 口径：记候选不入队、
    /// 非换层非终局；仅 `solve_layer_boss_shops` 链路触发）。
    BossKill,
}

/// 批3 F3：消耗一枚栈物品（`itemUseCounts=1` 语义 a.java:944/7770-7789——
/// uses>1 递减、否则从 types/uses 双数组弹栈；types 有而 uses 缺=按1处理）。
fn consume_item(p: &mut PlayerSpec, item: u8) {
    if let Some(i) = p.item_types.iter().position(|&t| t == item) {
        if p.item_uses.get(i).copied().unwrap_or(1) > 1 {
            if let Some(u) = p.item_uses.get_mut(i) {
                *u -= 1;
            }
        } else {
            p.item_types.remove(i);
            if i < p.item_uses.len() {
                p.item_uses.remove(i);
            }
        }
    }
}




/// 批4 C4-a：贪心零决策闭包的**杀怪档**（survey §4-1 三查询谓词参数化）。
/// 零伤怪（`predict_engagement` 损失=0）杀之不掉血，但**加 kills** ⇒ 三查询语义不同：
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum KillMode {
    /// 绝不自动杀（Q2 最少杀怪：旁路零伤怪收了=白加 kills，任务书点名禁）
    Never,
    /// 全收：可达域内所有零伤怪（Q1 血量最大 / Q3 不拿终极装备——两者都不以杀怪为代价）
    All,
    /// 只收「挡路必杀」的（Q2 的放行档：该怪是通往被门禁目标的必经通路）
    Blocking,
}

/// 批4 C4-a：贪心零决策闭包谓词（survey §4-1；magic-tower-AI `eatItem` 改造版）。
/// 闭包只决定「哪些转移是**确定性的**」——拾取/伤害仍由 `pickup_step`/`bump_outcome`
/// 真引擎判定（方案 B：❌不自算战斗、❌不第二套模型）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ClosurePolicy {
    /// 自动拾取可达道具（Q1/Q2/Q3 皆 true；false=闭包关闭拾取臂）
    pub pickup: bool,
    /// 零伤怪档（见 [`KillMode`]）
    pub kill_zero: KillMode,
    /// 永不自动拾取的道具类型（Q3=`Q3_EQUIP_SET` 36,40；禁令同时把该格判 `Blocked`——
    /// 引擎「踩格即拾」⇒ 不拿就不能踩，否则 Q3 可靠路过偷拿终极装备）
    pub ban: &'static [i32],
}

impl ClosurePolicy {
    /// 关（批0-3 既有测试与回归判据的默认面：零行为变化）
    pub const OFF: Self = Self {
        pickup: false,
        kill_zero: KillMode::Never,
        ban: &[],
    };
    /// Q1 剩余血量最大：零伤怪与道具全收（免费的都要）
    pub fn q1() -> Self {
        Self {
            pickup: true,
            kill_zero: KillMode::All,
            ban: &[],
        }
    }
    /// Q2 最少杀怪：道具照收，**零伤怪只收挡路必杀的**（旁路绝不自动收）
    pub fn q2() -> Self {
        Self {
            pickup: true,
            kill_zero: KillMode::Blocking,
            ban: &[],
        }
    }
    /// Q3 不拿终极装备能否通关：同 Q1，但排除被禁道具
    pub fn q3() -> Self {
        Self {
            pickup: true,
            kill_zero: KillMode::All,
            ban: crate::state::Q3_EQUIP_SET,
        }
    }
}

/// 批4 C4-a：闭包候选（`closure_step` 取最小键者）。
enum ClosureCand {
    /// 可达域内的拾取格
    Pick((i32, i32)),
    /// 邻接可达域的零伤怪（格, 槽, ty）
    Kill((i32, i32), usize, i32),
}

struct Search<'a> {
    input: &'a SolveInput,
    level: &'a LevelSpec,
    /// 批3 C3b：筋斗云换层锚点——层内**最后 spawn** 的 type7/type8 格
    /// （entity.rs:361/363 覆写序「最后 spawn 胜」；a.java marker>>5 转格同源）。
    stair_up: Option<(i32, i32)>,
    stair_down: Option<(i32, i32)>,
    /// 批3 C3b：是否记录筋斗云候选。`solve_layer`（批3 层间路径）置 true；
    /// 批2 `solve_local`/`run()` 保持 false——批2 裁定「UseItem 裁出⇒无筋斗云
    /// 换层边」零回归。
    cloud: bool,
    /// 批4 C4-a：贪心零决策闭包谓词（默认 `OFF`=批0-3 零行为变化）
    policy: ClosurePolicy,
    /// 批5 H4：可用商品表（`shop_goods.json` 解析结果）。旧入口恒 `&[]`
    /// ⇒ 批0-3 零行为变化；只有 `query` 路径传真表（lead 2026-11-16 认可的线程方案）。
    shops: &'a [ShopGoodDef],
    /// 【批8】hp 可采纳上界剪枝开关。`false`（默认）⇒ 全段不执行 ⇒ 零行为差异；
    /// 两个**求解入口**（`solve_local`、`solve_layer_with`）各自显式置 true。
    hp_prune: bool,
    /// 【批8】本次求解是否仍要「最优 kills」：`solve_local` 是（双查询各自出解），
    /// 层出口不是（只有 hp 目标）。为 true 时剪枝额外要求 kills 已劣于最优 Stair 目标。
    kill_objective: bool,
    /// S5 段分治：撞杀目标 boss ty（Some(ty) ⇒ 该 ty 的 bump 杀后记
    /// `ExitKind::BossKill` 候选不入队；None=默认链路零变化）。
    boss_target: Option<i32>,
}

/// 【批8】hp 上界剪枝的「已知最优」跟踪（只由真实 Goal 更新）。
///
/// `hp` 只增不减（取 max）；`kills_stair` 取所有已记录 Stair 目标的 kills 最小值
/// （`i32::MAX`=尚无 Stair 目标）。两者都由**已记录的真实 Goal** 更新，
/// 不存在「假想最优」。
#[derive(Debug, Clone, Copy)]
struct BestHp {
    /// 已见最优 out_hp；`None` = 尚无任何终态 ⇒ 一律不剪。
    hp: Option<i32>,
    /// 已见最优 Stair 目标的 kills；`i32::MAX` = 尚无 Stair 目标。
    kills_stair: i32,
}

impl BestHp {
    fn note(&mut self, g: &Goal) {
        if g.kind == ExitKind::Stair {
            self.kills_stair = self.kills_stair.min(g.kills);
        }
        self.hp = Some(match self.hp {
            Some(h) => h.max(g.hp),
            None => g.hp,
        });
    }

    /// 剪枝判据。`bound < best_hp`（**严格**）且（`kills_needed` 时）`kills` 已严格
    /// 劣于最优 Stair 目标。
    ///
    /// 健全性：`kills` 沿路径单调不减 ⇒ 后者保证该枝不可能产出更少击杀的 Stair
    /// 目标；`bound` 在**当前 68 槽语料**下与真最优同解 ⇒ 实测不产出更高 hp。
    /// 两条前提任一不成立就不剪。
    ///
    /// 【2026-11-19 更正（§3.6）】原文写「`bound` 是可采纳上界」——该**一般主张已
    /// 证伪**（f35 杀 ty69 ⇒ post-anim 组 7 生成 ty32×3 = 2400，杀后按 0 计 ⇒ 低估）。
    /// 现行口径：语料实测同解 + `hp_bound::plan` 的 `post_kill_hp_spawn` /
    /// `script_hp_spawn` / `shop_repeatable` 闸门兜住已知通道；不是逐态证明。
    fn prunable(
        &self,
        ord: &crate::hp_bound::HpBoundOrd,
        hp: i32,
        ents: &[EntRec],
        flags: u128,
        kills: i32,
        kills_needed: bool,
    ) -> bool {
        let Some(bh) = self.hp else {
            return false;
        };
        if kills_needed && !(self.kills_stair != i32::MAX && kills > self.kills_stair) {
            return false;
        }
        crate::hp_bound::should_prune(ord, bh, hp, ents, flags)
    }
}

impl<'a> Search<'a> {
    fn new(input: &'a SolveInput, level: &'a LevelSpec) -> Self {
        Self::from_entry(input, level, level.start)
    }

    /// 批3 C1：从任意入场格起搜（层间边：入场梯落点≠`level.start`）。
    fn from_entry(input: &'a SolveInput, level: &'a LevelSpec, _entry: (i32, i32)) -> Self {
        Search {
            input,
            level,
            stair_up: level
                .entities
                .iter()
                .rev()
                .find(|e| e.ty == 7)
                .map(|e| (e.x, e.y)),
            stair_down: level
                .entities
                .iter()
                .rev()
                .find(|e| e.ty == 8)
                .map(|e| (e.x, e.y)),
            cloud: false,
            policy: ClosurePolicy::OFF,
            shops: &[],
            hp_prune: false,
            kill_objective: false,
            boss_target: None,
        }
    }

    fn walkable(&self, x: i32, y: i32) -> bool {
        self.level.terrain[(y * self.level.cols + x) as usize]
    }

    /// 格分类（`scene.rs interact_with_cell` 的类目臂归约，提案 §6-7 单源：
    /// `entity_category_with_floor` 层覆写表；地形终审=楼梯豁免 `stair_count>0`）。
    /// 批5 H4：开价门 + 商品定位（`applyMerchantOffer` 语义）。
    /// `src = type==77 ? 0 : 1`（a.java:7320）、`idx = param & 0xff`、
    /// **开价门 `(param>>>8) & 2 != 0`**（a.java:7326 / case16 分派 a.java:5495）；
    /// 表内无对应行 ⇒ 无货 ⇒ 不出买臂（`var7<=0 → false` 同款，a.java:7333）。
    /// 注：`shops` 为 `&[]`（批0-3/旧入口）⇒ 恒 `None` ⇒ 只剩走臂 ⇒ 零行为变化。
    fn shop_offer(&self, st: &St, ty: i32, slot: usize) -> Option<(i32, ShopEffect)> {
        let p = st.ents[slot].param;
        if (p >> 8) & 2 == 0 {
            return None; // 未开价（`var7` 位2）——引擎不弹购买面
        }
        let src = if ty == 77 { 0 } else { 1 };
        let idx = p & 0xff;
        let g = self.shops.iter().find(|g| g.src == src && g.idx == idx)?;
        Some((g.price, g.effect.clone()))
    }

    fn classify(&self, st: &St, x: i32, y: i32) -> Cell {
        let mut monster: Option<(usize, i32)> = None;
        let mut door: Option<(usize, i32)> = None;
        let mut pickup = false;
        let mut pickup_ty: Option<i32> = None;
        let mut shop: Option<(usize, i32)> = None;
        let mut blocked = false;
        let mut stair_like = false;
        for (slot, e) in st.ents.iter().enumerate() {
            if e.x != x || e.y != y {
                continue;
            }
            match entity_category_with_floor(e.ty, self.input.floor) {
                8 => {
                    if monster.is_none() {
                        monster = Some((slot, e.ty));
                    }
                }
                1 => match e.ty {
                    // 门（C4 起可开：expand 内直调 consume_key_for_door，无钥 ⇒ ⊥）
                    1 | 2 | 3 => {
                        if door.is_none() {
                            door = Some((slot, e.ty));
                        }
                    }
                    // 炼丹台/真火/墙/撞击墙（C6f：81 移出——见下）
                    9 | 10 | 11 | 12 => blocked = true,
                    // 楼梯族（bump 显形/楼梯计数豁免地形终审）
                    6 | 7 | 8 => stair_like = true,
                    // 5 宝箱 / 83 场景门 / 76|82 组门：bump 放行
                    // 81 封印门：spawn 即隐形（sprite_spawn.rs:31 m_122 置
                    // visible=false；全仓仅 op_swd 改 visible 且同帧 type→4），
                    // 引擎 interact「仅可见时拦截、隐形期放行」（scene.rs
                    // 81 臂注「floor10 南北隐形封印门可穿，迭代65 差分实证」）
                    // ⇒ 生产恒放行，分类器不再挡（C6f 红绿实证层10 上行梯）。
                    _ => {}
                },
                // 2|4 道物（C6 拾取转移：harness try_step 落格触发 pickup_item_type）
                2 | 4 => {
                    pickup = true;
                    if pickup_ty.is_none() {
                        pickup_ty = Some(e.ty);
                    }
                }
                // 32 联动门（type72/84/87）：interact 面恒阻挡（a.java:5527 /
                // scene.rs:3664 `32 => allowed = false`）。floor1 上 type72 无层覆写
                // （sprite_defs floor_category_override 仅 floor50→cat8，
                // evidence a.java:6292 render_category=32、a.java:9681 default_category=32）
                // ⇒ 撞前不可达。0/16 装饰：可走
                32 => blocked = true,
                // 批5 H4：商人（cat16）——`77|78=>16`；与门同为「交互才产生转移」的格
                16 => {
                    if shop.is_none() {
                        shop = Some((slot, e.ty));
                    }
                }
                _ => {}
            }
        }
        // 地形终审（`scene.rs` 末段：stair_count>0 豁免）
        if !(self.walkable(x, y) || stair_like) {
            return Cell::Blocked;
        }
        if blocked {
            return Cell::Blocked;
        }
        // 门在可走地形上才产生开门转移（真层门格地形均可走；非可走地形=不入格不耗钥的保守近似）
        if let Some((slot, ty)) = door {
            return Cell::Door(slot, ty);
        }
        if let Some((slot, ty)) = shop {
            return Cell::Shop(slot, ty);
        }
        // 怪/拾取并存（批2 夹具无）：战斗优先——打赢后落格才拾取（引擎 interact→apply 时序）
        match (monster, pickup) {
            (Some((slot, ty)), _) => Cell::Monster(slot, ty),
            (None, true) => {
                // 批4 C4-a Q3：被禁道具**踩格即拾**（scene.rs pickup_item_type 落格触发）
                // ⇒ 不拿就不能踩 ⇒ 分类为 Blocked（`ClosurePolicy::OFF` 时 ban 空 ⇒ 恒 Pickup，
                // 批0-3 零行为变化）。
                if self.policy.ban.contains(&pickup_ty.unwrap_or(i32::MIN)) {
                    Cell::Blocked
                } else {
                    Cell::Pickup
                }
            }
            (None, false) => Cell::Move,
        }
    }

    /// 批4 C4-a 闭包用连通性：门/墙恒挡（**开门与杀怪都是决策**），怪格按 `open`
    /// 槽集放行；出口格照常可走——本函数问的是「杀哪些怪才通」，故不设出口边界。
    fn reach(&self, st: &St, start: (i32, i32), open: &BTreeSet<usize>) -> BTreeSet<(i32, i32)> {
        let mut seen: BTreeSet<(i32, i32)> = BTreeSet::new();
        seen.insert(start);
        let mut q: VecDeque<(i32, i32)> = VecDeque::new();
        q.push_back(start);
        while let Some(p) = q.pop_front() {
            for &(dx, dy) in &DIRS {
                let n = (p.0 + dx, p.1 + dy);
                if n.0 < 0 || n.1 < 0 || n.0 >= self.level.cols || n.1 >= self.level.rows {
                    continue;
                }
                if seen.contains(&n) {
                    continue;
                }
                let ok = match self.classify(st, n.0, n.1) {
                    Cell::Move | Cell::Pickup | Cell::Shop(..) => true,
                    Cell::Monster(slot, _) => open.contains(&slot),
                    _ => false,
                };
                if ok {
                    seen.insert(n);
                    q.push_back(n);
                }
            }
        }
        seen
    }

    /// 批4 C4-a Q2「挡路必杀」判定：只收**既充分又必要**的零伤怪——
    /// 充分=只开它就够走到某个被门禁目标；必要=关着它（其余全开）就走不到。
    /// 旁路怪两条都不满足 ⇒ 绝不自动收（C4-a 判据钉①）。
    /// 快路径：无被门禁目标（出口/门/道具在无杀可达域内）⇒ 直接空集，0 次 BFS。
    fn blocking_kill_slots(
        &self,
        st: &St,
        dist: &BTreeMap<(i32, i32), i32>,
        cands: &[((i32, i32, i32, i32), ClosureCand)],
    ) -> BTreeSet<usize> {
        // 目标=出口格 ∪ 门格 ∪ 允许拾取的道具格（被禁道具不是目标）
        let mut targets: BTreeSet<(i32, i32)> = self.level.exits.iter().map(|e| (e.x, e.y)).collect();
        for e in &st.ents {
            let t = (e.x, e.y);
            match entity_category_with_floor(e.ty, self.input.floor) {
                1 if matches!(e.ty, 1 | 2 | 3) => {
                    targets.insert(t);
                }
                2 | 4 if !self.policy.ban.contains(&e.ty) => {
                    targets.insert(t);
                }
                _ => {}
            }
        }
        // 被门禁=不在无杀可达域 R0 且无邻格在 R0（出口/门/道具格本身不入 R0）
        let gated: Vec<(i32, i32)> = targets
            .into_iter()
            .filter(|&t| {
                !dist.contains_key(&t)
                    && !DIRS
                        .iter()
                        .any(|&(dx, dy)| dist.contains_key(&(t.0 + dx, t.1 + dy)))
            })
            .collect();
        if gated.is_empty() {
            return BTreeSet::new();
        }
        // 全怪开仍走不到 ⇒ 瓶颈是门/墙不是怪 ⇒ 谁都不挡路
        let all: BTreeSet<usize> = (0..st.ents.len()).collect();
        let r_all = self.reach(st, st.pos, &all);
        if !gated.iter().any(|&t| r_all.contains(&t)) {
            return BTreeSet::new();
        }
        let mut keep = BTreeSet::new();
        for (_, c) in cands {
            let slot = match c {
                ClosureCand::Kill(_, s, _) => *s,
                _ => continue,
            };
            let mut only = BTreeSet::new();
            only.insert(slot);
            let r_only = self.reach(st, st.pos, &only);
            if !gated.iter().any(|&t| r_only.contains(&t)) {
                continue; // 不充分（只开它不够）
            }
            let mut minus = BTreeSet::new();
            for s in 0..st.ents.len() {
                if s != slot {
                    minus.insert(s);
                }
            }
            let r_minus = self.reach(st, st.pos, &minus);
            if gated.iter().any(|&t| r_minus.contains(&t)) {
                continue; // 不必要（别的怪也能通）
            }
            keep.insert(slot);
        }
        keep
    }

    /// 批4 C4-a 贪心零决策闭包（survey §4-1；magic-tower-AI `eatItem` 改造版）。
    ///
    /// 把「可达即捡的道具 / 零伤可秒的怪」确定性收到不动点，使其**不构成搜索分支**
    /// （闭包步=唯一后继 ⇒ 分支数不增；收完返回 `None` ⇒ 回到原四臂）。
    /// 谓词按查询参数化（[`ClosurePolicy`]）：Q1 全收 / Q2 只收挡路必杀 / Q3 排除被禁道具。
    ///
    /// 合规（方案 B）：闭包只决定「哪些转移是**确定性的**」——拾取走 `pickup_step`、
    /// 伤害走 `bump_outcome`（=`resolve_bump`+`predict_engagement`），❌不自算战斗。
    fn closure_step(&self, st: &St) -> Option<(Act, St, GateArrive)> {
        let pol = self.policy;
        if !pol.pickup && pol.kill_zero == KillMode::Never {
            return None;
        }
        // ① 可达域 R（BFS 父链）：只经 Move|Pickup（门/怪/墙停）；
        //    出口格设边界不走上去——踩出口=决策，不是闭包。
        let mut dist: BTreeMap<(i32, i32), i32> = BTreeMap::new();
        let mut from: BTreeMap<(i32, i32), (i32, i32)> = BTreeMap::new();
        let mut pickups: Vec<(i32, i32)> = Vec::new();
        let mut q: VecDeque<(i32, i32)> = VecDeque::new();
        dist.insert(st.pos, 0);
        q.push_back(st.pos);
        while let Some(p) = q.pop_front() {
            let d = dist[&p];
            for &(dx, dy) in &DIRS {
                let n = (p.0 + dx, p.1 + dy);
                if n.0 < 0 || n.1 < 0 || n.0 >= self.level.cols || n.1 >= self.level.rows {
                    continue;
                }
                if dist.contains_key(&n) || self.level.exits.iter().any(|e| (e.x, e.y) == n) {
                    continue;
                }
                match self.classify(st, n.0, n.1) {
                    Cell::Move | Cell::Shop(..) => {}
                    Cell::Pickup => pickups.push(n),
                    _ => continue,
                }
                dist.insert(n, d + 1);
                from.insert(n, p);
                q.push_back(n);
            }
        }
        // ② 候选：键=(距离, y, x, kind)——确定性取最小（禁 HashMap 序）
        let mut best: Option<((i32, i32, i32, i32), ClosureCand)> = None;
        let put = |k: (i32, i32, i32, i32),
                       c: ClosureCand,
                       b: &mut Option<((i32, i32, i32, i32), ClosureCand)>| {
            if b.as_ref().map_or(true, |(bk, _)| k < *bk) {
                *b = Some((k, c));
            }
        };
        if pol.pickup {
            for &c in &pickups {
                put((dist[&c], c.1, c.0, 0), ClosureCand::Pick(c), &mut best);
            }
        }
        if pol.kill_zero != KillMode::Never {
            let pc = st.player.to_combat();
            let mut cands: Vec<((i32, i32, i32, i32), ClosureCand)> = Vec::new();
            for (slot, e) in st.ents.iter().enumerate() {
                if entity_category_with_floor(e.ty, self.input.floor) != 8 {
                    continue;
                }
                let m = (e.x, e.y);
                let mut bd = i32::MAX;
                for &(dx, dy) in &DIRS {
                    if let Some(&d) = dist.get(&(m.0 + dx, m.1 + dy)) {
                        bd = bd.min(d + 1);
                    }
                }
                if bd == i32::MAX {
                    continue; // 与 R 不邻接（杀不到）
                }
                if bump_outcome(self.level, &pc, e.ty, slot) != Some(0) {
                    continue; // 有伤怪=搜索分支，闭包不碰（且 Q1 也不该白挨打）
                }
                cands.push(((bd, m.1, m.0, 1), ClosureCand::Kill(m, slot, e.ty)));
            }
            if pol.kill_zero == KillMode::Blocking {
                let keep = self.blocking_kill_slots(st, &dist, &cands);
                for (k, c) in cands {
                    let slot = match &c {
                        ClosureCand::Kill(_, s, _) => *s,
                        _ => continue,
                    };
                    if keep.contains(&slot) {
                        put(k, c, &mut best);
                    }
                }
            } else {
                for (k, c) in cands {
                    put(k, c, &mut best);
                }
            }
        }
        // ③ 走一步（或已在邻格 ⇒ 直接 bump/pickup）
        let (_, cand) = best?;
        let first_step = |t: (i32, i32)| -> Option<(i32, i32)> {
            let mut cur = t;
            loop {
                match from.get(&cur) {
                    Some(&p) if p == st.pos => return Some(cur),
                    Some(&p) => cur = p,
                    None => return None,
                }
            }
        };
        match cand {
            ClosureCand::Pick(t) => {
                let s = first_step(t)?;
                if s == t {
                    let mut ns = self.pickup_step(st, s.0, s.1)?;
                    let g = self.gate_arrive(&mut ns, s.0, s.1);
                    return Some((Act::Move(s.0, s.1), ns, g));
                }
                let mut ns = st.clone();
                ns.pos = s;
                ns.steps += 1;
                let g = self.gate_arrive(&mut ns, s.0, s.1);
                Some((Act::Move(s.0, s.1), ns, g))
            }
            ClosureCand::Kill(m, slot, ety) => {
                // 邻格=dist 中距 m 最小者
                let mut adj: Option<(i32, i32)> = None;
                let mut ad = i32::MAX;
                for &(dx, dy) in &DIRS {
                    let n = (m.0 + dx, m.1 + dy);
                    if let Some(&d) = dist.get(&n) {
                        if d < ad {
                            ad = d;
                            adj = Some(n);
                        }
                    }
                }
                let adj = adj?;
                if adj != st.pos {
                    let s = first_step(adj)?;
                    let mut ns = st.clone();
                    ns.pos = s;
                    ns.steps += 1;
                    let g = self.gate_arrive(&mut ns, s.0, s.1);
                    return Some((Act::Move(s.0, s.1), ns, g));
                }
                let pc = st.player.to_combat();
                let loss = bump_outcome(self.level, &pc, ety, slot)?;
                if loss != 0 {
                    return None; // 非零伤：不是闭包步（防御；候选侧已滤）
                }
                let mut ns = st.clone();
                ns.player.hp -= loss;
                ns.kills += 1;
                // 批6 C6a：击杀给金（引擎 scene.rs:2301-2309 同源；含 item25 双倍）
                let reward = kill_bounty(self.level, &pc, ety);
                game_core::combat::gain_gold_to(&mut ns.player.gold, reward);
                ns.ents.remove(slot);
                // 掉落（同表 game-core；只生成实体，见 apply_kill_drops）
                apply_kill_drops(&mut ns, ety, self.input.floor);
                ns.pos = m;
                ns.steps += 1;
                let gate = if self.input.floor == 50 && ety == 72 {
                    GateArrive::Ending
                } else {
                    GateArrive::Pass
                };
                Some((Act::Bump(ety, m.0, m.1), ns, gate))
            }
        }
    }

    /// 后继生成（solver 自算=搜索编排白名单；转移本体全为 §1.3 直调）。
    /// 第三元=到达格 StepGate 结果（C5；仅 Move/Door 到达面——踩格触发）。
    /// 批6 C6c：金币峰值采样包装（`expand_inner` 前加一层——不改任何臂的生成逻辑，
    /// 只把「入口态 + 全部生成态」的 `player.gold` 汇入 `GOLD_PEAK`）。
    fn expand(&self, st: &St, out: &mut Vec<(Act, St, GateArrive)>) {
        self.expand_inner(st, out);
        let mut m = st.player.gold;
        for (_, ns, _) in out.iter() {
            if ns.player.gold > m {
                m = ns.player.gold;
            }
        }
        GOLD_PEAK.with(|c| {
            if m > c.get() {
                c.set(m);
            }
        });
    }

    fn expand_inner(&self, st: &St, out: &mut Vec<(Act, St, GateArrive)>) {
        // 批4 C4-a：闭包步是**唯一**后继（确定性 ⇒ 不增分支）；收完 None ⇒ 原四臂。
        // 闭包步走普通 Node（parent/act 入链）⇒ `route_of` 天然含闭包移动、
        // dedup/Pareto 照常、幂等（已收则 `closure_step` 返回 None）。
        if let Some(s) = self.closure_step(st) {
            out.push(s);
            return;
        }
        for &(dx, dy) in &DIRS {
            let (tx, ty) = (st.pos.0 + dx, st.pos.1 + dy);
            if tx < 0 || ty < 0 || tx >= self.level.cols || ty >= self.level.rows {
                continue;
            }
            // C5d：不再按起点连通分量剪枝——走步邻接天然限分量（对纯走步是
            // 空操作），而门脚本 MOV_0 传送落点可跨分量（静态过滤会拦死）。
            //
            // P7 撞门臂：先于 `classify`——门格同时也可能是走格/墙格，撞门与二者
            // 互不排斥（撞门**不移动**）。只有「撞才触发、走上去不触发」的门才生成
            // 后继（`gate_bump` 头注：走能触发的门被走步臂支配）⇒ 无 bump-only 门的
            // 层零新后继、输出逐字段不变。
            if let Some(ns) = self.gate_bump(st, tx, ty) {
                out.push((Act::GateBump(tx, ty), ns, GateArrive::Pass));
            }
            match self.classify(st, tx, ty) {
                Cell::Blocked => {}
                Cell::Move => {
                    let mut ns = st.clone();
                    ns.pos = (tx, ty);
                    ns.steps += 1;
                    let g = self.gate_arrive(&mut ns, tx, ty);
                    out.push((Act::Move(tx, ty), ns, g));
                }
                // 批5 H4：商人两臂——① 走（=原 Move 行为，买不过也能过 ⇒ 永不误剪，
                // 批0-3 零回归）② 买（花金得效 + 实体移除 `markEntityRemoved` 同款）
                Cell::Shop(slot, ety) => {
                    if self.walkable(tx, ty) {
                        let mut ns = st.clone();
                        ns.pos = (tx, ty);
                        ns.steps += 1;
                        let g = self.gate_arrive(&mut ns, tx, ty);
                        out.push((Act::Move(tx, ty), ns, g));
                    }
                    if let Some((price, effect)) = self.shop_offer(st, ety, slot) {
                        if st.player.gold >= price {
                            let mut ns = st.clone();
                            ns.player.gold -= price;
                            if apply_shop_effect(&effect, &mut ns.player) {
                                // 批6 C6c：买臂统计（授红=AddKeys.red>0）
                                let red = match effect {
                                    ShopEffect::AddKeys { red, .. } => red > 0,
                                    _ => false,
                                };
                                SHOP_BUYS.with(|c| {
                                    let (n, r) = c.get();
                                    c.set((n + 1, r + if red { 1 } else { 0 }));
                                });
                                ns.ents.remove(slot);
                                ns.pos = (tx, ty);
                                ns.steps += 1;
                                let g = self.gate_arrive(&mut ns, tx, ty);
                                out.push((Act::Move(tx, ty), ns, g));
                            }
                        }
                    }
                }
                Cell::Monster(slot, ety) => {
                    let player = st.player.to_combat();
                    // Refused ⇒ ⊥（不产生状态、不计杀，提案 §1.1）
                    if let Some(loss) = bump_outcome(self.level, &player, ety, slot) {
                        let mut ns = st.clone();
                        ns.player.hp -= loss;
                        ns.kills += 1;
                        // 批6 C6a：击杀给金（引擎 scene.rs:2301-2309 同源；
                        // item25 双倍经 to_combat 的 find_item 判定）
                        let reward = kill_bounty(self.level, &player, ety);
                        game_core::combat::gain_gold_to(&mut ns.player.gold, reward);
                        ns.ents.remove(slot);
                        // 掉落（同表 game-core；只生成实体，见 apply_kill_drops）
                        apply_kill_drops(&mut ns, ety, self.input.floor);
                        // 引擎真值分叉（S5 复核 F1 子项）：ty75 在 floor<=50 撞杀会
                        // `battle_rerun_scene=true` ⇒ 不 restep ⇒ 玩家不移位、不计步。
                        if !ty75_no_restep(self.input.floor, ety, ns.flags) {
                            ns.pos = (tx, ty);
                            ns.steps += 1;
                        }
                        // 批3 C2 终局边：floor50 击败型72 → Ending（钩子
                        // scene.rs:3891-3894 → load_level_script(65) → END_0；
                        // floor50 cat8 覆写=sprite_defs floor50→cat8 ⇒ 可 bump，
                        // 其它层型72=cat32 恒阻挡进不了本臂）。语义=游戏结束
                        // ⇒ 记候选不入队（run_root 的 Ending 臂，同 Terminal 口径）。
                        // 怪格不建模 StepGate（batch2 夹具无共存；报审注明）
                        let gate = if self.input.floor == 50 && ety == 72 {
                            GateArrive::Ending
                        } else if self.boss_target == Some(ety) {
                            // S5 段分治：撞杀锚 boss = 段末目标达成（候选侧处理同
                            // Ending 臂口径；默认链路 boss_target=None 零变化）。
                            GateArrive::BossKill
                        } else {
                            GateArrive::Pass
                        };
                        out.push((Act::Bump(ety, tx, ty), ns, gate));
                    }
                }
                Cell::Door(slot, dty) => {
                    // 开门转移：直调 consume_key_for_door（combat.rs:170，§1.3 开门行，
                    // ❌不自写「有钥减一」）；门映射同 scene.rs:3397-3399（1黄/2红/_蓝）。
                    // 未消耗 ⇒ ⊥（无钥匙阻挡，§1.1 不产生状态）。
                    let door = match dty {
                        1 => game_core::combat::door_code::YELLOW,
                        2 => game_core::combat::door_code::RED,
                        _ => game_core::combat::door_code::BLUE,
                    };
                    let mut pc = st.player.to_combat();
                    let (consumed, _) = game_core::combat::consume_key_for_door(&mut pc, door);
                    if consumed {
                        let mut ns = st.clone();
                        ns.player = PlayerSpec::from_combat(pc);
                        ns.ents.remove(slot);
                        ns.pos = (tx, ty);
                        ns.steps += 1;
                        // floor1 (6,9)=门+83 同格：开门走入即踩格 ⇒ 同一到达面上
                        // 再判 StepGate（引擎 bump 开门→落格踩格触发，同拍语义）
                        let g = self.gate_arrive(&mut ns, tx, ty);
                        out.push((Act::Door(tx, ty), ns, g));
                    }
                }
                Cell::Pickup => {
                    // 拾取转移（C6，§1.3 拾取行字面路径）：harness pub `try_step` 落格
                    // → tick 推进 32px 归格 → `apply_step_cell_effects`（scene.rs:4093-4096
                    // `pickup_item_type`+`entity::remove` 同拍）→ 读回 player 全字段。
                    // 引擎时序直调；❌不自写物品效果表、❌不直调私有 pickup_item_type。
                    if let Some(mut ns) = self.pickup_step(st, tx, ty) {
                        let g = self.gate_arrive(&mut ns, tx, ty);
                        // 航点=普通格 `x,y`（route_gen 语法：走过去；Act::Move 序列化同）
                        out.push((Act::Move(tx, ty), ns, g));
                    }
                }
            }
        }
    }

    /// 【D 面】构造本层子集支配计划（闸门 G0-G5，任一不满足 ⇒ `Off(reason, ty)`）。
    ///
    /// `cert` 是**注入点**：生产路径传 `game_core::effect_registry::
    /// certified_pure_hp_gain`（真跑一次拾取臂的证书），测试可注入「伪造随 tier 变
    /// 的增益」把 fail-closed 钉在闸门上。
    fn plan_pill(&self, root: &St, cert: &dyn Fn(i32, i32) -> Option<i32>) -> PillDom {
        let floor = self.input.floor;
        // 层静态实体表（`EntSpec` → `EntRec`，与 `sprite_full` 同口径）
        let static_ents: Vec<EntRec> = self
            .level
            .entities
            .iter()
            .map(|e| EntRec {
                ty: e.ty,
                x: e.x,
                y: e.y,
                param: e.param,
            })
            .collect();
        // 药型候选：入场活体 ∪ 层静态实体里「拾取类目(cat 2|4) 且 pickup 面 kind=HpUp」。
        // 非 HpUp 的其它拾取物（栈增/钥匙/属性/装备）**不进药维**：它们照旧按精确
        // 实体比较（保守但零风险——规则只在可证的那一类上放宽）。
        let mut gains: PillGains = BTreeMap::new();
        for e in root.ents.iter().chain(static_ents.iter()) {
            let cat = entity_category_with_floor(e.ty, floor);
            if cat != 2 && cat != 4 {
                continue;
            }
            if game_core::effect_registry::pickup_kind_for_type(e.ty) != "HpUp" {
                continue;
            }
            if gains.contains_key(&e.ty) {
                continue;
            }
            // G1：证书——本层增益固定 >0 且除 hp 外零副作用
            match cert(e.ty, floor) {
                Some(g) if g > 0 => {
                    gains.insert(e.ty, g);
                }
                _ => return PillDom::Off("hp_gain_not_certified", e.ty),
            }
        }
        if gains.is_empty() {
            // G0：本层没有纯加血拾取 ⇒ 没得可剪（不是失败，但也不启用）
            return PillDom::Off("no_certified_pill", 0);
        }
        // G2：`ban` 里的拾取型会被判 Blocked ⇒ s1 进不去而 s2（药已不在）能 Move
        for ty in gains.keys() {
            if self.policy.ban.contains(ty) {
                return PillDom::Off("pill_banned_by_policy", *ty);
            }
        }
        // G4：踩格副作用臂型必须落在**不可达格**（`classify=Blocked`）
        for e in static_ents.iter().chain(root.ents.iter()) {
            if !STEP_ARM_HAZARD_TYPES.contains(&e.ty) {
                continue;
            }
            if !matches!(self.classify(root, e.x, e.y), Cell::Blocked) {
                return PillDom::Off("step_arm_hazard_reachable", e.ty);
            }
        }
        // G5：掉落不得生成危险型，也不得生成药型（同型同格会造出「同 (ty,x,y,
        // param) 的第二颗药」——位图表达不了重数，静态身份即失效）
        for e in static_ents.iter() {
            if entity_category_with_floor(e.ty, floor) != 8 {
                continue;
            }
            let events = [
                game_core::rules::boss::event_of(e.ty, floor),
                game_core::rules::boss::post_victory_event_of(e.ty, floor),
            ];
            for ev in events.into_iter().flatten() {
                for sp in game_core::rules::boss::event_spawns(ev) {
                    if STEP_ARM_HAZARD_TYPES.contains(&sp.ty) || gains.contains_key(&sp.ty) {
                        return PillDom::Off("drop_breaks_pill_identity", sp.ty);
                    }
                }
            }
        }
        // G3：药实体**独格**（层静态 ∪ 入场活体里同格只有它自己）+ 位宽
        let pill_ents: Vec<EntRec> = root
            .ents
            .iter()
            .filter(|e| gains.contains_key(&e.ty))
            .cloned()
            .collect();
        if pill_ents.len() > 32 {
            return PillDom::Off("too_many_pills", pill_ents.len() as i32);
        }
        // 并集按 `EntRec` 去重（同一颗药在「层静态表」与「入场活体」里各出现一次，
        // 那不是共享格；只有**不同**实体占同格才算）。
        let mut union: Vec<EntRec> = root.ents.clone();
        for e in static_ents.iter() {
            if !union.contains(e) {
                union.push(e.clone());
            }
        }
        for p in &pill_ents {
            let n = union.iter().filter(|e| e.x == p.x && e.y == p.y).count();
            if n != 1 {
                return PillDom::Off("pill_cell_shared", p.ty);
            }
        }
        // 身份表（位号=入场实体序；确定性，与哈希/遍历序无关）
        let mut bits = BTreeMap::new();
        for (i, p) in pill_ents.iter().enumerate() {
            bits.insert(p.clone(), i as u32);
        }
        PillDom::On(PillOrd { bits, gains })
    }

    /// 搜索主循环：BFS（FIFO）+ K₀ 去重 + Pareto 支配剪枝。
    /// 返回（任一出口可达, 全部非支配终态候选——双查询取解共用，§1.4）。
    fn run(&self) -> (bool, Vec<Goal>) {
        self.run_root(
            self.level.start,
            sprite_full(self.level),
            self.level.player.clone(),
            self.level.flags,
            layer_node_limit(),
        )
    }

    /// `run_root` 的生产转发：证书=game-core 的 `certified_pure_hp_gain`
    /// （无第二真值源；求解器侧不复制任何 31/32 的数值）。
    fn run_root(
        &self,
        entry: (i32, i32),
        root_ents: Vec<EntRec>,
        player: PlayerSpec,
        flags: u128,
        node_limit: usize,
    ) -> (bool, Vec<Goal>) {
        self.run_root_with(
            entry,
            root_ents,
            player,
            flags,
            node_limit,
            &|ty, floor| game_core::effect_registry::certified_pure_hp_gain(ty, floor),
        )
    }

    /// 批3 C1：任意入场格 + 初始存活位图起搜（层间边传播：入场格=上一层出场梯的
    /// 落点、活体列表=上一层终态——批2 `run` 的 `start`/sprite 全活特例化）。
    /// 批3 C2：玩家/flags 也由调用方注入（层间携带上一层 post；批2 的
    /// `level.player`/`level.flags` 特例化为 `run` 的两参转发）。
    /// `node_limit`：层节点上限由调用方**读一次**传入（原读 `layer_node_limit()`）——
    /// 与 `LAYER_MEMO` 键同源，杜绝「键说 10、界限用 200000」的竞态。
    /// `cert`：D 面药型证书（生产=game-core，测试=可注入伪造件）。
    fn run_root_with(
        &self,
        entry: (i32, i32),
        root_ents: Vec<EntRec>,
        player: PlayerSpec,
        flags: u128,
        node_limit: usize,
        cert: &dyn Fn(i32, i32) -> Option<i32>,
    ) -> (bool, Vec<Goal>) {
        // 【P7 撞门/A4】层界入口清 memo：两张门缓存都是**纯加速**（键完备 ⇒ 值确定），
        // 跨层清空只会丢复用、不会改变任何可达集；好处是内存有界、层间不串味
        // （旧实现全仓无 clear，长链会一直涨）。
        GATE_MEMO.with(|m| m.borrow_mut().clear());
        BUMP_MEMO.with(|m| m.borrow_mut().clear());
        let root = St {
            pos: entry,
            player,
            ents: root_ents,
            // T9(b) 重入分支：根态必须继承注入 flags（probe_gate 同源；批2 版读
            // `level.flags`——批3 起由调用方传入，层间边携带上一层终态）；
            // 硬编码 0 会让 flags[19] 置位夹具照样触发 ⇒ 假 reachable。
            flags,
            kills: 0,
            steps: 0,
        };
        let mut nodes = vec![Node {
            st: root,
            parent: u32::MAX,
            act: Act::Start,
        }];
        let mut pruned = vec![false];
        let mut queue: VecDeque<u32> = VecDeque::new();
        queue.push_back(0);
        let mut by_key: BTreeMap<Key, Vec<u32>> = BTreeMap::new();
        by_key.entry(key_of(&nodes[0].st)).or_default().push(0);
        // 【D 面】纯加血拾取的子集支配：计划（闸门 G0-G5）+ 药维桶 + 统计。
        // 计划 `Off` ⇒ 全段逻辑不执行（零行为差异，纯 fail-closed）。
        let plan = self.plan_pill(&nodes[0].st, cert);
        let mut pstat = PillDomStat {
            enabled: matches!(plan, PillDom::On(_)),
            pill_types: match &plan {
                PillDom::On(o) => o.gains.len(),
                PillDom::Off(_, _) => 0,
            },
            pills: match &plan {
                PillDom::On(o) => o.bits.len(),
                PillDom::Off(_, _) => 0,
            },
            reason: match &plan {
                PillDom::On(_) => "on",
                PillDom::Off(r, _) => r,
            },
            reason_ty: match &plan {
                PillDom::On(_) => 0,
                PillDom::Off(_, t) => *t,
            },
            states: 0,
            cmp: 0,
            pruned_new: 0,
            killed_old: 0,
            budget_exhausted: false,
        };
        // 药维桶：键=剥掉药实体的 `Key`，值=(节点 id, 剩余药位图)。
        let mut by_pill: HashMap<Key, Vec<(u32, u32)>> = HashMap::new();
        if let PillDom::On(ord) = &plan {
            let (rest, rem) = ord.split(&nodes[0].st.ents);
            let pk = key_of_ents(&nodes[0].st, rest);
            by_pill.entry(pk).or_default().push((0, rem));
            pstat.states = 1;
        }
        let mut goals: Vec<Goal> = Vec::new();
        // 【批8】hp 可采纳上界（fail-closed）：计划 `Off` ⇒ 以下逻辑全不执行。
        // 根态上界同期写回 `hp_bound_stat()`（诊断：root_bound 与真实 opt 的差）。
        let hplan = if self.hp_prune {
            crate::hp_bound::plan_and_stat(
                self.input.floor,
                self.input.difficulty,
                self.level,
                nodes[0].st.player.hp,
                &nodes[0].st.ents,
                nodes[0].st.flags,
                self.shops,
            )
        } else {
            crate::hp_bound::HpBoundPlan::Off("prune_not_enabled", 0)
        };
        let mut best = BestHp {
            hp: None,
            kills_stair: i32::MAX,
        };
        let (mut hpop, mut hentry): (u64, u64) = (0, 0);
        // 批3 C3b：根态云候选（真数据重入场携15 的回程通道；玩具根态通常不邻锚）
        self.record_cloud(&mut goals, &nodes, 0);
        // 批3 F3：根态 UseItem 符候选（真数据 S0 无符；测试/特殊起始携符时生效）
        self.record_useitem(&mut goals, &nodes, 0);

        while let Some(id) = queue.pop_front() {
            if pruned[id as usize] {
                continue;
            }
            if nodes.len() >= node_limit {
                // R1 层界（lead 裁定 A）：触界即停，截断标记带 (floor, nodes 实数)
                LAYER_TRUNC.with(|f| f.set(Some((self.input.floor, nodes.len()))));
                break;
            }
            let st = nodes[id as usize].st.clone();
            if let Some(e) = self
                .level
                .exits
                .iter()
                .find(|e| e.x == st.pos.0 && e.y == st.pos.1)
            {
                // 终态：记录候选（kind 决定量化域归属，§1.4 ② Exit#1），不扩展
                goals.push(Goal {
                    hp: st.player.hp,
                    kills: st.kills,
                    steps: st.steps,
                    keys: (st.player.yellow, st.player.blue, st.player.red),
                    kind: e.kind,
                    route: route_of(&nodes, id),
                    post: st.clone(),
                    cross_edge: None,
                });
                // 【批8】出口候选参与 best 跟踪（只影响剪枝强度，不改变候选集）。
                if let Some(g) = goals.last() {
                    best.note(g);
                }
                if id != 0 {
                    // 批3 C6g：根态恰在出口（step_off 四邻皆墙留梯格等）记录后仍须
                    // 扩展——否则层搜索饿死（f10 entry(1,11) 实测 goals=1）；
                    // 中途到达出口维持批2 语义=记目标不扩展。
                    continue;
                }
            }
            // 【批8】pop 侧剪枝：已入队的态可能被**后来**的更优解超过（顺序无关的
            // 安全性要求此处再查一次）。放在出口记 goal 之后 ⇒ 出口本身永不被剪。
            if id != 0 {
                if let crate::hp_bound::HpBoundPlan::On(ord) = &hplan {
                    if best.prunable(
                        ord,
                        st.player.hp,
                        &st.ents,
                        st.flags,
                        st.kills,
                        self.kill_objective,
                    ) {
                        pruned[id as usize] = true;
                        hpop += 1;
                        continue;
                    }
                }
            }
            let mut succ = Vec::new();
            self.expand(&st, &mut succ);
            for (act, ns, gate) in succ {
                match gate {
                    // 门命中但非终态/停摆 ⇒ ⊥（不产生状态，§1.1；C5 口径见 GateArrive）
                    GateArrive::Dropped => continue,
                    GateArrive::Terminal {
                        pending,
                        went_down,
                    } => {
                        // 脚本换层终止边：记录候选节点但**不入队不扩展**——本层搜索
                        // 到此为止（§2.2 ④ 不执行 swap_floor、不展开目标层续跑=BR-1）。
                        let new_id = nodes.len() as u32;
                        nodes.push(Node {
                            st: ns,
                            parent: id,
                            act,
                        });
                        pruned.push(false);
                        let n = &nodes[new_id as usize];
                        let mut route = route_of(&nodes, new_id);
                        // gate 航点（route_gen 无此 token=提案缺口，报审注明）。
                        // C6a：坐标取 act 到达格（st.pos 已被读回改写成脚本落点）
                        let (gx, gy) = match n.act {
                            Act::Move(x, y) | Act::Door(x, y) => (x, y),
                            _ => n.st.pos,
                        };
                        route.push(format!("gate {gx},{gy}"));
                        goals.push(Goal {
                            hp: n.st.player.hp,
                            kills: n.st.kills,
                            steps: n.st.steps,
                            keys: (n.st.player.yellow, n.st.player.blue, n.st.player.red),
                            kind: ExitKind::Script,
                            route,
                            post: n.st.clone(),
                            cross_edge: Some((pending, went_down)),
                        });
                        if let Some(g) = goals.last() {
                            best.note(g);
                        }
                        continue;
                    }
                    GateArrive::Ending => {
                        // 批3 C2 终局边（通关）：记录候选节点但**不入队不扩展**——
                        // 路线尾已是 `bump 72@x,y`（route_of Bump 臂），无需 gate 航点。
                        let new_id = nodes.len() as u32;
                        nodes.push(Node {
                            st: ns,
                            parent: id,
                            act,
                        });
                        pruned.push(false);
                        let n = &nodes[new_id as usize];
                        goals.push(Goal {
                            hp: n.st.player.hp,
                            kills: n.st.kills,
                            steps: n.st.steps,
                            keys: (n.st.player.yellow, n.st.player.blue, n.st.player.red),
                            kind: ExitKind::Ending,
                            route: route_of(&nodes, new_id),
                            post: n.st.clone(),
                            cross_edge: None,
                        });
                        if let Some(g) = goals.last() {
                            best.note(g);
                        }
                        continue;
                    }
                    GateArrive::BossKill => {
                        // S5 段分治锚击杀候选：记录候选节点但**不入队不扩展**（同
                        // Ending 臂口径；段末任务已达成，锚后无续跑语义）。route
                        // 尾=既有 Bump token（`bump ty@x,y`），可被
                        // replay_route_expect 直接回放。
                        let new_id = nodes.len() as u32;
                        nodes.push(Node {
                            st: ns,
                            parent: id,
                            act,
                        });
                        pruned.push(false);
                        let n = &nodes[new_id as usize];
                        goals.push(Goal {
                            hp: n.st.player.hp,
                            kills: n.st.kills,
                            steps: n.st.steps,
                            keys: (n.st.player.yellow, n.st.player.blue, n.st.player.red),
                            kind: ExitKind::BossKill,
                            route: route_of(&nodes, new_id),
                            post: n.st.clone(),
                            cross_edge: None,
                        });
                        if let Some(g) = goals.last() {
                            best.note(g);
                        }
                        continue;
                    }
                    GateArrive::Pass => {}
                }
                // 批3 C6a：出口到达即记 goal（批2 的 pop 侧记 goal 前移到达侧，
                // 与 Terminal/Ending 臂同构）——出口节点**绕过去重直记**：等长
                // 双解必须全记（t3 字典序 tie-break 的原材料，去重丢一支即断言
                // 打脸，实测过）；非出口等价态走下方 weakly_ge 丢=终止性
                // （DES 挂起实测 87k 节点/160 键）。出口节点不入队不扩展 ⇒
                // 重复到达有界（前驱已去重）。
                if let Some(e) = self
                    .level
                    .exits
                    .iter()
                    .find(|e| e.x == ns.pos.0 && e.y == ns.pos.1)
                {
                    let new_id = nodes.len() as u32;
                    nodes.push(Node {
                        st: ns,
                        parent: id,
                        act,
                    });
                    pruned.push(false);
                    let route = route_of(&nodes, new_id);
                    let n = &nodes[new_id as usize];
                    goals.push(Goal {
                        hp: n.st.player.hp,
                        kills: n.st.kills,
                        steps: n.st.steps,
                        keys: (n.st.player.yellow, n.st.player.blue, n.st.player.red),
                        kind: e.kind,
                        route,
                        post: n.st.clone(),
                        cross_edge: None,
                    });
                    if let Some(g) = goals.last() {
                        best.note(g);
                    }
                    // C3b/F3 候选口径与批2 入队态一致（出口态此前入队=有记录点）
                    self.record_cloud(&mut goals, &nodes, new_id);
                    self.record_useitem(&mut goals, &nodes, new_id);
                    continue;
                }
                // 【批8】入队侧剪枝：`bound < best_hp`（严格）才剪；目标 hp 相等的态
                // 必须留着（HpLoss 的 steps/route tie-break 依赖它们）。
                if let crate::hp_bound::HpBoundPlan::On(ord) = &hplan {
                    if best.prunable(
                        ord,
                        ns.player.hp,
                        &ns.ents,
                        ns.flags,
                        ns.kills,
                        self.kill_objective,
                    ) {
                        hentry += 1;
                        continue;
                    }
                }
                let k = key_of(&ns);
                let list = by_key.entry(k).or_default();
                let mut dominated = false;
                let mut kill: Vec<u32> = Vec::new();
                for &eid in list.iter() {
                    let e = &nodes[eid as usize].st;
                    // C6a 修复（batch2 潜伏缺陷，C6a Completed 多世界探索暴露）：
                    // 原 strict dominates 漏「全指标相等」态 ⇒ 同键等价态反复入队
                    // （等长替代路径每格皆有）⇒ 队列单调涨永不收敛（DES 测试
                    // 87k 节点/160 键实测）。改弱支配含相等=丢（镜像 solve_global
                    // push_state 的 weakly_ge 修，bounce 等价环同族）；e 非严格更优
                    // 即丢 ns，相等保留先到者=确定性。kill 腿 strict 不动。
                    if weakly_ge(e, &ns) {
                        dominated = true;
                        break;
                    }
                    if dominates(&ns, e) {
                        kill.push(eid);
                    }
                }
                if !dominated {
                    list.retain(|x| !kill.contains(x));
                    for &eid in &kill {
                        pruned[eid as usize] = true;
                    }
                }
                // 【D 面】子集支配（只在精确键块**之后**：精确键更细，先让它处理
                // 同药集的分叉；这里处理「同一格、药集不同」的那些态）。
                // 健全性前提=同桶 ⇒ pos/非药实体/flags/三钥/背包/atk/def 全等。
                let mut ns_rem = 0u32;
                let mut pk: Option<Key> = None;
                let mut d_dominated = false;
                if let PillDom::On(ord) = &plan {
                    if !dominated && !pstat.budget_exhausted {
                        let (rest, rem) = ord.split(&ns.ents);
                        ns_rem = rem;
                        let key = key_of_ents(&ns, rest);
                        let bucket = by_pill.entry(key.clone()).or_default();
                        let mut budget = PILL_CMP_BUDGET.saturating_sub(pstat.cmp);
                        let chosen: Vec<(u32, &St, u32)> = bucket
                            .iter()
                            .filter(|&&(eid, _)| !pruned[eid as usize])
                            .map(|&(eid, e_rem)| (eid, &nodes[eid as usize].st, e_rem))
                            .collect();
                        let (dom, dkill) = pill_scan(chosen, &ns, ns_rem, &mut budget);
                        d_dominated = dom;
                        pstat.cmp = PILL_CMP_BUDGET.saturating_sub(budget);
                        if budget == 0 {
                            pstat.budget_exhausted = true;
                        }
                        if !dkill.is_empty() {
                            bucket.retain(|(eid, _)| !dkill.contains(eid));
                            for &eid in &dkill {
                                pruned[eid as usize] = true;
                                pstat.killed_old += 1;
                            }
                        }
                        pk = Some(key);
                    }
                }
                if d_dominated {
                    pstat.pruned_new += 1;
                }
                if dominated || d_dominated {
                    continue;
                }
                let new_id = nodes.len() as u32;
                nodes.push(Node {
                    st: ns,
                    parent: id,
                    act,
                });
                pruned.push(false);
                list.push(new_id);
                queue.push_back(new_id);
                // 【D 面】入队态进药维桶（其后才能杀掉更差的旧态）；
                // 预算耗尽后不再入桶（省内存/耗时，只损失剪枝）。
                if let (Some(pk), false) = (pk, pstat.budget_exhausted) {
                    by_pill.entry(pk).or_default().push((new_id, ns_rem));
                    pstat.states += 1;
                }
                // 批3 C3b：过支配实际入队才记云候选（被支配/重复态不记——
                // 同 K₀ 优态已记过其云边）
                self.record_cloud(&mut goals, &nodes, new_id);
                // 批3 F3：过支配入队态才记 UseItem 候选（同上口径）
                self.record_useitem(&mut goals, &nodes, new_id);
            }
        }

        // C3：reachable = 任一出口可达；取解（max_hp/min_kills）由 Cost 泛型在
        // goals 上分叉（§1.4 双查询独立出解）。
        // 【D 面】统计写回（`pill_dom_stat()` 可观测：没启用时 reason/reason_ty
        // 指出卡在哪条闸门——禁静默）。
        pill_dom_stat_set(pstat);
        // 【批8】剪枝计数写回（禁静默：pruned_entry/pruned_pop 可观测）。
        if self.hp_prune {
            crate::hp_bound::stat_add_commit(hpop, hentry);
        }
        let reachable = !goals.is_empty();
        (reachable, goals)
    }

    /// 批3 C3b：筋斗云换层候选（四段链 a.java:5738-5753 tutorialSeen→f_bool_13
    /// →键49/55（a.java:5295-5306）→findFloorGateEntity（a.java:7115-7133 首个
    /// `param>>9==0` 的 type7/8）→changeFloor(..., strict=true)（a.java:7135-7178））。
    /// tutorialSeen≡背包含15（**背包权威源**）；写点实测 3 处 a.java:7539 pickup
    /// case15 置 true / a.java:9299 读档先置 false / a.java:9306 读档按
    /// itemStackTypes==15 回推 true——【2026-11-15 更正·实测】原「唯一写点」句
    /// 证伪（deobf reference/src/deobf/a.java grep tutorialSeen 三命中）；f_bool_13=
    /// 与锚点四邻**非同格**（a.java:5767-5806，站梯格上=false）。**区间门不在层内**
    /// ——min/maxFloorReached 是全局史，由 solve_global 落闸。
    fn cloud_goals(&self, st: &St) -> Vec<(i32, bool)> {
        let mut out: Vec<(i32, bool)> = Vec::new();
        if !self.cloud || !st.player.item_types.contains(&15) {
            return out;
        }
        let adj = |a: Option<(i32, i32)>| match a {
            Some((ax, ay)) => {
                let (px, py) = st.pos;
                (ax - px).abs() + (ay - py).abs() == 1
            }
            None => false,
        };
        if !(adj(self.stair_up) || adj(self.stair_down)) {
            return out;
        }
        // 按键序固定输出（确定性）：49（type7 → went_down=false）、55（type8 → true）
        for (ty, went_down) in [(7, false), (8, true)] {
            if let Some(e) = self
                .level
                .entities
                .iter()
                .find(|e| e.ty == ty && (e.param >> 9) == 0)
            {
                out.push((e.param & 0xff, went_down));
            }
        }
        out
    }

    /// 批3 C3b：把已入队态的筋斗云候选记为终止目标（与 Terminal 臂同构：不入队
    /// 不扩展、route=route_of、post=起跳前全态；steps=态步数=起跳不耗步）。
    fn record_cloud(&self, goals: &mut Vec<Goal>, nodes: &[Node], id: u32) {
        let st = &nodes[id as usize].st;
        for (target, went_down) in self.cloud_goals(st) {
            goals.push(Goal {
                hp: st.player.hp,
                kills: st.kills,
                steps: st.steps,
                keys: (st.player.yellow, st.player.blue, st.player.red),
                kind: ExitKind::Cloud,
                route: route_of(nodes, id),
                post: st.clone(),
                cross_edge: Some((target, went_down)),
            });
        }
    }

    /// 批3 F3：把已入队态的 UseItem 符21/22 候选记为换层目标（a.java case21/22
    /// =`changeFloor(currentFloor±1, went_down, false)`，var3=false 无区间门；
    /// 边界 `var1<0 || >floorCountMax` 拒 ⇒ 目标层缺规格由 solve_global 丢边）。
    /// 起跳不耗步、位置不变；post=扣符后的全态（`itemUseCounts=1` a.java:944
    /// ⇒ 成功即耗，stack uses>1 递减、否则弹栈 a.java:7770-7789）。
    fn record_useitem(&self, goals: &mut Vec<Goal>, nodes: &[Node], id: u32) {
        let st = &nodes[id as usize].st;
        for (item, delta, went_down) in [(21u8, 1i32, false), (22u8, -1, true)] {
            if !st.player.item_types.contains(&item) {
                continue;
            }
            let target = self.input.floor + delta;
            let mut post = st.clone();
            consume_item(&mut post.player, item);
            goals.push(Goal {
                hp: st.player.hp,
                kills: st.kills,
                steps: st.steps,
                keys: (st.player.yellow, st.player.blue, st.player.red),
                kind: ExitKind::UseItem,
                route: route_of(nodes, id),
                post,
                cross_edge: Some((target, went_down)),
            });
        }
    }

    /// StepGate 到达转移（§1.3 StepGate 行；踩格 `from_walk=true`，bump 面
    /// by construction 不触发）：扫到达格存活 type83 → 直调 `try_run_scene(param,true)`
    /// 判门 → 命中 `load_level_script` → 有拍驱动（`tickdrive`）→ `persist.pending_floor`
    /// 判终止边（§2.2 ④）。无门/未命中 ⇒ Pass（不建 harness）。
    fn gate_arrive(&self, st: &mut St, tx: i32, ty: i32) -> GateArrive {
        let mut gates: Vec<i32> = Vec::new();
        // C6a：st.ents=引擎活体槽序 ⇒ 门优先级与引擎载入序一致
        for e in st.ents.iter() {
            if e.x == tx && e.y == ty && e.ty == 83 {
                gates.push(e.param);
            }
        }
        if gates.is_empty() {
            return GateArrive::Pass;
        }
        // 槽位升序=夹具序（提案 §3.3）；首个命中者载入执行（apply_step_cell_effects
        // 同拍内先判先得；floor1 (6,9) 同格 19+66 —— 66 恒 false ⇒ 19 必判）
        // 批3 C5b：场景惰性构建——memo 全命中时零 build（build_scene 是逐次调用
        // 的主要成本之一，实测 floor51 单层 46s 分段定位得出）。
        let mut sc_opt: Option<GameScene> = None;
        for param in gates {
            // 批3 C5b：memo 命中=复用安全门结局（免 build 免 drive）。
            // C6a：键+入场 ents（DES 结果依赖 pre-entities）、值=pos/ents 全套
            let key = (
                self.input.floor,
                param,
                st.player.clone(),
                st.flags,
                st.ents.clone(),
            );
            if let Some((p, f, pos, ents)) = GATE_MEMO.with(|m| m.borrow().get(&key).cloned())
            {
                st.player = p;
                st.flags = f;
                st.pos = pos; // pos=引擎真值（无玩家走位脚本时=到达格，恒等）
                st.ents = ents;
                return GateArrive::Pass;
            }
            let sc = sc_opt.get_or_insert_with(|| build_scene(self.input, st, (tx, ty)));
            if sc.try_run_scene(param, true) {
                sc.load_level_script(param);
                return match tickdrive(sc) {
                    Drive::Pending(pending) => {
                        // 批3 C3a 终态回读（镜像 pickup_step 读回面：player 全字段
                        // + flags 位图；实体位图不同步=已知边界，报审注明+批5 兜底）
                        readback_script(st, &sc);
                        GateArrive::Terminal {
                            pending,
                            went_down: sc.persist.went_down,
                        }
                    }
                    Drive::Completed => {
                        // 批3 C5a：演出门续走（引擎语义 A 级：脚本跑完回 Map、
                        // 玩家留格可继续走）。C6b（A-lite，lead 批准 2026-11-13）：
                        // **分类器退役**——C6a 读回已把引擎真值（player+flags+pos+
                        // ents 四件套）导入层间态，脚本世界变（怪物 CES/实体 MOV/
                        // 拆障…）不再是幻影路径，任何 Completed 脚本均可续走；
                        // 原选项 D 白名单（script_cross_allowed）整删。停摆
                        // Stalled→Dropped 保守不变（结局未知）。
                        readback_script(st, &sc);
                        GATE_MEMO.with(|m| {
                            m.borrow_mut().insert(
                                key,
                                (
                                    st.player.clone(),
                                    st.flags,
                                    st.pos,
                                    st.ents.clone(),
                                ),
                            );
                        });
                        GateArrive::Pass
                    }
                    Drive::Stalled => {
                        GateArrive::Dropped
                    },
                };
            }
            // 判门未开（flags[idx] 已置）⇒ 脚本未执行 ⇒ 结局恒等（与脚本内容无关，
            // 危险门同理可缓存——没跑就不会改 ents/pos）
            GATE_MEMO.with(|m| {
                m.borrow_mut().insert(
                    key,
                    (st.player.clone(), st.flags, st.pos, st.ents.clone()),
                );
            });
        }
        GateArrive::Pass
    }

    /// 【P7 撞门】门候选：目标格上 `ty83` 且「**撞能触发 ∧ 走上去不触发**」的槽。
    ///
    /// 只挑这一类是**语义**决定，不是剪枝（见 [`Self::gate_bump`] 头注）。
    ///
    /// **不变量（精确版，含定义域）**：*在**可走格**上*，`walk 能开 ⇒ 撞门臂被走臂支配*。
    /// 依据：走上目标格本身就跑同一个脚本并停在原地（`interact_with_cell` 命中即
    /// `allowed=false`，scene.rs:3379-3393）。
    ///
    /// **定义域之外（P7 复核黄 F3 反例，勿当成全域真命题）**：门格**不可走**时，
    /// 走臂根本产生不了后继（`classify` ⇒ `Cell::Blocked`），于是
    /// `bump=1 ∧ walk=1` 的「两条臂都行」门会被本函数按 `walk` 排除 ⇒ **两臂都没有**。
    /// 现网两例：f41 `param=13 @(10,2)`、f48 `param=38 @(4,1)`（都是 `gate.rs` 直通臂，
    /// 格被 ty83 自身阻挡）。今日**零进度损失**——两条脚本都是 `CES_` 陷阱
    /// （槽 13=`CES_6_10_2 CES_60_10_2`、槽 38=`CES_6_4_1 CES_61_4_1`：只在原地生成
    /// 怪/火，无 hp/钥匙/层间效果），且门格不可达，少这两臂不改变可达集。
    /// 判据：`tests/gate_bump.rs::both_arm_gate_on_nonwalkable_cell_is_outside_the_dominance_invariant`。
    /// **若将来补臂**，正确改法是 `if !walk || !self.walkable(tx, ty)`——但那会改
    /// f41/f48 的状态空间，属行为变更，不在本片（A 片零可观测差异）范围内。
    ///
    /// 另一类：撞才能触发的门（`gate.rs` 的 `!from_walk` 专用臂：`1`/`2`/`24`/`32`）
    /// 走上去什么都不发生 ⇒ 只有 bump 面能跑脚本。全游戏 52 扇门里这一类**只有
    /// f35 `param=24` 三格**，且从 (1,11) 23 步可直接出层 ⇒ 本臂当前是**动作覆盖**、
    /// 零进度收益（见 `docs/findings/2026-11-19-f3-gate27-walk-only.md` §7 审计表）。
    ///
    /// 判定用轻量 [`StGate`]（不 `build_scene`）；最终是否真触发仍由引擎
    /// `try_run_scene(param,false)` 决定（本函数只预筛，判错只会少一个动作）。
    fn bump_only_gate_params(&self, st: &St, tx: i32, ty: i32) -> Vec<i32> {
        let mut out = Vec::new();
        for e in st.ents.iter() {
            if e.ty != 83 || e.x != tx || e.y != ty {
                continue;
            }
            let mk = || StGate {
                ents: &st.ents,
                difficulty: self.input.difficulty,
                hp: st.player.hp,
                items: &st.player.item_types,
                flags: st.flags,
            };
            let mut vb = mk();
            let bump = game_core::rules::gate::evaluate(e.param, &mut vb, false).run;
            if !bump {
                continue;
            }
            let mut vw = mk();
            let walk = game_core::rules::gate::evaluate(e.param, &mut vw, true).run;
            if !walk {
                out.push(e.param);
            }
        }
        out
    }

    /// 【P7 撞门】撞门转移（`interact_with_cell` 的 bump 面，`from_walk=false`；
    /// a.java:5508-5510、scene.rs:3369-3393）：门脚本在**玩家不移动**的前提下跑完。
    ///
    /// 语义依据（A 级）：
    /// - 门的第二参数由引擎定：踩格面传 `true`（scene.rs:3969-3977）、撞门面传
    ///   `false`（scene.rs:3379）——同一 `evaluate` 的两条入口（gate.rs）；
    /// - 撞门面命中即 `allowed=false` ⇒ `try_step` 不放行、`walk_phase` 不变
    ///   ⇒ 玩家留格；`load_level_script` 已在同拍装载脚本 ⇒ 需要 `tickdrive` 驱动；
    /// - 脚本效果**全部真跑引擎**（`try_run_scene` → `load_level_script` → `tickdrive`
    ///   → `readback_script`），❌不手写门判定、❌不手写脚本效果。
    ///
    /// 边界（保守 ⊥，不产假状态）：
    /// - `Pending`（脚本换层）/`Stalled`（停摆）⇒ None：撞门只建模**同层**脚本效果，
    ///   换层边仍只由走步臂的 `gate_arrive` 产（不新增第二条层间通道）；
    /// - 脚本内有玩家走位（读回 `pos != 入场格`）⇒ None：撞门不动是前提，动了就
    ///   说明该脚本走的不是纯「同层原地」效果面。
    ///
    /// 不变量（硬约束）：w能触发门（如 f51 `p40`、f54 `p64`）在这里恒 `None`
    /// ⇒ 那四层**不产生任何新后继**、输出逐字段不变。
    fn gate_bump(&self, st: &St, tx: i32, ty: i32) -> Option<St> {
        let params = self.bump_only_gate_params(st, tx, ty);
        if params.is_empty() {
            return None;
        }
        for param in params {
            GateBumpStat::bump(|s| s.candidates += 1);
            let key = BumpKey {
                floor: self.input.floor,
                param,
                pos: st.pos,
                gate: (tx, ty),
                player: st.player.clone(),
                flags: st.flags,
                ents: st.ents.clone(),
            };
            if let Some((p, f, ents)) = BUMP_MEMO.with(|m| m.borrow().get(&key).cloned()) {
                let mut ns = st.clone();
                ns.player = p;
                ns.flags = f;
                ns.ents = ents;
                GateBumpStat::bump(|s| {
                    s.script_hits += 1;
                    s.memo_hits += 1;
                });
                return Some(ns);
            }
            let mut sc = build_scene(self.input, st, st.pos);
            // 引擎判门（bump 面）：未开/不满足 ⇒ 撞这一下什么都不发生（⊥，不是走格）
            if !sc.try_run_scene(param, false) {
                GateBumpStat::bump(|s| s.rejected_not_fired += 1);
                continue;
            }
            sc.load_level_script(param);
            match tickdrive(&mut sc) {
                Drive::Completed => {
                    let mut ns = st.clone();
                    readback_script(&mut ns, &sc);
                    if ns.pos != st.pos {
                        // 脚本内含玩家走位 ⇒ 不建模（撞门不动的前提破了）。**不静默**：
                        // 记 `rejected_pos_moved`（判据断言真层恒 0；见 GateBumpStat 注释）。
                        GateBumpStat::bump(|s| s.rejected_pos_moved += 1);
                        continue;
                    }
                    BUMP_MEMO.with(|m| {
                        m.borrow_mut()
                            .insert(key, (ns.player.clone(), ns.flags, ns.ents.clone()));
                    });
                    GateBumpStat::bump(|s| s.script_hits += 1);
                    return Some(ns);
                }
                Drive::Pending(_) => {
                    GateBumpStat::bump(|s| s.rejected_pending += 1);
                    return None;
                }
                Drive::Stalled => {
                    GateBumpStat::bump(|s| s.rejected_stalled += 1);
                    return None;
                }
            }
        }
        None
    }

    /// 拾取转移（C6，§1.3 拾取行字面路径）：harness pub `try_step` 落格 → 有拍
    /// `tick` 推进 32px 归格 → `apply_step_cell_effects`（scene.rs:4093-4096
    /// `pickup_item_type`+`entity::remove` 同拍）→ 读回 player 全字段 + 实体移除
    /// + flags（Obs 读回三源之 flags，§1.3）。引擎时序直调；
    /// ❌不自写物品效果表、❌不直调私有 `pickup_item_type`。
    /// 返回 None ⇒ ⊥（try_step 被拒或步进未完成——不产生状态，§1.1）。
    fn pickup_step(&self, st: &St, tx: i32, ty: i32) -> Option<St> {
        let mut sc = build_scene(self.input, st, st.pos);
        // 拾取=Map 语境场步（tick 早退链：Dialog 在 walk_phase 分派前 return
        // ⇒ 必须 Map 才推进手动步；批2 夹具拾取面全为 Map 场景）
        sc.game_mode = GameMode::Map;
        sc.key_value = 0; // 无输入注入（步尽后的 handle_field_input 不得消费残留键）
        // facing（try_step：0=下 1=上 2=右 3=左）；DIRS 保证 (tx,ty) 邻接。
        // ⚠ try_step(facing) 只用参数算目标格、**不写 self.facing**——tick 步进臂
        // （scene.rs:3022 `match self.facing`）按场景 facing 字段走；引擎正常流=
        // handle_field_input 先置 facing 再 try_step。故此处必须同步置位，否则走错方向。
        let facing = match (tx - st.pos.0, ty - st.pos.1) {
            (0, 1) => 0,
            (0, -1) => 1,
            (1, 0) => 2,
            (-1, 0) => 3,
            _ => return None, // 非邻接（防御：expand 只走 DIRS）
        };
        sc.facing = facing;
        if !sc.try_step(facing) {
            return None; // 引擎拒步（interact_with_cell 不通过）⇒ ⊥
        }
        // 有拍推进：每 tick 8px、4 tick 归格触发 apply；拾取后 walk_phase 自动回 0
        let wt = sc.width_table.clone();
        let mut landed = false;
        for ft in 0..8i64 {
            if sc.walk_phase == 0
                && (sc.view.player_px >> 5) == tx
                && (sc.view.player_py >> 5) == ty
            {
                landed = true;
                break;
            }
            sc.tick(ft, &wt);
        }
        if !landed {
            return None; // 步进未完成（超时/异常）⇒ ⊥，保守不产假转移
        }
        // 读回（§1.3 拾取行：apply 已执行拾取+移除；逐字段镜像 from_combat 映射）
        let p = &sc.player;
        let mut ns = st.clone();
        ns.player = PlayerSpec {
            hp: p.hp,
            atk: p.atk,
            def: p.def,
            yellow: p.yellow_keys,
            blue: p.blue_keys,
            red: p.red_keys,
            gold: p.gold,
            item_types: p.item_types.clone(),
            item_uses: p.item_uses.clone(),
            item_stack_size: p.item_stack_size,
        };
        // 实体表读回（C6a A-lite）：sc=build_scene(st)→引擎 try_step+apply
        // 同拍 pickup+remove ⇒ sc.entities 活体压实序=拾取后真值（含本层此前
        // CES/DES 增删——比原 sprite 位图循环更全）。
        ns.ents = readback_ents(&sc);
        // flags 读回（§1.3 Obs.flags 同源；apply 内 case0 等置位面）
        for i in 0..sc.slots.script_line_flags.len().min(128) {
            if sc.slots.script_line_flags[i] {
                ns.flags |= 1u128 << i;
            } else {
                ns.flags &= !(1u128 << i);
            }
        }
        ns.pos = (tx, ty);
        ns.steps += 1;
        // 边界（报审注明）：拾取格与 type83 共存批2 夹具无（floor1 键格无 83）——
        // 共存时 apply 在本 harness 内已触发门、驱动不完整=批3 面，本批不建模。
        Some(ns)
    }
}

/// 【P5 纯拾取闭式】格分类**三态批投影**：`classify` 的单源复用
/// （禁第二份分类表——上一片 `plan_pill` 的 G4 就是靠复用 `classify` 才没漂移）。
///
/// 返回 `(cols, rows, kinds)`，`kinds[y*cols+x]`：`Move`⇒`Free`、`Pickup`⇒`Pickup`、
/// 其余（门/商店/怪/阻挡）⇒`Other`。分类用**入场实体表**（`root_ents`）——
/// 纯层内实体只减不增（无怪无脚本），故入场分类在整条路线上恒定。
pub(crate) fn pure_cell_kinds(
    input: &SolveInput,
    ents: &[EntRec],
) -> Option<(i32, i32, Vec<crate::pure_pickup::CellKind>)> {
    use crate::pure_pickup::CellKind;
    let level = input.level.as_ref()?;
    let s = Search::from_entry(input, level, level.start);
    let st = St {
        pos: level.start,
        player: level.player.clone(),
        ents: ents.to_vec(),
        flags: level.flags,
        kills: 0,
        steps: 0,
    };
    let mut out = Vec::with_capacity((level.cols * level.rows) as usize);
    for y in 0..level.rows {
        for x in 0..level.cols {
            out.push(match s.classify(&st, x, y) {
                Cell::Move => CellKind::Free,
                Cell::Pickup => CellKind::Pickup,
                _ => CellKind::Other,
            });
        }
    }
    Some((level.cols, level.rows, out))
}

/// 【P5 纯拾取闭式】回放产物（全部来自引擎读回，无推演）。
pub(crate) struct PickupReplayOut {
    pub player: PlayerSpec,
    pub flags: u128,
    pub pos: (i32, i32),
    pub ents: Vec<EntRec>,
    pub steps: i32,
}

/// 【P5 纯拾取闭式】逐步回放一条纯拾取路线——每步都走 [`Search::pickup_step`]
/// （真 `try_step` + 拍推进 + 读回三源），**任一步被拒/未落格 ⇒ `None`**
/// （fail-closed：不产假转移，也不把「模型里能走」当「引擎里能走」）。
/// `cells[0]` 必须是 `entry`（路线首 token=入场格约定，与 `route_of` 同口径）。
pub(crate) fn replay_pickup_route(
    input: &SolveInput,
    entry: (i32, i32),
    ents: &[EntRec],
    player: &PlayerSpec,
    flags: u128,
    cells: &[(i32, i32)],
) -> Option<PickupReplayOut> {
    let level = input.level.as_ref()?;
    if cells.first().copied() != Some(entry) {
        return None;
    }
    let search = Search::from_entry(input, level, entry);
    let mut st = St {
        pos: entry,
        player: player.clone(),
        ents: ents.to_vec(),
        flags,
        kills: 0,
        steps: 0,
    };
    for &(tx, ty) in &cells[1..] {
        st = search.pickup_step(&st, tx, ty)?;
    }
    Some(PickupReplayOut {
        player: st.player,
        flags: st.flags,
        pos: st.pos,
        ents: st.ents,
        steps: st.steps,
    })
}

/// T7（提案 §3.1 T7 / 计划 :319「find_path found 与 solver BFS found 一致」）：
/// `game_core::pathfind::find_path` 直调薄封装——转发其 found，不复刻任何怪癖；
/// T7 对拍测试用（生产侧 C5d 起无 subgraph）。谓词=terrain||slot。
/// 注：不能返回 `Walkability`（`has_entity_slot` 闭包借 `level` 悬垂=E0515），每次装配。
pub fn find_cell_path(level: &LevelSpec, from: (i32, i32), to: (i32, i32)) -> bool {
    let cols = level.cols;
    let rows = level.rows;
    let terrain: Vec<Vec<bool>> = (0..rows)
        .map(|y| {
            (0..cols)
                .map(|x| level.terrain[(y * cols + x) as usize])
                .collect()
        })
        .collect();
    let has_slot = |x: i32, y: i32| level.entities.iter().any(|e| e.x == x && e.y == y);
    let walk = Walkability {
        terrain,
        has_entity_slot: &has_slot,
        cols,
        rows,
    };
    find_path(&walk, from.0, from.1, to.0, to.1).found
}

/// T7 对拍的 solver 侧 BFS（独立遍历、同谓词 terrain||slot、同邻序左右上下）。
/// 语义按 `find_path` 实测对齐（分歧根因=其结构性怪癖，见报审 T7 段）：
/// - 目标格不可走（无槽）⇒ false——find_path 只 push 可走格，dst 永不入队；
/// - 起点自身**不需可走**——find_path 从起点直接 relax 邻格、不校验起点；
/// - `from == to` ⇒ 起点可走且有可走邻格才 true——find_path 起点不入队、
///   靠邻格回推；孤立格经幻影队头回溯失败（pathfind.rs:133）⇒ false；
/// - 不设 expansions≤100 上限（floor1 可推格 <100 不触发；触发即 T7 实测暴露）。
pub fn bfs_reachable(level: &LevelSpec, from: (i32, i32), to: (i32, i32)) -> bool {
    let cols = level.cols;
    let rows = level.rows;
    let inb = |x: i32, y: i32| x >= 0 && y >= 0 && x < cols && y < rows;
    if !inb(from.0, from.1) || !inb(to.0, to.1) {
        return false;
    }
    let pushable = |x: i32, y: i32| -> bool {
        level.terrain[(y * cols + x) as usize]
            || level.entities.iter().any(|e| e.x == x && e.y == y)
    };
    if !pushable(to.0, to.1) {
        return false;
    }
    if from == to {
        // find_path 起点不入队：须有可走邻格回推才 found；孤立格=false（幻影队头回溯失败）
        return pushable(from.0, from.1)
            && [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|&(dx, dy)| {
                    let (nx, ny) = (from.0 + dx, from.1 + dy);
                    inb(nx, ny) && pushable(nx, ny)
                });
    }
    // BFS：起点可不 pushable（find_path 从起点直接 relax 邻格）；仅可走格可被进入；
    // dst 在 push 时判（find_path 在 pop 时判，未封顶时二者同真值）
    let mut visited = vec![vec![false; cols as usize]; rows as usize];
    let mut q = VecDeque::new();
    visited[from.1 as usize][from.0 as usize] = true;
    q.push_back(from);
    while let Some((x, y)) = q.pop_front() {
        for &(dx, dy) in &[(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !inb(nx, ny) || visited[ny as usize][nx as usize] || !pushable(nx, ny) {
                continue;
            }
            if (nx, ny) == to {
                return true;
            }
            visited[ny as usize][nx as usize] = true;
            q.push_back((nx, ny));
        }
    }
    false
}

/// 路线重建（route_gen.py 航点：`route_gen.py:12-15`；route[0]=入口格）。
fn route_of(nodes: &[Node], id: u32) -> Vec<String> {
    let mut acts: Vec<(Act, (i32, i32))> = Vec::new();
    let mut cur = id;
    while cur != u32::MAX {
        let n = &nodes[cur as usize];
        acts.push((n.act.clone(), n.st.pos));
        cur = n.parent;
    }
    acts.reverse();
    acts.into_iter()
        .map(|(act, pos)| match act {
            Act::Start => format!("{},{}", pos.0, pos.1),
            Act::Move(x, y) => format!("{x},{y}"),
            Act::Bump(ty, x, y) => format!("bump {ty}@{x},{y}"),
            Act::Door(x, y) => format!("door {x},{y}"),
            // P7 撞门：门格坐标（玩家停在隔壁，未到达该格）
            Act::GateBump(x, y) => format!("gatebump {x},{y}"),
        })
        .collect()
}

// ---------------------------------------------------------------- C5 harness

/// 等价态灌一个 harness `GameScene`（提案 §2.2 管线③「从等价态灌 harness →
/// drive → 读回 → 并回抽象态」；❌不把 GameScene 当搜索态本体——仅 StepGate
/// 到达时临时构造）。`scene_test_support` 全程 include_str 夹具，无运行时 I/O。
fn build_scene(input: &SolveInput, st: &St, at: (i32, i32)) -> GameScene {
    let level = input
        .level
        .as_ref()
        .expect("gate 转移需要夹具（level=None 占位分支不经过此路径）");
    // C6a：st.ents=活体实体列表（引擎槽序）直灌——(type,x,y) 三元组进
    // scene_setup（param=0 起步），下方 restore 按 (type,格) 首配回灌 sprite param。
    let ents: Vec<(u8, i32, i32)> =
        st.ents.iter().map(|r| (r.ty as u8, r.x, r.y)).collect();
    let mut sc = scene_setup(
        at,
        &ents,
        SceneSetup {
            game_mode: Some(GameMode::Dialog),
            dialog_dialog_phase: Some(0),
            menu_difficulty_index: Some(input.difficulty),
            persist_floor: Some(input.floor),
            persist_current_floor: Some(input.floor),
            player_hp: Some(st.player.hp),
            player_atk: Some(st.player.atk),
            player_def: Some(st.player.def),
            player_yellow_keys: Some(st.player.yellow),
            player_blue_keys: Some(st.player.blue),
            player_red_keys: Some(st.player.red),
            player_gold: Some(st.player.gold),
            ..Default::default()
        },
    );
    // 物品栈连续性（批3 C3b）：build_scene 每步重建场景，而引擎 item_types/
    // item_uses/item_stack_size 挂在持久 player 上跨步保留；SceneSetup 只有 7
    // 个标量字段（scene_test_support.rs:220-226），不注入则拾取回读把先前栈
    // 丢掉——实证：T1 夹具 (1,1) 拾 type15→(8,7) 拾钥 type26 回读 [0×32]
    // （KeyEffect 只 ++yellow 不碰栈，effect_registry.rs:227/207-210），15 丢失
    // ⇒ 筋斗云边漏发。空 Vec=夹具未铺栈（≡引擎默认 32 槽零填、size0）不覆盖。
    if !st.player.item_types.is_empty() {
        sc.player.item_types = st.player.item_types.clone();
        sc.player.item_uses = st.player.item_uses.clone();
        sc.player.item_stack_size = st.player.item_stack_size;
    }
    // 地形：夹具 terrain 灌满 walkable；夹具外格=False（杀合成地图 y=3 行泄漏）。
    // 夹具 ≤13×13（scene() 合成几何）；更大层需扩展 harness 几何=批3 面。
    for y in 0..(level.rows as usize).min(sc.walkable.len()) {
        let cols = (level.cols as usize).min(sc.walkable[y].len());
        for x in 0..cols {
            sc.walkable[y][x] = level.terrain[y * level.cols as usize + x];
        }
    }
    // 实体 param 回灌：按 (type,格) 首配消费（scene() 三元组 param 恒 0；同格多枚
    // 按夹具序）。楼梯 param 必需——param=0 ⇒ change_floor(0) 撞 pending_floor=0
    // 哨兵（e2 实测 PENDING 前 pending=0）；script19 控制流不读 83 param（e2 实测
    // params=0 仍走到终止边）；scene() 会多生实体（e1 count=3 > 夹具 2），不能 zip。
    let mut pool: Vec<EntRec> = st.ents.clone();
    for sc_e in sc.entities.items.iter_mut() {
        if let Some(pos) = pool.iter().position(|s| {
            s.ty == sc_e.type_ as i32 && s.x == (sc_e.px >> 5) && s.y == (sc_e.py >> 5)
        }) {
            sc_e.param = pool.remove(pos).param as i16;
        }
    }
    // flags 位图灌入（script_line_flags pub，SceneSetup 无该字段；68 槽 ≤ u128）。
    for (i, f) in sc.slots.script_line_flags.iter_mut().enumerate() {
        *f = i < 128 && ((st.flags >> i) & 1) == 1;
    }
    sc
}

/// 有拍驱动（§2.2②③）：每拍注入 Confirm（`key_value=1`）+ `tick`——引擎节拍机
/// 自排序（phase0 逐指令→phase1 打字机→phase3 行走 32px 归格踩格→…）。
/// 返回 `Some(pending_floor)`=脚本换层终止边；`None`=行尾收尾进 Map（非终态
/// 完成）或停摆——两者在 C5 口径均为 ⊥（Dropped），安全兜底不产假 reachable。
/// 注：无拍 `side_a::drive` 实测产不出终止边（b35/b38 实证）——提案 C5 行
/// 「驱动复用 side_a::drive」字面不可行，偏离已记模块头 doc，报审注明。
/// 批3 C5a：有拍驱动结果三态（原 `Option<i32>` 把「脚本演出完成」与「停摆」
/// 混同于 None——真数据演出门（TAK/ROS/SEE 跑完回 Map）被误判 Dropped(⊥)
/// 卡死首层；判据③空心根因）。Pending=换层、Completed=演出完成可续走、
/// Stalled=停摆兜底 ⇒ ⊥。
enum Drive {
    Pending(i32),
    Completed,
    Stalled,
}

fn tickdrive(sc: &mut GameScene) -> Drive {
    let wt = sc.width_table.clone();
    let mut ft = 0i64;
    let mut last = String::new();
    let mut stall = 0;
    loop {
        if sc.persist.pending_floor != 0 {
            return Drive::Pending(sc.persist.pending_floor);
        }
        if matches!(sc.game_mode, GameMode::Map) {
            // C5d：脚本 walk（MOV/SEE flag=1）以背景相位在 Map 后继续推进——
            // armed 且未落 dialog_target 前不收尾（提前收尾实测读回半途
            // view=(2,1) 而非目标 (6,5)）；落点/无 armed walk ⇒ 收尾。
            // stall>500 兜底仍在（永动防护），walk 不落=Stalled=保守 ⊥。
            let landed = (sc.view.player_px >> 5, sc.view.player_py >> 5)
                == sc.dialog.dialog_target;
            if !(sc.slots.script_walk_armed && !landed) {
                return Drive::Completed; // 行尾收尾 transition_to(Map)：演出完成、玩家留格（C5a 续走）
            }
        }
        sc.key_value = 1;
        sc.tick(ft, &wt);
        ft += 1;
        let sig = format!(
            "{}/{}/{}/{}/{}",
            sc.dialog.dialog_phase,
            sc.slots.script_walk_armed,
            sc.walk_phase,
            sc.view.player_px,
            sc.view.player_py
        );
        if sig == last {
            stall += 1;
        } else {
            stall = 0;
        }
        last = sig;
        if stall > 500 || ft > 30_000 {
            return Drive::Stalled; // 停摆兜底 ⇒ ⊥（保守：不产生假 reachable）
        }
    }
}

/// 批3 C5a：门脚本终态读回（Pending/Completed 两臂共用；镜像 pickup_step
/// 读回面：player 全字段 + script_line_flags 位图回写 st.flags）。
fn readback_script(st: &mut St, sc: &GameScene) {
    let p = &sc.player;
    st.player.hp = p.hp;
    st.player.atk = p.atk;
    st.player.def = p.def;
    st.player.yellow = p.yellow_keys;
    st.player.blue = p.blue_keys;
    st.player.red = p.red_keys;
    st.player.gold = p.gold;
    st.player.item_types = p.item_types.clone();
    st.player.item_uses = p.item_uses.clone();
    st.player.item_stack_size = p.item_stack_size;
    for i in 0..sc.slots.script_line_flags.len().min(128) {
        if sc.slots.script_line_flags[i] {
            st.flags |= 1u128 << i;
        } else {
            st.flags &= !(1u128 << i);
        }
    }
    // C6a A-lite：pos 与实体表读引擎真值（pos 无玩家走位脚本时=到达格恒等；
    // ents=活体压实序，谓词=serialize_floor_state entity.rs:86-88）
    st.pos = (sc.view.player_px >> 5, sc.view.player_py >> 5);
    st.ents = readback_ents(sc);
}

thread_local! {
    /// 批3 C5d′/C6a/C6b：门结局 memo，线程级（原每 solve_layer/每 global pop
    /// 各建一份，真数据 e2e debug >7min 的主导开销）。键 `(floor, param, 入场
    /// player, 入场 flags, 入场 ents)`=引擎结局的全部输入（判门只读 flags[idx]
    /// scene.rs:928、脚本效果只吃 player/flags/ents/地形、地形由 floor 定）⇒
    /// 键完备（C6a 起 ents 入键：DES 结果依赖 pre-entities；A-lite 后无「安全集」
    /// 概念，全部 Completed 门都可缓存）。值=读回后全态（player/flags/pos/ents）。
    /// Pending/停摆 Stalled 不入缓存（现算）。thread_local=多线程测试隔离；
    /// 缓存纯加速、不改遍历序（确定性不受影响）。
    static GATE_MEMO: RefCell<
        HashMap<
            (i32, i32, PlayerSpec, u128, Vec<EntRec>),
            (PlayerSpec, u128, (i32, i32), Vec<EntRec>),
        >,
    > = RefCell::new(HashMap::new());
}
/// T9 探针（`probe_bump` 同法=与搜索共用 harness 门面路径，非自证）：夹具
/// flags/difficulty 下 `try_run_scene(param, true)` 的判门结果——只判不放
/// （不 load/tick；case19/66/67 臂只读 flags/difficulty，无副作用）。
pub fn probe_gate(input: &SolveInput, param: i32) -> bool {
    let level = input.level.as_ref().expect("probe_gate 需要夹具");
    let st = St {
        pos: level.start,
        player: level.player.clone(),
        ents: sprite_full(level),
        flags: level.flags,
        kills: 0,
        steps: 0,
    };
    let mut sc = build_scene(input, &st, level.start);
    sc.try_run_scene(param, true)
}

/// 【P7 撞门】`BUMP_MEMO` 键（P7 复核黄 F4 补全）。
///
/// 原键 `(floor, param, player, flags, ents)` **缺 `pos` 与门格**：脚本坐标虽然
/// 全绝对（今日不可观测），但「撞门结局」在语义上依赖玩家站哪、撞哪一扇门
/// ⇒ 键不完备就是潜伏的错复用（P3 同类）。这里显式补齐两维，并用具名字段
/// 防止再加维度时位置搞错。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BumpKey {
    floor: i32,
    param: i32,
    /// 撞门者所在格（bump 面不移动 ⇒ 出参与入参同格）
    pos: (i32, i32),
    /// 被撞的门格（同层同 param 的两扇门是两条结局）
    gate: (i32, i32),
    player: PlayerSpec,
    flags: u128,
    ents: Vec<EntRec>,
}

thread_local! {
    /// 【P7 撞门】bump 结局 memo，与 [`GATE_MEMO`] 同法但**键含 from_walk 语义分隔**
    /// （独立表 ⇒ 不与走上门的结局混淆）。值 `(player, flags, ents)`——撞门**不移动**
    /// （bump 面 `allowed=false`），故 pos 不入值，恒等入场格。
    ///
    /// **清理策略**：每层搜索入口（[`Search::run_root_with`]）清空 ⇒ 层间不串味、
    /// 内存有界（≤ 本层被撞过的键数），且不影响层内复用（同一层内键集合有限）。
    static BUMP_MEMO: RefCell<HashMap<BumpKey, (PlayerSpec, u128, Vec<EntRec>)>> =
        RefCell::new(HashMap::new());

    /// 【P7 撞门】臂统计（与 `SHOP_BUYS`/`PillDom::Off` 同法：**禁静默丢弃**）。
    ///
    /// P7 复核黄 F1：原实现把「判门未开 / 脚本换层 / 停摆 / 脚本移动玩家」四类
    /// 结局直接 `continue`/`return None`，全仓无计数 ⇒ 既不可测也不可观测。
    /// 这里给每类一个具名计数器（`rejected_*`），`candidates`/`script_hits` 保留
    /// 原二元口径（`gate_bump_stats()` 是它的投影，禁两套真相）。
    static GATE_BUMP_STAT: std::cell::Cell<GateBumpStat> =
        const { std::cell::Cell::new(GateBumpStat::CLEARED) };
}

/// 【P7 撞门】逐类计数（见 `GATE_BUMP_STAT`）。
///
/// - `candidates`：预筛（[`Search::bump_only_gate_params`]）认定的撞门候选数；
/// - `script_hits`：真跑引擎并产后继的数（含 memo 命中）；
/// - `memo_hits`：其中走 `BUMP_MEMO` 复用的；
/// - `rejected_not_fired`：`try_run_scene(param,false)==false`（StGate 预筛过判）；
/// - `rejected_pending`：脚本换层（`Lay`）⇒ 撞门面不建模（层间通道只由走步臂产）；
/// - `rejected_stalled`：`tickdrive` 停摆；
/// - `rejected_pos_moved`：脚本内玩家走位（撞门不动的前提破了）。
///
/// **裁定（P7 复核 A2 二选一）**：`pos` 变化**降级为具名计数**而非硬断言。
/// 理由：撞门面是「不移动」的**语义前提**，不是数据结构不变量——同一扇 bump-only
/// 门若脚本里含 `MOV_0_*` 传送，那是数据面合法但本模型不覆盖的转移；硬断言会在
/// 生产路径上 panic，而具名计数 + 判据（`bump_stat_counts_every_rejection` 断言
/// 今天的真层恒 0）既保留可观测，又不把模型边界变成崩溃点。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GateBumpStat {
    pub candidates: u64,
    pub script_hits: u64,
    pub memo_hits: u64,
    pub rejected_not_fired: u64,
    pub rejected_pending: u64,
    pub rejected_stalled: u64,
    pub rejected_pos_moved: u64,
}

impl GateBumpStat {
    pub const CLEARED: Self = Self {
        candidates: 0,
        script_hits: 0,
        memo_hits: 0,
        rejected_not_fired: 0,
        rejected_pending: 0,
        rejected_stalled: 0,
        rejected_pos_moved: 0,
    };
    fn bump<F: FnOnce(&mut Self)>(f: F) {
        GATE_BUMP_STAT.with(|c| {
            let mut s = c.get();
            f(&mut s);
            c.set(s);
        });
    }
}

/// 【P7 撞门】逐类计数快照（判据与证据工具的观测面）。
pub fn gate_bump_stat() -> GateBumpStat {
    GATE_BUMP_STAT.with(|c| c.get())
}

/// 【P7 撞门】旧二元投影 `(候选生成数, 脚本真命中数)`——与 [`gate_bump_stat`] 同源。
pub fn gate_bump_stats() -> (u64, u64) {
    let s = gate_bump_stat();
    (s.candidates, s.script_hits)
}

/// 【P7 撞门】轻量 `GateView`：判「这扇门只有撞才触发吗」。
///
/// 为什么不用 `build_scene`：`gate_arrive` 的注释（`GATE_MEMO` 上方）实测 `build_scene`
/// 是逐节点主要开销之一，而本判定要在**每个节点的每个邻格**上做 ⇒ 必须廉价。
///
/// 逐字段与引擎 `impl GateView for GameScene`（scene.rs:4824）同义；差异只在两处，
/// 都在撞门臂用不到的面上，且**偏向保守**（判 false ⇒ 只是少一个动作，不会造假状态）：
/// - `script_count`：夹具槽数 68（与 dynreach.rs 的 `FIXTURE_SCRIPT_SLOTS` 同量级），
///   只用于 `param` 越界判定——真实门 param 全 < 68；
/// - `cross_floor_snapshot` / `predict_hp_loss_vs`：`build_scene` 不注入跨层快照与
///   战斗上下文（`SceneSetup` 无这些字段）⇒ 引擎侧同样返回 false/0；
///   `predict_hp_loss_vs` 只被 `param=15`（活型 5 反判）读，与撞门臂无关。
struct StGate<'a> {
    ents: &'a [EntRec],
    difficulty: i32,
    hp: i32,
    items: &'a [u8],
    flags: u128,
}

impl game_core::rules::gate::GateView for StGate<'_> {
    fn script_count(&self) -> i32 {
        68
    }
    fn flag(&self, idx: i32) -> bool {
        idx >= 0 && idx < 128 && (self.flags >> idx) & 1 == 1
    }
    fn set_flag(&mut self, idx: i32) {
        if idx >= 0 && idx < 128 {
            self.flags |= 1u128 << idx;
        }
    }
    fn entity_at(&self, x: i32, y: i32, type_id: i32) -> i32 {
        self.ents
            .iter()
            .position(|e| e.x == x && e.y == y && e.ty == type_id)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }
    fn alive_type_on_floor(&self, type_id: i32) -> bool {
        self.ents.iter().any(|e| e.ty == type_id)
    }
    fn difficulty_index(&self) -> i32 {
        self.difficulty
    }
    fn player_hp(&self) -> i32 {
        self.hp
    }
    fn has_item(&self, item: i32) -> bool {
        item >= 0 && self.items.contains(&(item as u8))
    }
    fn cross_floor_snapshot(&mut self, _floor: i32, _type_id: i32) -> bool {
        false
    }
    fn predict_hp_loss_vs(&self, _ety: i32, _no_counter: bool) -> i32 {
        0
    }
}

/// 【P7 撞门】单参数判门探针：**与搜索同一个 [`StGate`]**（禁第二份 GateView 实现）。
///
/// `from_walk=false` = 撞门面（`try_run_scene(param,false)` 的判门口径）、
/// `true` = 踩格面。只判不放（不 load/tick）。
pub fn probe_gate_verdict(
    input: &SolveInput,
    player: &PlayerSpec,
    flags: u128,
    ents: Option<&[EntRec]>,
    param: i32,
    from_walk: bool,
) -> bool {
    let level = input.level.as_ref().expect("probe_gate_verdict 需要夹具");
    let ents: Vec<EntRec> = ents.map(|v| v.to_vec()).unwrap_or_else(|| sprite_full(level));
    let mut v = StGate {
        ents: &ents,
        difficulty: input.difficulty,
        hp: player.hp,
        items: &player.item_types,
        flags,
    };
    game_core::rules::gate::evaluate(param, &mut v, from_walk).run
}

/// 格分类探针（集成测试/证据工具面）：返回 `"blocked"`/`"move"`/`"door"`/
/// `"monster"`/`"pickup"`/`"shop"`——与 `Search::classify` **同一实现**（不是副本）。
pub fn probe_cell_class(input: &SolveInput, ents: Option<&[EntRec]>, at: (i32, i32)) -> &'static str {
    let level = input.level.as_ref().expect("probe_cell_class 需要夹具");
    let st = St {
        pos: level.start,
        player: level.player.clone(),
        ents: ents.map(|v| v.to_vec()).unwrap_or_else(|| sprite_full(level)),
        flags: level.flags,
        kills: 0,
        steps: 0,
    };
    let se = Search::new(input, level);
    match se.classify(&st, at.0, at.1) {
        Cell::Blocked => "blocked",
        Cell::Move => "move",
        Cell::Door(..) => "door",
        Cell::Monster(..) => "monster",
        Cell::Pickup => "pickup",
        Cell::Shop(..) => "shop",
    }
}

/// 【P7 撞门/A4】`BUMP_MEMO` 当前条目数（判据用：证明键含 `pos`、层界清空生效）。
pub fn bump_memo_len() -> usize {
    BUMP_MEMO.with(|m| m.borrow().len())
}

/// 【P7 撞门/A4】手工清空 `BUMP_MEMO`（判据前置；生产路径由层界入口自动清）。
pub fn bump_memo_clear() {
    BUMP_MEMO.with(|m| m.borrow_mut().clear());
}

/// 【P7 撞门】撞门探针（集成测试面）：在给定入场态上对邻格 `(tx,ty)` 走一次撞门，
/// 返回读回摘要。与 `expand` 调**同一个** `Search::gate_bump`（非自证：探针不是
/// 第二份实现），`None` ⇒ 该格不产生撞门后继。
pub fn probe_gate_bump(
    input: &SolveInput,
    entry: (i32, i32),
    player: &PlayerSpec,
    flags: u128,
    ents: Option<&[EntRec]>,
    at: (i32, i32),
) -> Option<BumpObs> {
    let level = input.level.as_ref().expect("probe_gate_bump 需要夹具");
    let st = St {
        pos: entry,
        player: player.clone(),
        ents: ents.map(|v| v.to_vec()).unwrap_or_else(|| sprite_full(level)),
        flags,
        kills: 0,
        steps: 0,
    };
    let se = Search::new(input, level);
    let ns = se.gate_bump(&st, at.0, at.1)?;
    Some(BumpObs {
        pos: ns.pos,
        hp: ns.player.hp,
        flags: ns.flags,
        ents: ns.ents,
    })
}

/// 【P7 撞门】[`probe_gate_bump`] 的读回摘要（逐字段镜像 `readback_script` 读回面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BumpObs {
    pub pos: (i32, i32),
    pub hp: i32,
    pub flags: u128,
    pub ents: Vec<EntRec>,
}

// ================================================================ 【P9】精确拾取 DP：引擎面
//
// 分工（P9，2026-11-19）：
// - **本段只提供引擎真值**：格分类、撞怪（掉血/赏金/掉落）、开门钥消耗、StepGate 踩格效果、
//   以及「一条含怪/门/gate 的路线」的逐步回放。DP 在 `exact_pickup.rs`；
//   这里**不复制任何判定表**——分类直调 `Search::classify`，战斗直调
//   `bump_outcome`（内部再直调 `BattleResolver`+`predict_engagement`），门直调
//   `consume_key_for_door`，gate 直调 `gate_arrive`（内部 `try_run_scene`+
//   `load_level_script`+`tickdrive`+读回）。
// - 为什么不能复用 `replay_pickup_route`：那条只走 `pickup_step`（`try_step` 落格），
//   **上不了怪格/门格**；精确层要「门开、怪杀、gate 踩」，故回放走与 `expand` 相同的
//   四条转移臂（Move/Monster/Door/Pickup）。

/// 精确 DP 的格类型（与 `Cell` 一一投影；`Cell::Shop` 按 `shop_can_buy` 判：可买 ⇒
/// `Off`，不可买 ⇒ 阻挡；见 `exact_layer_view`）。
pub const XC_BLOCKED: u8 = 0;
pub const XC_FREE: u8 = 1;
pub const XC_PICKUP: u8 = 2;
pub const XC_DOOR: u8 = 3;
pub const XC_MONSTER: u8 = 4;
pub const XC_GATE: u8 = 5;

/// 单格投影：`idx` = 对应维数组下标（拾取位/门位/杀位/gate 位），其余维为 0。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactCell {
    pub kind: u8,
    pub idx: usize,
    pub ty: i32,
    pub param: i32,
}

/// 商人格是否**真能改变状态**（机制推导，非 ty 白名单）。
///
/// A 级证据（单源 `game-core`）：
/// - `scene.rs::apply_merchant_offer`（:2522-2634）：`flags = (param >> 8) as i8`
///   （:2532）、`idx = param as i8`（:2533）、`src = ty == 77 ? 0 : 1`（:2530）；
///   `idx <= 0` ⇒ 无货直接返回（:2538）。
/// - **`flags & 2` 才是购买面入口**（:2541 买臂门；撞格触发臂也只有
///   `offer > 0` 才开面板，:3521）。玩家路径上 `apply_merchant_offer(true)` 的
///   **唯一**入口是报价面反馈 `-6|-5`（:2731），而 `controls.rs`（:265-290）只在
///   `flags&2` 时渲染 Buy/Accept 按钮，否则只有 Close ⇒ bit1=0 时整条买臂不可达。
/// - 买成才改状态：扣金（:2549）、施效（GiveGold/AddStack/StatPct/AddKeys/
///   SwapYellowKey/AddHp/PickupItem）、`param ^= 512`（:2597，`flags&4==0` 时）。
///   货表缺行也照走收尾块（:2545）⇒ 仍是状态改变，故「有无行」不进判据。
///
/// ⇒ 「`idx>0 ∧ flags&2`」是买臂可达的充要条件：有面即可能买成，无面即无入口。
///
/// **未建模边界（已披露，不是静默假设）**：`flags&2==0 ∧ flags&1!=0` 的告示格，
/// 关面板会 `param ^= 256` 且 `flags&4==0` 时移除实体（:2624/:2631）⇒ 该格事后可
/// 通行；DP 没有「告示已读」维，仍按阻挡处理（只可能**低估**可达性，不会凭空造出
/// 不可达的高 hp 解）。f3 的 `ty77 param=0x116` 正落此分支：`flags=1` ⇒ 不弹购买面；
/// 其 hp 结论另有算术上界 4027 独立佐证（见 `docs/findings/2026-11-19-exact-pickup-dp.md`）。
fn shop_can_buy(param: i32) -> bool {
    let flags = (param >> 8) as i8;
    let idx = param as i8;
    idx > 0 && flags & 2 != 0
}

/// 层视图：DP 的唯一静态输入（全部由 `classify` 单源产出，不含第二份规则表）。
#[derive(Debug, Clone)]
pub struct ExactLayerView {
    pub cols: i32,
    pub rows: i32,
    pub cells: Vec<ExactCell>,
    /// 只认 `Stair` 出口（`pure_pickup` G4 同口径：其它换层边不建模），且排除入场格。
    pub exits: Vec<((i32, i32), ExitKind)>,
    /// 维数组（位序=行主序扫描序，确定性）。
    pub pick_ty: Vec<i32>,
    pub door_ty: Vec<i32>,
    pub kill_ty: Vec<i32>,
    pub gate_param: Vec<i32>,
    /// 位 → 入场实体表下标（`root_ents` 槽位；用于「终态实体表」的精确推导）。
    pub pick_slot: Vec<usize>,
    pub door_slot: Vec<usize>,
    pub kill_slot: Vec<usize>,
}

impl ExactLayerView {
    pub fn cell(&self, p: (i32, i32)) -> Option<ExactCell> {
        if p.0 < 0 || p.1 < 0 || p.0 >= self.cols || p.1 >= self.rows {
            return None;
        }
        self.cells.get((p.1 * self.cols + p.0) as usize).copied()
    }
}

/// 构建层视图。**fail-closed**：任何「模型表达不了」的格都返回具名理由，
/// 调用方据此 `Off(reason)`——绝不静默跳过。
///
/// 复用面（逐条指名，禁复制）：
/// - 通行/门/怪/拾取分类 = `Search::classify`（`Cell` 单源）；
/// - 拾取增益认证 = `game_core::effect_registry::certified_monotone_pickup`；
/// - 商店：**逐格读 `param` 的机制位**（`shop_can_buy`），不是「表非空就拒」——
///   `flags&2` 才开价（`scene.rs:2541`）；开价格 ⇒ `Off("shop_present")`，
///   不开价格 ⇒ 阻挡格（引擎撞格即 `allowed=false`，见 `shop_can_buy` 的边界说明）。
///   商品表**不是**本函数的判定输入：判定读实体 `param`；表只喂 `hp_bound`/`Search`。
pub fn exact_layer_view(
    input: &SolveInput,
    entry: (i32, i32),
    ents: &[EntRec],
) -> Result<ExactLayerView, (&'static str, i32)> {
    use game_core::effect_registry::certified_monotone_pickup;
    let level = input.level.as_ref().ok_or(("no_level", 0))?;
    let s = Search::from_entry(input, level, entry);
    let st = St {
        pos: entry,
        player: level.player.clone(),
        ents: ents.to_vec(),
        flags: level.flags,
        kills: 0,
        steps: 0,
    };
    let mut cells = Vec::with_capacity((level.cols * level.rows) as usize);
    let (mut pick_ty, mut door_ty, mut kill_ty, mut gate_param) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut pick_slot, mut door_slot, mut kill_slot) = (Vec::new(), Vec::new(), Vec::new());
    for y in 0..level.rows {
        for x in 0..level.cols {
            let c = match s.classify(&st, x, y) {
                Cell::Blocked => ExactCell { kind: XC_BLOCKED, idx: 0, ty: 0, param: 0 },
                Cell::Move => match ents.iter().find(|e| e.ty == 83 && e.x == x && e.y == y) {
                    Some(g) => {
                        gate_param.push(g.param);
                        ExactCell {
                            kind: XC_GATE,
                            idx: gate_param.len() - 1,
                            ty: 83,
                            param: g.param,
                        }
                    }
                    None => ExactCell { kind: XC_FREE, idx: 0, ty: 0, param: 0 },
                },
                Cell::Pickup => {
                    let found = ents.iter().enumerate().find(|(_, e)| {
                        (e.x, e.y) == (x, y)
                            && matches!(entity_category_with_floor(e.ty, input.floor), 2 | 4)
                    });
                    let Some((slot, e)) = found else {
                        return Err(("pickup_slot_unknown", 0));
                    };
                    let ty = e.ty;
                    let Some(gain) = certified_monotone_pickup(ty, input.floor) else {
                        return Err(("pickup_uncertified", ty));
                    };
                    if !gain.is_gain() {
                        return Err(("pickup_not_gain", ty));
                    }
                    // 背包拾取会让「背包维」不可由 mask 推导（容量/叠放依赖引擎）⇒ 不建模。
                    if !gain.bag.is_empty() {
                        return Err(("pickup_adds_bag", ty));
                    }
                    pick_ty.push(ty);
                    pick_slot.push(slot);
                    ExactCell { kind: XC_PICKUP, idx: pick_ty.len() - 1, ty, param: 0 }
                }
                Cell::Door(slot, ty) => {
                    door_ty.push(ty);
                    door_slot.push(slot);
                    ExactCell { kind: XC_DOOR, idx: door_ty.len() - 1, ty, param: 0 }
                }
                Cell::Monster(slot, ty) => {
                    kill_ty.push(ty);
                    kill_slot.push(slot);
                    ExactCell { kind: XC_MONSTER, idx: kill_ty.len() - 1, ty, param: 0 }
                }
                Cell::Shop(slot, ty) => {
                    // 机制判据：买臂可达（`idx>0 ∧ flags&2`）⇒ 会改 hp/钥/金，位图推不出
                    // ⇒ fail-closed；不可买 ⇒ 与 `expand` 的「走臂也不成立」同义（引擎撞格
                    // 即阻挡，`scene.rs:3543 allowed=false`）⇒ 阻挡格（不静默放行）。
                    let param = st.ents.get(slot).map(|e| e.param).unwrap_or(0);
                    if shop_can_buy(param) {
                        return Err(("shop_present", ty));
                    }
                    ExactCell { kind: XC_BLOCKED, idx: 0, ty, param }
                }
            };
            cells.push(c);
        }
    }
    let mut exits: Vec<((i32, i32), ExitKind)> = level
        .exits
        .iter()
        .filter(|e| e.kind == ExitKind::Stair && (e.x, e.y) != entry)
        .map(|e| ((e.x, e.y), e.kind))
        .collect();
    exits.sort_by_key(|(p, _)| *p);
    exits.dedup();
    if exits.is_empty() {
        return Err(("no_stair_exit", 0));
    }
    Ok(ExactLayerView {
        cols: level.cols,
        rows: level.rows,
        cells,
        exits,
        pick_ty,
        door_ty,
        kill_ty,
        gate_param,
        pick_slot,
        door_slot,
        kill_slot,
    })
}

/// 撞怪真值（bump 面）：掉血 + 赏金 + 击杀后实体表（boss 出表 + 掉落实体 append）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactBump {
    pub loss: i32,
    pub gold: i32,
    pub ents: Vec<EntRec>,
}

/// 直调 `bump_outcome`（`BattleResolver::resolve_bump` + `predict_engagement(no_counter=true)`）
/// 与 `kill_bounty`/`apply_kill_drops`（与 `expand` 的怪臂**同一份**实现）。
pub fn exact_bump(
    input: &SolveInput,
    player: &PlayerSpec,
    ents: &[EntRec],
    ty: i32,
    slot: usize,
) -> Option<ExactBump> {
    let level = input.level.as_ref()?;
    if slot >= ents.len() {
        return None;
    }
    let pc = player.to_combat();
    let loss = bump_outcome(level, &pc, ty, slot)?;
    let gold = kill_bounty(level, &pc, ty);
    let mut st = St {
        pos: (0, 0),
        player: player.clone(),
        ents: ents.to_vec(),
        flags: 0,
        kills: 0,
        steps: 0,
    };
    st.ents.remove(slot);
    apply_kill_drops(&mut st, ty, input.floor);
    Some(ExactBump { loss, gold, ents: st.ents })
}

/// 踩上 StepGate 的真值：直调 `gate_arrive`（内部判门→脚本→有拍驱动→四源读回）。
/// 返回 `Some(读回后 player/flags/ents)` 仅当结局= `Pass`（可续走）；
/// `Terminal`/`Dropped`/`Ending` ⇒ `None`（精确层不建模换层/停摆门）。
pub fn exact_gate_effect(
    input: &SolveInput,
    entry: (i32, i32),
    ents: &[EntRec],
    player: &PlayerSpec,
    flags: u128,
    at: (i32, i32),
) -> Option<(PlayerSpec, u128, Vec<EntRec>)> {
    let level = input.level.as_ref()?;
    let s = Search::from_entry(input, level, entry);
    let mut st = St {
        pos: at,
        player: player.clone(),
        ents: ents.to_vec(),
        flags,
        kills: 0,
        steps: 0,
    };
    match s.gate_arrive(&mut st, at.0, at.1) {
        GateArrive::Pass => Some((st.player, st.flags, st.ents)),
        _ => None,
    }
}

/// 开门真值：直调 `consume_key_for_door`（门码映射同 `expand`/`scene.rs`：1黄/2红/_蓝）。
pub fn exact_open_door(player: &PlayerSpec, dty: i32) -> Option<PlayerSpec> {
    let door = match dty {
        1 => game_core::combat::door_code::YELLOW,
        2 => game_core::combat::door_code::RED,
        _ => game_core::combat::door_code::BLUE,
    };
    let mut pc = player.to_combat();
    let (consumed, _) = game_core::combat::consume_key_for_door(&mut pc, door);
    if !consumed {
        return None;
    }
    Some(PlayerSpec::from_combat(pc))
}

/// 回放产物（全部来自引擎读回）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactReplay {
    pub player: PlayerSpec,
    pub flags: u128,
    pub ents: Vec<EntRec>,
    pub pos: (i32, i32),
    pub steps: i32,
    pub kills: i32,
}

fn parse_xy(s: &str) -> Option<(i32, i32)> {
    let (a, b) = s.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// 逐步回放一条 exact 路线（唯一防线）：token 口径同 [`route_of`]——
/// `"x,y"`=走/拾取/踩 gate、`"door x,y"`=开门、`"bump ty@x,y"`=撞怪。
/// 每步都走与 `expand` **同一条**转移实现（`pickup_step` / `gate_arrive` /
/// `consume_key_for_door` / `bump_outcome`+`apply_kill_drops`）；任一步不成立 ⇒ `None`。
pub fn exact_replay_route(
    input: &SolveInput,
    entry: (i32, i32),
    ents: &[EntRec],
    player: &PlayerSpec,
    flags: u128,
    route: &[String],
) -> Option<ExactReplay> {
    let level = input.level.as_ref()?;
    let s = Search::from_entry(input, level, entry);
    let mut st = St {
        pos: entry,
        player: player.clone(),
        ents: ents.to_vec(),
        flags,
        kills: 0,
        steps: 0,
    };
    for tok in route {
        if let Some(rest) = tok.strip_prefix("bump ") {
            let (ty_s, xy) = rest.split_once('@')?;
            let ty: i32 = ty_s.trim().parse().ok()?;
            let (x, y) = parse_xy(xy)?;
            let slot = st.ents.iter().position(|e| {
                e.x == x && e.y == y && e.ty == ty && entity_category_with_floor(e.ty, input.floor) == 8
            })?;
            let b = exact_bump(input, &st.player, &st.ents, ty, slot)?;
            st.player.hp -= b.loss;
            st.player.gold += b.gold;
            st.ents = b.ents;
            st.kills += 1;
            st.steps += 1;
            // 撞杀后玩家移入被杀格（与 Search::expand Monster 臂同口径；DP 撞杀
            // 也移位——见 exact_pickup XC_MONSTER 臂）。不移位会让 bump 后续
            // walk token 报「非邻接位移」（S5 真数据 f51 回放实锤）。
            st.pos = (x, y);
        } else if let Some(rest) = tok.strip_prefix("door ") {
            let (x, y) = parse_xy(rest)?;
            let slot = st.ents.iter().position(|e| {
                e.x == x && e.y == y && entity_category_with_floor(e.ty, input.floor) == 1
            })?;
            let dty = st.ents[slot].ty;
            st.player = exact_open_door(&st.player, dty)?;
            st.ents.remove(slot);
            st.pos = (x, y);
            st.steps += 1;
            match s.gate_arrive(&mut st, x, y) {
                GateArrive::Pass => {}
                _ => return None,
            }
        } else {
            let (x, y) = parse_xy(tok)?;
            let dx = (x - st.pos.0).abs();
            let dy = (y - st.pos.1).abs();
            if dx + dy != 1 {
                return None; // 非邻接（防御：DP 只发邻接转移）
            }
            match s.classify(&st, x, y) {
                Cell::Pickup => {
                    st = s.pickup_step(&st, x, y)?;
                }
                Cell::Move => {
                    st.pos = (x, y);
                    st.steps += 1;
                    match s.gate_arrive(&mut st, x, y) {
                        GateArrive::Pass => {}
                        _ => return None,
                    }
                }
                _ => return None, // 门/怪/阻挡必须用专属 token
            }
        }
    }
    Some(ExactReplay { player: st.player, flags: st.flags, ents: st.ents, pos: st.pos, steps: st.steps, kills: st.kills })
}

/// 【S4-B】公开给 `outer_witness` 的坐标解析（与私有 [`parse_xy`] 同实现单源）。
pub fn parse_xy_pub(s: &str) -> Option<(i32, i32)> {
    parse_xy(s)
}

/// 【S4-B】全词表路线回放的具名失败（禁 `Option` 无诊断）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteReplayError {
    /// 0 起的 token 下标（回放定位 = 帧号）。
    pub token_index: usize,
    pub token: String,
    pub reason: String,
}

impl std::fmt::Display for RouteReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "token[{}] {:?}: {}", self.token_index, self.token, self.reason)
    }
}

/// 【S4-B】逐步回放一条 witness 路线（全 5 种 token：`x,y` / `door x,y` /
/// `bump ty@x,y` / `gatebump x,y` / `gate x,y`）。
///
/// 与 [`exact_replay_route`] 的差异：
/// - 商人格 `x,y` 在 `expand` 里有**两臂**（买/走，同 token）——回放分支回溯
///   ≤2 叉；给 [`replay_route_expect`] 时以「终态 == witness 记录的 out_*」定臂，
///   不给期望时多叉存活 ⇒ 具名 `ambiguous`（不猜）。
/// - `gatebump x,y` 走 `Search::gate_bump`（撞脚本门不移位不计步）。
/// - `gate x,y` 是 Script 终止边的**注记** token（坐标=读回后 pos），回放=无操作。
///
/// 每步都走与 `expand` 同一条转移实现（`pickup_step` / `gate_arrive` /
/// `gate_bump` / `consume_key_for_door` / `bump_outcome`+`apply_kill_drops` /
/// `shop_offer`+`apply_shop_effect`）；任一步不成立 ⇒ 具名 `Err`。
/// `Terminal` 是否出现在该层 route 的**末位**：其后只允许 `gate x,y` 注记 token。
///
/// 生产侧契约（Search 门臂）：命中 `GateArrive::Terminal` 后只记节点 + 追加
/// `gate x,y` 注记，**不再入队**（`solve_local.rs:2944-2983`）⇒ Terminal 是出层动作，
/// 其后不可能再有走格/撞怪 token。回放侧此前无条件接受 Terminal，mid-route 的
/// Terminal 会静默续走（S5 复核 F2 ①）。
/// ty75 撞杀后引擎**不 restep**（玩家留在原格）：`persist.floor <= 50` 的击杀钩子里
/// ty75 走 `try_run_scene(20,true)`（门 20 未被 `script_line_flags[20]` 关闭时成功）
/// ⇒ `battle_rerun_scene=true` ⇒ `scene.rs:3715` 的胜利补步不执行。
///
/// 与引擎同源：`scene.rs:3668-3699`（钩子作用域 floor<=50）+ `rules/gate.rs` 的
/// `gate = !flag(param)`（param=20）+ `scene.rs:3715`（`!battle_rerun_scene` 才 restep）。
/// 求解侧 flags 位图与引擎 `script_line_flags` 同索引（`StGate::flag`）。
///
/// 边界：只读 kill 时刻的 flag20；脚本 20 完成后置 flag20 的异步效果未在求解侧
/// readback（kill 钩子无读回面），同层第二只 ty75 会仍按「未置位」处理。
pub(crate) fn ty75_no_restep(floor: i32, ety: i32, flags: u128) -> bool {
    floor <= 50 && ety == 75 && (flags >> 20) & 1 == 0
}

fn terminal_at_route_end(route: &[String], i: usize) -> bool {
    route
        .get(i + 1..)
        .map_or(true, |rest| rest.iter().all(|t| t.starts_with("gate ")))
}

pub fn replay_route(
    input: &SolveInput,
    entry: (i32, i32),
    ents: &[EntRec],
    player: &PlayerSpec,
    flags: u128,
    route: &[String],
) -> Result<ExactReplay, RouteReplayError> {
    replay_route_inner(input, entry, ents, player, flags, route, &[], None)
}

/// 同 [`replay_route`]，但以 witness 记录的出层态定商店买/走臂：
/// 存活分支里**恰有一个**与 `(out_player, out_flags, out_ents, out_pos)` 全等
/// ⇒ 返回它；零个 ⇒ 具名 `no_match`；记录不唯一（witness 侧契约违例）⇒
/// 具名 `expect_ambiguous`。
pub fn replay_route_expect(
    input: &SolveInput,
    entry: (i32, i32),
    ents: &[EntRec],
    player: &PlayerSpec,
    flags: u128,
    route: &[String],
    shops: &[game_data::defs::ShopGoodDef],
    out_player: &PlayerSpec,
    out_flags: u128,
    out_ents: &[EntRec],
    out_pos: (i32, i32),
) -> Result<ExactReplay, RouteReplayError> {
    replay_route_inner(
        input,
        entry,
        ents,
        player,
        flags,
        route,
        shops,
        Some((out_player, out_flags, out_ents, out_pos)),
    )
}

#[allow(clippy::too_many_lines)]
fn replay_route_inner(
    input: &SolveInput,
    entry: (i32, i32),
    ents: &[EntRec],
    player: &PlayerSpec,
    flags: u128,
    route: &[String],
    shops: &[game_data::defs::ShopGoodDef],
    expect: Option<(&PlayerSpec, u128, &[EntRec], (i32, i32))>,
) -> Result<ExactReplay, RouteReplayError> {
    let level = input.level.as_ref().ok_or_else(|| RouteReplayError {
        token_index: 0,
        token: String::new(),
        reason: "level=None（模型缺口）".to_string(),
    })?;
    let mut s = Search::from_entry(input, level, entry);
    s.shops = shops; // 【S5 遗留黄1 修复】回放侧必须带真商品表，否则买臂永不可达（吞买臂不可观测）
    let start = St { pos: entry, player: player.clone(), ents: ents.to_vec(), flags, kills: 0, steps: 0 };
    // DFS：栈帧 = (状态, token 下标)。**分支死亡只弃本支**（商店双臂的存在使
    // 「错臂后续 token 必然失败」成为常态）——记录最深失败做具名诊断；栈空才报错。
    let mut stack: Vec<(St, usize)> = vec![(start, 0)];
    let mut survivors: Vec<St> = Vec::new();
    let mut last_err: Option<RouteReplayError> = None;
    while let Some((st, idx)) = stack.pop() {
        let mut cur = st;
        let mut i = idx;
        let mut dead: Option<RouteReplayError> = None;
        while i < route.len() {
            let tok = &route[i];
            let fail = |reason: &str| RouteReplayError {
                token_index: i,
                token: tok.clone(),
                reason: reason.to_string(),
            };
            // Terminal 只在 route 末位合法：其后只允许 gate 注记（见 `terminal_at_route_end`）。
            let gate_ok = |g: GateArrive| match g {
                GateArrive::Pass => None,
                GateArrive::Terminal { .. } if terminal_at_route_end(route, i) => None,
                GateArrive::Terminal { .. } => {
                    Some("Terminal 只允许出现在该层 route 末位（其后仅 gate 注记）")
                }
                _ => Some("gate_arrive 非 Pass/Terminal"),
            };
            let mut branched: Option<Vec<(St, usize)>> = None;
            if let Some(rest) = tok.strip_prefix("bump ") {
                let Some((ty_s, xy)) = rest.split_once('@') else { dead = Some(fail("bump token 缺 @")); break; };
                let Ok(ty) = ty_s.trim().parse::<i32>() else { dead = Some(fail("bump ty 非整数")); break; };
                let Some((x, y)) = parse_xy(xy) else { dead = Some(fail("bump 坐标非法")); break; };
                let Some(slot) = cur.ents.iter().position(|e| {
                    e.x == x && e.y == y && e.ty == ty && entity_category_with_floor(e.ty, input.floor) == 8
                }) else { dead = Some(fail("撞怪格无该怪实体")); break; };
                let Some(b) = exact_bump(input, &cur.player, &cur.ents, ty, slot) else {
                    dead = Some(fail("撞怪不可实施（打不动/规则拒绝）"));
                    break;
                };
                cur.player.hp -= b.loss;
                cur.player.gold += b.gold;
                cur.ents = b.ents;
                cur.kills += 1;
                // 撞杀后玩家**移入被杀格**（与 Search::expand Monster 臂同口径；漏移位
                // 会让后续 walk token 报「非邻接位移」——S5 真数据 f51 回放实锤）。
                // 例外（S5 复核 F1 子项）：ty75@floor<=50 引擎不 restep ⇒ 不移位、不计步。
                if !ty75_no_restep(input.floor, ty, cur.flags) {
                    cur.pos = (x, y);
                    cur.steps += 1;
                }
                i += 1;
            } else if let Some(rest) = tok.strip_prefix("door ") {
                let Some((x, y)) = parse_xy(rest) else { dead = Some(fail("door 坐标非法")); break; };
                let Some(slot) = cur.ents.iter().position(|e| {
                    e.x == x && e.y == y && entity_category_with_floor(e.ty, input.floor) == 1
                }) else { dead = Some(fail("开门格无门实体")); break; };
                let dty = cur.ents[slot].ty;
                let Some(np) = exact_open_door(&cur.player, dty) else { dead = Some(fail("无钥匙开门")); break; };
                cur.player = np;
                cur.ents.remove(slot);
                cur.pos = (x, y);
                cur.steps += 1;
                // 【S5 缺口6】Script 换层门（floor1 6,9=门+83）经 gate_arrive 返回
                // Terminal{..}——与 solver 出层边同语义（route 末尾跟 "gate x,y"
                // 注记 token），不是失败；只有 Dropped（Stalled）算死。
                if let Some(msg) = gate_ok(s.gate_arrive(&mut cur, x, y)) {
                    dead = Some(fail(msg));
                    break;
                }
                i += 1;
            } else if let Some(rest) = tok.strip_prefix("gatebump ") {
                let Some((x, y)) = parse_xy(rest) else { dead = Some(fail("gatebump 坐标非法")); break; };
                let Some(ns) = s.gate_bump(&cur, x, y) else {
                    dead = Some(fail("撞脚本门不成立（脚本拒/挂起/无参数）"));
                    break;
                };
                cur = ns; // 不移位、不计步（gate_bump 语义）
                i += 1;
            } else if tok.strip_prefix("gate ").is_some() {
                // Script 终止边注记（坐标=读回后 pos）——回放=无操作。
                i += 1;
            } else {
                let Some((x, y)) = parse_xy(tok) else { dead = Some(fail("坐标 token 非法")); break; };
                if (x, y) == cur.pos {
                    // 零位移**只允许 route 首位**：`route_gen` 约定首 token=入场格
                    // （`pos=entry`），属原位注记、无操作。mid-route 再出现同位置不是
                    // 注记而是状态漂移（会掩盖「上一步没落地」），必须 dead——旧实现对
                    // 任意重复位置静默 `i+=1`（S5 复核 F2 ②）。
                    if i == 0 {
                        i += 1;
                        continue;
                    }
                    dead = Some(fail("零位移只允许 route 首位（入场格 token）；mid-route 同位置无效"));
                    break;
                }
                if (x - cur.pos.0).abs() + (y - cur.pos.1).abs() != 1 {
                    dead = Some(fail("非邻接位移"));
                    break;
                }
                match s.classify(&cur, x, y) {
                    Cell::Pickup => {
                        let Some(ns) = s.pickup_step(&cur, x, y) else { dead = Some(fail("拾取不可实施")); break; };
                        cur = ns;
                        i += 1;
                    }
                    Cell::Move => {
                        cur.pos = (x, y);
                        cur.steps += 1;
                        if let Some(msg) = gate_ok(s.gate_arrive(&mut cur, x, y)) {
                            dead = Some(fail(msg));
                            break;
                        }
                        i += 1;
                    }
                    Cell::Shop(slot, ety) => {
                        // 两臂分支（买/走，同 token）——压栈回溯。
                        let mut arms: Vec<(St, usize)> = Vec::new();
                        // mid-route Terminal：两臂都不许进栈，最终报具名原因（不许被
                        // 「商店双臂皆不成立」掩盖）。
                        let mut mid_terminal = false;
                        // 臂1（走）：地形可走 ⇒ 原 Move 臂。
                        if s.walkable(x, y) {
                            let mut ns = cur.clone();
                            ns.pos = (x, y);
                            ns.steps += 1;
                            match gate_ok(s.gate_arrive(&mut ns, x, y)) {
                                None => arms.push((ns, i + 1)),
                                Some(msg) if msg.starts_with("Terminal") => mid_terminal = true,
                                Some(_) => {}
                            }
                        }
                        // 臂2（买）：开价门过 ∧ 买得起 ∧ 效果可实施。
                        if let Some((price, effect)) = s.shop_offer(&cur, ety, slot) {
                            if cur.player.gold >= price {
                                let mut ns = cur.clone();
                                ns.player.gold -= price;
                                if apply_shop_effect(&effect, &mut ns.player) {
                                    ns.ents.remove(slot);
                                    ns.pos = (x, y);
                                    ns.steps += 1;
                                    match gate_ok(s.gate_arrive(&mut ns, x, y)) {
                                        None => arms.push((ns, i + 1)),
                                        Some(msg) if msg.starts_with("Terminal") => mid_terminal = true,
                                        Some(_) => {}
                                    }
                                }
                            }
                        }
                        if arms.is_empty() {
                            dead = Some(if mid_terminal {
                                fail("Terminal 只允许出现在该层 route 末位（其后仅 gate 注记）")
                            } else {
                                fail("商店双臂皆不成立（不可走不可买）")
                            });
                            break;
                        }
                        branched = Some(arms);
                    }
                    _ => {
                        dead = Some(fail("该格需专属 token（门/怪/阻挡）"));
                        break;
                    }
                }
            }
            if let Some(arms) = branched {
                for a in arms {
                    stack.push(a);
                }
                dead = None;
                break; // 本帧让位给分支
            }
        }
        if let Some(e) = dead {
            // 只留最深失败（token_index 最大）——回放定位用帧号。
            let deeper = last_err.as_ref().map_or(true, |prev| e.token_index >= prev.token_index);
            if deeper {
                last_err = Some(e);
            }
            continue;
        }
        if i >= route.len() {
            survivors.push(cur);
        }
    }
    let (matched, _rejected): (Vec<St>, Vec<St>) = match expect {
        None => (survivors, Vec::new()),
        Some((ep, ef, ee, epos)) => {
            let mut m = Vec::new();
            let mut rj = Vec::new();
            for st in survivors {
                let ok = st.pos == epos
                    && st.player == *ep
                    && st.flags == ef
                    && st.ents.len() == ee.len()
                    && st.ents.iter().zip(ee.iter()).all(|(a, b)| a == b);
                if ok { m.push(st); } else { rj.push(st); }
            }
            (m, rj)
        }
    };
    if matched.len() == 1 {
        let st = &matched[0];
        return Ok(ExactReplay { player: st.player.clone(), flags: st.flags, ents: st.ents.clone(), pos: st.pos, steps: st.steps, kills: st.kills });
    }
    let reason = if matched.is_empty() {
        if expect.is_some() { "no_match（无分支与 witness 出层态全等）" } else { "dead_end（路线不可实施）" }
    } else if expect.is_some() {
        "expect_ambiguous（witness 记录不唯一，回放契约违例）"
    } else {
        "ambiguous（商店双臂同 token 且无期望态，不猜）"
    };
    Err(last_err.unwrap_or(RouteReplayError {
        token_index: route.len().saturating_sub(1),
        token: route.last().cloned().unwrap_or_default(),
        reason: reason.to_string(),
    }))
}

// ---------------------------------------------------------------- 批3 C1

/// 批3 C1 单元测试（`run_from`/终态 `St` 是私有面，集成测试（tests/）只够得着 pub API；
/// `script_abs.rs:565` 同法先例）。A9 面：本文件禁文件/网络/进程/时钟 I/O（判据 grep 的字面量不落注释——A9 零命中要求）。
/// 测试名统一带 `solve_global` 前缀（判据 `cargo test -p game-solver solve_global`
/// 存在性闸；批2 复核 Y2=过滤词漏配教训）。
#[cfg(test)]
mod solve_global_c1 {
    use super::*;
    use game_core::combat::scale_enemy_stats;

    /// C1 夹具：4×3 单走廊（y=1 行 (0..3,1) 可走）——start (0,1)、type41 怪挡道 (2,1)、
    /// Exit#1 type7 梯 (3,1)。与 `tests/solve_local.rs::fixture_3x3` 同构但加长一格：
    /// 自定义入场 (1,1) ≠ start，钉死「从入场格起搜」而非「仍从 start 起搜」。
    fn c1_input() -> SolveInput {
        let base = vec![EnemyDef {
            id: 41,
            hp: 35,
            atk: 18,
            def: 1,
            gold: 0,
            trait_flags: 0,
        }];
        // difficulty=0 ⇒ ×1（BR-4；tests fixture 同式 scale_enemy_stats(&base, 1)）
        let scaled = scale_enemy_stats(&base, 1);
        let level = LevelSpec {
            cols: 4,
            rows: 3,
            // 行主序：y=0 全墙、y=1 全可走、y=2 全墙
            terrain: vec![
                false, false, false, false, //
                true, true, true, true, //
                false, false, false, false,
            ],
            start: (0, 1),
            entities: vec![
                EntSpec {
                    ty: 41,
                    x: 2,
                    y: 1,
                    param: 0,
                },
                EntSpec {
                    ty: 7,
                    x: 3,
                    y: 1,
                    param: 2,
                },
            ],
            exits: vec![ExitSpec {
                x: 3,
                y: 1,
                kind: ExitKind::Stair,
                index: 1,
            }],
            player: PlayerSpec {
                hp: 300,
                atk: 10,
                def: 10,
                yellow: 0,
                blue: 0,
                red: 0,
                gold: 0,
                item_types: Vec::new(),
                item_uses: Vec::new(),
                item_stack_size: 0,
            },
            enemies: scaled,
            enemy_base: base,
            flags: 0,
        };
        SolveInput {
            floor: 1,
            difficulty: 0,
            level: Some(level),
        }
    }

    #[test]
    fn solve_global_c1_run_from自定义入场_初始位图_终态St() {
        let input = c1_input();
        let level = input.level.as_ref().unwrap();

        // (a) 怪存活：入场 (1,1) ≠ start (0,1)，走廊唯一通路必撞怪
        let (reach, goals) =
            Search::from_entry(&input, level, (1, 1)).run_root(
                (1, 1),
                sprite_full(level),
                level.player.clone(),
                level.flags,
                layer_node_limit(),
            );
        assert!(reach, "入场可达出口");
        let g = goals
            .iter()
            .find(|g| g.kind == ExitKind::Stair)
            .expect("必有 Exit#1 候选");
        assert_eq!(
            g.route,
            vec!["1,1", "bump 41@2,1", "3,1"],
            "路线从入场格起算（非 start）"
        );
        assert_eq!(g.steps, 2, "入场比 start 少一步 ⇒ entry 生效");
        assert_eq!(g.kills, 1, "走廊必撞怪 ⇒ 计 1 杀");
        assert_eq!(g.post.pos, (3, 1), "终态=出口格");
        assert_eq!(g.post.kills, 1, "终态 kills 与目标一致");
        assert!(
            !g.post.ents.iter().any(|r| r.ty == 41 && r.x == 2 && r.y == 1),
            "怪已杀 ⇒ 终态列表移除该记录：{:?}",
            g.post.ents
        );

        // (b) 预死怪位图（跨层继承：上一层已杀的怪不复活）⇒ 不计杀、位图保持
        let mut predead = sprite_full(level);
        predead.remove(0); // 预死=移除 sprite 首记录（type41）
        let (reach2, goals2) =
            Search::from_entry(&input, level, (1, 1)).run_root(
                (1, 1),
                predead.clone(),
                level.player.clone(),
                level.flags,
                layer_node_limit(),
            );
        assert!(reach2, "怪已死 ⇒ 走廊通");
        let g2 = goals2
            .iter()
            .find(|g| g.kind == ExitKind::Stair)
            .expect("必有 Exit#1 候选");
        assert_eq!(g2.kills, 0, "怪已死 ⇒ 不计杀（位图继承）");
        assert_eq!(g2.post.ents, predead, "终态列表=初始列表（无复活）");
        assert_eq!(g2.route, vec!["1,1", "2,1", "3,1"], "无怪 ⇒ 直穿");
        assert_eq!(g2.steps, 2, "两步走穿走廊");
    }
}

#[cfg(test)]
mod dominance_tests {
    use super::*;

    fn st(hp: i32, kills: i32, steps: i32, gold: i32) -> St {
        St {
            pos: (1, 1),
            player: PlayerSpec {
                hp,
                atk: 10,
                def: 10,
                yellow: 0,
                blue: 0,
                red: 0,
                gold,
                item_types: Vec::new(),
                item_uses: Vec::new(),
                item_stack_size: 0,
            },
            ents: Vec::new(),
            flags: 0,
            kills,
            steps,
        }
    }

    /// F2 红测试：同键下「钱多血少」不得被「钱少血多」判受支配。
    /// 反例即事故形态：把 800 金红钥匙唯一够得着的富态剪掉 ⇒ 误报「界内无解」。
    #[test]
    fn f2_rich_but_weaker_state_is_not_dominated() {
        let rich = st(100, 0, 0, 900);
        let poor = st(200, 0, 0, 10);
        assert!(!weakly_ge(&poor, &rich), "血多但穷的态不得吞掉血少但富的态");
        assert!(!dominates(&poor, &rich), "strict 支配同样不得跨金判定");
    }

    /// 金多且其余不劣 ⇒ 真支配（收紧方向，防「加了金维度就永远不支配」的偷懒修法）。
    #[test]
    fn f2_more_gold_with_no_worse_metrics_dominates() {
        let a = st(200, 1, 5, 900);
        let b = st(100, 2, 9, 10);
        assert!(weakly_ge(&a, &b));
        assert!(dominates(&a, &b));
    }

    /// 仅金币严格更优也应构成支配。
    #[test]
    fn f2_gold_only_strictness_counts() {
        let a = st(100, 0, 0, 500);
        let b = st(100, 0, 0, 499);
        assert!(dominates(&a, &b), "其余全等、金更多即严格更优");
        assert!(!dominates(&b, &a), "反向不成立");
    }

    /// 全等 ⇒ 弱支配成立（等价态可丢，保确定性）、strict 支配不成立。
    #[test]
    fn f2_equal_states_weakly_dominated_only() {
        let a = st(100, 0, 0, 50);
        let b = st(100, 0, 0, 50);
        assert!(weakly_ge(&a, &b));
        assert!(!dominates(&a, &b));
    }
}
// ---------------------------------------------------------------- D 面（子集支配）单测
/// 【D 面】纯加血拾取子集支配的判据测试（`pill_scan` 是**全部剪枝决策**所在，
/// 因此方向性判据直接打它；`plan_pill` 的闸门用注入证书打）。
///
/// 术语：`rem` = 剩余药位图（bit i = 第 i 颗入场药还没被捡）。支配要求
/// `rem(a) ⊇ rem(b)`（a 捡得更少），**反方向不成立**。
#[cfg(test)]
mod pill_subset_dominance_tests {
    use super::*;

    fn pill(ty: i32, x: i32, y: i32) -> EntRec {
        EntRec { ty, x, y, param: 0 }
    }

    fn pst(hp: i32, steps: i32, ents: Vec<EntRec>) -> St {
        St {
            pos: (3, 1),
            player: PlayerSpec {
                hp,
                atk: 10,
                def: 10,
                yellow: 0,
                blue: 0,
                red: 0,
                gold: 0,
                item_types: Vec::new(),
                item_uses: Vec::new(),
                item_stack_size: 0,
            },
            ents,
            flags: 0,
            kills: 0,
            steps,
        }
    }

    /// 判据 1：血更多且剩余药是超集 ⇒ 旧态支配新态（新态被剪）。
    ///
    /// 这条态对在引擎里**不可达**（无其它 hp 源时 hp 由已捡药唯一决定，见
    /// `pill_dominance_is_vacuous_without_independent_hp_source`），但它是本规则
    /// 的**语义定义**：方向钉死在 `pill_scan` 里，任何「剩药多就留」的粗暴改法都会
    /// 在这里翻车（判据 2）。
    #[test]
    fn pill_subset_dominance_prunes_dominated_state() {
        let a = pst(200, 0, vec![]);
        let b = pst(150, 0, vec![]);
        let mut budget = 8u64;
        // 旧态 a：hp 200、两颗药都还在（rem={A,B}）；新态 b：hp 150、只剩 A。
        let (dominated, kill) = pill_scan([(0u32, &a, 0b11u32)], &b, 0b01u32, &mut budget);
        assert!(dominated, "hp 更多且剩余药是超集 ⇒ 新态必须被剪");
        assert!(kill.is_empty(), "新态被剪时不该同时杀旧态");
        assert_eq!(budget, 7, "预算按实际比较对数扣减（可观测）");
    }

    /// 判据 2（反方向）：hp 更低但剩余药更多 ⇒ **不许**剪；hp 更高但剩余药更少 ⇒
    /// 同样**不许**剪。两个方向都钉住，规则才算「子集 + 血」双条件。
    ///
    /// 反例形态：把「还能回去捡药」的态剪掉 ⇒ 误丢最优 hp。
    #[test]
    fn pill_subset_dominance_keeps_better_hp() {
        // 对 1：血少但药多（(150,{A,B})） vs 血多但药少（(200,{A})）
        let more_pills = pst(150, 0, vec![]);
        let more_hp = pst(200, 0, vec![]);
        let mut budget = 8u64;
        // 旧=血多药少 先到，新=血少药多：hp 更低 ⇒ 不许剪新态
        let (dominated, kill) = pill_scan([(0u32, &more_hp, 0b01u32)], &more_pills, 0b11u32, &mut budget);
        assert!(!dominated, "hp 更低不得吞掉「还剩更多药」的态");
        assert!(kill.is_empty(), "旧态剩药更多 ⇒ 也不该被新态杀掉");

        // 旧=血少药多 先到，新=血多药少：旧态药更多 ⇒ 不许杀旧态
        let mut budget = 8u64;
        let (dominated, kill) = pill_scan([(0u32, &more_pills, 0b11u32)], &more_hp, 0b01u32, &mut budget);
        assert!(!dominated, "新态药更少 ⇒ 不能被旧态支配？反了：旧态 hp 低 ⇒ 不支配");
        assert!(kill.is_empty(), "旧态药更多 ⇒ 新态不得杀它");

        // 对照腿：血多且药多（(200,{A,B})）遇血少药少（(150,{A})）⇒ 后者被剪
        // （证明剪枝腿不是只读，只是**另两个方向**不成立）。
        let best = pst(200, 0, vec![]);
        let worst = pst(150, 0, vec![]);
        let mut budget = 8u64;
        let (dominated, kill) = pill_scan([(0u32, &best, 0b11u32)], &worst, 0b01u32, &mut budget);
        assert!(dominated, "血多且药多 ⇒ 新态（血少药少）被剪");
        assert!(kill.is_empty());
        let mut budget = 8u64;
        let (dominated, kill) = pill_scan([(0u32, &worst, 0b01u32)], &best, 0b11u32, &mut budget);
        assert!(!dominated);
        assert_eq!(kill, vec![0u32], "反过来：旧态（血少药少）可剪");
    }

    /// 判据 2 的第二形态：hp 相等、但剩余药不可比（各少一颗）⇒ 互不支配。
    #[test]
    fn pill_subset_dominance_needs_subset_not_just_count() {
        let a = pst(200, 0, vec![]);
        let b = pst(200, 0, vec![]);
        let mut budget = 8u64;
        let (dominated, kill) = pill_scan([(0u32, &a, 0b01u32)], &b, 0b10u32, &mut budget);
        assert!(!dominated, "rem 不可比 ⇒ 不支配");
        assert!(kill.is_empty());
    }

    /// 判据 2 的第三形态：steps 更差也不得被「血多药多」吞掉（`steps` 参与
    /// `Cost::HpLoss` 的 tie-break，见 `weakly_ge` 注释）。
    #[test]
    fn pill_subset_dominance_respects_step_budget() {
        let a = pst(200, 0, vec![]); // 旧态：步数 0
        let mut b = pst(150, 9, vec![]); // 新态：血少步多
        b.steps = 9;
        let mut budget = 8u64;
        let (dominated, _) = pill_scan([(0u32, &a, 0b11u32)], &b, 0b01u32, &mut budget);
        assert!(dominated, "旧态四项全不劣（血多/步少/药多）⇒ 新态被剪");
    }

    // ---- 闸门（fail-closed）----
    //
    // 夹具：13×3 走廊 y=1 行全可走（x=0..12），start=(0,1)，药挂在 (1,1)/(2,1)，
    // 出口梯 ty7 在 (12,1)。`floor=54` ⇒ `floor_tier=1` ⇒ ty31 +50 / ty32 +200。
    fn pill_input(pills: &[(i32, i32, i32)], hazards: &[(i32, i32)]) -> SolveInput {
        let cols = 13;
        let rows = 3;
        let mut terrain = vec![false; (cols * rows) as usize];
        for x in 0..cols {
            terrain[(1 * cols + x) as usize] = true;
        }
        let mut entities: Vec<EntSpec> = pills
            .iter()
            .map(|&(ty, x, y)| EntSpec { ty, x, y, param: 0 })
            .collect();
        for &(ty, x) in hazards {
            entities.push(EntSpec { ty, x, y: 1, param: 0 });
        }
        entities.push(EntSpec {
            ty: 7,
            x: cols - 1,
            y: 1,
            param: 2,
        });
        let level = LevelSpec {
            cols,
            rows,
            terrain,
            start: (0, 1),
            entities,
            exits: vec![ExitSpec {
                x: cols - 1,
                y: 1,
                kind: ExitKind::Stair,
                index: 1,
            }],
            player: PlayerSpec {
                hp: 300,
                atk: 10,
                def: 10,
                yellow: 0,
                blue: 0,
                red: 0,
                gold: 0,
                item_types: Vec::new(),
                item_uses: Vec::new(),
                item_stack_size: 0,
            },
            enemies: Vec::new(),
            enemy_base: Vec::new(),
            flags: 0,
        };
        SolveInput {
            floor: 54,
            difficulty: 0,
            level: Some(level),
        }
    }

    fn real_cert() -> impl Fn(i32, i32) -> Option<i32> {
        |ty, floor| game_core::effect_registry::certified_pure_hp_gain(ty, floor)
    }

    /// 判据 3：拿不到「本层固定增益」背书 ⇒ 整层退回通用搜索（`Off`），
    /// 且原因带触发类型（禁静默）。
    #[test]
    fn pill_plan_fails_closed_when_gain_not_certified() {
        let input = pill_input(&[(31, 1, 1)], &[]);
        let level = input.level.as_ref().expect("level");
        let search = Search::from_entry(&input, level, (0, 1));
        let root = St {
            pos: (0, 1),
            player: level.player.clone(),
            ents: sprite_full(level),
            flags: 0,
            kills: 0,
            steps: 0,
        };
        // 伪造「增益不固定」：证书对该型给不出常量 ⇒ 必须 fail-closed。
        let fake = |_ty: i32, _floor: i32| -> Option<i32> { None };
        match search.plan_pill(&root, &fake) {
            PillDom::Off(reason, ty) => {
                assert_eq!(reason, "hp_gain_not_certified", "原因须点名证书闸门");
                assert_eq!(ty, 31, "须带触发类型（可观测）");
            }
            PillDom::On(_) => panic!("证书给不出常量时不得启用"),
        }
        // 对照腿：真证书 ⇒ 启用（证明上面红的是证书闸门，不是别的闸门先挡）。
        match search.plan_pill(&root, &real_cert()) {
            PillDom::On(ord) => {
                assert_eq!(ord.gains.get(&31).copied(), Some(50), "f54 tier=1 ⇒ +50");
            }
            PillDom::Off(r, t) => panic!("真证书应启用，实际 Off({r},{t})"),
        }
    }

    /// 判据 3 的第二种 fail-closed：可踩的「踩格副作用臂」实体在可达格上 ⇒ 不启用
    /// （`pickup` 臂与 `Move` 臂的副作用集不同，模拟链不成立）。
    #[test]
    fn pill_plan_fails_closed_when_step_arm_hazard_reachable() {
        let input = pill_input(&[(31, 1, 1)], &[(61, 5)]);
        let level = input.level.as_ref().expect("level");
        let search = Search::from_entry(&input, level, (0, 1));
        let root = St {
            pos: (0, 1),
            player: level.player.clone(),
            ents: sprite_full(level),
            flags: 0,
            kills: 0,
            steps: 0,
        };
        match search.plan_pill(&root, &real_cert()) {
            PillDom::Off(reason, ty) => {
                assert_eq!(reason, "step_arm_hazard_reachable");
                assert_eq!(ty, 61);
            }
            PillDom::On(_) => panic!("有可达陷阱时不得启用（拾取臂与 Move 臂副作用不同）"),
        }
    }

    /// 【定理·代码化】无「独立 hp 源」（无陷阱/宝箱/组门/脚本/商店）时，
    /// hp 由**已捡药集合**唯一决定 ⇒ 本规则**恒剪 0 条**（f54 正是这种层）。
    ///
    /// 证明：hp = hp₀ + Σ_{已捡} gain，gain>0 ⇒ `rem(a) ⊇ rem(b)` ∧
    /// `hp(a) ≥ hp(b)` ⇒ 已捡 a ⊆ 已捡 b 且 Σ_a ≥ Σ_b ⇒ Σ_a = Σ_b ⇒ 已捡相等
    /// ⇒ `ents` 相等 ⇒ 已被**精确键**块（在 D 块之前）处理。
    #[test]
    fn pill_dominance_is_vacuous_without_independent_hp_source() {
        let input = pill_input(&[(31, 1, 1), (32, 2, 1)], &[]);
        let level = input.level.as_ref().expect("level");
        let search = Search::from_entry(&input, level, (0, 1));
        let (_, _goals) = search.run_root_with(
            (0, 1),
            sprite_full(level),
            level.player.clone(),
            0,
            10_000,
            &real_cert(),
        );
        let st = pill_dom_stat();
        assert!(st.enabled, "纯加血拾取层应启用：{st:?}");
        assert_eq!(st.pill_types, 2, "ty31/ty32 两颗型：{st:?}");
        assert!(st.cmp > 0, "规则必须真的比过候选（否则等于没接线）：{st:?}");
        assert_eq!(st.pruned_new, 0, "无独立 hp 源 ⇒ 恒剪 0（定理）：{st:?}");
        assert_eq!(st.killed_old, 0, "同上：{st:?}");
    }

    /// 判据 4（层内黑盒差分）：同一层、同一入场态，**启用** vs **关闭**本规则，
    /// 搜索输出必须逐字段相同。
    ///
    /// 「关闭」= 注入一条恒 `None` 证书 ⇒ `plan_pill` 返回 `Off` ⇒ 该层走的正是
    /// 本规则引入前的代码路径（旧行为）。因此这是一次真正的旧/新差分。
    #[test]
    fn pill_dominance_off_matches_off_path_field_by_field() {
        let input = pill_input(&[(31, 1, 1), (32, 2, 1), (31, 4, 1)], &[]);
        let level = input.level.as_ref().expect("level");
        let search = Search::from_entry(&input, level, (0, 1));
        let ents = sprite_full(level);
        let (on_reach, on_goals) = search.run_root_with(
            (0, 1),
            ents.clone(),
            level.player.clone(),
            0,
            10_000,
            &real_cert(),
        );
        let on_stat = pill_dom_stat();
        let (off_reach, off_goals) = search.run_root_with(
            (0, 1),
            ents,
            level.player.clone(),
            0,
            10_000,
            &|_t: i32, _f: i32| None,
        );
        let off_stat = pill_dom_stat();
        assert!(on_stat.enabled, "On 腿必须启用：{on_stat:?}");
        assert!(!off_stat.enabled, "Off 腿必须关闭（对照）：{off_stat:?}");
        assert_eq!(on_reach, off_reach, "reachable 必须相同");
        assert_eq!(on_goals.len(), off_goals.len(), "候选数必须相同");
        for (a, b) in on_goals.iter().zip(off_goals.iter()) {
            assert_eq!(a.hp, b.hp, "hp");
            assert_eq!(a.kills, b.kills, "kills");
            assert_eq!(a.steps, b.steps, "steps");
            assert_eq!(a.keys, b.keys, "keys");
            assert_eq!(a.kind, b.kind, "kind");
            assert_eq!(a.route, b.route, "route");
            assert_eq!(a.cross_edge, b.cross_edge, "cross_edge");
            assert_eq!(a.post.pos, b.post.pos, "post.pos");
            assert_eq!(a.post.ents, b.post.ents, "post.ents");
        }
        // On 腿的 hp 必须等于闭式最优（三颗药全捡：50+200+50）
        let best = on_goals.iter().map(|g| g.hp).max().expect("有解");
        assert_eq!(best, 300 + 50 + 200 + 50, "闭式最优：入场 300 + 全部药 300");
    }
}

// ================================================================== 【P10】出层分层软契约单测
#[cfg(test)]
mod p10_objective_pick {
    use super::*;
    use crate::objective::{cmp_exit, score_of_goal};
    use std::cmp::Ordering;

    /// LayerGoal 构造助手（只剩比较面相关字段有实义）。
    fn goal(hp: i32, gold: i32, steps: i32, route: &[&str]) -> LayerGoal {
        LayerGoal {
            hp,
            kills: 0,
            steps,
            keys: (0, 0, 0),
            kind: ExitKind::Stair,
            route: route.iter().map(|s| s.to_string()).collect(),
            post: PlayerSpec {
                hp,
                atk: 0,
                def: 0,
                yellow: 0,
                blue: 0,
                red: 0,
                gold,
                item_types: Vec::new(),
                item_uses: Vec::new(),
                item_stack_size: 0,
            },
            post_pos: (0, 0),
            post_ents: Vec::new(),
            post_flags: 0,
            cross_edge: None,
        }
    }

    /// 同 hp ⇒ gold 多者赢（gold 软约束层 2；变异 M2「gold 提到 hp 前」会在此红）。
    #[test]
    fn p10_same_hp_prefers_more_gold() {
        let a = goal(4027, 500, 90, &["0,0"]);
        let b = goal(4027, 100, 88, &["0,0"]);
        assert_eq!(cmp_exit(&score_of_goal(&a), &score_of_goal(&b)), Ordering::Less);
        assert!(better_exit(&a, &b), "pick_exit/better_exit 必须选金多的");
    }

    /// 血大者永远赢（gold 不许翻盘；hp 软约束层 1）。
    #[test]
    fn p10_hp_beats_gold() {
        let a = goal(4100, 0, 90, &["0,0"]);
        let b = goal(4000, 9999, 88, &["0,0"]);
        assert!(better_exit(&a, &b), "血优先于任何 gold 优势");
        assert!(!better_exit(&b, &a));
    }

    /// 同 hp 同 gold 同 steps ⇒ 路线字典序（决定性 tie-break，契约外不改序）。
    #[test]
    fn p10_equal_uses_route_lex() {
        let a = goal(4027, 500, 88, &["0,0"]);
        let b = goal(4027, 500, 88, &["1,0"]);
        assert!(better_exit(&a, &b));
        assert!(!better_exit(&b, &a));
    }

    /// pick_exit argbest 面：多目标里取比较序最优者。
    #[test]
    fn p10_pick_exit_argbest() {
        let goals = vec![goal(4027, 100, 88, &["0,0"]), goal(4027, 500, 90, &["1,0"]), goal(4000, 9999, 10, &["2,0"])];
        let best = pick_exit(&goals).expect("非空目标集必有最优");
        assert_eq!(best.post.gold, 500, "同 hp 里金多者胜（hp=4000 的金主不许翻盘）");
    }
}
