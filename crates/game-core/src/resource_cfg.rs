//! V2-P2 消费替换：资源配置表消费层（survey §2.1 顺序契约 / §4 R3 状态条件臂）。
//!
//! 真值宿主 = `data/tables/{sprite_defs,anim_defs,sheet_index}.json`（task-5 保真表，
//! 逐值锚三元组生成式产出；task-6 解析器五类拒载）。本模块把散在代码里的资源常量
//! （dispatchAnimFields 行号 / markEntityRemoved·markEntityDesRemoved 状态迁移特例行 /
//! dispatchSpriteSize 特例 / 图集槽位 / renderCategoryByType 10 臂）改由表驱动。
//!
//! **顺序契约钉死**（§2.1，反了 quirk 静默失效——R3 反例 2）：
//! **查表初始化（dispatchAnimFields a.java:6327-6330）→ 状态迁移覆写（transition_overrides）**。
//! 型 12 变形臂（a.java:5450-5457）的写序是契约的具象化：`type_id` →
//! `redispatch`（dispatchSpriteSize/dispatchAnimFields 重派=再查表初始化）→
//! 其余覆写字段（state/anim_row/visible）。重派若排到覆写之后，dispatchAnimFields
//! 会把 `anim_row=9` 吞回查表值——覆写静默失效。
//!
//! 宿主纪律：旧代码常量（`entity::ANIM_TABLE_IDX` 等）保留为**保真冻结镜像**，
//! 校验器断言双写 diff=0（一事实一宿主，survey §3 硬条件）——镜像只读，禁止回写。

use std::sync::OnceLock;

use game_data::asset_pipeline::{parse_json, Json};
use game_data::defs::{DefBundle, Reject};

use crate::entity::{self, EntityTable};

// ---------------------------------------------------------------- 嵌入保真表（确定性、零 I/O）

const SPRITE_SRC: &str = include_str!("../../../data/tables/sprite_defs.json");
const ANIM_SRC: &str = include_str!("../../../data/tables/anim_defs.json");
const SHEET_SRC: &str = include_str!("../../../data/tables/sheet_index.json");

// ---------------------------------------------------------------- 表结构（§2.1 事件条件规则形）

/// 规则臂的字段覆写（仅 `Some` 字段落写；`type_id`+`redispatch` 由 [`BumpOutcome`]
/// 路径按顺序契约处理）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldWrite {
    pub sprite_w: Option<i32>,
    pub sprite_h: Option<i32>,
    pub anim_row: Option<i32>,
    pub anim_frame: Option<i32>,
    pub visible: Option<bool>,
    pub state: Option<u8>,
    /// bump_reveal 变形目标型（a.java:5450 `entityType=11`）
    pub type_id: Option<i32>,
    /// Q-001 重派钩子（`["dispatchSpriteSize","dispatchAnimFields"]`——
    /// 落在 type_id 写入之后、其余覆写之前，顺序契约锚点）
    pub redispatch: Vec<String>,
    /// Q-004 阻挡标志（a.java:5456 `var6=false`；续扫 `continue` 语义留消费侧代码）
    pub block_step: bool,
}

impl FieldWrite {
    /// 落写普通字段（sprite/anim/visible/state）。`type_id`/`redispatch`/`block_step`
    /// 由调用路径按契约处理（见 [`ResourceCfg::apply_bump_reveal`]）。
    fn apply_fields(&self, table: &mut EntityTable, idx: usize) {
        if let Some(v) = self.sprite_w {
            table.items[idx].sprite_w = v;
        }
        if let Some(v) = self.sprite_h {
            table.items[idx].sprite_h = v;
        }
        if let Some(v) = self.anim_row {
            table.items[idx].anim_row = v;
        }
        if let Some(v) = self.anim_frame {
            table.items[idx].anim_frame = v;
        }
        if let Some(v) = self.visible {
            table.items[idx].visible = v;
        }
        if let Some(v) = self.state {
            table.items[idx].state = v;
        }
    }
}

/// `by_category` 臂（mark_removed cat 1 形态含内层 `by_type`/`otherwise`）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleArm {
    /// 直写字段（叶子臂）；容器臂（by_type/标志）为 None
    pub fields: Option<FieldWrite>,
    /// 内层按型分派（键形 `"1|2|3"`）
    pub by_type: Vec<(Vec<i32>, FieldWrite)>,
    /// 内层未命中兜底（noop 臂）
    pub otherwise: Option<FieldWrite>,
    /// Java case 1 显式 return（mark_des_removed cat 1：a.java:6335）
    pub returns: bool,
    /// mark_removed cat 1 臂尾 buildMinimap（a.java:6370）
    pub build_minimap: bool,
    /// 显式 noop 臂（mark_des_removed cat 1）
    pub noop: bool,
}

impl RuleArm {
    /// 取该型应落的覆写：有 by_type 先按型匹配，未中走内层 otherwise；叶子臂给直写字段。
    fn write_for(&self, t: i32) -> Option<&FieldWrite> {
        if self.noop {
            return None;
        }
        if !self.by_type.is_empty() {
            if let Some((_, w)) = self.by_type.iter().find(|(types, _)| types.contains(&t)) {
                return Some(w);
            }
            return self.otherwise.as_ref();
        }
        self.fields.as_ref()
    }
}

/// `transition_overrides` 一条（事件条件规则形，survey §2.1 ★R3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRule {
    /// 事件条件（`mark_removed` / `mark_des_removed` / `bump_reveal`）
    pub on: String,
    /// 规则级 state 写入（mark_removed=1 / mark_des_removed=2）
    pub state: Option<u8>,
    /// 按渲染分类分派（键形 `"1"` / `"2|4"` / `"8|16|32"`）
    pub by_category: Vec<(Vec<u8>, RuleArm)>,
    /// 按型分派（bump_reveal 形态）
    pub by_type: Vec<(Vec<i32>, FieldWrite)>,
    /// 无臂命中兜底（Java switch 无 default 臂 ⇒ noop）
    pub otherwise: Option<FieldWrite>,
    /// 证据锚
    pub evidence: Option<String>,
}

impl TransitionRule {
    fn arm_for(&self, cat: u8) -> Option<&RuleArm> {
        self.by_category
            .iter()
            .find(|(cats, _)| cats.is_empty() || cats.contains(&cat))
            .map(|(_, arm)| arm)
    }

    fn type_write(&self, t: i32) -> Option<&FieldWrite> {
        self.by_type.iter().find(|(types, _)| types.contains(&t)).map(|(_, w)| w)
    }
}

/// `size_dispatch.by_type` 一条（dispatchSpriteSize 特例臂）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeRule {
    pub frame_w: i32,
    pub frame_h: i32,
    /// case 67（a.java:6304-6305）：loadPackedImages(9) 后 `break` 落 fallback 臂——
    /// 表值即 fallback 值，语义走 fallback 而非字面（表改 fallback 时 67 随动）
    pub via_break_to_fallback: bool,
    /// Q-002 加载副作用锚（loadPackedImages(9)）——只留痕不消费
    /// （game-core 无 I/O；Rust 侧装载在 shell 构造期全量完成）
    pub load_packed_images: Option<i64>,
}

/// `size_dispatch`（dispatchSpriteSize a.java:6300-6325 的表形）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeDispatch {
    /// fallback 臂（32×32：default 臂图空落空 / case 67 break 落底）
    pub fallback: (i32, i32),
    /// 特例（67→break_to_fallback、69→96×32、72→32×45）
    pub by_type: Vec<(i32, SizeRule)>,
}

impl SizeDispatch {
    /// by_type 特例尺寸；None = default 臂（type_frame_width + 运行时图高）。
    pub fn special(&self, type_id: i32) -> Option<(i32, i32)> {
        self.rule(type_id).map(|r| (r.frame_w, r.frame_h))
    }

    /// by_type 特例规则。
    pub fn rule(&self, type_id: i32) -> Option<&SizeRule> {
        self.by_type.iter().find(|(t, _)| *t == type_id).map(|(_, r)| r)
    }
}

/// `quirks_in_code` 留痕条目（§2.1：装不下规则形的残余特例——禁静默留码）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quirk {
    pub quirk_id: String,
    pub reason: String,
    pub src: String,
    pub red_test_id: String,
}

/// `sheet_index.assembly_profiles.seg67`（SEG67 片件表：9 片件 xywh + 3×5 段帧）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seg67 {
    /// (sheet, src_x, src_y, w, h) 9 件（a.java:682-692 f_byte_arr2_00）
    pub pieces: Vec<(i32, i32, i32, i32, i32)>,
    /// 3 帧 × 5 段 (piece_id, dx, dy, 第 4 字节)
    pub frames: Vec<Vec<(i32, i32, i32, i32)>>,
}

// ---------------------------------------------------------------- 消费层

/// 三表联合配置（消费替换的单一查表入口）。
#[derive(Debug, Clone)]
pub struct ResourceCfg {
    bundle: DefBundle,
    transitions: Vec<TransitionRule>,
    size_dispatch: SizeDispatch,
    #[allow(dead_code)]
    quirks: Vec<Quirk>,
    seg67: Seg67,
    /// m_122 型→(容器,槽)（sheet_index.slots.claims 82 条覆盖 85 型；0/82/83/88 无槽）
    entity_slots: Vec<Option<(String, usize)>>,
}

/// bump_reveal 覆写结果（控制流语义回交消费侧——Q-004 续扫留在 scene 代码）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BumpOutcome {
    /// 命中 bump_reveal 规则
    pub applied: bool,
    /// 发生变形（type_id 覆写）
    pub morphed: bool,
    /// Q-004 阻挡（`var6=false`）
    pub block_step: bool,
}

impl ResourceCfg {
    /// 三表联合加载：task-6 解析器（五类拒载）+ 顶层 blob 结构化解析（task-7 消费面）。
    pub fn parse(sprite_src: &str, anim_src: &str, sheet_src: &str) -> Result<ResourceCfg, Reject> {
        // 五类拒载校验面（坏 JSON/坏外键/域溢出/帧序越界/unknown-field）——表真值过载
        let bundle = DefBundle::load(sprite_src, anim_src, sheet_src, &[])?;
        let sprite_doc = parse_json(sprite_src).map_err(|detail| Reject::BadJson { detail })?;
        let sheet_doc = parse_json(sheet_src).map_err(|detail| Reject::BadJson { detail })?;

        let transitions = match sprite_doc.get("transition_overrides") {
            Some(Json::Arr(items)) => items
                .iter()
                .enumerate()
                .map(|(i, r)| parse_transition_rule(r, &format!("transition_overrides[{i}]")))
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(schema("transition_overrides", "缺失或非数组")),
        };
        let size_dispatch = match sprite_doc.get("size_dispatch") {
            Some(obj @ Json::Obj(_)) => parse_size_dispatch(obj)?,
            _ => return Err(schema("size_dispatch", "缺失或非对象")),
        };
        let quirks = match sprite_doc.get("quirks_in_code") {
            Some(Json::Arr(items)) => items
                .iter()
                .enumerate()
                .map(|(i, q)| parse_quirk(q, &format!("quirks_in_code[{i}]")))
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(schema("quirks_in_code", "缺失或非数组")),
        };
        let seg67 = parse_seg67(&sheet_doc)?;
        let entity_slots = parse_entity_slots(&sheet_doc, &bundle)?;
        Ok(ResourceCfg { bundle, transitions, size_dispatch, quirks, seg67, entity_slots })
    }

    /// 嵌入保真表（`data/tables/*.json`，include_str! 编译期取数——无 I/O、确定性）。
    /// 表坏 = 装载失败（panic），与"保真表变更=勘误流程+金样重过"纪律配套。
    pub fn from_embedded() -> ResourceCfg {
        let cfg = ResourceCfg::parse(SPRITE_SRC, ANIM_SRC, SHEET_SRC)
            .expect("保真表装载失败（data/tables/*.json 坏——走勘误流程）");
        // 一事实一宿主校验（非荣誉制）：双写 diff=0——漂移=装载失败，
        // 处置=勘误流程+金样重过（survey §3 硬条件 / R2 保真红线）。
        cfg.validate_one_fact_one_host()
            .expect("一事实一宿主双写漂移（保真表 vs 冻结镜像）——走勘误流程+金样重过");
        cfg
    }

    /// 三表联合面（外键/帧序域校验产物；一事实一宿主校验器读面）。
    pub fn bundle(&self) -> &DefBundle {
        &self.bundle
    }

    /// SEG67 片件表（表中表）。
    pub fn seg67(&self) -> &Seg67 {
        &self.seg67
    }

    /// 16 容器名序（a.java:451 名序值行；张数表锚勘误=a.java:453 非 451）。
    pub fn container_names(&self) -> &[String] {
        &self.bundle.sheet_index.containers
    }

    /// m_122 型→(容器,槽) 查表（sheet_index.slots.claims）。
    /// None = 无图型/无臂型（0/82/83/88）。
    pub fn entity_sprite_slot(&self, type_id: i32) -> Option<(&str, usize)> {
        self.entity_slots
            .get(type_id as usize)
            .and_then(|e| e.as_ref())
            .map(|(c, s)| (c.as_str(), *s))
    }

    /// quirks_in_code 留痕（Q-001..Q-005）。
    pub fn quirks(&self) -> &[Quirk] {
        &self.quirks
    }

    fn sprite_def(&self, type_id: i32) -> Option<&game_data::defs::SpriteDef> {
        self.bundle.sprite_defs.iter().find(|s| s.type_id == type_id as i64)
    }

    fn transition(&self, on: &str) -> &TransitionRule {
        self.transitions
            .iter()
            .find(|r| r.on == on)
            .unwrap_or_else(|| panic!("transition_overrides 缺事件规则 {on:?}"))
    }

    /// 查表初始化行号（animRowByType，dispatchAnimFields a.java:6327-6330）。
    /// 型域外 = panic（域外三界①：ANIM_TABLE_IDX 89 项索引边）。
    pub fn anim_row(&self, type_id: i32) -> i32 {
        self.sprite_def(type_id)
            .unwrap_or_else(|| panic!("型号域 0..=88（sprite_defs 89 项）：{type_id}"))
            .anim_row
            .unwrap_or(0) as i32
    }

    /// 渲染分类（renderCategoryByType 静态语义；floor50 层相关覆写走
    /// [`Self::floor_category`]）。域外 = 0（旧 match `_ => 0` 语义保留）。
    pub fn render_category(&self, type_id: i32) -> u8 {
        self.sprite_def(type_id).map(|s| s.render_category).unwrap_or(0) as u8
    }

    /// 楼层分类覆写（floor_category_override；m_122 尾 a.java:9669-9667 的层相关语义）。
    pub fn floor_category(&self, type_id: i32, floor: i32) -> Option<u8> {
        self.sprite_def(type_id)?
            .floor_category_override
            .iter()
            .find(|o| o.floor == floor as i64)
            .map(|o| o.category as u8)
    }

    /// dispatchSpriteSize 特例尺寸（size_dispatch.by_type）；None = default 臂。
    pub fn size_special(&self, type_id: i32) -> Option<(i32, i32)> {
        self.size_dispatch.special(type_id)
    }

    /// dispatchSpriteSize 特例规则（size_dispatch.by_type）。
    pub fn size_rule(&self, type_id: i32) -> Option<&SizeRule> {
        self.size_dispatch.rule(type_id)
    }

    /// dispatchSpriteSize（a.java:6300-6325）：实体精灵尺寸分派（size_dispatch 表驱动）。
    /// 特例臂→表值；case 67（via=break_to_fallback）→fallback 臂；default 臂→
    /// type_frame_width 表宽×运行时图高；图空→fallback（Q-002：67/69 的
    /// loadPackedImages(9) 加载副作用留码，不属尺寸语义）。
    pub fn dispatch_sprite_size_into(
        &self,
        type_id: i32,
        idx: usize,
        table: &mut EntityTable,
        width_table: &[i32],
        img_h: Option<i32>,
    ) {
        if let Some(rule) = self.size_rule(type_id) {
            let (w, h) = if rule.via_break_to_fallback {
                self.size_fallback()
            } else {
                (rule.frame_w, rule.frame_h)
            };
            table.items[idx].sprite_w = w;
            table.items[idx].sprite_h = h;
            return;
        }
        if let Some(h) = img_h {
            table.items[idx].sprite_w = *width_table.get(type_id as usize).unwrap_or(&32);
            table.items[idx].sprite_h = h;
            return;
        }
        let (w, h) = self.size_fallback();
        table.items[idx].sprite_w = w;
        table.items[idx].sprite_h = h;
    }

    /// fallback 尺寸（32×32）。
    pub fn size_fallback(&self) -> (i32, i32) {
        self.size_dispatch.fallback
    }

    /// markEntityRemoved（a.java:6349-6387）状态迁移覆写（transition_overrides
    /// `mark_removed` 规则）。返回 build_minimap（cat 1 臂 a.java:6370）。
    pub fn apply_mark_removed(&self, table: &mut EntityTable, idx: usize) -> bool {
        let rule = self.transition("mark_removed");
        let t = table.items[idx].type_;
        // Java 写序：state 先落（a.java:6350），再按 renderCategoryByType 分派视觉字段
        if let Some(s) = rule.state {
            table.items[idx].state = s;
        }
        let cat = self.render_category(t);
        if let Some(arm) = rule.arm_for(cat) {
            let build_minimap = arm.build_minimap;
            if let Some(w) = arm.write_for(t) {
                w.apply_fields(table, idx);
            }
            build_minimap
        } else {
            if let Some(w) = &rule.otherwise {
                w.apply_fields(table, idx);
            }
            false
        }
    }

    /// markEntityDesRemoved（a.java:6332-6347）状态迁移覆写（`mark_des_removed` 规则）。
    /// Java switch **无 default 臂**：仅 case 1（return）/ 2/8/16/32（行 8 覆写）——
    /// 未列类目（0/4）视觉字段不动，由规则 `otherwise=noop` 承载。
    pub fn apply_mark_des_removed(&self, table: &mut EntityTable, idx: usize) {
        let rule = self.transition("mark_des_removed");
        let t = table.items[idx].type_;
        if let Some(s) = rule.state {
            table.items[idx].state = s;
        }
        let cat = self.render_category(t);
        if let Some(arm) = rule.arm_for(cat) {
            if let Some(w) = arm.write_for(t) {
                w.apply_fields(table, idx);
            }
        } else if let Some(w) = &rule.otherwise {
            w.apply_fields(table, idx);
        }
    }

    /// bump_reveal（显形行 9 路径）覆写。**顺序契约**（a.java:5450-5457 型 12 写序）：
    /// ① type_id 写入 → ② redispatch 钩子（dispatchSpriteSize/dispatchAnimFields
    /// 重派=查表初始化）→ ③ 其余覆写字段。重派后置会吞掉 anim_row 覆写（R3 反例 2）。
    pub fn apply_bump_reveal(
        &self,
        table: &mut EntityTable,
        idx: usize,
        width_table: &[i32],
        img_h: Option<i32>,
    ) -> BumpOutcome {
        let rule = self.transition("bump_reveal");
        let t = table.items[idx].type_;
        let Some(w) = rule.type_write(t) else {
            return BumpOutcome { applied: false, morphed: false, block_step: false };
        };
        // ① 型写入（a.java:5450 `entityType=11`）
        if let Some(new_t) = w.type_id {
            table.items[idx].type_ = new_t;
        }
        // ② 重派（Q-001：dispatchSpriteSize 需运行时图高——图高是运行时事实，
        //    规则形装不下，img_h 由消费侧提供；redispatch 序即 a.java:5451-5452）
        for hook in &w.redispatch {
            match hook.as_str() {
                "dispatchSpriteSize" => entity::dispatch_sprite_size(
                    table.items[idx].type_,
                    idx,
                    table,
                    width_table,
                    img_h,
                ),
                "dispatchAnimFields" => self.dispatch_anim_into(table.items[idx].type_, idx, table),
                other => panic!("transition_overrides 未知 redispatch 钩子：{other:?}"),
            }
        }
        // ③ 覆写字段（a.java:5453-5455 state/anim_row/visible）
        w.apply_fields(table, idx);
        BumpOutcome { applied: true, morphed: w.type_id.is_some(), block_step: w.block_step }
    }

    /// bump_reveal 变形目标型（Q-001 消费侧取运行时图高用；无变形规则 ⇒ None）。
    pub fn bump_reveal_target(&self, type_id: i32) -> Option<i32> {
        self.transition("bump_reveal").type_write(type_id).and_then(|w| w.type_id)
    }

    /// 一事实一宿主校验器（survey §3 硬条件）：四宿主双写 diff=0。
    ///
    /// 1. render_category：表宿主 vs [`crate::entity::legacy_render_category_mirror`]
    ///    （代码宿主=enemy_defs.sprite_ref 解引用值的今日仓形等价面；enemy_defs 属
    ///    表设计批，落地后其 sprite_ref 解引用侧直接加入本对拍）；
    /// 2. anim_row：表宿主 vs [`crate::entity::ANIM_TABLE_IDX`] 冻结镜像；
    /// 3. sprite_ref 解引用：sprite_defs.sheet/sheet_index vs
    ///    sheet_index.slots.claims 型→(容器,槽)（表间双写）；
    /// 4. size_dispatch 特例 vs 旧常量冻结值（69/72/67——width_cells 格数同源）。
    ///
    /// 返回 "OK" 或逐条漂移报告（全部收集，非首个即断）。
    pub fn validate_one_fact_one_host(&self) -> Result<String, String> {
        let mut diffs: Vec<String> = Vec::new();
        for t in 0i32..=88 {
            // 1. render_category 双写
            let table_rc = self.render_category(t);
            let mirror_rc = entity::legacy_render_category_mirror(t);
            if table_rc != mirror_rc {
                diffs.push(format!(
                    "render_category 型 {t}：表 {table_rc} != 冻结镜像 {mirror_rc}"
                ));
            }
            // 2. anim_row 双写
            let table_row = self.anim_row(t);
            let mirror_row = crate::entity::ANIM_TABLE_IDX[t as usize] as i32;
            if table_row != mirror_row {
                diffs.push(format!(
                    "anim_row 型 {t}：表 {table_row} != ANIM_TABLE_IDX 镜像 {mirror_row}"
                ));
            }
            // 3. sprite_ref 解引用双写（sprite_defs vs claims）
            let def = self.sprite_def(t).expect("型号域 0..=88（sprite_defs 89 项）");
            let table_slot = (def.sheet.as_deref(), def.sheet_index);
            let claim_slot = self.entity_sprite_slot(t).map(|(c, s)| (Some(c), Some(s as i64)));
            let claim_slot = match claim_slot {
                Some((c, s)) => (c, s),
                None => (None, None),
            };
            if table_slot != claim_slot {
                diffs.push(format!(
                    "sprite_ref 型 {t}：sprite_defs {table_slot:?} != claims 解引用 {claim_slot:?}"
                ));
            }
        }
        // 4. size_dispatch 特例双写（旧常量冻结值：a.java:6304-6314）
        for (t, want) in [(67i32, (32, 32)), (69, (96, 32)), (72, (32, 45))] {
            match self.size_special(t) {
                Some(got) if got == want => {}
                other => diffs.push(format!("size_dispatch 型 {t}：表 {other:?} != 冻结常量 {want:?}")),
            }
        }
        if self.size_fallback() != (32, 32) {
            diffs.push(format!(
                "size_dispatch fallback：表 {:?} != 冻结常量 (32, 32)",
                self.size_fallback()
            ));
        }
        if diffs.is_empty() {
            Ok("OK".to_string())
        } else {
            Err(diffs.join("\n"))
        }
    }

    /// dispatchAnimFields 查表初始化（行号=animRowByType[type]，帧 0）。
    pub fn dispatch_anim_into(&self, type_id: i32, idx: usize, table: &mut EntityTable) {
        table.items[idx].anim_row = self.anim_row(type_id);
        table.items[idx].anim_frame = 0;
    }
}

/// 进程级保真表单例（include_str! 嵌入表；确定性——同一二进制恒同配置）。
pub fn global() -> &'static ResourceCfg {
    static CFG: OnceLock<ResourceCfg> = OnceLock::new();
    CFG.get_or_init(ResourceCfg::from_embedded)
}

// ---------------------------------------------------------------- blob 解析（task-7 消费面）

fn schema(path: impl Into<String>, reason: impl Into<String>) -> Reject {
    Reject::Schema { path: path.into(), reason: reason.into() }
}

fn parse_field_write(rec: &Json, path: &str) -> Result<FieldWrite, Reject> {
    let mut w = FieldWrite::default();
    if !matches!(rec, Json::Obj(_)) {
        return Err(schema(path, "非对象"));
    }
    // 键序保持表内写序（bump_reveal 型 6 的 visible→state→anim_row = a.java:5413-5415）
    for (k, v) in obj_members(rec) {
        match k.as_str() {
            "sprite_w" => w.sprite_w = Some(int_val(v, path, k)? as i32),
            "sprite_h" => w.sprite_h = Some(int_val(v, path, k)? as i32),
            "anim_row" => w.anim_row = Some(int_val(v, path, k)? as i32),
            "anim_frame" => w.anim_frame = Some(int_val(v, path, k)? as i32),
            "state" => w.state = Some(int_val(v, path, k)? as u8),
            "type_id" => w.type_id = Some(int_val(v, path, k)? as i32),
            "visible" => match v {
                Json::Bool(b) => w.visible = Some(*b),
                _ => return Err(schema(format!("{path}.{k}"), "非布尔")),
            },
            "block_step" => match v {
                Json::Bool(b) => w.block_step = *b,
                _ => return Err(schema(format!("{path}.{k}"), "非布尔")),
            },
            "redispatch" => {
                w.redispatch = match v {
                    Json::Arr(items) => items
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            s.as_str().map(str::to_string).ok_or_else(|| {
                                schema(format!("{path}.redispatch[{i}]"), "非字符串")
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => return Err(schema(format!("{path}.redispatch"), "非数组")),
                }
            }
            // 臂级标志/容器键不在字段层消费
            "noop" | "returns" | "build_minimap" | "by_type" | "otherwise" | "src" => {}
            other => return Err(schema(path, format!("未知字段 {other:?}"))),
        }
    }
    Ok(w)
}

fn parse_rule_arm(rec: &Json, path: &str) -> Result<RuleArm, Reject> {
    if !matches!(rec, Json::Obj(_)) {
        return Err(schema(path, "非对象"));
    }
    let mut arm = RuleArm::default();
    if let Some(v) = rec.get("by_type") {
        arm.by_type = parse_type_map(v, &format!("{path}.by_type"))?;
    }
    match rec.get("otherwise") {
        None | Some(Json::Null) => {}
        Some(v) => arm.otherwise = Some(parse_field_write(v, &format!("{path}.otherwise"))?),
    }
    if let Some(Json::Bool(b)) = rec.get("build_minimap") {
        arm.build_minimap = *b;
    }
    if let Some(Json::Bool(b)) = rec.get("returns") {
        arm.returns = *b;
    }
    if let Some(Json::Bool(b)) = rec.get("noop") {
        arm.noop = *b;
    }
    // 叶子臂 = 直写字段键（sprite_*/anim_*/state/visible/type_id/redispatch/block_step）
    let is_leaf_key = |k: &str| {
        matches!(
            k,
            "sprite_w" | "sprite_h" | "anim_row" | "anim_frame" | "state" | "visible"
                | "type_id" | "redispatch" | "block_step"
        )
    };
    if obj_members(rec).iter().any(|(k, _)| is_leaf_key(k)) {
        let leaf = Json::Obj(
            obj_members(rec)
                .iter()
                .filter(|(k, _)| is_leaf_key(k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );
        arm.fields = Some(parse_field_write(&leaf, path)?);
    }
    Ok(arm)
}

fn parse_type_map(rec: &Json, path: &str) -> Result<Vec<(Vec<i32>, FieldWrite)>, Reject> {
    let mut out = Vec::new();
    for (k, v) in obj_members(rec) {
        let types = split_ints(k)
            .ok_or_else(|| schema(path, format!("型键 {k:?} 非 `a|b|c` 形态")))?;
        out.push((types, parse_field_write(v, &format!("{path}.{k}"))?));
    }
    Ok(out)
}

fn parse_category_map(rec: &Json, path: &str) -> Result<Vec<(Vec<u8>, RuleArm)>, Reject> {
    let mut out = Vec::new();
    for (k, v) in obj_members(rec) {
        // `"*"` = 通配（survey §2.1 示例形态；保真表未用）
        let cats = if k == "*" {
            Vec::new()
        } else {
            split_ints(k)
                .and_then(|v| v.into_iter().map(|x| u8::try_from(x).ok()).collect())
                .ok_or_else(|| schema(path, format!("类目键 {k:?} 非 `a|b|c` 形态")))?
        };
        out.push((cats, parse_rule_arm(v, &format!("{path}.{k}"))?));
    }
    Ok(out)
}

fn parse_transition_rule(rec: &Json, path: &str) -> Result<TransitionRule, Reject> {
    if !matches!(rec, Json::Obj(_)) {
        return Err(schema(path, "非对象"));
    }
    let on = rec
        .get("on")
        .and_then(Json::as_str)
        .ok_or_else(|| schema(format!("{path}.on"), "缺失或非字符串"))?
        .to_string();
    let state = match rec.get("state") {
        None | Some(Json::Null) => None,
        Some(v) => Some(int_val(v, path, "state")? as u8),
    };
    let by_category = match rec.get("by_category") {
        None | Some(Json::Null) => Vec::new(),
        Some(v) => parse_category_map(v, &format!("{path}.by_category"))?,
    };
    let by_type = match rec.get("by_type") {
        None | Some(Json::Null) => Vec::new(),
        Some(v) => parse_type_map(v, &format!("{path}.by_type"))?,
    };
    let otherwise = match rec.get("otherwise") {
        None | Some(Json::Null) => None,
        Some(v) => Some(parse_field_write(v, &format!("{path}.otherwise"))?),
    };
    let evidence = rec.get("evidence").and_then(Json::as_str).map(str::to_string);
    Ok(TransitionRule { on, state, by_category, by_type, otherwise, evidence })
}

fn parse_size_dispatch(rec: &Json) -> Result<SizeDispatch, Reject> {
    let path = "size_dispatch";
    let fb = rec
        .get("fallback")
        .ok_or_else(|| schema(format!("{path}.fallback"), "缺失"))?;
    let fallback = (
        int(fb, path, "frame_w")? as i32,
        int(fb, path, "frame_h")? as i32,
    );
    let mut by_type = Vec::new();
    let by_type_j = rec
        .get("by_type")
        .ok_or_else(|| schema(format!("{path}.by_type"), "缺失"))?;
    for (k, v) in obj_members(by_type_j) {
        let t: i32 = k
            .parse()
            .map_err(|_| schema(format!("{path}.by_type"), format!("型键 {k:?} 非整数")))?;
        let rule_path = format!("{path}.by_type.{k}");
        let via_break_to_fallback = match v.get("via") {
            None | Some(Json::Null) => false,
            Some(Json::Str(s)) if s == "break_to_fallback" => true,
            Some(_) => return Err(schema(format!("{rule_path}.via"), "未知 via 值")),
        };
        let load_packed_images = match v.get("load_packed_images") {
            None | Some(Json::Null) => None,
            Some(j) => Some(int_val(j, &rule_path, "load_packed_images")?),
        };
        by_type.push((t, SizeRule {
            frame_w: int(v, &rule_path, "frame_w")? as i32,
            frame_h: int(v, &rule_path, "frame_h")? as i32,
            via_break_to_fallback,
            load_packed_images,
        }));
    }
    Ok(SizeDispatch { fallback, by_type })
}

fn parse_quirk(rec: &Json, path: &str) -> Result<Quirk, Reject> {
    let get = |k: &str| -> Result<String, Reject> {
        rec.get(k)
            .and_then(Json::as_str)
            .map(str::to_string)
            .ok_or_else(|| schema(format!("{path}.{k}"), "缺失或非字符串"))
    };
    Ok(Quirk {
        quirk_id: get("quirk_id")?,
        reason: get("reason")?,
        src: get("src")?,
        red_test_id: get("red_test_id")?,
    })
}

/// sheet_index.slots.claims → 型→(容器,槽) 映射（含容器名序交叉校验+槽位域校验）。
fn parse_entity_slots(sheet_doc: &Json, bundle: &DefBundle) -> Result<Vec<Option<(String, usize)>>, Reject> {
    let mut entity_slots: Vec<Option<(String, usize)>> = vec![None; 89];
    let claims = match sheet_doc.get("slots").and_then(|s| s.get("claims")) {
        Some(Json::Arr(items)) => items.as_slice(),
        _ => return Err(schema("sheet_index.slots.claims", "缺失或非数组")),
    };
    for (i, c) in claims.iter().enumerate() {
        let path = format!("sheet_index.slots.claims[{i}]");
        let container = c
            .get("container")
            .and_then(Json::as_str)
            .ok_or_else(|| schema(format!("{path}.container"), "缺失或非字符串"))?
            .to_string();
        let container_index = int(c, &path, "container_index")? as usize;
        // 名序交叉校验（containers=claims 容器索引的宿主）
        if bundle.sheet_index.containers.get(container_index).map(|s| s.as_str())
            != Some(container.as_str())
        {
            return Err(schema(
                format!("{path}.container_index"),
                "与 sheet_index.containers 名序不一致",
            ));
        }
        let slot = int(c, &path, "slot")? as usize;
        // 槽位域校验（container_counts=槽位上界宿主，张数表锚 a.java:453）
        if bundle.sheet_index.container_counts.get(container_index).copied().unwrap_or(-1) < slot as i64 + 1 {
            return Err(schema(
                format!("{path}.slot"),
                format!("槽位 {slot} 越界 container_counts[{container_index}]"),
            ));
        }
        let types = c
            .get("used_by_types")
            .and_then(Json::as_arr)
            .ok_or_else(|| schema(format!("{path}.used_by_types"), "缺失或非数组"))?;
        for (j, ty) in types.iter().enumerate() {
            let t = int_val(ty, &path, "used_by_types")? as usize;
            if t >= 89 {
                return Err(schema(
                    format!("{path}.used_by_types[{j}]"),
                    "型号域 0..=88（map_entity_images 89 槽）",
                ));
            }
            if entity_slots[t].is_some() {
                return Err(schema(
                    format!("{path}.used_by_types[{j}]"),
                    format!("型 {t} 重复占用（id 稳定性冻结）"),
                ));
            }
            entity_slots[t] = Some((container.clone(), slot));
        }
    }
    Ok(entity_slots)
}

fn parse_seg67(sheet_doc: &Json) -> Result<Seg67, Reject> {
    let profiles = sheet_doc
        .get("assembly_profiles")
        .ok_or_else(|| schema("sheet_index.assembly_profiles", "缺失"))?;
    let seg = profiles
        .get("seg67")
        .ok_or_else(|| schema("sheet_index.assembly_profiles.seg67", "缺失"))?;
    let path = "sheet_index.assembly_profiles.seg67";
    let pieces_j = seg
        .get("pieces")
        .and_then(Json::as_arr)
        .ok_or_else(|| schema(format!("{path}.pieces"), "缺失或非数组"))?;
    let mut pieces = Vec::new();
    for (i, p) in pieces_j.iter().enumerate() {
        let vals = p
            .as_arr()
            .ok_or_else(|| schema(format!("{path}.pieces[{i}]"), "非数组"))?;
        if vals.len() != 5 {
            return Err(schema(format!("{path}.pieces[{i}]"), "片件应为 5 元组"));
        }
        pieces.push((
            vals[0].as_int().unwrap_or(0) as i32,
            vals[1].as_int().unwrap_or(0) as i32,
            vals[2].as_int().unwrap_or(0) as i32,
            vals[3].as_int().unwrap_or(0) as i32,
            vals[4].as_int().unwrap_or(0) as i32,
        ));
    }
    let frames_j = seg
        .get("frames")
        .and_then(Json::as_arr)
        .ok_or_else(|| schema(format!("{path}.frames"), "缺失或非数组"))?;
    let mut frames = Vec::new();
    for (i, f) in frames_j.iter().enumerate() {
        let segs_j = f
            .as_arr()
            .ok_or_else(|| schema(format!("{path}.frames[{i}]"), "非数组"))?;
        let mut segs = Vec::new();
        for (j, s) in segs_j.iter().enumerate() {
            let vals = s
                .as_arr()
                .ok_or_else(|| schema(format!("{path}.frames[{i}][{j}]"), "非数组"))?;
            if vals.len() != 4 {
                return Err(schema(format!("{path}.frames[{i}][{j}]"), "段帧应为 4 元组"));
            }
            segs.push((
                vals[0].as_int().unwrap_or(0) as i32,
                vals[1].as_int().unwrap_or(0) as i32,
                vals[2].as_int().unwrap_or(0) as i32,
                vals[3].as_int().unwrap_or(0) as i32,
            ));
        }
        frames.push(segs);
    }
    Ok(Seg67 { pieces, frames })
}

// ---------------------------------------------------------------- 小工具

fn obj_members(rec: &Json) -> &[(String, Json)] {
    match rec {
        Json::Obj(members) => members,
        _ => &[],
    }
}

fn int(rec: &Json, path: &str, key: &str) -> Result<i64, Reject> {
    rec.get(key)
        .and_then(Json::as_int)
        .ok_or_else(|| schema(format!("{path}.{key}"), "缺失或非整数"))
}

/// 值层整数（已是字段值，不再按键查找）。
fn int_val(v: &Json, path: &str, key: &str) -> Result<i64, Reject> {
    v.as_int().ok_or_else(|| schema(format!("{path}.{key}"), "缺失或非整数"))
}

/// `"1|2|3"` → [1,2,3]；`"11"` → [11]。
fn split_ints(key: &str) -> Option<Vec<i32>> {
    let mut out = Vec::new();
    for part in key.split('|') {
        out.push(part.parse().ok()?);
    }
    (!out.is_empty()).then_some(out)
}
