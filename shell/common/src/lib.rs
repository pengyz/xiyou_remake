//! game-shell-common —— 平台壳**共享实现**（lead F-4 裁决 2026-10-17）。
//!
//! 分层（AGENTS §2）：`game-platform` 只放 trait 与类型；平台相关实现各壳自持
//! （winmm/ALSA/AAudio、AAssetManager……）；**可跨壳复用**的实现收敛于本 crate，
//! 防双壳拷贝漂移（`docs/knowledge/pattern_dual-copy-algorithms-sync-fix.md`）。
//!
//! 范围：
//! - [`rms`]：RMS 文件语义核心（[`rms::FileRms`]，实现 `game_platform::Storage`）
//!   + MOT_IF 槽表序列化（[`rms::serialize_slot_if`]）。
//! - [`audio`]：BGM/PCM 共享面——纯字节、**零 `std::fs`/路径**（BgmController/Track/
//!   decode_ogg/Mixer/audio_loop，设备实现留各壳）。
//! - [`font`]：D2 字体缝（系统 TTF/TTC → game-core 字形钩子；候选路径由各壳给）。
//!
//! **已知第五缝不入本 crate**（docs/spec/platform-contract.md §0/§4.2，B2 登记）：
//! BGM 轨迹装载 `start()`/`track_path`/`REGEN_CMD` 是路径型装载（`--audio` 注入）=
//! Windows 壳专用；Android 轨道走 `AssetSource::read("track{i}.ogg")`。
//!
//! 平台差备注：存档文件名查找跟随宿主 FS 原生语义（Windows 不区分大小写 /
//! Android ext4 区分）——大小写杂散名的命中差异**不是 bug**（契约 §5 脚注；
//! 判据锚 `behavior_anchors::storage_lookup_follows_fs_case_rules_key_stays_canonical`）。

pub mod asset;
pub mod font;
pub mod handoff;
pub mod touch;
pub mod undo_ui;
pub mod audio;
pub mod controls_ui;
pub mod settings;
pub mod rms;

#[cfg(test)]
mod behavior_anchors {
    //! 行为锚（自 shell/win 迁入，断言零字节改动）：RMS 文件语义与资产读取锁现状，
    //! 迁移前后游戏行为零漂移的判据。纪律：断言文本与期望值**不许改**。
    use crate::asset::DirAssetSource as DirAssets;
    use crate::rms::FileRms as Rms;
    use std::path::PathBuf;
    use std::time::Duration;

    fn repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn tmp(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiyou-anchor-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn hex(v: &[u8]) -> String {
        v.iter().map(|b| format!("{b:02X}")).collect()
    }

    /// 文件只读位（跨平台注入 IO 失败：Windows 属性位 / POSIX 0444）。
    fn set_readonly(p: &std::path::Path, ro: bool) {
        let mut perms = std::fs::metadata(p).unwrap().permissions();
        perms.set_readonly(ro);
        std::fs::set_permissions(p, perms).unwrap();
    }

    /// Rms 键域 = 固定 8 键白名单（main.rs `Rms::load`）；save/ 下杂散文件**不得**进
    /// stores（契约 platform-contract §1.1 同语义：load_all 只返回域内记录）。
    #[test]
    fn storage_loads_exact_whitelist_and_ignores_strays() {
        let dir = tmp("keys");
        for name in [
            "MOT_L0", "MOT_L1", "MOT_L2", "MOT_L3", "MOT_L4", "MOT_L5", "MOT_IF", "SKY_WAR",
        ] {
            std::fs::write(dir.join(format!("{name}.bin")), [0xABu8]).unwrap();
        }
        std::fs::write(dir.join("FOO.bin"), [0xDEu8, 0xAD]).unwrap();
        std::fs::write(dir.join("readme.txt"), "杂散文本".as_bytes()).unwrap();
        let (_, loaded) = Rms::load(&dir);
        let keys: Vec<&str> = loaded.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            keys,
            [
                "MOT_L0@1", "MOT_L1@1", "MOT_L2@1", "MOT_L3@1", "MOT_L4@1", "MOT_L5@1",
                "MOT_IF@1", "SKY_WAR@1"
            ],
            "键域 8 键、键名 = 店名@1、白名单序；FOO.bin / readme.txt 不得出现"
        );
        assert!(
            loaded.iter().all(|(_, v)| *v == vec![0xABu8]),
            "8 键值与文件字节逐字节一致（文件→load_all 半环；persist 半环见 golden 基线），实际：{loaded:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 大小写：查找跟随**平台文件系统**语义（Windows 不区分 / Android ext4 区分），
    /// 但返回键恒为白名单**规范名**（game-core 消费者按 `MOT_L{n}@1` 字面匹配）。
    #[test]
    fn storage_lookup_follows_fs_case_rules_key_stays_canonical() {
        let dir = tmp("case");
        std::fs::write(dir.join("mot_l0.bin"), [0x11u8]).unwrap(); // 小写杂散名
        std::fs::write(dir.join("MOT_L1.bin"), [0x22u8]).unwrap(); // 规范名
        let (_, loaded) = Rms::load(&dir);
        let l0 = loaded.iter().find(|(k, _)| k == "MOT_L0@1");
        #[cfg(windows)]
        assert_eq!(
            l0.map(|(_, v)| v.clone()),
            Some(vec![0x11]),
            "Windows FS 不区分大小写：mot_l0.bin 被 MOT_L0.bin 查找命中（现状语义）"
        );
        #[cfg(not(windows))]
        assert!(l0.is_none(), "Android/Linux FS 区分大小写：mot_l0.bin 不命中（现状语义）");
        assert!(
            loaded.iter().all(|(k, _)| k.ends_with("@1") && !k.contains("mot_")),
            "返回键恒为白名单规范名，实际：{loaded:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// persist 格式对偶（键 MOT_x@1 ↔ 文件 MOT_x.bin）+ 写-if-changed 幂等：
    /// 同内容二次 persist 不重写（mtime 不变）且内容逐字节不变；异内容写穿。
    #[test]
    fn storage_persist_is_write_if_changed_and_content_exact() {
        let dir = tmp("persist");
        let (rms, _) = Rms::load(&dir);
        rms.persist(&[("MOT_L0@1".into(), b"abc".to_vec())]);
        let p = dir.join("MOT_L0.bin");
        assert_eq!(std::fs::read(&p).unwrap(), b"abc", "键 MOT_L0@1 ↔ 文件 MOT_L0.bin 逐字节");
        let m1 = std::fs::metadata(&p).unwrap().modified().unwrap();
        std::thread::sleep(Duration::from_millis(30));
        rms.persist(&[("MOT_L0@1".into(), b"abc".to_vec())]);
        assert_eq!(std::fs::read(&p).unwrap(), b"abc", "同内容内容不变（内容 hash 判据）");
        assert_eq!(
            std::fs::metadata(&p).unwrap().modified().unwrap(),
            m1,
            "同内容不重写（mtime 不变 = write-if-changed 幂等）"
        );
        rms.persist(&[("MOT_L0@1".into(), b"abd".to_vec())]);
        assert_eq!(std::fs::read(&p).unwrap(), b"abd", "异内容写穿");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// persist 错误吞掉不 panic（`.ok()` 语义，Java `catch {}` 同口径）：
    /// 只读文件写失败被吞、旧内容保留，其余键照常 best-effort 落盘；
    /// 存档目录不可创建（父路径是文件）亦不 panic。
    #[test]
    fn storage_persist_swallows_io_errors_without_panic() {
        let dir = tmp("ioerr");
        let (rms, _) = Rms::load(&dir);
        rms.persist(&[("MOT_L0@1".into(), b"old".to_vec())]);
        set_readonly(&dir.join("MOT_L0.bin"), true);
        rms.persist(&[("MOT_L0@1".into(), b"new".to_vec()), ("MOT_L1@1".into(), vec![0x77])]);
        assert_eq!(
            std::fs::read(dir.join("MOT_L0.bin")).unwrap(),
            b"old",
            "写失败被吞、旧内容保留"
        );
        assert_eq!(
            std::fs::read(dir.join("MOT_L1.bin")).unwrap(),
            vec![0x77],
            "逐键独立 best-effort：其余键照常落盘"
        );
        set_readonly(&dir.join("MOT_L0.bin"), false);
        // 不可创建目录（父路径是文件）：不 panic
        std::fs::write(dir.join("not_a_dir"), b"x").unwrap();
        let (rms2, _) = Rms::load(&dir.join("not_a_dir").join("save"));
        rms2.persist(&[("MOT_IF@1".into(), vec![1, 2, 3])]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 字节级基线（F4 销账：旧实现删除后仍可测）：固定目录布局 → 期望 hex。
    /// 采集：2026-10-17 迁移前 Rms（基线 ba14d4e）实跑确认；迁移后 FileRms 必须
    /// 逐字节复现本表。
    #[test]
    fn storage_golden_layout_baseline() {
        let dir = tmp("golden");
        std::fs::write(dir.join("MOT_L0.bin"), [0x00, 0x11, 0x22, 0x33]).unwrap();
        std::fs::write(dir.join("MOT_IF.bin"), [0x01, 0x02, 0x03, 0x04, 0x05]).unwrap();
        std::fs::write(dir.join("SKY_WAR.bin"), [0xFF]).unwrap();
        std::fs::write(dir.join("FOO.bin"), [0xDE, 0xAD]).unwrap();
        let (rms, loaded) = Rms::load(&dir);
        let got: Vec<(String, String)> =
            loaded.iter().map(|(k, v)| (k.clone(), hex(v))).collect();
        assert_eq!(
            got,
            [
                ("MOT_L0@1".to_string(), "00112233".to_string()),
                ("MOT_IF@1".to_string(), "0102030405".to_string()),
                ("SKY_WAR@1".to_string(), "FF".to_string()),
            ],
            "load 基线：域内缺失键缺席、键序 = 白名单序"
        );
        rms.persist(&[("MOT_L1@1".into(), vec![0xAA, 0xBB]), ("MOT_IF@1".into(), vec![0x09, 0x08, 0x07])]);
        for (name, want) in [
            ("MOT_L0.bin", "00112233"), // 未提及 ⇒ 不动
            ("MOT_L1.bin", "AABB"),     // 新键 ⇒ 创建
            ("MOT_IF.bin", "090807"),   // 同键 ⇒ 覆盖
            ("SKY_WAR.bin", "FF"),      // 未提及 ⇒ 不动
            ("FOO.bin", "DEAD"),        // 域外文件 ⇒ 不触碰
        ] {
            assert_eq!(
                hex(&std::fs::read(dir.join(name)).unwrap()),
                want,
                "persist 基线：{name}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 接缝④ 判据（B4 缝测，自 shell/win 随迁断言零字节改动）：资产读取经
    /// `game_platform::AssetSource` trait——DirAssets 经 dyn 读与 std::fs::read
    /// 逐字节一致；缺文件 ⇒ Err 不 panic（panic 归壳侧，文案由壳侧 panic_text_*
    /// 三类锚逐字锁）。
    #[test]
    fn asset_reads_go_through_assetsource_trait() {
        use game_platform::AssetSource;
        let raw = repo().join("assets/raw");
        let src = DirAssets::new(&raw);
        let via_trait: &dyn AssetSource = &src;
        for rel in ["mapbg", "maplv1", "sprite1", "l0.png"] {
            let via = via_trait.read(rel).unwrap();
            let direct = std::fs::read(raw.join(rel)).unwrap();
            assert_eq!(via, direct, "{rel} 经 dyn AssetSource 读 == std::fs::read 逐字节");
        }
        assert!(via_trait.read("no_such_asset").is_err(), "缺文件 ⇒ Err（不 panic）");
    }
}
