//! T1 操作级回撤：LIFO delta 栈（touch-native-undo-plan §0/§2）。
//!
//! 回撤单位=**操作**（用户判据："走过去+战斗=2 个操作，按一次回撤战斗，再按一次
//! 回撤位置"）。四型笔：位移（[`MoveStroke`]）/战斗（[`CombatStroke`]）/
//! 拾取开门（[`PickupStroke`]）/对话商店背包消耗（[`SpendStroke`]）。
//!
//! **入栈时机契约**（plan §2，T2 挂点按此喂）：
//! - 位移笔=寻路**整段起终点**；寻路打断=按**实际终点**入栈（`to`=中断后的最终落格：
//!   步在途时计入在途步的落格——那一步必然走完；起点 = 终点的零长段不入栈，STAGE-B-008）；
//! - 战斗笔=**战斗终结才入栈**（[`UndoJournal::begin_combat`] 置挂起、
//!   [`UndoJournal::finish_combat`] 入栈——中途 [`UndoJournal::undo`] 恒
//!   [`UndoOutcome::Blocked`]，journal 无战斗笔）；
//! - 拾取开门/消耗笔=立即入栈。
//!
//! **清栈边界**（plan §0 边界 1，v1 务实）：换层/存档/MOV 脚本事件格触发=清栈。
//! **不可反转副作用不进回撤域**——v1 不做脚本副作用反转（宝箱/陷阱/剧情/开关的
//! 剧情推进不可回撤，触发即清栈，诚实边界留痕）。
//!
//! 保真红线：本模块为**纯新增**（Stage B 加法），不触碰任何推进语义——classic
//! 面 key 行为链/撞击臂/战斗表零改动，金样/oracle 不经过本模块。

// ---------------------------------------------------------------- 逆放目标面

/// 玩家血/金/经验快照（回滚字段面）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerVitals {
    pub hp: i32,
    pub gold: i32,
    pub exp: i32,
}

/// 敌复活还原值（复活=原血+alive）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyRestore {
    /// 敌槽位（entity 表下标语义，T2 挂点定标）
    pub slot: usize,
    /// 敌型（复活后型不变——变形/召唤语义 v1 不入回撤域）
    pub type_id: i32,
    /// 复活原血（【BUG-045②】保留但不消费：复活不写全局属性表，战斗血由下次开战按基础表重算）
    pub hp: i32,
    /// 【BUG-045】开战前的实体整格快照（位置 / 精灵 / 动画 / state=0 / visible）——复活按此还原
    pub entity: crate::entity::Entity,
}

/// 门状态（拾取开门笔的逆放字段面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Locked,
    Open,
}

/// 逆放目标：游戏状态侧的最小还原面（T2/T3 在 Scene 上实现；测试用 mock）。
/// 逆放只写**状态副本**，不改任何推进语义。
pub trait UndoTarget {
    fn set_player_tile(&mut self, tile: (i32, i32));
    fn set_player_vitals(&mut self, vitals: PlayerVitals);
    fn revive_enemy(&mut self, enemy: EnemyRestore);
    fn set_item_count(&mut self, item_id: i32, count: i32);
    fn set_door_state(&mut self, cx: i32, cy: i32, state: DoorState);
}

// ---------------------------------------------------------------- 四型笔（delta 快照）

/// 位移笔：整段起终点（打断=按实际终点入栈，`to`=中断点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveStroke {
    pub from: (i32, i32),
    pub to: (i32, i32),
}

/// 战斗笔：**终结才入栈**（敌灭+玩家血/金/经验/掉落 → 逆=敌复活原血+回滚）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatStroke {
    pub enemy_slot: usize,
    pub enemy_type_id: i32,
    /// 复活原血
    pub enemy_hp_before: i32,
    /// 战前玩家血/金/经验
    pub vitals_before: PlayerVitals,
    /// 战利品/掉落回滚面（item_id, count_before）——掉落新增量按此退回
    pub items_before: Vec<(i32, i32)>,
    /// 【BUG-045】开战前的实体整格快照
    pub enemy_snapshot: crate::entity::Entity,
}

/// 拾取/开门笔：道具计数 delta + 门状态 delta（逆=退回道具+门复锁）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickupStroke {
    /// (item_id, count_before, count_after)
    pub item: Option<(i32, i32, i32)>,
    /// (cx, cy, state_before, state_after)
    pub door: Option<(i32, i32, DoorState, DoorState)>,
}

/// 消耗笔种类（对话/商店/背包使用=各自独立操作笔，plan §0 边界 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendKind {
    Dialogue,
    Shop,
    InventoryUse,
}

/// 消耗笔：消耗型决策（金/道具）→ 逆=回滚（反悔）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendStroke {
    pub kind: SpendKind,
    pub vitals_before: PlayerVitals,
    /// 回滚面（item_id, count_before）
    pub items_before: Vec<(i32, i32)>,
}

/// 操作笔（四型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stroke {
    Move(MoveStroke),
    Combat(CombatStroke),
    Pickup(PickupStroke),
    Spend(SpendStroke),
}

/// 逆放结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoOutcome {
    /// 逆放一笔（携带被撤的笔，供 UI/日志）
    Undone(Stroke),
    /// 战斗进行中——不可撤（战斗终结才入栈）
    Blocked,
    /// 栈空（或清栈后）
    Empty,
}

/// 清栈原因（plan §0 边界 1 三类触发 + 手动复位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearReason {
    /// 换层（changeFloor/LAY/传送）
    FloorChange,
    /// 存档
    Save,
    /// MOV/脚本事件格触发（宝箱/陷阱/剧情/开关）
    ScriptEvent,
    /// 【BUG-045】不可逆副作用：击杀掉落 / 机关生成实体（战斗期间实体表增长）、未记账的消费（炼丹 / 商人 / 道具使用）、
    /// 跨层探针重载实体表（下标重排）——撤销无法诚实复原，触发即清栈。
    Irreversible,
    /// 手动复位（菜单"重新开始"类）
    Restart,
}

// ---------------------------------------------------------------- journal

/// LIFO delta 栈（操作级回撤）。
#[derive(Debug, Default)]
pub struct UndoJournal {
    strokes: Vec<Stroke>,
    combat_pending: bool,
}

impl UndoJournal {
    pub fn new() -> Self {
        Self::default()
    }

    /// 可回撤步数（边界 3：撤销按钮"可回撤 N 步"可视化数据面）。
    pub fn len(&self) -> usize {
        self.strokes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty()
    }

    /// 战斗进行中（T2/中途态查询）。
    pub fn combat_in_progress(&self) -> bool {
        self.combat_pending
    }

    /// 位移笔入栈（寻路整段结束；打断=传实际终点）。
    pub fn push_move(&mut self, stroke: MoveStroke) {
        self.strokes.push(Stroke::Move(stroke));
    }

    /// 战斗开始（T2 挂点）：置挂起——期间 [`Self::undo`] 恒 [`UndoOutcome::Blocked`]。
    pub fn begin_combat(&mut self) {
        self.combat_pending = true;
    }

    /// 战斗终结入栈（T2 挂点契约：**终结才入栈**）。
    pub fn finish_combat(&mut self, stroke: CombatStroke) {
        self.combat_pending = false;
        self.strokes.push(Stroke::Combat(stroke));
    }

    /// 拾取/开门笔入栈（立即）。
    pub fn push_pickup(&mut self, stroke: PickupStroke) {
        self.strokes.push(Stroke::Pickup(stroke));
    }

    /// 对话/商店/背包消耗笔入栈（立即）。
    pub fn push_spend(&mut self, stroke: SpendStroke) {
        self.strokes.push(Stroke::Spend(stroke));
    }

    /// LIFO 逆放一笔（只写状态副本）。
    /// 战斗进行中= [`UndoOutcome::Blocked`]（中途不可撤）；栈空= [`UndoOutcome::Empty`]。
    pub fn undo(&mut self, target: &mut dyn UndoTarget) -> UndoOutcome {
        if self.combat_pending {
            // 战斗终结才入栈：中途不可撤（plan §0 决定 a）
            return UndoOutcome::Blocked;
        }
        let Some(stroke) = self.strokes.pop() else {
            return UndoOutcome::Empty;
        };
        // LIFO 逆放：只写状态副本，不触碰任何推进语义（保真红线）
        match &stroke {
            Stroke::Move(m) => target.set_player_tile(m.from),
            Stroke::Combat(c) => {
                // 逆=敌复活原血+玩家血金经验回滚+掉落退回
                target.revive_enemy(EnemyRestore {
                    slot: c.enemy_slot,
                    type_id: c.enemy_type_id,
                    hp: c.enemy_hp_before,
                    entity: c.enemy_snapshot,
                });
                target.set_player_vitals(c.vitals_before);
                for (item_id, count) in &c.items_before {
                    target.set_item_count(*item_id, *count);
                }
            }
            Stroke::Pickup(p) => {
                // 逆=退回道具+门复锁
                if let Some((item_id, before, _after)) = p.item {
                    target.set_item_count(item_id, before);
                }
                if let Some((cx, cy, before, _after)) = p.door {
                    target.set_door_state(cx, cy, before);
                }
            }
            Stroke::Spend(sp) => {
                // 逆=消耗回滚（金/道具反悔）
                target.set_player_vitals(sp.vitals_before);
                for (item_id, count) in &sp.items_before {
                    target.set_item_count(*item_id, *count);
                }
            }
        }
        UndoOutcome::Undone(stroke)
    }

    /// 清栈钩子：换层/存档/MOV 脚本事件格触发。
    ///
    /// **不可反转副作用不进回撤域**：v1 不做脚本副作用反转——剧情推进/宝箱一次性
    /// 事件等不可逆效果一旦发生，回撤无法诚实复原 ⇒ 触发即清栈（诚实边界，
    /// plan §0 边界 1）。清栈连带取消进行中战斗挂起（不留"永久不可撤"残态）。
    pub fn clear(&mut self, _reason: ClearReason) {
        self.strokes.clear();
        self.combat_pending = false;
    }
}
