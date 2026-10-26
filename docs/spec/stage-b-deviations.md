# Stage B 有意不一致登记册

> Stage B（现代化）相对基线 tag `original-reborn` 的**有意偏离**登记。纪律：
> 每条偏离=ID/偏离点（Java 锚 vs Rust 行为）/用户裁定原话+日期/理由/金样影响名单/
> 代码注释引用（`// STAGE-B-xxx`）/状态。差分与金样变更必须引用偏离 ID（仿 BUG-xxx
> TOLERATED 先例）——禁静默改基线。

## STAGE-B-002 动态逻辑分辨率双预设（classic/modern）

- **状态**：全卷复审有条件通过（2026-10-18，硬4 文本级整改即本节）
- **用户裁定原话**（2026-10-18）："我们支持'现代分辨率'，可以看到全图，也支持 classic mode，
  维持现有视口大小不变"；"游戏允许设置任意的逻辑分辨率（几个格），UI 自动适配
  （太小了不行，最小就是原分辨率）"；小地图歧义裁定："不得引发歧义"（色点避门色）。
- **金样声明（关键）**：**classic 默认零偏离 ⇒ 无新增豁免**——默认路径（classic 预设）
  的行为与像素零变化由 Legacy 等价判据证明（walk_camera_golden 4 测/game_view 钳制
  (8,-210)/ease 序列断言**一字未改**，adversary 独立复核 git diff 测试面=空）。
- **偏离点**（仅 modern 能力面，classic 不触达）：
  1. 动态逻辑分辨率（任意格数×32，最小=240×320）——modern 能力；
  2. UI 公式锚（顶栏贴左/贴右/黑填充界、HUD 贴底居中）——classic 代入化回原值；
  3. 相机参数公式化（EDGE_MARGIN=106/CAMERA_CLAMP_MAX=64 入 Layout，两预设同值
     （跟随舒适带宽不随分辨率缩放）+ classic 代入断言）；
  4. 设计常量 7 条 modern 公式（考证表在 layout.rs:361-372，逐条带理由；classic 回代）；
  5. R6 小地图信息增密（怪物/物品/机关色点）**+ 色相歧义修正**（用户裁定"不得引发歧义"：
     首版色点与门红/蓝/黄 hue 距 3° 三连撞 ⇒ 改品红/白/紫/炭黑 ≥25°+防撞门测试；
     语义偏离=怪物红系→品红、物品黄系→白，以"无歧义优先"入册）；
  6. 暂不分级（怪物威胁分级无实体表字段 ⇒ 统一色，理由入册）+ 细类分色=type 级考证待并项；
  7. game_view clamp 64 写域外待并册（考证表尾注）。
  8. 【2026-10-26 嫦娥剧情链 Modern 重跑登记】**剧情"等相机"相位时长随视口**：lockCameraOn 缓动起点 =
     `(视口−32)/2 − 相机`（视口派生，Modern 视口大、相机多被钳位）⇒ 收尾等相机（dialogPhase 4）/
     SEE 等相机（6）结束拍早晚 1-3 拍；剧情顺序、路径、实体位置不变。钉：`fixture_frames.rs`
     `chang_e_fixtures_modern_logic_matches_classic`（5 个嫦娥链夹具 × 12×16/13×13/18×18：首分歧只允许
     "该拍仅 gameMode 11↔3 不同且处 phase 4"，此后夹具按拍锁定的输入错相、止于该拍）+ Android 运行时
     状态驱动全程对拍 `script19_event_sequence_independent_of_logical_resolution`（事件序列逐条一致）。
- **覆盖边界（如实）**：modern 预设当前覆盖 **mode 3 游戏视图面**（地图/相机/状态栏/HUD/
  小地图）；菜单等其余界面全量自适配=**后续迭代项**（待并项）。
- **验证债**（头号风险，已挂牌）：L2/L3 帧金样全集离线（64 枚 STAGE-B-001 豁免）——
  夹具重制任务挂牌（验收=64 枚逐个摘除归零，**豁免数不许再涨**）。
  > 【2026-10-19 解清】lead 裁定 (c) 预设门控（STAGE-B-001 行为门控到 modern 预设，
  > classic 保留开机询问=原版时间轴）⇒ 64 枚豁免**全数摘除复绿**（含 shell×2），
  > classic 帧面与 Java 参照零偏离恢复，验证债清零；豁免数自本注起只降不升。
- **汇报口径**：测试计数须带豁免披露（现网 #[ignore] 67 枚=64 帧回放豁免+3 人工 dump）。
  > 【2026-10-19 更正】现网 #[ignore] = **3 枚**（仅 gameplay_floor1 人工 dump 诊断工具，
  > 常年 ignore 属工具常态非豁免债）——64 枚帧回放豁免已随门控清零。

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
  > 【2026-10-19 门控化更正】lead 裁定 (c)：**偏离点 1/2（询问屏移除+询问前置关声移除）
  > 门控到 modern 预设**（`Layout.boot_sound_prompt`，modern=false 跳过/classic=true
  > 原版时间轴 a.java:3371-3373→SoundAsk 照旧）——参照 STAGE-B-002 Layout 双预设体例。
  > classic 预设下帧面与 Java 参照零偏离恢复；偏离点 3（声音态持久化）经审为 shell 级
  > 帧面中立行为，留 modern 线自然生效（无存档格式面，未触发"不可门控即停"条件）。
  > 附带：001 行为收窄为 modern 预设门控面（stage_b001_boot_skips_sound_ask 测试改
  > modern 预设）+ 新增 classic 对测 stage_b001_classic_keeps_sound_ask（原版时间轴
  > 命门回归：SoundAsk + 软键 (1,2) + soundEnabled=false 三点）。
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
  > 【2026-10-19 解清】按 lead 裁定 (c) 预设门控收尾（非重录制）：64 枚 `#[ignore]`
  > 全数摘除、逐测复绿（原注释行保留并追加解禁日期注，§3.6 体例）；豁免归零。
  > 原"夹具重制（脚本重定时+Rust 意图金样基线）"路线**作废**——classic 预设恢复
  > 原版时间轴后 Java 参照性直接复得，无需基准切换。
  注释引 STAGE-B-001，登记册与代码注释双向可查。
- **偏离点 3 补注（R3）**：volume_level 同 sound_enabled 一并入全局设置持久
  （原版 volumeLevel 亦从不持久——写点 a.java:2240/4159/4234/4241 无回灌）；
  GlobalSave 70/71 只写不读=**趋同非偏离**（原版本就不回灌 soundEnabled）。
- **代码注释引用**：`// STAGE-B-001`（game-core 跳过点 / shell settings / 测试）。
