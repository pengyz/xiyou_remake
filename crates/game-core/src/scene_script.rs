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
    /// executeScriptInstruction（a.java:8239-8581）——**生产端已切到共享流程**
    /// 【2026-11-17 S3·④】：入口门 / 解析 / 脚本域写 / 尾递归由
    /// [`crate::script_exec::run_instruction`] 唯一实现，本端只提供两个钩子
    /// （行尾收尾、一条命令的世界动作）。此前这四步在生产端与 headless 执行器
    /// 各手抄一份，四类漂移各有一条历史事故（槽67 尾空格 / SWD·MVS 游标差一格 /
    /// 尾门误读）。
    pub(super) fn execute_script_instruction(&mut self) {
        crate::script_exec::run_instruction(self);
    }

    /// advanceScriptWalk（a.java:5822-5838）：弹顶步 → tryStep；步尽解除武装；
    /// 撞格（返回 false）解除武装并清步（walkPhase==5 战斗中除外）。
    pub(super) fn advance_script_walk(&mut self) {
        if self.walk_step_count > 0 {
            self.dec_walk_step_count();
            self.set_facing(self.walk_path_buffer[self.walk_step_count as usize] as i32);
            let ok = self.try_step(self.facing);
            if self.walk_step_count == 0 {
                self.set_script_walk_armed(false);
                return;
            }
            if !ok {
                self.set_script_walk_armed(false);
                if self.walk_phase != 5 {
                    self.set_walk_phase(0);
                    self.set_walk_step_count(0);
                }
            }
        }
    }
}

/// 【2026-11-17 S3·④】生产端执行钩子：真场景的世界动作 + 行尾收尾。
impl crate::script_exec::ScriptRunner for GameScene {
    fn slots(&mut self) -> &mut crate::subsystem::script_host::ScriptSlots {
        &mut self.slots
    }
    fn floor(&self) -> i32 {
        self.persist.floor
    }
    fn current_line(&self) -> String {
        self.slots.script_line.clone()
    }
    /// 入口门命中：phase=4（等相机）+ 置当前行 flag（32 除外）+ 相机回玩家。
    /// Java 侧同一份写发生在 2401 门体，只由偏移 22 的 `cursor+3 > len` 到达。
    fn on_line_end(&mut self, cur: usize) {
        self.dialog.set_dialog_phase(4);
        if cur != 32 {
            self.slots.script_line_flags[cur] = true;
        }
        self.lock_camera_on(0);
    }
    fn exec_op(&mut self, op: crate::subsystem::script_host::ScriptOp) {
        self.dispatch_op(op);
    }
}
