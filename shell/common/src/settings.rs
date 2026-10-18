//! STAGE-B-001（2026-10-18 用户裁定）：声音开关=全局应用设置，**不入游戏存档**。
//! 键值对文本（sound_enabled / volume_level），落 save 目录旁 `app_settings.cfg`。
//! shell 装载→应用到 game-core BgmState；toggle 变更由 shell 检测落盘（game-core 零 I/O）。

/// 应用全局设置（当前=声音对；后续可扩展语言/键位等）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppSettings {
    pub sound_enabled: bool,
    pub volume_level: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        // STAGE-B-001 偏离点 2：默认开声（原版经询问每次重决）
        AppSettings { sound_enabled: true, volume_level: 60 }
    }
}

impl AppSettings {
    /// 装载（缺文件/坏行 ⇒ 默认——设置容忍损坏，不阻启动）。
    pub fn load(path: &std::path::Path) -> AppSettings {
        let Ok(text) = std::fs::read_to_string(path) else {
            return AppSettings::default();
        };
        let mut s = AppSettings::default();
        for line in text.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("sound_enabled=") {
                s.sound_enabled = v.trim() == "true";
            } else if let Some(v) = line.strip_prefix("volume_level=") {
                if let Ok(n) = v.trim().parse() {
                    s.volume_level = n;
                }
            }
        }
        s
    }

    /// 落盘（原子写：tmp+rename；失败吞（设置丢失可重设，不阻游戏））。
    pub fn save(&self, path: &std::path::Path) {
        let text = format!("sound_enabled={}\nvolume_level={}\n", self.sound_enabled, self.volume_level);
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
        let s = AppSettings { sound_enabled: false, volume_level: 30 };
        s.save(&p);
        assert_eq!(AppSettings::load(&p), s, "round-trip");
        // 损坏行容忍
        std::fs::write(&p, "garbage\nsound_enabled=true\nvolume_level=abc\n").unwrap();
        let s2 = AppSettings::load(&p);
        assert!(s2.sound_enabled);
        assert_eq!(s2.volume_level, 60, "坏值回默认");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
