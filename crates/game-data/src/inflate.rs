//! zlib（RFC 1950）/ deflate（RFC 1951）纯确定性解压——PNG `IDAT` 的解压依赖。
//!
//! 零依赖自实现（game-data 红线：不做 I/O、不引外部 crate）。解压输出由 zlib 规范
//! 唯一确定，任何正确实现的输出逐字节一致；`adler32` 校验保证没解错。
//!
//! 算法骨架移植自 Mark Adler 的参考实现 puff（公有领域）的规范式解码：
//! 规范 Huffman（code-length → 计数表 + 符号表）逐位解码，天然覆盖
//! stored / fixed / dynamic 全部三种块型。

use crate::error::Error;

/// zlib 流解压（含头校验与 adler32 尾校验）。
pub fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() < 6 {
        return Err(Error::UnexpectedEof { offset: 0, needed: 6, available: data.len() });
    }
    let cmf = data[0];
    let flg = data[1];
    if cmf & 0x0F != 8 {
        return Err(Error::BadToken { token: format!("cmf={cmf:02x}"), reason: "zlib: 非 deflate 压缩法" });
    }
    if ((cmf as u16) * 256 + flg as u16) % 31 != 0 {
        return Err(Error::BadToken { token: format!("cmf={cmf:02x} flg={flg:02x}"), reason: "zlib: 头校验和非法" });
    }
    if flg & 0x20 != 0 {
        return Err(Error::BadToken { token: "fdict".into(), reason: "zlib: 不支持预设字典" });
    }
    let mut out = Vec::with_capacity(data.len() * 4);
    let consumed = inflate(&data[2..], &mut out)?;
    // 尾部 4 字节 adler32（大端）
    let tail = 2 + consumed;
    if tail + 4 > data.len() {
        return Err(Error::UnexpectedEof { offset: tail, needed: 4, available: data.len() - tail });
    }
    let expect = u32::from_be_bytes([data[tail], data[tail + 1], data[tail + 2], data[tail + 3]]);
    let actual = adler32(&out);
    if expect != actual {
        return Err(Error::BadToken {
            token: format!("expect={expect:08x} actual={actual:08x}"),
            reason: "zlib: adler32 校验失败",
        });
    }
    Ok(out)
}

/// deflate 原始流解压，返回消费的字节数。
fn inflate(data: &[u8], out: &mut Vec<u8>) -> Result<usize, Error> {
    let mut br = BitReader { data, pos: 0, bitbuf: 0, bitcnt: 0 };
    loop {
        let bfinal = br.bits(1)?;
        let btype = br.bits(2)?;
        match btype {
            0 => {
                // stored：回到字节边界，LEN/NLEN 各 2 字节（LE）
                br.align_byte()?;
                let pos = br.pos;
                if pos + 4 > data.len() {
                    return Err(Error::UnexpectedEof { offset: pos, needed: 4, available: data.len() - pos });
                }
                let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                let nlen = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
                if len != (!nlen & 0xFFFF) {
                    return Err(Error::BadToken { token: format!("len={len} nlen={nlen}"), reason: "deflate stored: LEN/NLEN 不互反" });
                }
                br.pos = pos + 4;
                let chunk = br.take(len)?;
                out.extend_from_slice(chunk);
            }
            1 => {
                let (lit, dist) = fixed_tables();
                inflate_block(&mut br, out, &lit, &dist)?;
            }
            2 => {
                let (lit, dist) = dynamic_tables(&mut br)?;
                inflate_block(&mut br, out, &lit, &dist)?;
            }
            _ => return Err(Error::BadToken { token: "btype=3".into(), reason: "deflate: 非法块型 3" }),
        }
        if bfinal == 1 {
            break;
        }
    }
    // 返回 deflate 部分实际消费的字节数（位缓冲中未消费的整字节不计）
    let buffered_bytes = (br.bitcnt / 8) as usize;
    Ok(br.pos - buffered_bytes)
}

struct BitReader<'d> {
    data: &'d [u8],
    /// 下一整字节的下标
    pos: usize,
    /// 低位在前的位缓冲
    bitbuf: u32,
    /// 缓冲中的有效位数
    bitcnt: u32,
}

impl<'d> BitReader<'d> {
    fn bits(&mut self, need: u32) -> Result<u32, Error> {
        while self.bitcnt < need {
            if self.pos >= self.data.len() {
                return Err(Error::UnexpectedEof { offset: self.pos, needed: 1, available: 0 });
            }
            self.bitbuf |= (self.data[self.pos] as u32) << self.bitcnt;
            self.pos += 1;
            self.bitcnt += 8;
        }
        let val = self.bitbuf & ((1u32 << need) - 1);
        self.bitbuf >>= need;
        self.bitcnt -= need;
        Ok(val)
    }

    fn align_byte(&mut self) -> Result<(), Error> {
        // 先把缓冲里多读的整字节退回数据流（puff 做法），再丢弃不足 1 字节的填充位
        while self.bitcnt >= 8 {
            self.pos -= 1;
            self.bitcnt -= 8;
        }
        self.bitbuf = 0;
        self.bitcnt = 0;
        Ok(())
    }

    fn take(&mut self, n: usize) -> Result<&'d [u8], Error> {
        // stored 块读取前已对齐：bitcnt 应为 0
        debug_assert_eq!(self.bitcnt, 0);
        if self.pos + n > self.data.len() {
            return Err(Error::UnexpectedEof { offset: self.pos, needed: n, available: self.data.len() - self.pos });
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
}

/// 规范 Huffman 表：每码长的符号数 + 按码长排序的符号表（puff 结构）。
struct Huffman {
    count: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn build(lengths: &[u8]) -> Result<Huffman, Error> {
        let mut count = [0u16; 16];
        for &l in lengths {
            count[l as usize] += 1;
        }
        count[0] = 0;
        // 过订阅检查（puff）：每种码长可用码位数不得为负
        let mut left = 1i32;
        for len in 1..=15 {
            left <<= 1;
            left -= count[len] as i32;
            if left < 0 {
                return Err(Error::BadToken { token: format!("len={len} left={left}"), reason: "deflate: Huffman 码过订阅" });
            }
        }
        let mut offs = [0u16; 16];
        for len in 1..15 {
            offs[len + 1] = offs[len] + count[len];
        }
        let mut symbols = vec![0u16; lengths.iter().filter(|&&l| l != 0).count()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offs[l as usize] as usize] = sym as u16;
                offs[l as usize] += 1;
            }
        }
        Ok(Huffman { count, symbols })
    }

    fn decode(&self, br: &mut BitReader<'_>) -> Result<u16, Error> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for len in 1..=15 {
            code |= br.bits(1)? as i32;
            let cnt = self.count[len] as i32;
            if code - first < cnt {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += cnt;
            first = (first + cnt) << 1;
            code <<= 1;
        }
        Err(Error::BadToken { token: "huffman".into(), reason: "deflate: 码长超出 15 位" })
    }
}

/// 固定 Huffman 表（RFC 1951 §3.2.6）。
fn fixed_tables() -> (Huffman, Huffman) {
    let mut lit_lens = [0u8; 288];
    for (i, l) in lit_lens.iter_mut().enumerate() {
        *l = match i {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }
    let dist_lens = [5u8; 30];
    // 固定表构造不可能失败
    (Huffman::build(&lit_lens).unwrap(), Huffman::build(&dist_lens).unwrap())
}

/// 动态 Huffman 表头（RFC 1951 §3.2.7）。
fn dynamic_tables(br: &mut BitReader<'_>) -> Result<(Huffman, Huffman), Error> {
    const ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let hlit = br.bits(5)? as usize + 257;
    let hdist = br.bits(5)? as usize + 1;
    let hclen = br.bits(4)? as usize + 4;
    let mut clen_lens = [0u8; 19];
    for i in 0..hclen {
        clen_lens[ORDER[i]] = br.bits(3)? as u8;
    }
    let clen = Huffman::build(&clen_lens)?;
    let mut lens = vec![0u8; hlit + hdist];
    let mut i = 0;
    while i < hlit + hdist {
        let sym = clen.decode(br)?;
        match sym {
            0..=15 => {
                lens[i] = sym as u8;
                i += 1;
            }
            16 => {
                if i == 0 {
                    return Err(Error::BadToken { token: "cl16".into(), reason: "deflate: 重复码无前值" });
                }
                let prev = lens[i - 1];
                let rep = 3 + br.bits(2)? as usize;
                for _ in 0..rep {
                    if i >= lens.len() {
                        return Err(Error::BadToken { token: "cl16".into(), reason: "deflate: 重复码越界" });
                    }
                    lens[i] = prev;
                    i += 1;
                }
            }
            17 => {
                let rep = 3 + br.bits(3)? as usize;
                i += rep;
                if i > lens.len() {
                    return Err(Error::BadToken { token: "cl17".into(), reason: "deflate: 零重复越界" });
                }
            }
            18 => {
                let rep = 11 + br.bits(7)? as usize;
                i += rep;
                if i > lens.len() {
                    return Err(Error::BadToken { token: "cl18".into(), reason: "deflate: 零重复越界" });
                }
            }
            _ => return Err(Error::BadToken { token: format!("cl{sym}"), reason: "deflate: 非法码长符号" }),
        }
    }
    if lens[256] == 0 {
        return Err(Error::BadToken { token: "eos".into(), reason: "deflate: 缺块终止码" });
    }
    Ok((Huffman::build(&lens[..hlit])?, Huffman::build(&lens[hlit..])?))
}

/// 单个 Huffman 块的 LZ77 解码（长度/距离码表 RFC 1951 §3.2.5）。
fn inflate_block(br: &mut BitReader<'_>, out: &mut Vec<u8>, lit: &Huffman, dist: &Huffman) -> Result<(), Error> {
    const LEN_BASE: [u16; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LEN_EXTRA: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [u16; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];
    loop {
        let sym = lit.decode(br)?;
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            257..=285 => {
                let idx = sym as usize - 257;
                let len = LEN_BASE[idx] as usize + br.bits(LEN_EXTRA[idx] as u32)? as usize;
                let dsym = dist.decode(br)? as usize;
                if dsym >= 30 {
                    return Err(Error::BadToken { token: format!("dist{dsym}"), reason: "deflate: 非法距离码" });
                }
                let d = DIST_BASE[dsym] as usize + br.bits(DIST_EXTRA[dsym] as u32)? as usize;
                if d > out.len() {
                    return Err(Error::BadToken { token: format!("dist{d}"), reason: "deflate: 距离超出输出窗口" });
                }
                let start = out.len() - d;
                for k in 0..len {
                    let b = out[start + k];
                    out.push(b);
                }
            }
            _ => return Err(Error::BadToken { token: format!("sym{sym}"), reason: "deflate: 非法字面码" }),
        }
    }
}

/// adler32（RFC 1950 §8）。
pub fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    /// zlib empty-stream 规范向量（python3 -c "import zlib;print(zlib.compress(b'').hex())"）。
    #[test]
    fn empty_stream() {
        let compressed = [0x78, 0x9C, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01];
        assert_eq!(zlib_decompress(&compressed).unwrap(), Vec::<u8>::new());
    }

    /// stored 块规范向量：`zlib.compress(b'abc', 0)` 产物。
    #[test]
    fn stored_stream() {
        let compressed = [
            0x78, 0x01, 0x01, 0x03, 0x00, 0xFC, 0xFF, b'a', b'b', b'c', 0x02, 0x4D, 0x01, 0x27,
        ];
        assert_eq!(zlib_decompress(&compressed).unwrap(), b"abc".to_vec());
    }

    /// fixed-huffman 规范向量：`zlib.compress(b'abc', 9)` 走固定/动态块。
    #[test]
    fn fixed_or_dynamic_stream() {
        let compressed = [0x78, 0xDA, 0x4B, 0x4C, 0x4A, 0x06, 0x00, 0x02, 0x4D, 0x01, 0x27];
        assert_eq!(zlib_decompress(&compressed).unwrap(), b"abc".to_vec());
    }

    /// 动态 huffman + 重复串（LZ77 回引）：`zlib.compress(b'hello world'*10, 9)`。
    #[test]
    fn dynamic_lz77_stream() {
        let compressed: [u8; 24] = [
            0x78, 0xDA, 0xCB, 0x48, 0xCD, 0xC9, 0xC9, 0x57, 0x28, 0xCF, 0x2F, 0xCA, 0x49, 0xC9,
            0xA0, 0x3D, 0x13, 0x00, 0x72, 0xD9, 0x2B, 0x99, 0x00, 0x00,
        ];
        assert_eq!(
            zlib_decompress(&compressed).unwrap(),
            b"hello world".repeat(10)
        );
    }

    /// adler32 规范向量：adler32(b"abc") = 0x024d0127。
    #[test]
    fn adler32_vector() {
        assert_eq!(adler32(b"abc"), 0x024D0127);
        assert_eq!(adler32(b""), 1);
    }
}
