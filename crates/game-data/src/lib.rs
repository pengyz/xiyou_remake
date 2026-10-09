//! # game-data —— 资源解析（纯解析 crate）
//!
//! 《囧囧西游-大闹天宫》（Nokia J2ME / MIDP-1.0）三种已考证资源格式的 Rust 解析器 +
//! script DSL 词法拆分辅助：
//!
//! | 模块 | 格式 | 证据基线 |
//! |---|---|---|
//! | [`packed_png`] | packed PNG 多图集容器 `N × [u16 LE 长度][PNG 字节]` | `docs/findings/resource-formats.md:74-94` |
//! | [`maplv`] | maplv 关卡：`u16 LE ×2` 头（`>>1` 得尺寸）+ 两张 `Y·Z·4` 字节网格 | `docs/findings/resource-formats.md:173-187` |
//! | [`sprite`] | sprite 对象表：4 字节计数头 + 变长记录 | `docs/findings/resource-formats.md:254-263` |
//! | [`script`] | script DSL 词法：3 字母指令词 + 下划线参数切分 | `docs/findings/script-dsl-semantics.md:31-41` |
//!
//! 字段语义严格对照 `docs/findings/resource-formats.md` 的字段布局表，**不发明行为**：
//! 每个字段的 doc 注释都带证据指针（`文件:行`/字节偏移）；游戏层语义（如 sprite 附加参数的
//! `+1`/查表拼接）留在解释器层，这里只还原原始字节。
//!
//! # 红线（AGENTS.md §2）
//!
//! - **纯解析无渲染**：不依赖任何渲染/平台 crate；[`parse`](packed_png::PackedPng::parse) 等
//!   入口只吃 `&[u8]`，**库内不做任何 I/O**（无 `std::fs`/`std::io`/`std::net`/`std::time`）。
//! - 解析结果必须过 L1 golden test（`tests/golden.rs`：55 maplv + 55 sprite + 16 packed PNG
//!   全量解析且零余字节，夹具从 `assets/raw` 读取或由 `tools/extract-assets/extract.py` 解包生成，
//!   路径可用环境变量 `XIYOU_ASSETS_RAW` 配置）。

pub mod error;
pub mod maplv;
pub mod packed_png;
pub mod script;
pub mod sprite;

pub use error::Error;
pub use maplv::Maplv;
pub use packed_png::{image_count, PackedPng, PngImage, RESOURCE_IMAGE_COUNTS, PNG_MAGIC};
pub use script::{is_known_opcode, lex_instr_stream, parse_guts_line, split_instr_token, GutsLine, Instr, OPCODES};
pub use sprite::{extra_len, SpriteRecord, SpriteTable};

/// 底层小端读取原语（对应 `reference/seed/a.java:4914-4915` 的
/// `private static short a(InputStream)`：`read()&0xFF | read()<<8&0xFF00`，即 u16 LE）。
pub(crate) mod read {
    use crate::error::Error;

    /// 读 `u16 LE`（`a.java:4914-4915`）。
    pub fn u16_le(data: &[u8], pos: usize) -> Result<u16, Error> {
        let bytes = take(data, pos, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// 取 `[pos, pos+len)` 切片，不足则 [`Error::UnexpectedEof`]。
    pub fn take(data: &[u8], pos: usize, len: usize) -> Result<&[u8], Error> {
        let end = pos.checked_add(len).ok_or(Error::UnexpectedEof {
            offset: pos,
            needed: len,
            available: 0,
        })?;
        if end > data.len() {
            return Err(Error::UnexpectedEof {
                offset: pos,
                needed: len,
                available: data.len() - pos,
            });
        }
        Ok(&data[pos..end])
    }
}
