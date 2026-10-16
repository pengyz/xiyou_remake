# BUG-006 走位实验实录：剧情点链触发实证（task-20 一锤定音判据）

> 2026-10-16。夹具：`reference/oracle/scenarios/prologue-walk-storypoint.txt`。
> 复现：`java -cp reference/oracle/_out/classes:analysis/frames-fixture/variantA:original/囧囧西游-大闹天宫.jar
> -Doracle.out=… -Doracle.max=1500 -Doracle.script=reference/oracle/scenarios/prologue-walk-storypoint.txt
> -Doracle.dumpStride=1 -Doracle.frames=none oracle.host.Runner`（trace 后用
> `python3 reference/oracle/fields.py --timeline <trace.txt> …` 读语义时间线）。
> 配套判定文档：docs/spec/bug-root-cause-behavior.md §1（三层重审）。

## 结论（BUG-006 B+ → A 级结案）

**剧情点链实证成立：SEE"冻结态"后按游戏指示走位到 (3,10) 踩点，script 41 续篇当拍触发。
BUG-006 死锁定性撤销——非 bug、非 shim、非投影；是原版教学走位设计 + 测试驱动未走位。**

## A 级证据

**1. 游戏自己给出走位指示（设计意图直证）**——SEE_3_10_166_166_1 打开的页 166 原文
（trace FLD 062，摘录同文见 crates/game-oracle/tests/fixtures/prologue_trace_excerpt.txt）：

```
菩提老祖: \cF8F8F8好了，现在请试着\cFFCC00按方向键移动到这里。
```

**2. 走位时间线（本次实跑，fields.py --timeline 语义投影）**：

```
 1021 | dialogPhase = 0
 1022 | dialogPage = 166, dialogPhase = 6        ← SEE_3_10_166_166_1 执行（cursor=43, walkStepCount=3）
 1101 | dialogPhase = 0
 1102 | dialogPhase = 4                          ← 行尾收尾门（a.java:8594-8604）
 1110 | gameMode = 3                             ← 进自由玩法（a.java:4113-4116 cameraAtTarget）
 1205 | playerCellY = 10                         ← 步1 (1,11)→(1,10)（键 -1 = facing1 = y-）
 1225 | playerCellX = 2                           ← 步2 (1,10)→(2,10)（键 -4 = facing2 = x+）
 1245 | playerCellX = 3                           ← 步3 (2,10)→(3,10) 踩中剧情点！
 1245 | gameMode = 11, currentScriptIndex = 41   ← loadLevelScript(41) 同拍触发（a.java:5710 step 臂）
 1246 | dialogPage = 167, dialogPhase = 1        ← script41 首指令 TAK_167_167
```

**3. 续篇文本（链条继续的直接观测）**——script41 页 167 原文（FLD 062）：

```
菩提老祖: \cF8F8F8很好，你已经学会太空步了。
```

**4. 触发臂归属**：踩点当拍（T1245 playerCellX 2→3）即 currentScriptIndex 40→41——
step 臂 applyStepCellEffects case 83 `tryRunScene(param, true)`（a.java:5710）经
tryRunScene 到达臂 `var3 &= var2`（a.java:8128-8147，case 41 在列）放行；
bump 臂（a.java:5466 `tryRunScene(param, false)`）被 from_walk 门挡掉后放行入格
（design：bump 不触发、走到才触发——与 task-17 重审预测一致）。

## 判据闭合对照（task-17 遗留四项）

1. ~~走位实验~~ → **本文闭合**（T1245 触发链实证）。
2. walkStepCount=3 = findPath 成功写点（SEE 臂 f_bool_26 唯一写点），非残留（走位实验同 run 复证）。
3. Rust find_path `!found` 与 Java findPath 成功的分歧仍在（P3.3 m_062 语义）——**不影响本结案**
   （走位是手动移动，不依赖 findPath）；留作移植保真缺口单独清偿。
4. "变异复核证伪一次（冻结态）"两说并存照旧（type-closure:167 vs lead 口述）；本结案后冻结态
   语义已明确（=等走位），变异法对该臂的观测面问题不再影响判定。

## 对 BUG-008/测试资产的连带提示

- l2_prologue/bug-ledger 的"死锁/冻结"表述按本文更正为"等待走位"（台账行由 lead 翻格）；
- 21k ticks 三输入脚本"收敛同一冻结态"的原判据失效：三脚本均未走位到 (3,10)，
  收敛态 = script40 完成态 + 自由玩法等待态，不是挂死。
