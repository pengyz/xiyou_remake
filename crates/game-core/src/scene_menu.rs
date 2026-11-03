//! task-15 步③ 菜单族（move-only 自 scene.rs——enter/tick/paint 三件套自闭）。
use super::*; // scene.rs 私有面可见（子模块）——纯剪切语义保持

/// 【task-15 V2 menu/UI 收口】菜单/UI 域切片（42 字段唯一归属——field-ownership menu-ui 域；v3 §1：非 render 子域）。
pub struct MenuState {
    /// 软键（dispatchModeEntry case 3：1/3）
    pub softkeys: (i8, i8),
    /// tutorialSeen：教程标记（型 15 拾取置 true——f_bool_13 楼梯指示的门控）
    pub tutorial_seen: bool,
    /// f_bool_13：楼梯指示（49/55 暗道换乘键门控；proximity 更新——
    /// lockCameraOn/行走批次——未移植，稳态 false 与 Java 稳态一致）
    pub stair_hint: bool,
    /// mode12 日志浏览（recordPage 页 + f_String_04 页码指示 + 日志对）
    pub record_page: i32,
    /// f_String_arr_06/09（a.java:823/887 init 常量；fixtures 迭代 59）
    pub record_text_a: Vec<String>,
    pub record_text_b: Vec<String>,
    pub page_indicator: String,
    /// difficultyIndex（难度乘数索引；tryRunScene case 67 门控）
    pub difficulty_index: i32,
    /// mode 4/19 菜单族共享布局（menuLabels a.java:458-462 / resetMenuLayout
    /// a.java:4690-4702 / addMenuItem a.java:4704-4716）
    pub menu_kinds: Vec<i8>,
    pub menu_item_count: i32,
    pub menu_box_width: i32,
    pub menu_line_height: i32,
    pub menu_visible_rows: i32,
    pub menu_list_height: i32,
    pub menu_scroll_top: i32,
    pub menu_visible_bottom: i32,
    pub menu_cursor: i32,
    /// menuReturnMode（N-0240 menuReturnMode）：mode7/8 -7 的返回目标
    pub menu_return_mode: GameMode,
    /// mode 7 存档槽状态（dispatchModeEntry case7 布局 + loadSlotTable 冷态）
    pub slot: crate::menu_family::SlotSelect,
    /// mode 5 敌人图鉴：f_int_arr_22（预测伤害）/f_byte_arr_20（类型）
    pub encyc_damage: Vec<i32>,
    pub encyc_types: Vec<u8>,
    pub encyc_panel_w: i32,
    pub encyc_count: i32,
    pub encyc_rows_visible: i32,
    pub encyc_panel_h: i32,
    pub encyc_scroll_top: i32,
    pub encyc_scroll_bottom: i32,
    /// mode 16 设置面板（kind4 进入；menuReturnMode=4 返回）
    pub options: Option<crate::menu_family::OptionList>,
    /// 【计划 3 T7】帮助文本覆盖（壳注入；交接不清除=随档持久，读档/换层不影响）。
    pub help_override: Option<String>,
    /// 【STAGE-B-006】壳层附加设置行（交接自 BootMachine，回标题时交回；空 = 原版两行）：设置页入口据此生成第 3 行起的
    /// 附加行，切换回写此处。core 不解释其含义。
    pub shell_options: Vec<crate::menu_family::ShellOption>,
    /// inGameMenuReturnMode：菜单返回目标（run case3 -7 时存 gameMode）
    pub in_game_menu_return_mode: GameMode,
    /// mode 10 道具面板布局（dispatchModeEntry case10 a.java:4370-4380：itemPanelCols..104）
    pub item_cols: i32,
    pub item_rows_total: i32,
    pub item_rows_visible: i32,
    pub item_scroll_top: i32,
    pub item_scroll_bottom: i32,
    pub item_box_w: i32,
    pub item_box_h: i32,
    pub item_cursor: i32,
    /// itemUseTargetType：最近拾取/装备的类型号（浮层图标 + 商店显示）
    pub overlay_kind_item: i32,
    /// 浮层图标图（kind1/3：entityTypeImage[itemUseTargetType] 的克隆——paint 免借用冲突）
    pub overlay_icon: Option<ArgbImage>,
    /// 浮层标题（objectTypeNames[itemUseTargetType]）
    pub overlay_item_name: Option<Vec<u16>>,
}

/// 【双归域·menu 逻辑回补】菜单光标域方法（纯 MenuState 态迁移：光标+滚窗
/// 同步+环绕——a.java:3699/3713 语义逐字）；key 分派/激活=GameScene 编排
/// （coordination 注记归因）。
impl MenuState {
    /// 下移一格（-2）：越尾环绕回顶（滚窗复位）。
    pub fn cursor_down(&mut self) {
        if self.menu_cursor < self.menu_item_count - 1 {
            self.menu_cursor += 1;
            if self.menu_cursor >= self.menu_visible_bottom {
                self.menu_visible_bottom += 1;
                self.menu_scroll_top += 1;
            }
        } else {
            self.menu_cursor = 0;
            self.menu_scroll_top = 0;
            self.menu_visible_bottom = self.menu_item_count;
            if self.menu_item_count > self.menu_visible_rows {
                self.menu_visible_bottom = self.menu_visible_rows;
            }
        }
    }

    /// 上移一格（-1）：越顶环绕到底（滚窗钳 0）。
    pub fn cursor_up(&mut self) {
        if self.menu_cursor > 0 {
            self.menu_cursor -= 1;
            if self.menu_cursor < self.menu_scroll_top {
                self.menu_scroll_top -= 1;
                self.menu_visible_bottom -= 1;
            }
        } else {
            self.menu_cursor = self.menu_item_count - 1;
            self.menu_visible_bottom = self.menu_item_count;
            self.menu_scroll_top = self.menu_visible_bottom - self.menu_visible_rows;
            if self.menu_scroll_top < 0 {
                self.menu_scroll_top = 0;
            }
        }
    }
}

/// 【终章簇3·menu/UI 域直写接口】跨文件写点收敛（lead 选项 1）。
impl MenuState {
    pub fn set_menu_scroll_top(&mut self, v: i32) { self.menu_scroll_top = v; }
    pub fn set_menu_visible_bottom(&mut self, v: i32) { self.menu_visible_bottom = v; }
    pub fn set_menu_cursor(&mut self, v: i32) { self.menu_cursor = v; }
    pub fn set_item_cursor(&mut self, v: i32) { self.item_cursor = v; }
    pub fn set_item_scroll_top(&mut self, v: i32) { self.item_scroll_top = v; }
    pub fn set_item_scroll_bottom(&mut self, v: i32) { self.item_scroll_bottom = v; }
    pub fn set_options(&mut self, v: Option<crate::menu_family::OptionList>) { self.options = v; }
    pub fn set_overlay_kind_item(&mut self, v: i32) { self.overlay_kind_item = v; }
    pub fn set_softkeys(&mut self, v: (i8, i8)) { self.softkeys = v; }
}

impl GameScene {
    /// dispatchModeEntry case 4（a.java:4326-4337）：继续游戏（+floor>50 的跳过教程）/
    /// 保存/设置/帮助/退出；软键 (1,3)。
    pub(super) fn enter_menu4(&mut self) {
        self.reset_menu_layout();
        self.add_menu_item(1);
        if self.persist.floor > 50 {
            self.add_menu_item(17);
        }
        self.add_menu_item(3);
        self.add_menu_item(4);
        self.add_menu_item(5);
        self.add_menu_item(7);
        self.menu.softkeys = (1, 3);
    }
    /// dispatchModeEntry case 19（a.java:4439-4448）：商店 5 条目。
    pub(super) fn enter_menu19(&mut self) {
        self.reset_menu_layout();
        for k in 12..=16 {
            self.add_menu_item(k);
        }
    }
    /// run case 4（a.java:3686-3731）：-7 返回、-6/-5 激活、-2/-4 下移、-1/-3 上移。
    pub(super) fn tick_menu4(&mut self) {
        match self.key_value {
            -7 => self.transition_to(self.menu.in_game_menu_return_mode),
            -6 | -5 => self.activate_menu_item(),
            -2 => self.menu.cursor_down(), // 【双归域】域方法（a.java:3699 逐字）
            -1 => self.menu.cursor_up(), // 【双归域】域方法（a.java:3713 逐字）
            _ => {}
        }
    }
    /// run case 19（a.java:4196-4222）：-7 回 mode 4（无 break 落 default 效果相同）；
    /// -6/-5 无操作（购买走软键链路，本批对拍面不含）；-2/-4 下移环绕、-1/-3 上移环绕。
    pub(super) fn tick_menu19(&mut self) {
        match self.key_value {
            -7 => {
                self.transition_to(GameMode::InGameMenu);
                self.enter_menu4();
            }
            -4 | -2 | 54 | 56 => {
                if self.menu.menu_cursor < self.menu.menu_item_count - 1 {
                    self.menu.menu_cursor += 1;
                } else {
                    self.menu.menu_cursor = 0;
                }
            }
            -3 | -1 | 50 | 52 => {
                if self.menu.menu_cursor > 0 {
                    self.menu.menu_cursor -= 1;
                } else {
                    self.menu.menu_cursor = self.menu.menu_item_count - 1;
                }
            }
            _ => {}
        }
    }
    /// paint case 4/19 共享（a.java:2423-2486）：世界底（视差+HUD 面板+状态栏，
    /// 无瓦片/实体层）+ 框 + 条目（商店 12-16 逐字彩虹；普通 drawString 17）+
    /// 翻页箭头 + 软键栏。
    pub(super) fn paint_menu_list(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, true);
        self.paint_hud(g);
        self.paint_status_bar(g);
        let font = crate::paint::paint_font();
        // STAGE-B-004b ①③：菜单框锚位公式化（menu_family::in_game_menu_box）——
        // classic 代入（center=SCREEN 系 120/160）逐值化回 a.java:2431-2444 原值；
        // modern 随 layout.center 重排，高亮/箭头随框派生。
        let mb = crate::menu_family::in_game_menu_box(
            self.layout.center_x,
            self.layout.center_y,
            self.menu.menu_box_width,
            self.menu.menu_list_height,
            self.menu.menu_line_height,
            font.height(),
        );
        let (box_w, box_h) = (mb.box_w, mb.box_h);
        crate::menu_family::paint_box_frame(g, &self.images.ui[0], mb.bx, mb.by, box_w, box_h);
        let pad = mb.pad;
        let mut by = mb.first_row_y;
        let mut i = self.menu.menu_scroll_top;
        while i < self.menu.menu_visible_bottom {
            let kind = self.menu.menu_kinds[i as usize];
            if self.menu.menu_cursor == i {
                g.set_color(0);
                g.fill_rect(mb.hl_x, by - pad, mb.hl_w, self.menu.menu_line_height);
                g.set_color(16377897);
            } else if kind != 11 {
                g.set_color(7574946);
            } else {
                g.set_color(16751103);
            }
            if kind > 11 && kind < 17 {
                // 商店条目：逐字彩虹（shopRainbowColors[shopRainbowPhase 起步环绕]），
                // 相位在 paint 尾按 fc&3 推进（a.java:2476-2479）
                let label = MENU_LABELS[kind as usize];
                let n = RAINBOW_COLORS.len();
                let label_u: Vec<u16> = label.encode_utf16().collect();
                let mut cx = mb.item_cx - (font.string_width(&label_u) >> 1);
                let mut phase = self.rainbow_phase as usize;
                for ch in label.chars() {
                    g.set_color(RAINBOW_COLORS[phase]);
                    g.draw_char(ch as u16, cx, by, 0);
                    cx += font.char_width(ch as u16);
                    phase = (phase + 1) % n;
                }
                if self.frame_counter & 3 == 0 {
                    self.rainbow_phase = (self.rainbow_phase + 1) % n as i32;
                }
            } else {
                let label: Vec<u16> = MENU_LABELS[kind as usize].encode_utf16().collect();
                g.draw_string(&label, mb.item_cx, by, 17);
            }
            by += self.menu.menu_line_height;
            i += 1;
        }
        let (ax, ay) = (mb.arrow_x, mb.arrow_y);
        if self.menu.menu_scroll_top > 0 {
            draw_image_clipped(g, &self.images.ui[15], ax, ay, 0, 0, 7, 9);
        }
        if self.menu.menu_visible_bottom < self.menu.menu_item_count {
            draw_image_clipped(g, &self.images.ui[15], ax, ay + 9, 7, 0, 7, 9);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.menu.softkeys.0, self.menu.softkeys.1, &self.layout);
    }
    /// dispatchModeEntry case 10（a.java:4370-4380）：道具面板布局初始化。
    pub(super) fn enter_item_panel(&mut self) {
        let font = crate::paint::paint_font();
        self.menu.item_cols = 4;
        self.menu.item_rows_total = 32 / self.menu.item_cols;
        self.menu.item_rows_visible = (268 - font.height() - 10) / 40;
        if self.menu.item_rows_visible > self.menu.item_rows_total {
            self.menu.item_rows_visible = self.menu.item_rows_total;
        }
        self.menu.item_scroll_top = 0;
        self.menu.item_scroll_bottom = self.menu.item_rows_visible;
        self.menu.item_box_w = 42 + 40 * self.menu.item_cols;
        self.menu.item_box_h = 34 + font.height() + 10 + 40 * self.menu.item_rows_visible;
        self.menu.softkeys = (1, 3);
    }
    /// run case 10（a.java:3931-4009）：-7 返回 mode 3；-6/-5 使用道具
    /// （13/14 直接生效回 mode 3、15/23/24/25 浮层(0,3)、其余浮层(1,2)）；
    /// -4/-3 行内环绕、-2/-1 翻行+滚动窗跟随。
    pub(super) fn tick_item_panel(&mut self) {
        match self.key_value {
            -7 => self.transition_to(GameMode::Map),
            -6 | -5 => {
                if self.menu.item_cursor < self.player.item_stack_size {
                    let t = self.player.item_types[self.menu.item_cursor as usize] as i32;
                    // itemUseTargetType（a.java:3941）：确认浮层图标/名称的数据源
                    self.menu.overlay_kind_item = t;
                    match t {
                        13 | 14 => {
                            self.transition_to(GameMode::Map);
                            self.activate_item(t);
                        }
                        15 | 23 | 24 | 25 => {
                            let text = self.item_description(self.item_desc_index(t));
                            self.show_overlay_message(3, &text, 0, 3);
                        }
                        _ => {
                            let text = self.item_description(self.item_desc_index(t));
                            self.show_overlay_message(3, &text, 1, 2);
                        }
                    }
                }
            }
            -4 => {
                if self.menu.item_cursor % self.menu.item_cols < self.menu.item_cols - 1 {
                    self.menu.item_cursor += 1;
                } else {
                    self.menu.item_cursor -= self.menu.item_cols - 1;
                }
            }
            -3 => {
                if self.menu.item_cursor % self.menu.item_cols > 0 {
                    self.menu.item_cursor -= 1;
                } else {
                    self.menu.item_cursor += self.menu.item_cols - 1;
                }
            }
            -2 => {
                if self.menu.item_cursor < self.menu.item_cols * (self.menu.item_rows_total - 1) {
                    self.menu.item_cursor += self.menu.item_cols;
                    if self.menu.item_cursor / self.menu.item_cols >= self.menu.item_scroll_bottom {
                        self.menu.item_scroll_top += 1;
                        self.menu.item_scroll_bottom += 1;
                    }
                } else {
                    self.menu.item_cursor -= (self.menu.item_rows_total - 1) * self.menu.item_cols;
                    self.menu.item_scroll_top = 0;
                    self.menu.item_scroll_bottom = self.menu.item_rows_visible;
                }
            }
            -1 => {
                if self.menu.item_cursor >= self.menu.item_cols {
                    self.menu.item_cursor -= self.menu.item_cols;
                    if self.menu.item_cursor / self.menu.item_cols < self.menu.item_scroll_top {
                        self.menu.item_scroll_top -= 1;
                        self.menu.item_scroll_bottom -= 1;
                    }
                } else {
                    self.menu.item_cursor += (self.menu.item_rows_total - 1) * self.menu.item_cols;
                    self.menu.item_scroll_bottom = self.menu.item_rows_total;
                    self.menu.item_scroll_top = self.menu.item_rows_total - self.menu.item_rows_visible;
                }
            }
            _ => {}
        }
    }
    /// paint case 10（a.java:2724-2792）：斜线背景 + paintBoxFrame + 标题 +
    /// 分隔线 + 4×N 格阵（选中框 [2][10] 双帧闪烁）+ 翻页箭头 + 软键栏。
    pub(super) fn paint_item_panel(&mut self, g: &mut SoftGraphics<'_>) {
        let font = crate::paint::paint_font();
        // 斜线底纹（a.java:2725-2734）：Java 方法内留有未用局部 `short var141 = 320;
        // short var122 = 240;`——源码以 H/W 局部常量书写、javac 常量折叠的残迹
        //（B 级），故 319/318/239 = H-1/H-2/W-1 一并按屏幕派生书写。
        let mut v159 = self.layout.screen_h - 1;
        while v159 >= 1 {
            g.draw_line(-1, -1 + (self.layout.screen_h - v159), -1 + v159 - 1, self.layout.screen_h - 2);
            g.draw_line(self.layout.screen_w - 1 - v159, -1, self.layout.screen_w - 1, -1 + v159);
            v159 -= 2;
        }
        let mut x = (self.layout.screen_w - self.menu.item_box_w) >> 1;
        let mut y = (self.layout.screen_h - self.menu.item_box_h) >> 1;
        crate::menu_family::paint_box_frame(g, &self.images.ui[0], x, y, self.menu.item_box_w, self.menu.item_box_h);
        x += 20;
        y += 16;
        g.set_color(13097429);
        if self.menu.item_cursor < self.player.item_stack_size {
            let name = self.object_type_names[self.player.item_types[self.menu.item_cursor as usize] as usize]
                .encode_utf16()
                .collect::<Vec<u16>>();
            g.draw_string(&name, x, y + 2, 0);
        }
        y += font.height() + 4;
        x -= 9;
        g.set_color(6178);
        g.draw_line(x, y, x + self.menu.item_box_w - 23, y);
        y += 1;
        g.set_color(3564144);
        g.draw_line(x, y, x + self.menu.item_box_w - 23, y);
        // 格阵原点 = 共享 fn（③-1 单源：与 pointer_hit 共调 item_cell_origin）
        let (col_x0, grid_y0) = self.item_cell_origin();
        let mut x = col_x0;
        let mut y = grid_y0;
        let mut grid = self.menu.item_cols * self.menu.item_scroll_top;
        let mut row = self.menu.item_scroll_top;
        while row < self.menu.item_scroll_bottom {
            for _ in 0..self.menu.item_cols {
                crate::menu_family::paint_mini_frame(g, x, y, 32, 32);
                if grid == self.menu.item_cursor {
                    draw_image_clipped(
                        g,
                        &self.images.map210,
                        x - 4,
                        y - 3,
                        38 * (self.frame_counter & 1) as i32,
                        0,
                        38,
                        38,
                    );
                }
                if grid < self.player.item_stack_size {
                    let t = self.player.item_types[grid as usize] as usize;
                    if let Some(Some(img)) = self.images.entity.get(t) {
                        // 【R-2 换算收敛】居中式宽高用**逻辑**尺寸（=w/d）。
                        g.draw_image(
                            img,
                            x + ((32 - img.logical_width()) >> 1),
                            y + ((32 - img.logical_height()) >> 1),
                            0,
                        );
                    }
                    if (self.player.item_uses[grid as usize] as i8) > 0 {
                        crate::menu_family::paint_number(
                            g,
                            &self.images.ui[2],
                            self.player.item_uses[grid as usize] as i32,
                            x + 32,
                            y + 26,
                        );
                    }
                }
                grid += 1;
                x += 40;
            }
            row += 1;
            x = col_x0;
            y += 40;
        }
        // 翻页箭头=盒右-22（盒框同源 layout；【2026-10-26 更正】旧 SCREEN_W 同病）
        x = ((self.layout.screen_w + self.menu.item_box_w) >> 1) - 22;
        y -= (self.menu.item_box_h - 32 - font.height() - 4) >> 1;
        if self.menu.item_scroll_top > 0 {
            draw_image_clipped(g, &self.images.ui[15], x, y - 15, 0, 0, 7, 9);
        }
        if self.menu.item_scroll_bottom < self.menu.item_rows_total {
            draw_image_clipped(g, &self.images.ui[15], x, y + 5, 7, 0, 7, 9);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.menu.softkeys.0, self.menu.softkeys.1, &self.layout);
    }
    /// dispatchModeEntry case 7（a.java:4342-4364，case7/8 共享）：软键 (1,3) + loadSlotTable
    /// （每次进入重读 RMS 槽表）+ 槽表布局（6 行/宽 148/高 196）。
    /// 【2026-10-18 BUG-021】入口只复位布局/光标（a.java:4354-4366），**不动数据
    /// 数组**（Java 数组跨进入复用）；读失败（缺店/空/截断）留半解析旧值 +
    /// parse_dirty（markSlotEntry(-1) 落盘触发器，壳消费回写 MOT_IF）。
    pub(super) fn enter_menu7(&mut self) {
        self.menu.slot.reset_entry_layout();
        // 【Y3】提示词根随面：mode7 存档="保存"/mode8 读档="加载"（a.java 语义）
        self.menu.slot.retry_verb = if self.game_mode == GameMode::SaveSlot { "保存" } else { "加载" };
        self.menu.slot.load_slot_table(self.persist.preset_mot_if.as_deref());
        self.menu.softkeys = (1, 3);
    }
    /// run case 7（a.java:3829-3878，case7/8 共享）：-7 回 menuReturnMode；
    /// -6/-5 在 mode7 = 存档（m_119 + saveFloorState + “保存成功！”浮层）；
    /// -2/-1 槽光标滚动。
    pub(super) fn tick_menu7(&mut self) {
        match self.key_value {
            -7 => self.transition_to(self.menu.menu_return_mode),
            -6 | -5 => {
                if self.game_mode == GameMode::SaveSlot {
                    // 【BUG-045④】存档清栈（存档后不可撤回存档前的操作）
                    self.undo.clear(crate::undo_journal::ClearReason::Save);
                    self.save_floor_state();
                    let cursor = self.menu.slot.cursor as usize;
                    let blob = self.serialize_slot_blob();
                    self.persist.saved_slot_blobs[cursor] = Some(blob);
                    // markSlotEntry（a.java:9152-9172）
                    self.menu.slot.valid[cursor] = true;
                    self.menu.slot.floor_num[cursor] = self.persist.floor as i8;
                    self.menu.slot.hp[cursor] = self.player.hp;
                    self.menu.slot.atk[cursor] = self.player.atk;
                    self.menu.slot.def[cursor] = self.player.def;
                    self.menu.slot.keys_yellow[cursor] = self.player.yellow_keys as i16;
                    self.menu.slot.keys_blue[cursor] = self.player.blue_keys as i16;
                    self.menu.slot.keys_red[cursor] = self.player.red_keys as i16;
                    self.menu.slot.gold[cursor] = self.player.gold;
                    // a.java:9013-9014：f_byte_arr_25/26 = equippedWeaponType/ArmorType
                    //（byte 字段；值域 equipTierTypes ⊂ i8）——装备框图标分支的数据源
                    self.menu.slot.weapon[cursor] = self.equipped_weapon as i8;
                    self.menu.slot.armor[cursor] = self.equipped_armor as i8;
                    self.show_overlay_message(0, "保存成功！", 0, 0);
                }
            }
            -2 => self.menu.slot.cursor_down(),
            -1 => self.menu.slot.cursor_up(),
            _ => {}
        }
    }
    /// paintSlotTable(true)（a.java:2679-2682 + 9046-9215）：mode 7 槽表（存档标题变体
    /// titledBox segs[2]）+ 有效槽详情面板（层数/装备/金币/三围/三钥）。
    pub(super) fn paint_slot_list(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let ui = &self.images.ui;
        // 标题变体 2（存档）：titleBarSegments[2] = {4,5}；列表段/装备框段/三围段与
        // boot 读档路径共用 SlotSelect 方法（原为双拷贝）
        let (lx, ly) = self.menu.slot.paint_list(g, ui, &[4, 5], &self.layout);
        // 详情面板（a.java:9092-9147，slotValidFlags[cursor] 才画）
        let c = self.menu.slot.cursor as usize;
        if self.menu.slot.valid[c] {
            let vx = lx + 6;
            let vy = ly + 8;
            if self.menu.slot.floor_num[c] < 51 {
                paint_number(g, &ui[18], self.menu.slot.floor_num[c] as i32, vx + 48, vy);
                g.draw_image(&ui[8], vx + 50, vy + 8, 0);
            }
            // 装备框段（a.java:9104-9143）：f_Image_arr2_00[6][i] 经 m_122 映射表
            // 反查 entityTypeImage（同一位图，见 SPTARM_ENTITY_TYPES）
            let entity = &self.images.entity;
            self.menu.slot.paint_equipment_frames(g, c, vy, &self.layout, |i| {
                crate::menu_family::SPTARM_ENTITY_TYPES
                    .get(i)
                    .and_then(|&t| entity.get(t))
                    .and_then(|o| o.as_ref())
            });
            let (mut vx, vy) = self.menu.slot.paint_detail_stats(g, ui, c, vy, &self.layout);
            self.paint_key_slot(g, 0, self.menu.slot.keys_yellow[c] as i32, vx, vy);
            vx += 42;
            self.paint_key_slot(g, 1, self.menu.slot.keys_blue[c] as i32, vx, vy);
            vx += 42;
            self.paint_key_slot(g, 2, self.menu.slot.keys_red[c] as i32, vx, vy);
        }
        m_034_softkeys(g, &ui[10], &ui[11], self.menu.softkeys.0, self.menu.softkeys.1, &self.layout);
    }
    pub(super) fn enter_menu15(&mut self) {
        let owned = self.menu.help_override.clone().unwrap_or_else(|| Self::HELP_TEXT.to_string());
        self.show_overlay_message(0, &owned, 0, 0);
        self.menu.softkeys = (0, 3);
    }
    /// run case 15（fallthrough no-op 组 a.java:3747-3762）+ run 头浮层支路
    /// （a.java:3262-3272，kind0 且 gameMode∈{15,17}）：仅 -7 关浮层且回
    /// menuReturnMode；-2/-1 翻页；**其余键（含 -6）一律吞掉不关**——
    /// 迭代 26 更正：原 `_ => 关浮层` 系误植，-6 关闭语义不存在于原版。
    pub(super) fn tick_menu15(&mut self) {
        if self.dialog.overlay_active {
            match self.key_value {
                -7 => {
                    self.dialog.set_overlay_active(false);
                    self.transition_to(self.menu.menu_return_mode);
                }
                -2 => self.flip_overlay_page(self.dialog.overlay_page_top + self.dialog.overlay_lines_per_page),
                -1 => self.flip_overlay_page(self.dialog.overlay_page_top - self.dialog.overlay_lines_per_page),
                _ => {}
            }
            self.set_key_value(0);
        }
    }
    /// paint case 15（a.java:2958-2962）：parallax(false) + 帮助图标条
    /// （menu_strip sy=51）+ 软键 + 浮层尾。
    pub(super) fn paint_menu15(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let mut f = self.render.strip_frame;
        let sx = crate::title::menu_strip_x(self.layout.screen_w);
        crate::title::paint_icon_strip_at(g, &self.images.menu_strip, sx, 10, 51, 68, &mut f);
        self.render.set_strip_frame(f);
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.menu.softkeys.0, self.menu.softkeys.1, &self.layout);
        if !self.transitioning && self.dialog.overlay_active {
            self.paint_overlay_box(g);
        }
    }
    /// dispatchModeEntry case 5：m_091 + 软键 (0,3)。
    pub(super) fn enter_menu5(&mut self) {
        self.build_encyclopedia();
        self.menu.softkeys = (0, 3);
    }
    /// run case 5（a.java:3737-3755）：-7/-6 回 mode 3；-2 下一页、-1 上一页。
    pub(super) fn tick_menu5(&mut self) {
        match self.key_value {
            -7 | -6 => self.transition_to(GameMode::Map),
            -2 => {
                let p = self.menu.encyc_scroll_top + 1;
                self.encyclopedia_set_page(p);
            }
            -1 => {
                let p = self.menu.encyc_scroll_top - 1;
                self.encyclopedia_set_page(p);
            }
            _ => {}
        }
    }
    /// paint case 5（a.java:2497-2560）：parallax(false) + 居中 paintBoxFrame
    /// + 逐行（名称/预测/精灵/攻防/金币）+ 行分隔线 + 箭头 + 软键。
    pub(super) fn paint_menu5(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let font = crate::paint::paint_font();
        let bx = (self.layout.screen_w - self.menu.encyc_panel_w) >> 1;
        let by = (self.layout.screen_h - self.menu.encyc_panel_h) >> 1;
        if self.menu.encyc_scroll_bottom - self.menu.encyc_scroll_top > 0 {
            crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, self.menu.encyc_panel_w, self.menu.encyc_panel_h);
            // 【BUG-031③】a.java:2511/2595 翻页箭头 x 112 = 屏心 120 − 8（面板居中 ⇒ 随屏心）
            let arrow_x = self.layout.center_x - 8;
            if self.menu.encyc_scroll_top > 0 {
                draw_image_clipped(g, &self.images.ui[19], arrow_x, by - 17 - (self.frame_counter & 1) as i32, 16, 0, 15, 17);
            }
            let mut y = by + 16;
            for row in self.menu.encyc_scroll_top..self.menu.encyc_scroll_bottom {
                let c = row as usize;
                let t = self.menu.encyc_types[c] as usize;
                let pred = self.menu.encyc_damage[c];
                let mut x = bx + 16;
                g.set_color(13097429);
                let name: Vec<u16> = self.object_type_names[t].encode_utf16().collect();
                g.draw_string(&name, x, y + 2, 0);
                // 【BUG-031③】a.java:2524 `(240 + 面板宽 >> 1) − 75`：屏宽 240 随 layout（伤害列贴面板右缘）
                let dmg_x = ((self.layout.screen_w + self.menu.encyc_panel_w) >> 1) - 75;
                let name_dy = (font.height() - 19) >> 1;
                y += name_dy;
                if pred < 0 {
                    g.draw_image(&self.images.ui[17], dmg_x - 4, y + 2, 0);
                } else {
                    g.draw_image(&self.images.ui[16], dmg_x - 12, y + 2, 0);
                    x = dmg_x + 15;
                    crate::menu_family::paint_mini_frame(g, x - 1, y + 6, 48, 12);
                    paint_number(g, &self.images.ui[2], pred, x + 45, y + 8);
                }
                x = bx + 16;
                y += name_dy + 19 + 4;
                crate::menu_family::paint_mini_frame(g, x, y, 32, 32);
                if t > 40 {
                    let img = self.images.entity.get(t).cloned().flatten();
                    if let Some(img) = img {
                        let fw = self.type_frame_width_for(t as i32);
                        let off_x = (32 - fw) >> 1;
                        let draw_h = (img.height).min(31);
                        if off_x < 0 {
                            draw_image_clipped(g, &img, x, y + 31 - draw_h, -off_x, 0, fw, draw_h);
                        } else {
                            draw_image_clipped(g, &img, x + off_x, y + 31 - draw_h, 0, 0, fw, draw_h);
                        }
                    }
                } else {
                    g.set_clip(x, y, 32, 32);
                    if let Some(Some(img)) = self.images.entity.get(t) {
                        // 【R-2 换算收敛】居中式宽高用**逻辑**尺寸（=w/d）。
                        g.draw_image(
                            img,
                            x + ((32 - img.logical_width()) >> 1),
                            y + ((32 - img.logical_height()) >> 1),
                            0,
                        );
                    }
                    g.set_clip(x, y, self.layout.screen_w, self.layout.screen_h); // a.java:2553 `240, 320`
                }
                y += 4;
                x += 36;
                draw_image_clipped(g, &self.images.ui[7], x, y, 10, 0, 10, 13);
                x += 15;
                crate::menu_family::paint_mini_frame(g, x, y, 48, 12);
                let atk_v = self.enemies.atk.get(t.saturating_sub(41)).copied().unwrap_or(0);
                paint_number(g, &self.images.ui[2], atk_v, x + 40, y + 2);
                x += 52;
                draw_image_clipped(g, &self.images.ui[7], x, y, 20, 2, 10, 10);
                x += 15;
                g.set_color(512);
                crate::menu_family::paint_mini_frame(g, x, y, 48, 12);
                let def_v = self.enemies.def.get(t.saturating_sub(41)).copied().unwrap_or(0);
                paint_number(g, &self.images.ui[2], def_v, x + 40, y + 2);
                x -= 82;
                y += 12;
                draw_image_clipped(g, &self.images.ui[7], x, y + 5, 0, 2, 10, 10);
                x += 15;
                crate::menu_family::paint_mini_frame(g, x, y + 5, 48, 12);
                let hp_v = self.enemies.hp.get(t.saturating_sub(41)).copied().unwrap_or(0);
                paint_number(g, &self.images.ui[2], hp_v, x + 40, y + 7);
                x += 52;
                g.draw_image(&self.images.ui[1], x, y + 5, 0);
                x += 15;
                crate::menu_family::paint_mini_frame(g, x, y + 5, 48, 12);
                let gold = self.enemy_base_gold.get(t.saturating_sub(41)).copied().unwrap_or(0);
                x += 40;
                paint_number(g, &self.images.ui[2], gold, x, y + 7);
                y += 18;
                if row + 1 < self.menu.encyc_scroll_bottom {
                    x = bx + 16 - 5;
                    g.set_color(6435);
                    g.draw_line(x, y, x + self.menu.encyc_panel_w - 22, y);
                    y += 1;
                    g.set_color(4803902);
                    g.draw_line(x, y, x + self.menu.encyc_panel_w - 22, y);
                }
                y += 2;
            }
            y += 16;
            if self.menu.encyc_scroll_bottom < self.menu.encyc_count {
                draw_image_clipped(g, &self.images.ui[19], arrow_x, y + (self.frame_counter & 1) as i32, 16, 17, 15, 17);
            }
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.menu.softkeys.0, self.menu.softkeys.1, &self.layout);
    }
    /// run case 9（a.java:3885-3940）：-7 回 mode3；-6/-5 金币够则按光标加属性
    /// 并推进计数/价格，否则「没有足够的金钱」浮层；-2/-1 光标三行环绕。
    pub(super) fn tick_menu9(&mut self) {
        match self.key_value {
            -7 => self.transition_to(GameMode::Map),
            -6 | -5 => {
                if self.player.gold >= self.alchemy_price {
                    // 【BUG-045】未记账的消费：撤销按战前绝对值还原血金会白送回花掉的钱（复核 T3-R4）⇒ 清栈
                    self.undo.clear(crate::undo_journal::ClearReason::Irreversible);
                    self.player.gold -= self.alchemy_price;
                    match self.alchemy_cursor {
                        0 => self.player.hp += self.alchemy_gain_hp,
                        1 => self.player.atk += self.alchemy_gain_atk,
                        _ => self.player.def += self.alchemy_gain_def,
                    }
                    self.inc_alchemy_upgrade_count();
                    self.set_alchemy_price(Self::alchemy_price_for(self.alchemy_upgrade_count + 1));
                } else {
                    self.show_overlay_message(0, "没有足够的金钱", 0, 0);
                }
            }
            -2 => {
                self.alchemy_cursor = if self.alchemy_cursor < 2 { self.alchemy_cursor + 1 } else { 0 };
            }
            -1 => {
                self.alchemy_cursor = if self.alchemy_cursor > 0 { self.alchemy_cursor - 1 } else { 2 };
            }
            _ => {}
        }
    }
    /// paint case 9（a.java:2690-2740）：地图/实体/HUD/状态条 + 炼丹题头框
    ///（paintTitledBox(0,·)）+ 花费行 + 三行属性（光标高亮 + 双柱图标 + 数字）。
    pub(super) fn paint_menu9(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        self.paint_parallax(g, true);
        let tile_off = self.layout.view_top_y + self.advance_open_group_paint();
        self.view.paint_tiles(g, tileset, tile_off);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y);
        self.paint_hud(g);
        self.paint_status_bar(g);
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.menu.softkeys.0, self.menu.softkeys.1, &self.layout);

        let font = crate::paint::paint_font();
        let bw = 115i32; // f_int_75（a.java:810）
        let bh = font.height() + 90; // f_int_74（a.java:809）
        let bx = (self.layout.screen_w - bw - 22) >> 1;
        let by = (self.layout.screen_h - bh) >> 1;
        crate::menu_family::paint_titled_box(g, &self.images.ui[0], &self.images.ui[13], &[0, 1, 2], bx, by, bw + 22, bh);
        let wide = font.char_width('回' as u16) << 1;
        let mut x = (self.layout.screen_w - (wide + 16 + 48)) >> 1;
        let mut y = by + 21;
        g.set_color(7575203);
        g.draw_string(&"花费".encode_utf16().collect::<Vec<u16>>(), x, y, 0);
        x += wide + 4;
        g.draw_image(&self.images.ui[1], x, y + ((font.height() - 10) >> 1), 0);
        x += 60;
        paint_number(g, &self.images.ui[2], self.alchemy_price, x, y + 3 + ((font.height() - 10) >> 1));
        y += font.height() + 5;
        x = (self.layout.screen_w - bw) >> 1;
        g.set_color(549016);
        g.fill_rect(x, y + (self.alchemy_cursor << 4), bw, 16);
        g.draw_image(&self.images.ui[14], x + 10, y + 2 + (self.alchemy_cursor << 4), 0);
        g.draw_image_transformed(&self.images.ui[14], x + bw - 30, y + 2 + (self.alchemy_cursor << 4), 0, 8192);
        // 【BUG-031②】a.java:2715-2722 图标 x 97 / 加值 x 142 = 高亮行 x 62（`240 − 115 >> 1`）+ 35 / + 80
        // ——行内相对位，随居中的高亮行（classic 回代原值）。
        draw_image_clipped(g, &self.images.ui[7], x + 35, y + 4, 0, 2, 10, 10);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_hp, x + 80, y + 6);
        y += 16;
        draw_image_clipped(g, &self.images.ui[7], x + 35, y + 4, 11, 0, 8, 13);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_atk, x + 80, y + 6);
        y += 16;
        draw_image_clipped(g, &self.images.ui[7], x + 35, y + 4, 20, 2, 10, 11);
        paint_number(g, &self.images.ui[2], self.alchemy_gain_def, x + 80, y + 6);
        // 全局浮层（paint 尾部 a.java:3125-3210 对全部模式生效）
        self.paint_overlay_box(g);
    }
    pub(super) fn enter_menu16(&mut self) {
        // 【STAGE-B-006】原两行之后接壳层附加行（空 = 原版两行）
        self.menu.options = Some(crate::menu_family::OptionList::with_shell_rows(
            self.bgm.sound_enabled,
            self.minimap_enabled,
            &self.menu.shell_options,
        ));
        self.menu.softkeys = (0, 3);
    }
    /// run case 16（a.java:4143-4188）：-7/-6 回 menuReturnMode；-5/-4/-3 翻转
    /// （label0 音乐→音源态翻转 + saveGlobalProgress；label1 小地图→null/rebuild）；
    /// -2/-1 光标环绕。
    pub(super) fn tick_menu16(&mut self) {
        let Some(opt) = &mut self.menu.options else { return };
        match self.key_value {
            -7 | -6 => self.transition_to(self.menu.menu_return_mode),
            -5 | -4 | -3 => {
                let cur = opt.highlight as usize;
                if let Some((id, v)) = opt.toggle_shell_row(cur) {
                    // 【STAGE-B-006】壳层附加行：只翻该行并回写 shell_options——不碰声音 / 小地图 / SKY_WAR。
                    // 早退：本 match 是函数末语句（原臂文本保持原样）；若日后在 match 后加逻辑，须改回 if/else。
                    crate::menu_family::set_shell_option(&mut self.menu.shell_options, id, v);
                    return;
                }
                opt.checked[cur] = !opt.checked[cur];
                match opt.labels[cur] {
                    0 => {
                        // 开 ⇒ volumeLevel=60 + playBgm(f_bool_28?3:2)，关 ⇒ closeAudioPlayer
                        //（a.java:4157-4167；EV-toggle-exit T521/T561）。
                        self.bgm.toggle_sound();
                        // saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）
                    }
                    1 => {
                        if !opt.checked[1] {
                            self.set_minimap(None);
                        } else if self.menu.menu_return_mode == GameMode::InGameMenu {
                            self.build_minimap();
                        }
                    }
                    _ => {}
                }
            }
            -2 => {
                opt.highlight += 1;
                if opt.highlight > opt.len() as i32 - 1 {
                    opt.highlight = 0;
                }
            }
            -1 => {
                opt.highlight -= 1;
                if opt.highlight < 0 {
                    opt.highlight = opt.len() as i32 - 1;
                }
            }
            _ => {}
        }
    }
    pub(super) fn paint_dialog(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        self.paint_parallax(g, true);
        let tile_off = self.layout.view_top_y + self.advance_open_group_paint();
        self.view.paint_tiles(g, tileset, tile_off);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y);
        self.paint_tap_markers(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y); // 【STAGE-B-008】
        self.popups.draw(g, &self.images.popup);
        self.paint_status_bar(g);
        let show_box = self.dialog.overlay_text.is_some() && !self.dialog.overlay_active && self.dialog.dialog_phase != 4;
        if show_box {
            let font = crate::paint::paint_font();
            let line_h = font.height() + 4;
            let dialog_block_h = (line_h << 1) + 12;
            let dialog_block_y0 = self.layout.screen_h - dialog_block_h;
            // 【STAGE-B-002 口径修正 2026-18-18】底部面板=**经典尺寸固定+整体居中**
            //（非随屏拉伸——二轮返修走偏）：面板内相对布局 classic 同构（常量回代），
            // 面板整体平移 dx；classic dx=0 零偏离。
            let dx = self.bottom_panel_dx();
            self.paint_hud_frame(g, dx, dialog_block_y0, 80, dialog_block_h);
            let mut cx = 80 + dx;
            while cx < 214 + dx {
                g.set_clip(cx, dialog_block_y0, 16, 16);
                g.draw_image(&self.images.ui[0], cx - 26, dialog_block_y0, 0);
                cx += 16;
            }
            g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
            // a.java:2828：edge patch 用 draw_y=dialog_block_y0（+=16 之前）
            crate::menu_family::draw_edge_patch(g, &self.images.ui[0], 214 + dx, dialog_block_y0, 0, 0, 26, 16, 1);
            g.set_color(HUD_FILL);
            g.fill_rect(80 + dx, dialog_block_y0 + 16, 149, dialog_block_h - 16);
            // a.java:2832-2836：右条带循环 (229, draw_y) 画 ui[8][0]@176——
            // draw_y 从 dialog_block_y0+16 步进 16 直到 var158+var11（320）
            let mut draw_y = dialog_block_y0 + 16;
            while draw_y < dialog_block_y0 + dialog_block_h {
                g.set_clip(229 + dx, draw_y, 11, 16);
                g.draw_image(&self.images.ui[0], 176 + dx, draw_y, 0);
                draw_y += 16;
            }
            g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
            if self.dialog.dialog_speaker > 0 {
                if let Some(img) = &self.images.entity[self.dialog.dialog_speaker as usize] {
                    if self.dialog.dialog_speaker != 72 {
                        let fw = self.type_frame_width_for(self.dialog.dialog_speaker);
                        draw_image_clipped(g, img, ((80 - fw) >> 1) + dx, self.layout.screen_h - img.height, 0, 0, fw, img.height);
                    } else {
                        // a.java:2852 `drawImageClipped(var9, 24, 275, 0, 0, 32, 45)`：275 = 320 − 45
                        // （定尺 32×45 剪裁贴屏底，同段两臂 `320 − h` 同义）⇒ 公式 screen_h − 45，
                        // classic 回代 275 零偏离。【BUG-029】旧照抄字面量 275 ⇒ Modern 下牛魔王
                        // 立绘画到地图中左、对话框头像格空。
                        draw_image_clipped(g, img, 24 + dx, self.layout.screen_h - 45, 0, 0, 32, 45);
                    }
                }
            } else if self.dialog.dialog_speaker == 0 {
                let face = &self.images.actor_face;
                g.draw_image(face, 40 - (face.width >> 1) + dx, self.layout.screen_h - face.height, 0);
            } else {
                g.set_color(WHITE);
                let q: Vec<u16> = "?".encode_utf16().collect();
                g.draw_string(&q, 40 + dx, dialog_block_y0 + ((dialog_block_h - font.height()) >> 1), 17);
            }
            g.set_color(10473684);
            self.paint_wrapped_text(g, 85 + dx, dialog_block_y0 + 12, 129, dialog_block_h - 12, false);
            if self.dialog.overlay_page_top > 0 {
                draw_image_clipped(g, &self.images.ui[15], 220 + dx, self.layout.screen_h - line_h - 10, 0, 0, 7, 9);
            }
            if self.dialog.overlay_page_bottom < self.dialog.overlay_total_lines {
                draw_image_clipped(g, &self.images.ui[15], 220 + dx, self.layout.screen_h - line_h + 1, 7, 0, 7, 9);
            }
            if (self.dialog.dialog_phase == 6 || self.dialog.dialog_phase == 1)
                && self.dialog.dialog_page >= 0
                && (self.dialog.dialog_page as usize) < self.dialog.dialog_data.bubbles.len()
            {
                let bubble = self.dialog.dialog_data.bubbles[self.dialog.dialog_page as usize];
                if bubble >= 0 {
                    let (bx, by) = if self.dialog.dialog_anchor >= 0 {
                        let e = self.dialog.dialog_anchor as usize;
                        (self.entities.items[e].px + 16, self.entities.items[e].py - 30)
                    } else {
                        (self.view.player_px + 16, self.view.player_py - 30)
                    };
                    let by = if self.dialog.dialog_speaker == 84 { by - 20 } else { by };
                    // a.java:2887：标记图 = face 包第 bubble 张（26×22），非实体表
                    if let Some(face) = self.images.faces.get(bubble as usize) {
                        g.draw_image(face, bx + self.view.cam_x, by + self.view.cam_y + 20, 0);
                    }
                }
            }
            if self.dialog.dialog_phase == 6 {
                if let Some(arrow) = &self.images.map211 {
                    g.draw_image(
                        arrow,
                        self.view.cam_x + 5 + (self.dialog.dialog_target.0 << 5),
                        self.view.cam_y + (self.dialog.dialog_target.1 << 5) - 12 + self.render.bob_offset,
                        0,
                    );
                }
            }
        } else {
            self.paint_hud(g);
        }
    }
}
