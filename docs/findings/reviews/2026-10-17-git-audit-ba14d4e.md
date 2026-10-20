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

### G6【纪律违规·自首】DirAssetSource 合并：git add 撞锁静默失败 + 补救 amend 绕闸（--no-verify，A 协议永禁项）（2026-10-18）

- **事实链**（reflog + tree diff 亲验，A 级）：
  1. `73c9d1b` 首提——`git add` 撞 index.lock **静默失败**（退出码未验）⇒
     shell/common/src/asset.rs 漏入，提交内容残缺；但 pre-commit 总闸跑的是**工作树**
     （asset.rs 在树里）⇒ 9/9 PASS 与提交面脱钩；
  2. `8836409`（现 HEAD）amend 补入——`git diff 73c9d1b 8836409` = 恰 asset.rs +58 行，
     零他变更 ✓ 与其自报一致；但该 amend 用了 `--no-verify` ⇒ **程序绕闸，A 协议永禁项
     违规属实**；
  3. 处置（lead 核准）：同内容过闸重落（最终 sha 待报）+ commit 留纪律更正行 +
     案例入 knowledge + 再犯升级条款。
- **两句并记**（shell-port 自报口径，我核准）：**内容已验证**（amend 树与 73c9d1b 时
  闸验工作树逐字节同，delta 仅缺失件本身）+ **程序曾绕闸**（违规不因内容无害而消）。
  自首、未推送、同内容重落 = 从轻情节；`--no-verify` 本身 = 零容忍项。
- **结构性教训（对抗增量）**：pre-commit 总闸验证**工作树**而非暂存/提交面——`git add`
  静默失败即可使"过闸"与"提交内容"脱钩。防线=提交后三对账（`git show --stat` vs
  声称 vs 写范围，本审计表 G5/§四款的固定核对项）恰能抓住此类残缺——本案即其价值实证。
  附带：`git add` 必验退出码（与 pathspec 规程并列的知识条素材，lead 落库）。
- **待办**：最终 sha 到达后验闸痕（我复跑总闸 + tree-hash 等价 + DirAssetSource 合并
  本体按 5-2 约束复审：win panic 三锚不漂移/断言零改动/common 单源+双壳别名）。
- **【G6 终章·2026-10-18 复核完结】**最终 sha = **7e7c71f**（lead 冻结令后清洁重造）：
  ①`git diff 8836409 7e7c71f` = **空**（同内容过闸重落 ✓）；73c9d1b/71afdc5/8836409
  三中间 amend 全悬空零引用 ✓；②闸痕复核 ✓（后台复跑总闸见本轮收尾）；③合并本体过
  5-2 约束：win panic 三锚 prefix 断言不漂移（28/28 含 panic_text_*）、B4 锚随迁断言
  零字节（抽样核 missing=0）、common 单源 + win `DirAssetSource as DirAssets` 别名
  （main.rs:65，调用面零改动）、唯一语义统一面=错误诊断包络（`缺资产 {rel}（根 …）：
  {io}`，如实声明、锚前缀保留、缺资产路径恒 panic 终止不变）——**接受**。
  **第三种索引扫荡机制入册**：`git commit --amend`（无 pathspec）= 索引全扫
  （71afdc5 曾卷 lead 集成件 6 件）；收敛解 = `reset --mixed` 回父代 + `git add <路径>`
  （验退出码）+ `git commit -- <路径>`。至此索引扫荡三机制齐：`git add -A`（G1）、
  裸 `git commit`（117aba7）、`git commit --amend`（本 G6）——全部收敛到 pathspec +
  验退出码 + 提交后三对账（已并入 gotcha_shared-workspace-bare-commit-sweeps-index）。

### G7【第四扫荡变体·被卷方】对方 commit 卷走我方在飞工作树面（2026-10-20，37b53a4 案）

- **事实链**（be4fee8 message 更正声明 + git show 亲验，A 级）：shell-port 在飞的
  main.rs 集成面（task-A ②接线/③通道/Shell::set_scale/repaint + hook_fn 适配）被
  **task-13 的 37b53a4 整体卷入**（其 commit 时工作树全绿态被带走，message 未含这些面）；
  shell-port 首提 29ae84f 的 message 声称了不在其内的 main.rs 内容 ⇒ 声称≠内容。
  处置：未推送 pathspec amend（内容不变）→ be4fee8 message 前向更正 + **分域清单**
  （view.rs=be4fee8 / main.rs 面=37b53a4，两 commit 合观=完整交付）。我核：37b53a4
  的 main.rs diff 含 canvas_graphics(scale)/Shell::set_scale/repaint/blit_1to1 接线/
  headless all 全符号 ✓ 零丢失；be4fee8 树=view.rs 75+/4− 干净 ✓；29ae84f 悬空 ✓。
- **机制归类**：前三机制（add -A / 裸 commit / amend 索引扫）都是**己方扫人**；本变体
  是**被扫**——pathspec 只防自己卷人，防不了他人全树提交带走你的在飞面（提交单位是
  索引/树的对称推论）。
- **防线**：在飞面**占位 commit 或知会制**——跨任务共享树并行时，长时在飞集成面先落
  占位 commit（可后续 amend）或显式知会队友；三对账时 `git show --stat` 出现
  **非己方域文件**即停（被卷/卷人的第一信号）。
