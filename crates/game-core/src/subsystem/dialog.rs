//! dialog face（P2 批3 终形）——DialogSubsystem：对话推进面（spec §3.1 tick + Host 边界 2 方法）。
//!
//! 形照 spec §3.1：`fn tick(&mut self, ctx: &mut Ctx) -> DialogStep`。
//! 2026-11-07 批3：打字机域步进（a.java:4029-4026）→ DialogStep 六臂终形+DialogResolver；
//! Host 边界 2 方法（set_typewriter/m_082_set_dialog_target）自 HostCtx 逐字迁入（spec 扩展注记④）。
//! 【批0 骨架期措辞留痕】原记「实现迁移=批3（scene_dialog.rs tick_dialog 簇→impl DialogSubsystem）」——已落。

use crate::scene::DialogState;

/// 对话步结果（批3 终形——scene_dialog.rs TypewriterStep 保留同名别名，零测试改动）。
/// 六臂=a.java:4029-4026 打字机域步进决策（derive 同源 TypewriterStep）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogStep {
    /// 打字机推进拍（隔帧 +1+转义跳宽 \c=+8 \r=+2——advance_typewriter 域步进）
    Typing,
    /// key≠0 打字中=跳行尾请求（编排面 skip_text_to_line_end 处置）
    SkipToLineEnd,
    /// 到尾 key=-2 翻下页（编排面 flip_overlay_page(top+lines_per_page)）
    PageNext,
    /// 到尾 key=-1 翻上页（编排面 flip_overlay_page(top-lines_per_page)）
    PagePrev,
    /// 到尾 key=0 静置等待
    Wait,
    /// 到尾 key 其他=确认（域写 dialog_char=1——编排面 open_dialog_page(page+1) 接续）
    Confirm,
}

/// DialogCtx——DialogSubsystem 专属 Ctx（纯数据）。
///
/// dialog=记录槽（DialogState 22 字段经槽写——写面归因登记 face=DialogCtx）；
/// key_value/key_held/frame_odd=值拷贝读面（coordinator 域只读，不计写集）。
pub struct DialogCtx<'a> {
    pub dialog: &'a mut DialogState,
    pub key_value: i32,
    pub key_held: i32,
    pub frame_odd: bool,
}

/// spec §3.1：对话子系统。
pub trait DialogSubsystem {
    /// 打字机域步进（a.java:4029-4026 run case 11 域决策——编排面按 DialogStep 分派）。
    fn tick(&mut self, ctx: &mut DialogCtx) -> DialogStep;
    /// 【批3 自 HostCtx 逐字迁】layoutWrappedText 打字机装填（ScriptEngine 消费面——Host 边界方法）。
    fn set_typewriter(&mut self, text: &str, y: i32, pos: i32);
    /// 【批3 自 HostCtx 逐字迁】ROS_2 tryEquip 对白目标（ScriptEngine 消费面——Host 边界方法）。
    fn m_082_set_dialog_target(&mut self, v: i32);
}

/// 对话解析器（无状态——全部状态经 DialogCtx 借用槽；Coordinator 内联构造调用）。
pub struct DialogResolver;

impl DialogSubsystem for DialogResolver {
    fn tick(&mut self, ctx: &mut DialogCtx) -> DialogStep {
        let text = ctx.dialog.overlay_text.clone().unwrap_or_default();
        let step = ctx.dialog.advance_typewriter(ctx.key_value, ctx.frame_odd, &text);
        if step == DialogStep::Confirm {
            ctx.dialog.set_dialog_char(1);
        }
        step
    }
    fn set_typewriter(&mut self, _text: &str, _y: i32, _pos: i32) {
        // Host 边界方法（ScriptEngine 消费面）——resolver 不服务此面（stub）。
    }
    fn m_082_set_dialog_target(&mut self, _v: i32) {
        // Host 边界方法（ScriptEngine 消费面）——resolver 不服务此面（stub）。
    }
}

/// 对话盲区清单登记（plan 批3 判据「盲区登记入档」）——批3 只迁移打字机域步进+Host 边界 2 方法；
/// 以下对话面存量=Coordinator 编排遗留/跨域面（批10 清零面逐条处置），非本 face 承诺范围：
pub const DIALOG_BLIND_ZONES: &[&str] = &[
    "layout_full_text/layout_wrapped_text_limited 排版面（scene.rs:859/1109——render 域编排）",
    "skip_text_to_line_end 跳行尾算法（scene.rs:1828——SkipToLineEnd 臂编排处置）",
    "溢出翻页检（lines_per_page 算术+flip_overlay_page 调用——tick_dialog 编排）",
    "open_dialog_page 页面开合编排（scene.rs:1047 GameScene inherent）",
    "phase 0-5 跨域分派（script 延迟/行走/相机/战斗——tick_dialog 相位机非 dialog 域）",
    "phase6 Confirm 臂 ease 相机 4 setter（camera_ease 域编排尾）",
    "dialog_phase/dialog_page 编排直写（Coordinator 遗留直写——批10 清零面）",
    "typewriterPos=dialog_char（已入 DialogCtx 记录槽写面登记——对照注记非盲区）",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{DialogData, DialogState};

    fn dialog_state() -> DialogState {
        DialogState {
            dialog_phase: 0,
            dialog_page: 0,
            dialog_page_end: 0,
            dialog_speaker: 0,
            dialog_anchor: -1,
            dialog_char: 0,
            dialog_target: (0, 0),
            dialog_data: DialogData { scripts: vec![], texts: vec![], speakers: vec![], bubbles: vec![] },
            overlay_active: false,
            overlay_kind: 0,
            overlay_text: None,
            overlay_box_w: 0,
            overlay_box_h: 0,
            overlay_left_icon: 0,
            overlay_right_icon: 0,
            overlay_lines: vec![],
            overlay_page_top: 0,
            overlay_page_bottom: 0,
            overlay_lines_per_page: 0,
            overlay_total_lines: 0,
            overlay_content_w: 0,
            overlay_saved_color: 0,
        }
    }

    #[test]
    fn p2b3_dialog_step_shape() {
        let steps = [
            DialogStep::Typing,
            DialogStep::SkipToLineEnd,
            DialogStep::PageNext,
            DialogStep::PagePrev,
            DialogStep::Wait,
            DialogStep::Confirm,
        ];
        assert_eq!(steps.len(), 6, "DialogStep 终形六臂");
    }

    #[test]
    fn p2b3_resolve_typewriter_domain() {
        // 打字机域步进钉（a.java:4029-4026：隔帧推进+转义跳宽+键分派）。
        // 打字拍：char 0→1（frame_odd 推进）
        let mut d = dialog_state();
        d.overlay_text = Some("abc".into());
        let mut ctx = DialogCtx { dialog: &mut d, key_value: 0, key_held: 0, frame_odd: true };
        let mut r = DialogResolver;
        assert_eq!(r.tick(&mut ctx), DialogStep::Typing);
        assert_eq!(ctx.dialog.dialog_char, 1, "隔帧推进 +1");
        // 转义跳宽：\c=+8（首字符推进 1 后 idx=0='\\' → +8 → 9）
        let mut d = dialog_state();
        d.overlay_text = Some("\\cdef".into());
        let mut ctx = DialogCtx { dialog: &mut d, key_value: 0, key_held: 0, frame_odd: true };
        assert_eq!(r.tick(&mut ctx), DialogStep::Typing);
        assert_eq!(ctx.dialog.dialog_char, 9, "转义跳宽 \\c=+8（a.java 字效）");
        // key≠0=跳行尾请求（打字中）
        let mut d = dialog_state();
        d.overlay_text = Some("abc".into());
        let mut ctx = DialogCtx { dialog: &mut d, key_value: 3, key_held: 0, frame_odd: true };
        assert_eq!(r.tick(&mut ctx), DialogStep::SkipToLineEnd);
        assert_eq!(ctx.dialog.dialog_char, 0, "跳行尾请求不推进");
        // 到尾键分派：-2/-1/0/其他
        let mut d = dialog_state();
        d.overlay_text = Some("ab".into());
        d.dialog_char = 2;
        let mut ctx = DialogCtx { dialog: &mut d, key_value: -2, key_held: 0, frame_odd: true };
        assert_eq!(r.tick(&mut ctx), DialogStep::PageNext);
        let mut d = dialog_state();
        d.overlay_text = Some("ab".into());
        d.dialog_char = 2;
        let mut ctx = DialogCtx { dialog: &mut d, key_value: -1, key_held: 0, frame_odd: true };
        assert_eq!(r.tick(&mut ctx), DialogStep::PagePrev);
        let mut d = dialog_state();
        d.overlay_text = Some("ab".into());
        d.dialog_char = 2;
        let mut ctx = DialogCtx { dialog: &mut d, key_value: 0, key_held: 0, frame_odd: true };
        assert_eq!(r.tick(&mut ctx), DialogStep::Wait);
        // Confirm：域写 dialog_char=1（编排面随后 open_dialog_page）
        let mut d = dialog_state();
        d.overlay_text = Some("ab".into());
        d.dialog_char = 2;
        let mut ctx = DialogCtx { dialog: &mut d, key_value: 7, key_held: 0, frame_odd: true };
        assert_eq!(r.tick(&mut ctx), DialogStep::Confirm);
        assert_eq!(ctx.dialog.dialog_char, 1, "Confirm 域写 dialog_char=1");
    }
}
