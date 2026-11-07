//! ui_render face（P2 批8 终形）——UIRender：UiFrame 纯读快照面（spec §3.1 注③=只读边界件，不入互调通信面）。
//!
//! 形照 plan:96 锚：`fn snapshot(&self) -> UiFrame` 纯读（E3 RenderState 消费面）。
//! 判据=纯读断言（无 &mut——sig_lock _sig_ui_snapshot 编译期钉）+帧夹具（FRAME_OUT 对拍面=
//! game-oracle fixture_frames.rs:310-313 像素取证缝，31 帧哈希夹具判据）。
//!
//! 【批8 处置·批3 registry 注记指针】op_see ease 簇（script_ops.rs ease 写）+op_ces 双簇=
//! set_ease_cam_x/y+set_ease_target_x/y（scene_paint.rs:27-34 render 域直写接口家——【2026-11-08 批10 勘误】原 26-33 逐行 +1 漂移）——归 render 域
//! /UIRender 面，非 dialog 域（dialog.rs 盲区注记指回本行）。
//!
//! 【HostCtx N/A】输入/渲染面无 HostCtx 方法（批8 无再收窄——批5 处置账表形）。
//! 【消费面披露】snapshot 生产消费点=0（shell 接线未入本批——批7 挂账①同口径）：本批只立面+终形。

use crate::scene::RenderState;

/// 【批8 终形】UI 快照帧=E3 RenderState 六字段值拷贝（render 域 paint 变更态；
/// 金样扩面钉在 dump（V0 已扩）+帧哈希 31 夹具判据）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiFrame {
    /// iconStripFrame（a.java:464 全局；mode16 图标条动画帧）
    pub strip_frame: usize,
    /// drawParallaxBackdrop 的滚动相位（每 paint -1，PARALLAX_WRAP 回绕）
    pub backdrop_scroll: i32,
    /// advanceBobPhase 浮沉相位（paint 内推进）
    pub bob_offset: i32,
    pub bob_rising: bool,
    /// openAnimTick：组开门动画计数（18 置 removed 动画、24 收尾）
    pub open_anim_tick: i32,
    /// tileBob：遮幅纵向抖动（±2 交替）
    pub tile_bob: i32,
}

/// spec §3.1：UI 渲染纯读边界件（不入互调通信面——只读）。
pub trait UIRender {
    fn snapshot(&self) -> UiFrame;
}

/// 只读快照解析件（借用槽——ZST 无法承载读源；批8 特例形）。
pub struct UIRenderResolver<'a>(pub &'a RenderState);

impl UIRender for UIRenderResolver<'_> {
    fn snapshot(&self) -> UiFrame {
        UiFrame {
            strip_frame: self.0.strip_frame,
            backdrop_scroll: self.0.backdrop_scroll,
            bob_offset: self.0.bob_offset,
            bob_rising: self.0.bob_rising,
            open_anim_tick: self.0.open_anim_tick,
            tile_bob: self.0.tile_bob,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rs_fix() -> crate::scene::RenderState {
        crate::scene::RenderState {
            strip_frame: 3,
            backdrop_scroll: -7,
            bob_offset: 5,
            bob_rising: true,
            open_anim_tick: 11,
            tile_bob: -2,
        }
    }

    /// 【P2 批8 红测试】UiFrame=E3 RenderState 六字段值拷贝逐字段钉（plan:96 判据）。
    #[test]
    fn p2b8_ui_frame_snapshot_value_copy() {
        let rs = rs_fix();
        let f = { let r = UIRenderResolver(&rs); r.snapshot() };
        assert_eq!((f.strip_frame, f.backdrop_scroll, f.bob_offset), (3, -7, 5), "值拷贝前三字段");
        assert_eq!((f.bob_rising, f.open_anim_tick, f.tile_bob), (true, 11, -2), "值拷贝后三字段");
    }

    /// 【P2 批8 红测试】纯读断言（plan:96 判据「无 &mut」）：经 &T 可调 snapshot——
    /// 若签名含 &mut 本函数不可编译（编译期结构断言）。
    #[test]
    fn p2b8_ui_render_pure_read() {
        fn via_ref<T: UIRender>(t: &T) -> UiFrame {
            t.snapshot()
        }
        let rs = rs_fix();
        let f = via_ref(&UIRenderResolver(&rs));
        assert_eq!(f.strip_frame, 3);
        assert_eq!(f.bob_rising, true);
    }

    /// 【P2 批8 红测试】UiFrame 字段集==RenderState E3 六字段同集钉（防单侧加字段）。
    #[test]
    fn p2b8_ui_frame_field_set_matches_render_state() {
        let f = { let rs = rs_fix(); let r = UIRenderResolver(&rs); r.snapshot() };
        let names_f = ["strip_frame", "backdrop_scroll", "bob_offset", "bob_rising", "open_anim_tick", "tile_bob"];
        let names_r = ["strip_frame", "backdrop_scroll", "bob_offset", "bob_rising", "open_anim_tick", "tile_bob"];
        // 【批10 黄④判别力补强】名单比对本身恒真——真判据=下方解构钉：两侧字段集
        // 与名单一一对应，任一侧新增/删字段即编译错（非恒真可判别）。
        let UiFrame { strip_frame: _, backdrop_scroll: _, bob_offset: _, bob_rising: _, open_anim_tick: _, tile_bob: _ } = f;
        let crate::scene::RenderState {
            strip_frame: _, backdrop_scroll: _, bob_offset: _, bob_rising: _,
            open_anim_tick: _, tile_bob: _,
        } = rs_fix();
        assert_eq!(names_f, names_r, "六字段同集（E3 render 域）");
        let _ = f;
    }
}
