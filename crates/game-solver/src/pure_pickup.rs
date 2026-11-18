//! 【P5→P6】**单调拾取层**闭式解（fail-closed）——`out_hp = entry_hp + Σ hp_delta`，
//! 路线经真引擎回放，且**每一轴**（hp/atk/def/三钥/背包）逐字段与「全捡」期望相等。
//!
//! **为什么单独成模块而不是塞进 `solve_local`**：这是**第二个**独立的层入口规则
//! （第一个是 D 面子集支配 `plan_pill`，同一文件里已住着）；它自带闸门表、路线构造
//! 与回放三件事，且**只对满足「单调拾取」定义的层生效**——与通用搜索空间正交。
//!
//! > 【2026-11-19 P6 口径变更（本片）】本模块的语义**不再是「纯加血」**。
//! > P5 只认 `certified_pure_hp_gain == Some(delta)`（hp-only，其余字段零变化）；
//! > P6 起改认 `certified_monotone_pickup == Some(PickupGain)`（**单调**：hp/atk/def/
//! > 三钥/背包项全部不下降，且其余字段零变化）。旧的窄证书仍在 `game-core` 里保留
//! > （`certified_pure_hp_gain` 语义未改），只是本模块不再用它做闸门。
//! > 带来的后果（**实测，不是推断**）：
//! > - f54 的 18 枚拾取物**全是 HpUp**（`31×8 + 32×10 = 2400`，见 `w20.json` 的
//! >   `ents_in`）⇒ P5 的窄证书已覆盖它，P5/P6 在 f54 上**逐字段相同**（回归实测）。
//! >   P6 唯一给 f54 加的东西是「atk/def/三钥/背包也有期望值与断言」，不是新命中。
//! > - 证书面虽然从 hp-only 扩到五轴，但 **f1..f54 里没有新的层因此获得闭式资格**
//! >   （扫描见 `docs/findings/2026-11-19-pure-pickup-closed-form.md` §7.1：只有 f54 命中；
//!   其余层被 G1 的 `cat8` 怪或 G2/G2a 的 `ty83` 场景门挡住，与拾取物是哪种类型无关）。
//!
//! ## 定理（本模块的全部正确性都挂在它上面，代码注释与判据一一对应）
//!
//! 若某层满足
//! 1. **G1** 无怪、**G2** 无危险步臂可达、**G2a** 入场格安全（三闸，定义见下表），且
//! 2. **G3′** 每个可达拾取物的效果经**真引擎采样**认证为**单调**：
//!    hp/atk/def/三钥/背包项**全部不下降**，且除这些字段外**其余字段零变化**
//!    （没有「捡了会变差」的副作用：gold/tutorial_seen/装备槽都不许动），
//! 则「把可达拾取物全捡一遍再出层」是 **HP 最优**，闭式 `out_hp = in_hp + Σ hp_delta`。
//!
//! 证明（为什么「全捡」支配「少捡」，且与顺序无关）：
//! - G3′ 的采样结果对**同一层**是**常量向量** Δ（`certified_monotone_pickup(ty,floor)`
//!   只依赖 ty 与 floor；`HpUp/StatUp` 的 `floor_tier` 在层内恒定）⇒ 捡第 i 枚的效果
//!   与「何时捡、之前捡了什么」无关（背包追加除外，见下）；
//! - 各轴 Δ ≥ 0 且互不抵触（每一轴都是独立计数器相加，没有「上限」或「互相抵消」）⇒
//!   任意子集的合计 = 子集内 Δ 逐项相加 ⇒ **全集 ≥ 任意子集**（逐轴）；
//! - 背包纯追加（只往空格写 `(ty,uses)`、旧槽逐字节不变）⇒ 多捡一件不会让已有的变差；
//! - hp 是其中一轴 ⇒ `hp(全捡) = in_hp + Σ hp_delta` 是层内 HP 上界；G4 给出一条
//!   「能走上去」的出口 ⇒ 该上界**可达**；
//! - 步数不影响 hp（G1 无战斗、G2 无踩格扣血）⇒ 多绕路不损失 hp，只影响 tie-break。
//!
//! **这条定理不覆盖什么**（宁可说清也不许含糊）：只要层内有怪（G1 破），
//! 「全捡」就可能必须杀怪 ⇒ 扣血 ⇒ 少于全捡的路线反而更优（f3 就是这种层，
//! 见 `docs/findings/2026-11-19-pure-pickup-closed-form.md` §9.2）。
//!
//! ## 判定「纯拾取子层」（闸门，任一不成立 ⇒ `None` ⇒ 调用方走通用搜索）
//!
//! | 闸门 | 条件 | 依据 |
//! |---|---|---|
//! | G0 | `level` 存在、`entry` 在界内 | 层外/层界外没有几何可言 |
//! | G1 | 层内无怪物 | `entity_category_with_floor(ty,floor)==8`（与求解器 `monster_ents` 同源）。**不**用任务书的 `41..=76` 区间：那一段混着 6 种踩格副作用臂，会把陷阱层永远以 `monster_ents` 提前失败、盖掉 G2 的具名理由（见 §G1 注释） |
//! | G2a | **入场格**的步臂只允许「门已开的场景门」 | 引擎进图链在出生格也跑 `apply_step_cell_effects`（`game-core/src/scene.rs:3878-3887`）⇒ 入场格的陷阱/宝箱在真实玩家进层时就触发。`ty83` 且 `probe_gate=true`（门已开 ⇒ 脚本不跑）才可证中性 |
//! | G2 | 危险步臂格不可达（入场格除外） | `STEP_ARM_HAZARD_TYPES`（`solve_local.rs:1321`，单一真相源，不复制数值）；「可达」= 几何 BFS（hazard 格本身按可走算，否则判定空转） |
//! | G3′ | 全部拾取物经认证**单调**、每格仅 1 枚、**不在入场格**、背包放得下 | `certified_monotone_pickup(ty,floor)=Some(PickupGain)`（`crates/game-core/src/effect_registry.rs`，运行时探针背书：六轴 ≥0 + 背包纯追加 + 其余零变化）；`cat∈{2,4}`；`bag_no_room` 见实现 |
//! | G4 | 至少一个 `ExitKind::Stair` 出口**能从纯图上走上去** | 纯图 = `classify` 判 `Move`/`Pickup` ∧ 非 hazard 格 ∧ **入场格不可作中间点**（`step_ok` 的 `cell != entry`）。**结构性排除**：只能从入场格走上去的出口（f54 的 `(7,10)`）**永远不进候选集**——这不是「未证明」，是本模块明确不覆盖的边（F1 口径更正） |
//! | G5 | 入场 `hp > 0` | 0/占位 hp 不是真实入场态 |
//! | G6 | `policy == ClosurePolicy::OFF` | 闭包谓词（Q1/Q2/Q3）改变搜索语义，闭式解不建模 |
//!
//! 编号沿用实现注释，**不是执行序**：短路的闸先跑（G0→G6→G5→G2a→G2→G1→G3→G4），
//! 这样「层里有怪」与「陷阱在可达格」不会被彼此的早退遮蔽（理由面是本片唯一可观测通道）。
//!
//! ## 为什么 `out_hp = entry_hp + Σ hp_delta` 是**上界且可达**
//!
//! 上界：G1 无怪 ⇒ 无战斗扣血；G2 无可踩危险臂 ⇒ 无陷阱/宝箱/脚本副作用；G3′ 全部拾取
//! 各轴非降（hp 只是其中一轴）；G5 入场 hp 真实 ⇒ 层内 hp 只能「+Σ已捡」，故
//! `≤ entry_hp + Σ hp_delta`。
//! 可达：G4 + 纯图连通 ⇒ 存在一条把可达拾取物全捡完再去出口的走法（拾取不挡路——拾取格
//! 与普通格同价 +1 步），步数只是 tie-break（`HpLoss`：hp 大 → steps 小 → route 字典序）。
//! **非 hp 轴的可达性不由闭式「推断」，而由回放逐字段背书**：`expected_after_all` 算出
//! 「全捡」的 atk/def/三钥/背包期望，回放后逐轴比对（不符 ⇒ `replay_stat_mismatch` /
//! `replay_key_mismatch` / `replay_bag_mismatch` 具名回退）。
//!
//! ## 路线必须真走通（本片的硬要求，不是可选项）
//!
//! 闭式只给血量上界；`route` 必须是**引擎里真能跑完、并真拿到那个血量**的路径。
//! 做法：纯图上反复 BFS「最近未拾取物优先」→ 走完再 BFS 到出口；然后**逐步喂
//! `Search::pickup_step`**（`solve_local.rs:2916`，真 `try_step` + 拍推进 + 读回）回放，
//! 断言每步成功、终态 `hp == 期望`、`pos == 出口`、**每一轴 == 全捡期望**（atk/def/三钥/
//! 背包；gold 必须零变化）、拾取物全消失。
//! 任一项不符 ⇒ `None`（stat 记具名 `reason`）⇒ 回退通用搜索。**禁静默**。
//!
//! 「最近优先」贪心不保证成功（某颗药可能被先捡的物品隔断——本层不会，但别的层可能），
//! 故预留 `ORDERS` 四种顺序（最近/最远/行序/逆行），**上限写死 4**，全部失败即 fail-closed。

use crate::solve_local::{
    probe_gate, pure_cell_kinds, replay_pickup_route, sprite_full, ClosurePolicy, EntRec, ExitKind,
    LayerGoal, PlayerSpec, SolveInput, STEP_ARM_HAZARD_TYPES,
};
use game_core::effect_registry::{certified_monotone_pickup, PickupGain};
use game_core::subsystem::script_host::entity_category_with_floor;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// 格分类的**三态投影**（`solve_local::classify` 的只读投影，禁第二份分类表）：
/// `Free`=可走空格（含楼梯族豁免格）、`Pickup`=拾取格、`Other`=门/商店/怪/阻挡
/// （闭式解一律视为不可进——它们都意味着「有代价或未建模的臂」）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Free,
    Pickup,
    Other,
}

/// 四个方向（与 `solve_local::DIRS` 同序：上、下、左、右）——BFS 邻居展开顺序决定
/// 同距多路的先后，写死在此保证**跨运行确定性**（不依赖 HashMap 迭代序）。
const DIRS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

/// 拾取顺序策略（写死 4 种；`orders_tried` 记录实际试到第几种才成功）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickOrder {
    /// 最近未拾取物优先（主策略：贪心，最可能贴合 `HpLoss` 的 steps 次目标）
    Nearest,
    /// 最远优先（反向贪心：若最近的药把某条通道堵住，反向常能绕开）
    Farthest,
    /// 行序（`(y,x)` 升序：从上到下、从左到右扫）
    ScanForward,
    /// 逆行序（`(y,x)` 降序）
    ScanBackward,
}

const ORDERS: [PickOrder; 4] = [
    PickOrder::Nearest,
    PickOrder::Farthest,
    PickOrder::ScanForward,
    PickOrder::ScanBackward,
];

/// 闭式解可观测统计（`pure_pickup_stat()`）。体例照 [`crate::solve_local::PillDomStat`]：
/// **禁静默**——没采用时必须能从 `reason`/`reason_ty` 看出卡在哪条闸门。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurePickupStat {
    /// 本层是否尝试过闭式（早退路径 ⇒ `false` + `reason="not_run"`）
    pub attempted: bool,
    /// 是否采用（`false` ⇒ 通用搜索，行为与上一片逐字节同）
    pub adopted: bool,
    /// 采用时 `"adopted"`；否则为 fail-closed 具名原因；未跑过层 `"not_run"`
    pub reason: &'static str,
    /// 触发 fail-closed 的实体型 / 与具体类型无关时为 0
    pub reason_ty: i32,
    /// 层内认证纯加血拾取物枚数
    pub pickups: usize,
    /// Σ delta（= `out_hp - entry_hp`）
    pub sum_delta: i32,
    /// 采用时的路线长度（token 数，含入场格）
    pub route_len: usize,
    /// 采用时的引擎回放步数（应等于 `route_len - 1`）
    pub replay_steps: i32,
    /// 采用时实际试到第几种拾取顺序（0=最近优先即成功）
    pub orders_tried: usize,
    /// 入场格上的危险步臂数（本片唯一允许的例外：入场是起点、不是「到达」，
    /// `gate_arrive` 不在起点触发；路线构造禁止再进入入场格，回放兜底 ⇒ 不会真踩）
    pub entry_hazards: usize,
    /// 闭式断言的血量（`entry_hp + Σ delta`）
    pub hp_expected: i32,
    /// 引擎回放读回的血量（采用时应 == `hp_expected`）
    pub hp_replayed: i32,
    /// 采用时的出口格
    pub exit: Option<(i32, i32)>,
    /// Σ atk 增量（「全捡」期望；P6 单调层才有非零值）
    pub sum_atk: i32,
    /// Σ def 增量
    pub sum_def: i32,
    /// Σ (黄,蓝,红) 钥匙增量
    pub sum_keys: (i32, i32, i32),
    /// Σ 背包纯追加件数
    pub sum_bag: usize,
}

impl PurePickupStat {
    /// 未跑过层的形态（`reason="not_run"`——不是空白，是「本层没问过」）。
    pub const CLEARED: PurePickupStat = PurePickupStat {
        attempted: false,
        adopted: false,
        reason: "not_run",
        reason_ty: 0,
        pickups: 0,
        sum_delta: 0,
        route_len: 0,
        replay_steps: 0,
        orders_tried: 0,
        entry_hazards: 0,
        hp_expected: 0,
        hp_replayed: 0,
        exit: None,
        sum_atk: 0,
        sum_def: 0,
        sum_keys: (0, 0, 0),
        sum_bag: 0,
    };
}

impl PurePickupStat {
    /// 本层闭式判定的**可消费标签**：`None` = 本层没问过闭式（`attempted=false`）；
    /// `Some("adopted")` = 闭式命中；其余 = fail-closed 具名拒绝原因。
    ///
    /// 触发实体型非 0 时拼成 `reason/ty`（形如 `monster_ents/41`）——只报 `reason`
    /// 会让「41 型怪」与「77 型怪」在 JSON 里撞成同一字符串，下游无法定位。
    pub fn reason_label(&self) -> Option<String> {
        if !self.attempted {
            return None;
        }
        if self.reason_ty == 0 {
            Some(self.reason.to_string())
        } else {
            Some(format!("{}/{}", self.reason, self.reason_ty))
        }
    }
}

thread_local! {
    /// 本线程最近一次层尝试的统计（thread_local = 多线程测试隔离；纯诊断面，
    /// 不进任何搜索键、不改输出）。memo 命中时由 `solve_layer_with` 回放同一份值。
    static STAT: Cell<PurePickupStat> = const { Cell::new(PurePickupStat::CLEARED) };
}

/// 读最近一次统计（未跑过层 ⇒ [`PurePickupStat::CLEARED`]）。
pub fn pure_pickup_stat() -> PurePickupStat {
    STAT.with(|c| c.get())
}

/// 写统计（`solve_layer_with` 在 memo 命中/早退路径上显式写入）。
pub fn pure_pickup_stat_set(s: PurePickupStat) {
    STAT.with(|c| c.set(s));
}

/// 纯图（格分类 + hazard 标记 + 拾取格 delta）——闭式解的全部几何依据。
struct Grid {
    cols: i32,
    rows: i32,
    kind: Vec<CellKind>,
    /// 危险步臂格 → 该格上的 hazard 实体型（0 表示无）
    hazard: BTreeMap<(i32, i32), i32>,
    /// 认证**单调**拾取格 → `(增益向量, ty)`（P6 起语义从「纯加血」扩到「单调」）
    pickup: BTreeMap<(i32, i32), (PickupGain, i32)>,
}

impl Grid {
    fn inb(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.cols && y < self.rows
    }

    fn at(&self, x: i32, y: i32) -> usize {
        (y * self.cols + x) as usize
    }

    /// `Move`/`Pickup` = 纯格；`Other`（门/商店/怪/阻挡/monster 边）一律非纯。
    fn pure(&self, x: i32, y: i32) -> bool {
        self.inb(x, y) && matches!(self.kind[self.at(x, y)], CellKind::Free | CellKind::Pickup)
    }

    /// 路线可进格：纯格 ∧ 非 hazard ∧ **不是入场格**（入场只作起点，禁再进入）。
    ///
    /// 【F1 口径更正（2026-11-19 复核黄 F1）】`cell != entry` 这一项是**结构性排除**，
    /// 不是「未证明」：它让「只能从入场格走上去」的出口格**永远进不了闭式的候选集**。
    /// f54 的 `(7,10)` 出口就是这样——它唯一的邻格是入场格 `(7,9)` ⇒ 闭式**只覆盖**
    /// `(7,1)`，`(7,10)` 那条边的出层最优值**本模块不声称、也不计算**（旧通用搜索能给出
    /// `2227/26/(7,10)`，那是搜索的答案，不是闭式的）。
    /// 为什么不做「允许穿回入场格」：入场格的步臂只有「门已开的场景门」被 G2a 证过中性，
    /// 重入等于把未建模的踩格臂放回路线里 ⇒ 宁可让目标集合小、也不能让上界假。
    fn step_ok(&self, cell: (i32, i32), entry: (i32, i32)) -> bool {
        cell != entry && self.pure(cell.0, cell.1) && !self.hazard.contains_key(&cell)
    }

    /// 几何可达（hazard 格按可走算）：用于 G2「hazard 格是否在能走到的地方」。
    fn geo_reach(&self, start: (i32, i32)) -> BTreeSet<(i32, i32)> {
        let mut seen: BTreeSet<(i32, i32)> = BTreeSet::new();
        seen.insert(start);
        let mut q: VecDeque<(i32, i32)> = VecDeque::new();
        q.push_back(start);
        while let Some(p) = q.pop_front() {
            for &(dx, dy) in &DIRS {
                let n = (p.0 + dx, p.1 + dy);
                // 「几何可达」= 玩家身体走得到：危险步臂格**算可走**（否则 G2 永远看不见
                // 陷阱/场景门，闸门形同虚设），可不可踩由 G2 判。
                if seen.contains(&n) || !(self.pure(n.0, n.1) || self.hazard.contains_key(&n)) {
                    continue;
                }
                seen.insert(n);
                q.push_back(n);
            }
        }
        seen
    }

    /// 从 `from` 出发的步数表（同一可走判据：`step_ok`；`from` 自身恒在表内）。
    fn bfs_dist(&self, from: (i32, i32), entry: (i32, i32)) -> BTreeMap<(i32, i32), i32> {
        let mut dist: BTreeMap<(i32, i32), i32> = BTreeMap::new();
        dist.insert(from, 0);
        let mut q: VecDeque<(i32, i32)> = VecDeque::new();
        q.push_back(from);
        while let Some(p) = q.pop_front() {
            let d = dist[&p];
            for &(dx, dy) in &DIRS {
                let n = (p.0 + dx, p.1 + dy);
                if dist.contains_key(&n) || !self.step_ok(n, entry) {
                    continue;
                }
                dist.insert(n, d + 1);
                q.push_back(n);
            }
        }
        dist
    }

    /// 最短路径 `from → to`（不含 `from`；`from == to` ⇒ 空）。BFS + 父指针回溯，
    /// 邻居序写死 ⇒ 同输入同路径（确定性）。
    fn bfs_path(
        &self,
        from: (i32, i32),
        to: (i32, i32),
        entry: (i32, i32),
    ) -> Option<Vec<(i32, i32)>> {
        if from == to {
            return Some(Vec::new());
        }
        // 目标格自身允许进入（出口可能就在 hazard 之外，但仍受 step_ok 约束我们不走它）
        if !self.step_ok(to, entry) {
            return None;
        }
        let mut parent: BTreeMap<(i32, i32), (i32, i32)> = BTreeMap::new();
        let mut seen: BTreeSet<(i32, i32)> = BTreeSet::new();
        seen.insert(from);
        let mut q: VecDeque<(i32, i32)> = VecDeque::new();
        q.push_back(from);
        while let Some(p) = q.pop_front() {
            for &(dx, dy) in &DIRS {
                let n = (p.0 + dx, p.1 + dy);
                if seen.contains(&n) || !self.step_ok(n, entry) {
                    continue;
                }
                seen.insert(n);
                parent.insert(n, p);
                if n == to {
                    let mut path: Vec<(i32, i32)> = Vec::new();
                    let mut cur = to;
                    loop {
                        path.push(cur);
                        if cur == from {
                            break;
                        }
                        cur = *parent.get(&cur).expect("BFS parent chain");
                    }
                    path.reverse();
                    path.remove(0); // 去掉起点：调用方自己持有 `from`
                    return Some(path);
                }
                q.push_back(n);
            }
        }
        None
    }

    /// 按策略给出拾取次序；不可达的拾取物 ⇒ `None`（该策略作废，换下一种）。
    fn pick_sequence(
        &self,
        order: PickOrder,
        start: (i32, i32),
        entry: (i32, i32),
        targets: &[(i32, i32)],
    ) -> Option<Vec<(i32, i32)>> {
        match order {
            PickOrder::ScanForward | PickOrder::ScanBackward => {
                let mut v: Vec<(i32, i32)> = targets.to_vec();
                v.sort_by_key(|&(x, y)| (y, x));
                if order == PickOrder::ScanBackward {
                    v.reverse();
                }
                Some(v)
            }
            PickOrder::Nearest | PickOrder::Farthest => {
                let mut left: Vec<(i32, i32)> = targets.to_vec();
                let mut seq: Vec<(i32, i32)> = Vec::new();
                let mut cur = start;
                while !left.is_empty() {
                    let dist = self.bfs_dist(cur, entry);
                    let mut best: Option<((i32, i32), i32)> = None;
                    for &t in &left {
                        let d = *dist.get(&t)?; // 有拾取物不可达 ⇒ 该策略作废
                        let better = match best {
                            None => true,
                            Some((bt, bd)) => {
                                let tie = (t.1, t.0) < (bt.1, bt.0); // tie-break：最小 (y,x)
                                if order == PickOrder::Nearest {
                                    d < bd || (d == bd && tie)
                                } else {
                                    d > bd || (d == bd && tie)
                                }
                            }
                        };
                        if better {
                            best = Some((t, d));
                        }
                    }
                    let (t, _) = best?;
                    left.retain(|&c| c != t);
                    seq.push(t);
                    cur = t;
                }
                Some(seq)
            }
        }
    }
}

/// fail-closed：写具名原因 + 回显统计，返回 `None`（调用方走通用搜索）。

/// 引擎侧「空背包」的规范形：`readback` 给的是定长 32 槽、空槽填 0（实测 `[]` → `[0;32]`），
/// 而入参可能用 `[]` 表空。比较前统一把尾部 0 槽裁掉——**这不是放宽断言**：非零槽（真持有
/// 道具）仍逐槽比对，只要拾取往背包里塞了任何东西就必然不等。证据见
/// `docs/findings/2026-11-19-pure-pickup-closed-form.md` §「侧效应比较的规范形」。
fn trim_slots(v: &[u8]) -> Vec<u8> {
    let mut n = v.len();
    while n > 0 && v[n - 1] == 0 {
        n -= 1;
    }
    v[..n].to_vec()
}

/// 拒绝理由的**优先级**：越靠前越"值钱"。
///
/// 动机（实测，变异自证 A）：f54 有两条 Stair 出口，(7,10) 那条在纯图上不可达（唯一邻格就是
/// 入场格）。若只"记最后一条理由"，`no_stair_exit_reachable` 会把真正有价值的
/// `replay_hp_mismatch` 覆盖掉 —— 而前者是**预期内**的、后者是**假设被打破**。理由面是
/// 本片唯一的可观测通道，不许被无害失败淹没。
const REASON_RANK_REPLAY: u8 = 3;
/// 背包放不下（入参 `item_types` 没有空槽）——比 `no_stair_exit_reachable` 值钱：
/// 它说明「这层本来能走闭式，只是入参的背包容量信息不可用」，是数据面缺口不是几何缺口。
const REASON_RANK_BAG: u8 = 2;
const REASON_RANK_PICKUP_UNREACHABLE: u8 = 2;
const REASON_RANK_NO_STAIR_EXIT: u8 = 1;
const REASON_RANK_DEFAULT: u8 = 0;

/// 记账：只在 rank 更高时覆盖（同级保留先出现的）。
fn note_reason(slot: &mut (&'static str, i32, u8), reason: &'static str, ty: i32, rank: u8) {
    if rank > slot.2 {
        *slot = (reason, ty, rank);
    }
}

fn give_up(s: &mut PurePickupStat, reason: &'static str, ty: i32) -> Option<Vec<LayerGoal>> {
    s.adopted = false;
    s.reason = reason;
    s.reason_ty = ty;
    pure_pickup_stat_set(*s);
    None
}

/// 全部拾取实体（`cat ∈ {2,4}`）——与 `classify` 的拾取判定同一单源。
fn is_pickup(ty: i32, floor: i32) -> bool {
    matches!(entity_category_with_floor(ty, floor), 2 | 4)
}


/// 引擎回放结果（全部字段来自 `readback_*` 读回，无推演）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineReplay {
    pub player: PlayerSpec,
    pub flags: u128,
    pub pos: (i32, i32),
    pub ents: Vec<EntRec>,
    pub steps: i32,
}

/// **公开**回放入口：把一条 `route`（token 形如 `"x,y"`，首 token=入场格）逐步喂回真引擎。
///
/// 这是本片判据 2「路线必须真走通」的外显验证面：调用方拿 [`crate::solve_local::ExitSolve`]
/// 的 `route` 直接回放，就能独立复算「终态 hp/位置」。任何非 `"x,y"` token（`bump`/`door`/
/// `gate`）⇒ `None`（纯拾取路线不该出现它们，出现即说明闭式解被误用）。
/// `ents_in=None` ⇒ 该层从未推进过（`sprite_full`，新层首入语义）。
pub fn replay_route_in_engine(
    input: &SolveInput,
    entry: (i32, i32),
    player_in: &PlayerSpec,
    flags_in: u128,
    ents_in: Option<&[EntRec]>,
    route: &[String],
) -> Option<EngineReplay> {
    let level = input.level.as_ref()?;
    let ents: Vec<EntRec> = match ents_in {
        Some(v) => v.to_vec(),
        None => sprite_full(level),
    };
    let mut cells: Vec<(i32, i32)> = Vec::with_capacity(route.len());
    for t in route {
        let (a, b) = t.split_once(',')?;
        cells.push((a.trim().parse().ok()?, b.trim().parse().ok()?));
    }
    let out = replay_pickup_route(input, entry, &ents, player_in, flags_in, &cells)?;
    Some(EngineReplay {
        player: out.player,
        flags: out.flags,
        pos: out.pos,
        ents: out.ents,
        steps: out.steps,
    })
}

/// 纯拾取层闭式解主入口（`solve_layer_with` 在 memo 命中之后调用）。
///
/// 返回 `Some(goals)` ⇒ 每条可出层 `Stair` 出口一条 `LayerGoal`（hp = 闭式上界，
/// route 已**逐步引擎回放验证**过）；返回 `None` ⇒ fail-closed，调用方走通用搜索，
/// 且 `pure_pickup_stat()` 里有具名原因。
/// 「把 `seq` 里的拾取物全捡一遍」的**期望终态**（逐轴，不是只有 hp）。
///
/// 计算依据就是定理本身：每枚拾取的增益是常量向量 Δ（`PickupGain`），各轴 ≥0
/// ⇒ 逐轴相加；背包是**纯追加** ⇒ 按 `seq` 顺序落到 `[n, n+k)` 槽位。
/// `None` = 预期放不进背包（容量信息不足/越界）⇒ 调用方 fail-closed。
fn expected_after_all(
    player_in: &PlayerSpec,
    grid: &Grid,
    seq: &[(i32, i32)],
    hp_expected: i32,
) -> Option<PlayerSpec> {
    let mut p = player_in.clone();
    p.hp = hp_expected;
    let mut types = player_in.item_types.clone();
    let mut uses = player_in.item_uses.clone();
    let mut n = player_in.item_stack_size.max(0) as usize;
    for cell in seq {
        let (g, _ty) = grid.pickup.get(cell)?;
        p.atk = p.atk.checked_add(g.atk)?;
        p.def = p.def.checked_add(g.def)?;
        p.yellow = p.yellow.checked_add(g.yellow)?;
        p.blue = p.blue.checked_add(g.blue)?;
        p.red = p.red.checked_add(g.red)?;
        for &(t, u) in &g.bag {
            if n >= types.len() || n >= 32 {
                return None;
            }
            types[n] = t;
            uses[n] = u;
            n += 1;
        }
    }
    p.item_types = types;
    p.item_uses = uses;
    p.item_stack_size = n as i32;
    Some(p)
}

pub(crate) fn try_closed_form(
    input: &SolveInput,
    entry: (i32, i32),
    root_ents: &[EntRec],
    player_in: &PlayerSpec,
    flags_in: u128,
    policy: ClosurePolicy,
) -> Option<Vec<LayerGoal>> {
    let mut st = PurePickupStat {
        attempted: true,
        ..PurePickupStat::CLEARED
    };
    pure_pickup_stat_set(st);

    // ---- G6 闭包谓词：非 OFF 语义不同（Q1/Q2/Q3 是「按谓词剪枝」的搜索语义） ----
    if policy != ClosurePolicy::OFF {
        return give_up(&mut st, "policy_not_off", 0);
    }
    let Some(level) = input.level.as_ref() else {
        return give_up(&mut st, "no_level", 0);
    };
    // ---- G0 入场格在界内 ----
    if entry.0 < 0 || entry.1 < 0 || entry.0 >= level.cols || entry.1 >= level.rows {
        return give_up(&mut st, "entry_out_of_bounds", 0);
    }
    // ---- G5 入场 hp 真实 ----
    if player_in.hp <= 0 {
        return give_up(&mut st, "entry_hp_not_real", player_in.hp);
    }

    // ---- 格分类（复用 `classify`：禁止第二份分类表） ----
    let Some((cols, rows, kinds)) = pure_cell_kinds(input, root_ents) else {
        return give_up(&mut st, "no_level", 0);
    };

    let mut grid = Grid {
        cols,
        rows,
        kind: kinds,
        hazard: BTreeMap::new(),
        pickup: BTreeMap::new(),
    };
    // hazard 格登记（步臂实体型；类型表复用 `solve_local` 常量，不复制数值）
    let mut entry_arms: Vec<(i32, i32)> = Vec::new();
    for e in root_ents {
        if STEP_ARM_HAZARD_TYPES.contains(&e.ty) {
            grid.hazard.insert((e.x, e.y), e.ty);
            if (e.x, e.y) == entry {
                entry_arms.push((e.ty, e.param));
            }
        }
    }
    st.entry_hazards = entry_arms.len();

    // ---- G2' 入场格的步臂：**只允许「门已开的场景门」**，别的一律失败 ----
    // 为什么这道闸不能省（A 级，`crates/game-core/src/scene.rs:3878-3887`）：
    // 引擎的进图链（`dispatchModeEntry` case 3）在**出生格**上也调 `apply_step_cell_effects`
    // ⇒ 入场格的陷阱/宝箱/组门**在真实玩家进入该层时就会触发**。求解器的 `build_scene`
    // 不走这条链（本片不改引擎语义），所以闭式解的 `entry_hp + Σ` 只有在入场格步臂
    // **可证 hp/实体中性**时才成立：`ty83` 且 `probe_gate=true`（门已开 ⇒ `gate_arrive`
    // 直接 Pass，脚本根本不跑，`effect_registry` 侧已由上一片证过脚本 64 不动 hp）。
    // 其余（`5/60/61/62/76`，或关着的门）⇒ 回退通用搜索。
    for &(ty, param) in &entry_arms {
        if ty != 83 || !probe_gate(input, param) {
            return give_up(&mut st, "step_arm_hazard_on_entry", ty);
        }
    }

    // ---- G2 危险步臂格不可达（入场格例外：起点不是「到达」，且路线禁再进入） ----
    // 判据看「几何可达」而非 `classify` 的 `pure`：`ty60/61/62` 在本作数据里 category=8
    // （会被 classify 归成 Monster⇒非纯），但它们的真实语义是**踩格即扣血的陷阱**；
    // 引擎的踩格副作用臂按 **实体类型** 触发，与分类无关 ⇒ 可达即失败，宁严不宽。
    let geo = grid.geo_reach(entry);
    for (&cell, &ty) in &grid.hazard {
        if cell == entry {
            continue;
        }
        if geo.contains(&cell) {
            return give_up(&mut st, "step_arm_hazard_reachable", ty);
        }
    }

    // ---- G1 层内无怪物：`cat8`（=求解器 `monster_ents` 的口径） ----
    // 任务书写「无 41..=76 型实体」，但那一段里混着 6 种**踩格副作用臂**（`5/60/61/62/76/83`，
    // 见 `STEP_ARM_HAZARD_TYPES`）；把它们当怪会让「陷阱层」永远以 `monster_ents` 提前失败、
    // 盖掉本该由 G2 给出的 `step_arm_hazard_reachable` 具名理由（可观测性损失）。
    // 故怪物用与求解器同源的 `cat8` 判，hazard 型由 G2 独立守——两条闸门互不遮挡。
    for e in root_ents {
        if entity_category_with_floor(e.ty, input.floor) == 8 {
            return give_up(&mut st, "monster_ents", e.ty);
        }
    }

    // ---- G3 拾取物：逐格唯一 + 逐型认证纯加血 ----
    let mut per_cell: BTreeMap<(i32, i32), Vec<i32>> = BTreeMap::new();
    for e in root_ents {
        if is_pickup(e.ty, input.floor) {
            per_cell.entry((e.x, e.y)).or_default().push(e.ty);
        }
    }
    for (&cell, tys) in &per_cell {
        if tys.len() > 1 {
            // 同格多枚：先捡哪颗是可观测差异（顺序依赖引擎 apply 序）⇒ 不猜
            return give_up(&mut st, "pickup_cell_shared", tys[0]);
        }
        if cell == entry {
            // 入场不触发拾取 ⇒ 这枚药要么计入 Σ 破上界、要么不算破「全捡」
            return give_up(&mut st, "pickup_on_entry_cell", tys[0]);
        }
        if !grid.pure(cell.0, cell.1) {
            return give_up(&mut st, "pickup_cell_not_pure", tys[0]);
        }
        let ty = tys[0];
        // G3′（P6）：证书从「纯加血」放宽到「单调」——真引擎采样出**六轴增量 +
        // 背包纯追加**，任一轴下降或有额外副作用（gold/tutorial/装备/背包乱动）
        // 都返回 None ⇒ fail-closed。禁按 ty 号硬编码：值全部来自 `certified_monotone_pickup`。
        let Some(gain) = certified_monotone_pickup(ty, input.floor) else {
            return give_up(&mut st, "pickup_not_certified", ty);
        };
        if !gain.is_gain() {
            return give_up(&mut st, "pickup_not_certified", ty);
        }
        grid.pickup.insert(cell, (gain, ty));
    }
    if grid.pickup.is_empty() {
        // 无拾取物 ⇒ hp 恒定，闭式退化成「走到出口」：交给通用搜索（不扩大行为面）
        return give_up(&mut st, "no_certified_pickup", 0);
    }
    // 「全捡」的期望增益向量（Σ 各轴；背包件数只数个数，内容按 `seq` 顺序在下面拼）。
    // 顺序无关：每枚拾取的效果是**常量向量**且各轴 ≥0 ⇒ 逐轴相加与捡取次序无关。
    let mut total = PickupGain::default();
    for (g, _) in grid.pickup.values() {
        total.add_assign(g);
    }
    let sum_delta = total.hp;
    // 溢出 ⇒ 不是「无解」而是「这个层不该走闭式」，必须走具名 give_up（`?` 会让 stat 停在
    // 上一次的 `attempted` 快照上 ⇒ 静默，违反本片「禁静默」）。
    let Some(hp_expected) = player_in.hp.checked_add(sum_delta) else {
        return give_up(&mut st, "hp_overflow", 0);
    };
    // 背包容量：引擎的 `StackEffect` 追加守卫是 `n < item_types.len()`（BUG-050 静默吞面）
    // ⇒ 入参没有空槽时「捡起来」会静默失败。此时不能声称 hp 上界可达 ⇒ fail-closed。
    let bag_need = total.bag.len();
    let n0 = player_in.item_stack_size.max(0) as usize;
    if bag_need > 0 && (n0 + bag_need > 32 || n0 + bag_need > player_in.item_types.len()) {
        return give_up(&mut st, "bag_no_room", 0);
    }
    st.pickups = grid.pickup.len();
    st.sum_delta = sum_delta;
    st.sum_atk = total.atk;
    st.sum_def = total.def;
    st.sum_keys = (total.yellow, total.blue, total.red);
    st.sum_bag = bag_need;
    st.hp_expected = hp_expected;

    // ---- G4 出口：只认 `Stair`（`goal_edge` 可由出口梯实体取参；Script/Cloud/UseItem
    //      的边语义由脚本/背包决定，闭式解不建模 ⇒ fail-closed 交给通用搜索） ----
    let mut exits: Vec<(i32, i32)> = level
        .exits
        .iter()
        .filter(|e| e.kind == ExitKind::Stair)
        .map(|e| (e.x, e.y))
        .collect();
    exits.sort_unstable();
    exits.dedup();
    exits.retain(|&c| c != entry && grid.step_ok(c, entry));
    if exits.is_empty() {
        return give_up(&mut st, "no_stair_exit_reachable", 0);
    }

    let targets: Vec<(i32, i32)> = grid.pickup.keys().copied().collect();
    let mut goals: Vec<LayerGoal> = Vec::new();
    let mut last: (&'static str, i32, u8) = ("route_construct_failed", 0, REASON_RANK_DEFAULT);
    for &exit in &exits {
        let mut done = false;
        for (oi, &order) in ORDERS.iter().enumerate() {
            if done {
                break;
            }
            let Some(seq) = grid.pick_sequence(order, entry, entry, &targets) else {
                note_reason(&mut last, "pickup_unreachable", 0, REASON_RANK_PICKUP_UNREACHABLE);
                continue;
            };
            // 逐段连路：entry → 第一颗 → … → 最后一颗 → 出口
            let mut cells: Vec<(i32, i32)> = vec![entry];
            let mut cur = entry;
            let mut ok = true;
            for t in &seq {
                let Some(leg) = grid.bfs_path(cur, *t, entry) else {
                    ok = false;
                    break;
                };
                cells.extend(leg);
                cur = *t;
            }
            if !ok {
                note_reason(&mut last, "pickup_unreachable", 0, REASON_RANK_PICKUP_UNREACHABLE);
                continue;
            }
            let Some(leg) = grid.bfs_path(cur, exit, entry) else {
                note_reason(&mut last, "no_stair_exit_reachable", 0, REASON_RANK_NO_STAIR_EXIT);
                continue;
            };
            cells.extend(leg);

            // ---- 真引擎回放（硬要求）：逐步 `pickup_step` (`try_step` + 拍推进 + 读回) ----
            let Some(out) = replay_pickup_route(input, entry, root_ents, player_in, flags_in, &cells)
            else {
                note_reason(&mut last, "replay_step_failed", 0, REASON_RANK_REPLAY);
                continue;
            };
            // 终态位置
            if out.pos != exit {
                note_reason(&mut last, "replay_pos_mismatch", 0, REASON_RANK_REPLAY);
                continue;
            }
            // 终态血量 = 闭式上界（一字不差）
            if out.player.hp != hp_expected {
                note_reason(&mut last, "replay_hp_mismatch", 0, REASON_RANK_REPLAY);
                st.hp_replayed = out.player.hp;
                continue;
            }
            // 逐轴校验「引擎真走出来的终态 == 全捡期望」——这是 P6 的核心断言：
            // 不再是「除 hp 外零变化」，而是「每一轴都等于 Σ」，任何一轴漂了就 fail-closed。
            // 拆成具名理由是为了可观测（只说 side_effect 会让 stat/atk/背包混成一个串）。
            let Some(want) = expected_after_all(player_in, &grid, &seq, hp_expected) else {
                note_reason(&mut last, "bag_no_room", 0, REASON_RANK_BAG);
                continue;
            };
            if out.player.atk != want.atk || out.player.def != want.def {
                note_reason(&mut last, "replay_stat_mismatch", 0, REASON_RANK_REPLAY);
                continue;
            }
            if (out.player.yellow, out.player.blue, out.player.red)
                != (want.yellow, want.blue, want.red)
            {
                note_reason(&mut last, "replay_key_mismatch", 0, REASON_RANK_REPLAY);
                continue;
            }
            // gold 不在证书覆盖的轴里（证书要求 gold 零变化）⇒ 它一動就是没建模的臂。
            if out.player.gold != player_in.gold {
                note_reason(&mut last, "replay_side_effect", 0, REASON_RANK_REPLAY);
                continue;
            }
            if trim_slots(&out.player.item_types) != trim_slots(&want.item_types)
                || trim_slots(&out.player.item_uses) != trim_slots(&want.item_uses)
                || out.player.item_stack_size != want.item_stack_size
            {
                note_reason(&mut last, "replay_bag_mismatch", 0, REASON_RANK_REPLAY);
                continue;
            }
            if out.flags != flags_in {
                note_reason(&mut last, "replay_flags_changed", 0, REASON_RANK_REPLAY);
                continue;
            }
            // 实体表：非拾取物按 (ty,x,y,param) 多重集不变；拾取物必须全消失
            let norm = |v: &[EntRec], keep_pickup: bool| -> Vec<(i32, i32, i32, i32)> {
                let mut r: Vec<(i32, i32, i32, i32)> = v
                    .iter()
                    .filter(|e| keep_pickup == is_pickup(e.ty, input.floor))
                    .map(|e| (e.ty, e.x, e.y, e.param))
                    .collect();
                r.sort_unstable();
                r
            };
            if !norm(&out.ents, true).is_empty() {
                // 有药没被捡走 ⇒ 路线没真捡（或该格另有拾取臂），闭式不成立
                note_reason(&mut last, "replay_pickup_left", 0, REASON_RANK_REPLAY);
                continue;
            }
            if norm(&out.ents, false) != norm(root_ents, false) {
                note_reason(&mut last, "replay_ents_mismatch", 0, REASON_RANK_REPLAY);
                continue;
            }
            // 步数自洽（每格 +1 步；`pickup_step` 也是 +1）
            if out.steps != (cells.len() as i32 - 1) {
                note_reason(&mut last, "replay_steps_mismatch", 0, REASON_RANK_REPLAY);
                continue;
            }

            st.adopted = true;
            st.reason = "adopted";
            st.reason_ty = 0;
            st.route_len = cells.len();
            st.replay_steps = out.steps;
            st.orders_tried = oi;
            st.hp_replayed = out.player.hp;
            st.exit = Some(exit);
            pure_pickup_stat_set(st);
            let route: Vec<String> = cells.iter().map(|&(x, y)| format!("{x},{y}")).collect();
            goals.push(LayerGoal {
                hp: out.player.hp,
                kills: 0, // G1 无怪 ⇒ 闭式层击杀恒 0
                steps: out.steps,
                keys: (out.player.yellow, out.player.blue, out.player.red),
                kind: ExitKind::Stair,
                route,
                post: out.player,
                post_pos: out.pos,
                post_ents: out.ents,
                post_flags: out.flags,
                cross_edge: None, // Stair 由出口梯实体 param 取参（`solve_global::stair_target`）
            });
            done = true;
        }
    }
    if goals.is_empty() {
        return give_up(&mut st, last.0, last.1);
    }
    Some(goals)
}
