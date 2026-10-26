//! SHA-256（FIPS 180-4）纯确定性实现。
//!
//! 消费方：oracle FRAME 哈希对拍（TickHooks.afterPaint 同流）、渲染 golden。
//! 自实现理由：三方 crate（platform/core/data）零运行时依赖约定下共享同一实现，
//! 且 Java 侧 `MessageDigest` 语义（`update` 逐字节喂入）可直接对齐。

/// SHA-256 增量上下文。
#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256 {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buf: [0; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    /// `MessageDigest.update` 语义（可多次调用）。
    pub fn update(&mut self, mut data: &[u8]) {
        self.total_len += data.len() as u64;
        if self.buf_len > 0 {
            let need = 64 - self.buf_len;
            let take = need.min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        while data.len() >= 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[..64]);
            self.compress(&block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    /// 32 字节摘要（上下文可继续 clone 使用）。
    pub fn finalize(&self) -> [u8; 32] {
        let mut st = self.clone();
        let bit_len = self.total_len.wrapping_mul(8);
        // padding：0x80 + 零填充至 56 + 64bit 大端总长（FIPS 180-4 §5.1.1）
        st.buf[st.buf_len] = 0x80;
        st.buf_len += 1;
        if st.buf_len > 56 {
            for i in st.buf_len..64 {
                st.buf[i] = 0;
            }
            let block = st.buf;
            st.compress(&block);
            st.buf_len = 0;
        }
        for i in st.buf_len..56 {
            st.buf[i] = 0;
        }
        st.buf[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let block = st.buf;
        st.compress(&block);
        let mut out = [0u8; 32];
        for (i, w) in st.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
        }
        out
    }

    fn compress(&mut self, block: &[u8; 64]) {
        // 【门禁提速 2026-10-26】x86_64 SHA-NI 硬件路径（运行时探测；同一 FIPS 180-4
        // 压缩函数，逐字节等价钉 accelerated_matches_portable）——其余平台/无 SHA-NI
        // 走可移植实现（Android/wasm 编译面零变化）。
        #[cfg(target_arch = "x86_64")]
        {
            if shani::available() {
                // SAFETY: available() 已运行时确认 sha/sse2/ssse3/sse4.1。
                unsafe { shani::compress(&mut self.state, block) };
                return;
            }
        }
        compress_portable(&mut self.state, block);
    }
}

/// 可移植压缩函数（FIPS 180-4 §6.2.2）——加速路径的参照实现。
fn compress_portable(state: &mut [u32; 8], block: &[u8; 64]) {
    {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        let add = [a, b, c, d, e, f, g, h];
        for (s, v) in state.iter_mut().zip(add) {
            *s = s.wrapping_add(v);
        }
    }
}

/// 可移植路径一次性摘要（加速路径等价钉的参照）。
#[cfg(test)]
fn portable_sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    let mut st = h.state;
    let mut chunks = data.chunks_exact(64);
    for c in &mut chunks {
        compress_portable(&mut st, c.try_into().unwrap());
    }
    h.state = st;
    h.total_len = (data.len() - chunks.remainder().len()) as u64;
    // 尾块/填充：update 剩余字节后 finalize（finalize 内 compress 走分派路径——
    // 为保参照纯净，这里手工做填充并全程用可移植压缩）
    let rem = chunks.remainder();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut tail = [0u8; 128];
    tail[..rem.len()].copy_from_slice(rem);
    tail[rem.len()] = 0x80;
    let n = if rem.len() + 1 > 56 { 128 } else { 64 };
    tail[n - 8..n].copy_from_slice(&bit_len.to_be_bytes());
    let mut st = h.state;
    for c in tail[..n].chunks_exact(64) {
        compress_portable(&mut st, c.try_into().unwrap());
    }
    let mut out = [0u8; 32];
    for (i, w) in st.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
    }
    out
}

/// 加速路径是否可用（测试面 SKIP 标注用）。
#[cfg(test)]
fn accelerated_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        shani::available()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// x86_64 SHA-NI 压缩（Intel SHA Extensions 标准序列：sha256rnds2 两轮/条 + msg1/msg2
/// 消息调度；state 按 ABEF/CDGH 打包）。
#[cfg(target_arch = "x86_64")]
mod shani {
    use std::arch::x86_64::*;

    pub fn available() -> bool {
        is_x86_feature_detected!("sha")
            && is_x86_feature_detected!("sse2")
            && is_x86_feature_detected!("ssse3")
            && is_x86_feature_detected!("sse4.1")
    }

    #[target_feature(enable = "sha,sse2,ssse3,sse4.1")]
    pub unsafe fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
        // 大端字节序换位掩码（消息字 = 大端 u32）
        let mask = _mm_set_epi64x(0x0c0d_0e0f_0809_0a0b, 0x0405_0607_0001_0203);
        let k = |i: usize| _mm_loadu_si128(super::K.as_ptr().add(i) as *const __m128i);

        // state → ABEF / CDGH
        let mut tmp = _mm_loadu_si128(state.as_ptr() as *const __m128i); // DCBA
        let mut s1 = _mm_loadu_si128(state.as_ptr().add(4) as *const __m128i); // HGFE
        tmp = _mm_shuffle_epi32(tmp, 0xB1); // CDAB
        s1 = _mm_shuffle_epi32(s1, 0x1B); // EFGH
        let mut s0 = _mm_alignr_epi8(tmp, s1, 8); // ABEF
        s1 = _mm_blend_epi16(s1, tmp, 0xF0); // CDGH
        let (abef_save, cdgh_save) = (s0, s1);

        let p = block.as_ptr() as *const __m128i;
        let mut m0 = _mm_shuffle_epi8(_mm_loadu_si128(p), mask);
        let mut m1 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(1)), mask);
        let mut m2 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(2)), mask);
        let mut m3 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(3)), mask);

        // 16 组 × 4 轮（每组：两次 sha256rnds2；第 4..15 组含消息扩展）
        macro_rules! rounds4 {
            ($m:expr, $i:expr) => {{
                let msg = _mm_add_epi32($m, k($i));
                s1 = _mm_sha256rnds2_epu32(s1, s0, msg);
                s0 = _mm_sha256rnds2_epu32(s0, s1, _mm_shuffle_epi32(msg, 0x0E));
            }};
        }
        // 消息扩展：W[i..i+4] 由 m_a(W[i-16..]), m_b(W[i-12..]), m_c(W[i-8..]), m_d(W[i-4..])
        macro_rules! schedule {
            ($a:ident, $b:ident, $c:ident, $d:ident) => {{
                let t = _mm_add_epi32(_mm_sha256msg1_epu32($a, $b), _mm_alignr_epi8($d, $c, 4));
                $a = _mm_sha256msg2_epu32(t, $d);
            }};
        }
        rounds4!(m0, 0);
        rounds4!(m1, 4);
        rounds4!(m2, 8);
        rounds4!(m3, 12);
        for g in 1..4 {
            let base = g * 16;
            schedule!(m0, m1, m2, m3);
            rounds4!(m0, base);
            schedule!(m1, m2, m3, m0);
            rounds4!(m1, base + 4);
            schedule!(m2, m3, m0, m1);
            rounds4!(m2, base + 8);
            schedule!(m3, m0, m1, m2);
            rounds4!(m3, base + 12);
        }

        s0 = _mm_add_epi32(s0, abef_save);
        s1 = _mm_add_epi32(s1, cdgh_save);

        // ABEF / CDGH → state
        tmp = _mm_shuffle_epi32(s0, 0x1B); // FEBA
        s1 = _mm_shuffle_epi32(s1, 0xB1); // DCHG
        s0 = _mm_blend_epi16(tmp, s1, 0xF0); // DCBA
        s1 = _mm_alignr_epi8(s1, tmp, 8); // HGFE
        _mm_storeu_si128(state.as_mut_ptr() as *mut __m128i, s0);
        _mm_storeu_si128(state.as_mut_ptr().add(4) as *mut __m128i, s1);
    }
}

/// 一次性摘要。
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize()
}

/// 十六进制小写串。
pub fn sha256_hex(data: &[u8]) -> String {
    hex(&sha256(data))
}

pub fn hex(digest: &[u8]) -> String {
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIPS 180-4 与 NIST 规范向量。
    #[test]
    fn fips_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    /// 多段 update 与一次性一致（TickHooks 逐字节喂入路径）。
    #[test]
    fn chunked_update_matches() {
        let data: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
        let one = sha256(&data);
        let mut h = Sha256::new();
        for chunk in data.chunks(7) {
            h.update(chunk);
        }
        assert_eq!(h.finalize(), one);
    }

    /// 【门禁提速 2026-10-26】硬件加速路径（x86 SHA-NI）与可移植实现**逐字节等价**：
    /// 0..2000 全长度 + 帧级长度（240×320×4、576²×4）+ 多段 update 切分；无 SHA-NI 的
    /// 机器=加速路径不可用，断言只覆盖可移植路径（显式打印 SKIP 半边）。
    #[test]
    fn accelerated_matches_portable() {
        let mut seed = 0x2026_1026u32;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as u8
        };
        let lens: Vec<usize> = (0..2000).chain([240 * 320 * 4, 576 * 576 * 4]).collect();
        if !accelerated_available() {
            eprintln!("SKIP(半边)：本机无 SHA-NI——仅可移植路径参与断言");
        }
        for len in lens {
            let data: Vec<u8> = (0..len).map(|_| next()).collect();
            let portable = portable_sha256(&data);
            assert_eq!(sha256(&data), portable, "len={len}：加速/分派路径 ≠ 可移植");
            let mut h = Sha256::new();
            for chunk in data.chunks(1 + len % 97) {
                h.update(chunk);
            }
            assert_eq!(h.finalize(), portable, "len={len}：多段 update ≠ 可移植");
        }
    }
}
