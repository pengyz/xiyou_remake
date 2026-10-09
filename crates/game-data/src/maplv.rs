//! ② maplv 关卡：头 `u16 LE ×2`（值为真实尺寸 ×2，读取后 `>>1`）+ 两张等长网格。
//!
//! # 字段布局表（严格对照 `docs/findings/resource-formats.md:173-180`）
//!
//! | 偏移 | 类型 | 含义 | 证据 |
//! |---|---|---|---|
//! | +0 | u16 LE（原始值 = 真实宽 ×2） | 真实网格宽 `Y = raw>>1` | `docs/findings/resource-formats.md:177`；`a.java:7175` |
//! | +2 | u16 LE（原始值 = 真实高 ×2） | 真实网格高 `Z = raw>>1` | `docs/findings/resource-formats.md:178`；`a.java:7176` |
//! | +4 .. +4+Y·Z·4 | byte[Y·Z·4] | 网格 1（地形 tile-id）：`id&7`=图集列、`id>>3`=图集行（各 ×16px） | `docs/findings/resource-formats.md:179`；`a.java:7178,7181,7183`；`a.java:4996-4999` |
//! | +4+Y·Z·4 .. +4+2·Y·Z·4 | byte[Y·Z·4] | 网格 2（贴图翻转/变换码）：取值域 0-7（`switch(n7){case 1..7}` 8 种变换） | `docs/findings/resource-formats.md:180`；`a.java:7178,7182,7184`；`a.java:2708-2744`；`a.java:5000` |
//!
//! # 「每格 4 字节」与「每格 1 字节」两种视角（findings 注 1，字节等价）
//!
//! 每张网格共 `Y·Z·4` 字节（`a.java:7178` `int n3 = a2.Y * a2.Z << 2`）：
//! - 视角 A（契约/注 1 表述）：`Y×Z` 个「大格」，每格 4 字节；
//! - 视角 B（引擎实际索引）：`2Y×2Z` 个 1 字节格——绘制循环 `a.java:4991`
//!   `n2 = n3 = this.ai + (this.ak * this.Y << 1)`（线性下标 = 列 + 行 × 2Y）、
//!   `a.java:5009` `n2 = n3 += this.Y << 1`（行跨距 = `Y<<1` = 2Y）、每步 16px（`a.java:5002` `n5 += 16`），
//!   且逐格只取 1 字节 `this.i[n2]` / `this.j[n2]`（`a.java:4996`、`a.java:5000`）。
//!
//! 两种视角字节总量恒等（`4·Y·Z = 2Y·2Z·1`）。本解析器按代码字面量存 `Y·Z·4` 字节的
//! 平铺网格（= `byte[] i`/`byte[] j`，`a.java:7181-7182`），并提供按视角 B 的行列访问
//! [`Maplv::tile_at`]/[`Maplv::flip_at`]（行跨距 2Y）。`docs/findings/resource-formats.md:182-187`
//! 注 1 的「设计者原意」仍为待证点（findings 假设 1），不在本 crate 下结论。
//!
//! # 证据（A 级）
//!
//! - 读取：`reference/seed/a.java:7169-7184` `private void s(int n)`——
//!   `a.java:7174` 读 `maplv{n}` 资源、`a.java:7175-7176` 两个 LE16 `>>1`、
//!   `a.java:7178` `n3 = Y*Z<<2`、`a.java:7183-7184` 顺序读两张网格。
//! - 字节复现（55/55 自洽，含 `maplv51` 非方阵 12×13/1252B）：`docs/findings/resource-formats.md:120-148`、`291-296`。
//! - 网格 2 翻转码语义：`a.java:2708-2744` 7 参数绘制方法的 `switch(n7){case 1..7}` 镜像/旋转分支，
//!   `a.java:5000` 把 `this.j[n2]` 传入该参数（`docs/findings/resource-formats.md:163-168`）。

use crate::error::Error;
use crate::read;

/// maplv 关卡（网格平铺为 `byte[]`，与引擎 `this.i`/`this.j` 同构）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Maplv {
    /// +0：`u16 LE`，原始值 = 真实宽 ×2。Evidence: `docs/findings/resource-formats.md:177`；`a.java:7175`。
    pub raw_width: u16,
    /// +2：`u16 LE`，原始值 = 真实高 ×2。Evidence: `docs/findings/resource-formats.md:178`；`a.java:7176`。
    pub raw_height: u16,
    /// 真实网格宽 `Y = raw_width >> 1`。Evidence: `a.java:7175` `a2.Y = a.a(inputStream) >> 1`。
    pub width: u16,
    /// 真实网格高 `Z = raw_height >> 1`。Evidence: `a.java:7176`。
    pub height: u16,
    /// 网格 1（地形 tile-id）：`Y·Z·4` 字节平铺 = 引擎 `byte[] i`（`a.java:7181,7183`）。
    /// 每字节一个 tile-id：`id&7`=图集列、`id>>3`=图集行（`a.java:4998-4999`）。
    pub terrain: Vec<u8>,
    /// 网格 2（贴图翻转/变换码）：`Y·Z·4` 字节平铺 = 引擎 `byte[] j`（`a.java:7182,7184`）。
    /// 传入 7 参数绘制方法的 `switch(n7){case 1..7}` 变换分支（`a.java:2708-2744`；`a.java:5000`）。
    pub flip: Vec<u8>,
}

impl Maplv {
    /// 解析前缀，返回（结果，消费字节数）。消费恒为 `4 + 2·(Y·Z·4)`。
    pub fn parse_partial(data: &[u8]) -> Result<(Self, usize), Error> {
        // +0 / +2：u16 LE，原始值 = 真实尺寸 ×2（a.java:7175-7176；docs/findings/resource-formats.md:177-178）
        let raw_width = read::u16_le(data, 0)?;
        let raw_height = read::u16_le(data, 2)?;
        let width = (raw_width >> 1) as usize;
        let height = (raw_height >> 1) as usize;
        // 每张网格 Y·Z·4 字节（a.java:7178 `int n3 = a2.Y * a2.Z << 2`）
        let grid_bytes = width * height * 4;
        // 网格 1：+4 .. +4+Y·Z·4（a.java:7181,7183；docs/findings/resource-formats.md:179）
        let terrain = read::take(data, 4, grid_bytes)?.to_vec();
        // 网格 2：紧接其后同长（a.java:7182,7184；docs/findings/resource-formats.md:180）
        let flip = read::take(data, 4 + grid_bytes, grid_bytes)?.to_vec();
        let consumed = 4 + 2 * grid_bytes;
        Ok((
            Maplv {
                raw_width,
                raw_height,
                width: width as u16,
                height: height as u16,
                terrain,
                flip,
            },
            consumed,
        ))
    }

    /// 严格解析：解析完必须**零余字节**（`docs/findings/resource-formats.md:120-148` 的
    /// `expected total == filesize` 判据）。
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        let (maplv, consumed) = Self::parse_partial(data)?;
        if consumed != data.len() {
            return Err(Error::TrailingBytes { consumed, total: data.len() });
        }
        Ok(maplv)
    }

    /// 每张网格字节数 = `Y·Z·4`（`a.java:7178`）。
    pub fn grid_bytes(&self) -> usize {
        self.terrain.len()
    }

    /// 视角 B 的列数 = `2Y` = `raw_width`（行跨距 `Y<<1`，`a.java:4991`、`a.java:5009`）。
    pub fn cell_cols(&self) -> usize {
        self.raw_width as usize
    }

    /// 视角 B 的行数 = `2Z` = `raw_height`（每步 16px，`a.java:5002`）。
    pub fn cell_rows(&self) -> usize {
        self.raw_height as usize
    }

    /// 地形 tile-id（视角 B：1 字节/格，下标 = `row*2Y + col`，`a.java:4991`、`a.java:4996`）。
    pub fn tile_at(&self, col: usize, row: usize) -> Option<u8> {
        if col >= self.cell_cols() || row >= self.cell_rows() {
            return None;
        }
        self.terrain.get(row * self.cell_cols() + col).copied()
    }

    /// 翻转/变换码（同上索引；喂给绘制方法第 8 参数，`a.java:5000`、`a.java:2708-2744`）。
    pub fn flip_at(&self, col: usize, row: usize) -> Option<u8> {
        if col >= self.cell_cols() || row >= self.cell_rows() {
            return None;
        }
        self.flip.get(row * self.cell_cols() + col).copied()
    }

    /// tile-id → 图集列（`id&7`，×16px）。Evidence: `a.java:4998` `int n9 = (n4 & 7) << 4`；
    /// `docs/findings/resource-formats.md:179`。
    pub fn tile_atlas_col(id: u8) -> u8 {
        id & 7
    }

    /// tile-id → 图集行（`id>>3`，×16px）。Evidence: `a.java:4999` `n4 = n4 >> 3 << 4`；
    /// `docs/findings/resource-formats.md:179`。
    pub fn tile_atlas_row(id: u8) -> u8 {
        id >> 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 13×13 特例骨架：头 (26,26)，两网格各 676B，总 1356B
    ///（`docs/findings/resource-formats.md:132` 复现输出）。
    #[test]
    fn square_level_1356_bytes() {
        let mut data = vec![0x1a, 0x00, 0x1a, 0x00];
        data.resize(4 + 2 * 13 * 13 * 4, 0);
        let (maplv, consumed) = Maplv::parse_partial(&data).unwrap();
        assert_eq!((maplv.raw_width, maplv.raw_height), (26, 26));
        assert_eq!((maplv.width, maplv.height), (13, 13));
        assert_eq!(maplv.grid_bytes(), 676);
        assert_eq!(consumed, 1356);
        data.push(0); // 加余字节 ⇒ 严格解析必须拒绝
        assert_eq!(
            Maplv::parse(&data).unwrap_err(),
            Error::TrailingBytes { consumed: 1356, total: 1357 }
        );
    }

    /// maplv51 非方阵特例骨架：头 (24,26) → 12×13，两网格各 624B，总 1252B
    ///（`docs/findings/resource-formats.md:293-296`）。
    #[test]
    fn non_square_level_1252_bytes() {
        let mut data = vec![0x18, 0x00, 0x1a, 0x00];
        data.resize(4 + 2 * 12 * 13 * 4, 0);
        let (maplv, consumed) = Maplv::parse_partial(&data).unwrap();
        assert_eq!((maplv.width, maplv.height), (12, 13));
        assert_eq!(maplv.grid_bytes(), 624);
        assert_eq!(consumed, 1252);
    }

    #[test]
    fn truncated_grid_rejected() {
        // 头声明 13×13（676B/网格），实际只给 100 字节
        let mut data = vec![0x1a, 0x00, 0x1a, 0x00];
        data.resize(4 + 100, 0);
        let err = Maplv::parse(&data).unwrap_err();
        assert_eq!(
            err,
            Error::UnexpectedEof { offset: 4, needed: 676, available: 100 }
        );
    }

    /// 视角 B 行列索引与平铺下标一致（`a.java:4991` 下标 = 列 + 行×2Y）。
    #[test]
    fn cell_indexing_matches_engine_stride() {
        let mut data = vec![0x1a, 0x00, 0x1a, 0x00];
        data.resize(4 + 2 * 13 * 13 * 4, 0);
        data[4 + 26 * 3 + 5] = 42; // 网格 1：col=5,row=3（2Y=26）
        data[4 + 676 + 26 * 3 + 5] = 7;
        let maplv = Maplv::parse(&data).unwrap();
        assert_eq!(maplv.tile_at(5, 3), Some(42));
        assert_eq!(maplv.flip_at(5, 3), Some(7));
        assert_eq!(maplv.tile_at(26, 3), None);
        assert_eq!((Maplv::tile_atlas_col(42), Maplv::tile_atlas_row(42)), (2, 5));
    }
}
