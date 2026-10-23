//! paint 调用树（`a.java:2333` 起 876 行的 `paint(Graphics)` 的 Rust 端口）。
//!
//! # 结构
//!
//! `paint` 是按 `gameMode` 分发的巨型状态投影：每个分支把当前游戏状态绘制到
//! 画布。Rust 端保持同一分发结构与绘制顺序，绘制原语走
//! [`crate::render::SoftGraphics`]（shim 像素模型的逐字节复刻）。
//!
//! # 对拍裁判（三层）
//!
//! 1. **OPS 流**：每 tick 的绘制调用序列（trace 里 `  setColor(...)` 缩进行）
//! 2. **FRAME sha**：每帧全屏像素哈希（trace `FRAME sha=`，截断 16 字节 / 32 hex）
//! 3. 回放测试逐 tick 对拍（`game-oracle/tests/boot_replay.rs`）
//!
//! # 进度
//!
//! - [x] 入口：`setFont(f_Font_00)`（SIZE_SMALL，a.java:2336、24）
//! - [x] mode 0 全分支：白底+启动图居中（a.java:2338-2344）/ paintLogoAnimation logo 层
//! - [x] mode 0 tick 逻辑：加载轮播 + logo 时间线（a.java:3349-3374）
//! - [ ] mode 21（logo 后过渡）、mode 4/19 主菜单、mode 1/2 加载、其余分支

use crate::enums::GameMode;
use crate::render::{ArgbImage, SoftFont, SoftGraphics, font_size, font_style};

/// paint 入口持有的常量字体（a.java:24：`f_Font_00 = Font.getFont(0, 0, 8)`）。
pub fn paint_font() -> SoftFont {
    SoftFont::get_font(0, font_style::PLAIN, font_size::SMALL)
}

/// `nokiaTransformTable`（a.java:44）：`drawImageWithNokiaTransform` 的 Nokia 变换码表。
/// 全部值 >7 ⇒ shim `drawImageTransformed` 的 switch 落 default = 原样绘制
/// （op 文本仍记录原始码，`drawImageT`）。
pub const NOKIA_TRANSFORM_TABLE: [i32; 8] = [0, 8192, 16384, 24576, 8462, 270, 90, 8282];

/// `drawImageWithNokiaTransform`（a.java:4563-4559）：DirectGraphics 变换绘制（shim 下视觉 = 原样）。
fn draw_image_with_nokia_transform(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, kind: i32) {
    let t = NOKIA_TRANSFORM_TABLE[kind as usize];
    g.draw_image_transformed(img, x, y, 0, t);
}

/// `drawImageClipped`（a.java:4551-4549）：源矩形 clip 绘制——clip(x,y,w,h) 后把图
/// 画在 `(x-sx, y-sy)`，只露出 `(sx,sy)` 起的子区。
pub fn draw_image_clipped(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, sx: i32, sy: i32, w: i32, h: i32) {
    g.set_clip(x, y, w, h);
    g.draw_image(img, x - sx, y - sy, 0);
    g.reset_clip();
}

/// `paintSoftkeyBar`（a.java:6006-6010）：软键栏。`left`/`right` = softkeyLeftKind/softkeyRightKind
/// （0 = 不绘制；kind 1..n 选 ui[11] 精灵条的第 n 个 12×10 图标）。
pub fn m_034_softkeys(
    g: &mut SoftGraphics<'_>,
    ui10: &ArgbImage,
    ui11: &ArgbImage,
    left: i8,
    right: i8,
    layout: &crate::layout::Layout,
) {
    // 【STAGE-B-004a ①b】软键图标悬空锚修复：锚随 layout（modern 贴屏底/右缘）——
    // classic 代入=原值零偏离；旧版用 classic 自由函数致 modern 下图标悬空。
    if left != 0 {
        g.draw_image(ui10, 0, layout.softkey_base_y, 0);
        draw_image_clipped(
            g,
            ui11,
            crate::layout::SOFTKEY_ICON_LEFT_X,
            layout.softkey_icon_y,
            (left as i32 - 1) * crate::layout::SOFTKEY_ICON_W,
            0,
            crate::layout::SOFTKEY_ICON_W,
            crate::layout::SOFTKEY_ICON_H,
        );
    }
    if right != 0 {
        draw_image_with_nokia_transform(g, ui10, layout.softkey_right_x, layout.softkey_base_y, 1);
        draw_image_clipped(
            g,
            ui11,
            layout.softkey_icon_right_x,
            layout.softkey_icon_y,
            (right as i32 - 1) * crate::layout::SOFTKEY_ICON_W,
            0,
            crate::layout::SOFTKEY_ICON_W,
            crate::layout::SOFTKEY_ICON_H,
        );
    }
}

/// mode 21（声音询问）绘制：a.java:3066-3067 + paintSoftkeyBar 软键栏。
///
/// 软键 (1,2) 由 dispatchModeEntry case 21 设置（a.java:4482-4477）。
pub fn paint_sound_prompt(
    g: &mut SoftGraphics<'_>,
    ui10: &ArgbImage,
    ui11: &ArgbImage,
    softkeys: (i8, i8),
    layout: &crate::layout::Layout,
) {
    // 【STAGE-B-004a】锚位公式化：黑底满画布 + 提示居中 + 软键随 layout——
    // classic 代入=原值（SCREEN_*/prompt_cx/cy 同式）零偏离。
    g.set_color(crate::layout::BLACK);
    g.fill_rect(0, 0, layout.screen_w, layout.screen_h);
    g.set_color(crate::layout::WHITE);
    g.draw_string(
        crate::layout::SOUND_PROMPT_TEXT,
        layout.prompt_cx,
        layout.prompt_cy,
        anchor_top_hcenter(),
    );
    m_034_softkeys(g, ui10, ui11, softkeys.0, softkeys.1, layout);
}

/// mode 15 帮助 overlay 文本（a.java:4405-4405，逐字）。
pub const HELP_TEXT: &str = "游戏描述：\n\\c99FFCC有人的地方就有江湖，有神仙的地方何尝不是江湖；百战百胜的本事，换不回女人的真心，兄弟的真义；齐天大圣又如何，没有真情实义，做神仙跟做咸鱼有什么区别？\n\n操作方式：按左软键调出物品栏，左右选择一件道具，按确定键使用。\n游戏操作：\n上方向键/2：向上行走\n下方向键/8：向下行走\n左方向键/4：向左行走\n右方向键/6：向右行走\n确定键/5:探索地图\n左软键：打开道具列表\n右软键：打开游戏中菜单\n\n代理发行：广州易诚计算机科技有限公司\n发行商网站：www.9266.net\n客服电话：4006509913\n客服信箱：kefu@9266.net";

/// mode 17 关于 overlay 文本（a.java:4424-4431，逐字）。
pub const ABOUT_TEXT: &str = "版权所有：\n上海雪鲤鱼计算机科技有限公司\nwww.kgame.com.cn\n手机上网：\nwap.kgame.com.cn\n制作人：梁一\n编剧：王之浣\n策划：孙悦\n程序：杨政\n美术：梁一、王之浣、黄吉力\n测试：金鑫，王毅，计成毅\n版本：V1.0\n客服电话：4006305518";

/// `Graphics.TOP | Graphics.HCENTER` = 17（drawString 第三参的常用组合）。
pub fn anchor_top_hcenter() -> i32 {
    crate::render::anchor::TOP | crate::render::anchor::HCENTER
}

/// mode 0（启动屏）绘制：a.java:2338-2347。
///
/// 相位 <2：整屏白 + 启动图几何居中（Java `-` 优先于 `>>` ⇒ `(240-w)>>1`）；
/// 相位 >=2：`paintLogoAnimation` logo 动画层（[`crate::logo_anim::LogoAnim::paint`]）。
/// 【问题1】标题铺满矩形（cover 等比撑满裁切）：返回 (x,y,w,h)——w/h **≥ 画布边长**
/// （宿主钉：576²/832² 画布断言）。Modern additivce；classic 不经此面。
pub fn title_fill_rect(canvas_w: i32, canvas_h: i32, img_w: i32, img_h: i32) -> (i32, i32, i32, i32) {
    let (iw, ih) = (img_w.max(1), img_h.max(1));
    let (cw, ch) = (canvas_w.max(1), canvas_h.max(1));
    let k = (cw as f64 / iw as f64).max(ch as f64 / ih as f64);
    let w = (iw as f64 * k).ceil() as i32;
    let h = (ih as f64 * k).ceil() as i32;
    ((cw - w) / 2, (ch - h) / 2, w, h)
}

pub fn paint_boot(g: &mut SoftGraphics<'_>, boot: &BootPaintState, layout: &crate::layout::Layout) {
    if boot.phase < 2 {
        // setColor(-1)：Java int -1 位模式 & 0xFFFFFF = 0xFFFFFF
        // 【追加件 2026-10-19】boot logo（mode 0）分级适配同款：白底**满画布**
        //（黑边根因=固定 SCREEN_* 常量）+ logo 原尺寸居中；延展带纯色=白（同底色）。
        // classic 代入同值零偏离。
        g.set_color(crate::layout::WHITE);
        g.fill_rect(0, 0, layout.screen_w, layout.screen_h);
        if let Some(img) = &boot.image {
            // 【2026-10-23 更正】铺满裁定作用于**标题画面美术**（title.rs map_bg），
            // 非 boot logo——本处保留 2026-10-19 追加件裁定"白底满画布+logo 原尺寸
            // 居中"（modern_boot_white_background_fills_canvas 钉）。
            let x = (layout.screen_w - img.width) >> 1;
            let y = (layout.screen_h - img.height) >> 1;
            g.draw_image(img, x, y, 0);
        }
    } else {
        match (&boot.logo, &boot.sflogo7) {
            (Some(logo), Some(sflogo7)) => logo.paint(g, sflogo7),
            // logic 已切 bootLoadPhase>=2 但首个 runLogoAnimation 未跑：logoItemCount=0 ⇒ paintLogoAnimation 仅白底
            // （A-boot-menu T32 硬锚：a682fa57=纯白）
            _ => {
                g.set_color(crate::layout::WHITE);
                g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
            }
        }
    }
}

/// mode 0 启动状态机：tick 逻辑（a.java:3349-3374）+ paint 输入投影。
///
/// # 时序合同（差分实测，A-boot-menu 四硬锚交叉验证）
///
/// 每 tick **逻辑先 paint 后**：TICK n 的帧 = 第 n 次逻辑推进后的投影。
/// 硬锚：T17=l1（logic#17 加载即生效）、T31=l1/T32=白（bootLoadPhase 切换
/// 发生在 31/32 帧之间）、T33=首个 logo 帧、T40=时间线 8 状态。
/// 计数细节：logic#1 走 `bootPhaseCounter==0` 加载分支（不加计数），`++` 自
/// logic#2 起拍——l0 显示 TICK 1-16（16 帧）、l1 显示 TICK 17-31（15 帧）。
pub struct BootMachine {
    /// gameMode（0=启动屏, 21=声音询问；1=title 菜单未端口）。
    pub mode: GameMode,
    /// keyValue（keyPressed 设置、tick 末清零，a.java:4664/4251）。
    pub key_value: i32,
    /// bootLoadPhase：加载相位（0=l0, 1=l1, 2+=logo 时间线）。
    pub phase: i32,
    /// bootPhaseCounter：相位内计数（轮播每张 16 tick：1..15 显示，16 切换；时间线 1..35）。
    pub counter: i32,
    /// f_Image_00：当前启动图（l0/l1）。
    pub image: Option<ArgbImage>,
    /// logo 动画层（bootLoadPhase>=2 起参与绘制）。
    pub logo: Option<crate::logo_anim::LogoAnim>,
    /// l0/l1 预解码件（a.java:3354 的 `/l{n}.png`）。
    boot_images: Vec<ArgbImage>,
    /// sflogo 容器 8 张（loadPackedImages(0) 首次进 logo 相位时加载）。
    sflogo: Vec<ArgbImage>,
    /// ui 容器 25 张（loadPackedImages(8)，dispatchModeEntry case 21 a.java:4481 加载）。
    ui: Vec<ArgbImage>,
    /// softkeyLeftKind/softkeyRightKind 软键栏状态（paintSoftkeyBar 绘制输入）。
    pub softkeys: (i8, i8),
    /// mode 1 标题菜单（gameMode=1 起接管 tick/paint）。
    pub title: Option<crate::title::TitleMachine>,
    /// SKY_WAR 全局存档（saveGlobalProgress a.java:8917-8942；两机同构拷贝）。
    pub global: crate::save::GlobalSave,
    /// 全局态脏位（壳轮询持久化；取走即清）。
    pub global_dirty: bool,
    /// menu 容器 2 张（loadPackedImages(10)：资源名表第 10 项 "menu"，mode 1 背景/图标带）。
    menu: Vec<ArgbImage>,
    /// gameRandom（构造时 setSeed(VTime=0)；JavaRandom LCG）。
    pub rng: game_platform::JavaRandom,
    /// 布局面（STAGE-B-004a）：modern 画布尺寸/锚位源——classic=原值零偏离。
    /// 壳经 apply_layout 换轨（与 GameScene::apply_layout 同口径）。
    pub layout: crate::layout::Layout,
    /// mapbg 容器 1 张（loadPackedImages(1)：资源名表第 1 项，parallax 平铺 77x320）。
    mapbg: ArgbImage,
    /// 视差背景滚动（paint 内推进）。
    pub backdrop: crate::menu_family::ParallaxBackdrop,
    /// overlay 文本引擎（overlayActive 层）。
    pub overlay: crate::menu_family::OverlayEngine,
    /// overlayActive：overlay 激活。
    pub overlay_active: bool,
    /// menuReturnMode：overlay/菜单的「返回模式」。
    pub return_mode: GameMode,
    /// animBobOffset/animBobRising（a.java:144-145）：mode 1 每 tick 推进
    ///（run case 1 头 a.java:3387）；进游戏后由 paintEntityLayer 接力
    ///（a.java:6499）。跨 mode 单一状态，统一流交接给 GameScene。
    pub bob_offset: i32,
    pub bob_rising: bool,
    /// mode 8 槽位表。
    pub slots: Option<crate::menu_family::SlotSelect>,
    /// mode 16 选项表。
    pub options: Option<crate::menu_family::OptionList>,
    /// iconStripFrame 共享（菜单族与 title 同一字段 a.java:464）。
    pub strip_frame: usize,
    /// 虚拟 RMS：preset 解析产物（store→blob；槽载入/进度读档用）
    pub preset_stores: Vec<(String, Vec<u8>)>,
    /// sptarm 容器（mode8 详情面板装备位图）
    pub sptarm: Vec<ArgbImage>,
    /// 槽载入完成信号（run case8 -5 → mode2 链走完 → mode3）：
    /// Some(slot_cursor) = 应构建 GameScene 并接管
    pub slot_loaded: Option<usize>,
    /// 音频语义（soundEnabled/volumeLevel/track/f_bool_28/anyKeyPlayed/
    /// bgm，crate::audio；docs/spec/audio.md）。soundEnabled 的唯一真相源。
    pub bgm: crate::audio::BgmState,
    /// intro 容器 2 张（loadPackedImages(11)，引子页 0/1 图）。
    intro_imgs: Vec<ArgbImage>,
    /// end 容器 1 张（loadPackedImages(14)，引子页 2 图）。
    end_imgs: Vec<ArgbImage>,
    /// load 容器 2 张（loadPackedImages(15)，进度条图）。
    load_imgs: Vec<ArgbImage>,
    /// mode 14 引子状态。
    pub intro: Option<crate::intro::IntroSequence>,
    /// mode 2 加载链。
    pub loading: Option<crate::intro::LoadProgress>,
    /// f_int_02 帧间隔 ms（构造 75，a.java:27；mode 0 每 tick 设 100，
    /// a.java:3350；切 mode 21 时回 75，a.java:3377）。
    pub frame_interval_ms: i64,
    /// paint 计数（对拍 TICK n 用）。
    pub paints: u32,
    /// mode 0 结束标志（logo 时间线 35 耗尽，a.java:3370-3373 切 gameMode=21）。
    pub finished: bool,
    /// 新游戏链完成（mode 2 装载进度 ≥100 且非槽载入，dispatchModeEntry case 3）：
    /// 宿主据此构造冷启动场景（step8 统计 floor51/hp300/atk10/def10，
    /// a.java:3496-3506）并接管 mode 3。与 `finished`（mode 0/退出语义）分离。
    pub new_game_ready: bool,
}

impl BootMachine {
    /// 布局换轨（STAGE-B-004a）：modern 画布/锚位生效点——与 GameScene 同口径，
    /// classic 传回原值零偏离。
    pub fn apply_layout(&mut self, layout: crate::layout::Layout) {
        self.layout = layout;
    }

    /// WIN-OP③（C-2 补面 2026-10-18）：标题侧设置面板（GameMode::About）指针命中——
    /// OptionList::row_band 单源（与 GameScene 侧共用几何）；confirm=-5（翻转键同款）。
    pub fn pointer_hit(&self, _x: i32, y: i32) -> crate::scene::PointerHit {
        if self.mode == GameMode::EquipDetail {
            // 标题侧读档槽列表（paint case 8）：SlotSelect 行带单源
            if let Some(slots) = self.slots.as_ref() {
                for vis in 0..(slots.scroll_bottom - slots.scroll_top).max(0) {
                    let (ry, rh) = slots.row_band(vis);
                    if y >= ry && y < ry + rh {
                        let index = slots.scroll_top + vis;
                        return crate::scene::PointerHit::Row { index, confirm_key: -5 };
                    }
                }
            }
            return crate::scene::PointerHit::None;
        }
        if self.mode == GameMode::About {
            if let Some(list) = self.options.as_ref() {
                for i in 0..list.len() {
                    let (ry, rh) = list.row_band(i);
                    if y >= ry && y < ry + rh {
                        return crate::scene::PointerHit::Row { index: i as i32, confirm_key: -5 };
                    }
                }
            }
        }
        crate::scene::PointerHit::None
    }

    /// 指针选行（只动高亮，零其他状态）。
    pub fn select_row(&mut self, index: i32) {
        if self.mode == GameMode::EquipDetail {
            if let Some(slots) = self.slots.as_mut() {
                slots.set_cursor(index);
            }
            return;
        }
        if self.mode == GameMode::About {
            if let Some(list) = self.options.as_mut() {
                if index >= 0 {
                    list.set_highlight(index as usize);
                }
            }
        }
    }


    /// 构造：预解码全部依赖资源（解码耗时与 tick 语义无关）。
    pub fn new(
        l0: ArgbImage,
        l1: ArgbImage,
        sflogo: Vec<ArgbImage>,
        ui: Vec<ArgbImage>,
        menu: Vec<ArgbImage>,
        mapbg: ArgbImage,
        intro_imgs: Vec<ArgbImage>,
        end_imgs: Vec<ArgbImage>,
        load_imgs: Vec<ArgbImage>,
    ) -> BootMachine {
        assert_eq!(sflogo.len(), 8, "sflogo 容器 8 张（a.java:453 resourceImageCounts 计数表）");
        assert_eq!(ui.len(), 25, "ui 容器 25 张（a.java:453 resourceImageCounts 计数表）");
        assert_eq!(menu.len(), 2, "menu 容器 2 张（a.java:453 resourceImageCounts 计数表第 10 项）");
        BootMachine {
            mode: GameMode::Boot,
            key_value: 0,
            phase: 0,
            counter: 0,
            image: None,
            logo: None,
            boot_images: vec![l0, l1],
            sflogo,
            ui,
            softkeys: (0, 0),
            title: None,
            global: crate::save::GlobalSave::default_7(),
            global_dirty: false,
            layout: crate::layout::Layout::classic(),
            menu,
            mapbg,
            backdrop: crate::menu_family::ParallaxBackdrop::new(),
            overlay: crate::menu_family::OverlayEngine::new(),
            overlay_active: false,
            return_mode: GameMode::Boot,
            slots: None,
            options: None,
            strip_frame: 0,
            preset_stores: Vec::new(),
            sptarm: Vec::new(),
            slot_loaded: None,
            bgm: crate::audio::BgmState::new(),
            bob_offset: 0,
            bob_rising: false,
            intro_imgs,
            end_imgs,
            load_imgs,
            intro: None,
            loading: None,
            rng: game_platform::JavaRandom::new_seeded(0),
            frame_interval_ms: 75,
            paints: 0,
            finished: false,
            new_game_ready: false,
        }
    }

    /// 一次 paint（serviceRepaints → a.paint）。按 [`Self::mode`] 分发。
    /// 【task-14】内容居中平移（开场链经典构图整体平移）：
    /// dx=(screen_w−经典宽)/2（classic dx=0 恒等）。
    fn content_dx(&self) -> i32 {
        (self.layout.screen_w - crate::layout::SCREEN_W) / 2
    }

    pub fn paint(&mut self, g: &mut SoftGraphics<'_>) {
        self.paints += 1;
        match self.mode {
            GameMode::SoundAsk => paint_sound_prompt(g, &self.ui[10], &self.ui[11], self.softkeys, &self.layout),
            GameMode::MainMenu => {
                let mut t = self.title.take().unwrap();
                let sf = self.strip_frame;
                t.set_strip_frame(sf);
                t.paint(g, &self.menu[0], &self.menu[1], &self.ui[14], &self.layout);
                self.strip_frame = t.strip_frame_value();
                self.title = Some(t);
            }
            GameMode::EquipDetail => {
                self.backdrop.paint(g, &self.mapbg, false, 0);
                let segs = [3, 5]; // titleBarSegments[1]
                if let Some(slots) = &self.slots {
                    let sptarm = self.sptarm_for_slots();
                    slots.paint(g, &self.ui, &segs, &sptarm, || 0);
                }
                m_034_softkeys(g, &self.ui[10], &self.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
            }
            GameMode::Help | GameMode::IconView => {
                self.backdrop.paint(g, &self.mapbg, false, 0);
                let sy = if self.mode == GameMode::Help { 51 } else { 68 };
                let mut f = self.strip_frame;
                crate::title::paint_icon_strip_at(g, &self.menu[1], 86, 10, sy, 68, &mut f);
                self.strip_frame = f;
                m_034_softkeys(g, &self.ui[10], &self.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
                self.paint_overlay_tail(g);
            }
            GameMode::About => {
                self.backdrop.paint(g, &self.mapbg, false, 0);
                let mut f = self.strip_frame;
                crate::title::paint_icon_strip_at(g, &self.menu[1], 86, 10, 34, 68, &mut f);
                self.strip_frame = f;
                if let Some(opts) = &self.options {
                    opts.paint(g, &self.ui, &crate::paint::paint_font());
                }
                m_034_softkeys(g, &self.ui[10], &self.ui[11], self.softkeys.0, self.softkeys.1, &self.layout);
            }
            GameMode::LogoAnim => {
                let logo = self.logo.as_ref().unwrap();
                let sf7 = self.sflogo[7].clone();
                logo.paint(g, &sf7);
            }
            GameMode::IntroCutscene => {
                // 【task-14】intro 已全体公式化（center_x 锚/字幕框居中/跳过贴右）
                // ⇒ 不加臂平移（双移=左露底带 artifact 实证 2026-10-20）
                let intro = self.intro.as_mut().unwrap();
                let mut particles = self.title.as_mut().map(|t| std::mem::replace(&mut t.particles, crate::title::Particles::new()));
                if let Some(p) = particles.as_mut() {
                    let font = paint_font();
                    intro.paint(g, &mut self.overlay, p, &font, &self.layout);
                }
                if let Some(t) = self.title.as_mut() {
                    if let Some(p) = particles { t.particles = p; }
                }
            }
            GameMode::Loading => {
                self.loading.as_ref().unwrap().paint(g, &self.load_imgs, &self.layout);
            }
            _ => {
                let state = BootPaintState {
                    phase: self.phase,
                    image: self.image.clone(),
                    logo: self.logo.clone(),
                    sflogo7: Some(self.sflogo[7].clone()),
                };
                paint_boot(g, &state, &self.layout);
            }
        }
    }

    /// 一次逻辑 tick。`key` 为本 tick 边界投递的按键码（`keyValue`，无则 0）。
    ///
    /// - mode 0：a.java:3349-3374（加载轮播/logo 时间线/清理切换）
    /// - mode 21：a.java:4229-4242（-6 确认 / -7 否定 → gameMode=1）
    ///
    /// SKY_WAR 快照（a.java:7284 语义：[0] 声音行 = soundEnabled 镜像；[1] 取勾选表
    /// 现值，无表则保持存档值；[2][3] 死槽原样）。
    pub fn global_snapshot(&self) -> crate::save::GlobalSave {
        let mut g = self.global.clone();
        g.option_checked[0] = self.bgm.sound_enabled;
        if let Some(opts) = self.options.as_ref() {
            if let Some(v) = opts.checked.get(1) {
                g.option_checked[1] = *v;
            }
        }
        g
    }

    /// 消费脏位（壳每 tick 轮询；true ⇒ 需落盘）。
    pub fn take_global_dirty(&mut self) -> bool {
        std::mem::take(&mut self.global_dirty)
    }

    /// 装载 SKY_WAR（壳启动时；不置脏）。
    pub fn apply_global(&mut self, g: crate::save::GlobalSave) {
        self.global = g;
        self.global_dirty = false;
    }

    /// 水位抬升（a.java:4297-4299/7163-7164 共用条件：`floor<51 && floor>水位`）。
    pub fn note_max_floor(&mut self, floor: i32) {
        if floor < 51 && floor > self.global.max_floor_watermark {
            self.global.max_floor_watermark = floor;
            self.global_dirty = true;
        }
    }

    pub fn tick(&mut self, key: i32) {
        // keyPressed 的音频半（a.java:4665-4674）：投递发生在上一拍 paint 之后、本拍
        // logic 之前（preTick），故用本拍入口的 gameMode。仅 anyKeyPlayed 时生效。
        if key != 0 {
            self.bgm.on_key_pressed(self.mode);
        }
        match self.mode {
            GameMode::SoundAsk => {
                // a.java:4231-4242：switch (keyValue)
                match key {
                    -7 | -6 => {
                        // -7: soundEnabled=false + volumeLevel=0；-6: true（音量 0→60）
                        //（a.java:4232-4241）。两者都 gameMode=1 + dispatchModeEntry()
                        self.bgm.answer_sound_prompt(key == -6);
                        self.mode = GameMode::MainMenu;
                        self.title = Some(crate::title::TitleMachine::new());
                        // dispatchModeEntry case 1 音频半：f_bool_28=false + playBgm(2)（a.java:4295/4311）
                        self.bgm.enter_title();
                    }
                    _ => {}
                }
                self.key_value = 0;
            }
            GameMode::MainMenu => {
                // a.java:3391-3440。frame_counter：迭代 N 拍用 f=N-1
                //（for 自增在体后）；paints 在 tick 时 = 已 paint 数 = N-1。
                // case 1 头部（a.java:3387）：advanceBobPhase 每 tick。
                crate::entity::advance_bob(&mut self.bob_offset, &mut self.bob_rising);
                let f = self.paints as i32;
                let mut next = None;
                if let Some(t) = self.title.as_mut() {
                    let sf = self.strip_frame;
                    t.set_strip_frame(sf);
                    next = t.tick(key, f, &mut self.rng, &self.layout);
                    self.strip_frame = t.strip_frame_value();
                }
                if let Some(mode) = next {
                    self.mode = mode;
                    // menuReturnMode：kind 1/2/3/4 → 1（a.java:3401/3397/3402）；
                    // kind 0(14)/5(22) 不设
                    if (1..=4).contains(&self.title_kind()) {
                        self.return_mode = GameMode::MainMenu;
                    }
                    self.enter_menu_mode();
                }
                self.key_value = 0;
            }
            GameMode::EquipDetail => {
                // run case 8（a.java:3829-3878）
                match key {
                    -7 => self.mode = self.return_mode,
                    -2 => self.slots.as_mut().unwrap().cursor_down(),
                    -1 => self.slots.as_mut().unwrap().cursor_up(),
                    // -6/-5：槽有效才读档（a.java:3844-3845）→ startLoadTask(3,true)
                    // + stepLoadTask 步序列 → mode 2 步进机（a.java:3845-3857）
                    -6 | -5 => {
                        let cursor = self.slots.as_ref().unwrap().cursor as usize;
                        if self.slots.as_ref().unwrap().valid[cursor] {
                            self.slot_loaded = Some(cursor); // 信号位：完成时置 Some
                            self.loading = Some(crate::intro::LoadProgress::new(
                                GameMode::Map,
                                true,
                                &[6, 8, 5, 13, 9, 16, 10, 2, 3, 11],
                            ));
                            self.mode = GameMode::Loading;
                        }
                    }
                    _ => {}
                }
                self.key_value = 0;
            }
            GameMode::Help | GameMode::IconView => {
                // overlay 层 case 0 的 15/17 分支（a.java:3258-3265）
                match key {
                    -7 => {
                        self.overlay_active = false;
                        self.mode = self.return_mode;
                    }
                    -2 => {
                        let per = self.overlay.lines_per_page();
                        let top = self.overlay.page_top();
                        self.overlay.flip_page(top + per);
                    }
                    -1 => {
                        let per = self.overlay.lines_per_page();
                        let top = self.overlay.page_top();
                        self.overlay.flip_page(top - per);
                    }
                    _ => {}
                }
                self.key_value = 0;
            }
            GameMode::About => {
                // run case 16（a.java:4143-4185）
                match key {
                    -7 | -6 => self.mode = self.return_mode,
                    -5 | -4 | -3 => {
                        if let Some(opts) = self.options.as_mut() {
                            let i = opts.highlight as usize;
                            opts.checked[i] = !opts.checked[i];
                            if opts.labels[i] == 0 {
                                // case 0：声音联动（a.java:4156-4167）：翻转 + 开则
                                // volumeLevel=60 + playBgm(f_bool_28?3:2)，关则 closeAudioPlayer
                                self.bgm.toggle_sound();
                            }
                            // saveGlobalProgress（a.java:4169）：勾选同步进 SKY_WAR 并置脏
                            self.global.option_checked[0] =
                                opts.checked.first().copied().unwrap_or(false);
                            self.global.option_checked[1] =
                                opts.checked.get(1).copied().unwrap_or(false);
                            self.global_dirty = true;
                        }
                    }
                    -2 => {
                        if let Some(opts) = self.options.as_mut() {
                            opts.highlight += 1;
                            if opts.highlight > opts.labels.len() as i32 - 1 {
                                opts.highlight = 0;
                            }
                        }
                    }
                    -1 => {
                        if let Some(opts) = self.options.as_mut() {
                            opts.highlight -= 1;
                            if opts.highlight < 0 {
                                opts.highlight = opts.labels.len() as i32 - 1;
                            }
                        }
                    }
                    _ => {}
                }
                self.key_value = 0;
            }
            GameMode::IntroCutscene => {
                // run case 14（a.java:4137-4136）：runIntroViewer + 粒子雨 4 分频
                let f = self.paints as i32;
                if f & 3 == 0 {
                    // 【STAGE-B-004a ①a】粒子生成域**随画布**（序章白点散射根因）：
                    // 旧 randomBelow(240/SCREEN_H) 固定经典域——modern 画布下白点只
                    // 散布左 240px 带。随 layout 域后 classic=同值零偏离。
                    let x = self.rng.random_below(self.layout.screen_w);
                    let y = self.layout.screen_h - 25 - self.rng.random_below(150);
                    if let Some(t) = self.title.as_mut() {
                        t.particles.spawn(x, y);
                    }
                }
                let done = self
                    .intro
                    .as_mut()
                    .unwrap()
                    .tick(key, &self.intro_imgs, &self.end_imgs, &mut self.overlay, &paint_font());
                if done {
                    // startNewGameLoad（a.java:7174-7181）：新游戏加载链
                    self.loading = Some(crate::intro::LoadProgress::new_game());
                    self.mode = GameMode::Loading;
                }
                self.key_value = 0;
            }
            GameMode::Loading => {
                // run case 2（a.java:3449-3544）：进度追赶模型
                if let Some((target, call_m000)) = self.loading.as_mut().unwrap().tick() {
                    self.loading = None;
                    self.mode = target;
                    if call_m000 {
                        // dispatchModeEntry case 3 音频半（两支同一入口）：f_bool_28=true + playBgm(3)
                        //（a.java:4316/4321；A-enter-game T417 / EV-toggle-exit T397）
                        self.bgm.enter_game();
                        if self.slot_loaded.is_some() {
                            // 槽载入完成：dispatchModeEntry case 3 的软键/清理由宿主在
                            // 场景构建时落实（mode 3 交 GameScene 接管）
                            self.softkeys = (1, 3);
                            self.overlay_active = false;
                        } else {
                            // 新游戏链完成（dispatchModeEntry case 3，a.java:4312-4318 同款软键/
                            // 浮层清位；冷启动场景由宿主按 step8 统计构造——
                            // floor51/hp300/atk10/def10，a.java:3496-3506）
                            self.softkeys = (1, 3);
                            self.overlay_active = false;
                            self.new_game_ready = true;
                        }
                    }
                }
                self.key_value = 0;
            }
            GameMode::LogoAnim => {
                // run case 22（a.java:4249-4252）：++bootPhaseCounter ≤ 70 →
                // runLogoAnimation(1, t)；否则退出进程
                self.counter += 1;
                if self.counter <= 70 {
                    let lg = self
                        .logo
                        .get_or_insert_with(crate::logo_anim::LogoAnim::new);
                    lg.set_canvas(self.layout.screen_w, self.layout.screen_h);
                    lg.tick_help(&self.sflogo, self.counter);
                } else {
                    self.finished = true; // CMidlet.dispatchModeEntry()（进程退出）
                }
                self.key_value = 0;
            }
            _ => self.tick_mode0(),
        }
    }

    /// BGM 宿主契约：应当在放的轨道（`audio::TRACKS` 下标），None = 停止/无声。
    /// 语义 = Java `bgm != null ? track : 无`（docs/spec/audio.md）。
    pub fn bgm_request(&self) -> Option<usize> {
        self.bgm.request()
    }

    /// 当前播放器序号：同轨 serial 变化 = Java 关闭重建（从头播放）。
    pub fn bgm_serial(&self) -> u32 {
        self.bgm.serial()
    }

    /// 当前播放器 VolumeControl 级别（0..=100）。
    pub fn bgm_volume(&self) -> Option<i32> {
        self.bgm.volume()
    }

    /// gameRandom 种子重设（a.java:2325 setSeed(VTime)：VTime 在 a() 构造时
    /// 的取值随 Runner 前置行为（如 preset 触发的 RecordStore 类载入点）变化
    /// ——preset 场景实测 11813（T73 粒子 (208,206) 暴力反推），非 preset 场景 0。
    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng = game_platform::JavaRandom::new_seeded(seed);
    }

    /// mode8 详情面板的 sptarm 容器（preset_stores 不可承载图像，单独注）。
    fn sptarm_for_slots(&self) -> Vec<ArgbImage> {
        self.sptarm.clone()
    }

    /// 当前 title 光标对应的 kind（mode 1 出口分派键）。
    fn title_kind(&self) -> i32 {
        self.title
            .as_ref()
            .map(|t| t.menu_kinds[t.cursor as usize] as i32)
            .unwrap_or(-1)
    }

    /// dispatchModeEntry 的菜单族 case（a.java:4360/4386-4425）：进入模式时初始化。
    fn enter_menu_mode(&mut self) {
        match self.mode {
            GameMode::EquipDetail => {
                // case 8（a.java:4360-4379）：槽表（loadSlotTable 读 MOT_IF）
                // + 布局常量 + 软键 (1,3)。【2026-10-18 BUG-021】入口复用已有
                // 槽表对象、只复位布局/光标（a.java:4354-4366 不动数据数组）；
                // 读失败留半解析旧值 + parse_dirty（壳消费回写 MOT_IF）。
                let mot_if = self
                    .preset_stores
                    .iter()
                    .find(|(k, _)| k.starts_with("MOT_IF@"))
                    .map(|(_, v)| v.clone());
                let mut sl = self.slots.take().unwrap_or_else(crate::menu_family::SlotSelect::new);
                sl.reset_entry_layout();
                sl.load_slot_table(mot_if.as_deref());
                if std::env::var("PROBE_SLOTS").is_ok() {
                    eprintln!(
                        "MOT_IF len={} valid0={}",
                        mot_if.as_ref().map(|d| d.len()).unwrap_or(0),
                        sl.valid[0]
                    );
                }
                self.slots = Some(sl);
                self.softkeys = (1, 3);
            }
            GameMode::Help => {
                // case 15（a.java:4402-4406）：帮助 overlay + 软键 (0,3)
                let text = HELP_TEXT.encode_utf16().collect::<Vec<u16>>();
                let font = paint_font();
                self.overlay.show_kind0(&text, &font, (0, 0));
                self.overlay_active = true;
                self.softkeys = (0, 3);
            }
            GameMode::IconView => {
                // case 17（a.java:4423-4435）：关于 overlay + 软键 (0,3)
                let text = ABOUT_TEXT.encode_utf16().collect::<Vec<u16>>();
                let font = paint_font();
                self.overlay.show_kind0(&text, &font, (0, 0));
                self.overlay_active = true;
                self.softkeys = (0, 3);
            }
            GameMode::About => {
                // case 16（a.java:4414-4416）：选项表 + 软键 (0,3)。
                // optionChecked[1]=true：mode 21 的 RMS 失败 catch（a.java:4467）
                self.options = Some(crate::menu_family::OptionList::new(
                    self.bgm.sound_enabled,
                    true,
                ));
                self.softkeys = (0, 3);
            }
            GameMode::IntroCutscene => {
                // case 14（a.java:4386-4388 区）：intro/end 容器 + 引子状态机
                //（粒子表沿用 title 的 Particles——Java 全局唯一）
                self.intro = Some(crate::intro::IntroSequence::new());
            }
            GameMode::LogoAnim => {
                // case 22（a.java:4487）：closeAudioPlayer（A-menu-sweep T1761
                // close(state=400)）；bootPhaseCounter 已为 0（mode 0→21 清理）
                self.bgm.close();
                self.counter = 0;
            }
            _ => {
                self.finished = true; // 其余未端口模式
            }
        }
    }

    /// paint 公共尾 overlay（a.java:3129-3171，overlayActive && !floorTransitioning）。
    fn paint_overlay_tail(&mut self, g: &mut SoftGraphics<'_>) {
        if !self.overlay_active {
            return;
        }
        let x = (crate::layout::SCREEN_W - self.overlay.box_w) >> 1;
        let y = (crate::layout::SCREEN_H - self.overlay.box_h) >> 1;
        crate::menu_family::paint_box_frame(g, &self.ui[0], x, y, self.overlay.box_w, self.overlay.box_h);
        g.set_color(crate::layout::WHITE);
        let text: Vec<u16> = if self.mode == GameMode::Help {
            HELP_TEXT.encode_utf16().collect()
        } else {
            ABOUT_TEXT.encode_utf16().collect()
        };
        let f = self.paints as i32 - 1;
        let font = paint_font();
        let (tw, th) = (crate::layout::OVERLAY_TEXT_W, crate::layout::OVERLAY_TEXT_MAX_H);
        self.overlay.paint(g, &text, x + 16, y + 16, tw, th, true, f, &font);
    }

    /// gameMode 0 的逻辑 tick（a.java:3349-3374）。
    fn tick_mode0(&mut self) {
        self.frame_interval_ms = 100; // a.java:3350（mode 0 每 tick 首行）
        if self.phase < 2 {
            if self.counter == 0 {
                // a.java:3354：加载 /l{phase}.png；夹具恒存在 ⇒ 恒成功路径
                let idx = self.phase as usize;
                self.image = self.boot_images.get(idx).cloned();
                if self.image.is_some() {
                    self.counter = 1;
                } else {
                    self.phase += 1; // 防御：a.java:3358-3354
                }
            } else {
                self.counter += 1;
                if self.counter > 15 {
                    self.phase += 1; // a.java:3365
                    self.counter = 0;
                }
            }
        } else {
            self.counter += 1; // a.java:3368
            if self.counter <= 35 {
                let lg = self
                    .logo
                    .get_or_insert_with(crate::logo_anim::LogoAnim::new);
                lg.set_canvas(self.layout.screen_w, self.layout.screen_h);
                lg.tick(&self.sflogo, self.counter);
            } else {
                // a.java:3371-3373：清理 + gameMode=21 + dispatchModeEntry()
                self.logo.as_mut().unwrap().clear_all(); // removeLogoItem(-1)
                self.logo.as_mut().unwrap().reset_layout_flag(); // logoLayoutDone=false（a.java:3372）
                self.image = None;
                self.counter = 0;
                self.bgm.close(); // closeAudioPlayer（a.java:3376，此时无播放器）
                self.frame_interval_ms = 75; // f_int_02=75（a.java:3377）
                if self.layout.boot_sound_prompt {
                    // classic 原版时间轴（a.java:3371-3373 → SoundAsk + case21 入场）——
                    // STAGE-B-001 门控化（2026-10-19 lead 裁定）：001 行为只在 modern
                    // 预设生效；classic 预设保留开机声音询问 = 原版时间轴，64 枚帧夹具
                    // Java 参照面零偏离（docs/spec/stage-b-deviations.md 001 节更正）。
                    self.mode = GameMode::SoundAsk;
                    // dispatchModeEntry case 21（a.java:4482-4478）：软键 (1,2)；
                    // soundEnabled=false（a.java:4484）；RMS "SKY_WAR" 读档失败路径
                    //（oracle 内存库恒空 ⇒ catch 删库重置）
                    self.softkeys = (1, 2);
                    self.bgm.enter_sound_prompt();
                    self.finished = false; // mode 21 属对拍范围
                } else {
                    // STAGE-B-001（modern 预设面，docs/spec/stage-b-deviations.md）：
                    // 跳过 mode 21 声音询问——直入标题，默认开声。偏离点 1/2。
                    self.mode = GameMode::MainMenu;
                    // mode1 入场半：建 TitleMachine（照 SoundAsk 回答臂的 gameMode=1 语义
                    // a.java:4232-4241 同款）——跳过询问不跳过初始化（首版漏此致 title=None panic）
                    self.title = Some(crate::title::TitleMachine::new());
                    self.bgm.enter_title(); // m_000 case 1 音频半（a.java:4295/4311）——
                    // 标题 BGM=track2（STAGE-B-001 硬要求 3：防直入标题带出无声屏）
                    self.finished = false;
                    // SKY_WAR 读档失败路径（oracle 内存库恒空 ⇒ catch 删库重置）——
                    // 原在 case21 入场（a.java:4450-4479）；Rust 侧构造链独立（shell boot）
                }
            }
        }
        self.key_value = 0; // a.java:4276（tick 末 keyValue=0）
    }
}

/// mode 0 分支的绘制输入投影（a.java:2339 `bootLoadPhase` 等的只读快照）。
#[derive(Clone)]
pub struct BootPaintState {
    /// 启动加载相位：0/1 → 轮播 `/l{n}.png`；>=2 → logo 动画层。
    pub phase: i32,
    /// 启动图（l0.png 164×117 / l1.png 176×138，加载线程 a.java:3351-3359）。
    pub image: Option<ArgbImage>,
    /// logo 动画层（phase>=2 时 Some）。
    pub logo: Option<crate::logo_anim::LogoAnim>,
    /// sflogo#7（paintLogoAnimation 字形槽源图，a.java:10486）。
    pub sflogo7: Option<ArgbImage>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_platform::hash::sha256_hex;
    use std::path::PathBuf;

    /// 与 TickHooks 同流的截断帧哈希（FRAME sha= 前 32 hex）。
    fn frame_sha(screen: &ArgbImage) -> String {
        sha256_hex(&screen.hash_stream())[..32].to_string()
    }

    fn synth(w: i32, h: i32) -> ArgbImage {
        ArgbImage::create(w, h)
    }

    fn boot_state() -> BootMachine {
        BootMachine::new(
            synth(164, 117),
            synth(176, 138),
            vec![synth(24, 24); 8],
            vec![synth(24, 24); 25],
            vec![synth(24, 24); 2],
            synth(77, 320),
            vec![],
            vec![],
            vec![],
        )
    }

    /// STAGE-B-004a ①a 判据（序章白点散射根因，viewport 复核 V2 点名项）：
    /// 粒子生成域**随画布**——modern 画布（416 宽）下粒子须散布全宽；旧
    /// randomBelow(240) 固定经典域 ⇒ x≥240 带永不落点（白点只散左带=症状）。
    #[test]
    fn modern_particle_spawn_domain_spans_canvas() {
        use game_platform::JavaRandom;
        let layout = crate::layout::Layout::modern(13, 13); // 416×416
        assert!(layout.screen_w > 240, "前置：modern 画布宽 > 240");
        let mut t = crate::title::TitleMachine::new();
        let mut rng = JavaRandom::new_seeded(20261019);
        for f in 0..240 {
            t.tick(0, f, &mut rng, &layout);
        }
        let max_x = t.particles.max_active_x();
        assert!(
            max_x > 240,
            "modern 生成域须越 240（实测最大 x={max_x}——旧固定域恒 ≤239 即红）"
        );
    }

    /// STAGE-B-004a ①b 判据（软键图标悬空锚）：modern 画布下软键图标须贴
    /// layout 锚（屏底 13px/右缘 14px）；旧 classic 自由函数致图标悬空在
    /// 经典坐标（y=307）而画布 416 高=浮在半空。
    #[test]
    fn modern_softkey_icons_anchor_to_layout() {
        use crate::paint::m_034_softkeys;
        let layout = crate::layout::Layout::modern(13, 13);
        let mut ui10 = synth(26, 18);
        ui10.argb.fill(0xFFFF_FFFF); // 实心白（空白图绘制=零像素不可检）
        let mut ui11 = synth(120, 10);
        ui11.argb.fill(0xFFFF_FFFF);
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        {
            let mut g = SoftGraphics::new(&mut img);
            m_034_softkeys(&mut g, &ui10, &ui11, 1, 1, &layout);
        }
        // modern 锚行（layout.softkey_icon_y）须有图标像素；经典锚行（320-13=307）
        // 不得再出现图标（悬空=红）。
        let iy = layout.softkey_icon_y as usize;
        let classic_iy = (crate::layout::SCREEN_H - 13) as usize;
        let w = layout.screen_w as usize;
        let has_icon = |row: usize| (0..w).any(|x| img.argb[row * w + x] != 0);
        assert!(has_icon(iy), "modern 锚行 y={iy} 须有软键图标像素");
        if iy != classic_iy {
            assert!(!has_icon(classic_iy), "经典锚行 y={classic_iy} 不得残留（悬空 artifact）");
        }
    }

    /// 用户验收补修判据（2026-10-19 boot 错位）：logo 组元素锚位公式化——
    /// 绘出内容 bbox 水平中心=画布中心 ±1（旧 classic 坐标直画 modern 画布=偏左上
    /// 即红）；署名（底缘元素）y 须在画布下缘区域（底部居中布局）。
    #[test]
    fn modern_boot_logo_elements_centered() {
        let layout = crate::layout::Layout::modern(13, 13);
        let mut la = crate::logo_anim::LogoAnim::new();
        la.set_canvas(layout.screen_w, layout.screen_h);
        // 真 sflogo 素材（glyph 度量系真图尺寸——合成图构图碎，v6 同法载真）
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let packed = std::fs::read(repo.join("assets/raw/sflogo")).unwrap();
        let sflogo: Vec<ArgbImage> = game_data::PackedPng::parse(&packed, 8)
            .unwrap()
            .images
            .iter()
            .map(|s| ArgbImage::from_decoded_png(game_data::decode_png(s.bytes).unwrap()))
            .collect();
        let solid = sflogo[7].clone();
        // timeline 28=经典实测态（boot-classic.png 帧 60=动画 28 拍，构图就位基准）
        for t in 1..=28 {
            la.tick(&sflogo, t);
        }
        let sflogo7 = solid;
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        {
            let mut g = SoftGraphics::new(&mut img);
            la.paint(&mut g, &sflogo7);
        }
        let w = layout.screen_w as usize;
        let h = layout.screen_h as usize;
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (w, 0usize, h, 0usize);
        // 内容像素=非白且非未铺黑画布（黑=白底外的漏铺区，计入会假过居中断言）
        for y in 0..h {
            for x in 0..w {
                let p = img.argb[y * w + x];
                if p != 0xFFFF_FFFF && p != 0x0000_0000 {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }
        assert!(max_x > min_x, "内容须绘出 bbox x[{min_x},{max_x}] y[{min_y},{max_y}]");
        // 经典基准（boot-classic.png bbox 实测 x[76,164] y[91,231] 心 120,161≈画布心
        // 120,160）：整组居中（logo 组+署名同组；署名=组内底部成员）。
        let cx = (min_x + max_x) / 2;
        let cy = (min_y + max_y) / 2;
        let expect_cx = (layout.screen_w / 2) as usize;
        let expect_cy = (layout.screen_h / 2) as usize;
        assert!(
            cx.abs_diff(expect_cx) <= 1,
            "logo 组 bbox 水平中心=画布中心±1（现={cx} vs {expect_cx}）——偏左上 artifact"
        );
        assert!(
            cy.abs_diff(expect_cy) <= 1,
            "logo 组 bbox 垂直中心=画布中心±1（现={cy} vs {expect_cy}）bbox x[{min_x},{max_x}] y[{min_y},{max_y}]"
        );
    }

    /// 追加件判据（2026-10-19 boss 004b 验证发现）：boot logo（mode 0 开机屏黑边）
    /// ——白底须**满画布**（黑边=固定 SCREEN_* 常量致右/下未铺）；logo 原尺寸居中。
    #[test]
    fn modern_boot_white_background_fills_canvas() {
        let layout = crate::layout::Layout::modern(13, 13);
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        let mut logo = synth(164, 117);
        logo.argb.fill(0xFF884422);
        let boot = crate::paint::BootPaintState {
            phase: 0,
            image: Some(logo),
            logo: None,
            sflogo7: None,
        };
        {
            let mut g = SoftGraphics::new(&mut img);
            crate::paint::paint_boot(&mut g, &boot, &layout);
        }
        let w = layout.screen_w as usize;
        let h = layout.screen_h as usize;
        for (x, y) in [(5, 5), (w - 6, 5), (5, h - 6), (w - 6, h - 6)] {
            let p = img.argb[y * w + x];
            assert_eq!(p, 0xFFFF_FFFF, "四角须白底满幅（{x},{y} 现={p:#010x}）——黑边 artifact");
        }
    }

    /// 用户验收修正判据（2026-10-19）：标题左右延展带=**顶带同款纯蓝**（天空连续感，
    /// 无渐变无波浪纹）——取图中缘像素（云纹边缘）即红。
    #[test]
    fn modern_title_side_bands_match_top_pure_color() {
        let layout = crate::layout::Layout::modern(13, 13);
        let mut t = crate::title::TitleMachine::new();
        // 标题图：上缘纯蓝 + 中缘云纹（波浪纹源）
        let mut bg = synth(240, 320);
        let sky: u32 = 0xFF3388CC;
        let wavy: u32 = 0xFF7799AA;
        for y in 0..320 {
            for x in 0..240 {
                bg.argb[y * 240 + x] = if y < 4 { sky } else if x % 37 == 0 { wavy } else { sky };
            }
        }
        let strip = synth(60, 17);
        let arrow = synth(7, 13);
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        {
            let mut g = SoftGraphics::new(&mut img);
            t.paint(&mut g, &bg, &strip, &arrow, &layout);
        }
        let w = layout.screen_w as usize;
        let sample_y = 200usize; // 图中带行
        let left_px = img.argb[sample_y * w + 5];
        let right_px = img.argb[sample_y * w + (w - 6)];
        assert_eq!(left_px, sky, "左带=顶带同款纯蓝（现={left_px:#010x}）");
        assert_eq!(right_px, sky, "右带=顶带同款纯蓝（现={right_px:#010x}）");
    }

    /// 用户验收修正判据（2026-10-19）：序章立绘两侧延展带=**素材边缘色纯色带**
    ///（云雾色，自然过渡）——留黑/取中缘波浪纹即红。
    #[test]
    fn modern_intro_side_bands_use_asset_edge_color() {
        
        let layout = crate::layout::Layout::modern(13, 13);
        let mut iv = crate::intro::IntroSequence::new();
        let mist: u32 = 0xFFCCDDEE;
        let mut portrait = synth(120, 200);
        for y in 0..200 {
            for x in 0..120 {
                portrait.argb[y * 120 + x] = if x == 0 || y == 0 { mist } else { 0xFFAA8866 };
            }
        }
        // 直接测绘制：构造最小 paint 输入（image 字段注入）
        let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
        {
            let mut g = SoftGraphics::new(&mut img);
            let mut ov = crate::menu_family::OverlayEngine::new();
            let mut parts = crate::title::Particles::new();
            let font = crate::paint::paint_font();
            iv.set_image_for_test(portrait);
            iv.paint(&mut g, &mut ov, &mut parts, &font, &layout);
        }
        let w = layout.screen_w as usize;
        let band_y = ((layout.screen_h - 200 - (12 + 4) * 3) >> 1) as usize;
        let left_px = img.argb[(band_y + 100) * w + 5];
        assert_eq!(left_px, mist, "左带=素材边缘色纯色（现={left_px:#010x}）");
    }

    /// 设置勾选翻转 ⇒ SKY_WAR dirty（a.java:4154 翻转 + 4169 saveGlobalProgress）。
    #[test]
    fn settings_toggle_marks_dirty() {
        let mut m = boot_state();
        m.mode = GameMode::About;
        m.options = Some(crate::menu_family::OptionList::new(true, true));
        let before = m.global_snapshot();
        m.tick(-5);
        assert!(m.take_global_dirty(), "设置翻转 ⇒ dirty（4169 触发）");
        let g = m.global_snapshot();
        assert_eq!(g.option_checked[0], !before.option_checked[0], "声音行随翻转");
        assert!(!m.take_global_dirty(), "dirty 一次性消费");
    }

    /// 水位助手条件（a.java:4297/7163 共用）：50 抬升；51 结局层不计；不回退。
    #[test]
    fn watermark_bump_conditions() {
        let mut m = boot_state();
        m.note_max_floor(50);
        assert!(m.take_global_dirty(), "水位抬升 ⇒ dirty");
        assert_eq!(m.global_snapshot().max_floor_watermark, 50);
        m.note_max_floor(51);
        assert!(!m.take_global_dirty(), "51 不计入（maxFloor<51）");
        assert_eq!(m.global_snapshot().max_floor_watermark, 50);
        m.note_max_floor(30);
        assert_eq!(m.global_snapshot().max_floor_watermark, 50, "单调不回退");
    }

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn decode(name: &str) -> ArgbImage {
        let bytes = std::fs::read(repo().join("assets/raw").join(name)).unwrap();
        ArgbImage::from_decoded_png(game_data::decode_png(&bytes).unwrap())
    }

    /// trace A-boot-menu 两相启动图与帧哈希逐一比对：
    /// - l0.png（164x117@38,101）→ TICK 8 帧 `57173976…`
    /// - l1.png（176x138@32,91）→ TICK 24 帧 `6a8aa7a0…`
    ///
    /// 证据：`reference/oracle/_out/A-boot-menu/trace.txt` T8/T24 的 OPS 行
    /// （白底 + drawImage）与 FRAME sha（`(240-w)>>1, (320-h)>>1` 居中算式）。
    #[test]
    fn boot_phase0_matches_trace_frames() {
        for (file, w, h, x, y, tick, sha) in [
            ("l0.png", 164, 117, 38, 101, 8, "57173976f11eeae0aed2c0029b6d5dd6"),
            ("l1.png", 176, 138, 32, 91, 24, "6a8aa7a0f744030761e35dac5b0c002b"),
        ] {
            let img = decode(file);
            assert_eq!((img.width, img.height), (w, h), "{file} 尺寸");

            let mut screen = ArgbImage::create(crate::layout::SCREEN_W, crate::layout::SCREEN_H);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut screen);
                g.set_font(Some(paint_font()));
                let boot = BootPaintState {
                    phase: 0,
                    image: Some(img),
                    logo: None,
                    sflogo7: None,
                };
                paint_boot(&mut g, &boot, &crate::layout::Layout::classic());
                ops = g.ops.clone();
            }
            assert_eq!(
                frame_sha(&screen),
                sha,
                "{file} 白底+居中必须与 Java 帧（TICK {tick}）逐字节一致"
            );
            assert_eq!(
                ops,
                vec![
                    "setFont(size=8)".to_string(),
                    "setColor(#ffffff)".to_string(),
                    "fillRect(0,0,240,320,#ffffff)".to_string(),
                    format!("drawImage({w}x{h},{x},{y},0)"),
                ],
                "{file} OPS 序列"
            );
        }
    }

    /// trace TICK 32：f_Image_00 已释放（null）⇒ 纯白帧。
    #[test]
    fn boot_phase0_no_image_matches_trace_t32() {
        let mut screen = ArgbImage::create(crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_font(Some(paint_font()));
            let boot = BootPaintState { phase: 0, image: None, logo: None, sflogo7: None };
            paint_boot(&mut g, &boot, &crate::layout::Layout::classic());
        }
        // TICK 32 帧哈希 = 纯白画布（与 L1b blank_white golden 同值，交叉验证）
        assert_eq!(frame_sha(&screen), "a682fa570213181c0f6fd50f7da5ef6f");
    }
}
