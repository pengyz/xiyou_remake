//! 批4 · 三查询跑轮（plan §4 批4 的 `query.rs`）。
//!
//! 三查询语义写死在 plan §1.2：
//! - **Q1 剩余血量最大** = `ClosurePolicy::q1()`（拾取+零伤怪全收）下 `max(hp)`
//! - **Q2 最少杀怪** = `ClosurePolicy::q2()`（**绝不自动收零伤怪**，只收挡路必杀）
//!   下 `min(kills)`
//! - **Q3 不拿终极装备能否通关** = 禁令集（缺省 `Q3_EQUIP_SET`，可被 `--ultimate`
//!   覆盖）下 `reachable = !goals.is_empty()`
//!
//! 三者都是**有界求解**：先说 `GlobalResult::status`
//! （`witness|exhausted|capped|model_broken`），再看 `coverage` 是否完整；
//! `confidence` 只是它的派生展示值（P1 起不再有 `feasible` 这种含混词）。
//! （`optimal | bounded-best | exhausted-no-solution | capped-search`）；plan §4 批4 要求 findings 结论
//! 先标「待批5 oracle 回放确认」。
//!
//! 纯度：本文件属 `lib`，**禁 `std::fs`/`std::net`/`std::process`/`std::time`**
//! （A9 闸 `gates/cli.py` grep lib）。落盘/读 assets 全在 `bin/solver.rs`。
//! JSON 手拼（与 `GlobalResult::to_json` 同风格，不引 serde）。

use game_data::defs::ShopGoodDef;
use std::collections::BTreeMap;

use crate::solve_global::{node_cap_env, solve_global_with, GlobalInput, GlobalResult};
use crate::solve_local::{ClosurePolicy, KillMode};
use crate::state::Q3_EQUIP_SET;

/// 一轮查询的可调参数（CLI 侧透传；缺省 = plan §1.2 口径）。
#[derive(Debug, Clone)]
pub struct QueryOpts {
    /// `solve_global` 的 bound（总扩展数）。
    pub bound: u64,
    /// 全局 node cap（缺省 = `SOLVER_NODE_LIMIT` 缺省）。
    pub node_cap: u64,
    /// Q3 禁令集（缺省 `Q3_EQUIP_SET = [36,40]`，A3 实文 `--ultimate 36,40`）。
    pub ultimate: Vec<i32>,
}

impl Default for QueryOpts {
    fn default() -> Self {
        QueryOpts {
            bound: 1_000_000,
            node_cap: node_cap_env(),
            ultimate: Q3_EQUIP_SET.to_vec(),
        }
    }
}

/// 判据值（三查询各自形状不同，故枚举而非裸数字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryValue {
    /// `max_hp` / `min_kills`：有界内最优值；`0` + `status=capped` 表示搜索被截断、
    /// **不知道**有没有解；`0` + `status=exhausted` 才是「模型内真无解」。
    Best(u64),
    /// `reachable`：`!goals.is_empty()`
    Reachable(bool),
}

/// 单条查询结果 = 全局求解结果 + 该查询的判据值。
#[derive(Debug)]
pub struct QueryResult {
    /// `q1` | `q2` | `q3`
    pub key: &'static str,
    /// 判据名：`max_hp` / `min_kills` / `reachable`
    pub metric: &'static str,
    pub value: QueryValue,
    pub confidence: String,
    pub result: GlobalResult,
}

impl QueryResult {
    /// plan §4 批4 的控制台行：
    /// `Q1 max_hp=<N>(<置信级>)` / `Q2 min_kills=<M>(<置信级>)` /
    /// `Q3 reachable=<true|false>`
    pub fn console_line(&self) -> String {
        let tag = self.key.to_uppercase();
        match self.value {
            QueryValue::Best(v) => {
                format!("{tag} {}={v}({})", self.metric, self.confidence)
            }
            QueryValue::Reachable(v) => format!("{tag} {}={v}", self.metric),
        }
    }

    /// 落盘用 JSON 值：`{metric, value, confidence, global:{…GlobalResult…}}`。
    pub fn to_json(&self) -> String {
        let v = match self.value {
            QueryValue::Best(v) => v.to_string(),
            QueryValue::Reachable(v) => v.to_string(),
        };
        format!(
            "{{\"metric\":\"{}\",\"value\":{},\"confidence\":\"{}\",\"global\":{}}}",
            self.metric,
            v,
            self.confidence,
            self.result.to_json()
        )
    }
}

/// 跑一条查询。`q` = `1`/`2`/`3`。
pub fn run_query(
    input: &GlobalInput,
    q: u8,
    opts: &QueryOpts,
    shops: &[ShopGoodDef],
) -> QueryResult {
    // Q3 的禁令集可能来自 CLI 现 parse 的 Vec ⇒ `Box::leak` 换 `'static`
    //（`ClosurePolicy.ban` 是 `&'static [i32]`，memo 键要求 Copy；进程级一次性，
    //  lib 无 fs/net/process/time，A9 纯度闸不受影响）。
    let ban: &'static [i32] = if q == 3 {
        if opts.ultimate == Q3_EQUIP_SET {
            Q3_EQUIP_SET
        } else {
            let v = opts.ultimate.clone().into_boxed_slice();
            Box::leak(v)
        }
    } else {
        &[]
    };
    let policy = match q {
        1 => ClosurePolicy::q1(),
        2 => ClosurePolicy::q2(),
        _ => ClosurePolicy { pickup: true, kill_zero: KillMode::All, ban },
    };
    let mut input = input.clone();
    input.bound = opts.bound;
    let result = solve_global_with(&input, opts.node_cap, policy, shops);
    let confidence = result.confidence.to_string();
    let (key, metric, value) = match q {
        1 => (
            "q1",
            "max_hp",
            QueryValue::Best(result.goals.iter().map(|g| g.hp as u64).max().unwrap_or(0)),
        ),
        2 => (
            "q2",
            "min_kills",
            QueryValue::Best(result.goals.iter().map(|g| g.kills as u64).min().unwrap_or(0)),
        ),
        _ => ("q3", "reachable", QueryValue::Reachable(!result.goals.is_empty())),
    };
    QueryResult { key, metric, value, confidence, result }
}

/// `query --all`：三查询一轮，返回 `{q1,q2,q3}`（plan §4 批4 的 python assert
/// `{'q1','q2','q3'} <= set(d)` 就打在这张表上）。
pub fn run_all(
    input: &GlobalInput,
    opts: &QueryOpts,
    shops: &[ShopGoodDef],
) -> BTreeMap<String, QueryResult> {
    let mut m = BTreeMap::new();
    for q in [1u8, 2, 3] {
        let r = run_query(input, q, opts, shops);
        m.insert(r.key.to_string(), r);
    }
    m
}

/// 落盘 JSON（`--out`）：`{q1:{…}, q2:{…}, q3:{…}}`。
pub fn to_doc(results: &BTreeMap<String, QueryResult>) -> String {
    let body: Vec<String> = results.iter().map(|(k, r)| format!("\"{k}\":{}", r.to_json())).collect();
    format!("{{{}}}", body.join(","))
}
