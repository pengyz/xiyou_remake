//! 全量 golden test（L1）：55 个 maplv + 55 个 sprite + 16 个 packed PNG 文件
//! **全部解析且零余字节**（含 `maplv51` 非方阵特例）。
//!
//! 夹具来源（AGENTS.md §2：`assets/` 可再生、`original/` 只读）：
//! - 默认读 `assets/raw/`（`tools/extract-assets/extract.py` 的产物）；
//! - 路径可用环境变量 `XIYOU_ASSETS_RAW` 配置；
//! - 默认路径缺失时自动调 `python3 tools/extract-assets/extract.py` 从 `original/` 解包生成。
//!
//! 语料不变量的锚点值（文件尺寸/张数/取值域）均出自
//! `docs/findings/resource-formats.md`（行号见各断言注释），逐条可回查。

use std::path::PathBuf;
use std::process::Command;

use game_data::{extra_len, image_count, parse_guts_line, Maplv, PackedPng, SpriteTable, OPCODES};

/// 夹具目录：`XIYOU_ASSETS_RAW` 可配置；缺 `maplv0` 时解包生成。
fn raw_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XIYOU_ASSETS_RAW") {
        let dir = PathBuf::from(dir);
        assert!(dir.join("maplv0").exists(), "XIYOU_ASSETS_RAW={} 缺 maplv0", dir.display());
        return dir;
    }
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let raw = repo_root.join("assets/raw");
    if !raw.join("maplv0").exists() {
        let extract = repo_root.join("tools/extract-assets/extract.py");
        let out = Command::new("python3")
            .arg(&extract)
            .output()
            .unwrap_or_else(|e| panic!("无法运行 python3 {}: {e}", extract.display()));
        assert!(
            out.status.success(),
            "解包失败 {}: {}",
            extract.display(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    raw
}

fn read_raw(name: &str) -> Vec<u8> {
    let path = raw_dir().join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("读夹具失败 {}: {e}", path.display()))
}

/// 55 个 maplv 全量解析 + 零余字节（`docs/findings/resource-formats.md:120-148` 复现判据）。
#[test]
fn maplv_all_55_parse_zero_trailing() {
    let mut sizes = 0usize;
    for i in 0..55 {
        let data = read_raw(&format!("maplv{i}"));
        let (maplv, consumed) = Maplv::parse_partial(&data)
            .unwrap_or_else(|e| panic!("maplv{i}: {e}"));
        assert_eq!(consumed, data.len(), "maplv{i} 有余字节");
        Maplv::parse(&data).unwrap_or_else(|e| panic!("maplv{i} 严格解析: {e}"));
        // 零余字节 + 尺寸公式：4 + 2·(Y·Z·4)（findings:120-148）
        assert_eq!(
            data.len(),
            4 + 2 * maplv.grid_bytes(),
            "maplv{i} 总长与 4+2·Y·Z·4 不符"
        );
        assert_eq!(maplv.terrain.len(), maplv.flip.len());
        sizes += data.len();
    }
    // 54×1356B（13×13）+ 1252B（maplv51 的 12×13）
    assert_eq!(sizes, 54 * 1356 + 1252);
}

/// maplv 尺寸分布：54 个 13×13 + maplv51 特例 12×13（`docs/findings/resource-formats.md:134-135`、`:293-296`）。
#[test]
fn maplv_dims_and_maplv51_special() {
    for i in 0..55 {
        let data = read_raw(&format!("maplv{i}"));
        let maplv = Maplv::parse(&data).unwrap();
        if i == 51 {
            // 非方阵特例：头 (24,26) → 12×13，1252B（findings:293-296）
            assert_eq!((maplv.raw_width, maplv.raw_height), (24, 26), "maplv51 头");
            assert_eq!((maplv.width, maplv.height), (12, 13), "maplv51 网格");
            assert_eq!(maplv.grid_bytes(), 624);
            assert_eq!(data.len(), 1252);
        } else {
            assert_eq!((maplv.raw_width, maplv.raw_height), (26, 26), "maplv{i} 头");
            assert_eq!((maplv.width, maplv.height), (13, 13), "maplv{i} 网格");
            assert_eq!(data.len(), 1356);
        }
    }
}

/// 网格 2（翻转/变换码）取值域 ⊆ 0..=7（绘制方法 `switch(n7){case 1..7}`，
/// `a.java:2708-2744`；`docs/findings/resource-formats.md:163-168`）。
/// 语料实测域为 {0,1,2,3,4}（findings:161-162 的 `{0,1,2}` 是 maplv0 单文件抽样，
/// 全量语料另有 3/4：maplv1/6/8/11/14/16/18/54）。
#[test]
fn maplv_flip_codes_within_transform_domain() {
    let mut seen = [false; 8];
    for i in 0..55 {
        let data = read_raw(&format!("maplv{i}"));
        let maplv = Maplv::parse(&data).unwrap();
        for &code in &maplv.flip {
            assert!(code <= 7, "maplv{i} 翻转码 {code} 超出 0..=7");
            seen[code as usize] = true;
        }
        // 视角 B（2Y×2Z、1 字节/格，a.java:4991/5009）行列访问与平铺一致
        assert_eq!(
            maplv.tile_at(0, 0),
            maplv.terrain.first().copied(),
            "maplv{i} 行列索引"
        );
    }
    assert_eq!(
        seen.iter().filter(|s| **s).count(),
        5,
        "全量语料翻转码域应为 5 个值 {{0,1,2,3,4}}"
    );
    for code in [0u8, 1, 2, 3, 4] {
        assert!(seen[code as usize], "缺翻转码 {code}");
    }
}

/// 地形 tile-id 域：`id&7`=图集列、`id>>3`=图集行（`a.java:4998-4999`），
/// 图集 `map` 第 1 张子图 128×208 = 8 列 × 13 行 16px 格（`docs/findings/resource-formats.md:96-97`）
/// ⇒ `id ≤ 8·13-1 = 103`。语料实测 max=103、102 种取值。
#[test]
fn maplv_tile_ids_within_atlas() {
    let mut max_id = 0u8;
    let mut distinct = [false; 256];
    for i in 0..55 {
        let data = read_raw(&format!("maplv{i}"));
        let maplv = Maplv::parse(&data).unwrap();
        for &id in &maplv.terrain {
            distinct[id as usize] = true;
            max_id = max_id.max(id);
            assert!(Maplv::tile_atlas_row(id) <= 12, "maplv{i} tile-id {id} 超出图集行");
        }
    }
    assert_eq!(max_id, 103);
    assert_eq!(distinct.iter().filter(|d| **d).count(), 102);
}

/// 55 个 sprite 全量解析 + 零余字节（`docs/findings/resource-formats.md:220-249` 复现判据）。
#[test]
fn sprite_all_55_parse_zero_trailing() {
    let mut total_records = 0u32;
    for i in 0..55 {
        let data = read_raw(&format!("sprite{i}"));
        let (table, consumed) = SpriteTable::parse_partial(&data)
            .unwrap_or_else(|e| panic!("sprite{i}: {e}"));
        assert_eq!(consumed, data.len(), "sprite{i} 有余字节");
        SpriteTable::parse(&data).unwrap_or_else(|e| panic!("sprite{i} 严格解析: {e}"));
        assert_eq!(
            table.count() as usize,
            table.records.len(),
            "sprite{i} 计数头与记录数不符"
        );
        for rec in &table.records {
            assert_eq!(rec.extra.len(), extra_len(rec.type_code), "sprite{i} 附加参数长度");
        }
        total_records += table.count();
    }
    assert_eq!(total_records, 1703, "全量语料记录总数");
}

/// sprite 计数高 16 位恒为 0（`docs/findings/resource-formats.md:259`「本语料实测恒为 0」）。
#[test]
fn sprite_count_hi_word_is_zero() {
    for i in 0..55 {
        let data = read_raw(&format!("sprite{i}"));
        let table = SpriteTable::parse(&data).unwrap();
        assert_eq!(table.count_hi, 0, "sprite{i} 计数高字非 0");
    }
}

/// 类型码全集 = {1..15, 21, 23, 25..81, 83..86}（`docs/findings/resource-formats.md:250-252`、`:297-299`）。
/// 注：findings 表述为 `{1..15, 21, 23, 25..86}`，全量实测 **82 也从未出现**（与 0/16-20/22 同为未用码），
/// 取值均 ≤ 86 < `this.m` 88 项（`a.java:545`）。
#[test]
fn sprite_type_code_set_matches_corpus() {
    let mut seen = [false; 256];
    for i in 0..55 {
        let data = read_raw(&format!("sprite{i}"));
        let table = SpriteTable::parse(&data).unwrap();
        for rec in &table.records {
            seen[rec.type_code as usize] = true;
        }
    }
    for code in 0..=255u8 {
        let expected = matches!(code, 1..=15 | 21 | 23 | 25..=81 | 83..=86);
        assert_eq!(seen[code as usize], expected, "类型码 {code} 出现情况与语料基线不符");
    }
}

/// 16 个 packed PNG 容器全量解析 + 零余字节 + 每张 PNG 魔数
///（`docs/findings/resource-formats.md:42-68` 复现判据；尺寸锚点 findings:64-68）。
#[test]
fn packed_png_all_16_parse_zero_trailing() {
    // findings:64-68 实测文件尺寸
    let expected_sizes = [
        ("sflogo", 7505usize), ("mapbg", 9912), ("map", 16372), ("actor", 4293),
        ("sptmap", 14184), ("sptprop", 7248), ("sptarm", 3015), ("sptenemy1", 16230),
        ("ui", 7203), ("xtq", 6759), ("menu", 25286), ("intro", 47580),
        ("face", 3239), ("sptenemy2", 20907), ("end", 16696), ("load", 806),
    ];
    assert_eq!(expected_sizes.len(), 16);
    for (name, expected_size) in expected_sizes {
        let count = image_count(name).unwrap_or_else(|| panic!("{name} 不在张数常量表"));
        let data = read_raw(name);
        assert_eq!(data.len(), expected_size, "{name} 文件尺寸");
        let (packed, consumed) = PackedPng::parse_partial(&data, count as usize)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(consumed, data.len(), "{name} 有余字节");
        PackedPng::parse(&data, count as usize).unwrap_or_else(|e| panic!("{name} 严格解析: {e}"));
        assert_eq!(packed.images.len(), count as usize, "{name} 子图张数");
        for img in &packed.images {
            // 每张起始 8 字节 = PNG 魔数（findings:44、:80）；解析器已强制，此处回读断言
            assert_eq!(img.bytes[..8], game_data::PNG_MAGIC[..], "{name} 子图魔数");
        }
    }
}

/// script 资源 `GUTS:` 段全量行可词法拆分（辅助解析器，供后续解释器）。
/// 语料基线：64 条编号行（层号 0..67 缺 39/58/59/60，`docs/findings/script-dsl-semantics.md:160`）
/// + 1 行段终止 `END`；全部指令词 ∈ [`OPCODES`]（`docs/findings/script-dsl-semantics.md:36`）。
#[test]
fn script_guts_lines_lex() {
    let text = String::from_utf8_lossy(&read_raw("script")).into_owned();
    let mut lines = text
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .skip_while(|l| *l != "GUTS:");
    assert_eq!(lines.next(), Some("GUTS:"));
    let mut layers = Vec::new();
    for line in lines.filter(|l| !l.trim().is_empty()) {
        if line.trim() == "END" {
            continue; // 段终止行（GAME:/GUTS: 同形），非 GUTS 行
        }
        let guts = parse_guts_line(line).unwrap_or_else(|e| panic!("{line}: {e}"));
        layers.push(guts.layer);
        for instr in &guts.instrs {
            assert!(
                game_data::is_known_opcode(instr.opcode),
                "层 {} 未知指令词 {}",
                guts.layer,
                instr.opcode
            );
        }
    }
    assert_eq!(layers.len(), 64);
    let expected: Vec<i32> = (0..=67).filter(|l| !matches!(l, 39 | 58 | 59 | 60)).collect();
    assert_eq!(layers, expected);
    assert_eq!(OPCODES.len(), 16);
}
