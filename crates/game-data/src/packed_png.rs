//! ① packed PNG 多图集容器：`N × [u16 LE 长度][PNG 字节]`，`N` 不写入文件内。
//!
//! 一个资源文件顺序存放 N 张 PNG，每张前面是**自己**的 2 字节 LE 长度前缀；
//! 张数由代码里的常量表给出（[`RESOURCE_IMAGE_COUNTS`]），文件内不存该计数。
//!
//! # 字段布局表（严格对照 `docs/findings/resource-formats.md:74-81`）
//!
//! | 偏移 | 类型 | 含义 | 证据 |
//! |---|---|---|---|
//! | 文件级：无固定头，总张数不写入文件内 | — | 张数来自常量表 | `a.java:465-466` |
//! | 每张记录 +0 | u16 LE | 本张 PNG 字节长度 | `a.java:2667`、`a.java:4914-4915` |
//! | 每张记录 +2 .. +2+len | byte[len] | 标准 PNG（魔数 `89504E470D0A1A0A` 开头） | `docs/findings/resource-formats.md:80` |
//! | （下一张紧接上一张末尾） | — | 顺序排列，无填充/对齐 | `docs/findings/resource-formats.md:81`（`remaining=0`） |
//!
//! # 证据（A 级）
//!
//! - 读取循环：`reference/seed/a.java:2657-2676` `private void a(int n)`——
//!   `a.java:2667` 逐图读 `u16 LE` 长度（等价 `a.java:4914-4915` `private static short a(InputStream)`），
//!   `a.java:2674-2675` 按长度取 PNG 字节并 `Image.createImage` 解码。
//! - 张数常量表：`reference/seed/a.java:465`（`this.b` 资源名数组）与 `a.java:466`（`this.a` 计数数组）
//!   一一对应；与 `docs/findings/resource-formats.md:83-94` 表格一致。
//! - 字节级复现（16/16 文件零余字节 + 每张 PNG 魔数）：`docs/findings/resource-formats.md:42-68`。

use crate::error::Error;
use crate::read;

/// PNG 魔数（每张子图起始 8 字节必须是它）。
/// Evidence: `docs/findings/resource-formats.md:80`（字段布局表）；`docs/findings/resource-formats.md:44`（复现判据）。
pub const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// 资源名 → 容器内子图张数（顺序与 `this.b[n]`/`this.a[n]` 一一对应）。
/// Evidence: `reference/seed/a.java:465-466`；`docs/findings/resource-formats.md:83-94`。
pub const RESOURCE_IMAGE_COUNTS: [(&str, u16); 16] = [
    ("sflogo", 8),
    ("mapbg", 1),
    ("map", 12),
    ("actor", 4),
    ("sptmap", 13),
    ("sptprop", 23),
    ("sptarm", 10),
    ("sptenemy1", 20),
    ("ui", 25),
    ("xtq", 6),
    ("menu", 2),
    ("intro", 2),
    ("face", 12),
    ("sptenemy2", 20),
    ("end", 1),
    ("load", 2),
];

/// 按资源名查子图张数（`this.b[n]` ↔ `this.a[n]`，`reference/seed/a.java:465-466`）。
pub fn image_count(resource: &str) -> Option<u16> {
    RESOURCE_IMAGE_COUNTS
        .iter()
        .find(|(name, _)| *name == resource)
        .map(|(_, count)| *count)
}

/// 容器内一张子图（零拷贝切片）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PngImage<'a> {
    /// 本张记录在容器内的起始字节偏移（该偏移处即 `u16 LE` 长度前缀）。
    pub offset: usize,
    /// `u16 LE` 长度前缀 = 本张 PNG 字节数。Evidence: `a.java:2667`；`docs/findings/resource-formats.md:79`。
    pub length: u16,
    /// PNG 字节（魔数开头）。Evidence: `a.java:2674-2675`；`docs/findings/resource-formats.md:80`。
    pub bytes: &'a [u8],
}

/// 解析结果：N 张子图，按文件内顺序。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedPng<'a> {
    /// 子图列表（长度 = 解析时传入的 `image_count`）。
    pub images: Vec<PngImage<'a>>,
}

impl<'a> PackedPng<'a> {
    /// 解析前缀，返回（结果，消费字节数）。`image_count` 来自 [`RESOURCE_IMAGE_COUNTS`]（文件内不存张数）。
    pub fn parse_partial(data: &'a [u8], image_count: usize) -> Result<(Self, usize), Error> {
        let mut pos = 0usize;
        let mut images = Vec::with_capacity(image_count);
        for image in 0..image_count {
            let offset = pos;
            // 每张记录 +0：u16 LE 长度（a.java:2667；a.java:4914-4915）
            let length = read::u16_le(data, pos)?;
            pos += 2;
            // 每张记录 +2 .. +2+len：PNG 字节（docs/findings/resource-formats.md:80）
            let bytes = read::take(data, pos, length as usize)?;
            if bytes.len() < PNG_MAGIC.len() || bytes[..PNG_MAGIC.len()] != PNG_MAGIC {
                return Err(Error::BadPngSignature { image, offset });
            }
            images.push(PngImage { offset, length, bytes });
            pos += length as usize;
        }
        Ok((PackedPng { images }, pos))
    }

    /// 严格解析：解析完必须**零余字节**（`docs/findings/resource-formats.md:81` 的 `remaining=0` 判据）。
    pub fn parse(data: &'a [u8], image_count: usize) -> Result<Self, Error> {
        let (parsed, consumed) = Self::parse_partial(data, image_count)?;
        if consumed != data.len() {
            return Err(Error::TrailingBytes { consumed, total: data.len() });
        }
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 常量表与 findings 表格逐项一致（`docs/findings/resource-formats.md:85-94`）。
    #[test]
    fn count_table_matches_findings() {
        let expected = [
            ("sflogo", 8u16), ("ui", 25), ("mapbg", 1), ("xtq", 6),
            ("map", 12), ("menu", 2), ("actor", 4), ("intro", 2),
            ("sptmap", 13), ("face", 12), ("sptprop", 23), ("sptenemy2", 20),
            ("sptarm", 10), ("end", 1), ("sptenemy1", 20), ("load", 2),
        ];
        for (name, count) in expected {
            assert_eq!(image_count(name), Some(count), "{name}");
        }
        assert_eq!(image_count("nope"), None);
    }

    /// 空容器（0 张）在空数据上合法。
    #[test]
    fn empty_container() {
        let (packed, consumed) = PackedPng::parse_partial(&[], 0).unwrap();
        assert_eq!((packed.images.len(), consumed), (0, 0));
    }

    #[test]
    fn truncated_length_prefix() {
        let err = PackedPng::parse(&[0x01], 1).unwrap_err();
        assert_eq!(
            err,
            Error::UnexpectedEof { offset: 0, needed: 2, available: 1 }
        );
    }

    #[test]
    fn truncated_payload() {
        // 长度前缀=4，但只有 2 字节数据
        let err = PackedPng::parse(&[0x04, 0x00, 0x89, 0x50], 1).unwrap_err();
        assert_eq!(
            err,
            Error::UnexpectedEof { offset: 2, needed: 4, available: 2 }
        );
    }

    #[test]
    fn bad_png_magic_rejected() {
        // 2 字节 payload 凑不出 8 字节魔数 ⇒ BadPngSignature
        let data = [0x02, 0x00, 0x89, 0x00];
        let err = PackedPng::parse(&data, 1).unwrap_err();
        assert_eq!(err, Error::BadPngSignature { image: 0, offset: 0 });
    }

    #[test]
    fn trailing_bytes_rejected() {
        // 1 张 payload 正好 8 字节 = 魔数本身，再追加 1 个余字节
        let mut data = vec![0x08, 0x00];
        data.extend_from_slice(&PNG_MAGIC);
        let (packed, consumed) = PackedPng::parse_partial(&data, 1).unwrap();
        assert_eq!((packed.images.len(), consumed), (1, 10));
        data.push(0xFF);
        assert_eq!(
            PackedPng::parse(&data, 1).unwrap_err(),
            Error::TrailingBytes { consumed: 10, total: 11 }
        );
    }
}
