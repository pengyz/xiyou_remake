//! 【分层收缩搜索 S2】外层骨架（提案 §7 S2 / §2.2 / §3.5）。
//!
//! 职责边界：
//! - **消费 S1 块表** `docs/findings/block-interfaces.json`（schema
//!   `xiyou/block-interfaces/v2`）——按「机制/题面分离」它是**输入数据**：
//!   加载失败一律具名 [`OuterError`]，**禁静默回落**（map-vs-mechanism 规矩）。
//! - 外层状态 =（层，块，keys[y/r/b]，hp，gold，atk，def，关键道具集，picked 集合，步数）。
//!   磨属性（H6，用户已拍板）：商店 id6 `StatPct` 免费可重复，**每次购买成本 = 1 步**
//!   （触发商人对话的最小行动），作为显式行动进转移；不复利推导上界，让搜索自己停
//!   （节点帽 [`SearchBudget`]）。
//! - 拾取效果**零复制**：一律走引擎采样面
//!   [`game_core::effect_registry::certified_monotone_pickup`]（与 ItemResolver 同分发）。
//! - 锚可杀性判据零复制：走 [`game_core::rules::sensitivity`]（game-core 单一真相源）。
//!
//! v1 口径（显式、不猜）——`sealed4`/`sealed81`/`script_gate` 三种切块门**按可通行、
//! 零钥匙代价**处理（乐观口径，与 S1 dynreach 同族），每条经过它的路径记
//! `optimistic_gate` 注记并在输出汇总里具名；真实开关语义（flags/脚本）是 S3 边界。

use game_core::effect_registry::certified_monotone_pickup;
use game_core::rules::sensitivity::{sensitivity_table, SensitivityRow};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};

pub const BLOCK_IFACE_SCHEMA: &str = "xiyou/block-interfaces/v2";

/// 具名加载/前提错误（禁静默回落——map-vs-mechanism 规矩）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OuterError {
    BadJson(String),
    SchemaMismatch { got: String },
    MissingField(String),
    NoAnchorBlock(i32),
    /// 敏感表缺该怪（表来自调用方的缩放敌表）——具名能力缺口。
    NeedsSensitivity(i32),
}

impl std::fmt::Display for OuterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OuterError::BadJson(s) => write!(f, "block-interfaces.json 解析失败: {s}"),
            OuterError::SchemaMismatch { got } => {
                write!(f, "块表 schema 不符: 期望 {BLOCK_IFACE_SCHEMA}, 实际 {got}")
            }
            OuterError::MissingField(s) => write!(f, "块表缺字段: {s}"),
            OuterError::NoAnchorBlock(ty) => write!(f, "锚 boss ty{ty} 不在任何块的 exits"),
            OuterError::NeedsSensitivity(ty) => write!(f, "needs_sensitivity({ty})"),
        }
    }
}

// ---------- S1 块表（输入数据面） ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutDoorDoc {
    pub cell: (i32, i32),
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportDoc {
    pub ty: i32,
    pub cell: (i32, i32),
    pub class: String,
    /// 商店货单 `[price, effect_key, params_json]`（S1 按 (src,idx) 匹配后的投影）。
    pub goods: Vec<(i32, String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitDoc {
    pub kind: String,
    pub ty: Option<i32>,
    pub slot: Option<i32>,
    pub cell: (i32, i32),
    pub to: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDoc {
    pub id: String,
    pub cells: Vec<(i32, i32)>,
    pub cut_doors: Vec<CutDoorDoc>,
    pub exports: Vec<ExportDoc>,
    pub exits: Vec<ExitDoc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorDoc {
    pub floor: i32,
    pub blocks: Vec<BlockDoc>,
    pub entry: Option<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInterfaces {
    pub floors: BTreeMap<i32, FloorDoc>,
}

fn jcell(v: &game_data::asset_pipeline::Json, what: &str) -> Result<(i32, i32), OuterError> {
    let arr = v
        .as_arr()
        .ok_or_else(|| OuterError::MissingField(format!("{what} 非数组")))?;
    let x = arr
        .first()
        .and_then(game_data::asset_pipeline::Json::as_int)
        .ok_or_else(|| OuterError::MissingField(format!("{what}[0] 非整数")))?;
    let y = arr
        .get(1)
        .and_then(game_data::asset_pipeline::Json::as_int)
        .ok_or_else(|| OuterError::MissingField(format!("{what}[1] 非整数")))?;
    Ok((x as i32, y as i32))
}

/// 解析块表。schema 不符 / 字段缺失 ⇒ 具名 Err（禁回落）。
pub fn load_block_interfaces(raw: &str) -> Result<BlockInterfaces, OuterError> {
    let root = game_data::asset_pipeline::parse_json(raw)
        .map_err(OuterError::BadJson)?;
    let got = root
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OuterError::MissingField("schema".into()))?
        .to_string();
        if got != BLOCK_IFACE_SCHEMA {
        return Err(OuterError::SchemaMismatch { got });
    }
    let mut floors = BTreeMap::new();
    for fv in root
        .get("floors")
        .and_then(|v| v.as_arr())
        .ok_or_else(|| OuterError::MissingField("floors".into()))?
    {
        let floor = fv
            .get("floor")
            .and_then(|v| v.as_int())
            .ok_or_else(|| OuterError::MissingField("floors[].floor".into()))?
            as i32;
        // entry=null（块图标注 no_entry 的层，如 f51/f1）是合法输入：
        // 外层起点由 RunSpec.start 具名给出，这里记 Option 让 build 的 start_node 主持。
        let entry = match fv.get("entry") {
            None => None,
            Some(v) if v.as_arr().is_none() => None,
            Some(v) => Some(jcell(v, "entry")?),
        };
        let mut blocks = Vec::new();
        for bv in fv
            .get("blocks")
            .and_then(|v| v.as_arr())
            .ok_or_else(|| OuterError::MissingField("floors[].blocks".into()))?
        {
            let id = bv
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| OuterError::MissingField("blocks[].id".into()))?
                .to_string();
            let mut cells = Vec::new();
            for cv in bv
                .get("cells")
                .and_then(|v| v.as_arr())
                .ok_or_else(|| OuterError::MissingField("blocks[].cells".into()))?
            {
                cells.push(jcell(cv, "cells[]")?);
            }
            let mut cut_doors = Vec::new();
            for dv in bv
                .get("cut_doors")
                .and_then(|v| v.as_arr())
                .ok_or_else(|| OuterError::MissingField("blocks[].cut_doors".into()))?
            {
                cut_doors.push(CutDoorDoc {
                    cell: jcell(dv.get("cell").ok_or_else(|| OuterError::MissingField("cut_doors[].cell".into()))?, "cut_doors[].cell")?,
                    kind: dv
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| OuterError::MissingField("cut_doors[].kind".into()))?
                        .to_string(),
                });
            }
            let mut exports = Vec::new();
            for ev in bv
                .get("exports")
                .and_then(|v| v.as_arr())
                .ok_or_else(|| OuterError::MissingField("blocks[].exports".into()))?
            {
                let mut goods = Vec::new();
                if let Some(garr) = ev.get("goods").and_then(|v| v.as_arr()) {
                    for g in garr {
                        let garr = g
                            .as_arr()
                            .ok_or_else(|| OuterError::MissingField("goods[] 非数组".into()))?;
                        let price = garr
                            .first()
                            .and_then(|v| v.as_int())
                            .ok_or_else(|| OuterError::MissingField("goods[][0] price".into()))?
                            as i32;
                        let key = garr
                            .get(1)
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| OuterError::MissingField("goods[][1] key".into()))?
                            .to_string();
                        let params = garr
                            .get(2)
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| OuterError::MissingField("goods[][2] params".into()))?
                            .to_string();
                        goods.push((price, key, params));
                    }
                }
                exports.push(ExportDoc {
                    ty: ev
                        .get("ty")
                        .and_then(|v| v.as_int())
                        .ok_or_else(|| OuterError::MissingField("exports[].ty".into()))?
                        as i32,
                    cell: jcell(ev.get("cell").ok_or_else(|| OuterError::MissingField("exports[].cell".into()))?, "exports[].cell")?,
                    class: ev
                        .get("class")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    goods,
                });
            }
            let mut exits = Vec::new();
            for xv in bv
                .get("exits")
                .and_then(|v| v.as_arr())
                .ok_or_else(|| OuterError::MissingField("blocks[].exits".into()))?
            {
                exits.push(ExitDoc {
                    kind: xv
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| OuterError::MissingField("exits[].kind".into()))?
                        .to_string(),
                    ty: xv.get("ty").and_then(|v| v.as_int()).map(|v| v as i32),
                    slot: xv.get("slot").and_then(|v| v.as_int()).map(|v| v as i32),
                    cell: jcell(xv.get("cell").ok_or_else(|| OuterError::MissingField("exits[].cell".into()))?, "exits[].cell")?,
                    to: xv.get("to").and_then(|v| v.as_int()).map(|v| v as i32),
                });
            }
            blocks.push(BlockDoc { id, cells, cut_doors, exports, exits });
        }
        floors.insert(floor, FloorDoc { floor, blocks, entry });
    }
    Ok(BlockInterfaces { floors })
}

// ---------- 段计划（v1：第 1 段可跑，其余具名 Off） ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSpec {
    /// 主线拓扑序的层（锚所在层收尾）。
    pub floors: Vec<i32>,
    /// （层, 格）起点——层在 floors[0]，格是块表坐标。
    pub start: (i32, (i32, i32)),
    pub anchor_floor: i32,
    /// 锚 boss 的怪 ty（敏感表判据）。
    pub anchor_boss_ty: i32,
    /// S3 A 片断链注入开关：屏蔽这些（层, 格）出口边（模拟「f4 出口链断开」）。
    pub masked_exits: Vec<(i32, (i32, i32))>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegmentPlan {
    Run(RunSpec),
    /// 具名跳过：能力/数据缺失，禁静默。
    Off { name: String, reason: String },
}

/// 主线拓扑序（修订版提案 §4 锚序：f53 捆仙绳 → f15 金勺子 → f19 太公杖 → 杨戬 → f50）。
/// v1 只展开第 1 段；f25..f49 支线具名跳过（S3）。
pub fn plan_segments(doc: &BlockInterfaces) -> Vec<(String, SegmentPlan)> {
    let mut plans = Vec::new();
    let seg1: Vec<i32> = [51, 52, 53, 54, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].to_vec();
    let anchor = anchor_block(&doc, 10, 75);
    let missing = anchor.is_err();
    plans.push((
        "seg1_f51-f54_f1-f10".into(),
        if missing {
            SegmentPlan::Off {
                name: "seg1_f51-f54_f1-f10".into(),
                reason: anchor.err().unwrap().to_string(),
            }
        } else {
            SegmentPlan::Run(RunSpec {
                floors: seg1,
                start: (51, (1, 11)),
                anchor_floor: 10,
                anchor_boss_ty: 75,
                masked_exits: Vec::new(),
            })
        },
    ));
    plans.push((
        "seg2_f11-f15".into(),
        SegmentPlan::Off {
            name: "seg2_f11-f15".into(),
            reason: "s2_minimum_loop_segment1_only".into(),
        },
    ));
    plans.push((
        "seg3_f16-f24".into(),
        SegmentPlan::Off {
            name: "seg3_f16-f24".into(),
            reason: "needs_sensitivity(69)".into(), // 芭蕉扇唯一来源 boss69@f35 在段后（矛盾探测器另记）
        },
    ));
    plans.push((
        "branch_f25-f49".into(),
        SegmentPlan::Off {
            name: "branch_f25-f49".into(),
            reason: "branch_floor_no_block_graph(25)".into(),
        },
    ));
    plans.push((
        "segfinal_f50".into(),
        SegmentPlan::Off {
            name: "segfinal_f50".into(),
            reason: "s2_minimum_loop_segment1_only".into(),
        },
    ));
    plans
}

/// 锚块：某层含 boss_kill ty 的块下标。
pub fn anchor_block(doc: &BlockInterfaces, floor: i32, boss_ty: i32) -> Result<usize, OuterError> {
    let fd = doc
        .floors
        .get(&floor)
        .ok_or_else(|| OuterError::MissingField(format!("floors[{floor}] 不在块表")))?;
    fd.blocks
        .iter()
        .position(|b| b.exits.iter().any(|e| e.kind == "boss_kill" && e.ty == Some(boss_ty)))
        .ok_or(OuterError::NoAnchorBlock(boss_ty))
}

// ---------- 外层图 ----------

/// 节点 id =（层，块下标）。切块门需求：door1→黄、door2→红、door3→蓝；
/// sealed4/sealed81/script_gate → 乐观通行（记 `optimistic_gate`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeDemand {
    pub to: usize,
    pub yellow: i32,
    pub red: i32,
    pub blue: i32,
    pub optimistic_gates: u32,
}

pub struct OuterGraph<'a> {
    pub doc: &'a BlockInterfaces,
    /// 段内层序（RunSpec.floors）。
    pub floors: Vec<i32>,
    /// (floor, block_idx) → 节点 id。
    pub node: HashMap<(i32, usize), usize>,
    pub floor_of: Vec<i32>,
    pub block_of: Vec<usize>,
    pub edges: Vec<Vec<EdgeDemand>>,
    /// 楼梯/脚本换层边：from node → to node。
    pub trans: Vec<Vec<(usize, String)>>,
    pub skipped_trans: Vec<String>,
    /// S3 A 片：被屏蔽的出口边（具名回显，禁静默）。
    pub masked_exits: Vec<String>,
}

impl<'a> OuterGraph<'a> {
    /// S3 A 片：问题空间 = 全部可达层（块表全量建图）。
    pub fn build_all(doc: &'a BlockInterfaces) -> OuterGraph<'a> {
        let floors: Vec<i32> = doc.floors.keys().copied().collect();
        Self::build(doc, &floors, &[])
    }

    /// `masked`：屏蔽这些（层, 格）出口边（断链注入开关）。
    pub fn build(doc: &'a BlockInterfaces, floors: &[i32], masked: &[(i32, (i32, i32))]) -> OuterGraph<'a> {
        let masked: HashSet<(i32, (i32, i32))> = masked.iter().copied().collect();
        let mut masked_report = Vec::new();
        let mut node = HashMap::new();
        let mut floor_of = Vec::new();
        let mut block_of = Vec::new();
        for &f in floors {
            if let Some(fd) = doc.floors.get(&f) {
                for i in 0..fd.blocks.len() {
                    node.insert((f, i), floor_of.len());
                    floor_of.push(f);
                    block_of.push(i);
                }
            }
        }
        let n = floor_of.len();
        let mut edges: Vec<Vec<EdgeDemand>> = vec![Vec::new(); n];
        let mut trans: Vec<Vec<(usize, String)>> = vec![Vec::new(); n];
        let mut skipped_trans = Vec::new();
        for ni in 0..n {
            let f = floor_of[ni];
            let fd = &doc.floors[&f];
            let bi = block_of[ni];
            let bd = &fd.blocks[bi];
            // 层内切块边（成对收集两侧 cut_doors）
            for (nj, other) in fd.blocks.iter().enumerate() {
                if nj == bi {
                    continue;
                }
                let acell: HashSet<(i32, i32)> = bd.cells.iter().copied().collect();
                let bcell: HashSet<(i32, i32)> = other.cells.iter().copied().collect();
                let near = |from: &HashSet<(i32, i32)>, c: &(i32, i32)| {
                    from.contains(&(c.0 + 1, c.1))
                        || from.contains(&(c.0 - 1, c.1))
                        || from.contains(&(c.0, c.1 + 1))
                        || from.contains(&(c.0, c.1 - 1))
                };
                // 两块相邻 = 单元直接相邻，或隔着一格「门格」（该门格是两块之一的
                // cut_door——门不隶属任何块）。前者无门需求，后者按门计需求。
                let mut y = 0;
                let mut r = 0;
                let mut bl = 0;
                let mut gates = 0;
                let mut touching = acell.iter().any(|c| near(&bcell, c));
                for (owner, other2) in [(bd, other), (other, bd)] {
                    let oc: HashSet<(i32, i32)> = owner.cells.iter().copied().collect();
                    let tc: HashSet<(i32, i32)> = other2.cells.iter().copied().collect();
                    for cd in &owner.cut_doors {
                        if near(&oc, &cd.cell) && near(&tc, &cd.cell) {
                            touching = true;
                            match cd.kind.as_str() {
                                "door1" => y += 1,
                                "door2" => r += 1,
                                "door3" => bl += 1,
                                "sealed4" | "sealed81" | "script_gate" => gates += 1,
                                _ => {}
                            }
                        }
                    }
                }
                if !touching {
                    continue;
                }
                edges[ni].push(EdgeDemand {
                    to: node[&(f, nj)],
                    yellow: y,
                    red: r,
                    blue: bl,
                    optimistic_gates: gates,
                });
            }
            // 换层边
            for ex in &bd.exits {
                let Some(to_floor) = ex.to else { continue };
                if !floors.contains(&to_floor) {
                    continue;
                }
                // S3 A 片断链注入：屏蔽指定（层, 格）出口边（具名回显，禁静默）。
                if masked.contains(&(f, ex.cell)) {
                    masked_report.push(format!("f{}:({},{})", f, ex.cell.0, ex.cell.1));
                    continue;
                }
                let to_fd = &doc.floors[&to_floor];
                // v1 乐观口径，优先级链（逐级回落、全走乐观）：
                // 1) 含到场格的块（如 f54→f1 落 (7,1)∈f1b0）
                // 2) 含反向梯的块（孤梯格如 f52→f53 落 (11,11)，反向 ty8(6,11)∈f53b1）
                // 3) 全部**邻接到场格**的块（块间通道格如 f3 script→f2 (2,11)）
                // 4) 具名 skip
                let (cx, cy) = ex.cell;
                let near = |b2: &BlockDoc| -> bool {
                    b2.cells
                        .iter()
                        .any(|&(x, y)| (x - cx).abs() + (y - cy).abs() == 1)
                };
                let in_cell: Vec<usize> = to_fd
                    .blocks
                    .iter()
                    .enumerate()
                    .filter(|(_, b2)| b2.cells.contains(&ex.cell))
                    .map(|(j, _)| j)
                    .collect();
                let arr: Vec<usize> = if !in_cell.is_empty() {
                    in_cell
                } else {
                    let rev: Vec<usize> = to_fd
                        .blocks
                        .iter()
                        .enumerate()
                        .filter(|(_, b2)| {
                            ex.kind == "stair"
                                && b2
                                    .exits
                                    .iter()
                                    .any(|e2| e2.kind == "stair" && e2.to == Some(f))
                        })
                        .map(|(j, _)| j)
                        .collect();
                    if !rev.is_empty() {
                        rev
                    } else {
                        to_fd
                            .blocks
                            .iter()
                            .enumerate()
                            .filter(|(_, b2)| near(b2))
                            .map(|(j, _)| j)
                            .collect()
                    }
                };
                if arr.is_empty() {
                    skipped_trans.push(format!(
                        "f{f}:{} --{}--> f{to_floor}: 无到场块（具名跳过）",
                        bd.id, ex.kind
                    ));
                } else {
                    for tj in arr {
                        trans[ni].push((node[&(to_floor, tj)], ex.kind.clone()));
                    }
                }
            }
        }
        OuterGraph { doc, floors: floors.to_vec(), node, floor_of, block_of, edges, trans, skipped_trans, masked_exits: masked_report }
    }

    pub fn start_node(&self, floor: i32, cell: (i32, i32)) -> Result<usize, OuterError> {
        let fd = self
            .doc
            .floors
            .get(&floor)
            .ok_or_else(|| OuterError::MissingField(format!("floors[{floor}] 不在块表")))?;
        fd.blocks
            .iter()
            .position(|b| b.cells.contains(&cell))
            .map(|bi| self.node[&(floor, bi)])
            .ok_or_else(|| OuterError::MissingField(format!("起点 {floor:?},{cell:?} 不在任何块")))
    }
}

// ---------- 搜索 ----------

/// 搜索预算（H6：不复利推导上界，让搜索自己停）。
#[derive(Debug, Clone, Copy)]
pub struct SearchBudget {
    pub max_pops: u64,
}

impl Default for SearchBudget {
    fn default() -> Self {
        SearchBudget { max_pops: 200_000 }
    }
}

/// 外层状态（维序只影响输出可读性；Ord 用于去重键）。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OuterState {
    pub node: usize,
    pub yellow: i32,
    pub red: i32,
    pub blue: i32,
    pub hp: i32,
    pub gold: i32,
    pub atk: i32,
    pub def: i32,
    /// 关键道具位图（item ty < 128）。
    pub held: u128,
    /// 已拾取块位图。
    pub picked: Vec<u8>,
    /// 已购买次数（磨属性显式行动计数；v1 上限 [`MAX_BUYS`]，见模块 doc 已知边界）。
    pub buys: u32,
}

/// v1 已知边界：单个状态链的磨属性购买次数上限。+3% 免费复利无限买会把
/// 状态空间撑爆（atk 饱和到 i32::MAX 才停）；256 次 ≈ atk 10→~3900，足够覆盖
/// 第 1 段全部战斗阈值，且让 Dijkstra 在锚附近收敛。放开=S3 的事。
pub const MAX_BUYS: u32 = 256;

/// 一条前沿记录（Pareto：hp/gold/keys/道具/atk/def vs 步数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontierEntry {
    pub steps: u64,
    /// 到锚时敏感表判定是否可杀（false = 到了但杀不动——诊断面）。
    pub killable: bool,
    /// F2 复核修正：killable 但撞面总损血 ≥ 当前 hp ⇒ 必死（fatal 诊断列）。
    /// killable && !fatal 才是「活着杀完」。
    pub fatal: bool,
    pub state: OuterState,
    /// 块路径 `f{floor}:{block_id}`（含锚）。
    pub path: Vec<String>,
    /// 乐观门注记（v1 口径证据）。
    pub gates: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct OuterResult {
    pub frontier: Vec<FrontierEntry>,
    pub pops: u64,
    pub truncated: bool,
    /// 具名跳过/未应用面（不静默）。
    pub unapplied_exports: u64,
    pub needs_sensitivity: Vec<i32>,
    pub skipped_trans: Vec<String>,
    /// S3 A 片：跨层回拾候选边计数（trans 展开目标层在 floors 序早于当前层）。
    pub reentry_edges: u64,
    /// S3 A 片：被注入屏蔽的出口边（具名回显，禁静默）。
    pub masked_exits: Vec<String>,
}

struct HeapItem {
    steps: u64,
    seq: u64,
    state: OuterState,
    path: Vec<String>,
    gates: Vec<String>,
}

impl Eq for HeapItem {}
impl PartialEq for HeapItem {
    fn eq(&self, o: &Self) -> bool {
        self.steps == o.steps && self.seq == o.seq
    }
}
impl PartialOrd for HeapItem {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for HeapItem {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        (Reverse(self.steps), self.seq).cmp(&(Reverse(o.steps), o.seq))
    }
}

fn bit_set(bits: &mut Vec<u8>, idx: usize) {
    bits[idx / 8] |= 1 << (idx % 8);
}
fn bit_get(bits: &[u8], idx: usize) -> bool {
    bits[idx / 8] & (1 << (idx % 8)) != 0
}

/// 外层 Dijkstra（步数=行动数）。拾取进块自动结算；磨属性=显式 1 步行动。
pub fn outer_search(
    graph: &OuterGraph,
    run: &RunSpec,
    scaled_enemies: &[game_data::defs::EnemyDef],
    budget: SearchBudget,
) -> OuterResult {
    let mut res = OuterResult {
        skipped_trans: graph.skipped_trans.clone(),
        masked_exits: graph.masked_exits.clone(),
        ..Default::default()
    };
    // S3 A 片：floors 序位置表（回拾判定 = 目标层序早于当前层）。
    let floor_pos: HashMap<i32, usize> =
        graph.floors.iter().enumerate().map(|(i, &f)| (f, i)).collect();
    let start_node = match graph.start_node(run.start.0, run.start.1) {
        Ok(n) => n,
        Err(e) => {
            res.skipped_trans.push(format!("起段失败: {e}"));
            return res;
        }
    };
    let anchor = match anchor_block(graph.doc, run.anchor_floor, run.anchor_boss_ty) {
        Ok(b) => graph.node[&(run.anchor_floor, b)],
        Err(e) => {
            res.skipped_trans.push(format!("锚缺失: {e}"));
            return res;
        }
    };
    let n = graph.floor_of.len();
    let start_state = OuterState {
        node: start_node,
        yellow: 0,
        red: 0,
        blue: 0,
        hp: 300,
        gold: 0,
        atk: 10,
        def: 10,
        held: 0,
        picked: vec![0u8; (n + 7) / 8],
        buys: 0,
    };
    let mut heap = BinaryHeap::new();
    heap.push(HeapItem { steps: 0, seq: 0, state: start_state.clone(), path: Vec::new(), gates: Vec::new() });
    let mut best: HashMap<OuterState, u64> = HashMap::new();
    // 同节点支配档案：picked 位图 → (buys, 全维数值档案)
    let mut seen: HashMap<usize, Vec<(Vec<u8>, u32, i32, i32, i32, i32, i32, i32, i32, i32, u128)>> =
        HashMap::new();
    let mut seq: u64 = 1;
    loop {
        let Some(HeapItem { steps, state, path, gates, .. }) = heap.pop() else {
            break;
        };
        res.pops += 1;
        if res.pops > budget.max_pops {
            res.truncated = true;
            break;
        }
        // 支配剪枝：同节点、同 picked、同 buys 档，其余全维不劣且步数不增 ⇒ 剪。
        // （不同 picked/held 的状态各自保留——支配对未拾取未来不完整。）
        if dominated_seen(&mut seen, &state) {
            continue;
        }
        // 进块拾取（未拾取过才结算；块可重入）
        let mut state = state;
        let mut path = path;
        let mut gates = gates;
        if !bit_get(&state.picked, state.node) {
            bit_set(&mut state.picked, state.node);
            let f = graph.floor_of[state.node];
            let bd = &graph.doc.floors[&f].blocks[graph.block_of[state.node]];
            for ex in &bd.exports {
                if ex.class != "const" {
                    continue;
                }
                match certified_monotone_pickup(ex.ty, f) {
                    Some(g) => {
                        state.hp += g.hp;
                        state.atk += g.atk;
                        state.def += g.def;
                        state.yellow += g.yellow;
                        state.blue += g.blue;
                        state.red += g.red;
                        state.gold += 0;
                        for &(ty, cnt) in &g.bag {
                            for _ in 0..cnt {
                                if (ty as u32) < 128 {
                                    state.held |= 1u128 << ty;
                                }
                            }
                        }
                    }
                    None => res.unapplied_exports += 1,
                }
            }
            path.push(format!("f{f}:{}", bd.id));
        }
        // 锚判定（先判后展——锚块也可穿越）
        if state.node == anchor {
            match anchor_killable(scaled_enemies, run.anchor_boss_ty, state.atk, state.def, state.hp, state.held) {
                Ok((k, fatal)) => res.frontier.push(FrontierEntry { steps, killable: k, fatal, state: state.clone(), path: path.clone(), gates: gates.clone() }),
                Err(OuterError::NeedsSensitivity(ty)) => {
                    if !res.needs_sensitivity.contains(&ty) {
                        res.needs_sensitivity.push(ty);
                    }
                }
                Err(_) => {}
            }
        }
        if best.get(&state).is_some_and(|&s| s <= steps) {
            continue;
        }
        best.insert(state.clone(), steps);
        // 邻块移动
        for e in &graph.edges[state.node] {
            if state.yellow < e.yellow || state.red < e.red || state.blue < e.blue {
                continue;
            }
            let mut ns = state.clone();
            ns.node = e.to;
            ns.yellow -= e.yellow;
            ns.red -= e.red;
            ns.blue -= e.blue;
            let mut ng = gates.clone();
            if e.optimistic_gates > 0 {
                ng.push(format!(
                    "optimistic_gate×{} f{}:{}→f{}:{}",
                    e.optimistic_gates,
                    graph.floor_of[state.node],
                    graph.doc.floors[&graph.floor_of[state.node]].blocks[graph.block_of[state.node]].id,
                    graph.floor_of[e.to],
                    graph.doc.floors[&graph.floor_of[e.to]].blocks[graph.block_of[e.to]].id
                ));
            }
            heap.push(HeapItem { steps: steps + 1, seq: seq, state: ns, path: path.clone(), gates: ng });
            seq += 1;
        }
        // 换层（S3 A 片：允许回头边——回拾候选边按「目标层序早于当前层」计数）
        for &(to, ref kind) in &graph.trans[state.node] {
            let mut ns = state.clone();
            ns.node = to;
            let from_floor = graph.floor_of[state.node];
            let to_floor = graph.floor_of[to];
            if floor_pos.get(&to_floor).copied().unwrap_or(usize::MAX) < floor_pos.get(&from_floor).copied().unwrap_or(usize::MAX) {
                res.reentry_edges += 1;
            }
            heap.push(HeapItem { steps: steps + 1, seq, state: ns, path: path.clone(), gates: gates.clone() });
            seq += 1;
            let _ = kind;
        }
        // 磨属性：id6 StatPct（price=0 免费可重复，成本=1 步——H6）
        let f = graph.floor_of[state.node];
        let bd = &graph.doc.floors[&f].blocks[graph.block_of[state.node]];
        for ex in &bd.exports {
            if ex.ty != 77 && ex.ty != 78 {
                continue;
            }
            for &(price, ref key, ref params) in &ex.goods {
                if key != "StatPct" {
                    continue;
                }
                if state.buys >= MAX_BUYS {
                    continue;
                }
                let pct = parse_pct(params);
                let mut ns = state.clone();
                ns.buys += 1;
                // 整数复利逐次模拟（提案 §3.5）；饱和防溢出（免费无限买 ⇒ 搜索靠帽停）。
                ns.atk = state.atk.saturating_add(state.atk.saturating_mul(pct.0) / 100);
                ns.def = state.def.saturating_add(state.def.saturating_mul(pct.1) / 100);
                heap.push(HeapItem { steps: steps + 1 + price as u64 / 1000_000, seq, state: ns, path: path.clone(), gates: gates.clone() });
                seq += 1;
            }
        }
    }
    res.frontier = pareto(std::mem::take(&mut res.frontier));
    res
}

/// 同节点支配判定与登记：picked 完全相同、buys 相同档位时，若档案中已有
/// 步数更少且数值全维不劣的条目则剪掉；否则登记。（held 超集维度保守地
/// 不比——见模块 doc 已知边界。）
/// 前置判据（复核 F6）：支配剪枝不含步数维，健全性依赖「所有转移成本 ≥1，
/// 堆按 steps 弹出 ⇒ 归档条目 steps 必 ≤ 当前」。加 0 成本边前必须先改这里。
/// （审计断言：编译期由 OM2 变异守护购买成本；邻块/换层 +1 为字面量。）
fn dominated_seen(
    seen: &mut HashMap<usize, Vec<(Vec<u8>, u32, i32, i32, i32, i32, i32, i32, i32, i32, u128)>>,
    state: &OuterState,
) -> bool {
    let entry = (
        state.picked.clone(),
        state.buys,
        state.yellow,
        state.red,
        state.blue,
        state.hp,
        state.gold,
        state.atk,
        state.def,
        0,
        state.held,
    );
    let arc = seen.entry(state.node).or_default();
    let dominated = arc.iter().any(|(_, b, y, r, bl, hp, g, a, d, _, h)| {
        *b <= state.buys
            && *y >= state.yellow
            && *r >= state.red
            && *bl >= state.blue
            && *hp >= state.hp
            && *g >= state.gold
            && *a >= state.atk
            && *d >= state.def
            && *h & !state.held == 0
    });
    if !dominated {
        arc.push(entry);
    }
    dominated
}

/// 解析 StatPct params JSON 里的 atk_pct/def_pct（缺省 0）。
fn parse_pct(params: &str) -> (i32, i32) {
    match game_data::asset_pipeline::parse_json(params) {
        Ok(v) => (
            v.get("atk_pct").and_then(|x| x.as_int()).unwrap_or(0) as i32,
            v.get("def_pct").and_then(|x| x.as_int()).unwrap_or(0) as i32,
        ),
        Err(_) => (0, 0),
    }
}

/// 锚可杀性：敏感表单一真相源判定（零复制）。
/// 持 23/24 由 held 位图读出（23=太公杖位、24=捆仙绳位——combat trait 语义）。
/// 锚可杀判定（敏感表单一真相源，零新公式）：
/// - `killable`：能杀死（`can_kill`：eff > edef）；
/// - `fatal`：能杀死但撞面总损血 `counter_loss ≥ 当前 hp` ⇒ 必死。
///   复核 F2：只判 can_kill 会把「打 100 回合被反击 5247>hp5200」误判可杀
///   （真数据 8/53 条致死）。killable && !fatal 才是存活击杀。
pub fn anchor_killable(
    scaled_enemies: &[game_data::defs::EnemyDef],
    boss_ty: i32,
    atk: i32,
    def: i32,
    hp: i32,
    held: u128,
) -> Result<(bool, bool), OuterError> {
    if !scaled_enemies.iter().any(|e| e.id == boss_ty) {
        return Err(OuterError::NeedsSensitivity(boss_ty));
    }
    let hold23 = held & (1u128 << 23) != 0;
    let hold24 = held & (1u128 << 24) != 0;
    let table: Vec<SensitivityRow> = sensitivity_table(scaled_enemies, atk, def, hold23, hold24);
    Ok(table
        .iter()
        .find(|r| r.ety == boss_ty)
        .map(|r| {
            let killable = r.can_kill && r.counter_loss >= 0;
            let fatal = killable && r.counter_loss >= hp;
            (killable, fatal)
        })
        .unwrap_or((false, false)))
}

/// Pareto 前沿：steps 少 / atk def hp gold 高 / keys 多者保留（held 不进支配比较，
/// 记为已知边界——道具维在 v1 段内语义单调，段间传递时由调用方保留全集）。
fn pareto(entries: Vec<FrontierEntry>) -> Vec<FrontierEntry> {
    let mut kept: Vec<FrontierEntry> = Vec::new();
    for e in entries {
        let dominated = kept.iter().any(|k| {
            k.steps <= e.steps
                && k.state.atk >= e.state.atk
                && k.state.def >= e.state.def
                && k.state.hp >= e.state.hp
                && k.state.gold >= e.state.gold
                && k.state.yellow >= e.state.yellow
                && k.state.red >= e.state.red
                && k.state.blue >= e.state.blue
                && (k.state.held & !e.state.held) == 0
        });
        if dominated {
            continue;
        }
        kept.retain(|k| {
            !(e.steps <= k.steps
                && e.state.atk >= k.state.atk
                && e.state.def >= k.state.def
                && e.state.hp >= k.state.hp
                && e.state.gold >= k.state.gold
                && e.state.yellow >= k.state.yellow
                && e.state.red >= k.state.red
                && e.state.blue >= k.state.blue
                && (e.state.held & !k.state.held) == 0)
        });
        kept.push(e);
    }
    kept.sort_by_key(|e| e.steps);
    kept
}

// ---------- f13 矛盾探测器（引擎实测，不猜） ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireRingProbe {
    pub ty10_cells: Vec<(i32, i32)>,
    pub stairs: Vec<(i32, i32)>,
    /// 两楼梯格是否可从起点 (1,11) 到达（火环=墙、其余实体不挡）。
    pub stairs_reachable: bool,
    /// 金箍棒格 (6,3) 是否在不穿越火环的前提下可达。
    pub cudgel_reachable_without_fire: bool,
}

/// f13 主径判定：地形 walkable + 实体 ty10（火环）=墙，其余实体不挡。
/// stairs=[(1,11),(11,11)]，cudgel=(6,3)——坐标是**当前题面常量**（maplv/sprite 实测）。
pub fn probe_fire_ring(
    cols: i32,
    rows: i32,
    terrain: &[bool],
    ents: &[(i32, i32, i32)],
    start: (i32, i32),
    stairs: &[(i32, i32)],
    cudgel: (i32, i32),
) -> Result<FireRingProbe, String> {
    let walk = |x: i32, y: i32| -> bool {
        x >= 0 && y >= 0 && x < cols && y < rows && terrain[(y * cols + x) as usize]
    };
    let ty10: Vec<(i32, i32)> = ents
        .iter()
        .filter(|&&(ty, _, _)| ty == 10)
        .map(|&(_, x, y)| (x, y))
        .collect();
    let blocked: HashSet<(i32, i32)> = ty10.iter().copied().collect();
    // 楼梯格是实体通道：引擎口径 `walkable || stair_like`（solve_local.rs:2117），
    // 地形本判不可走也放行（真数据 f13 起点 (1,11)=ty8 正是这种格）。
    let stair_cells: HashSet<(i32, i32)> = stairs.iter().copied().collect();
    let enter = |x: i32, y: i32| -> bool { walk(x, y) || stair_cells.contains(&(x, y)) };
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut q = std::collections::VecDeque::new();
    if !enter(start.0, start.1) {
        return Err(format!("起点 {start:?} 地形不可走且非楼梯"));
    }
    q.push_back(start);
    seen.insert(start);
    while let Some((x, y)) = q.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x + dx;
            let ny = y + dy;
            if enter(nx, ny) && !blocked.contains(&(nx, ny)) && seen.insert((nx, ny)) {
                q.push_back((nx, ny));
            }
        }
    }
    Ok(FireRingProbe {
        stairs_reachable: stairs.iter().all(|s| seen.contains(s)),
        cudgel_reachable_without_fire: seen.contains(&cudgel),
        ty10_cells: ty10,
        stairs: stairs.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_data::defs::EnemyDef;

    fn enemy(id: i32, hp: i32, atk: i32, def: i32, flags: u8) -> EnemyDef {
        EnemyDef { id, hp, atk, def, gold: 0, trait_flags: flags }
    }

    fn enemies() -> Vec<EnemyDef> {
        use game_core::combat::trait_flags_for;
        vec![enemy(41, 35, 18, 1, trait_flags_for(41)), enemy(75, 100, 65, 15, trait_flags_for(75))]
    }

    /// 最小 v2 块表夹具：fA(51) 两块 door1 相连 + 拾取黄钥匙；fB(10) 两块 door1 相连，
    /// b1 是锚（boss75）。梯：fA b0 stair→10，f10 b0 stair→51（到场块）。
    fn mini_doc() -> String {
        r#"{
  "schema": "xiyou/block-interfaces/v2",
  "floors": [
    { "floor": 51, "entry": [1,11], "blocks": [
      { "id": "a0", "cells": [[1,11],[1,10],[2,10]],
        "cut_doors": [],
        "exports": [ { "ty": 26, "cell": [1,10], "class": "const", "detail": "黄钥匙" } ],
        "exits": [ { "kind": "stair", "ty": 7, "cell": [2,10], "to": 10, "same_floor": false } ] },
      { "id": "a1", "cells": [[4,10],[5,10]],
        "cut_doors": [ { "cell": [3,10], "kind": "door1" } ],
        "exports": [],
        "exits": [] }
    ] },
    { "floor": 10, "entry": [2,10], "blocks": [
      { "id": "b0", "cells": [[2,10],[2,9],[3,9]],
        "cut_doors": [],
        "exports": [ { "ty": 26, "cell": [2,9], "class": "const", "detail": "黄钥匙" } ],
        "exits": [ { "kind": "stair", "ty": 8, "cell": [2,10], "to": 51, "same_floor": false } ] },
      { "id": "b1", "cells": [[5,9],[6,9]],
        "cut_doors": [ { "cell": [4,9], "kind": "door1" } ],
        "exports": [],
        "exits": [ { "kind": "boss_kill", "ty": 75, "cell": [6,9], "events": [], "post_victory": false, "drops": {} } ] }
    ] }
  ]
}"#.to_string()
    }

    /// 判据（复核 F2 红项）：killable 必须含损血约束——counter_loss < hp 才算
    /// 存活击杀，否则 fatal=true（诊断列）。变异判据：删 counter_loss 约束 ⇒ 此
    /// 测试的 fatal 断言红（atk16/def12/hp5200 对 boss75 损血 5247 必死）。
    #[test]
    fn anchor_killable_requires_survivable_counter_loss() {
        let en = enemies();
        // 旗舰误判样本（复核实锤）：atk16/def12/hp5200 ⇒ can_kill 但损血 (65-12)*99=5247 ≥ hp
        let (k, f) = anchor_killable(&en, 75, 16, 12, 5200, 0).unwrap();
        assert!(k, "atk16>def15 能杀死");
        assert!(f, "损血 5247 ≥ hp5200 ⇒ fatal（复核 F2 的 8 条致死路径之一）");
        // def30 ⇒ 损血 (65-30)*99=3465 < 5200 ⇒ 存活击杀
        let (k2, f2) = anchor_killable(&en, 75, 16, 30, 5200, 0).unwrap();
        assert!(k2 && !f2, "def30 损血 3465<5200 应存活");
        // 打不动：atk10 ≤ def15 ⇒ 不 killable 不 fatal
        let (k3, f3) = anchor_killable(&en, 75, 10, 12, 5200, 0).unwrap();
        assert!(!k3 && !f3);
    }

    /// 判据（加载契约）：schema 不符 ⇒ 具名 SchemaMismatch，禁回落。
    #[test]
    fn load_rejects_wrong_schema_named() {
        let raw = mini_doc().replace("xiyou/block-interfaces/v2", "xiyou/block-interfaces/v1");
        let err = load_block_interfaces(&raw).unwrap_err();
        assert_eq!(err, OuterError::SchemaMismatch { got: "xiyou/block-interfaces/v1".into() });
        let err2 = load_block_interfaces("{ not json").unwrap_err();
        assert!(matches!(err2, OuterError::BadJson(_)));
    }

    /// 判据（门钥匙语义）：door1 无黄钥匙不可通行；拾取 1 黄后可通行；
    /// 密封/脚本门乐观通行并记注记。
    #[test]
    fn door1_requires_yellow_key_and_pickup_gates_it() {
        let doc = load_block_interfaces(&mini_doc()).unwrap();
        let floors = vec![51, 10];
        let g = OuterGraph::build(&doc, &floors, &[]);
        let start = g.start_node(51, (1, 11)).unwrap();
        let enemies = enemies();
        let run = RunSpec { floors: floors.clone(), start: (51, (1, 11)), anchor_floor: 10, anchor_boss_ty: 75, masked_exits: Vec::new() };
        let res = outer_search(&g, &run, &enemies, SearchBudget { max_pops: 5000 });
        assert!(!res.truncated);
        assert!(!res.frontier.is_empty(), "锚应可达（拾 1 黄开门 + 敏感表判 75 可杀）");
        let best = &res.frontier[0];
        // 路径：a0(拾黄) → stair → b0(拾黄) → door1 → b1(锚)
        // 两枚黄钥匙拾取、一扇 door1：任何前沿条目最多剩 1（门确实扣了钥匙；
        // 用全前沿而非 [0]——步数并列时 [0] 不确定）。
        assert!(
            res.frontier.iter().all(|e| e.state.yellow <= 1),
            "黄钥匙应被门消耗"
        );
        assert!(best.path.iter().any(|s| s.contains("b1")), "path={:?}", best.path);
        assert!(best.state.atk >= 10);
        assert!(!best.killable, "初始 atk10 ≤ 75 防拾5 ⇒ 到锚但杀不动（诊断面）");
    }

    /// 判据（needs_sensitivity）：锚怪不在缩放敌表 ⇒ 具名 NeedsSensitivity。
    #[test]
    fn missing_boss_reports_needs_sensitivity() {
        let doc = load_block_interfaces(&mini_doc()).unwrap();
        let floors = vec![51, 10];
        let g = OuterGraph::build(&doc, &floors, &[]);
        let run = RunSpec { floors, start: (51, (1, 11)), anchor_floor: 10, anchor_boss_ty: 75, masked_exits: Vec::new() };
        // 敌表只给 41（75 缺席）——能力缺口必须具名，不许回落成「不可杀」。
        let res = outer_search(&g, &run, &[enemy(41, 35, 18, 1, game_core::combat::trait_flags_for(41))], SearchBudget { max_pops: 5000 });
        assert_eq!(res.needs_sensitivity, vec![75]);
        assert!(res.frontier.is_empty());
    }

    /// 判据（磨属性 H6）：StatPct 每次购买 = +1 步、整数复利逐次模拟；
    /// 「不复利推导上界、让搜索自己停」——状态空间在 i32 饱和处自然封顶，
    /// 搜索在有界 pops 内自行终止（无需人工上界），且出现可杀锚点。
    #[test]
    fn stat_pct_purchase_costs_one_step_and_saturates_budget() {
        // 给 b0 挂一个含 StatPct 的商店 export（直接改 JSON：exports 挂到锚块）
        // 给 a1（门后块）挂一个含 StatPct 的商店 export：免费可重复 ⇒ 复利。
        let doc_raw = mini_doc().replace(
            "\"exports\": [],\n        \"exits\": [] }",
            "\"exports\": [ { \"ty\": 26, \"cell\": [4,10], \"class\": \"const\", \"detail\": \"黄钥匙\" }, { \"ty\": 77, \"cell\": [5,10], \"class\": \"timepoint\", \"param\": 21, \"goods\": [[0, \"StatPct\", \"{\\\"atk_pct\\\": 50, \\\"def_pct\\\": 0}\"]] } ],\n        \"exits\": [] }",
        );
        let doc = load_block_interfaces(&doc_raw).unwrap();
        let f10 = doc.floors.get(&51).unwrap();
        let floors = vec![51, 10];
        let g = OuterGraph::build(&doc, &floors, &[]);
        let run = RunSpec { floors, start: (51, (1, 11)), anchor_floor: 10, anchor_boss_ty: 75, masked_exits: Vec::new() };
        let res = outer_search(&g, &run, &enemies(), SearchBudget { max_pops: 3000 });
        assert!(!res.truncated, "搜索应自行停（复利饱和 ⇒ 状态空间封顶）");
        assert!(res.pops < 3000);
        assert!(!res.frontier.is_empty());
        assert!(res.frontier.iter().any(|e| e.state.atk > 10), "复利把 atk 推高过初值");
        assert!(res.frontier.iter().any(|e| e.killable), "磨属性后 75 应可杀");
        // 判据（H6 计步成本）：可杀至少需 2 次购买（10→15→22>def15），
        // 基础路 a0→a1→a0→b0→b1 = 4 步 + 2 买步 ⇒ 最短可杀 ≥ 6。
        let min_killable = res.frontier.iter().filter(|e| e.killable).map(|e| e.steps).min().unwrap();
        assert!(min_killable >= 6, "购买必须计 1 步（H6）：min_killable={min_killable}");
    }

    // ---------- S3 A 片：跨层回拾（问题空间 = 全部可达块图） ----------

    /// 三层夹具：f51 →(stair)→ f10 →(stair)→ f12。a0 拾黄钥；a1 是 f51 门后块
    /// （补拾 surplus 黄钥）；b1 门1 后是 f10 的过路块；c0（f12）是锚 boss75。
    /// f10 有反向梯（回 f51）与前向梯（去 f12）——mask 前向即「f4 出口链断开」。
    fn s3_chain_doc() -> String {
        r#"{
  "schema": "xiyou/block-interfaces/v2",
  "floors": [
    { "floor": 51, "entry": [1,11], "blocks": [
      { "id": "a0", "cells": [[1,11],[1,10],[2,10]],
        "cut_doors": [],
        "exports": [ { "ty": 26, "cell": [1,10], "class": "const", "detail": "黄钥匙" } ],
        "exits": [ { "kind": "stair", "ty": 7, "cell": [2,10], "to": 10, "same_floor": false } ] },
      { "id": "a1", "cells": [[4,10],[5,10],[6,10]],
        "cut_doors": [ { "cell": [3,10], "kind": "door1" } ],
        "exports": [ { "ty": 26, "cell": [4,10], "class": "const", "detail": "黄钥匙2" },
                     { "ty": 26, "cell": [5,10], "class": "const", "detail": "黄钥匙3" },
                     { "ty": 26, "cell": [6,10], "class": "const", "detail": "黄钥匙4" } ],
        "exits": [] }
    ] },
    { "floor": 10, "entry": [2,10], "blocks": [
      { "id": "b0", "cells": [[2,10],[2,9],[3,9]],
        "cut_doors": [],
        "exports": [],
        "exits": [ { "kind": "stair", "ty": 8, "cell": [2,10], "to": 51, "same_floor": false } ] },
      { "id": "b1", "cells": [[5,9],[6,9]],
        "cut_doors": [ { "cell": [4,9], "kind": "door1" } ],
        "exports": [],
        "exits": [ { "kind": "stair", "ty": 7, "cell": [6,9], "to": 12, "same_floor": false } ] }
    ] },
    { "floor": 12, "entry": [1,9], "blocks": [
      { "id": "c0", "cells": [[1,9]],
        "cut_doors": [],
        "exports": [],
        "exits": [ { "kind": "stair", "ty": 8, "cell": [1,9], "to": 10, "same_floor": false },
                   { "kind": "boss_kill", "ty": 75, "cell": [1,9], "events": [], "post_victory": false, "drops": {} } ] }
    ] }
  ]
}"#.to_string()
    }

    fn s3_run(floors: Vec<i32>) -> RunSpec {
        RunSpec {
            floors,
            start: (51, (1, 11)),
            anchor_floor: 12,
            anchor_boss_ty: 75,
            masked_exits: Vec::new(),
        }
    }

    /// S3 核心验收：出口链断开（mask f10 前向梯）后，外层不报错不终止——
    /// 搜索继续推进，前沿/堆出现回早前层块（f51）的候选边（reentry_edges>0），
    /// 且注入的断链被具名回显（禁静默）。
    #[test]
    fn s3_reentry_when_chain_breaks() {
        let doc = load_block_interfaces(&s3_chain_doc()).unwrap();
        let floors = vec![51, 10, 12];
        let mut run = s3_run(floors.clone());
        run.anchor_floor = 12;
        run.masked_exits = vec![(10, (6, 9))];
        let g = OuterGraph::build(&doc, &floors, &run.masked_exits);
        let res = outer_search(&g, &run, &enemies(), SearchBudget { max_pops: 20_000 });
        assert!(!res.truncated, "断链后搜索必须自行收敛，不许截断/报错");
        assert!(res.frontier.is_empty(), "锚在 f12，前向梯已断 ⇒ 不可达");
        assert!(res.reentry_edges >= 1, "必须出现回早前层块的候选边：reentry_edges={}", res.reentry_edges);
        assert!(res.pops > 5, "断链后搜索必须继续推进：pops={}", res.pops);
        assert_eq!(res.masked_exits, vec!["f10:(6,9)".to_string()], "注入断链必须具名回显");
        // 对照：不 mask 时锚可达（f12 c0），证明断链是 mask 注入的效应。
        let run2 = { let mut r = s3_run(floors.clone()); r.anchor_floor = 12; r };
        let g2 = OuterGraph::build(&doc, &floors, &run2.masked_exits);
        let res2 = outer_search(&g2, &run2, &enemies(), SearchBudget { max_pops: 20_000 });
        assert!(!res2.frontier.is_empty(), "不 mask 时锚应可达");
        assert!(res2.frontier.iter().any(|e| e.path.iter().any(|p| p.contains("c0"))));
    }

    /// S3 资源多层状态：f51 拾的黄钥在 f10 的 door1 扣减生效（跨层携带）；
    /// 回拾 surplus（f51 门后 a1 的第二枚）跨层带入 f10 且回拾路径定价精确
    /// （零奖励零惩罚：a0→a1→a0→b0→b1→c0 = 5 步、yellow==1）。
    #[test]
    fn s3_keys_carry_across_floors() {
        let doc = load_block_interfaces(&s3_chain_doc()).unwrap();
        let floors = vec![51, 10, 12];
        let run = s3_run(floors.clone());
        let g = OuterGraph::build(&doc, &floors, &run.masked_exits);
        let res = outer_search(&g, &run, &enemies(), SearchBudget { max_pops: 20_000 });
        assert!(!res.truncated);
        assert!(!res.frontier.is_empty());
        // 最短：a0(拾黄) → b0 → b1(door1 扣跨层黄钥) → c0 = 3 步，黄钥剩 0。
        let min_steps = res.frontier.iter().map(|e| e.steps).min().unwrap();
        assert_eq!(min_steps, 3, "最短路径应为 3 步（跨层钥匙直接开门）");
        assert!(
            res.frontier.iter().any(|e| e.steps == 3 && e.state.yellow == 0),
            "跨层拾取的黄钥应被 f10 的门消耗"
        );
        // 回拾 surplus：a0→a1(+1)→a0(+1)→b0(+1)→b1(+1)→c0(+1) = 5 步、yellow==1。
        for ni in 0..g.floor_of.len() {
            eprintln!("DBG node f{}:{} edges={:?} trans={:?}",
                g.floor_of[ni], g.doc.floors[&g.floor_of[ni]].blocks[g.block_of[ni]].id,
                g.edges[ni].iter().map(|e| (g.floor_of[e.to], g.doc.floors[&g.floor_of[e.to]].blocks[g.block_of[e.to]].id.clone(), e.yellow)).collect::<Vec<_>>(),
                g.trans[ni].iter().map(|t| (g.floor_of[t.0], g.doc.floors[&g.floor_of[t.0]].blocks[g.block_of[t.0]].id.clone())).collect::<Vec<_>>());
        }
        assert!(
            res.frontier.iter().any(|e| e.steps == 5 && e.state.yellow == 1),
            "回早前层补拾的 surplus 键必须可跨层携带且步数精确定价（5 步）"
        );
        assert!(
            res.frontier.iter().all(|e| e.steps >= 3),
            "回拾不得有隐藏奖励/捷径：最短仍为 3 步"
        );
    }

    /// S3 防硬编码收割：outer.rs 源面不得出现回拾特判分支（收割/奖励字面量）；
    /// 回拾收益只能从经济链边权涌现（B 片），A 片只放行回头边。
    #[test]
    fn s3_no_hardcoded_harvest() {
        let src = include_str!("outer.rs");
        for (a, b) in [("reentry", "gain"), ("harvest", "_"), ("reentry", "bonus")] {
            let bad = format!("{a}{b}");
            assert!(!src.contains(&bad), "禁止回拾硬编码分支: {bad}");
        }
    }

    /// S3 问题空间：build_all 覆盖块表全部层与块（55 层 / 383 块 / 换层边 ≥105）。
    #[test]
    fn s3_build_all_covers_full_block_graph() {
        let raw = include_str!("../../../docs/findings/block-interfaces.json");
        let doc = load_block_interfaces(raw).unwrap();
        assert_eq!(doc.floors.len(), 55);
        let g = OuterGraph::build_all(&doc);
        assert_eq!(g.floor_of.len(), 383, "全块图节点数");
        assert_eq!(g.floors.len(), 55);
        let trans: usize = g.trans.iter().map(|t| t.len()).sum();
        assert!(trans >= 105, "真数据楼梯换层边至少 105 条：trans={trans}");
        // 段内 build 的节点数 = 该段层数的块数（S2 回归口径不变）。
        let g1 = OuterGraph::build(&doc, &[51, 52, 53, 54, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], &[]);
        assert!(
            g1.floor_of.len() < g.floor_of.len(),
            "段内图应是全块图的子集"
        );
    }

    /// 判据（段计划）：第 1 段 Run，后续段具名 Off（禁静默）。
    #[test]
    fn plan_segments_names_every_off() {
        let doc = load_block_interfaces(&mini_doc()).unwrap();
        let plans = plan_segments(&doc);
        assert!(matches!(plans[0].1, SegmentPlan::Run(_)));
        for (_, p) in plans.iter().skip(1) {
            match p {
                SegmentPlan::Off { reason, .. } => assert!(!reason.is_empty()),
                SegmentPlan::Run(_) => panic!("v1 不应有第二个 Run 段"),
            }
        }
        assert!(plans.iter().any(|(_, p)| matches!(p, SegmentPlan::Off { reason, .. } if reason.contains("branch_floor_no_block_graph(25)"))));
    }

    /// f13 探测器（引擎口径，合成夹具）：火环列把 (6,3) 与主径隔断、楼梯可达
    /// ⇒ stairs_reachable=true、cudgel_reachable_without_fire=false。
    #[test]
    fn fire_ring_probe_synthetic() {
        let cols = 12i32;
        let rows = 13i32;
        let mut terrain = vec![true; (cols * rows) as usize];
        // (6,3) 只经 (6,4..6,6) 火环列进入：把 (5,3) 与 (7,3) 设为不可走
        terrain[(3 * cols + 5) as usize] = false;
        terrain[(3 * cols + 7) as usize] = false;
        terrain[(2 * cols + 6) as usize] = false; // (6,2) 侧封死：(6,3) 只能经火环列进入
        let ents = vec![(10, 6, 4), (10, 6, 5), (10, 6, 6)];
        let p = probe_fire_ring(cols, rows, &terrain, &ents, (1, 11), &[(1, 11), (11, 11)], (6, 3)).unwrap();
        assert_eq!(p.ty10_cells, vec![(6, 4), (6, 5), (6, 6)]);
        assert!(p.stairs_reachable);
        assert!(!p.cudgel_reachable_without_fire, "火环隔断金箍棒格 ⇒ 主径无矛盾");
    }
}
