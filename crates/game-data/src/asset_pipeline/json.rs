//! 中间态 JSON 的最小型读写（零依赖：game-data 红线保持无第三方依赖、无 I/O）。
//!
//! 分工：**写出 = canonical**（排版/键序/数字格式全固定，见 `super` 模块文档的
//! schema 节）；**读取 = 宽松**（空白不敏感、未知字段忽略、重复键报错、仅支持
//! 整数）。写出恒 canonical ⇒ 字节稳定、git diff 可读（复核硬 5）。
//!
//! 公开面（`[`super::Json`]`/`[`super::parse_json`]` 重导出）供集成测试/对照半侧
//! 读中间态字段（decode-equivalence 等）。

/// 最小 JSON 值（中间态 schema 用整数/字符串/数组/对象；Bool/Null 仅为
/// **未知字段前向兼容**而解析——写出端永不产生）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Json {
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
    Bool(bool),
    Null,
}

impl Json {
    /// 对象取键（非对象 ⇒ None）。
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Json::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(items) => Some(items),
            _ => None,
        }
    }

    /// 必填整数字段（缺失/类型不符 ⇒ Err）。
    pub fn need_int(&self, key: &str) -> Result<i64, String> {
        self.get(key)
            .and_then(Json::as_int)
            .ok_or_else(|| format!("字段 {key} 缺失或非整数"))
    }

    /// 必填字符串字段。
    pub fn need_str(&self, key: &str) -> Result<&str, String> {
        self.get(key)
            .and_then(Json::as_str)
            .ok_or_else(|| format!("字段 {key} 缺失或非字符串"))
    }

    /// 必填数组字段。
    pub fn need_arr(&self, key: &str) -> Result<&[Json], String> {
        self.get(key)
            .and_then(Json::as_arr)
            .ok_or_else(|| format!("字段 {key} 缺失或非数组"))
    }
}

/// 字符串 → canonical JSON 字面量（转义仅 `\"` `\\` 与控制字符 `\u00XX`）。
pub(crate) fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// canonical：顶层字段的整数/字符串数组——元素 ≤16/行（行首缩进 4，元素间 `, `）。
/// 空数组内联 `[]`。`elems` 已是渲染好的元素文本（整数或 [`quote`]）。
pub(crate) fn push_wrapped_arr(out: &mut String, key: &str, elems: &[String], comma: bool) {
    if elems.is_empty() {
        out.push_str(&format!("  \"{key}\": []"));
    } else {
        out.push_str(&format!("  \"{key}\": [\n"));
        for (i, chunk) in elems.chunks(16).enumerate() {
            let line = chunk.join(", ");
            out.push_str(&format!("    {line}"));
            if (i + 1) * 16 < elems.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]");
    }
    if comma {
        out.push(',');
    }
    out.push('\n');
}

/// canonical：整数数组便捷版（元素格式化为十进制）。
pub(crate) fn push_int_arr(out: &mut String, key: &str, vals: &[u8], comma: bool) {
    let elems: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
    push_wrapped_arr(out, key, &elems, comma);
}

/// canonical：字符串数组便捷版。
#[allow(dead_code)] // C3 packed_png 族消费（images 帧名清单）
pub(crate) fn push_str_arr(out: &mut String, key: &str, vals: &[String], comma: bool) {
    let elems: Vec<String> = vals.iter().map(|v| quote(v)).collect();
    push_wrapped_arr(out, key, &elems, comma);
}

// ---------------------------------------------------------------- 读取端（宽松）

/// 宽松解析：空白不敏感、未知字段忽略（由上层字段提取器体现）、重复键报错。
pub fn parse(input: &str) -> Result<Json, String> {
    let mut p = Parser { b: input.as_bytes(), i: 0 };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(format!("JSON 尾部多余内容（偏移 {}）", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.number(),
            Some(b't') => self.lit("true", Json::Bool(true)),
            Some(b'f') => self.lit("false", Json::Bool(false)),
            Some(b'n') => self.lit("null", Json::Null),
            other => Err(format!("JSON 值起始非法（偏移 {}，{:?}）", self.i, other)),
        }
    }

    /// 字面量 `true`/`false`/`null`（未知字段前向兼容用）。
    fn lit(&mut self, word: &str, v: Json) -> Result<Json, String> {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(format!("JSON 字面量非法（偏移 {}）", self.i))
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.i += 1; // {
        let mut members: Vec<(String, Json)> = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Obj(members));
        }
        loop {
            self.ws();
            let key = self.string()?;
            if members.iter().any(|(k, _)| *k == key) {
                return Err(format!("JSON 重复键 {key}"));
            }
            self.ws();
            if self.peek() != Some(b':') {
                return Err(format!("JSON 对象缺 ':'（偏移 {}）", self.i));
            }
            self.i += 1;
            self.ws();
            let val = self.value()?;
            members.push((key, val));
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(members));
                }
                _ => return Err(format!("JSON 对象分隔非法（偏移 {}）", self.i)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.i += 1; // [
        let mut items = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(format!("JSON 数组分隔非法（偏移 {}）", self.i)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        if self.peek() != Some(b'"') {
            return Err(format!("JSON 字符串起始非法（偏移 {}）", self.i));
        }
        self.i += 1;
        let mut s = String::new();
        loop {
            let c = *self
                .b
                .get(self.i)
                .ok_or_else(|| format!("JSON 字符串未闭合（偏移 {}）", self.i))?;
            self.i += 1;
            match c {
                b'"' => return Ok(s),
                b'\\' => {
                    let e = *self
                        .b
                        .get(self.i)
                        .ok_or_else(|| format!("JSON 转义未闭合（偏移 {}）", self.i))?;
                    self.i += 1;
                    match e {
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'b' => s.push('\u{8}'),
                        b'f' => s.push('\u{c}'),
                        b'u' => {
                            let hex = self
                                .b
                                .get(self.i..self.i + 4)
                                .ok_or_else(|| format!("\\u 转义截断（偏移 {}）", self.i))?;
                            let n = u32::from_str_radix(
                                std::str::from_utf8(hex).map_err(|e| e.to_string())?,
                                16,
                            )
                            .map_err(|e| e.to_string())?;
                            self.i += 4;
                            s.push(char::from_u32(n).ok_or_else(|| format!("\\u{n:04x} 非法"))?);
                        }
                        other => return Err(format!("JSON 转义非法 \\{}", other as char)),
                    }
                }
                _ => {
                    // 非 ASCII 字节按 UTF-8 原样续接（容错：从字节重建）
                    let start = self.i - 1;
                    let len = utf8_len(c);
                    self.i = start + len;
                    let chunk = self
                        .b
                        .get(start..self.i)
                        .ok_or_else(|| format!("UTF-8 截断（偏移 {start}）"))?;
                    s.push_str(std::str::from_utf8(chunk).map_err(|e| e.to_string())?);
                }
            }
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.i += 1;
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(format!("JSON 仅支持整数（偏移 {start}）"));
        }
        let text = std::str::from_utf8(&self.b[start..self.i]).map_err(|e| e.to_string())?;
        text.parse::<i64>().map(Json::Int).map_err(|e| e.to_string())
    }
}

fn utf8_len(first: u8) -> usize {
    match first {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 写出 canonical（键序/缩进/数字/16-per-line 固定），读取宽松（空白/未知字段）。
    #[test]
    fn canonical_write_and_lenient_read() {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str("  \"version\": 1,\n");
        push_int_arr(&mut out, "cells", &[1, 2, 3], true);
        out.push_str("  \"name\": \"x\"\n}\n");
        assert_eq!(
            out,
            "{\n  \"version\": 1,\n  \"cells\": [\n    1, 2, 3\n  ],\n  \"name\": \"x\"\n}\n"
        );
        // 宽松读取：任意空白 + 未知字段（含 bool/null 值）忽略 + 重复键报错
        let v =
            parse("{ \"a\": [1, 2], \"b\": \"s\", \"unknown\": [9, 9], \"u2\": true, \"u3\": null }")
                .unwrap();
        assert_eq!(v.get("a").unwrap().as_arr().unwrap().len(), 2);
        assert_eq!(v.get("b").unwrap().as_str(), Some("s"));
        assert!(v.get("c").is_none(), "未知字段不导出到已知字段面（读取端忽略）");
        assert!(parse("{ \"a\": 1, \"a\": 2 }").is_err(), "重复键报错");
        assert!(parse("{ \"a\": 1.5 }").is_err(), "小数拒绝（schema 数字=整数）");
    }

    /// 16 元素/行折行规则。
    #[test]
    fn wrap_rule_16_per_line() {
        let mut out = String::new();
        let vals: Vec<u8> = (0..17).collect();
        push_int_arr(&mut out, "v", &vals, false);
        assert_eq!(
            out,
            "  \"v\": [\n    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,\n    16\n  ]\n"
        );
    }

    /// 字符串转义最小集。
    #[test]
    fn quote_escapes_minimal() {
        assert_eq!(quote("a\"b\\c\nd"), "\"a\\\"b\\\\c\\nd\"");
    }
}
