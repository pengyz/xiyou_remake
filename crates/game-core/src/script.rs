//! 脚本 DSL 解释器 —— Java `executeScriptInstruction` 的忠实移植。
//!
//! 合同：docs/spec/p3-script-interpreter.md；语义证据 = reference/src/deobf/a.java
//! executeScriptInstruction（:8216-8564）/ openDialogPage（:8185-8210）/
//! loadLevelScript（:7997-8012）。字段名与 data/naming 台账语义名对齐。
//!
//! 边界（P3.1）：实体/相机/地图副作用经 [`HostCtx`] trait 注入（P3.2 实体模块
//! 落地真实实现）；MVS 数据域零使用，显式 P3.2 边界。批1：实体面 9 方法迁
//! [`crate::subsystem::entity_service::EntityService`]（HostCtx 拆小首件）。

use crate::subsystem::entity_service::EntityService;
use crate::subsystem::item_service::ItemService;
use crate::subsystem::dialog::{DialogCtx, DialogStep, DialogSubsystem};
use crate::subsystem::script_host::ScriptOp;

/// 对白数据表（来源：deobf a.java:1124/1194 夹具，tests/fixtures/）。
pub struct DialogueTable {
    pub scripts: Vec<String>,
    pub texts: Vec<String>,
    pub speakers: Vec<i8>,
    pub type_names: Vec<String>,
    /// 类型→类目表（Java renderCategoryByType，a.java:6227-6269 初始化；CES 延迟查此表）
    pub type_categories: Vec<u8>,
}

/// 脚本层可写字段的状态集（Java god class 中被 executeScriptInstruction /
/// openDialogPage / loadLevelScript 触碰的字段子集；命名对齐台账）。
use crate::enums::DialogPhase;
use crate::enums::GameMode;
use crate::enums::WalkPhase;

pub struct ScriptState {
    pub current_script_index: usize,
    pub script_cursor: usize,
    pub script_line_flags: Vec<bool>,
    pub dialog_phase: DialogPhase,
    pub dialog_page: i32,
    pub dialog_char_pos: i32,
    pub speaker: i32,
    pub speaker_entity: i32,
    pub overlay_text: Option<String>,
    pub tak_page_start: i32,
    pub tak_page_end: i32,
    pub camera_anchor_x: i32,
    pub camera_anchor_y: i32,
    pub scene_delay: i32,
    pub game_mode: GameMode,
    /// 转移计数（件 A' 收口观测，harness 面）
    pub transition_count: u32,
    /// 转移日志（件 A' 收口观测，harness 面）
    pub last_transition: Option<(GameMode, GameMode)>,
    pub script_walk_armed: bool,
    pub walk_request_flag: bool,
    pub walk_phase: WalkPhase,
    pub layer_byte: i32,
    pub difficulty_index: i32,
    pub current_floor: i32,
    pub player_cell_x: i32,
    pub player_cell_y: i32,
    pub player_hp: i32,
    pub player_atk: i32,
    pub player_def: i32,
    pub yellow_keys: i32,
    pub blue_keys: i32,
    pub red_keys: i32,
    pub gold: i32,
    pub item_stack_size: i32,
    pub alchemy_upgrade_count: i32,
    pub min_floor_reached: i32,
    pub max_floor_reached: i32,
    pub facing_direction: i32,
    /// floorTransitioning（语义待考证：LAY_ 置位 + 对白行走门控共用，机械名保留）
    pub flag_bool_16: bool,
    /// 玩家像素坐标（ROS_1 直写；行走模块 P3.2 接管）
    pub player_pixel_x: i32,
    pub player_pixel_y: i32,
    /// inventoryEntryCount：背包条目数（saveFloorState 存档按其写条目对）
    pub inventory_entry_count: i32,
    /// 解析终位（Java `lastDelimPos` 同构物：分隔符所在下标）。
    pub last_delim_pos: usize,
}

impl Default for ScriptState {
    fn default() -> Self {
        Self {
            current_script_index: 0,
            script_cursor: 0,
            script_line_flags: Vec::new(),
            dialog_phase: DialogPhase::ScriptStep,
            dialog_page: 0,
            dialog_char_pos: 0,
            speaker: 0,
            speaker_entity: -1,
            overlay_text: None,
            tak_page_start: 0,
            tak_page_end: 0,
            camera_anchor_x: 0,
            camera_anchor_y: 0,
            scene_delay: 0,
            game_mode: GameMode::Boot,
            transition_count: 0,
            last_transition: None,
            script_walk_armed: false,
            walk_request_flag: false,
            walk_phase: WalkPhase::Idle,
            layer_byte: 0,
            difficulty_index: 0,
            current_floor: 0,
            // 默认位形对齐序章冻结态（gameplay/BUG-006 trace 实证 cell(1,11)）
            player_cell_x: 1,
            player_cell_y: 11,
            player_hp: 0,
            player_atk: 0,
            player_def: 0,
            yellow_keys: 0,
            blue_keys: 0,
            red_keys: 0,
            gold: 0,
            item_stack_size: 0,
            alchemy_upgrade_count: 0,
            min_floor_reached: 0,
            max_floor_reached: 0,
            facing_direction: 0,
            flag_bool_16: false,
            player_pixel_x: 0,
            player_pixel_y: 0,
            inventory_entry_count: 0,
            last_delim_pos: 0,
        }
    }
}

/// 实体/相机/系统副作用边界（P3.2 由实体模块提供真实实现；测试用确定性桩）。
/// 批1：实体面 9 方法已迁 crate::subsystem::entity_service::EntityService（拆小首件），余 12 暂留。
/// 批3：对话面 2 方法已迁 crate::subsystem::dialog::DialogSubsystem（Host 边界），余 10 暂留。
pub trait HostCtx {
    fn find_path(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool;
    // 【批6 摘 2】give_gold/pickup_item 已逐字迁 crate::subsystem::item_service::ItemService（HostCtx 余 8）。
    /// lockCameraOn 相机部分（无条件）：相机锚 = 玩家像素位。
    fn snap_camera_to_player(&mut self);
    /// lockCameraOn 说话人实体定位 → dialogAnchorEntity。
    fn locate_speaker_entity(&mut self, speaker: i32) -> i32;
    /// CES 尾部：相机 = 玩家屏幕位；若实体存在则 target = 实体像素位。
    fn ces_camera(&mut self, entity_idx: i32);
    /// easeCameraToTarget：SEE 的相机目标设置。
    fn set_camera_anchor_target(&mut self, x: i32, y: i32);
    /// CMidlet.dispatchModeEntry：退出应用。
    fn exit_application(&mut self);
    /// MVS（P3.2：deobf a.java:8513-8581；数据域 68 条脚本零使用）。
    fn mvs_exchange(&mut self);
    /// ROS_1 后 lockCameraOn(0)：相机锚 = 新玩家像素位。
    fn snap_camera_after_ros1(&mut self, px: i32, py: i32);
}

pub struct ScriptEngine<'d> {
    data: &'d DialogueTable,
    pub state: ScriptState,
    /// 测试/调试辅助：覆盖当前脚本行文本（正常流程经 load_level_script）。
    pub script_line_override: Option<String>,
}

impl<'d> ScriptEngine<'d> {
    pub fn new(data: &'d DialogueTable, script_index: usize) -> Self {
        let mut state = ScriptState::default();
        state.script_line_flags = vec![false; data.scripts.len()];
        let mut eng = Self { data, state, script_line_override: None };
        eng.load_level_script(&mut NoopHost, script_index);
        eng
    }

    /// 测试/调试辅助：覆盖脚本行文本（正常数据流经 DialogueTable）。
    /// 模式转移收口（件 A'，harness 面）：ScriptState.game_mode 唯一写点——
    /// 与 GameScene::transition_to 同口径（转移计数/日志观测字段）。
    fn state_transition(&mut self, mode: GameMode) {
        let from = self.state.game_mode;
        self.state.transition_count += 1;
        self.state.last_transition = Some((from, mode));
        self.state.game_mode = mode;
    }

    pub fn set_script_line_for_test(&mut self, text: &str) {
        self.script_line_override = Some(text.to_string());
    }

    fn line(&self) -> String {
        match &self.script_line_override {
            Some(s) => s.clone(),
            None => self.data.scripts[self.state.current_script_index].clone(),
        }
    }

    /// GUT_ 目标装载 = Java loadLevelScript（a.java:8020-8029）。
    /// 批1 界收窄：snap_camera（HostCtx）+reload_floor_entities（EntityService）双面。
    pub fn load_level_script(&mut self, host: &mut (impl HostCtx + EntityService), idx: usize) {
        self.state.script_walk_armed = false;
        self.state_transition(GameMode::Dialog);
        self.state.current_script_index = idx;
        self.state.dialog_page = 0;
        host.snap_camera_to_player(); // lockCameraOn(0)
        self.state.dialog_phase = DialogPhase::ScriptStep;
        self.state.overlay_text = None;
        self.state.script_cursor = 0;
        // Java case 31 特例（地图/实体重排，a.java:8028-8028）：
        // 层重排属实体域，P3.1 以层重载 hook 表达可见副作用的最小保真。
        if idx == 31 {
            host.reload_floor_entities(self.state.current_floor);
        }
    }

    /// 对白页推进（玩家按键语义的引擎侧入口）= Java openDialogPage(page+1)。
    pub fn advance_dialog_page(&mut self, host: &mut (impl HostCtx + DialogSubsystem)) {
        let next = self.state.dialog_page + 1;
        self.open_dialog_page(host, next);
    }

    /// openDialogPage 忠实移植（a.java:8208-8227）。
    pub fn open_dialog_page(&mut self, host: &mut (impl HostCtx + DialogSubsystem), page: i32) {
        if page <= self.state.tak_page_end {
            self.state.dialog_page = page;
            self.state.speaker = if page < 0 {
                -1
            } else {
                self.data.speakers[page as usize] as i32
            };
            self.state.speaker_entity = host.locate_speaker_entity(self.state.speaker);
            host.snap_camera_to_player(); // lockCameraOn 头部相机部分（无条件）
            let composed = if self.state.speaker < 0 {
                format!("?: \\cF8F8F8{}", self.data.texts[page as usize])
            } else {
                let name = &self.data.type_names[self.state.speaker as usize];
                format!("{}: \\cF8F8F8{}", name, self.data.texts[page as usize])
            };
            // Java String.length() = UTF-16 单位数（复核 R5）：CJK 名须按字符计数
            self.state.dialog_char_pos = if self.state.speaker < 0 {
                11
            } else {
                (self.data.type_names[self.state.speaker as usize].chars().count() + 10) as i32
            };
            host.set_typewriter(&composed, 129, self.state.dialog_char_pos);
            self.state.overlay_text = Some(composed);
        } else {
            self.state.dialog_phase = DialogPhase::ScriptStep;
            self.state.dialog_char_pos = 0;
        }
    }

    /// 主循环步进：dialogPhase==0 时消费脚本（Java run dialogPhase 0 分支：
    /// scriptStepDelay>0 递减否则 execute，a.java:4000-4003 的镜像）。
    /// 批1 界收窄：透传 execute_script_instruction（EntityService 面）。
    pub fn step(&mut self, host: &mut (impl HostCtx + EntityService + DialogSubsystem + ItemService)) {
        if self.state.dialog_phase == DialogPhase::ScriptStep {
            if self.state.scene_delay > 0 {
                self.state.scene_delay -= 1;
            } else {
                self.execute_script_instruction(host);
            }
        }
    }

    /// executeScriptInstruction 忠实移植（a.java:8239-8581）。
    ///
    /// 【2026-11-17 S3·① 解析统一】本函数不再内联 15 个 `table_count == "XX"`
    /// 解析分支（旧实现逐 op 自写 `parse_script_int`），改为与生产执行器共用同一
    /// 纯解析函数 [`crate::scene::GameScene::parse_op`]，自己只保留「入口门 +
    /// op 分派 + 宿主副作用」。解析面因此单一来源：槽67 尾空格、SWD/MVS 游标
    /// 差一格一类「两侧解析漂移」缺陷在结构上不再可能。
    pub fn execute_script_instruction(
        &mut self,
        host: &mut (impl HostCtx + EntityService + DialogSubsystem + ItemService),
    ) {
        let line = self.line();
        let script_cursor0 = self.state.script_cursor;
        if script_cursor0 + 3 > line.len() {
            // 入口门（javap 偏移 22 的 `cursor+3 > len` ⇒ 2401 门体）：序章 cursor=43 冻结 phase=4 实证
            self.state.dialog_phase = DialogPhase::AwaitCamera;
            if self.state.current_script_index != 32 {
                let cur = self.state.current_script_index;
                self.state.script_line_flags[cur] = true;
            }
            host.snap_camera_to_player();
            return;
        }
        // 纯解析（与生产同一函数）；未知 op ⇒ 静默停摆（a.java:8594-8595 原地 return，
        // 无臂写 scriptCursor ⇒ 下一拍重解析同一 token）
        let Some(op) = crate::scene::GameScene::parse_op(&line, script_cursor0) else {
            return;
        };
        self.exec_op(host, op);
    }

    /// op 分派（旧内联体逐 op 归位）：游标统一按解析层算好的值写（含 SWD/MVS 的
    /// `orig+4` 口径），其余为宿主副作用。逐 op 对齐生产 `GameScene::execute_op`。
    fn exec_op(
        &mut self,
        host: &mut (impl HostCtx + EntityService + DialogSubsystem + ItemService),
        op: ScriptOp,
    ) {
        self.state.script_cursor = op.cursor();
        match op {
            ScriptOp::Tak { start, end, .. } => {
                self.state.tak_page_start = start;
                self.state.tak_page_end = end;
                self.state.dialog_phase = DialogPhase::Typewriter;
                self.open_dialog_page(host, start);
            }
            ScriptOp::Mov { target_type, cell, dst, .. } => {
                self.state.camera_anchor_x = dst.0;
                self.state.camera_anchor_y = dst.1;
                if target_type > 0 {
                    let (cx, cy) = cell.expect("五参已解析");
                    let idx = host.cell_entity(cx, cy, target_type);
                    if idx >= 0 {
                        if host.entity_type(idx) == crate::enums::entity_kind::ROUTE {
                            host.set_entity_route_flag(idx);
                        }
                        let (px, py) = host.entity_pixel(idx);
                        host.find_path(px >> 5, py >> 5, dst.0, dst.1);
                        // Java 无条件置 2（a.java:8284，findPath 结果不影响 phase）
                        self.state.dialog_phase = DialogPhase::EntityRoute;
                    } else {
                        // 实体未找到 ⇒ 同拍尾递归执行下一条（a.java:8287）
                        self.execute_script_instruction(host);
                    }
                } else if host.find_path(
                    self.state.player_cell_x,
                    self.state.player_cell_y,
                    dst.0,
                    dst.1,
                ) {
                    // a.java:8288-8284：三写点缺一不可（复核 R2）
                    self.state.script_walk_armed = true;
                    self.state.dialog_phase = DialogPhase::Walk;
                    self.state.walk_phase = crate::enums::WalkPhase::Idle;
                    host.snap_camera_to_player(); // lockCameraOn(0)
                }
            }
            ScriptOp::Gut { target, .. } => {
                let cur = self.state.current_script_index;
                self.state.script_line_flags[cur] = true;
                self.load_level_script(host, target as usize);
            }
            ScriptOp::DesAll { want, .. } => host.remove_all_of_type(want),
            ScriptOp::DesAt { ty, cx, cy, .. } => {
                let idx = host.cell_entity(cx, cy, ty);
                if idx >= 0 {
                    host.remove_entity(idx);
                }
            }
            ScriptOp::Swd { .. } => host.swd_convert_type81(),
            ScriptOp::Lay { floor, .. } => {
                host.reload_floor_entities(self.state.current_floor); // m_119
                self.state.layer_byte = floor;
                self.state.flag_bool_16 = true; // a.java:8340（复核 R7）
                self.state.overlay_text = None;
                self.state.dialog_phase = DialogPhase::LayCutscene;
            }
            ScriptOp::RosMove { px, py, .. } => {
                // ROS_1（a.java:8348-8347）：玩家像素坐标直写 + lockCameraOn(0)（复核 R9）
                self.state.player_pixel_x = px << 5;
                self.state.player_pixel_y = py << 5;
                host.snap_camera_after_ros1(self.state.player_pixel_x, self.state.player_pixel_y);
            }
            ScriptOp::RosEquip { tier, .. } => host.m_082_set_dialog_target(tier),
            ScriptOp::RosHp { v, .. } => self.state.player_hp = v,
            ScriptOp::RosFace { v, .. } => self.state.facing_direction = v,
            ScriptOp::RosAtk { v, .. } => self.state.player_atk = v,
            ScriptOp::RosDef { v, .. } => self.state.player_def = v,
            ScriptOp::RosNoop { .. } => {}
            ScriptOp::Ces { ty, cx, cy, .. } => {
                let idx = host.spawn_entity(ty, cx << 5, cy << 5);
                host.ces_camera(idx);
                // scene_delay：switch(renderCategoryByType[type]) 类目表（a.java:8390；
                // 类目表初始化 a.java:6227-6269——复核 R3 纠正：查类目非裸 type）
                self.state.scene_delay = crate::enums::Category::from_raw(
                    self.data.type_categories[ty as usize],
                )
                .scene_delay();
            }
            ScriptOp::Gin { kind, value, .. } => {
                match kind {
                    0 => host.pickup_item(value),
                    1 => host.give_gold(value),
                    _ => {}
                }
                self.execute_script_instruction(host); // 唯一显式尾递归（a.java:8420）
            }
            ScriptOp::Add { floor, ty, cx, cy, .. } => {
                if self.state.current_floor != floor {
                    host.reload_floor_entities(self.state.current_floor);
                    host.reload_floor_entities(floor);
                }
                let _ = host.spawn_entity(ty, cx << 5, cy << 5);
                host.reload_floor_entities(floor);
                if self.state.current_floor != floor {
                    host.reload_floor_entities(self.state.current_floor);
                }
            }
            ScriptOp::Glv { .. } => {}
            ScriptOp::Res { .. } => {
                if self.state.difficulty_index == 0 {
                    self.state.yellow_keys = 0;
                    self.state.blue_keys = 0;
                    self.state.red_keys = 0;
                    self.state.item_stack_size = 0;
                    self.state.gold = 4;
                    self.state.alchemy_upgrade_count = 0;
                    self.state.min_floor_reached = 1;
                    self.state.max_floor_reached = 3;
                    self.state.player_atk = 10;
                    self.state.player_def = 10;
                    self.state.player_hp = 400;
                    self.state.inventory_entry_count = 0; // inventoryEntryCount=0（a.java:8465，复核 R6）
                }
            }
            ScriptOp::See { tx, ty, page, page_end, flag, .. } => {
                self.state.camera_anchor_x = tx;
                self.state.camera_anchor_y = ty;
                self.state.tak_page_start = page;
                self.state.tak_page_end = page_end;
                self.open_dialog_page(host, page);
                // Java switch(seeModeFlag) case 1 落穿 case 0/default（a.java:8479-8481）
                if flag == 1
                    && host.find_path(
                        self.state.player_cell_x,
                        self.state.player_cell_y,
                        self.state.camera_anchor_x,
                        self.state.camera_anchor_y,
                    )
                {
                    self.state.walk_request_flag = true;
                    self.state.walk_phase = crate::enums::WalkPhase::Idle;
                }
                self.state.dialog_phase = DialogPhase::TypewriterCamera;
                host.set_camera_anchor_target(
                    self.state.camera_anchor_x,
                    self.state.camera_anchor_y,
                );
            }
            ScriptOp::End { .. } => {
                self.state_transition(crate::enums::GameMode::Ending);
                host.exit_application();
            }
            ScriptOp::Sms { .. } => {}
            ScriptOp::Mvs { .. } => host.mvs_exchange(),
        }

        // 【2026-11-16 删门·字节码裁定】行尾检查不是收尾门：javap
        // executeScriptInstruction（偏移 0–2431，analysis/rename-pipeline/renamed/a.class）
        //   2397 if_icmpge 2431 / 2400 return / 2431 return —— 两分支均直接返回、零字段写；
        //   2401 的 `dialogPhase=4 + flags[idx] + lockCameraOn(0)` 只由偏移 22 的
        //   `cursor+3 > len` 到达，即**下一拍入口门**（本函数开头）。
        // 裁定全文见 docs/superpowers/proposals/2026-11-16-script-command-pipeline-proposal.md §0′.4′。
    }

}

/// 无操作宿主（new() 初始化路径用——load_level_script 会调 snap_camera）。
pub struct NoopHost;
impl HostCtx for NoopHost {
    fn find_path(&mut self, _: i32, _: i32, _: i32, _: i32) -> bool { false }
    fn snap_camera_to_player(&mut self) {}
    fn locate_speaker_entity(&mut self, _: i32) -> i32 { -1 }
    fn ces_camera(&mut self, _: i32) {}
    fn set_camera_anchor_target(&mut self, _: i32, _: i32) {}
    fn exit_application(&mut self) {}
    fn mvs_exchange(&mut self) {}
    fn snap_camera_after_ros1(&mut self, _: i32, _: i32) {}
}
impl DialogSubsystem for NoopHost {
    fn tick(&mut self, _: &mut DialogCtx) -> DialogStep { DialogStep::Wait }
    fn set_typewriter(&mut self, _: &str, _: i32, _: i32) {}
    fn m_082_set_dialog_target(&mut self, _: i32) {}
}
impl ItemService for NoopHost {
    fn pickup(&mut self, _: &mut crate::effect_registry::ItemCtx, _: i32) -> crate::effect_registry::ItemOutcome {
        crate::effect_registry::ItemOutcome::NoOp // pickup=resolver 服务面——host 不服务
    }
    fn give_gold(&mut self, _: i32) {}
    fn pickup_item(&mut self, _: i32) {}
}
impl EntityService for NoopHost {
    fn cell_entity(&self, _: i32, _: i32, _: i32) -> i32 { -1 }
    fn entity_type(&self, _: i32) -> i32 { -1 }
    fn remove_entity(&mut self, _: i32) {}
    fn remove_all_of_type(&mut self, _: i32) {}
    fn swd_convert_type81(&mut self) {}
    fn entity_pixel(&self, _: i32) -> (i32, i32) { (0, 0) }
    fn reload_floor_entities(&mut self, _: i32) {}
    fn spawn_entity(&mut self, _: i32, _: i32, _: i32) -> i32 { -1 }
    fn set_entity_route_flag(&mut self, _: i32) {}
}
