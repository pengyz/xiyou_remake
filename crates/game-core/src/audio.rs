//! BGM 选轨/启停语义（MMAPI 播放器状态的确定性投影）。规格：docs/spec/audio.md。
//!
//! Java 侧只有**一个**播放入口 `m_139(track)`（a.java:10158-10206）与一个关闭入口
//! `closeAudioPlayer()`（a.java:10208-10215），播放器是 static 单例 `f_Player_00`
//! （a.java:401）。本模块把它们与各调用点的**音频半**忠实搬为纯状态机（无 I/O）：
//! 宿主（桌面壳）只读 [`BgmState::request`] / [`BgmState::serial`] /
//! [`BgmState::volume`] 去驱动真实输出，游戏逻辑不感知平台。
//!
//! 调用点全集（javap renamed a.class 字节码偏移，A 级；`iconst_2/iconst_3` 为轨道实参）：
//!
//! | 入口 | 偏移 | 本模块 |
//! |---|---|---|
//! | run case 0 末（mode 0→21） | run@1291 closeAudioPlayer | [`BgmState::close`] |
//! | m_000 case 21 | a.java:4484 `soundEnabled=false` | [`BgmState::enter_sound_prompt`] |
//! | run case 21 -7/-6 | a.java:4231-4241 | [`BgmState::answer_sound_prompt`] |
//! | m_000 case 1 | m_000@412 m_139(2) | [`BgmState::enter_title`] |
//! | m_000 case 3 | m_000@562 m_139(3) | [`BgmState::enter_game`] |
//! | run case 16 label 0 | run@6273/6283 m_139(2/3)、run@6291 close | [`BgmState::toggle_sound`] |
//! | m_000 case 22 | m_000@1021 closeAudioPlayer | [`BgmState::close`] |
//! | hideNotify | hideNotify@26 closeAudioPlayer | [`BgmState::on_hide_notify`] |
//! | keyPressed | keyPressed@43/@52 m_139(2/3) | [`BgmState::on_key_pressed`] |
//!
//! 轨道 0/1（logostart/logoquit）**无任何调用点**（全部 6 处 m_139 实参只有 2/3）——
//! 死资源，见 BUG-010。

/// 轨道表 `f_String_arr_15`（a.java:2239）。下标即 [`BgmState::request`] 的值。
/// BUG-010：下标 0/1 无调用点（死资源），保真保留表项。
pub const TRACKS: [&str; 4] = ["logostart.mid", "logoquit.mid", "menu.mid", "game.mid"];
/// 标题/菜单曲（m_000 case 1 a.java:4311）。
pub const TRACK_MENU: u8 = 2;
/// 游戏曲（m_000 case 3 a.java:4321）。
pub const TRACK_GAME: u8 = 3;
/// volumeLevel 初值与「开声」复位值（a.java:2240 / 4159 / 4241）。
pub const DEFAULT_VOLUME: i32 = 60;

/// 播放器实例（`f_Player_00 != null`）。Java 侧每条到达 `m_139` 末尾的路径都以
/// `start()` 收尾，`closeAudioPlayer` 同步置 null ⇒ 存活实例恒为 STARTED(400)；
/// 状态机其余态（UNREALIZED/REALIZED/PREFETCHED 停驻/CLOSED 未置 null）在本游戏
/// 不可达（B 级静态；A 级旁证：`reference/oracle/_out/` 全部 trace 共 109 次
/// createPlayer / 54 次 close / **0 次 stop**，见 docs/spec/audio.md §3）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    /// 创建时的轨道下标（= 创建成功后写入的 f_byte_28，a.java:10199）。
    pub track: u8,
    /// 创建时下发的 VolumeControl 级别（a.java:10189-10194：prefetch 后 state=300
    /// 满足 `(state&300)==300` ⇒ setLevel(volumeLevel)）。同轨续播臂不重设音量。
    pub volume: i32,
    /// 第几个被创建的播放器（壳侧据此区分「续播」与「重建＝从头播放」）。
    pub serial: u32,
}

/// 音频语义状态（Java 字段一一对应 + 宿主观测用的 serial）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BgmState {
    /// soundEnabled（a.java:399，初值 true a.java:2241）。
    pub sound_enabled: bool,
    /// volumeLevel（a.java:398，初值 60 a.java:2240）。
    pub volume_level: i32,
    /// f_byte_28（a.java:400，初值 -1 a.java:2242）：当前播放器的轨道，-1 = 无。
    pub current_track: i8,
    /// f_bool_28（a.java:396）：m_000 case 1 置 false（a.java:4295）、case 3 置 true
    /// （a.java:4316）；决定「恢复播放」选 menu(2) 还是 game(3)。
    pub in_game: bool,
    /// anyKeyPlayed（a.java:40）：hideNotify 置 true（a.java:4510），下一次
    /// keyPressed 续播后清零（a.java:4674）。
    pub any_key_played: bool,
    /// f_Player_00（a.java:401，static 单例）。
    player: Option<Player>,
    /// createPlayer 计数（非 Java 字段：宿主区分续播/重建）。
    created: u32,
}

impl Default for BgmState {
    fn default() -> Self {
        Self::new()
    }
}

impl BgmState {
    /// 构造期初值（a.java:2240-2242；f_bool_28/anyKeyPlayed 为 Java 默认 false，
    /// f_Player_00 为 null）。
    pub fn new() -> BgmState {
        BgmState {
            sound_enabled: true,
            volume_level: DEFAULT_VOLUME,
            current_track: -1,
            in_game: false,
            any_key_played: false,
            player: None,
            created: 0,
        }
    }

    /// 「刚进入 mode 3」的状态：初值 + m_000 case 3（声音开）。供脱离 BootMachine
    /// 独立构造的 GameScene 作默认值；桌面壳交接时以 BootMachine 的状态整体覆盖。
    pub fn in_game_default() -> BgmState {
        let mut b = BgmState::new();
        b.enter_game();
        b
    }

    /// 宿主契约：应当在放的轨道（[`TRACKS`] 下标 0..=3），None = 停止/无声。
    pub fn request(&self) -> Option<usize> {
        self.player.as_ref().map(|p| p.track as usize)
    }

    /// 当前播放器的 VolumeControl 级别（0..=100；None = 无播放器）。
    pub fn volume(&self) -> Option<i32> {
        self.player.as_ref().map(|p| p.volume)
    }

    /// 当前播放器序号（0 = 尚无）。同一轨道 serial 变化 ⇒ Java 侧关闭重建 ⇒ 从头播放。
    pub fn serial(&self) -> u32 {
        self.player.as_ref().map_or(0, |p| p.serial)
    }

    /// 只读：播放器实例。
    pub fn player(&self) -> Option<&Player> {
        self.player.as_ref()
    }

    /// `f_bool_28 ? 3 : 2`——三处恢复播放共用的选轨（a.java:4160-4163 / 4667-4670）。
    pub fn resume_track(&self) -> u8 {
        if self.in_game {
            TRACK_GAME
        } else {
            TRACK_MENU
        }
    }

    /// `m_139(track, -1)`（a.java:10158-10206）。
    ///
    /// - soundEnabled=false ⇒ 整体跳过（a.java:10159，**连 close 都不做**）；
    /// - 同轨且播放器在 ⇒ 续播臂：STARTED 先 stop() 再落穿 PREFETCHED 的
    ///   setLoopCount(-1)+start() 后 return（a.java:10163-10174）——MMAPI stop/start
    ///   保持媒体位置 ⇒ 不重建、不重设音量、serial 不变；
    /// - 否则 closeAudioPlayer → createPlayer/realize/prefetch → setLoopCount(-1)
    ///   （无限循环）→ setLevel(volumeLevel) → start → f_byte_28=track
    ///   （a.java:10178-10199）。
    ///
    /// 未建模：createPlayer/realize 抛异常的 catch 臂（a.java:10200，平台相关；
    /// 壳侧缺音频产物时静默降级，不回写游戏状态）。
    pub fn play(&mut self, track: u8) {
        if !self.sound_enabled {
            return;
        }
        if self.current_track == track as i8 && self.player.is_some() {
            return; // 续播臂（stop→start，位置不变）
        }
        self.close();
        self.created = self.created.wrapping_add(1);
        self.player = Some(Player {
            track,
            volume: self.volume_level,
            serial: self.created,
        });
        self.current_track = track as i8;
    }

    /// `closeAudioPlayer()`（a.java:10208-10215）：close → null → f_byte_28=-1。
    /// **不受 soundEnabled 门控**。
    pub fn close(&mut self) {
        self.player = None;
        self.current_track = -1;
    }

    /// m_000 case 21 的音频半（a.java:4484）：`soundEnabled=false`（询问期静音；
    /// 前序 run case 0 末已 closeAudioPlayer，a.java:3376）。
    pub fn enter_sound_prompt(&mut self) {
        self.sound_enabled = false;
    }

    /// run case 21 的音频半（a.java:4231-4241）。`enable`：-6=true / -7=false。
    /// 随后调用方切 gameMode=1 + m_000 ⇒ 须接 [`Self::enter_title`]。
    pub fn answer_sound_prompt(&mut self, enable: bool) {
        if enable {
            self.sound_enabled = true;
            if self.volume_level == 0 {
                self.volume_level = DEFAULT_VOLUME;
            }
        } else {
            self.sound_enabled = false;
            self.volume_level = 0;
        }
    }

    /// m_000 case 1 的音频半：`f_bool_28=false`（a.java:4295）+ `m_139(2)`（a.java:4311）。
    pub fn enter_title(&mut self) {
        self.in_game = false;
        self.play(TRACK_MENU);
    }

    /// m_000 case 3 的音频半：`f_bool_28=true`（a.java:4316）+ `m_139(3)`（a.java:4321）。
    pub fn enter_game(&mut self) {
        self.in_game = true;
        self.play(TRACK_GAME);
    }

    /// run case 16 label 0（a.java:4157-4167）：翻转 soundEnabled；开 ⇒ volumeLevel=60
    /// + m_139(f_bool_28?3:2)；关 ⇒ closeAudioPlayer。（其后 saveGlobalProgress 为
    /// RMS 写，非音频，a.java:4169。）
    pub fn toggle_sound(&mut self) {
        self.sound_enabled = !self.sound_enabled;
        if self.sound_enabled {
            self.volume_level = DEFAULT_VOLUME;
            let t = self.resume_track();
            self.play(t);
        } else {
            self.close();
        }
    }

    /// hideNotify 的音频半（a.java:4509-4510）：closeAudioPlayer + anyKeyPlayed=true。
    /// （同方法的非音频臂「gameMode==3 → 4 + m_000」不在本模块；hideNotify 整体
    /// 尚未接入任何宿主路径，见 docs/spec/audio.md §4。）
    pub fn on_hide_notify(&mut self) {
        self.close();
        self.any_key_played = true;
    }

    /// keyPressed 的音频半（a.java:4665-4674）：anyKeyPlayed 时，gameMode ∉ {21, 0}
    /// 才恢复播放 m_139(f_bool_28?3:2)；无论是否恢复都清 anyKeyPlayed。
    pub fn on_key_pressed(&mut self, game_mode: i32) {
        if self.any_key_played {
            if game_mode != 21 && game_mode != 0 {
                let t = self.resume_track();
                self.play(t);
            }
            self.any_key_played = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造期 = 无播放器（a.java:2240-2242）。
    #[test]
    fn initial_state_is_silent() {
        let b = BgmState::new();
        assert_eq!(b.request(), None);
        assert_eq!((b.sound_enabled, b.volume_level, b.current_track), (true, 60, -1));
        assert_eq!(b.serial(), 0);
        assert_eq!(TRACKS[b.resume_track() as usize], "menu.mid");
    }

    /// 启动链：mode0 末 close → m_000 case21 静音 → -6 开声 → m_000 case1 = menu(2)
    /// （EV-ending-title T71：createPlayer + setLevel(60)）。
    #[test]
    fn prompt_yes_plays_menu_track() {
        let mut b = BgmState::new();
        b.close();
        b.enter_sound_prompt();
        assert_eq!(b.request(), None);
        b.answer_sound_prompt(true);
        b.enter_title();
        assert_eq!(b.request(), Some(2));
        assert_eq!(b.volume(), Some(60));
        assert_eq!((b.current_track, b.in_game, b.serial()), (2, false, 1));
    }

    /// -7 关声：volumeLevel=0、m_000 case1 的 m_139 被门控（EV-toggle-exit T71：
    /// 无 createPlayer 事件）。
    #[test]
    fn prompt_no_stays_silent() {
        let mut b = BgmState::new();
        b.enter_sound_prompt();
        b.answer_sound_prompt(false);
        b.enter_title();
        assert_eq!(b.request(), None);
        assert_eq!((b.sound_enabled, b.volume_level, b.in_game), (false, 0, false));
        assert_eq!(b.serial(), 0);
    }

    /// 标题设置开关（EV-toggle-exit T161/T201/T241）：开→menu(2) 音量复位 60；
    /// 关→close；再开→重建（serial 递增 = 从头播放）。
    #[test]
    fn title_toggle_rebuilds_menu_track() {
        let mut b = BgmState::new();
        b.enter_sound_prompt();
        b.answer_sound_prompt(false);
        b.enter_title();
        b.toggle_sound();
        assert_eq!((b.request(), b.volume(), b.serial()), (Some(2), Some(60), 1));
        b.toggle_sound();
        assert_eq!((b.request(), b.current_track), (None, -1));
        b.toggle_sound();
        assert_eq!((b.request(), b.serial()), (Some(2), 2));
    }

    /// 进游戏 m_000 case3：menu→game 关闭重建（A-enter-game T417 / EV T397：
    /// close(state=400) + createPlayer）。
    #[test]
    fn enter_game_switches_to_game_track() {
        let mut b = BgmState::new();
        b.enter_sound_prompt();
        b.answer_sound_prompt(true);
        b.enter_title();
        b.enter_game();
        assert_eq!((b.request(), b.in_game, b.serial()), (Some(3), true, 2));
    }

    /// 游戏内设置开关：f_bool_28=true ⇒ 恢复 game(3)（EV-toggle-exit T521/T561）。
    #[test]
    fn in_game_toggle_resumes_game_track() {
        let mut b = BgmState::in_game_default();
        assert_eq!(b.request(), Some(3));
        b.toggle_sound();
        assert_eq!(b.request(), None);
        b.toggle_sound();
        assert_eq!((b.request(), b.volume()), (Some(3), Some(60)));
    }

    /// 退回标题 m_000 case1（结局任意键 EV-ending-title T1301 / 菜单退出 EV-toggle-exit
    /// T651）：game→menu 重建，f_bool_28 复位。
    #[test]
    fn back_to_title_switches_to_menu_track() {
        let mut b = BgmState::in_game_default();
        let s = b.serial();
        b.enter_title();
        assert_eq!((b.request(), b.in_game), (Some(2), false));
        assert_eq!(b.serial(), s + 1);
    }

    /// 同轨续播臂：不重建、不重设音量（a.java:10163-10174）。
    #[test]
    fn same_track_play_continues_without_rebuild() {
        let mut b = BgmState::in_game_default();
        let s = b.serial();
        b.volume_level = 10; // 续播不下发 setLevel
        b.play(TRACK_GAME);
        assert_eq!((b.request(), b.serial(), b.volume()), (Some(3), s, Some(60)));
    }

    /// 门控：soundEnabled=false 时 m_139 连 close 都不做（a.java:10159）。
    #[test]
    fn play_is_fully_gated_by_sound_enabled() {
        let mut b = BgmState::in_game_default();
        b.sound_enabled = false; // 人为构造（游戏内不可达：关声必伴 close）
        b.play(TRACK_MENU);
        assert_eq!(b.request(), Some(3));
    }

    /// mode 22 入场 closeAudioPlayer（A-menu-sweep T1761 close(state=400)）。
    #[test]
    fn exit_mode_closes_player() {
        let mut b = BgmState::new();
        b.answer_sound_prompt(true);
        b.enter_title();
        b.close();
        assert_eq!((b.request(), b.current_track), (None, -1));
    }

    /// hideNotify → keyPressed 续播：mode 21/0 不恢复但仍清标志（a.java:4665-4674）。
    #[test]
    fn hide_notify_then_key_resumes_by_mode() {
        let mut b = BgmState::in_game_default();
        b.on_hide_notify();
        assert_eq!((b.request(), b.any_key_played), (None, true));
        b.on_key_pressed(21);
        assert_eq!((b.request(), b.any_key_played), (None, false));

        b.on_hide_notify();
        b.on_key_pressed(4);
        assert_eq!((b.request(), b.any_key_played), (Some(3), false));

        // 未 hide 时按键无音频副作用
        let s = b.serial();
        b.on_key_pressed(3);
        assert_eq!(b.serial(), s);
    }

    /// 不变量：任意转场序列下「有播放器 ⇒ soundEnabled」（关声路径必伴 close，
    /// 开声路径只经 m_139）。穷举长度 ≤5 的操作序列。
    #[test]
    fn player_implies_sound_enabled_over_all_short_sequences() {
        fn apply(b: &mut BgmState, op: u8) {
            match op {
                0 => b.enter_title(),
                1 => b.enter_game(),
                2 => b.toggle_sound(),
                3 => b.close(),
                4 => b.on_hide_notify(),
                5 => b.on_key_pressed(3),
                _ => unreachable!(),
            }
        }
        for start_yes in [true, false] {
            for n in 0..=5u32 {
                for code in 0..6u32.pow(n) {
                    let mut b = BgmState::new();
                    b.close();
                    b.enter_sound_prompt();
                    b.answer_sound_prompt(start_yes);
                    b.enter_title();
                    let mut c = code;
                    for _ in 0..n {
                        apply(&mut b, (c % 6) as u8);
                        c /= 6;
                        if b.request().is_some() {
                            assert!(b.sound_enabled, "有播放器但 soundEnabled=false");
                        }
                        assert_eq!(b.request().is_some(), b.current_track >= 0);
                    }
                }
            }
        }
    }
}
