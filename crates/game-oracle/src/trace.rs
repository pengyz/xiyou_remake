//! oracle trace 子集解析：TICK/INPUT/FLD 行（字段声明序 0 基）。
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TickRecord {
    pub tick: u32,
    pub input: Option<String>,
    pub fields: HashMap<u32, String>,
}

pub fn parse(text: &str) -> Vec<TickRecord> {
    let mut out: Vec<TickRecord> = Vec::new();
    let mut cur: Option<TickRecord> = None;
    for line in text.lines() {
        if let Some(t) = line.strip_prefix("TICK ") {
            if let Some(c) = cur.take() {
                out.push(c);
            }
            let n = t.split_whitespace().next().unwrap_or("0");
            cur = Some(TickRecord {
                tick: n.parse().unwrap_or(0),
                input: None,
                fields: HashMap::new(),
            });
        } else if let Some(i) = line.strip_prefix("INPUT ") {
            if let Some(c) = cur.as_mut() {
                c.input = Some(i.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("  FLD ") {
            let mut it = rest.splitn(3, ' ');
            let idx: u32 = it.next().unwrap_or("999").parse().unwrap_or(999);
            let val = it.next().unwrap_or("").to_string();
            if let Some(c) = cur.as_mut() {
                c.fields.entry(idx).or_insert(val);
            }
        }
    }
    if let Some(c) = cur.take() {
        out.push(c);
    }
    out
}
