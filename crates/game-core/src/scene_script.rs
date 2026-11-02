//! task-15 步⑤ 脚本解释器（move-only 自 scene.rs）——**execute_script_instruction
//! 16 op 语义冻结面**（逐值对照体随迁不动；金样 74 场景同闸把关）。
use super::*; // scene.rs 私有面可见（子模块）——纯剪切语义保持

impl GameScene {
    /// parseScriptInt（a.java:8583-8584）：from 起找 delim，返回值与终位。
    pub(super) fn parse_script_int(line: &str, from: usize, delim: char) -> (i32, usize) {
        let rel = line[from..].find(delim).expect("脚本整数分隔符缺失");
        let end = from + rel;
        (line[from..end].parse().expect("脚本整数解析失败"), end)
    }
    /// executeScriptInstruction 的 script 40 可达子集：TAK/SEE/END/SMS。
    /// 其余 opcode（MOV/DES/CES/GUT/LAY/ROS/RES/ADD/MVS）在对应脚本对拍批实现。
    pub(super) fn execute_script_instruction(&mut self) {
        let line = self.script_line.clone();
        let script_cursor0 = self.script_cursor;
        if script_cursor0 + 3 > line.len() {
            // 行尾收尾（a.java:8597-8604）：phase=4 等相机 + 置 flag + 相机回玩家
            self.dialog_phase = 4;
            let cur = self.current_script_index;
            if cur != 32 {
                self.script_line_flags[cur] = true;
            }
            self.lock_camera_on(0);
            return;
        }
        // 命令模式（refactor-plan §3 第 2 步）：parse_op 纯解析（条件取参逐字节
        // 等价）→ execute_op 执行（逐 op 独立函数）。UNKNOWN ⇒ None ⇒ 停摆。
        let Some(op) = Self::parse_op(&line, script_cursor0) else {
            // 真未知 op（16 标签外）：Java 静默停摆（a.java:8594-8595 原地 return：
            // 无臂写 scriptCursor ⇒ 下一拍重解析同一 token）。禁止 3+N 跳字节续行。
            // 语义见 docs/spec/evidence-todos.md T-1 R-TODO-1b。
            return;
        };
        self.execute_op(op);
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
    /// advanceScriptWalk（a.java:5822-5838）：弹顶步 → tryStep；步尽解除武装；
    /// 撞格（返回 false）解除武装并清步（walkPhase==5 战斗中除外）。
    pub(super) fn advance_script_walk(&mut self) {
        if self.walk_step_count > 0 {
            self.walk_step_count -= 1;
            self.facing = self.walk_path_buffer[self.walk_step_count as usize] as i32;
            let ok = self.try_step(self.facing);
            if self.walk_step_count == 0 {
                self.script_walk_armed = false;
                return;
            }
            if !ok {
                self.script_walk_armed = false;
                if self.walk_phase != 5 {
                    self.walk_phase = 0;
                    self.walk_step_count = 0;
                }
            }
        }
    }
}
