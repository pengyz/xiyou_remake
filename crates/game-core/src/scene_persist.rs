//! task-15 步④ 装载/存档/换层（move-only 自 scene.rs）。
use super::*; // scene.rs 私有面可见（子模块）——纯剪切语义保持

/// 【task-15 V2 persist 收口】持久域切片（v3 §1 域切纪律：一次性切禁双写）——
/// 装载/存档/换层状态 13 字段唯一归属（field-ownership.json persist 域）。
/// dump 名不变（state_dump 名集面零漂）。
pub struct PersistState {
    /// SKY_WAR 全局存档（saveGlobalProgress a.java:8917-8942；两机同构拷贝）。
    pub global: crate::save::GlobalSave,
    /// 全局态脏位（壳轮询持久化）。
    pub global_dirty: bool,
    /// HUD 数值源（paintHudPanel/paintStatusBar 读；战斗/拾取写）
    pub floor: i32,
    pub current_floor: i32,
    /// minFloorReached/maxFloorReached
    pub min_floor: i32,
    pub max_floor: i32,
    /// f_byte_arr2_05：层状态存档（m_119 写/m_122 读；索引=层号）
    pub saved_floor_states: Vec<Option<Vec<u8>>>,
    /// 层资产表（按层号索引）
    pub floor_assets: Vec<Option<FloorAssets>>,
    /// 虚拟 RMS：MOT_L{slot} 槽位 blob（saveFloorState 写侧；game-core 无 I/O）
    pub saved_slot_blobs: [Option<Vec<u8>>; 6],
    /// MOT_IF 槽表 blob（preset 虚拟 RMS；loadSlotTable 每次进 mode7 重读）
    pub preset_mot_if: Option<Vec<u8>>,
    /// pendingFloor：目标层
    pub pending_floor: i32,
    /// wentDown：下楼标记（型7 上楼传 false）
    pub went_down: bool,
    pub journal: Vec<(i32, i32)>,
}

impl GameScene {
    pub fn load_floor(
        maplv: &[u8],
        player_px: i32,
        player_py: i32,
        records: &[(u8, u16, u16, i16, bool)],
        width_table: &[i32],
        tile_walkability: &[i32],
        images: SceneImages,
        floor_assets: Vec<Option<FloorAssets>>,
    ) -> Result<GameScene, String> {
        let mut scene = GameScene {
            tap: crate::tap_walk::TapState {
    ui_selected: None,
    ui_frame: 0,
    walkto_arrows: None,
    battle_undo_snap: None,
    auto_battle: false,
    tap_target: None,
    tap_reject: None,
    tap_bump: None,
    tap_followup_ready: false,
    tap_selected: None,
    tap_pending: None,
            },
            persist: PersistState {
    global: crate::save::GlobalSave::default_7(),
    global_dirty: false,
    floor: 1,
    pending_floor: 0,
    went_down: false,
    min_floor: 1,
    max_floor: 1,
    current_floor: 1,
    floor_assets,
    saved_floor_states: vec![None; 56],
    journal: Vec::new(),
    preset_mot_if: None,
    saved_slot_blobs: [None, None, None, None, None, None],
            },
            view: GameView::from_maplv(maplv, player_px, player_py, crate::game_view::ViewDims::classic())?,
            entities: EntityTable::new(100),
            grid: CellGrid::new(1, 1),
            images,
            walkable: Vec::new(),
            minimap: None,
            minimap_enabled: true,
            layout: crate::layout::Layout::classic(),
            frame_counter: 0,
            render: scene_paint::RenderState {
    bob_offset: 0,
    bob_rising: false,
    backdrop_scroll: 0,
    open_anim_tick: 0,
    tile_bob: 0,
    strip_frame: 0,
            },
            view_bottom: view_bottom_y(),
            facing: 0,
            walk_phase: 0,
            player_frame: 0,
            player_bob_applied: false,
            player: crate::combat::PlayerCombat {
                hp: 0,
                atk: 0,
                def: 0,
                yellow_keys: 0,
                blue_keys: 0,
                red_keys: 0,
                gold: 0,
                item_types: vec![0; 16],
                item_uses: vec![0; 16],
                item_stack_size: 0,
            },
            enemies: crate::combat::ScaledEnemies::default(),
            enemy_base_hp: Vec::new(),
            enemy_base_gold: Vec::new(),
            menu: scene_menu::MenuState {
    softkeys: (1, 3),
    tutorial_seen: false,
    stair_hint: false,
    overlay_kind_item: 0,
    overlay_icon: None,
    overlay_item_name: None,
    record_page: 0,
    record_text_a: include_str!("../tests/fixtures/record_texts_a.txt")
        .lines()
        .map(|l| l.replace("\\n", "\n"))
        .collect(),
    record_text_b: include_str!("../tests/fixtures/record_texts_b.txt")
        .lines()
        .map(|l| l.replace("\\n", "\n"))
        .collect(),
    page_indicator: String::new(),
    difficulty_index: 0,
    menu_kinds: Vec::new(),
    menu_item_count: 0,
    menu_box_width: 0,
    menu_line_height: 0,
    menu_visible_rows: 0,
    menu_list_height: 0,
    menu_scroll_top: 0,
    menu_visible_bottom: 0,
    menu_cursor: 0,
    menu_return_mode: GameMode::MainMenu,
    slot: crate::menu_family::SlotSelect::new(),
    encyc_damage: Vec::new(),
    encyc_types: Vec::new(),
    encyc_panel_w: ENCYC_PANEL_W,
    encyc_count: 0,
    encyc_rows_visible: 0,
    encyc_panel_h: 0,
    encyc_scroll_top: 0,
    encyc_scroll_bottom: 0,
    options: None,
    help_override: None,
    shell_options: Vec::new(),
    in_game_menu_return_mode: GameMode::Boot, // BUG-008：Java 无初值（默认 0）保真对齐；原值 3 属顺手修 bug 偏差
    item_cols: 4,
    item_rows_total: 8,
    item_rows_visible: 6,
    item_scroll_top: 0,
    item_scroll_bottom: 6,
    item_box_w: 202,
    item_box_h: 296,
    item_cursor: 0,
            },
            key_value: 0,
            key_held: 0,
            player_cell_x: player_px >> 5,
            player_cell_y: player_py >> 5,
            step_progress: 0,
            walk_step_count: 0,
            battle_target: -1,
            battle_enemy_hp: 0,
            battle_tick: 0,
            battle_anim_tick: 0,
            battle_finishing: false,
            battle_rerun_scene: false,
            battle_no_counter: false,
            battle_active: false,
            popups: crate::popup::PopupRing::default(),
            rng: game_platform::JavaRandom::new_seeded(0),
            transitioning: false,
            wipe_closing: true,
            wipe: 0,
            route_entity: 0,
            route_dir: 0,
            route_phase: 0,
            route_progress: 0,
            route_start: (0, 0),
            object_type_names: {
                let raw = include_str!("../tests/fixtures/object_type_names.txt");
                raw.lines().map(|l| l.trim().to_string()).collect()
            },
            equipped_weapon: 0,
            equipped_armor: 0,
            equip_bonuses: [0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220],
            equip_types: [0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40],
            dialog: scene_dialog::DialogState {
    overlay_active: false,
    overlay_kind: 0,
    overlay_text: None,
    overlay_box_w: 0,
    overlay_box_h: 0,
    overlay_left_icon: 0,
    overlay_right_icon: 0,
    overlay_lines: Vec::new(),
    overlay_page_top: 0,
    overlay_page_bottom: 0,
    overlay_lines_per_page: 0,
    overlay_total_lines: 0,
    overlay_content_w: 0,
    dialog_phase: 0,
    dialog_page: 0,
    dialog_page_end: 0,
    dialog_speaker: 0,
    dialog_anchor: -1,
    dialog_char: 0,
    dialog_target: (0, 0),
    dialog_data: DialogData::from_fixtures(),
    overlay_saved_color: 0,
            },
            open_group: Vec::new(),
            game_mode: GameMode::Map,
            transition_count: 0,
            last_transition: None,
            ending_scroll: 0,
            ending_anim: 0,
            ending_done: false,
            script_line: String::new(),
            script_cursor: 0,
            script_delay: 0,
            script_line_flags: vec![false; 68],
            current_script_index: 0,
            script_walk_armed: false,
            alchemy_price: 0,
            alchemy_gain_hp: 0,
            alchemy_gain_atk: 0,
            alchemy_gain_def: 0,
            alchemy_cursor: 0,
            alchemy_upgrade_count: 0,
            merchant_entity: -1,
            offer_queue_src: Vec::new(),
            offer_queue_idx: Vec::new(),
            offer_queue_cursor: 0,
            bgm: crate::audio::BgmState::in_game_default(),
            rainbow_phase: 0,
            path_arrows_visible: false,
            walk_path_buffer: Vec::new(),
            undo: crate::undo_journal::UndoJournal::new(),
            route: crate::pathfind::RouteTracker::new(),
            walkto_draining: false,
            width_table: width_table.to_vec(),
            tile_walk: tile_walkability.to_vec(),
            afterimage_x: [0; 4],
            afterimage_y: [0; 4],
            view_locked: false,
            show_battle_digits: false,
            battle_digit_by_type: vec![0; 89],
            ease_cam_x: 0,
            ease_cam_y: 0,
            ease_target_x: 0,
            ease_target_y: 0,
        };
        scene.load_floor_data(maplv, player_px, player_py, records, scene.persist.floor)?;
        Ok(scene)
    }
    /// loadFloorData 的原地重载（换层遮幅中段 a.java:3189 调用同方法）：
    /// 重建 view/entities/grid/walkable，其余场景状态（玩家/HUD/RNG/popup）保留。
    pub fn load_floor_data(
        &mut self,
        maplv: &[u8],
        player_px: i32,
        player_py: i32,
        records: &[(u8, u16, u16, i16, bool)],
        floor: i32,
    ) -> Result<(), String> {
        // 【BUG-045④】换层清栈（旧层坐标的位移笔在新层无意义——复核探针 D：撤销把玩家传回上一层坐标）
        self.undo.clear(crate::undo_journal::ClearReason::FloorChange);
        self.clear_tap_state();
        // 【P0 回归修复 2026-10-20】换层重建视图**布局换轨收口**：from_maplv 显式
        // 带 ViewDims（编译强制=漏网-proof）——旧 classic 常量初始致换层残留
        // 240-era col/row 范围=modern 地形半绘根因（floor1 正常=初始已换轨、
        // 切层后半绘实证）。
        let view = GameView::from_maplv(
            maplv,
            player_px,
            player_py,
            crate::game_view::ViewDims::from_layout(&self.layout),
        )?;
        let cols = view.wide as usize;
        let rows = view.high as usize;
        let mut walkable = vec![vec![false; cols]; rows];
        for (y, row) in walkable.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                // rebuildWalkability（a.java:6964-6976）的索引：**行距 wide·4、
                // 列距 2**（py_key 行尾 +=wide<<1 叠加列循环的 ×2）——与 paintTileLayer
                // 的行距 wide·2、列距 1 是同一数组的两套读法（原版共存）。
                // A 级：oracle FLD 134（walkableGrid）13×13 逐格复算匹配。
                let idx = (y * cols * 4 + x * 2) as i32;
                let tile = view.terrain_at(0, idx) as usize;
                *cell = tile < self.tile_walk.len() && self.tile_walk[tile] == 1;
            }
        }
        let mut entities = EntityTable::new(100);
        let mut grid = CellGrid::new(view.wide, view.high);
        // m_122 双分支（a.java:9465+）：有层存档 → 从存档恢复（**死怪不复活**、
        // 已开门保持开）；无存档 → sprite{n} 权威数据 + 5/81/6/12/76/82/83 隐藏
        // 存档态按**目标层**取键（a.java:9465+ f_byte_arr2_05[py_key]）——跨层装载
        // （probe/ADD）时不得取当前层（潜伏缺陷 2026-10-15 修）
        if let Some(data) = self.persist.saved_floor_states[floor as usize].clone() {
            // m_122 存档分支核=entity::restore_floor_state（第 1 步文件合流）；
            // 恢复臂 index quirk（a.java:9638）随核保留。
            let img_h_of = |t: i32| self.images.entity.get(t as usize).cloned().flatten().map(|im| im.height);
            entity::restore_floor_state(&mut entities, &mut grid, &data, &self.width_table, &img_h_of);
            // 层特例 switch(currentFloor)（a.java:9648-9673）= RESTORE_RULES 表驱动
            //（第 4 步特例批配置化；case 12/39 一并入表——原"未移植"欠账清偿）。
            self.apply_restore_rules(&mut entities, &mut grid);
        } else {
            for &(type_code, x, y, param, visible) in records {
                let img_h = self.images.entity[type_code as usize].as_ref().map(|i| i.height);
                let idx = entity::spawn(
                    &mut entities,
                    &mut grid,
                    type_code as i32,
                    x as i32,
                    y as i32,
                    param,
                    &self.width_table,
                    img_h,
                );
                if !visible {
                    entities.items[idx].visible = false; // m_122 特例分支（5/81/6/12/76/82/83）
                }
            }
        }
        entity::sort_by_y(&mut entities, &mut grid);
        // m_122 尾（a.java:9669-9667）：type 72 类别 floor50=8、其余 32——
        // render_category 静态表只覆盖非 50 层（本批范围 1/2）
        self.view = view;
        self.entities = entities;
        self.grid = grid;
        self.walkable = walkable;
        self.minimap = None;
        Ok(())
    }
    /// changeFloor（a.java:7126-7164）：层号合法性（strict=键 49/55 路径；
    /// 楼梯路径 strict=false 跳过已达层检查）→ 置遮幅态 + 记录目标层。
    /// 越界提示浮层（miscTexts 0-3）属交互批次——todo 守卫。
    pub fn change_floor(&mut self, target: i32, down: bool, strict: bool) -> bool {
        if strict {
            // 已达层界提示（a.java:7133-7144）：下界 miscTexts[0]、上界 [1]
            if target < self.persist.min_floor {
                self.show_overlay_message(0, MISC_TEXTS[0], 0, 0);
                return false;
            }
            if target > self.persist.max_floor {
                self.show_overlay_message(0, MISC_TEXTS[1], 0, 0);
                return false;
            }
        }
        // 层界提示（a.java:7150-7156）：负层 [2]、超 floorCountMax=55 [3]（a.java:808）
        if target < 0 {
            self.show_overlay_message(0, MISC_TEXTS[2], 0, 0);
            return false;
        }
        if target > 55 {
            self.show_overlay_message(0, MISC_TEXTS[3], 0, 0);
            return false;
        }
        // m_119(currentFloor)（a.java:9370-9388）：序列化**存活**实体
        // （removed/state==1 跳过）——死怪/已开门跨层持久
        self.save_floor_state();
        self.transitioning = true;
        // > 【2026-10-26 更正·BUG-026】原此处 `transition_to(Map)`（注"changeFloor 起遮幅即回 mode 3，
        // > A-preset-item21 T402 实证"）系**调用方语义误挂**：Java changeFloor（a.java:7135-7180）
        // > 不写 gameMode；mode 3 来自道具 21 调用方 `changeFloor; keyHeldCode=0; gameMode=3`
        // > （a.java:7863-7866）。误挂使**脚本行走踩楼梯**（mode 11 中 MOV 走上梯格，如一层嫦娥
        // > 走位戏 script19 尾 `MOV_0_11_1 MOV_0_1_11`）切层即回 mode 3、丢弃剩余脚本——Java 在
        // > mode 11 续跑（fx-chang-e-floor1-taps T786：Java gameMode=11，2 层续走 MOV_0_1_11）。
        if target < self.persist.min_floor {
            self.persist.min_floor = target;
        } else if target > self.persist.max_floor {
            self.persist.max_floor = target;
            self.note_max_floor(target); // f_int_154 水位（a.java:7163-7164）
        }
        self.persist.pending_floor = target;
        self.persist.went_down = down;
        true
    }
    /// m_119（a.java:9373-9400）：层状态序列化。格式：u16 计数（BE，m_120）+
    /// 每存活实体 [type u8, x u16 BE, y u16 BE, visible u8, param u16 BE]。
    ///
    /// 槽先置 null（a.java:9383），**仅 entityCount > 0** 才分配写入（a.java:9384）
    /// ——实体表全空的层存 None（重入时走 sprite{n} 权威数据），而"有实体但全
    /// 死/已开"的层存 `[00 00]`（重入恢复 0 个）。注意判据是 entityCount（含已
    /// 移除槽位），非存活数：故存档恢复出 0 实体的层再次离开时 entityCount==0
    /// → None → 第三次进入从 sprite 重新生成全部实体（原版"清层复活"行为，保真）。
    pub(super) fn save_floor_state(&mut self) {
        // m_119：表操作核=entity::serialize_floor_state（第 1 步文件合流）；
        // count==0 ⇒ 存 null（a.java:9383-9384）留本层。
        if self.entities.count == 0 {
            self.persist.saved_floor_states[self.persist.floor as usize] = None;
            return;
        }
        let data = entity::serialize_floor_state(&self.entities);
        self.persist.saved_floor_states[self.persist.floor as usize] = Some(data);
    }
    /// 遮幅中段换层（a.java:3186-3206）：loadFloorData + 落点特例
    /// （0/50/1-上行 place_player_at_cell；其余 findFloorGateEntity → step_off_stairs）+ 小地图 + 步效果。
    pub(super) fn swap_floor(&mut self) {
        self.persist.floor = self.persist.pending_floor;
        self.persist.current_floor = self.persist.pending_floor;
        let fa = self.persist.floor_assets[self.persist.pending_floor as usize]
            .clone()
            .expect("层资产未提供");
        let px = self.view.player_px;
        let py = self.view.player_py;
        self.load_floor_data(&fa.maplv, px, py, &fa.records, self.persist.floor)
            .expect("层资产重载失败");
        if self.persist.floor == 0 {
            self.place_player_at_cell(1, 2);
            self.view.center_on_player();
        } else if self.persist.floor == 50 {
            self.place_player_at_cell(6, 7);
            self.view.center_on_player();
        } else if self.persist.floor == 1 && !self.persist.went_down {
            self.place_player_at_cell(6, 11);
            self.view.center_on_player();
        } else if let Some((gx, gy)) = self.find_floor_gate(self.persist.went_down) {
            self.step_off_stairs(gx, gy);
        }
        self.build_minimap();
        self.apply_step_cell_effects();
    }
    pub(super) fn find_floor_gate(&self, up: bool) -> Option<(i32, i32)> {
        let want = if up { 7 } else { 8 };
        for e in 0..self.entities.count {
            if self.entities.items[e].type_ == want && (self.entities.items[e].param >> 9) == 0 {
                return Some((self.entities.items[e].px >> 5, self.entities.items[e].py >> 5));
            }
        }
        None
    }
    pub fn reload_floor_entities(&mut self, floor: i32) {
        let mut entities = entity::EntityTable::new(100);
        let mut grid = entity::CellGrid::new(self.view.wide, self.view.high);
        if let Some(data) = self.persist.saved_floor_states[floor as usize].clone() {
            // m_122 存档分支核=entity::restore_floor_state（第 1 步文件合流）；
            // 恢复臂 index quirk（按循环下标回填 visible/removed，a.java:9638）随核保留。
            let img_h_of = |t: i32| self.images.entity.get(t as usize).cloned().flatten().map(|im| im.height);
            entity::restore_floor_state(&mut entities, &mut grid, &data, &self.width_table, &img_h_of);
            self.apply_restore_rules(&mut entities, &mut grid);
        } else if let Some(fa) = self.persist.floor_assets.get(floor as usize).cloned().flatten() {
            for &(type_code, x, y, param, visible) in &fa.records {
                let img_h = self.images.entity[type_code as usize].as_ref().map(|i| i.height);
                let idx = entity::spawn(
                    &mut entities,
                    &mut grid,
                    type_code as i32,
                    x as i32,
                    y as i32,
                    param,
                    &self.width_table,
                    img_h,
                );
                if !visible {
                    entities.items[idx].visible = false;
                }
            }
        }
        entity::sort_by_y(&mut entities, &mut grid);
        self.entities = entities;
        self.grid = grid;
        // 【BUG-045】实体表按活体重建并重排下标（跨层探针 m_119/m_122 往返）⇒ 撤销笔里的实体槽号失效（复核 T3-R3）⇒ 清栈
        self.undo.clear(crate::undo_journal::ClearReason::Irreversible);
    }
    /// probeCrossFloorSnapshot（a.java:8640-8659）：跨层探存活实体——忠实往返：
    /// m_119 存当前 → m_122（reload_floor_entities）载目标 → 扫 → 还原（不存目标层）。
    /// 结果以真装载后的实体表为准（floor23 存档态无型12 ⇒ false，与参照跑
    /// T281 mode=3 吻合）。
    pub fn probe_cross_floor_snapshot(&mut self, floor: i32, type_id: i32) -> bool {
        let foreign = self.persist.floor != floor;
        if foreign {
            self.save_floor_state();
            self.reload_floor_entities(floor);
        }
        let mut found = false;
        for e in 0..self.entities.count {
            if !self.entities.items[e].removed && self.entities.items[e].type_ == type_id {
                found = true;
                break;
            }
        }
        if foreign {
            self.reload_floor_entities(self.persist.floor);
        }
        found
    }
    /// loadLevelScript（a.java:8035-8052）：mode 11 + 脚本行装载。
    /// case 31 层重排特例（a.java:8028-8028）未达（script 40 不涉），todo 守卫。
    pub fn load_level_script(&mut self, idx: i32) {
        // 【2026-10-26·BUG-025】剧情接管 = 收束触屏寻路段（WalkTo 走进剧情点）：段收于实际终点、清余步。
        // 否则剧情中投键（press_key 见段开放即打断）截断脚本 MOV 行走，dp3 步尽臂还会把
        // 剧情中途切回 Map。经典面（无寻路段）interrupt_route 零操作。
        self.interrupt_route();
        // 【BUG-045④】剧情推进不可撤（undo_journal 清栈边界：MOV / 脚本事件触发 = 清栈）
        self.undo.clear(crate::undo_journal::ClearReason::ScriptEvent);
        self.clear_tap_state();
        // 【BUG-043】寻路被剧情接管：箭头暂存**不在此放回**——撞格触发剧情时本函数在 advanceScriptWalk 撞格臂之前执行，
        // 立即放回会被随后的 `walk_step_count = 0` 清掉（复核 R2）；改由 settle_walkto_arrows 在寻路释放 dp 后放回
        self.walkto_draining = false; // 【BUG-040】dp 归剧情（打断后的寻路收尾作废）
        self.script_walk_armed = false;
        self.transition_to(GameMode::Dialog);
        self.dialog.dialog_page = 0;
        self.lock_camera_on(0); // ease 相机初始化到玩家
        self.dialog.dialog_phase = 0;
        self.current_script_index = idx as usize;
        self.script_line = self.dialog.dialog_data.scripts[idx as usize].clone();
        self.dialog.overlay_text = None;
        self.script_cursor = 0;
        // scriptLineFlags[idx] 在行尾收尾（AwaitCamera）或 GUT/DES 时置位——
        // loadLevelScript 本身不置（a.java:8035-8052 无 flag 写）
    }
    /// saveFloorState 写侧（a.java:9182-9244）：blob 序 = parse_floor_save 读序
    /// 的镜像（difficulty..follow 对/128 旗标/道具栈/56 层触发）。
    /// 装备位恒 0：equippedWeaponType/ArmorType 为装备批次字段（当前层恒无装备）。
    /// 【双归域·编排注记归因】槽 blob 序列化=跨域编排面（persist+player+装备+坐标逐参写盘）——
    /// **不可迁**（非域内闭合）；coordination 面显式留置。
    pub(super) fn serialize_slot_blob(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.push(self.menu.difficulty_index as u8);
        o.push(self.persist.floor as u8);
        o.push(self.persist.min_floor as u8);
        o.push(self.persist.max_floor as u8);
        o.push(self.facing as u8);
        o.push(self.equipped_weapon as u8); // equippedWeaponType（a.java:9192 写真实值）
        o.push(self.equipped_armor as u8); // equippedArmorType（a.java:9193）
        // writeShort(playerCellX/Y)（a.java:9197-9198）：**2 字节**——旧写法
        // `(px>>5).to_be_bytes()` 是 i32 四字节，令其后全部字段错位 4 字节
        o.extend_from_slice(&(self.player_cell_x as i16).to_be_bytes());
        o.extend_from_slice(&(self.player_cell_y as i16).to_be_bytes());
        for v in [self.player.hp, self.player.atk, self.player.def] {
            o.extend_from_slice(&v.to_be_bytes());
        }
        for v in [
            self.player.yellow_keys as i16,
            self.player.blue_keys as i16,
            self.player.red_keys as i16,
        ] {
            o.extend_from_slice(&v.to_be_bytes());
        }
        o.extend_from_slice(&self.player.gold.to_be_bytes());
        // writeShort(alchemyUpgradeCount)（a.java:9206；旧写法恒 0）
        o.extend_from_slice(&(self.alchemy_upgrade_count as i16).to_be_bytes());
        // writeShort(inventoryEntryCount，原 inventoryEntryCount) + f_byte_arr_14/15 对（a.java:9207-9212）= mode12 日志
        //（读侧 a.java:9278-9283 → journal；旧写法取恒空的 special_item_types）
        o.extend_from_slice(&(self.persist.journal.len() as i16).to_be_bytes());
        for &(kind, idx) in &self.persist.journal {
            o.push(kind as u8);
            o.push(idx as u8);
        }
        // 128 字节脚本旗标（不足补 false）
        for k in 0..128 {
            let v = self.script_line_flags.get(k).copied().unwrap_or(false);
            o.push(v as u8);
        }
        // 道具栈
        o.push(self.player.item_stack_size as u8);
        for k in 0..self.player.item_stack_size as usize {
            o.push(self.player.item_types[k]);
            o.push(self.player.item_uses[k]);
        }
        // 56 层触发状态
        for k in 0..56usize {
            match &self.persist.saved_floor_states[k] {
                Some(d) => {
                    o.extend_from_slice(&(d.len() as i16).to_be_bytes());
                    o.extend_from_slice(d);
                }
                None => o.extend_from_slice(&(-1i16).to_be_bytes()),
            }
        }
        o
    }
}
