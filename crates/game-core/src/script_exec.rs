//! 【2026-11-17 S3·④】脚本**执行流程**单一来源。
//!
//! 到 S3·③ 为止，统一的只有三层：解析（`parse_op`，S3·①）、宿主面（`WorldHost` /
//! `PresentationHost`，S3·③）、脚本域写（`ScriptSlots::apply`，S3·②）。剩下的
//! **流程**仍是两份手抄：
//!
//! | 流程步骤 | 生产（`GameScene`） | headless（`ScriptEngine`） |
//! |---|---|---|
//! | 入口门 `cursor+3 > len` | `scene_script.rs` | `script.rs` |
//! | 解析 | `Self::parse_op` | `GameScene::parse_op` |
//! | 脚本域写 | `apply_script_domain` | `slots.apply` |
//! | 尾递归（GIN） | 函数尾 `if Recurse` | 函数尾 `if Recurse` |
//!
//! 四步两份 = 四类漂移机会。本模块把这四步收成 [`run_instruction`] 一份，
//! 两个执行端只保留**世界动作**（`exec_op`）与**行尾收尾**（`on_line_end`）两个钩子——
//! 那部分本就不可能相同：一个是真场景，一个是状态模型。
//!
//! 判据：两端的流程分支由本模块唯一实现；两端只实现钩子。
use crate::subsystem::script_host::{ScriptSlots, ScriptStep};
use crate::subsystem::script_host::ScriptOp;

/// 执行端：提供脚本域状态槽 + 自己的世界动作。
pub trait ScriptRunner {
    /// 脚本域六槽（游标/旗标/delay/行/索引/行走武装）。
    fn slots(&mut self) -> &mut ScriptSlots;
    /// CES 类目查表的楼层（值拷贝，避免与 slots 借位冲突）。
    fn floor(&self) -> i32;
    /// 当前脚本行（行尾可被 override 替换）。
    fn current_line(&self) -> String;
    /// 行尾收尾（入口门命中时）：phase=4 + 置 flag + 相机回玩家。
    fn on_line_end(&mut self, cur: usize);
    /// 一条命令的世界动作（不含脚本域写、不含入口门）。
    fn exec_op(&mut self, op: ScriptOp);
}

/// 唯一执行流程：入口门 → 解析 → 脚本域写 → 世界动作 → 尾递归。
///
/// 逐拍语义与 Java `executeScriptInstruction`（a.java:8239-8581）一致：
/// 入口门只由「本拍开始时的游标」判定；行尾检查（javap 2397/2400/2431）
/// 零字段写，收尾写只发生在**下一拍**的入口门。
pub fn run_instruction<R: ScriptRunner>(r: &mut R) {
    let line = r.current_line();
    let cursor0 = r.slots().script_cursor;
    if cursor0 + 3 > line.len() {
        let cur = r.slots().current_script_index;
        r.on_line_end(cur);
        return;
    }
    // 未知 op ⇒ 静默停摆：无臂写游标 ⇒ 下一拍重解析同一 token（a.java:8594-8595）。
    let Some(op) = crate::scene::GameScene::parse_op(&line, cursor0) else {
        return;
    };
    let floor = r.floor();
    let step = r.slots().apply(op, floor);
    r.exec_op(op);
    if matches!(step, ScriptStep::Recurse) {
        run_instruction(r); // GIN 同拍尾递归（a.java:8435-8436）
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 假执行端：只记录流程走到哪一步，用于钉住「流程只有一份」。
    #[derive(Default)]
    struct Rec {
        slots: ScriptSlots,
        line: String,
        ended: Option<usize>,
        ops: Vec<&'static str>,
    }

    impl ScriptRunner for Rec {
        fn slots(&mut self) -> &mut ScriptSlots { &mut self.slots }
        fn floor(&self) -> i32 { 1 }
        fn current_line(&self) -> String { self.line.clone() }
        fn on_line_end(&mut self, cur: usize) { self.ended = Some(cur); }
        fn exec_op(&mut self, op: ScriptOp) {
            self.ops.push(match op {
                ScriptOp::Gin { .. } => "GIN",
                ScriptOp::Tak { .. } => "TAK",
                ScriptOp::Sms { .. } => "SMS",
                _ => "OTHER",
            });
        }
    }

    fn rec(line: &str, cursor: usize) -> Rec {
        let mut r = Rec::default();
        r.line = line.to_string();
        r.slots.script_line_flags = vec![false; 68];
        r.slots.script_cursor = cursor;
        r
    }

    /// 入口门：cursor+3 > len ⇒ 收尾写一次、不动游标、不执行命令。
    #[test]
    fn s3d4_entry_gate_ends_line_without_exec() {
        let mut r = rec("TAK_1_2 ", 10);
        r.slots.current_script_index = 7;
        run_instruction(&mut r);
        assert_eq!(r.ended, Some(7), "入口门必须调 on_line_end");
        assert!(r.ops.is_empty(), "入口门命中不得执行命令");
        assert_eq!(r.slots.script_cursor, 10, "入口门不动游标");
    }

    /// 正常指令：走解析 → 脚本域写（TAK 后 cursor 前移）→ 世界动作。
    #[test]
    fn s3d4_runs_op_and_advances_cursor() {
        let mut r = rec("TAK_1_2 ", 0);
        run_instruction(&mut r);
        assert_eq!(r.ops, vec!["TAK"]);
        assert_eq!(r.slots.script_cursor, 8, "TAK 承载 6 字符 + 分隔符 1 + 末位 1");
        assert_eq!(r.ended, None);
    }

    /// 尾递归：GIN 携带 Recurse ⇒ 同拍继续执行下一条（由流程统一判定，不由端自判）。
    #[test]
    fn s3d4_gin_recurse_same_tick() {
        let mut r = rec("GIN_1_0 SMS_0 ", 0);
        run_instruction(&mut r);
        assert_eq!(r.ops, vec!["GIN", "SMS"], "GIN 必须同拍尾递归到下一条");
        assert_eq!(r.slots.script_cursor, 14, "两条命令连续消费");
    }

    /// 未知 op：静默停摆（游标不动）——下一拍重解析同一 token。
    #[test]
    fn s3d4_unknown_op_stalls() {
        let mut r = rec("ZZZ_1_1 ", 0);
        run_instruction(&mut r);
        assert!(r.ops.is_empty());
        assert_eq!(r.slots.script_cursor, 0, "未知 op 不得推进游标");
    }
}
