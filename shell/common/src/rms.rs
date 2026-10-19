//! RMS 文件语义核心（共享实现）：`save/{店}.bin` 平铺具名 store + MOT_IF 槽表序列化。
//!
//! 契约面 = `game_platform::Storage`（docs/spec/platform-contract.md §1）：键模型只用
//! 契约的 [`game_platform::RMS_STORES`] / [`game_platform::rms_key`] /
//! [`game_platform::rms_store_name`]——**不自带第二套白名单**（adversary 抽取判据 3）。
//!
//! 等价性（契约 §1.3）：Java RMS 在本游戏可达路径恒"每店单记录"（写前必
//! delete+addRecord 或 setRecord(1)）⇒ 观测等价于"具名 store 单 blob 覆写"；
//! `{店}.bin` 平铺与此观测等价（A 级行锚见契约 §1.3）。
//!
//! 平台差：文件查找跟随宿主 FS 原生语义（Windows 不区分大小写 / Android ext4 区分），
//! 契约不强行归一（§5 脚注）。

use std::path::PathBuf;

use game_platform::{rms_key, rms_store_name, Storage, RMS_STORES};

/// 文件版 RecordStore：每 store 一文件（`{店}.bin`）。与 BootMachine.preset_stores
/// 同键名（MOT_L0-5 / MOT_IF / SKY_WAR）。
///
/// 由 shell/win `Rms`（main.rs:163-192，基线 ba14d4e）迁入；`FileRms::load` 构造 +
/// 载入、`Storage::persist` 持久（同键覆盖、新键创建、未提及不动、写-if-changed 幂等、
/// 错误 best-effort 吞掉——Java `catch {}` 同口径 a.java:8955-8956）。
pub struct FileRms {
    dir: PathBuf,
}

impl FileRms {
    /// 构造 + 载入记录域内已持久记录（缺失 store 静默缺席）。
    /// 键序 = `RMS_STORES` 序（行为锚 storage_loads_exact_whitelist 钉）。
    pub fn load(dir: &PathBuf) -> (FileRms, Vec<(String, Vec<u8>)>) {
        let mut stores = Vec::new();
        // 【2026-10-17 BUG-018】原只载 MOT_L0-2 三键——槽 3-5 的保存永不落盘（丢失）。
        // 槽表 6 槽（menu_family::SlotSelect），键域须全 6 槽（键域现走契约 RMS_STORES）。
        for name in RMS_STORES {
            if let Ok(data) = std::fs::read(dir.join(format!("{name}.bin"))) {
                stores.push((rms_key(name), data));
            }
        }
        (FileRms { dir: dir.clone() }, stores)
    }
    /// 持久化一批记录：同键覆盖、新键创建（upsert）；未提及的键不动；写-if-changed。
    /// 任何失败 best-effort 吞掉（Java `catch {}` 同口径，a.java:8955-8956）。
    pub fn persist(&self, stores: &[(String, Vec<u8>)]) {
        std::fs::create_dir_all(&self.dir).ok();
        for (k, v) in stores {
            // 域外键 = 契约违约 ⇒ 丢弃 + 日志、不 panic（契约 §1.1；口径对齐
            // game-platform/tests/storage_contract.rs:83-87）。Shell 调用面恒域内
            // （MOT_L{0..5}@1 / MOT_IF@1 / SKY_WAR@1）⇒ 游戏可达行为零漂移。
            if !RMS_STORES.iter().any(|s| rms_key(s) == *k) {
                eprintln!("[rms] 域外键丢弃（契约违约）：{k}");
                continue;
            }
            let path = self.dir.join(format!("{}.bin", rms_store_name(k)));
            if std::fs::read(&path).ok().as_deref() != Some(v.as_slice()) {
                std::fs::write(&path, v).ok();
            }
        }
    }
}

impl Storage for FileRms {
    fn load_all(&self) -> Vec<(String, Vec<u8>)> {
        FileRms::load(&self.dir).1
    }

    fn persist(&self, records: &[(String, Vec<u8>)]) {
        FileRms::persist(self, records)
    }
}

/// scene.slot → MOT_IF 序列化（SlotSelect::load_slot_table 读序的镜像：
/// 6 × (valid u8; valid 时 floor/weapon/armor u8 + hp/atk/def/三钥/金 7×i32 BE)）。
/// 自 shell/win `main.rs:196-218` 迁入（消费方 shell/win Shell::step 存档块）。
pub fn serialize_slot_if(slot: &game_core::menu_family::SlotSelect) -> Vec<u8> {
    let mut o = Vec::new();
    for k in 0..6 {
        o.push(slot.valid[k] as u8);
        if slot.valid[k] {
            o.push(slot.floor_num[k] as u8);
            o.push(slot.weapon[k] as u8);
            o.push(slot.armor[k] as u8);
            for v in [
                slot.hp[k],
                slot.atk[k],
                slot.def[k],
                slot.keys_yellow[k] as i32,
                slot.keys_blue[k] as i32,
                slot.keys_red[k] as i32,
                slot.gold[k],
            ] {
                o.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
    o
}

#[cfg(test)]
mod slot_if_duality {
    //! MOT_IF 字段级对偶（android-shell S3 清单 B① 认领归 shell-port，2026-10-18）：
    //! serialize_slot_if → load_slot_table **全字段**回读相等（load_slot_table 读序
    //! 全镜像：六槽 valid + floor/weapon/armor/hp/atk/def/三钥/金）。新增测试，
    //! 不涉既有断言；既有六槽 valid 对偶锚 rms_roundtrips_all_six_slots_and_mot_if
    //! 留 shell/win（测试名与断言零改动）。
    use super::serialize_slot_if;
    use game_core::menu_family::SlotSelect;

    #[test]
    fn serialize_slot_if_round_trips_every_field() {
        let mut slot = SlotSelect::new();
        for k in 0..6 {
            slot.valid[k] = true;
            slot.floor_num[k] = (k + 1) as i8; // android 清单最小面 floor_num=1..6
            slot.weapon[k] = (k * 2 + 1) as i8;
            slot.armor[k] = (50 - k) as i8;
            slot.hp[k] = 300 + k as i32 * 11;
            slot.atk[k] = 10 + k as i32;
            slot.def[k] = 7 - k as i32;
            slot.keys_yellow[k] = (k * 3) as i16;
            slot.keys_blue[k] = (k as i16) + 100;
            slot.keys_red[k] = (k * 7 + 1) as i16;
            slot.gold[k] = 9999 - k as i32 * 13;
        }
        let bytes = serialize_slot_if(&slot);
        let mut back = SlotSelect::new();
        back.load_slot_table(Some(&bytes));
        for k in 0..6 {
            assert!(back.valid[k], "slot{k}.valid 回读");
            assert_eq!(back.floor_num[k], slot.floor_num[k], "slot{k}.floor_num");
            assert_eq!(back.weapon[k], slot.weapon[k], "slot{k}.weapon");
            assert_eq!(back.armor[k], slot.armor[k], "slot{k}.armor");
            assert_eq!(back.hp[k], slot.hp[k], "slot{k}.hp");
            assert_eq!(back.atk[k], slot.atk[k], "slot{k}.atk");
            assert_eq!(back.def[k], slot.def[k], "slot{k}.def");
            assert_eq!(back.keys_yellow[k], slot.keys_yellow[k], "slot{k}.keys_yellow");
            assert_eq!(back.keys_blue[k], slot.keys_blue[k], "slot{k}.keys_blue");
            assert_eq!(back.keys_red[k], slot.keys_red[k], "slot{k}.keys_red");
            assert_eq!(back.gold[k], slot.gold[k], "slot{k}.gold");
        }
    }
}
