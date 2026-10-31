//! 【STAGE-B-008 · 计划 3 T1 · spec §3】功能查询（纯、无副作用）：由 core 状态派生当前情境，
//! 给出工具栏 / 摇杆标注共用的钮集合。情境判定口径见 spec §3.1；钮与键码见 §3.2 情境表
//!（表逐行的键码证据在计划 3 各任务 / 复核中逐臂核对）。
//!
//! classic 按键链不经本模块（只被壳层呈现消费）；查询不改任何状态。

/// 工具栏 / 摇杆标注的功能标识（spec §3.2 表逐行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionId {
    Bag,
    Observe,
    Undo,
    CloudUp,
    CloudDown,
    Menu,
    Stop,
    Cancel,
    Fight,
    ExitObserve,
    PrevPage,
    NextPage,
    ScrollUp,
    ScrollDown,
    Continue,
    Use,
    Back,
    Buy,
    Accept,
    Refuse,
    Close,
    PrevRec,
    NextRec,
    Confirm,
    Save,
    Read,
    Refine,
    PrevItem,
    NextItem,
    Skip,
}

impl FunctionId {
    /// 中文名（壳标注 / 帮助文本用；两呈现读同一查询）。
    pub fn label(&self) -> &'static str {
        match self {
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
            FunctionId::Accept => "接受",
            FunctionId::Refuse => "拒绝",
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
}

/// 按下执行的动作（spec §3.1）：发原版键码，或无键码动作（停 / 取消 / 开战 / 停下后打开 / 撤销）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Key(i32),
    /// 打断路线（`interrupt_route`）
    Stop,
    /// 清怪物选中
    Cancel,
    /// 执行已选中怪物的撞击（`engage_selected`）
    Fight,
    /// 排空路线落 Map wp0 后发键（与排队共用机制，spec §5.2）
    StopThen(i32),
    Undo,
}

/// 一个钮。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlButton {
    pub function: FunctionId,
    pub action: Action,
}

/// 查询结果：左 / 中（0–4 个）/ 右。None / 空 = 该位隐藏（不画、不可点）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Controls {
    pub left: Option<ControlButton>,
    pub mid: Vec<ControlButton>,
    pub right: Option<ControlButton>,
}

impl Controls {
    pub fn hidden() -> Controls {
        Controls::default()
    }

    fn key(function: FunctionId, code: i32) -> ControlButton {
        ControlButton { function, action: Action::Key(code) }
    }
}

use crate::enums::GameMode;
use crate::scene::GameScene;

impl GameScene {
    /// 【STAGE-B-008 · spec §3.1/§3.2】功能查询（纯）。浮层优先（原版浮层分支先于各模式吞键，
    /// a.java:3252-3345）；页面类（菜单 / 存档 / 设置 / 帮助 / 图鉴 / 背包 / 炼丹）归 Task 2。
    pub fn controls(&self) -> Controls {
        if self.transitioning {
            // 换层遮幅 / 转场（spec §3.2"开机 logo / 退场动画 / Loading / 换层遮幅 = 全隐藏"）
            return Controls::hidden();
        }
        if self.overlay_active {
            return self.overlay_controls();
        }
        match self.game_mode {
            GameMode::Map => self.map_controls(),
            GameMode::Dialog => self.dialog_controls(),
            GameMode::Ending => {
                if self.ending_done {
                    Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Continue, -5)) }
                } else {
                    Controls::hidden()
                }
            }
            // ── 页面类（spec §3.2；BootMachine 侧页面归 BootMachine::controls，Task 2b）
            GameMode::InGameMenu => Controls {
                left: Some(Controls::key(FunctionId::Confirm, -6)),
                mid: vec![],
                right: Some(Controls::key(FunctionId::Back, -7)),
            },
            GameMode::SaveSlot => Controls {
                // GameScene 侧 mode 7 = 存档（Boot 侧 mode 8 读档归 BootMachine::controls）
                left: Some(Controls::key(FunctionId::Save, -6)),
                mid: vec![],
                right: Some(Controls::key(FunctionId::Back, -7)),
            },
            GameMode::About => Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Back, -7)) },
            // 帮助（15）：kind0 专臂整页翻 ±lpp——上/下页恒在（spec 行无"可翻时"修饰）
            GameMode::Help => Controls {
                left: None,
                mid: vec![Controls::key(FunctionId::PrevPage, -1), Controls::key(FunctionId::NextPage, -2)],
                right: Some(Controls::key(FunctionId::Back, -7)),
            },
            // 怪物图鉴（5）：−1/−2 = 行滚 ±1 条目（a.java:3740-3745 m_092 顶指针）≠ 整页翻
            //（【复核 T2-R2】spec 表原措辞"上页/下页"已原地更正）
            GameMode::EnemyInfo => Controls {
                left: None,
                mid: vec![Controls::key(FunctionId::PrevItem, -1), Controls::key(FunctionId::NextItem, -2)],
                right: Some(Controls::key(FunctionId::Back, -7)),
            },
            // 【复核 T2-R3】商店（19）：−7 返回菜单（与 Boot 面同）
            GameMode::Shop => Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Back, -7)) },
            GameMode::ItemPanel => Controls {
                left: Some(Controls::key(FunctionId::Use, -6)),
                mid: vec![],
                right: Some(Controls::key(FunctionId::Back, -7)),
            },
            GameMode::Alchemy => Controls {
                left: Some(Controls::key(FunctionId::Refine, -6)),
                mid: vec![Controls::key(FunctionId::PrevItem, -1), Controls::key(FunctionId::NextItem, -2)],
                right: Some(Controls::key(FunctionId::Back, -7)),
            },
            // Shop（mode 19）：spec §3.2 未列行——按"表外即隐藏"处理，Task 2b 复核确认是否补表
            _ => Controls::hidden(),
        }
    }

    /// 地图族（spec §3.2 前 6 行）。点地寻路中归 `dialog_controls`（Dialog dp3）。
    fn map_controls(&self) -> Controls {
        let bag = Controls::key(FunctionId::Bag, -6);
        let menu = Controls::key(FunctionId::Menu, -7);
        match self.walk_phase {
            0 => {
                if self.tap_selected.is_some() {
                    // 已选中怪物：中间 = 开战 · 取消 · 撤销
                    return Controls {
                        left: Some(bag),
                        mid: vec![
                            ControlButton { function: FunctionId::Fight, action: Action::Fight },
                            ControlButton { function: FunctionId::Cancel, action: Action::Cancel },
                            ControlButton { function: FunctionId::Undo, action: Action::Undo },
                        ],
                        right: Some(menu),
                    };
                }
                // 地图空闲：观察 · 撤销 ·（邻梯时）筋斗云↑/↓
                let mut mid = vec![
                    Controls::key(FunctionId::Observe, -5),
                    ControlButton { function: FunctionId::Undo, action: Action::Undo },
                ];
                // 49/55 门控 = stair_hint（f_bool_13，a.java:5297/5303）∧ 层门实体在（型 7 上 / 型 8 下）
                if self.stair_hint {
                    if self.find_floor_gate_entity(true).is_some() {
                        mid.push(Controls::key(FunctionId::CloudUp, 49));
                    }
                    if self.find_floor_gate_entity(false).is_some() {
                        mid.push(Controls::key(FunctionId::CloudDown, 55));
                    }
                }
                Controls { left: Some(bag), mid, right: Some(menu) }
            }
            1 => Controls { left: Some(bag), mid: vec![], right: Some(menu) }, // 手动行走中
            2 => {
                // 观察模式：未锁定 = 平移期（中 = 退出观察）；已锁定 = 回锁期（中隐藏，spec §3.2）
                let mid = if self.view_locked {
                    vec![]
                } else {
                    vec![Controls::key(FunctionId::ExitObserve, -5)]
                };
                Controls { left: Some(bag), mid, right: Some(menu) }
            }
            5 => Controls { left: Some(bag), mid: vec![], right: Some(menu) }, // 战斗中（背包/菜单可开，spec §3.2）
            _ => Controls::hidden(),
        }
    }

    /// 对话族：点地寻路占用 dp3 = 寻路情境（停下后打开）；dp1/dp6 = 翻页对话；其余相位全隐藏。
    fn dialog_controls(&self) -> Controls {
        if self.walkto_owns_dp3() {
            // 点地寻路中：背包 / 菜单 = 停下后打开（与排队共用机制，Task 2 落执行）；中间 = 停下
            return Controls {
                left: Some(ControlButton { function: FunctionId::Bag, action: Action::StopThen(-6) }),
                mid: vec![ControlButton { function: FunctionId::Stop, action: Action::Stop }],
                right: Some(ControlButton { function: FunctionId::Menu, action: Action::StopThen(-7) }),
            };
        }
        if self.dialog_phase == 1 || self.dialog_phase == 6 {
            let mut mid = vec![];
            if self.overlay_page_top > 0 {
                mid.push(Controls::key(FunctionId::PrevPage, -1));
            }
            if self.overlay_page_bottom < self.overlay_total_lines {
                mid.push(Controls::key(FunctionId::NextPage, -2));
            }
            return Controls {
                left: None,
                mid,
                right: Some(Controls::key(FunctionId::Continue, -5)),
            };
        }
        Controls::hidden()
    }

    /// 浮层族（spec §3.2 kind 0–5 行；键码 = 各 tick 臂实际接受键）。
    fn overlay_controls(&self) -> Controls {
        match self.overlay_kind {
            0 | 1 => {
                // 【复核 T1-R2】Help（mode 15）/ IconView（mode 17）进页即带 kind0 浮层，但走页面专臂
                //（−7 关层、−1/−2 整页翻、其余吞）——属页面类（Task 2），此处不得给出 kind0 通用钮集。
                if matches!(self.game_mode, GameMode::Help | GameMode::IconView) {
                    return Controls::hidden();
                }
                // kind 1 吞键子类（scene.rs run 浮层臂，a.java case1 441）：有图标 ∧ right ∉ {2,3} ⇒ 全吞键
                if self.overlay_kind == 1
                    && (self.overlay_left_icon != 0 || self.overlay_right_icon != 0)
                    && self.overlay_right_icon != 2
                    && self.overlay_right_icon != 3
                {
                    return Controls::hidden();
                }
                let mut mid = vec![];
                if self.overlay_page_top > 0 {
                    mid.push(Controls::key(FunctionId::ScrollUp, -1));
                }
                if self.overlay_page_bottom < self.overlay_total_lines {
                    mid.push(Controls::key(FunctionId::ScrollDown, -2));
                }
                Controls { left: None, mid, right: Some(Controls::key(FunctionId::Close, -5)) }
            }
            3 => {
                // 道具说明子类（15/23/24/25，overlay_kind_item）只有关闭；其余 = 使用确认
                if matches!(self.overlay_kind_item, 15 | 23 | 24 | 25) {
                    Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Close, -7)) }
                } else {
                    Controls {
                        left: Some(Controls::key(FunctionId::Use, -6)),
                        mid: vec![],
                        right: Some(Controls::key(FunctionId::Back, -7)),
                    }
                }
            }
            4 => {
                // 商人框：src 0 = 型 77 文本源（spec §2.1）；flags = param 高字节
                let (src, flags) = if self.merchant_entity >= 0 {
                    let ent = &self.entities.items[self.merchant_entity as usize];
                    let src = if ent.type_ == 77 { 0 } else { 1 };
                    (src, (ent.param as i32) >> 8)
                } else {
                    (1, 0)
                };
                if src == 1 && flags & 2 != 0 {
                    // 商人报价：买 / 不买
                    Controls {
                        left: Some(Controls::key(FunctionId::Buy, -6)),
                        mid: vec![],
                        right: Some(Controls::key(FunctionId::Refuse, -7)),
                    }
                } else if src == 0 && flags & 2 != 0 {
                    // 赠予 / 报价（点空白不代按，计划 2 T9）：接受 / 拒绝
                    Controls {
                        left: Some(Controls::key(FunctionId::Accept, -5)),
                        mid: vec![],
                        right: Some(Controls::key(FunctionId::Refuse, -7)),
                    }
                } else {
                    // 告示：关闭
                    Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Close, -7)) }
                }
            }
            5 => {
                // 记录浏览：上一条 · 下一条 · 上滚 · 下滚 / 关闭
                let mut mid = vec![
                    Controls::key(FunctionId::PrevRec, -3),
                    Controls::key(FunctionId::NextRec, -4),
                ];
                if self.overlay_page_top > 0 {
                    mid.push(Controls::key(FunctionId::ScrollUp, -1));
                }
                if self.overlay_page_bottom < self.overlay_total_lines {
                    mid.push(Controls::key(FunctionId::ScrollDown, -2));
                }
                Controls { left: None, mid, right: Some(Controls::key(FunctionId::Close, -7)) }
            }
            _ => Controls::hidden(),
        }
    }
}


impl crate::paint::BootMachine {
    /// 【计划 3 T2b · spec §3.2】BootMachine 侧页面行：读档（mode 8）读取 −5 / 返回 −7；设置（16）
    /// 返回 −7；帮助（15）上 / 下页 −1/−2（spec 无"可翻时"修饰，恒在）；关于（17，IconView）上 / 下页
    /// 可翻时；片头（intro）跳过 −7；标题整条隐藏（自有三区点按 + 横滑）；logo / 退场 / Loading 全隐藏。
    pub fn controls(&self) -> Controls {
        use crate::enums::GameMode as G;
        // 帮助 / 关于的 kind0 浮层：整页翻（±lpp）+ 关层 −7（页面专臂，a.java case15/17）
        if self.overlay_active && matches!(self.mode, G::Help | G::IconView) {
            let (top, total) = (self.overlay.page_top(), self.overlay.total_lines());
            let bottom = self.overlay.page_bottom();
            let always = self.mode == G::Help; // 帮助行无"可翻时"修饰 ⇒ 恒在；关于行可翻时（含 bottom<total，同 T1-R1 修正口径）
            let mut mid = vec![];
            if always || top > 0 {
                mid.push(Controls::key(FunctionId::PrevPage, -1));
            }
            if always || bottom < total {
                mid.push(Controls::key(FunctionId::NextPage, -2));
            }
            return Controls { left: None, mid, right: Some(Controls::key(FunctionId::Back, -7)) };
        }
        match self.mode {
            // 【复核 T2-R1】Boot 读档列表 = gameMode 8（EquipDetail 变体；a.java:3400/3836-3856）——
            // mode 7（SaveSlot）只在 GameScene 侧出现（存档页）
            G::EquipDetail => Controls {
                left: Some(Controls::key(FunctionId::Read, -5)),
                mid: vec![],
                right: Some(Controls::key(FunctionId::Back, -7)),
            },
            // 【复核 T2-R3】商店（19）：−7 返回菜单（a.java:4197-4200）——spec 表原缺行，已原地补注
            G::Shop => Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Back, -7)) },
            G::About => Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Back, -7)) },
            G::IntroCutscene => Controls { left: None, mid: vec![], right: Some(Controls::key(FunctionId::Skip, -7)) },
            _ => Controls::hidden(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::scene_test_support::{scene, ROW};

    fn btn(f: FunctionId, a: Action) -> ControlButton {
        ControlButton { function: f, action: a }
    }

    /// 【spec §3.2】地图空闲：背包 | 观察 · 撤销（无梯）| 菜单。
    #[test]
    fn t1_map_idle_buttons() {
        let mut sc = scene((2, ROW), &[]);
        let c = sc.controls();
        assert_eq!(c.left, Some(btn(FunctionId::Bag, Action::Key(-6))));
        assert_eq!(c.right, Some(btn(FunctionId::Menu, Action::Key(-7))));
        assert_eq!(
            c.mid,
            vec![btn(FunctionId::Observe, Action::Key(-5)), btn(FunctionId::Undo, Action::Undo)]
        );
    }

    /// 【spec §3.2】邻梯（stair_hint 亮 ∧ 型 7/8 在）⇒ 筋斗云↑/↓ 入中列；单侧门只出单钮。
    #[test]
    fn t1_map_idle_cloud_buttons_gated() {
        let mut sc = scene((2, ROW), &[]);
        let up = sc.entities.count;
        crate::entity::spawn(&mut sc.entities, &mut sc.grid, 7, 3 << 5, (ROW - 1) << 5, 10, &sc.width_table, None);
        let _ = up;
        sc.stair_hint = true;
        let c = sc.controls();
        assert_eq!(
            c.mid,
            vec![
                btn(FunctionId::Observe, Action::Key(-5)),
                btn(FunctionId::Undo, Action::Undo),
                btn(FunctionId::CloudUp, Action::Key(49)),
            ],
            "只有上行门 ⇒ 只出筋斗云↑"
        );
        // 门在但 stair_hint（f_bool_13）未亮 ⇒ 云仍不出（门控变异被杀）
        sc.stair_hint = false;
        assert_eq!(
            sc.controls().mid,
            vec![btn(FunctionId::Observe, Action::Key(-5)), btn(FunctionId::Undo, Action::Undo)],
            "门在而 hint 未亮 ⇒ 无云"
        );
        // 双门（型 7 上 + 型 8 下）⇒ 云↑/云↓ 都在
        crate::entity::spawn(&mut sc.entities, &mut sc.grid, 8, 4 << 5, (ROW - 1) << 5, 20, &sc.width_table, None);
        sc.stair_hint = true;
        assert_eq!(
            sc.controls().mid,
            vec![
                btn(FunctionId::Observe, Action::Key(-5)),
                btn(FunctionId::Undo, Action::Undo),
                btn(FunctionId::CloudUp, Action::Key(49)),
                btn(FunctionId::CloudDown, Action::Key(55)),
            ],
            "双门 ⇒ 云↑/云↓"
        );
    }

    /// 【spec §3.2】手动行走（wp1）/ 战斗（wp5）：左右大键在、中间空。
    #[test]
    fn t1_map_walking_and_battle() {
        let mut sc = scene((2, ROW), &[]);
        sc.walk_phase = 1;
        assert_eq!(sc.controls().mid, Vec::new(), "手动行走中间隐藏");
        assert_eq!(sc.controls().left.map(|b| b.function), Some(FunctionId::Bag));
        sc.walk_phase = 5;
        assert_eq!(sc.controls().mid, Vec::new(), "战斗中间隐藏");
        assert_eq!(sc.controls().right.map(|b| b.action), Some(Action::Key(-7)), "战斗中菜单可开");
    }

    /// 【spec §3.2】已选中怪物：开战 · 取消 · 撤销（无键码动作）。
    #[test]
    fn t1_map_selected_monster() {
        let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
        let e = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        sc.tap_selected = Some(crate::tap_walk::SelectedEnemy { entity: e, type_: 41, cell: (8, ROW), floor: sc.current_floor });
        let c = sc.controls();
        assert_eq!(
            c.mid,
            vec![
                btn(FunctionId::Fight, Action::Fight),
                btn(FunctionId::Cancel, Action::Cancel),
                btn(FunctionId::Undo, Action::Undo),
            ]
        );
    }

    /// 【spec §3.2】观察模式：未锁定 = 退出观察；回锁期（view_locked）中间隐藏。
    #[test]
    fn t1_observe_and_relock() {
        let mut sc = scene((2, ROW), &[]);
        sc.walk_phase = 2;
        sc.view_locked = false;
        assert_eq!(sc.controls().mid, vec![btn(FunctionId::ExitObserve, Action::Key(-5))]);
        sc.view_locked = true; // 回锁期
        assert_eq!(sc.controls().mid, Vec::new(), "回锁期中间隐藏");
        assert_eq!(sc.controls().left.map(|b| b.function), Some(FunctionId::Bag), "回锁期左右照旧");
    }

    /// 【spec §3.2】点地寻路中：背包/菜单 = 停下后打开、中间 = 停下；对话 dp1 = 翻页/继续；其余相位隐藏。
    #[test]
    fn t1_walkto_and_dialog_paging() {
        let mut sc = scene((2, ROW), &[]);
        let _ = sc.tap_cell(8, ROW); // 开寻路（Walking/Bumping）
        assert_eq!(sc.game_mode, crate::enums::GameMode::Dialog);
        let c = sc.controls();
        assert_eq!(c.left, Some(btn(FunctionId::Bag, Action::StopThen(-6))), "背包 = 停下后打开");
        assert_eq!(c.right, Some(btn(FunctionId::Menu, Action::StopThen(-7))), "菜单 = 停下后打开");
        assert_eq!(c.mid, vec![btn(FunctionId::Stop, Action::Stop)]);
        // 打断回 Map 后设对话翻页相位（直接构造 dp1 态查纯函数）
        let mut sc = scene((2, ROW), &[]);
        sc.game_mode = GameMode::Dialog;
        sc.dialog_phase = 1;
        sc.overlay_page_top = 40;
        sc.overlay_page_bottom = 49;
        sc.overlay_total_lines = 120;
        let c = sc.controls();
        assert_eq!(c.mid, vec![btn(FunctionId::PrevPage, Action::Key(-1)), btn(FunctionId::NextPage, Action::Key(-2))]);
        assert_eq!(c.right, Some(btn(FunctionId::Continue, Action::Key(-5))));
        sc.overlay_page_top = 0;
        sc.overlay_page_bottom = 9;
        assert_eq!(sc.controls().mid, vec![btn(FunctionId::NextPage, Action::Key(-2))], "顶页无上页");
        // 【复核 T1-R1】恰剩一行在下方（bottom = total-1）⇒ 下页仍须在场
        sc.overlay_page_top = 110;
        sc.overlay_page_bottom = 119;
        assert_eq!(sc.controls().mid, vec![btn(FunctionId::PrevPage, Action::Key(-1)), btn(FunctionId::NextPage, Action::Key(-2))], "bottom=total-1 ⇒ 下页在场");
        sc.overlay_page_bottom = 120;
        assert_eq!(sc.controls().mid, vec![btn(FunctionId::PrevPage, Action::Key(-1))], "bottom=total ⇒ 末页无下页");
        // dp6 同 dp1（带目标文本相位）
        sc.dialog_phase = 6;
        assert_eq!(sc.controls().right, Some(btn(FunctionId::Continue, Action::Key(-5))));
        sc.dialog_phase = 3;
        assert_eq!(sc.controls(), Controls::hidden(), "其他相位全隐藏");
    }

    /// 【spec §3.2】提示 / 拾取框（kind 0/1）：上滚 / 下滚（可滚时）+ 关闭；kind 1 全吞键子类整条隐藏。
    #[test]
    fn t1_overlay_kind0_and_kind1() {
        let mut sc = scene((2, ROW), &[]);
        sc.overlay_active = true;
        sc.overlay_kind = 0;
        sc.overlay_total_lines = 5;
        sc.overlay_lines_per_page = 4;
        sc.overlay_page_top = 0;
        sc.overlay_page_bottom = 4; // = min(top+lpp, total)：恰剩 1 行在下方 ⇒ 可下滚（复核 T1-R1 反例）
        let c = sc.controls();
        assert_eq!(c.mid, vec![btn(FunctionId::ScrollDown, Action::Key(-2))], "top=0 无上滚；bottom=4<total=5 ⇒ 下滚在（恰剩 1 行）");
        assert_eq!(c.right, Some(btn(FunctionId::Close, Action::Key(-5))));
        // kind 1 吞键子类：有图标 ∧ right ∉ {2,3}
        sc.overlay_kind = 1;
        sc.overlay_left_icon = 3;
        sc.overlay_right_icon = 0; // 有图标 ∧ right ∉ {2,3} ⇒ 全吞
        assert_eq!(sc.controls(), Controls::hidden(), "kind1 吞键子类整条隐藏");
        // right ∈ {2,3} 支路：−2/−1 滚、其余（含 −5）关层 ⇒ 滚钮 + 关闭
        sc.overlay_right_icon = 2;
        let c = sc.controls();
        assert_eq!(c.mid, vec![btn(FunctionId::ScrollDown, Action::Key(-2))], "right=2：滚钮照常（top=0 ⇒ 仅下滚）");
        assert_eq!(c.right, Some(btn(FunctionId::Close, Action::Key(-5))), "right=2：−5 关层在");
        // 【复核 T1-R2】Help / IconView 的 kind0 浮层走页面专臂（−7 关层）：此处不落 kind0 通用钮集
        sc.overlay_kind = 0;
        sc.game_mode = GameMode::Help;
        assert_eq!(sc.controls(), Controls::hidden(), "Help 的 kind0 浮层 = 页面类（Task 2）");
        sc.game_mode = GameMode::IconView;
        assert_eq!(sc.controls(), Controls::hidden(), "IconView 的 kind0 浮层 = 页面类（Task 2）");
    }

    /// 【spec §3.2】道具使用确认（Use/Back）与道具说明子类（Close）；商人报价 / 赠予 / 告示三子类。
    #[test]
    fn t1_overlay_kind3_and_kind4() {
        let mut sc = scene((2, ROW), &[]);
        sc.overlay_active = true;
        sc.overlay_kind = 3;
        sc.overlay_kind_item = 16; // 非说明类 ⇒ 使用确认
        let c = sc.controls();
        assert_eq!(c.left, Some(btn(FunctionId::Use, Action::Key(-6))));
        assert_eq!(c.right, Some(btn(FunctionId::Back, Action::Key(-7))));
        sc.overlay_kind_item = 15; // 说明类
        let c = sc.controls();
        assert_eq!(c.right, Some(btn(FunctionId::Close, Action::Key(-7))));
        assert_eq!(c.left, None, "说明类无左钮");

        // kind 4 族：型 77 src0
        let i77 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 77);
        if i77.is_none() {
            // 夹具无 77：用商人指向直接构造（spawn 一个 78 当 src1、手工置 merchant 指向再改型不可行——
            // 用 spawn 77）
            crate::entity::spawn(&mut sc.entities, &mut sc.grid, 77, 5 << 5, ROW << 5, 0, &sc.width_table, None);
        }
        let i77 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 77).unwrap();
        sc.overlay_kind = 4;
        sc.merchant_entity = i77 as i32;
        sc.entities.items[i77].param = (2 << 8) | 1;
        let c = sc.controls();
        assert_eq!(c.left, Some(btn(FunctionId::Accept, Action::Key(-5))), "型 77 报价 = 接受");
        assert_eq!(c.right, Some(btn(FunctionId::Refuse, Action::Key(-7))));
        sc.entities.items[i77].param = 1; // 告示
        let c = sc.controls();
        assert_eq!(c.right, Some(btn(FunctionId::Close, Action::Key(-7))), "告示 = 关闭");
        assert_eq!(c.left, None);
        // src 1（型 78）报价
        crate::entity::spawn(&mut sc.entities, &mut sc.grid, 78, 6 << 5, ROW << 5, (2 << 8) | 1, &sc.width_table, None);
        let i78 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 78).unwrap();
        sc.merchant_entity = i78 as i32;
        let c = sc.controls();
        assert_eq!(c.left, Some(btn(FunctionId::Buy, Action::Key(-6))), "型 78 报价 = 买");
        assert_eq!(c.right, Some(btn(FunctionId::Refuse, Action::Key(-7))));
    }

    /// 【计划 3 T2 · spec §3.2】页面行（GameScene 面）：菜单/存档/设置/帮助/图鉴/背包/炼丹；遮幅全隐藏。
    #[test]
    fn t2_page_rows() {
        let mut sc = scene((2, ROW), &[]);
        for (mode, l, m, r) in [
            (GameMode::InGameMenu, Some(FunctionId::Confirm), 0usize, Some(FunctionId::Back)),
            (GameMode::SaveSlot, Some(FunctionId::Save), 0, Some(FunctionId::Back)),
            (GameMode::About, None, 0, Some(FunctionId::Back)),
            (GameMode::Shop, None, 0, Some(FunctionId::Back)),
            (GameMode::ItemPanel, Some(FunctionId::Use), 0, Some(FunctionId::Back)),
            (GameMode::Alchemy, Some(FunctionId::Refine), 2, Some(FunctionId::Back)),
        ] {
            sc.game_mode = mode;
            let c = sc.controls();
            assert_eq!(c.left.map(|b| b.function), l, "{mode:?} 左");
            assert_eq!(c.mid.len(), m, "{mode:?} 中");
            assert_eq!(c.mid.iter().map(|b| b.function).collect::<Vec<_>>(), if mode == GameMode::Alchemy {
                vec![FunctionId::PrevItem, FunctionId::NextItem]
            } else if m == 2 {
                vec![FunctionId::PrevPage, FunctionId::NextPage]
            } else { vec![] }, "{mode:?} 中列");
            assert_eq!(c.right.map(|b| b.function), r, "{mode:?} 右");
            assert!(c.right.map(|b| b.action) == Some(Action::Key(-7)), "{mode:?} 右 = −7");
        }
        // 帮助 = 上/下页（整页翻）；图鉴 = 上一条/下一条（行滚 ±1 条目，复核 T2-R2）
        sc.game_mode = GameMode::Help;
        assert_eq!(
            sc.controls().mid.iter().map(|b| b.function).collect::<Vec<_>>(),
            vec![FunctionId::PrevPage, FunctionId::NextPage]
        );
        sc.game_mode = GameMode::EnemyInfo;
        assert_eq!(
            sc.controls().mid.iter().map(|b| b.function).collect::<Vec<_>>(),
            vec![FunctionId::PrevItem, FunctionId::NextItem],
            "图鉴 = 行滚条目，非整页翻"
        );
        sc.game_mode = GameMode::Map;
        sc.transitioning = true;
        assert_eq!(sc.controls(), Controls::hidden(), "换层遮幅全隐藏");
    }

    /// 【计划 3 T2 · spec §3.1/§3.2】Action 执行器：StopThen 排空后开背包；Stop/Cancel/Fight/Key。
    #[test]
    fn t2_perform_action_chain() {
        use crate::tap_walk::TapResult;
        // StopThen：寻路中按"背包"= 打断 + 排队；排空落 Map wp0 后键值 −6 → 背包打开
        let mut sc = scene((1, ROW), &[]);
        assert_eq!(sc.tap_cell(10, ROW), TapResult::Walking);
        let fc = walk_until_x_(&mut sc, 4);
        assert!(sc.perform_action(Action::StopThen(-6)), "受理");
        assert!(matches!(sc.controls().left.map(|b| b.action), Some(Action::StopThen(-6))), "情境仍寻路中");
        settle_(&mut sc, fc, 200);
        assert_eq!(sc.game_mode, GameMode::ItemPanel, "排空后 −6 打开背包");
        // Key：背包页"返回"= −7 边沿键值
        assert!(sc.perform_action(Action::Key(-7)));
        assert_eq!(sc.key_value, -7, "键值置上（无 keyHeld 锁存）");
        assert_eq!(sc.key_held, 0, "无 keyHeld 锁存");
        settle_(&mut sc, 0, 60);
        assert_eq!(sc.game_mode, GameMode::Map, "−7 返回地图");
        // Stop：寻路中停下
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x_(&mut sc, 3);
        assert!(sc.perform_action(Action::Stop));
        assert!(sc.tap_pending.is_none() && sc.tap_target.is_none());
        settle_(&mut sc, fc, 120);
        assert_eq!(sc.game_mode, GameMode::Map, "排空后停在地图");
        // Cancel / Fight：选中怪
        let mut sc = scene((2, ROW), &[(41, 8, ROW)]);
        strong_(&mut sc);
        let _ = sc.tap_cell(8, ROW); // Selected
        assert!(sc.tap_selected.is_some());
        assert!(sc.perform_action(Action::Cancel));
        assert!(sc.tap_selected.is_none(), "取消清选中");
        let _ = sc.tap_cell(8, ROW); // 再选中
        assert!(sc.perform_action(Action::Fight), "开战受理");
        assert!(sc.tap_selected.is_none(), "开战即清选中");
        settle_(&mut sc, 0, 160);
        let i41 = (0..sc.entities.count).find(|&i| sc.entities.items[i].type_ == 41).unwrap();
        assert!(sc.entities.items[i41].removed || sc.entities.items[i41].state == 1, "Fight 端到端：怪被击杀");
        // StopThen 清目标标记 + 链仍达（复核 T2-R5）
        let mut sc = scene((1, ROW), &[]);
        let _ = sc.tap_cell(10, ROW);
        let fc = walk_until_x_(&mut sc, 4);
        let _ = sc.perform_action(Action::StopThen(-6));
        assert!(sc.tap_target.is_none(), "StopThen 打断 ⇒ 目标标记消失");
        settle_(&mut sc, fc, 200);
        assert_eq!(sc.game_mode, GameMode::ItemPanel, "StopThen 链仍达（开背包）");
        // Undo 恒 false（壳职责）
        assert!(!sc.perform_action(Action::Undo));
    }

    fn walk_until_x_(sc: &mut GameScene, x: i32) -> i64 {
        let wt = sc.width_table.clone();
        let mut fc = 0;
        while sc.player_cell_x < x {
            sc.tick(fc, &wt);
            fc += 1;
        }
        fc
    }

    fn settle_(sc: &mut GameScene, from: i64, n: i64) {
        let wt = sc.width_table.clone();
        for fc in from..from + n {
            sc.tick(fc, &wt);
        }
    }

    fn strong_(sc: &mut GameScene) {
        sc.player.atk = 100_000;
        sc.player.def = 100_000;
        sc.player.hp = 100_000;
    }

    /// 【spec §3.2】记录浏览（kind 5）：四中钮 + 关闭；结局 ending_done 前后。
    #[test]
    fn t1_overlay_kind5_and_ending() {
        let mut sc = scene((2, ROW), &[]);
        sc.overlay_active = true;
        sc.overlay_kind = 5;
        sc.overlay_total_lines = 3;
        sc.overlay_page_top = 1;
        sc.overlay_page_bottom = 1;
        let c = sc.controls();
        assert_eq!(
            c.mid,
            vec![
                btn(FunctionId::PrevRec, Action::Key(-3)),
                btn(FunctionId::NextRec, Action::Key(-4)),
                btn(FunctionId::ScrollUp, Action::Key(-1)),
                btn(FunctionId::ScrollDown, Action::Key(-2)),
            ]
        );
        assert_eq!(c.right, Some(btn(FunctionId::Close, Action::Key(-7))));
        // 结局
        sc.overlay_active = false;
        sc.game_mode = GameMode::Ending;
        sc.ending_done = false;
        assert_eq!(sc.controls(), Controls::hidden());
        sc.ending_done = true;
        assert_eq!(sc.controls().right, Some(btn(FunctionId::Continue, Action::Key(-5))));
    }
}
