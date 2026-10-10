//! PNG → ARGB 解码，与 Java shim `Image.decodePng` 逐字节同语义。
//!
//! # 支持面（= 参考版资源实际出现的形式 + shim 防御分支）
//!
//! - color-type 3（调色板）bit depth 1/2/4/8 + `tRNS` 透明索引——**原版 JAR 全部
//!   164 个 PNG 签名均为 `(depth,3,0)`**（`reference/oracle/_diff/report.md` 资源盘点）
//! - 防御性：color-type 0/2/4/6 且 depth 8（shim 同样支持，参考版未用到）
//! - 非隔行（interlace≠0 报错）；不校验 CRC（shim 亦不校验）
//!
//! # 证据（A 级）
//!
//! - 全部行为锚点：`reference/shim/src/javax/microedition/lcdui/Image.java:92-265`
//! - unfilter 字节算术：Java `byte` 溢出回绕 ⇒ Rust `u8` wrapping
//! - 像素产出 `0xAARRGGBB` 位模式（Java int 的位模式，Rust 以 u32 存储）

use crate::error::Error;
use crate::inflate;

/// 解码产物：width×height 的 ARGB（0xAARRGGBB 位模式）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPng {
    pub width: i32,
    pub height: i32,
    pub argb: Vec<u32>,
}

/// PNG 签名（`Image.decodePng` 的 `sig`，`Image.java:98`）。
pub const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// 解码 PNG 字节流（语义 = `Image.decodePng`）。
pub fn decode(data: &[u8]) -> Result<DecodedPng, Error> {
    if data.len() < 8 {
        return Err(Error::UnexpectedEof { offset: 0, needed: 8, available: data.len() });
    }
    if data[..8] != PNG_SIGNATURE {
        return Err(Error::BadToken { token: "png".into(), reason: "PNG 魔数不符" });
    }
    let mut p = 8usize;
    let end = data.len();
    let (mut w, mut h, mut bit_depth, mut color_type) = (0i32, 0i32, 0u8, 0u8);
    let mut plte: Option<&[u8]> = None;
    let mut trns: Option<&[u8]> = None;
    let mut idat: Vec<u8> = Vec::new();
    let mut saw_iend = false;
    while p + 8 <= end {
        let chunk_len = be32(data, p) as usize;
        let type_str = &data[p + 4..p + 8];
        let body = p + 8;
        if body + chunk_len + 4 > end {
            return Err(Error::BadToken {
                token: String::from_utf8_lossy(type_str).into_owned(),
                reason: "PNG chunk 越界",
            });
        }
        if type_str == b"IHDR" {
            w = be32(data, body) as i32;
            h = be32(data, body + 4) as i32;
            bit_depth = data[body + 8];
            color_type = data[body + 9];
            let interlace = data[body + 12];
            if interlace != 0 {
                return Err(Error::BadToken { token: "interlace".into(), reason: "不支持隔行 PNG" });
            }
        } else if type_str == b"PLTE" {
            plte = Some(&data[body..body + chunk_len]);
        } else if type_str == b"tRNS" {
            trns = Some(&data[body..body + chunk_len]);
        } else if type_str == b"IDAT" {
            idat.extend_from_slice(&data[body..body + chunk_len]);
        } else if type_str == b"IEND" {
            saw_iend = true;
            break;
        }
        p = body + chunk_len + 4;
    }
    let _ = saw_iend;
    if w <= 0 || h <= 0 {
        return Err(Error::BadToken { token: "ihdr".into(), reason: "PNG 缺 IHDR 或尺寸非法" });
    }
    let raw = inflate::zlib_decompress(&idat)?;
    let mut argb = vec![0u32; (w * h) as usize];
    unfilter_decode(&raw, &mut argb, w, h, bit_depth, color_type, plte, trns)?;
    Ok(DecodedPng { width: w, height: h, argb })
}

fn be32(b: &[u8], off: usize) -> u32 {
    u32::from_be_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// 反滤波逐行解码（`Image.unfilterDecode`，`Image.java:147-208`）。
fn unfilter_decode(
    raw: &[u8],
    out: &mut [u32],
    w: i32,
    h: i32,
    bit_depth: u8,
    color_type: u8,
    plte: Option<&[u8]>,
    trns: Option<&[u8]>,
) -> Result<(), Error> {
    let channels: usize = match color_type {
        0 | 3 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => return Err(Error::BadToken { token: format!("ct{color_type}"), reason: "PNG color-type 非法" }),
    };
    if color_type == 3 && plte.is_none() {
        return Err(Error::BadToken { token: "plte".into(), reason: "调色板 PNG 缺 PLTE" });
    }
    let bit_depth = bit_depth as usize;
    let bpp = ((channels * bit_depth) / 8).max(1);
    let row_bytes = (w as usize * channels * bit_depth + 7) / 8;
    let (mut prev, mut cur) = (vec![0u8; row_bytes], vec![0u8; row_bytes]);
    let mut pos = 0usize;
    for y in 0..h as usize {
        if pos + 1 + row_bytes > raw.len() {
            return Err(Error::UnexpectedEof { offset: pos, needed: 1 + row_bytes, available: raw.len() - pos });
        }
        let filter = raw[pos];
        pos += 1;
        cur.copy_from_slice(&raw[pos..pos + row_bytes]);
        pos += row_bytes;
        match filter {
            0 => {}
            1 => {
                for i in bpp..row_bytes {
                    cur[i] = cur[i].wrapping_add(cur[i - bpp]);
                }
            }
            2 => {
                for i in 0..row_bytes {
                    cur[i] = cur[i].wrapping_add(prev[i]);
                }
            }
            3 => {
                for i in 0..row_bytes {
                    let left = if i >= bpp { cur[i - bpp] as u32 } else { 0 };
                    cur[i] = cur[i].wrapping_add(((left + prev[i] as u32) >> 1) as u8);
                }
            }
            4 => {
                for i in 0..row_bytes {
                    let a = if i >= bpp { cur[i - bpp] as i32 } else { 0 };
                    let b = prev[i] as i32;
                    let c = if i >= bpp { prev[i - bpp] as i32 } else { 0 };
                    let pp = a + b - c;
                    let (pa, pb, pc) = ((pp - a).abs(), (pp - b).abs(), (pp - c).abs());
                    let pred = if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c };
                    cur[i] = cur[i].wrapping_add(pred as u8);
                }
            }
            _ => return Err(Error::BadToken { token: format!("filter{filter}"), reason: "PNG filter 非法" }),
        }
        for x in 0..w as usize {
            out[y * w as usize + x] = pixel_at(&cur, x, bit_depth, color_type, plte, trns)?;
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    Ok(())
}

/// 单像素采样（`Image.pixelAt`，`Image.java:210-255`）。
fn pixel_at(
    row: &[u8],
    x: usize,
    bit_depth: usize,
    color_type: u8,
    plte: Option<&[u8]>,
    trns: Option<&[u8]>,
) -> Result<u32, Error> {
    if color_type == 3 {
        let plte = plte.unwrap();
        let idx = sample_index(row, x, bit_depth)?;
        if idx * 3 + 2 >= plte.len() {
            return Err(Error::BadToken { token: format!("idx{idx}"), reason: "调色板索引越界" });
        }
        let r = plte[idx * 3] as u32;
        let g = plte[idx * 3 + 1] as u32;
        let b = plte[idx * 3 + 2] as u32;
        let a = match trns {
            Some(t) if idx < t.len() => t[idx] as u32,
            _ => 0xFF,
        };
        return Ok((a << 24) | (r << 16) | (g << 8) | b);
    }
    if bit_depth != 8 {
        return Err(Error::BadToken {
            token: format!("depth{bit_depth}"),
            reason: "非调色板 PNG 仅支持 bit depth 8",
        });
    }
    Ok(match color_type {
        0 => {
            let g = row[x] as u32;
            let key = trns
                .filter(|t| t.len() >= 2)
                .map(|t| (t[0] as u32) << 8 | t[1] as u32)
                .unwrap_or(u32::MAX);
            let a = if g == key { 0 } else { 0xFF };
            (a << 24) | (g << 16) | (g << 8) | g
        }
        2 => {
            let o = x * 3;
            let r = row[o] as u32;
            let g = row[o + 1] as u32;
            let b = row[o + 2] as u32;
            let mut a = 0xFF;
            if let Some(t) = trns {
                if t.len() >= 6 {
                    let tr = (t[0] as u32) << 8 | t[1] as u32;
                    let tg = (t[2] as u32) << 8 | t[3] as u32;
                    let tb = (t[4] as u32) << 8 | t[5] as u32;
                    if tr == r && tg == g && tb == b {
                        a = 0;
                    }
                }
            }
            (a << 24) | (r << 16) | (g << 8) | b
        }
        4 => {
            let o = x * 2;
            let g = row[o] as u32;
            let a = row[o + 1] as u32;
            (a << 24) | (g << 16) | (g << 8) | g
        }
        6 => {
            let o = x * 4;
            let r = row[o] as u32;
            let g = row[o + 1] as u32;
            let b = row[o + 2] as u32;
            let a = row[o + 3] as u32;
            (a << 24) | (r << 16) | (g << 8) | b
        }
        _ => return Err(Error::BadToken { token: format!("ct{color_type}"), reason: "PNG color-type 非法" }),
    })
}

/// 子字节采样（`Image.sampleIndex`，`Image.java:257-265`）。
fn sample_index(row: &[u8], x: usize, bit_depth: usize) -> Result<usize, Error> {
    Ok(match bit_depth {
        8 => row[x] as usize,
        4 => ((row[x >> 1] >> if x & 1 == 0 { 4 } else { 0 }) & 0x0F) as usize,
        2 => ((row[x >> 2] >> (6 - 2 * (x & 3))) & 0x03) as usize,
        1 => ((row[x >> 3] >> (7 - (x & 7))) & 0x01) as usize,
        _ => return Err(Error::BadToken { token: format!("depth{bit_depth}"), reason: "PNG bit-depth 非法" }),
    })
}
