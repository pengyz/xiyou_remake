//! persist_codec face（P2 批0 骨架）——PersistCodec：存档格式编解码面（spec §3.1 注④=只管格式编解码，行为不入）。
//!
//! 形照 spec §3.1：`fn serialize(&self, st: &SceneState) -> Blob`；plan 批4=serialize/deserialize 双向。

/// 【批4 占位】域切片状态集（批4 终形=PersistState 族切片集）。
pub struct SceneState;

/// RMS 字节 blob（Vec<u8> newtype——save.rs RMS 字节 B 级）。
pub struct Blob(pub Vec<u8>);

/// spec §3.1：持久化编解码（格式面——行为不入）。
pub trait PersistCodec {
    fn serialize(&self, st: &SceneState) -> Blob;
    fn deserialize(&self, blob: &Blob) -> SceneState;
}
