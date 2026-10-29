# 设计件对抗复核：双操控方案 + 功能键面板（STAGE-B-007 / 008）（2026-10-29）

> 对象：`docs/superpowers/specs/2026-10-29-dual-control-schemes-design.md`（未提交稿）。
> 基线：HEAD `0af1d1e`。
> 复核者：独立对抗复核子代理。只读，除本文件外未改任何文件，未提交。
>
> **总判定：不能按现稿进入 plan。**
> - §0 背景事实大部分成立（见"核对通过的事实"）。
> - 但有 3 处 Critical：
>   - 点地寻路首步不判碰撞结果，会穿墙、穿怪、站上炼丹炉（运行时实测）；
>   - 撤销对战斗的逆放是坏的，且撤销日志从不清栈（运行时实测）；
>   - "你无法战胜它"浮层是 classic 移植缺口，spec 误写成"原版静默"。
> - 前两条直接推翻 §4.2"撞一步复用 try_step"和 Q8"误操作靠撤销"这两个前提。

## 1. 范围与方法

| 手段 | 内容 |
|---|---|
| 事实核对 | §0 每条逐一对照 `reference/src/deobf/a.java`、`reference/seed/a.java`，以及 Rust 现码（文件:行见各条） |
| 运行时探针 | 临时宿主测试 `shell/android/tests/zz_adv_probe_1029.rs`。路径：真资产 → `Runtime::step` → 新游戏开场链 → 首个 Map，再施加条件，调用生产入口 `GameScene::walk_to` / `native::scene_tap` / `undo_ui::undo_last`。命令：`cargo test --offline --release -p game-android --test zz_adv_probe_1029 -- --nocapture --test-threads=1`。**跑完已删除**，`git status` 确认源码树无残留；不计入 ratchet |
| 隐藏通路探针复跑 | `target/verify/probe_hidden_paths.rs` 临时复制为 `shell/android/tests/zz_probe_hidden_paths_tmp.rs`，跑完即删 |

探针编号（下文引用）：

| 编号 | 条件 | 实录（A 级） |
|---|---|---|
| A | 1 层。玩家置于 (8,2)，(9,2) 放型 70，玩家 hp=1（打不过）。`walk_to(10,2)`（find_path 经怪格，steps=[2,2]） | 玩家 px 256→288，**站到怪物格 (9,2) 上**（Map，wp0）。怪物 state=0，未开战 |
| B | 同上，(9,2) 放型 41，hp=5000，`walk_to(10,2)` → 战斗胜 → 连续 3 次 `undo_last` | 战后 undo_len=3。逆放 Combat 后：hp/gold 回滚；**实体 state=0 但 removed=true、格槽已空**；`enemies.hp[26]` **1200→0**（实体槽号 26 被当成 type−41 下标）；`enemy_hp_before=0` |
| C | 23 层。路线第 3 步途经 t12 (2,4)，目标 (2,5) | 停在 (2,3)，t12 显形为型 11，未进墙（非首步走 `advance_script_walk` 臂，撞格清步） |
| D | 1 层点地走一段 → `change_floor(2)` → `undo_last` | 换层后 undo_len 仍为 1；撤销把玩家传到 **1 层坐标 (8,2)**（已在 2 层） |
| E | 23 层。玩家 (5,3) 紧邻 t12 (5,4)，`walk_to(5,5)` | 轨迹 (5,3)→(5,4)→(5,5)：**穿过隐形墙**，墙显形为型 11 |
| F | 玩家 (8,2)，(9,2) 放炼丹炉型 9，`walk_to(10,2)` | **站上炉格 (9,2)** |
| G | 1 层 ≥5 步点地寻路途中，逐拍查 `field_tap_allowed`，再调 `scene_tap` 点起点格 | 12 拍里 `field_tap_allowed` 恒 false（拍间 walk_phase 恒 1）。`scene_tap` 返回 **Some(-5)**（点空白代按），不会改道 |
| H | 1–54 层逐层：BFS（地形∨槽）可达集 vs `find_path` | 37 层 (1,1) BFS 可达，find_path 未命中（100 扩展上限）。51 层 12×13 非方形 |
| I | 炉 (8,1) 在玩家 (8,2) 正上方，`walk_to(8,1)` → 炼丹炉页按 −7 | 进 Alchemy 时 wp=1 残留。离开后回到 Map **走进炉格 (8,1)** |
| M | 14 层。模拟三只 53 已死、(11,3) 踏板已踩，从 (11,3) 点 (8,3) | 末块踏板在路线终点触发 wp4（开封印门），`finish_walkto` 回 Map 后正常。**未**构造出"中途 wp4 卡死"，见 m10 |
| HP | 隐藏通路探针复跑 | 与 §0 一致：54 层 12/46 格，`walk_to(4,2)` 10 步、显形 5 格；23 层 t12 24 处 |

## 2. 意见表

证据分级：A = 字节码 / 源码逐行 / 运行时实测；B = 静态推理；C = 假设（已标明）。

### Critical

| 编号 | 意见 | 证据 | 建议修法 |
|---|---|---|---|
| C1 | **点地寻路首步不判碰撞结果，会"穿格"。** dp3 消费机 wp0 先置 `walk_phase=1` 再 `try_step`，不看返回值；`try_step` 只在"有浮层 ∧ wp==1"时复位。所以首步被挡、又不弹浮层时，玩家照样走进去：打不过的怪（Rust 缺浮层，见 C3）、隐形墙 t12、炼丹炉 9、实地形墙、联动门 32 都会这样。第 2 步起走 `advance_script_walk`，撞格会清步（探针 C 正常）。<br>对 spec 的影响：<br>① §4.2"撞一步复用 try_step、终点到撞击之间不插入额外拍"——邻接目标（点身边的怪 / 门 / 炉 / 墙）的撞击正好是首步，若把撞击步塞进寻路缓冲就会穿格；<br>② §4.1"隐形墙 = 地面，碰上后效果与手动走完全一致"不成立；<br>③ §8"23 层隐形墙沿路照常撞到并显形"只断言显形，抓不住穿墙。<br>现网已可触发：find_path 经怪物槽（探针 A）；邻接隐形墙时点对侧（探针 E） | A：`scene.rs:2903-2907`（wp0 `walk_phase = 1; self.try_step(..)`）；`scene.rs:5156-5177`（try_step 复位条件）；`scene.rs:5956-5971`（advance_script_walk 撞格清步）；Java 同式 `a.java:4080-4084`；探针 A / E / F / I | ① 撞击步不得经 dp3 wp0：0 步时以 walk_phase 0 直接 `try_step`；≥1 步时作为缓冲末步，走 `advance_script_walk` 臂。② 0 步撞击也要开 route 段，否则 `battle_undo_snap` 不记（scene.rs:5408 `if self.route.is_open()`），见 C2。③ 现网缺陷登记 bug-ledger（Stage B 触屏）。④ 测试矩阵：{怪(可胜/不可胜), 门(有/无钥), 炉(朝上/横向), 墙, t12, 83, 32} × {邻接, 远端}，断言玩家格与相位 |
| C2 | **撤销对战斗的逆放是坏的，日志也从不清栈**，Q8"误操作靠撤销"和 §0"误触开战可撤回"失去前提。<br>① 被杀怪物死亡动画结束后已摘槽（removed=true）。`revive_enemy` 只写 type / state=0 / visible，不恢复 removed，也不重新登记格槽 ⇒ 怪物不画、不挡路。撤销 = 白杀一只怪，外加回血回金。<br>② `revive_enemy` 用实体槽号写 `enemies.hp`，而这张表按 type−41 索引 ⇒ 破坏全局怪物属性表（实测型 67 的 HP 1200→0）。<br>③ 战斗快照取的是赋值前的 `battle_enemy_hp`（旧值）。<br>④ 生产代码对 `UndoJournal::clear` 零调用，`begin_combat` 零调用 ⇒ 换层 / 存档 / 剧情后仍可撤回，玩家会被传到别层坐标（探针 D）。<br>2026-10-23 终审的"清栈 ✅"只测了 journal 本身（测试自证）。<br>§6 把撤销放进工具栏常驻，暴露面扩大 | A：探针 B / D；`shell/common/src/undo_ui.rs:36-46`；`scene.rs:5409-5422`（快照在 :5413，赋值在 :5422）；`entity.rs:535-541` detach 置 removed；`scene.rs:6405` 绘制跳过 removed；`rg "undo\.clear\|begin_combat" crates shell` 仅测试命中 | 本件之前另案修复并登记 bug-ledger（现网隐形热区已可触发）：<br>- 复活要恢复 removed 与格槽，hp 写回 `battle_enemy_hp` 语义而不是属性表；<br>- 快照取赋值后的值；<br>- 换层 / 存档 / 脚本事件接 `clear`。<br>测试必须走生产路径：`walk_to` → 战斗 → `undo_last` → 再撞同格必须开战。<br>Q8"自动战斗"排在修复之后 |
| C3 | **"你无法战胜它"是 classic 移植缺口，不是"原版静默"。** 原版在不可胜分支弹 kind 0 浮层（a.java:5489）；Rust 只留了注释。<br>§4.2"原版静默挡住，提示未移植"前半句事实错误。<br>这属于 Stage A 保真偏离（Rust 少了原版行为）：既不是原版 bug，也未登记。<br>Java 里这个浮层恰好是避免 C1 穿怪的机制（浮层 ⇒ try_step 复位 wp）。<br>docs/spec/gameplay.md R-battle-4、fixture-plan-boss FX-BOSS-68B 都写了此浮层，但 trace 门禁没抓到 | A：`a.java:5487-5490`；`scene.rs:5426-5432`（`// "你无法战胜它" 提示浮层：交互批次`）；`rg 无法战胜 crates` 零命中 | ① 改 §4.2 措辞。② 登记 bug-ledger（classic 移植缺口），按 BUG-037 体例移植。修复后 classic 向 Java 收敛，trace 基线须说明。③ 列入 §9 前置。④ 移植后再定 modern 的"打不过"表现（大概率原版浮层即够，"开战时闪红"可删），避免与原版浮层叠加 |

### Major

| 编号 | 意见 | 证据 | 建议修法 |
|---|---|---|---|
| M1 | §3.3"寻路 / 行走中"和"战斗中"两行的"背包 / 菜单"在点地寻路时无效。原因：`walk_to` 把 game_mode 置 Dialog；tick 遇 Dialog 早退，够不着 −7/−6 分派。壳层先 `interrupt_route` 再 press，实际效果只是"停下"。由点地寻路撞出的战斗在 Dialog dp3 wp5，同样无效。只有手动行走 / 手动撞怪（Map）时 −7/−6 才生效。结果是标着"可用"的按钮无效果，违反 §8 自己的一致性判据 | A：`scene.rs:1934`（`game_mode = Dialog`）、`:4823-4838`（Dialog 早退）、`:4856-4867`（−7/−6 分派）、`:2951`（dp3 wp5）；`native.rs:315-323` replay 先 interrupt | 情境按 game_mode + `walkto_owns_dp3` + 战斗来源细分。若要"寻路中按菜单 = 停下并开菜单"，写明壳层两拍合成（先停、到 Map 后再发 −7），并加测试 |
| M2 | "行走中点别处 = 改道"和"战斗中点按 = 闪红叉、零状态变化"与现码不符：<br>- 寻路途中拍间 walk_phase 恒 1 ⇒ `field_tap_allowed` 恒 false ⇒ 点按走 `blank_tap_key`，Dialog 时返回 −5 ⇒ 实际是"停下"（探针 G）；<br>- 点地寻路撞出的战斗中点画面，同样是 −5 + `interrupt_route`，会清掉战后续走——不是"零状态变化"；<br>- §4.3 又说 `field_tap_allowed` / `blank_tap_key` 不变，二者矛盾 | A：探针 G；`scene.rs:1955-1963`、`:9564-9575`；`native.rs:332-397` | 三选一并写进 spec：<br>a) 点按排队到格边界再改道；<br>b) 放宽 BUG-038 门（须回归 BUG-038 / 040 用例）；<br>c) 接受"行走中点按 = 停下"，改表。<br>战斗中点按要么真正零变化（不代按 −5），要么改表 |
| M3 | §8 一致性判据"可用 ⇔ 按下状态有变"自相矛盾：<br>- §3.2 隐藏的是"同义键"（设置页 −6 仍返回，对话 dp1 任意键都翻页），隐藏 ≠ 按下无效；<br>- M2（10-28）承认任意键都有副作用（覆盖 key_held，战后不续走），摇杆灰键照发 ⇒ "不可用"键按下状态会变；<br>- GameScene 不是 Clone，也无快照接口；<br>- 判据若由 controls() 推导期望值即测试自证。<br>"隐藏无功能钮"与"灰键照发"两条规则在外观层并不冲突（实体键 vs 屏上钮），冲突在判据层 | A：`scene.rs:4597`（`-7 \| -6 =>` 返回）、`:2869-2873`；`scene.rs:239`（GameScene 无 derive Clone） | 判据改为两条：①"标可用 ⇒ 按下产生所标功能"；②"隐藏 ⇒ 不画不发"。另定义快照域：排除 key_value / key_held / 反馈计时。快照接口列为前置任务。裁判 = 克隆场景实跑 tick，期望不得来自 controls() |
| M4 | controls()"紧贴各 tick 臂、不另写第三份按键语义"与"classic 零变化"之间没有落点：不另写就得把 tick 臂改成表驱动（动 classic 代码）。trace / visual 只覆盖已录场景，不保证每个臂分支 | B：`scene.rs:4690-4867` 各臂为手写 match；game-oracle 场景覆盖有限（`crates/game-oracle` 无 walk_to 引用） | 写明"classic tick 臂一行不动、controls() 独立实现、以实跑一致性测试为裁判"，或给出重构臂的逐臂 trace 证据要求 |
| M5 | 表中开机侧情境（读档列表、片头、标题设置 / 帮助）属于 BootMachine（paint.rs），§2 只写 game-core `controls()`。另外漏了"关于"（mode 17，IconView）、开机 logo / 退场动画、Loading。10-28 M3"双移植面"教训未落到本件 | A：`paint.rs:842-860`（Help / IconView 臂）；10-28 复核 M3 | 明确 GameScene / BootMachine 两个查询面，共享功能 ID 枚举；补漏行；两面各自加一致性测试 |
| M6 | kind 4 / kind 3 的子类表述不准，按图标分会错：<br>- 型 77（src 0）的报价（flags&2，如送 1000 金、送 13 火眼金睛）图标 (0,0)，归"告示"后"关闭(−7)"= 拒绝，而点空白 −5 = 接受；<br>- 型 78 的告示（flags&2=0）图标 (1,2)，显示"买 / 不买"，两键实为同效；<br>- 一次性（flags&1）两键都消耗实体；<br>- 说明类道具 15/23/24/25（kind 3，图标 0,3）的 −6 也会落 `use_item_stack`。<br>10-28 §6 已约定"告示类 = 确定、说明类 = 关闭"，本件未沿用 | A：`scene.rs:5443-5466`（src / 图标）、`:4412-4580` apply_merchant_offer（flags 分支）、`:4730-4743`（kind 3 −6 无条件 use）、`:3429-3436` | 情境由 (kind, flags, src, 图标) 派生；逐子类写表行与测试 |
| M7 | §4.1"可见实体"未定义：<br>- interact / `cell_entity` 只看 state≠1，不看 removed / visible（81 例外）。刚开的门、刚杀的怪在动画期 visible=true、槽仍在：按 visible 判会让它们短时挡路，按槽判又回到泄露；<br>- 类目要取 `scene.entity_category`：50 层型 72 = 8（怪），而 §0 写 32；<br>- 多格实体（69 宽 3 格）怎么选中 / 邻格未定；<br>- 类目 32 (72/84/87) 与道具格不在 §4.2 目标表；<br>- 沿途"可经过"道具会被顺路全部拾取（含装备，40 号会中途改变陷阱判定）。<br>"类目 4 = 道具"成立（装备 33–40/79/80，拾取臂 `2 \| 4`）。<br>型 5/76 不可见按地形：这两型是踏板，本身无危害；但中途触发 wp4 有风险，见 m10 | A：`scene.rs:5199`、`entity.rs:369-385`、`scene.rs:6385-6390`、`:5842-5845`、`entity.rs:225-235`（69 宽 96 px = 3 格） | 定义"存在且可见" = visible ∧ state≠1 ∧ ¬removed；类目用 `entity_category`；补目标表行；写明沿途拾取是预期行为 |
| M8 | 陷阱事实不全，且引用口径错：<br>- 装备 40 同时免 60 的 −200，只保留 60 上爬；§0 写"免 61/62"；<br>- gameplay.md R-scene-4"（60 不免疫）"同错，须原地更正；<br>- §0 引的 a.java:3694-3761 是 seed 行号，与 §0 声明的 deobf 口径不符；<br>- 原式怪癖须同式复刻：62 / 60 的右邻判界用 mapCellsHigh；61 夹击两侧都须在界内；51 层 12×13 非方形 | A：deobf `a.java:5555-5621`；seed `a.java:3694-3750`（`var11_16 == 0` 门包住 60 计数）；`scene.rs:5754-5810`；探针 H | `tap_passable` 的陷阱判定抽成与 `apply_step_cell_effects` 共用的纯函数，或逐式复刻并加对拍测试（含 51 层、装备 40×60）。更正 R-scene-4 与 spec §0 |
| M9 | STAGE-B-006 附加行机制承载不了"仅触屏显示"的动态行，还会丢值：<br>- OptionList 在进页时构造，切换操控方式后自动战斗行不会出现 / 消失（须重建，盒高随之变）；<br>- `Runtime::app_settings()` 按附加行"缺行取默认"拼快照 ⇒ 摇杆模式下 auto_battle 被快照为 false 并同拍落盘，切回触屏时用户设置已丢；<br>- `tips_seen_*` 没有对应行 ⇒ 恒为默认 false ⇒ 每次启动都弹提示卡 | A：`scene.rs:4580-4588`；`paint.rs:865`；`runtime.rs:136-146`；`settings.rs` 只含 5 键 | 新字段不经行快照，单独读写；行的显隐由壳在切换时重建 options（两处设置页），保持高亮；加"切到摇杆再切回，自动战斗保留"和"重启不再弹提示卡"两例 |
| M10 | 怪物选中状态的生命周期只写了"点别处 / 滑动 / 方向键 / 撤销"。未覆盖：换层（实体表重载）、读档、打开菜单 / 背包并使用道具（20 对角瞬移、21 上一层、86 清相邻、18 破全墙）、剧本 MOV 搬怪、怪物被他处击杀、浮层弹出、切换操控方式、hide_notify | B：`scene.rs:1170-1196`（换层重建实体表）；`op_mov` / `tick_entity_route` 搬实体 | 每拍校验（楼层号 + 类型 + 格 + state≠1），失效即清；逐项加测试 |
| M11 | §3.1 的 4 倍公式含糊，还有用户可感知的代价：<br>① s 落在 [s_read, 4) 时取 s 还是退回？"现有公式"用摇杆面板的 V（P=816）还是工具栏视口？<br>② 硬编码 9 列：1080 宽主流机（4 倍需 1152）会退回 3 倍，8 列 ×4（1024）被排除 ⇒ Q13"4 倍"在主流机上不生效；<br>③ `Layout::modern(9,16)` 屏宽 288：HUD 右块 175 px 贴右、左块贴左，是否重叠未验（C 级假设） | B：`portrait.rs:292-321`；`layout.rs:550-575` | 写成单一公式，钉 1080×2400@420 / 1200×2536@480 / 720×1600@320 三档；"8 列 4 倍 vs 退回 3 倍"交用户裁定；288 宽 HUD 出截图验证 |
| M12 | §8 门禁抓不住主要回归：<br>- trace / visual / oracle 不调用 `walk_to` / `scene_tap`，对点地规则零覆盖；<br>- Windows modern 5 帧基线是"自证冻结"，只含键盘态（boot/title/intro/game/menu），已知依赖本机 save（kairos 10-28），也不在 `gates check` 里 ⇒ "不变"证明不了 Windows 点击行为未变。Windows `pointer_tap` 调同一个 core `walk_to`，新规则若落在 walk_to 内，Windows 一并改变，spec 未表态 | A：`rg walk_to crates/game-oracle` 无；`modern-frame-baseline.json` purpose 字段；`shell/win/src/main.rs:686-735` | 补宿主测试：C1 矩阵、C2 生产路径、journal 清栈、find_path 上限（37 层）、BootMachine 查询、Windows 点击（或写明 Windows 保留旧规则并钉住） |

### Minor

| 编号 | 意见 | 证据 | 建议修法 |
|---|---|---|---|
| m1 | §4.2"最近"排序（距离 → 路径 → 行主序）与 `furthest_reachable_toward`（投影 → 距离 → 行主序）主键不同，谈不上"对齐"。新通行若沿用 find_path 内核，会继承 100 扩展上限：37 层有 BFS 可达而 find_path 未命中的格，会误闪红 | A：`pathfind.rs:322-357`、`:109`（扩展上限 100）；探针 H | 写明"替换为 BFS 可达集 + 新排序"，旧函数只留给经典口径 |
| m2 | HELP_TEXT 有两份：`paint.rs:119`（BootMachine，进页 `:844` 与每帧绘制 `:897` 两处读点）和 `scene.rs:3981`（游戏内 `:3988`）。§7 覆盖入口只提一份 | A | 三处读点都接覆盖；classic 逐字测试覆盖两份 |
| m3 | §0 引用漂移：findPath 在 `a.java:9898`（relax `:9967`），9869-9928 是粒子代码（pathfind.rs 头注同错）；陷阱行号见 M8 | A：`sed -n 9865,9870p` | 更正 |
| m4 | "地图空闲"中间按钮可能有 4 个（同层上下梯都邻近时，49 / 55 都可用），超出"0–3 个" | B：`scene.rs:5120-5140` | 定上限或合并 |
| m5 | 情境表细节：<br>- 记录浏览漏了 −1/−2 滚动；<br>- 提示框翻页距为 ±1 行，帮助为 ±1 页；<br>- 结局应在 ending_done 前隐藏"继续"；<br>- 点地寻路触发的战斗中缺"停下"；<br>- 对话只在 dp1/6 收键 | A：`scene.rs:4700-4715`（kind 5）、`:1867-1874`（±1 行）、`:4716-4729`（帮助 ±页）、`:4840-4852`（结局）、`:2846`（dp1/6） | 补行 |
| m6 | 用摇杆面板的确定键（−5 翻转行）切换操控方式：`apply_portrait` 会重建 TouchMapper 并 `route.clear()`，抬起事件丢失；面板消失后手指下方变成视口。§8 只列了"地图中 / 设置页中"两例 | A：`native.rs:543-553` | 加"面板确定键切换"一例，断言 key_held 归零、无误点 |
| m7 | Q9 需要新接手势：Android 视口现只在 Up 记点按，无 Move（`native.rs:595-606`），滑动须保证不再额外产生 Up 点按。观察模式拖动是"模拟 key_held"还是"直写相机"未定（现接口为 wp2 读 key_held，每拍 16 px） | A | 写明手势路由与相机接口 |
| m8 | 提示卡：Windows modern 是否弹未定（若 core 自动弹，会改变 Windows game 帧）；提示超过一页时点空白（−5）即关，后页未读 | B | 由壳显式调用；文案限一页 |
| m9 | BUG-043 R4（击杀后无条件 `route.resume`）是本件"撞怪开战"的必经路径，不宜留到"plan 中再决定" | A：`scene.rs:5666-5692`（`:5691` resume） | 列为前置修复 |
| m10 | dp3 消费机没有 walkPhase 4 臂：寻路中途踩到组内最后一块踏板（型 5 / 76 置 wp4 开门动画）理论上会卡在 Dialog dp3。B 级，探针 M 未构造出中途情形（终点情形正常） | B：`scene.rs:2896-2953`（dp3 只有 wp 0/1/5 臂）、`:5865-5872`（型 5 置 wp4）、`:5900-5906`（型 76 置 wp4） | plan 中构造中途用例；或在 tap_passable 里把不可见 5/76 视为"仅可作目标" |

## 3. 须登记 / 更正

- **bug-ledger 新增**：
  - C1 点地寻路首步穿格（Stage B 现网）；
  - C2 撤销战斗逆放损坏 + 日志不清栈（Stage B 现网）；
  - C3 "你无法战胜它"浮层未移植（classic 移植缺口）。
- **原地更正**：
  - `docs/spec/gameplay.md` R-scene-4"（60 不免疫）"；
  - spec §0 陷阱条与 findPath 行号；
  - `pathfind.rs` 头注行号（代码注释，非本复核可改）。
- **stage-b-deviations**：STAGE-B-007 / 008 新节；触屏寻路 / 撤销（touch-native-undo）在登记册里找不到对应节（STAGE-B-004 编号空缺，去向未核实）。

## 4. 核对通过的事实

- `pointer_hit`（`scene.rs:9578`）只覆盖菜单 / 商店、存档 / 读档槽、设置行、背包格；`blank_tap_key`（`:9564`）只在对话 / 结局 / 非是否框代按 −5。
- `scene_tap`（`native.rs:332`）→ `classify_tap`（`touch.rs:138`）：可走格 → `walk_to`（`scene.rs:1901`）→ `plan_route`（`pathfind.rs:257`）；墙格 → 从玩家位按主轴 `probe_step`；右上 56×20 撤销热区（`touch.rs:56`）。另：`plan_route` 的墙格分支在生产路径不可达，`plan.probe` 无人执行。
- find_path 通行 = 地形 ∨ 槽（`pathfind.rs:174`；Java `a.java:9967` 同式）。
- 54 层 12 / 46 格、10 步、显形 5 格；23 层 t12 共 24 处；5/9/12/14 层路线途经门、怪（另含道具、封印门 4、踏板 5）。
- 类目表与 `legacy_render_category_mirror` 一致；载入置不可见的是 5/81/6/12/76/82/83（`a.java:9514-9625`）。
- 观察模式不需道具，持 13 才画数字（`scene.rs:5098-5114`）。
- 炼丹炉仅朝上进 mode 9（`scene.rs:5326-5338`）。
- Xiaomi 14：s_b=5（1200×1600）；scene 3 倍 12×16（1152×1536）；面板 936；去掉面板仍是 3 倍（V=2248 时 s_whole=2、s_read=3）；4 倍 = 9×16 = 1152×2048，放得进 2248。
- `replan_portrait` → `adopt_portrait` → `Runtime::apply_layouts` → `GameScene::apply_layout`（`native.rs:1135/1210`，`runtime.rs:172`）；`set_layout_dims` 会 `center_on_player`，相机随之重钳。
- 战斗分支在 route 开放时记 `PlayerVitals`（`scene.rs:5408-5419`）。"记录"成立，"可撤回"不成立（C2）。
- 原版设置页 −6 / −7 同义（`a.java:4146-4148`）。

## 5. 未能核实

- m10：寻路中途 wp4 卡死（未构造出真实数据场景）。
- 288 宽 HUD 是否重叠（M11③，C 级）。
- STAGE-B-004 的去向；game-oracle state dump 是否会枚举 GameScene 新增字段（若按字段反射导出，会影响 `state-dump-golden.json`）。
- `gates/cli.py status` 本机运行超时（>120 s），未取得门禁现状。
- 实机行为（按要求未连 adb）。

## 6. 第二轮复核（v1）

> 对象：spec v1（§11 处置表）+ 未提交的 `docs/bug-ledger.md` BUG-044/045/046、`docs/spec/gameplay.md` R-scene-4 更正、
> `docs/spec/input-operations-inventory.md` §8 更正。基线 HEAD `0af1d1e`。复核者：第二轮独立对抗复核子代理。
> 只读：本节是唯一写入；未提交、未连 adb。
>
> **总判定：仍不能进入 plan。** 1 Critical、7 Major。
> - Critical：新发现的现网穿墙缺陷（R2-1）。v1 的 0 步撞击和滑动都直接建在这个入口上。
> - Major 中最要紧的是 R2-2 / R2-3：C1 的撞击语义修法（§5.2）在炼丹炉上实测失效，而且没有覆盖 BUG-044 的隐形墙情形。
>   R2-4：M2 的排队改道以 `field_tap_allowed` 为真作触发，但寻路途中这个条件永远不成立（§0 自己也写了"恒 false"）。

### 6.1 方法

| 手段 | 内容 |
|---|---|
| 静态核对 | v1 §0 / §3.2 / §5 / §8 每条事实对照 `reference/src/deobf/a.java` 与 Rust 现码，行号见各条 |
| 运行时探针 | 临时宿主测试 `shell/android/tests/zz_r2_probe_1029.rs`。1 层真资产 → 开场链 → 首个 Map；为驱动控制流，把 (2,4)–(4,4)、(2,5)、(2,6) 的地形强行置可走（仅 R2-A 用到）。命令：`cargo test --offline --release -p game-android --test zz_r2_probe_1029 -- --nocapture --test-threads=1`。**跑完已删除**，`git status` 确认源码树无残留；不计入 ratchet |

| 探针 | 条件 | 实录（A 级） |
|---|---|---|
| R2-A | 玩家 (2,6)，炉（型 9，宽 3 格）放 (2,4)，`walk_to(2,4)`，steps=[1,1]（撞击是第 2 步，走 `advance_script_walk` 臂） | 40 拍内**从未处于 Alchemy**。终态 Map wp0、玩家 (2,5)；`last_transition = (Dialog, Alchemy)`，transition_count 2→3 ⇒ 进过炼丹炉，同一拍被 `finish_walkto` 改回 Map |
| R2-B | 玩家 (2,6)，(2,5) 放型 41（可胜）。Map wp0 下 `route.begin` 后 `facing=1; try_step(1)`（v1 §5.2 的 0 步撞击） | 撞后 **Map wp5**、route 挂起；撤销栈 +1（`route_event_suspend` 推入 from==to 的零长位移笔）。胜后 `route.resume` 重开段，Map wp1 步进；步末 Map wp1 臂 `route_arrived` 收段（`scene.rs:4919-4924`） |
| R2-C | 玩家 (2,5)，`facing=3`（左），`probe_step(0)`（向下） | ok=true，玩家**向左**走到 (1,5) |
| R2-C2 | 玩家 (1,5)，`facing` 指向不可走的 (1,6)，`probe_step(1)`（向上，空格） | ok=true，玩家走进 **(1,6)，地形不可走**（穿墙） |

### 6.2 意见表

#### Critical

| 编号 | 严重度 | 意见 | 证据 | 建议修法 |
|---|---|---|---|---|
| R2-1 | Critical | **`probe_step` 不写 `facing`，现网可穿墙。**<br>`probe_step(dir)` 拿 `dir` 判目标格（`try_step(dir)`）。但通过后 Map wp1 臂按 `self.facing` 步进，这个值是上一次的朝向。所以玩家可能朝另一个方向走一整格，而且这一步不做碰撞判定。<br>现网触发路径：点一个远处的墙格，`classify_tap` 按主轴发 `ProbeStep`（`touch.rs:156-167`）。只要该方向的邻格空，而旧朝向指着墙，就会穿墙。Windows 点击和滑动同样触发（`Dispatcher::swipe`）。<br>另外 `interact_with_cell` 的炼丹炉判的是 `self.facing == 1`（`scene.rs:5330`），所以朝上探步进不了炉，横向探步在旧朝向为上时反而会进炉。<br>对 v1 的影响：§5.2"0 步撞击 = 直接 `try_step`"和 §5.3"滑动 = `probe_step`，与方向键撞格同语义"都不成立。方向键路径是先 `self.facing = facing` 再 `try_step`（`scene.rs:5150-5152`） | A：探针 R2-C / R2-C2；`scene.rs:1942-1948`（probe_step 无 facing 写入）；Map wp1 臂按 facing 位移（`scene.rs:4880-4905`） | ① 登记新 BUG（Stage B 触屏现网，穿墙），列入 §2.1 前置。修法：`probe_step` 在 `try_step` 前置 `self.facing = dir`，与 `handle_field_input` 同式。<br>② §5.2 0 步撞击写明"置 facing 后 try_step"。<br>③ 测试：{旧朝向 ∈ 四向} × {探步方向}，断言终点格 = 探步方向邻格或原地；炉 × {朝上探步、横向探步 + 旧朝向为上} |

#### Major

| 编号 | 严重度 | 意见 | 证据 | 建议修法 |
|---|---|---|---|---|
| R2-2 | Major | **C1 处置不成立：≥1 步时把撞击作为路线末步、走 `advance_script_walk` 臂，炼丹炉会被吞。**<br>经过：步末 `apply_step_cell_effects` → `handle_field_input`（Dialog）→ `advance_script_walk` 弹出末步 → `try_step(1)` → `transition_to(Alchemy)` → 返回 false。此时步数已为 0，直接 return。dp3 wp1 臂随后见"步数 0 ∧ 寻路占用"，调用 `finish_walkto`，把 `game_mode` 直写为 Map。<br>另外 `(Dialog, Alchemy)` 不在 `TRANSFER_EDGES` 里，debug 构建（`cargo test` 默认）会在 `transition_to` 的 `debug_assert` 处 panic。所以 §9 的"炉（朝上）× 远端"用例在 debug 下会直接崩，release 下则是静默吞页。<br>"效果与按方向键撞同一格一致"对炉不成立。其余撞击目标（门、商人、墙、怪）不改 game_mode，没有这个问题 | A：探针 R2-A；`scene.rs:2939-2946`（wp1 步末 `finish_walkto`）、`:1993-1999`（直写 Map）、`:5956-5971`、`:5326-5338`；`scene.rs:711-779`（无 Dialog→Alchemy 边）、`:1503`（debug_assert） | 统一撞击语义：**撞击一律在 Map wp0 执行**。路线只走到停靠格，由 `finish_walkto` 回 Map；同拍或下一拍再按 0 步撞击处理（置 facing、开段、`try_step`，见 R2-1）。这样也不用给经典边表加 Dialog→Alchemy 边。§9 BUG-044 矩阵的炉行在 debug 与 release 下都要跑 |
| R2-3 | Major | **BUG-044 修法只覆盖"计划内的撞击步"，漏了"计划内可走、实际被挡"的步。**<br>v1 §5.1 把不可见实体当不存在，所以隐形墙 t12 = 地面，BFS 可以经过它。玩家紧邻 t12、点对侧时，t12 就是首步，仍经 dp3 wp0 不判返回值，玩家照样穿墙。这正是 BUG-044 自己的复现第 2 条（探针 E）。<br>dp3 wp0 不判返回值的弹步还有两处：<br>- 某步踩到 60 扣血（`step_triggered`）时，步末不调 `advance_script_walk`，wp 置 0，下一步回 dp3 wp0；<br>- wp4 开门动画结束后（由 paint 清 wp，见 R2-12），下一步也回 dp3 wp0。<br>所以"第 2 步起走 advance_script_walk"并非总成立。§9"23 层隐形墙沿路撞到即停、玩家不进墙格"在邻接情形会红 | A：探针 E（第一轮）；`scene.rs:2901-2907`；`scene.rs:5911-5923` 与 Java `a.java:5731-5747`（`!var4` 门）；B：wp4 路径（`scene.rs:6979-7001`） | 在 dp3 wp0 臂加**仅寻路占用时生效**的分支（`walkto_owns_dp3()`）：弹步 → 置 wp=1 → `ok = try_step` → 若 `!ok ∧ wp ≠ 5`，清步数并置 wp0，下拍走 `finish_walkto`。<br>经典 MOV 不开 route 段，所以 trace 零变化；在 commit 中写明。<br>BUG-044 条目的"首步"改为"所有经 dp3 wp0 弹出的步（首步、陷阱扣血后、wp4 后）"。矩阵补"隐形墙在途中首步 / 陷阱后一步" |
| R2-4 | Major | **M2 方案 a 的触发条件不可达。**<br>v1 §5.2 写"到下一个格边界（`field_tap_allowed` 为真）时改道"。但每个格边界都在同一拍内被 `advance_script_walk` 弹出下一步并置 wp=1，拍间 wp 恒为 1。v1 §0 自己也写了"寻路途中拍间 walk_phase 恒 1 ⇒ field_tap_allowed 恒 false"。<br>结果：排队的点按要等整条路线到达、Map wp1 走完末步后才执行，变成"到了再去新目标"。<br>"停下后打开"（§3.2）也缺失效条件。当前步若撞怪开战、踩剧情点或楼梯，"回到 Map"可能发生在战斗或剧情结束之后，届时背包 / 菜单会突然弹出 | A：探针 G（第一轮）；`scene.rs:1955-1963`；`scene.rs:2925-2946` + `:5911-5923`（同拍续弹） | 改为"点按 ⇒ 立即 `interrupt_route`（BUG-040 排空：走完当前步后 `finish_walkto` 回 Map wp0）+ core 记下待执行目标；`field_tap_allowed` 首次为真的那一拍执行 `walk_to`"。这条路径与 BUG-038 / 040 的门不冲突，排空本身就是 BUG-040 的路径。<br>失效条件（排队改道与"停下后打开"共用）：排空没有在当前步结束时回到 Map wp0（开战、剧情接管、换层、浮层），或期间有任何新输入 ⇒ 丢弃。逐条加测试 |
| R2-5 | Major | **"战斗来源"按 game_mode 区分，与实际状态不符。**<br>0 步撞击开的战斗是 Map wp5（route 挂起），不是 Dialog dp3 wp5。最常见的"选中邻格怪 → 开战"恰好是 0 步，按表会落进"手动撞出的战斗（Map wp5）"行，显示背包 / 菜单。远端撞出的战斗则全隐藏。<br>§5.2"寻路撞出的战斗中点画面 = 不代按 −5"也因 mode 而异：Map wp5 时 `blank_tap_key` 本来就返回 None；Dialog 时返回 −5。<br>采纳 R2-2 后，所有点地撞出的战斗都在 Map wp5，"Dialog dp3 wp5"行会消失 | A：探针 R2-B（Map wp=5，suspended=true）；`scene.rs:9564-9575` | 战斗来源改由 `route.is_suspended()`（或专用标志）派生。定下点地战斗在 Map wp5 时背包 / 菜单是否显示（原版 Map wp5 允许 −7 / −6），据此删并 §3.2 两行 |
| R2-6 | Major | **§7 Windows 撤销的前提是错的。**Windows 没有撤销快捷键；右上 56×20 在 Windows 上是**画出来的可见按钮**（`draw_undo_button`），不是隐形热区。按 §7 去掉 `undo_zone_hit` 后，Windows 就没有任何撤销入口了 | A：`shell/win/src/main.rs:1322-1335`（绘制撤销钮）、`:673-677`（唯一 `A::Undo` 消费点）；`rg "Undo\|KeyZ" shell/win/src/main.rs` 只命中这两处 | Windows 保留可见撤销钮，或明确新增快捷键并加测试；改 §7 措辞 |
| R2-7 | Major | **M9 处置自相矛盾，照字面实现会让开关失效。**<br>§8 说新字段"不经附加行快照，由壳直接读写"，又说设置页新增「操控方式」「自动战斗」两行。但附加行是 core 翻转的（`tick_menu16` → `set_shell_option`；BootMachine 设置臂同式），壳只能从 `shell_options` 读到结果。不读行，翻行就不生效。<br>重建 options 也没有现成入口：`set_shell_option` 只改已有 id、不能增删（`menu_family.rs:1314-1317`）。活页 `options` 在进页时用声音 / 小地图状态构造（开机侧小地图恒传 true）。`shell_options` 还在两机之间随交接拷贝（`handoff.rs:173/221/235`） | A：`scene.rs:4580-4588`、`:4600-4604`；`paint.rs:635-645`、`:865`；`runtime.rs:136-147` | 改写规则：<br>- `control_mode` / `auto_battle`：行存在时以行为准，每拍同步进 Runtime 字段；行缺失时保留 Runtime 字段；<br>- `tips_seen_*`：纯 Runtime 字段，`app_settings()` 直接拷入。<br>重建由 core 新入口完成（如 `replace_shell_rows(rows)`：同时改 `shell_options` 与活页 `options`，保留高亮）。两台机各一份；交接两向都测 |

#### Minor

| 编号 | 严重度 | 意见 | 证据 | 建议修法 |
|---|---|---|---|---|
| R2-8 | Minor | §0 kind 4 的 flags 语义写错。"&1 一次性（两键都消耗）、&4 入队"不对：<br>- 入队发生在 flags&2=0 的交互时，与 &4 无关；<br>- flags&2=0 ∧ &1 时两键都消耗，除非 &4；<br>- flags&2 ∧ 买成功时：&4 = 不翻转、不消耗（可重复）；否则 ^512，再按 &1 入队并续显 kind 4（图标 0,0；此时 flags&2 已翻为 0，落入"告示"行），否则移除。<br>`scene.rs:5436` 的代码注释"&1=open 入队…&4=一次性"同样不准 | A：`scene.rs:5455-5462`、`:4535-4553`、`:4556-4570`、`:4572-4578` | 按 (src, flags 各位) 重写 §0 该句；§3.2 注明买后续显落"告示"行；`scene.rs:5436` 注释另案更正（不改码） |
| R2-9 | Minor | kind 4 src 0 的报价图标是 (0,0)，点空白代按 −5 = **接受**（BUG-039 按图标判是否框）。§3.2 给了接受 / 拒绝两钮，却没说点空白的行为：触屏下点一下空白就收下了赠予 / 报价 | A：`scene.rs:9568-9571`、`:5452` | 裁定：保留（写明），或改按 (kind, flags) 判是否框（另登 STAGE-B 偏离） |
| R2-10 | Minor | 摇杆呈现与键码不一致。多行的查询键码是 −5（对话继续、kind 0/1 关闭、结局、kind 4 src 0 接受、读档读取），而摇杆左右软键实际发 −6 / −7。<br>已核对同效：对话任意键翻下一页（`scene.rs:2873-2879`）；kind 0 非翻页键即关（`:1867-1873`）；kind 4 −6\|−5（`:4697`）；开机读档 −6\|−5（`paint.rs:594`）。<br>例外：kind 1 带图标且右图标 ∉ {2,3} 时全吞键（`scene.rs:4746-4755`），−5 关不掉。<br>§4.2 示意图"左大键（原 −6）/ 右大键（原 −7）"也与表中发 −5 的行矛盾 | A：各行号如左 | §9 判据 ① 对每个带标注的功能同时实跑"工具栏键码"与"摇杆物理键码"；kind 1 吞键子类单列一行；改示意图措辞 |
| R2-11 | Minor | §3.2"关于（Boot IconView）"写"键码按 BootMachine 臂核对"，其实现在就能定：−7 返回，−1 / −2 翻页 | A：`paint.rs:610-628` | 填行：中间 = 上页 · 下页（可翻时），右 = 返回（−7） |
| R2-12 | Minor | §5.4 / m10"dp3 无 wp4 臂会卡住"的推断不成立：wp4 由 paint 清零（`advance_open_group_paint` 计满 24 拍置 wp0），`paint_dialog` 也调用它，所以生产路径不会卡。<br>但不调 paint 的宿主测试会看到假卡死。真实风险是 wp4 结束后下一步回到不判返回值的 dp3 wp0（并入 R2-3） | A：`scene.rs:6979-7001`、`:7010`；B：中途 5 / 76 场景未构造 | 改写 §5.4 该条；中途用例的测试须每拍调 paint |
| R2-13 | Minor | BUG-044 条目三处不准：<br>① 阻挡清单里的"实墙"：find_path 只经"地形可走 ∨ 有槽"，纯地形墙不会成为 dp3 首步；<br>② 漏了 debug 构建下炉例会 panic（Dialog→Alchemy 未登记）；<br>③ 范围只写"首步"（见 R2-3） | A：`pathfind.rs:172-175`；`scene.rs:711-779` | 原地补正（保留原文，加日期更正） |
| R2-14 | Minor | 0 步撞击开 route 段会产生零长位移笔：开战时 `route_event_suspend` 推一笔 from==to（探针 R2-B 撤销栈 +1）。不开战的撞击（墙、门、商人）段一直开着，下次输入打断时再推一笔零长笔 ⇒ 撤销要多按一次空操作 | A：探针 R2-B；`pathfind.rs:427-432`（close 不判 from==to） | 只在开战时开段，或撞后不开战立即收段且跳过零长笔；加撤销计数测试 |
| R2-15 | Minor | `input-operations-inventory.md` §8 的更正引用块插在表格第 4、5 行之间，第 5 行脱离表格（渲染成普通段落） | A：该文件 §8 第 121-127 行 | 把引用块移到表格之后 |
| R2-16 | Minor | §5.3"滑动 = 先打断路线再 `probe_step`"：点地寻路途中 `field_tap_allowed` 为假，`probe_step` 直接拒绝，滑动在路线中会静默失效 | A：`scene.rs:1942-1945` | 与 R2-4 同机制（排空后执行），或定义"路线中滑动 = 停下" |
| R2-17 | Minor | 观察模式拖动"直写相机"跳过了 wp2 平移臂对 `facing` 的写入（每个平移方向都会改 facing，影响角色朝向和其后炉的 facing==1 判定）。退出观察（−5）要经过多拍回锁（`view_locked` 后 ease 到位才回 wp0），回锁期间钮是否显示未定 | A：`scene.rs:4927-4963`、`:4979-4993` | 写明拖动不改 facing（或同式改）；回锁期间按"观察模式"行处理并测试 |
| R2-18 | Minor | §2.2 / §13"新字段与 state dump 关系待查"现在就能关：`state_dump.rs` 逐项手工 push，新字段不加就不会进 `state-dump-golden.json` | A：`crates/game-oracle/src/state_dump.rs:75` 起 | 改为已证实，删除遗留假设 |
| R2-19 | Minor | 陷阱纯函数"trace / oracle 证明不变"证据不足：oracle 夹具只见 `fx-knockback-floor43`（60 上爬），未见 61 夹击、62、装备 40 的场景。<br>静态等价性成立（B）：原式里判定与 `apply_hp_delta`、`fire_trap_climb_up` 交错，但上爬只改 (x,y−1) / (x,y−2)，发生在左右计数之后、下邻 (x,y+1) 计数之前，不影响后续判定。所以"先纯判定、再按 61 → 62 → 上爬 → 60 原序施加"等价。<br>另：§0 的"8 = 怪物（41–78 除 77/78）"漏了 72、76；且 60 / 61 / 62 本身就是类目 8 的怪 | A：`scene.rs:5756-5810`、`a.java:5555-5625`、`entity.rs:251-263`；`crates/game-oracle/tests/fixtures/` 清单 | 加穷举对拍：保留旧块的测试专用冻结副本，跑 54 层 × 全格 × 装备 {0, 40} + 边界合成摆放（含 51 层 12×13）。纯函数不含上爬副作用（`tap_passable` 专用）。§0 补"陷阱型本身是怪（选中 / 开战适用）"并改正类目范围 |
| R2-20 | Minor | §4.1 公式有缺口：<br>- s_t 降到 s_read 仍不满足 C_min 时怎么办，没写；<br>- 高度条件 `rows_max·32·s_t ≤ V_t` 要求整层 16 行（`rows_max` 含 HUD）。1080×2400@420 的 V_t ≈ 2400 − inset_top − 168（64dp）− G（≥ 84），可能 < 2048，4 倍会先被高度否掉，与 C_min 无关；§12 只从宽度讲取舍 | A：`portrait.rs:306-317` | 写明兜底（取 s_read）；三档窗口按真实 inset 算出 (s_t, 格阵) 写进 §4.1，再请用户裁定 C_min |
| R2-21 | Minor | 炼丹炉宽 3 格（探针 R2-A：`width_cells(9)=3`），"停在其下方朝上撞"未定是哪一格 | A：探针 R2-A；`entity.rs:225-235` | 写明"三格任一的下方，按 §5.2 排序取最近" |
| R2-22 | Minor | BFS 替换的连带改动没列全：<br>- `classify_tap` 墙格分支要改成"走到邻格再撞"（`touch.rs:156-167`），其测试（`touch.rs:285`、`:307`）的期望随之改变；<br>- `crates/game-core/tests/walk_route.rs` 依赖 `plan_route`，若删函数会降 ratchet 计数（`rust-test-counts.json`） | A：如左；`data/status/baselines/rust-test-counts.json:58` | plan 中列出受影响的测试并逐个迁移，不减测试数；`plan_route` 保留或迁移，不直接删 |
| R2-23 | Minor | M3 判据可执行，但还缺三点：<br>① 情境覆盖清单（§3.2 每行至少一个夹具，含 kind 4 各子类、`ending_done`、炼丹炉、观察回锁）；<br>②「停下」「取消」「开战」「停下后打开」没有单一键码，查询返回类型需要"动作"而不是键码；<br>③ 摇杆物理键码一并实跑（R2-10） | B：§3.2 / §9 | 在 §9 补上这三点 |

### 6.3 已落实确认

- **C2 / BUG-045**：条目事实准确。`undo_ui.rs:36-46`（`:43` 按 `e.slot` 写 `enemies.hp`）；快照 `scene.rs:5413`、赋值 `:5422`；`UndoJournal::clear` / `begin_combat` 只在测试中调用。Q8 与 §7 已标依赖，自动战斗已排在其后。
- **C3 / BUG-046**：`a.java:5487-5490` 原文核实；`rg 无法战胜 crates` 只命中注释 `scene.rs:5431`；R-battle-4、FX-BOSS-68B 的引用属实。§5.2"打不过"已改为原版提示框。
- **C1 / BUG-044**：根因描述（`scene.rs:2903-2907`、try_step 复位条件 `:5156-5176`、Java `a.java:4080-4084`）准确。修法不足，见 R2-2 / R2-3 / R2-13。
- **M8**：gameplay.md R-scene-4 的更正准确（`a.java:5555` 的 `var7 = equippedArmorType == 40`；60 计数块在 `if (!var52b)` 内；62 / 60 右邻判界用 `mapCellsHigh`，`a.java:5578/5604`）。§0 已改为 deobf 行号。
- **M1 / M5 / M6**：情境已拆行；开机侧各行已补；kind 3 / kind 4 已按子类拆行（细节见 R2-5 / R2-8 / R2-11）。逐行核对通过：kind 3 −6 无条件 `use_item_stack`、−7 回背包（`scene.rs:4730-4743`）；说明类图标 (0,3)（`:3429-3432`）；kind 5 −3/−4 换条、−1/−2 ±1 行、−5/−6/−7 关回背包（`:4700-4715`）；kind 0/1 ±1 行（`:1867-1873`）；帮助 / 关于 ±页（`:4716-4729`、`paint.rs:610-628`）；对话 dp1/6 的 −1/−2 页内翻、其余键下一页（`:2846-2880`）；结局 `ending_done` 门（`:4840-4852`）；图鉴 −7/−6 回 Map（`:4098`）；炼丹炉 −1/−2、−6/−5、−7（`:4240-4261`）；开机读档 −6/−5 仅有效槽、−7 返回（`paint.rs:586-606`）。
- **M4**：§2.2 已写明 classic tick 臂不动、查询独立实现、实跑测试作裁判。
- **M7**："存在且可见"的定义、`entity_category`（`scene.rs:6384-6390`）、多格怪、类目 32、沿途拾取均已写入。
- **M10 / M12 / m1–m9**：已落实。m2 的三处读点 `paint.rs:844/897`、`scene.rs:3988` 核实；m3 中 `pathfind.rs` 头注仍是 9869，已排到 P3。
- **§0 其余行号抽查全部通过**：`pointer_hit` 9578、`blank_tap_key` 9564、`walk_to` 1901 / 1929-1934、`scene_tap` native.rs:332、replay 315-323、`apply_portrait` 543-553、视口 Up 595-606、`adopt_portrait` / `replan_portrait` 1135/1210、`runtime.rs` 136-146/172、`touch.rs` 56/138、`portrait.rs:292`、`pathfind.rs:109`、findPath `a.java:9898`、relax `:9967`、`enter_menu16` 4580-4588、`paint.rs:865`、观察模式 5099-5113、炉 5326-5338、kind 4 5443-5466、`apply_merchant_offer` 4412 起、−7/−6 分派 4856-4867。

### 6.4 未能核实

- R2-3 中"wp4 结束后下一步经 dp3 wp0"与"陷阱扣血后下一步经 dp3 wp0"是 B 级（静态推理），未构造真实关卡场景。
- 1080×2400@420 的真实 inset（R2-20），需实机或系统规格值。
- 片头 −7 跳过沿用盘点 §6 结论，本轮未逐行核对 `intro.rs`。
