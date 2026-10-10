//! PNG 解码 golden（L1b-px）：`game-data::png` 与 Java shim `Image.decodePng`
//! 对**原版全部 164 张资源 PNG**（3 独立 + 16 packed 容器全部子图）产出
//! 逐字节一致的 ARGB 哈希。
//!
//! 裁判：`data/golden/render-golden.json` 的 `images` 段
//! （Java 微驱动 RenderGolden 产出，sha 输入流与 TickHooks 帧哈希同构）。
//! 同时验证自实现 zlib inflate（stored/fixed/dynamic 全块型）在真实语料上无损。

use std::path::PathBuf;
use std::process::Command;

use game_data::{decode_png, image_count, PackedPng};
use game_platform::hash::sha256_hex;

/// 夹具目录：与 `golden.rs` 同一可再生链路（`assets/raw` 或 XIYOU_ASSETS_RAW）。
fn raw_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XIYOU_ASSETS_RAW") {
        return PathBuf::from(dir);
    }
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let raw = repo_root.join("assets/raw");
    if !raw.join("maplv0").exists() {
        let extract = repo_root.join("tools/extract-assets/extract.py");
        let out = Command::new("python3")
            .arg(&extract)
            .output()
            .unwrap_or_else(|e| panic!("无法运行 python3 {}: {e}", extract.display()));
        assert!(out.status.success(), "解包失败: {}", String::from_utf8_lossy(&out.stderr));
    }
    raw
}

/// golden 裁判（Java 侧产出，随 Java 语义变化必须重跑微驱动）。
fn golden_json() -> json::Value {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let raw = std::fs::read_to_string(repo_root.join("data/golden/render-golden.json"))
        .expect("读 data/golden/render-golden.json 失败");
    json::parse(&raw)
}

/// 极小 JSON 取值（避免为测试引 serde：只需 name/w/h/sha 平铺结构）。
mod json {
    /// 从 `"key":value` 平铺 JSON 里抓取数组段中每个对象的字段。
    pub fn parse(s: &str) -> Value {
        Value(s.to_string())
    }

    pub struct Value(String);

    impl Value {
        /// images 数组：逐对象提取 (name, w, h, sha)。
        pub fn images(&self) -> Vec<(String, i64, i64, String)> {
            let seg = slice_array(&self.0, "images");
            let mut out = Vec::new();
            for obj in seg.split("},") {
                let name = string_field(obj, "name");
                let w = num_field(obj, "w");
                let h = num_field(obj, "h");
                let sha = string_field(obj, "sha");
                if !name.is_empty() {
                    out.push((name, w, h, sha));
                }
            }
            out
        }
    }

    fn slice_array<'a>(s: &'a str, key: &str) -> &'a str {
        let start = s.find(&format!("\"{key}\": [")).expect("缺数组段") + key.len() + 5;
        let rest = &s[start..];
        let end = rest.find("\n  ]").expect("数组未闭合");
        &rest[..end]
    }

    fn string_field(obj: &str, key: &str) -> String {
        let pat = format!("\"{key}\":\"");
        if let Some(i) = obj.find(&pat) {
            let rest = &obj[i + pat.len()..];
            return rest[..rest.find('"').unwrap()].to_string();
        }
        String::new()
    }

    fn num_field(obj: &str, key: &str) -> i64 {
        let pat = format!("\"{key}\":");
        if let Some(i) = obj.find(&pat) {
            let rest = &obj[i + pat.len()..];
            let end = rest.find([',', '}']).unwrap();
            return rest[..end].trim().parse().unwrap_or(0);
        }
        0
    }
}

/// 3 独立 PNG + 16 packed 容器全部子图：解码 → (w,h,sha) 逐图比对 Java golden。
#[test]
fn all_164_images_match_java_decode() {
    let raw = raw_dir();
    let golden = golden_json();
    let expect: std::collections::HashMap<String, (i64, i64, String)> = golden
        .images()
        .into_iter()
        .map(|(name, w, h, sha)| (name, (w, h, sha)))
        .collect();
    assert_eq!(expect.len(), 164, "golden 应含 164 张图（资源盘点数）");

    let mut checked = 0usize;
    for name in ["i62x62.png", "l0.png", "l1.png"] {
        let data = std::fs::read(raw.join(name)).unwrap();
        let img = decode_png(&data).unwrap_or_else(|e| panic!("{name}: {e}"));
        let &(ew, eh, ref esha) = expect.get(name).unwrap();
        assert_eq!((img.width as i64, img.height as i64), (ew, eh), "{name} 尺寸");
        assert_eq!(sha256_hex(&hash_stream(&img.argb)), esha.as_str(), "{name} 像素哈希");
        checked += 1;
    }
    for c in 0..16usize {
        let (cname, count) = super_container(c);
        let data = std::fs::read(raw.join(cname)).unwrap();
        let parsed = PackedPng::parse(&data, image_count(cname).unwrap() as usize)
            .unwrap_or_else(|e| panic!("{cname}: {e}"));
        assert_eq!(parsed.images.len(), count, "{cname} 张数");
        for (idx, sub) in parsed.images.iter().enumerate() {
            let key = format!("{cname}#{idx}");
            let img = decode_png(sub.bytes).unwrap_or_else(|e| panic!("{key}: {e}"));
            let &(ew, eh, ref esha) = expect.get(&key).unwrap();
            assert_eq!((img.width as i64, img.height as i64), (ew, eh), "{key} 尺寸");
            assert_eq!(sha256_hex(&hash_stream(&img.argb)), esha.as_str(), "{key} 像素哈希");
            checked += 1;
        }
    }
    assert_eq!(checked, 164);
}

fn super_container(i: usize) -> (&'static str, usize) {
    const CONTAINERS: [(&str, usize); 16] = [
        ("sflogo", 8), ("mapbg", 1), ("map", 12), ("actor", 4),
        ("sptmap", 13), ("sptprop", 23), ("sptarm", 10), ("sptenemy1", 20),
        ("ui", 25), ("xtq", 6), ("menu", 2), ("intro", 2),
        ("face", 12), ("sptenemy2", 20), ("end", 1), ("load", 2),
    ];
    CONTAINERS[i]
}

/// 与 TickHooks 帧哈希同构的像素流（逐像素大端 4 字节）。
fn hash_stream(argb: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(argb.len() * 4);
    for &p in argb {
        out.extend_from_slice(&[(p >> 24) as u8, (p >> 16) as u8, (p >> 8) as u8, p as u8]);
    }
    out
}
