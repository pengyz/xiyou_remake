# 对抗抽查：git log/diff 审计（ba14d4e 结构迁移 commit）

> 审查人：adversary。对象：ba14d4e（refactor(repo)：game-desktop → shell/win）及其后工作区。
> 性质：主动抽查（非受托审查）。发现 1 阻塞 2 建议。

## 阻塞

### G1【工作区产物入仓 + 声称与内容不符】save/MOT_IF.bin、save/SKY_WAR.bin 被提交

- **事实**（A 级）：`git show ba14d4e --name-status` 显示 `A save/MOT_IF.bin`、
  `A save/SKY_WAR.bin`（二进制新增）；`.gitignore` 无 `save/` 条目。
  这两个文件是**用户实机试玩的存档产物**（审查前工作区里还是 `?? save/` 未跟踪状态），
  与 AGENTS §2"analysis/ assets/ target/ 可再生工作区不入库"同性质——save/ 是运行时
  存储工作区，不是源码/夹具。
- **证伪点**：该 commit 正文的 Evidence 只列 ①构建零错②测试全绿③git mv 保留历史，
  **未提"新增两份用户存档二进制"**（gotcha_commit-claims-must-match-content 前科在案）。
- **风险**：用户试玩状态进版本历史；后续 Rms::load 若以仓库根为 save 目录会被污染
  （测试用 temp_dir，主程序默认 exe 目录\save，当前无直接耦合——但根目录 save/ 的
  存在本身会误导）。
- **整改**：`git rm --cached save/MOT_IF.bin save/SKY_WAR.bin` + `.gitignore` 加
  `save/`（若有测试夹具需要样例存档，应放 crates/*/tests/fixtures/ 并写明来源）；
  commit 说明写明是撤销误入产物（历史不改写，证伪更正文化）。

## 建议

### G2【陈旧注释】game-core/src/audio.rs:215-216 与生产接线矛盾

注释称"hideNotify 整体尚未接入任何宿主路径"，但 shell/win 已生产接线：
main.rs:1067-1068 每拍轮询 `set_minimized` → `hide_notify`（main.rs:581-612，
含 sc.hide_notify() 全臂 scene.rs:2821-2827）。android-shell 方案引用时一度被误导
（其引用反而正确）。按证伪更正文化**原地标注更正**，勿删原文。

### G3【注释与代码 1 层出入】"55 层全量装载"实载 54

main.rs:118 注释"55 层全量装载"，但装载循环 `skip(1).take(54)`（main.rs:130）只载
maplv1..maplv54；assets/raw 实有 maplv0..maplv54 共 55 文件，maplv0 未被载。
哪边为真未考证（**C 级**：maplv0 可能是死资源）——建议考证后修注释或补载，勿让
"55 层"数字继续被下游方案转引（android v1 已转引）。

### G4【工作区漂移·登记】reference/oracle/_diff/report.md 被再生成改写（未提交）

2026-10-17 23:4x 发现该文件工作区内容与 HEAD 分叉（A/B/C 三变体 trace 哈希全变、
tick 数 500→150，判定仍全 PASS）。来源疑为某次门禁/差分 harness 再生成（含我启动后
中止的 `gates/cli.py status` 后台任务的可能副作用）。属再生成产物漂移，**不应随手
提交**；建议持有者 `git checkout -- reference/oracle/_diff/report.md` 复位，或按
再生成口径显式提交并说明 tick 数变化缘由。我按职责不改非审查文件，仅登记。

### G5【117aba7 索引竞态事故·拆分终验】全四项通过（字节级，2026-10-18 01:0x）

裸 `git commit` 卷走队友暂存件（Unit 2，G1 同族第二起）的守卫式拆分终态核验：

| # | 检查 | 手段 | 结果 |
|---|---|---|---|
| ① | 117aba7 零引用 | `git rev-list --all` / `git log --all` 双查 | ✅ 无输出（悬空对象） |
| ② | 壳测逐行等价 | `git diff 117aba7 9f63a63 -- shell/win/src/{main,audio}.rs` | ✅ **0 行 diff**（字节级相同，强于"逐行等价"声称） |
| ③ | 卷入件归还零损失 | `git diff 117aba7 485f542 -- <两文件>` = 0 行；`process_control_contract.rs` blob 同为 `bf9dfd6a…`；Unit2 lib.rs 增量 diff(a859408→117aba7) == diff(a859408→485f542)（1176==1176，`-ceq` True） | ✅ 字节级零损失 |
| ④ | 三对账 | 9f63a63 = 2 文件 517 insertions 零删除、message"行为锚 26->38"与内容写范围一致；485f542 = 2 文件 103 insertions（+17/+86=620-517 ✓） | ✅ 通过 |

链：9f63a63 → 485f542 → a859408 → 41b5c93。
【2026-10-18 更正】核验③首跑我用错基线父（拿 41b5c93 当 117aba7 的父比 lib.rs 增量，
得 2808≠1176 的假异常）——117aba7 真父是 a859408（`git log --format=%P -1` 核），
换正确父后逐字节相等。方法教训：增量等价比较必须先核 `git log --format=%P` 取真父。
另登记小疵：a859408/485f542 subject 带 UTF-8 BOM 前缀（PowerShell 捕获产物），message
卫生项，不改写。
