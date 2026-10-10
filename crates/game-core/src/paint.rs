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
//! - [x] mode 0 tick 逻辑：加载轮播 + logo 时间线（a.java:3341-3372）
//! - [ ] mode 21（logo 后过渡）、mode 4/19 主菜单、mode 1/2 加载、其余分支

use crate::render::{ArgbImage, SoftFont, SoftGraphics, font_size, font_style};

/// paint 入口持有的常量字体（a.java:24：`f_Font_00 = Font.getFont(0, 0, 8)`）。
pub fn paint_font() -> SoftFont {
    SoftFont::get_font(0, font_style::PLAIN, font_size::SMALL)
}

/// `nokiaTransformTable`（a.java:44）：`drawImageWithNokiaTransform` 的 Nokia 变换码表。
/// 全部值 >7 ⇒ shim `drawImageTransformed` 的 switch 落 default = 原样绘制
/// （op 文本仍记录原始码，`drawImageT`）。
pub const NOKIA_TRANSFORM_TABLE: [i32; 8] = [0, 8192, 16384, 24576, 8462, 270, 90, 8282];

/// `drawImageWithNokiaTransform`（a.java:4549-4551）：DirectGraphics 变换绘制（shim 下视觉 = 原样）。
fn draw_image_with_nokia_transform(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, kind: i32) {
    let t = NOKIA_TRANSFORM_TABLE[kind as usize];
    g.draw_image_transformed(img, x, y, 0, t);
}

/// `drawImageClipped`（a.java:4537-4541）：源矩形 clip 绘制——clip(x,y,w,h) 后把图
/// 画在 `(x-sx, y-sy)`，只露出 `(sx,sy)` 起的子区。
pub fn draw_image_clipped(g: &mut SoftGraphics<'_>, img: &ArgbImage, x: i32, y: i32, sx: i32, sy: i32, w: i32, h: i32) {
    g.set_clip(x, y, w, h);
    g.draw_image(img, x - sx, y - sy, 0);
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
}

/// `paintSoftkeyBar`（a.java:5979-5989）：软键栏。`left`/`right` = softkeyLeftKind/softkeyRightKind
/// （0 = 不绘制；kind 1..n 选 ui[11] 精灵条的第 n 个 12×10 图标）。
pub fn m_034_softkeys(g: &mut SoftGraphics<'_>, ui10: &ArgbImage, ui11: &ArgbImage, left: i8, right: i8) {
    if left != 0 {
        g.draw_image(ui10, 0, crate::layout::softkey_base_y(), 0);
        draw_image_clipped(
            g,
            ui11,
            crate::layout::SOFTKEY_ICON_LEFT_X,
            crate::layout::softkey_icon_y(),
            (left as i32 - 1) * crate::layout::SOFTKEY_ICON_W,
            0,
            crate::layout::SOFTKEY_ICON_W,
            crate::layout::SOFTKEY_ICON_H,
        );
    }
    if right != 0 {
        draw_image_with_nokia_transform(g, ui10, crate::layout::softkey_right_x(), crate::layout::softkey_base_y(), 1);
        draw_image_clipped(
            g,
            ui11,
            crate::layout::softkey_icon_right_x(),
            crate::layout::softkey_icon_y(),
            (right as i32 - 1) * crate::layout::SOFTKEY_ICON_W,
            0,
            crate::layout::SOFTKEY_ICON_W,
            crate::layout::SOFTKEY_ICON_H,
        );
    }
}

/// mode 21（声音询问）绘制：a.java:3058-3065 + paintSoftkeyBar 软键栏。
///
/// 软键 (1,2) 由 m_000 case 21 设置（a.java:4468-4469）。
pub fn paint_sound_prompt(g: &mut SoftGraphics<'_>, ui10: &ArgbImage, ui11: &ArgbImage, softkeys: (i8, i8)) {
    g.set_color(0);
    g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
    g.set_color((-1i32) as u32);
    g.draw_string(
        &crate::layout::SOUND_PROMPT_TEXT,
        crate::layout::prompt_cx(),
        crate::layout::prompt_cy(),
        anchor_top_hcenter(),
    );
    m_034_softkeys(g, ui10, ui11, softkeys.0, softkeys.1);
}

/// mode 15 帮助 overlay 文本（a.java:4391-4397，逐字）。
pub const HELP_TEXT: &str = "游戏描述：\n\\c99FFCC有人的地方就有江湖，有神仙的地方何尝不是江湖；百战百胜的本事，换不回女人的真心，兄弟的真义；齐天大圣又如何，没有真情实义，做神仙跟做咸鱼有什么区别？\n\n操作方式：按左软键调出物品栏，左右选择一件道具，按确定键使用。\n游戏操作：\n上方向键/2：向上行走\n下方向键/8：向下行走\n左方向键/4：向左行走\n右方向键/6：向右行走\n确定键/5:探索地图\n左软键：打开道具列表\n右软键：打开游戏中菜单\n\n代理发行：广州易诚计算机科技有限公司\n发行商网站：www.9266.net\n客服电话：4006509913\n客服信箱：kefu@9266.net";

/// mode 17 关于 overlay 文本（a.java:4410-4423，逐字）。
pub const ABOUT_TEXT: &str = "版权所有：\n上海雪鲤鱼计算机科技有限公司\nwww.kgame.com.cn\n手机上网：\nwap.kgame.com.cn\n制作人：梁一\n编剧：王之浣\n策划：孙悦\n程序：杨政\n美术：梁一、王之浣、黄吉力\n测试：金鑫，王毅，计成毅\n版本：V1.0\n客服电话：4006305518";

/// `Graphics.TOP | Graphics.HCENTER` = 17（drawString 第三参的常用组合）。
pub fn anchor_top_hcenter() -> i32 {
    crate::render::anchor::TOP | crate::render::anchor::HCENTER
}

/// mode 0（启动屏）绘制：a.java:2338-2347。
///
/// 相位 <2：整屏白 + 启动图几何居中（Java `-` 优先于 `>>` ⇒ `(240-w)>>1`）；
/// 相位 >=2：`paintLogoAnimation` logo 动画层（[`crate::logo_anim::LogoAnim::paint`]）。
pub fn paint_boot(g: &mut SoftGraphics<'_>, boot: &BootPaintState) {
    if boot.phase < 2 {
        // setColor(-1)：Java int -1 位模式 & 0xFFFFFF = 0xFFFFFF
        g.set_color((-1i32) as u32);
        g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        if let Some(img) = &boot.image {
            let x = crate::layout::SCREEN_W - img.width >> 1;
            let y = crate::layout::SCREEN_H - img.height >> 1;
            g.draw_image(img, x, y, 0);
        }
    } else {
        match (&boot.logo, &boot.sflogo7) {
            (Some(logo), Some(sflogo7)) => logo.paint(g, sflogo7),
            // logic 已切 bootLoadPhase>=2 但首个 runLogoAnimation 未跑：logoItemCount=0 ⇒ paintLogoAnimation 仅白底
            // （A-boot-menu T32 硬锚：a682fa57=纯白）
            _ => {
                g.set_color(0xFFFFFF);
                g.fill_rect(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
            }
        }
    }
}

/// mode 0 启动状态机：tick 逻辑（a.java:3341-3372）+ paint 输入投影。
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
    pub mode: i32,
    /// keyValue（keyPressed 设置、tick 末清零，a.java:4650/4251）。
    pub key_value: i32,
    /// bootLoadPhase：加载相位（0=l0, 1=l1, 2+=logo 时间线）。
    pub phase: i32,
    /// bootPhaseCounter：相位内计数（轮播每张 16 tick：1..15 显示，16 切换；时间线 1..35）。
    pub counter: i32,
    /// f_Image_00：当前启动图（l0/l1）。
    pub image: Option<ArgbImage>,
    /// logo 动画层（bootLoadPhase>=2 起参与绘制）。
    pub logo: Option<crate::logo_anim::LogoAnim>,
    /// l0/l1 预解码件（a.java:3346 的 `/l{n}.png`）。
    boot_images: Vec<ArgbImage>,
    /// sflogo 容器 8 张（m_001(0) 首次进 logo 相位时加载）。
    sflogo: Vec<ArgbImage>,
    /// ui 容器 25 张（m_001(8)，m_000 case 21 a.java:4467 加载）。
    ui: Vec<ArgbImage>,
    /// softkeyLeftKind/softkeyRightKind 软键栏状态（paintSoftkeyBar 绘制输入）。
    pub softkeys: (i8, i8),
    /// mode 1 标题菜单（gameMode=1 起接管 tick/paint）。
    pub title: Option<crate::title::TitleMachine>,
    /// menu 容器 2 张（m_001(10)：资源名表第 10 项 "menu"，mode 1 背景/图标带）。
    menu: Vec<ArgbImage>,
    /// gameRandom（构造时 setSeed(VTime=0)；JavaRandom LCG）。
    pub rng: game_platform::JavaRandom,
    /// mapbg 容器 1 张（m_001(1)：资源名表第 1 项，parallax 平铺 77x320）。
    mapbg: ArgbImage,
    /// 视差背景滚动（paint 内推进）。
    pub backdrop: crate::menu_family::ParallaxBackdrop,
    /// overlay 文本引擎（overlayActive 层）。
    pub overlay: crate::menu_family::OverlayEngine,
    /// overlayActive：overlay 激活。
    pub overlay_active: bool,
    /// menuReturnMode：overlay/菜单的「返回模式」。
    pub return_mode: i32,
    /// mode 8 槽位表。
    pub slots: Option<crate::menu_family::SlotSelect>,
    /// mode 16 选项表。
    pub options: Option<crate::menu_family::OptionList>,
    /// iconStripFrame 共享（菜单族与 title 同一字段 a.java:464）。
    pub strip_frame: usize,
    /// soundEnabled（mode 16 勾选联动）。
    pub sound_enabled: bool,
    /// intro 容器 2 张（m_001(11)，引子页 0/1 图）。
    intro_imgs: Vec<ArgbImage>,
    /// end 容器 1 张（m_001(14)，引子页 2 图）。
    end_imgs: Vec<ArgbImage>,
    /// load 容器 2 张（m_001(15)，进度条图）。
    load_imgs: Vec<ArgbImage>,
    /// mode 14 引子状态。
    pub intro: Option<crate::intro::IntroSequence>,
    /// mode 2 加载链。
    pub loading: Option<crate::intro::LoadProgress>,
    /// f_int_02 帧间隔 ms（构造 75，a.java:27；mode 0 每 tick 设 100，
    /// a.java:3342；切 mode 21 时回 75，a.java:3369）。
    pub frame_interval_ms: i64,
    /// paint 计数（对拍 TICK n 用）。
    pub paints: u32,
    /// mode 0 结束标志（logo 时间线 35 耗尽，a.java:3362-3371 切 gameMode=21）。
    pub finished: bool,
}

impl BootMachine {
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
            mode: 0,
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
            menu: menu,
            mapbg: mapbg,
            backdrop: crate::menu_family::ParallaxBackdrop::new(),
            overlay: crate::menu_family::OverlayEngine::new(),
            overlay_active: false,
            return_mode: 0,
            slots: None,
            options: None,
            strip_frame: 0,
            sound_enabled: false,
            intro_imgs: intro_imgs,
            end_imgs: end_imgs,
            load_imgs: load_imgs,
            intro: None,
            loading: None,
            rng: game_platform::JavaRandom::new_seeded(0),
            frame_interval_ms: 75,
            paints: 0,
            finished: false,
        }
    }

    /// 一次 paint（serviceRepaints → a.paint）。按 [`Self::mode`] 分发。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>) {
        self.paints += 1;
        match self.mode {
            21 => paint_sound_prompt(g, &self.ui[10], &self.ui[11], self.softkeys),
            1 => {
                let mut t = self.title.take().unwrap();
                let sf = self.strip_frame;
                t.set_strip_frame(sf);
                t.paint(g, &self.menu[0], &self.menu[1], &self.ui[14]);
                self.strip_frame = t.strip_frame_value();
                self.title = Some(t);
            }
            8 => {
                self.backdrop.paint(g, &self.mapbg, false, 0);
                let segs = [3, 5]; // titleBarSegments[1]
                if let Some(slots) = &self.slots {
                    slots.paint(g, &self.ui, &segs, || 0);
                }
                m_034_softkeys(g, &self.ui[10], &self.ui[11], self.softkeys.0, self.softkeys.1);
            }
            15 | 17 => {
                self.backdrop.paint(g, &self.mapbg, false, 0);
                let sy = if self.mode == 15 { 51 } else { 68 };
                let mut f = self.strip_frame;
                crate::title::paint_icon_strip_at(g, &self.menu[1], 86, 10, sy, 68, &mut f);
                self.strip_frame = f;
                m_034_softkeys(g, &self.ui[10], &self.ui[11], self.softkeys.0, self.softkeys.1);
                self.paint_overlay_tail(g);
            }
            16 => {
                self.backdrop.paint(g, &self.mapbg, false, 0);
                let mut f = self.strip_frame;
                crate::title::paint_icon_strip_at(g, &self.menu[1], 86, 10, 34, 68, &mut f);
                self.strip_frame = f;
                if let Some(opts) = &self.options {
                    opts.paint(g, &self.ui, &crate::paint::paint_font());
                }
                m_034_softkeys(g, &self.ui[10], &self.ui[11], self.softkeys.0, self.softkeys.1);
            }
            22 => {
                let logo = self.logo.as_ref().unwrap();
                let sf7 = self.sflogo[7].clone();
                logo.paint(g, &sf7);
            }
            14 => {
                let intro = self.intro.as_mut().unwrap();
                let mut particles = self.title.as_mut().map(|t| std::mem::replace(&mut t.particles, crate::title::Particles::new()));
                if let Some(p) = particles.as_mut() {
                    let font = paint_font();
                    intro.paint(g, &mut self.overlay, p, &font);
                }
                if let Some(t) = self.title.as_mut() {
                    if let Some(p) = particles { t.particles = p; }
                }
            }
            2 => {
                self.loading.as_ref().unwrap().paint(g, &self.load_imgs);
            }
            _ => {
                let state = BootPaintState {
                    phase: self.phase,
                    image: self.image.clone(),
                    logo: self.logo.clone(),
                    sflogo7: Some(self.sflogo[7].clone()),
                };
                paint_boot(g, &state);
            }
        }
    }

    /// 一次逻辑 tick。`key` 为本 tick 边界投递的按键码（`keyValue`，无则 0）。
    ///
    /// - mode 0：a.java:3341-3372（加载轮播/logo 时间线/清理切换）
    /// - mode 21：a.java:4215-4234（-6 确认 / -7 否定 → gameMode=1）
    pub fn tick(&mut self, key: i32) {
        match self.mode {
            21 => {
                // a.java:4217-4234：switch (keyValue)
                match key {
                    -7 | -6 => {
                        // -7: soundEnabled=false；-6: true（音量 0→60）
                        //（a.java:4218-4227）。两者都 gameMode=1 + m_000()
                        self.sound_enabled = key == -6;
                        self.mode = 1;
                        self.title = Some(crate::title::TitleMachine::new());
                    }
                    _ => {}
                }
                self.key_value = 0;
            }
            1 => {
                // a.java:3383-3438。frame_counter：迭代 N 拍用 f=N-1
                //（for 自增在体后）；paints 在 tick 时 = 已 paint 数 = N-1
                let f = self.paints as i32;
                let mut next = None;
                if let Some(t) = self.title.as_mut() {
                    let sf = self.strip_frame;
                    t.set_strip_frame(sf);
                    next = t.tick(key, f, &mut self.rng);
                    self.strip_frame = t.strip_frame_value();
                }
                if let Some(mode) = next {
                    self.mode = mode;
                    // menuReturnMode：kind 1/2/3/4 → 1（a.java:3393/3397/3402）；
                    // kind 0(14)/5(22) 不设
                    if (1..=4).contains(&self.title_kind()) {
                        self.return_mode = 1;
                    }
                    self.enter_menu_mode();
                }
                self.key_value = 0;
            }
            8 => {
                // run case 8（a.java:3819-3874）
                match key {
                    -7 => self.mode = self.return_mode,
                    -2 => self.slots.as_mut().unwrap().cursor_down(),
                    -1 => self.slots.as_mut().unwrap().cursor_up(),
                    // -6/-5：mode 8 且槽有效才读档（oracle 恒无效 ⇒ 无操作）
                    _ => {}
                }
                self.key_value = 0;
            }
            15 | 17 => {
                // overlay 层 case 0 的 15/17 分支（a.java:3250-3263）
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
            16 => {
                // run case 16（a.java:4129-4177）
                match key {
                    -7 | -6 => self.mode = self.return_mode,
                    -5 | -4 | -3 => {
                        if let Some(opts) = self.options.as_mut() {
                            let i = opts.highlight as usize;
                            opts.checked[i] = !opts.checked[i];
                            if opts.labels[i] == 0 {
                                // case 0：声音联动（a.java:4142-4155）
                                self.sound_enabled = !self.sound_enabled;
                            }
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
            14 => {
                // run case 14（a.java:4123-4128）：m_014 + 粒子雨 4 分频
                let f = self.paints as i32;
                if f & 3 == 0 {
                    let x = self.rng.random_below(240);
                    let y = 320 - 25 - self.rng.random_below(150);
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
                    // m_067（a.java:7147-7160）：新游戏加载链
                    self.loading = Some(crate::intro::LoadProgress::new_game());
                    self.mode = 2;
                }
                self.key_value = 0;
            }
            2 => {
                // run case 2（a.java:3441-3540）：进度追赶模型
                if let Some((target, call_m000)) = self.loading.as_mut().unwrap().tick() {
                    self.loading = None;
                    self.mode = target;
                    if call_m000 {
                        self.finished = true; // m_000 case 3（游戏态初始化）未端口
                    }
                }
                self.key_value = 0;
            }
            22 => {
                // run case 22（a.java:4235-4244）：++bootPhaseCounter ≤ 70 →
                // runLogoAnimation(1, t)；否则退出进程
                self.counter += 1;
                if self.counter <= 70 {
                    self.logo
                        .get_or_insert_with(crate::logo_anim::LogoAnim::new)
                        .tick_help(&self.sflogo, self.counter);
                } else {
                    self.finished = true; // CMidlet.m_000()（进程退出）
                }
                self.key_value = 0;
            }
            _ => self.tick_mode0(),
        }
    }

    /// 当前 title 光标对应的 kind（mode 1 出口分派键）。
    fn title_kind(&self) -> i32 {
        self.title
            .as_ref()
            .map(|t| t.menu_kinds[t.cursor as usize] as i32)
            .unwrap_or(-1)
    }

    /// m_000 的菜单族 case（a.java:4346/4386-4425）：进入模式时初始化。
    fn enter_menu_mode(&mut self) {
        match self.mode {
            8 => {
                // case 8（a.java:4346-4371）：槽表 + 布局常量 + 软键 (1,3)
                self.slots = Some(crate::menu_family::SlotSelect::new());
                self.softkeys = (1, 3);
            }
            15 => {
                // case 15（a.java:4388-4398）：帮助 overlay + 软键 (0,3)
                let text = HELP_TEXT.encode_utf16().collect::<Vec<u16>>();
                let font = paint_font();
                self.overlay.show_kind0(&text, &font, (0, 0));
                self.overlay_active = true;
                self.softkeys = (0, 3);
            }
            17 => {
                // case 17（a.java:4409-4427）：关于 overlay + 软键 (0,3)
                let text = ABOUT_TEXT.encode_utf16().collect::<Vec<u16>>();
                let font = paint_font();
                self.overlay.show_kind0(&text, &font, (0, 0));
                self.overlay_active = true;
                self.softkeys = (0, 3);
            }
            16 => {
                // case 16（a.java:4400-4408）：选项表 + 软键 (0,3)。
                // optionChecked[1]=true：mode 21 的 RMS 失败 catch（a.java:4453）
                self.options = Some(crate::menu_family::OptionList::new(
                    self.sound_enabled,
                    true,
                ));
                self.softkeys = (0, 3);
            }
            14 => {
                // case 14（a.java:4372-4380 区）：intro/end 容器 + 引子状态机
                //（粒子表沿用 title 的 Particles——Java 全局唯一）
                self.intro = Some(crate::intro::IntroSequence::new());
            }
            22 => {
                // case 22（a.java:4472）：closeAudio（shim 无副作用）；
                // bootPhaseCounter 已为 0（mode 0→21 清理）
                self.counter = 0;
            }
            _ => {
                self.finished = true; // 其余未端口模式
            }
        }
    }

    /// paint 公共尾 overlay（a.java:3121-3169，overlayActive && !f_bool_16）。
    fn paint_overlay_tail(&mut self, g: &mut SoftGraphics<'_>) {
        if !self.overlay_active {
            return;
        }
        let x = 240 - self.overlay.box_w >> 1;
        let y = 320 - self.overlay.box_h >> 1;
        crate::menu_family::paint_box_frame(g, &self.ui[0], x, y, self.overlay.box_w, self.overlay.box_h);
        g.set_color((-1i32) as u32);
        let text: Vec<u16> = if self.mode == 15 {
            HELP_TEXT.encode_utf16().collect()
        } else {
            ABOUT_TEXT.encode_utf16().collect()
        };
        let f = self.paints as i32 - 1;
        let font = paint_font();
        self.overlay.paint(g, &text, x + 16, y + 16, 180, 240, true, f, &font);
    }

    /// gameMode 0 的逻辑 tick（a.java:3341-3372）。
    fn tick_mode0(&mut self) {
        self.frame_interval_ms = 100; // a.java:3342（mode 0 每 tick 首行）
        if self.phase < 2 {
            if self.counter == 0 {
                // a.java:3346：加载 /l{phase}.png；夹具恒存在 ⇒ 恒成功路径
                let idx = self.phase as usize;
                self.image = self.boot_images.get(idx).cloned();
                if self.image.is_some() {
                    self.counter = 1;
                } else {
                    self.phase += 1; // 防御：a.java:3350-3352
                }
            } else {
                self.counter += 1;
                if self.counter > 15 {
                    self.phase += 1; // a.java:3357
                    self.counter = 0;
                }
            }
        } else {
            self.counter += 1; // a.java:3360
            if self.counter <= 35 {
                self.logo.get_or_insert_with(crate::logo_anim::LogoAnim::new)
                    .tick(&self.sflogo, self.counter);
            } else {
                // a.java:3363-3371：清理 + gameMode=21 + m_000()
                self.logo.as_mut().unwrap().clear_all(); // removeLogoItem(-1)
                self.logo.as_mut().unwrap().reset_layout_flag(); // logoLayoutDone=false（a.java:3364）
                self.image = None;
                self.counter = 0;
                self.mode = 21;
                self.frame_interval_ms = 75; // f_int_02=75（a.java:3369）
                // m_000 case 21（a.java:4468-4470）：软键 (1,2)；soundEnabled=false；
                // RMS "SKY_WAR" 读档失败路径（oracle 内存库恒空 ⇒ catch 删库重置）
                self.softkeys = (1, 2);
                self.finished = false; // mode 21 属对拍范围
            }
        }
        self.key_value = 0; // a.java:4262（tick 末 keyValue=0）
    }
}

/// mode 0 分支的绘制输入投影（a.java:2339 `bootLoadPhase` 等的只读快照）。
#[derive(Clone)]
pub struct BootPaintState {
    /// 启动加载相位：0/1 → 轮播 `/l{n}.png`；>=2 → logo 动画层。
    pub phase: i32,
    /// 启动图（l0.png 164×117 / l1.png 176×138，加载线程 a.java:3343-3357）。
    pub image: Option<ArgbImage>,
    /// logo 动画层（phase>=2 时 Some）。
    pub logo: Option<crate::logo_anim::LogoAnim>,
    /// sflogo#7（paintLogoAnimation 字形槽源图，a.java:10451）。
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

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn decode(name: &str) -> ArgbImage {
        let bytes = std::fs::read(repo().join("assets/raw").join(name)).unwrap();
        ArgbImage::from_decoded_png(game_data::decode_png(&bytes).unwrap())
    }

    fn load_sflogo() -> Vec<ArgbImage> {
        let data = std::fs::read(repo().join("assets/raw/sflogo")).unwrap();
        game_data::PackedPng::parse(&data, 8)
            .unwrap()
            .images
            .iter()
            .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
            .collect()
    }

    /// trace A-boot-menu 两相启动图与帧哈希逐一比对：
    /// - l0.png（164x117@38,101）→ TICK 8 帧 `57173976…`
    /// - l1.png（176x138@32,91）→ TICK 24 帧 `6a8aa7a0…`
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

            let mut screen = ArgbImage::create(240, 320);
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
                paint_boot(&mut g, &boot);
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
        let mut screen = ArgbImage::create(240, 320);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_font(Some(paint_font()));
            let boot = BootPaintState { phase: 0, image: None, logo: None, sflogo7: None };
            paint_boot(&mut g, &boot);
        }
        // TICK 32 帧哈希 = 纯白画布（与 L1b blank_white golden 同值，交叉验证）
        assert_eq!(frame_sha(&screen), "a682fa570213181c0f6fd50f7da5ef6f");
    }
}
