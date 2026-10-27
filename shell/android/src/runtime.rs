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

    /// 游戏内"声音"开关（活动机 `bgm.sound_enabled`；交接时随 bgm 续接，设置页翻转即变）——
    /// 壳侧按键音据此门控（feedback.rs）。只读，不碰游戏状态。
    pub fn sound_enabled(&self) -> bool {
        match self.scene.as_ref() {
            Some(sc) => sc.bgm.sound_enabled,
            None => self.machine.bgm.sound_enabled,
        }
    }

    /// 标题态（触点只走轮播手势；场景活动时恒 false）。
    pub fn title_mode(&self) -> bool {
        self.scene.is_none() && self.machine.title_geom().is_some()
    }

    /// 进程退出请求（BUG-033 消费面；win 壳 `finished()` 同口径）：开场链 BootMachine 置 `finished`
    /// = CMidlet.dispatchModeEntry 退出信号（mode 22 退场动画 70 拍走完，paint.rs）。粘滞；场景活动时恒 false。
    pub fn exit_requested(&self) -> bool {
        self.scene.is_none() && self.machine.finished
    }

    /// 【BUG-033 · 壳层输入映射】标题态返回（BACK 键 / 边缘返回手势）= 主菜单"退出"项（menuItemKinds 5）
    /// 选中 + 确认——走游戏自身退出链（mode 22 退场动画 → finished → 壳 finish + 结束进程），不另造退出路径；
    /// 游戏层零改动（同虚拟键盘：壳把平台输入译成既有按键）。返回投递键批；非标题态 = None（调用方保留
    /// BACK 原映射右软键 -7）。
    pub fn title_back_keys(&mut self) -> Option<Vec<KeyAction>> {
        if !self.title_mode() {
            return None;
        }
        let idx = self.machine.title.as_ref()?.menu_kinds.iter().position(|&k| k == 5)? as i32;
        self.machine.select_row(idx);
        Some(vec![KeyAction::Press(crate::input::KEY_FIRE), KeyAction::Release])
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
            Some(sc) if sc.game_mode == GameMode::Map => OverlayState::Game,
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
        if self.scene.is_some() {
            self.step_scene(keys, taps);
        } else {
            self.step_boot(src, keys, gestures, taps);
        }
        self.frame_counter += 1;
        self.sync_bgm();
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
                // 行外点按=确认 -5（win 壳 click_key 基线同语义）：序章快进/翻页、
                // 子页确认——标题态不经此（轮播手势独立通道）
                _ => batch.push(KeyAction::Press(-5)),
            }
        }
        let key = boot_key_from_batch(&batch);
        if !batch.is_empty() {
            log_input(&format!("tick batch={batch:?} → key={key}"));
        }
        // 读档列表同步真源（win 壳 step 同口径：会话内保存后返回主菜单即可见）
        self.machine.preset_stores = self.stores.clone();
        self.machine.tick(key);
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
