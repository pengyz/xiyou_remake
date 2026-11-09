//! 【求解器批0】输入构造：maplv/sprite/fixture 脚本/三表 → `SolverInput`（计划批0）。
//!
//! **红线（计划 §0/§2.7，BUG-053 docs/bug-ledger.md:1035）**：items.json 22-25 的 effect 字段
//! **禁作求解器效果源**（22 已由 6efd30e 修；23/24/25 待被动判定点考证）——本模块只按清单索引读取，
//! 效果建模一律以**原版 activateItem** 与实考证（docs/findings/2026-11-09-qol-passive-item-triggers.md §6）为准。
use game_core::sprite_spawn::{spawn_param_and_visibility, DoorTables};
use game_data::defs::{parse_enemies, parse_items, parse_shop_goods, EnemyDef, ItemDef, ShopGoodDef};
use game_data::maplv::Maplv;
use game_data::sprite::SpriteRecord;

/// 一条 sprite 生成记录（记录本体+生成参数/隐藏位——`spawn_param_and_visibility` 语义冻结面）。
pub struct SpawnRec {
    pub record: SpriteRecord,
    pub param: i16,
    pub hidden: bool,
}

/// 求解器输入（纯数据；由 bin/测试构造——lib 禁 I/O）。
pub struct SolverInput {
    /// 55 层几何（maplv0..54）。
    pub floors: Vec<Maplv>,
    /// 各层 sprite 记录（平铺；`per_floor_records[i]`=第 i 层条数）。
    pub spawns: Vec<SpawnRec>,
    /// 各层记录计数（census 对账面）。
    pub per_floor_records: Vec<usize>,
    /// 68 脚本槽（level_scripts.txt 每行一槽，指令流原文）。
    pub scripts: Vec<String>,
    /// enemies.json 表（35 行）。
    pub enemies: Vec<EnemyDef>,
    /// items.json 表（32 行）。
    pub items: Vec<ItemDef>,
    /// shop_goods.json 表（17 行）。
    pub shops: Vec<ShopGoodDef>,
}

/// census 快照（判据输出 `floors=55 enemies=502 scripts=68 enemies_table=35 items=32 shops=17`）。
pub struct Census {
    pub floors: usize,
    /// sprite 敌人记录（type 41..=75）总数。
    pub enemies: usize,
    pub scripts: usize,
    pub enemies_table: usize,
    pub items: usize,
    pub shops: usize,
}

/// 构造 `SolverInput`。maplvs/sprites 各 55 槽（maplv{i}/sprite{i} 逐字节）。
pub fn ingest(
    maplvs: &[&[u8]],
    sprites: &[&[u8]],
    scripts_txt: &str,
    door12: &str,
    door13: &str,
    enemies_json: &str,
    items_json: &str,
    shops_json: &str,
) -> Result<SolverInput, String> {
    let mut floors = Vec::with_capacity(maplvs.len());
    for (i, raw) in maplvs.iter().enumerate() {
        floors.push(Maplv::parse(raw).map_err(|e| format!("maplv{i}: {e:?}"))?);
    }
    let doors = DoorTables {
        door12: door12.lines().filter_map(|l| l.trim().parse().ok()).collect(),
        door13: door13.lines().filter_map(|l| l.trim().parse().ok()).collect(),
    };
    let mut spawns = Vec::new();
    let mut per_floor_records = Vec::with_capacity(sprites.len());
    for (i, raw) in sprites.iter().enumerate() {
        let table = game_data::sprite::SpriteTable::parse(raw).map_err(|e| format!("sprite{i}: {e:?}"))?;
        let n = table.records.len();
        for rec in table.records {
            let (param, hidden) = spawn_param_and_visibility(rec.type_code, &rec.extra, &doors);
            spawns.push(SpawnRec { record: rec, param, hidden });
        }
        per_floor_records.push(n);
    }
    let scripts: Vec<String> = scripts_txt.lines().map(|l| l.to_string()).collect();
    let (enemies, _unknown) = parse_enemies(enemies_json).map_err(|e| format!("enemies: {e:?}"))?;
    let items = parse_items(items_json).map_err(|e| format!("items: {e:?}"))?;
    let item_ids: Vec<i32> = items.iter().map(|it| it.id).collect();
    let shops = parse_shop_goods(shops_json, &item_ids).map_err(|e| format!("shops: {e:?}"))?;
    Ok(SolverInput {
        floors,
        spawns,
        per_floor_records,
        scripts,
        enemies,
        items,
        shops,
    })
}

/// census 快照：与计划 §2 数字对拍。
pub fn census(input: &SolverInput) -> Census {
    Census {
        floors: input.floors.len(),
        enemies: input.spawns.iter().filter(|s| (41..=75).contains(&s.record.type_code)).count(),
        scripts: input.scripts.len(),
        enemies_table: input.enemies.len(),
        items: input.items.len(),
        shops: input.shops.len(),
    }
}
