//! 【求解器批1 Commit3】A/B 双解释器对拍消费面——(a) 经求解器直调门面、(b) 经 `ScriptEngine`。
//!
//! 计划 docs/superpowers/proposals/../plans/2026-11-09-solver-implementation-plan.md :295/:302：
//! (a) 经 game-solver 直调门面、(b) 经第二解释器（`game_core::script::ScriptEngine`），
//! 构造同一 fixture 行 → 断言可观测状态一致（hp/gold/keys/实体集合）。
//!
//! # R1 红线（提案 §5，违者对拍判无效、批1 不得合）
//! - (b) 侧唯一合法路径 = `ScriptEngine`（script.rs:154/162/270）：
//!   **本文件 `side_b` 模块零 `game_core::scene::solver_api` import、零 GameScene 驱动**，
//!   宿主 `BHost` 自建（GameScene 未 impl 四 host trait——生产 ScriptEngine 无宿主，
//!   实体语义按 op_* 生产面镜像，见下表）。
//! - `Snapshot::from_scene` 属 harness 层：只**读** GameScene 公开字段灌初态，
//!   不调用其任何方法、不驱动其解释器——两腿初态同源，不构成 (b) 侧路径。
//! - (a) 侧 `side_a` 是唯一 import `game_core::scene::solver_api` 的消费方（直调门面）。
//!
//! # 对拍观测域（计划 :302 的 hp/gold/keys/实体集合 + 支撑域）
//! stats=[hp,atk,def,yellow,blue,red,gold,item_stack_size]、实体集合（count+stair+items
//! 逐字段）、脚本域（cursor/index/flags）、facing、player_pixel。
//! **不比**：dialog/camera/menu/phase/walk/route/floor/minimap/popups/alchemy/min-max/
//! inventory_entry_count——两侧机制本不同构（(a) 写 GameScene、(b) 写 ScriptState/host），
//! 且非计划要求的可观测状态。
//!
//! # 槽67 数据偏差（DC-5 不可达，commit2 同款披露）
//! `DialogData::from_fixtures`（scene.rs:253 `.trim()`）吃掉末行尾空格 ⇒ (a) 槽67
//! `TAK_264_268` 无分隔符 panic；(b) `table()` split 不 trim ⇒ 可运行。槽67 经
//! `tryRunScene case67` 门控恒 false 不可达（docs/spec/dead-code-rulings.md DC-5 ☠️）
//! ⇒ 对拍行走 `0..SLOT67` 排除 + 数据守卫断言钉死偏差存在。
//!
//! # BHost 镜像表（(b) 宿主各 hook 对齐 (a) op_* 生产语义的依据）
//! - `give_gold(v)` ← `combat::gain_gold`（combat.rs:191，= op_gin kind1 同源）
//! - `pickup_item(v)` ← 自建 ItemCtx → `ItemResolver::pickup`（= op_gin kind0 的统计面，
//!   scene.rs:4221 `pickup_item_type` 同一 resolver；浮层/popups 非比对域省略）
//! - `m_082_set_dialog_target(v)` ← Equip 效果（a.java:8368-8396 ROS case2 = tryEquip；
//!   engine ROS_2 外派该 hook ⇒ 宿主按 `GameScene::try_equip`（scene.rs:869）同参镜像
//!   `builtin_effect("Equip").apply(EffectParams::Equip{tier, force:true})`）
//! - `spawn_entity` ← `entity::spawn(..., img_h=images.entity[t].height)`（= op_ces 同参）
//! - `ces_camera(idx)` ← `entity::remove_des`（op_ces script_ops.rs:254 的
//!   markEntityDesRemoved 钩子——**唯一区分 CES 与 ADD 的 hook**：op_add 无此调用）
//! - `remove_all_of_type` ← 镜像 op_des_all（script_ops.rs:208-214 倒序 type_==want →
//!   `entity::remove`；不用 entity.rs:601 的 state==1 口径——语料负参 DES×5 全在 slot63）
//! - `swd_convert_type81` ← 镜像 op_swd（script_ops.rs:226-237：倒序 type81 →
//!   visible/type=4/dispatch_sprite_size/state=2/anim_row=8）
//! - `set_entity_route_flag(idx)` ← type==72（ROUTE enums.rs:170）→ anim_row=1（= op_mov）
//! - `reload_floor_entities` ← **no-op**：测试场景 `floor_assets=vec![None;56]`（scene.rs
//!   `scene()`）⇒ (a) op_add/op_lay 均不 reload；且语料 ADD floor==2==persist.floor
//!   （engine ADD 无条件 reload，宿主 no-op 后与 (a) 对齐）
//! - `find_path` ← 恒 false：结果只进不比对域（armed/walk/phase）
//! - camera/dialog tick/mvs ← no-op（非比对域）
//!
//! # 判据（计划 :307-311 一字不动）
//! ```text
//! cargo test -p game-solver --lib -- --list | grep -c script_abs        # ≥1
//! cargo test -p game-solver --lib script_abs                            # 全绿（23 变体）
//! grep -c 对拍 crates/game-solver/src/script_abs.rs                     # ≥1
//! cargo test -p game-solver --lib 对拍                                  # fixture 68 槽 0 差异
//! python3 gates/cli.py check                                            # FAIL=0（13/13）
//! ```

use game_core::entity::Entity;
use game_core::scene::GameScene;

/// 对拍终态快照（两侧同型，`assert_eq!` 逐字段红字定位）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EntObs {
    pub count: usize,
    pub stair_up: Option<(i32, i32)>,
    pub stair_down: Option<(i32, i32)>,
    pub items: Vec<Entity>,
}

/// 可观测终态：stats=hp/atk/def/yellow/blue/red/gold/item_stack_size（计划 :302 口径）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Obs {
    pub cursor: usize,
    pub index: usize,
    pub flags: Vec<bool>,
    pub stats: [i32; 8],
    pub facing: i32,
    pub px: i32,
    pub py: i32,
    pub entities: EntObs,
}

fn ent_obs_of(entities: &game_core::entity::EntityTable) -> EntObs {
    EntObs {
        count: entities.count,
        stair_up: entities.stair_up,
        stair_down: entities.stair_down,
        items: entities.items[..entities.count].to_vec(),
    }
}

/// 两腿初态同源快照：只读 GameScene 公开字段（harness 层，见模块 doc R1 条款）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// [hp, atk, def, yellow_keys, blue_keys, red_keys, gold, item_stack_size]
    pub stats: [i32; 8],
    pub item_types: Vec<u8>,
    pub item_uses: Vec<u8>,
    pub facing: i32,
    pub px: i32,
    pub py: i32,
    pub difficulty: i32,
    pub floor: i32,
    pub equipped_weapon: i32,
    pub equipped_armor: i32,
    pub equip_types: [i32; 12],
    pub equip_bonuses: [i32; 12],
    pub tutorial_seen: bool,
    pub width_table: Vec<i32>,
    pub img_heights: Vec<Option<i32>>,
    pub grid_cols: i32,
    pub grid_rows: i32,
    pub entities: EntObs,
}

impl Snapshot {
    pub fn from_scene(sc: &GameScene) -> Self {
        Snapshot {
            stats: [
                sc.player.hp,
                sc.player.atk,
                sc.player.def,
                sc.player.yellow_keys,
                sc.player.blue_keys,
                sc.player.red_keys,
                sc.player.gold,
                sc.player.item_stack_size,
            ],
            item_types: sc.player.item_types.clone(),
            item_uses: sc.player.item_uses.clone(),
            facing: sc.facing,
            px: sc.view.player_px,
            py: sc.view.player_py,
            difficulty: sc.menu.difficulty_index,
            floor: sc.persist.floor,
            equipped_weapon: sc.equipped_weapon,
            equipped_armor: sc.equipped_armor,
            equip_types: sc.equip_types,
            equip_bonuses: sc.equip_bonuses,
            tutorial_seen: sc.menu.tutorial_seen,
            width_table: sc.width_table.clone(),
            img_heights: sc
                .images
                .entity
                .iter()
                .map(|o| o.as_ref().map(|im| im.height))
                .collect(),
            grid_cols: sc.grid.cols,
            grid_rows: sc.grid.rows,
            entities: ent_obs_of(&sc.entities),
        }
    }
}

/// (b) 侧对话表：复刻 `script_golden.rs:15-37 table()` 构造（include_str 同源夹具、
/// 同 split/unescape 口径；路径按本文件位置重定位到 game-core 夹具）。
pub fn dialogue_table() -> game_core::script::DialogueTable {
    use game_core::script::DialogueTable;
    let scripts: Vec<String> = include_str!("../../game-core/tests/fixtures/level_scripts.txt")
        .split("\u{1e}\n")
        .map(|s| s.trim_end_matches('\u{1e}').to_string())
        .collect();
    let texts: Vec<String> = include_str!("../../game-core/tests/fixtures/dialogue_texts.txt")
        .lines()
        .map(game_core::scene::unescape_text_fixture)
        .collect();
    let speakers: Vec<i8> = include_str!("../../game-core/tests/fixtures/speaker_types.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    let type_names: Vec<String> = include_str!("../../game-core/tests/fixtures/object_type_names.txt")
        .lines()
        .map(|s| s.to_string())
        .collect();
    let type_categories: Vec<u8> =
        include_str!("../../game-core/tests/fixtures/type_categories.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
    DialogueTable { scripts, texts, speakers, type_names, type_categories }
}

/// (a) 腿：经求解器直调门面驱动 GameScene（唯一 import `scene::solver_api` 处）。
pub mod side_a {
    use super::{ent_obs_of, Obs};
    use game_core::scene::{solver_api, GameScene};

    /// 驱动单槽到停摆（协议与 commit2 drive_facade 同构）：门步/收尾步不改 (index,cursor)
    /// ⇒ 停止；GIN 尾递归、MOV miss 尾递归在 execute 内自行完成。
    pub fn drive(sc: &mut GameScene, idx: usize) -> Obs {
        sc.script_line = sc.dialog.dialog_data.scripts[idx].clone();
        sc.script_cursor = 0;
        sc.current_script_index = idx;
        let mut steps = 0usize;
        loop {
            let before = (sc.current_script_index, sc.script_cursor);
            solver_api::execute_script_instruction(sc);
            if (sc.current_script_index, sc.script_cursor) == before {
                break;
            }
            steps += 1;
            assert!(steps < 100_000, "(a) 槽{idx} 疑似死循环（GUT 环未在语料预检内）");
        }
        Obs {
            cursor: sc.script_cursor,
            index: sc.current_script_index,
            flags: sc.script_line_flags.clone(),
            stats: [
                sc.player.hp,
                sc.player.atk,
                sc.player.def,
                sc.player.yellow_keys,
                sc.player.blue_keys,
                sc.player.red_keys,
                sc.player.gold,
                sc.player.item_stack_size,
            ],
            facing: sc.facing,
            px: sc.view.player_px,
            py: sc.view.player_py,
            entities: ent_obs_of(&sc.entities),
        }
    }
}

/// (b) 腿：`ScriptEngine` + 自建宿主（R1：零 solver_api、零 GameScene 驱动）。
pub mod side_b {
    use super::{ent_obs_of, Obs, Snapshot};
    use game_core::combat::PlayerCombat;
    use game_core::effect_registry::{builtin_effect, EffectParams, ItemCtx, ItemOutcome};
    use game_core::entity::{self, CellGrid, EntityTable};
    use game_core::script::{DialogueTable, HostCtx, ScriptEngine};
    use game_core::subsystem::dialog::{DialogCtx, DialogStep, DialogSubsystem};
    use game_core::subsystem::entity_service::EntityService;
    use game_core::subsystem::item_service::{ItemResolver, ItemService};

    /// (b) 侧宿主：四 host trait 镜像实现（镜像表见模块 doc）。
    /// GameScene 未 impl 这些 trait ⇒ 生产 ScriptEngine 无宿主；本宿主实体语义
    /// 按 op_* 生产面逐钩镜像，初态由 `Snapshot`（harness 读 pub 字段）灌入。
    pub struct BHost {
        pub player: PlayerCombat,
        pub entities: EntityTable,
        pub grid: CellGrid,
        pub width_table: Vec<i32>,
        pub img_heights: Vec<Option<i32>>,
        pub equipped_weapon: i32,
        pub equipped_armor: i32,
        pub equip_types: [i32; 12],
        pub equip_bonuses: [i32; 12],
        pub tutorial_seen: bool,
        pub floor: i32,
    }

    impl BHost {
        /// 初态重建：spawn 重放完成格登记（cell_type/slots/楼梯锚点），
        /// 记录逐字段覆盖回初值（Entity: Copy ⇒ 与 (a) 初态逐位相等）。
        fn from_snapshot(s: &Snapshot) -> Self {
            let mut entities = EntityTable::new(100);
            let mut grid = CellGrid::new(s.grid_cols, s.grid_rows);
            for (i, e) in s.entities.items.iter().enumerate() {
                let img_h = s.img_heights.get(e.type_ as usize).copied().flatten();
                entity::spawn(
                    &mut entities,
                    &mut grid,
                    e.type_,
                    e.px,
                    e.py,
                    e.param,
                    &s.width_table,
                    img_h,
                );
                entities.items[i] = *e;
            }
            entities.count = s.entities.items.len();
            entities.stair_up = s.entities.stair_up;
            entities.stair_down = s.entities.stair_down;
            BHost {
                player: PlayerCombat {
                    hp: s.stats[0],
                    atk: s.stats[1],
                    def: s.stats[2],
                    yellow_keys: s.stats[3],
                    blue_keys: s.stats[4],
                    red_keys: s.stats[5],
                    gold: s.stats[6],
                    item_types: s.item_types.clone(),
                    item_uses: s.item_uses.clone(),
                    item_stack_size: s.stats[7],
                },
                entities,
                grid,
                width_table: s.width_table.clone(),
                img_heights: s.img_heights.clone(),
                equipped_weapon: s.equipped_weapon,
                equipped_armor: s.equipped_armor,
                equip_types: s.equip_types,
                equip_bonuses: s.equip_bonuses,
                tutorial_seen: s.tutorial_seen,
                floor: s.floor,
            }
        }

        fn stats(&self) -> [i32; 8] {
            [
                self.player.hp,
                self.player.atk,
                self.player.def,
                self.player.yellow_keys,
                self.player.blue_keys,
                self.player.red_keys,
                self.player.gold,
                self.player.item_stack_size,
            ]
        }

        fn set_stat(&mut self, i: usize, v: i32) {
            match i {
                0 => self.player.hp = v,
                1 => self.player.atk = v,
                2 => self.player.def = v,
                3 => self.player.yellow_keys = v,
                4 => self.player.blue_keys = v,
                5 => self.player.red_keys = v,
                6 => self.player.gold = v,
                7 => self.player.item_stack_size = v,
                _ => unreachable!(),
            }
        }

        fn item_ctx(&mut self) -> ItemCtx<'_> {
            ItemCtx {
                player: &mut self.player,
                floor: self.floor,
                tutorial_seen: &mut self.tutorial_seen,
                equipped_weapon: &mut self.equipped_weapon,
                equipped_armor: &mut self.equipped_armor,
                equip_types: self.equip_types,
                equip_bonuses: self.equip_bonuses,
            }
        }
    }

    impl HostCtx for BHost {
        /// 结果只进不比对域（armed/walk/phase）——恒 false 不影响对拍判据。
        fn find_path(&mut self, _fx: i32, _fy: i32, _tx: i32, _ty: i32) -> bool {
            false
        }
        fn snap_camera_to_player(&mut self) {}
        fn locate_speaker_entity(&mut self, _speaker: i32) -> i32 {
            -1
        }
        /// 关键区分钩子：op_ces = spawn + markEntityDesRemoved（script_ops.rs:254），
        /// op_add = spawn only（无本调用）⇒ 承载 remove_des 以区分 CES/ADD 两臂。
        fn ces_camera(&mut self, idx: i32) {
            entity::remove_des(&mut self.entities, idx as usize);
        }
        fn set_camera_anchor_target(&mut self, _x: i32, _y: i32) {}
        fn exit_application(&mut self) {}
        /// 语料零 MVS（实测 296 指令无 MVS）——与 script_golden StubHost 同为边界桩。
        fn mvs_exchange(&mut self) {}
        fn snap_camera_after_ros1(&mut self, _px: i32, _py: i32) {}
    }

    impl ItemService for BHost {
        /// 统计面直通 resolver（= op_gin kind0 的 `pickup_item_type` 同一 resolver，
        /// scene.rs:4221；浮层/popups 非比对域省略）。
        fn pickup(&mut self, ctx: &mut ItemCtx, t: i32) -> ItemOutcome {
            ItemResolver.pickup(ctx, t)
        }
        /// = op_gin kind1：combat::gain_gold 同源（script_ops.rs:86-89）。
        fn give_gold(&mut self, amount: i32) {
            let _ = game_core::combat::gain_gold(&mut self.player, amount);
        }
        fn pickup_item(&mut self, item: i32) {
            let mut ctx = self.item_ctx();
            let _ = ItemResolver.pickup(&mut ctx, item);
        }
    }

    impl DialogSubsystem for BHost {
        fn tick(&mut self, _ctx: &mut DialogCtx) -> DialogStep {
            DialogStep::Wait
        }
        fn set_typewriter(&mut self, _s: &str, _a: i32, _b: i32) {}
        /// engine ROS_2 外派本 hook（script.rs:383）；生产语义 = tryEquip(tier, true)
        /// （a.java:8368-8396 ROS case2）⇒ 按 GameScene::try_equip（scene.rs:869）同参
        /// 镜像 Equip 效果（force=true ⇒ RES 强制档位语义）。
        fn m_082_set_dialog_target(&mut self, v: i32) {
            let mut ctx = self.item_ctx();
            let _ = builtin_effect("Equip").apply(&mut ctx, &EffectParams::Equip { tier: v, force: true });
        }
    }

    impl EntityService for BHost {
        fn cell_entity(&self, x: i32, y: i32, t: i32) -> i32 {
            entity::cell_entity(&self.grid, &self.entities, x, y, t)
        }
        fn entity_type(&self, idx: i32) -> i32 {
            self.entities.items[idx as usize].type_
        }
        fn remove_entity(&mut self, idx: i32) {
            if idx >= 0 {
                entity::remove(&mut self.entities, idx as usize);
            }
        }
        /// 镜像 op_des_all（script_ops.rs:208-214）：倒序 type_==want → entity::remove。
        /// 不用 entity.rs::remove_all_of_type（state==1 口径不同）。
        fn remove_all_of_type(&mut self, want: i32) {
            for i in (0..self.entities.count).rev() {
                if self.entities.items[i].type_ == want {
                    entity::remove(&mut self.entities, i);
                }
            }
        }
        /// 镜像 op_swd（script_ops.rs:226-237）。
        fn swd_convert_type81(&mut self) {
            for i in (0..self.entities.count).rev() {
                if !self.entities.items[i].removed && self.entities.items[i].type_ == 81 {
                    self.entities.items[i].visible = true;
                    self.entities.items[i].type_ = 4;
                    let img_h = self.img_heights.get(4).copied().flatten();
                    entity::dispatch_sprite_size(4, i, &mut self.entities, &self.width_table, img_h);
                    self.entities.items[i].state = 2;
                    self.entities.items[i].anim_row = 8;
                }
            }
        }
        fn entity_pixel(&self, idx: i32) -> (i32, i32) {
            let e = &self.entities.items[idx as usize];
            (e.px, e.py)
        }
        /// no-op（镜像表依据：测试场景 floor_assets=None、语料 ADD floor==2）。
        fn reload_floor_entities(&mut self, _floor: i32) {}
        fn spawn_entity(&mut self, t: i32, x: i32, y: i32) -> i32 {
            // img_h 同 op_ces（script_ops.rs:245-247）：images.entity[t].height
            let img_h = self.img_heights.get(t as usize).copied().flatten();
            entity::spawn(&mut self.entities, &mut self.grid, t, x, y, 0, &self.width_table, img_h)
                as i32
        }
        /// 镜像 op_mov（script_ops.rs:185-187）：type==ROUTE(72) → anim_row=1。
        fn set_entity_route_flag(&mut self, idx: i32) {
            let e = &mut self.entities.items[idx as usize];
            if e.type_ == game_core::enums::entity_kind::ROUTE {
                e.anim_row = 1;
            }
        }
    }

    fn eng_stats(eng: &ScriptEngine) -> [i32; 8] {
        [
            eng.state.player_hp,
            eng.state.player_atk,
            eng.state.player_def,
            eng.state.yellow_keys,
            eng.state.blue_keys,
            eng.state.red_keys,
            eng.state.gold,
            eng.state.item_stack_size,
        ]
    }

    fn eng_set_stat(eng: &mut ScriptEngine, i: usize, v: i32) {
        match i {
            0 => eng.state.player_hp = v,
            1 => eng.state.player_atk = v,
            2 => eng.state.player_def = v,
            3 => eng.state.yellow_keys = v,
            4 => eng.state.blue_keys = v,
            5 => eng.state.red_keys = v,
            6 => eng.state.gold = v,
            7 => eng.state.item_stack_size = v,
            _ => unreachable!(),
        }
    }

    /// 驱动单槽到停摆：与 (a) 同协议（(index,cursor) 不变即停；GIN/MOV 尾递归在
    /// execute 内自行完成）。stats 逐字段同步：引擎改→灌宿主、宿主改→灌引擎、
    /// 同字段同拍双改→冲突 panic（语料预检不可达：GIN 全在行尾只撞门）。
    pub fn drive(init: &Snapshot, table: &DialogueTable, idx: usize) -> Obs {
        let mut host = BHost::from_snapshot(init);
        let mut eng = ScriptEngine::new(table, idx);
        // ScriptEngine::new 只置 flags/游标/当前行；stats/facing/pixel/难度/楼层
        // 从 (a) 初态快照灌入（两侧初态同源 ⇒ RES 等重置臂有同一起点）。
        eng.state.player_hp = init.stats[0];
        eng.state.player_atk = init.stats[1];
        eng.state.player_def = init.stats[2];
        eng.state.yellow_keys = init.stats[3];
        eng.state.blue_keys = init.stats[4];
        eng.state.red_keys = init.stats[5];
        eng.state.gold = init.stats[6];
        eng.state.item_stack_size = init.stats[7];
        eng.state.facing_direction = init.facing;
        eng.state.player_pixel_x = init.px;
        eng.state.player_pixel_y = init.py;
        eng.state.difficulty_index = init.difficulty;
        eng.state.current_floor = init.floor;
        let mut steps = 0usize;
        loop {
            let before = (eng.state.current_script_index, eng.state.script_cursor);
            let pre_e = eng_stats(&eng);
            let pre_h = host.stats();
            eng.execute_script_instruction(&mut host);
            let post_e = eng_stats(&eng);
            let post_h = host.stats();
            for i in 0..8 {
                let e_ch = post_e[i] != pre_e[i];
                let h_ch = post_h[i] != pre_h[i];
                assert!(
                    !(e_ch && h_ch),
                    "stats[{i}] 同步冲突：引擎/宿主同拍双改（语料预检不可达）"
                );
                if e_ch {
                    host.set_stat(i, post_e[i]);
                } else if h_ch {
                    eng_set_stat(&mut eng, i, post_h[i]);
                }
            }
            if (eng.state.current_script_index, eng.state.script_cursor) == before {
                break;
            }
            steps += 1;
            assert!(steps < 100_000, "(b) 槽{idx} 疑似死循环（GUT 环未在语料预检内）");
        }
        Obs {
            cursor: eng.state.script_cursor,
            index: eng.state.current_script_index,
            flags: eng.state.script_line_flags.clone(),
            stats: host.stats(),
            facing: eng.state.facing_direction,
            px: eng.state.player_pixel_x,
            py: eng.state.player_pixel_y,
            entities: ent_obs_of(&host.entities),
        }
    }
}

/// 计划 :293-305 的 A/B/C 分类（实测修正：GLV 空体归 B、END 单独终止符、
/// MVS 双重死码 + ROS 其余子码不取参不推进 → C）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// A 类：改状态（16 变体，红先行断言对象）
    A,
    /// B 类：状态不变纯推进/对话（护栏）
    B,
    /// 终止符：END（脚本流终止 + 游标推进）
    Termin,
    /// C 类：死码/no-op（MVS 数据域零使用；ROS 其余子码）
    C,
}

pub fn class_of(op: &game_core::scene::ScriptOp) -> Class {
    use game_core::scene::ScriptOp::*;
    use Class::*;
    match op {
        Gin { .. } | Add { .. } | DesAll { .. } | DesAt { .. } | Ces { .. } | Mov { .. }
        | RosMove { .. } | RosEquip { .. } | RosHp { .. } | RosFace { .. } | RosAtk { .. }
        | RosDef { .. } | Lay { .. } | Swd { .. } | Res { .. } | Gut { .. } => A,
        Tak { .. } | See { .. } | Sms { .. } | Glv { .. } => B,
        End { .. } => Termin,
        Mvs { .. } | RosNoop { .. } => C,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::enums::GameMode;
    use game_core::scene::ScriptOp;
    use game_core::scene_test_support::{scene_setup, SceneSetup};

    const SLOTS: usize = 68;
    const SLOT67: usize = 67;

    fn new_scene() -> GameScene {
        scene_setup(
            (5, 3),
            &[(4, 6, 3), (84, 7, 3)],
            SceneSetup {
                game_mode: Some(GameMode::Dialog),
                dialog_dialog_phase: Some(0),
                ..SceneSetup::default()
            },
        )
    }

    /// 判据 grep 对拍 ≥1 + `--lib 对拍` 过滤命中：fixture 68 槽逐行 0 差异。
    #[test]
    fn script_abs_对拍_fixture68槽逐行零差异() {
        let table = dialogue_table();
        for idx in 0..SLOT67 {
            let mut sc = new_scene();
            let snap = Snapshot::from_scene(&sc);
            let oa = side_a::drive(&mut sc, idx);
            let ob = side_b::drive(&snap, &table, idx);
            assert_eq!(oa, ob, "槽{idx} 两侧终态逐字段不等（(a) 门面 vs (b) ScriptEngine）");
        }
        let _ = SLOTS;
    }

    /// A 类改状态红先行：RES_0 重置集（difficulty==0 ⇒ hp400/atk10/def10/keys0/gold4）。
    #[test]
    fn script_abs_a类_res_0重置集两侧相等() {
        let table = dialogue_table();
        let mut sc = new_scene();
        let snap = Snapshot::from_scene(&sc);
        let oa = side_a::drive(&mut sc, 0);
        assert_eq!(oa.stats, [400, 10, 10, 0, 0, 0, 4, 0], "RES_0 重置集（(a) 门面腿）");
        let ob = side_b::drive(&snap, &table, 0);
        assert_eq!(ob.stats, oa.stats, "RES_0：(b) ScriptEngine 腿与 (a) 逐字段相等");
    }

    /// A 类改状态红先行：GIN_1_1000 给金（gain_gold 同源 + 尾递归撞门不改金）。
    #[test]
    fn script_abs_a类_gin_1给金两侧相等() {
        let table = dialogue_table();
        let mut sc = new_scene();
        let snap = Snapshot::from_scene(&sc);
        let oa = side_a::drive(&mut sc, 26);
        assert_eq!(oa.stats[6] - snap.stats[6], 1000, "GIN_1_1000 净增 1000（(a)）");
        let ob = side_b::drive(&snap, &table, 26);
        assert_eq!(ob.stats, oa.stats, "GIN_1：(b) 与 (a) 逐字段相等");
    }

    /// A 类改状态红先行：DES_84_11_4 定点移除（占用格 (11,4) 的 type84 实体）。
    #[test]
    fn script_abs_a类_des_84定点移除两侧相等() {
        let table = dialogue_table();
        let mut sc = scene_setup(
            (5, 3),
            &[(84, 11, 4)],
            SceneSetup {
                game_mode: Some(GameMode::Dialog),
                dialog_dialog_phase: Some(0),
                ..SceneSetup::default()
            },
        );
        let snap = Snapshot::from_scene(&sc);
        let oa = side_a::drive(&mut sc, 26);
        assert_ne!(
            oa.entities.items, snap.entities.items,
            "DES_84_11_4 已移除初态实体（(a)）"
        );
        let ob = side_b::drive(&snap, &table, 26);
        assert_eq!(ob.entities, oa.entities, "DES：(b) 与 (a) 实体集合逐字段相等");
        assert_eq!(ob.stats, oa.stats, "DES：stats 同比（GIN 尾随给金）");
    }

    /// 23 变体分类全覆盖（计划 :307 第二条「23 变体全覆盖」的分类面）。
    #[test]
    fn script_abs_23变体分类全覆盖() {
        use Class::*;
        let samples: Vec<(Class, ScriptOp)> = vec![
            (A, ScriptOp::Gin { kind: 1, value: 0, cursor: 4 }),
            (A, ScriptOp::Add { floor: 1, ty: 1, cx: 0, cy: 0, cursor: 4 }),
            (A, ScriptOp::DesAll { want: -1, cursor: 4 }),
            (A, ScriptOp::DesAt { ty: 1, cx: 0, cy: 0, cursor: 4 }),
            (A, ScriptOp::Ces { ty: 1, cx: 0, cy: 0, cursor: 4 }),
            (A, ScriptOp::Mov { target_type: 1, cell: None, dst: (0, 0), cursor: 4 }),
            (A, ScriptOp::RosMove { px: 0, py: 0, cursor: 4 }),
            (A, ScriptOp::RosEquip { tier: 1, cursor: 4 }),
            (A, ScriptOp::RosHp { v: 1, cursor: 4 }),
            (A, ScriptOp::RosFace { v: 1, cursor: 4 }),
            (A, ScriptOp::RosAtk { v: 1, cursor: 4 }),
            (A, ScriptOp::RosDef { v: 1, cursor: 4 }),
            (A, ScriptOp::Lay { floor: 1, cursor: 4 }),
            (A, ScriptOp::Swd { cursor: 4 }),
            (A, ScriptOp::Res { cursor: 4 }),
            (A, ScriptOp::Gut { target: 1, cursor: 4 }),
            (B, ScriptOp::Tak { start: 0, end: 0, cursor: 4 }),
            (B, ScriptOp::See { tx: 0, ty: 0, page: 0, page_end: 0, flag: 0, cursor: 4 }),
            (B, ScriptOp::Sms { cursor: 4 }),
            (B, ScriptOp::Glv { cursor: 4 }),
            (Termin, ScriptOp::End { cursor: 4 }),
            (C, ScriptOp::Mvs { cursor: 4 }),
            (C, ScriptOp::RosNoop { cursor: 4 }),
        ];
        assert_eq!(samples.len(), 23, "ScriptOp 23 变体全枚举");
        for (i, (want, op)) in samples.iter().enumerate() {
            assert_eq!(class_of(op), *want, "第 {i} 变体分类不符计划 :293-305");
        }
        assert_eq!(samples.iter().filter(|(c, _)| *c == A).count(), 16, "A 类 16 变体");
        assert_eq!(samples.iter().filter(|(c, _)| *c == B).count(), 4, "B 类 4 变体");
    }

    /// 槽67 数据守卫：偏差存在且已知（from_fixtures trim 尾空格，DC-5 不可达）。
    #[test]
    fn script_abs_槽67数据守卫_dc5不可达() {
        let table = dialogue_table();
        assert_eq!(table.scripts.len(), SLOTS, "fixture 68 槽");
        // (b) 侧 split 口径：尾空格保留
        assert_eq!(
            table.scripts[SLOT67], "TAK_264_268 ",
            "槽67 (b) 语料带尾空格；(a) from_fixtures trim ⇒ 无空格 panic（DC-5 不可达）"
        );
        // 夹具原文同样带尾空格 ⇒ 分歧源 = trim，非夹具本身
        assert!(
            include_str!("../../game-core/tests/fixtures/level_scripts.txt")
                .ends_with("TAK_264_268 "),
            "夹具原文末尾带尾空格"
        );
    }
}
