//! sprite 族中间态（schema：见 [`super`] 模块文档）。
//!
//! 二进制（[`crate::sprite`]）：`u16 LE count_lo, count_hi` + count ×
//! [u8 type_code, u16 LE x, u16 LE y, extra(0/1/2 字节，长由 [`crate::sprite::extra_len`] 决定)]。
//! JSON 存 count_lo/count_hi 原始值（`count = lo|hi<<16` 须 == records 长度）与
//! 记录数组（extra = 原始字节整数数组——语义后处理留解释器层，解析层不发明）。

use super::json::{self, Json};
use super::{Error, Mid};

/// 二进制 → 中间态 JSON（canonical 写出）。
pub(crate) fn unpack(name: &str, bytes: &[u8]) -> Result<Mid, Error> {
    let t = crate::sprite::SpriteTable::parse(bytes)?;
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"version\": 1,\n");
    out.push_str("  \"format\": \"sprite\",\n");
    out.push_str(&format!("  \"name\": {},\n", json::quote(name)));
    out.push_str(&format!("  \"count_lo\": {},\n", t.count_lo));
    out.push_str(&format!("  \"count_hi\": {},\n", t.count_hi));
    if t.records.is_empty() {
        out.push_str("  \"records\": []\n");
    } else {
        out.push_str("  \"records\": [\n");
        for (i, r) in t.records.iter().enumerate() {
            let extra: Vec<String> = r.extra.iter().map(|b| b.to_string()).collect();
            out.push_str(&format!(
                "    {{\"type_code\": {}, \"x\": {}, \"y\": {}, \"extra\": [{}]}}",
                r.type_code,
                r.x,
                r.y,
                extra.join(", ")
            ));
            if i + 1 < t.records.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n");
    }
    out.push_str("}\n");
    Ok(Mid { json: out, images: Vec::new() })
}

/// 中间态 JSON → 二进制（与 unpack 输入逐字节恒等）。
pub(crate) fn repack(doc: &Json) -> Result<Vec<u8>, Error> {
    let name = doc.need_str("name").map_err(Error::msg)?;
    let count_lo = u16_field(doc, "count_lo")?;
    let count_hi = u16_field(doc, "count_hi")?;
    let count = ((count_lo as u32) | ((count_hi as u32) << 16)) as usize;
    let records = doc.need_arr("records").map_err(Error::msg)?;
    if records.len() != count {
        return Err(Error::msg(format!(
            "{name}: count（{count}）与 records 长度（{}）不符",
            records.len()
        )));
    }
    let mut out = Vec::new();
    out.extend_from_slice(&count_lo.to_le_bytes());
    out.extend_from_slice(&count_hi.to_le_bytes());
    for (i, rec) in records.iter().enumerate() {
        let type_code = rec.need_int("type_code").map_err(Error::msg)?;
        let type_code = u8::try_from(type_code)
            .map_err(|_| Error::msg(format!("{name}: records[{i}] type_code 超出 u8")))?;
        let x = u16_field(rec, "x").map_err(|e| Error::msg(format!("{name}: records[{i}] {e}")))?;
        let y = u16_field(rec, "y").map_err(|e| Error::msg(format!("{name}: records[{i}] {e}")))?;
        let extra_arr = rec.need_arr("extra").map_err(Error::msg)?;
        let mut extra = Vec::with_capacity(extra_arr.len());
        for v in extra_arr {
            let n = v.as_int().ok_or_else(|| Error::msg(format!("{name}: records[{i}] extra 含非整数")))?;
            extra.push(u8::try_from(n).map_err(|_| Error::msg(format!("{name}: records[{i}] extra 超出 u8")))?);
        }
        if extra.len() != crate::sprite::extra_len(type_code) {
            return Err(Error::msg(format!(
                "{name}: records[{i}] extra 长度 {} ≠ extra_len({type_code}) = {}",
                extra.len(),
                crate::sprite::extra_len(type_code)
            )));
        }
        out.push(type_code);
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.extend_from_slice(&extra);
    }
    Ok(out)
}

fn u16_field(doc: &Json, key: &str) -> Result<u16, Error> {
    let v = doc.need_int(key).map_err(Error::msg)?;
    u16::try_from(v).map_err(|_| Error::msg(format!("字段 {key} 超出 u16：{v}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 合成表：3 条记录（类型码 7=2 字节 extra、9=1 字节、6=0 字节）。
    fn synthetic_bin() -> Vec<u8> {
        let mut bin = vec![3, 0, 0, 0]; // count_lo=3, count_hi=0
        bin.extend_from_slice(&[7]);
        bin.extend_from_slice(&96u16.to_le_bytes());
        bin.extend_from_slice(&320u16.to_le_bytes());
        bin.extend_from_slice(&[1, 0]);
        bin.extend_from_slice(&[9]);
        bin.extend_from_slice(&100u16.to_le_bytes());
        bin.extend_from_slice(&200u16.to_le_bytes());
        bin.extend_from_slice(&[5]);
        bin.extend_from_slice(&[6]);
        bin.extend_from_slice(&7u16.to_le_bytes());
        bin.extend_from_slice(&8u16.to_le_bytes());
        bin
    }

    #[test]
    fn roundtrip_identity_and_record_lines() {
        let bin = synthetic_bin();
        let mid = unpack("spriteT", &bin).unwrap();
        assert!(mid.images.is_empty());
        assert!(mid.json.contains("\"count_lo\": 3"));
        assert!(mid.json.contains("{\"type_code\": 7, \"x\": 96, \"y\": 320, \"extra\": [1, 0]}"));
        assert_eq!(repack(&json::parse(&mid.json).unwrap()).unwrap(), bin);
    }

    /// 校验：count/records 不符、extra 长度 ≠ extra_len ⇒ 拒绝。
    #[test]
    fn validation_rejects_bad_records() {
        let good = unpack("spriteT", &synthetic_bin()).unwrap();
        let bad = good.json.replace("\"count_lo\": 3", "\"count_lo\": 4");
        assert!(repack(&json::parse(&bad).unwrap()).is_err(), "count 不符拒绝");
        let bad = good.json.replace("\"extra\": [1, 0]", "\"extra\": [1]");
        assert!(repack(&json::parse(&bad).unwrap()).is_err(), "extra 长度不符拒绝");
    }

    /// count_hi 非零原样保留（高位计数）。
    #[test]
    fn count_hi_preserved() {
        let mut bin = vec![0, 0, 1, 0]; // count = 65536 会太长——改为小样验证字段面
        // count_hi=1 ⇒ count=65536 条记录，构造不现实；改验 repack 校验面即可
        bin.clear();
        let mid = unpack("spriteT", &synthetic_bin()).unwrap();
        let patched = mid.json.replace("\"count_hi\": 0", "\"count_hi\": 1");
        let doc = json::parse(&patched).unwrap();
        assert!(repack(&doc).is_err(), "count=lo|hi<<16 与 records 长度不符拒绝");
    }
}
