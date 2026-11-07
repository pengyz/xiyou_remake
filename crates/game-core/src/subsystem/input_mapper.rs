//! input_mapper face（P2 批7 终形）——InputMapper：RawInput→Action 输入映射面（spec §3.1）。
//!
//! 形照 spec §3.1：`fn map(&self, ev: RawInput) -> Action`（&self 纯查询）。plan:35 问题陈述=
//! 「输入解析散两域（controls.rs/tap_walk.rs）；RawInput→Action 抽象缺」——本面收拢映射抽象。
//! `apply` 为批7 扩展（spec F2 注记⑥）：perform_action 的输入域写收拢（key_value/tap 状态族），
//! 编排（interrupt_route/engage_selected）留 Coordinator 由 [`InputStep`] 桥接（批2/5 同款）。

use crate::controls::Action;

/// 原始输入事件（批7 终形）：三动作产出边界事件——shell 三漏斗实证
/// （shell/win/src/main.rs:669 地图格/shell/common/src/controls_ui.rs 工具栏命中/shell/common/src/touch.rs:145
/// TapTarget=Undo 区>地图格>图外/scene.rs:488 键盘）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawInput {
    /// 物理/软键键码（key_value 同币——边沿）。
    Key(i32),
    /// 工具栏/摇杆标注命中按钮（**自带绑定**=controls 两表单源；host 差异随 ControlButton 携带——
    /// GameScene:203-205 Bag/Menu=StopThen vs BootMachine 测试 :362-363=Key，map 不自设绑定防双轨）。
    TapButton(crate::controls::ControlButton),
    /// 撤销区触点（touch.rs:145 TapTarget::Undo）。
    TapUndo,
}

/// [`InputResolver::apply`] 返回的编排步（BattleOutcome/ScriptStep 先例形）：域写已在 resolver 内完成，
/// 剩余跨域编排由 Coordinator 执行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputStep {
    /// 受理（域写完成，无剩余编排）。
    Accepted,
    /// Undo 恒 false（core 无撤销入口——撤销执行在壳 undo_ui，spec perform_action 语义）。
    Ignored,
    /// 需打断寻路（Coordinator 执行 interrupt_route）。
    Interrupt,
    /// 需撞选中怪（Coordinator 执行 engage_selected）。
    Engage,
    /// 打断后排队键码（域写=resume: tap_pending=StopThen(c) 已在 resolver 装配；
    /// Coordinator 仅执行 interrupt_route——interrupt_route:1646 不触 tap_pending，序等价 A 级）。
    InterruptThen(i32),
}

/// 输入域写槽（ItemCtx 先例：借用槽+值拷贝纯数据）。
pub struct InputCtx<'a> {
    /// key_value 写槽（scene.rs:333）。
    pub key_value: &'a mut i32,
    /// tap 状态族写槽（tap_walk.rs:9 TapState 11 字段）。
    pub tap: &'a mut crate::tap_walk::TapState,
    /// 读槽：寻路中（值拷贝——perform_action 两臂分支门）。
    pub walk_in_progress: bool,
}

/// spec §3.1：输入映射（Action=controls.rs:44 既有 pub enum，复用不新建）。
pub trait InputMapper {
    fn map(&self, ev: RawInput) -> Action;
    /// 【批7 扩展·spec F2 注记⑥】Action 执行域写收拢（perform_action 逐臂等价）。
    fn apply(&mut self, ctx: &mut InputCtx, a: Action) -> InputStep;
}

/// ZST 解析核（批2 BattleResolver/批6 ItemResolver 先例）。
pub struct InputResolver;

impl InputMapper for InputResolver {
    fn map(&self, ev: RawInput) -> Action {
        match ev {
            RawInput::Key(c) => Action::Key(c),
            RawInput::TapButton(cb) => cb.action,
            RawInput::TapUndo => Action::Undo,
        }
    }

    fn apply(&mut self, ctx: &mut InputCtx, a: Action) -> InputStep {
        match a {
            Action::Key(c) => {
                *ctx.key_value = c;
                InputStep::Accepted
            }
            Action::Stop => {
                if ctx.walk_in_progress {
                    ctx.tap.tap_bump = None;
                    ctx.tap.tap_selected = None;
                    ctx.tap.tap_pending = None;
                    ctx.tap.tap_target = None;
                    InputStep::Interrupt
                } else {
                    InputStep::Accepted
                }
            }
            Action::Cancel => {
                ctx.tap.tap_selected = None;
                InputStep::Accepted
            }
            Action::Fight => InputStep::Engage,
            Action::StopThen(c) => {
                if ctx.walk_in_progress {
                    ctx.tap.tap_bump = None;
                    ctx.tap.tap_selected = None;
                    ctx.tap.tap_target = None; // 【复核 T2-R5】打断 ⇒ 目标标记消失（spec §6）
                    ctx.tap.tap_pending = Some(crate::tap_walk::PendingTap::StopThen(c));
                    InputStep::InterruptThen(c)
                } else {
                    *ctx.key_value = c;
                    InputStep::Accepted
                }
            }
            Action::Undo => InputStep::Ignored,
        }
    }
}

/// tap-input 域盲区登记（plan:69-71 点名 tap 键——本批销账）。
pub const INPUT_BLIND_ZONES: &[&str] = &[
    // plan:69 点名 tap 键（唯一点名项）：auto_battle —— 写点 scene_persist.rs:57（init false）+
    // 读门 scene.rs:1279（!auto_battle && !again 首点选中门）；夹具钉 t6_auto_battle_fights_on_first_tap
    // （t2_walkto_route_tests.rs:1740）。本批 PLAN_NAMED_PENDING 销账归档于此。
    "tap.auto_battle（plan:69 tap 点名项）：写 scene_persist.rs:57 init+读门 scene.rs:1279 首点选中；t6 夹具钉",
    // 评估行（非点名）：tap 状态机内部态——ui_selected/ui_frame（两段式选中面）、tap_reject/tap_followup_ready、
    // walkto_arrows/battle_undo_snap=战斗撤销快照面；tap_target/tap_bump/tap_selected/tap_pending=FLD 无 dump
    // 覆盖（B 级评估：行为瞬态非持久盲区）。
    "tap 状态族评估（非点名）：ui_selected/ui_frame/tap_reject/tap_followup_ready/walkto_arrows/battle_undo_snap 瞬态；tap_target/tap_bump/tap_selected/tap_pending FLD 无 dump（B 级）",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::{Action, ControlButton, FunctionId};

    fn btn(f: FunctionId, a: Action) -> ControlButton {
        ControlButton { function: f, action: a }
    }

    fn fresh_tap() -> crate::tap_walk::TapState {
        crate::tap_walk::TapState {
            ui_selected: None,
            ui_frame: 0,
            walkto_arrows: None,
            battle_undo_snap: None,
            auto_battle: false,
            tap_target: None,
            tap_reject: None,
            tap_bump: None,
            tap_followup_ready: false,
            tap_selected: None,
            tap_pending: None,
        }
    }

    fn sel() -> crate::tap_walk::SelectedEnemy {
        crate::tap_walk::SelectedEnemy { entity: 3, type_: 42, cell: (1, 2), floor: 7 }
    }

    /// p2b7①：map 三臂语义钉（Key 边沿同币/TapButton 绑定单源 cb.action/TapUndo→Undo）。
    #[test]
    fn p2b7_map_raw_input_to_action() {
        let r = InputResolver;
        assert_eq!(r.map(RawInput::Key(5)), Action::Key(5), "Key 边沿键值同币");
        let cb = btn(FunctionId::Fight, Action::Fight);
        assert_eq!(r.map(RawInput::TapButton(cb)), Action::Fight, "TapButton 绑定单源=cb.action");
        let cb2 = btn(FunctionId::Bag, Action::StopThen(-6));
        assert_eq!(r.map(RawInput::TapButton(cb2)), Action::StopThen(-6), "host 差异随 ControlButton 携带");
        assert_eq!(r.map(RawInput::TapUndo), Action::Undo, "撤销区触点→Undo");
    }

    /// p2b7②：apply 域写收拢钉（perform_action 域写逐臂等价）。
    #[test]
    fn p2b7_apply_domain_writes() {
        let mut kv = 0i32;
        let mut tap = fresh_tap();
        // Key(c) → key_value=c
        {
            let mut ctx = InputCtx { key_value: &mut kv, tap: &mut tap, walk_in_progress: false };
            let mut r = InputResolver;
            assert_eq!(r.apply(&mut ctx, Action::Key(7)), InputStep::Accepted);
            assert_eq!(kv, 7, "Key 臂写 key_value");
        }
        // Undo → Ignored（core 无撤销入口）
        {
            let mut ctx = InputCtx { key_value: &mut kv, tap: &mut tap, walk_in_progress: false };
            let mut r = InputResolver;
            assert_eq!(r.apply(&mut ctx, Action::Undo), InputStep::Ignored);
        }
        // Cancel → tap_selected=None
        {
            tap.tap_selected = Some(sel());
            let mut ctx = InputCtx { key_value: &mut kv, tap: &mut tap, walk_in_progress: false };
            let mut r = InputResolver;
            assert_eq!(r.apply(&mut ctx, Action::Cancel), InputStep::Accepted);
            assert!(tap.tap_selected.is_none(), "Cancel 清选中");
        }
        // Fight → Engage（编排留 Coordinator）
        {
            let mut ctx = InputCtx { key_value: &mut kv, tap: &mut tap, walk_in_progress: false };
            let mut r = InputResolver;
            assert_eq!(r.apply(&mut ctx, Action::Fight), InputStep::Engage);
        }
        // Stop 寻路中 → 清四态+Interrupt
        {
            tap.tap_bump = Some(((3, 4), 1u8));
            tap.tap_selected = Some(sel());
            tap.tap_pending = Some(crate::tap_walk::PendingTap::StopThen(9));
            tap.tap_target = Some((5, 6));
            let mut ctx = InputCtx { key_value: &mut kv, tap: &mut tap, walk_in_progress: true };
            let mut r = InputResolver;
            assert_eq!(r.apply(&mut ctx, Action::Stop), InputStep::Interrupt);
            assert!(tap.tap_bump.is_none() && tap.tap_selected.is_none() && tap.tap_pending.is_none() && tap.tap_target.is_none(), "Stop 清四态");
        }
        // StopThen(c) 非寻路 → key_value=c（原分支）
        {
            let mut ctx = InputCtx { key_value: &mut kv, tap: &mut tap, walk_in_progress: false };
            let mut r = InputResolver;
            assert_eq!(r.apply(&mut ctx, Action::StopThen(-6)), InputStep::Accepted);
            assert_eq!(kv, -6, "非寻路 StopThen 直发键");
        }
        // StopThen(c) 寻路中 → 清三态+InterruptThen(c)（pending 装配在 resolver；T2-R5 目标标记消失）
        {
            tap.tap_bump = Some(((3, 4), 1u8));
            tap.tap_selected = Some(sel());
            tap.tap_target = Some((5, 6));
            tap.tap_pending = None;
            let mut ctx = InputCtx { key_value: &mut kv, tap: &mut tap, walk_in_progress: true };
            let mut r = InputResolver;
            assert_eq!(r.apply(&mut ctx, Action::StopThen(-7)), InputStep::InterruptThen(-7));
            assert!(tap.tap_bump.is_none() && tap.tap_selected.is_none() && tap.tap_target.is_none(), "T2-R5 目标标记消失");
            assert_eq!(tap.tap_pending, Some(crate::tap_walk::PendingTap::StopThen(-7)), "pending 装配在 resolver");
        }
    }

    /// p2b7③：tap_two_stage 两段式等价夹具（tap_walk.rs:44-56 语义钉——计划判据）。
    #[test]
    fn p2b7_tap_two_stage_equivalence() {
        let mut t = fresh_tap();
        let w = crate::menu_family::TWO_STAGE_MIN_STEPS;
        assert!(!t.tap_two_stage(3, 10), "首点=选中不执行");
        assert_eq!(t.ui_selected, Some((3, 10)));
        assert!(!t.tap_two_stage(3, 10 + w - 1), "窗口内重按不执行");
        assert!(t.tap_two_stage(3, 10 + w), "窗口外同钮=执行");
        assert!(t.ui_selected.is_none(), "执行后清选中");
        assert!(!t.tap_two_stage(5, 100), "异钮=改选");
        assert_eq!(t.ui_selected, Some((5, 100)));
    }

    /// p2b7④：盲区登记（INPUT_BLIND_ZONES）+PLAN_NAMED_PENDING tap:autoBattle 销账断言。
    #[test]
    fn p2b7_blind_zones_cover_plan_named() {
        let all: String = [INPUT_BLIND_ZONES.join("\n"), super::super::script_host::PLAN_NAMED_PENDING.join("\n")].join("\n");
        assert!(!all.contains("autoBattle"), "tap:autoBattle 本批销账（plan:69 点名）");
        assert!(INPUT_BLIND_ZONES.iter().any(|s| s.contains("auto_battle")), "auto_battle 盲区登记");
    }
}
