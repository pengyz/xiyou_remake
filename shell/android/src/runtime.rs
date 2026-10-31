//! Android 壳运行时（宿主可测纯调度）：Boot/GameScene 双机 + 交接 + 存档写穿。
//! android_main 主循环与宿主流程钉（tests/flow.rs）**同一调度体**——实机"进不了游戏"
//! （读条完 mode=Map 无人接管、tick_mode0 回落标题）的修复面在此单测。
//!
//! 交接体/资产装载/存档写穿 = `game_shell_common::handoff`（win 壳同源，消费不拷贝）；
//! 本模块只做 Android 调度：拍前输入解析（标题手势/UI 行点按/场景点按）→ 活动机 tick →
//! 交接检查 → kind7 退出换回标题。

use crate::input::KeyAction;
use crate::native::{
    boot_key_from_batch, log_input, replay_scene_keys, resolve_title_gestures, scene_tap, OverlayState,
    TitleGesture,
};
use game_core::enums::GameMode;
use game_core::paint::BootMachine;
use game_core::render::{ArgbImage, SoftGraphics};
use game_core::scene::GameScene;
use game_shell_common::audio::{AudioSink, BgmController, BgmSnapshot, NullSink};
use game_shell_common::handoff::{self, GameAssets};
use game_shell_common::settings::AppSettings;

/// 【STAGE-B-006】壳层附加设置行 id（core 不解释含义；行序 = 设置页第 3-5 行）。
pub const PREF_SPLASH: u8 = 1;
pub const PREF_HAPTIC: u8 = 2;
pub const PREF_CLICK: u8 = 3;
/// 【STAGE-B-008 计划 3 T3】操控方式行（checked = 触屏；默认摇杆=unchecked）。行增减归 T3.2。
pub const PREF_CONTROL_MODE: u8 = 4;
/// 【STAGE-B-008 计划 3 T3】自动战斗行（仅触屏模式出现，T3.2 增减）。
pub const PREF_AUTO_BATTLE: u8 = 5;

/// 【STAGE-B-006】设置 → 附加行：开屏画面 / 按键震动 / 按键音（排在原版"声音、小地图"之后）。
pub fn shell_rows(s: &AppSettings) -> Vec<game_core::menu_family::ShellOption> {
    [(PREF_SPLASH, "开屏画面", s.show_splash), (PREF_HAPTIC, "按键震动", s.key_haptic), (PREF_CLICK, "按键音", s.key_click)]
        .into_iter()
        .map(|(id, label, checked)| game_core::menu_family::ShellOption { id, label: label.encode_utf16().collect(), checked })
        .collect()
}

/// 附加行勾选（缺行 ⇒ `default`）。
fn row_checked(rows: &[game_core::menu_family::ShellOption], id: u8, default: bool) -> bool {
    rows.iter().find(|r| r.id == id).map_or(default, |r| r.checked)
}

/// BootMachine 装配（APK/目录资产 → 游戏机；win 壳 Shell::new 同参）。
pub fn build_boot_machine(src: &dyn game_platform::AssetSource, layout: game_core::layout::Layout) -> BootMachine {
    let png = |name: &str| {
        ArgbImage::from_decoded_png(
            game_data::decode_png(&src.read(name).unwrap_or_else(|e| panic!("缺资产 {name}：{e}")))
                .unwrap_or_else(|e| panic!("{name} 解码失败：{e}")),
        )
    };
    let mapbg = handoff::load_container(src, "mapbg", 1);
    let mut machine = BootMachine::new(
        png("l0.png"),
        png("l1.png"),
        handoff::load_container(src, "sflogo", 8),
        handoff::load_container(src, "ui", 25),
        handoff::load_container(src, "menu", 2),
        mapbg[0].clone(),
        handoff::load_container(src, "intro", 2),
        handoff::load_container(src, "end", 1),
        handoff::load_container(src, "load", 2),
    );
    machine.sptarm = handoff::load_container(src, "sptarm", 10);
    machine.apply_layout(layout);
    machine
}

/// 双机运行时。
pub struct Runtime {
    /// BootMachine（场景活动期=换机储备：kind7 退出回标题即换回，标题态保真）。
    pub machine: BootMachine,
    /// 活动游戏场景（Some=GameScene 接管 tick/paint）。
    pub scene: Option<Box<GameScene>>,
    /// 场景资产（首次交接时装载：55 层 + 场景容器）。
    pub game_assets: Option<GameAssets>,
    /// 持久记录真源（MOT_L0-5/MOT_IF/SKY_WAR）。
    pub stores: Vec<(String, Vec<u8>)>,
    storage: Box<dyn game_platform::Storage>,
    /// 已落盘槽存档指纹（sync_scene_saves 幂等判据）。
    pub persisted_blobs: [Option<Vec<u8>>; 6],
    /// 场景帧计数（GameScene::tick frame_counter；win 壳 self.frame_counter 同口径）。
    pub frame_counter: u64,
    /// 场景崩溃（game-core panic 被捕获）：停拍留帧，日志 GAME-CRASH。
    pub crashed: bool,
    /// 【竖屏动态分辨率】游戏场景布局（交接时 apply；开场链布局=machine.layout）——
    /// 两档逻辑分辨率共用同一物理视口（portrait.rs）。缺省=machine 同布局（旧单档口径）。
    pub scene_layout: game_core::layout::Layout,
    /// BGM 命令翻译（缺省出口静音；android_main 注入音频线程通道）。
    bgm: BgmController,
    /// 壳层退出请求（BUG-034：根界面返回 = 立即退出；粘滞）。
    shell_exit: bool,
    /// 【STAGE-B-005】系统退出对话框未决（已请求弹出、未收到结果）：游戏暂停、输入不进游戏。
    exit_dialog_pending: bool,
    /// 【STAGE-B-006】设置文件（路径, 已落盘值）：None = 未接线（宿主流程测试 / 无持久场景）不落盘。
    settings_file: Option<(std::path::PathBuf, AppSettings)>,
    /// 【STAGE-B-008 计划 3 T3 · spec §8】运行时操控设置：control_mode/auto_battle 行存在时以行为准
    ///（T3.2 行同步接入），行缺失时保留本字段；tips_seen_* 恒纯 Runtime 字段。
    pub rt_control_mode: game_shell_common::settings::ControlMode,
    pub rt_auto_battle: bool,
    pub rt_tips_seen_pad: bool,
    pub rt_tips_seen_touch: bool,
}

impl Runtime {
    /// 装配：存档接线（读档列表真源 + SKY_WAR 装载，win 壳 Shell::new 同口径）。
    pub fn new(
        mut machine: BootMachine,
        stores: Vec<(String, Vec<u8>)>,
        storage: Box<dyn game_platform::Storage>,
    ) -> Runtime {
        machine.preset_stores = stores.clone();
        handoff::apply_stored_global(&mut machine, &stores);
        let scene_layout = machine.layout.clone();
        Runtime {
            machine,
            scene: None,
            game_assets: None,
            stores,
            storage,
            persisted_blobs: Default::default(),
            frame_counter: 0,
            crashed: false,
            scene_layout,
            bgm: BgmController::new(Box::new(NullSink)),
            shell_exit: false,
            exit_dialog_pending: false,
            settings_file: None,
            rt_control_mode: game_shell_common::settings::ControlMode::Pad,
            rt_auto_battle: false,
            rt_tips_seen_pad: false,
            rt_tips_seen_touch: false,
        }
    }

    /// 【STAGE-B-006】应用设置（启动时、首拍前）：声音对 → 三行附加行 → 开屏关则跳过开屏。顺序不可换：跳过即进标题，
    /// `enter_title` 按当时的声音开关建播放器（复核 M5③）。只写开场链机器（启动时无场景）。
    pub fn apply_app_settings(&mut self, s: &AppSettings) {
        // 【STAGE-B-008 计划 3 T3】运行时操控设置（行覆盖待 T3.2 接线）
        self.rt_control_mode = s.control_mode;
        self.rt_auto_battle = s.auto_battle;
        self.rt_tips_seen_pad = s.tips_seen_pad;
        self.rt_tips_seen_touch = s.tips_seen_touch;
        self.machine.bgm.sound_enabled = s.sound_enabled;
        self.machine.bgm.volume_level = s.volume_level;
        self.machine.shell_options = shell_rows(s);
        if !s.show_splash {
            self.machine.skip_boot_splash();
        }
    }

    /// 【STAGE-B-006】当前设置快照：读活动机（场景活动读场景，否则读开场链）的声音对与三行勾选；缺行取默认。
    /// 场景活动期开场链那份是交接前的旧值，壳侧一律经本函数读，不直接读 `machine.shell_options`（复核 T4-F2）。
    pub fn app_settings(&self) -> AppSettings {
        let (bgm, rows) = match self.scene.as_ref() {
            Some(sc) => (&sc.bgm, &sc.shell_options),
            None => (&self.machine.bgm, &self.machine.shell_options),
        };
        let d = AppSettings::default();
        AppSettings {
            sound_enabled: bgm.sound_enabled,
            volume_level: bgm.volume_level,
            show_splash: row_checked(rows, PREF_SPLASH, d.show_splash),
            key_haptic: row_checked(rows, PREF_HAPTIC, d.key_haptic),
            key_click: row_checked(rows, PREF_CLICK, d.key_click),
            // 【STAGE-B-008 计划 3 T3】操控方式/自动战斗：行存在时以行为准（T3.2 接线）；
            // 此处先取 Runtime 字段（spec §8：行缺失不回默认）。tips_seen_* 恒 Runtime 值。
            control_mode: self.rt_control_mode,
            auto_battle: self.rt_auto_battle,
            tips_seen_pad: self.rt_tips_seen_pad,
            tips_seen_touch: self.rt_tips_seen_touch,
        }
    }

    /// 【STAGE-B-006】设置文件接线（android_main：Runtime::new 之后、首拍前调用一次）：装载 → 应用 → 记住路径与已落盘值。
    /// 之后每拍末快照变化即写（[`Self::step`]，同一拍内落盘：退出请求后不再出拍也不丢，复核 L1）。
    pub fn open_app_settings(&mut self, path: std::path::PathBuf) {
        let s = AppSettings::load(&path);
        self.apply_app_settings(&s);
        log_input(&format!("app settings loaded {s:?} → mode={:?}", self.machine.mode));
        self.settings_file = Some((path, s));
    }

    /// 【STAGE-B-006】设置快照相对已落盘值变化 ⇒ 写文件（tmp + rename，失败吞）。
    fn persist_app_settings_if_changed(&mut self) {
        let cur = self.app_settings();
        if let Some((path, saved)) = self.settings_file.as_mut() {
            if *saved != cur {
                cur.save(path);
                *saved = cur;
                log_input(&format!("app settings persisted {cur:?}"));
            }
        }
    }

    /// 【竖屏动态分辨率】双档布局换轨：开场链 → machine、游戏 → 现场景 + 后续交接。
    pub fn apply_layouts(&mut self, boot: game_core::layout::Layout, scene: game_core::layout::Layout) {
        self.machine.apply_layout(boot);
        if let Some(sc) = self.scene.as_mut() {
            sc.apply_layout(scene.clone());
        }
        self.scene_layout = scene;
    }

    /// 活动机对应的画布档（场景活动=scene 档，否则=boot 档）。
    pub fn active_canvas(&self, plan: &crate::portrait::PortraitPlan) -> crate::portrait::Canvas {
        if self.scene.is_some() {
            plan.scene
        } else {
            plan.boot
        }
    }

    /// 活动机软键种类（面板软键图标=游戏同款，随态切换）。
    pub fn softkeys(&self) -> (i8, i8) {
        match self.scene.as_ref() {
            Some(sc) => sc.softkeys,
            None => self.machine.softkeys,
        }
    }

    /// 游戏内"声音"开关（活动机 `bgm.sound_enabled`；交接时随 bgm 续接，设置页翻转即变）。只读，不碰游戏状态。
    /// > 【2026-10-28 更正 · STAGE-B-006】按键音不再据此门控（改看设置页"按键音"，[`Self::key_feedback`]）。
    pub fn sound_enabled(&self) -> bool {
        match self.scene.as_ref() {
            Some(sc) => sc.bgm.sound_enabled,
            None => self.machine.bgm.sound_enabled,
        }
    }

    /// 【STAGE-B-006】按键反馈开关（触感, 按键音）= 活动机设置页"按键震动 / 按键音"两行（缺行默认开）；与"声音"无关。
    pub fn key_feedback(&self) -> (bool, bool) {
        let s = self.app_settings();
        (s.key_haptic, s.key_click)
    }

    /// 标题态（触点只走轮播手势；场景活动时恒 false）。
    pub fn title_mode(&self) -> bool {
        self.scene.is_none() && self.machine.title_geom().is_some()
    }

    /// 进程退出请求（BUG-033 消费面；win 壳 `finished()` 同口径）：① 开场链 BootMachine 置 `finished`
    /// = CMidlet.dispatchModeEntry 退出信号（菜单"退出" → mode 22 退场动画 70 拍走完，paint.rs）；② 壳层返回
    /// 立即退出（BUG-034，[`Runtime::back_requests_exit`]）。粘滞；场景活动时 ① 恒 false。
    pub fn exit_requested(&self) -> bool {
        self.shell_exit || (self.scene.is_none() && self.machine.finished)
    }

    /// 【BUG-034 · 壳层输入映射】返回（BACK 键 / 边缘返回手势）在**根界面**——开机 logo（mode 0）、主菜单
    /// （mode 1）、退场动画（mode 22）——= 立即退出：置退出请求、返回 true（调用方丢弃该 BACK 的 -7）。
    /// 语义 = Nokia 红键（AMS 随时销毁 MIDlet）：原版退出链 `CMidlet.m_000` = `destroyApp(true)`（空方法）+
    /// `notifyDestroyed()`，无存档副作用 ⇒ 跳过 70 拍 logo 退场动画不丢任何状态。BUG-033 版"选中退出项 + 确认"
    /// 走 mode 22 logo 动画，用户实机反馈形似"进程内重置、重播片头"且动画期返回无效——已弃。
    /// 其余态返回 false（BACK 照常 = 右软键 -7：子页返回主菜单 / 序章跳过 / 游戏内菜单）。
    pub fn back_requests_exit(&mut self) -> bool {
        let root = self.scene.is_none()
            && matches!(self.machine.mode, GameMode::Boot | GameMode::MainMenu | GameMode::LogoAnim);
        if root {
            self.shell_exit = true;
        }
        root
    }

    /// 【STAGE-B-005 · 壳层·系统对话框】游戏内返回（BACK 键 / 边缘返回手势）：场景活动 ∧ 地图行走态 ∧ 原版浮层未开
    /// ∧ 非换层遮幅 ∧ 无未决对话框 ⇒ 置未决、返回 true（调用方弹 Android 标准退出确认框并丢弃该 BACK 的 -7）。
    /// 其余 ⇒ false（BACK 照旧 -7：浮层关/拒、菜单返回、对话推进…）。面板右软键不经此口（地图上照旧开游戏内菜单）。
    /// 游戏核心零改动：未决期 [`Runtime::step`] 不出拍（暂停），结果经 [`Runtime::on_exit_dialog_result`] 回灌。
    pub fn back_requests_exit_dialog(&mut self) -> bool {
        if self.exit_dialog_pending {
            return false;
        }
        let ok = matches!(
            self.scene.as_ref(),
            Some(sc) if sc.game_mode == GameMode::Map && !sc.overlay_active && !sc.transitioning
        );
        self.exit_dialog_pending = ok;
        ok
    }

    /// 【STAGE-B-005】系统退出对话框是否未决。
    pub fn exit_dialog_pending(&self) -> bool {
        self.exit_dialog_pending
    }

    /// 【STAGE-B-005】对话框结果回灌：`exit` = 选"退出" ⇒ 进程退出请求（既有退出链）；否则（取消 / 返回 / 点框外 /
    /// 弹出失败）⇒ 结束未决、游戏原样继续。
    pub fn on_exit_dialog_result(&mut self, exit: bool) {
        self.exit_dialog_pending = false;
        if exit {
            self.shell_exit = true;
        }
    }

    /// 逻辑拍间隔（mode 0=100 / 其余 75；场景恒 75——win 壳 frame_interval 同源）。
    pub fn frame_interval_ms(&self) -> u64 {
        if self.scene.is_some() {
            75
        } else {
            self.machine.frame_interval_ms.max(1) as u64
        }
    }

    /// 活动机 BGM 请求（两机同构 bgm_request）。
    pub fn bgm_request(&self) -> Option<usize> {
        match self.scene.as_ref() {
            Some(sc) => sc.bgm_request(),
            None => self.machine.bgm_request(),
        }
    }

    /// 布局换轨（窗口重建；两机同口径 apply_layout——旧单档口径）。
    pub fn apply_layout(&mut self, layout: game_core::layout::Layout) {
        self.apply_layouts(layout.clone(), layout);
    }

    /// hideNotify 边沿（win 壳 hide_notify 同构）：游戏态=GameScene::hide_notify（mode3→4
    /// 弹菜单 + closeAudioPlayer + anyKeyPlayed）；Boot 态=bgm.on_hide_notify。随即同步 BGM
    /// （closeAudioPlayer ⇒ Stop 立发：切后台不等下一拍）。
    pub fn hide_notify(&mut self) {
        match self.scene.as_mut() {
            Some(sc) => sc.hide_notify(),
            None => self.machine.bgm.on_hide_notify(),
        }
        self.sync_bgm();
    }

    /// 叠加框态（框=当前态生效命中面）。
    pub fn overlay_state(&self) -> OverlayState {
        match self.scene.as_ref() {
            Some(sc) if sc.game_mode == GameMode::Map && !self.exit_dialog_pending => OverlayState::Game,
            Some(_) => OverlayState::Ui,
            None => match self.machine.title_geom() {
                Some(geo) => OverlayState::Title(geo),
                None => OverlayState::Ui,
            },
        }
    }

    /// 一拍。`keys`=本拍键动作批（到达序）；`gestures`=标题手势；`taps`=画布内点按
    /// （逻辑坐标）。`src`=资产源（首次交接时装载场景资产）。
    pub fn step(
        &mut self,
        src: &dyn game_platform::AssetSource,
        keys: &[KeyAction],
        gestures: Vec<TitleGesture>,
        taps: Vec<(f32, f32)>,
    ) {
        // 【STAGE-B-005】系统退出对话框未决 ⇒ 暂停：不出逻辑拍，本拍键/点按丢弃（系统对话框持焦点，输入本不进游戏）
        if self.exit_dialog_pending {
            return;
        }
        if self.scene.is_some() {
            self.step_scene(keys, taps);
        } else {
            self.step_boot(src, keys, gestures, taps);
        }
        self.frame_counter += 1;
        self.sync_bgm();
        self.persist_app_settings_if_changed(); // 【STAGE-B-006】设置页切换同拍落盘
    }

    /// 注入 BGM 命令出口（android_main = 音频线程通道；宿主测试 = 录制器）。翻译基线随之
    /// 归零（新出口从"未播放"起算）。
    pub fn set_bgm_sink(&mut self, sink: Box<dyn AudioSink>) {
        self.bgm = BgmController::new(sink);
    }

    /// 活动机 BGM 快照（场景活动 = GameScene，否则 = BootMachine；win 壳 bgm_snapshot 同源）。
    pub fn bgm_snapshot(&self) -> BgmSnapshot {
        match self.scene.as_ref() {
            Some(sc) => BgmSnapshot::of_scene(sc),
            None => BgmSnapshot::of_boot(&self.machine),
        }
    }

    /// 快照 → Play/Stop（只在变化时发；shell-common BgmController = win 壳 sync_bgm 同一翻译）。
    /// 每拍末 + hideNotify 后调用（切后台即停，不等下一拍）。
    fn sync_bgm(&mut self) {
        let snap = self.bgm_snapshot();
        self.bgm.update(snap);
    }

    /// Boot 拍：拍前解析标题手势 / UI 行点按 → 锁存键 → tick → SKY_WAR 写穿 → 交接
    /// （读条完成 mode=Map 且有交接信号 ⇒ GameScene 接管；win 壳 step Boot 臂同构）。
    fn step_boot(
        &mut self,
        src: &dyn game_platform::AssetSource,
        keys: &[KeyAction],
        gestures: Vec<TitleGesture>,
        taps: Vec<(f32, f32)>,
    ) {
        let mut batch = keys.to_vec();
        if !gestures.is_empty() {
            let k = resolve_title_gestures(&self.machine, gestures.clone());
            log_input(&format!("title gestures={gestures:?} → {k:?}"));
            batch.extend(k);
        }
        for (tx_, ty_) in taps {
            let hit = self.machine.pointer_hit(tx_ as i32, ty_ as i32);
            log_input(&format!("pointer_hit({tx_:.0},{ty_:.0})→{hit:?}"));
            match hit {
                game_core::scene::PointerHit::Row { index, confirm_key } => {
                    self.machine.select_row(index);
                    batch.push(KeyAction::Press(confirm_key));
                }
                // 点空白：序章快进 / 翻页代按 -5；读档列表、设置页不代按（BUG-039）——
                // 标题态不经此（轮播手势独立通道）
                _ => {
                    if let Some(k) = self.machine.blank_tap_key() {
                        batch.push(KeyAction::Press(k));
                    }
                }
            }
        }
        let key = boot_key_from_batch(&batch);
        if !batch.is_empty() {
            log_input(&format!("tick batch={batch:?} → key={key}"));
        }
        // 读档列表同步真源（win 壳 step 同口径：会话内保存后返回主菜单即可见）
        self.machine.preset_stores = self.stores.clone();
        self.machine.tick(key);
        // 【BUG-035】菜单"退出"确认 ⇒ 游戏进 mode 22（原版退出链入口：closeAudioPlayer + 70 拍 logo 退场动画后
        // CMidlet.m_000）⇒ 壳即刻退出，不播动画——与根界面返回同口径（BUG-034）。用户实机：动画 5.3 s 与开机同款
        // logo（"重播片头"），用户 4.25 s 时离开并在最近任务划掉，退出链未走完。m_000 = destroyApp（空）+
        // notifyDestroyed，无副作用。
        if self.machine.mode == GameMode::LogoAnim && !self.shell_exit {
            self.shell_exit = true;
            log_input("menu exit → 立即退出（不播 mode 22 退场动画）");
        }
        if self.machine.take_global_dirty() {
            handoff::put_global(&mut self.stores, &self.machine.global_snapshot());
            self.storage.persist(&self.stores);
        }
        // MOT_IF 读失败回写（BUG-021 ②c；win 壳 persist_slot_if_if_dirty 同口径）
        if let Some(slot) = self.machine.slots.as_mut().and_then(|s| s.take_parse_dirty().then(|| s.clone())) {
            handoff::put_slot_if(&mut self.stores, &slot);
            self.storage.persist(&self.stores);
        }
        if self.machine.mode != GameMode::Map || !(self.machine.new_game_ready || self.machine.slot_loaded.is_some()) {
            return;
        }
        if self.game_assets.is_none() {
            let t = std::time::Instant::now();
            self.game_assets = Some(handoff::load_game_assets(src));
            log_input(&format!("game assets loaded in {}ms", t.elapsed().as_millis()));
        }
        let assets = self.game_assets.as_ref().unwrap();
        let sc = match handoff::slot_scene(assets, &mut self.machine, &self.stores) {
            Some(sc) => Some(sc),
            None if self.machine.new_game_ready => Some(handoff::new_game_scene(assets, &mut self.machine)),
            None => None,
        };
        if let Some(mut sc) = sc {
            // win 壳 enter_scene 同口径：布局经 apply_layout 换轨 + SKY_WAR 交接拷贝
            // 【竖屏】游戏场景用 scene 档布局（可与开场链不同档，同一物理视口）
            sc.apply_layout(self.scene_layout.clone());
            sc.apply_global(self.machine.global_snapshot());
            self.persisted_blobs = sc.saved_slot_blobs.clone();
            log_input(&format!("handoff → GameScene floor={} mode={:?}", sc.floor, sc.game_mode));
            self.scene = Some(Box::new(sc));
        }
    }

    /// 场景拍（win 壳 step Game 臂同构）：点按/键按到达序投递 → 存档写穿 → tick
    /// （panic 捕获=停拍留帧）→ SKY_WAR 写穿 → kind7 退出换回标题。
    fn step_scene(&mut self, keys: &[KeyAction], taps: Vec<(f32, f32)>) {
        let Some(sc) = self.scene.as_mut() else { return };
        let mut batch = keys.to_vec();
        for (tx_, ty_) in taps {
            let k = scene_tap(sc, tx_ as i32, ty_ as i32);
            log_input(&format!("scene_tap({tx_:.0},{ty_:.0}) mode={:?} → {k:?}", sc.game_mode));
            if let Some(c) = k {
                batch.push(KeyAction::Press(c));
                batch.push(KeyAction::Release);
            }
        }
        if !batch.is_empty() {
            log_input(&format!("scene keys={batch:?} mode={:?}", sc.game_mode));
        }
        replay_scene_keys(sc, &batch);
        if handoff::sync_scene_saves(sc, &mut self.stores, &mut self.persisted_blobs) {
            self.storage.persist(&self.stores);
            log_input("slot save persisted");
        }
        if !self.crashed {
            let fc = self.frame_counter as i64;
            let wt = &self.game_assets.as_ref().unwrap().width_table;
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sc.tick(fc, wt)));
            if let Err(e) = r {
                let msg = e
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| e.downcast_ref::<&str>().copied())
                    .unwrap_or("");
                log_input(&format!("GAME-CRASH: {msg}"));
                self.crashed = true;
            }
        }
        if sc.take_global_dirty() {
            handoff::put_global(&mut self.stores, &sc.global_snapshot());
            self.storage.persist(&self.stores);
        }
        // MOT_IF 读失败回写（BUG-021 ②c）+ Game 侧重读源同步（BUG-018 环）
        if sc.slot.take_parse_dirty() {
            let slot = sc.slot.clone();
            sc.preset_mot_if = Some(handoff::put_slot_if(&mut self.stores, &slot));
            self.storage.persist(&self.stores);
        }
        if sc.game_mode == GameMode::MainMenu {
            // 【BUG-041】反向交接：游戏内改过的声音 / SKY_WAR 回传（否则换回交接前旧值）
            handoff::return_to_title(&mut self.machine, sc);
            self.machine.mode = GameMode::MainMenu;
            // dispatchModeEntry case 1 音频半（a.java:4295/4311）：f_bool_28=false + playBgm(2)
            self.machine.bgm.enter_title();
            self.machine.preset_stores = self.stores.clone();
            self.scene = None;
            log_input("scene → title（kind7 退出换回 BootMachine）");
        }
    }

    /// 活动机绘制。
    pub fn paint(&mut self, g: &mut SoftGraphics<'_>) {
        match self.scene.as_mut() {
            Some(sc) => sc.paint(g, &self.game_assets.as_ref().unwrap().tileset),
            None => self.machine.paint(g),
        }
    }
}
