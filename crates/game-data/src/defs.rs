//! 资源配置 V2 三表解析：`sprite_defs` / `anim_defs` / `sheet_index`。
//!
//! 契约（单一真相源）：`docs/spec/resource-config-survey.md` v2
//! **§2.1**（schema 字段定义=解析契约）+ **§2.3**（外键政策 + 加载层失败语义：五类拒载）。
//!
//! # 文件封套（envelope）
//!
//! 每表文件为 JSON 对象，首字段 `schema_version` 必须等于 [`SCHEMA_VERSION`]（不符/缺失=拒载，
//! §2.3「schema 版本字段」）：
//!
//! ```jsonc
//! { "schema_version": 2, "sprite_defs": [ …逐条 SpriteDef… ] }   // sprite_defs.json
//! { "schema_version": 2, "anim_defs":   [ …逐条 AnimDef… ] }     // anim_defs.json
//! { "schema_version": 2, "containers": [...], "container_counts": [...],
//!   "slots": {...}, "evidence": [...] }                          // sheet_index.json（§2.1 扁平形）
//! ```
//!
//! # 五类拒载（§2.3，错误信息均含定位=JSON 路径或字节偏移）
//!
//! 1. 坏 JSON 拒载 [`Reject::BadJson`]；
//! 2. 坏外键拒载 [`Reject::BadForeignKey`]（`anim_profile`→`anim_defs`、`assembly_profile`→SEG 表、
//!    `sheet`→`sheet_index.containers` 孤儿拒载；`sprite_ref` 指空=拒载，见 [`DefBundle::validate_sprite_ref`]；
//!    级联=删 `sprite_defs` 行前必须先清引用——[`DefBundle::referencing_type_ids`] 供删前校验；
//!    id 稳定性=type_id/anim id 重复即拒载，保真 id 冻结不重排）；
//! 3. 域溢出拒载 [`Reject::DomainOverflow`]（型号域 0..=88，见 [`TYPE_ID_MAX`]）；
//! 4. 帧序越界拒载 [`Reject::FrameIndexOutOfBounds`]（`anim_defs.frames` 下标 ≥ 表长=所引
//!    图集容器张数 `container_counts`）；
//! 5. unknown-field=拒载 [`Reject::UnknownField`]（防拼写静默——任何 schema 外键名直接拒载）。
//!
//! # 保真与 mod（§2.2 两份制，本批只留缝）
//!
//! 保真 id 冻结（type_id/anim id 不重排、不复用）是表维护纪律；mod id 命名空间（型号 ≥1000
//! 或前缀）与 mod 表加载属 Phase 3（task-8）mod 骨架，本解析器只解析保真面。
//!
//! # 证据
//!
//! 字段定义逐条锚 survey §2.1 schema 主件；`density` 字段为 HD 探针 R-5 回写预留
//! （plan 拍板 2：写进 sprite_defs 默认 1，本解析器不引渲染语义）。

use crate::asset_pipeline::{parse_json, Json};

/// schema 版本字段值（survey §2.3「schema 版本字段」；当前=v2）。
pub const SCHEMA_VERSION: i64 = 2;

/// 型号域下界（含）。
pub const TYPE_ID_MIN: i64 = 0;
/// 型号域上界（含）：0..=88 共 89 型，对应 `ANIM_TABLE_IDX`（animRowByType）89 项
/// （survey §0.1）；溢出=拒载（§2.3 域溢出）。
pub const TYPE_ID_MAX: i64 = 88;

// ---------------------------------------------------------------- 严格键集（第 5 类 unknown-field=拒载）

/// `sprite_defs.json` 封套键（task-5 真表实形，commit a284203）：
/// `transition_overrides`（规则形移至顶层）/`size_dispatch`（dispatch_sprite_size 特例表）/
/// `quirks_in_code`（§2.1 quirks 留痕）三 blob **原样保留**（内部结构由消费替换步 task-7 读取，
/// 本层只验容器形态，防拼写静默由封套键集承担）。
const ENVELOPE_SPRITE_KEYS: &[&str] =
    &["schema_version", "sprite_defs", "transition_overrides", "size_dispatch", "quirks_in_code"];
/// `anim_defs.json` 封套键。
const ENVELOPE_ANIM_KEYS: &[&str] = &["schema_version", "anim_defs"];
/// `sheet_index.json` 封套键（§2.1 扁平形 + task-5 `assembly_profiles`：SEG 片件表 id 目录）。
const ENVELOPE_SHEET_KEYS: &[&str] = &[
    "schema_version",
    "containers",
    "container_counts",
    "slots",
    "assembly_profiles",
    "evidence",
];
/// `sprite_defs` 记录键（§2.1 schema 主件逐字段）。
const SPRITE_KEYS: &[&str] = &[
    "type_id",
    "sheet",
    "sheet_index",
    "frame_w",
    "frame_h",
    "render_category",
    "anim_row",
    "type_frame_width",
    "anim_profile",
    "assembly_profile",
    "insert_block",
    "floor_category_override",
    "transition_overrides",
    "density",
    "evidence",
];
/// `floor_category_override` 条目键。
const FLOOR_KEYS: &[&str] = &["floor", "category"];
/// `transition_overrides` 条目键。
const TRANSITION_KEYS: &[&str] = &["on", "by_category", "evidence"];
/// `transition_overrides.by_category` 值键。
const CATEGORY_KEYS: &[&str] = &["anim_row", "sprite_h"];
/// `anim_defs` 记录键。
const ANIM_KEYS: &[&str] = &["id", "frames", "step_ticks", "loop_mode", "evidence"];
/// 逐值锚三元组键（{index, value, src}）。
const ANCHOR_KEYS: &[&str] = &["index", "value", "src"];

/// 加载层拒载错误（§2.3 五类 + schema 版本/形态违例）。**每条信息含定位**（JSON 路径或字节偏移）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reject {
    /// 第 1 类：坏 JSON（语法错，`detail` 来自底层解析器，含字节偏移）。
    BadJson {
        /// 底层语法错误（含偏移定位）
        detail: String,
    },
    /// schema 版本字段缺失/不符（§2.3）。
    SchemaVersion {
        /// 文件里的版本（缺失=None）
        found: Option<i64>,
        /// 期望版本
        expected: i64,
    },
    /// schema 形态违例：字段缺失/类型不符（含定位路径）。
    Schema {
        /// JSON 路径定位，如 `sprite_defs[3].type_id`
        path: String,
        /// 原因
        reason: String,
    },
    /// 第 5 类：unknown-field=拒载（防拼写静默）。
    UnknownField {
        /// JSON 路径定位（记录/对象级）
        path: String,
        /// 未知字段名
        field: String,
    },
    /// 第 2 类：坏外键（孤儿/跨表悬空/sprite_ref 指空）。
    BadForeignKey {
        /// JSON 路径定位
        path: String,
        /// 悬空引用详情
        detail: String,
    },
    /// 第 3 类：域溢出（型号域 0..=88）。
    DomainOverflow {
        /// JSON 路径定位
        path: String,
        /// 实际值
        value: i64,
        /// 域描述
        domain: &'static str,
    },
    /// 第 4 类：帧序越界（`frames` 下标超表长）。
    FrameIndexOutOfBounds {
        /// JSON 路径定位
        path: String,
        /// 越界下标
        index: i64,
        /// 表长（图集容器张数）
        table_len: usize,
    },
}

impl std::fmt::Display for Reject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reject::BadJson { detail } => write!(f, "坏 JSON 拒载：{detail}"),
            Reject::SchemaVersion { found, expected } => write!(
                f,
                "schema 版本拒载（路径 schema_version）：found {found:?}，期望 {expected}"
            ),
            Reject::Schema { path, reason } => {
                write!(f, "schema 违例拒载（路径 {path}）：{reason}")
            }
            Reject::UnknownField { path, field } => {
                write!(f, "unknown-field 拒载（路径 {path}）：未知字段 {field:?}")
            }
            Reject::BadForeignKey { path, detail } => {
                write!(f, "坏外键拒载（路径 {path}）：{detail}")
            }
            Reject::DomainOverflow { path, value, domain } => {
                write!(f, "域溢出拒载（路径 {path}）：值 {value} 超出域 {domain}")
            }
            Reject::FrameIndexOutOfBounds { path, index, table_len } => write!(
                f,
                "帧序越界拒载（路径 {path}）：frames 下标 {index} 超表长 {table_len}"
            ),
        }
    }
}

impl std::error::Error for Reject {}

// ---------------------------------------------------------------- 数据结构（§2.1 字段逐条）

/// 逐值锚三元组 `{index, value, src}`（survey §2.1 ★R1 逐值锚；value 形态不设限=原样保留）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAnchor {
    /// 表内序号
    pub index: i64,
    /// 锚定值（对象/数组/标量原样保留）
    pub value: Json,
    /// 证据指针（`a.java:行` 等）
    pub src: String,
}

/// `floor_category_override` 条目（§2.1：`{floor, category}`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorCategoryOverride {
    /// 楼层号
    pub floor: i64,
    /// 渲染分类
    pub category: i64,
}

/// `transition_overrides.by_category` 值（§2.1：`{anim_row?, sprite_h?}`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryOverride {
    /// 覆写动画面行（可缺）
    pub anim_row: Option<i64>,
    /// 覆写精灵高（可缺）
    pub sprite_h: Option<i64>,
}

/// `transition_overrides` 条目——事件条件规则形（§2.1 ★R3 修正）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionOverride {
    /// 事件条件（`spawn_state1` / `mark_removed` / `mark_des_removed` …）
    pub on: String,
    /// 按渲染分类分派（键=`"1"`/`"2"`/`"*"`…，保持插入序）
    pub by_category: Vec<(String, CategoryOverride)>,
    /// 证据锚（§2.1 各规则条目自带）
    pub evidence: Option<String>,
}

/// `sprite_defs` 一条（型→精灵属性，§2.1 schema 主件）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteDef {
    /// 型号 id（域 0..=88，保真 id 冻结）
    pub type_id: i64,
    /// 图集容器名（外键 → `sheet_index.containers`；可空=无图型，如 type 0）
    pub sheet: Option<String>,
    /// 图集内槽位下标（与 `sheet` 同生共死）
    pub sheet_index: Option<i64>,
    /// 逻辑帧宽
    pub frame_w: i64,
    /// 逻辑帧高（可空=型高由运行时定，真表 82/89 为 null——task-5 表实形）
    pub frame_h: Option<i64>,
    /// 渲染分类
    pub render_category: i64,
    /// 动画面行（`ANIM_TABLE_IDX`/animRowByType 逐型行，task-5 表实形）
    pub anim_row: Option<i64>,
    /// 型宽（`type_width_px` 88 项迁入值，task-5 表实形；evidence 补录在表侧）
    pub type_frame_width: Option<i64>,
    /// 动画档外键（→ `anim_defs.id`）
    pub anim_profile: String,
    /// SEG67 片件表引用（表中表外键，可空）
    pub assembly_profile: Option<String>,
    /// 插块标记
    pub insert_block: bool,
    /// 楼层分类覆写
    pub floor_category_override: Vec<FloorCategoryOverride>,
    /// 事件条件规则形覆写（★R3）
    pub transition_overrides: Vec<TransitionOverride>,
    /// 像素密度（HD 预留，默认 1；素材尺寸=逻辑尺寸×density）
    pub density: i64,
    /// 逐值锚（★R1）
    pub evidence: Vec<EvidenceAnchor>,
}

/// `anim_defs` 一条（§2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimDef {
    /// 动画档 id（外键目标；保真 id 冻结）
    pub id: String,
    /// 帧序（下标域=所引精灵图集容器张数，越界=拒载）
    pub frames: Vec<i64>,
    /// 每帧 tick 数
    pub step_ticks: i64,
    /// 循环模式（`cycle` 等，原样保留）
    pub loop_mode: String,
    /// 逐值锚
    pub evidence: Vec<EvidenceAnchor>,
}

/// `sheet_index`（§2.1 扁平形：容器名序 + 张数表 + slots + evidence）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetIndex {
    /// 16 容器名序（a.java:450）
    pub containers: Vec<String>,
    /// 16 容器张数表（a.java:453 `resourceImageCounts`（2026-10-22 task-5 勘误：非 451））
    pub container_counts: Vec<i64>,
    /// 槽位表（§2.1 字段定义为对象，内部结构 schema 未定——原样保留，不发明语义）
    pub slots: Json,
    /// SEG 片件表 id 目录（task-5 表实形：`assembly_profiles` 对象的键集）
    pub assembly_profiles: Vec<String>,
    /// 逐值锚
    pub evidence: Vec<EvidenceAnchor>,
}

// ---------------------------------------------------------------- 解析

/// 解析 `sprite_defs.json` → [`SpriteDef`] 列表（单表层：schema 版本+字段形态）。
pub fn parse_sprite_defs(src: &str) -> Result<Vec<SpriteDef>, Reject> {
    let doc = parse_envelope(src, ENVELOPE_SPRITE_KEYS)?;
    // 顶层三 blob（task-5 表实形）只验容器形态：数组/对象/数组
    if let Some(v) = doc.get("transition_overrides") {
        if !matches!(v, Json::Arr(_)) {
            return Err(schema("<root>.transition_overrides", "非数组"));
        }
    }
    if let Some(v) = doc.get("size_dispatch") {
        if !matches!(v, Json::Obj(_)) {
            return Err(schema("<root>.size_dispatch", "非对象"));
        }
    }
    if let Some(v) = doc.get("quirks_in_code") {
        if !matches!(v, Json::Arr(_)) {
            return Err(schema("<root>.quirks_in_code", "非数组"));
        }
    }
    let items = expect_arr(&doc, "sprite_defs", "sprite_defs")?;
    items
        .iter()
        .enumerate()
        .map(|(i, rec)| parse_sprite_def(rec, &format!("sprite_defs[{i}]")))
        .collect()
}

/// 解析 `anim_defs.json` → [`AnimDef`] 列表。
pub fn parse_anim_defs(src: &str) -> Result<Vec<AnimDef>, Reject> {
    let doc = parse_envelope(src, ENVELOPE_ANIM_KEYS)?;
    let items = expect_arr(&doc, "anim_defs", "anim_defs")?;
    items
        .iter()
        .enumerate()
        .map(|(i, rec)| parse_anim_def(rec, &format!("anim_defs[{i}]")))
        .collect()
}

/// 解析 `sheet_index.json` → [`SheetIndex`]。
pub fn parse_sheet_index(src: &str) -> Result<SheetIndex, Reject> {
    let doc = parse_envelope(src, ENVELOPE_SHEET_KEYS)?;
    let containers_j = expect_arr(&doc, "containers", "sheet_index")?;
    let containers = containers_j
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| schema(format!("sheet_index.containers[{i}]"), "非字符串"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let counts_j = expect_arr(&doc, "container_counts", "sheet_index")?;
    let container_counts = counts_j
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_int()
                .ok_or_else(|| schema(format!("sheet_index.container_counts[{i}]"), "非整数"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let slots = doc
        .get("slots")
        .filter(|v| matches!(v, Json::Obj(_)))
        .ok_or_else(|| schema("sheet_index.slots", "缺失或非对象"))?
        .clone();
    let assembly_profiles = match doc.get("assembly_profiles") {
        None => Vec::new(),
        Some(Json::Obj(members)) => members.iter().map(|(k, _)| k.clone()).collect(),
        Some(_) => return Err(schema("sheet_index.assembly_profiles", "非对象")),
    };
    let evidence = match doc.get("evidence") {
        None => Vec::new(),
        Some(Json::Arr(items)) => items
            .iter()
            .enumerate()
            .map(|(i, a)| parse_anchor(a, &format!("sheet_index.evidence[{i}]")))
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(schema("sheet_index.evidence", "非数组")),
    };
    Ok(SheetIndex { containers, container_counts, slots, assembly_profiles, evidence })
}

/// 语法解析 + 封套 unknown-field 拒载（第 5 类）+ schema 版本校验（§2.3）。
// ─────────────────────────────────────────────────────────────────────────────
// DDDSA P1-6 enemies.json（EnemyDef 聚合——per-type 属性束；SoA 散表收束：
// enemyBaseHp/Atk/Def/Gold 4×35 横索引→一行一型；difficulty_multipliers=
// 全局表非 per-type 随信封；behavior.trait_flags=traitFlagsByType 行为键）
// 生成器 tools/enemy-defs/gen_enemies.py（deobf a.java 逐值迁）。
// ─────────────────────────────────────────────────────────────────────────────

const ENVELOPE_ENEMY_KEYS: &[&str] = &["schema_version", "note", "difficulty_multipliers", "rows"];

/// EnemyDef 聚合记录（id=敌型 41..=75（SoA 横索引 t-41）；一行一型）。
/// 【2026-11-06 amend 更正·§0 引反】前注"族表保留独立"**方向反**——spec
/// v3 §0:16 实文："存量迁移：89 型怪族表/道具描述表→per-type 记录（值逐
/// 字迁+V2 逐值锚重生+金样护航）"——**族表=本件正做迁移**（enemyBase 四表
/// 聚合即族表收束）。cellSlotTable（实例空间索引）按 §0:12-13 判据
/// （"某类资源怎么摆"→保留表）独立面保留。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyDef {
    pub id: i32,
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub gold: i32,
    /// traitFlagsByType 行为键（a.java:6266-6267 稀疏：49/53/74=1、69=2）
    pub trait_flags: u8,
}

/// 解析 `enemies.json` →（EnemyDef 行, difficulty_multipliers 全局表）。
/// 五类拒载 §2.3 同式（BadJson/BadForeignKey[icon 面无]/DomainOverflow[id
/// 0..=88]/ParamsRange[trait_flags≤255]/UnknownField）。
pub fn parse_enemies(src: &str) -> Result<(Vec<EnemyDef>, Vec<i64>), Reject> {
    let doc = parse_envelope(src, ENVELOPE_ENEMY_KEYS)?;
    let mult: Vec<i64> = match doc.get("difficulty_multipliers") {
        Some(Json::Arr(a)) => a
            .iter()
            .map(|v| v.as_int().ok_or_else(|| schema("<root>.difficulty_multipliers", "非整数")))
            .collect::<Result<_, _>>()?,
        _ => return Err(schema("<root>.difficulty_multipliers", "缺/非数组")),
    };
    let items = expect_arr(&doc, "rows", "rows")?;
    let rows = items
        .iter()
        .enumerate()
        .map(|(i, rec)| parse_enemy(rec, &format!("rows[{i}]")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((rows, mult))
}

const ENEMY_ROW_KEYS: &[&str] = &["id", "hp", "atk", "def", "gold", "behavior", "evidence"];

fn parse_enemy(rec: &Json, path: &str) -> Result<EnemyDef, Reject> {
    if let Json::Obj(kvs) = rec {
        for (k, _) in kvs {
            if !ENEMY_ROW_KEYS.contains(&k.as_str()) {
                return Err(Reject::UnknownField {
                    path: format!("{path}.{k}"),
                    field: k.clone(),
                });
            }
        }
    }
    let id = rec.need_int("id").map_err(|e| schema(path, e))? as i32;
    if !(0..=TYPE_ID_MAX as i32).contains(&id) {
        return Err(Reject::DomainOverflow {
            path: format!("{path}.id"),
            value: id as i64,
            domain: "0..=88（TYPE_ID_MAX）",
        });
    }
    let beh = rec.get("behavior").ok_or_else(|| schema(path, "缺 behavior"))?;
    if let Json::Obj(kvs) = beh {
        for (k, _) in kvs {
            if k.as_str() != "trait_flags" {
                return Err(Reject::UnknownField {
                    path: format!("{path}.behavior.{k}"),
                    field: k.clone(),
                });
            }
        }
    }
    let tf = beh.need_int("trait_flags").map_err(|e| schema(path, e))?;
    if !(0..=255).contains(&tf) {
        return Err(Reject::FrameIndexOutOfBounds {
            path: format!("{path}.behavior.trait_flags"),
            index: tf,
            table_len: 256,
        });
    }
    Ok(EnemyDef {
        id,
        hp: rec.need_int("hp").map_err(|e| schema(path, e))? as i32,
        atk: rec.need_int("atk").map_err(|e| schema(path, e))? as i32,
        def: rec.need_int("def").map_err(|e| schema(path, e))? as i32,
        gold: rec.need_int("gold").map_err(|e| schema(path, e))? as i32,
        trait_flags: tf as u8,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// DDDSA P1-1 items.json（ItemDef 聚合记录——per-type 属性束；用户数据模型总则）
// 生成器 tools/item-defs/gen_items.py（五源逐值迁：names.txt/D[22]/ITEM_EFFECTS+
// dispatch/equip_types+bonuses/icon 外键）。强类型 Params（spec 红线：禁 JSON map
// 弱类型——每 EffectKey 携 typed params）。
// ─────────────────────────────────────────────────────────────────────────────

const ENVELOPE_ITEMS_KEYS: &[&str] = &["schema_version", "note", "rows"];

/// 图标外键（entity 图引用=images.entity[t] 渲染面；sprite_defs 引用面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemIcon {
    pub entity_type: i32,
}

/// 面板分派目标（强类型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemPanel {
    EnemyInfo,
    RecordBrowser,
}

/// 钥匙色（强类型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyColor {
    Yellow,
    Red,
    Blue,
}

/// 属性面（强类型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatKind {
    Atk,
    Def,
}

/// EffectKey（强类型 params——每键自带 schema；加载期校验转换）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectKey {
    OpenPanel { panel: ItemPanel, page: i32 },
    RemoveAdjacent { target: i32, msg: String },
    RemoveAllType { target: i32 },
    Heal { mul: i32, div: i32, msg_fmt: String },
    KillAdjacent4,
    MirrorTeleport,
    ChangeFloorUp,
    TutorialSeenStack,
    Passive { kind: String },
    Key { color: KeyColor },
    StatUp { stat: StatKind },
    HpUp { mul: i32 },
    Equip { tier: usize, bonus: i32 },
}

/// ItemDef 聚合记录（id<1000=保真表行；一行一物=per-type 属性束）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDef {
    pub id: i32,
    pub name: String,
    pub icon: ItemIcon,
    pub desc: String,
    pub pickup_msg: String,
    pub effect: EffectKey,
}

/// 解析 `items.json` → [`ItemDef`] 列表（五类拒载 §2.3 同式）。
///
/// 拒载映射：①BadJson（parse_envelope）②BadForeignKey（icon.entity_type 域外
/// =sprite 引用面拒；effect.params 外键字段名错）③DomainOverflow（id 0..=88
/// [`TYPE_ID_MAX`]）④ParamsRange（Heal div=0/Equip tier≥12——FrameIndex 越界
/// 同族）⑤UnknownField（行键/effect.key/枚举值外=拒载防拼写静默）。
pub fn parse_items(src: &str) -> Result<Vec<ItemDef>, Reject> {
    parse_items_in_domain(src, 0, TYPE_ID_MAX as i32)
}

/// 【DDDSA P1-5 mod 通道·两份制】mod 表解析（id 域 1000..=9999——保真表
/// id<1000/金样禁 mod 表（mod 表不入 FIDELITY_TABLES 哈希闸=用户内容面）；
/// 行解析/五类拒载与保真面同构（DomainOverflow 域=1000..9999）。
pub fn parse_items_mod(src: &str) -> Result<Vec<ItemDef>, Reject> {
    parse_items_in_domain(src, 1000, 9999)
}

fn parse_items_in_domain(src: &str, lo: i32, hi: i32) -> Result<Vec<ItemDef>, Reject> {
    let doc = parse_envelope(src, ENVELOPE_ITEMS_KEYS)?;
    let items = expect_arr(&doc, "rows", "rows")?;
    items
        .iter()
        .enumerate()
        .map(|(i, rec)| parse_item(rec, &format!("rows[{i}]"), lo, hi))
        .collect()
}

const ITEM_ROW_KEYS: &[&str] = &["id", "name", "icon", "desc", "pickup_msg", "effect", "evidence"];

fn parse_item(rec: &Json, path: &str, domain_lo: i32, domain_hi: i32) -> Result<ItemDef, Reject> {
    // ⑤ UnknownField：行键外拒载（evidence=随行证据槽允许）
    if let Json::Obj(kvs) = rec {
        for (k, _) in kvs {
            if !ITEM_ROW_KEYS.contains(&k.as_str()) {
                return Err(Reject::UnknownField {
                    path: format!("{path}.{k}"),
                    field: k.clone(),
                });
            }
        }
    }
    let id = rec.need_int("id").map_err(|e| schema(path, e))? as i32;
    // ③ DomainOverflow：型号域 0..=88
    if !(domain_lo..=domain_hi).contains(&id) {
        return Err(Reject::DomainOverflow {
            path: format!("{path}.id"),
            value: id as i64,
            domain: if domain_lo == 0 { "0..=88（TYPE_ID_MAX 保真域）" } else { "1000..=9999（mod 域）" },
        });
    }
    let name = rec.need_str("name").map_err(|e| schema(path, e))?.to_string();
    let desc = rec.need_str("desc").map_err(|e| schema(path, e))?.to_string();
    let pickup_msg = rec.need_str("pickup_msg").map_err(|e| schema(path, e))?.to_string();
    // icon 外键
    let icon_o = rec.get("icon").ok_or_else(|| schema(path, "缺 icon"))?;
    let et = icon_o.need_int("entity_type").map_err(|e| schema(path, e))? as i32;
    if !(0..=TYPE_ID_MAX as i32).contains(&et) {
        return Err(Reject::BadForeignKey {
            path: format!("{path}.icon.entity_type"),
            detail: format!("entity_type {et} 域外（sprite 引用面拒）"),
        });
    }
    let effect_o = rec.get("effect").ok_or_else(|| schema(path, "缺 effect"))?;
    let effect = parse_effect(effect_o, &format!("{path}.effect"))?;
    Ok(ItemDef {
        id,
        name,
        icon: ItemIcon { entity_type: et },
        desc,
        pickup_msg,
        effect,
    })
}

fn parse_effect(rec: &Json, path: &str) -> Result<EffectKey, Reject> {
    let key = rec.need_str("key").map_err(|e| schema(path, e))?;
    let p = rec.get("params").ok_or_else(|| schema(path, "缺 effect.params"))?;
    let pi = format!("{path}.params");
    let need_i = |k: &str| -> Result<i32, Reject> {
        p.need_int(k).map(|v| v as i32).map_err(|e| schema(&pi, e))
    };
    let need_s = |k: &str| -> Result<String, Reject> {
        p.need_str(k).map(|v| v.to_string()).map_err(|e| schema(&pi, e))
    };
    match key {
        "OpenPanel" => {
            let panel = match p.need_str("panel").map_err(|e| schema(&pi, e))? {
                "EnemyInfo" => ItemPanel::EnemyInfo,
                "RecordBrowser" => ItemPanel::RecordBrowser,
                other => {
                    return Err(Reject::UnknownField {
                        path: format!("{pi}.panel"),
                        field: other.to_string(),
                    })
                }
            };
            let page = need_i("page")?;
            Ok(EffectKey::OpenPanel { panel, page })
        }
        "RemoveAdjacent" => Ok(EffectKey::RemoveAdjacent {
            target: need_i("target")?,
            msg: need_s("msg")?,
        }),
        "RemoveAllType" => Ok(EffectKey::RemoveAllType { target: need_i("target")? }),
        "Heal" => {
            let mul = need_i("mul")?;
            let div = need_i("div")?;
            if div == 0 {
                return Err(Reject::FrameIndexOutOfBounds {
                    path: format!("{pi}.div"),
                    index: 0,
                    table_len: 0,
                });
            }
            Ok(EffectKey::Heal { mul, div, msg_fmt: need_s("msg_fmt")? })
        }
        "KillAdjacent4" => Ok(EffectKey::KillAdjacent4),
        "MirrorTeleport" => Ok(EffectKey::MirrorTeleport),
        "ChangeFloorUp" => Ok(EffectKey::ChangeFloorUp),
        "TutorialSeenStack" => Ok(EffectKey::TutorialSeenStack),
        "Passive" => Ok(EffectKey::Passive { kind: need_s("kind")? }),
        "Key" => {
            let color = match p.need_str("color").map_err(|e| schema(&pi, e))? {
                "yellow" => KeyColor::Yellow,
                "red" => KeyColor::Red,
                "blue" => KeyColor::Blue,
                other => {
                    return Err(Reject::UnknownField {
                        path: format!("{pi}.color"),
                        field: other.to_string(),
                    })
                }
            };
            Ok(EffectKey::Key { color })
        }
        "StatUp" => {
            let stat = match p.need_str("stat").map_err(|e| schema(&pi, e))? {
                "atk" => StatKind::Atk,
                "def" => StatKind::Def,
                other => {
                    return Err(Reject::UnknownField {
                        path: format!("{pi}.stat"),
                        field: other.to_string(),
                    })
                }
            };
            Ok(EffectKey::StatUp { stat })
        }
        "HpUp" => {
            let mul = need_i("mul")?;
            if mul <= 0 {
                return Err(Reject::FrameIndexOutOfBounds {
                    path: format!("{pi}.mul"),
                    index: 0,
                    table_len: 0,
                });
            }
            Ok(EffectKey::HpUp { mul })
        }
        "Equip" => {
            let tier = need_i("tier")?;
            if !(0..12).contains(&tier) {
                return Err(Reject::FrameIndexOutOfBounds {
                    path: format!("{pi}.tier"),
                    index: tier as i64,
                    table_len: 12,
                });
            }
            Ok(EffectKey::Equip { tier: tier as usize, bonus: need_i("bonus")? })
        }
        other => Err(Reject::UnknownField {
            path: format!("{path}.key"),
            field: other.to_string(),
        }),
    }
}

fn parse_envelope(src: &str, allowed_keys: &[&str]) -> Result<Json, Reject> {
    let doc = parse_json(src).map_err(|detail| Reject::BadJson { detail })?;
    check_keys(&doc, "<root>", allowed_keys)?;
    match doc.get("schema_version").map(Json::as_int) {
        Some(Some(v)) if v == SCHEMA_VERSION => {}
        Some(Some(v)) => {
            return Err(Reject::SchemaVersion { found: Some(v), expected: SCHEMA_VERSION })
        }
        Some(None) => return Err(schema("schema_version", "非整数")),
        None => return Err(Reject::SchemaVersion { found: None, expected: SCHEMA_VERSION }),
    }
    Ok(doc)
}

fn parse_sprite_def(rec: &Json, path: &str) -> Result<SpriteDef, Reject> {
    if !matches!(rec, Json::Obj(_)) {
        return Err(schema(path, "非对象"));
    }
    check_keys(rec, path, SPRITE_KEYS)?;
    let type_id = need_int(rec, path, "type_id")?;
    // 第 3 类：域溢出拒载——型号域 0..=88（§2.3；ANIM_TABLE_IDX 89 项）
    if !(TYPE_ID_MIN..=TYPE_ID_MAX).contains(&type_id) {
        return Err(Reject::DomainOverflow {
            path: format!("{path}.type_id"),
            value: type_id,
            domain: "0..=88",
        });
    }
    // sheet/sheet_index 可空（无图型，如 type 0；task-5 真表实形）
    let sheet = match rec.get("sheet") {
        None | Some(Json::Null) => None,
        Some(Json::Str(s)) => Some(s.clone()),
        Some(_) => return Err(schema(format!("{path}.sheet"), "非字符串/非 null")),
    };
    let sheet_index = match rec.get("sheet_index") {
        None | Some(Json::Null) => None,
        Some(Json::Int(v)) => Some(*v),
        Some(_) => return Err(schema(format!("{path}.sheet_index"), "非整数/非 null")),
    };
    let anim_row = match rec.get("anim_row") {
        None | Some(Json::Null) => None,
        Some(Json::Int(v)) => Some(*v),
        Some(_) => return Err(schema(format!("{path}.anim_row"), "非整数")),
    };
    let type_frame_width = match rec.get("type_frame_width") {
        None | Some(Json::Null) => None,
        Some(Json::Int(v)) => Some(*v),
        Some(_) => return Err(schema(format!("{path}.type_frame_width"), "非整数")),
    };
    let frame_w = need_int(rec, path, "frame_w")?;
    let frame_h = match rec.get("frame_h") {
        None | Some(Json::Null) => None,
        Some(Json::Int(v)) => Some(*v),
        Some(_) => return Err(schema(format!("{path}.frame_h"), "非整数/非 null")),
    };
    let render_category = need_int(rec, path, "render_category")?;
    let anim_profile = need_str(rec, path, "anim_profile")?.to_string();
    let assembly_profile = match rec.get("assembly_profile") {
        None | Some(Json::Null) => None,
        Some(Json::Str(s)) => Some(s.clone()),
        Some(_) => return Err(schema(format!("{path}.assembly_profile"), "非字符串/非 null")),
    };
    let insert_block = match rec.get("insert_block") {
        Some(Json::Bool(b)) => *b,
        Some(_) => return Err(schema(format!("{path}.insert_block"), "非布尔")),
        None => return Err(schema(format!("{path}.insert_block"), "缺失")),
    };
    let floor_category_override = match rec.get("floor_category_override") {
        None => Vec::new(),
        Some(Json::Arr(items)) => items
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let p = format!("{path}.floor_category_override[{i}]");
                if !matches!(a, Json::Obj(_)) {
                    return Err(schema(&p, "非对象"));
                }
                check_keys(a, &p, FLOOR_KEYS)?;
                Ok(FloorCategoryOverride {
                    floor: need_int(a, &p, "floor")?,
                    category: need_int(a, &p, "category")?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(schema(format!("{path}.floor_category_override"), "非数组")),
    };
    let transition_overrides = match rec.get("transition_overrides") {
        None => Vec::new(),
        Some(Json::Arr(items)) => items
            .iter()
            .enumerate()
            .map(|(i, a)| parse_transition_override(a, &format!("{path}.transition_overrides[{i}]")))
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(schema(format!("{path}.transition_overrides"), "非数组")),
    };
    let density = match rec.get("density") {
        None => 1, // 默认 1（plan 拍板 2 / HD 探针 R-5）
        Some(Json::Int(v)) => *v,
        Some(_) => return Err(schema(format!("{path}.density"), "非整数")),
    };
    let evidence = match rec.get("evidence") {
        None => Vec::new(),
        Some(Json::Arr(items)) => items
            .iter()
            .enumerate()
            .map(|(i, a)| parse_anchor(a, &format!("{path}.evidence[{i}]")))
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(schema(format!("{path}.evidence"), "非数组")),
    };
    Ok(SpriteDef {
        type_id,
        sheet,
        sheet_index,
        frame_w,
        frame_h,
        render_category,
        anim_row,
        type_frame_width,
        anim_profile,
        assembly_profile,
        insert_block,
        floor_category_override,
        transition_overrides,
        density,
        evidence,
    })
}

fn parse_transition_override(rec: &Json, path: &str) -> Result<TransitionOverride, Reject> {
    if !matches!(rec, Json::Obj(_)) {
        return Err(schema(path, "非对象"));
    }
    check_keys(rec, path, TRANSITION_KEYS)?;
    let on = need_str(rec, path, "on")?.to_string();
    let by_category_j = rec
        .get("by_category")
        .and_then(Json::as_arr_or_obj)
        .ok_or_else(|| schema(format!("{path}.by_category"), "缺失或非对象"))?;
    let mut by_category = Vec::new();
    for (k, v) in by_category_j {
        let p = format!("{path}.by_category.{k}");
        if !matches!(v, Json::Obj(_)) {
            return Err(schema(&p, "非对象"));
        }
        check_keys(v, &p, CATEGORY_KEYS)?;
        let anim_row = match v.get("anim_row") {
            None => None,
            Some(Json::Int(n)) => Some(*n),
            Some(_) => return Err(schema(format!("{p}.anim_row"), "非整数")),
        };
        let sprite_h = match v.get("sprite_h") {
            None => None,
            Some(Json::Int(n)) => Some(*n),
            Some(_) => return Err(schema(format!("{p}.sprite_h"), "非整数")),
        };
        by_category.push((k.clone(), CategoryOverride { anim_row, sprite_h }));
    }
    let evidence = match rec.get("evidence") {
        None | Some(Json::Null) => None,
        Some(Json::Str(s)) => Some(s.clone()),
        Some(_) => return Err(schema(format!("{path}.evidence"), "非字符串")),
    };
    Ok(TransitionOverride { on, by_category, evidence })
}

fn parse_anim_def(rec: &Json, path: &str) -> Result<AnimDef, Reject> {
    if !matches!(rec, Json::Obj(_)) {
        return Err(schema(path, "非对象"));
    }
    check_keys(rec, path, ANIM_KEYS)?;
    let id = need_str(rec, path, "id")?.to_string();
    let frames_j = expect_arr(rec, "frames", path)?;
    let frames = frames_j
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_int()
                .ok_or_else(|| schema(format!("{path}.frames[{i}]"), "非整数"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let step_ticks = need_int(rec, path, "step_ticks")?;
    let loop_mode = need_str(rec, path, "loop_mode")?.to_string();
    let evidence = match rec.get("evidence") {
        None => Vec::new(),
        Some(Json::Arr(items)) => items
            .iter()
            .enumerate()
            .map(|(i, a)| parse_anchor(a, &format!("{path}.evidence[{i}]")))
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(schema(format!("{path}.evidence"), "非数组")),
    };
    Ok(AnimDef { id, frames, step_ticks, loop_mode, evidence })
}

fn parse_anchor(rec: &Json, path: &str) -> Result<EvidenceAnchor, Reject> {
    if !matches!(rec, Json::Obj(_)) {
        return Err(schema(path, "非对象"));
    }
    check_keys(rec, path, ANCHOR_KEYS)?;
    let index = need_int(rec, path, "index")?;
    let value = rec
        .get("value")
        .cloned()
        .ok_or_else(|| schema(format!("{path}.value"), "缺失"))?;
    let src = need_str(rec, path, "src")?.to_string();
    Ok(EvidenceAnchor { index, value, src })
}

// ---------------------------------------------------------------- 联合加载（跨表校验层）

/// 三表联合加载结果（§2.3 外键政策 + 加载层失败语义的跨表校验层）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefBundle {
    /// 型→精灵属性表
    pub sprite_defs: Vec<SpriteDef>,
    /// 动画档表
    pub anim_defs: Vec<AnimDef>,
    /// 图集索引
    pub sheet_index: SheetIndex,
}

impl DefBundle {
    /// 三表联合加载：单表解析 + 跨表校验（id 稳定性/外键/帧序域）。
    ///
    /// `assembly_ids`：SEG 片件表（`assembly_profile` 外键目标，SEG67_PIECES/SEG67_FRAMES 族）
    /// 的 id 清单——该表在资源侧，由调用方提供（§2.3 跨表引用校验）。
    pub fn load(
        sprite_src: &str,
        anim_src: &str,
        sheet_src: &str,
        assembly_ids: &[&str],
    ) -> Result<DefBundle, Reject> {
        let sprite_defs = parse_sprite_defs(sprite_src)?;
        let anim_defs = parse_anim_defs(anim_src)?;
        let sheet_index = parse_sheet_index(sheet_src)?;
        if sheet_index.containers.len() != sheet_index.container_counts.len() {
            return Err(schema(
                "sheet_index.containers/container_counts",
                "containers 与 container_counts 长度不一致",
            ));
        }
        // id 稳定性（§2.3）：保真 id 冻结 ⇒ 重复 id 拒载
        for (i, s) in sprite_defs.iter().enumerate() {
            if sprite_defs[..i].iter().any(|p| p.type_id == s.type_id) {
                return Err(schema(
                    format!("sprite_defs[{i}].type_id"),
                    format!("type_id {} 重复（id 稳定性冻结）", s.type_id),
                ));
            }
        }
        for (i, a) in anim_defs.iter().enumerate() {
            if anim_defs[..i].iter().any(|p| p.id == a.id) {
                return Err(schema(
                    format!("anim_defs[{i}].id"),
                    format!("anim id {:?} 重复（id 稳定性冻结）", a.id),
                ));
            }
        }
        // 第 2 类：坏外键拒载（§2.3 孤儿/跨表引用校验）
        for (i, s) in sprite_defs.iter().enumerate() {
            if !anim_defs.iter().any(|a| a.id == s.anim_profile) {
                return Err(Reject::BadForeignKey {
                    path: format!("sprite_defs[{i}].anim_profile"),
                    detail: format!(
                        "anim_profile {:?} 在 anim_defs 中不存在（孤儿拒载）",
                        s.anim_profile
                    ),
                });
            }
            if let Some(asm) = &s.assembly_profile {
                let known = assembly_ids.contains(&asm.as_str())
                    || sheet_index.assembly_profiles.iter().any(|a| a == asm);
                if !known {
                    return Err(Reject::BadForeignKey {
                        path: format!("sprite_defs[{i}].assembly_profile"),
                        detail: format!(
                            "assembly_profile {asm:?} 在 SEG 片件表中不存在（跨表悬空拒载）"
                        ),
                    });
                }
            }
            if let Some(sheet) = &s.sheet {
                if !sheet_index.containers.contains(sheet) {
                    return Err(Reject::BadForeignKey {
                        path: format!("sprite_defs[{i}].sheet"),
                        detail: format!(
                            "sheet {sheet:?} 在 sheet_index.containers 中不存在（跨表悬空拒载）"
                        ),
                    });
                }
            }
        }
        // 第 4 类：帧序越界拒载（§2.3：frames 下标超表长=所引图集容器张数 container_counts；
        // 无图型 sheet=None 无帧表可越，跳过）
        for (i, s) in sprite_defs.iter().enumerate() {
            let Some(sheet) = &s.sheet else { continue };
            let cpos = sheet_index
                .containers
                .iter()
                .position(|c| c == sheet)
                .expect("FK 已校验 sheet 存在");
            let raw_len = sheet_index.container_counts[cpos];
            if raw_len < 0 {
                return Err(schema(
                    format!("sheet_index.container_counts[{cpos}]"),
                    "张数为负",
                ));
            }
            let table_len = raw_len as usize;
            let apos = anim_defs
                .iter()
                .position(|a| a.id == s.anim_profile)
                .expect("FK 已校验 anim_profile 存在");
            for (j, &f) in anim_defs[apos].frames.iter().enumerate() {
                if f < 0 || f as usize >= table_len {
                    return Err(Reject::FrameIndexOutOfBounds {
                        path: format!("anim_defs[{apos}].frames[{j}]（引用方 sprite_defs[{i}]）"),
                        index: f,
                        table_len,
                    });
                }
            }
        }
        Ok(DefBundle { sprite_defs, anim_defs, sheet_index })
    }

    /// `sprite_ref` 指空=拒载（§2.3 孤儿政策）；指向未知 id 同拒。
    pub fn validate_sprite_ref(&self, sprite_ref: Option<i64>) -> Result<(), Reject> {
        match sprite_ref {
            None => Err(Reject::BadForeignKey {
                path: "sprite_ref".to_string(),
                detail: "sprite_ref 指空=拒载（§2.3 孤儿政策）".to_string(),
            }),
            Some(id) if self.sprite_defs.iter().any(|s| s.type_id == id) => Ok(()),
            Some(id) => Err(Reject::BadForeignKey {
                path: "sprite_ref".to_string(),
                detail: format!("sprite_ref {id} 在 sprite_defs 中不存在（孤儿拒载）"),
            }),
        }
    }

    /// 级联政策（§2.3）：删 `sprite_defs` 行/`anim_defs` 行前校验引用——返回引用 `anim_id` 的型号清单。
    pub fn referencing_type_ids(&self, anim_id: &str) -> Vec<i64> {
        self.sprite_defs
            .iter()
            .filter(|s| s.anim_profile == anim_id)
            .map(|s| s.type_id)
            .collect()
    }
}

// ---------------------------------------------------------------- 小工具

/// unknown-field=拒载（第 5 类，防拼写静默）：对象键名必须全在 `allowed` 内。
/// `slots` 内部除外（§2.1 字段定义为对象、内部结构 schema 未定，不发明语义）。
fn check_keys(rec: &Json, path: &str, allowed: &[&str]) -> Result<(), Reject> {
    let members = match rec {
        Json::Obj(members) => members,
        _ => return Ok(()), // 形态错误由各 need_* 报 Schema
    };
    for (k, _) in members {
        if !allowed.contains(&k.as_str()) {
            return Err(Reject::UnknownField { path: path.to_string(), field: k.clone() });
        }
    }
    Ok(())
}

fn schema(path: impl Into<String>, reason: impl Into<String>) -> Reject {
    Reject::Schema { path: path.into(), reason: reason.into() }
}

fn need_int(rec: &Json, path: &str, key: &str) -> Result<i64, Reject> {
    rec.get(key)
        .and_then(Json::as_int)
        .ok_or_else(|| schema(format!("{path}.{key}"), "缺失或非整数"))
}

fn need_str<'a>(rec: &'a Json, path: &str, key: &str) -> Result<&'a str, Reject> {
    rec.get(key)
        .and_then(Json::as_str)
        .ok_or_else(|| schema(format!("{path}.{key}"), "缺失或非字符串"))
}

fn expect_arr<'a>(rec: &'a Json, key: &str, path: &str) -> Result<&'a [Json], Reject> {
    rec.get(key)
        .and_then(Json::as_arr)
        .ok_or_else(|| schema(format!("{path}.{key}"), "缺失或非数组"))
}

impl Json {
    /// 对象成员视图（非对象 ⇒ None）。
    fn as_arr_or_obj(&self) -> Option<&Vec<(String, Json)>> {
        match self {
            Json::Obj(members) => Some(members),
            _ => None,
        }
    }
}
