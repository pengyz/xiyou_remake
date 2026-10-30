//! 【STAGE-B-008】点地策略纯逻辑（spec §5.1 / §5.2）：格分类 [`TapGrid`]、安全通行 BFS、目标规划 [`plan_tap`]，
//! 以及 `GameScene` 点地状态用到的类型。无 I/O、不读场景私有字段——场景在 scene.rs 里建 `TapGrid` 后调用本模块。
//!
//! 方向码 0 下（y+1）、1 上（y−1）、2 右（x+1）、3 左（x−1）；路线步序 = 目标端优先、自末端消费
//!（与 `walk_path_buffer` 同语义：`steps[len-1]` 是第一步）。

/// 点中格的目标类别。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    /// 可站立：地形可走或已显形隐形路径，且无"存在且可见"的阻挡实体（道具 / 装备格属此类，沿途拾取）
    Floor,
    /// 墙：地形不可走且无可见实体（含未显形的隐形路径——看起来就是墙）
    Wall,
    /// 交互物：门、封印门、商人、告示、楼梯、火、可挖墙、类目 32 及其它可见非怪实体——不可途经，只能作目标（走到邻格再撞）
    Interact,
    /// 炼丹炉（型 9）：优先从正下方朝上撞
    Alchemy,
    /// 怪物（类目 8，含陷阱怪 60/61/62），值 = 实体下标
    Monster(usize),
}

/// 一格的点地分类。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TapCell {
    pub kind: TargetKind,
    /// 站上去会被陷阱伤到（只对 Floor 有意义）——途中不经过；作目标仍可去，无安全停靠格时也可作停靠格（复核 R1）
    pub danger: bool,
}

/// 全图点地分类（行主序）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TapGrid {
    pub cols: i32,
    pub rows: i32,
    pub cells: Vec<TapCell>,
}

impl TapGrid {
    /// 全墙初始化。
    pub fn new(cols: i32, rows: i32) -> TapGrid {
        let n = (cols.max(0) * rows.max(0)) as usize;
        TapGrid { cols, rows, cells: vec![TapCell { kind: TargetKind::Wall, danger: false }; n] }
    }

    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && x < self.cols && y < self.rows).then(|| (y * self.cols + x) as usize)
    }

    pub fn get(&self, x: i32, y: i32) -> Option<TapCell> {
        self.idx(x, y).map(|i| self.cells[i])
    }

    pub fn set(&mut self, x: i32, y: i32, c: TapCell) {
        if let Some(i) = self.idx(x, y) {
            self.cells[i] = c;
        }
    }

    /// 途中可经过：Floor 且不危险。
    pub fn passable(&self, x: i32, y: i32) -> bool {
        matches!(self.get(x, y), Some(TapCell { kind: TargetKind::Floor, danger: false }))
    }
}

/// 四邻（方向码, dx, dy），BFS 扩展与平局取舍都按此固定顺序（确定性）。
pub const DIRS: [(u8, i32, i32); 4] = [(0, 0, 1), (1, 0, -1), (2, 1, 0), (3, -1, 0)];

/// 相邻两格的方向码（不相邻 = None）。
pub fn dir_between(from: (i32, i32), to: (i32, i32)) -> Option<u8> {
    DIRS.iter().find(|&&(_, dx, dy)| (from.0 + dx, from.1 + dy) == to).map(|&(d, _, _)| d)
}

/// BFS 可达表：`dist[i] < 0` = 不可达；`parent[i]` = 前驱格下标。起点恒可达（dist 0）。
pub struct Reach {
    cols: i32,
    pub dist: Vec<i32>,
    parent: Vec<i32>,
}

impl Reach {
    pub fn dist_at(&self, x: i32, y: i32) -> Option<i32> {
        if x < 0 || y < 0 || x >= self.cols {
            return None;
        }
        let i = (y * self.cols + x) as usize;
        self.dist.get(i).copied().filter(|&d| d >= 0)
    }
}

/// 自 `from` 起、只经 [`TapGrid::passable`] 格的 BFS（无扩展上限）。
pub fn bfs(grid: &TapGrid, from: (i32, i32)) -> Reach {
    let n = grid.cells.len();
    let mut dist = vec![-1; n];
    let mut parent = vec![-1; n];
    let mut q = std::collections::VecDeque::new();
    if let Some(i) = grid.idx(from.0, from.1) {
        dist[i] = 0;
        q.push_back(from);
    }
    while let Some((x, y)) = q.pop_front() {
        let i = grid.idx(x, y).unwrap();
        for &(_, dx, dy) in DIRS.iter() {
            let (nx, ny) = (x + dx, y + dy);
            if let Some(j) = grid.idx(nx, ny) {
                if dist[j] < 0 && grid.passable(nx, ny) {
                    dist[j] = dist[i] + 1;
                    parent[j] = i as i32;
                    q.push_back((nx, ny));
                }
            }
        }
    }
    Reach { cols: grid.cols, dist, parent }
}

/// 自起点到可达格 `to` 的步序（目标端优先）。`to` 不可达 = 空。
pub fn path_steps(reach: &Reach, to: (i32, i32)) -> Vec<u8> {
    let cols = reach.cols;
    let mut steps = Vec::new();
    let Some(_) = reach.dist_at(to.0, to.1) else { return steps };
    let mut cur = (to.1 * cols + to.0) as usize;
    while reach.parent[cur] >= 0 {
        let p = reach.parent[cur] as usize;
        let from = ((p as i32) % cols, (p as i32) / cols);
        let at = ((cur as i32) % cols, (cur as i32) / cols);
        steps.push(dir_between(from, at).expect("BFS 父子相邻"));
        cur = p;
    }
    steps
}

/// 点地规划结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TapPlan {
    /// 走到目标格（目标是 Floor）
    Walk { steps: Vec<u8>, dest: (i32, i32) },
    /// 走到停靠格后朝 `dir` 撞一步（墙 / 交互物 / 炉 / 怪）；steps 可空（已在邻格）
    Bump { steps: Vec<u8>, dock: (i32, i32), dir: u8 },
    /// 到不了：走到离目标最近的可达格（steps 可空 = 原地），调用方闪红
    Dock { steps: Vec<u8>, dock: (i32, i32) },
    /// 点在图外
    Ignore,
}

/// 格坐标 (x, y)。
type Cell = (i32, i32);
/// 停靠候选：(停靠格, 撞击方向)。
type DockCand = (Cell, u8);

/// 离目标最近的可达格：曼哈顿距离最小 → 路径最短 → 行主序（y 再 x）。
fn nearest_reachable(grid: &TapGrid, reach: &Reach, target: (i32, i32)) -> (i32, i32) {
    let mut best: Option<((i32, i32, i32, i32), Cell)> = None;
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if let Some(d) = reach.dist_at(x, y) {
                let key = ((x - target.0).abs() + (y - target.1).abs(), d, y, x);
                if best.is_none_or(|(k, _)| key < k) {
                    best = Some((key, (x, y)));
                }
            }
        }
    }
    best.map(|(_, c)| c).expect("起点恒可达")
}

/// 候选停靠格中路径最短者（平局取候选顺序在前者），返回 (停靠格, 撞击方向)。
fn best_dock(reach: &Reach, candidates: &[DockCand]) -> Option<DockCand> {
    let mut best: Option<(i32, DockCand)> = None;
    for &(c, dir) in candidates {
        if let Some(d) = reach.dist_at(c.0, c.1) {
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, (c, dir)));
            }
        }
    }
    best.map(|(_, x)| x)
}

/// `cell` 的四邻候选（按 DIRS 序），方向 = 自该邻格走进 / 撞向 `cell` 的方向码。
fn neighbor_cands(cell: Cell) -> Vec<DockCand> {
    DIRS.iter()
        .map(|&(_, dx, dy)| (cell.0 - dx, cell.1 - dy))
        .filter_map(|c| dir_between(c, cell).map(|d| (c, d)))
        .collect()
}

/// 经 `cell` 的某个 BFS 可达邻格一步走进 `cell` 的步序（目标端优先：进入 `cell` 的一步放最前）。
/// 邻格路径最短者胜，平局按 DIRS 序；无可达邻格 = None。用于危险地面（不可途经，只能作终点 / 停靠格）。
fn enter_via_neighbor(reach: &Reach, cell: Cell) -> Option<Vec<u8>> {
    let (via, dir) = best_dock(reach, &neighbor_cands(cell))?;
    let mut steps = path_steps(reach, via);
    steps.insert(0, dir); // 目标端优先：最后一步放最前
    Some(steps)
}

/// 撞击类目标（墙 / 交互物 / 炉 / 怪）的停靠：先在候选中找安全可达停靠格（[`best_dock`]）；
/// 找不到时，允许候选停靠格是危险地面（`kind == Floor && danger`），经其可达邻格一步进入
///（[`enter_via_neighbor`]），总步数最短者胜，平局按候选顺序、再按邻格 DIRS 序。仍找不到 = None（调用方 Dock）。
///
/// 停靠格只在无安全停靠格时才可为危险格（spec §5.1 目标格例外的延伸，2026-10-30 复核 R1）——
/// 例如陷阱怪 60/62 四邻皆伤害格、门紧邻陷阱时，否则永远点不到。
fn bump_plan(grid: &TapGrid, reach: &Reach, cands: &[DockCand]) -> Option<TapPlan> {
    if let Some((dock, dir)) = best_dock(reach, cands) {
        return Some(TapPlan::Bump { steps: path_steps(reach, dock), dock, dir });
    }
    let mut best: Option<(Vec<u8>, DockCand)> = None;
    for &(c, dir) in cands {
        if !matches!(grid.get(c.0, c.1), Some(TapCell { kind: TargetKind::Floor, danger: true })) {
            continue;
        }
        if let Some(steps) = enter_via_neighbor(reach, c) {
            if best.as_ref().is_none_or(|(bs, _)| steps.len() < bs.len()) {
                best = Some((steps, (c, dir)));
            }
        }
    }
    best.map(|(steps, (dock, dir))| TapPlan::Bump { steps, dock, dir })
}

/// 点中 `target` 的规划（spec §5.2；怪物的"先选中"由调用方处理，这里只给撞击计划）。
///
/// `target` 或 `from` 不在图内 ⇒ [`TapPlan::Ignore`]（2026-10-30 复核 R8）；
/// `target == from` ⇒ [`TapPlan::Ignore`]：点自己由调用方处理（寻路中 = 停下）（复核 R9）。
pub fn plan_tap(grid: &TapGrid, from: (i32, i32), target: (i32, i32)) -> TapPlan {
    let Some(cell) = grid.get(target.0, target.1) else { return TapPlan::Ignore };
    if grid.get(from.0, from.1).is_none() || target == from {
        return TapPlan::Ignore;
    }
    let reach = bfs(grid, from);
    let dock_plan = |reach: &Reach| {
        let dock = nearest_reachable(grid, reach, target);
        TapPlan::Dock { steps: path_steps(reach, dock), dock }
    };
    match cell.kind {
        TargetKind::Floor => {
            if reach.dist_at(target.0, target.1).is_some() {
                return TapPlan::Walk { steps: path_steps(&reach, target), dest: target };
            }
            // 危险格本身作目标：经可达邻格走进去（安全的不可达地面不会有可达邻格，自然落到 Dock）
            if let Some(steps) = enter_via_neighbor(&reach, target) {
                return TapPlan::Walk { steps, dest: target };
            }
            dock_plan(&reach)
        }
        TargetKind::Wall | TargetKind::Interact => {
            bump_plan(grid, &reach, &neighbor_cands(target)).unwrap_or_else(|| dock_plan(&reach))
        }
        TargetKind::Monster(e) => {
            // 多格怪（型 69 宽 3 格，每格都登记槽）：同一实体任一格的外侧邻格都可停靠（spec §5.2"含多格怪的任一格"）
            let mut cands: Vec<DockCand> = Vec::new();
            for y in 0..grid.rows {
                for x in 0..grid.cols {
                    if grid.get(x, y).map(|c| c.kind) != Some(TargetKind::Monster(e)) {
                        continue;
                    }
                    for &(d, dx, dy) in DIRS.iter() {
                        let c = (x - dx, y - dy);
                        if grid.get(c.0, c.1).map(|k| k.kind) != Some(TargetKind::Monster(e)) {
                            cands.push((c, d));
                        }
                    }
                }
            }
            bump_plan(grid, &reach, &cands).unwrap_or_else(|| dock_plan(&reach))
        }
        TargetKind::Alchemy => {
            // 同一行连续的炉格（炉宽 3 格），每格正下方朝上撞。
            // 平局规则：路径最短 → x 小者（候选按 x 升序，best_dock 先到先得；2026-10-30 复核 R6）
            let mut xs = vec![target.0];
            let mut x = target.0 - 1;
            while matches!(grid.get(x, target.1), Some(TapCell { kind: TargetKind::Alchemy, .. })) {
                xs.push(x);
                x -= 1;
            }
            let mut x = target.0 + 1;
            while matches!(grid.get(x, target.1), Some(TapCell { kind: TargetKind::Alchemy, .. })) {
                xs.push(x);
                x += 1;
            }
            xs.sort();
            let cands: Vec<DockCand> = xs.into_iter().map(|ax| ((ax, target.1 + 1), 1u8)).collect();
            bump_plan(grid, &reach, &cands).unwrap_or_else(|| dock_plan(&reach))
        }
    }
}

/// 已选中的怪物（spec §5.2：首点选中、再点开战）。`cell` = 实体主格（`px >> 5, py >> 5`），`floor` = 选中时楼层号；
/// 每拍按这些字段校验，任一变化即清选中。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedEnemy {
    pub entity: usize,
    pub type_: i32,
    pub cell: (i32, i32),
    pub floor: i32,
}

/// [`crate::scene::GameScene::tap_cell`] 的结果（壳据此决定反馈）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapResult {
    /// 不处理（点自己、观察模式、场上不收点按、图外）
    Ignored,
    /// 开始走向目标格
    Walking,
    /// 走到邻格再撞（或已在邻格、当场撞）
    Bumping,
    /// 到不了：走到最近可达格（或原地），目标格闪红
    Docking,
    /// 怪物首点：选中（Task 6）
    Selected,
    /// 点地寻路中：排队，走完当前格后改道（Task 7）
    Queued,
    /// 点地寻路中点自己：停下
    Stopped,
    /// 场上忙（手动行走 / 战斗）：闪红叉，状态不变
    Rejected,
}

/// 闪红持续拍数（Android / Windows 逻辑拍 75 ms ⇒ 约 0.6 s）。
pub const TAP_REJECT_TICKS: i64 = 8;

#[cfg(test)]
mod tests {
    use super::*;

    const F: TapCell = TapCell { kind: TargetKind::Floor, danger: false };
    const W: TapCell = TapCell { kind: TargetKind::Wall, danger: false };

    /// 字符画建格：`.` 地面、`#` 墙、`D` 交互物、`A` 炉、`M` 怪（实体 0）、`!` 危险地面。
    fn grid(rows: &[&str]) -> TapGrid {
        let mut g = TapGrid::new(rows[0].len() as i32, rows.len() as i32);
        for (y, r) in rows.iter().enumerate() {
            for (x, ch) in r.chars().enumerate() {
                let c = match ch {
                    '.' => F,
                    '#' => W,
                    'D' => TapCell { kind: TargetKind::Interact, danger: false },
                    'A' => TapCell { kind: TargetKind::Alchemy, danger: false },
                    'M' => TapCell { kind: TargetKind::Monster(0), danger: false },
                    '!' => TapCell { kind: TargetKind::Floor, danger: true },
                    _ => panic!("未知格 {ch}"),
                };
                g.set(x as i32, y as i32, c);
            }
        }
        g
    }

    /// 按步序（目标端优先）自起点走一遍，返回经过的格（不含起点）。
    fn walk(from: (i32, i32), steps: &[u8]) -> Vec<(i32, i32)> {
        let mut at = from;
        let mut out = Vec::new();
        for &d in steps.iter().rev() {
            let &(_, dx, dy) = DIRS.iter().find(|&&(c, _, _)| c == d).unwrap();
            at = (at.0 + dx, at.1 + dy);
            out.push(at);
        }
        out
    }

    #[test]
    fn walk_reachable_floor_by_shortest_path() {
        let g = grid(&["....", ".##.", "...."]);
        let plan = plan_tap(&g, (0, 0), (3, 2));
        let TapPlan::Walk { steps, dest } = plan else { panic!("应直达：{plan:?}") };
        assert_eq!(dest, (3, 2));
        assert_eq!(steps.len(), 5, "最短 5 步");
        assert_eq!(walk((0, 0), &steps).last(), Some(&(3, 2)));
        assert!(walk((0, 0), &steps).iter().all(|&(x, y)| g.passable(x, y)), "全程只经可通行格");
    }

    #[test]
    fn route_never_crosses_interact_monster_or_danger() {
        // 唯一直线通道上分别放门 / 怪 / 危险格：路线必须绕行或停靠，绝不穿过
        for mid in ['D', 'M', '!'] {
            let row1 = format!(".{mid}.");
            let g = grid(&["...", row1.as_str(), "..."]);
            let plan = plan_tap(&g, (0, 1), (2, 1));
            let TapPlan::Walk { steps, .. } = plan else { panic!("{mid}：可绕行应直达：{plan:?}") };
            assert!(!walk((0, 1), &steps).contains(&(1, 1)), "{mid}：路线不经 (1,1)");
        }
    }

    #[test]
    fn unreachable_floor_docks_nearest_reachable() {
        let g = grid(&["..#..", "..#..", "..#.."]);
        let plan = plan_tap(&g, (0, 1), (4, 1));
        assert_eq!(plan, TapPlan::Dock { steps: vec![2], dock: (1, 1) }, "墙后不可达：停在曼哈顿最近的 (1,1)");
    }

    #[test]
    fn wall_target_walks_to_adjacent_then_bumps() {
        let g = grid(&[".....", "....#", "....."]);
        let plan = plan_tap(&g, (0, 1), (4, 1));
        let TapPlan::Bump { steps, dock, dir } = plan else { panic!("{plan:?}") };
        assert_eq!((dock, dir), ((3, 1), 2), "停在墙左邻、朝右撞");
        assert_eq!(walk((0, 1), &steps).last(), Some(&(3, 1)));
    }

    #[test]
    fn adjacent_target_bumps_with_zero_steps() {
        let g = grid(&[".D."]);
        assert_eq!(plan_tap(&g, (0, 0), (1, 0)), TapPlan::Bump { steps: vec![], dock: (0, 0), dir: 2 });
    }

    #[test]
    fn walled_in_interact_docks_nearest() {
        let g = grid(&["..###", "..#D#", "..###"]);
        let plan = plan_tap(&g, (0, 1), (3, 1));
        assert!(matches!(plan, TapPlan::Dock { dock: (1, 1), .. }), "交互物四邻都不可达 ⇒ 停靠最近：{plan:?}");
    }

    #[test]
    fn multi_cell_monster_docks_beside_any_of_its_cells() {
        // 三格怪 (1..=3,1)，上下是墙：点中间格 (2,1) 无正交邻格可停，须停在整只怪的左侧外邻 (0,1) 朝右撞
        let g = grid(&["#####", ".MMM.", "#####"]);
        assert_eq!(plan_tap(&g, (0, 1), (2, 1)), TapPlan::Bump { steps: vec![], dock: (0, 1), dir: 2 });
    }

    #[test]
    fn alchemy_is_bumped_from_below() {
        // 炉三格 (1..=3,0)；正下方 (2,1) 可达，左右侧也可达——必须选正下方朝上
        let g = grid(&[".AAA.", "....."]);
        let plan = plan_tap(&g, (0, 0), (1, 0));
        let TapPlan::Bump { dock, dir, .. } = plan else { panic!("{plan:?}") };
        assert_eq!(dir, 1, "朝上撞");
        assert_eq!(dock.1, 1, "停在炉的下一行");
        assert!((1..=3).contains(&dock.0), "停在三格之一的正下方");
    }

    #[test]
    fn alchemy_without_reachable_below_docks() {
        let g = grid(&[".AAA.", "#####"]);
        assert!(matches!(plan_tap(&g, (0, 0), (2, 0)), TapPlan::Dock { .. }), "下方都不可达 ⇒ 停靠闪红");
    }

    #[test]
    fn danger_floor_as_target_is_entered_last() {
        let g = grid(&["..!"]);
        let plan = plan_tap(&g, (0, 0), (2, 0));
        let TapPlan::Walk { steps, dest } = plan else { panic!("{plan:?}") };
        assert_eq!(dest, (2, 0));
        assert_eq!(walk((0, 0), &steps), vec![(1, 0), (2, 0)], "危险格只作终点");
    }

    #[test]
    fn outside_is_ignored_and_bfs_has_no_expansion_cap() {
        let g = grid(&["."; 1]);
        assert_eq!(plan_tap(&g, (0, 0), (5, 5)), TapPlan::Ignore);
        // 40×40 蛇形长路（> find_path 的 100 扩展上限）
        let mut rows: Vec<String> = Vec::new();
        for y in 0..40 {
            let r: String = (0..40)
                .map(|x| if y % 2 == 1 && ((y / 2) % 2 == 0 && x != 39 || (y / 2) % 2 == 1 && x != 0) { '#' } else { '.' })
                .collect();
            rows.push(r);
        }
        let refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
        let g = grid(&refs);
        let plan = plan_tap(&g, (0, 0), (0, 39));
        assert!(matches!(plan, TapPlan::Walk { .. }), "长蛇形路线可达（BFS 无扩展上限）");
    }

    // ---- 2026-10-30 复核补测（target/verify/review_plan2_T2.md R1–R9）----
    // 期望一律手算。方向码 0 下 1 上 2 右 3 左；步序目标端优先（steps[len-1] 是第一步）；
    // BFS 按 DIRS 序（下、上、右、左）扩展，先入队者成为后继格的父格。

    #[test]
    fn r1_trap_monster_and_door_dock_on_danger_floor_when_no_safe_dock() {
        // p6：怪 (3,0) 唯一图内邻格 (2,0) 是危险地面 ⇒ 无安全停靠格，回退到危险停靠格：
        // (0,0) 右走到 (1,0)（path_steps = [2]），再右进 (2,0)（insert(0, 2)）⇒ [2,2]；在 (2,0) 朝右(2)撞怪
        let g = grid(&["..!M"]);
        assert_eq!(
            plan_tap(&g, (0, 0), (3, 0)),
            TapPlan::Bump { steps: vec![2, 2], dock: (2, 0), dir: 2 },
            "p6 陷阱怪：唯一邻格是危险格时，危险格作停靠格"
        );
        // p7：门 (3,0) 紧邻危险格 (2,0)，推导同 p6
        let g = grid(&["..!D"]);
        assert_eq!(
            plan_tap(&g, (0, 0), (3, 0)),
            TapPlan::Bump { steps: vec![2, 2], dock: (2, 0), dir: 2 },
            "p7 门紧邻危险格：危险格作停靠格"
        );
    }

    #[test]
    fn r1_safe_dock_preferred_over_shorter_danger_dock() {
        // 对照：怪 (3,0) 的左邻 (2,0) 危险（经 (1,0) 进入共 2 步），下邻 (3,1) 安全（4 步）。
        // 有安全停靠格就不选危险停靠格，即使更远 ⇒ 停 (3,1) 朝上(1)撞。
        // BFS：(0,0) 先扩下 (0,1) 再扩右 (1,0)；(0,1) 先出队 ⇒ (1,1) 父为 (0,1)；之后 (2,1)、(3,1) 依次向右。
        // 动作序 下 右 右 右 ⇒ 目标端优先 [2,2,2,0]（非回文，同时钉住步序方向，R2）
        let g = grid(&["..!M", "...."]);
        assert_eq!(
            plan_tap(&g, (0, 0), (3, 0)),
            TapPlan::Bump { steps: vec![2, 2, 2, 0], dock: (3, 1), dir: 1 },
            "有安全停靠格时不选危险停靠格"
        );
    }

    #[test]
    fn r2_exact_step_order_on_turning_routes() {
        // p3：危险终点 (2,1)；其上邻 (2,0) 是墙，左邻 (1,1) 可达。
        // 到 (1,1)：(0,0)→右(1,0)→下(1,1)，path_steps = [0,2]；再 insert(0, 右2) ⇒ [2,0,2]
        let g = grid(&["..#", "#.!"]);
        assert_eq!(
            plan_tap(&g, (0, 0), (2, 1)),
            TapPlan::Walk { steps: vec![2, 0, 2], dest: (2, 1) },
            "p3 危险终点带拐弯：进入步在最前"
        );
        // p4：L 形唯一通道 (0,0)→右(1,0)→下(1,1)→下(1,2)→右(2,2)，动作序 [2,0,0,2]，倒排后相同
        let g = grid(&["..#", "#.#", "#.."]);
        assert_eq!(
            plan_tap(&g, (0, 0), (2, 2)),
            TapPlan::Walk { steps: vec![2, 0, 0, 2], dest: (2, 2) },
            "p4 L 形唯一通道"
        );
    }

    #[test]
    fn r3_open_grid_routes_are_shortest() {
        // p5：开阔 5×5，(0,0)→(4,4) 曼哈顿 4+4 = 8 即最短（DFS 会绕成 16 步）
        let g = grid(&["....."; 5]);
        let plan = plan_tap(&g, (0, 0), (4, 4));
        let TapPlan::Walk { steps, dest } = plan else { panic!("开阔图应直达：{plan:?}") };
        assert_eq!(dest, (4, 4));
        assert_eq!(steps.len(), 8, "p5 最短 8 步");
        let cells = walk((0, 0), &steps);
        assert_eq!(cells.last(), Some(&(4, 4)), "p5 终点 (4,4)");
        assert!(cells.iter().all(|&(x, y)| g.passable(x, y)), "p5 全程可通行");
        // (2,2)→(2,0) 正上方两格：上、上
        assert_eq!(
            plan_tap(&g, (2, 2), (2, 0)),
            TapPlan::Walk { steps: vec![1, 1], dest: (2, 0) },
            "p5 正上方两格 = [1,1]"
        );
    }

    #[test]
    fn r4_unreachable_floor_tie_breaks_path_then_row_major() {
        // 封闭地面 (2,2)（上邻 (2,1)、左邻 (1,2) 皆墙）。可达：(0,0)(1,0)(2,0)(0,1)(0,2)。
        // 曼哈顿最小 = 2：(2,0) 与 (0,2)。
        let g = grid(&["...", ".##", ".#."]);
        // p1：自 (0,0)，两者路径都 2 ⇒ 行主序取 y 小的 (2,0)；路线 右 右 ⇒ [2,2]
        assert_eq!(
            plan_tap(&g, (0, 0), (2, 2)),
            TapPlan::Dock { steps: vec![2, 2], dock: (2, 0) },
            "p1 曼哈顿、路径平局 ⇒ 行主序 (2,0)"
        );
        // p2：自 (0,1)，(0,2) 路径 1 < (2,0) 路径 3 ⇒ 路径更短者胜过行主序；路线 下 ⇒ [0]
        assert_eq!(
            plan_tap(&g, (0, 1), (2, 2)),
            TapPlan::Dock { steps: vec![0], dock: (0, 2) },
            "p2 路径更短者 (0,2) 胜过行主序"
        );
    }

    #[test]
    fn r5_alchemy_uses_sibling_cell_below() {
        // p11：点炉 (3,0)，其正下方 (3,1) 与中格下方 (2,1) 是墙，只有左格下方 (1,1) 可达。
        // (0,0)→下(0,1)→右(1,1)（炉格不可途经）⇒ [2,0]；朝上(1)撞
        let g = grid(&[".AAA.", "..##."]);
        assert_eq!(
            plan_tap(&g, (0, 0), (3, 0)),
            TapPlan::Bump { steps: vec![2, 0], dock: (1, 1), dir: 1 },
            "p11 被点格下方不通时用兄弟格下方"
        );
    }

    #[test]
    fn r6_alchemy_tie_prefers_smaller_x() {
        // p12：自 (2,2) 点炉 (2,0)；中格下方 (2,1) 是墙；(1,1)、(3,1) 都是 2 步 ⇒ 平局取 x 小 (1,1)。
        // BFS：(2,2) 先扩右 (3,2) 再扩左 (1,2)；(1,1) 父为 (1,2)。动作 左 上 ⇒ [1,3]
        let g = grid(&[".AAA.", "..#..", "....."]);
        assert_eq!(
            plan_tap(&g, (2, 2), (2, 0)),
            TapPlan::Bump { steps: vec![1, 3], dock: (1, 1), dir: 1 },
            "p12 炉平局：路径最短 → x 小者"
        );
    }

    #[test]
    fn r7_bump_tie_follows_candidate_order_and_bfs_follows_dirs() {
        // p13：自 (1,0) 点墙 (2,1)；上邻 (2,0)（右 1 步）与左邻 (1,1)（下 1 步）同为 1 步。
        // 候选按 DIRS 序：上邻(朝下 0) → 下邻(图外) → 左邻(朝右 2) → 右邻(图外) ⇒ 取上邻 (2,0)，朝下撞
        let g = grid(&["...", "..#"]);
        assert_eq!(
            plan_tap(&g, (1, 0), (2, 1)),
            TapPlan::Bump { steps: vec![2], dock: (2, 0), dir: 0 },
            "p13 墙停靠平局按候选序：上邻优先"
        );
        // 同长多路：3×3 开阔，(0,0)→(1,1) 有 下右 / 右下 两条 2 步路线。
        // BFS 先扩下 (0,1) 再扩右 (1,0)；(0,1) 先出队 ⇒ (1,1) 父为 (0,1)：动作 下 右 ⇒ [2,0]
        let g = grid(&["..."; 3]);
        assert_eq!(
            plan_tap(&g, (0, 0), (1, 1)),
            TapPlan::Walk { steps: vec![2, 0], dest: (1, 1) },
            "同长多路按 DIRS 序（下先于右）取 下、右"
        );
    }

    #[test]
    fn r8_from_outside_grid_is_ignored() {
        // p9：起点不在图内 ⇒ Ignore（原先会 panic「起点恒可达」）
        let g = grid(&["..."]);
        assert_eq!(plan_tap(&g, (5, 5), (0, 0)), TapPlan::Ignore, "p9 起点远在图外");
        assert_eq!(plan_tap(&g, (-1, 0), (0, 0)), TapPlan::Ignore, "p9 起点 x = -1");
        assert_eq!(plan_tap(&g, (3, 0), (0, 0)), TapPlan::Ignore, "p9 起点 x = cols（右缘外）");
    }

    #[test]
    fn r9_tap_self_is_ignored() {
        // p10：点自己 ⇒ Ignore（由调用方处理：寻路中 = 停下）
        let g = grid(&["..!"]);
        assert_eq!(plan_tap(&g, (0, 0), (0, 0)), TapPlan::Ignore, "p10 站在普通地面点自己");
        assert_eq!(plan_tap(&g, (2, 0), (2, 0)), TapPlan::Ignore, "p10 站在危险格上点自己");
    }
}
