//! 解析错误。变体尽量携带字节偏移，便于对照 hexdump 复现（AGENTS.md §3 证据纪律）。

use std::fmt;

/// game-data 解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// 数据提前结束：在 `offset` 处需要 `needed` 字节，仅剩 `available` 字节。
    UnexpectedEof {
        /// 需要读取的起始偏移
        offset: usize,
        /// 需要的字节数
        needed: usize,
        /// 实际可用字节数
        available: usize,
    },
    /// 解析完成后仍有余字节——golden test 要求「零余字节」，
    /// 严格解析（`parse`）把余字节视为格式不符（`docs/findings/resource-formats.md:81` 复现命令的 `remaining=0` 判据）。
    TrailingBytes {
        /// 实际消费的字节数
        consumed: usize,
        /// 文件总字节数
        total: usize,
    },
    /// packed PNG：第 `image` 张子图不以 PNG 魔数开头（`docs/findings/resource-formats.md:80`）。
    BadPngSignature {
        /// 子图序号（0-based）
        image: usize,
        /// 该子图 PNG 字节的起始偏移
        offset: usize,
    },
    /// script DSL：指令 token 词法错误（`token` 为原串，`reason` 为原因）。
    BadToken {
        /// 出错的 token 原文
        token: String,
        /// 原因
        reason: &'static str,
    },
    /// script DSL：GUTS 行不是「`<层号> <指令流…>`」形态
    ///（`docs/findings/script-dsl-semantics.md:41`；段落终止行 `END` 不是 GUTS 行）。
    BadGutsLine {
        /// 出错的整行原文
        line: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedEof { offset, needed, available } => write!(
                f,
                "unexpected EOF at byte {offset}: need {needed} bytes, only {available} available"
            ),
            Error::TrailingBytes { consumed, total } => write!(
                f,
                "trailing bytes: consumed {consumed} of {total} ({} left)",
                total - consumed
            ),
            Error::BadPngSignature { image, offset } => {
                write!(f, "image #{image} at byte {offset} does not start with PNG magic")
            }
            Error::BadToken { token, reason } => {
                write!(f, "bad script token {token:?}: {reason}")
            }
            Error::BadGutsLine { line } => write!(f, "bad GUTS line {line:?}"),
        }
    }
}

impl std::error::Error for Error {}
