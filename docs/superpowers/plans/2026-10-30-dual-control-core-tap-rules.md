# 双操控方案 core 点地策略 Implementation Plan（计划 2 / 3）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 spec（`docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md`）§5 点地规则、§6 反馈状态与绘制、§5.2 kind 4 赠予框点空白不代按落到 game-core，并让 Android / Windows 两个壳改用新入口。

**Architecture:** 新纯逻辑模块 `crates/game-core/src/tap_walk.rs`（格分类 `TapGrid`、BFS、目标规划 `plan_tap`、点地状态类型），不依赖场景私有字段、可单测。`GameScene` 在 scene.rs 增加入口 `tap_cell` / `swipe_step` / `accepts_field_tap` / `clear_tap_state`，负责建格分类（含陷阱谓词）、怪物选中、撞击（一律在 Map walkPhase 0 执行）、行走中排队、标记绘制。旧 `walk_to` 保留（测试与脚本面仍用），壳层改调 `tap_cell`。全部只影响点地 / 鼠标入口与 modern 绘制；classic 按键链、oracle 回放零变化。

**Tech Stack:** Rust（cargo test --offline），Python 3（gates）。

**范围外（计划 3）：** 工具栏、操控方式 / 自动战斗设置行（本计划只给 core 字段 `auto_battle`，默认 false）、Android 视口滑动 / 拖动手势、"停下后打开"、点按震动 / 按键音、提示卡、帮助文本、去掉 Android 隐形撤销热区。

**通用约定（每个任务都适用）：**
- 纪律：`AGENTS.md`。commit message 正文必须含"动手前三问"与 `Evidence:` / `Confidence:` 行。
- **mtime / 落盘 gotcha**（`docs/knowledge/gotcha_agent-edit-tool-keeps-mtime-cargo-stale.md`）：编辑工具写入可能晚于后续命令落盘、mtime 可能不变。改源码后 `sleep 2; touch <文件>` 再跑测试，确认输出有 `Compiling <crate>` 且测试条数符合预期；做变异备份前先 sleep，恢复后 grep 确认新增内容仍在；布尔条件变异用整体替换。
- 夹具 `scene_test_support::scene` 的 `player.hp` 默认 0：需要开战的测试显式设 hp > 0。`tick` 不推进 wp4 开门动画（paint 推进）。
- 全量测试：`cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle`，统计 `grep -cE '^test .* FAILED$|panicked'` 应为 0。
- 每个任务 GREEN 后、commit 前派独立复核子代理（提示词模板见文末附录 A），记录写 `target/verify/review_plan2_T<N>.md`，认同的意见修完再提交。
- 测试数增加的 ratchet 更新统一在 Task 13 单独 `gate(baselines)` 提交。
- 方向码：0 下（y+1）、1 上（y−1）、2 右（x+1）、3 左（x−1）。路线步序（`walk_path_buffer`）为**目标端优先、自末端消费**（`steps[len-1]` 是第一步）。

---

### Task 1: 零长位移笔不入撤销栈

**Files:**
- Modify: `crates/game-core/src/pathfind.rs`（`RouteTracker::close`，约 426 行）
- Modify: `crates/game-core/src/scene.rs`（测试 `t2_interrupt_all_channels_clear_buffer` 的一条断言，约 10911 行）
- Test: `crates/game-core/tests/walk_route.rs` 末尾追加

背景：`close` 不判 `from == to`，撞墙 / 撞门 / 紧邻开战时收出一笔零长位移，撤销要多按一次空操作（spec §5.2、BUG-044 余项③、计划 1 复核 T2-3 / T4-R6）。

- [ ] **Step 1: 写失败测试**

`crates/game-core/tests/walk_route.rs` 末尾追加：

```rust
/// 【STAGE-B-008 · spec §5.2】零长位移笔（段起点 = 终点）不入撤销栈：撞墙 / 紧邻开战收段时撤销不多一次空操作。
#[test]
fn zero_length_segment_is_not_pushed() {
    let mut journal = UndoJournal::new();
    let mut track = RouteTracker::new();
    track.begin((4, 4));
    track.arrive((4, 4), &mut journal);
    assert_eq!(journal.len(), 0, "arrive 于起点：不入栈");
    track.begin((4, 4));
    track.interrupt((4, 4), &mut journal);
    assert_eq!(journal.len(), 0, "interrupt 于起点：不入栈");
    track.begin((4, 4));
    track.suspend((4, 4), &mut journal);
    assert_eq!(journal.len(), 0, "suspend（紧邻开战）于起点：不入栈");
    assert!(track.is_suspended(), "挂起语义不变");
    track.begin((4, 4));
    track.arrive((5, 4), &mut journal);
    assert_eq!(journal.len(), 1, "非零长照常入栈");
}
```

- [ ] **Step 2: 运行，确认失败**

Run: `cargo test --offline -p game-core --test walk_route zero_length -- --nocapture`
Expected: FAIL（"arrive 于起点：不入栈"，left 1 right 0）。

- [ ] **Step 3: 实现**

`crates/game-core/src/pathfind.rs` 的 `close`：

```rust
    fn close(&mut self, at: (i32, i32), journal: &mut UndoJournal) {
        if self.seg_open {
            // 【STAGE-B-008】零长段（起点 = 终点：撞墙 / 撞门 / 紧邻开战）不入栈——撤销不多一次看不见效果的空操作
            if self.seg_from != at {
                journal.push_move(MoveStroke { from: self.seg_from, to: at });
            }
            self.seg_open = false;
        }
    }
```

`crates/game-core/src/scene.rs` 测试 `t2_interrupt_all_channels_clear_buffer` 中：

```rust
        assert_eq!(sc.undo.len(), 1, "中断段=实际终点入栈（未动=起点段）");
```
改为：
```rust
        assert_eq!(sc.undo.len(), 0, "中断段于起点（未动）= 零长段，不入栈（STAGE-B-008）");
```

- [ ] **Step 4: 运行，确认通过 + 全量**

Run: `cargo test --offline -p game-core --test walk_route` → 全部 passed。
Run 全量测试（见通用约定）。若有其它测试断言零长笔入栈而变红：确认它钉的就是零长笔（起点 = 终点），按"不入栈"改期望并在 commit 正文逐条列出；否则停下报告。

- [ ] **Step 5: 复核 + 提交**

附录 A 复核（T1）。提交：

```bash
git add crates/game-core/src/pathfind.rs crates/game-core/src/scene.rs crates/game-core/tests/walk_route.rs
git commit -F - <<'EOF'
feat(core): 零长位移笔不入撤销栈（STAGE-B-008 · spec §5.2）

RouteTracker::close 遇起点 = 终点不 push：撞墙 / 撞门 / 紧邻开战收段时撤销不再多一次空操作（BUG-044 余项③）。
撤销是 Stage B 功能，classic 不开路段，零变化。

动手前三问：
1. 层：Stage B 撤销日志（pathfind.rs RouteTracker）。
2. 消费者：undo_last（Android / Windows 撤销入口）。
3. 证据：A 级——walk_route zero_length_segment_is_not_pushed RED→GREEN；全量测试 0 失败。
Evidence: walk_route.rs 新测；scene.rs t2_interrupt_all_channels_clear_buffer 期望由 1 改 0（零长笔）。
Confidence: 90
EOF
```

---

### Task 2: 纯逻辑模块 `tap_walk.rs`（格分类、BFS、目标规划）

**Files:**
- Create: `crates/game-core/src/tap_walk.rs`
- Modify: `crates/game-core/src/lib.rs`（加 `pub mod tap_walk;`）

- [ ] **Step 1: 写模块骨架与失败测试**

Create `crates/game-core/src/tap_walk.rs`：

```rust
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
    /// 站上去会被陷阱伤到（只对 Floor 有意义）——途中不经过，作目标仍可去
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

/// 离目标最近的可达格：曼哈顿距离最小 → 路径最短 → 行主序（y 再 x）。
fn nearest_reachable(grid: &TapGrid, reach: &Reach, target: (i32, i32)) -> (i32, i32) {
    let mut best: Option<((i32, i32, i32, i32), (i32, i32))> = None;
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if let Some(d) = reach.dist_at(x, y) {
                let key = ((x - target.0).abs() + (y - target.1).abs(), d, y, x);
                if best.map_or(true, |(k, _)| key < k) {
                    best = Some((key, (x, y)));
                }
            }
        }
    }
    best.map(|(_, c)| c).expect("起点恒可达")
}

/// 候选停靠格中路径最短者（平局取候选顺序在前者），返回 (停靠格, 撞击方向)。
fn best_dock(reach: &Reach, candidates: &[((i32, i32), u8)]) -> Option<((i32, i32), u8)> {
    let mut best: Option<(i32, ((i32, i32), u8))> = None;
    for &(c, dir) in candidates {
        if let Some(d) = reach.dist_at(c.0, c.1) {
            if best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, (c, dir)));
            }
        }
    }
    best.map(|(_, x)| x)
}

/// 点中 `target` 的规划（spec §5.2；怪物的"先选中"由调用方处理，这里只给撞击计划）。
pub fn plan_tap(grid: &TapGrid, from: (i32, i32), target: (i32, i32)) -> TapPlan {
    let Some(cell) = grid.get(target.0, target.1) else { return TapPlan::Ignore };
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
            // 危险格本身作目标：经可达邻格走进去
            let cands: Vec<((i32, i32), u8)> = DIRS
                .iter()
                .map(|&(_, dx, dy)| (target.0 - dx, target.1 - dy))
                .filter_map(|c| dir_between(c, target).map(|d| (c, d)))
                .collect();
            if let Some((dock, dir)) = best_dock(&reach, &cands) {
                let mut steps = path_steps(&reach, dock);
                steps.insert(0, dir); // 目标端优先：最后一步放最前
                return TapPlan::Walk { steps, dest: target };
            }
            dock_plan(&reach)
        }
        TargetKind::Wall | TargetKind::Interact => {
            let cands: Vec<((i32, i32), u8)> = DIRS
                .iter()
                .map(|&(_, dx, dy)| (target.0 - dx, target.1 - dy))
                .filter_map(|c| dir_between(c, target).map(|d| (c, d)))
                .collect();
            match best_dock(&reach, &cands) {
                Some((dock, dir)) => TapPlan::Bump { steps: path_steps(&reach, dock), dock, dir },
                None => dock_plan(&reach),
            }
        }
        TargetKind::Monster(e) => {
            // 多格怪（型 69 宽 3 格，每格都登记槽）：同一实体任一格的外侧邻格都可停靠（spec §5.2"含多格怪的任一格"）
            let mut cands: Vec<((i32, i32), u8)> = Vec::new();
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
            match best_dock(&reach, &cands) {
                Some((dock, dir)) => TapPlan::Bump { steps: path_steps(&reach, dock), dock, dir },
                None => dock_plan(&reach),
            }
        }
        TargetKind::Alchemy => {
            // 同一行连续的炉格（炉宽 3 格），每格正下方朝上撞
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
            let cands: Vec<((i32, i32), u8)> = xs.into_iter().map(|ax| ((ax, target.1 + 1), 1u8)).collect();
            match best_dock(&reach, &cands) {
                Some((dock, dir)) => TapPlan::Bump { steps: path_steps(&reach, dock), dock, dir },
                None => dock_plan(&reach),
            }
        }
    }
}

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
}
```

`crates/game-core/src/lib.rs` 在 `pub mod undo_journal;` 前加一行：

```rust
pub mod tap_walk;
```

- [ ] **Step 2: 运行**

Run: `sleep 2; touch crates/game-core/src/tap_walk.rs crates/game-core/src/lib.rs; cargo test --offline -p game-core --lib tap_walk`
Expected: 11 passed（模块新建，测试与实现同提交；先写测试再写实现的 RED 证据：把 `plan_tap` 的 `TargetKind::Floor` 臂整体替换为 `TapPlan::Ignore` 跑一次，Expected 多条 FAIL，恢复）。

- [ ] **Step 3: 变异**

逐个做、逐个恢复：
- `passable` 改为 `!matches!(…Wall…)`（允许经过交互物）⇒ `route_never_crosses_interact_monster_or_danger` 红；
- `Alchemy` 臂的候选改为四邻 ⇒ `alchemy_is_bumped_from_below` 红；
- `nearest_reachable` 的 key 去掉曼哈顿项 ⇒ `unreachable_floor_docks_nearest_reachable` 红；
- `Monster` 臂改回只取点中格的四邻 ⇒ `multi_cell_monster_docks_beside_any_of_its_cells` 红。

- [ ] **Step 4: 复核 + 提交**

附录 A 复核（T2）。提交：

```bash
git add crates/game-core/src/tap_walk.rs crates/game-core/src/lib.rs
git commit -F - <<'EOF'
feat(core): 点地策略纯逻辑模块 tap_walk（格分类 / 安全通行 BFS / 目标规划）（STAGE-B-008）

TapGrid 五类目标（地面 / 墙 / 交互物 / 炉 / 怪）+ 危险标记；BFS 只经安全地面、无扩展上限；plan_tap：
可达地面直达、危险地面只作终点、墙 / 交互物 / 怪走到邻格再撞、炉优先从正下方朝上撞、到不了停最近可达格（曼哈顿 → 路径 → 行主序）。
纯函数，尚无调用方。

动手前三问：
1. 层：Stage B 新增模块（不触 classic）。
2. 消费者：后续 GameScene::tap_cell（Task 5）。
3. 证据：A 级——11 条单测 + 4 个变异被杀。
Evidence: crates/game-core/src/tap_walk.rs tests。
Confidence: 90
EOF
```

---

### Task 3: 陷阱伤害谓词 `trap_damage_at`（与 classic 原块逐格对拍）

**Files:**
- Modify: `crates/game-core/src/scene.rs`（在 `entity_at_cell`（约 6443 行）之后新增方法；测试追加到 `mod t2_walkto_route_tests` 末尾）

背景：classic 陷阱块在 `apply_step_cell_effects`（约 5806 行起）：61 左右或上下夹击、62 邻接、60 邻接扣血；装备 40 三者全免（a.java:5555-5621，gameplay.md R-scene-4 已更正）。原式怪癖：62 / 60 右邻判界用 `view.high`；61 两侧都须在界内；60 / 62 左邻无显式判界（`entity_at_cell` 对越界返回 −1）。
> 【2026-10-30 更正 · 复核 T3-R3】Java 的 62 / 60 左邻**都有**显式 `x > 0`（javap 字节码 applyStepCellEffects 偏移 231–233、399–401）；只有 Rust classic 块的 62 左邻省略，靠 `cell_entity` 越界返回 −1 保持等价。另：唯一非方形层是 floor 51（12×13，无陷阱），floor 49 为 13×13。谓词**独立实现、无副作用**，classic 块一行不动；等价性由对拍测试证明。

- [ ] **Step 1: 写失败测试（对拍）**

`mod t2_walkto_route_tests` 末尾追加：

```rust
    /// 【STAGE-B-008 · spec §2.2 保真红线】陷阱谓词与 classic 原块（apply_step_cell_effects）逐格对拍：
    /// 合成摆放 × 全格 × 装备 {0, 40}，"站上该格是否扣血"须一致。原块是裁判（实跑扣血），谓词不得由原块推导。
    #[test]
    fn trap_predicate_matches_step_effects_everywhere() {
        // 摆放：61 夹击（横 / 竖）、62 邻接、60 邻接（含 60 上邻的上爬分支）、贴边（判界怪癖）
        let layouts: &[&[(u8, i32, i32)]] = &[
            &[(61, 4, ROW), (61, 6, ROW)],
            &[(61, 5, ROW - 1), (61, 5, ROW + 1)],
            &[(62, 7, ROW)],
            &[(60, 3, ROW)],
            &[(60, 8, ROW - 1)],
            &[(62, 0, ROW), (60, 12, ROW)],
            &[(61, 0, ROW), (61, 2, ROW), (62, 11, ROW + 1)],
        ];
        for (li, ents) in layouts.iter().enumerate() {
            for armor in [0, 40] {
                for y in 0..13 {
                    for x in 0..13 {
                        if ents.iter().any(|&(_, ex, ey)| (ex, ey) == (x, y)) {
                            continue;
                        }
                        let mut sc = scene((1, ROW), ents);
                        sc.equipped_armor = armor;
                        let predicted = sc.trap_damage_at(x, y);
                        sc.player.hp = 10_000;
                        sc.view.player_px = x << 5;
                        sc.view.player_py = y << 5;
                        sc.apply_step_cell_effects();
                        let damaged = sc.player.hp < 10_000;
                        assert_eq!(predicted, damaged, "摆放 {li} 装备 {armor} 格 ({x},{y})：谓词 = 原块");
                    }
                }
            }
        }
    }
```

- [ ] **Step 2: 运行，确认失败**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib trap_predicate 2>&1 | grep -E "error|test result"`
Expected: 编译失败（`trap_damage_at` 未定义）。

- [ ] **Step 3: 实现**

`entity_at_cell` 方法之后新增：

```rust
    /// 【STAGE-B-008】站上 (x, y) 是否会被陷阱伤到（点地寻路途中避让用；纯判定、无副作用）。
    /// 与 `apply_step_cell_effects` 的陷阱块逐式同判（a.java:5555-5621）：装备 40 全免；61 左右或上下夹击（两侧都须在界内）；
    /// 62 / 60 四邻（右邻判界用 view.high——原式怪癖照搬）。等价性由 `trap_predicate_matches_step_effects_everywhere` 对拍钉住。
    fn trap_damage_at(&self, x: i32, y: i32) -> bool {
        if self.equipped_armor == 40 {
            return false;
        }
        let hit_lr = x > 0
            && self.entity_at_cell(x - 1, y, 61) >= 0
            && x < self.view.wide - 1
            && self.entity_at_cell(x + 1, y, 61) >= 0;
        let hit_ud = y > 0
            && self.entity_at_cell(x, y - 1, 61) >= 0
            && y < self.view.high - 1
            && self.entity_at_cell(x, y + 1, 61) >= 0;
        if hit_lr || hit_ud {
            return true;
        }
        let adjacent = |t: i32| {
            (x > 0 && self.entity_at_cell(x - 1, y, t) >= 0)
                || (x < self.view.high - 1 && self.entity_at_cell(x + 1, y, t) >= 0)
                || (y > 0 && self.entity_at_cell(x, y - 1, t) >= 0)
                || (y < self.view.high - 1 && self.entity_at_cell(x, y + 1, t) >= 0)
        };
        adjacent(62) || adjacent(60)
    }
```

注：62 的原块左邻无 `x > 0` 判断，但 `entity_at_cell(-1, …)` 恒 −1，与加判断等价。若对拍在某格不一致，以原块为准修谓词（不得改原块），并在 commit 正文写明差异与原因。

- [ ] **Step 4: 运行，确认通过 + 变异**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib trap_predicate`
Expected: 1 passed。变异（逐个、恢复）：去掉装备 40 判断 ⇒ 红；`adjacent(62) || adjacent(60)` 改为 `adjacent(62)` ⇒ 红；`hit_lr` 去掉 `x < self.view.wide - 1` ⇒ 若仍绿，在测试摆放里补一个贴右边的 61 对（如 `(61, 11, ROW), (61, 13, ROW)` 不在图内则改为右边界 `x = 12` 的情形），使其变红。

- [ ] **Step 5: 复核 + 提交**

附录 A 复核（T3），重点请复核者对 floor 49 / 51（12×13 非方形）真实数据补一轮对拍探针。提交：

```bash
git add crates/game-core/src/scene.rs
git commit -F - <<'EOF'
feat(core): 陷阱伤害谓词 trap_damage_at（点地避让用，与 classic 原块逐格对拍）（STAGE-B-008）

独立实现、无副作用的"站上是否扣血"判定；classic 陷阱块（apply_step_cell_effects）一行不动。
对拍：7 组合成摆放 × 13×13 全格 × 装备 {0, 40}，以原块实跑扣血为裁判。

动手前三问：
1. 层：Stage B 新增判定（不改 classic 块）。
2. 消费者：后续 tap_grid（Task 4）。
3. 证据：A 级——a.java:5555-5621 / scene.rs 原块；对拍测试 + 变异被杀。
Evidence: trap_predicate_matches_step_effects_everywhere。
Confidence: 90
EOF
```

---

### Task 4: 场景格分类 `tap_grid`

**Files:**
- Modify: `crates/game-core/src/scene.rs`（`trap_damage_at` 之后新增 `tap_grid`；测试追加到 `mod t2_walkto_route_tests`）

规则（spec §5.1）："存在且可见" = `visible ∧ state ≠ 1 ∧ ¬removed`；类目取 `self.entity_category`（含层覆写）。每格按槽表遍历：
- 型 9 ⇒ Alchemy；类目 8 ⇒ Monster(e)；类目 2 / 4 或型 6 ⇒ 不阻挡（地面）；其它可见实体 ⇒ Interact。优先级 Monster > Alchemy > Interact > 地面。
- 无阻挡可见实体时：地形可走（`self.walkable`）或有可见型 6 ⇒ Floor，否则 Wall。
- Floor 格 `danger = trap_damage_at(x, y)`。
- 不可见实体（隐形路径 6、隐形墙 12、剧情点 83、踏板 5 / 76 等）对分类不存在，只看地形。

- [ ] **Step 1: 写失败测试**

```rust
    /// 【STAGE-B-008 · spec §5.1】格分类：不可见实体只看地形（隐形路径 = 墙、隐形墙 / 剧情点 = 地面），
    /// 可见的门 / 商人 / 怪 / 炉按类别，道具格是地面，陷阱邻格标危险，已死 / 开门动画中的实体不挡。
    #[test]
    fn tap_grid_classifies_by_visible_entities_and_terrain() {
        use crate::tap_walk::TargetKind;
        // 走廊 y=ROW、x∈1..=11 可走；其余墙
        let mut sc = scene((1, ROW), &[(6, 2, ROW + 1), (12, 3, ROW), (83, 4, ROW), (1, 5, ROW), (41, 6, ROW), (9, 8, ROW), (13, 11, ROW), (62, 10, ROW - 1)]);
        let g = sc.tap_grid();
        let kind = |x: i32, y: i32| g.get(x, y).unwrap().kind;
        assert_eq!(kind(2, ROW + 1), TargetKind::Wall, "未显形隐形路径 = 墙（地形不可走）");
        assert_eq!(kind(3, ROW), TargetKind::Floor, "未显形隐形墙 = 地面（地形可走）");
        assert_eq!(kind(4, ROW), TargetKind::Floor, "剧情点（不可见）= 地面");
        assert_eq!(kind(5, ROW), TargetKind::Interact, "门 = 交互物");
        assert!(matches!(kind(6, ROW), TargetKind::Monster(_)), "怪");
        assert_eq!(kind(8, ROW), TargetKind::Alchemy, "炉（3 格宽：8..=10）");
        assert_eq!(kind(10, ROW), TargetKind::Alchemy, "炉第三格");
        assert_eq!(kind(11, ROW), TargetKind::Floor, "道具格 = 地面");
        assert_eq!(kind(0, ROW), TargetKind::Wall, "地形墙");
        assert!(g.get(10, ROW - 2).map_or(true, |c| !c.danger), "非邻格不危险");
        // 显形后的隐形路径 = 地面；死亡 / 移除的怪不挡
        let i6 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 6).unwrap();
        sc.entities.items[i6].visible = true;
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.entities.items[i41].state = 1;
        let g = sc.tap_grid();
        assert_eq!(g.get(2, ROW + 1).unwrap().kind, TargetKind::Floor, "已显形隐形路径 = 地面");
        assert_eq!(g.get(6, ROW).unwrap().kind, TargetKind::Floor, "死亡动画中的怪不挡");
    }

    /// 危险标记：62 上方紧邻的走廊格危险；装备 40 时不危险。
    #[test]
    fn tap_grid_marks_trap_danger() {
        let mut sc = scene((1, ROW), &[(62, 7, ROW - 1)]);
        assert!(sc.tap_grid().get(7, ROW).unwrap().danger, "62 正下方危险");
        assert!(!sc.tap_grid().get(5, ROW).unwrap().danger, "远处不危险");
        sc.equipped_armor = 40;
        assert!(!sc.tap_grid().get(7, ROW).unwrap().danger, "装备 40 全免");
    }
```

注：夹具 `scene()` 载入时按 m_122 把 5/81/6/12/76/82/83 置不可见（`scene.rs` load_floor 无存档分支）；若某型在夹具里不可见性与断言不符，先打印 `sc.entities.items[i].visible` 核对，再按实际载入语义调整断言措辞（不改分类规则）。

- [ ] **Step 2: 运行，确认失败**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib tap_grid 2>&1 | grep -E "error\[|test result"`
Expected: 编译失败（`tap_grid` 未定义）。

- [ ] **Step 3: 实现**

`trap_damage_at` 之后新增：

```rust
    /// 【STAGE-B-008 · spec §5.1】点地格分类（见 [`crate::tap_walk::TapGrid`]）。
    pub fn tap_grid(&self) -> crate::tap_walk::TapGrid {
        use crate::tap_walk::{TapCell, TapGrid, TargetKind};
        let (cols, rows) = (self.view.wide, self.view.high);
        let mut g = TapGrid::new(cols, rows);
        for y in 0..rows {
            for x in 0..cols {
                let ct = self.grid.cell_type[y as usize][x as usize] as usize;
                let cap = self.grid.capacity[ct] as usize;
                let mut blocker: Option<TargetKind> = None;
                let mut revealed_path = false;
                for s in 0..cap {
                    let e = self.grid.slots[ct][s] as usize - 1;
                    let ent = &self.entities.items[e];
                    if !ent.visible || ent.state == 1 || ent.removed {
                        continue; // 不可见 / 开门或死亡动画中 / 已移除：对寻路不存在
                    }
                    let t = ent.type_;
                    let cat = self.entity_category(t);
                    let k = if t == 9 {
                        Some(TargetKind::Alchemy)
                    } else if cat == 8 {
                        Some(TargetKind::Monster(e))
                    } else if cat == 2 || cat == 4 {
                        None
                    } else if t == 6 {
                        revealed_path = true;
                        None
                    } else {
                        Some(TargetKind::Interact)
                    };
                    let rank = |k: &Option<TargetKind>| match k {
                        Some(TargetKind::Monster(_)) => 3,
                        Some(TargetKind::Alchemy) => 2,
                        Some(TargetKind::Interact) => 1,
                        _ => 0,
                    };
                    if rank(&k) > rank(&blocker) {
                        blocker = k;
                    }
                }
                let kind = match blocker {
                    Some(k) => k,
                    None if self.walkable[y as usize][x as usize] || revealed_path => TargetKind::Floor,
                    None => TargetKind::Wall,
                };
                let danger = kind == TargetKind::Floor && self.trap_damage_at(x, y);
                g.set(x, y, TapCell { kind, danger });
            }
        }
        g
    }
```

- [ ] **Step 4: 运行，确认通过 + 变异**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib tap_grid` → 2 passed。
变异（逐个、恢复）：去掉 `!ent.visible` 判断 ⇒ "未显形隐形路径 = 墙"一条红；去掉 `ent.state == 1` 判断 ⇒ "死亡动画中的怪不挡"红；去掉 `danger` 计算（恒 false）⇒ `tap_grid_marks_trap_danger` 红。

- [ ] **Step 5: 复核 + 提交**

附录 A 复核（T4），重点：多格实体（型 69 宽 3 格，只在主格登记槽？核对 `entity::spawn` 的 width_cells 槽登记——每格都登记）、50 层型 72 层覆写、可见型 5 / 76 / 81 / 82 的归类。提交：

```bash
git add crates/game-core/src/scene.rs
git commit -F - <<'EOF'
feat(core): 场景点地格分类 tap_grid（STAGE-B-008 · spec §5.1）

"存在且可见" = visible ∧ state≠1 ∧ ¬removed；怪 > 炉 > 交互物 > 地面；道具 / 装备与已显形隐形路径是地面；
不可见实体只看地形（隐形路径 = 墙、隐形墙 / 剧情点 = 地面）；地面格按陷阱谓词标危险。尚未接入点地入口。

动手前三问：
1. 层：Stage B 新增（不触 classic）。
2. 消费者：后续 GameScene::tap_cell（Task 5）。
3. 证据：A 级——单测 + 变异被杀；类目表 entity::legacy_render_category_mirror（a.java:6268-6298）。
Evidence: tap_grid_classifies_by_visible_entities_and_terrain、tap_grid_marks_trap_danger。
Confidence: 85
EOF
```

---

### Task 5: 点地入口 `tap_cell`（走 / 停靠 / 走到邻格再撞，撞击在 Map wp0 执行）

**Files:**
- Modify: `crates/game-core/src/tap_walk.rs`（加 `TapResult`、`TAP_REJECT_TICKS`）
- Modify: `crates/game-core/src/scene.rs`：
  - `GameScene` 结构体字段（`battle_undo_snap` 附近，约 524 行）与 `load_floor` 里的结构体初始化（`battle_undo_snap: None,` 附近，约 1106 行）；
  - `walk_to`（约 1908 行）拆出 `begin_walk_route`；新增 `accepts_field_tap` / `tap_cell` / `start_tap_plan` / `execute_bump` / `flash_reject` / `clear_tap_state` / `tick_tap_state`；
  - `finish_walkto`（约 2003 行）置 `tap_followup_ready`；`tick`（约 4722 行）入口调 `tick_tap_state`；
  - `press_key`（约 4686 行）、`load_level_script`（约 1754 行）、`load_floor_data`（约 1125 行）清点地后续。
- Test: `mod t2_walkto_route_tests` 末尾追加

设计要点（spec §5.2 + 第二轮复核 R2-2 / R2-4）：
- 撞击**一律在 Map wp0 执行**：0 步撞击当场执行；≥1 步时路线只走到停靠格，记 `tap_bump = (停靠格, 方向)`，`finish_walkto` 回 Map 后的**下一拍开头**（`tick_tap_state`）执行；届时玩家不在停靠格（途中被隐形墙挡住等）、出现浮层 / 遮幅 / 新按键，就作废。
- 撞击 = 置 facing → 开路段（撞怪开战时 interact 据此记撤销快照）→ `try_step`；未开战且被挡 ⇒ 收段（零长段不入栈，Task 1）。走进去了（门开了、楼梯）⇒ 由 Map wp1 步末现有逻辑收段。
- 本任务里怪物也直接"走到邻格再撞"；"首点选中"在 Task 6 加。点地寻路中再点（Dialog dp3）本任务先按"停下"处理，Task 7 改为排队改道。

- [ ] **Step 1: 类型**

`crates/game-core/src/tap_walk.rs` 末尾（`#[cfg(test)]` 之前）加：

```rust
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
```

- [ ] **Step 2: 写失败测试**

`mod t2_walkto_route_tests` 末尾追加：

```rust
    fn settle(sc: &mut GameScene, from: i64, n: i64) {
        let wt = sc.width_table.clone();
        for fc in from..from + n {
            sc.tick(fc, &wt);
        }
    }

    /// 【STAGE-B-008 · spec §5.2】点可达地面：走过去（BFS），到达后回 Map、目标标记清除。
    #[test]
    fn tap_cell_walks_to_reachable_floor() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Walking);
        assert_eq!(sc.tap_target, Some((7, ROW)), "目标标记");
        settle(&mut sc, 0, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (7, ROW));
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap_target, None, "到达后标记清除");
    }

    /// 点远处的墙：走到墙边可达邻格，再朝墙撞一步（不移动、朝向 = 撞击方向、不留零长撤销笔）。
    #[test]
    fn tap_cell_far_wall_walks_adjacent_then_bumps() {
        let mut sc = scene((2, ROW), &[]);
        let _ = sc.tap_cell(8, ROW - 1);
        settle(&mut sc, 0, 160);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (8, ROW), "停在墙下方");
        assert_eq!(sc.facing, 1, "朝上撞过墙");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.undo.len(), 1, "只有一笔 (2,ROW)→(8,ROW) 的位移笔（撞墙零长段不入栈）");
    }

    /// 点门（有钥匙）：走到门旁、撞门开门并走进门格。
    #[test]
    fn tap_cell_door_opens_with_key() {
        let mut sc = scene((2, ROW), &[(1, 6, ROW)]);
        sc.player.yellow_keys = 1;
        let _ = sc.tap_cell(6, ROW);
        settle(&mut sc, 0, 200);
        assert_eq!(sc.player.yellow_keys, 0, "耗钥开门");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "走进门格");
    }

    /// 点正上方的炼丹炉：停在炉正下方朝上撞 ⇒ 打开炼丹炉页（Map → Alchemy，不经 Dialog——debug 构建不 panic）。
    #[test]
    fn tap_cell_alchemy_from_below_opens_page() {
        let mut sc = scene((2, ROW), &[(9, 5, ROW - 1)]);
        let _ = sc.tap_cell(6, ROW - 1);
        settle(&mut sc, 0, 160);
        assert_eq!(sc.game_mode, GameMode::Alchemy, "炉页打开");
        assert!(!sc.script_walk_armed && !sc.route.is_open(), "无寻路残留");
    }

    /// 到不了的地面：停在最近可达格，目标格闪红。
    #[test]
    fn tap_cell_unreachable_floor_docks_and_flashes() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        sc.walkable[ROW as usize][7] = false; // 走廊中间砌墙
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Docking);
        assert!(matches!(sc.tap_reject, Some(((9, r), _)) if r == ROW), "目标格闪红");
        settle(&mut sc, 0, 120);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (6, ROW), "停在墙前最近格");
    }

    /// 【spec §0 隐藏通路泄露】只能经隐形路径到达的地面：不得走过去、隐形路径不显形（停靠 + 闪红）。
    #[test]
    fn tap_cell_does_not_leak_hidden_path() {
        let mut sc = scene((2, ROW), &[(6, 6, ROW)]);
        sc.walkable[ROW as usize][6] = false; // 隐形路径格的地形 = 墙（真实关卡同形）
        let _ = sc.tap_cell(9, ROW);
        settle(&mut sc, 0, 160);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "停在隐形路径前");
        let i6 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 6).unwrap();
        assert!(!sc.entities.items[i6].visible, "隐形路径未显形");
    }

    /// 途中撞上隐形墙（看起来是地面）：被挡停下，撞击后续作废（玩家不在停靠格）。
    #[test]
    fn tap_cell_bump_cancelled_when_route_stops_early() {
        let mut sc = scene((2, ROW), &[(12, 5, ROW)]);
        let _ = sc.tap_cell(9, ROW - 1); // 墙：停靠 (9,ROW)，路线途经 (5,ROW) 隐形墙
        settle(&mut sc, 0, 200);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (4, ROW), "撞隐形墙停下");
        assert_ne!(sc.facing, 1, "朝上的撞击已作废");
        assert_eq!(sc.game_mode, GameMode::Map);
    }

    /// 观察模式点按无效；手动行走中点按闪红、状态不变。
    #[test]
    fn tap_cell_ignored_in_observe_and_rejected_while_walking() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[]);
        sc.walk_phase = 2;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Ignored);
        sc.walk_phase = 1;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected);
        assert!(sc.tap_reject.is_some() && !sc.route.is_open());
        sc.walk_phase = 5;
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected, "战斗中闪红");
        assert_eq!(sc.walk_phase, 5, "战斗状态不变");
        sc.walk_phase = 0;
        sc.script_walk_armed = true; // Map 语境脚本行走的步间
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Rejected, "脚本行走中不得插入点地路线");
        assert!(!sc.route.is_open());
    }
```

- [ ] **Step 3: 运行，确认失败**

Run: `sleep 2; touch crates/game-core/src/scene.rs crates/game-core/src/tap_walk.rs; cargo test --offline -p game-core --lib tap_cell 2>&1 | grep -E "error\[|test result"`
Expected: 编译失败（`tap_cell` 等未定义）。

- [ ] **Step 4: 实现——字段**

`GameScene` 结构体（`battle_undo_snap` 字段之后）加：

```rust
    /// 【STAGE-B-008】自动战斗（触屏模式设置项，壳写入；默认 false = 点怪先选中、再点开战）
    pub auto_battle: bool,
    /// 【STAGE-B-008】点地目标标记（走向 / 撞向的格；路线与撞击结束即清）
    pub tap_target: Option<(i32, i32)>,
    /// 【STAGE-B-008】闪红（格, 到期拍）
    pub tap_reject: Option<((i32, i32), i64)>,
    /// 【STAGE-B-008】路线走完后的撞击（停靠格, 方向）——在 Map wp0 执行
    tap_bump: Option<((i32, i32), u8)>,
    /// 【STAGE-B-008】点地寻路收尾后下一拍执行后续（撞击 / 排队）
    tap_followup_ready: bool,
```

`load_floor` 的结构体初始化（`battle_undo_snap: None,` 之后）加：

```rust
            auto_battle: false,
            tap_target: None,
            tap_reject: None,
            tap_bump: None,
            tap_followup_ready: false,
```

- [ ] **Step 5: 实现——拆出 `begin_walk_route`**

把 `walk_to` 中 `self.route.begin((cx, cy));` 起到 `self.lock_camera_on(0);` 止的代码移到新方法，`walk_to` 改调它：

```rust
    pub fn walk_to(&mut self, tx: i32, ty: i32) -> crate::pathfind::RoutePlan {
        if !self.field_tap_allowed() {
            let at = (self.player_cell_x, self.player_cell_y);
            return crate::pathfind::RoutePlan { steps: Vec::new(), dock: at, reached: false, probe: None };
        }
        self.interrupt_route();
        self.walkto_draining = false; // 新路线 = 段重新开放（改道）
        let (cx, cy) = (self.player_cell_x, self.player_cell_y);
        let plan = {
            let walk = crate::pathfind::Walkability {
                terrain: self.walkable.clone(),
                has_entity_slot: &|x, y| {
                    let ct = self.grid.cell_type[y as usize][x as usize] as usize;
                    self.grid.capacity[ct] > 0
                },
                cols: self.view.wide,
                rows: self.view.high,
            };
            crate::pathfind::plan_route(&walk, (cx, cy), (tx, ty))
        };
        self.begin_walk_route(plan.steps.clone());
        plan
    }

    /// 【T2 / STAGE-B-008】按给定步序（目标端优先）开点地寻路：开路段、暂存教程箭头（BUG-043）、喂 dp3 消费机（Dialog 语境）。
    fn begin_walk_route(&mut self, steps: Vec<u8>) {
        let (cx, cy) = (self.player_cell_x, self.player_cell_y);
        self.route.begin((cx, cy));
        // 【BUG-043】箭头可见 ⇒ 先把箭头步序移出 walkPathBuffer（改道时已暂存，保留原暂存）
        if self.path_arrows_visible && self.walkto_arrows.is_none() {
            let n = (self.walk_step_count.max(0) as usize).min(self.walk_path_buffer.len());
            self.walkto_arrows = Some(self.walk_path_buffer[..n].to_vec());
        }
        self.walk_step_count = steps.len() as i32;
        self.walk_path_buffer = steps;
        self.script_walk_armed = true;
        self.dialog_phase = 3;
        self.walk_phase = 0;
        // dp3 消费机器在 Dialog 语境（op_mov 同款——脚本行走=mode 11 语境）
        self.game_mode = GameMode::Dialog;
        self.lock_camera_on(0);
    }
```

- [ ] **Step 6: 实现——入口与后续**

`begin_walk_route` 之后新增：

```rust
    /// 【STAGE-B-008】场上是否接收点地（壳据此决定点按走 [`Self::tap_cell`] 还是行命中 / 点空白）：
    /// 无浮层、非遮幅，且为地图态或点地寻路自身驱动的 dp3。手动行走 / 战斗 / 观察模式也接收（由 tap_cell 闪红或忽略）。
    pub fn accepts_field_tap(&self) -> bool {
        if self.overlay_active || self.transitioning {
            return false;
        }
        match self.game_mode {
            GameMode::Map => true,
            GameMode::Dialog => self.dialog_phase == 3 && self.walkto_owns_dp3(),
            _ => false,
        }
    }

    /// 【STAGE-B-008 · spec §5.2】点中格 (cx, cy)。
    pub fn tap_cell(&mut self, cx: i32, cy: i32) -> crate::tap_walk::TapResult {
        use crate::tap_walk::TapResult;
        if !self.accepts_field_tap() {
            return TapResult::Ignored;
        }
        let at = (self.player_cell_x, self.player_cell_y);
        if self.game_mode == GameMode::Dialog {
            // 点地寻路中（Task 7 改为排队改道）：先停下
            self.tap_bump = None;
            self.interrupt_route();
            return TapResult::Stopped;
        }
        match self.walk_phase {
            0 if !self.script_walk_armed => {}
            2 => return TapResult::Ignored, // 观察模式：点按无效（拖动平移由壳接，计划 3）
            _ => {
                // 手动行走 / 战斗 / Map 语境的脚本行走（wp0 ∧ 武装，field_tap_allowed 同口径拒绝）：闪红、状态不变
                self.flash_reject((cx, cy));
                return TapResult::Rejected;
            }
        }
        if (cx, cy) == at {
            return TapResult::Ignored;
        }
        let grid = self.tap_grid();
        let plan = crate::tap_walk::plan_tap(&grid, at, (cx, cy));
        self.start_tap_plan(plan, (cx, cy))
    }

    fn start_tap_plan(&mut self, plan: crate::tap_walk::TapPlan, target: (i32, i32)) -> crate::tap_walk::TapResult {
        use crate::tap_walk::{TapPlan, TapResult};
        match plan {
            TapPlan::Walk { steps, .. } => {
                self.walkto_draining = false;
                self.begin_walk_route(steps);
                self.tap_target = Some(target);
                TapResult::Walking
            }
            TapPlan::Bump { steps, dock, dir } => {
                self.tap_target = Some(target);
                if steps.is_empty() {
                    self.execute_bump(dir);
                } else {
                    self.walkto_draining = false;
                    self.tap_bump = Some((dock, dir));
                    self.begin_walk_route(steps);
                }
                TapResult::Bumping
            }
            TapPlan::Dock { steps, .. } => {
                self.flash_reject(target);
                if !steps.is_empty() {
                    self.walkto_draining = false;
                    self.begin_walk_route(steps);
                }
                TapResult::Docking
            }
            TapPlan::Ignore => TapResult::Ignored,
        }
    }

    /// 【STAGE-B-008 · R2-2】Map wp0 撞一步：置朝向 → 开路段（撞怪开战时记撤销快照）→ try_step。
    /// 未开战且被挡 ⇒ 收段（零长段不入栈）；走进去了 ⇒ Map wp1 步末收段。
    fn execute_bump(&mut self, dir: u8) {
        let at = (self.player_cell_x, self.player_cell_y);
        self.facing = dir as i32;
        self.route.begin(at);
        let ok = self.try_step(dir as i32);
        if self.walk_phase == 5 || ok {
            return;
        }
        self.route_arrived();
    }

    fn flash_reject(&mut self, cell: (i32, i32)) {
        self.tap_reject = Some((cell, self.frame_counter + crate::tap_walk::TAP_REJECT_TICKS));
    }

    /// 【STAGE-B-008】清点地状态（撤销 / 换层 / 剧情 / 按键时）。
    pub fn clear_tap_state(&mut self) {
        self.tap_target = None;
        self.tap_reject = None;
        self.tap_bump = None;
        self.tap_followup_ready = false;
    }

    /// 【STAGE-B-008】每拍开头：闪红到期；点地寻路收尾后的第一拍执行撞击（回到 Map wp0、无浮层 / 遮幅 / 新按键，
    /// 且玩家正在停靠格），否则作废；路线与撞击都结束即清目标标记。classic 下各字段恒空，零操作。
    fn tick_tap_state(&mut self) {
        if let Some((_, until)) = self.tap_reject {
            if self.frame_counter >= until {
                self.tap_reject = None;
            }
        }
        if self.tap_followup_ready {
            self.tap_followup_ready = false;
            let bump = self.tap_bump.take();
            let ready = self.game_mode == GameMode::Map
                && self.walk_phase == 0
                && !self.overlay_active
                && !self.transitioning
                && self.key_value == 0
                && self.key_held == 0;
            if ready {
                if let Some((dock, dir)) = bump {
                    if (self.player_cell_x, self.player_cell_y) == dock {
                        self.execute_bump(dir);
                    }
                }
            }
        }
        if self.tap_target.is_some()
            && !(self.route.is_open() || self.walkto_draining || self.tap_bump.is_some() || self.tap_followup_ready)
        {
            self.tap_target = None;
        }
    }
```

`finish_walkto` 末尾加：

```rust
        // 【STAGE-B-008】下一拍开头执行点地后续（撞击 / 排队）
        self.tap_followup_ready = true;
```

`tick` 开头 `self.frame_counter = frame_counter;` 之后加：

```rust
        self.tick_tap_state();
```

`press_key` 函数体第一行加（任意按键取消点地后续，BUG-040"手动优先"同口径）：

```rust
        self.tap_bump = None;
```

`load_level_script` 中 `self.undo.clear(...ScriptEvent)` 之后、`load_floor_data` 中 `self.undo.clear(...FloorChange)` 之后各加：

```rust
        self.clear_tap_state();
```

- [ ] **Step 7: 运行，确认通过**

Run: `sleep 2; touch crates/game-core/src/scene.rs crates/game-core/src/tap_walk.rs; cargo test --offline -p game-core --lib tap_cell`
Expected: 8 passed。若 `tap_cell_far_wall_walks_adjacent_then_bumps` 的 `undo.len()` 不为 1：打印撤销栈逐笔核对（应为 Walk 段一笔；撞墙段零长不入栈），按实际原因修实现而非改期望。

变异（逐个、恢复）：
- `tick_tap_state` 去掉"玩家在停靠格"判断 ⇒ `tap_cell_bump_cancelled_when_route_stops_early` 红；
- `start_tap_plan` 的 Bump 臂把撞击改为路线末步（`steps.insert(0, dir)` 并不设 tap_bump）⇒ `tap_cell_alchemy_from_below_opens_page` 红或 debug panic；
- `tap_grid` 的不可见判断去掉 ⇒ `tap_cell_does_not_leak_hidden_path` 红。

- [ ] **Step 8: 全量 + classic 门禁**

全量测试 0 失败；`python3 gates/cli.py trace` PASS（classic 按键链不经 tap_cell；`tick_tap_state` / `press_key` 新增行在 classic 下作用于空状态）。

- [ ] **Step 9: 复核 + 提交**

附录 A 复核（T5），重点：撞怪（0 步与 ≥1 步）的撤销快照与 BUG-045 / 048 胜利尾三分支的交互、撞楼梯换层、撞商人 kind 4、`press_key` 新增行对经典按键链的影响（应为零）。提交：

```bash
git add crates/game-core/src/tap_walk.rs crates/game-core/src/scene.rs
git commit -F - <<'EOF'
feat(core): 点地入口 tap_cell——安全通行 BFS、到不了停靠闪红、走到邻格再在 Map wp0 撞（STAGE-B-008）

tap_cell：可达地面直达；墙 / 门 / 炉 / 怪走到停靠格，路线收尾后下一拍在 Map wp0 撞一步（玩家须在停靠格，否则作废）；
到不了停最近可达格并闪红；观察模式忽略、手动行走 / 战斗闪红。walk_to 拆出 begin_walk_route（行为不变）。
修掉 BUG-044 余项①②（撞炉不再经 Dialog→Alchemy、途中进炉页被吞）——新入口不把撞击放进 dp3 路线。

动手前三问：
1. 层：Stage B 点地策略（game-core 新入口；classic 按键链与 oracle 回放不经过）。
2. 消费者：后续壳层（Task 11 / 12）；本任务仅宿主测试调用。
3. 证据：A 级——8 条场景测试 + 3 个变异被杀；全量测试 0 失败；gates trace PASS。
Evidence: tap_cell_* 测试（含隐藏通路不泄露、炉自下方打开且 debug 不 panic）。
Confidence: 85
EOF
```

---
### Task 6: 怪物首点选中、再点开战、自动战斗、每拍校验

**Files:**
- Modify: `crates/game-core/src/tap_walk.rs`（加 `SelectedEnemy`）
- Modify: `crates/game-core/src/scene.rs`：字段 `tap_selected`；`tap_cell` 的 Map 分支；新增 `selection_valid` / `engage_selected`；`tick_tap_state`、`clear_tap_state`、`press_key`
- Test: `mod t2_walkto_route_tests` 末尾追加

规则（spec §5.2 Q8 + §3.2）：
- 点中 `Monster(e)`：`auto_battle` 关且当前选中的不是 e ⇒ 选中 e（`Selected`，不动、不开路线、不置目标框）；否则（再点同一只的任一格，或自动战斗开）⇒ 清选中并按 `plan_tap` 撞上。
- 点别处 / 点自己 ⇒ 清选中再执行该点按。
- 每拍校验（`tick_tap_state` 开头）：Map 态、walk_phase 0、无浮层、非遮幅、楼层号不变、实体未移除、`state ≠ 1`、可见、类型不变、主格不变。任一不满足即清。"操控模式未变"由计划 3 切换时调 `clear_tap_state`。
- 任何按键（`press_key`）、`clear_tap_state`（撤销 / 换层 / 剧情）都清选中。

- [ ] **Step 1: 类型**

`crates/game-core/src/tap_walk.rs` 的 `TapResult` 之前加：

```rust
/// 已选中的怪物（spec §5.2：首点选中、再点开战）。`cell` = 实体主格（`px >> 5, py >> 5`），`floor` = 选中时楼层号；
/// 每拍按这些字段校验，任一变化即清选中。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedEnemy {
    pub entity: usize,
    pub type_: i32,
    pub cell: (i32, i32),
    pub floor: i32,
}
```

- [ ] **Step 2: 写失败测试**

```rust
    fn strong(sc: &mut GameScene) {
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
    }

    /// 【STAGE-B-008 · spec §5.2 Q8】点怪：首点只选中（不动、不开路线）；再点同一只 = 走到邻格撞上开战（战斗笔可撤销）。
    #[test]
    fn t6_first_tap_selects_second_tap_fights() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        strong(&mut sc);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        assert_eq!(sc.tap_selected.map(|s| s.entity), Some(e));
        assert_eq!(sc.tap_target, None, "选中不画目标框（有自己的闪烁框）");
        settle(&mut sc, 0, 20);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "选中不移动");
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert!(sc.tap_selected.is_some(), "空闲时选中保持");
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Bumping);
        assert!(sc.tap_selected.is_none(), "开战即清选中");
        settle(&mut sc, 20, 400);
        let ent = &sc.entities.items[e];
        assert!(ent.removed || ent.state == 1, "怪被击杀");
        assert!(journal_has_combat(&mut sc), "战斗笔入栈（误触靠撤销）");
    }

    /// 自动战斗开：一点即战。
    #[test]
    fn t6_auto_battle_fights_on_first_tap() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
        strong(&mut sc);
        sc.auto_battle = true;
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Bumping);
        assert!(sc.tap_selected.is_none());
    }

    /// 点另一只 = 改选；点别处 = 清选中并执行。
    #[test]
    fn t6_tap_elsewhere_or_other_monster_replaces_selection() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(41, 6, ROW), (42, 9, ROW)]);
        assert_eq!(sc.tap_cell(6, ROW), TapResult::Selected);
        assert_eq!(sc.tap_cell(9, ROW), TapResult::Selected, "点另一只 = 改选");
        assert_eq!(sc.tap_selected.map(|s| s.type_), Some(42));
        assert_eq!(sc.tap_cell(4, ROW), TapResult::Walking, "点别处 = 清选中并执行");
        assert!(sc.tap_selected.is_none());
    }

    /// 多格怪（型 69 宽 3 格）：点任一格选中同一只，再点另一格 = 开战（停靠在整只怪的外侧邻格）。
    #[test]
    fn t6_multi_cell_monster_any_cell_is_same_selection() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((2, ROW), &[(69, 6, ROW)]);
        strong(&mut sc);
        assert_eq!(sc.tap_cell(8, ROW), TapResult::Selected, "点 69 的第三格");
        assert_eq!(sc.tap_cell(7, ROW), TapResult::Bumping, "再点同一只的中间格 = 开战");
    }

    /// 选中有效性每拍校验：移除 / 死亡动画 / 被搬走 / 变型 / 换层 / 浮层 / 按键 / 清状态，任一即清。
    #[test]
    fn t6_selection_invalidated_every_tick() {
        let mk = || {
            let mut sc = scene((2, ROW), &[(41, 6, ROW)]);
            let _ = sc.tap_cell(6, ROW);
            assert!(sc.tap_selected.is_some(), "前置：已选中");
            sc
        };
        let cases: Vec<(&str, Box<dyn Fn(&mut GameScene)>)> = vec![
            ("实体移除（他处击杀）", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].removed = true; })),
            ("死亡动画 state = 1", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].state = 1; })),
            ("被搬走（MOV）", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].px += 32; })),
            ("变型（道具改写）", Box::new(|sc| { let e = sc.tap_selected.unwrap().entity; sc.entities.items[e].type_ = 42; })),
            ("楼层号变化", Box::new(|sc| sc.current_floor += 1)),
            ("浮层", Box::new(|sc| sc.overlay_active = true)),
        ];
        for (what, f) in cases {
            let mut sc = mk();
            f(&mut sc);
            settle(&mut sc, 0, 1);
            assert!(sc.tap_selected.is_none(), "{what} ⇒ 清选中");
        }
        let mut sc = mk();
        sc.press_key(-2);
        assert!(sc.tap_selected.is_none(), "任何按键 ⇒ 清选中");
        let mut sc = mk();
        sc.clear_tap_state();
        assert!(sc.tap_selected.is_none(), "撤销 / 换层 / 剧情 ⇒ 清选中");
        let mut sc = mk();
        assert_eq!(sc.engage_selected(), crate::tap_walk::TapResult::Bumping, "「开战」= 撞已选中的怪");
    }
```

- [ ] **Step 3: 运行，确认失败**

Run: `sleep 2; touch crates/game-core/src/scene.rs crates/game-core/src/tap_walk.rs; cargo test --offline -p game-core --lib t6_ 2>&1 | grep -E "error\[|test result"`
Expected: 编译失败（`tap_selected` / `engage_selected` 未定义）。

- [ ] **Step 4: 实现**

字段（`tap_followup_ready` 之后）与 `load_floor` 初始化：

```rust
    /// 【STAGE-B-008】已选中的怪物（首点选中、再点开战；每拍校验）
    pub tap_selected: Option<crate::tap_walk::SelectedEnemy>,
```
```rust
            tap_selected: None,
```

`tap_cell` 中 Task 5 的这两段：

```rust
        if (cx, cy) == at {
            return TapResult::Ignored;
        }
        let grid = self.tap_grid();
        let plan = crate::tap_walk::plan_tap(&grid, at, (cx, cy));
        self.start_tap_plan(plan, (cx, cy))
```
替换为：

```rust
        self.tap_bump = None;
        let grid = self.tap_grid();
        if let Some(crate::tap_walk::TapCell { kind: crate::tap_walk::TargetKind::Monster(e), .. }) = grid.get(cx, cy) {
            let again = self.tap_selected.map_or(false, |s| s.entity == e);
            if !self.auto_battle && !again {
                let ent = &self.entities.items[e];
                self.tap_selected = Some(crate::tap_walk::SelectedEnemy {
                    entity: e,
                    type_: ent.type_,
                    cell: (ent.px >> 5, ent.py >> 5),
                    floor: self.current_floor,
                });
                self.tap_target = None;
                return TapResult::Selected;
            }
        }
        // 点别处 / 点自己 / 再点同一只 / 自动战斗：清选中，再执行这次点按
        self.tap_selected = None;
        if (cx, cy) == at {
            return TapResult::Ignored;
        }
        let plan = crate::tap_walk::plan_tap(&grid, at, (cx, cy));
        self.start_tap_plan(plan, (cx, cy))
```

`clear_tap_state` 之后新增：

```rust
    /// 【STAGE-B-008 · spec §5.2】选中是否仍有效（每拍校验的判据）。
    fn selection_valid(&self, s: crate::tap_walk::SelectedEnemy) -> bool {
        if self.game_mode != GameMode::Map
            || self.walk_phase != 0
            || self.overlay_active
            || self.transitioning
            || self.current_floor != s.floor
            || s.entity >= self.entities.count
        {
            return false;
        }
        let ent = &self.entities.items[s.entity];
        !ent.removed && ent.state != 1 && ent.visible && ent.type_ == s.type_ && (ent.px >> 5, ent.py >> 5) == s.cell
    }

    /// 【STAGE-B-008 · spec §3.2「开战」】撞已选中的怪（工具栏按钮由计划 3 接入）。无有效选中 = Ignored。
    pub fn engage_selected(&mut self) -> crate::tap_walk::TapResult {
        match self.tap_selected {
            Some(s) if self.selection_valid(s) => self.tap_cell(s.cell.0, s.cell.1),
            _ => {
                self.tap_selected = None;
                crate::tap_walk::TapResult::Ignored
            }
        }
    }
```

`tick_tap_state` 开头（闪红到期判断之前）加：

```rust
        if let Some(s) = self.tap_selected {
            if !self.selection_valid(s) {
                self.tap_selected = None;
            }
        }
```

`clear_tap_state` 加 `self.tap_selected = None;`；`press_key` 第一行（Task 5 加的 `self.tap_bump = None;`）之后加 `self.tap_selected = None;`。

- [ ] **Step 5: 运行，确认通过 + 变异**

Run: `sleep 2; touch crates/game-core/src/scene.rs crates/game-core/src/tap_walk.rs; cargo test --offline -p game-core --lib t6_` → 5 passed；`--lib tap_cell` 仍 8 passed（T5 测试里的墙 / 门 / 炉不是怪，行为不变）。

变异（逐个、恢复）：
- 去掉 `!self.auto_battle &&` ⇒ `t6_auto_battle_fights_on_first_tap` 红；
- `again` 恒 false ⇒ `t6_first_tap_selects_second_tap_fights` 红（第二点仍是 Selected）；
- `selection_valid` 去掉主格比较 ⇒ "被搬走"一条红；去掉楼层比较 ⇒ "楼层号变化"一条红。

- [ ] **Step 6: 复核 + 提交**

附录 A 复核（T6），重点：选中期间的 BUG-046 不可胜提示（撞上后弹 kind 0，选中已清）、陷阱怪 60/61/62 的选中与撞击、50 层型 72 层覆写为怪、选中后用方向键走开再点同一只（应重新选中，不是开战——按键已清选中）；spec §9 列的其余生命周期事件（读档、道具 18/20/21/86、hide_notify 切后台）逐个确认会经 `load_floor_data` / `load_level_script` / 每拍校验清掉，没覆盖的报 Major。提交：

```bash
git add crates/game-core/src/tap_walk.rs crates/game-core/src/scene.rs
git commit -F - <<'EOF'
feat(core): 点怪首点选中、再点开战、自动战斗、选中每拍校验（STAGE-B-008 · spec §5.2 Q8）

tap_cell 点中怪物：自动战斗关 ⇒ 先选中（不动），再点同一只任一格 = 走到邻格撞上；自动战斗开 ⇒ 一点即战。
选中每拍校验（Map wp0、无浮层 / 遮幅、楼层、实体存活可见、类型、主格），按键 / 清状态即清。engage_selected 供计划 3 工具栏「开战」。

动手前三问：
1. 层：Stage B 点地策略（game-core；classic 按键链不经过）。
2. 消费者：壳层 tap_cell（Task 11 / 12）；auto_battle 由计划 3 设置行写入（本计划恒 false）。
3. 证据：A 级——5 条场景测试 + 4 个变异被杀；全量测试 0 失败。
Evidence: t6_* 测试。
Confidence: 85
EOF
```

---

### Task 7: 点地寻路中的点按 / 滑动排队（改道、点自己停下、`swipe_step`）

**Files:**
- Modify: `crates/game-core/src/tap_walk.rs`（加 `PendingTap`）
- Modify: `crates/game-core/src/scene.rs`：字段 `tap_pending`；`tap_cell` 的 Dialog 分支；新增 `swipe_step`；`tick_tap_state`、`clear_tap_state`、`press_key`
- Test: `mod t2_walkto_route_tests` 末尾追加

规则（spec §5.2"排队"，滑动部分见 §5.3）：
- 点地寻路中（Dialog dp3 归寻路）点别处 ⇒ `interrupt_route`（BUG-040 排空：走完当前步由 `finish_walkto` 回 Map wp0），记下新目标（只留最后一次），目标框立即换到新目标 ⇒ `Queued`；点自己 ⇒ 停下（`Stopped`，清排队与目标框）。
- 排队动作在 `finish_walkto` 后下一拍开头执行（与撞击同一时点、同一就绪条件：Map wp0、无浮层 / 遮幅、无新按键）；不就绪即丢弃。
- 排队与撞击后续只在"寻路仍占 dp3"或"收尾待执行"期间有效；其余时刻发现残留（开战挂起、剧情接管等没走到 `finish_walkto` 的路径）即丢弃。
- `swipe_step(dir)`：地图空闲 ⇒ `probe_step`（BUG-047 已先置朝向）；寻路中 ⇒ 排队；观察模式忽略；手动行走 / 战斗 / 脚本行走 ⇒ `Rejected`（无格可闪红，不画）。
- 任何按键、`clear_tap_state`、换层、剧情载入都清排队。

- [ ] **Step 1: 类型**

`crates/game-core/src/tap_walk.rs` 的 `SelectedEnemy` 之后加：

```rust
/// 点地寻路中到达的输入，排队到排空回 Map wp0 后执行（spec §5.2；只保留最后一次）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingTap {
    /// 新目标格（执行 = `tap_cell`）
    Cell(i32, i32),
    /// 滑动方向（执行 = `swipe_step`）
    Swipe(u8),
}
```

- [ ] **Step 2: 写失败测试**

```rust
    /// 自起步跑到玩家格 x ≥ `x` 且仍在点地寻路中（Dialog），返回下一拍帧号。
    fn walk_until_x(sc: &mut GameScene, x: i32) -> i64 {
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while sc.player_cell_x < x {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 200, "前置：走到 x={x}");
        }
        assert_eq!(sc.game_mode, GameMode::Dialog, "前置：仍在寻路中");
        fc
    }

    /// 【STAGE-B-008 · spec §5.2 排队】寻路中点别处：走完当前格后改道去新目标，标记立即换到新目标。
    #[test]
    fn t7_tap_while_walking_queues_reroute() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        assert_eq!(sc.tap_target, Some((2, ROW)), "标记立即换到新目标");
        settle(&mut sc, fc, 300);
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (2, ROW), "改道到新目标");
        assert_eq!(sc.game_mode, GameMode::Map);
        assert_eq!(sc.tap_target, None);
    }

    /// 寻路中点自己：停下（走完当前格），无排队、无标记。
    #[test]
    fn t7_tap_self_while_walking_stops() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        let me = (sc.player_cell_x, sc.player_cell_y);
        assert_eq!(sc.tap_cell(me.0, me.1), TapResult::Stopped);
        settle(&mut sc, fc, 200);
        assert!((4..=5).contains(&sc.player_cell_x), "走完当前格即停（实得 {}）", sc.player_cell_x);
        assert!(!sc.route.is_open() && sc.game_mode == GameMode::Map);
        assert_eq!(sc.tap_target, None);
    }

    /// 排队失效：排空期间按键 ⇒ 丢弃；排空落地那一拍有浮层 ⇒ 丢弃。
    #[test]
    fn t7_queued_tap_discarded_on_key_or_blocked_landing() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        sc.press_key(-2); // 朝下（墙）
        sc.release_key();
        settle(&mut sc, fc, 200);
        assert!(sc.player_cell_x >= 4, "按键作废排队：不改道回 x=2（实得 {}）", sc.player_cell_x);

        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let mut fc = walk_until_x(&mut sc, 4);
        assert_eq!(sc.tap_cell(2, ROW), TapResult::Queued);
        let wt = sc.width_table.clone();
        while sc.game_mode != GameMode::Map {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300);
        }
        sc.overlay_active = true; // 收尾后、执行前出现浮层（如拾取提示）
        sc.overlay_text = Some("提示".into());
        sc.tick(fc, &wt);
        sc.overlay_active = false;
        settle(&mut sc, fc + 1, 200);
        assert!(sc.player_cell_x >= 4, "浮层作废排队");
        assert!(!sc.route.is_open() && sc.tap_target.is_none());
    }

    /// 滑动：空闲 = 一格（朝墙 = 撞停并转向）；寻路中 = 排队，排空回 Map 后朝该方向踏一步。
    #[test]
    fn t7_swipe_steps_when_idle_and_queues_while_walking() {
        use crate::tap_walk::TapResult;
        let mut sc = scene((3, ROW), &[]);
        assert_eq!(sc.swipe_step(2), TapResult::Walking);
        settle(&mut sc, 0, 40);
        assert_eq!(sc.player_cell_x, 4, "空闲滑动 = 一格");
        assert_eq!(sc.swipe_step(1), TapResult::Bumping, "朝墙滑 = 撞停");
        assert_eq!(sc.facing, 1, "撞墙后朝向 = 滑动方向（BUG-047）");

        let _ = sc.tap_cell(10, ROW);
        let wt = sc.width_table.clone();
        let mut fc = 40;
        while sc.player_cell_x < 6 {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 300);
        }
        assert_eq!(sc.swipe_step(3), TapResult::Queued);
        while sc.game_mode != GameMode::Map {
            sc.tick(fc, &wt);
            fc += 1;
            assert!(fc < 400);
        }
        let landed = sc.player_cell_x;
        settle(&mut sc, fc, 60);
        assert_eq!(sc.player_cell_x, landed - 1, "排空落地后朝左踏一步");
    }
```

- [ ] **Step 3: 运行，确认失败**

Run: `sleep 2; touch crates/game-core/src/scene.rs crates/game-core/src/tap_walk.rs; cargo test --offline -p game-core --lib t7_ 2>&1 | grep -E "error\[|test result"`
Expected: 编译失败（`swipe_step` 未定义）；先只加 `swipe_step` 空壳（恒 `Ignored`）再跑，`t7_tap_while_walking_queues_reroute` 等 FAIL（T5 的 Dialog 分支返回 `Stopped`）。

- [ ] **Step 4: 实现**

字段（`tap_selected` 之后）与初始化：

```rust
    /// 【STAGE-B-008】点地寻路中排队的输入（排空回 Map wp0 后下一拍执行）
    tap_pending: Option<crate::tap_walk::PendingTap>,
```
```rust
            tap_pending: None,
```

`tap_cell` 中 Task 5 的 Dialog 分支整体替换为：

```rust
        if self.game_mode == GameMode::Dialog {
            // 【spec §5.2 排队】点地寻路中：打断（走完当前步回 Map wp0），记下新目标；点自己 = 停下
            self.tap_bump = None;
            self.tap_selected = None;
            self.interrupt_route();
            if (cx, cy) == at {
                self.tap_pending = None;
                self.tap_target = None;
                return TapResult::Stopped;
            }
            self.tap_pending = Some(crate::tap_walk::PendingTap::Cell(cx, cy));
            self.tap_target = Some((cx, cy));
            return TapResult::Queued;
        }
```

`tap_cell` Map 分支中 Task 6 的 `self.tap_bump = None;` 之后加 `self.tap_pending = None;`。

`engage_selected` 之后新增：

```rust
    /// 【STAGE-B-008 · spec §5.3】滑动一格：地图空闲 = 朝 `dir` 探步（`probe_step`，先置朝向）；点地寻路中 = 排队；
    /// 观察模式忽略；手动行走 / 战斗 / 脚本行走 = Rejected（无目标格，不闪红）。
    pub fn swipe_step(&mut self, dir: u8) -> crate::tap_walk::TapResult {
        use crate::tap_walk::{PendingTap, TapResult};
        if !self.accepts_field_tap() || dir > 3 {
            return TapResult::Ignored;
        }
        if self.game_mode == GameMode::Dialog {
            self.tap_bump = None;
            self.tap_selected = None;
            self.interrupt_route();
            self.tap_pending = Some(PendingTap::Swipe(dir));
            self.tap_target = None;
            return TapResult::Queued;
        }
        match self.walk_phase {
            0 if !self.script_walk_armed => {}
            2 => return TapResult::Ignored,
            _ => return TapResult::Rejected,
        }
        self.tap_bump = None;
        self.tap_pending = None;
        self.tap_selected = None;
        self.tap_target = None;
        if self.probe_step(dir) {
            TapResult::Walking
        } else {
            TapResult::Bumping
        }
    }
```

`tick_tap_state` 的后续块（Task 5 的 `if self.tap_followup_ready { … }`）整体替换为：

```rust
        // 排队 / 撞击后续只在寻路仍占 dp3 或收尾待执行时有效：其余时刻的残留（开战挂起、剧情接管等）丢弃
        if (self.tap_pending.is_some() || self.tap_bump.is_some())
            && !self.tap_followup_ready
            && !(self.game_mode == GameMode::Dialog && self.walkto_owns_dp3())
        {
            self.tap_pending = None;
            self.tap_bump = None;
        }
        if self.tap_followup_ready {
            self.tap_followup_ready = false;
            let bump = self.tap_bump.take();
            let pending = self.tap_pending.take();
            let ready = self.game_mode == GameMode::Map
                && self.walk_phase == 0
                && !self.overlay_active
                && !self.transitioning
                && self.key_value == 0
                && self.key_held == 0;
            if ready {
                match pending {
                    Some(crate::tap_walk::PendingTap::Cell(x, y)) => {
                        let _ = self.tap_cell(x, y);
                    }
                    Some(crate::tap_walk::PendingTap::Swipe(d)) => {
                        let _ = self.swipe_step(d);
                    }
                    None => {
                        if let Some((dock, dir)) = bump {
                            if (self.player_cell_x, self.player_cell_y) == dock {
                                self.execute_bump(dir);
                            }
                        }
                    }
                }
            }
        }
```

目标框清除条件加入排队：`!(self.route.is_open() || self.walkto_draining || self.tap_bump.is_some() || self.tap_followup_ready || self.tap_pending.is_some())`。

`clear_tap_state` 加 `self.tap_pending = None;`；`press_key` 开头加 `self.tap_pending = None;`。

- [ ] **Step 5: 运行，确认通过 + 变异**

Run: `sleep 2; touch crates/game-core/src/scene.rs crates/game-core/src/tap_walk.rs; cargo test --offline -p game-core --lib t7_` → 4 passed；`--lib tap_cell` 8 passed、`--lib t6_` 5 passed。
注：`press_key` 在 Dialog dp3 中只置 `key_value`，Map 落地后由 Map wp0 消费（朝墙 ⇒ 不动）；若该键在落地拍前已被消费、`key_value` 归 0，作废靠 `press_key` 开头清 `tap_pending`，不靠就绪条件。

变异（逐个、恢复）：
- `press_key` 去掉清 `tap_pending` ⇒ `t7_queued_tap_discarded_on_key_or_blocked_landing` 第一段红；
- 就绪条件去掉 `!self.overlay_active` ⇒ 第二段红；
- Dialog 分支去掉"点自己 = 停下"（一律排队）⇒ `t7_tap_self_while_walking_stops` 红或通过——若通过（排队到自己格 = Ignored，效果同停下），在 commit 正文说明该变异等价、不计入；
- `PendingTap::Swipe` 执行改为什么都不做 ⇒ `t7_swipe_steps_when_idle_and_queues_while_walking` 红。

- [ ] **Step 6: 全量 + classic 门禁**

全量测试 0 失败；`python3 gates/cli.py trace` PASS。

- [ ] **Step 7: 复核 + 提交**

附录 A 复核（T7），重点：残留丢弃规则（构造开战挂起 / 拾取挂起 / 剧情接管三种排空没走到 `finish_walkto` 的路径，确认排队不会在之后某次 `finish_walkto` 被误执行）、BUG-040 排空在门 / 楼梯 / 暗墙处的收尾、排队执行时再走 `tap_cell` 的怪物选中语义（排队点怪 = 选中，不开战）。提交：

```bash
git add crates/game-core/src/tap_walk.rs crates/game-core/src/scene.rs
git commit -F - <<'EOF'
feat(core): 点地寻路中的点按 / 滑动排队改道、点自己停下、swipe_step（STAGE-B-008 · spec §5.2 / §5.3）

寻路中点别处 = 打断并记下新目标（只留最后一次），排空回 Map wp0 后下一拍执行；点自己 = 停下。
就绪条件与撞击后续相同（Map wp0、无浮层 / 遮幅、无新按键），否则丢弃；排空没走到 finish_walkto 的残留每拍丢弃。
swipe_step：空闲探步一格，寻路中排队。

动手前三问：
1. 层：Stage B 点地策略（game-core；classic 按键链不经过——press_key 新增行作用于空状态）。
2. 消费者：壳层 tap_cell / swipe_step（Task 11 / 12）。
3. 证据：A 级——4 条场景测试 + 变异被杀；全量测试 0 失败；gates trace PASS。
Evidence: t7_* 测试。
Confidence: 85
EOF
```

---

### Task 8: 点地反馈绘制 `paint_tap_markers`（modern 专用，classic 零 op）

**Files:**
- Modify: `crates/game-core/src/scene.rs`：模块常量；新增 `paint_tap_markers`；`paint` 的地图路径（`paint_entities` 之后，约 6312 行）与 `paint_dialog`（`paint_entities` 之后，约 7078 行）各调一次
- Test: `mod t2_walkto_route_tests` 末尾追加

规则（spec §6）：层级 = 实体层之上、HUD / 小地图 / 浮层之下；只在 `layout.preset == Preset::Modern` 画；闪红按帧计数（`frame_counter < until`）；选中怪物框按 `frame_counter >> 1` 闪烁，持 13 号火眼金睛时在框旁画预计掉血（`predict_hp_loss(t, true)`，与观察模式 a.java:6617-6638 的 `battle_digit_by_type` 同源同位置，不可胜画同一"无法战胜"字形）。

- [ ] **Step 1: 写失败测试**

```rust
    fn marker_ops(sc: &GameScene) -> Vec<String> {
        let mut img = crate::render::ArgbImage::create(sc.layout.screen_w, sc.layout.screen_h);
        let mut g = crate::render::SoftGraphics::new(&mut img);
        sc.paint_tap_markers(&mut g, sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        g.ops.clone()
    }

    fn full_frame_ops(sc: &mut GameScene) -> Vec<String> {
        let mut img = crate::render::ArgbImage::create(sc.layout.screen_w, sc.layout.screen_h);
        let mut g = crate::render::SoftGraphics::new(&mut img);
        sc.paint(&mut g, &crate::render::ArgbImage::create(2, 2));
        g.ops.clone()
    }

    fn put_markers(sc: &mut GameScene) {
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.tap_target = Some((5, ROW));
        sc.tap_reject = Some(((4, ROW), sc.frame_counter + crate::tap_walk::TAP_REJECT_TICKS));
        sc.tap_selected = Some(crate::tap_walk::SelectedEnemy { entity: e, type_: 41, cell: (8, ROW), floor: sc.current_floor });
    }

    /// 【STAGE-B-008 · spec §6】classic：反馈层零 op，整帧 op 流与无点地状态时逐条相同。
    #[test]
    fn t8_markers_classic_zero_ops() {
        let mut a = scene((2, ROW), &[(41, 8, ROW)]);
        let mut b = scene((2, ROW), &[(41, 8, ROW)]);
        put_markers(&mut a);
        assert!(marker_ops(&a).is_empty(), "classic 反馈层零 op");
        assert_eq!(full_frame_ops(&mut a), full_frame_ops(&mut b), "classic 整帧 op 流不变");
    }

    /// modern：目标金框 / 闪红红框加叉（到期即不画）/ 选中闪烁框；火眼金睛才画预计掉血；整帧里确有反馈层。
    #[test]
    fn t8_markers_modern_draw_target_reject_and_selection() {
        let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        put_markers(&mut sc);
        let (ox, oy) = (sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
        let ops = marker_ops(&sc);
        assert!(ops.contains(&format!("drawRect({},{},32,32,#ffe070)", ox + 5 * 32, oy + ROW * 32)), "目标金框：{ops:?}");
        assert!(ops.iter().any(|o| o.contains("#ff3030")), "闪红");
        assert!(ops.iter().any(|o| o.starts_with(&format!("drawRect({},{},32,32,", ox + 8 * 32, oy + ROW * 32))), "选中框");
        sc.frame_counter += crate::tap_walk::TAP_REJECT_TICKS;
        assert!(!marker_ops(&sc).iter().any(|o| o.contains("#ff3030")), "闪红到期不画");
        let without = marker_ops(&sc).len();
        sc.player.item_types[0] = 13;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        assert!(marker_ops(&sc).len() > without, "持火眼金睛：多画预计掉血");
        assert!(full_frame_ops(&mut sc).iter().any(|o| o.contains("#ffe070")), "整帧含反馈层");
    }
```

- [ ] **Step 2: 运行，确认失败**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib t8_ 2>&1 | grep -E "error\[|test result"`
Expected: 编译失败（`paint_tap_markers` 未定义）。

- [ ] **Step 3: 实现**

scene.rs 模块常量区（`MINIMAP_PLAYER_DOT` 附近）加：

```rust
/// 【STAGE-B-008 · spec §6】点地反馈色（modern 专用）：目标 / 选中框金、闪红红、选中闪烁的另一相位橙。
const TAP_TARGET_RGB: u32 = 0xFFE070;
const TAP_REJECT_RGB: u32 = 0xFF3030;
const TAP_SELECT_ALT_RGB: u32 = 0xFF8040;
```

`paint_entities` 之前新增：

```rust
    /// 【STAGE-B-008 · spec §6】点地反馈层：目标框（金）、闪红（红框 + 叉）、选中怪物（闪烁框；持火眼金睛（13）旁标预计掉血，
    /// 与观察模式 a.java:6617-6638 同源同位）。层级 = 实体层之上、HUD / 浮层之下。classic 零绘制 op。
    fn paint_tap_markers(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        if self.layout.preset != crate::layout::Preset::Modern {
            return;
        }
        fn frame(g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32) {
            g.draw_rect(x, y, w, 32);
            g.draw_rect(x + 1, y + 1, w - 2, 30);
        }
        if let Some((x, y)) = self.tap_target {
            g.set_color(TAP_TARGET_RGB);
            frame(g, cam_x + (x << 5), cam_y + (y << 5), 32);
        }
        if let Some(((x, y), until)) = self.tap_reject {
            if self.frame_counter < until {
                let (sx, sy) = (cam_x + (x << 5), cam_y + (y << 5));
                g.set_color(TAP_REJECT_RGB);
                frame(g, sx, sy, 32);
                g.draw_line(sx + 8, sy + 8, sx + 23, sy + 23);
                g.draw_line(sx + 23, sy + 8, sx + 8, sy + 23);
            }
        }
        if let Some(s) = self.tap_selected {
            let w = entity::width_cells(s.type_, &self.width_table) << 5;
            let (sx, sy) = (cam_x + (s.cell.0 << 5), cam_y + (s.cell.1 << 5));
            g.set_color(if (self.frame_counter >> 1) & 1 == 0 { TAP_TARGET_RGB } else { TAP_SELECT_ALT_RGB });
            frame(g, sx, sy, w);
            if self.player.find_item(13) >= 0 {
                let digit = self.predict_hp_loss(s.type_, true);
                if digit >= 0 {
                    crate::popup::draw_digit_strip(g, &self.images.battle_strip, digit, sx + 30, sy - 17);
                } else {
                    draw_image_clipped(g, &self.images.battle_strip, sx + 14, sy - 17, 70, 0, 7, 9);
                }
            }
        }
    }
```

`paint` 地图路径与 `paint_dialog` 中，`self.paint_entities(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y);` 之后各加：

```rust
        self.paint_tap_markers(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y); // 【STAGE-B-008】
```

（`paint_menu9` 炼丹页不加：进炉页时点地状态已结束。）

- [ ] **Step 4: 运行，确认通过 + 门禁 + 变异**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib t8_` → 2 passed。
Run: `python3 gates/cli.py visual`（SKIP 为既有项则照常 SKIP）、`python3 tools/frames_fixture.py check`、全量测试 0 失败。
变异（逐个、恢复）：去掉 preset 判断 ⇒ `t8_markers_classic_zero_ops` 红；闪红去掉到期判断 ⇒ modern 测试"闪红到期不画"红；火眼金睛判断改恒真 ⇒ "多画预计掉血"红。

- [ ] **Step 5: 复核 + 提交**

附录 A 复核（T8），重点：Windows classic 预设（鼠标点地）不画任何反馈——属 spec §6"只在 modern 布局出现"的取舍，复核者确认 Windows modern 下框位置与绘制网格一致（相机偏移、`view_top_y`）；`predict_hp_loss` 对类目 8 的每种型（含 60/61/62、50 层型 72）不越界。提交：

```bash
git add crates/game-core/src/scene.rs
git commit -F - <<'EOF'
feat(core): 点地反馈绘制——目标金框 / 闪红 / 选中闪烁框与火眼金睛预计掉血（STAGE-B-008 · spec §6）

paint_tap_markers 画在实体层之上、HUD / 浮层之下，地图与点地寻路的 Dialog 帧各调一次；只在 modern 布局画。
预计掉血与观察模式同源（predict_hp_loss(t, true)、同位置、同"无法战胜"字形），持 13 号道具才画（Q10）。

动手前三问：
1. 层：Stage B 绘制（game-core paint，modern 专用）。
2. 消费者：Android / Windows modern 画面；classic 回放与帧夹具不经过（零 op）。
3. 证据：A 级——classic 整帧 op 流逐条相同 + modern 绘制测试 + 3 个变异被杀；frames_fixture check / gates visual 照旧。
Evidence: t8_markers_classic_zero_ops、t8_markers_modern_draw_target_reject_and_selection。
Confidence: 85
EOF
```

---

### Task 9: 赠予 / 报价框（kind 4、src 0、flags&2）点空白不代按

**Files:**
- Modify: `crates/game-core/src/scene.rs`（`GameScene::blank_tap_key`，约 9630 行；告示牌 / 商人注释，约 5480 行）
- Test: 同文件 `blank_tap_key` 既有测试所在模块（`grep -n "fn .*blank_tap" crates/game-core/src/scene.rs` 定位）末尾追加

背景（spec §5.2、§2.1 kind 4 子类，A 级：`scene.rs:5480-5512` 开框、`apply_merchant_offer` `:4448` 起）：型 77 报价框图标是 (0,0)，`blank_tap_key` 现按图标判 ⇒ 点空白代按 −5 = 接受（扣金 / 送物）。按 BUG-039"要做决定的框点空白不代按"原则改为 None。判据与 −5 实际执行的分支同源：`merchant_entity` 指向型 77 且其 param 高字节 &2（买成功后 ^512 清掉 &2、续显的告示类子框照旧代按）。

- [ ] **Step 1: 写失败测试**

```rust
    /// 【STAGE-B-008 · spec §5.2】赠予 / 报价（kind 4、型 77、flags&2）点空白不代按；同实体 flags&2 = 0（告示子类）照旧代按 −5；
    /// 型 78 报价（○/× 图标）本就不代按。
    #[test]
    fn blank_tap_does_not_accept_type77_offer() {
        let mut sc = scene((2, ROW), &[(77, 5, ROW), (78, 7, ROW)]);
        let i77 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 77).unwrap();
        let i78 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 78).unwrap();
        sc.overlay_active = true;
        sc.overlay_kind = 4;
        sc.merchant_entity = i77 as i32;
        sc.entities.items[i77].param = (2 << 8) | 1;
        sc.overlay_left_icon = 0;
        sc.overlay_right_icon = 0;
        assert_eq!(sc.blank_tap_key(), None, "型 77 报价：点空白不接受");
        sc.entities.items[i77].param = 1;
        assert_eq!(sc.blank_tap_key(), Some(-5), "型 77 告示子类（flags&2 = 0）：点空白关闭");
        sc.merchant_entity = i78 as i32;
        sc.entities.items[i78].param = (2 << 8) | 1;
        sc.overlay_left_icon = 1;
        sc.overlay_right_icon = 2;
        assert_eq!(sc.blank_tap_key(), None, "型 78 报价（○/×）不代按（原规则）");
        sc.merchant_entity = -1;
        sc.overlay_left_icon = 0;
        sc.overlay_right_icon = 0;
        assert_eq!(sc.blank_tap_key(), Some(-5), "无商人实体的 kind 4（长文浏览）照旧");
    }
```

- [ ] **Step 2: 运行，确认失败**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib blank_tap_does_not_accept`
Expected: FAIL（"型 77 报价：点空白不接受"，left Some(-5)）。

- [ ] **Step 3: 实现**

`GameScene::blank_tap_key` 的 `if self.overlay_active {` 块开头加：

```rust
            // 【STAGE-B-008 · spec §5.2】赠予 / 报价（kind 4、型 77 = 文本源 0、param 高字节 &2）无脚注图标，但 −5 = 接受（扣金 / 送物），
            // 是要做决定的框 ⇒ 点空白不代按（曾代按 −5）。判据与 apply_merchant_offer 同源（买成功 ^512 后的告示子框照旧代按）。
            if self.overlay_kind == 4 && self.merchant_entity >= 0 {
                let ent = &self.entities.items[self.merchant_entity as usize];
                if ent.type_ == 77 && ((ent.param as i32) >> 8) & 2 != 0 {
                    return None;
                }
            }
```

函数头注的"道具使用确认 kind 3、商人报价 kind 4"之后补"、赠予 / 报价（kind 4 型 77 flags&2，STAGE-B-008）"。

注释更正（spec §11"`scene.rs:5436` kind 4 flags 注释"，现约 5482 行）：

```rust
                                // 高字节=标志（&1=open 入队、&2=效果表、&4=一次性），
```
改为：
```rust
                                // 高字节=标志（&1=交互入队且两键都消耗实体、&2=报价（效果表）、&4=不消耗 / 买后不翻转（可重复））
                                // > 【2026-10-30 更正 · STAGE-B-008】原注"&4=一次性"与 apply_merchant_offer（:4535-4578）相反。
```

- [ ] **Step 4: 运行，确认通过**

Run: `sleep 2; touch crates/game-core/src/scene.rs; cargo test --offline -p game-core --lib blank_tap` → 全部 passed；全量测试 0 失败（`shell/android/tests/tap_routing.rs` 中既有 kind 4 / 是否框用例照绿）。

- [ ] **Step 5: 复核 + 提交**

附录 A 复核（T9），重点：`merchant_entity` 在关框后是否残留（残留时下一次非商人 kind 4 框会被误判？——给出所有 `show_overlay_message(4, …)` 调用点的 `merchant_entity` 状态）、`param` 符号扩展（i16 → i32 右移）对 &2 的影响。提交：

```bash
git add crates/game-core/src/scene.rs
git commit -F - <<'EOF'
feat(core): 赠予 / 报价框（kind 4 型 77 flags&2）点空白不代按（STAGE-B-008 · spec §5.2）

blank_tap_key 现按图标判，型 77 报价无图标 ⇒ 点空白代按 −5 = 接受。按 BUG-039 原则改为不代按；判据与 apply_merchant_offer 同源。
附带更正告示牌 / 商人 flags 注释（&4 实为不消耗 / 可重复，原注"一次性"相反）。

动手前三问：
1. 层：Stage B 触屏点空白映射（blank_tap_key 只被壳层点按调用；classic 按键链不经过）。
2. 消费者：Android scene_tap、Windows click_key（点空白兜底）。
3. 证据：A 级——scene.rs 开框分支与 apply_merchant_offer 读同一 param；新测 RED→GREEN。
Evidence: blank_tap_does_not_accept_type77_offer。
Confidence: 90
EOF
```

---

### Task 10: 壳公共层改为"触点 → 格"，分类交给 core（touch.rs / input.rs / undo_ui.rs）

**Files:**
- Modify: `shell/common/src/touch.rs`（`TapAction` / `classify_tap` → `TapTarget` / `tap_target`；`ShellAction`；`Dispatcher`；8 条测试改写，条数不变）
- Modify: `shell/android/src/input.rs`（`TouchInput::feed` 去掉 `player` / `walkable` 参数；2 条测试改期望）
- Modify: `shell/common/src/undo_ui.rs`（`undo_last` 先 `sc.clear_tap_state()`；加 1 条测试）

背景：旧 `classify_tap` 在壳层按地形判"点地 = walk_to、点墙 = 探步"，与 spec §5 的格分类（实体可见性、怪物选中、停靠）重复且不一致（spec §5.4"`classify_tap` 墙格分支改为走到邻格再撞"）。改为壳只做"触点 → 撤销钮 / 格 / 图外"，格的语义全在 core `tap_cell`。点按 / 滑动不再先发 `Interrupt`：寻路中的点按由 core 排队（Task 7），先打断会让排队无从发生。按键通道照旧先发 `Interrupt`。撤销钮只在 `field_tap_allowed` 时执行（BUG-038 同口径，由壳执行时判）。

- [ ] **Step 1: 改写测试（先红）**

`shell/common/src/touch.rs` 测试模块：
- `cell_at_four_corners_of_tile_window`、`cell_at_out_of_map_is_none`、`undo_zone_hit_corners`、`swipe_maps_to_facing_and_one_step_per_gesture` 不动。
- `swipe_wall_stop_no_repeat` 中 `Dispatcher::swipe(2)` 的期望改为 `vec![ShellAction::Swipe(2)]`，断言文字改为"滑动 = 一次 Swipe（探步 / 排队由 core swipe_step 决定）"。
- `tap_classifies_walk_wall_probe_and_undo` 改名并改写为：

```rust
    #[test]
    fn tap_target_maps_cells_undo_and_outside() {
        let g = geom();
        assert_eq!(tap_target(90, 80, &g, 240, 320), TapTarget::Cell(7, 2), "空地格");
        assert_eq!(tap_target(50, 40, &g, 240, 320), TapTarget::Cell(6, 1), "墙格也只报格（墙 / 门 / 怪由 core tap_cell 分类）");
        assert_eq!(tap_target(200, 5, &g, 240, 320), TapTarget::Undo, "撤销区优先于格");
        assert_eq!(tap_target(17, 37, &g, 240, 320), TapTarget::Cell(5, 1), "玩家格照报（点自己 = 停下 / 无动作由 core 判）");
        assert_eq!(tap_target(5000, 0, &g, 240, 320), TapTarget::Outside, "图外");
    }
```
- `dual_mode_keys_and_taps_coexist`：`Dispatcher::tap(tap_target(90, 80, …))` 期望 `vec![ShellAction::TapCell((7, 2))]`；`Dispatcher::tap(TapTarget::Cell(12, 2))` 期望 `vec![ShellAction::TapCell((12, 2))]`；按键 / 抬起期望不变。
- `interrupt_signal_on_every_channel` 改名 `interrupt_only_on_key_channels`：

```rust
    #[test]
    fn interrupt_only_on_key_channels() {
        let g = geom();
        // 点按 / 滑动 / 撤销不先打断：寻路中的点按与滑动由 core 排队（STAGE-B-008 spec §5.2），撤销仅空闲时执行
        assert_eq!(Dispatcher::tap(tap_target(90, 80, &g, 240, 320)), vec![ShellAction::TapCell((7, 2))]);
        assert_eq!(Dispatcher::swipe(0), vec![ShellAction::Swipe(0)]);
        assert_eq!(Dispatcher::tap(TapTarget::Undo), vec![ShellAction::Undo]);
        assert_eq!(Dispatcher::tap(TapTarget::Outside), Vec::new());
        // 按键通道照旧先打断（手动优先，BUG-040）
        assert_eq!(Dispatcher::key(-1)[0], ShellAction::Interrupt, "方向键通道发打断");
        assert_eq!(Dispatcher::key(-7)[0], ShellAction::Interrupt, "软键通道发打断");
    }
```

`shell/android/src/input.rs` 两条测试：`feed` 调用去掉末两个参数；`dual_mode_pad_keys_and_free_taps_coexist` 中 tap 期望改为 `vec![ShellAction::TapCell((7, 2))]`（`a6[0]`）；`free_tap_wall_probe_undo_and_swipe_step` 改名 `free_tap_cell_undo_and_swipe`，点墙格期望 `vec![ShellAction::TapCell((6, 1))]`，撤销区期望 `vec![ShellAction::Undo]`，滑动期望 `vec![ShellAction::Swipe(2)]`。

`shell/common/src/undo_ui.rs` 测试模块追加：

```rust
    /// 【STAGE-B-008 · spec §7】撤销同时清目标标记、怪物选中、排队动作。
    #[test]
    fn undo_clears_tap_state() {
        let mut sc = <该模块既有的场景构造（与 `walk_to(3, 10)` 用例同一夹具）>;
        let _ = sc.walk_to(3, 10);
        <跑到位（同既有用例）>;
        sc.tap_target = Some((3, 10));
        sc.tap_reject = Some(((4, 10), sc.frame_counter + 8));
        let _ = undo_last(&mut sc);
        assert_eq!(sc.tap_target, None);
        assert_eq!(sc.tap_reject, None);
    }
```
（尖括号处照抄同模块 `walk_to(3, 10)` 那条既有测试的夹具与推进代码，不新造夹具。）

Run: `sleep 2; touch shell/common/src/*.rs shell/android/src/input.rs; cargo test --offline -p game-shell-common --lib 2>&1 | grep -E "error\[|test result"`
Expected: 编译失败（`tap_target` / `TapTarget` / `ShellAction::TapCell` 未定义）。

- [ ] **Step 2: 实现 touch.rs**

`TapAction` 与 `classify_tap` 整体替换为：

```rust
/// 触点落点（撤销区 > 地图格 > 图外）。格的语义（地面 / 墙 / 交互物 / 怪 / 点自己）由 core `GameScene::tap_cell` 判（STAGE-B-008）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapTarget {
    Undo,
    Cell(i32, i32),
    Outside,
}

/// 触点 → 落点。
/// > 【2026-10-30 更正 · STAGE-B-008】原 `classify_tap` 在壳层按地形判"点地 = walk_to、点墙 = 探步"，
/// > 与 spec §5 的格分类（实体可见性、怪物选中、停靠再撞）不一致，已由本函数 + core `tap_cell` 取代。
pub fn tap_target(x: i32, y: i32, geom: &TouchGeom, screen_w: i32, screen_h: i32) -> TapTarget {
    if undo_zone_hit(x, y, screen_w, screen_h) {
        return TapTarget::Undo;
    }
    match geom.cell_at(x, y) {
        Some((cx, cy)) => TapTarget::Cell(cx, cy),
        None => TapTarget::Outside,
    }
}
```

`ShellAction` 与 `Dispatcher` 替换为：

```rust
/// 壳侧动作流（win / android 同构消费；执行 = core 入口）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellAction {
    /// 打断寻路（按键通道首动作；core interrupt_route 幂等）
    Interrupt,
    /// 点中格（core `tap_cell`）
    TapCell((i32, i32)),
    /// 滑动一格（core `swipe_step`，facing 域 0=Down 1=Up 2=Right 3=Left）
    Swipe(u8),
    /// 撤销（壳在 `field_tap_allowed` 时执行）
    Undo,
    Key(i32),
    KeyRelease,
}

/// 全通道分发器：按键通道先发 Interrupt（手动优先）；点按 / 滑动不先打断（寻路中由 core 排队，STAGE-B-008）；
/// 双模并存（键 / 触动作流无互斥态）。
pub struct Dispatcher;

impl Dispatcher {
    pub fn tap(t: TapTarget) -> Vec<ShellAction> {
        match t {
            TapTarget::Undo => vec![ShellAction::Undo],
            TapTarget::Cell(cx, cy) => vec![ShellAction::TapCell((cx, cy))],
            TapTarget::Outside => Vec::new(),
        }
    }

    /// 滑动 = 一格（空闲探步 / 寻路中排队，由 core 决定）。
    pub fn swipe(dir: u8) -> Vec<ShellAction> {
        vec![ShellAction::Swipe(dir)]
    }

    pub fn key(code: i32) -> Vec<ShellAction> {
        vec![ShellAction::Interrupt, ShellAction::Key(code)]
    }

    pub fn release() -> Vec<ShellAction> {
        vec![ShellAction::KeyRelease]
    }
}
```

- [ ] **Step 3: 实现 input.rs / undo_ui.rs**

`shell/android/src/input.rs` `TouchInput::feed`：签名去掉 `player: (i32, i32), walkable: impl Fn(i32, i32) -> bool`；Up 分支改为 `Gesture::Tap(x, y) => tch::Dispatcher::tap(tch::tap_target(x, y, geom, screen_w, screen_h)),`；头注"点按移动 / 滑动踏步 / 撤销钮"改为"点按（格交 core tap_cell）/ 滑动（core swipe_step）/ 撤销钮"。

`shell/common/src/undo_ui.rs` `undo_last` 第一行加：

```rust
    // 【STAGE-B-008 · spec §7】撤销同时清目标标记、怪物选中、排队动作
    sc.clear_tap_state();
```

- [ ] **Step 4: 运行**

Run: `sleep 2; touch shell/common/src/*.rs shell/android/src/input.rs; cargo test --offline -p game-shell-common --lib` → 全部 passed（touch 8 条 + undo_ui 新增 1 条）。
此时 `game-android` / `game-desktop` 因 `classify_tap` / `ShellAction::WalkTo` 被删而编译失败——预期内，Task 11 / 12 修复；本任务**不单独提交**，与 Task 11、12 一起在 Task 12 末尾提交（工作区始终可编译的提交粒度）。

---

### Task 11: Android `scene_tap` 改走 `tap_cell`，宿主测试按新规则重写

**Files:**
- Modify: `shell/android/src/native.rs`（`scene_tap`，约 332 行）
- Modify: `shell/android/tests/tap_routing.rs`（导入、`field_points`、BUG-043 用例；新增 4 条真资产用例）
- Modify: `shell/android/tests/native_parse.rs`（`tap_chain_buf_to_walk_to`）

- [ ] **Step 1: 改宿主测试（先红）**

`shell/android/tests/tap_routing.rs`：
- 导入改为 `use game_shell_common::touch::TouchGeom;` 与 `use game_core::tap_walk::{TapResult, TargetKind};`。
- `field_points` 改为按 core 格分类找点（"点地"= 可达 Floor 格、"探步"= Wall 格），不再调 `classify_tap`：

```rust
fn field_points(h: &H) -> ((f32, f32), (f32, f32)) {
    let sc = h.sc();
    let geom = TouchGeom { cam_x: sc.view.cam_x, cam_y: sc.view.cam_y, tile_off: sc.layout.view_top_y, cols: sc.view.wide, rows: sc.view.high };
    let grid = sc.tap_grid();
    let me = (sc.player_cell_x, sc.player_cell_y);
    let (mut walk, mut probe) = (None, None);
    for y in (40..sc.layout.screen_h - 40).step_by(8) {
        for x in (8..sc.layout.screen_w - 8).step_by(8) {
            let Some(c) = geom.cell_at(x, y) else { continue };
            if c == me {
                continue;
            }
            match grid.get(c.0, c.1).map(|k| k.kind) {
                Some(TargetKind::Floor) if walk.is_none() => walk = Some((x as f32, y as f32)),
                Some(TargetKind::Wall) if probe.is_none() => probe = Some((x as f32, y as f32)),
                _ => {}
            }
        }
    }
    (walk.expect("有地面格"), probe.expect("有墙格"))
}
```
- `tutorial_arrows_stay_on_their_path_during_tap_walk`：点地路线现由 BFS 给出，可能与箭头路线不同（spec §5.4）。参照局改为"方向键沿**点地实际走的路线**逐格走"：点按前用 `game_core::tap_walk::plan_tap(&sc.tap_grid(), from, target)` 取步序（被测路线是**输入**；期望 = 经典方向键走同一路线后的余下箭头，由经典按键链独立产生），断言点按后 `walk_path_buffer[..walk_step_count]` 与该步序一致，再按步序（目标端优先 ⇒ 逆序）换成方向键码（0→−2、1→−1、2→−4、3→−3）驱动参照局。逐拍"箭头是原路线尾段"的断言保留。
- `tap_walk_case`：`free` 增加"非陷阱危险格"条件（`!sc.tap_grid().get(x, y).unwrap().danger`），其余不动（BUG-042 点位来自绘制公式，不变）。
- 新增：

```rust
/// 【STAGE-B-008 · spec §0 隐藏通路】54 层：(4,2) 只能经未显形的隐形路径到达——点它不得走到、隐形路径不得显形。
#[test]
fn tap_does_not_leak_hidden_path_floor54() {
    let mut h = to_map("hidden54");
    h.sc_mut().change_floor(54, false, false);
    for _ in 0..60 {
        h.step(vec![], vec![]);
    }
    let target = (4, 2);
    // 前置（独立 BFS，不经被测代码）：只走地形可走格不可达；把隐形路径格算作可走则可达
    let (no_hidden, with_hidden) = {
        let sc = h.sc();
        let hidden: std::collections::HashSet<(i32, i32)> = sc.entities.items[..sc.entities.count]
            .iter()
            .filter(|e| e.type_ == 6 && !e.removed && !e.visible)
            .map(|e| (e.px >> 5, e.py >> 5))
            .collect();
        let reach = |with: bool| {
            let me = (sc.player_cell_x, sc.player_cell_y);
            let mut seen = std::collections::HashSet::from([me]);
            let mut q = std::collections::VecDeque::from([me]);
            while let Some((x, y)) = q.pop_front() {
                for n in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    let inside = n.0 >= 0 && n.1 >= 0 && n.0 < sc.view.wide && n.1 < sc.view.high;
                    if inside && (sc.walkable[n.1 as usize][n.0 as usize] || (with && hidden.contains(&n))) && seen.insert(n) {
                        q.push_back(n);
                    }
                }
            }
            seen.contains(&target)
        };
        (reach(false), reach(true))
    };
    assert!(!no_hidden && with_hidden, "前置：(4,2) 只经隐形路径可达（实得 {no_hidden}/{with_hidden}）");
    let (sx, sy) = { let sc = h.sc(); (sc.view.cam_x + (target.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (target.1 << 5) + 16) };
    h.step(vec![], vec![(sx as f32, sy as f32)]);
    for _ in 0..200 {
        h.step(vec![], vec![]);
    }
    let sc = h.sc();
    assert_ne!((sc.player_cell_x, sc.player_cell_y), target, "不得走到");
    assert!(sc.entities.items[..sc.entities.count].iter().filter(|e| e.type_ == 6 && !e.removed).all(|e| !e.visible), "隐形路径不显形");
}

/// 23 层：隐形墙（型 12，看起来是地面）——点它：路线照常走过去，撞上即显形并停在墙前，玩家从不站上隐形墙格（与方向键同效）。
#[test]
fn tap_into_hidden_wall_reveals_and_blocks_floor23() {
    let mut h = to_map("hidden23");
    h.sc_mut().change_floor(23, false, false);
    for _ in 0..60 {
        h.step(vec![], vec![]);
    }
    // 目标 = 首个在屏、经地形可走格（独立 BFS）可达的未显形型 12 格
    let (target, e12) = {
        let sc = h.sc();
        <独立 BFS（同上一用例 reach(false) 的写法）求可达集；在其中找 type_ == 12 ∧ !visible ∧ !removed ∧ 在屏（tap_walk_case 的 on_screen 同式）的实体，取第一个，返回 (格, 实体下标)>
    };
    let walls12: Vec<(i32, i32)> = { let sc = h.sc(); sc.entities.items[..sc.entities.count].iter().filter(|e| e.type_ == 12 && !e.removed).map(|e| (e.px >> 5, e.py >> 5)).collect() };
    let (sx, sy) = { let sc = h.sc(); (sc.view.cam_x + (target.0 << 5) + 16, sc.view.cam_y + sc.layout.view_top_y + (target.1 << 5) + 16) };
    h.step(vec![], vec![(sx as f32, sy as f32)]);
    for _ in 0..240 {
        h.step(vec![], vec![]);
        let me = (h.sc().player_cell_x, h.sc().player_cell_y);
        assert!(!walls12.contains(&me), "玩家站上隐形墙格 {me:?}");
    }
    assert!(h.sc().entities.items[e12].visible, "撞上的隐形墙显形");
    assert_ne!((h.sc().player_cell_x, h.sc().player_cell_y), target);
}

/// 寻路中再点：排队改道（生产路径 scene_tap → tap_cell）。
#[test]
fn tap_during_walk_reroutes() {
    let mut h = to_map("queue");
    let (a, b) = <在 tap_walk_case 的可达在屏空格表里取曼哈顿距离玩家最远的格 a，和距 a 最远的格 b（b ≠ 玩家格）>;
    <点 a（绘制公式求点）；推进到玩家离开出生格且 game_mode == Dialog；点 b；推进 ≤ 300 拍直到停在地图态>;
    assert_eq!((h.sc().player_cell_x, h.sc().player_cell_y), b, "改道到第二次点中的格");
}

/// 战斗中点画面：不代按、不打断战斗，闪红（scene_tap 直调；战斗态由场景字段构造，不跑 tick）。
#[test]
fn tap_during_battle_is_rejected_without_key() {
    let mut h = to_map("battle");
    let (walk, _) = field_points(&h);
    let sc = h.sc_mut();
    sc.walk_phase = 5;
    let key = game_android::native::scene_tap(sc, walk.0 as i32, walk.1 as i32);
    assert_eq!(key, None, "不代按");
    assert_eq!(sc.walk_phase, 5, "战斗不被打断");
    assert!(sc.tap_reject.is_some(), "闪红");
}
```
（尖括号处由实施者按注明的方法写出；期望值必须由独立 BFS / 绘制公式求得，不得调 `tap_grid` / `plan_tap` 求期望。）

`shell/android/tests/native_parse.rs` `tap_chain_buf_to_walk_to` 改名 `tap_chain_buf_to_cell`，导入改 `use game_shell_common::touch::{tap_target, TapTarget, TouchGeom};`，断言改为 `assert_eq!(tap_target(lx as i32, ly as i32, &geom, 576, 576), TapTarget::Cell((lx as i32) / 32, (ly as i32) / 32), "点格→格全链");`。

- [ ] **Step 2: 实现 `scene_tap`**

`shell/android/src/native.rs` `scene_tap` 的 `if sc.field_tap_allowed() { … return None; }` 整块替换为：

```rust
    use game_shell_common::touch::{tap_target, Dispatcher, ShellAction, TouchGeom};
    if sc.accepts_field_tap() {
        let geom = TouchGeom {
            cam_x: sc.view.cam_x,
            cam_y: sc.view.cam_y,
            tile_off: sc.layout.view_top_y,
            cols: sc.view.wide,
            rows: sc.view.high,
        };
        let t = tap_target(x, y, &geom, sc.layout.screen_w, sc.layout.screen_h);
        for a in Dispatcher::tap(t) {
            match a {
                ShellAction::TapCell((cx, cy)) => {
                    let r = sc.tap_cell(cx, cy);
                    log_input(&format!(
                        "field tap ({x},{y}) cam=({},{}) cell=({cx},{cy}) player=({},{}) → {r:?}",
                        geom.cam_x, geom.cam_y, sc.player_cell_x, sc.player_cell_y
                    ));
                }
                ShellAction::Undo => {
                    // BUG-038：只在场上空闲时撤销（寻路中 / 战斗中点撤销钮无动作）
                    if sc.field_tap_allowed() {
                        let _ = game_shell_common::undo_ui::undo_last(sc);
                    }
                }
                ShellAction::Interrupt | ShellAction::Swipe(_) | ShellAction::Key(_) | ShellAction::KeyRelease => {}
            }
        }
        return None;
    }
```
函数头注第一条改为"场上收点地（[`GameScene::accepts_field_tap`]：地图态或点地寻路中，无浮层 / 遮幅）：触点 → 撤销钮 / 格 → core `tap_cell`（走 / 停靠 / 撞 / 选中 / 排队 / 闪红，STAGE-B-008），返回 None"。

- [ ] **Step 3: 运行**

Run: `sleep 2; touch shell/android/src/*.rs shell/common/src/*.rs crates/game-core/src/*.rs; cargo test --offline -p game-android 2>&1 | grep -E "^test .*FAILED|panicked|test result"`
Expected: 全部 passed（tap_routing 新增 4 条）。若 BUG-042 两条 `tap_walk_case` 有目标因 BFS 停靠语义变化而走不到：先确认该目标在新格分类下是否 Floor 且安全可达（打印 `tap_grid` 该格与路线），属测试前提变化的按 spec §5.4 改前提并在 commit 正文列出；属实现错误的修实现。
变异：`scene_tap` 的 `accepts_field_tap` 改回 `field_tap_allowed` ⇒ `tap_during_walk_reroutes` 与 `tap_during_battle_is_rejected_without_key` 红。

---

### Task 12: Windows 鼠标点地 / 拖动改走 `tap_cell` / `swipe_step`；三壳一起提交

**Files:**
- Modify: `shell/win/src/main.rs`（`pointer_tap` 约 686 行、`apply_touch_action` 约 655 行、拖动 `Dispatcher::swipe` 约 1186 行；测试 `t3_touch_actions_wire_to_scene` 约 2350 行）

- [ ] **Step 1: 改测试（先红）**

`t3_touch_actions_wire_to_scene`：两处 `A::WalkTo((px - 2, py))` 改为 `A::TapCell((px - 2, py))`；"打断动作清缓冲停步"一段保留（`A::Interrupt` 仍是按键通道首动作）；追加：

```rust
        // 拖动 = Swipe：空闲时朝左探步一格（core swipe_step）
        let x0 = h.scene().player_cell_x;
        h.shell.apply_touch_action(A::Swipe(3));
        for _ in 0..20 {
            h.run_tick(&[]);
        }
        assert!(h.scene().player_cell_x <= x0, "Swipe 动作经 core swipe_step（左探步，墙则原地）");
```

- [ ] **Step 2: 实现**

`apply_touch_action`：`A::WalkTo` 臂改 `A::TapCell((x, y))` ⇒ `let _ = sc.tap_cell(x, y);`；`A::ProbeStep` 臂改 `A::Swipe(d)` ⇒ `let _ = sc.swipe_step(d);`；`A::Undo` 臂内加 `if sc.field_tap_allowed()` 判断（BUG-038 同口径）。

`pointer_tap`：守卫 `Some(Screen::Game(sc)) if sc.field_tap_allowed()` 改 `if sc.accepts_field_tap()`；动作改为 `Dispatcher::tap(tap_target(x, y, &geom, self.layout.screen_w, self.layout.screen_h))`，删去 `player` / `walkable` 局部量；导入改 `use game_shell_common::touch::{tap_target, Dispatcher, ShellAction, TouchGeom};`；头注同 Task 11 口径更新。拖动处（`Dispatcher::swipe(dir)`）代码不变（动作已是 `Swipe`）。

- [ ] **Step 3: 全量 + 门禁**

Run: `sleep 2; touch shell/win/src/main.rs; cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle 2>&1 | grep -cE '^test .* FAILED$|panicked'` → 0。
Run: `python3 gates/cli.py check`、`python3 gates/cli.py trace` PASS。

- [ ] **Step 4: 复核 + 提交（Task 10–12 合一提交）**

附录 A 复核（T10–12），重点：Android pad 模式下右上隐形撤销区仍可用（计划 3 才去掉）、寻路中点撤销区 = 无动作（不打断、不撤销）是否可接受、Windows 可见撤销钮（`main.rs` 约 1322-1335 绘制）点击仍走 `TapTarget::Undo`、Windows classic 预设下鼠标点地新规则生效但不画反馈（Task 8 取舍）、`TouchInput` 是否还有生产调用者（应无，仅测试）。提交：

```bash
git add shell/common/src/touch.rs shell/common/src/undo_ui.rs shell/android/src/input.rs shell/android/src/native.rs \
        shell/android/tests/tap_routing.rs shell/android/tests/native_parse.rs shell/win/src/main.rs
git commit -F - <<'EOF'
feat(platform): 壳层点地改走 core tap_cell / swipe_step，壳只做"触点 → 格"（STAGE-B-008 · spec §5.4）

touch.rs：classify_tap（壳层按地形判点地 / 点墙）→ tap_target（撤销钮 / 格 / 图外）；ShellAction WalkTo / ProbeStep → TapCell / Swipe；
点按与滑动不再先发 Interrupt（寻路中由 core 排队），按键通道照旧。撤销只在 field_tap_allowed 时执行；undo_last 清点地状态。
Android scene_tap 与 Windows pointer_tap / 拖动改用新入口，accepts_field_tap 放行寻路中与战斗中的点按（排队 / 闪红）。

动手前三问：
1. 层：Stage B 壳层输入（shell/common、shell/android、shell/win）；classic 回放不经过壳层点按。
2. 消费者：Android 竖屏视口点按（pending_taps → scene_tap）、Windows 鼠标点击 / 拖动。
3. 证据：A 级——tap_routing 真资产用例（54 层隐藏通路不泄露、23 层隐形墙撞停显形、寻路中改道、战斗中闪红不代按）；
   改写测试逐条：touch.rs 4 条改期望 / 改名、input.rs 2 条、native_parse 1 条、Windows t3 1 条、tap_routing field_points / BUG-043 参照路线 / tap_walk_case 前提。
Evidence: cargo test 全量 0 失败；gates check / trace PASS。
Confidence: 80
EOF
```

---

### Task 13: 偏离登记、台账收口、ratchet 入账、实机冒烟

**Files:**
- Modify: `docs/spec/stage-b-deviations.md`（新增 STAGE-B-008 节，本计划部分）
- Modify: `docs/bug-ledger.md`（BUG-044 余项①②③收口注）
- Modify: `docs/superpowers/plans/2026-10-23-touch-native-undo-plan.md`（点墙 = 探步规则的原地更正注）
- Modify: `data/status/baselines/`（由 `gates/cli.py ratchet-update` 写，单独提交）

- [ ] **Step 1: 偏离登记**

`docs/spec/stage-b-deviations.md` 末尾追加（按 STAGE-B-006 节同式，内容按实际实施结果填，下为骨架）：

```markdown
## STAGE-B-008 双操控方案——点地规则与反馈（计划 2 部分）

- **状态**：部分实施（2026-10-30，计划 2：core 点地规则 / 反馈绘制 / 壳层改接；工具栏、操控方式与自动战斗设置行、Android 滑动 / 拖动手势、
  提示卡、帮助归计划 3）
- **用户裁定原话**：Q7"点哪里就往哪里走，点中的是东西就去和它交互"；Q8"默认是B,但配置中可以选：自动战斗（仅触屏模式有选项），选了就不用二次确认……
  使用回退来对抗错误操作"；Q5 A（目标标记、红色拒绝闪烁）；Q10 A（持火眼金睛才显示预计掉血）。
- **偏离点**：
  1. 点地寻路改为安全通行 BFS（只经可见地面、避开陷阱危险格、无扩展上限），点墙 / 门 / 炉 / 怪 = 走到邻格后在 Map wp0 撞一步；到不了 = 停最近可达格并闪红。
     原版无触屏；旧 Stage B 规则（find_path + 点墙探步）见 touch-native-undo 计划，已更正。
  2. 点怪首点选中、再点开战；`auto_battle`（core 字段，默认关；设置行计划 3）开时一点即战。
  3. 点地寻路中点按 / 滑动排队，走完当前格后执行；点自己 = 停下；战斗 / 手动行走中点按闪红、状态不变。
  4. 零长位移笔不入撤销栈；撤销清点地状态。
  5. 赠予 / 报价框（kind 4 型 77 flags&2）点空白不代按（原 BUG-039 规则按图标判会代按"接受"）。
  6. modern 布局画目标框 / 闪红 / 选中框与火眼金睛预计掉血。
- **门控**：只经壳层点按入口（Android scene_tap、Windows pointer_tap / 拖动）与 modern 绘制；classic 按键链、oracle 回放、帧夹具零变化
  （t8 classic 整帧 op 流逐条相同；gates trace / visual / frames_fixture check 照旧）。Windows classic 预设下鼠标点地用新规则、不画反馈。
- **金样影响**：无。
- **改写的测试**：<Task 1 / 10 / 11 / 12 提交正文列出的逐条清单>。
- **新增宿主钉**：<各任务新增测试名与条数>。
- **代码注释**：`【STAGE-B-008】`（tap_walk.rs / pathfind.rs / scene.rs / touch.rs / undo_ui.rs / input.rs / native.rs / win main.rs）。
- **设计件 / 计划 / 复核**：docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md / docs/superpowers/plans/2026-10-30-dual-control-core-tap-rules.md
  （各任务复核记录在 target/verify/review_plan2_T*.md，不入库）。
```

- [ ] **Step 2: 台账与旧计划更正**

`docs/bug-ledger.md` BUG-044 节"余项"之后加：

```markdown
  > 【2026-10-30 收口 · STAGE-B-008 计划 2】①② 生产路径不再出现：壳层点地改走 `tap_cell`，撞击一律在 Map wp0 执行（不进 dp3 路线），
  > 炉自正下方朝上撞直接 Map → Alchemy（`tap_cell_alchemy_from_below_opens_page`）。旧入口 `walk_to` 仅剩宿主测试 / 脚本面调用，未改。
  > ③ 已修：`RouteTracker::close` 零长段不入栈（`zero_length_segment_is_not_pushed`）。提交：<T1 / T5 / T12 的短哈希>。
```
标题中"部分修复——余项见下"改为"已修复（余项①② 随 STAGE-B-008 生产路径消除、③ 已修）"。

`docs/superpowers/plans/2026-10-23-touch-native-undo-plan.md` 中"点墙格 = 探步入"规则所在处（`grep -n "探步" 该文件` 定位首处规则陈述）加：

```markdown
> 【2026-10-30 更正 · STAGE-B-008】点墙 = 探步、点地 = find_path 的壳层分类已由 core `tap_cell`（安全通行 BFS、走到邻格再撞、怪物先选中）取代，
> 见 docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md §5。
```

提交：

```bash
git add docs/spec/stage-b-deviations.md docs/bug-ledger.md docs/superpowers/plans/2026-10-23-touch-native-undo-plan.md
git commit -F - <<'EOF'
docs(docs): 登记 STAGE-B-008（点地规则与反馈，计划 2 部分）；BUG-044 余项收口；touch-native-undo 点墙规则原地更正

动手前三问：
1. 层：文档（偏离登记 / 台账 / 旧计划更正）。
2. 消费者：后续计划 3 与复核者。
3. 证据：各条指向 Task 1–12 提交与测试名。
EOF
```

- [ ] **Step 3: 全量门禁 + ratchet**

Run: `python3 gates/cli.py status` 记下既有 SKIP / TOLERATED；`python3 gates/cli.py all`（约 20 分钟，后台跑，日志写 `target/verify/gates_all_plan2.log`）。
Expected: 无 FAIL；SKIP / TOLERATED 与 status 一致（逐项核对，不得新增）。
Run: `python3 gates/cli.py ratchet-update && git diff --stat data/status/baselines/`
Expected: 只有测试计数变化；total 从 777 增加（逐 crate 核对与各任务新增条数一致：T1 1、T2 11、T3 1、T4 2、T5 8、T6 5、T7 4、T8 2、T9 1、T10 undo_ui 1、T11 tap_routing 4；touch.rs / input.rs / native_parse / Windows t3 条数不变）。

```bash
git add data/status/baselines/
git commit -F - <<'EOF'
gate(baselines): ratchet 入账（STAGE-B-008 计划 2 点地规则新增测试；total 777→<实际>）

动手前三问：
1. 层：门禁基线（ratchet）。
2. 消费者：gates/cli.py test-ratchet。
3. 证据：gates all 无回退（target/verify/gates_all_plan2.log）；新增测试见各任务提交。
EOF
```

- [ ] **Step 4: 实机冒烟（Xiaomi 14，`c362a15b`）**

只做冒烟，不替代宿主测试。**只用 `c362a15b`**。注入前确认用户没在操作手机（`/tmp/jjxy_guard.sh`；检测到人为触摸即停止并询问）。
先备份用户数据（`files/` → `target/verify/device_plan2/files_before_plan2.tar`），测完恢复并逐文件 md5 比对。

构建与安装（先 touch，防 aarch64 不重编）：

```bash
touch crates/game-core/src/*.rs shell/common/src/*.rs shell/android/src/*.rs
CARGO_BUILD_JOBS=8 cargo ndk -t arm64-v8a -P 26 -o android/app/src/main/jniLibs build --offline --release -p game-android
ANDROID_HOME=$HOME/Android/Sdk /tmp/gradle-8.5/bin/gradle -p android assembleDebug --offline -q
adb -s c362a15b install -r android/app/build/outputs/apk/debug/app-debug.apk
```

逐项操作并截图（`/tmp/jjxy_view.sh <name>` → `target/verify/device_plan2/<name>.jpg`），结果写 `target/verify/plan2_device.md`：
1. 点远处可达地面：走过去，途中目标格有金框，到达后消失。
2. 点远处墙：走到墙边后朝墙转向，无多余撤销步（撤销一次即回起点）。
3. 点被墙隔开的地面：停在最近格，目标格红框加叉约 0.6 s。
4. 点一只怪：出现闪烁框、人物不动；再点：走过去开战。持火眼金睛的存档另测框旁数字（无此存档则记"未测"）。
5. 寻路途中点另一格：走完当前格后改道。
6. 右上隐形撤销区：空闲时撤销照旧可用。
7. 炼丹炉（有炉的层）：点炉，走到正下方后打开炉页。
设备坐标：屏幕 X = 24 + 3·逻辑 x、Y = 134 + 32 + 3·逻辑 y；先截图核对画布位置再注入。需要特定楼层时用 `/tmp/jjxy_preset.sh` 写预置存档（已备份前提下）。

- [ ] **Step 5: 收尾**

中文简报：每项通过 / 不通过 + 截图路径；不通过项先在宿主复现再改（回到对应任务）。`git status` 确认无遗留探针文件，询问用户是否 push。

---

## 附录 A：每任务对抗复核提示词模板

派 `general-task-execution` 子代理，提示词：

```text
你是独立对抗复核者，目标是推翻本次改动的正确性。仓库 /home/peng/workspace/xiyou_remake（纪律见 AGENTS.md：证据 A/B/C、classic 预设须与参考版 Java 一致、门禁不得弱化）。
任务：<任务名，如 T5 点地入口 tap_cell>。改动 = `git diff`（未提交）。设计依据：docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md §5 / §6（及本任务涉及的 §2.1 机制条目）；
计划：docs/superpowers/plans/2026-10-30-dual-control-core-tap-rules.md 的 Task <N>（含"复核重点"）。
只读：不得修改源码或文档；唯一可写 target/verify/review_plan2_T<N>.md。可跑 cargo test --offline 与只读命令；临时探针若放进源码树，跑完必须删除并在记录中说明。不要提交，不要连 adb。
请核查：① 实现是否覆盖 spec 对应条目的全部情形（逐条构造反例，含真实楼层数据探针）；② 是否触达 classic（找出所有调用点，说明为何 classic 按键链 / oracle 回放 / 帧夹具不经过，或给出会经过的路径）；
③ 测试是否会在实现被撤销时变红（做变异）、期望值是否独立于被测代码；④ 有无引入新问题（相邻状态机、撤销、BUG-038/039/040/043/044/045/047/048 的既有钉、debug 构建 transition_to 断言）。
输出：意见表（编号 | 严重度 Critical/Major/Minor | 意见 | 证据 文件:行 / 命令输出 | 建议修法），以及"已确认无问题"清单。最终回复用中文简要列出全部意见。
```
