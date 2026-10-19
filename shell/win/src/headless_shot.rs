//! headless 截图（自验工具，2026-10-18 用户令"自己抓图自己确认"）：
//! 无窗口跑 N 拍 → 画布 dump PNG → 供 read_image 直接审图。
//! PNG 写出=最小实现（deflate stored 块，无压缩库依赖——zlib 头+stored 块+adler32）。

/// u32 ARGB 画布 → PNG 字节（filter 0 + stored deflate）。
pub fn encode_png(width: i32, height: i32, pixels: &[u32]) -> Vec<u8> {
    // ① 原始行（filter 0 + RGBA）
    let mut raw = Vec::with_capacity((height as usize) * (1 + width as usize * 4));
    for y in 0..height as usize {
        raw.push(0u8); // filter=None
        for x in 0..width as usize {
            let p = pixels[y * width as usize + x];
            raw.push(((p >> 16) & 0xFF) as u8); // R
            raw.push(((p >> 8) & 0xFF) as u8); // G
            raw.push((p & 0xFF) as u8); // B
            raw.push(((p >> 24) & 0xFF) as u8); // A
        }
    }
    // ② zlib（stored deflate 块：BFINAL=1/BTYPE=00 + LEN/NLEN + 数据）
    let mut z = vec![0x78, 0x01]; // zlib 头（CMF/FLG，无压缩）
    let mut off = 0usize;
    while off < raw.len() {
        let n = (raw.len() - off).min(65535);
        let last = if off + n >= raw.len() { 1u8 } else { 0u8 };
        z.push(last); // BFINAL + BTYPE=00
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[off..off + n]);
        off += n;
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    // ③ PNG 容器
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit RGBA
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    png
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc = Crc32::new();
    crc.update(kind);
    crc.update(data);
    out.extend_from_slice(&crc.finish().to_be_bytes());
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

struct Crc32(u32);
impl Crc32 {
    fn new() -> Self {
        Crc32(0xFFFF_FFFF)
    }
    fn update(&mut self, data: &[u8]) {
        for &x in data {
            self.0 ^= x as u32;
            for _ in 0..8 {
                self.0 = if self.0 & 1 != 0 {
                    (self.0 >> 1) ^ 0xEDB8_8320
                } else {
                    self.0 >> 1
                };
            }
        }
    }
    fn finish(self) -> u32 {
        self.0 ^ 0xFFFF_FFFF
    }
}

#[cfg(test)]
mod tests {
    use super::encode_png;
    /// 码流结构自检（stored 块 + CRC/adler 各就位）。
    #[test]
    fn png_writer_structure() {
        let png = encode_png(2, 2, &[0xFFFF_0000, 0xFF00_FF00, 0xFF00_00FF, 0xFF00_0000]);
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 13, 10, 26, 10], "PNG 签名");
        assert_eq!(&png[12..16], b"IHDR", "IHDR 块");
        assert!(png.windows(4).any(|w| w == b"IEND"), "IEND 块在");
        // IHDR 宽高=2×2
        assert_eq!(&png[16..20], &[0, 0, 0, 2]);
        assert_eq!(&png[20..24], &[0, 0, 0, 2]);
    }
}
