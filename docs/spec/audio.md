# 音频（BGM）语义规格：选轨/启停/音量 → game-core `audio.rs`

> 状态：S1.1 已移植（2026-10-14）。本游戏**只有 BGM、无音效**：`f_Player_00` 仅被
> `m_139`/`closeAudioPlayer` 两个方法触碰（`grep -n f_Player_00 a.java` 全部命中在
> a.java:401/10163-10213）。行号均指 `reference/src/deobf/a.java`；字节码偏移指
> `analysis/rename-pipeline/renamed/a.class` 的 `javap -c -p`。

## 1. 结论（宿主契约）

`BootMachine::bgm_request()` / `GameScene::bgm_request()` → `Option<usize>`：
**Java `f_Player_00 != null` 时为 `f_byte_28`（轨道表下标），否则 None**。
辅助查询：`bgm_serial()`（第几个播放器——同轨 serial 变化 = 关闭重建 = 从头播放）、
`bgm_volume()`（创建时下发的 VolumeControl 级别 0..=100）。

| 状态 | 轨道 | 证据 |
|---|---|---|
| mode 0 启动画面 / mode 21 声音询问 | None | 初值 `f_byte_28=-1`（a.java:2242）；mode 0 末 closeAudioPlayer（a.java:3376）；m_000 case 21 `soundEnabled=false`（a.java:4484） |
| 询问选「是」(-6) → 标题及其子菜单（8/14/15/16/17）与 mode 2 载入 | 2 `menu.mid` | m_000 case 1 `m_139(2)`（a.java:4311）；子菜单入场不调 m_000 case 1/3 ⇒ 不换轨 |
| 询问选「否」(-7) | None（直到设置里开声） | a.java:4233-4234 `soundEnabled=false; volumeLevel=0` ⇒ m_139 门控（a.java:10159） |
| 进入 mode 3（新游戏/读档载入完成）及其后全部游戏内模式（含 mode 20 结局卷轴） | 3 `game.mid` | m_000 case 3 `m_139(3)`（a.java:4321）；游戏内其余转场不经 m_000 case 1/3 |
| 设置（mode 16）关声 | None | closeAudioPlayer（a.java:4166） |
| 设置开声 | `f_bool_28 ? 3 : 2`，从头播放 | a.java:4157-4163（volumeLevel 复位 60） |
| 结局任意键 / 游戏内菜单「退出」→ 标题 | 2，从头播放 | m_000 case 1（a.java:4295/4311） |
| mode 22 退出动画 | None | m_000 case 22 closeAudioPlayer（a.java:4487） |

轨道 0/1（`logostart.mid`/`logoquit.mid`）**不可达**——见 [BUG-010](../bug-ledger.md)。
循环：每次创建都 `setLoopCount(-1)`（a.java:10184）⇒ 无限循环。

## 2. 证据表（Java 锚 → Rust 映射）

### 2.1 状态字段

| Java 字段 | 声明 / 初值 | Rust（`crates/game-core/src/audio.rs`） |
|---|---|---|
| `soundEnabled` | a.java:399 / true a.java:2241 | `BgmState.sound_enabled`（BootMachine/GameScene 的 soundEnabled 唯一真相源） |
| `volumeLevel` | a.java:398 / 60 a.java:2240 | `BgmState.volume_level` |
| `f_byte_28`（当前轨） | a.java:400 / -1 a.java:2242 | `BgmState.current_track` |
| `f_bool_28`（在游戏中） | a.java:396 / false | `BgmState.in_game` |
| `anyKeyPlayed` | a.java:40 / false | `BgmState.any_key_played` |
| `f_Player_00`（static） | a.java:401 / null | `BgmState.player: Option<Player{track,volume,serial}>` |
| 轨道表 `f_String_arr_15` | a.java:2239 | `audio::TRACKS` |

### 2.2 方法与调用点

调用点全集由 javap 机械枚举（A 级，复现：`javap -c -p analysis/rename-pipeline/renamed/a.class | grep -n "m_139\|closeAudioPlayer"`）：
m_139 共 6 处、实参只有 `iconst_2`/`iconst_3`；closeAudioPlayer 共 5 处。

| # | Java 语义 | 行 / 字节码 | Rust 映射 | 运行时证据（A 级 trace） |
|---|---|---|---|---|
| 1 | `m_139(t)`：soundEnabled 门控（连 close 都跳过）；同轨且播放器在 ⇒ stop→start 续播 return；否则 close→createPlayer/realize/prefetch→setLoopCount(-1)→setLevel(volumeLevel)→start→`f_byte_28=t` | a.java:10158-10206 | `BgmState::play` | 每次创建 EVT 序列恒为 `createPlayer→realize→prefetch→setLoopCount(-1)→setLevel(60)→start(state=300)` |
| 2 | `closeAudioPlayer`：close→null→`f_byte_28=-1`（不门控） | a.java:10208-10215 | `BgmState::close` | `close(state=400)` + 下一拍 FLD `f_byte_28=-1` |
| 3 | run case 0 末（mode 0→21）closeAudioPlayer | a.java:3376 / run@1291 | `BootMachine::tick_mode0` → `bgm.close()` | 此时无播放器（无 EVT） |
| 4 | m_000 case 21：`soundEnabled=false` | a.java:4484 | `bgm.enter_sound_prompt()` | EV-toggle-exit / EV-ending-title **T68** `soundEnabled=false` |
| 5 | run case 21：-7 ⇒ `soundEnabled=false; volumeLevel=0`；-6 ⇒ `soundEnabled=true`，音量 0 则复位 60；两支都 gameMode=1 + m_000 | a.java:4229-4246 | `bgm.answer_sound_prompt(key==-6)` | -7：EV-toggle-exit **T71** `volumeLevel=0` 且**无** createPlayer；-6：EV-ending-title **T71** createPlayer + `f_byte_28=2` |
| 6 | m_000 case 1：`f_bool_28=false` + `m_139(2)` | a.java:4295/4311 / m_000@412 | `bgm.enter_title()`：BootMachine mode 21 出口；GameScene mode 20→1 | EV-ending-title **T1301**（结局→标题）、EV-toggle-exit **T651**（菜单「退出」→标题）：close + createPlayer，`f_bool_28=false f_byte_28=2` |
| 7 | m_000 case 3：`f_bool_28=true` + `m_139(3)` | a.java:4316/4321 / m_000@562 | `bgm.enter_game()`：BootMachine mode 2 完成（call_m000 两支） | A-enter-game **T417**（新游戏）、EV-toggle-exit **T397**（读档）：close + createPlayer，`f_byte_28=3` |
| 8 | run case 16 label 0：翻转；开 ⇒ `volumeLevel=60` + `m_139(f_bool_28?3:2)`；关 ⇒ closeAudioPlayer | a.java:4157-4167 / run@6273、@6283、@6291 | `bgm.toggle_sound()`（BootMachine 与 GameScene 两处 mode 16） | 标题侧 EV-toggle-exit **T161/T201/T241**（开→2 / 关→-1 / 开→2）；游戏内 **T521/T561**（关→-1 / 开→3） |
| 9 | m_000 case 22：closeAudioPlayer | a.java:4487 / m_000@1021 | `BootMachine::enter_menu_mode(22)` → `bgm.close()` | A-menu-sweep **T1761** `close(state=400)` |
| 10 | hideNotify：(mode3→4 + m_000) + closeAudioPlayer + `anyKeyPlayed=true` | a.java:4502-4513 / hideNotify@26 | `bgm.on_hide_notify()`（仅音频半，见 §4） | oracle 不触发 hideNotify（B 级静态） |
| 11 | keyPressed：anyKeyPlayed 时 mode∉{21,0} ⇒ `m_139(f_bool_28?3:2)`；无论是否恢复都清标志 | a.java:4663-4676 / keyPressed@43、@52 | `bgm.on_key_pressed(mode)`：`GameScene::press_key`、`BootMachine::tick` 入口 | 同上（标志恒 false ⇒ 惰性，B 级） |

不换轨的模式转换（B 级 + A 级旁证）：标题子菜单 8/14/15/16/17 与 mode 2 载入、游戏内
4/5/7/9/10/11/12/15/16/19/20 的入场分支均不调 m_139/closeAudioPlayer（m_000 对应 case
无音频调用，a.java:4292-4492），返回走 `gameMode = menuReturnMode`（不调 m_000）。旁证：
EV-toggle-exit T281（16→1）、T441（3→4）、T601（16→4）、EV-ending-title T1152（→20）均无 EVT。

### 2.3 运行时证据再生

```bash
python3 tools/audio/evidence.py            # 变体 A（原始字节码），dumpStride=1 逐 tick 全量
python3 tools/audio/evidence.py --trace reference/oracle/_out/A-menu-sweep/trace.txt
```

输入脚本 `tools/audio/evidence/{toggle-exit,ending-title}.txt`（RMS 预置复用
`reference/oracle/scenarios/preset-{floor1,ending}.preset`）；产物
`reference/oracle/_out/EV-*/trace.txt`（gitignore，可再生）。EVT 行（Manager shim 事件，
`reference/shim/src/javax/microedition/media/Manager.java`）在 postTick(n) 先于 `TICK n`
刷出，归属 tick n。

EV-toggle-exit 时间线（2026-10-14 实跑，摘录）：

```
T0068 gameMode=21 soundEnabled=false
T0071 gameMode=1 volumeLevel=0                       ← -7：无 createPlayer
T0131 gameMode=16                                    ← 标题「载入进度」项（BUG-007 错位）
T0161 volumeLevel=60 soundEnabled=true f_byte_28=2   EVT[createPlayer … setLevel(60) start]
T0201 soundEnabled=false f_byte_28=-1                EVT[close(state=400)]
T0241 soundEnabled=true f_byte_28=2                  EVT[createPlayer …]
T0397 gameMode=3 f_bool_28=true f_byte_28=3          EVT[close(state=400); createPlayer …]
T0521 soundEnabled=false f_byte_28=-1                EVT[close(state=400)]
T0561 soundEnabled=true f_byte_28=3                  EVT[createPlayer …]
T0651 gameMode=1 f_bool_28=false f_byte_28=2         EVT[close(state=400); createPlayer …]
```

## 3. 不可达 / 未建模（显式登记）

- **同轨续播臂**（a.java:10163-10174，STARTED→stop→start / PREFETCHED→start）：
  `reference/oracle/_out/` 全部 trace（2026-10-14：61 个目录，含 EV-*）合计 109 次
  createPlayer、54 次 close、**0 次 stop**（A 级计数，`grep -h "^EVT" reference/oracle/_out/*/trace.txt | sort | uniq -c`）。静态上（B 级）
  每个调用点调用前播放器要么为 null、要么轨道不同。Rust 照搬语义（不重建、不重设音量）。
- **非 STARTED 存活态**：每条 m_139 成功路径以 start() 收尾、close 同步置 null ⇒ 存活
  实例恒 STARTED；其余状态臂落穿重建，Rust 不单列。
- **createPlayer/realize 异常臂**（a.java:10200 catch）：平台相关，未建模；桌面壳缺音频
  产物时静默降级，不回写游戏状态。
- `m_008()` 关流、`saveGlobalProgress()`（a.java:4169）：I/O，不属 game-core。

## 4. 已知缝（非本批）

- **hideNotify 整体**（a.java:4502-4513）：含非音频臂「mode 3 → 4 + m_000 case 4」。
  桌面壳失焦采用**壳级冻结**（不推进逻辑 tick、暂停音频输出），不映射 hideNotify；
  `BgmState::on_hide_notify/on_key_pressed` 已按 Java 移植并单测，但无宿主调用 ⇒
  `anyKeyPlayed` 恒 false。若将来需要真机中断语义，须整体移植 hideNotify。
- **游戏内菜单「退出」kind 7**（a.java:4757-4769）：GameScene `activate_menu_item` 尚未
  移植该臂（既有缝），故 EV-toggle-exit T651 在 Rust 侧不可复现；mode 20→1 已挂 m_000
  case 1 音频半，但 **GameScene → BootMachine 反向移交**仍未实现（标题菜单不会出现）。

## 5. 宿主侧实现约定（game-desktop，非 Java 语义）

- 音色：MIDI 经 `tools/audio/` 离线合成为 OGG（SF2 音色≠原机合成器，Stage A 已知偏差，
  见 [tools/audio/README.md](../../tools/audio/README.md)）。
- 音量：MMAPI 规定级别 0..100 线性刻度，「产生线性乘数的映射由实现决定」
  （JSR-135 VolumeControl 文档）；壳取幅度乘数 `level/100`（60 ⇒ ×0.6）。
- 循环：OGG 带 `LOOPSTART/LOOPLENGTH`（样本），壳在 `LOOPSTART+LOOPLENGTH` 处跳回
  `LOOPSTART`；serial 变化或轨道变化 ⇒ 从头播放，否则续播。
