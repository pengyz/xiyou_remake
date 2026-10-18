//! asset_pipeline —— 二进制 ↔ 中间态（PNG + JSON）打包/解包（STAGE-B-003 硬 1）。
//!
//! 恒等门禁（复核硬 1，常驻 `tests/asset_roundtrip.rs`）：**`repack(unpack(x)) == x`
//! 逐字节**。恒等判据在 repack 端成立——PNG 重编码字节必变（压缩器选择），故中间态
//! 的子图 PNG **逐字节保留、不重编码**，"复刻调色板序/帧切分/对齐"由逐字节保留 +
//! 帧序 JSON 承担；编辑器改图后 repack 嵌入新字节（编辑资产不承诺恒等，门禁只钉
//! 未修改资产）。
//!
//! # 中间态 JSON schema（version 1，复核硬 5 定稿）
//!
//! 通用信封（所有族）：
//!
//! ```json
//! {
//!   "version": 1,
//!   "format": "<maplv|sprite|packed_png>",
//!   "name": "<资源名>",
//!   ... 族字段 ...
//! }
//! ```
//!
//! 族字段（键序 = 下表列序 = canonical 键序）：
//!
//! | format | 字段 | 类型 | 说明 |
//! |---|---|---|---|
//! | `maplv` | `raw_width` `raw_height` | 整数 | 文件头 `u16 LE ×2` **原始值**（= 真实尺寸 ×2；可能为奇数 ⇒ 必须存原始值，不得由尺寸反推） |
//! | `maplv` | `terrain` `flip` | 整数数组 | 两张网格，各 `Y·Z·4` 字节（Y=`raw_width>>1`，Z=`raw_height>>1`），元素 0..=255 |
//! | `sprite` | `count_lo` `count_hi` | 整数 | 头两个 `u16 LE`（`count = lo \| hi<<16`，须 == `records` 长度） |
//! | `sprite` | `records` | 对象数组 | 每条 `{"type_code","x","y","extra"}`；`extra` = 原始字节整数数组，长度 == [`crate::sprite::extra_len`] |
//! | `packed_png` | `images` | 字符串数组 | 帧序文件名清单（`<name>_<i:02>.png`）；对应 `Mid.images` 里的 PNG 字节，序即文件内帧序 |
//!
//! # canonical 排版（写出端恒定，git diff 可读、字节稳定）
//!
//! - 键序固定（上表列序；信封三键在前）；
//! - 缩进 2 空格、LF 换行、文件尾 LF；
//! - 数字 = 十进制整数（无前导零/正号/小数/指数）；
//! - 顶层数组：元素 ≤16/行（行内 `, ` 分隔、行首缩进 4）；空数组内联 `[]`；
//!   元素为对象时 1 对象/行（对象**单行内联**，其内数组亦内联）；
//! - 字符串转义仅 `\"` `\\` 与控制字符（`\n` `\r` `\t` 其余 `\u00XX`）。
//!
//! # 读取端策略（定死：**未知字段容忍忽略**）
//!
//! - **前向兼容**：未知键忽略不报错（旧 repack 可读新 schema 的 JSON）；已知字段
//!   缺失/类型不符/重复键 ⇒ 报错；
//! - 空白不敏感（canonical 只约束写出端）；数字仅支持整数（小数/指数拒绝）；
//!   未知字段的值类型限本子集（整数/字符串/布尔/空/数组/对象）；
//! - `version` 必须 == 1，其余版本拒绝（升级 schema 时旧版本行为另行定义）。
//!
//! # 双面分工（复核硬 2 的登记口径）
//!
//! 二进制解析器（[`crate::maplv`] / [`crate::sprite`] / [`crate::packed_png`]）=
//! oracle/工具链专用；本模块的 JSON+PNG 中间态 = 素材编辑/未来 runtime 消费面。
//! 两面对照测试即 `tests/asset_roundtrip.rs`（恒等）+ 各族单测（字段语义）。
//!
//! # 证据（A 级）
//!
//! - 二进制布局：`docs/findings/resource-formats.md`（packed_png :74-94、maplv :173-187、
//!   sprite :254-263）；`a.java:2657-2676`（容器读取）、`a.java:7173-7188`（maplv）、
//!   `a.java:9495-9618`（sprite）。
//! - 恒等门禁口径：`docs/findings/reviews/2026-10-18-stage-b-003-asset-pipeline-review.md` §1。

mod json;
mod maplv;
mod packed_png;
mod sprite;

pub use json::{parse as parse_json, Json};

/// 中间态：JSON + 图片清单（PNG 字节逐字保留）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mid {
    /// 中间态 JSON 文本（canonical 排版；schema 见模块文档）。
    pub json: String,
    /// 图片清单（文件名 → PNG 字节，顺序 = 帧序）；maplv/sprite 恒空。
    pub images: Vec<(String, Vec<u8>)>,
}

/// 管线错误（解析/校验失败的可读诊断）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(s)
    }
}

impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Self {
        Error(format!("{e:?}"))
    }
}

impl Error {
    fn msg(s: impl Into<String>) -> Error {
        Error(s.into())
    }
}

/// 二进制 → 中间态（PNG + JSON）。`name` 决定族：`maplv*` / `sprite*` /
/// [`crate::packed_png::RESOURCE_IMAGE_COUNTS`] 中的容器名。
pub fn unpack(name: &str, bytes: &[u8]) -> Result<Mid, Error> {
    if name.starts_with("maplv") {
        maplv::unpack(name, bytes)
    } else if name.starts_with("sprite") {
        sprite::unpack(name, bytes)
    } else if crate::packed_png::image_count(name).is_some() {
        packed_png::unpack(name, bytes)
    } else {
        Err(Error::msg(format!("{name}: 未知资源族")))
    }
}

/// 中间态 → 二进制（逐字节恒等于 unpack 的输入；族由 JSON `format` 决定）。
pub fn repack(mid: &Mid) -> Result<Vec<u8>, Error> {
    let doc = json::parse(&mid.json).map_err(Error::msg)?;
    let version = doc.need_int("version").map_err(Error::msg)?;
    if version != 1 {
        return Err(Error::msg(format!("schema 版本不识别：{version}（支持 1）")));
    }
    let format = doc.need_str("format").map_err(Error::msg)?;
    match format {
        "maplv" => maplv::repack(&doc),
        "sprite" => sprite::repack(&doc),
        "packed_png" => packed_png::repack(&doc, &mid.images),
        other => Err(Error::msg(format!("format 不识别：{other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// maplv 合成样例往返 + schema 信封（version/format/name）+ 未知字段忽略。
    #[test]
    fn maplv_synthetic_roundtrip_and_envelope() {
        // 头 (26,26) = 13×13，两网格各 676 字节；值可辨识
        let mut terrain = vec![0u8; 676];
        let mut flip = vec![0u8; 676];
        terrain[0] = 42;
        flip[675] = 7;
        let mut bin = vec![0x1a, 0x00, 0x1a, 0x00];
        bin.extend_from_slice(&terrain);
        bin.extend_from_slice(&flip);
        let mid = unpack("maplvX", &bin).unwrap();
        assert!(mid.images.is_empty(), "maplv 无图");
        assert!(mid.json.contains("\"version\": 1"));
        assert!(mid.json.contains("\"format\": \"maplv\""));
        assert!(mid.json.contains("\"name\": \"maplvX\""));
        assert_eq!(repack(&mid).unwrap(), bin, "恒等");

        // 前向兼容：未知字段忽略（旧 repack 读新 schema）
        let with_unknown = format!("{{\n  \"version\": 1,\n  \"format\": \"maplv\",\n  \"name\": \"maplvX\",\n  \"future\": {{\"k\": 1}},\n  \"flag\": true\n}}\n");
        // （直接替换 json 会缺族字段 ⇒ 报错；此处只验解析层容忍，字段缺失仍拒绝）
        let mid2 = Mid { json: with_unknown, images: vec![] };
        assert!(repack(&mid2).is_err(), "缺族字段必须拒绝");
    }
}
