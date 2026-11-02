//! task-15 步② 渲染簇（move-only 自 scene.rs——只读绘制面，纯剪切零行为差）。
use super::*; // scene.rs 私有面可见（子模块）——纯剪切语义保持

impl GameScene {
    /// loadFloorData（a.java:9404-9438）：maplv 装载 + initCellGrid 格索引重建 +
    /// 可行走重建 + m_122 实体生成 + sortEntitiesByY 排序。
    /// `floor_assets`：按层号索引的换层资产（索引缺省 = 该层不可达）。
    #[allow(clippy::too_many_arguments)]
    /// 浮层框绘制（paint 公共尾 a.java:3129-3168）：盒框 + kind 分派内容 +
    /// 左右软键图标。mode 3/4/7/10/16/19 各 paint 共用。
    pub(super) fn paint_overlay_box(&mut self, g: &mut SoftGraphics<'_>) {
        // 提示浮层（paint 公共尾 a.java:3129-3168，!floorTransitioning && overlayActive）：
        // 盒框 + kind 0/2/4/5 纯文本 / kind 1/3 图标+文本 + 软键图标
        if !self.transitioning && self.overlay_active {
            let bx = (self.layout.screen_w - self.overlay_box_w) >> 1;
            let by = (self.layout.screen_h - self.overlay_box_h) >> 1;
            crate::menu_family::paint_box_frame(g, &self.images.ui[0], bx, by, self.overlay_box_w, self.overlay_box_h);
            let font = crate::paint::paint_font();
            let mut tx = bx + 16;
            let ty = by + 16;
            if matches!(self.overlay_kind, 0 | 2 | 4 | 5) {
                g.set_color(WHITE);
                self.paint_wrapped_text(g, tx, ty, OVERLAY_TEXT_W, OVERLAY_TEXT_MAX_H, true);
            } else {
                paint_mini_frame(g, tx, ty, 32, 32);
                // 图标 = entityTypeImage[overlayKindItem]（itemUseTargetType——拾取时记录）
                // 【R-2 换算收敛】居中式 (32-img.width)>>1：宽高用**逻辑**尺寸（=w/d）。
                if let Some(img) = &self.overlay_icon {
                    g.draw_image(
                        img,
                        tx + ((32 - img.logical_width()) >> 1),
                        ty + ((32 - img.logical_height()) >> 1),
                        0,
                    );
                }
                tx += 42;
                g.set_color(16770173);
                if let Some(name) = &self.overlay_item_name {
                    g.draw_string(name, tx, ty + ((32 - font.height()) >> 1), 0);
                }
                g.set_color(WHITE);
                self.paint_wrapped_text(g, bx + 16, ty + 36, OVERLAY_TEXT_W, self.overlay_box_h - 32 - 36, true);
            }
            let iy = by + self.overlay_box_h - 12 - 18;
            if self.overlay_left_icon != 0 {
                draw_image_clipped(g, &self.images.ui[11], bx + 13, iy + 6, (self.overlay_left_icon as i32 - 1) * 12, 0, 12, 10);
            }
            if self.overlay_right_icon != 0 {
                draw_image_clipped(g, &self.images.ui[11], bx + self.overlay_box_w - 25, iy + 6, (self.overlay_right_icon as i32 - 1) * 12, 0, 12, 10);
            }
        }
    }
    /// paintWrappedText（a.java:5044-5121）：整行按 \\c 色段 flush（drawSubstring
    /// 语义 = 片段 drawString）+ 跨页起始色继承 + \\r 恢复 + 翻页三角。
    /// 忠实点：段起始 var12/游标 var16 的推进序、非法 hex 从反斜杠重画、
    /// 未知转义丢反斜杠、var15 每行清 0 的 `!= -1` 怪癖。
    /// paintWrappedText（a.java:5021-5119）：2026-10-17 单源适配——绘制本体 =
    /// `menu_family::paint_wrapped_lines`（边缘语义以 Java 为准：overlaySavedColor
    /// 开场置 `getColor()`、每次 `\c` 刷新、跨行持久；旧拷贝行内/跨 paint 两向偏离）。
    pub(super) fn paint_wrapped_text(&mut self, g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32, _max_h: i32, arrows: bool) {
        let font = crate::paint::paint_font();
        let line_h = font.height() + 4;
        let lines: Vec<Vec<u16>> = self
            .overlay_lines
            .iter()
            .map(|l| l.encode_utf16().collect())
            .collect();
        self.overlay_saved_color = crate::menu_family::paint_wrapped_lines(
            g,
            &lines,
            self.overlay_page_top,
            self.overlay_page_bottom,
            self.overlay_total_lines,
            line_h * self.overlay_lines_per_page,
            line_h,
            x,
            y,
            w,
            arrows,
            self.frame_counter as i32,
            &font,
        ) as i32;
    }
    /// paint case 3（a.java:2385-2422）。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage) {
        // 【A 黄2·2026-10-23】Modern 对称清屏（照 BootMachine 先例）——
        // 升格钉当场红实证残留（局部绘制态留底，旧靠 paint_parallax 连带覆盖
        // ="暂不残留"非保障）；classic 路径零动（零漂移）。
        if self.layout.preset == crate::layout::Preset::Modern {
            g.set_color(crate::layout::WHITE);
            g.fill_rect(0, 0, self.layout.screen_w, self.layout.screen_h);
        }
        if self.game_mode == GameMode::EnemyInfo {
            self.paint_menu5(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::About {
            // paint case 16（a.java:2963-2990）：parallax(false) + 图标条 +
            // paintBoxFrame + 选项行（OptionList::paint）【task-14：内容居中；
            // 视差=全幅层自豁免】
            self.paint_parallax(g, false);
            let mut f = self.strip_frame;
            let sx = crate::title::menu_strip_x(self.layout.screen_w);
            crate::title::paint_icon_strip_at(g, &self.images.menu_strip, sx, 10, 34, 68, &mut f);
            self.strip_frame = f;
            if let Some(opt) = &self.options {
                let font = crate::paint::paint_font();
                opt.paint(g, &self.images.ui, &font, &self.layout);
            }
            m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::SaveSlot {
            // paint case 7（a.java:2679-2682）：paintSlotTable(true) 存档变体
            self.paint_slot_list(g);
            self.paint_overlay_box(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::Help {
            // paint case 15（a.java:2958-2962）
            self.paint_menu15(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::InGameMenu || self.game_mode == GameMode::Shop {
            // paint case 4/19 共享（a.java:2427-2490）
            // 【task-14 实测更正 2026-10-20】paint_menu_list=公式锚（center_x）
            // ⇒ 不加臂平移（双移=右偏 88px，ops 差分实证）；公式化=STAGE-B-004a 既定机制
            g.set_origin(0);
            self.paint_menu_list(g);
            self.paint_overlay_box(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::ItemPanel {
            self.paint_item_panel(g);
            self.paint_overlay_box(g);
            // 面板内道具触发 changeFloor：遮幅公共尾同样共跑（a.java:3177-3215）
            self.paint_wipe_tail(g);
            return;
        }
        if self.game_mode == GameMode::Ending {
            self.paint_ending(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::RecordBrowser {
            self.paint_mode12(g);
            return;
        }
        if self.game_mode == GameMode::Dialog {
            self.paint_dialog(g, tileset);
            // 【BUG-049】提示浮层在 paint 公共尾对所有模式共跑（a.java:3129-3168；switch 各 case 皆 break、无提前 return）：
            // mode 11 里弹出的提示（脚本 / 点地寻路行走撞上打不过的怪、无钥匙的门等）原先不画——画面无提示却吞键。
            self.paint_overlay_box(g);
            return self.paint_wipe_tail(g);
        }
        if self.game_mode == GameMode::Alchemy {
            // paint case 9（a.java:2690-2740）
            self.paint_menu9(g, tileset);
            return self.paint_wipe_tail(g);
        }
        self.paint_parallax(g, true);
        let tile_off = self.layout.view_top_y + self.advance_open_group_paint();
        self.view.paint_tiles(g, tileset, tile_off);
        self.paint_entities(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y);
        self.paint_tap_markers(g, self.view.cam_x, self.view.cam_y + self.layout.view_top_y); // 【STAGE-B-008】
        // 小地图（optionChecked[1] && minimapImage）：右上角，玩家点
        if self.minimap_enabled {
            if let Some(mm) = &self.minimap {
                let ppc = self.layout.minimap_px_per_cell;
                let dot = self.layout.minimap_dot_px;
                let off = (ppc - dot) / 2; // 点居格中（classic 4/4 ⇒ 0，零偏离）
                let x = self.layout.screen_w - ((self.view.wide + 1) * ppc);
                g.draw_image(mm, x, self.layout.view_top_y, 0);
                g.set_color(MINIMAP_PLAYER_DOT);
                // a.java:2397-2398：点 = playerCellX/Y（**格坐标**——步进中
                // 与像素 >>5 不同，上/左行时像素已进目标格而格坐标未更新）
                g.fill_rect(
                    x + self.player_cell_x * ppc + off,
                    self.layout.view_top_y + self.player_cell_y * ppc + off,
                    dot,
                    dot,
                );
            }
        }
        self.paint_hud(g);
        self.paint_status_bar(g);
        // 楼梯指示（f_bool_13；稳态 false—— proximity 更新在 lockCameraOn/行走批次）
        self.popups.draw(g, &self.images.popup);
        // paint case 3 内断言软键 (1,3)（a.java:2421-2422）——菜单族改写的
        // softkey 落回 mode 3 时由 paint 复位（Java 字段赋值语义）
        self.softkeys = (1, 3);
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
        // 提示浮层（paint 公共尾 a.java:3129-3168，!floorTransitioning && overlayActive）
        if !self.transitioning && self.overlay_active {
            self.paint_overlay_box(g);
        }
        self.paint_wipe_tail(g);
    }
    /// 换层遮幅（paint 公共尾 a.java:3177-3215——**所有 mode 后置共跑**，
    /// 非 mode3 专属）：状态机先行、后画黑格。闭合（wipe_closing）1..=4
    /// 全黑 → 中段换层（swap_floor）→ 开启 4..=0。mode10 面板内道具触发
    /// changeFloor 同样经过此尾（迭代 45 T401 实证缺口）。
    pub(super) fn paint_wipe_tail(&mut self, g: &mut SoftGraphics<'_>) {
        // 【task-14】遮幅尾=全画布层——归零平移（各臂内容居中后统一复位点）
        g.set_origin(0);
        if self.transitioning {
            if self.wipe_closing {
                self.wipe += 1;
                if self.wipe > 4 {
                    self.wipe = 4;
                    self.wipe_closing = false;
                    self.swap_floor();
                }
            } else {
                self.wipe -= 1;
                if self.wipe <= 0 {
                    self.wipe_closing = true;
                    self.transitioning = false;
                }
            }
            let size = self.wipe * 2;
            let off = 4 - self.wipe;
            g.set_color(BLACK);
            // 【BUG-022 修复·STAGE-B-002 铺满】网格界随 Layout 铺满画布——
            // Java 写死 40 行×30 列（=传统 240×320 全覆盖，deobf a.java:3217-3222
            // `var176<40`/`var181<30`），扩屏后只盖左上 240×320（切屏动画不覆盖
            // 全屏）。格数=ceil(屏/格距)：classic 240/8=30 列、320/8=40 行 代入
            // =原值零变化（格距 WIPE_CELL_PX=8 见 layout）。
            let cell = WIPE_CELL_PX;
            let cols = (self.layout.screen_w + cell - 1) / cell;
            let rows = (self.layout.screen_h + cell - 1) / cell;
            for row in 0..rows {
                let y = off + row * cell;
                for col in 0..cols {
                    g.fill_rect(off + col * cell, y, size, size);
                }
            }
        }
    }
    /// drawParallaxBackdrop（a.java:6850-6871）：滚动 -1/拍（PARALLAX_WRAP 回绕），
    /// PARALLAX_COL_STEP 列步进、屏高行步进、列内交替镜像。
    /// 【STAGE-B-002 返修②】列/行界与基线随 Layout 铺满画布（classic 240×320
    /// 代入=原值零变化：base=270-320+6=-44、列界 240、行界 -320）。
    pub(super) fn paint_parallax(&mut self, g: &mut SoftGraphics<'_>, game_view: bool) {
        // 【task-14 豁免】背景=全幅层（铺满画布）——不随 origin 平移（否则左缘
        // 露空）；绘后恢复调用方平移（ops 面不受影响=origin 隐式）。
        let saved_origin = g.origin();
        g.set_origin(0);
        self.backdrop_scroll -= 1;
        if self.backdrop_scroll < PARALLAX_WRAP {
            self.backdrop_scroll = 0;
        }
        let bh = self.images.backdrop.height;
        let base_y = if game_view { self.layout.view_bottom_y - bh + 6 } else { 0 };
        let bw = self.images.backdrop.width as usize;
        let mut col_x = self.backdrop_scroll;
        while col_x < self.layout.screen_w {
            if self.layout.screen_h > bh {
                // 【v6 返修】画布高于背景图 ⇒ 边缘**钳制延展**（顶带=图顶行色、
                // 底带=图底行色）+ 仅画基行——镜像铺贴会把渐变反射（沙地翻上屏顶/
                // 天空翻下底带）=用户"颜色奇怪/过渡不自然"根因（v5 镜像补铺同病）；
                // classic 画布=图高 ⇒ 走下方原式原样。
                let top = self.images.backdrop.argb[0];
                let bottom = self.images.backdrop.argb[(bh as usize - 1) * bw];
                g.set_color(top & 0x00FF_FFFF | 0xFF00_0000);
                g.fill_rect(col_x, 0, bw as i32, base_y);
                g.set_color(bottom & 0x00FF_FFFF | 0xFF00_0000);
                g.fill_rect(col_x, base_y + bh, bw as i32, self.layout.screen_h - (base_y + bh));
                g.draw_image(&self.images.backdrop, col_x, base_y, 0);
            } else {
                // classic 原式（a.java:6850-6871）：行内交替镜像，一行可见=原样零变化
                let mut mirrored = false;
                let mut y = base_y;
                while y > -self.layout.screen_h {
                    if mirrored {
                        g.draw_image_transformed(&self.images.backdrop, col_x, y, 0, 2);
                    } else {
                        g.draw_image(&self.images.backdrop, col_x, y, 0);
                    }
                    mirrored = !mirrored;
                    y -= bh;
                }
            }
            col_x += PARALLAX_COL_STEP;
        }
        g.set_origin(saved_origin);
    }
    /// 【STAGE-B-008 · spec §6】点地反馈层：目标框（金）、闪红（红框 + 叉）、选中怪物（闪烁框；持火眼金睛（13）旁标预计掉血，
    /// 与观察模式战后数字条同源同位（a.java:6617-6638：通用怪先 −ENTITY_LIFT、69/67 分支不抬；标记为静态位、无波动相位）。
    /// 层级 = 实体层之上、HUD / 浮层之下。classic 零绘制 op。
    pub(super) fn paint_tap_markers(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        if self.layout.preset != crate::layout::Preset::Modern {
            return;
        }
        pub(super) fn frame(g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32) {
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
                // 与观察模式数字条同基位：通用怪先 −ENTITY_LIFT（a.java:6613 var10 −= 4），69/67 不抬（复核 T8-R1）
                let lift = if s.type_ != 69 && s.type_ != 67 { ENTITY_LIFT } else { 0 };
                let digit = self.predict_hp_loss(s.type_, true);
                if digit >= 0 {
                    crate::popup::draw_digit_strip(g, &self.images.battle_strip, digit, sx + 30, sy - lift - 17);
                } else {
                    draw_image_clipped(g, &self.images.battle_strip, sx + 14, sy - lift - 17, 70, 0, 7, 9);
                }
            }
        }
    }
    pub fn paint_entities(&mut self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        entity::advance_bob(&mut self.bob_offset, &mut self.bob_rising);
        let bob = self.bob_offset;
        let mut player_drawn = false;
        let cull_bottom = self.layout.view_top_y + self.view.view_h;

        for e in 0..self.entities.count {
            if !self.entities.items[e].removed && self.entities.items[e].visible {
                let ex = self.entities.items[e].px;
                let ey = self.entities.items[e].py;
                let w = self.entities.items[e].sprite_w;
                let h = self.entities.items[e].sprite_h;
                let state = self.entities.items[e].state;
                let t = self.entities.items[e].type_;
                let mut sx = cam_x + ex;
                let mut sy = cam_y + ey;
                if sx >= -w && sx <= self.view.view_w && sy >= ENTITY_CULL_TOP && sy <= cull_bottom {
                    if let Some(img) = &self.images.entity[t as usize] {
                        // 玩家插入（单次；blocksPlayerInsert 在 0/5/6/7/8/10 型为 true，
                        // 这些类型占据插入窗口时不插画玩家——a.java:6520）
                        if !player_drawn
                            && self.view.player_px > ex - PLAYER_INSERT_WINDOW
                            && self.view.player_px < ex + PLAYER_INSERT_WINDOW
                            && self.view.player_py >= ey - PLAYER_INSERT_WINDOW
                            && ey > self.view.player_py
                            && !BLOCKS_PLAYER_INSERT.get(t as usize).copied().unwrap_or(false)
                        {
                            self.paint_player(g, cam_x, cam_y);
                            player_drawn = true;
                        }
                        let anim = self.entities.items[e].anim_row as usize;
                        let frame = self.entities.items[e].anim_frame;
                        // 帧偏移 = 帧宽 × animOffsetTable[行][帧]（Java open_n 算式）
                        let frame_off = w * ANIM_OFFSET_TABLE[anim][frame as usize];
                        // 注：横向居中 (32-w)>>1 在**各臂内**取 sx 现值计算——
                        // case 8 抖动 / case 1 的 6/9 特例先改 sx 再用（Java cur_idx 语义）
                        let cx_off = (CELL_PX - w) >> 1;
                        match self.entity_category(t) {
                            1 => {
                                if t == 6 {
                                    sy += 5 - bob;
                                } else if t == 9 {
                                    sx += CELL_PX;
                                }
                                let img = if state == 1 { &self.images.opened_door } else { img };
                                draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                            }
                            2 => {
                                if state == 1 {
                                    draw_image_clipped(g, &self.images.opened_door, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else if state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t < SMALL_ITEM_MAX_TYPE {
                                    g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                    g.draw_image(img, sx + cx_off, sy - (h - ANCHOR_MARGIN_24) + bob, 0);
                                } else {
                                    g.draw_image(img, sx + cx_off, sy - (h - ANCHOR_MARGIN_30), 0);
                                }
                            }
                            4 => {
                                g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                draw_image_clipped(g, img, sx + cx_off, sy - (h - ANCHOR_MARGIN_24) + bob, frame_off, 0, w, h);
                            }
                            8 => {
                                if state == 3 {
                                    // 战斗目标抖动（a.java:6543-6541）：x/y 各随机 ±2
                                    sx += self.rng.random_below(5) - 2;
                                    sy += self.rng.random_below(5) - 2;
                                }
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t == 69 {
                                    // a.java:6590-6600：型69 拼装——阴影 +40、
                                    // 全身图整图直画 (x+3, y−49+bob)（无裁剪无
                                    // 帧偏移）、帧偏移 >0 时 xtq[5] 26×13 效果帧
                                    //（src = 26×(偏移−1)）。
                                    g.draw_image(&self.images.shadow, sx + 40, sy + 22, 0);
                                    let raw_off = ANIM_OFFSET_TABLE[anim][frame as usize];
                                    let sx69 = sx + 3;
                                    let sy69 = sy - 49 + self.bob_offset;
                                    g.draw_image(img, sx69, sy69, 0);
                                    if raw_off > 0 {
                                        draw_image_clipped(
                                            g,
                                            &self.images.xtq[5],
                                            sx69 + 32,
                                            sy69 + 27,
                                            (raw_off - 1) * 26,
                                            0,
                                            26,
                                            13,
                                        );
                                    }
                                } else if t != 67 {
                                    g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                    sy -= ENTITY_LIFT;
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else {
                                    // 型 67 多段拼装（a.java:6580-6603 逐字）：
                                    // 影子 @ (x+8, y+22)（6581）→ 基点 (x+16,
                                    // y+16)（6582-6586 倒手）→ SEG67_FRAMES[
                                    // 裸帧偏移] 5 段，每段片件 SEG67_PIECES[
                                    // seg[0]] 画 xtq[seg.sheet] @ (基点+dx,
                                    // 基点+dy) 源 (srcX, srcY) w×h（6591-6601）。
                                    // 裸帧偏移 = ANIM_OFFSET_TABLE[anim][frame]
                                    // **不乘精灵宽**（a.java:6578，对照通用臂
                                    // 6528 乘宽）。型 67 精灵盒 32×32（6323-6324）。
                                    // 【2026-10-16 注】deobf:6592 将片件表元素
                                    // 强转 (Object[]) 与 :682 的 byte[][] 声明
                                    // 矛盾——C 变体（deobf 重编译）画 67 即
                                    // ClassCastException，系投影伪影非原字节码
                                    // 语义；Rust 侧以 (sheet,srcX,srcY,w,h) 元组
                                    // 建模规避，与 A 变体 fx-boss67a 逐帧像素
                                    // 等价。类型语义合同移交
                                    // reference/oracle/scenarios/
                                    // bug-draft-paint67-deobf-cast.md（lead 入库）。
                                    g.draw_image(&self.images.shadow, sx + 8, sy + 22, 0);
                                    let base_x = sx + 16;
                                    let base_y = sy + 16;
                                    let seg_frame = ANIM_OFFSET_TABLE[anim][frame as usize] as usize;
                                    let seg67 = crate::resource_cfg::global().seg67();
                                    for seg in seg67.frames[seg_frame].clone() {
                                        let (sheet, src_x, src_y, w, h) =
                                            seg67.pieces[seg.0 as usize];
                                        draw_image_clipped(
                                            g,
                                            &self.images.xtq[sheet as usize],
                                            base_x + seg.1,
                                            base_y + seg.2,
                                            src_x,
                                            src_y,
                                            w,
                                            h,
                                        );
                                    }
                                }
                                // walkPhase==2 战后数字条（a.java:6617-6638）：
                                // showBattleDigits 开关 + iconStrip 波动 + 预测伤害 drawDigitStrip
                                if self.walk_phase == 2 && self.show_battle_digits {
                                    let f8 = (self.frame_counter & 7) as usize;
                                    let d = crate::title::STRIP_DX[f8];
                                    let e = crate::title::STRIP_DY[f8];
                                    g.draw_image(&self.images.battle_icon, sx - 3 + d, sy - 24 + e, 0);
                                    let digit = self.battle_digit_by_type[t as usize];
                                    if digit >= 0 {
                                        crate::popup::draw_digit_strip(
                                            g,
                                            &self.images.battle_strip,
                                            digit,
                                            sx + 30 + d,
                                            sy - 17 + e,
                                        );
                                    } else {
                                        draw_image_clipped(g, &self.images.battle_strip, sx + 14 + d, sy - 17 + e, 70, 0, 7, 9);
                                    }
                                }
                            }
                            16 => {
                                if state != 1 && state != 2 {
                                    if let Some(img15) = &self.images.entity[15] {
                                        g.draw_image(img15, sx + 1, sy + 8 + bob, 0);
                                    }
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - ANCHOR_MARGIN_24) + bob, frame_off, 0, w, h);
                                } else {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                }
                            }
                            32 => {
                                g.draw_image(&self.images.shadow, sx + SHADOW_DX, sy + SHADOW_DY, 0);
                                if state == 1 || state == 2 {
                                    draw_image_clipped(g, &self.images.open_anim, sx + OPEN_ANIM_DX, sy + OPEN_ANIM_DX - (frame * OPEN_ANIM_STEP), frame_off, 0, OPEN_ANIM_W, OPEN_ANIM_H);
                                } else if t != 72 {
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - ANCHOR_MARGIN_16) + bob, frame_off, 0, w, h);
                                } else if self.route_entity != e as i32 {
                                    // 型 72 非追踪实体：静态（无 bob），y-(h-32)（a.java:6657-6652）
                                    draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, 0, w, h);
                                } else {
                                    // 追踪实体：步方向帧行（f_byte_19；==3 时镜像，a.java:6663-6664）
                                    let mut src_y = h * self.route_dir;
                                    if self.route_dir == 3 {
                                        src_y -= h;
                                        crate::menu_family::draw_edge_patch(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, src_y, w, h, 1);
                                    } else {
                                        draw_image_clipped(g, img, sx + cx_off, sy - (h - CELL_PX), frame_off, src_y, w, h);
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // 空图文字标签分支（a.java:6672-6683）：**全语料不可达
                        // 死码**——A 级证明链：空图类型仅 82/83（m_122 图表
                        // 9456-9485 二者恒 null）；82/83 出层即 hidden
                        // （a.java:9605/9618）；entityVisible 置 true 写点清单
                        // （6407/5413/5455/8345/9638）无一覆盖 82/83；语料
                        // CES/ADD 类型参数不含 82/83；全 55 层 sprite 型 82=0、
                        // 83=52 全隐藏 ⇒ 循环门 entityVisible 恒 false ⇒ 分支
                        // 不可达（evidence-todos.md T-3 裁定 B 升 A 且更强=全
                        // 语料域）。保真期**不删分支**：todo!() 作死码哨兵；
                        // 分支几何公式已取证（6673-6682），若未来语料以
                        // CES/ADD 生成 82/83 或 SWD 改型 83 则按公式实现。
                        todo!(
                            "paintEntityLayer 空图标签分支：全语料不可达死码（evidence-todos.md T-3）"
                        )
                    }
                }
            }
        }

        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        if !player_drawn {
            self.paint_player(g, cam_x, cam_y);
        }
        // walkPhase==5 星光（a.java:6692-6704）：battleTick&7 选帧，sparkFrames
        // 帧号×4 取 sparkQuads 四联组（sx,sy,w,h）
        if self.walk_phase == 5 && self.battle_target >= 0 {
            let e = self.battle_target as usize;
            // a.java:6692-6687：星光坐标用**裸 cameraPixelX/Y**（非 paintEntityLayer 入参
            // 的 cameraPixelY+view_top——尾段直接读字段）
            let x = self.view.cam_x + self.entities.items[e].px + 16;
            let y = self.view.cam_y + self.entities.items[e].py + 32;
            let phase = (self.battle_tick & 7) as usize;
            let frame = SPARK_FRAME[phase];
            if frame > 0 {
                let q = ((frame - 1) << 2) as usize;
                draw_image_clipped(
                    g,
                    &self.images.spark_sheet,
                    x - (SPARK_QUADS[q + 2] >> 1),
                    y - (SPARK_QUADS[q + 3] >> 1),
                    SPARK_QUADS[q],
                    SPARK_QUADS[q + 1],
                    SPARK_QUADS[q + 2],
                    SPARK_QUADS[q + 3],
                );
            }
        }
        // 楼梯浮标（paintEntityLayer 尾段 a.java:6712-6720）
        if let Some((ux, uy)) = self.entities.stair_up {
            let x = cam_x + ux;
            let y = cam_y + uy;
            if x >= MARKER_CULL_MARGIN && x <= self.view.view_w && y >= ENTITY_CULL_TOP && y <= cull_bottom {
                g.draw_image(&self.images.stair_icons.0, x + MARKER_UP_DX, y + MARKER_DY + bob, 0);
            }
        }
        if let Some((dx, dy)) = self.entities.stair_down {
            let x = cam_x + dx;
            let y = cam_y + dy;
            if x >= MARKER_CULL_MARGIN && x <= self.view.view_w && y >= ENTITY_CULL_TOP && y <= cull_bottom {
                g.draw_image(&self.images.stair_icons.1, x, y + MARKER_DY + bob, 0);
            }
        }
    }
    /// paintPlayerSprite（a.java:5841-5909）：玩家。facing 0/1/2/3 = 下/上/右/左行
    /// （PLAYER_ANCHORS 的 (dx,dy,行)；case 3 为镜像——drawEdgePatch transform 1）；
    /// walkPhase==2 残影（afterimageX/Y）战斗批次。
    pub fn paint_player(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32) {
        // 【STAGE-B-002 偏离点·观察形态(a)（lead 裁决 2026-10-19，用户批准）】
        // 空格观察形态（walkPhase==2 未锁定）modern：**角色恒居中**（与残影蛇
        // 收敛点同锚=视口中心 easeAfterimages 目标），地图随相机滑动=以角色
        // 为中心环视；classic 保持原机制（cam+player）零偏离。
        let soul_view = self.walk_phase == 2
            && !self.view_locked
            && self.layout.preset == crate::layout::Preset::Modern;
        let (px, mut py) = if soul_view {
            // 与残影蛇收敛点严格同点（easeAfterimages 目标换屏坐标=视口中心）：
            // cam_y 含 view_top 分量（paint_entities 传参），故回补之
            (
                (self.view.view_w - 41) >> 1,
                (cam_y - self.view.cam_y) + (self.view.view_h >> 1) + self.bob_offset,
            )
        } else {
            (cam_x + self.view.player_px, cam_y + self.view.player_py)
        };
        if !soul_view && self.player_bob_applied {
            py -= self.bob_offset;
        }
        g.draw_image(&self.images.shadow, px + SHADOW_DX, py + SHADOW_DY, 0);
        let frame_x = PLAYER_FRAME_TABLE[self.player_frame as usize] * PLAYER_FRAME_W;
        let (fw, fh) = (PLAYER_FRAME_W, PLAYER_FRAME_H);
        let idx = self.facing.clamp(0, 3) as usize;
        let (dx, dy, row) = PLAYER_ANCHORS[idx];
        let sy = fh * row;
        // walkPhase==2 残影（a.java:5860-5904）：facing 1 在主精灵**后**画，
        // 其余在前；i=3..0 逆序。残影源 = playerGhostSheet（actor 条带别名）
        let ghost = self.walk_phase == 2 && self.facing != 1;
        if ghost {
            self.paint_afterimages(g, cam_x, cam_y, frame_x);
        }
        if self.facing == 3 {
            crate::menu_family::draw_edge_patch(g, &self.images.actor, px + dx, py + dy, frame_x, sy, fw, fh, 1);
        } else {
            draw_image_clipped(g, &self.images.actor, px + dx, py + dy, frame_x, sy, fw, fh);
        }
        if self.walk_phase == 2 && self.facing == 1 {
            self.paint_afterimages(g, cam_x, cam_y, frame_x);
        }
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        // pathArrowsVisible 路径箭头（a.java:5929-5956）：自玩家格沿 walkPathBuffer 逐格外扩，
        // buffer 顶 = 玩家侧第一步；[8][22] 三联贴片 21×22（0 下/1 上裁剪源、2 右镜像、3 左），
        // 0/1 的 bob 加在 y、2/3 加在 x（Java 原样）；setColor(136) 每步重设。
        if self.path_arrows_visible {
            let mut ax = (self.player_cell_x << 5) + cam_x;
            let mut ay = (self.player_cell_y << 5) + cam_y;
            let bob = self.bob_offset;
            let piece = &self.images.ui[22];
            let (buf, count) = self.path_arrow_steps();
            for i in (0..count).rev() {
                g.set_color(136);
                match buf[i] {
                    0 => {
                        ay += 32;
                        draw_image_clipped(g, piece, ax + 5, ay + 5 + bob, 21, 0, 21, 22);
                    }
                    1 => {
                        ay -= 32;
                        draw_image_clipped(g, piece, ax + 5, ay + 5 + bob, 0, 0, 21, 22);
                    }
                    2 => {
                        ax += 32;
                        crate::menu_family::draw_edge_patch(g, piece, ax + 5 + bob, ay + 5, 42, 0, 21, 22, 1);
                    }
                    3 => {
                        ax -= 32;
                        draw_image_clipped(g, piece, ax + 5 + bob, ay + 5, 42, 0, 21, 22);
                    }
                    _ => {}
                }
            }
        }
        // walkPhase==2 四角装饰（a.java:5959-5959，画后直接 return 语义）
        if self.walk_phase == 2 {
            let bob = self.bob_offset;
            let vb = self.view_bottom;
            // 【BUG-031①】a.java:5964-5967 字面量 148 / 109 / 215 = 屏心 y − 12 / 屏心 x − 11 / 屏宽 − 25
            // （classic 回代原值）；下臂 viewBottom 原本即变量——同块另三臂随之换轨。
            let (cx, cy, w) = (self.layout.center_x, self.layout.center_y, self.layout.screen_w);
            draw_image_clipped(g, &self.images.corner_sprites, 5 - bob, cy - 12, 44, 0, 22, 24);
            draw_image_clipped(g, &self.images.corner_sprites, cx - 11, 25 - bob, 0, 0, 22, 24);
            draw_image_clipped(g, &self.images.corner_sprites, cx - 11, vb - 29 + bob, 22, 0, 22, 24);
            crate::menu_family::draw_edge_patch(g, &self.images.corner_sprites, w - 25 + bob, cy - 12, 44, 0, 22, 24, 1);
        }
        // walkPhase==3 道具面板：道具批次（spec 待办）
    }
    /// walkPhase==2 的四向残影（a.java:5860-5904）。
    pub(super) fn paint_afterimages(&self, g: &mut SoftGraphics<'_>, cam_x: i32, cam_y: i32, frame_x: i32) {
        let (fw, fh) = (PLAYER_FRAME_W, PLAYER_FRAME_H);
        for i in (0..4).rev() {
            let ax = cam_x + self.afterimage_x[i];
            let ay = cam_y + self.afterimage_y[i];
            match self.facing {
                0 => draw_image_clipped(g, &self.images.actor, ax - 4, ay - 14, frame_x, 0, fw, fh),
                1 => draw_image_clipped(g, &self.images.actor, ax - 8, ay - 14, frame_x, fh, fw, fh),
                2 => draw_image_clipped(g, &self.images.actor, ax - 6, ay - 14, frame_x, fh * 2, fw, fh),
                _ => crate::menu_family::draw_edge_patch(g, &self.images.actor, ax, ay - 14, frame_x, fh * 2, fw, fh, 1),
            }
        }
    }
    /// paintHudFrame（a.java:6134-6157）：HUD 大框（ui[8][0] 边框件 64×16）。
    pub(super) fn paint_hud_frame(&mut self, g: &mut SoftGraphics<'_>, x: i32, y: i32, w: i32, h: i32) {
        let img = &self.images.ui[0];
        let (corner, mid, side) = (BORDER_CORNER_W, BORDER_MID_W, BORDER_SIDE_W);
        let strip = BORDER_STRIP_H;
        g.set_clip(x, y, corner, strip);
        g.draw_image(img, x, y, 0);
        let mut cx = x + corner;
        while cx < x + w - corner {
            g.set_clip(cx, y, mid, strip);
            g.draw_image(img, cx - corner, y, 0);
            cx += mid;
        }
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        crate::menu_family::draw_edge_patch(g, img, x + w - corner, y, 0, 0, corner, strip, 1);
        g.set_color(HUD_FILL);
        g.fill_rect(x + side, y + strip, w - side * 2, h - strip);
        let right = x + w - side;
        let mut cy = y + strip;
        while cy < y + h {
            g.set_clip(x, cy, side, strip);
            g.draw_image(img, x - BORDER_SIDE_SRC_L, cy, 0);
            g.set_clip(right, cy, side, strip);
            g.draw_image(img, right - BORDER_SIDE_SRC_R, cy, 0);
            cy += strip;
        }
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
    }
    /// paintHudPanel（a.java:6052-6106）：HUD（头像/HP/攻/防/武器/甲）。
    pub fn paint_hud(&mut self, g: &mut SoftGraphics<'_>) {
        let font = crate::paint::paint_font();
        // STAGE-B-004b ②：底部 HUD 信息栏锚位公式化（menu_family::hud_panel_anchor
        // 贴底随宽，v5 底部带背景同款拉伸判据；classic 代入化回 bottom_panel_dx()+240
        // 原值）。立绘/属性条内容锚仍走 Layout（hud_face_ax/hud_icon_x 居中不放大）。
        let anchor = crate::menu_family::hud_panel_anchor(self.layout.screen_w, self.view_bottom, self.layout.hud_frame_h);
        let base_y = anchor.y;
        self.paint_hud_frame(g, anchor.x, base_y, anchor.w, anchor.h);
        let face = &self.images.actor_face;
        g.draw_image(face, self.layout.hud_face_ax - (face.width >> 1), base_y + HUD_FRAME_H - face.height, 0);
        let icon_x = self.layout.hud_icon_x;
        let bar_x = icon_x + HUD_BAR_DX;
        let mut y = base_y + HUD_STAT_Y0;
        // HP 行：图标 HUD_ICON_CLIP 见方 + 条框 + 数值
        g.set_clip(icon_x, y, HUD_ICON_CLIP, HUD_ICON_CLIP);
        g.draw_image(&self.images.ui[7], icon_x, y + HUD_ICON_LIFT, 0);
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.hp, bar_x + HUD_NUM_DX, y + 2);
        // 攻行：图标条带片 (10,0,10,13)
        y += HUD_ROW_STEP;
        draw_image_clipped(g, &self.images.ui[7], icon_x, y, 10, 0, HUD_ICON_CLIP, HUD_ICON_STRIP_H);
        g.set_color(HUD_ROW_ACCENT);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.atk, bar_x + HUD_NUM_DX, y + 2);
        // 防行：图标条带片 (20,0,10,13)
        y += HUD_ROW_STEP;
        draw_image_clipped(g, &self.images.ui[7], icon_x, y, 20, 0, HUD_ICON_CLIP, HUD_ICON_STRIP_H);
        g.set_color(HUD_ROW_ACCENT);
        paint_mini_frame(g, bar_x, y + 1, HUD_BAR_W, HUD_BAR_H);
        paint_number(g, &self.images.ui[2], self.player.def, bar_x + HUD_NUM_DX, y + 2);
        // 装备槽（a.java:6079-6110）：equipped==0 → 白字“无”
        // （equipmentMaterialNames[0]）；否则画 entityTypeImage[equipped]
        // 居中（槽 32×32，(32-图宽>>1)/(32-图高>>1)）。
        // 槽 x：var11 = HUD_ICON_X+16 后 +60 → 159；甲槽 +34 → 193（a.java:6080/6087）
        let slot_y = base_y + HUD_SLOT_DY;
        let mut slot_x = bar_x + 60;
        for equipped in [self.equipped_weapon, self.equipped_armor] {
            paint_mini_frame(g, slot_x, slot_y, HUD_SLOT_SIZE, HUD_SLOT_SIZE);
            if equipped == 0 {
                let none: Vec<u16> = "无".encode_utf16().collect();
                let cw = font.char_width(none[0]);
                g.set_color(WHITE);
                g.draw_string(
                    &none,
                    slot_x + ((HUD_SLOT_SIZE - cw) >> 1),
                    slot_y + ((HUD_SLOT_SIZE - font.height()) >> 1),
                    0,
                );
            } else if let Some(img) = self
                .images
                .entity
                .get(equipped as usize)
                .and_then(|o| o.as_ref())
            {
                g.draw_image(
                    img,
                    slot_x + ((HUD_SLOT_SIZE - img.width) >> 1),
                    slot_y + ((HUD_SLOT_SIZE - img.height) >> 1),
                    0,
                );
            }
            slot_x += HUD_SLOT_STEP;
        }
    }
    /// paint case 11（a.java:2798-2896）：世界层 + 对话框/立绘/气泡/翻页箭头。
    /// overlayText==null 或 overlayActive 或 phase==4 时走 paintHudPanel 分支。
    /// paint case 20（a.java:3039-3067）：结局卷轴。状态推进在 paint 内
    /// （Java 原样）：滚动 f_int_22 奇数拍 ++（>w-240 → f_bool_04），
    /// 打字机 (fc&3)>>1 门控推进并重排 209 宽限版；信箱 3156024 上下
    /// 各 img_y0 高；卷轴画 (−f_int_22, img_y0)；底栏 paintWrappedText
    /// (texts[260], 15, 320−fontHeight−10, 210, fontHeight+10)。
    pub(super) fn paint_ending(&mut self, g: &mut SoftGraphics<'_>) {
        let img = self.images.ending.clone();
        let text = self.dialog_data.texts[260].clone();
        let font = crate::paint::paint_font();
        let img_y0 = (self.layout.screen_h - img.height) >> 1;
        self.ending_anim += 2;
        if !self.ending_done && (self.frame_counter & 1) != 0 {
            self.ending_scroll += 1;
            if self.ending_scroll > img.width - SCREEN_W {
                self.ending_done = true;
            }
        }
        let text_len = text.chars().count() as i32;
        if (self.frame_counter & 3) >> 1 != 0 && self.dialog_char < text_len {
            self.dialog_char += 1;
            let line_h = font.height() + 4;
            self.layout_wrapped_text_limited(&text, 209, (line_h << 1) + 12, self.dialog_char);
            self.overlay_page_top = (self.overlay_total_lines - 1).max(0);
            self.overlay_page_bottom = self.overlay_total_lines;
        }
        g.set_color(3156024);
        // 【A 黄2·2026-10-23】顶带几何源改 layout（旧 classic 常量 SCREEN_W ⇒
        // Modern 576 下右 336px 不铺+横滚图右缘露底——与下带相邻行对照实锤；
        // 根因=classic 常量泄漏非缺清屏，三问选根修不动帧面清屏语义）
        g.fill_rect(0, 0, self.layout.screen_w, img_y0);
        g.fill_rect(0, self.layout.screen_h - img_y0, self.layout.screen_w, img_y0);
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        g.draw_image(&img, -self.ending_scroll, img_y0, 0);
        g.set_color(0xFFFF_FFFF);
        self.paint_wrapped_text(g, 15, self.layout.screen_h - font.height() - 10, 210, font.height() + 10, false);
    }
    /// paint case 12（a.java:2866-2880）：视差 + 页码描边指示 + 双箭头
    /// （左 transform 镜像 60−bob / 右 160+bob）+ 软键条。
    pub(super) fn paint_mode12(&mut self, g: &mut SoftGraphics<'_>) {
        self.paint_parallax(g, false);
        let text_bottom_y = self.layout.screen_h - crate::paint::paint_font().height - 4;
        let indicator = self.page_indicator.clone();
        // 【BUG-031④】a.java:2906-2914 页码 x 120 / 箭头 60−bob、160+bob = 屏心派生 ⇒ Layout 字段
        //（center_x / arrow_pair_left_x / arrow_pair_right_x，classic 回代原值）
        let cx = self.layout.center_x;
        self.draw_shadow_text(g, &indicator, cx, text_bottom_y + 2, 4202520, 0xFFFF_FFFFu32 as i32);
        // 【BUG-032】a.java:2908 `var46 += fontHeight + 4 - 13 >> 1`：复合赋值右侧整体 = (fh+4−13)>>1
        // （javap 5242-5255：iload_2 … iconst_1 ishr iadd）——旧译 `(var46 + …) >> 1` 把行底也减半（classic
        // 画到 y 153，Java 304）。
        let arrow_y = text_bottom_y + ((crate::paint::paint_font().height + 4 - 13) >> 1);
        let arrow = self.images.ui[14].clone();
        let bob = self.bob_offset;
        if self.record_page > 0 {
            // a.java:2910 drawImageWithNokiaTransform(…, 1) ⇒ 变换码 = NOKIA_TRANSFORM_TABLE[1] = 8192（水平翻转；
            // Java trace `drawImageT(20x13,60,304,0,8192)`）。【BUG-032】旧直传 1（shim 表 1 = 垂直翻转）。
            g.draw_image_transformed(&arrow, self.layout.arrow_pair_left_x - bob, arrow_y, 0, crate::paint::NOKIA_TRANSFORM_TABLE[1]);
        }
        if self.record_page < self.persist.journal.len() as i32 - 1 {
            g.draw_image(&arrow, self.layout.arrow_pair_right_x + bob, arrow_y, 0);
        }
        m_034_softkeys(g, &self.images.ui[10], &self.images.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
        // 公共尾 overlay 框（paintOverlayBox）：日志文本 kind5 框在指示/箭头之上
        self.paint_overlay_box(g);
    }
    /// paintStatusBar（a.java:6018-6035）：顶部状态栏（层数/三钥/金币）+ paintKeySlot。
    /// 锚位随 `self.layout`（STAGE-B-002 ②：楼层贴左/钥匙金币贴右/中间填充界）。
    pub fn paint_status_bar(&mut self, g: &mut SoftGraphics<'_>) {
        let base_y = 0;
        let font = crate::paint::paint_font();
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        g.draw_image(&self.images.ui[3], 0, base_y, 0);
        if self.persist.floor > 50 {
            // "引子" outline 文本（drawOutlinedText a.java:4487-4495）：anchor 17 =
            // TOP(16)|HCENTER(1) 顶部居中——8 向偏移点各画一次全串（色[0]）+
            // 中心 fill（色[1]）。shim 常量：VCENTER=2 非 16！
            let text: Vec<u16> = "引子".encode_utf16().collect();
            let x0 = 32 - 1;
            let y0 = base_y + 9 + ((19 - font.height()) >> 1) - 1;
            let outline = [4202520u32, WHITE];
            let deltas = [(1i32, 0i32), (1, 0), (0, 1), (0, 1), (-1, 0), (-1, 0), (0, -1), (0, -1)];
            let mut pts = Vec::new();
            let (mut x, mut y) = (x0, y0);
            for (dx, dy) in deltas {
                x += dx;
                y += dy;
                pts.push((x, y));
            }
            pts.push((x0 + 1, y0 + 1)); // 中心 fill（script_cursor0++→32、table_count 循环回 y0+1）
            for (i, (px, py)) in pts.iter().enumerate() {
                g.set_color(if i == 8 { outline[1] } else { outline[0] });
                g.draw_string(&text, *px, *py, 17);
            }
        } else {
            paint_number(g, &self.images.ui[18], self.persist.floor, STATUS_FLOOR_NUM_X, base_y + 4 + 2);
            g.draw_image(&self.images.ui[8], STATUS_TIER_ICON_X, base_y + 4 + 10, 0);
        }
        g.set_color(STATUS_FILL);
        let fill_x = self.layout.status_fill_x;
        g.fill_rect(fill_x, base_y, self.layout.screen_w - fill_x, STATUS_BAR_H);
        self.paint_key_slot(g, 0, self.player.yellow_keys, self.layout.status_key_x[0], base_y);
        self.paint_key_slot(g, 1, self.player.blue_keys, self.layout.status_key_x[1], base_y);
        self.paint_key_slot(g, 2, self.player.red_keys, self.layout.status_key_x[2], base_y);
        let y = base_y + 3;
        g.draw_image(&self.images.ui[1], self.layout.status_gold_icon_x, y, 0);
        paint_number(g, &self.images.ui[2], self.player.gold, self.layout.status_gold_num_x, y + 2);
    }
    /// paintKeySlot（a.java:6043-6044）：钥匙格（ui[8][6] KEY_SLOT_W 三联图 + 数量）。
    pub(super) fn paint_key_slot(&self, g: &mut SoftGraphics<'_>, kind: i32, count: i32, x: i32, y: i32) {
        g.set_clip(x, y, KEY_SLOT_W, KEY_SLOT_H);
        g.draw_image(&self.images.ui[6], x - kind * KEY_SLOT_W, y, 0);
        g.set_clip(0, 0, self.layout.screen_w, self.layout.screen_h);
        paint_number(g, &self.images.ui[2], count, x + KEY_NUM_DX, y + KEY_NUM_DY);
    }
}
