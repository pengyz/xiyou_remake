//! maplv 族中间态（schema：见 [`super`] 模块文档）。
//!
//! 二进制（[`crate::maplv`]）：`u16 LE raw_width, raw_height`（= 真实尺寸 ×2）+
//! terrain[Y·Z·4] + flip[Y·Z·4]。JSON 存 **raw 原始值**（可能为奇数 ⇒ 不得由
//! `width<<1` 反推）与两网格字节数组（元素 0..=255）。

use super::json::{self, Json};
use super::{Error, Mid};

/// 二进制 → 中间态 JSON（canonical 写出）。
pub(crate) fn unpack(name: &str, bytes: &[u8]) -> Result<Mid, Error> {
    let m = crate::maplv::Maplv::parse(bytes)?;
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"version\": 1,\n");
    out.push_str("  \"format\": \"maplv\",\n");
    out.push_str(&format!("  \"name\": {},\n", json::quote(name)));
    out.push_str(&format!("  \"raw_width\": {},\n", m.raw_width));
    out.push_str(&format!("  \"raw_height\": {},\n", m.raw_height));
    json::push_int_arr(&mut out, "terrain", &m.terrain, true);
    json::push_int_arr(&mut out, "flip", &m.flip, false);
    out.push_str("}\n");
    Ok(Mid { json: out, images: Vec::new() })
}

/// 中间态 JSON → 二进制（与 unpack 输入逐字节恒等）。
pub(crate) fn repack(doc: &Json) -> Result<Vec<u8>, Error> {
    let name = doc.need_str("name").map_err(Error::msg)?;
    let raw_width = u16_field(doc, "raw_width")?;
    let raw_height = u16_field(doc, "raw_height")?;
    let terrain = bytes_field(doc, "terrain")?;
    let flip = bytes_field(doc, "flip")?;
    let y = (raw_width >> 1) as usize;
    let z = (raw_height >> 1) as usize;
    let grid = y * z * 4;
    if terrain.len() != grid || flip.len() != grid {
        return Err(Error::msg(format!(
            "{name}: 网格长度不符（terrain {} / flip {}，期望各 {grid} = Y·Z·4）",
            terrain.len(),
            flip.len()
        )));
    }
    let mut out = Vec::with_capacity(4 + 2 * grid);
    out.extend_from_slice(&raw_width.to_le_bytes());
    out.extend_from_slice(&raw_height.to_le_bytes());
    out.extend_from_slice(&terrain);
    out.extend_from_slice(&flip);
    Ok(out)
}

fn u16_field(doc: &Json, key: &str) -> Result<u16, Error> {
    let v = doc.need_int(key).map_err(Error::msg)?;
    u16::try_from(v).map_err(|_| Error::msg(format!("字段 {key} 超出 u16：{v}")))
}

/// 整数数组字段 → 字节（元素必须 0..=255）。
fn bytes_field(doc: &Json, key: &str) -> Result<Vec<u8>, Error> {
    let arr = doc.need_arr(key).map_err(Error::msg)?;
    arr.iter()
        .map(|v| {
            let n = v.as_int().ok_or_else(|| Error::msg(format!("字段 {key} 含非整数元素")))?;
            u8::try_from(n).map_err(|_| Error::msg(format!("字段 {key} 元素超出 u8：{n}")))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_bin() -> Vec<u8> {
        // 头 (26,26) = 13×13，两网格各 676 字节
        let mut terrain = vec![0u8; 676];
        let mut flip = vec![0u8; 676];
        terrain[0] = 42;
        terrain[675] = 7;
        flip[3] = 1;
        let mut bin = vec![0x1a, 0x00, 0x1a, 0x00];
        bin.extend_from_slice(&terrain);
        bin.extend_from_slice(&flip);
        bin
    }

    /// 合成样例往返恒等 + canonical 信封键序。
    #[test]
    fn roundtrip_identity_and_canonical_keys() {
        let bin = synthetic_bin();
        let mid = unpack("maplvT", &bin).unwrap();
        assert!(mid.images.is_empty());
        // canonical 键序（super 文档 schema 表列序）
        let keys = ["\"version\": 1", "\"format\": \"maplv\"", "\"name\": \"maplvT\"", "\"raw_width\": 26", "\"raw_height\": 26", "\"terrain\":", "\"flip\":"];
        let mut last = 0;
        for k in keys {
            let at = mid.json.find(k).unwrap_or_else(|| panic!("缺 {k}"));
            assert!(at >= last, "键序错乱：{k}");
            last = at;
        }
        assert!(mid.json.ends_with("}\n"), "文件尾 LF + 闭括号");
        assert_eq!(repack(&json::parse(&mid.json).unwrap()).unwrap(), bin, "恒等");
    }

    /// raw 值原样保留（奇数 raw 不得被 `width<<1` 反推破坏恒等）。
    #[test]
    fn odd_raw_values_survive() {
        let mut bin = vec![0x1b, 0x00, 0x19, 0x00]; // raw 27×25 ⇒ Y·Z·4 = 13*12*4 = 624
        bin.resize(4 + 2 * 624, 3);
        let mid = unpack("maplvT", &bin).unwrap();
        assert!(mid.json.contains("\"raw_width\": 27"));
        assert!(mid.json.contains("\"raw_height\": 25"));
        assert_eq!(repack(&json::parse(&mid.json).unwrap()).unwrap(), bin);
    }

    /// 网格长度不符 / 元素越界 ⇒ 拒绝。
    #[test]
    fn validation_rejects_bad_fields() {
        let good = unpack("maplvT", &synthetic_bin()).unwrap();
        // 网格长度不符
        let bad = good.json.replace("\"terrain\": [", "\"terrain\": [0, ");
        assert!(repack(&json::parse(&bad).unwrap()).is_err(), "网格长度不符必须拒绝");
        // 元素越界 256
        let bad = good.json.replace("\"flip\": [", "\"flip\": [256, ");
        // （256 会改长度——构造独立用例）
        let mut doc = json::parse(&good.json).unwrap();
        if let Json::Obj(members) = &mut doc {
            for (k, v) in members.iter_mut() {
                if k == "flip" {
                    *v = Json::Arr(vec![Json::Int(256); 676]);
                }
            }
        }
        assert!(repack(&doc).is_err(), "元素超出 u8 必须拒绝");
        let _ = bad;
    }
}
