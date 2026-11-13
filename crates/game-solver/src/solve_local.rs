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
use game_data::defs::EnemyDef;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};

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
    /// 三态置信（计划 :448 A1）：UseItem 裁出 ⇒ 批2 恒 `bounded-best`（BR-7）
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
            confidence: "feasible",
            route: Vec::new(),
            route_min_kills: Vec::new(),
            keys_used: (0, 0, 0),
        };
    };
    let search = Search::new(input, level);
    let (reachable, goals) = search.run();
    let hp_goal = pick::<HpLoss>(&goals);
    let kill_goal = pick::<Kills>(&goals);
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
        // BR-7/§64：批2 单层版策略空间受限但空间内最优 ⇒ 一律 bounded-best，不冒充 optimal
        confidence: "bounded-best",
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
    LAYER_TRUNC.with(|f| f.set(None));
    let Some(level) = input.level.as_ref() else {
        return (false, Vec::new());
    };
    let root_ents = match ents {
        Some(v) => v.to_vec(),
        None => sprite_full(level),
    };
    // C5d″：层结果缓存 lookup——全局队列里同 K₀ 不同 kills/steps 的变体会以完全
    // 相同输入反复 pop 同一层（f52 实测同输入 8 连 pop、单层状态 80 万）。
    let memo_key = (
        input.floor,
        entry,
        player.clone(),
        flags,
        root_ents.clone(),
        level_digest(input),
    );
    if let Some((hit_ok, hit_goals, hit_trunc)) =
        LAYER_MEMO.with(|m| m.borrow().get(&memo_key).cloned())
    {
        LAYER_TRUNC.with(|f| f.set(hit_trunc)); // memo 命中回放截断标记（可观测性不变）
        return (hit_ok, hit_goals);
    }
    let mut search = Search::from_entry(input, level, entry);
    search.cloud = true; // 批3 C3b：层间路径开筋斗云候选（批2 solve_local/run 关）
    let (reachable, goals) = search.run_root(entry, root_ents, player, flags);
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
    // C5d″：纯函数结果入缓存（键=全部输入；gate memo 只提速不改输出）；
    // 截断标记随结果入缓存（同输入⇒同截断⇒确定性）
    LAYER_MEMO.with(|m| {
        m.borrow_mut().insert(
            memo_key,
            (reachable, mapped.clone(), LAYER_TRUNC.with(|f| f.get())),
        );
    });
    (reachable, mapped)
}

thread_local! {
    /// C5d″：层求解结果缓存——`solve_layer` 是（层规格+入场态）的纯函数 ⇒ 同键同
    /// 输出。全局队列中同 K₀ 不同 kills/steps 的状态以相同输入反复 pop 同层
    /// （f52 实测同输入 8 连 pop、单层 80 万状态、单层秒级）——缓存=跳过重复层搜索。
    /// 键含 `level_digest`（FNV-1a 层规格内容摘要）防「同 floor 不同夹具」串味
    /// （玩具测试同 floor 多套 LevelSpec）；thread_local=多线程测试隔离；缓存纯
    /// 加速、不改输出与遍历序（确定性不受影响）。
    static LAYER_MEMO: RefCell<
        HashMap<
            (i32, (i32, i32), PlayerSpec, u128, Vec<EntRec>, u64),
            (bool, Vec<LayerGoal>, Option<(i32, usize)>),
        >,
    > = RefCell::new(HashMap::new());
    /// R1 层界截断标记（lead 裁定 A 2026-11-13 条件1「截断可观测」）：
    /// `Some((floor, nodes))`=该层在 nodes 达上限时截断。solve_layer 入口清零、
    /// run_root 触界置位、memo 命中按缓存值回放 ⇒ 全局层每调一次都可观测。
    static LAYER_TRUNC: std::cell::Cell<Option<(i32, usize)>> = std::cell::Cell::new(None);
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

/// 取走并清零层截断标记（solve_global 消费；见 LAYER_TRUNC doc）。
pub(crate) fn take_layer_trunc() -> Option<(i32, usize)> {
    LAYER_TRUNC.with(|f| f.get())
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    pos: (i32, i32),
    ents: Vec<EntRec>,
    flags: u128,
    keys: (i32, i32, i32),
    bag: (Vec<u8>, Vec<u8>, i32),
    atk: i32,
    def: i32,
}

fn key_of(st: &St) -> Key {
    Key {
        pos: st.pos,
        ents: st.ents.clone(),
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

/// 支配判定（§4.1：hp₁≥hp₂ ∧ kills₁≤kills₂ ∧ steps₁≤steps₂ 且至少一严格 ⇒ 丢 s₂）。
/// 弱支配（含相等）：同 K₀ 下 e 各指标不劣于 ns 即丢 ns（等价态=丢）。
/// 与 solve_global::push_state 的 weakly_ge 同口径（C3b bounce 环修）。
fn weakly_ge(a: &St, b: &St) -> bool {
    a.player.hp >= b.player.hp && a.kills <= b.kills && a.steps <= b.steps
}

fn dominates(a: &St, b: &St) -> bool {
    let ge_hp = a.player.hp >= b.player.hp;
    let le_kills = a.kills <= b.kills;
    let le_steps = a.steps <= b.steps;
    let strict = a.player.hp > b.player.hp || a.kills < b.kills || a.steps < b.steps;
    ge_hp && le_kills && le_steps && strict
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

/// 代价泛型（§5 C3「同骨架两代价泛型」+ §1.4 双查询独立出解）。
/// 支配剪枝与代价无关 ⇒ 搜索骨架只跑一次，代价只在取解时分叉。
trait Cost {
    /// 量化域（HpLoss：全部出口；Kills：仅 Exit#1 步行楼梯，§1.4 ②）
    fn eligible(kind: ExitKind) -> bool;
    /// 优劣：true = a 优于 b（并列取步数少者、再并列字典序最小路线，§3.3）
    fn better(a: &Goal, b: &Goal) -> bool;
}

/// Q1：max_hp（域=全部合法路，含 Exit#2；§1.4 ①）
struct HpLoss;

impl Cost for HpLoss {
    fn eligible(_kind: ExitKind) -> bool {
        true
    }
    fn better(a: &Goal, b: &Goal) -> bool {
        if a.hp != b.hp {
            a.hp > b.hp
        } else if a.steps != b.steps {
            a.steps < b.steps
        } else {
            a.route < b.route
        }
    }
}

/// Q2：min_kills（域=终态 ∈ Exit#1；§1.4 ②）
struct Kills;

impl Cost for Kills {
    fn eligible(kind: ExitKind) -> bool {
        matches!(kind, ExitKind::Stair)
    }
    fn better(a: &Goal, b: &Goal) -> bool {
        if a.kills != b.kills {
            a.kills < b.kills
        } else if a.steps != b.steps {
            a.steps < b.steps
        } else {
            a.route < b.route
        }
    }
}

/// 取解：域内 argbest（None = 该域无解；`reachable` 仍由「任一出口」决定）。
fn pick<C: Cost>(goals: &[Goal]) -> Option<&Goal> {
    let mut best: Option<&Goal> = None;
    for g in goals.iter().filter(|g| C::eligible(g.kind)) {
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
enum Cell {
    Blocked,
    Move,
    Monster(usize, i32), // 槽位, type_id
    /// 黄/红/蓝门（C4：expand 内直调 `consume_key_for_door`；未消耗 ⇒ ⊥）
    Door(usize, i32), // 槽位, type_id
    /// 拾取实体（C6：cat 2|4 道物，`apply_step_cell_effects` `scene.rs:4093-4096`
    /// 拾取+移除同拍——expand 内 harness `try_step` 落格触发，读回 player keys/实体移除）
    Pickup, // cat 2|4 道物（拾取面=该格全部存活 cat2|4，expand 内 harness 直调）
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
        }
    }

    fn walkable(&self, x: i32, y: i32) -> bool {
        self.level.terrain[(y * self.level.cols + x) as usize]
    }

    /// 格分类（`scene.rs interact_with_cell` 的类目臂归约，提案 §6-7 单源：
    /// `entity_category_with_floor` 层覆写表；地形终审=楼梯豁免 `stair_count>0`）。
    fn classify(&self, st: &St, x: i32, y: i32) -> Cell {
        let mut monster: Option<(usize, i32)> = None;
        let mut door: Option<(usize, i32)> = None;
        let mut pickup = false;
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
                2 | 4 => pickup = true,
                // 32 联动门（type72/84/87）：interact 面恒阻挡（a.java:5527 /
                // scene.rs:3664 `32 => allowed = false`）。floor1 上 type72 无层覆写
                // （sprite_defs floor_category_override 仅 floor50→cat8，
                // evidence a.java:6292 render_category=32、a.java:9681 default_category=32）
                // ⇒ 撞前不可达。0/16 装饰：可走
                32 => blocked = true,
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
        // 怪/拾取并存（批2 夹具无）：战斗优先——打赢后落格才拾取（引擎 interact→apply 时序）
        match (monster, pickup) {
            (Some((slot, ty)), _) => Cell::Monster(slot, ty),
            (None, true) => Cell::Pickup,
            (None, false) => Cell::Move,
        }
    }

    /// 后继生成（solver 自算=搜索编排白名单；转移本体全为 §1.3 直调）。
    /// 第三元=到达格 StepGate 结果（C5；仅 Move/Door 到达面——踩格触发）。
    fn expand(&self, st: &St, out: &mut Vec<(Act, St, GateArrive)>) {
        for &(dx, dy) in &DIRS {
            let (tx, ty) = (st.pos.0 + dx, st.pos.1 + dy);
            if tx < 0 || ty < 0 || tx >= self.level.cols || ty >= self.level.rows {
                continue;
            }
            // C5d：不再按起点连通分量剪枝——走步邻接天然限分量（对纯走步是
            // 空操作），而门脚本 MOV_0 传送落点可跨分量（静态过滤会拦死）。
            match self.classify(st, tx, ty) {
                Cell::Blocked => {}
                Cell::Move => {
                    let mut ns = st.clone();
                    ns.pos = (tx, ty);
                    ns.steps += 1;
                    let g = self.gate_arrive(&mut ns, tx, ty);
                    out.push((Act::Move(tx, ty), ns, g));
                }
                Cell::Monster(slot, ety) => {
                    let player = st.player.to_combat();
                    // Refused ⇒ ⊥（不产生状态、不计杀，提案 §1.1）
                    if let Some(loss) = bump_outcome(self.level, &player, ety, slot) {
                        let mut ns = st.clone();
                        ns.player.hp -= loss;
                        ns.kills += 1;
                        ns.ents.remove(slot);
                        ns.pos = (tx, ty);
                        ns.steps += 1;
                        // 批3 C2 终局边：floor50 击败型72 → Ending（钩子
                        // scene.rs:3891-3894 → load_level_script(65) → END_0；
                        // floor50 cat8 覆写=sprite_defs floor50→cat8 ⇒ 可 bump，
                        // 其它层型72=cat32 恒阻挡进不了本臂）。语义=游戏结束
                        // ⇒ 记候选不入队（run_root 的 Ending 臂，同 Terminal 口径）。
                        // 怪格不建模 StepGate（batch2 夹具无共存；报审注明）
                        let gate = if self.input.floor == 50 && ety == 72 {
                            GateArrive::Ending
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

    /// 搜索主循环：BFS（FIFO）+ K₀ 去重 + Pareto 支配剪枝。
    /// 返回（任一出口可达, 全部非支配终态候选——双查询取解共用，§1.4）。
    fn run(&self) -> (bool, Vec<Goal>) {
        self.run_root(
            self.level.start,
            sprite_full(self.level),
            self.level.player.clone(),
            self.level.flags,
        )
    }

    /// 批3 C1：任意入场格 + 初始存活位图起搜（层间边传播：入场格=上一层出场梯的
    /// 落点、活体列表=上一层终态——批2 `run` 的 `start`/sprite 全活特例化）。
    /// 批3 C2：玩家/flags 也由调用方注入（层间携带上一层 post；批2 的
    /// `level.player`/`level.flags` 特例化为 `run` 的两参转发）。
    fn run_root(
        &self,
        entry: (i32, i32),
        root_ents: Vec<EntRec>,
        player: PlayerSpec,
        flags: u128,
    ) -> (bool, Vec<Goal>) {
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
        let mut goals: Vec<Goal> = Vec::new();
        // 批3 C3b：根态云候选（真数据重入场携15 的回程通道；玩具根态通常不邻锚）
        self.record_cloud(&mut goals, &nodes, 0);
        // 批3 F3：根态 UseItem 符候选（真数据 S0 无符；测试/特殊起始携符时生效）
        self.record_useitem(&mut goals, &nodes, 0);

        while let Some(id) = queue.pop_front() {
            if pruned[id as usize] {
                continue;
            }
            if nodes.len() >= layer_node_limit() {
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
                if id != 0 {
                    // 批3 C6g：根态恰在出口（step_off 四邻皆墙留梯格等）记录后仍须
                    // 扩展——否则层搜索饿死（f10 entry(1,11) 实测 goals=1）；
                    // 中途到达出口维持批2 语义=记目标不扩展。
                    continue;
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
                    // C3b/F3 候选口径与批2 入队态一致（出口态此前入队=有记录点）
                    self.record_cloud(&mut goals, &nodes, new_id);
                    self.record_useitem(&mut goals, &nodes, new_id);
                    continue;
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
                if dominated {
                    continue;
                }
                list.retain(|x| !kill.contains(x));
                for &eid in &kill {
                    pruned[eid as usize] = true;
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
                // 批3 C3b：过支配实际入队才记云候选（被支配/重复态不记——
                // 同 K₀ 优态已记过其云边）
                self.record_cloud(&mut goals, &nodes, new_id);
                // 批3 F3：过支配入队态才记 UseItem 候选（同上口径）
                self.record_useitem(&mut goals, &nodes, new_id);
            }
        }

        // C3：reachable = 任一出口可达；取解（max_hp/min_kills）由 Cost 泛型在
        // goals 上分叉（§1.4 双查询独立出解）。
        let reachable = !goals.is_empty();
        (reachable, goals)
    }

    /// 批3 C3b：筋斗云换层候选（四段链 a.java:5738-5753 tutorialSeen→f_bool_13
    /// →键49/55（a.java:5295-5306）→findFloorGateEntity（a.java:7115-7133 首个
    /// `param>>9==0` 的 type7/8）→changeFloor(..., strict=true)（a.java:7135-7178））。
    /// tutorialSeen≡背包含15（唯一写点 pickup case15 a.java:7539）；f_bool_13=
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
        for i in 0..sc.script_line_flags.len().min(128) {
            if sc.script_line_flags[i] {
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
    for (i, f) in sc.script_line_flags.iter_mut().enumerate() {
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
            if !(sc.script_walk_armed && !landed) {
                return Drive::Completed; // 行尾收尾 transition_to(Map)：演出完成、玩家留格（C5a 续走）
            }
        }
        sc.key_value = 1;
        sc.tick(ft, &wt);
        ft += 1;
        let sig = format!(
            "{}/{}/{}/{}/{}",
            sc.dialog.dialog_phase,
            sc.script_walk_armed,
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
    for i in 0..sc.script_line_flags.len().min(128) {
        if sc.script_line_flags[i] {
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
