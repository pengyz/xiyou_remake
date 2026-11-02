//! task-15 V1 文件归位（v3 §0 真实剩余 1/E1）：script_ops=enum+22 op impls+
//! parse/execute 三处汇 1（move-only）——16 op 冻结面（a.java:8220-8568）字节不动。
use super::*;

pub(crate) enum ScriptOp {
    /// TAK_a_b（a.java:8257-8263）
    Tak { start: i32, end: i32, cursor: usize },
    /// LAY_x（a.java:8355-8362）
    Lay { floor: i32, cursor: usize },
    /// GIN_a_b（a.java:8424-8437）
    Gin { kind: i32, value: i32, cursor: usize },
    /// RES_x（a.java:8470-8490）
    Res { cursor: usize },
    /// ADD_a_b_c_d（a.java:8438-8468）
    Add { floor: i32, ty: i32, cx: i32, cy: i32, cursor: usize },
    /// SEE_a_b_c_d_e（a.java:8487-8510）
    See { tx: i32, ty: i32, page: i32, page_end: i32, flag: i32, cursor: usize },
    /// GUT_n（a.java:8308-8311）——cursor 由 loadLevelScript 覆写为 0
    Gut { target: i32, cursor: usize },
    /// MOV（a.java:8274-8309）：var55>0 ⇒ cell=Some(五参实体路线)；否则三参玩家行
    Mov { target_type: i32, cell: Option<(i32, i32)>, dst: (i32, i32), cursor: usize },
    /// DES 负参（a.java:8311-8320）：全表批量移除 -type
    DesAll { want: i32, cursor: usize },
    /// DES 正参（a.java:8321-8334）：格上定点移除
    DesAt { ty: i32, cx: i32, cy: i32, cursor: usize },
    /// SWD（a.java:8336-8352）
    Swd { cursor: usize },
    /// CES_t_cx_cy（a.java:8386-8420）
    Ces { ty: i32, cx: i32, cy: i32, cursor: usize },
    /// GLV_n（a.java:8464-8466）
    Glv { cursor: usize },
    /// END_n（a.java:8445-8463 区）
    End { cursor: usize },
    /// SMS_n（a.java 纯推进）
    Sms { cursor: usize },
    /// ROS_1（a.java:8361-8366）：玩家格坐标
    RosMove { px: i32, py: i32, cursor: usize },
    /// ROS_2（a.java:8367-8369）：强制装备
    RosEquip { tier: i32, cursor: usize },
    /// ROS_3（a.java:8370-8372）：HP
    RosHp { v: i32, cursor: usize },
    /// ROS_4（a.java:8373-8375）：朝向（Java byte 字段 ⇒ i8 截断保真）
    RosFace { v: i32, cursor: usize },
    /// ROS_5（a.java:8376-8378）：ATK
    RosAtk { v: i32, cursor: usize },
    /// ROS_6（a.java:8379-8381）：DEF
    RosDef { v: i32, cursor: usize },
    /// ROS 其余子码（a.java:8381 default）：不取参不推进（v2 不动）
    RosNoop { cursor: usize },
    /// MVS（a.java:8519-8592）
    Mvs { cursor: usize },
}

impl GameScene {
    /// TAK_a_b（a.java:8257-8263）：页区间 + 对话相位 1 + 开页（pageEnd 先写——
    /// 不先更新会误判"越界"而丢页）。
    pub(super) fn op_tak(&mut self, start: i32, end: i32) {
        self.dialog.dialog_page = start;
        self.dialog.dialog_page_end = end;
        self.dialog.dialog_phase = 1;
        self.open_dialog_page(start);
    }

    /// LAY_x（a.java:8355-8362）：m_119 存态 + 目标层 + 遮幅（phase=5）+ 清 overlay。
    pub(super) fn op_lay(&mut self, floor: i32) {
        self.save_floor_state();
        self.persist.pending_floor = floor;
        self.transitioning = true;
        self.dialog.overlay_text = None;
        self.dialog.dialog_phase = 5;
    }

    /// GIN_a_b（a.java:8424-8437）：0=pickupItemType(b)/1=gainGold(b)；
    /// 完后**尾递归**执行下一条（8435-8436）。
    pub(super) fn op_gin(&mut self, kind: i32, value: i32) {
        match kind {
            0 => self.pickup_item_type(value),
            1 => {
                crate::combat::gain_gold(&mut self.player, value);
            }
            _ => {}
        }
        self.execute_script_instruction();
    }

    /// RES_x（a.java:8470-8490）：easy（difficultyIndex==0）重置 run 统计；非 easy 空转。
    pub(super) fn op_res(&mut self) {
        if self.difficulty_index == 0 {
            self.player.yellow_keys = 0;
            self.player.red_keys = 0;
            self.player.blue_keys = 0;
            self.player.item_stack_size = 0;
            self.player.gold = 4;
            self.alchemy_upgrade_count = 0;
            self.persist.min_floor = 1;
            self.persist.max_floor = 3;
            self.player.atk = 10;
            self.player.def = 10;
            self.player.hp = 400;
            self.persist.journal.clear();
        }
    }

    /// ADD_a_b_c_d（a.java:8438-8468）：跨层生成实体——目标层非当前时
    /// m_119 存当前→m_122 载目标→spawn→m_119 存目标→还原（8462-8464）。
    pub(super) fn op_add(&mut self, floor: i32, ty: i32, cx: i32, cy: i32) {
        if self.persist.floor != floor {
            self.save_floor_state();
            if self.persist.floor_assets.get(floor as usize).cloned().flatten().is_some() {
                self.reload_floor_entities(floor);
            }
        }
        entity::spawn(
            &mut self.entities,
            &mut self.grid,
            ty,
            cx << 5,
            cy << 5,
            0,
            &self.width_table,
            None,
        );
        self.save_floor_state();
        if self.persist.floor != floor {
            if self.persist.floor_assets.get(self.persist.floor as usize).cloned().flatten().is_some() {
                self.reload_floor_entities(self.persist.floor);
            }
        }
    }

    /// SEE_a_b_c_d_e（a.java:8487-8510）：pageStart/End 先解析入字段再开页
    ///（8487-8495）；flag==1 ⇒ findPath 成功 → 路径箭头 + walkPhase=0（8497-8501，
    /// case1 **无 break 落穿 default**：easeCameraToTarget 与箭头两者都做）；
    /// 收尾 phase=6 + easeCameraToTarget ease 相机目标 = 目标格。
    pub(super) fn op_see(&mut self, tx: i32, ty: i32, page: i32, page_end: i32, flag: i32) {
        self.dialog.dialog_target = (tx, ty);
        self.dialog.dialog_page = page;
        self.dialog.dialog_page_end = page_end;
        self.open_dialog_page(page);
        if flag == 1 && self.find_path(self.player_cell_x, self.player_cell_y, tx, ty) {
            self.path_arrows_visible = true;
            self.walk_phase = 0;
        }
        self.dialog.dialog_phase = 6;
        self.ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;
        self.ease_cam_y = ((self.view.view_h - 32) >> 1) - self.view.cam_y;
        self.ease_target_x = tx << 5;
        self.ease_target_y = ty << 5;
    }

    /// GUT_n（a.java:8308-8311）：旗标置位 + loadLevelScript(n)（游标由 load
    /// 重置为 0——Java 原样不前进）。
    pub(super) fn op_gut(&mut self, target: i32) {
        let cur = self.current_script_index;
        self.script_line_flags[cur] = true;
        self.load_level_script(target);
    }

    /// MOV（a.java:8274-8309）：var55>0 ⇒ 五参实体路线（dp2/openWallOnRoute）；否则三参
    /// 玩家脚本行走（findPath 成功 → scriptWalkArmed+dp3+walkPhase=0+lockCameraOn(0)；
    /// 失败无相位副作用）。**miss 尾递归**（8301-8302）：同拍执行下一条。
    pub(super) fn op_mov(&mut self, target_type: i32, cell: Option<(i32, i32)>, dst: (i32, i32)) {
        let (mx, my) = dst;
        self.dialog.dialog_target = (mx, my);
        if target_type > 0 {
            let (ent_x, ent_y) = cell.expect("五参已解析");
            // a.java:8289：route_entity = findBlockingEntity(...)（原 findBlockingEntity）——未命中也写入（-1）
            self.route_entity = self.entity_at_cell(ent_x, ent_y, target_type);
            if self.route_entity >= 0 {
                let ei = self.route_entity as usize;
                self.route_phase = 0; // routePhase = 0（a.java:8291）
                if self.entities.items[ei].type_ == 72 {
                    self.entities.items[ei].anim_row = 1; // a.java:8293-8294
                }
                // f_int_129/130 = 实体起点格（a.java:8297-8298）
                self.route_start = (self.entities.items[ei].px >> 5, self.entities.items[ei].py >> 5);
                // findPath(起点格 → 目标格)：返回值不判（a.java:8299）——openWallOnRoute
                // 只认 walkPathBuffer/walkStepCount（find_path 恒重写，含回溯
                // 失败的部分前缀）
                self.find_path(self.route_start.0, self.route_start.1, mx, my);
                self.dialog.dialog_phase = 2;
            } else {
                // 实体未找到 → 同拍尾递归执行下一条（a.java:8301-8302）
                self.execute_script_instruction();
            }
        } else if self.find_path(self.player_cell_x, self.player_cell_y, mx, my) {
            self.script_walk_armed = true;
            self.dialog.dialog_phase = 3;
            self.walk_phase = 0;
            self.lock_camera_on(0);
        }
    }

    /// DES 负参（a.java:8311-8320）：全表批量移除 -type。
    pub(super) fn op_des_all(&mut self, want: i32) {
        for i in (0..self.entities.count).rev() {
            if self.entities.items[i].type_ == want && entity::remove(&mut self.entities, i) {
                self.build_minimap();
            }
        }
    }

    /// DES 正参（a.java:8321-8334）：格上首只该类型移除（findBlockingEntity + markEntityRemoved）。
    pub(super) fn op_des_at(&mut self, ty: i32, cx: i32, cy: i32) {
        let idx = self.entity_at_cell(cx, cy, ty);
        if idx >= 0 && entity::remove(&mut self.entities, idx as usize) {
            self.build_minimap();
        }
    }

    /// SWD（a.java:8336-8352）：全表扫 81 → 显形 + 改型 4 + DES 状态动画
    ///（state=2 行 8——advance_frames 帧尽自解为常态）。
    pub(super) fn op_swd(&mut self) {
        for i in (0..self.entities.count).rev() {
            if !self.entities.items[i].removed && self.entities.items[i].type_ == 81 {
                self.entities.items[i].visible = true;
                self.entities.items[i].type_ = 4;
                let img_h = self.images.entity.get(4).cloned().flatten().map(|im| im.height);
                entity::dispatch_sprite_size(4, i, &mut self.entities, &self.width_table, img_h);
                self.entities.items[i].state = 2;
                self.entities.items[i].anim_row = 8;
            }
        }
    }

    /// CES（a.java:8386-8420）：spawnEntity(type, cx<<5, cy<<5, 0) +
    /// markEntityDesRemoved + 相机回中后缓向新实体 + scriptStepDelay
    ///（cat 1/8/16=5、2/4=8、其余 0）。
    pub(super) fn op_ces(&mut self, ty: i32, cx: i32, cy: i32) {
        let img_h = self.images.entity.get(ty as usize).cloned().flatten().map(|im| im.height);
        let idx = entity::spawn(
            &mut self.entities,
            &mut self.grid,
            ty,
            cx << 5,
            cy << 5,
            0,
            &self.width_table,
            img_h,
        );
        entity::remove_des(&mut self.entities, idx);
        self.ease_cam_x = ((self.view.view_w - 32) >> 1) - self.view.cam_x;
        self.ease_cam_y = ((self.view.view_h - 32) >> 1) - self.view.cam_y;
        // Java `if (sort_i >= 0)` 守卫（a.java:8398）：spawn 返回 usize 恒非负 ⇒ 恒真
        self.ease_target_x = self.entities.items[idx].px;
        self.ease_target_y = self.entities.items[idx].py;
        self.script_delay = match self.entity_category(ty) {
            1 | 8 | 16 => 5,
            2 | 4 => 8,
            _ => 0,
        };
    }

    /// GLV_n（a.java:8464-8466）：纯解析推进（无副作用）。
    pub(super) fn op_glv(&mut self) {}

    /// END_n（a.java:8445-8463 区）：进结局态——dispatchModeEntry case 20（a.java:4443-4450）
    /// 可观测部分：typewriterPos=1 + 初次 layout（对象 = **当时 overlayText 残页**，
    /// Java 原样）；f_int_22/23/f_bool_04 不重置。
    pub(super) fn op_end(&mut self) {
        self.transition_to(GameMode::Ending);
        self.dialog.dialog_char = 1;
        if let Some(t) = self.dialog.overlay_text.clone() {
            let line_h = crate::paint::paint_font().height + 4;
            self.layout_wrapped_text_limited(&t, 209, (line_h << 1) + 12, 1);
        }
    }

    /// SMS_n（a.java 纯推进）：无副作用。
    pub(super) fn op_sms(&mut self) {}

    /// ROS_1（a.java:8361-8366）：玩家格坐标（px<<5/py<<5）+ 锁相机回玩家。
    pub(super) fn op_ros_move(&mut self, px: i32, py: i32) {
        self.view.player_px = px << 5;
        self.view.player_py = py << 5;
        self.lock_camera_on(0);
    }

    /// ROS_2（a.java:8367-8369）：强制装备 tryEquip(tier,true)。
    pub(super) fn op_ros_equip(&mut self, tier: i32) {
        self.try_equip(tier, true);
    }

    /// ROS_3（a.java:8370-8372）：HP。
    pub(super) fn op_ros_hp(&mut self, v: i32) {
        self.player.hp = v;
    }

    /// ROS_4（a.java:8373-8375）：朝向（Java byte 字段 ⇒ i8 截断保真）。
    pub(super) fn op_ros_face(&mut self, v: i32) {
        self.facing = (v as i8) as i32;
    }

    /// ROS_5（a.java:8376-8378）：ATK。
    pub(super) fn op_ros_atk(&mut self, v: i32) {
        self.player.atk = v;
    }

    /// ROS_6（a.java:8379-8381）：DEF。
    pub(super) fn op_ros_def(&mut self, v: i32) {
        self.player.def = v;
    }

    /// MVS（a.java:8519-8592）：型 82 传怪点成对互传。**双重死码**
    ///（语料 297 条指令 0 条 MVS + 全 55 层 sprite 型 82=0，
    /// evidence-todos.md T-1）——语料不可达，为忠实而保。体随迁移原样。
    pub(super) fn op_mvs(&mut self) {

        // MVS（a.java:8519-8592）：型 82 传怪点成对互传。**双重死码**
        // （语料 297 条指令 0 条 MVS + 全 55 层 sprite 型 82=0，
        // evidence-todos.md T-1）——语料不可达，为忠实而保。
        // 倒序扫 type==82 按 param&255 两两配对进 16 对短缓存
        // （f_short_arr_04 等价，Java 每次 MVS 重建，a.java:8521-8522）；
        // 每对两向（8571-8573 交换重复）：扫描格槽组内 renderCategory==8
        // 怪物 → 双向 removeGroupAtCellByType(两格, 82)（82 点位消耗）
        // → carry_monsters_to 搬到对端。槽组循环 var8 为**循环前**容量快照、槽表
        // 就地压缩不清尾（a.java:6440-6443）⇒ stale 尾重触发 quirk 保真。
        let mut buf = [0i16; 32];
        for v6 in (0..self.entities.count as i32).rev() {
            if self.entities.items[v6 as usize].type_ == 82 {
                let key = self.entities.items[v6 as usize].param as i32 & 255;
                let mut v7 = 0;
                while v7 < 32 {
                    let v70 = buf[v7] as i32 - 1;
                    if v70 <= 0 {
                        buf[v7] = (v6 + 1) as i16;
                        break;
                    }
                    if key == (self.entities.items[v70 as usize].param as i32 & 255) {
                        buf[v7 + 1] = (v6 + 1) as i16;
                        break;
                    }
                    v7 += 2;
                }
            }
        }
        let mut v11 = 0;
        while v11 < 32 {
            let mut v70 = buf[v11] as i32;
            let mut v9 = buf[v11 + 1] as i32;
            if v70 <= 0 {
                break;
            }
            if v9 > 0 {
                v70 -= 1;
                v9 -= 1;
                let mut cx86 = self.entities.items[v70 as usize].px >> 5;
                let mut cy6 = self.entities.items[v70 as usize].py >> 5;
                let ct44 = self.grid.cell_type[cy6 as usize][cx86 as usize] as usize;
                let n8 = self.grid.capacity[ct44] as usize;
                let mut v12 = 0;
                while v12 < n8 {
                    let v10 = self.grid.slots[ct44][v12] as usize - 1;
                    let t56 = self.entities.items[v10].type_;
                    if entity::render_category(t56) == 8 {
                        let tx = self.entities.items[v9 as usize].px >> 5;
                        let ty = self.entities.items[v9 as usize].py >> 5;
                        entity::remove_group_at_cell_by_type(tx, ty, 82, &mut self.entities, &mut self.grid);
                        entity::remove_group_at_cell_by_type(cx86, cy6, 82, &mut self.entities, &mut self.grid);
                        self.carry_monsters_to(v10, tx, ty);
                    }
                    v12 += 1;
                }
                // A/B 交换（a.java:8571-8573）后重复反向
                core::mem::swap(&mut v70, &mut v9);
                cx86 = self.entities.items[v70 as usize].px >> 5;
                cy6 = self.entities.items[v70 as usize].py >> 5;
                let ct46 = self.grid.cell_type[cy6 as usize][cx86 as usize] as usize;
                let n8b = self.grid.capacity[ct46] as usize;
                let mut v93 = 0;
                while v93 < n8b {
                    let v92 = self.grid.slots[ct46][v93] as usize - 1;
                    let t56 = self.entities.items[v92].type_;
                    if entity::render_category(t56) == 8 {
                        let tx = self.entities.items[v9 as usize].px >> 5;
                        let ty = self.entities.items[v9 as usize].py >> 5;
                        entity::remove_group_at_cell_by_type(tx, ty, 82, &mut self.entities, &mut self.grid);
                        entity::remove_group_at_cell_by_type(cx86, cy6, 82, &mut self.entities, &mut self.grid);
                        self.carry_monsters_to(v92, tx, ty);
                    }
                    v93 += 1;
                }
            }
            v11 += 2;
        }
    }

    /// 纯解析层（游标契约，refactor-plan 条件 4）：条件取参清单逐字节等价——
    /// MOV var55>0 续取 3 参（五参）/ DES 负参分支 / ROS 子码条件取参 /
    /// GUT 无游标（load 重置）。每变体自带**推进后游标**（各 op 的 +1 口径
    /// 逐 op 锚注释）；16 标签外 ⇒ None（R-TODO-1b 停摆）。
    pub(super) fn parse_op(line: &str, script_cursor0: usize) -> Option<ScriptOp> {
        let op = &line[script_cursor0..script_cursor0 + 3];
        // 与旧臂同口径：substring 后 script_cursor0 无条件 +3（a.java:8244-8239）——
        // 首参基址 = script_cursor0+1；SWD/MVS 游标 = script_cursor0+1
        let script_cursor0 = script_cursor0 + 3;
        let base = script_cursor0 + 1;
        Some(match op {
            // TAK_a_b（a.java:8257-8263）：游标 = 尾分隔位 + 1
            "TAK" => {
                let (start, e1) = Self::parse_script_int(line, base, '_');
                let (end, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                ScriptOp::Tak { start, end, cursor: e2 + 1 }
            }
            // LAY_x（a.java:8355-8362）
            "LAY" => {
                let (floor, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Lay { floor, cursor: e + 1 }
            }
            // GIN_a_b（a.java:8424-8437）：0=pickupItemType(b)/1=gainGold(b)；尾递归由执行层做
            "GIN" => {
                let (kind, e1) = Self::parse_script_int(line, base, '_');
                let (value, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                ScriptOp::Gin { kind, value, cursor: e2 + 1 }
            }
            // RES_x（a.java:8470-8490）
            "RES" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Res { cursor: e + 1 }
            }
            // ADD_a_b_c_d（a.java:8438-8468）
            "ADD" => {
                let (a, e1) = Self::parse_script_int(line, base, '_');
                let (b, e2) = Self::parse_script_int(line, e1 + 1, '_');
                let (c, e3) = Self::parse_script_int(line, e2 + 1, '_');
                let (d, e4) = Self::parse_script_int(line, e3 + 1, ' ');
                ScriptOp::Add { floor: a, ty: b, cx: c, cy: d, cursor: e4 + 1 }
            }
            // SEE_a_b_c_d_e（a.java:8487-8510）：五参条件取参
            "SEE" => {
                let (tx, a) = Self::parse_script_int(line, base, '_');
                let (ty, b) = Self::parse_script_int(line, a + 1, '_');
                let (pg, c) = Self::parse_script_int(line, b + 1, '_');
                let (pg2, d) = Self::parse_script_int(line, c + 1, '_');
                let (flag, e) = Self::parse_script_int(line, d + 1, ' ');
                ScriptOp::See { tx, ty, page: pg, page_end: pg2, flag, cursor: e + 1 }
            }
            // GUT_n（a.java:8308-8311）：游标由 loadLevelScript 重置——解析层
            // 仍算尾位（执行层统一写后被 load 覆写，等价）
            "GUT" => {
                let (target, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Gut { target, cursor: e + 1 }
            }
            // MOV（a.java:8274-8309）：**var55>0 续取 3 参**（条件取参清单①）
            "MOV" => {
                let (target_type, mut v2) = Self::parse_script_int(line, base, '_');
                let mut cell = None;
                if target_type > 0 {
                    let (cx, e) = Self::parse_script_int(line, v2 + 1, '_');
                    v2 = e;
                    let (cy, e) = Self::parse_script_int(line, v2 + 1, '_');
                    v2 = e;
                    cell = Some((cx, cy));
                }
                let (mx, e) = Self::parse_script_int(line, v2 + 1, '_');
                v2 = e;
                let (my, e) = Self::parse_script_int(line, v2 + 1, ' ');
                ScriptOp::Mov { target_type, cell, dst: (mx, my), cursor: e + 1 }
            }
            // DES（a.java:8311-8334）：**负参分支**（条件取参清单②）
            "DES" => {
                let (t0, v2) = Self::parse_script_int(line, base, '_');
                if t0 < 0 {
                    let (_, e) = Self::parse_script_int(line, v2 + 1, ' ');
                    ScriptOp::DesAll { want: -t0, cursor: e + 1 }
                } else {
                    let (cx, e1) = Self::parse_script_int(line, v2 + 1, '_');
                    let (cy, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                    ScriptOp::DesAt { ty: t0, cx, cy, cursor: e2 + 1 }
                }
            }
            // SWD（a.java:8336-8352）：无参，游标 = script_cursor0+1
            "SWD" => ScriptOp::Swd { cursor: script_cursor0 + 1 },
            // CES_t_cx_cy（a.java:8386-8420）
            "CES" => {
                let (ty, v2) = Self::parse_script_int(line, base, '_');
                let (cx, e1) = Self::parse_script_int(line, v2 + 1, '_');
                let (cy, e2) = Self::parse_script_int(line, e1 + 1, ' ');
                ScriptOp::Ces { ty, cx, cy, cursor: e2 + 1 }
            }
            // GLV_n（a.java:8464-8466）
            "GLV" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Glv { cursor: e + 1 }
            }
            // END_n（a.java:8445-8463 区）
            "END" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::End { cursor: e + 1 }
            }
            // SMS_n（a.java:8500s 区）：纯推进
            "SMS" => {
                let (_v, e) = Self::parse_script_int(line, base, ' ');
                ScriptOp::Sms { cursor: e + 1 }
            }
            // ROS_sub（a.java:8359-8381）：**子码条件取参**（条件取参清单④）——
            // 1=格坐标两参；2-6=单参；其余不取（v2 不动）；游标 = v2+1
            "ROS" => {
                let (sub, mut v2) = Self::parse_script_int(line, base, '_');
                match sub {
                    1 => {
                        let (px, e) = Self::parse_script_int(line, v2 + 1, '_');
                        v2 = e;
                        let (py, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosMove { px, py, cursor: v2 + 1 }
                    }
                    2 => {
                        let (tier, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosEquip { tier, cursor: v2 + 1 }
                    }
                    3 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosHp { v, cursor: v2 + 1 }
                    }
                    4 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosFace { v, cursor: v2 + 1 }
                    }
                    5 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosAtk { v, cursor: v2 + 1 }
                    }
                    6 => {
                        let (v, e) = Self::parse_script_int(line, v2 + 1, ' ');
                        v2 = e;
                        ScriptOp::RosDef { v, cursor: v2 + 1 }
                    }
                    _ => ScriptOp::RosNoop { cursor: v2 + 1 },
                }
            }
            // MVS（a.java:8519-8592）：无参，游标 = script_cursor0+1
            "MVS" => ScriptOp::Mvs { cursor: script_cursor0 + 1 },
            _ => return None,
        })
    }

    /// 执行层：统一写游标（解析层逐 op 计算）→ 逐 op 独立函数。
    /// GUT 的游标随后被 loadLevelScript 重置为 0（a.java:8308-8311 原样）。
    pub(super) fn execute_op(&mut self, op: ScriptOp) {
        self.script_cursor = op.cursor();
        match op {
            ScriptOp::Tak { start, end, .. } => self.op_tak(start, end),
            ScriptOp::Lay { floor, .. } => self.op_lay(floor),
            ScriptOp::Gin { kind, value, .. } => self.op_gin(kind, value),
            ScriptOp::Res { .. } => self.op_res(),
            ScriptOp::Add { floor, ty, cx, cy, .. } => self.op_add(floor, ty, cx, cy),
            ScriptOp::See { tx, ty, page, page_end, flag, .. } => {
                self.op_see(tx, ty, page, page_end, flag)
            }
            ScriptOp::Gut { target, .. } => self.op_gut(target),
            ScriptOp::Mov { target_type, cell, dst, .. } => self.op_mov(target_type, cell, dst),
            ScriptOp::DesAll { want, .. } => self.op_des_all(want),
            ScriptOp::DesAt { ty, cx, cy, .. } => self.op_des_at(ty, cx, cy),
            ScriptOp::Swd { .. } => self.op_swd(),
            ScriptOp::Ces { ty, cx, cy, .. } => self.op_ces(ty, cx, cy),
            ScriptOp::Glv { .. } => self.op_glv(),
            ScriptOp::End { .. } => self.op_end(),
            ScriptOp::Sms { .. } => self.op_sms(),
            ScriptOp::RosMove { px, py, .. } => self.op_ros_move(px, py),
            ScriptOp::RosEquip { tier, .. } => self.op_ros_equip(tier),
            ScriptOp::RosHp { v, .. } => self.op_ros_hp(v),
            ScriptOp::RosFace { v, .. } => self.op_ros_face(v),
            ScriptOp::RosAtk { v, .. } => self.op_ros_atk(v),
            ScriptOp::RosDef { v, .. } => self.op_ros_def(v),
            ScriptOp::RosNoop { .. } => {}
            ScriptOp::Mvs { .. } => self.op_mvs(),
        }
    }

}
