//! ③ sprite 对象表：4 字节头（两个 `u16 LE` 拼 `u32` 计数）+ 变长记录。
//!
//! # 字段布局表（严格对照 `docs/findings/resource-formats.md:254-263`）
//!
//! | 偏移 | 类型 | 含义 | 证据 |
//! |---|---|---|---|
//! | +0 | u16 LE | 条目计数低 16 位 | `docs/findings/resource-formats.md:258`；`a.java:7270` |
//! | +2 | u16 LE | 条目计数高 16 位（本语料实测恒为 0） | `docs/findings/resource-formats.md:259`；`a.java:7270` |
//! | 每条记录 +0 | u8 | 实体/对象类型码（索引 `this.m[code]` 得中文名，88 项） | `docs/findings/resource-formats.md:260`；`a.java:7272`；`a.java:545` |
//! | 每条记录 +1 | u16 LE | x 坐标（像素） | `docs/findings/resource-formats.md:261`；`a.java:7273` |
//! | 每条记录 +3 | u16 LE | y 坐标 | `docs/findings/resource-formats.md:262`；`a.java:7274` |
//! | 每条记录 +5 | 0/1/2 字节（由类型码决定） | 附加参数原始字节 | `docs/findings/resource-formats.md:263`；`a.java:7275-7341` |
//!
//! # 附加参数字节数表（`extra_len`，逐类型码来自 switch 分支）
//!
//! Evidence: `reference/seed/a.java:7275` `switch (n7)` 的各 case（已逐行核对）：
//!
//! | 字节数 | 类型码 | 代码分支 |
//! |---|---|---|
//! | 2 | `7` `8` | `a.java:7305-7310` `read() \| read()<<8`（LE u16） |
//! | 2 | `76` `82` | `a.java:7333-7338` `read() \| (read()+1)<<8` |
//! | 1 | `9` | `a.java:7282-7286` |
//! | 1 | `4` | `a.java:7287-7291`（`read()+1`） |
//! | 1 | `5` `81` | `a.java:7292-7298`（`read()+1`） |
//! | 1 | `83` | `a.java:7299-7304` |
//! | 1 | `57` `59` `70` `71` `72` `73` | `a.java:7311-7320`（`read()+1`） |
//! | 1 | `77` | `a.java:7321-7326`（`read()` 后与 `this.m[]` 表拼高字节） |
//! | 1 | `78` | `a.java:7327-7332`（`read()` 后与 `this.n[]` 表拼高字节） |
//! | 0 | `6` `12` | `a.java:7276-7281` |
//! | 0 | 其余（`default`） | `a.java:7340-7343` |
//!
//! 注意：本解析器只还原**原始字节**，switch 里的值后处理（`+1`、与 `this.m[]`/`this.n[]` 拼接、
//! `case 76/82` 高字节 `+1`）是解释器层语义，不在解析层发明
//!（`docs/findings/resource-formats.md:310-313` 假设 2 明确未穷举业务含义）。
//!
//! # 证据（A 级）
//!
//! - 读取循环：`reference/seed/a.java:7264-7342`——`a.java:7270` 两个 LE16 拼 32 位计数、
//!   `a.java:7271` 循环计数次、`a.java:7272-7274` 类型码 + 2×u16 坐标、`a.java:7275-7341` 变长 switch。
//! - 字节复现（55/55 文件 `consumed==filesize`）：`docs/findings/resource-formats.md:220-249`。
//! - 类型码取值域（≤ 86，落在 `this.m` 88 项内）：`docs/findings/resource-formats.md:250-252`。

use crate::error::Error;
use crate::read;

/// 按类型码决定的附加参数字节数（0/1/2）。
/// Evidence: `reference/seed/a.java:7275-7341` switch 分支（表见模块文档）。
pub fn extra_len(type_code: u8) -> usize {
    match type_code {
        // 2 字节：a.java:7305-7310（7/8）、a.java:7333-7338（76/82）
        7 | 8 | 76 | 82 => 2,
        // 1 字节：a.java:7282-7286（9）、7287-7291（4）、7292-7298（5/81）、7299-7304（83）、
        //         7311-7320（57/59/70/71/72/73）、7321-7326（77）、7327-7332（78）
        4 | 5 | 9 | 57 | 59 | 70 | 71 | 72 | 73 | 77 | 78 | 81 | 83 => 1,
        // 0 字节：a.java:7276-7281（6/12）与 a.java:7340-7343（default）
        _ => 0,
    }
}

/// sprite 一条对象记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteRecord {
    /// 实体/对象类型码（索引 `this.m[code]` 得中文名）。
    /// Evidence: `docs/findings/resource-formats.md:260`；`a.java:7272`；类型名表 `a.java:545`。
    pub type_code: u8,
    /// x 坐标（u16 LE，像素；32px 网格对齐是游戏层语义，见 `a.java:4409-4418`）。
    /// Evidence: `docs/findings/resource-formats.md:261`；`a.java:7273`。
    pub x: u16,
    /// y 坐标（u16 LE，像素）。Evidence: `docs/findings/resource-formats.md:262`；`a.java:7274`。
    pub y: u16,
    /// 附加参数**原始字节**（长度 = [`extra_len`]，0/1/2 字节）。
    /// Evidence: `docs/findings/resource-formats.md:263`；`a.java:7275-7341`。
    pub extra: Vec<u8>,
}

/// sprite 对象表（整个 `sprite{n}` 资源）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteTable {
    /// +0：计数低 16 位。Evidence: `docs/findings/resource-formats.md:258`；`a.java:7270`。
    pub count_lo: u16,
    /// +2：计数高 16 位（本语料实测恒为 0）。Evidence: `docs/findings/resource-formats.md:259`；`a.java:7270`。
    pub count_hi: u16,
    /// 记录表（长度 = [`Self::count`]）。
    pub records: Vec<SpriteRecord>,
}

impl SpriteTable {
    /// 条目计数 = `count_lo | count_hi << 16`（`a.java:7270`
    /// `a.a(inputStream2) & 0xFFFF | a.a(inputStream2) << 16`）。
    pub fn count(&self) -> u32 {
        (self.count_lo as u32) | ((self.count_hi as u32) << 16)
    }

    /// 解析前缀，返回（结果，消费字节数）。
    pub fn parse_partial(data: &[u8]) -> Result<(Self, usize), Error> {
        // +0 / +2：两个 u16 LE 拼 u32 计数（a.java:7270；docs/findings/resource-formats.md:258-259）
        let count_lo = read::u16_le(data, 0)?;
        let count_hi = read::u16_le(data, 2)?;
        let count = ((count_lo as u32) | ((count_hi as u32) << 16)) as usize;
        let mut pos = 4usize;
        let mut records = Vec::with_capacity(count.min(4096));
        for _ in 0..count {
            // +0：类型码 1 字节（a.java:7272）
            let type_code = read::take(data, pos, 1)?[0];
            pos += 1;
            // +1 / +3：x、y 各 u16 LE（a.java:7273-7274）
            let x = read::u16_le(data, pos)?;
            pos += 2;
            let y = read::u16_le(data, pos)?;
            pos += 2;
            // +5：附加参数 0/1/2 字节，长度由类型码决定（a.java:7275-7341）
            let extra = read::take(data, pos, extra_len(type_code))?.to_vec();
            pos += extra.len();
            records.push(SpriteRecord { type_code, x, y, extra });
        }
        Ok((SpriteTable { count_lo, count_hi, records }, pos))
    }

    /// 严格解析：解析完必须**零余字节**（`docs/findings/resource-formats.md:220-249` 的
    /// `consumed==filesize` 判据）。
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        let (table, consumed) = Self::parse_partial(data)?;
        if consumed != data.len() {
            return Err(Error::TrailingBytes { consumed, total: data.len() });
        }
        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(type_code: u8, x: u16, y: u16, extra: &[u8]) -> Vec<u8> {
        let mut out = vec![type_code];
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.extend_from_slice(extra);
        out
    }

    /// 附加参数字节数表逐分支核对（表内证据见模块文档）。
    #[test]
    fn extra_len_table() {
        for code in [7u8, 8, 76, 82] {
            assert_eq!(extra_len(code), 2, "code {code}");
        }
        for code in [4u8, 5, 9, 57, 59, 70, 71, 72, 73, 77, 78, 81, 83] {
            assert_eq!(extra_len(code), 1, "code {code}");
        }
        for code in [0u8, 1, 6, 12, 86, 255] {
            assert_eq!(extra_len(code), 0, "code {code}");
        }
    }

    /// `sprite1` 实测构成骨架：35 条记录、184B 总长（1×2B 附加 + 3×1B 附加 + 31×0B 附加，
    /// 对应类型码 7/83/其余——`docs/findings/resource-formats.md:222-223` 的 184B 复现锚点）。
    #[test]
    fn mixed_record_lengths_consume_exactly() {
        let mut data = vec![35, 0, 0, 0]; // lo=35, hi=0（a.java:7270）
        data.extend(record(7, 1, 2, &[0x34, 0x12])); // 2 字节附加（a.java:7305-7310）
        for _ in 0..31 {
            data.extend(record(6, 1, 2, &[])); // 0 字节附加（a.java:7276-7281）
        }
        for _ in 0..3 {
            data.extend(record(83, 1, 2, &[7])); // 1 字节附加（a.java:7299-7304）
        }
        assert_eq!(data.len(), 184);
        let table = SpriteTable::parse(&data).unwrap();
        assert_eq!((table.count(), table.records.len()), (35, 35));
        assert_eq!(table.records[0].extra, vec![0x34, 0x12]);
        assert_eq!(table.records[32].extra, vec![7]);
        data.push(0); // 余字节必须被拒
        assert_eq!(
            SpriteTable::parse(&data).unwrap_err(),
            Error::TrailingBytes { consumed: 184, total: 185 }
        );
    }

    /// 2 字节附加记录（`case 7/8` LE u16）。
    #[test]
    fn two_byte_extra() {
        let mut data = vec![1, 0, 0, 0];
        data.extend(record(7, 3, 4, &[0x34, 0x12]));
        let table = SpriteTable::parse(&data).unwrap();
        assert_eq!(table.records[0].extra, vec![0x34, 0x12]);
        assert_eq!((table.records[0].x, table.records[0].y), (3, 4));
    }

    #[test]
    fn truncated_record_rejected() {
        // 声明 2 条但只有 1 条的字节
        let mut data = vec![2, 0, 0, 0];
        data.extend(record(6, 1, 2, &[]));
        let err = SpriteTable::parse(&data).unwrap_err();
        assert_eq!(
            err,
            Error::UnexpectedEof { offset: 9, needed: 1, available: 0 }
        );
    }
}
