# Stage B 有意不一致登记册

> Stage B（现代化）相对基线 tag `original-reborn` 的**有意偏离**登记。纪律：
> 每条偏离=ID/偏离点（Java 锚 vs Rust 行为）/用户裁定原话+日期/理由/金样影响名单/
> 代码注释引用（`// STAGE-B-xxx`）/状态。差分与金样变更必须引用偏离 ID（仿 BUG-xxx
> TOLERATED 先例）——禁静默改基线。

## STAGE-B-003 素材管线（能力先行·素材不动）

- **状态**：核心实施中（2026-10-18 task-8 复核通过）
- **框架**（用户裁定）：管线现在建、**素材维持原样**（可审计可对拍）；替换通道打通，
  未来换素材+开 modern 模式同日上。
- **两门分工**（adversary task-8 记录 §7 移交项）：
  - **管线保真门**：未编辑资产 
epack(unpack(x))==x 逐字节恒等（126 资产，常驻 gates）；
  - **内容正确门**：编辑资产的正确性由硬2 解码等价（decode(json+png)==parse(binary)）
    + 运行时金样承担；**改图后 repack 面须有自己的校验门**（新字节嵌入+容器重建过解析侧校验）。
- **真相源层级**：original/ 仍唯一真相（assets/art=gitignored 生成物+哈希清单三态核）。

## STAGE-B-001 开机声音询问移除 + 声音态跨启动持久化

- **状态**：实施中（2026-10-18）
- **用户裁定原话**（2026-10-18）："游戏开始时会询问用户是否开启声音；这是诺基亚时代
  游戏的特点，现在我们不需要用户做确认了，默认开启，游戏中可以关闭（设置中本来就有），
  声音开启状态需要持久化"；"声音开关是全局状态，不放在存档中"。
- **偏离点**（三条，adversary 复审定案 2026-10-18-stage-b-001-sound-ask-review.md）：
  1. **询问屏移除**：原版 a.java:3371-3373 logo 尾切 mode 21（SoundAsk）等用户回答
     （-6 确认/-7 否定 → mode 1，a.java:4229-4242）⇒ Rust 直入 MainMenu。
  2. **询问前置关声移除**：原版 dispatchModeEntry case 21 强制 `soundEnabled=false`
     （a.java:4484，等回答）⇒ Rust 保持默认/已设状态（BgmState::new=true）。
  3. **声音态跨启动持久化**（原版无此行为——adversary A 级 grep 证伪"RMS 本就装载"：
     soundEnabled 写点全集 a.java:2241/4157-4159/4233-4241/4484，**无 RMS→soundEnabled
     回灌**；:7290 存侧镜像单向）⇒ Rust 独立全局设置文件持久（**不入游戏存档**，
     用户裁定），shell 装载应用 + toggle 变更落盘。
- **理由**：诺基亚时代交互惯例已过时；声音偏好=应用级全局设置。
- **金样/夹具影响名单**（硬要求 2，实测枚举定案）：
  - `crates/game-core/tests/mov_route_scene.rs::preset_floor2_mov_frames_match`
    与 `preset_floor40_mov_frames_match`——frames.txt=**Java 参照帧**（A/B/C trace
    逐 TICK 抽取，tools/frames_fixture.py gen 再生），原版含询问屏而 Rust 跳过 ⇒
    boot 窗口起帧链漂移**为预期且不可调和**（Java 参照不会再有无询问屏的版本）。
    处置=**STAGE-B-001 豁免**（`#[ignore]` + 注释引 ID，仿 BUG-xxx TOLERATED 先例）；
  - 后续工程（登记待办）：夹具输入脚本重定时（原脚本含询问屏应答键）+ 金样改为
    Rust 意图基线（放弃 Java 参照性，登记为有意不一致的基准切换）；
  - `shell/win/src/main.rs::shell_tests::hold_walk_and_bgm_match_java_trace`
    ——BGM 切轨拍对照 Java EVT（createPlayer@T71）；boot 时间轴缩短 ⇒ 必然漂移。
    处置=STAGE-B-001 豁免（用例重定时待办）；
  - `shell/win/src/main.rs::shell_tests::hide_in_modes_0_and_21_clears_flag_without_resume`
    ——mode 21 已移除=前提消失。处置=STAGE-B-001 豁免（用例重写待办）；
  - `crates/game-oracle/tests/boot_replay.rs::enter_game_frames_match_tick_by_tick`
    与 `boot_menu_frames_match_tick_by_tick`——boot 帧哈希对照 Java 参照，TICK 68
    起漂移。处置=STAGE-B-001 豁免（夹具重定时待办）。
  > 【2026-10-18 更正】原文"其余 boot 链夹具实测零漂移"**为假**——oracle boot_replay
  > 双枚同因漂移（实施后期实测推翻）；已并入上列。豁免总数=7 枚。
- **豁免合计 64 枚**（全部 `*_frames_match` 帧回放夹具族——全经 boot 链时间轴，同因实测失败全集：mov_route×2 / slot_equip×1 / shell×2 / oracle fixture_frames+preset_cases 等 59）——全数 `#[ignore]` + 注释引 STAGE-B-001，登记册与代码注释双向可查；**夹具重制=Stage B 首个后续工程**（脚本重定时+Rust 意图金样基线）
  注释引 STAGE-B-001，登记册与代码注释双向可查。
- **偏离点 3 补注（R3）**：volume_level 同 sound_enabled 一并入全局设置持久
  （原版 volumeLevel 亦从不持久——写点 a.java:2240/4159/4234/4241 无回灌）；
  GlobalSave 70/71 只写不读=**趋同非偏离**（原版本就不回灌 soundEnabled）。
- **代码注释引用**：`// STAGE-B-001`（game-core 跳过点 / shell settings / 测试）。
