//! 【STAGE-B-008 计划 3 T5 · spec §2.2/§4.2】功能查询的呈现面（壳侧）：FunctionId 中文标签
//!（M8：core 只给数据不给文字）+ 工具栏命中分区。执行走 core `perform_action` / `undo_last`。

use game_core::controls::{ControlButton, Controls, FunctionId};

/// 功能名（工具栏钮 / 摇杆软键标注共用；spec §3.3 两种呈现读同一查询）。
pub fn label(f: FunctionId) -> &'static str {
    match f {
        FunctionId::Bag => "背包",
        FunctionId::Observe => "观察",
        FunctionId::Undo => "撤销",
        FunctionId::CloudUp => "筋斗云↑",
        FunctionId::CloudDown => "筋斗云↓",
        FunctionId::Menu => "菜单",
        FunctionId::Stop => "停下",
        FunctionId::Cancel => "取消",
        FunctionId::Fight => "开战",
        FunctionId::ExitObserve => "退出观察",
        FunctionId::PrevPage => "上页",
        FunctionId::NextPage => "下页",
        FunctionId::ScrollUp => "上滚",
        FunctionId::ScrollDown => "下滚",
        FunctionId::Continue => "继续",
        FunctionId::Use => "使用",
        FunctionId::Back => "返回",
        FunctionId::Buy => "买",
        // spec §3.2 src1 报价右钮原文「不买」（T1-R3 更正：与赠予拒绝措辞区分）
        FunctionId::Refuse => "不买",
        FunctionId::Accept => "接受",
        FunctionId::Close => "关闭",
        FunctionId::PrevRec => "上一条",
        FunctionId::NextRec => "下一条",
        FunctionId::Confirm => "确定",
        FunctionId::Save => "存档",
        FunctionId::Read => "读取",
        FunctionId::Refine => "炼丹",
        FunctionId::PrevItem => "上一项",
        FunctionId::NextItem => "下一项",
        FunctionId::Skip => "跳过",
    }
}

/// 工具栏命中（spec §4.2）：触点 → 左/中[idx]/右钮。返回 None=条外。
/// mid 区均分为 `mid_n` 格（n=查询中列钮数；0=中间不可点）。
#[derive(Debug, PartialEq, Eq)]
pub enum ToolbarHit {
    Left,
    Mid(usize),
    Right,
}

pub fn hit_toolbar(x: i32, y: i32, left: (i32, i32, i32, i32), mid: (i32, i32, i32, i32), right: (i32, i32, i32, i32), mid_n: usize) -> Option<ToolbarHit> {
    let (lx, ly, lw, lh) = left;
    if x >= lx && x < lx + lw && y >= ly && y < ly + lh {
        return Some(ToolbarHit::Left);
    }
    let (rx, ry, rw, rh) = right;
    if x >= rx && x < rx + rw && y >= ry && y < ry + rh {
        return Some(ToolbarHit::Right);
    }
    if mid_n == 0 {
        return None;
    }
    let (mx, my, mw, mh) = mid;
    if x >= mx && x < mx + mw && y >= my && y < my + mh {
        let idx = ((x - mx) * mid_n as i32 / mw).clamp(0, mid_n as i32 - 1) as usize;
        return Some(ToolbarHit::Mid(idx));
    }
    None
}

/// 取查询里第 idx 个中钮。
pub fn mid_button(c: &Controls, idx: usize) -> Option<ControlButton> {
    c.mid.get(idx).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::controls::Action;

    #[test]
    fn t5_labels_and_hit() {
        // 标签：与 spec §3.2 措辞一致（src1 报价右=不买）
        assert_eq!(label(FunctionId::Bag), "背包");
        assert_eq!(label(FunctionId::Refuse), "不买");
        assert_eq!(label(FunctionId::CloudUp), "筋斗云↑");
        // 命中：三分区 + mid 均分
        let l = (0, 100, 100, 60);
        let m = (110, 100, 120, 60);
        let r = (240, 100, 100, 60);
        assert!(matches!(hit_toolbar(50, 130, l, m, r, 2), Some(ToolbarHit::Left)));
        assert!(matches!(hit_toolbar(120, 130, l, m, r, 2), Some(ToolbarHit::Mid(0))));
        assert_eq!(hit_toolbar(200, 130, l, m, r, 2), Some(ToolbarHit::Mid(1)));
        assert!(matches!(hit_toolbar(300, 130, l, m, r, 2), Some(ToolbarHit::Right)));
        assert!(matches!(hit_toolbar(150, 130, l, m, r, 0), None), "mid_n=0 中区不可点");
        assert!(hit_toolbar(105, 50, l, m, r, 2).is_none(), "条外");
        // mid_button
        let c = Controls {
            left: Some(ControlButton { function: FunctionId::Bag, action: Action::Key(-6) }),
            mid: vec![ControlButton { function: FunctionId::Observe, action: Action::Key(-5) }, ControlButton { function: FunctionId::Undo, action: Action::Undo }],
            right: Some(ControlButton { function: FunctionId::Menu, action: Action::Key(-7) }),
        };
        assert_eq!(mid_button(&c, 1).unwrap().function, FunctionId::Undo);
        assert!(mid_button(&c, 2).is_none());
    }
}
