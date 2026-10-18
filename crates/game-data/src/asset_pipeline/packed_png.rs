//! packed_png 族中间态（schema：见 [`super`] 模块文档）。
//!
//! 二进制（[`crate::packed_png`]）：`N × [u16 LE 长度][PNG 字节]`，N 来自
//! [`crate::packed_png::RESOURCE_IMAGE_COUNTS`]（文件内不存张数）。
//! 中间态：`images` = 帧序文件名清单（`<name>_<i:02>.png`，两位零填起），
//! [`Mid::images`] 携**逐字节保留**的 PNG（不重编码——"复刻调色板序/帧切分/对齐"
//! 由逐字节保留 + 帧序清单承担；编辑器改图后 repack 嵌入新字节）。

use super::json::{self, Json};
use super::{Error, Mid};

/// 二进制 → 中间态（PNG 清单 + JSON）。
pub(crate) fn unpack(name: &str, bytes: &[u8]) -> Result<Mid, Error> {
    let count = crate::packed_png::image_count(name)
        .ok_or_else(|| Error::msg(format!("{name}: 不在 RESOURCE_IMAGE_COUNTS 域")))?;
    let packed = crate::packed_png::PackedPng::parse(bytes, count as usize)?;
    let mut images = Vec::with_capacity(packed.images.len());
    let mut names = Vec::with_capacity(packed.images.len());
    for (i, img) in packed.images.iter().enumerate() {
        let fname = format!("{name}_{i:02}.png");
        names.push(fname.clone());
        images.push((fname, img.bytes.to_vec()));
    }
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"version\": 1,\n");
    out.push_str("  \"format\": \"packed_png\",\n");
    out.push_str(&format!("  \"name\": {},\n", json::quote(name)));
    json::push_str_arr(&mut out, "images", &names, false);
    out.push_str("}\n");
    Ok(Mid { json: out, images })
}

/// 中间态 → 二进制（u16 LE 长度前缀 + PNG 字节逐字节重组，帧序=清单序）。
pub(crate) fn repack(doc: &Json, images: &[(String, Vec<u8>)]) -> Result<Vec<u8>, Error> {
    let name = doc.need_str("name").map_err(Error::msg)?;
    let frames = doc.need_arr("images").map_err(Error::msg)?;
    let mut out = Vec::new();
    for (i, frame) in frames.iter().enumerate() {
        let fname = frame
            .as_str()
            .ok_or_else(|| Error::msg(format!("{name}: images[{i}] 非字符串")))?;
        let (_, png) = images
            .iter()
            .find(|(n, _)| n == fname)
            .ok_or_else(|| Error::msg(format!("{name}: images[{i}] 缺图 {fname}")))?;
        if png.len() > u16::MAX as usize {
            return Err(Error::msg(format!("{name}: {fname} 超出 u16 长度前缀域（{}B）", png.len())));
        }
        if png.len() < crate::PNG_MAGIC.len() || png[..crate::PNG_MAGIC.len()] != crate::PNG_MAGIC {
            return Err(Error::msg(format!("{name}: {fname} 非 PNG（魔数不符）")));
        }
        out.extend_from_slice(&(png.len() as u16).to_le_bytes());
        out.extend_from_slice(png);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小合法 PNG 字节（魔数 + 伪负载；解析只要求魔数）。
    fn fake_png(fill: u8) -> Vec<u8> {
        let mut v = crate::PNG_MAGIC.to_vec();
        v.extend_from_slice(&[fill; 8]);
        v
    }

    fn synthetic_bin() -> Vec<u8> {
        let mut bin = Vec::new();
        for fill in [0x11u8, 0x22] {
            let png = fake_png(fill);
            bin.extend_from_slice(&(png.len() as u16).to_le_bytes());
            bin.extend_from_slice(&png);
        }
        bin
    }

    #[test]
    fn roundtrip_identity_and_frame_names() {
        // "menu" 在 RESOURCE_IMAGE_COUNTS 域（2 帧）——族 unpack 按域校验张数
        let bin = synthetic_bin();
        let mid = unpack("menu", &bin).unwrap();
        assert_eq!(
            mid.images.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            ["menu_00.png", "menu_01.png"],
            "帧名两位零填、序=帧序"
        );
        assert!(
            mid.json.contains("\"images\": [\n    \"menu_00.png\", \"menu_01.png\"\n  ]"),
            "canonical：字符串数组折行（≤16/行）"
        );
        assert_eq!(repack(&json::parse(&mid.json).unwrap(), &mid.images).unwrap(), bin);
        assert!(unpack("uiT", &bin).is_err(), "不在张数表的名字拒绝");
    }

    /// 校验：缺图 / 非 PNG 魔数 ⇒ 拒绝。
    #[test]
    fn validation_rejects_missing_or_bad_png() {
        let mid = unpack("menu", &synthetic_bin()).unwrap();
        let doc = json::parse(&mid.json).unwrap();
        let only = vec![mid.images[0].clone()];
        assert!(repack(&doc, &only).is_err(), "缺图拒绝");
        let mut bad = mid.images.clone();
        bad[1].1 = vec![0u8; 8];
        assert!(repack(&doc, &bad).is_err(), "魔数不符拒绝");
    }
}
