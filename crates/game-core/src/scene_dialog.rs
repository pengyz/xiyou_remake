//! task-15 V2 dialog-VM 域（v3 §1：tick_dialog 归位+22 字段切片——move+切）。
use super::*;

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

impl GameScene {
    /// run case 11 相位机（a.java:4011-4068）。advanceEntityFrames 由调用方公共段跑。
    pub(super) fn tick_dialog(&mut self) {
        self.step_camera_toward_target();
        match self.dialog.dialog_phase {
            0 => {
                if self.script_delay > 0 {
                    self.script_delay -= 1;
                } else {
                    self.execute_script_instruction();
                }
            }
            1 | 6 => {
                if self.dialog.dialog_phase != 6 || self.camera_at_target() {
                    let text = self.dialog.overlay_text.clone().unwrap_or_default();
                    let text_len = text.chars().count() as i32;
                    let line_h = crate::paint::paint_font().height + 4;
                    if self.dialog.dialog_char < text_len {
                        if self.key_value != 0 {
                            // 全量 layout + 跳到行尾（a.java:4029-4026）
                            self.layout_full_text(&text, 129, (line_h << 1) + 12);
                            self.dialog.dialog_char = self.skip_text_to_line_end(self.dialog.dialog_char);
                        } else if (self.frame_counter & 1) == 1 {
                            self.dialog.dialog_char += 1;
                            // 转义跳宽（charAt(typewriterPos-1)=='\\'）
                            let chars: Vec<char> = text.chars().collect();
                            let idx = self.dialog.dialog_char as usize - 1;
                            if idx < chars.len() && chars[idx] == '\\' {
                                match chars.get(idx + 1).copied().unwrap_or(' ') {
                                    'c' => self.dialog.dialog_char += 8,
                                    'r' => self.dialog.dialog_char += 2,
                                    _ => {}
                                }
                            }
                        }
                        self.layout_wrapped_text_limited(&text, 129, (line_h << 1) + 12, self.dialog.dialog_char);
                        if self.dialog.overlay_total_lines > self.dialog.overlay_page_bottom {
                            self.flip_overlay_page(self.dialog.overlay_total_lines - self.dialog.overlay_lines_per_page);
                        }
                    } else {
                        match self.key_value {
                            -2 => self.flip_overlay_page(self.dialog.overlay_page_top + self.dialog.overlay_lines_per_page),
                            -1 => self.flip_overlay_page(self.dialog.overlay_page_top - self.dialog.overlay_lines_per_page),
                            0 => {}
                            _ => {
                                self.dialog.dialog_char = 1;
                                self.open_dialog_page(self.dialog.dialog_page + 1);
                                if self.dialog.dialog_phase == 6 {
                                    // easeCameraToTarget：ease 目标 = SEE 目标格
                                    self.ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;
                                    self.ease_cam_y = ((self.view.view_h - 32) >> 1) - self.view.cam_y;
                                    self.ease_target_x = self.dialog.dialog_target.0 << 5;
                                    self.ease_target_y = self.dialog.dialog_target.1 << 5;
                                }
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
                                self.walk_step_count -= 1;
                                self.facing =
                                    self.walk_path_buffer[self.walk_step_count as usize] as i32;
                                self.walk_phase = 1;
                                let ok = self.try_step(self.facing);
                                // 【BUG-044】点地寻路专用：被挡且未开战（wp 仍为 1）⇒ 清步、归 wp0，下拍走 finish_walkto。
                                // 原版 MOV 消费机不判返回值（a.java:4080-4084，脚本路线由剧本保证可走）；
                                // 经典脚本行走不占用 dp3（owned 恒假），此分支不触达 classic 按键链。
                                if !ok && self.walk_phase == 1 && owned {
                                    self.walk_step_count = 0;
                                    self.walk_phase = 0;
                                    // 被挡步把模式切走（朝上撞炉 ⇒ Alchemy）：dp3 不再 tick、finish_walkto 等不到，就地收尾
                                    //（不写回 Map，保留炉页；复核 T2-1：否则武装残留 ⇒ 出页后点地全被拒、方向键连走穿墙）
                                    if self.game_mode != GameMode::Dialog && self.walkto_owns_dp3() {
                                        self.route_arrived();
                                        self.restore_walkto_arrows();
                                        self.walkto_draining = false;
                                        self.script_walk_armed = false;
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
                                self.script_walk_armed = false;
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
                            self.step_progress += 8;
                            self.ease_target_x = self.view.player_px;
                            self.ease_target_y = self.view.player_py;
                            if self.player_frame < PLAYER_FRAME_TABLE.len() as i32 - 1 {
                                self.player_frame += 1;
                            } else {
                                self.player_frame = 0;
                            }
                            if self.step_progress >= 32 {
                                self.step_progress = 0;
                                self.player_frame = 0;
                                self.walk_phase = 0;
                                self.player_cell_x = self.view.player_px >> 5;
                                self.player_cell_y = self.view.player_py >> 5;
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
