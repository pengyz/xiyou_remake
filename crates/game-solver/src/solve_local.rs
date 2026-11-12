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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub post_ents: Vec<u64>,
    /// 终态 `script_line_flags` 位图（下一层注入 `flags`）
    pub post_flags: u128,
}

/// 批3 C2 单层求解门面（`solve_local`↔`solve_global` 唯一接口；`Search` 保持私有）。
///
/// - `entry`：入场格（批2 特例=`level.start`；层间边=上一层出场梯落点，
///   `swap_floor` 语义 `scene_persist.rs:410-433`）；
/// - `ents`：初始存活位图；`None` ⇒ `all_alive`（新层首入）；
/// - `player`/`flags`：注入的入场玩家状态与脚本旗标（层间携带上一层 `post`；
///   `run()` 特例=`level.player`/`level.flags`）。
///
/// 返回 `(任一层目标可达, 全部非支配层目标)`；`level=None` ⇒ `(false, vec![])`。
pub fn solve_layer(
    input: &SolveInput,
    entry: (i32, i32),
    ents: Option<&[u64]>,
    player: PlayerSpec,
    flags: u128,
) -> (bool, Vec<LayerGoal>) {
    let Some(level) = input.level.as_ref() else {
        return (false, Vec::new());
    };
    let root_ents = match ents {
        Some(v) => v.to_vec(),
        None => all_alive(level.entities.len()),
    };
    let search = Search::from_entry(input, level, entry);
    let (reachable, goals) = search.run_root(entry, root_ents, player, flags);
    let mapped = goals
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
        })
        .collect();
    (reachable, mapped)
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

// ---------------------------------------------------------------- 搜索态

/// 搜索态（提案 §1 状态 s 的批2 切面：hp/kills 为支配维度，不进去重键）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct St {
    pos: (i32, i32),
    player: PlayerSpec,
    /// 存活位图（槽位=`LevelSpec.entities` 下标；层内单调）
    ents: Vec<u64>,
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
    ents: Vec<u64>,
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
    /// 批3 C1：终态全态（层间传播用——pos/ents 位图/背包/atk,def/flags 继承到下一层）
    post: St,
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
    /// 门命中且脚本换层终止边（harness 读 `persist.pending_floor`≠0，§2.2 ④）
    Terminal,
    /// 门命中但非终态完成或驱动停摆 ⇒ ⊥（C5 口径：批2 夹具不触发，报审注明；
    /// T6 对 floor1 reachable=true 兜底——若真触发会即刻暴露）
    Dropped,
    /// 批3 C2 终局边（通关）：floor50 击败型72 的 bump 后继——游戏到此结束
    /// （Ending 非换层、非 StepGate，由 expand 的怪臂直接产出，不经 gate_arrive）。
    Ending,
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
        Self::from_entry(input, level, level.start)
    }

    /// 批3 C1：从任意入场格起搜（层间边：入场梯落点≠`level.start` ⇒ `sub` 按入场格重算）。
    fn from_entry(input: &'a SolveInput, level: &'a LevelSpec, entry: (i32, i32)) -> Self {
        Search {
            input,
            level,
            sub: subgraph_from(level, entry),
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
            if !self.sub[(ty * self.level.cols + tx) as usize] {
                continue;
            }
            match self.classify(st, tx, ty) {
                Cell::Blocked => {}
                Cell::Move => {
                    let mut ns = st.clone();
                    ns.pos = (tx, ty);
                    ns.steps += 1;
                    let g = self.gate_arrive(&ns, tx, ty);
                    out.push((Act::Move, ns, g));
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
                        clear_alive(&mut ns.ents, slot);
                        ns.pos = (tx, ty);
                        ns.steps += 1;
                        // floor1 (6,9)=门+83 同格：开门走入即踩格 ⇒ 同一到达面上
                        // 再判 StepGate（引擎 bump 开门→落格踩格触发，同拍语义）
                        let g = self.gate_arrive(&ns, tx, ty);
                        out.push((Act::Door(tx, ty), ns, g));
                    }
                }
                Cell::Pickup => {
                    // 拾取转移（C6，§1.3 拾取行字面路径）：harness pub `try_step` 落格
                    // → tick 推进 32px 归格 → `apply_step_cell_effects`（scene.rs:4093-4096
                    // `pickup_item_type`+`entity::remove` 同拍）→ 读回 player 全字段。
                    // 引擎时序直调；❌不自写物品效果表、❌不直调私有 pickup_item_type。
                    if let Some(ns) = self.pickup_step(st, tx, ty) {
                        let g = self.gate_arrive(&ns, tx, ty);
                        // 航点=普通格 `x,y`（route_gen 语法：走过去；Act::Move 序列化同）
                        out.push((Act::Move, ns, g));
                    }
                }
            }
        }
    }

    /// 搜索主循环：BFS（FIFO）+ K₀ 去重 + Pareto 支配剪枝。
    /// 返回（任一出口可达, 全部非支配终态候选——双查询取解共用，§1.4）。
    fn run(&self) -> (bool, Vec<Goal>) {
        let n = self.level.entities.len();
        self.run_root(
            self.level.start,
            all_alive(n),
            self.level.player.clone(),
            self.level.flags,
        )
    }

    /// 批3 C1：任意入场格 + 初始存活位图起搜（层间边传播：入场格=上一层出场梯的
    /// 落点、位图=上一层终态——批2 `run` 的 `start`/`all_alive` 特例化）。
    /// 批3 C2：玩家/flags 也由调用方注入（层间携带上一层 post；批2 的
    /// `level.player`/`level.flags` 特例化为 `run` 的两参转发）。
    fn run_root(
        &self,
        entry: (i32, i32),
        root_ents: Vec<u64>,
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
                    post: st.clone(),
                });
                continue;
            }
            let mut succ = Vec::new();
            self.expand(&st, &mut succ);
            for (act, ns, gate) in succ {
                match gate {
                    // 门命中但非终态/停摆 ⇒ ⊥（不产生状态，§1.1；C5 口径见 GateArrive）
                    GateArrive::Dropped => continue,
                    GateArrive::Terminal => {
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
                        // gate 航点（route_gen 无此 token=提案缺口，报审注明）
                        route.push(format!("gate {},{}", n.st.pos.0, n.st.pos.1));
                        goals.push(Goal {
                            hp: n.st.player.hp,
                            kills: n.st.kills,
                            steps: n.st.steps,
                            keys: (n.st.player.yellow, n.st.player.blue, n.st.player.red),
                            kind: ExitKind::Script,
                            route,
                            post: n.st.clone(),
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
                        });
                        continue;
                    }
                    GateArrive::Pass => {}
                }
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

    /// StepGate 到达转移（§1.3 StepGate 行；踩格 `from_walk=true`，bump 面
    /// by construction 不触发）：扫到达格存活 type83 → 直调 `try_run_scene(param,true)`
    /// 判门 → 命中 `load_level_script` → 有拍驱动（`tickdrive`）→ `persist.pending_floor`
    /// 判终止边（§2.2 ④）。无门/未命中 ⇒ Pass（不建 harness）。
    fn gate_arrive(&self, st: &St, tx: i32, ty: i32) -> GateArrive {
        let mut gates: Vec<i32> = Vec::new();
        for (slot, e) in self.level.entities.iter().enumerate() {
            if e.x == tx && e.y == ty && e.ty == 83 && alive(&st.ents, slot) {
                gates.push(e.param);
            }
        }
        if gates.is_empty() {
            return GateArrive::Pass;
        }
        let mut sc = build_scene(self.input, st, (tx, ty));
        // 槽位升序=夹具序（提案 §3.3）；首个命中者载入执行（apply_step_cell_effects
        // 同拍内先判先得；floor1 (6,9) 同格 19+66 —— 66 恒 false ⇒ 19 必判）
        for param in gates {
            if sc.try_run_scene(param, true) {
                sc.load_level_script(param);
                return match tickdrive(&mut sc) {
                    Some(_) => GateArrive::Terminal,
                    None => GateArrive::Dropped,
                };
            }
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
        // 实体移除镜像：该格全部存活 cat2|4（引擎倒序逐个 pickup+remove）
        for (slot, e) in self.level.entities.iter().enumerate() {
            if e.x == tx
                && e.y == ty
                && alive(&ns.ents, slot)
                && matches!(entity_category_with_floor(e.ty, self.input.floor), 2 | 4)
            {
                clear_alive(&mut ns.ents, slot);
            }
        }
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

/// `find_path` 可达子图（§1.3 可达=find_path 直调）：从 `from` 到每格 `found`；
/// `sub[from]=true` 强制。过度近似安全——我方动作的可通行语义 ⊆
/// `is_cell_walkable || can_path_through_entity`。
/// 批3 C1：从 `from` 出发的可达域（`sub` 语义=「入场格的连通分量」——批2 只从
/// `level.start` 起算，层间边入场格≠start ⇒ 按入场格重算）。
fn subgraph_from(level: &LevelSpec, from: (i32, i32)) -> Vec<bool> {
    let cols = level.cols;
    let rows = level.rows;
    let mut sub = vec![false; (cols * rows) as usize];
    for y in 0..rows {
        for x in 0..cols {
            sub[(y * cols + x) as usize] = find_cell_path(level, from, (x, y));
        }
    }
    let s = (from.1 * cols + from.0) as usize;
    if s < sub.len() {
        sub[s] = true;
    }
    sub
}

/// T7（提案 §3.1 T7 / 计划 :319「find_path found 与 solver BFS found 一致」）：
/// `game_core::pathfind::find_path` 直调薄封装——转发其 found，不复刻任何怪癖；
/// `subgraph` 与 T7 对拍测试共用本函数（生产/测试单源）。谓词=terrain||slot。
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
            Act::Start | Act::Move => format!("{},{}", pos.0, pos.1),
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
    // 只灌存活槽（死亡怪/已开门不进 harness）；(type,x,y) 三元组=槽位序来源。
    let alive_specs: Vec<&EntSpec> = level
        .entities
        .iter()
        .enumerate()
        .filter(|(slot, _)| alive(&st.ents, *slot))
        .map(|(_, e)| e)
        .collect();
    let ents: Vec<(u8, i32, i32)> = alive_specs
        .iter()
        .map(|e| (e.ty as u8, e.x, e.y))
        .collect();
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
    let mut pool: Vec<&EntSpec> = alive_specs;
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
fn tickdrive(sc: &mut GameScene) -> Option<i32> {
    let wt = sc.width_table.clone();
    let mut ft = 0i64;
    let mut last = String::new();
    let mut stall = 0;
    loop {
        if sc.persist.pending_floor != 0 {
            return Some(sc.persist.pending_floor);
        }
        if matches!(sc.game_mode, GameMode::Map) {
            return None; // 行尾收尾 transition_to(Map)：非终态完成
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
            return None; // 停摆兜底 ⇒ ⊥（保守：不产生假 reachable）
        }
    }
}

/// T9 探针（`probe_bump` 同法=与搜索共用 harness 门面路径，非自证）：夹具
/// flags/difficulty 下 `try_run_scene(param, true)` 的判门结果——只判不放
/// （不 load/tick；case19/66/67 臂只读 flags/difficulty，无副作用）。
pub fn probe_gate(input: &SolveInput, param: i32) -> bool {
    let level = input.level.as_ref().expect("probe_gate 需要夹具");
    let st = St {
        pos: level.start,
        player: level.player.clone(),
        ents: all_alive(level.entities.len()),
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
                all_alive(2),
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
        assert!(!alive(&g.post.ents, 0), "怪已杀 ⇒ 终态位图清除");

        // (b) 预死怪位图（跨层继承：上一层已杀的怪不复活）⇒ 不计杀、位图保持
        let mut predead = all_alive(2);
        clear_alive(&mut predead, 0);
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
        assert_eq!(g2.post.ents, predead, "终态位图=初始位图（无复活）");
        assert_eq!(g2.route, vec!["1,1", "2,1", "3,1"], "无怪 ⇒ 直穿");
        assert_eq!(g2.steps, 2, "两步走穿走廊");
    }
}
