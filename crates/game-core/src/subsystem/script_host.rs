//! script_host face（P2 批5 终形）——ScriptHost：脚本 op 执行面（spec §3.1 `execute_op(ctx,op)->ScriptStep`）。
//!
//! 【切面·script_ops 既有形】plan 批5「execute_op(ctx,op) trait 化（script_ops 既有形+HostCtx 余方法处置账）」：
//! resolver=**脚本域写面收拢**（普适 cursor 写+Gut 旗标+Ces scriptDelay+GIN 尾递归信号化）；
//! op_* 编排体留 Coordinator 面（跨域调用=遗留直写批10 清零面——本批不迁，避免 21 helper 借用面重构风险）。
//!
//! 【HostCtx 余方法处置账】（plan 批5 明文交付；**批6 摘 2 后 HostCtx 现 8 方法**[give_gold/pickup_item 已迁批6 ItemService]；
//! 批7 InputMapper 无 HostCtx 面=N/A（输入面三漏斗不涉 host 方法）——逐条处置归宿如下）：
//! | 方法 | 消费面实测 | 处置归宿 |
//! |---|---|---|
//! | find_path | script MOV/SEE+tap_walk 寻路 | pathfind 域独立（plan §1）——不入 trait 面 |
//! | give_gold / pickup_item | GIN op 面 | ~~批6 ItemService~~ **已迁（批6 落定）** |
//! | snap_camera_to_player / ces_camera / set_camera_anchor_target / snap_camera_after_ros1 | 脚本相机编排 | Coordinator 编排面（plan §1 相机 4 不入 trait） |
//! | exit_application / mvs_exchange | **仅 ScriptEngine 边界**（script.rs:496/507；game 面 op_end=transition_to/op_mvs=内联体不走 host） | 平台边界 stub 留 HostCtx |
//! | locate_speaker_entity | open_dialog_page（script.rs:224） | 批3 已注 dialog 面边界件 |
//!
//! 【SCRIPT_BLIND_ZONES】script 域 dump 盲区：scriptLine/scriptCursor/scriptDelay（state_dump 无 FLD）；
//! scriptLineFlags/currentScriptIndex/scriptWalkArmed=FLD 覆盖非盲区。plan:69 点名项无 script 域项
//! （dialog/persist 域已登记=p2b5 跨域断言；menu 4+tap 1=显式 pending 批7/批9 销账）。

use crate::combat::PlayerCombat;
use crate::entity::EntityTable;

/// ScriptOp 前置引用（script_ops.rs pub enum——批1 可见性 1 词提升面）。
pub use crate::scene::ScriptOp;

/// 脚本域状态切片（ScriptCtx——写面 6 槽=field-ownership script 域+读 player/entities/floor 值拷贝）。
pub struct ScriptCtx<'a> {
    pub current_script_index: &'a mut usize,
    pub script_cursor: &'a mut usize,
    pub script_delay: &'a mut i32,
    pub script_line: &'a mut String,
    pub script_line_flags: &'a mut Vec<bool>,
    pub script_walk_armed: &'a mut bool,
    pub player: &'a PlayerCombat,
    pub entities: &'a EntityTable,
    /// persist.floor 值拷贝（entity_category 的层覆写查表用——读槽不计写集）。
    pub floor: i32,
}

/// spec §3.1：脚本 op 执行面。
pub trait ScriptHost {
    fn execute_op(&mut self, ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep;
}

/// 执行步信号（终形）：Advance=普通步进；Recurse=同拍尾递归执行下一条
/// （GIN a.java:8435-8436 完后尾递归——体随迁移信号化，Coordinator 执行递归）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptStep {
    Advance,
    Recurse,
}

/// 脚本 op 解析器（无状态 ZST——脚本域写经 ctx 槽）。
pub struct ScriptResolver;

impl ScriptHost for ScriptResolver {
    /// 脚本域写面收拢（对照 script_ops.rs 旧体逐行归位）：
    /// - 普适 `set_script_cursor(op.cursor())`（旧 execute_op:545）→ ctx.script_cursor
    /// - GUT 旗标（旧 op_gut:155-156）→ ctx.script_line_flags[cur]
    /// - CES scriptStepDelay（旧 op_ces:251-254，a.java:8398-8406 cat 1|8|16=5/2|4=8/其余 0）
    ///   → ctx.script_delay（entity_category 层覆写语义：floor_category 表 else render_category）
    /// - GIN 尾递归（旧 op_gin:84）→ ScriptStep::Recurse
    /// 其余 op=Advance（编排请求由 Coordinator 按 op 面执行 op_* helper——遗留直写批10）。
    fn execute_op(&mut self, ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep {
        *ctx.script_cursor = op.cursor();
        match op {
            ScriptOp::Gut { .. } => {
                let cur = *ctx.current_script_index;
                if (cur as usize) < ctx.script_line_flags.len() {
                    ctx.script_line_flags[cur] = true;
                }
                ScriptStep::Advance
            }
            ScriptOp::Ces { ty, .. } => {
                *ctx.script_delay = match entity_category_with_floor(ty, ctx.floor) {
                    1 | 8 | 16 => 5,
                    2 | 4 => 8,
                    _ => 0,
                };
                ScriptStep::Advance
            }
            ScriptOp::Gin { .. } => ScriptStep::Recurse,
            _ => ScriptStep::Advance,
        }
    }
}

/// entity_category 层覆写语义单源（scene.rs:4523 薄委托——禁双轨：真体在此）。
pub fn entity_category_with_floor(t: i32, floor: i32) -> u8 {
    crate::resource_cfg::global()
        .floor_category(t, floor)
        .unwrap_or_else(|| crate::entity::render_category(t))
}

/// script 域 dump 盲区清单（state_dump 无对应 FLD）。
pub const SCRIPT_BLIND_ZONES: &[&str] = &[
    "scriptLine（script_line）：load_level_script 装填+advance 翻行——dump 盲区（state_dump 无 scriptLine）",
    "scriptCursor（script_cursor）：execute_op 普适写 op.cursor()+load 重置 0——dump 盲区（state_dump 无 scriptCursor）",
    "scriptDelay（script_delay）：CES cat 写+dec_script_delay 消耗——dump 盲区（state_dump 无 scriptDelay）",
    "（评估）scriptLineFlags/currentScriptIndex/scriptWalkArmed=FLD 覆盖非盲区；plan:69 点名项无 script 域项",
];

/// plan:69 点名项显式 pending（批7 InputMapper/tap+批9 ShopGoods/menu 登记时销账——批3 红1 教训横切）。
pub const PLAN_NAMED_PENDING: &[&str] = &[];
// plan:69-71 点名 13 项已全部登记销账：dialog4 批3（DIALOG_BLIND_ZONES）+persist4 批4（PERSIST_BLIND_ZONES）
// +menu4 批6（MENU_BLIND_ZONES）+tap1 批7（INPUT_BLIND_ZONES tap.auto_battle）。本表清空，挂账机制保留。

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> (PlayerCombat, EntityTable) {
        let p = PlayerCombat {
            hp: 100, atk: 10, def: 5,
            yellow_keys: 0, blue_keys: 0, red_keys: 0, gold: 0,
            item_types: Vec::new(), item_uses: Vec::new(), item_stack_size: 0,
        };
        (p, EntityTable::new(0))
    }

    fn run(player: &PlayerCombat, entities: &EntityTable, flags: &mut Vec<bool>, cidx: &mut usize,
           cur: &mut usize, delay: &mut i32, line: &mut String, armed: &mut bool,
           op: ScriptOp) -> ScriptStep {
        let mut ctx = ScriptCtx {
            current_script_index: cidx,
            script_cursor: cur,
            script_delay: delay,
            script_line: line,
            script_line_flags: flags,
            script_walk_armed: armed,
            player,
            entities,
            floor: 1,
        };
        ScriptResolver.execute_op(&mut ctx, op)
    }

    /// 【P2 批5 红测试】脚本域写面收拢钉：Gut 旗标/Ces delay/普适 cursor 写。
    #[test]
    fn p2b5_execute_op_script_writes() {
        let (p, e) = fixtures();
        let (mut cidx, mut cur, mut delay) = (0usize, 0usize, 0i32);
        let (mut line, mut armed) = (String::new(), false);
        let mut flags = vec![false; 4];
        let s = run(&p, &e, &mut flags, &mut cidx, &mut cur, &mut delay, &mut line, &mut armed,
                    ScriptOp::Gut { target: 22, cursor: 17 });
        assert_eq!(s, ScriptStep::Advance, "Gut 步进");
        assert!(flags[0], "Gut 旗标写入 ctx.script_line_flags[cur]");
        assert_eq!(cur, 17, "普适 cursor 写=op.cursor()");
        run(&p, &e, &mut flags, &mut cidx, &mut cur, &mut delay, &mut line, &mut armed,
            ScriptOp::Ces { ty: 1, cx: 0, cy: 0, cursor: 3 });
        assert_eq!(delay, 5, "Ces cat1 delay=5");
        run(&p, &e, &mut flags, &mut cidx, &mut cur, &mut delay, &mut line, &mut armed,
            ScriptOp::Ces { ty: 85, cx: 0, cy: 0, cursor: 3 });
        assert_eq!(delay, 8, "Ces cat2(delay=8) ty=85");
        run(&p, &e, &mut flags, &mut cidx, &mut cur, &mut delay, &mut line, &mut armed,
            ScriptOp::Ces { ty: 72, cx: 0, cy: 0, cursor: 3 });
        assert_eq!(delay, 0, "Ces cat32 delay=0 ty=72");
    }

    /// 【P2 批5 红测试】GIN 尾递归信号化（a.java:8435-8436 同拍下一条）。
    #[test]
    fn p2b5_gin_recurse_signal() {
        let (p, e) = fixtures();
        let (mut cidx, mut cur, mut delay) = (0usize, 0usize, 0i32);
        let (mut line, mut armed) = (String::new(), false);
        let mut flags = vec![false; 4];
        assert_eq!(
            run(&p, &e, &mut flags, &mut cidx, &mut cur, &mut delay, &mut line, &mut armed,
                ScriptOp::Gin { kind: 1, value: 5, cursor: 0 }),
            ScriptStep::Recurse, "Gin=同拍尾递归信号");
        assert_eq!(
            run(&p, &e, &mut flags, &mut cidx, &mut cur, &mut delay, &mut line, &mut armed,
                ScriptOp::RosNoop { cursor: 0 }),
            ScriptStep::Advance, "RosNoop=普通步进");
    }

    /// 【P2 批5 红测试】盲区登记：script 域清单+plan:69 点名项已域登记或显式 pending（批3 红1 教训横切）。
    #[test]
    fn p2b5_blind_zones_cover_plan_named() {
        let s = SCRIPT_BLIND_ZONES.join("\n");
        for name in ["scriptLine", "scriptCursor", "scriptDelay"] {
            assert!(s.contains(name), "script 盲区漏登记：{name}");
        }
        let d = crate::subsystem::dialog::DIALOG_BLIND_ZONES.join("\n");
        for name in ["typewriterPos", "dialogSpeaker", "dialogAnchor", "dialogTarget"] {
            assert!(d.contains(name), "dialog 盲区漏登记 plan:69 点名项：{name}");
        }
        let pp = crate::subsystem::persist_codec::PERSIST_BLIND_ZONES.join("\n");
        for name in ["minFloor", "maxFloor", "pendingFloor", "wentDown"] {
            assert!(pp.contains(name), "persist 盲区漏登记 plan:69 点名项：{name}");
        }
        assert!(PLAN_NAMED_PENDING.is_empty(), "pending 全清（批7 tap 销账）");
        let ip = crate::subsystem::input_mapper::INPUT_BLIND_ZONES.join("\n");
        assert!(ip.contains("auto_battle"), "tap 盲区漏登记 plan:69 点名项：autoBattle");
    }
}
