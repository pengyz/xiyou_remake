//! task-15 V2 item/effect 域（v3 §1：activate_item 表驱动仿 RestoreAction）——
//! 新增道具不改 GameScene 主体（§5.1 git diff 断言）：加道具=ITEM_EFFECTS 加行。

use super::*;

/// 道具效果动作（a.java:7786-7886 activateItem 消费面逐 case 抽象——行为面
/// 零差：执行序=表序）。
pub enum ItemAction {
    /// 打开面板（EnemyInfo/RecordBrowser 等——mode 分派+面板钩）
    OpenPanel { panel: ItemPanel, page: i32 },
    /// 四邻格 gates：移除目标型实体（未命中提示，不消耗）
    RemoveAdjacent { target: i32, msg: &'static str },
    /// 全场移除某型实体 + 重建小地图 + 回 Map
    RemoveAllType { target: i32 },
    /// (atk+def)*mul/div 加血 + 提示 + 消耗
    Heal { mul: i32, div: i32, msg_fmt: &'static str },
    /// 四邻 killAdjacentAt + 回 Map（a.java:7876-7886）
    KillAdjacent4,
    /// 对角瞬移（40 层禁用；镜像格 interact 放行）
    MirrorTeleport,
    /// 换层 +1（消耗=change_floor 返回；keyHeld=0+回 Map）
    ChangeFloorUp,
    ChangeFloorDown,
}

/// 面板分派目标。
pub enum ItemPanel {
    EnemyInfo,
    RecordBrowser,
}

/// 道具效果表（a.java activateItem 消费面逐 case 配置化——**新增道具=加行**）。
pub static ITEM_EFFECTS: &[(i32, &[ItemAction])] = &[
    (13, &[ItemAction::OpenPanel { panel: ItemPanel::EnemyInfo, page: 0 }]),
    (14, &[ItemAction::OpenPanel { panel: ItemPanel::RecordBrowser, page: 0 }]),
    (16, &[ItemAction::RemoveAdjacent { target: 10, msg: "你必须面对三昧真火再使用它。" }]),
    (17, &[ItemAction::RemoveAdjacent { target: 11, msg: "你必须面对一堵墙使用" }]),
    (18, &[ItemAction::RemoveAllType { target: 11 }]),
    (85, &[ItemAction::RemoveAllType { target: 1 }]),
    (19, &[ItemAction::Heal { mul: 74, div: 10, msg_fmt: "增加了{}血量" }]),
    (86, &[ItemAction::KillAdjacent4]),
    (20, &[ItemAction::MirrorTeleport]),
    (21, &[ItemAction::ChangeFloorUp]),
    (22, &[ItemAction::ChangeFloorDown]),
];

/// 表查询：道具型 t 的动作序列。
pub fn item_actions(t: i32) -> Option<&'static [ItemAction]> {
    ITEM_EFFECTS.iter().find(|(id, _)| *id == t).map(|(_, a)| *a)
}

/// 【双归域·item/effect 编排面（coordination 注记归因）】动作执行=跨域副作用
/// （mode 分派/实体表/小地图/相机/换层）——不可迁面显式留置；**表驱动本体**
/// 保证新增道具零改此处（C1 §5.1 git diff 断言）。
/// 【终章簇3·item-effect 域直写接口】（alchemy 状态跨文件写收敛）。
impl GameScene {
    pub(super) fn set_alchemy_price(&mut self, v: i32) { self.alchemy_price = v; }
    pub(super) fn set_alchemy_upgrade_count(&mut self, v: i32) { self.alchemy_upgrade_count = v; }
    pub(super) fn inc_alchemy_upgrade_count(&mut self) { self.alchemy_upgrade_count += 1; }
    /// 执行一个道具动作（a.java 语义逐字——执行序=表序）。
    pub(super) fn execute_item_action(&mut self, a: &ItemAction) -> bool {
        match a {
            ItemAction::OpenPanel { panel, page } => {
                match panel {
                    ItemPanel::EnemyInfo => {
                        self.transition_to(GameMode::EnemyInfo);
                        self.enter_menu5();
                    }
                    ItemPanel::RecordBrowser => {
                        self.transition_to(GameMode::RecordBrowser);
                        self.open_record_page(*page);
                        self.menu.softkeys.1 = 3;
                    }
                }
                false
            }
            ItemAction::RemoveAdjacent { target, msg } => {
                if self.remove_adjacent_entity_of_type(*target) {
                    self.transition_to(GameMode::Map);
                    true
                } else {
                    self.show_overlay_message(0, msg, 0, 0);
                    false
                }
            }
            ItemAction::RemoveAllType { target } => {
                let mut e = self.entities.count;
                while e > 0 {
                    e -= 1;
                    if self.entities.items[e].type_ == *target
                        && entity::remove(&mut self.entities, e)
                    {
                        self.build_minimap();
                    }
                }
                self.transition_to(GameMode::Map);
                true
            }
            ItemAction::Heal { mul, div, msg_fmt } => {
                let delta = (self.player.atk + self.player.def) * mul / div;
                self.apply_hp_delta(delta);
                self.show_overlay_message(0, &msg_fmt.replace("{}", &delta.to_string()), 0, 0);
                true
            }
            ItemAction::KillAdjacent4 => {
                let cx = self.view.player_px >> 5;
                let cy = self.view.player_py >> 5;
                self.kill_adjacent_at(cx, cy - 1);
                self.kill_adjacent_at(cx, cy + 1);
                self.kill_adjacent_at(cx - 1, cy);
                self.kill_adjacent_at(cx + 1, cy);
                self.transition_to(GameMode::Map);
                true
            }
            ItemAction::MirrorTeleport => {
                if self.persist.floor == 40 {
                    self.show_overlay_message(0, "本层不能直接瞬移。", 0, 0);
                    false
                } else {
                    let tx = self.view.wide - 1 - (self.view.player_px >> 5);
                    let ty = self.view.high - 1 - (self.view.player_py >> 5);
                    let pass = self.interact_with_cell(tx, ty);
                    if pass {
                        self.place_player_at_cell(tx, ty);
                        self.view.set_camera(
                            ((self.view.view_w - 32) >> 1) - self.view.player_px,
                            ((self.view.view_h - 32) >> 1) - self.view.player_py,
                        );
                        self.apply_step_cell_effects();
                        self.transition_to(GameMode::Map);
                        true
                    } else {
                        if !self.dialog.overlay_active {
                            self.show_overlay_message(0, "无法移动到该位置", 0, 0);
                        }
                        false
                    }
                }
            }
            ItemAction::ChangeFloorUp => {
                let consumed = self.change_floor(self.persist.floor + 1, false, false);
                self.set_key_held(0);
                self.transition_to(GameMode::Map);
                consumed
            }
            ItemAction::ChangeFloorDown => {
                let consumed = self.change_floor(self.persist.floor - 1, true, false);
                self.set_key_held(0);
                self.transition_to(GameMode::Map);
                consumed
            }
        }
    }

    /// 【C1 达成面】activate_item=表查询+逐动作执行——**新增道具不改此主体**
    /// （ITEM_EFFECTS 加行即可）；未知型=不消耗（原 _ => false 语义）。
    pub(super) fn activate_item(&mut self, t: i32) -> bool {
        let Some(actions) = item_actions(t) else {
            return false;
        };
        let mut consumed = false;
        for a in actions {
            consumed = self.execute_item_action(a);
        }
        consumed
    }
}
