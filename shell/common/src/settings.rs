//! STAGE-B-001（2026-10-18 用户裁定）：声音开关=全局应用设置，**不入游戏存档**。
//! 键值对文本（sound_enabled / volume_level），落 save 目录旁 `app_settings.cfg`。
//! shell 装载→应用到 game-core BgmState；toggle 变更由 shell 检测落盘（game-core 零 I/O）。
//! 【STAGE-B-006】新增 show_splash / key_haptic / key_click（Android 设置页"开屏画面 / 按键震动 / 按键音"）；
//! 布尔值只认 `true` / `false`，坏值与缺键取默认。

/// 【STAGE-B-008 计划 3 T3 · spec §8】操控方式（摇杆 / 触屏，默认摇杆——仅 Android 壳消费，
/// Windows 不读；设置行增减由 core `replace_shell_rows` 完成，壳只从行读结果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Pad,
    Touch,
}

/// 应用全局设置（声音对 + STAGE-B-006 三项 + STAGE-B-008 计划 3 四项）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppSettings {
    pub sound_enabled: bool,
    pub volume_level: i32,
    /// 【STAGE-B-006】开屏画面（mode 0 三屏）：默认关 = 启动第一帧即主菜单。
    pub show_splash: bool,
    /// 【STAGE-B-006】控制面板虚拟键按下的触感：默认开。
    pub key_haptic: bool,
    /// 【STAGE-B-006】控制面板虚拟键按下的按键音（与"声音"互不影响）：默认开。
    pub key_click: bool,
    /// 【STAGE-B-008 计划 3】操控方式：默认摇杆（spec §8）。
    pub control_mode: ControlMode,
    /// 【STAGE-B-008 计划 3】自动战斗（仅触屏模式有设置行；core 字段同步）：默认关。
    pub auto_battle: bool,
    /// 【STAGE-B-008 计划 3】各模式提示卡已看过（纯 Runtime 字段，弹出即置位落盘）：默认 false。
    pub tips_seen_pad: bool,
    pub tips_seen_touch: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        // STAGE-B-001 偏离点 2：默认开声（原版经询问每次重决）；STAGE-B-006：开屏默认关、按键震动 / 按键音默认开
        AppSettings {
            sound_enabled: true,
            volume_level: 60,
            show_splash: false,
            key_haptic: true,
            key_click: true,
            control_mode: ControlMode::Pad,
            auto_battle: false,
            tips_seen_pad: false,
            tips_seen_touch: false,
        }
    }
}

/// 布尔值：只认小写 `true` / `false`（两侧空白容忍；程序只写这两种），其余（含大小写变体）视为坏值、该键不变。
fn set_bool(slot: &mut bool, v: &str) {
    match v.trim() {
        "true" => *slot = true,
        "false" => *slot = false,
        _ => {}
    }
}

impl AppSettings {
    /// 装载（缺文件/坏行/坏值 ⇒ 默认——设置容忍损坏，不阻启动）。
    pub fn load(path: &std::path::Path) -> AppSettings {
        let Ok(text) = std::fs::read_to_string(path) else {
            return AppSettings::default();
        };
        let mut s = AppSettings::default();
        for line in text.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("sound_enabled=") {
                set_bool(&mut s.sound_enabled, v);
            } else if let Some(v) = line.strip_prefix("volume_level=") {
                if let Ok(n) = v.trim().parse() {
                    s.volume_level = n;
                }
            } else if let Some(v) = line.strip_prefix("show_splash=") {
                set_bool(&mut s.show_splash, v);
            } else if let Some(v) = line.strip_prefix("key_haptic=") {
                set_bool(&mut s.key_haptic, v);
            } else if let Some(v) = line.strip_prefix("key_click=") {
                set_bool(&mut s.key_click, v);
            } else if let Some(v) = line.strip_prefix("control_mode=") {
                match v.trim() {
                    "pad" => s.control_mode = ControlMode::Pad,
                    "touch" => s.control_mode = ControlMode::Touch,
                    _ => {} // 坏值回默认（pad）
                }
            } else if let Some(v) = line.strip_prefix("auto_battle=") {
                set_bool(&mut s.auto_battle, v);
            } else if let Some(v) = line.strip_prefix("tips_seen_pad=") {
                set_bool(&mut s.tips_seen_pad, v);
            } else if let Some(v) = line.strip_prefix("tips_seen_touch=") {
                set_bool(&mut s.tips_seen_touch, v);
            }
        }
        s
    }

    /// 落盘（原子写：tmp+rename；失败吞（设置丢失可重设，不阻游戏））。写全部键。
    pub fn save(&self, path: &std::path::Path) {
        let control_mode = match self.control_mode {
            ControlMode::Pad => "pad",
            ControlMode::Touch => "touch",
        };
        let text = format!(
            "sound_enabled={}\nvolume_level={}\nshow_splash={}\nkey_haptic={}\nkey_click={}\n\
             control_mode={}\nauto_battle={}\ntips_seen_pad={}\ntips_seen_touch={}\n",
            self.sound_enabled,
            self.volume_level,
            self.show_splash,
            self.key_haptic,
            self.key_click,
            control_mode,
            self.auto_battle,
            self.tips_seen_pad,
            self.tips_seen_touch
        );
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }
}

#[cfg(test)]
mod settings_tests {
    use super::AppSettings;

    /// STAGE-B-001 TDD：读写 round-trip + 损坏容忍 + 默认开声。
    #[test]
    fn settings_roundtrip_and_defaults() {
        let dir = std::env::temp_dir().join(format!("xiyou-settings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app_settings.cfg");
        let _ = std::fs::remove_file(&p);
        // 默认开声
        assert!(AppSettings::default().sound_enabled, "STAGE-B-001：默认开声");
        // 缺文件=默认
        assert_eq!(AppSettings::load(&p), AppSettings::default());
        // round-trip
        let s = AppSettings { sound_enabled: false, volume_level: 30, ..AppSettings::default() };
        s.save(&p);
        assert_eq!(AppSettings::load(&p), s, "round-trip");
        // 损坏行容忍
        std::fs::write(&p, "garbage\nsound_enabled=true\nvolume_level=abc\n").unwrap();
        let s2 = AppSettings::load(&p);
        assert!(s2.sound_enabled);
        assert_eq!(s2.volume_level, 60, "坏值回默认");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 【STAGE-B-006】三个新键：默认（开屏关 / 震动开 / 按键音开）、往返、旧版本文件缺键取默认、布尔坏值取默认
    /// （复核 L1：旧写法 `== "true"` 把坏值读成 false；声音行同改）。
    #[test]
    fn stage_b006_keys_defaults_roundtrip_and_bad_values() {
        let dir = std::env::temp_dir().join(format!("xiyou-settings-b006-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app_settings.cfg");
        let d = AppSettings::default();
        assert_eq!((d.show_splash, d.key_haptic, d.key_click), (false, true, true), "默认：开屏关、震动开、按键音开");
        // 往返：四个布尔的 16 种组合逐一（各键独立，防写反 / 接错槽，复核 T5-F1）
        for bits in 0..16u8 {
            let s = AppSettings {
                sound_enabled: bits & 1 != 0,
                volume_level: bits as i32,
                show_splash: bits & 2 != 0,
                key_haptic: bits & 4 != 0,
                key_click: bits & 8 != 0,
                ..AppSettings::default()
            };
            s.save(&p);
            assert_eq!(AppSettings::load(&p), s, "五键往返 bits={bits:04b}");
        }
        std::fs::write(&p, "sound_enabled=false\nvolume_level=0\n").unwrap();
        assert_eq!(
            AppSettings::load(&p),
            AppSettings { sound_enabled: false, volume_level: 0, ..AppSettings::default() },
            "旧版本文件（两键）：新键取默认"
        );
        // 坏值：只认小写 true / false（程序只写这两种）；大小写变体、数字、yes/no、空值都按坏值取默认（复核 T5-F3）
        for bad in ["off", "yes", "0", "1", "no", "", "True", "FALSE"] {
            std::fs::write(&p, format!("sound_enabled={bad}\nshow_splash={bad}\nkey_haptic={bad}\nkey_click={bad}\n")).unwrap();
            assert_eq!(AppSettings::load(&p), AppSettings::default(), "坏值 {bad:?} 一律取默认");
        }
        std::fs::write(&p, "show_splash= true \nkey_click=false\n").unwrap();
        let t = AppSettings::load(&p);
        assert!(t.show_splash && !t.key_click && t.key_haptic, "值两侧空白容忍（与 volume_level 同口径）");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 【STAGE-B-008 计划 3 T3 · spec §8】四个新键：默认（摇杆 / 自动战斗关 / 两提示卡未看）、
    /// control_mode 只认 pad/touch（坏值回默认）、旧文件缺键取默认、九键全翻转往返。
    #[test]
    fn plan3_keys_defaults_roundtrip_and_bad_values() {
        use super::ControlMode;
        let dir = std::env::temp_dir().join(format!("xiyou-settings-plan3-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app_settings.cfg");
        let d = AppSettings::default();
        assert_eq!(
            (d.control_mode, d.auto_battle, d.tips_seen_pad, d.tips_seen_touch),
            (ControlMode::Pad, false, false, false),
            "默认：摇杆、自动战斗关、两提示卡未看"
        );
        let s = AppSettings {
            sound_enabled: false,
            volume_level: 1,
            show_splash: true,
            key_haptic: false,
            key_click: true,
            control_mode: ControlMode::Touch,
            auto_battle: true,
            tips_seen_pad: true,
            tips_seen_touch: true,
        };
        s.save(&p);
        assert_eq!(AppSettings::load(&p), s, "九键往返");
        for bad in ["PAD", "Touch", "joystick", "", "1"] {
            std::fs::write(&p, format!("control_mode={bad}\n")).unwrap();
            assert_eq!(AppSettings::load(&p).control_mode, ControlMode::Pad, "坏值 {bad:?} 回默认");
        }
        std::fs::write(&p, "sound_enabled=true\nvolume_level=50\nshow_splash=false\nkey_haptic=true\nkey_click=true\n").unwrap();
        assert_eq!(
            AppSettings::load(&p),
            AppSettings { volume_level: 50, ..AppSettings::default() },
            "旧文件：新键取默认"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
