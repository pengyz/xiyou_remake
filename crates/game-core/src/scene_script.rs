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
            self.dialog.dialog_phase = 4;
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
    /// advanceScriptWalk（a.java:5822-5838）：弹顶步 → tryStep；步尽解除武装；
    /// 撞格（返回 false）解除武装并清步（walkPhase==5 战斗中除外）。
    pub(super) fn advance_script_walk(&mut self) {
        if self.walk_step_count > 0 {
            self.dec_walk_step_count();
            self.set_facing(self.walk_path_buffer[self.walk_step_count as usize] as i32);
            let ok = self.try_step(self.facing);
            if self.walk_step_count == 0 {
                self.script_walk_armed = false;
                return;
            }
            if !ok {
                self.script_walk_armed = false;
                if self.walk_phase != 5 {
                    self.set_walk_phase(0);
                    self.set_walk_step_count(0);
                }
            }
        }
    }
}
