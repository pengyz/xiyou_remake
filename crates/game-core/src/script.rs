//! 脚本 DSL 解释器 —— Java `executeScriptInstruction` 的忠实移植。
//!
//! 合同：docs/spec/p3-script-interpreter.md；语义证据 = reference/src/deobf/a.java
//! executeScriptInstruction（:8216-8564）/ openDialogPage（:8185-8210）/
//! loadLevelScript（:7997-8012）。字段名与 data/naming 台账语义名对齐。
//!
//! 边界（P3.1）：实体/相机/地图副作用经 [`HostCtx`] trait 注入（P3.2 实体模块
//! 落地真实实现）；MVS 数据域零使用，显式 P3.2 边界。

/// 对白数据表（来源：deobf a.java:1124/1194 夹具，tests/fixtures/）。
pub struct DialogueTable {
    pub scripts: Vec<String>,
    pub texts: Vec<String>,
    pub speakers: Vec<i8>,
    pub type_names: Vec<String>,
    /// 类型→类目表（Java f_byte_arr_03，a.java:6204-6252 初始化；CES 延迟查此表）
    pub type_categories: Vec<u8>,
}

/// 脚本层可写字段的状态集（Java god class 中被 executeScriptInstruction /
/// openDialogPage / loadLevelScript 触碰的字段子集；命名对齐台账）。
use crate::enums::DialogPhase;
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
    pub game_mode: i32,
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
    /// f_bool_16（语义待考证：LAY_ 置位 + 对白行走门控共用，机械名保留）
    pub flag_bool_16: bool,
    /// 玩家像素坐标（ROS_1 直写；行走模块 P3.2 接管）
    pub player_pixel_x: i32,
    pub player_pixel_y: i32,
    /// f_int_89：背包条目数（m_114 存档按其写条目对）
    pub inventory_entry_count: i32,
    /// 解析终位（Java `f_int_122` 同构物：分隔符所在下标）。
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
            game_mode: 0,
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
pub trait HostCtx {
    fn find_path(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool;
    /// m_100：格 (x,y) 上 type_id 类型的实体索引（无则 -1）。
    fn cell_entity(&self, x: i32, y: i32, type_id: i32) -> i32;
    /// 实体类型读取（entityType[idx]）。
    fn entity_type(&self, idx: i32) -> i32;
    /// m_047：移除实体。
    fn remove_entity(&mut self, idx: i32);
    /// DES 负参变体：按类型批量移除（Java 全表遍历，Host 侧持有实体表）。
    fn remove_all_of_type(&mut self, type_id: i32);
    /// SWD：类型 81 → 4 改写（Java 写五字段：hidden/texture/solid/anim/param）。
    fn swd_convert_type81(&mut self);
    fn entity_pixel(&self, idx: i32) -> (i32, i32);
    fn give_gold(&mut self, amount: i32);
    fn pickup_item(&mut self, item_id: i32);
    /// m_104 相机部分（无条件）：相机锚 = 玩家像素位。
    fn snap_camera_to_player(&mut self);
    /// m_104 说话人实体定位 → f_int_119。
    fn locate_speaker_entity(&mut self, speaker: i32) -> i32;
    /// m_018：打字机装填。
    fn set_typewriter(&mut self, text: &str, y: i32, pos: i32);
    /// m_119/m_122：层实体表重载。
    fn reload_floor_entities(&mut self, floor: i32);
    /// m_048：像素坐标生成实体 → 索引。
    fn spawn_entity(&mut self, type_id: i32, px: i32, py: i32) -> i32;
    /// CES 尾部：相机 = 玩家屏幕位；若实体存在则 target = 实体像素位。
    fn ces_camera(&mut self, entity_idx: i32);
    /// m_103：SEE 的相机目标设置。
    fn set_camera_anchor_target(&mut self, x: i32, y: i32);
    /// CMidlet.m_000：退出应用。
    fn exit_application(&mut self);
    /// MVS（P3.2：deobf a.java:8490-8564；数据域 68 条脚本零使用）。
    fn mvs_exchange(&mut self);
    /// ROS_2：m_082（对白目标设置，实体域）。
    fn m_082_set_dialog_target(&mut self, v: i32);
    /// MOV 5 参分支：实体 72 的路线标志写（f_int_arr_12[idx]=1，a.java:8252-8254）。
    fn set_entity_route_flag(&mut self, idx: i32);
    /// ROS_1 后 m_104(0)：相机锚 = 新玩家像素位。
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
    pub fn set_script_line_for_test(&mut self, text: &str) {
        self.script_line_override = Some(text.to_string());
    }

    fn line(&self) -> String {
        match &self.script_line_override {
            Some(s) => s.clone(),
            None => self.data.scripts[self.state.current_script_index].clone(),
        }
    }

    /// GUT_ 目标装载 = Java loadLevelScript（a.java:7997-8012）。
    pub fn load_level_script(&mut self, host: &mut impl HostCtx, idx: usize) {
        self.state.script_walk_armed = false;
        self.state.game_mode = 11;
        self.state.current_script_index = idx;
        self.state.dialog_page = 0;
        host.snap_camera_to_player(); // m_104(0)
        self.state.dialog_phase = DialogPhase::ScriptStep;
        self.state.overlay_text = None;
        self.state.script_cursor = 0;
        // Java case 31 特例（地图/实体重排，a.java:8005-8011）：
        // 层重排属实体域，P3.1 以层重载 hook 表达可见副作用的最小保真。
        if idx == 31 {
            host.reload_floor_entities(self.state.current_floor);
        }
    }

    /// 对白页推进（玩家按键语义的引擎侧入口）= Java m_096(page+1)。
    pub fn advance_dialog_page(&mut self, host: &mut impl HostCtx) {
        let next = self.state.dialog_page + 1;
        self.open_dialog_page(host, next);
    }

    /// openDialogPage 忠实移植（a.java:8185-8210）。
    pub fn open_dialog_page(&mut self, host: &mut impl HostCtx, page: i32) {
        if page <= self.state.tak_page_end {
            self.state.dialog_page = page;
            self.state.speaker = if page < 0 {
                -1
            } else {
                self.data.speakers[page as usize] as i32
            };
            self.state.speaker_entity = host.locate_speaker_entity(self.state.speaker);
            host.snap_camera_to_player(); // m_104 头部相机部分（无条件）
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
    /// f_int_118>0 递减否则 execute，a.java:3991-3999 的镜像）。
    pub fn step(&mut self, host: &mut impl HostCtx) {
        if self.state.dialog_phase == DialogPhase::ScriptStep {
            if self.state.scene_delay > 0 {
                self.state.scene_delay -= 1;
            } else {
                self.execute_script_instruction(host);
            }
        }
    }

    /// parseScriptInt 忠实移植（a.java:8560-8567）：
    /// 从 from 起找 delim，值 = substring，解析终位（Java f_int_122）= 分隔符位。
    fn parse_script_int(&mut self, line: &str, from: usize, delim: char) -> i32 {
        let rel = line[from..].find(delim).expect("脚本整数的分隔符缺失");
        let d = from + rel;
        self.state.last_delim_pos = d;
        line[from..d].parse().expect("脚本整数解析失败")
    }

    /// executeScriptInstruction 忠实移植（a.java:8216-8564）。
    pub fn execute_script_instruction(&mut self, host: &mut impl HostCtx) {
        let line = self.line();
        let var2 = self.state.script_cursor;
        let has_opcode = var2 + 3 <= line.len();
        if !has_opcode {
            // 行尾收尾门（a.java:8551-8563）：序章 cursor=43 冻结 phase=4 实证
            self.state.dialog_phase = DialogPhase::AwaitCamera;
            if self.state.current_script_index != 32 {
                let cur = self.state.current_script_index;
                self.state.script_line_flags[cur] = true;
            }
            host.snap_camera_to_player();
            return;
        }
        let var3 = &line[var2..var2 + 3];
        let var2p4 = var2 + 4;

        if var3 == "TAK" {
            self.state.tak_page_start = self.parse_script_int(&line, var2p4, '_');
            let p = self.state.last_delim_pos;
            self.state.tak_page_end = self.parse_script_int(&line, p + 1, ' ');
            self.state.script_cursor = self.state.last_delim_pos + 1;
            self.state.dialog_phase = DialogPhase::Typewriter;
            let page = self.state.tak_page_start;
            self.open_dialog_page(host, page);
        } else if var3 == "MOV" {
            let mut p = var2p4;
            let target_type = self.parse_script_int(&line, p, '_');
            p = self.state.last_delim_pos;
            // 5 参形态 y 也用 '_' 分隔（a.java:8239-8241；真实数据 MOV_72_3_7_1_8）
            let mut cell_x = 0;
            let mut cell_y = 0;
            if target_type > 0 {
                cell_x = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                cell_y = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
            }
            self.state.camera_anchor_x = self.parse_script_int(&line, p + 1, '_');
            p = self.state.last_delim_pos;
            self.state.camera_anchor_y = self.parse_script_int(&line, p + 1, ' ');
            self.state.script_cursor = self.state.last_delim_pos + 1; // 分支前推进（a.java:8245）
            if target_type > 0 {
                let idx = host.cell_entity(cell_x, cell_y, target_type);
                if idx >= 0 {
                    if host.entity_type(idx) == 72 {
                        host.set_entity_route_flag(idx);
                    }
                    let (px, py) = host.entity_pixel(idx);
                    host.find_path(px >> 5, py >> 5, self.state.camera_anchor_x, self.state.camera_anchor_y);
                    // Java 无条件置 2（a.java:8261，findPath 结果不影响 phase）
                    self.state.dialog_phase = DialogPhase::Choice;
                } else {
                    self.execute_script_instruction(host); // Java 尾递归（a.java:8264）
                }
            } else if host.find_path(
                self.state.player_cell_x,
                self.state.player_cell_y,
                self.state.camera_anchor_x,
                self.state.camera_anchor_y,
            ) {
                // a.java:8265-8267：三写点缺一不可（复核 R2）
                self.state.script_walk_armed = true;
                self.state.dialog_phase = DialogPhase::Walk;
                self.state.walk_phase = crate::enums::WalkPhase::Idle;
                host.snap_camera_to_player(); // m_104(0)
            }
        } else if var3 == "GUT" {
            let n = self.parse_script_int(&line, var2p4, ' ');
            let cur = self.state.current_script_index;
            self.state.script_line_flags[cur] = true;
            self.load_level_script(host, n as usize);
        } else if var3 == "DES" {
            let mut p = var2p4;
            let t = self.parse_script_int(&line, p, '_');
            p = self.state.last_delim_pos;
            if t < 0 {
                let _x = self.parse_script_int(&line, p + 1, ' ');
                host.remove_all_of_type(-t);
            } else {
                let x = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                let y = self.parse_script_int(&line, p + 1, ' ');
                let idx = host.cell_entity(x, y, t);
                if idx >= 0 {
                    host.remove_entity(idx);
                }
            }
            self.state.script_cursor = self.state.last_delim_pos + 1;
        } else if var3 == "SWD" {
            host.swd_convert_type81();
            self.state.script_cursor = var2 + 1; // 无参数（Java：var2 未推进 ⇒ +1）
        } else if var3 != "MVS" {
            if var3 == "LAY" {
                host.reload_floor_entities(self.state.current_floor); // m_119
                self.state.layer_byte = self.parse_script_int(&line, var2p4, ' ');
                self.state.script_cursor = self.state.last_delim_pos + 1;
                self.state.flag_bool_16 = true; // a.java:8317（复核 R7）
                self.state.overlay_text = None;
                self.state.dialog_phase = DialogPhase::LayCutscene;
            } else if var3 == "ROS" {
                let sub = self.parse_script_int(&line, var2p4, '_');
                let p = self.state.last_delim_pos;
                match sub {
                    1 => {
                        // ROS_1（a.java:8325-8330）：玩家像素坐标直写 + m_104(0)（复核 R9）
                        let x = self.parse_script_int(&line, p + 1, '_');
                        let pm = self.state.last_delim_pos;
                        let y = self.parse_script_int(&line, pm + 1, ' ');
                        self.state.player_pixel_x = x << 5;
                        self.state.player_pixel_y = y << 5;
                        host.snap_camera_after_ros1(self.state.player_pixel_x, self.state.player_pixel_y);
                    }
                    2 => {
                        let v = self.parse_script_int(&line, p + 1, ' ');
                        host.m_082_set_dialog_target(v);
                    }
                    3 => self.state.player_hp = self.parse_script_int(&line, p + 1, ' '),
                    4 => self.state.facing_direction = self.parse_script_int(&line, p + 1, ' '),
                    5 => self.state.player_atk = self.parse_script_int(&line, p + 1, ' '),
                    6 => self.state.player_def = self.parse_script_int(&line, p + 1, ' '),
                    _ => {}
                }
                self.state.script_cursor = self.state.last_delim_pos + 1;
            } else if var3 == "CES" {
                let mut p = var2p4;
                let t = self.parse_script_int(&line, p, '_');
                p = self.state.last_delim_pos;
                let cx = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                let cy = self.parse_script_int(&line, p + 1, ' ');
                let idx = host.spawn_entity(t, cx << 5, cy << 5);
                host.ces_camera(idx);
                // scene_delay：switch(f_byte_arr_03[type]) 类目表（a.java:8367；
                // 类目表初始化 a.java:6204-6252——复核 R3 纠正：查类目非裸 type）
                self.state.scene_delay = match self.data.type_categories[t as usize] {
                    1 | 8 | 16 => 5,
                    2 | 4 => 8,
                    _ => 0,
                };
                self.state.script_cursor = self.state.last_delim_pos + 1;
            } else if var3 == "GIN" {
                let k = self.parse_script_int(&line, var2p4, '_');
                let p = self.state.last_delim_pos;
                let v = self.parse_script_int(&line, p + 1, ' ');
                match k {
                    0 => host.pickup_item(v),
                    1 => host.give_gold(v),
                    _ => {}
                }
                self.state.script_cursor = self.state.last_delim_pos + 1;
                self.execute_script_instruction(host); // 唯一显式尾递归（a.java:8397）
            } else if var3 == "ADD" {
                let mut p = var2p4;
                let floor = self.parse_script_int(&line, p, '_');
                p = self.state.last_delim_pos;
                let t = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                let cx = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                let cy = self.parse_script_int(&line, p + 1, ' ');
                if self.state.current_floor != floor {
                    host.reload_floor_entities(self.state.current_floor);
                    host.reload_floor_entities(floor);
                }
                let _ = host.spawn_entity(t, cx << 5, cy << 5);
                host.reload_floor_entities(floor);
                if self.state.current_floor != floor {
                    host.reload_floor_entities(self.state.current_floor);
                }
                self.state.script_cursor = self.state.last_delim_pos + 1;
            } else if var3 == "GLV" {
                self.parse_script_int(&line, var2p4, ' ');
                self.state.script_cursor = self.state.last_delim_pos + 1;
            } else if var3 == "RES" {
                self.parse_script_int(&line, var2p4, ' ');
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
                    self.state.inventory_entry_count = 0; // f_int_89=0（a.java:8442，复核 R6）
                }
                self.state.script_cursor = self.state.last_delim_pos + 1;
            } else if var3 == "SEE" {
                let mut p = var2p4;
                self.state.camera_anchor_x = self.parse_script_int(&line, p, '_');
                p = self.state.last_delim_pos;
                self.state.camera_anchor_y = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                self.state.tak_page_start = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                self.state.tak_page_end = self.parse_script_int(&line, p + 1, '_');
                p = self.state.last_delim_pos;
                let mode_e = self.parse_script_int(&line, p + 1, ' ');
                self.state.script_cursor = self.state.last_delim_pos + 1;
                let page = self.state.tak_page_start;
                self.open_dialog_page(host, page);
                // Java switch(f_int_117) case 1 落穿 case 0/default（a.java:8456-8464）
                if mode_e == 1
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
            } else if var3 == "END" {
                self.parse_script_int(&line, var2p4, ' ');
                self.state.game_mode = 20;
                host.exit_application();
                self.state.script_cursor = self.state.last_delim_pos + 1;
            } else if var3 == "SMS" {
                self.parse_script_int(&line, var2p4, ' ');
                self.state.script_cursor = self.state.last_delim_pos + 1;
            }
        } else {
            // MVS：P3.2 边界（数据域 68 条脚本零使用）
            panic!("MVS P3.2 边界：语义移植待实体模块（deobf a.java:8490-8564）");
        }

        // 尾部收尾门（a.java:8551-8563）：行被消费完 ⇒ phase=4 + 旗标 + 相机
        if self.state.script_cursor >= line.len() {
            self.state.dialog_phase = DialogPhase::AwaitCamera;
            if self.state.current_script_index != 32 {
                let cur = self.state.current_script_index;
                self.state.script_line_flags[cur] = true;
            }
            host.snap_camera_to_player();
        }
    }

}

/// 无操作宿主（new() 初始化路径用——load_level_script 会调 snap_camera）。
pub struct NoopHost;
impl HostCtx for NoopHost {
    fn find_path(&mut self, _: i32, _: i32, _: i32, _: i32) -> bool { false }
    fn cell_entity(&self, _: i32, _: i32, _: i32) -> i32 { -1 }
    fn remove_entity(&mut self, _: i32) {}
    fn remove_all_of_type(&mut self, _: i32) {}
    fn swd_convert_type81(&mut self) {}
    fn entity_pixel(&self, _: i32) -> (i32, i32) { (0, 0) }
    fn give_gold(&mut self, _: i32) {}
    fn pickup_item(&mut self, _: i32) {}
    fn snap_camera_to_player(&mut self) {}
    fn locate_speaker_entity(&mut self, _: i32) -> i32 { -1 }
    fn set_typewriter(&mut self, _: &str, _: i32, _: i32) {}
    fn reload_floor_entities(&mut self, _: i32) {}
    fn spawn_entity(&mut self, _: i32, _: i32, _: i32) -> i32 { -1 }
    fn ces_camera(&mut self, _: i32) {}
    fn set_camera_anchor_target(&mut self, _: i32, _: i32) {}
    fn exit_application(&mut self) {}
    fn mvs_exchange(&mut self) {}
    fn m_082_set_dialog_target(&mut self, _: i32) {}
    fn set_entity_route_flag(&mut self, _: i32) {}
    fn entity_type(&self, _: i32) -> i32 { -1 }
    fn snap_camera_after_ros1(&mut self, _: i32, _: i32) {}
}
