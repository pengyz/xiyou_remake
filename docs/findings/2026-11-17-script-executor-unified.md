# 脚本执行器统一（S3·④）：流程单一来源，世界动作两端各自实现

日期：2026-11-17　范围：`crates/game-core`（`script_exec.rs` 新增）+ 生产路径切换
前置：S3·① 解析统一（`ced8a91`）、S3·② 脚本域写统一（`87e4bf8`）、S3·③ 宿主面拆分（`7587c68`）

## 1. 此前「两套执行器」到底差在哪

到 S3·③ 为止，解析、宿主面、脚本域写都已单一来源，但**流程本身**仍是两份手抄：

| 流程步骤 | 生产（`GameScene`，`scene_script.rs`） | headless（`ScriptEngine`，`script.rs`） |
|---|---|---|
| 入口门 `cursor+3 > len` | 手写 | 手写 |
| 解析 | `Self::parse_op` | `GameScene::parse_op` |
| 脚本域写 | `apply_script_domain` | `slots.apply` |
| GIN 尾递归 | 函数尾 `if Recurse` | 函数尾 `if Recurse` |
| 世界动作 | `match op → self.op_*` | `match op → host.*` |
| 行尾收尾 | phase=4 + flag + 相机回玩家 | 同形（写引擎自身 state） |

前四步本应只有一份；后两步**本来就不可能相同**（一个是真场景，一个是状态模型）。
此前是六步全抄两份 ⇒ 每一处历史事故都落在这条缝上（槽67 尾空格、SWD/MVS 游标差一格、
行尾收尾门误读）。

## 2. 现在

新增 `crates/game-core/src/script_exec.rs`：

```rust
pub trait ScriptRunner {
    fn slots(&mut self) -> &mut ScriptSlots;
    fn floor(&self) -> i32;
    fn current_line(&self) -> String;
    fn on_line_end(&mut self, cur: usize);   // 钩子：行尾收尾
    fn exec_op(&mut self, op: ScriptOp);     // 钩子：一条命令的世界动作
}

pub fn run_instruction<R: ScriptRunner>(r: &mut R) { /* 入口门→解析→apply→exec_op→尾递归 */ }
```

两个实现端：

- `impl ScriptRunner for GameScene`（`scene_script.rs`）：`exec_op` → `self.dispatch_op(op)`；
  `on_line_end` → phase=4 + flag + `lock_camera_on(0)`。
- `EngineRunner<'a,'d,H>`（`script.rs`，包 `&mut ScriptEngine` + `&mut H`）：同样的两个钩子，
  写引擎 state 与宿主副作用。

生产入口 `GameScene::execute_script_instruction` 现在是一行：
`crate::script_exec::run_instruction(self)` —— **生产路径已切到共享流程**。

脚本域状态也并成一份类型：`GameScene` 的 6 个平铺字段
（`script_line` / `script_cursor` / `script_delay` / `script_line_flags` /
`current_script_index` / `script_walk_armed`）收进
`pub slots: ScriptSlots`，与求解器 headless 执行器同一个结构；`script_ctx()` 简化为
`self.slots.ctx(self.persist.floor)`。

## 3. 判据（可复跑）

- `script_exec::tests::s3d4_*` 四条：入口门不执行命令且不动游标 / 正常指令推进游标 /
  GIN 同拍尾递归 / 未知 op 静默停摆。
- 跨端一致性仍由既有对拍腿把关：`script_abs::tests::script_abs_对拍_fixture68槽逐行零差异`
  （side_a 生产 GameScene ⟷ side_b 引擎+BHost），本轮通过。
- 回归：`cargo test -p game-core`（lib 394）、`-p game-solver --lib` 7、
  `-p game-oracle` 全绿（含 walk replay 45、fixture_frames 45）；
  `gates/cli.py check` PASS=13 FAIL=0，74 金样 sha 全同；ratchet 1091→1095。

## 4. 残留（明确不假装已解决）

世界动作仍是两份实现（`dispatch_op` 生产 / 引擎分派），因为「玩家血量写进真 PlayerCombat」
与「写进状态模型」注定不同。区别在于：**现在两份只覆盖世界动作，流程与状态槽已无第二份**，
任何流程级漂移在类型上无处可藏；语义级漂移由 68 槽对拍 + 74 金样 + walk replay 三层判据盯。
