//! script_host face（P2 批5 终形）——ScriptHost：脚本 op 执行面（spec §3.1 `execute_op(ctx,op)->ScriptStep`）。
//!
//! 【切面·script_ops 既有形】plan 批5「execute_op(ctx,op) trait 化（script_ops 既有形+HostCtx 余方法处置账）」：
//! resolver=**脚本域写面收拢**（普适 cursor 写+Gut 旗标+Ces scriptDelay+GIN 尾递归信号化）；
//! op_* 编排体留 Coordinator 面（跨域调用=遗留直写批10 清零面——本批不迁，避免 21 helper 借用面重构风险）。
//!
//! 【P2 批10 C·Coordinator 遗留直写终裁】按 plan 批10 切面「能归域的归域，编排面按批3-9 先例注记归因登记，
//! 不强迁」——终裁=编排面注记归因登记（各批 BLIND_ZONES/域注记已携归因）：op_* 编排体（跨域 helper 调用序列）、
//! apply_merchant_offer 收尾块（param^=512 装备口令翻转/offer_queue 挂起/text 装配/BUG-045 清栈）、paint 簇 97 fn、
//! ease 相机簇 4 setter（render 域直写接口家）均归编排面登记不迁。382 直写站点=D 项测试构造面（已收敛 SceneSetup），
//! 与本账不同层。
//!
//! 【HostCtx 余方法处置账】（plan 批5 明文交付；**批6 摘 2 后 HostCtx 现 8 方法**[give_gold/pickup_item 已迁批6 ItemService]；
//! 批7 输入面/批8 渲染面均无 HostCtx 方法=N/A（零再收窄）——逐条处置归宿如下）：
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
//! （dialog/persist 域已登记=p2b5 跨域断言；menu4 批6/tap1 批7 已销账——pending 全清）。


/// ScriptOp 前置引用（script_ops.rs pub enum——批1 可见性 1 词提升面）。
pub use crate::scene::ScriptOp;

/// 脚本域状态切片（ScriptCtx——写面 6 槽 + floor 值拷贝）。
///
/// 【2026-11-17 S3·②】原 `player`/`entities` 读槽已删：resolver 全仓唯一消费点是
/// CES 类目查表，而 `entity_category_with_floor(t, floor)` 只吃 `t` 与 `floor`
/// ——两个读槽是历史遗留的死重（保留会让第二执行器为了构造 ctx 而虚构一份玩家/实体）。
pub struct ScriptCtx<'a> {
    pub current_script_index: &'a mut usize,
    pub script_cursor: &'a mut usize,
    pub script_delay: &'a mut i32,
    pub script_line: &'a mut String,
    pub script_line_flags: &'a mut Vec<bool>,
    pub script_walk_armed: &'a mut bool,
    /// persist.floor 值拷贝（entity_category 的层覆写查表用——读槽不计写集）。
    pub floor: i32,
}

/// 脚本域状态的规范载体（S3·②）：六个脚本写槽。
///
/// 生产侧 `GameScene` 暂时仍按散字段存放（4900 行场外引用面，行锚迁移单独成批），
/// 但两执行器的**写规则**已收拢到 [`apply_script_domain`]；引擎侧直接持有本结构，
/// 不再自写游标/旗标/delay/尾递归。
#[derive(Default)]
pub struct ScriptSlots {
    pub current_script_index: usize,
    pub script_cursor: usize,
    pub script_delay: i32,
    pub script_line: String,
    pub script_line_flags: Vec<bool>,
    pub script_walk_armed: bool,
}

impl ScriptSlots {
    /// 投影出执行面 ctx（floor：CES 类目查表的值拷贝）。
    pub fn ctx(&mut self, floor: i32) -> ScriptCtx<'_> {
        ScriptCtx {
            current_script_index: &mut self.current_script_index,
            script_cursor: &mut self.script_cursor,
            script_delay: &mut self.script_delay,
            script_line: &mut self.script_line,
            script_line_flags: &mut self.script_line_flags,
            script_walk_armed: &mut self.script_walk_armed,
            floor,
        }
    }

    /// 单一写入口（游标/GUT 旗标/CES delay/GIN 尾递归信号）。
    /// `ScriptStep::Recurse` ⇒ 调用方同拍重入下一条。
    pub fn apply(&mut self, op: ScriptOp, floor: i32) -> ScriptStep {
        let mut ctx = self.ctx(floor);
        apply_script_domain(&mut ctx, op)
    }
}

/// 脚本域写规则的唯一实现（两执行器共用；禁第二份）。
pub fn apply_script_domain(ctx: &mut ScriptCtx, op: ScriptOp) -> ScriptStep {
    ScriptHost::execute_op(&mut ScriptResolver, ctx, op)
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

/// plan:69 点名项显式 pending（已全清——批3 红1 教训横切机制保留）。
pub const PLAN_NAMED_PENDING: &[&str] = &[];
// plan:69-71 点名 14 token 已全部登记销账（挂账④计数勘误 13→14）：dialog5 批3（DIALOG_BLIND_ZONES 含 overlay）+persist4 批4（PERSIST_BLIND_ZONES）
// +menu4 批6（MENU_BLIND_ZONES）+tap1 批7（INPUT_BLIND_ZONES tap.auto_battle）。本表清空，挂账机制保留。

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(flags_len: usize) -> ScriptSlots {
        ScriptSlots { script_line_flags: vec![false; flags_len], ..Default::default() }
    }

    /// 【P2 批5 红测试】脚本域写面收拢钉：Gut 旗标/Ces delay/普适 cursor 写。
    /// 【2026-11-17 S3·②】改为经共享载体 `ScriptSlots::apply`（两执行器同一入口）。
    #[test]
    fn p2b5_execute_op_script_writes() {
        let mut s = slots(4);
        assert_eq!(
            s.apply(ScriptOp::Gut { target: 22, cursor: 17 }, 1),
            ScriptStep::Advance, "Gut 步进");
        assert!(s.script_line_flags[0], "Gut 旗标写入 script_line_flags[cur]");
        assert_eq!(s.script_cursor, 17, "普适 cursor 写=op.cursor()");
        s.apply(ScriptOp::Ces { ty: 1, cx: 0, cy: 0, cursor: 3 }, 1);
        assert_eq!(s.script_delay, 5, "Ces cat1 delay=5");
        s.apply(ScriptOp::Ces { ty: 85, cx: 0, cy: 0, cursor: 3 }, 1);
        assert_eq!(s.script_delay, 8, "Ces cat2(delay=8) ty=85");
        s.apply(ScriptOp::Ces { ty: 72, cx: 0, cy: 0, cursor: 3 }, 1);
        assert_eq!(s.script_delay, 0, "Ces cat32 delay=0 ty=72");
    }

    /// 【2026-11-17 S3·② 红测试】CES delay 必须由 `entity_category_with_floor(ty, floor)`
    /// 决定——钉住「不许回到静态 type_categories 表」：同一 ty 在不同 floor 上，
    /// 只要覆写表给出不同类目，delay 就必须跟着变。
    #[test]
    fn s3b2_ces_delay_uses_floor_category() {
        use crate::subsystem::script_host::entity_category_with_floor;
        let delay_of = |cat: u8| match cat { 1 | 8 | 16 => 5, 2 | 4 => 8, _ => 0 };
        // 覆盖若干 (ty, floor) 组合：断言 apply 的结果等于「按 floor 解析类目再映射」
        for ty in [1i32, 16, 21, 55, 72, 85, 100] {
            for floor in [1i32, 3, 7, 15, 26, 41, 55] {
                let mut s = slots(4);
                s.apply(ScriptOp::Ces { ty, cx: 0, cy: 0, cursor: 3 }, floor);
                let want = delay_of(entity_category_with_floor(ty, floor));
                assert_eq!(s.script_delay, want,
                    "CES ty={ty} floor={floor}：delay 必须跟随 floor 覆写类目");
            }
        }
    }

    /// 【P2 批5 红测试】GIN 尾递归信号化（a.java:8435-8436 同拍下一条）。
    #[test]
    fn p2b5_gin_recurse_signal() {
        let mut s = slots(4);
        assert_eq!(
            s.apply(ScriptOp::Gin { kind: 1, value: 5, cursor: 0 }, 1),
            ScriptStep::Recurse, "Gin=同拍尾递归信号");
        assert_eq!(
            s.apply(ScriptOp::RosNoop { cursor: 0 }, 1),
            ScriptStep::Advance, "RosNoop=普通步进");
    }

    /// 【P2 批5 红测试】盲区登记：script 域清单+plan:69 点名项已域登记或显式 pending（批3 红1 教训横切）。
    /// 【P2 批10 F / 2026-11-17 S3·③ 改判】宿主面拆分终值对账：
    /// 原「HostCtx 8 方法」拆成 世界面 3 + 表现面 6（其中 1 条从表现面迁回世界面）。
    #[test]
    fn p2b10_host_ctx_final_8_methods() {
        let src = include_str!("../script.rs");
        let world = src.split("pub trait WorldHost").nth(1).unwrap().split("\n}").next().unwrap();
        let pres = src.split("pub trait PresentationHost").nth(1).unwrap().split("\n}").next().unwrap();
        let w_want = ["find_path", "mvs_exchange", "mark_entity_des_removed"];
        for n in w_want {
            assert!(world.contains(&format!("fn {n}(")), "WorldHost 失联：{n}");
        }
        let p_want = ["snap_camera_to_player", "locate_speaker_entity", "ces_camera",
            "set_camera_anchor_target", "exit_application", "snap_camera_after_ros1"];
        for n in p_want {
            assert!(pres.contains(&format!("fn {n}(")), "PresentationHost 失联：{n}");
        }
        assert_eq!(w_want.len() + p_want.len(), 9, "3+6=9：原 8 方法 + CES 状态面归来 1");
        // HostCtx 自身不再承载方法（纯组合别名）
        let alias = src.split("pub trait HostCtx").nth(1).unwrap().split("\n}").next().unwrap();
        assert!(!alias.contains("fn "), "HostCtx 应为纯组合别名，不得自带方法");
    }

    /// 【2026-11-17 S3·③ 红测试】表现面必须「可零实现」：全部方法带默认体。
    /// 判据：PresentationHost 块内每个 `fn` 签名行以 `{` 结尾（无默认体只能写 `;`）。
    /// 反例即事故形态：状态藏进表现面 ⇒ headless 跳过 UI 就跳过状态。
    #[test]
    fn s3b3_presentation_host_all_methods_have_defaults() {
        let src = include_str!("../script.rs");
        let pres = src.split("pub trait PresentationHost").nth(1).unwrap().split("\n}").next().unwrap();
        let mut n = 0;
        for line in pres.lines() {
            let t = line.trim();
            if !t.starts_with("fn ") && !t.contains(" fn ") {
                continue;
            }
            n += 1;
            // 无默认体的 trait 方法以 `;` 收尾；有默认体的以 `{` 或 `{}` 收尾
            assert!(!t.ends_with(';'), "表现面方法缺默认体（headless 无法零实现）：{t}");
        }
        assert_eq!(n, 6, "表现面应有 6 个方法且全部带默认体，实测 {n}");
    }

    /// 【P2 批10 E】盲区完备核（plan:69-71 点名 14 token 逐项登记核——statediff 命名契约层口径）：
    /// 每 token 必须现身于对应域 BLIND_ZONES 常量（dialog5/persist4/menu4/tap1）。
    #[test]
    fn p2b10_blind_zones_cover_all_plan_named() {
        let d = include_str!("dialog.rs");
        let p = include_str!("persist_codec.rs");
        let m = include_str!("item_service.rs");
        let i = include_str!("input_mapper.rs");
        let groups: [(&str, &str, &[&str]); 4] = [
            ("dialog", d, &["typewriterPos", "dialogSpeaker", "dialogAnchor", "dialogTarget", "overlay"]),
            ("persist", p, &["minFloor", "maxFloor", "pendingFloor", "wentDown"]),
            ("menu", m, &["menuCursor", "itemCursor", "recordPage", "difficultyIndex"]),
            ("tap", i, &["autoBattle"]),
        ];
        for (dom, body, toks) in groups {
            for t in toks {
                assert!(body.contains(t), "plan 点名 token 未登记于 {dom} BLIND_ZONES：{t}");
            }
        }
    }

    #[test]
    fn p2b5_blind_zones_cover_plan_named() {
        let s = SCRIPT_BLIND_ZONES.join("\n");
        for name in ["scriptLine", "scriptCursor", "scriptDelay"] {
            assert!(s.contains(name), "script 盲区漏登记：{name}");
        }
        let d = crate::subsystem::dialog::DIALOG_BLIND_ZONES.join("\n");
        for name in ["typewriterPos", "dialogSpeaker", "dialogAnchor", "dialogTarget", "overlay"] {
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
