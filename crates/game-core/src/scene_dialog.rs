//! task-15 V2 dialog-VM 域（v3 §1：tick_dialog 归位+22 字段切片——move+切）。
use super::*;
use crate::subsystem::dialog::{DialogResolver, DialogSubsystem};

/// 【task-15 V2 dialog-VM】对话/浮层域切片（22 字段唯一归属——field-ownership dialog-vm）。
pub struct DialogState {
    /// dialogPhase：0 脚本步进 / 1 TAK 打字 / 2 实体路径 / 3 脚本行走 / 4 等相机 / 6 SEE 打字
    pub dialog_phase: i32,
    /// dialogPage：当前对白页（dialogueTexts 下标）
    pub dialog_page: i32,
    /// dialogPageEnd：TAK 页尾（openDialogPage 边界）
    pub dialog_page_end: i32,
    /// dialogSpeakerType：说话者类型（>=0 实体立绘 / 0 玩家头像 / <0 "?"）
    pub dialog_speaker: i32,
    /// dialogAnchorEntity：lockCameraOn 返回的锚实体（气泡定位）
    pub dialog_anchor: i32,
    /// typewriterPos：打字机游标（UTF-16 单位）
    pub dialog_char: i32,
    /// dialogMoveX/115：SEE/MOV 目标格
    pub dialog_target: (i32, i32),
    /// 对白数据集
    pub dialog_data: DialogData,
    /// 提示浮层（showOverlayMessage a.java:4898-4916）：active/kind/text/框宽高/页
    pub overlay_active: bool,
    pub overlay_kind: i32,
    pub overlay_text: Option<String>,
    pub overlay_box_w: i32,
    pub overlay_box_h: i32,
    pub overlay_left_icon: i8,
    pub overlay_right_icon: i8,
    /// layoutWrappedText 产物（行列表 + 分页游标）
    pub overlay_lines: Vec<String>,
    pub overlay_page_top: i32,
    pub overlay_page_bottom: i32,
    pub overlay_lines_per_page: i32,
    pub overlay_total_lines: i32,
    pub overlay_content_w: i32,
    /// overlaySavedColor（paintWrappedText \c 保存/\r 恢复的色；Java 字段默认 0）
    pub overlay_saved_color: i32,
}


/// 【双归域·逻辑迁移】dialog-VM 域方法（lead 2026-11-02 裁定：域方法持域状态）——
/// 打字机推进=纯对话域态（dialog_char/转义跳宽/翻页·确认决策）；跨域副作用
/// （layout/ease/脚本分派/相机）由 GameScene 编排消费（coordination 面注记归因）。
impl DialogState {
    /// 打字机步进（a.java:4029-4026 语义逐字）：key≠0=全量跳行尾请求；隔帧
    /// 自动推进+转义跳宽（\c=+8 \r=+2）；到尾后 key 分派=翻页/-1、确认=到下页。
    /// 返回 [`TypewriterStep`] 供编排执行跨域动作（行为面零差：决策序不变）。
    pub fn advance_typewriter(&mut self, key: i32, frame_odd: bool, text: &str) -> TypewriterStep {
        let text_len = text.chars().count() as i32;
        if self.dialog_char < text_len {
            if key != 0 {
                return TypewriterStep::SkipToLineEnd;
            }
            if frame_odd {
                self.dialog_char += 1;
                // 转义跳宽（charAt(typewriterPos-1)=='\\'——a.java 语义逐字）
                let chars: Vec<char> = text.chars().collect();
                let idx = self.dialog_char as usize - 1;
                if idx < chars.len() && chars[idx] == '\\' {
                    match chars.get(idx + 1).copied().unwrap_or(' ') {
                        'c' => self.dialog_char += 8,
                        'r' => self.dialog_char += 2,
                        _ => {}
                    }
                }
            }
            return TypewriterStep::Typing;
        }
        match key {
            -2 => TypewriterStep::PageNext,
            -1 => TypewriterStep::PagePrev,
            0 => TypewriterStep::Wait,
            _ => TypewriterStep::Confirm,
        }
    }
}

/// 【双归域】打字机步进结果（域内决策枚举——编排面执行跨域副作用）。
/// 2026-11-07 批3：枚举体迁 crate::subsystem::dialog::DialogStep（face 终形六臂）——此处保同名
/// 别名，既有 match 臂路径（scene_dialog::TypewriterStep::X）与测试零改动。原六臂 doc 随体迁移。
pub type TypewriterStep = crate::subsystem::dialog::DialogStep;

/// 【终章簇2·dialog 域直写接口】跨文件写点收敛（lead 选项 1）。
impl DialogState {
    pub fn set_dialog_phase(&mut self, v: i32) { self.dialog_phase = v; }
    pub fn set_dialog_char(&mut self, v: i32) { self.dialog_char = v; }
    pub fn set_dialog_page(&mut self, v: i32) { self.dialog_page = v; }
    pub fn set_overlay_text(&mut self, v: Option<String>) { self.overlay_text = v; }
    pub fn set_overlay_active(&mut self, v: bool) { self.overlay_active = v; }
    pub fn add_dialog_char(&mut self, d: i32) { self.dialog_char += d; }
    pub fn set_overlay_page_top(&mut self, v: i32) { self.overlay_page_top = v; }
    pub fn set_overlay_page_bottom(&mut self, v: i32) { self.overlay_page_bottom = v; }
}

impl GameScene {
    /// run case 11 相位机（a.java:4011-4068）。advanceEntityFrames 由调用方公共段跑。
    /// 【批3】DialogCtx 装配（dialog 记录槽 &mut+key_value/key_held/frame_odd 值拷贝——
    /// disjoint field borrow；调用前 hoist self 读，ctx 存活期禁触 self）。
    pub(super) fn dialog_ctx(&mut self) -> crate::subsystem::dialog::DialogCtx<'_> {
        crate::subsystem::dialog::DialogCtx {
            dialog: &mut self.dialog,
            key_value: self.key_value,
            key_held: self.key_held,
            frame_odd: (self.frame_counter & 1) == 1,
        }
    }

    pub(super) fn tick_dialog(&mut self) {
        self.step_camera_toward_target();
        match self.dialog.dialog_phase {
            0 => {
                if self.script_delay > 0 {
                    self.dec_script_delay();
                } else {
                    self.execute_script_instruction();
                }
            }
            1 | 6 => {
                // 【双归域·编排面（coordination 注记归因）】打字机决策=DialogState
                // 域方法；跨域副作用（layout/ease/open_dialog_page/相机）留此编排。
                if self.dialog.dialog_phase != 6 || self.camera_at_target() {
                    let text = self.dialog.overlay_text.clone().unwrap_or_default();
                    let line_h = crate::paint::paint_font().height + 4;
                    let step = {
                        let mut dlg_ctx = self.dialog_ctx();
                        let mut dlg_r = DialogResolver;
                        dlg_r.tick(&mut dlg_ctx)
                    };
                    match step {
                        scene_dialog::TypewriterStep::SkipToLineEnd => {
                            // 全量 layout + 跳到行尾（a.java:4029-4026）
                            self.layout_full_text(&text, 129, (line_h << 1) + 12);
                            self.dialog.set_dialog_char(self.skip_text_to_line_end(self.dialog.dialog_char));
                            // 公共尾（原式 fall-through：skip 后同走 wrapped layout+翻页检）
                            self.layout_wrapped_text_limited(&text, 129, (line_h << 1) + 12, self.dialog.dialog_char);
                            if self.dialog.overlay_total_lines > self.dialog.overlay_page_bottom {
                                self.flip_overlay_page(self.dialog.overlay_total_lines - self.dialog.overlay_lines_per_page);
                            }
                        }
                        scene_dialog::TypewriterStep::Typing => {
                            self.layout_wrapped_text_limited(&text, 129, (line_h << 1) + 12, self.dialog.dialog_char);
                            if self.dialog.overlay_total_lines > self.dialog.overlay_page_bottom {
                                self.flip_overlay_page(self.dialog.overlay_total_lines - self.dialog.overlay_lines_per_page);
                            }
                        }
                        scene_dialog::TypewriterStep::PageNext => {
                            self.flip_overlay_page(self.dialog.overlay_page_top + self.dialog.overlay_lines_per_page);
                        }
                        scene_dialog::TypewriterStep::PagePrev => {
                            self.flip_overlay_page(self.dialog.overlay_page_top - self.dialog.overlay_lines_per_page);
                        }
                        scene_dialog::TypewriterStep::Wait => {}
                        scene_dialog::TypewriterStep::Confirm => {
                            // dialog_char=1 域写已入 DialogResolver::tick（批3）；此处编排接续。
                            self.open_dialog_page(self.dialog.dialog_page + 1);
                            if self.dialog.dialog_phase == 6 {
                                // easeCameraToTarget：ease 目标 = SEE 目标格
                                self.set_ease_cam_x(((self.view.view_w - 32) >> 1) - self.view.cam_x);
                                self.set_ease_cam_y(((self.view.view_h - 32) >> 1) - self.view.cam_y);
                                self.set_ease_target_x(self.dialog.dialog_target.0 << 5);
                                self.set_ease_target_y(self.dialog.dialog_target.1 << 5);
                            }
                        }
                    }
                }
            }
            2 => self.tick_entity_route(),
            3 => {
                // dp3 脚本行走（a.java:4077-4114）：wp0 弹路径步 tryStep /
                // 耗尽 → scriptWalkArmed=false + dp0；wp1 8px 步进（无边缘
                // 跟随，cameraTarget 随玩家）+ m_025 帧 + 32px 归格
                // applyStepCellEffects；wp5 战斗拍。
                if !self.transitioning {
                    match self.walk_phase {
                        0 => {
                            if self.walk_step_count > 0 {
                                // 【BUG-044】弹步前记"本步归点地寻路"：被挡步可能经剧情接管（型 83 → load_level_script 先收段）
                                // 让 walkto_owns_dp3() 在 try_step 之后变假，门控须看弹步时的归属（复核 T2-2）。
                                let owned = self.walkto_owns_dp3();
                                self.dec_walk_step_count();
                                self.set_facing(
                                    self.walk_path_buffer[self.walk_step_count as usize] as i32,
                                );
                                self.set_walk_phase(1);
                                let ok = self.try_step(self.facing);
                                // 【BUG-044】点地寻路专用：被挡且未开战（wp 仍为 1）⇒ 清步、归 wp0，下拍走 finish_walkto。
                                // 原版 MOV 消费机不判返回值（a.java:4080-4084，脚本路线由剧本保证可走）；
                                // 经典脚本行走不占用 dp3（owned 恒假），此分支不触达 classic 按键链。
                                if !ok && self.walk_phase == 1 && owned {
                                    self.set_walk_step_count(0);
                                    self.set_walk_phase(0);
                                    // 被挡步把模式切走（朝上撞炉 ⇒ Alchemy）：dp3 不再 tick、finish_walkto 等不到，就地收尾
                                    //（不写回 Map，保留炉页；复核 T2-1：否则武装残留 ⇒ 出页后点地全被拒、方向键连走穿墙）
                                    if self.game_mode != GameMode::Dialog && self.walkto_owns_dp3() {
                                        self.route_arrived();
                                        self.restore_walkto_arrows();
                                        self.set_walkto_draining(false);
                                        self.set_script_walk_armed(false);
                                        self.dialog.dialog_phase = 0;
                                    }
                                }
                            } else {
                                // 【T2】WalkTo 段终位（整段起终点入栈）+回 Map 模式；
                                // 经典脚本行走（route 未开放）零改动——mode 回归留脚本机器
                                // 【BUG-040】打断后的收尾（walkto_draining）同走寻路收尾
                                if self.walkto_owns_dp3() {
                                    self.finish_walkto();
                                } else {
                                    self.route_arrived();
                                }
                                self.set_script_walk_armed(false);
                                self.dialog.dialog_phase = 0;
                            }
                        }
                        1 => {
                            match self.facing {
                                0 => self.view.player_py += 8,
                                1 => self.view.player_py -= 8,
                                2 => self.view.player_px += 8,
                                _ => self.view.player_px -= 8,
                            }
                            self.add_step_progress(8);
                            self.set_ease_target_x(self.view.player_px);
                            self.set_ease_target_y(self.view.player_py);
                            if self.player_frame < PLAYER_FRAME_TABLE.len() as i32 - 1 {
                                self.inc_player_frame();
                            } else {
                                self.set_player_frame(0);
                            }
                            if self.step_progress >= 32 {
                                self.set_step_progress(0);
                                self.set_player_frame(0);
                                self.set_walk_phase(0);
                                self.set_player_cell(self.view.player_px >> 5, self.view.player_py >> 5);
                                self.apply_step_cell_effects();
                                // 【T2】WalkTo 段终位：步尽归格=实际终点（整段起终点
                                // 入栈）+回 Map；经典脚本行走 route 未开放=零改动
                                // 【BUG-040】打断后走完当前步同此收尾；收尾解除武装
                                if self.walk_step_count == 0 && self.walkto_owns_dp3() {
                                    self.finish_walkto();
                                }
                            }
                        }
                        5 => self.tick_battle(self.battle_active),
                        _ => {}
                    }
                }
            }
            4 => {
                if self.camera_at_target() {
                    self.transition_to(GameMode::Map);
                }
            }
            5 => {
                // LAY 批续跑缝（a.java:4123-4127）：floorTransitioning ∧ !wipeClosing
                // （换层已过、遮幅开启中）⇒ 相位归 0 续跑脚本。op_lay 停表于此；
                // 2026-10-17 前为空臂桩（"floorTransitioning 组合（LAY 批）"未译）⇒ 剧情链
                // 永停（用户实机 BUG-016：3→2 天牢切层后牛魔王对话不触发 + 输入假死）。
                if self.transitioning && !self.wipe_closing {
                    self.dialog.dialog_phase = 0;
                }
            }
            _ => {}
        }
    }
}
