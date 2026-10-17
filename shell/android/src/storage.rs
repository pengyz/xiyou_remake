//! Storage 实现：应用私有目录文件版 RMS（宿主注入根路径）。
//!
//! # 契约与等价性（docs/spec/platform-contract.md §1/§1.3）
//!
//! Java RMS 在本游戏可达路径恒"每店单记录"（读=首条枚举记录 a.java:4457-4459/
//! 8978-8980/9260-9262；写=整店替换 8928-8954/9027-9052/9190-9244）⇒ 观测等价于
//! "具名 store 单 blob 覆写"——`{店}.bin` 平铺实现与此观测等价（完整论证与边角
//! a-d/edge d 见契约 §1.3，**引用不另立口径**）。
//!
//! # Android 落地
//!
//! 根路径 = **应用私有目录**，由宿主注入（NativeActivity `activity->internalDataPath`
//! ≈ `Context.getFilesDir()`；`docs/knowledge/decision_save-path-shell-injection.md`
//! ——游戏层零路径，平台路径规范差异归壳）。文件语义（键白名单 RMS_STORES/rms_key、
//! `{店}.bin` 映射、upsert、写-if-changed、域外键丢弃+日志、吞错 best-effort）
//! **单源**在 `game-shell-common::rms::FileRms`——本模块只做根路径注入的薄接线
//! （对抗审查 F-4 裁决：消费不拷贝；主测试在 shell/common 测试面）。

use game_platform::Storage;
use game_shell_common::rms::FileRms;
use std::path::PathBuf;

/// Android Storage 实现：注入根路径（应用私有目录）的文件版 RMS。
pub struct AndroidStorage {
    rms: FileRms,
}

impl AndroidStorage {
    /// 注入根路径并载入已持久记录（缺失 store 静默缺席；键序 = `RMS_STORES` 序）。
    /// 根 = NativeActivity `internalDataPath`（宿主注入；宿主测试用临时目录）。
    pub fn open(files_dir: &PathBuf) -> (AndroidStorage, Vec<(String, Vec<u8>)>) {
        let (rms, stores) = FileRms::load(files_dir);
        (AndroidStorage { rms }, stores)
    }
}

impl Storage for AndroidStorage {
    fn load_all(&self) -> Vec<(String, Vec<u8>)> {
        self.rms.load_all()
    }

    fn persist(&self, records: &[(String, Vec<u8>)]) {
        self.rms.persist(records)
    }
}
