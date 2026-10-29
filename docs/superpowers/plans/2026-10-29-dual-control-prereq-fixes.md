# 双操控方案前置缺陷修复 Implementation Plan（计划 1 / 3）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修掉双操控方案 spec（`docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md` §2.1）列出的前置缺陷 P0 / P1 / P3 / P4 / P5：BUG-047、BUG-044、BUG-045、BUG-048（新登记，原 BUG-043 复核 R4）、BUG-046。

**Architecture:** 前四项都是 Stage B 点地寻路 / 撤销链的缺陷，修在 `game-core`（scene.rs / entity.rs / undo_journal.rs）与 `game-shell-common`（undo_ui.rs），只在点地寻路占用 dp3 或路段开放时生效，classic 面零变化。BUG-046 是 classic 移植缺口，补一行原版提示框，并用 Java 参照帧夹具（`tools/frames_fixture.py gen`）做对拍。

**Tech Stack:** Rust（cargo test --offline）、Python 3（gates / frames_fixture）、OpenJDK 21 + Xvfb（Java 参照帧）。

**计划拆分**（spec 体量大，分三份，每份独立可测）：
- 计划 1（本件）：前置缺陷 P0 / P1 / P3 / P4 / P5。
- 计划 1b：BUG-037 筋斗云补移植（P2，classic 收敛 + Java 对拍；沿用 10-28 复核 §6 的范围）。
- 计划 2：core 点地策略（安全通行 BFS、Map wp0 撞击、排队、怪物选中、反馈标记、kind 4 src 0 点空白）。
- 计划 3：功能查询（含 P6 快照接口）+ 工具栏 / 面板标注 + 触屏布局 + 手势 + 设置 + 提示卡 / 帮助。

计划 2 / 3 在本计划执行完后再写——它们要基于本计划改完后的代码行号与接口。

**通用约定（每个任务都适用）：**
- 纪律：`AGENTS.md`。commit message 正文必须含"动手前三问"（层 / 消费者 / 证据）与 `Evidence:` / `Confidence:` 行。
- 测试命令（全量）：`cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle`。
- 每个任务 GREEN 后、commit 前：派一个独立复核子代理对本任务 diff 做对抗复核（提示词模板见文末附录 A），复核记录写 `target/verify/review_plan1_T<N>.md`；认同的意见修完再提交。
- commit 由 pre-commit 自动跑 `python3 gates/cli.py check`；任一 FAIL 不提交、不 `--no-verify`。
- 测试数增加会让 `test-ratchet` 提示可上调基线；基线更新统一放到 Task 6 单独 `gate(baselines)` 提交。

---

### Task 1: BUG-047 探步 `probe_step` 先写朝向

**Files:**
- Modify: `crates/game-core/src/scene.rs:1942-1948`（`probe_step`）
- Test: `crates/game-core/src/scene.rs`，模块 `t2_walkto_route_tests`（约 10566 行起，`run_until_settled` 所在模块）末尾追加
- Modify: `docs/bug-ledger.md`（BUG-047 节）

背景：`probe_step(dir)` 用 `dir` 判目标格，却不写 `self.facing`；放行后 Map wp1 臂按旧 `facing` 位移（`scene.rs:4880-4905`），可穿墙。方向键路径先 `self.facing = facing` 再 `try_step`（`scene.rs:5150-5152`）。夹具 `scene_test_support::scene`：13×13，只有走廊 y = `ROW`(3)、x ∈ 1..=11 可走，其余是墙。方向码：0 下、1 上、2 右、3 左。

- [ ] **Step 1: 写失败测试**

在 `mod t2_walkto_route_tests` 末尾（模块右花括号前）追加：

```rust
    /// 【BUG-047】探步按探步方向走，不按旧朝向走：{旧朝向 × 4} × {探步方向 × 4}。
    /// 走廊 y=ROW 横向可走、上下是墙 ⇒ 右探到 (6,ROW)、左探到 (4,ROW)、上下探原地不动；朝向 = 探步方向。
    #[test]
    fn bug047_probe_step_moves_in_probe_direction_for_every_old_facing() {
        for old in 0..4 {
            for dir in 0u8..4 {
                let mut sc = scene((5, ROW), &[]);
                let wt = sc.width_table.clone();
                sc.facing = old;
                let _ = sc.probe_step(dir);
                for fc in 0..20 {
                    sc.tick(fc, &wt);
                }
                let want = match dir {
                    2 => (6, ROW),
                    3 => (4, ROW),
                    _ => (5, ROW),
                };
                assert_eq!(
                    (sc.player_cell_x, sc.player_cell_y),
                    want,
                    "旧朝向 {old}、探步方向 {dir}：终点须是探步方向邻格或原地"
                );
                assert_eq!(
                    (sc.view.player_px, sc.view.player_py),
                    (want.0 << 5, want.1 << 5),
                    "旧朝向 {old}、探步方向 {dir}：像素位置与格一致（不得走进墙格半途）"
                );
                assert_eq!(sc.facing, dir as i32, "旧朝向 {old}、探步方向 {dir}：朝向 = 探步方向");
            }
        }
    }

    /// 【BUG-047】炼丹炉只在朝上走入时打开（scene.rs:5326-5338 判 facing == 1）：
    /// 旧朝向为上、向右横向探步撞炉，不得打开炼丹炉页，玩家不动。
    #[test]
    fn bug047_sideways_probe_into_alchemy_with_old_facing_up_does_not_open_it() {
        let mut sc = scene((5, ROW), &[(9, 6, ROW)]);
        let wt = sc.width_table.clone();
        sc.facing = 1;
        let _ = sc.probe_step(2);
        for fc in 0..20 {
            sc.tick(fc, &wt);
        }
        assert_ne!(sc.game_mode, GameMode::Alchemy, "横向撞炉不进炼丹炉");
        assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "被炉挡住，原地");
    }
```

- [ ] **Step 2: 运行，确认失败**

Run: `cargo test --offline -p game-core --lib bug047 -- --nocapture`
Expected: 两个测试 FAIL。第一个在"旧朝向 1、探步方向 2"一类组合上报终点或像素不符（玩家按旧朝向走进墙）；第二个报"横向撞炉不进炼丹炉"。

- [ ] **Step 3: 最小实现**

`crates/game-core/src/scene.rs` 的 `probe_step`（1942 行附近）：

```rust
    pub fn probe_step(&mut self, dir: u8) -> bool {
        if !self.field_tap_allowed() {
            return false;
        }
        self.interrupt_route();
        // 【BUG-047】先置朝向再撞（与 handle_field_input 同式，scene.rs:5150-5152）：
        // 放行后 Map wp1 臂按 self.facing 位移；不写则按旧朝向走出一整格、且该方向不做碰撞（可穿墙）。
        self.facing = dir as i32;
        self.try_step(dir as i32)
    }
```

- [ ] **Step 4: 运行，确认通过**

Run: `cargo test --offline -p game-core --lib bug047`
Expected: 2 passed。

再跑变异确认测试有效：临时删掉 `self.facing = dir as i32;` 一行，重跑同命令，Expected FAIL；恢复该行。

- [ ] **Step 5: 全量测试**

Run: `cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c`
Expected: 全部 `test result: ok`，无 FAILED。若 `shell/android/tests/tap_routing.rs` 或 `shell/common/src/touch.rs` 中有测试依赖"探步按旧朝向走"的旧行为而变红：读失败信息，确认它断言的是 BUG-047 的错误行为，再按"探步方向"改期望；不能确认就停下报告。

- [ ] **Step 6: 更新 bug 台账**

`docs/bug-ledger.md` 的 BUG-047 节：标题末尾 `未修）` 改为 `已修复）`；`- **Stage B 修复方案**：` 行之后追加一行：

```markdown
- **修复**（2026-10-29）：`probe_step` 在 `try_step` 前置 `self.facing = dir`。宿主钉：core `bug047_probe_step_moves_in_probe_direction_for_every_old_facing`
  （16 组合）、`bug047_sideways_probe_into_alchemy_with_old_facing_up_does_not_open_it`；删去该行的变异被杀。
```

- [ ] **Step 7: 对抗复核**

按附录 A 派复核子代理（任务名 T1 BUG-047，diff = `git diff`），记录写 `target/verify/review_plan1_T1.md`。认同的意见修完回到 Step 4。

- [ ] **Step 8: 提交**

```bash
git add crates/game-core/src/scene.rs docs/bug-ledger.md
git commit -F - <<'EOF'
fix(core): BUG-047 探步 probe_step 先写朝向——旧朝向位移可穿墙

probe_step 用探步方向判目标格，放行后 Map wp1 却按旧 facing 位移；现改为先置 facing 再 try_step，
与方向键 handle_field_input 同式。classic 不经过 probe_step（Stage B 点地 / 滑动 / Windows 点击入口）。

动手前三问：
1. 层：Rust 移植层（Stage B 新增入口 probe_step 的用法错误），非原版行为。
2. 消费者：Android scene_tap / Windows pointer_tap 的墙格探步、Dispatcher::swipe。
3. 证据：A 级——宿主 RED（16 组合中旧朝向 ≠ 探步方向的组合玩家按旧朝向走进墙）→ GREEN；删行变异被杀。
Evidence: core bug047_* 两条 RED→GREEN；scene.rs:1942-1948 无 facing 写入（修复前）。
Confidence: 90
EOF
```

---

### Task 2: BUG-044 点地寻路经 dp3 wp0 弹出的步要判碰撞结果

**Files:**
- Modify: `crates/game-core/src/scene.rs:2901-2907`（`tick_dialog` 的 dp3 wp0 臂）
- Test: `crates/game-core/src/scene.rs`，模块 `t2_walkto_route_tests` 末尾追加
- Modify: `docs/bug-ledger.md`（BUG-044 节）

背景：dp3 wp0 臂先置 `walk_phase = 1` 再 `try_step`、不看返回值；`try_step` 只在"有浮层 ∧ wp==1"时复位相位（`scene.rs:5156-5177`）。点地寻路（`walk_to`）的路线由 `find_path` 规划，会经过有实体槽的格（炉、隐形墙、怪物），首步撞上不弹浮层的阻挡时玩家照样走进去。其余步走 `advance_script_walk`（`scene.rs:5953-5971`），撞格即清步，正常。经典 MOV 不开路段（`walkto_owns_dp3()` 恒假），本修复不触达。

夹具中三种首步阻挡：炼丹炉 9（横向撞，`facing == 1` 才放行）、隐形墙 12（载入即不可见，撞后显形为 11 并阻挡）、打不过的怪 70（hp=1；BUG-046 移植前无浮层）。

- [ ] **Step 1: 写失败测试**

在 `mod t2_walkto_route_tests` 末尾追加：

```rust
    /// 【BUG-044】点地寻路首步撞上不弹浮层的阻挡：玩家不得进入该格，寻路收尾回 Map。
    /// find_path 经实体槽规划路线（walk_to(7,ROW) 自 (5,ROW) 起两步，首步即 (6,ROW)）。
    #[test]
    fn bug044_tap_walk_first_step_into_blocker_stays_put() {
        let cases: &[(u8, &str)] = &[(9, "炼丹炉（横向撞）"), (12, "隐形墙"), (70, "打不过的怪")];
        for &(t, label) in cases {
            let mut sc = scene((5, ROW), &[(t, 6, ROW)]);
            if t == 70 {
                sc.player.hp = 1;
            }
            let plan = sc.walk_to(7, ROW);
            assert!(!plan.steps.is_empty(), "{label}：前置——find_path 经实体槽规划出路线");
            run_until_settled(&mut sc, 300);
            assert_eq!((sc.player_cell_x, sc.player_cell_y), (5, ROW), "{label}：首步被挡，玩家留在原格");
            assert_eq!((sc.view.player_px, sc.view.player_py), (5 << 5, ROW << 5), "{label}：像素位置未动");
            assert_eq!(sc.game_mode, GameMode::Map, "{label}：寻路收尾回 Map");
            assert!(!sc.route.is_open(), "{label}：路段已收");
            assert_ne!(sc.walk_phase, 5, "{label}：未开战");
        }
    }

    /// 【BUG-044】可胜的怪仍照常开战（返回 false 但 wp=5 的分支不得被当成"被挡"清掉）。
    #[test]
    fn bug044_tap_walk_first_step_into_winnable_enemy_still_fights() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        for fc in 0..40 {
            sc.tick(fc, &wt);
            saw_battle |= sc.walk_phase == 5;
        }
        assert!(saw_battle, "首步撞可胜的怪 ⇒ 开战（walk_phase 5）");
    }
```

- [ ] **Step 2: 运行，确认失败**

Run: `cargo test --offline -p game-core --lib bug044 -- --nocapture`
Expected: `bug044_tap_walk_first_step_into_blocker_stays_put` FAIL（第一个用例"炼丹炉（横向撞）：首步被挡，玩家留在原格"，实际玩家在 (6,3) 或 (7,3)）；`bug044_tap_walk_first_step_into_winnable_enemy_still_fights` PASS（现行为正确，作为回归钉）。

- [ ] **Step 3: 最小实现**

`crates/game-core/src/scene.rs` dp3 wp0 臂（2901 行附近）改为：

```rust
                        0 => {
                            if self.walk_step_count > 0 {
                                self.walk_step_count -= 1;
                                self.facing =
                                    self.walk_path_buffer[self.walk_step_count as usize] as i32;
                                self.walk_phase = 1;
                                let ok = self.try_step(self.facing);
                                // 【BUG-044】点地寻路专用：被挡且未开战（wp 仍为 1）⇒ 清步、归 wp0，下拍走 finish_walkto。
                                // 原版 MOV 消费机不判返回值（a.java:4080-4084，脚本路线由剧本保证可走）；
                                // 经典脚本行走不占用 dp3（walkto_owns_dp3 恒假），此分支不触达 classic。
                                if !ok && self.walk_phase == 1 && self.walkto_owns_dp3() {
                                    self.walk_step_count = 0;
                                    self.walk_phase = 0;
                                }
                            } else {
```

（`else` 分支及其后保持原样。）

- [ ] **Step 4: 运行，确认通过**

Run: `cargo test --offline -p game-core --lib bug044`
Expected: 2 passed。

变异：把条件里的 `&& self.walkto_owns_dp3()` 去掉，跑 `cargo test --offline -p game-oracle` ——若有 classic 回放变红，说明该门控必要（记录在复核记录里）；恢复。再把 `self.walk_phase = 0;` 删掉，重跑 `bug044`，Expected FAIL；恢复。

- [ ] **Step 5: 全量测试 + classic 门禁**

Run: `cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c`
Expected: 全绿。

Run: `python3 gates/cli.py trace`
Expected: 无回退（classic 不经过新分支）。

- [ ] **Step 6: 更新 bug 台账**

`docs/bug-ledger.md` BUG-044 节：标题 `未修）` → `已修复）`；在最后一个更正块之后追加：

```markdown
- **修复**（2026-10-29）：dp3 wp0 臂加点地寻路专用分支——`try_step` 返回 false 且 wp 仍为 1（未开战、无浮层复位）⇒ 清步、归 wp0，
  下拍 `finish_walkto` 回 Map。宿主钉：core `bug044_tap_walk_first_step_into_blocker_stays_put`（炉 / 隐形墙 / 打不过的怪）、
  `bug044_tap_walk_first_step_into_winnable_enemy_still_fights`（回归钉）。陷阱扣血后 / wp4 后的下一步同经此臂，由同一分支覆盖；
  撞击目标的语义（点门 / 怪 / 炉）在 STAGE-B-008 计划 2 改为 Map wp0 执行。
```

- [ ] **Step 7: 对抗复核**

按附录 A 派复核子代理（任务名 T2 BUG-044），重点请复核者构造"陷阱扣血后下一步""wp4 后下一步"两种情形验证分支覆盖。记录 `target/verify/review_plan1_T2.md`。

- [ ] **Step 8: 提交**

```bash
git add crates/game-core/src/scene.rs docs/bug-ledger.md
git commit -F - <<'EOF'
fix(core): BUG-044 点地寻路经 dp3 wp0 弹出的步不判碰撞——首步可站上炉 / 穿隐形墙 / 站上打不过的怪

dp3 wp0 臂加点地寻路专用分支：try_step 被挡且未开战 ⇒ 清步归 wp0，下拍 finish_walkto。
经典 MOV 不占用 dp3（walkto_owns_dp3 恒假），trace 零变化。

动手前三问：
1. 层：Rust 移植层（Stage B 点地寻路借用 dp3 消费机的用法错误）；原版 MOV 消费机不动。
2. 消费者：walk_to 发起的 dp3 行走（Android / Windows 点地）。
3. 证据：A 级——宿主 RED（炉 / 隐形墙 / 打不过的怪三例玩家进入阻挡格）→ GREEN；gates trace 无回退。
Evidence: core bug044_* RED→GREEN；scene.rs:2901-2907 修复前不判 try_step 返回值；a.java:4080-4084 原版同式。
Confidence: 85
EOF
```

---

### Task 3: BUG-045 撤销战斗逆放 + 撤销日志清栈

**Files:**
- Modify: `crates/game-core/src/entity.rs:18`（`Entity` derive）、`entity.rs:535` 之后新增 `reattach_to_cell`
- Modify: `crates/game-core/src/undo_journal.rs`（`EnemyRestore`、`CombatStroke` 加实体快照字段；`undo` 传递）
- Modify: `crates/game-core/src/scene.rs:524`（`battle_undo_snap` 类型）、`:5404-5422`（开战记快照 + `begin_combat`）、`:5672-5690`（入栈）、
  `:1754`（`load_level_script` 清栈）、`:1125`（`load_floor_data` 清栈）、`:3853`（存档清栈）
- Modify: `shell/common/src/undo_ui.rs:36-46`（`revive_enemy`）及其测试
- Modify（机械补字段）：`crates/game-core/tests/walk_route.rs:165,272`、`crates/game-core/tests/undo_journal.rs:82,126,195,233`
- Test: `shell/common/src/undo_ui.rs` 测试模块；`crates/game-core/src/scene.rs` 模块 `t2_walkto_route_tests`
- Modify: `docs/bug-ledger.md`（BUG-045 节）

背景（A 级，复核探针 B / D）：
1. 怪物死亡动画后已摘格槽（`removed = true`，`entity.rs:535-541`）；`revive_enemy` 只写 type / state / visible，不恢复 removed、不重登格槽 ⇒ 撤销后怪不画、不挡路。
2. `revive_enemy` 按实体槽号写 `enemies.hp`，该表按 `type − 41` 索引 ⇒ 写坏全局怪物属性表。
3. 生产代码从不调用 `begin_combat` / `clear` ⇒ 战斗中可撤、换层 / 存档 / 剧情后仍可撤。

修法：开战时把整个 `Entity`（Copy）存进快照，撤销时整格还原 + 按需重登格槽；不再写 `enemies.hp`；路段开放的开战调 `begin_combat`；
换层（`load_floor_data`）/ 存档 / 剧情（`load_level_script`）调 `clear`。classic 不读撤销日志（state_dump 不导出），零变化。

- [ ] **Step 1: 写失败测试（shell-common，生产路径）**

`shell/common/src/undo_ui.rs` 测试模块：把现有 `fn scene() -> GameScene` 改为带参数的 `scene_with`，并保留 `scene()`：

```rust
    fn fixture_numbers(name: &str) -> Vec<i32> {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::fs::read_to_string(repo.join("crates/game-core/tests/fixtures").join(name))
            .unwrap()
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect()
    }

    /// 轻量 scene（scene_test_support 同构：合成图像 + 仓库夹具 maplv1 + 真实数值表）。
    fn scene_with(player: (i32, i32), ents: &[(u8, i32, i32)]) -> GameScene {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fix = repo.join("crates/game-core/tests/fixtures");
        let width_table = fixture_numbers("type_width_px.txt");
        let walk = fixture_numbers("tile_walkability.txt");
        let maplv = std::fs::read(fix.join("maplv1.bin")).unwrap();
        let images = SceneImages {
            backdrop: solid(77, 320),
            shadow: solid(24, 24),
            actor: solid(24, 24),
            actor_face: solid(24, 24),
            ui: many(25),
            stair_icons: (solid(24, 24), solid(24, 24)),
            open_anim: solid(24, 24),
            opened_door: solid(24, 24),
            entity: SceneImages::map_entity_images(&many(13), &many(23), &many(10), &many(20), &many(20)),
            popup: game_core::popup::PopupImages {
                digit_strips: [solid(24, 24), solid(24, 24), solid(24, 24)],
                icon_strip: solid(24, 24),
                battle_digits: solid(24, 24),
            },
            spark_sheet: solid(24, 24),
            corner_sprites: solid(24, 24),
            map211: None,
            map210: solid(24, 24),
            menu_strip: solid(24, 24),
            battle_icon: solid(24, 24),
            battle_strip: solid(24, 24),
            faces: many(12),
            ending: solid(24, 24),
            xtq: many(6),
        };
        let recs: Vec<(u8, u16, u16, i16, bool)> =
            ents.iter().map(|&(t, x, y)| (t, (x << 5) as u16, (y << 5) as u16, 0, true)).collect();
        let floors: Vec<Option<FloorAssets>> = (0..56).map(|_| None).collect();
        let mut sc = GameScene::load_floor(&maplv, player.0 << 5, player.1 << 5, &recs, &width_table, &walk, images, floors)
            .unwrap();
        let tables = game_core::combat::CombatTables {
            enemy_base_hp: fixture_numbers("enemybasehp.txt"),
            enemy_base_atk: fixture_numbers("enemybaseatk.txt"),
            enemy_base_def: fixture_numbers("enemybasedef.txt"),
            difficulty_multipliers: fixture_numbers("difficulty_multipliers.txt"),
        };
        sc.enemies = game_core::combat::scale_enemy_stats(&tables, 1);
        sc.enemy_base_hp = tables.enemy_base_hp.clone();
        sc.enemy_base_gold = fixture_numbers("enemybasegold.txt");
        sc
    }

    fn scene() -> GameScene {
        scene_with((3, 10), &[])
    }
```

（删除旧 `scene()` 函数体；`solid` / `many` 保持不动。）再追加测试：

```rust
    /// 【BUG-045】点地撞怪开战 → 胜 → 撤销：怪在原格复活（可见、在格槽里、再撞即开战），
    /// 全局怪物属性表不变，玩家血 / 金回到战前。maplv1 教程层 (2,10)(3,10) 是可走格（教程箭头格）。
    #[test]
    fn bug045_undo_after_tap_battle_revives_enemy_in_place() {
        use game_core::enums::GameMode;
        use game_core::undo_journal::Stroke;
        let mut sc = scene_with((2, 10), &[(41, 3, 10)]);
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 500;
        sc.player.gold = 7;
        let hp_table_before = sc.enemies.hp.clone();
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(3, 10);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 3000 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 && sc.game_mode == GameMode::Map && sc.walk_phase == 0 && !sc.route.is_open() {
                break;
            }
        }
        assert!(saw_battle, "前置：开战");
        for _ in 0..60 {
            sc.tick(fc, &wt); // 死亡动画 / 摘槽走完
            fc += 1;
        }
        assert!(sc.player.gold > 7, "前置：胜利得金");
        let mut undone_combat = false;
        for _ in 0..6 {
            match undo_last(&mut sc) {
                UndoOutcome::Undone(Stroke::Combat(_)) => {
                    undone_combat = true;
                    break;
                }
                UndoOutcome::Undone(_) => continue,
                other => panic!("撤销栈提前结束：{other:?}"),
            }
        }
        assert!(undone_combat, "栈里有战斗笔");
        let e = game_core::entity::cell_entity(&sc.grid, &sc.entities, 3, 10, 41);
        assert!(e >= 0, "怪物回到 (3,10) 的格槽");
        let ent = &sc.entities.items[e as usize];
        assert!(!ent.removed && ent.visible && ent.state == 0, "怪物活着且可见：{:?}", (ent.removed, ent.visible, ent.state));
        assert_eq!(sc.enemies.hp, hp_table_before, "全局怪物属性表不变");
        assert_eq!((sc.player.hp, sc.player.gold), (500, 7), "玩家血 / 金回到战前");
        // 再撞同格必须开战
        sc.player_cell_x = 2;
        sc.player_cell_y = 10;
        sc.view.player_px = 2 << 5;
        sc.view.player_py = 10 << 5;
        sc.press_key(-4);
        let mut fought_again = false;
        for _ in 0..10 {
            sc.tick(fc, &wt);
            fc += 1;
            fought_again |= sc.walk_phase == 5;
        }
        sc.release_key();
        assert!(fought_again, "撤销后再撞同格必须开战");
    }

    /// 【BUG-045】路段开放的战斗进行中不可撤（begin_combat 生产调用）。
    #[test]
    fn bug045_undo_is_blocked_mid_tap_battle() {
        let mut sc = scene_with((2, 10), &[(41, 3, 10)]);
        sc.player.atk = 1; // 打得过但要很多回合（预测掉血 < hp）
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(3, 10);
        let mut fc = 0;
        while sc.walk_phase != 5 && fc < 40 {
            sc.tick(fc, &wt);
            fc += 1;
        }
        assert_eq!(sc.walk_phase, 5, "前置：战斗中");
        assert_eq!(undo_last(&mut sc), UndoOutcome::Blocked, "战斗中撤销被拒");
    }
```

注：`bug045_undo_is_blocked_mid_tap_battle` 若因 atk=1 导致预测"打不过"而不开战，把 `sc.player.atk` 调到刚好大于怪 41 的防御（读 `fixture_numbers("enemybasedef.txt")[0] + 1`），保持多回合即可。

- [ ] **Step 2: 写失败测试（core，清栈）**

`crates/game-core/src/scene.rs` 模块 `t2_walkto_route_tests` 末尾追加：

```rust
    /// 【BUG-045④】换层 / 存档 / 剧情接管后撤销栈清空（undo_journal.rs 清栈边界契约的生产接线）。
    #[test]
    fn bug045_journal_clears_on_floor_change_save_and_script() {
        use crate::undo_journal::MoveStroke;
        let stroke = MoveStroke { from: (1, ROW), to: (2, ROW) };
        // 换层（load_floor_data = changeFloor / LAY 的原地重载）
        let mut sc = scene((5, ROW), &[]);
        sc.undo.push_move(stroke);
        sc.load_floor_data(&super::scene_test_support::maplv(), 5 << 5, ROW << 5, &[], 2).unwrap();
        assert!(sc.undo.is_empty(), "换层清栈");
        // 剧情接管
        let mut sc = scene((5, ROW), &[]);
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string(); 2];
        sc.undo.push_move(stroke);
        sc.load_level_script(0);
        assert!(sc.undo.is_empty(), "剧情清栈");
        // 存档（mode 7 确认键）
        let mut sc = scene((5, ROW), &[]);
        let wt = sc.width_table.clone();
        sc.undo.push_move(stroke);
        sc.menu_return_mode = GameMode::InGameMenu;
        sc.game_mode = GameMode::SaveSlot;
        sc.enter_menu7();
        sc.press_key(-5);
        sc.tick(0, &wt);
        sc.release_key();
        assert!(sc.slot.valid[sc.slot.cursor as usize], "前置：存档写入");
        assert!(sc.undo.is_empty(), "存档清栈");
    }
```

若 `menu_return_mode` / `enter_menu7` / `slot` 的可见性导致编译不过：它们在同一文件内，测试模块 `use super::*;` 可访问私有项；若字段名不同，按 `tick_menu7`（`scene.rs:3851`）实际使用的字段名改。

- [ ] **Step 3: 运行，确认失败**

Run: `cargo test --offline -p game-shell-common --lib bug045 -- --nocapture; cargo test --offline -p game-core --lib bug045 -- --nocapture`
Expected：
- `bug045_undo_after_tap_battle_revives_enemy_in_place` FAIL："怪物回到 (3,10) 的格槽"或"全局怪物属性表不变"。
- `bug045_undo_is_blocked_mid_tap_battle` FAIL：实际 `Undone(..)`。
- `bug045_journal_clears_on_floor_change_save_and_script` FAIL："换层清栈"。

- [ ] **Step 4: 实现——实体快照与重登格槽（entity.rs）**

`crates/game-core/src/entity.rs:18`：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity {
```

在 `detach_from_cell`（535 行）之后新增：

```rust
/// 【BUG-045 · Stage B 撤销】`detach_from_cell` 的逆：removed=false + 重新登记入实体当前像素所在格的槽表
/// （格类型为 0 时分配新类型，与 spawn 槽登记同式）。只摘了主格的槽，只重登主格（与 detach 对称）。
pub fn reattach_to_cell(grid: &mut CellGrid, table: &mut EntityTable, idx: usize) {
    table.items[idx].removed = false;
    let cy = (table.items[idx].py >> 5) as usize;
    let cx = (table.items[idx].px >> 5) as usize;
    let mut ct = grid.cell_type[cy][cx];
    if ct == 0 {
        grid.next_cell_type += 1;
        ct = grid.next_cell_type;
        grid.cell_type[cy][cx] = ct;
    }
    grid.slot_push(ct as usize, idx + 1);
}
```

- [ ] **Step 5: 实现——撤销笔带实体快照（undo_journal.rs）**

`EnemyRestore` 加字段：

```rust
pub struct EnemyRestore {
    /// 敌槽位（entity 表下标语义，T2 挂点定标）
    pub slot: usize,
    /// 敌型（复活后型不变——变形/召唤语义 v1 不入回撤域）
    pub type_id: i32,
    /// 复活原血
    pub hp: i32,
    /// 【BUG-045】开战前的实体整格快照（位置 / 精灵 / 动画 / state=0 / visible）——复活按此还原
    pub entity: crate::entity::Entity,
}
```

`CombatStroke` 加字段（放在 `items_before` 之后）：

```rust
    /// 【BUG-045】开战前的实体整格快照
    pub enemy_snapshot: crate::entity::Entity,
```

`undo()` 的 `Stroke::Combat(c)` 臂：

```rust
                target.revive_enemy(EnemyRestore {
                    slot: c.enemy_slot,
                    type_id: c.enemy_type_id,
                    hp: c.enemy_hp_before,
                    entity: c.enemy_snapshot,
                });
```

机械补字段：`crates/game-core/tests/walk_route.rs:165,272`、`crates/game-core/tests/undo_journal.rs:82,126,195,233`、`shell/common/src/undo_ui.rs`
测试 `undo_blocked_mid_combat_and_empty` 里每个 `CombatStroke { ... }` 字面量加一行 `enemy_snapshot: Default::default(),`。

- [ ] **Step 6: 实现——开战快照、begin_combat、入栈（scene.rs）**

`scene.rs:524` 字段类型：

```rust
    battle_undo_snap: Option<(usize, i32, i32, crate::undo_journal::PlayerVitals, entity::Entity)>,
```

`scene.rs:5404` 起的可胜分支改为（`state = 3` 之前取快照；快照血取赋值后的值）：

```rust
                                if loss >= 0 && loss < self.player.hp {
                                    // 【BUG-045】撤销快照须在改 state 之前取（复活 = 还原到开战前的整格实体）
                                    let snapshot = self.entities.items[e];
                                    self.walk_phase = 5;
                                    self.entities.items[e].state = 3;
                                    self.battle_target = e as i32;
                                    // battleEnemyHp = enemyBaseHp[type-41]（**基础表**非缩放表）
                                    self.battle_enemy_hp = self.enemy_base_hp[(t - 41) as usize];
                                    // 【T2】撞敌=事件挂起（段收于实际终点=敌格，T1 挂点）
                                    // ——仅 WalkTo/探步寻路段记账；经典面 route 未开放=零写入
                                    if self.route.is_open() {
                                        self.route_event_suspend();
                                        // 【BUG-045③】战斗进行中不可撤（T1 契约的生产接线）
                                        self.undo.begin_combat();
                                        self.battle_undo_snap = Some((
                                            e,
                                            snapshot.type_,
                                            self.battle_enemy_hp,
                                            crate::undo_journal::PlayerVitals {
                                                hp: self.player.hp,
                                                gold: self.player.gold,
                                                exp: 0, // 本作无经验字段（PlayerCombat 面）——T1 通用字段恒 0
                                            },
                                            snapshot,
                                        ));
                                    }
                                    self.battle_active = true;
                                    self.battle_no_counter = true;
                                } else {
```

注意：原代码 `battle_enemy_hp` 赋值在 `route` 记账之后；移到之前不改变 classic 行为（classic 不进 `if self.route.is_open()`，赋值本身不变）。

`scene.rs:5672` 起的入栈改为解构 5 元组：

```rust
                    if let Some((slot, ety, hp_before, vitals_before, snapshot)) = self.battle_undo_snap.take()
                    {
                        let items_before = self
                            .player
                            .item_types
                            .iter()
                            .zip(self.player.item_uses.iter())
                            .take(self.player.item_stack_size as usize)
                            .map(|(t, u)| (*t as i32, *u as i32))
                            .collect();
                        self.route_combat_finished(crate::undo_journal::CombatStroke {
                            enemy_slot: slot,
                            enemy_type_id: ety,
                            enemy_hp_before: hp_before,
                            vitals_before,
                            items_before,
                            enemy_snapshot: snapshot,
                        });
                        let at = (self.player_cell_x, self.player_cell_y);
                        self.route.resume(at);
                    }
```

- [ ] **Step 7: 实现——清栈接线（scene.rs）**

`load_level_script`（1754 行）在 `self.interrupt_route();` 之后加：

```rust
        // 【BUG-045④】剧情推进不可撤（undo_journal 清栈边界：MOV / 脚本事件触发 = 清栈）
        self.undo.clear(crate::undo_journal::ClearReason::ScriptEvent);
```

`load_floor_data`（1125 行）函数体第一行加：

```rust
        // 【BUG-045④】换层清栈（旧层坐标的位移笔在新层无意义——复核探针 D：撤销把玩家传回上一层坐标）
        self.undo.clear(crate::undo_journal::ClearReason::FloorChange);
```

`tick_menu7` 存档分支（3853 行 `if self.game_mode == GameMode::SaveSlot {` 内第一行）加：

```rust
                    // 【BUG-045④】存档清栈（存档后不可撤回存档前的操作）
                    self.undo.clear(crate::undo_journal::ClearReason::Save);
```

- [ ] **Step 8: 实现——复活整格还原（undo_ui.rs）**

`shell/common/src/undo_ui.rs` 的 `revive_enemy`：

```rust
    fn revive_enemy(&mut self, e: EnemyRestore) {
        let sc = &mut *self.0;
        if e.slot >= sc.entities.items.len() {
            return;
        }
        // 【BUG-045①】死亡动画后已摘槽（removed=true）⇒ 还原后须重登格槽，否则不画、不挡路
        let detached = sc.entities.items[e.slot].removed;
        sc.entities.items[e.slot] = e.entity; // 开战前整格快照（type / 位置 / 精灵 / 动画 / state=0 / visible）
        if detached {
            game_core::entity::reattach_to_cell(&mut sc.grid, &mut sc.entities, e.slot);
        }
        // 【BUG-045②】不写 sc.enemies.hp：该表按 type−41 索引的全局属性表，不是实体血量
    }
```

同时把文件头注释里"敌复活=state 回 0 + type 还原"改为"敌复活=开战前整格快照还原 + 按需重登格槽（BUG-045）"。

- [ ] **Step 9: 运行，确认通过**

Run: `cargo test --offline -p game-shell-common --lib bug045; cargo test --offline -p game-core --lib bug045`
Expected: 3 passed。

变异（逐个做、逐个恢复）：
- 删掉 `reattach_to_cell` 调用 ⇒ `bug045_undo_after_tap_battle_revives_enemy_in_place` FAIL；
- 恢复 `enemies.hp` 写入 ⇒ 同上 FAIL（属性表断言）；
- 删掉 `begin_combat` ⇒ `bug045_undo_is_blocked_mid_tap_battle` FAIL；
- 删掉任一处 `clear` ⇒ `bug045_journal_clears_on_floor_change_save_and_script` FAIL。

- [ ] **Step 10: 全量测试 + classic 门禁**

Run: `cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c`
Expected: 全绿。若 `walk_route.rs` / `tap_routing.rs` 中有断言"换层 / 剧情后撤销栈仍有笔"的旧测试变红：它钉的是 BUG-045④ 的错误行为，按"清栈"改期望并在 commit 正文列出。

Run: `python3 gates/cli.py trace`
Expected: 无回退。

- [ ] **Step 11: 更新 bug 台账**

`docs/bug-ledger.md` BUG-045 节：标题 `未修）` → `已修复）`；追加：

```markdown
- **修复**（2026-10-29）：开战时把整格 `Entity` 存进撤销快照（改 state 之前取）；撤销按快照整格还原，已摘槽则 `entity::reattach_to_cell`
  重登主格；不再写 `enemies.hp`；路段开放的开战调 `begin_combat`（战斗中撤销 = Blocked）；`load_floor_data` / 存档 / `load_level_script`
  清栈。宿主钉：shell-common `bug045_undo_after_tap_battle_revives_enemy_in_place`（生产路径 walk_to → 战斗 → undo_last → 再撞即开战）、
  `bug045_undo_is_blocked_mid_tap_battle`；core `bug045_journal_clears_on_floor_change_save_and_script`。四个变异均被杀。
```

- [ ] **Step 12: 对抗复核**

按附录 A 派复核子代理（任务名 T3 BUG-045），重点：多格怪（型 69 宽 3 格）复活后的格槽、撤销后 `sort_by_y` 是否会改动实体下标、开战前快照是否含 `state == 3` 等中间态。记录 `target/verify/review_plan1_T3.md`。

- [ ] **Step 13: 提交**

```bash
git add crates/game-core/src/entity.rs crates/game-core/src/undo_journal.rs crates/game-core/src/scene.rs \
        crates/game-core/tests/walk_route.rs crates/game-core/tests/undo_journal.rs shell/common/src/undo_ui.rs docs/bug-ledger.md
git commit -F - <<'EOF'
fix(core): BUG-045 撤销战斗逆放损坏 + 撤销日志从不清栈

开战时存整格实体快照，撤销整格还原并重登格槽；不再按实体槽号写全局怪物属性表；路段开放的开战调 begin_combat；
换层 / 存档 / 剧情接管清栈。撤销是 Stage B 新增功能，classic 不读撤销日志（state_dump 不导出），零变化。

动手前三问：
1. 层：Rust 移植层（Stage B touch-native-undo 的逆放与清栈未接到生产路径）。
2. 消费者：shell-common undo_last（Android / Windows 撤销入口）。
3. 证据：A 级——生产路径宿主 RED（复活后不在格槽、属性表被写坏、战斗中可撤、换层后仍可撤）→ GREEN；四个变异被杀；gates trace 无回退。
Evidence: shell-common bug045_* / core bug045_journal_clears_* RED→GREEN；undo_ui.rs:36-46 修复前按 e.slot 写 enemies.hp。
Confidence: 85
EOF
```

---

### Task 4: BUG-048 击杀触发剧情后不得重开寻路段、不得补入战斗笔（原 BUG-043 复核 R4）

**Files:**
- Modify: `crates/game-core/src/scene.rs:5672-5690`（`tick_battle` 胜利尾的入栈 / 续走）
- Test: `crates/game-core/src/scene.rs`，模块 `t2_walkto_route_tests` 末尾追加
- Modify: `docs/bug-ledger.md`（新增 BUG-048 节；BUG-043 节"另案"行加指针）

背景：`tick_battle` 胜利尾先跑类型特判（69/70/71/72 会 `load_level_script`，剧情接管 ⇒ `interrupt_route` 收段 + Task 3 起清栈），
再无条件 `route_combat_finished` + `route.resume(at)`（`scene.rs:5691`）⇒ 剧情运行中寻路段被重开、清过的栈又被补入战斗笔（撤销可在剧情后复活 boss）。
判据：开战时路段被 `route_event_suspend` 挂起；剧情接管会把挂起解除（`RouteTracker::interrupt` 置 `suspended = false`，`pathfind.rs:420-423`）。
所以"胜利尾仍挂起"⇔"没有被剧情接管"。

前置：Task 3 已提交（`load_level_script` 清栈）。

- [ ] **Step 1: 写失败测试**

在 `mod t2_walkto_route_tests` 末尾追加：

```rust
    /// 【BUG-048】点地撞怪开战、击杀触发剧情（49 层型 70 → load_level_script(35)）：
    /// 剧情接管后寻路段不得被重开，撤销栈保持清空（不补入战斗笔）。
    #[test]
    fn bug048_victory_script_takeover_keeps_route_closed_and_journal_empty() {
        let mut sc = scene((5, ROW), &[(70, 6, ROW)]);
        sc.floor = 49;
        sc.current_floor = 49;
        sc.dialog_data.scripts = vec!["TAK_0_0 ".to_string(); 40];
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle, "前置：开战");
        assert!(sc.battle_target < 0, "前置：战斗结束");
        assert!(!sc.route.is_open(), "剧情接管后寻路段不得被重开");
        assert!(sc.undo.is_empty(), "剧情接管 ⇒ 撤销栈已清，不再补入战斗笔");
    }

    /// 【BUG-048】回归钉：无剧情钩子的击杀照常入战斗笔并续走（路段在胜利尾重开）。
    #[test]
    fn bug048_plain_victory_still_pushes_combat_and_resumes() {
        let mut sc = scene((5, ROW), &[(41, 6, ROW)]);
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        let wt = sc.width_table.clone();
        let _ = sc.walk_to(6, ROW);
        let mut saw_battle = false;
        let mut fc = 0;
        while fc < 600 {
            sc.tick(fc, &wt);
            fc += 1;
            saw_battle |= sc.walk_phase == 5;
            if saw_battle && sc.battle_target < 0 {
                break;
            }
        }
        assert!(saw_battle && sc.battle_target < 0, "前置：开战并结束");
        assert!(sc.route.is_open(), "无剧情：胜利尾续走（路段重开，restep 进敌格）");
        let has_combat = {
            let mut j = std::mem::take(&mut sc.undo);
            struct Nop;
            impl crate::undo_journal::UndoTarget for Nop {
                fn set_player_tile(&mut self, _: (i32, i32)) {}
                fn set_player_vitals(&mut self, _: crate::undo_journal::PlayerVitals) {}
                fn revive_enemy(&mut self, _: crate::undo_journal::EnemyRestore) {}
                fn set_item_count(&mut self, _: i32, _: i32) {}
                fn set_door_state(&mut self, _: i32, _: i32, _: crate::undo_journal::DoorState) {}
            }
            let mut found = false;
            while let crate::undo_journal::UndoOutcome::Undone(s) = j.undo(&mut Nop) {
                found |= matches!(s, crate::undo_journal::Stroke::Combat(_));
            }
            found
        };
        assert!(has_combat, "无剧情：栈里有战斗笔");
    }
```

- [ ] **Step 2: 运行，确认失败**

Run: `cargo test --offline -p game-core --lib bug048 -- --nocapture`
Expected: `bug048_victory_script_takeover_keeps_route_closed_and_journal_empty` FAIL（"剧情接管后寻路段不得被重开"）；`bug048_plain_victory_still_pushes_combat_and_resumes` PASS（回归钉）。

若第一个测试在"前置：开战"处失败：49 层型 70 的数值表可能使预测为打不过——把 `sc.player.atk` 再调大或改用 `sc.floor = 50` + 型 72（`load_level_script(65)`，脚本表长度相应改为 70）。

- [ ] **Step 3: 最小实现**

`scene.rs` 胜利尾（Task 3 改过的那段）改为：

```rust
                    // 【T2】战斗终结才入栈（T1 挂点契约）+挂起解除+续走新段
                    // 【BUG-048】仅当路段仍挂起（未被胜利特判里的剧情接管打断）才入栈、续走；
                    // 剧情接管时 load_level_script 已收段并清栈，补入战斗笔会让撤销在剧情后复活怪物，重开路段会截断剧情行走。
                    if self.route.is_suspended() {
                        if let Some((slot, ety, hp_before, vitals_before, snapshot)) = self.battle_undo_snap.take()
                        {
                            let items_before = self
                                .player
                                .item_types
                                .iter()
                                .zip(self.player.item_uses.iter())
                                .take(self.player.item_stack_size as usize)
                                .map(|(t, u)| (*t as i32, *u as i32))
                                .collect();
                            self.route_combat_finished(crate::undo_journal::CombatStroke {
                                enemy_slot: slot,
                                enemy_type_id: ety,
                                enemy_hp_before: hp_before,
                                vitals_before,
                                items_before,
                                enemy_snapshot: snapshot,
                            });
                            let at = (self.player_cell_x, self.player_cell_y);
                            self.route.resume(at);
                        }
                    } else {
                        self.battle_undo_snap = None;
                    }
```

- [ ] **Step 4: 运行，确认通过**

Run: `cargo test --offline -p game-core --lib bug048`
Expected: 2 passed。变异：把 `if self.route.is_suspended()` 改成 `if true`，Expected 第一个 FAIL；恢复。

- [ ] **Step 5: 全量测试 + classic 门禁**

Run: `cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c`
Expected: 全绿。
Run: `python3 gates/cli.py trace`
Expected: 无回退（classic 战斗不开路段，`battle_undo_snap` 恒 None）。

- [ ] **Step 6: 登记并标修复**

`docs/bug-ledger.md` 末尾追加：

```markdown

## BUG-048 点地撞怪击杀触发剧情后，寻路段被重开、撤销栈被补入战斗笔（Stage B 点地寻路 / 撤销，BUG-043 复核 R4，已修复）

> 【2026-10-29】BUG-043 对抗复核 R4 登记为另案；双操控 spec 复核 m9 列为前置。
- **现象**：点地寻路撞怪开战、胜利特判触发剧情（35 层型 69、49 层型 70/71、50 层型 72 → `load_level_script`）：剧情接管已收段并清栈，
  随后胜利尾无条件 `route_combat_finished` + `route.resume`（`scene.rs:5691`）⇒ 剧情运行中寻路段被重开（截断剧情 MOV 行走的风险，BUG-025 同型），
  撤销栈被补入战斗笔（剧情后可撤销复活 boss）。
- **根因**：Stage B T2 挂点只看 `battle_undo_snap` 是否存在，不看路段是否已被剧情接管。
- **Stage A 处理**：classic 不受影响（classic 战斗不开路段）。
- **修复**（2026-10-29）：胜利尾只在 `route.is_suspended()`（开战时挂起、未被剧情打断）时入栈并续走，否则丢弃快照。
  宿主钉：core `bug048_victory_script_takeover_keeps_route_closed_and_journal_empty`、`bug048_plain_victory_still_pushes_combat_and_resumes`（回归钉）。
Evidence: A 级——宿主 RED（剧情接管后 route.is_open() 为真）→ GREEN；`if true` 变异被杀。Confidence: 85
```

BUG-043 节"另案（复核 R4，修复前既有，未修）"一行末尾追加：` > 【2026-10-29】已登记为 BUG-048 并修复。`

- [ ] **Step 7: 对抗复核**

按附录 A 派复核子代理（任务名 T4 BUG-048），重点：35 层型 69 的 `spawn_boss_event(7)` 路径、胜利 restep 再开战的连锁、`battle_undo_snap` 在非路段战斗下的状态。记录 `target/verify/review_plan1_T4.md`。

- [ ] **Step 8: 提交**

```bash
git add crates/game-core/src/scene.rs docs/bug-ledger.md
git commit -F - <<'EOF'
fix(core): BUG-048 点地撞怪击杀触发剧情后不重开寻路段、不补入战斗笔

tick_battle 胜利尾只在路段仍挂起（未被胜利特判的剧情接管打断）时入战斗笔并续走。classic 战斗不开路段，零变化。

动手前三问：
1. 层：Rust 移植层（Stage B T2 挂点）。
2. 消费者：点地寻路驱动的战斗（Android / Windows 点地）与撤销日志。
3. 证据：A 级——宿主 RED（49 层型 70 击杀触发剧情后 route.is_open() 为真）→ GREEN；变异被杀；gates trace 无回退。
Evidence: core bug048_* RED→GREEN；scene.rs:5691 修复前无条件 route.resume。
Confidence: 85
EOF
```

---

### Task 5: BUG-046 移植"你无法战胜它"提示框（classic 收敛 Java，Java 参照帧对拍）

**Files:**
- Create: `crates/game-oracle/tests/fixtures/fx-boss68b.preset`、`fx-boss68b.txt`、`fx-boss68b.frames.txt`（最后一个由工具生成）
- Modify: `crates/game-oracle/tests/fixture_frames.rs`（在 `fx_boss68a_kill_spawn_frames_match` 之后追加用例）
- Modify: `crates/game-core/src/scene.rs:5426-5432`（不可胜分支）
- Modify: `docs/bug-ledger.md`（BUG-046 节）、`docs/spec/fixture-plan-boss.md`（FX-BOSS-68B 销账）

背景：原版 `a.java:5487-5490` 不可胜分支 `walkStepCount = 0; showOverlayMessage(0, "你无法战胜它", 0, 0)`；Rust 只清步、留注释。
对拍用例按 `docs/spec/fixture-plan-boss.md` FX-BOSS-68B：25 层、玩家 (6,7) 朝上、atk=310（= 型 68 缩放防御 ⇒ 预测 −1）、def=1000、hp=1000，
多次北撞 (6,6) 的型 68。夹具管线说明见 `tools/frames_fixture.py` 头注（`gen` 在隔离工作区跑 Java A/B/C 三变体，三方一致才抽取帧链）。

- [ ] **Step 1: 生成预置**

Run:
```bash
python3 tools/frames_fixture.py preset floor=25 x=6 y=7 facing=1 hp=1000 atk=310 def=1000 gold=0 yk=0 bk=0 rk=0 sky=5 \
  > crates/game-oracle/tests/fixtures/fx-boss68b.preset
diff <(sed -n 2p crates/game-oracle/tests/fixtures/fx-boss68b.preset | cut -c1-40) <(sed -n 2p crates/game-oracle/tests/fixtures/fx-boss68a.preset | cut -c1-40)
```
Expected: 文件 3 行（`MOT_IF@1=` / `MOT_L0@1=` / `SKY_WAR@1=`）；与 68a 预置只在 atk 字段不同（diff 可能显示首 40 字符相同）。

- [ ] **Step 2: 写场景脚本**

Create `crates/game-oracle/tests/fixtures/fx-boss68b.txt`：

```text
# ticks: 560
# preset: fx-boss68b.preset
# cover: FX-BOSS-68B 太上老君拒战负例（fixture-plan-boss §FX-BOSS-68B，a.java:5487-5490）：
#   boot 70/-6 100/-2 150/-5 200/-5 → 槽载入 floor25 (6,7) 朝上 → 300 -1 北撞 (6,6) 型68，
#   预测 -1（atk310 = 缩放防御）⇒ "你无法战胜它" kind 0 提示、walkStepCount=0、不开战 →
#   360 -5 关提示（kind 0 的 -1/-2 是翻页键，不关框）→ 420 -1 再撞（再弹提示）→ 480 -5 关提示。
#   preset：python3 tools/frames_fixture.py preset floor=25 x=6 y=7 facing=1 hp=1000 atk=310
#   def=1000 gold=0 yk=0 bk=0 rk=0 sky=5
70 -6 press
72 -6 release
100 -2 press
102 -2 release
150 -5 press
152 -5 release
200 -5 press
202 -5 release
300 -1 press
302 -1 release
360 -5 press
362 -5 release
420 -1 press
422 -1 release
480 -5 press
482 -5 release
```

- [ ] **Step 3: 生成 Java 参照帧**

Run: `python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-boss68b.txt`（耗时数分钟，可 `run_in_background` 后轮询）
Expected: 末尾打印三方 trace sha256 一致并写出 `crates/game-oracle/tests/fixtures/fx-boss68b.frames.txt`（560 帧）。

Run: `python3 tools/frames_fixture.py check crates/game-oracle/tests/fixtures/fx-boss68b.frames.txt --script crates/game-oracle/tests/fixtures/fx-boss68b.txt`
Expected: `[frames_fixture] 夹具自洽`。

取证（确认 Java 侧确实弹了提示）：`python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-boss68b.txt --frames all` 后，
查看 `analysis/frames-fixture/runs/A-fx-boss68b/` 下 T305 左右的帧 PNG，画面中央应有"你无法战胜它"提示框。若 gen 在本机失败
（Xvfb / JDK 问题），停下报告，不要手写帧文件。

- [ ] **Step 4: 写失败测试**

`crates/game-oracle/tests/fixture_frames.rs`，在 `fx_boss68a_kill_spawn_frames_match` 函数之后追加：

```rust
/// 【BUG-046】FX-BOSS-68B 拒战负例（fixture-plan-boss §FX-BOSS-68B，a.java:5487-5490）：
/// 预测 −1 ⇒ "你无法战胜它" kind 0 提示框、不开战、玩家与型 68 都不动；Java 参照帧逐帧一致。
#[test]
fn fx_boss68b_refuse_frames_match() {
    let records = frame_records(include_str!("fixtures/fx-boss68b.frames.txt"));
    assert_fixture_matches_script(&records, include_str!("fixtures/fx-boss68b.txt"));
    let (checked, sc) = replay("fx-boss68b", include_str!("fixtures/fx-boss68b.preset"), &records);
    assert_eq!(checked, records.len(), "T1-560 全帧必须比对");
    assert_eq!(sc.floor, 25);
    assert!(!sc.battle_active, "不开战");
    assert_eq!((sc.view.player_px >> 5, sc.view.player_py >> 5), (6, 7), "玩家不动");
    assert_eq!((sc.player.hp, sc.player.gold), (1000, 0), "血 / 金不变");
    let boss = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 68).expect("型 68 仍在");
    assert_eq!(sc.entities.items[boss].state, 0, "型 68 未进战斗态");
}
```

- [ ] **Step 5: 运行，确认失败**

Run: `cargo test --offline -p game-oracle --test fixture_frames fx_boss68b -- --nocapture`
Expected: FAIL，首个不一致帧在 T301 左右（Java 画出提示框，Rust 没有）。

- [ ] **Step 6: 最小实现**

`crates/game-core/src/scene.rs` 不可胜分支（5426 行附近）：

```rust
                                } else {
                                    self.walk_step_count = 0;
                                    // 【BUG-043】同上：箭头暂存随共用 walkStepCount 一并清空
                                    if let Some(a) = self.walkto_arrows.as_mut() {
                                        a.clear();
                                    }
                                    // 【BUG-046】a.java:5487-5490：拒战提示框（kind 0，无脚注图标）
                                    self.show_overlay_message(0, "你无法战胜它", 0, 0);
                                }
```

- [ ] **Step 7: 运行，确认通过**

Run: `cargo test --offline -p game-oracle --test fixture_frames fx_boss68b`
Expected: PASS（560 帧逐帧一致）。

- [ ] **Step 8: 全量测试 + 全部 classic 门禁**

Run: `cargo test --offline --no-fail-fast -p game-core -p game-shell-common -p game-android -p game-desktop -p game-oracle 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c`
Expected: 全绿。若 Task 2 的 `bug044_tap_walk_first_step_into_blocker_stays_put`"打不过的怪"用例因浮层而行为变化：该用例断言玩家不动、回 Map，浮层不影响这两条，应仍通过；若失败，读信息后报告。

Run: `python3 gates/cli.py trace; python3 gates/cli.py visual`
Expected: 无回退。若某个既有 classic 回放 / 帧夹具变红：说明该场景经过不可胜分支——它的 Java 参照帧本应含提示框，Rust 旧行为与 Java 不一致却未被发现。
逐个确认（看该夹具的 Java 帧是否在对应拍出现提示框）后再报告用户，**不得**改夹具或放宽比对。

- [ ] **Step 9: 更新台账与夹具计划**

`docs/bug-ledger.md` BUG-046 节：标题 `未修）` → `已修复）`；追加：

```markdown
- **修复**（2026-10-29）：不可胜分支补 `show_overlay_message(0, "你无法战胜它", 0, 0)`。Java 对拍：`fx-boss68b`（25 层 atk310 三次北撞，
  A/B/C 三方 trace 一致抽帧 560 帧）RED（T301 起不一致）→ GREEN；classic 向 Java 收敛，既有回放 / 帧夹具无变化（或列出变化项）。
```

`docs/spec/fixture-plan-boss.md` FX-BOSS-68B 节末尾追加：`> 【2026-10-29】已落地：crates/game-oracle/tests/fixtures/fx-boss68b.*，用例 fx_boss68b_refuse_frames_match（BUG-046）。`

- [ ] **Step 10: 对抗复核**

按附录 A 派复核子代理（任务名 T5 BUG-046），重点：提示框之后的吞键 / 关闭语义与 Java 是否一致（第二次北撞前的关闭键）、点地寻路撞上打不过的怪时浮层与 BUG-044 分支的交互。
记录 `target/verify/review_plan1_T5.md`。

- [ ] **Step 11: 提交**

```bash
git add crates/game-core/src/scene.rs crates/game-oracle/tests/fixture_frames.rs \
        crates/game-oracle/tests/fixtures/fx-boss68b.preset crates/game-oracle/tests/fixtures/fx-boss68b.txt \
        crates/game-oracle/tests/fixtures/fx-boss68b.frames.txt docs/bug-ledger.md docs/spec/fixture-plan-boss.md
git commit -F - <<'EOF'
fix(core): BUG-046 移植"你无法战胜它"拒战提示框（classic 收敛 Java）

不可胜分支补 showOverlayMessage(0, "你无法战胜它", 0, 0)（a.java:5487-5490）；新增 Java 参照帧夹具 fx-boss68b
（fixture-plan-boss FX-BOSS-68B）逐帧对拍。classic 行为向 Java 收敛：此前 Rust 撞打不过的怪静默挡住，与原版不一致。

动手前三问：
1. 层：Rust 移植层（classic 移植缺口，Rust 少了原版行为）；参照版 Java 不动。
2. 消费者：GameScene interact_with_cell 怪物臂（classic 与 modern 共用）。
3. 证据：A 级——a.java:5487-5490 原文；Java A/B/C 三方一致的 560 帧参照，Rust RED（T301 起）→ GREEN；gates trace / visual 结果见下。
Evidence: fx_boss68b_refuse_frames_match RED→GREEN；既有回放 / 帧夹具（列出是否有变化）。
Confidence: 95
EOF
```

---

### Task 6: 测试计数基线入账

**Files:**
- Modify: `data/status/baselines/`（由 `gates/cli.py ratchet-update` 写）

- [ ] **Step 1: 全量门禁**

Run: `python3 gates/cli.py all`
Expected: 无 FAIL；SKIP / TOLERATED 项与 `python3 gates/cli.py status` 列出的既有项一致（逐项核对，不得新增）。

- [ ] **Step 2: 更新基线**

Run: `python3 gates/cli.py ratchet-update && git diff --stat data/status/baselines/`
Expected: 只有测试计数基线变化；total 从 752 增加（本计划新增 11 个测试：T1 2、T2 2、T3 3、T4 2、T5 1，外加 T3 若拆分则以实际为准）。

- [ ] **Step 3: 提交**

```bash
git add data/status/baselines/
git commit -F - <<'EOF'
gate(baselines): ratchet 入账（BUG-044 / 045 / 046 / 047 / 048 前置修复新增测试）

动手前三问：
1. 层：门禁基线（ratchet）。
2. 消费者：gates/cli.py test-ratchet。
3. 证据：gates all 无回退；新增测试见各修复提交。
EOF
```

---

### Task 7: 实机冒烟（Xiaomi 14，`c362a15b`）

只做冒烟，不替代宿主测试。**只用 `c362a15b`**，不碰其他设备。注入前先确认用户没在操作手机（`/tmp/jjxy_tap.sh` 只在焦点为 com.jjxy.game 时注入；
若检测到用户触摸，停止注入并询问）。

- [ ] **Step 1: 构建与安装**

```bash
touch shell/android/src/native.rs
CARGO_BUILD_JOBS=8 cargo ndk -t arm64-v8a -P 26 -o android/app/src/main/jniLibs build --offline --release -p game-android
ANDROID_HOME=$HOME/Android/Sdk /tmp/gradle-8.5/bin/gradle -p android assembleDebug --offline -q
adb -s c362a15b install -r android/app/build/outputs/apk/debug/app-debug.apk
adb -s c362a15b shell am start -W -n com.jjxy.game/android.app.NativeActivity
```
Expected: `Success`，应用冷启动到标题。

- [ ] **Step 2: 冒烟项**

逐项操作并截图（`/tmp/jjxy_shot.sh <name>`），结果写 `target/verify/plan1_device.md`：
1. 教程层点远处墙格（BUG-047）：人物不穿墙。
2. 有怪的层点一只可胜的怪走过去开战 → 胜 → 点右上隐形撤销热区（逻辑右上 56×20；Xiaomi 14 竖屏 scene 画布 1152×1536 居中于 1200×1600 视口、
   视口顶 = 屏幕 y 134 ⇒ 屏幕约 (1090, 195)，注入前先截图核对画布位置）：怪回到原格且再撞会开战（BUG-045）。
3. 撞一只打不过的怪：出现"你无法战胜它"提示（BUG-046）。

若需要特定楼层 / 数值，先请用户提供存档或同意用 `fx` 预置写入设备存档；不擅自改用户存档（先备份 `files/` 到 `target/verify/device_plan1/`）。

- [ ] **Step 3: 收尾**

把冒烟结果简报给用户（中文）：每项通过 / 不通过 + 截图路径。不通过项回到对应任务排查（先宿主复现，再改）。

---

## 附录 A：每任务对抗复核提示词模板

派 `general-task-execution` 子代理，提示词：

```text
你是独立对抗复核者，目标是推翻本次改动的正确性。仓库 /home/peng/workspace/xiyou_remake（纪律见 AGENTS.md：证据 A/B/C、classic 预设须与参考版 Java 一致、门禁不得弱化）。
任务：<任务名，如 T2 BUG-044>。改动 = `git diff`（未提交）。相关 bug 台账：docs/bug-ledger.md 的 <BUG 编号> 节；设计依据：docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md §2.1。
只读：不得修改源码或文档；唯一可写 target/verify/review_plan1_T<N>.md。可跑 cargo test --offline 与只读命令；临时探针若放进源码树，跑完必须删除并在记录中说明。不要提交，不要连 adb。
请核查：① 修复是否覆盖台账描述的全部情形（逐条构造反例）；② 是否触达 classic（找出所有调用点，说明为何 classic 不经过，或给出会经过的路径）；
③ 测试是否会在修复被撤销时变红（做变异）、期望值是否独立于被测代码；④ 有无引入新问题（相邻状态机、撤销、BUG-038/039/040/043 的既有钉）。
输出：意见表（编号 | 严重度 Critical/Major/Minor | 意见 | 证据 文件:行 / 命令输出 | 建议修法），以及"已确认无问题"清单。最终回复用中文简要列出全部意见。
```
