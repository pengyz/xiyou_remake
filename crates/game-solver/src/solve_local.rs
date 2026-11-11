//! 【求解器批2 C1-C4】单层求解 `solve_local`（提案 §5：C1 骨架+子命令壳 / C2 可达性+战斗 /
//! C3 目标双查询 / C4 开门+钥匙经济）。
//!
//! **纯度边界**（`lib.rs` 契约 + 计划 :456 A9）：本模块=纯函数——输入 `SolveInput`
//! （由 bin/测试装配），输出 `SolveResult`；**禁文件/网络/时钟/进程 I/O**、禁 GameRng/Clock。
//! 后续切片：C5 脚本 → C6 floor1 端到端 → C7 对拍回归。
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
//! - 搜索编排/去重支配/目标/路线序列化 = 本模块自算（§1.3 白名单）。
use crate::state::s0;
use game_core::combat::PlayerCombat;
use game_core::pathfind::{find_path, Walkability};
use game_core::subsystem::battle::{
    predict_engagement, BattleCtx, BattleEvent, BattleOutcome, BattleResolver, BattleSubsystem,
    EnemyTypeId,
};
use game_core::subsystem::script_host::entity_category_with_floor;
use game_data::defs::EnemyDef;
use std::collections::{BTreeMap, VecDeque};

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
}

/// 实体夹具记录（type_id + 格坐标 + 场景参数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntSpec {
    pub ty: i32,
    pub x: i32,
    pub y: i32,
    pub param: i32,
}

/// 出口类别（批2=Exit#1 步行楼梯；Exit#2 脚本换层 C5 接入）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitKind {
    Stair,
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
        confidence: "feasible",
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
    // 地形（行主序；索引式与 route_gen.py:52 `wk` 同源）
    let mut terrain = vec![false; (cols * rows) as usize];
    for y in 0..rows {
        for x in 0..cols {
            let idx = ((y * cols + x) * 2) as usize;
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

// ---------------------------------------------------------------- 搜索态

/// 搜索态（提案 §1 状态 s 的批2 切面：hp/kills 为支配维度，不进去重键）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct St {
    pos: (i32, i32),
    player: PlayerSpec,
    /// 存活位图（槽位=`LevelSpec.entities` 下标；层内单调）
    ents: Vec<u64>,
    /// `script_line_flags` 位图（C5 脚本门用；批2 恒 0）
    flags: u64,
    kills: i32,
    steps: i32,
}

/// 去重键 K₀（提案 §4.1：pos, ents_digest, flags, keys, bag, atk, def；
/// hp/kills/steps/gold **不进键**）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    pos: (i32, i32),
    ents: Vec<u64>,
    flags: u64,
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
fn dominates(a: &St, b: &St) -> bool {
    let ge_hp = a.player.hp >= b.player.hp;
    let le_kills = a.kills <= b.kills;
    let le_steps = a.steps <= b.steps;
    let strict = a.player.hp > b.player.hp || a.kills < b.kills || a.steps < b.steps;
    ge_hp && le_kills && le_steps && strict
}

/// 动作（路线重建用；`Move` 的目标格=节点自身 pos）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Act {
    Start,
    Move,
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
}

fn all_alive(n: usize) -> Vec<u64> {
    let mut v = vec![0u64; n.div_ceil(64)];
    for i in 0..n {
        v[i / 64] |= 1u64 << (i % 64);
    }
    v
}

fn alive(v: &[u64], slot: usize) -> bool {
    (v[slot / 64] >> (slot % 64)) & 1 == 1
}

fn clear_alive(v: &mut [u64], slot: usize) {
    v[slot / 64] &= !(1u64 << (slot % 64));
}

struct Search<'a> {
    input: &'a SolveInput,
    level: &'a LevelSpec,
    /// `find_path` 可达子图（直调 `pathfind.rs:55`；BR-3 怪癖只对拍 found）
    sub: Vec<bool>,
}

impl<'a> Search<'a> {
    fn new(input: &'a SolveInput, level: &'a LevelSpec) -> Self {
        Search {
            input,
            level,
            sub: subgraph(level),
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
        let mut blocked = false;
        let mut stair_like = false;
        for (slot, e) in self.level.entities.iter().enumerate() {
            if e.x != x || e.y != y || !alive(&st.ents, slot) {
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
                    // 炼丹台/真火/墙/撞击墙/81
                    9 | 10 | 11 | 12 | 81 => blocked = true,
                    // 楼梯族（bump 显形/楼梯计数豁免地形终审）
                    6 | 7 | 8 => stair_like = true,
                    // 5 宝箱 / 83 场景门 / 76|82 组门：bump 放行
                    _ => {}
                },
                // 2|4 道物、0/16/32 装饰：可走
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
        match monster {
            Some((slot, ty)) => Cell::Monster(slot, ty),
            None => Cell::Move,
        }
    }

    /// 后继生成（solver 自算=搜索编排白名单；转移本体全为 §1.3 直调）。
    fn expand(&self, st: &St, out: &mut Vec<(Act, St)>) {
        for &(dx, dy) in &DIRS {
            let (tx, ty) = (st.pos.0 + dx, st.pos.1 + dy);
            if tx < 0 || ty < 0 || tx >= self.level.cols || ty >= self.level.rows {
                continue;
            }
            if !self.sub[(ty * self.level.cols + tx) as usize] {
                continue;
            }
            match self.classify(st, tx, ty) {
                Cell::Blocked => {}
                Cell::Move => {
                    let mut ns = st.clone();
                    ns.pos = (tx, ty);
                    ns.steps += 1;
                    out.push((Act::Move, ns));
                }
                Cell::Monster(slot, ety) => {
                    let player = st.player.to_combat();
                    // Refused ⇒ ⊥（不产生状态、不计杀，提案 §1.1）
                    if let Some(loss) = bump_outcome(self.level, &player, ety, slot) {
                        let mut ns = st.clone();
                        ns.player.hp -= loss;
                        ns.kills += 1;
                        clear_alive(&mut ns.ents, slot);
                        ns.pos = (tx, ty);
                        ns.steps += 1;
                        out.push((Act::Bump(ety, tx, ty), ns));
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
                        clear_alive(&mut ns.ents, slot);
                        ns.pos = (tx, ty);
                        ns.steps += 1;
                        out.push((Act::Door(tx, ty), ns));
                    }
                }
            }
        }
    }

    /// 搜索主循环：BFS（FIFO）+ K₀ 去重 + Pareto 支配剪枝。
    /// 返回（任一出口可达, 全部非支配终态候选——双查询取解共用，§1.4）。
    fn run(&self) -> (bool, Vec<Goal>) {
        let n = self.level.entities.len();
        let root = St {
            pos: self.level.start,
            player: self.level.player.clone(),
            ents: all_alive(n),
            flags: 0,
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

        while let Some(id) = queue.pop_front() {
            if pruned[id as usize] {
                continue;
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
                });
                continue;
            }
            let mut succ = Vec::new();
            self.expand(&st, &mut succ);
            for (act, ns) in succ {
                let k = key_of(&ns);
                let list = by_key.entry(k).or_default();
                let mut dominated = false;
                let mut kill: Vec<u32> = Vec::new();
                for &eid in list.iter() {
                    let e = &nodes[eid as usize].st;
                    if dominates(e, &ns) {
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
            }
        }

        // C3：reachable = 任一出口可达；取解（max_hp/min_kills）由 Cost 泛型在
        // goals 上分叉（§1.4 双查询独立出解）。
        let reachable = !goals.is_empty();
        (reachable, goals)
    }
}

/// `find_path` 可达子图（§1.3 可达=find_path 直调）：从 start 到每格 `found`；
/// `sub[start]=true` 强制。过度近似安全——我方动作的可通行语义 ⊆
/// `is_cell_walkable || can_path_through_entity`。
fn subgraph(level: &LevelSpec) -> Vec<bool> {
    let cols = level.cols;
    let rows = level.rows;
    let terrain: Vec<Vec<bool>> = (0..rows)
        .map(|y| {
            (0..cols)
                .map(|x| level.terrain[(y * cols + x) as usize])
                .collect()
        })
        .collect();
    let has_slot = |x: i32, y: i32| -> bool {
        level
            .entities
            .iter()
            .any(|e| e.x == x && e.y == y)
    };
    let walk = Walkability {
        terrain,
        has_entity_slot: &has_slot,
        cols,
        rows,
    };
    let mut sub = vec![false; (cols * rows) as usize];
    for y in 0..rows {
        for x in 0..cols {
            sub[(y * cols + x) as usize] = find_path(&walk, level.start.0, level.start.1, x, y).found;
        }
    }
    let s = (level.start.1 * cols + level.start.0) as usize;
    if s < sub.len() {
        sub[s] = true;
    }
    sub
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
            Act::Start | Act::Move => format!("{},{}", pos.0, pos.1),
            Act::Bump(ty, x, y) => format!("bump {ty}@{x},{y}"),
            Act::Door(x, y) => format!("door {x},{y}"),
        })
        .collect()
}
