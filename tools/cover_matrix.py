#!/usr/bin/env python3
"""cover_matrix.py — 覆盖矩阵生成器 v2.1（层次一账本；2026-10-15 刷账：v2 重写 + v2.1 行锚归属治理）。

逐 Java 方法（改名类 javap 全量，含 <init>/<clinit>，按 name+descriptor 唯一）判定三色：

  绿 = 有移植账 ∧ 被 Rust **全帧对拍**用例窗口内的 Java 执行触达（列出用例/场景/t_hit）
  黄 = 有移植账 ∧ 无全帧对拍窗口触达（列出仅被无用例场景触达的情况）
  白 = 无移植账（再三分账 a 真未移植 / b 不触达论证候选 / c 判定存疑〔工具方法论〕，见 TRIAGE）

数据源（全部机械可复现；人工条目逐条带证据且由本脚本校验锚点存在）：
  ① 方法清单：`javap -p -s analysis/rename-pipeline/renamed/a.class`
  ② 触达（A 级）：tools/method_reach/snapshot.json —— tools/method_reach.py 以 JDI
     一次性断点实测每场景每方法首次命中 tick（t_hit），并把测得运行的 FRAME sha
     序列与 Rust 用例消费的 A-trace 逐 tick 对照（equal 才计入验证）
  ③ 用例 → 场景 → 窗口：TESTS_DIRS（crates/game-oracle/tests + crates/game-core/tests，只认 git 已追踪
     文件）机械解析（collect_tests；帧级 = 非 #[ignore] 且 ① A-trace 用例 `assert_eq!(&sha, expect…)`
     或 ② 夹具帧用例 `assert_eq!(sha, rec.sha…)` + `<stem>.frames.txt`〔夹具帧链须 = 快照实测帧链〕；
     窗口由 checked 断言派生）
  ④ 移植账（只扫移植目标 crate：PORT_CRATES；game-oracle 是差分宿主不计）：
     Rust fn 直证（同名/蛇形/台账机械名 m_NNN）∪ Rust 注释 a.java 行锚（v2.1 归属规则，
     见下）∪ PORT_NOTES 人工条目（ported/absorbed/not-ported，逐条校验锚点）
  ⑤ 白项分账：TRIAGE 人工条目（a/b/c + 论证 + a.java 行锚，逐条校验）；b 类带「裁决 DC-n」者经
     tools/dead_code_ruling.py 字节码不动点证死 + docs/spec/dead-code-rulings.md 裁决书锚校验 + 全场景
     零触达三重校验后记 ☠️（不计判据①赤字）；v1 旧账（LEGACY_V1）原样保留只用于"刷新前后对照"复算
  ⑥ 黄项不触达论证：YELLOW_NOTES（调用者集合 / 驱动面缺席 / Rust 锚机械校验）；臂级缺口：§5 todo! 扫描
     + ARM_GAPS 人工登记（无 todo! 标记或壳侧 BLOCKED 者）

v2.1 行锚归属规则（2026-10-15 审阅发现 v2 的"行锚落入现行方法体区间即算移植"会误归绿：
m_072 吃到 game-data 28 条 seed 方言锚、addSpark/m_006/m_128/m_131 等吃到漂移锚——
实测见 docs/spec/completeness.md「覆盖矩阵刷账（2026-10-15）」）：
  - 方言：显式 `seed/a.java:N` → reference/seed/a.java（CFR 冻结件）；文件内出现过显式
    seed 锚的文件（game-data 全部）其裸锚按 seed 读（N 超出 seed 行数者除外）；其余按 deobf。
    seed/各历史 deobf 与现行 deobf 的方法序数对齐（163 个、逐序参数类型一致）在运行时校验。
  - 名证锚：锚所在行点名了 Java 方法（现名/台账别名/引用的 CFR 签名）⇒ 只归属被点名者，
    且锚须落在被点名方法区间 ±NAMED_TOL 行内。
  - 稳健锚：行内未点名 ⇒ 锚区间外扩 ROBUST_MARGIN 行后仍须完全落在该方法于 VF 时代
    全部 deobf 历史版本区间的**交集**内（seed 方言用 seed 区间）；字段初始化器行
    （static → <clinit>，实例 → <init>）不外扩。
  - 两个容差取自本脚本自测的名证锚偏移分布（§0 行锚精度自检，渲染进矩阵）。

用法：python3 tools/cover_matrix.py            # 产出 docs/coverage-matrix.md
      python3 tools/cover_matrix.py --check    # 只校验：重渲染与账面一致（Rust 行号/锚计数
                                               # 漂移归一化后比较；严格逐字节另行报告）
先决：python3 tools/method_reach.py（场景/预置/改名类变化后重测；--check 判陈旧）
"""
from __future__ import annotations

import glob
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RENAMED = ROOT / "analysis/rename-pipeline/renamed/a.class"
DEOBF = ROOT / "reference/src/deobf/a.java"
DEOBF_REL = "reference/src/deobf/a.java"
SEED = ROOT / "reference/seed/a.java"
LEDGER = ROOT / "data/naming/ledger.jsonl"
# 帧级用例扫描目录：game-oracle（A-trace 用例 + 夹具帧用例）+ game-core（夹具帧用例，如
# slot_equip_scene.rs；2026-10-15 补口径盲区——completeness §3.4 遗留⑤）
TESTS_DIRS = (ROOT / "crates/game-oracle/tests", ROOT / "crates/game-core/tests")
SNAPSHOT = ROOT / "tools/method_reach/snapshot.json"
OUT = ROOT / "docs" / "coverage-matrix.md"
# 白项 b 类死代码裁决书（主张/证据/可复现命令/结论；tools/dead_code_ruling.py 机械复核）
RULINGS_DOC = ROOT / "docs/spec/dead-code-rulings.md"

# 移植目标 crate（AGENTS.md §2：game-oracle 是差分宿主，不是移植目标）。
# 【2026-10-15 更正】原列 game-wasm——该 crate 不存在（crates/ 无此目录，workspace members 亦无），删去。
PORT_CRATES = ("game-core", "game-data", "game-platform", "game-desktop")
# VF 时代起点（反编译引擎 CFR→Vineflower，deobf 行号体系自此稳定；game-core 首提
# 41d8fce 晚于此，deobf 方言锚全部写于 VF 时代）
VF_SINCE = "996a29d"
# 前任 v2 刷账计算时的 deobf 版本（a97d70e 起至 b1b7ee7 行号体系不变）：v2 口径复算钉在此行号上
V2_DEOBF_REV = "a97d70e"
# v1 旧账（迭代 72）提交：v1 伪名按其时方法名复算
V1_REV = "2eed129"
NAMED_TOL = 20      # 名证锚：落在被点名方法区间 ±20 行内即归属
ROBUST_MARGIN = 20  # 稳健锚：锚区间外扩 20 行仍须落在跨版本区间交集内

# ---------------------------------------------------------------- 人工条目
# PORT_NOTES：机械信号缺失/误判时的人工移植账。值 = (判定, 证据)：
#   "ported"     有移植（证据须含 `crates/…rs` `标识符`，脚本校验文件存在且含该标识符）
#   "absorbed"   无需移植代码：Java 方法无可观测效应或被 Rust 结构吸收（证据须含落在
#                该方法体内的 a.java 行锚；可附 Rust 锚，同样校验）
#   "not-ported" 机械信号命中但实为未移植（压制假阳性）
PORT_NOTES: dict[str, tuple[str, str]] = {
    # —— 基线 23 白项之 c 类：移植存在但机械信号漏判（fn 异名 + 无/陈旧行锚）——
    "initCellGrid": ("ported",
              "crates/game-core/src/entity.rs `pub fn new(cols: i32, rows: i32) -> Self`"
              "（CellGrid::new 四件初始化 = a.java:6957-6960；doc 点名「m_059 初始化」）"),
    "findItemStackIndex": ("ported",
                           "crates/game-core/src/combat.rs `pub fn find_item(&self, item_type: u8) -> i32`"
                           "（doc「findItemStackIndex 忠实移植」，首个同型槽/无则 -1 = a.java:7659-7668；"
                           "其 Rust 行锚 8485 陈旧，落入 executeScriptInstruction）"),
    "addItemToItemStack": ("ported",
                           "crates/game-core/src/scene.rs `fn add_item_to_stack(&mut self, t: i32)`"
                           "（追加/叠加次数 + showOverlayMessage = a.java:7722-7731/7733；行锚 7702-7714 漂移落入 "
                           "itemTypeToStackIndex）。未移植 itemUseTargetType 写（a.java:7732）——无可见效应：拾取路径调用前 "
                           "已置 overlay_kind_item（crates/game-core/src/scene.rs `self.overlay_kind_item = t`），"
                           "商人路径浮层同调用内被关闭/替换（2026-10-15 独立审计核）"),
    "easeCameraToTarget": ("ported",
              "crates/game-core/src/scene.rs `// m_103：ease 相机目标 = 目标格`（SEE 臂）+ "
              "crates/game-core/src/scene.rs `// m_103：ease 目标 = SEE 目标格`（翻页臂）两处内联，"
              "四赋值 = a.java:8663-8666"),
    "initFloorStates": ("ported",
              "crates/game-core/src/scene.rs `saved_floor_states: vec![None; 56]`（新游戏构造即 56 槽空 = "
              "a.java:9366-9370；读档路径 crates/game-core/src/scene.rs "
              "`scene.saved_floor_states = save.floor_states.clone()`）"),
    "m_120": ("ported",
              "crates/game-core/src/scene.rs `fn save_floor_state(&mut self)`（u16 BE 写内联为 to_be_bytes = "
              "a.java:9403-9404）"),
    "restoreFloorState": ("ported",
              "crates/game-core/src/scene.rs `u16::from_be_bytes([d[*p], d[*p + 1]])`（层存档恢复 read_u16 "
              "闭包 = a.java:9680，m_122 恢复臂 a.java:9626-9638 内联）"),
    "closeStreamsAndClear": ("absorbed",
              "流关闭 + 三字段置 null（a.java:4635-4660）；javap 字段普查：f_InputStream_00/"
              "f_OutputStream_00/f_InputStream_01 读写者仅 readCountPrefix/closeStreamsAndClear/playBgm/m_148/m_151/<init>（I/O 方法）"
              "⇒ 无游戏态效应；Rust 以内存切片解码无流可关（crates/game-data/src/packed_png.rs "
              "`pub fn parse(data: &'a [u8], image_count: usize) -> Result<Self, Error>`）"),
    # —— 方法论修正（v2.1 行锚归属）暴露、经核验确有移植/吸收者 ——
    "createSolidImage": ("absorbed",
              "静态纯函数（造 32×32 纯色 RGB 图，a.java:4606-4613）；仅有的两个调用点 loadFloorData "
              "a.java:9423-9424 结果被丢弃（javap：invokestatic createSolidImage 后紧跟 pop）⇒ 无可观测效应"),
    "releaseSceneImages": ("absorbed",
                           "图像容器/entityTypeImage 置 null 释放内存（a.java:4715-4727）；读者在读前经 "
                           "readCountPrefix 判空重载（a.java:4519）或 initEntityTables 重建 entityTypeImage（javap："
                           "put 仅 initEntityTables/releaseSceneImages）；Rust 构造期全量装载不释放"
                           "（crates/game-core/src/scene.rs `懒加载资源容器（Rust 构造期全量装载，等价 no-op）`）"),
    "resetNewGameState": ("ported",
              "新游戏态复位分散移植：crates/game-desktop/src/main.rs `GameScene::load_floor(` 以 (32,352) "
              "落点 = placePlayerAtCell(1,11)（a.java:7197 floor51 臂，新游戏 step8 已置 51 层）+ "
              "crates/game-core/src/scene.rs `difficulty_index: 0`（装备/跟随/金/道具栈/炼丹/难度构造默认 0 = "
              "a.java:7192-7193、7213-7217）+ crates/game-desktop/src/main.rs `scale_enemy_stats(&tables, 1)`"
              "（= a.java:7218 scaleEnemyStats(multipliers[0])）"),
    "addOption": ("ported",
              "crates/game-core/src/menu_family.rs `fn add(&mut self, label_idx: usize, checked: bool)`"
              "（OptionList::add = addOption：stringWidth+80 定框宽、首项 32+行高 = a.java:7270-7280；"
              "原 28 条行锚系 game-data seed 方言锚误归属）"),
    "itemTypeToStackIndex": ("ported",
                             "crates/game-core/src/scene.rs `fn item_desc_index(&self, t: i32) -> usize`"
                             "（85→20 / 86→21 / 其余 t-13 = a.java:7707-7716；add_item_to_stack 内联同式）"),
    "deleteRecordStoreQuietly": ("absorbed",
                                 "删库后重建（a.java:9325-9328）；三调用点 a.java:4476（SKY_WAR 读失败→删→"
                                 "saveGlobalProgress 重建）/9021（MOT_IF）/9184（MOT_L{n}）均为删后即写；"
                                 "壳 RMS 以覆盖写吸收删+写（crates/game-desktop/src/main.rs "
                                 "`std::fs::write(&path, v).ok();`）；SKY_WAR 路径的持久化缺口归 saveGlobalProgress（a）"),
    "closeRecordStoreAndGc": ("absorbed",
              "RecordStore/流关闭 + 五字段置 null + gc（a.java:9332-9362）；f_byte_arr_27 读者"
              "（dispatchModeEntry/loadSlotTable/loadFloorSave）均先 getRecord 赋值再读（a.java:4459/8974/9256）"
              "⇒ 无可观测效应"),
    "initParticles": ("ported",
              "crates/game-core/src/title.rs `pub fn new() -> Particles`（32 槽粒子数组清零 + 游标 0 = "
              "a.java:9822-9830）"),
    # —— 压制假阳性：Rust 注释点名 + 行锚命中，但注释本身声明未移植 ——
    "loadPackedImages": ("ported",
                         "打包图像容器装载器（a.java:4516-4549：resourceNames[i] 开流、按 "
                         "resourceImageCounts[i] 循环读 u16 LE 长度 → Image.createImage）——"
                         "crates/game-data/src/packed_png.rs `pub fn parse`/`pub fn parse_partial` 解析 "
                         "N×[u16LE len][PNG]（L1 golden 守护）+ crates/game-desktop/src/main.rs "
                         "`fn load_container` 消费 = 移植替换形态；名实更正 e772fef（N-0434）；"
                         "c 类行号方言误判已由 v2.1 规则修复"),
    "saveGlobalProgress": ("ported",
                           "b62d9db：save::GlobalSave 格式层（a.java:8931-8942 写/4463-4474 读）+ "
                           "BootMachine/GameScene 同构 global 状态（`fn global_snapshot`/`fn note_max_floor`）+ "
                           "壳 SKY_WAR@1 轮询持久化（crates/game-desktop/src/main.rs `fn persist_global_if_dirty`）+ "
                           "catch 语义回写（default_7 ⇒ [1]=true）；回标题触发点（4297-4299）随后续反向移交后置（账本裁定）"),
    # startLoadTask（原 m_131）：v2 唯一行锚（pathfind.rs findPath 注）系误归属；v2.1 另有合法名证锚（intro.rs:274
    # 「m_131(target, call_m000)」），机械仍判移植——此条只补记真实移植位置，不改色
    "startLoadTask": ("ported",
              "crates/game-core/src/intro.rs `pub fn new(target: i32, call_m000: bool, steps: &[i8]) -> LoadProgress`"
              "（loadTargetMode/loadCallInit/步表/计数复位 = a.java:9877-9884；readCountPrefix(15) 由构造期装载吸收）"),
}

# TRIAGE：白项三分账。值 = (类, 来源, 论证)。类：
#   "a" 真未移植（后续铺层/壳层赤字）；"b" 不触达论证候选（死内容/冷路径，B 级以上论证）；
#   "c" 判定存疑（工具方法论问题；须有 PORT_NOTES 或归属规则修正作为解决，现色随之变化）。
# 来源："基线23" = 前任 v2 刷账输出的 23 白项；"修正暴露" = v2.1 行锚归属治理后新暴露者。
# 复现命令约定：JAVAP = `javap -p -c analysis/rename-pipeline/renamed/a.class`
TRIAGE: dict[str, tuple[str, str, str]] = {
    # ---- a 真未移植 ----
    "m_106": ("a", "基线23",
              "MOV 五参实体路径缓动（a.java:8738-8870）：run() dialogPhase 2 臂调用（a.java:4073）；dialogPhase=2 "
              "由 MOV 五参臂在 findBlockingEntity（原 m_100）命中实体时置（a.java:8288-8299）；levelScriptLines（a.java:1124-1193，68 条）"
              "含五参 MOV 36 处（脚本 1/2/4/5/7/8/10/12/19/30/57/61），其中 20 处起点（型,格）在 jar sprite{n} "
              "初始布置中有对应实体（如脚本 1 `MOV_72_3_7_1_8` = sprite2 型 72@(3,7)；脚本 12 的 13 处全落 "
              "sprite40）⇒ 正常游玩可达。【2026-10-15 已移植】6a37155：crates/game-core/src/scene.rs "
              "`fn tick_entity_route(&mut self)`（doc 点名 m_106 a.java:8738-8870），帧级验证 = "
              "crates/game-core/tests/mov_route_scene.rs `fn preset_floor2_mov_frames_match` / "
              "crates/game-core/tests/mov_route_scene.rs `fn preset_floor40_mov_frames_match`（夹具场景 "
              "preset-floor2-mov / preset-floor40-mov）——即 convergence-plan P1 挂起项 dp2=m_106 已落地"),
    "m_108": ("a", "基线23",
              "实体跨格槽表重登记（a.java:8880-8904）：唯一调用者 m_106（a.java:8806/8848；javap 调用图同）"
              "⇒ 随 m_106 可达、随 m_106 一并移植。【2026-10-15 已移植】6a37155：crates/game-core/src/entity.rs "
              "`pub fn relocate_slot(grid: &mut CellGrid, table: &EntityTable, idx: usize, from_x: i32, from_y: i32)`"),
    "saveGlobalProgress": ("a", "基线23",
                           "SKY_WAR 全局进度写（a.java:8917-8951：optionChecked×4 + f_int_arr_35 + "
                           "f_int_152..154）；调用点 dispatchModeEntry a.java:4299/4478、run a.java:4169（选项切换后）。"
                           "Rust：save.rs 对 SKY_WAR 既不编码也不解码（只有 parse_floor_save/parse_preset）；game-core 无 I/O"
                           "（crates/game-core/src/scene.rs `saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）`），"
                           "boot 恒走 SKY_WAR 读失败回退（crates/game-core/src/paint.rs `RMS \"SKY_WAR\" 读档失败路径`）；"
                           "壳 Rms 只改写 MOT_L*/MOT_IF（crates/game-desktop/src/main.rs "
                           "`stores.push((\"MOT_IF@1\".into(), mot_if.clone()));`）⇒ 选项/全局旗标跨会话不持久"
                           "（读写双缺，读臂在 dispatchModeEntry case 21 a.java:4450-4479）——壳层赤字（会话内帧不可见，非铺层赤字）。"
                           "【2026-10-15 已移植】b62d9db：读写双补 + 壳持久化（见 PORT_NOTES）"),
    # ---- b 不触达（死代码）：2026-10-15 终裁 ☠️——论证末尾的「裁决 DC-n」由渲染时机械复核
    #      （tools/dead_code_ruling.py 字节码不动点 + docs/spec/dead-code-rulings.md 裁决书锚校验），
    #      复核不成立即 ⚠ 且退回「待裁」 ----
    "drawLeftTriangleArrow": ("b", "基线23",
              "左向三角箭头（a.java:8034-8036）：调用点 2 处（a.java:5980/5985）均在 paintPlayerSprite walkPhase "
              "case 3 道具轮盘臂；JAVAP 中 putfield walkPhase 共 23 处，前导常量 ∈{0,1,2,4,5}（无 3，"
              "亦无计算值写入）⇒ walkPhase 永不为 3，case 3 绘制臂与 run() `switch (walkPhase)`（a.java:3565）的 "
              "case 3 键处理臂（a.java:3647-3673）皆死代码；裁决 DC-1"),
    "m_107": ("b", "基线23",
              "实体换格（a.java:8872-8878）：调用点 2 处（a.java:8567/8585）均在 executeScriptInstruction 的 MVS 臂"
              "（a.java:8519-8592，`!var3.equals(\"MVS\")` 之 else，DSL 第 16 码）；levelScriptLines（a.java:1124-1193）"
              "68 条脚本 MVS 出现 0 次；executeScriptInstruction 三调用点均传 dialogScriptLine（a.java:4021/8302/8437）；"
              "JAVAP：dialogScriptLine 唯一写点 loadLevelScript（aaload levelScriptLines），levelScriptLines 仅 <init> "
              "赋值且无元素写 ⇒ 脚本文本源封闭、MVS 死指令（crates/game-core/src/script.rs `数据域 68 条脚本零使用`）；"
              "裁决 DC-2"),
    "m_148": ("b", "基线23",
              "联网簇·连接关闭（a.java:10610-10639）：调用点 run mode99 臂 a.java:4261、m_151/m_153（簇内）。"
              "mode 99 不可达：JAVAP putfield gameMode 54 处，常量 ∈{0-5,7-12,14-17,19-22}，计算来源 "
              "menuReturnMode∈{1,4}/loadTargetMode∈{3}/inGameMenuReturnMode←gameMode ⇒ 无 99；"
              "另 master-plan §4 范围裁剪（HttpConnection 联网整体剔除）；裁决 DC-3"),
    "m_149": ("b", "基线23", "联网簇·响应体 u16 读（a.java:10641-10643）：仅 m_150/m_153 调用（簇内）；"
                           "簇不可达论证同 m_148；裁决 DC-3"),
    "m_150": ("b", "基线23", "联网簇·响应体 u32 读（a.java:10645-10647）：仅 m_153 调用（簇内）；同 m_148；裁决 DC-3"),
    "m_151": ("b", "基线23",
              "联网簇·HTTP POST（a.java:10651-10769）：仅 run() f_bool_31 臂调用（a.java:3229-3242）；JAVAP "
              "putfield f_bool_31：<init>/run×2 置 0、唯一置 1 者 m_153（mode99 内）⇒ 不可达；同 m_148；裁决 DC-3"),
    "m_152": ("b", "基线23", "联网簇·状态读（a.java:10771-10782）：调用点 run mode99 臂 a.java:4271 与 m_153；"
                           "同 m_148；裁决 DC-3"),
    "m_153": ("b", "基线23",
              "联网簇·轮询/重试状态机（a.java:10784-10868）：唯一调用点 run mode99 臂 a.java:4267，且该臂先要求 "
              "f_bool_31 已真（a.java:4258），而 f_bool_31 唯一置真点就在 m_153 内（自门控）⇒ 即便 mode99 可达也永不执行；"
              "同 m_148（2026-10-15 独立审计补强）；裁决 DC-3"),
    "m_154": ("b", "基线23", "联网簇·hex 字符解码（a.java:10870-10878）：仅 m_151 调用（a.java:10730）；同 m_148；裁决 DC-3"),
    "addSpark": ("b", "修正暴露",
                 "火花登记（a.java:9683-9689）：调用点 4 处（a.java:3019-3031）均在 paint `switch (gameMode)` case 18"
                 "（a.java:3003-3031）；gameMode 写点无 18（论证同 m_148 的 JAVAP 普查）⇒ 死代码（completeness 迭代 60 "
                 "mode18 火花 ☠️）。v2 的唯一移植证据系漂移锚（combat.rs 注「spawnPopup kind 常量（…9679-9686…）」）；"
                 "裁决 DC-4"),
    # ---- c 判定存疑（工具方法论），解决方式见 PORT_NOTES / 归属规则 ----
    "loadPackedImages": ("c", "基线23",
              "packed 容器解码（a.java:4516-4549）已移植于 game-data packed_png.rs：其 seed 方言锚 "
              "`reference/seed/a.java:2657-2676` = seed `private void a(int n)`（序数对齐 = 本方法，原 m_001），v2 按 deobf "
              "行号读成 paint 区间。解决：v2.1 方言规则（无需人工条目）。注：dc470b2 原命名 readCountPrefix，"
              "e772fef 名实更正为 loadPackedImages（N-0434，方法体=打包图像容器装载器）"
              "疑名实不符——方法体为 packed 图像容器装载（resourceNames[i] 流内 u16 LE 长度前缀 + PNG 逐张 "
              "Image.createImage，a.java:4516-4549），非「读计数前缀」；data/naming 不在本任务写入范围，待命名批复核"),
    "readShortLE": ("c", "基线23",
                    "u16 读原语（a.java:6952-6954）已移植于 game-data lib.rs read::u16_le：其 seed 锚引 "
                    "`private static short a(InputStream)`（序数对齐 = 本方法）。解决：v2.1 方言 + CFR 签名点名规则。"
                    "注：方法体为 LE（低字节先），原名 readU16BE 名实不符——【2026-10-15 更正】276ac96 已按 N-0268 "
                    "名实更正改名 readShortLE（台账证据同步更正），本条随之改键"),
    "<clinit>": ("c", "基线23",
                 "static{}（a.java:10880-10882）：JAVAP static{} 0-51 putstatic f_int_arr_01 = "
                 "{0,8192,16384,24576,8462,270,90,8282}，54-93 另造一个 int[8] 后丢弃（无 putstatic）；VF 将前者"
                 "投影为字段初始化器 a.java:44，v2 不把字段初始化器行归属 <clinit>。Rust："
                 "crates/game-core/src/paint.rs `pub const NOKIA_TRANSFORM_TABLE: [i32; 8] = [0, 8192, 16384, 24576, 8462, 270, 90, 8282];`。"
                 "解决：v2.1 字段初始化器归属规则"),
    "initCellGrid": ("c", "基线23", "CellGrid::new 已移植（a.java:6956-6961；见 PORT_NOTES）：fn 异名且无行锚"),
    "findItemStackIndex": ("c", "基线23", "PlayerCombat::find_item 已移植（a.java:7658-7669；见 PORT_NOTES）：fn 异名 + 行锚陈旧"),
    "addItemToItemStack": ("c", "基线23", "add_item_to_stack 已移植（a.java:7721-7734；见 PORT_NOTES）：fn 异名 + 行锚漂移"),
    "easeCameraToTarget": ("c", "基线23", "两处内联已移植（a.java:8662-8667；见 PORT_NOTES）：无 fn、注释点名无行锚"),
    "initFloorStates": ("c", "基线23", "构造期吸收（a.java:9365-9371；见 PORT_NOTES）"),
    "m_120": ("c", "基线23", "内联于 save_floor_state（a.java:9402-9405；见 PORT_NOTES）"),
    "restoreFloorState": ("c", "基线23", "内联于层存档恢复（a.java:9679-9681；见 PORT_NOTES）"),
    "closeStreamsAndClear": ("c", "基线23", "I/O 清理无可观测效应（a.java:4634-4661；见 PORT_NOTES absorbed）：v2 无「吸收」口径"),
    "createSolidImage": ("c", "修正暴露", "结果被丢弃的纯函数（a.java:4605-4614；见 PORT_NOTES absorbed）；v2 证据系漂移锚"),
    "releaseSceneImages": ("c", "修正暴露", "内存释放被构造期装载吸收（a.java:4714-4728；见 PORT_NOTES）；v2 证据系漂移锚"),
    "resetNewGameState": ("c", "修正暴露", "新游戏复位分散移植（a.java:7191-7219；见 PORT_NOTES）；v2 证据系漂移锚 + seed 误读"),
    "addOption": ("c", "修正暴露", "OptionList::add 已移植（a.java:7269-7286；见 PORT_NOTES）；v2 证据系 28 条 seed 误读"),
    "itemTypeToStackIndex": ("c", "修正暴露", "item_desc_index 已移植（a.java:7705-7719；见 PORT_NOTES）；v2 证据系漂移锚"),
    "deleteRecordStoreQuietly": ("c", "修正暴露", "删后即写被壳覆盖写吸收（a.java:9324-9329；见 PORT_NOTES）；v2 证据系漂移锚"),
    "closeRecordStoreAndGc": ("c", "修正暴露", "RMS 清理无可观测效应（a.java:9331-9363；见 PORT_NOTES）；v2 证据系漂移锚"),
    "initParticles": ("c", "修正暴露", "Particles::new 已移植（a.java:9821-9831；见 PORT_NOTES）；v2 证据系漂移锚"),
}

# YELLOW_NOTES：黄项不触达论证（convergence-plan §0.1「黄项全部带不触达论证」）。逐条机械校验：
#   arg 中 a.java 行锚须落在该方法体内、Rust 锚（crates/…rs `标识符`）须存在；
#   only_callers ⇒ 两个 class 的字节码调用图中，该方法的调用者集合恰为所列（tools/dead_code_ruling.py 调用图）；
#   absent ⇒ (glob, 正则)：匹配文件的非注释行不得出现该模式（证明驱动面不含该事件）。
YELLOW_NOTES: dict[str, dict] = {
    "hideNotify": {
        "arg": ("MIDP 生命周期回调（a.java:4502-4513）：类 a 内零调用点，字节码唯一调用者 CMidlet.pauseApp@4"
                "（invokevirtual a.hideNotify），另由平台在 Canvas 离屏时回调；oracle 宿主只反射调 startApp、场景脚本"
                "只有按键事件（Runner.loadScript `<tick> <keycode> <press|release>`），shim/宿主源码无任何 "
                "pauseApp/hideNotify 调用 ⇒ 任何场景（含夹具场景）不可达。**非死代码**（真机来电/切后台必达），属"
                "场景驱动面盲区（扩驱动面须改 reference/ 宿主）。语义由壳侧宿主测试覆盖（ab8368a）："
                "crates/game-desktop/src/main.rs `fn in_game_hide_closes_bgm_and_first_key_resumes_game_track`、"
                "crates/game-desktop/src/main.rs `fn hide_in_game_menu_keeps_mode_4`、"
                "crates/game-desktop/src/main.rs `fn title_hide_resumes_menu_track_in_java_event_order`、"
                "crates/game-desktop/src/main.rs `fn hide_in_modes_0_and_21_clears_flag_without_resume`、"
                "crates/game-desktop/src/main.rs `fn hide_in_mode3_opens_in_game_menu`（gameMode 3→4 臂 a.java:4504-4507，"
                "47169cc 接通 crates/game-core/src/scene.rs `pub fn hide_notify(&mut self)` 后摘 #[ignore]）"),
        "only_callers": ["CMidlet.pauseApp"],
        "absent": [("reference/oracle/src/**/*.java", r'"(?:pauseApp|hideNotify)"'),
                   ("reference/oracle/src/**/*.java", r"(?<!void )\b(?:pauseApp|hideNotify)\s*\("),
                   ("reference/shim/src/**/*.java", r"(?<!void )\b(?:pauseApp|hideNotify)\s*\(")],
    },
}

# ARM_GAPS：方法级绿/黄内的**无 todo! 标记**或壳侧 BLOCKED 的臂级缺口（人工登记；a.java 行锚须落在方法体内、
# Rust 锚须存在——缺口被修复后锚失效 ⇒ ⚠ 提示销账）。§5 的 todo! 清单是机械扫描，本表补其盲区。
ARM_GAPS: list[tuple[str, str]] = [
    ("interactWithCell",
     "型 76/82 交互臂未移植——crates/game-core/src/scene.rs `_ => {} // 76/82：后续批次` 吞掉（无 todo! 标记，语义待考）。"
     "【2026-10-15 销账并改锚】原登记的型 10/11/12 撞击臂已移植（ee52191，a.java:5436-5456 逐值；"
     "fx_firebump_floor13_frames_match 解禁即绿，T301 钉点）——原锚 `_ => {} // 11 / 76/82` 失效即销账信号。"
     "【2026-10-16 更正】Java interactWithCell 无 case 76/82（default: continue=bump 放行），"
     "Rust `_ => {}` 已忠实——原“ARM_GAPS 在账”按 evidence-todos.md T-1 附带登记销账"),
]

# ---------------------------------------------------------------- v1 旧账（冻结，仅复算对照）
# 迭代 72（2eed129）原表逐字保留：v1 的 methods() 用 `l.split()[-1].split("(")[0]` 取名，
# 多参方法被吞成末参类型伪名（"int);" 等）、构造器成 "a"、<clinit> 漏收；本表另含字段名
# 与已不存在的方法名。复算这套口径即得旧账 44/1/58（见 §6 刷新前后对照）。
LEGACY_V1: dict[str, tuple[str, list[str]]] = {
    "boot/载入/存读档": ("green", ["m_000", "m_001", "m_006", "m_091", "m_093", "m_117", "m_118",
        "loadSlotTable", "loadFloorSave", "saveGlobalProgress", "m_119", "m_122", "m_059", "m_068",
        "loadProgressTarget", "bootLoadPhase", "loadStepIndex", "loadStepCount", "loadTargetMode",
        "loadCallInit", "slotCursor"]),
    "行走/相机": ("green", ["tryStep", "handleFieldInput", "m_055", "m_025", "isCellWalkable", "m_062",
        "rebuildWalkability", "findPath", "m_134", "m_103", "easeAfterimages", "lockCameraOn",
        "setCameraClamped", "stepCameraTowardTarget", "cameraAtTarget", "walkPhase", "walkStepCount",
        "walkPathBuffer", "stepProgressPx"]),
    "交互/门/钥匙": ("green", ["interactWithCell", "m_100", "m_101", "removeSlotAt", "consumeKeyForDoor",
        "spawnPopup", "pullEntityFromCell", "removeGroupAtCellByType", "m_062", "dispatchSpriteSize",
        "dispatchAnimFields", "spawnEntity", "markEntityRemoved", "markEntityDesRemoved", "m_107",
        "m_108", "predictHpLossVsType"]),
    "战斗/赏金": ("green", ["tickBattle", "spawnBossEvent", "predictHpLossVsType", "scaleEnemyStats",
        "enemyAtkScaled", "enemyHpScaled", "gainGold", "enemyBaseGold", "battleSceneRerun",
        "battleFinishing", "battleAnimTick", "removeLogoItem"]),
    "脚本 DSL": ("green", ["executeScriptInstruction", "tryRunScene", "loadLevelScript", "openDialogPage",
        "flipOverlayPage", "layoutWrappedText", "layoutFullText", "layoutWrappedTextLimited",
        "skipTextToLineEnd", "advanceScriptWalk", "scriptLineFlags", "dialogScriptLine",
        "parseScriptInt", "scriptCursor", "dialogPhase", "scriptStepDelay"]),
    "绘制/状态栏/HUD": ("green", ["paintEntityLayer", "paintTileLayer", "drawParallaxBackdrop",
        "paintPlayerSprite", "drawIconStrip", "paintMiniFrame", "paintNumber", "paintHud",
        "paintKeySlot", "paintSoftkeyBar", "drawEdgePatch", "buildMinimap", "m_003", "m_004", "m_142",
        "updateAndDrawParticles", "spawnParticle", "addSpark", "drawString"]),
    "道具/装备/炼丹": ("green", ["pickupItemType", "tryEquip", "m_083", "equipTierTypes",
        "equipTierBonuses", "addItemToItemStack", "itemTypeToStackIndex", "findItemStackIndex",
        "useItemStack", "m_075", "m_076", "alchemyPrice", "alchemyUpgradeCount", "m_082"]),
    "mode/菜单族": ("green", ["m_092", "m_093", "m_094", "m_095", "m_096", "m_097", "m_098", "m_099",
        "m_102", "menuCursorIndex", "menuItemKinds", "optionChecked", "m_091"]),
    "已知挂起（黄）": ("yellow", ["m_106", "m_107"]),
}


def git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, encoding="utf-8",
                          errors="replace")


def ledger_renames() -> dict[str, str]:
    """台账 method 记录（applied）：旧引用名（ref 末段，如 `a.m_093` → m_093）→ 现名。"""
    out = {}
    for l in LEDGER.read_text(encoding="utf-8").splitlines():
        if not l.strip():
            continue
        d = json.loads(l)
        if d.get("kind") == "method" and d.get("status") == "applied":
            tail = d.get("ref", "").split(".")[-1]
            if tail and tail != d.get("new"):
                out[tail] = d["new"]
    return out


def resolve_config(names: set[str]) -> list[str]:
    """人工条目的键若已被改名批次改掉（方法表里不存在、台账有 旧名→现名 且现名存在），就地改键
    （幂等）。返回改键说明——渲染进 §0 提示把条目键更新为现名；不再因改名批次 KeyError 崩溃
    （2026-10-15 readU16BE→readShortLE 前车）。"""
    al = ledger_renames()
    moved = []
    for table in (PORT_NOTES, TRIAGE, YELLOW_NOTES):
        for k in list(table):
            if k not in names and al.get(k) in names and al[k] not in table:
                table[al[k]] = table.pop(k)
                moved.append(f"{k}→{al[k]}")
    for i, (n, ev) in enumerate(ARM_GAPS):
        if n not in names and al.get(n) in names:
            ARM_GAPS[i] = (al[n], ev)
            moved.append(f"{n}→{al[n]}")
    return moved


# ---------------------------------------------------------------- ① 方法清单

def inventory() -> list[dict]:
    out = subprocess.run(["javap", "-p", "-s", str(RENAMED)], capture_output=True, text=True)
    if out.returncode != 0:
        print(f"javap 失败: {out.stderr}", file=sys.stderr)
        sys.exit(2)
    rows, pending = [], None
    for line in out.stdout.splitlines():
        s = line.strip()
        if pending is not None:
            m = re.match(r"descriptor: (\S+)", s)
            if m:
                pending["desc"] = m.group(1)
                pending["key"] = pending["name"] + m.group(1)
                rows.append(pending)
            pending = None
            continue
        if s == "static {};":
            pending = {"name": "<clinit>", "decl": s, "legacy": None}
        elif "(" in s and not s.startswith(("Compiled from", "public final class", "class ")):
            head = s.split("(")[0].split()[-1]
            name = "<init>" if head == "a" else head
            # v1 口径（复算旧账用）：末个空白分隔 token 去 "(" 后缀
            pending = {"name": name, "decl": s, "legacy": s.split()[-1].split("(")[0]}
    return rows


# ---------------------------------------------------------------- Java 源投影的区间模型

DECL_RE = re.compile(r"^( {3,4})((?:(?:public|private|protected|static|final|synchronized|native|abstract)\s+)+)"
                     r"([^;=(]*?)\b(\w+)\s*\(([^)]*)\)")
FIELD_INIT_RE = re.compile(r"^( {3,4})((?:public|private|protected|static|final|volatile|transient)\s+)+"
                           r"[\w\[\]<>.$]+\s+\w+\s*=")
SIG_QUOTE_RE = re.compile(r"(?:private|public|protected)\s+(?:static\s+|final\s+|synchronized\s+)*"
                          r"([\w\[\]<>.]+)\s+(\w+)\s*\(([^)`]*)\)")


def _strip_lits(s: str) -> str:
    s = re.sub(r'"(\\.|[^"\\])*"', "", s)
    return re.sub(r"'(\\.|[^'\\])'", "", s)


def _param_types(params: str) -> tuple[str, ...]:
    params = params.strip()
    if not params:
        return ()
    out = []
    for p in params.split(","):
        p = re.sub(r"^final\s+", "", p.strip())
        # "int var1" → int；"String" → String（签名引用常省参数名）；全限定名取末段
        t = p.rsplit(" ", 1)[0] if " " in p else p
        out.append(t.split(".")[-1])
    return tuple(out)


def parse_java(text: str) -> dict:
    """一个 a.java 投影（VF 3 空格 / CFR 4 空格缩进）→ 源序方法表 + 体区间 + 字段初始化器区间。"""
    lines = text.split("\n")
    decls = []
    for i, l in enumerate(lines, 1):
        if re.match(r"^ {3,4}static \{", l):
            decls.append({"name": "<clinit>", "start": i, "ret": "", "types": ()})
            continue
        m = DECL_RE.match(l)
        if m:
            # 构造器 = 无返回类型且名为类名 a（CFR seed 里大量普通方法也叫 a，须靠返回类型区分）
            ctor = m.group(4) == "a" and not m.group(3).strip()
            decls.append({"name": "<init>" if ctor else m.group(4), "start": i,
                          "ret": m.group(3).strip().split(".")[-1], "types": _param_types(m.group(5))})
    for d in decls:
        depth, started = 0, False
        for j in range(d["start"] - 1, len(lines)):
            t = _strip_lits(lines[j])
            depth += t.count("{") - t.count("}")
            started = started or "{" in t
            if started and depth == 0:
                d["end"] = j + 1
                break
    inside = set()
    for d in decls:
        inside.update(range(d["start"], d.get("end", d["start"]) + 1))
    fields = []  # (is_static, start, end)
    j = 0
    while j < len(lines):
        if (j + 1) not in inside and FIELD_INIT_RE.match(lines[j]):
            start, depth = j + 1, 0
            while j < len(lines):
                t = _strip_lits(lines[j])
                depth += t.count("{") - t.count("}")
                if depth <= 0 and t.rstrip().endswith(";"):
                    break
                j += 1
            fields.append((" static " in f" {FIELD_INIT_RE.match(lines[start - 1]).group(0)} ", start, j + 1))
        j += 1
    return {"decls": decls, "fields": fields, "n_lines": len(lines)}


def deobf_history() -> list[tuple[str, str]]:
    """VF 时代（VF_SINCE 起）deobf a.java 的全部历史 blob（去重）。git 不可用 ⇒ 空表。"""
    r = git("log", "--format=%H", f"{VF_SINCE}^..HEAD", "--", DEOBF_REL)
    if r.returncode != 0:
        return []
    seen, out = set(), []
    for sha in r.stdout.split():
        b = git("rev-parse", f"{sha}:{DEOBF_REL}").stdout.strip()
        if not re.fullmatch(r"[0-9a-f]{40}", b) or b in seen:
            continue
        seen.add(b)
        out.append((sha[:7], git("cat-file", "-p", b).stdout))
    return out


def java_model() -> dict:
    """现行 deobf 区间 + VF 历史版本求交的稳健区间 + seed 区间（均按方法序数对齐到现行名）。"""
    cur = parse_java(DEOBF.read_text(encoding="utf-8"))
    names = [d["name"] for d in cur["decls"]]
    sig = [d["types"] for d in cur["decls"]]
    warn = []

    def aligned(p: dict) -> bool:
        return len(p["decls"]) == len(names) and all(d["types"] == t for d, t in zip(p["decls"], sig))

    hist = []
    for label, text in deobf_history():
        p = parse_java(text)
        if aligned(p):
            hist.append((label, p))
        else:
            warn.append(f"deobf@{label} 方法序数与现行不对齐（{len(p['decls'])} 个），不参与稳健区间求交")
    versions = [cur] + [p for _, p in hist]
    robust: dict[str, list[tuple[int, int]]] = defaultdict(list)
    for k, n in enumerate(names):
        s = max(v["decls"][k]["start"] for v in versions)
        e = min(v["decls"][k]["end"] for v in versions)
        if s <= e:
            robust[n].append((s, e))
    fields_ok = [v for v in versions if [f[0] for f in v["fields"]] == [f[0] for f in cur["fields"]]]
    if len(fields_ok) != len(versions):
        warn.append(f"{len(versions) - len(fields_ok)} 个历史版本字段初始化器序列与现行不同，字段区间只用一致者求交")
    field_robust: dict[str, list[tuple[int, int]]] = defaultdict(list)
    for k, (is_static, _, _) in enumerate(cur["fields"]):
        s = max(v["fields"][k][1] for v in fields_ok)
        e = min(v["fields"][k][2] for v in fields_ok)
        if s <= e:
            field_robust["<clinit>" if is_static else "<init>"].append((s, e))
    seed = parse_java(SEED.read_text(encoding="utf-8"))
    seed_rng: dict[str, list[tuple[int, int]]] = defaultdict(list)
    seed_field: dict[str, list[tuple[int, int]]] = defaultdict(list)
    seed_by_sig: dict[tuple, str] = {}
    if aligned(seed):
        cnt = Counter((d["ret"], d["name"], d["types"]) for d in seed["decls"])
        for k, d in enumerate(seed["decls"]):
            seed_rng[names[k]].append((d["start"], d["end"]))
            if cnt[(d["ret"], d["name"], d["types"])] == 1:  # 签名（含返回类型）唯一者才可被引用点名
                seed_by_sig[(d["ret"], d["name"], d["types"])] = names[k]
        for is_static, s, e in seed["fields"]:
            seed_field["<clinit>" if is_static else "<init>"].append((s, e))
    else:
        warn.append("reference/seed/a.java 方法序数与现行不对齐——seed 方言锚不予归属")
    cur_rng = {d["name"]: (d["start"], d["end"]) for d in cur["decls"]}
    cur_field: dict[str, list[tuple[int, int]]] = defaultdict(list)
    for is_static, s, e in cur["fields"]:
        cur_field["<clinit>" if is_static else "<init>"].append((s, e))
    # v2 口径复算（§7 对照 +「修正暴露」一致性校验）钉在前任 v2 计算时的 deobf 行号体系：其后 deobf 行号漂移
    # （如 dc470b2 applyStepCellEffects +3 行）不得改变「前任口径」本身。按方法序数对齐到现名。
    v2_rng = cur_rng
    r = git("show", f"{V2_DEOBF_REV}:{DEOBF_REL}")
    if r.returncode == 0:
        p = parse_java(r.stdout)
        if aligned(p):
            v2_rng = {names[k]: (d["start"], d["end"]) for k, d in enumerate(p["decls"])}
        else:
            warn.append(f"deobf@{V2_DEOBF_REV} 与现行方法序数不对齐——v2 口径退回现行行号")
    return {"names": names, "cur": cur_rng, "cur_field": cur_field, "robust": robust,
            "field_robust": field_robust, "seed": seed_rng, "seed_field": seed_field,
            "seed_by_sig": seed_by_sig, "v2": v2_rng,
            "cur_by_sig": {(d["ret"], d["name"], d["types"]): d["name"] for d in cur["decls"]},
            "seed_lines": seed["n_lines"], "n_versions": len(versions), "warn": warn}


def java_ranges() -> dict[str, tuple[int, int]]:
    """现行 deobf a.java 每个方法的体区间（显示/调用点归属用）。"""
    p = parse_java(DEOBF.read_text(encoding="utf-8"))
    return {d["name"]: (d["start"], d["end"]) for d in p["decls"]}


def enclosing(rng: dict, line: int) -> str | None:
    best = None
    for n, (s, e) in rng.items():
        if s <= line <= e and (best is None or s > rng[best][0]):
            best = n
    return best


def call_sites(rng: dict, names: list[str]) -> dict[str, list[tuple[int, str]]]:
    """deobf a.java 中每个方法的调用点（行号, 所在方法）。B 级静态证据：文本调用点，
    与 javap 调用图交叉核对（cross_check_callers）。"""
    lines = DEOBF.read_text(encoding="utf-8").split("\n")
    decl_lines = {s for (s, _) in rng.values()}
    sites = defaultdict(list)
    # VF 投影常把 this 赋给局部别名再调用（`var13.m_107(...)`、`var21.addSpark(...)`）——
    # 接收者只认 this / a / varN；其他接收者（g.drawImage 等库调用）不算
    pats = {n: re.compile(rf"(?:\b(?:this|a|var\d+)\.|(?<![\w$.])){re.escape(n)}\(") for n in names
            if not n.startswith("<")}
    for i, l in enumerate(lines, 1):
        if i in decl_lines:
            continue
        code = re.sub(r'"(\\.|[^"\\])*"', '""', l)
        for n, p in pats.items():
            if p.search(code):
                sites[n].append((i, enclosing(rng, i) or "?"))
    return sites


def javap_callers() -> dict[str, set[str]]:
    out = subprocess.run(["javap", "-p", "-c", str(RENAMED)], capture_output=True, text=True).stdout
    callers, cur = defaultdict(set), None
    for line in out.splitlines():
        if line.startswith("  ") and not line.startswith("   ") and line.rstrip().endswith(";"):
            s = line.strip()
            cur = "<clinit>" if s == "static {};" else s.split("(")[0].split()[-1]
            cur = "<init>" if cur == "a" else cur
            continue
        m = re.search(r"invoke\w+\s+#\d+\s+// (?:Interface)?Method ([\w$<>\"]+):\(", line)
        if m and cur:
            callers[m.group(1).strip('"')].add(cur)
    return callers


# ---------------------------------------------------------------- ③ 用例 → 场景 → 窗口

def tracked_files() -> set[str] | None:
    """git 索引中的 crates/ 文件（含已暂存未提交者）；git 不可用 ⇒ None（不过滤）。
    并行任务的在制（未入库）测试不进账面——账面 = 入库树的函数（与 method_reach 夹具场景同口径）。"""
    r = git("ls-files", "--", "crates")
    return set(r.stdout.split()) if r.returncode == 0 else None


FRAME_FIX_RE = re.compile(r"assert_eq!\(\s*&?sha,\s*&?rec\.sha\b")
FRAMES_INC_RE = re.compile(r'include_str!\("([^"]+\.frames\.txt)"\)')


def read_frames_fixture(p: Path) -> dict:
    """tools/frames_fixture.py 夹具：注释头 `A-<stem>/trace.txt` 认领场景；帧链口径同 method_reach。"""
    stem, shas = None, []
    for line in p.read_text(encoding="utf-8").splitlines():
        if line.startswith("#"):
            m = re.search(r"A-([\w-]+)/trace\.txt", line)
            stem = stem or (m.group(1) if m else None)
        elif line.strip():
            shas.append(line.split()[1])
    import hashlib
    return {"stem": stem, "frames": len(shas), "chain": hashlib.sha256("\n".join(shas).encode()).hexdigest()}


def collect_tests() -> list[dict]:
    """TESTS_DIRS 下 *.rs（只认 git 已追踪者）机械解析。两类帧级用例：
    ① A-trace 用例（game-oracle 传统式）= 非 #[ignore] + 读 _out/A-*/trace.txt + `assert_eq!(&sha, expect…)`；
    ② 夹具帧用例（fixture-frames，2026-10-15 起）= 非 #[ignore] + 引用 `<stem>.frames.txt`（体内
       include_str!，或文件内唯一的该类 include 经辅助函数读入）+ 帧断言 `assert_eq!(sha, rec.sha…)`
       在体内或在体内调用的同文件辅助函数里；场景 = 夹具注释头的 `A-<stem>`。
    窗口：`assert_eq!(checked, records.len())` → [1, 帧数]（full）；`if rec.tick >= S` +
    `assert_eq!(checked, C)` → [S, S+C-1]；`assert_eq!(checked, C)` → [1, C]；
    `assert!(checked >= C)` → [1, C]（保守下界）。"""
    tracked = tracked_files()
    tests = []
    for d in TESTS_DIRS:
        for f in sorted(d.glob("*.rs")):
            rel = str(f.relative_to(ROOT))
            if tracked is not None and rel not in tracked:
                continue
            tests += _collect_file(f, rel)
    return tests


def _collect_file(f: Path, rel: str) -> list[dict]:
    text = f.read_text(encoding="utf-8")
    # 属性块允许夹 /// 文档注释（preset_floor35/10/46/42/43 的 #[test] 与 fn 间有文档）；
    # 只认以 `#[` 起首的行为属性（文档正文里出现 "#[ignore]" 字样不算）
    heads = list(re.finditer(r"((?:^[ \t]*(?:#\[[^\n]*\]|//[^\n]*)[ \t]*\n)+)[ \t]*fn (\w+)\(",
                             text, re.M))
    # 同文件辅助函数（任意顶层 fn）中含夹具帧断言者；及其 checked 窗口断言（`assert_eq!(checked, X.ticks…)`）
    fn_heads = list(re.finditer(r"^(?:pub )?fn (\w+)\(", text, re.M))
    frame_helpers, helper_ticks = set(), set()
    for i, m in enumerate(fn_heads):
        body = text[m.end():fn_heads[i + 1].start() if i + 1 < len(fn_heads) else len(text)]
        if FRAME_FIX_RE.search(body):
            frame_helpers.add(m.group(1))
            if re.search(r"assert_eq!\(\s*checked,\s*\w+\.ticks\b", body):
                helper_ticks.add(m.group(1))
    # 夹具规格常量（mov_route_scene.rs 式）：`const NAME: T = T { … frames: include_str!("…frames.txt"), …
    # ticks: N, … };` ⇒ NAME → (frames 路径, N)
    spec_consts = {}
    for m in re.finditer(r"^(?:pub )?(?:const|static) (\w+)\s*:[^=]*=\s*\w+\s*\{(.*?)^\};", text, re.M | re.S):
        inc = FRAMES_INC_RE.findall(m.group(2))
        tk = re.search(r"\bticks:\s*(\d+)", m.group(2))
        if len(inc) == 1:
            spec_consts[m.group(1)] = (inc[0], int(tk.group(1)) if tk else None)
    file_frames = sorted(set(FRAMES_INC_RE.findall(text)))
    out = []
    for i, m in enumerate(heads):
        attrs = [l.strip() for l in m.group(1).splitlines() if l.strip().startswith("#[")]
        name = m.group(2)
        if "#[test]" not in attrs:
            continue
        body = text[m.end():heads[i + 1].start() if i + 1 < len(heads) else len(text)]
        t = {"file": f.name, "path": rel, "fn": name, "line": text[:m.start(2)].count("\n") + 1,
             "ignored": any(x.startswith("#[ignore") for x in attrs),
             "trace_stem": None, "kind": None, "window": None, "fixture": None}
        tr = re.search(r"_out/A-([\w-]+)/trace\.txt", body)
        if tr:
            t["trace_stem"] = tr.group(1)
        frame = bool(re.search(r"assert_eq!\(\s*&sha,\s*expect", body))
        used_consts = [c for c in spec_consts if re.search(rf"(?<![\w$]){re.escape(c)}(?![\w$])", body)]
        inc = FRAMES_INC_RE.findall(body) or (
            [spec_consts[used_consts[0]][0]] if len(used_consts) == 1 else
            (file_frames if len(file_frames) == 1 else []))
        called = [h for h in frame_helpers if re.search(rf"\b{h}\(", body)]
        fx_frame = bool(FRAME_FIX_RE.search(body) or called)
        if inc and not tr:
            fp = (f.parent / inc[0]).resolve()
            if fp.exists():
                fx = read_frames_fixture(fp)
                t["fixture"] = {"path": str(fp.relative_to(ROOT)), **fx}
                t["trace_stem"] = fx["stem"]
        if t["ignored"]:
            t["kind"] = "ignored"
        elif (tr and frame) or (t["fixture"] and t["fixture"]["stem"] and fx_frame):
            t["kind"] = "frame"
            g = re.search(r"if rec\.tick >= (\d+)", body)
            c = re.search(r"assert_eq!\(\s*checked,\s*(\d+)", body)
            c2 = re.search(r"assert!\(\s*checked\s*>=\s*(\d+)", body)
            if re.search(r"assert_eq!\(\s*checked,\s*records\.len\(\)", body):
                t["window"] = (1, None)
            elif g and c:
                t["window"] = (int(g.group(1)), int(g.group(1)) + int(c.group(1)) - 1)
            elif c:
                t["window"] = (1, int(c.group(1)))
            elif c2:
                t["window"] = (1, int(c2.group(1)))
            elif (t["fixture"] and len(used_consts) == 1 and spec_consts[used_consts[0]][1]
                  and any(h in helper_ticks for h in called)):
                # 辅助函数断言 checked == 规格 ticks（逐帧循环遍历夹具全部记录）⇒ [1, ticks]
                t["window"] = (1, spec_consts[used_consts[0]][1])
            else:
                t["kind"] = "frame-unbounded"  # 帧断言但无 checked 计数：不计入验证
        elif tr:
            t["kind"] = "trace-partial"   # 读 A-trace 但非逐帧（ops 前缀/锚点）
        elif "include_str!(\"fixtures/" in body or inc:
            t["kind"] = "fixture"
        else:
            t["kind"] = "structural"
        out.append(t)
    return out


# ---------------------------------------------------------------- ④ 移植信号

def snake(c: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", c).lower()


def ledger_mech_ids() -> dict[str, set[str]]:
    """台账：当前名 → 机械名（m_NNN）集合（Rust 有以机械名命名的 fn，如 m_083_index）。"""
    out = defaultdict(set)
    for l in LEDGER.read_text(encoding="utf-8").splitlines():
        if not l.strip():
            continue
        d = json.loads(l)
        if d.get("kind") == "method" and d.get("status") == "applied":
            m = re.search(r"\bm_(\d{3})\b", d.get("ref", ""))
            if m:
                out[d["new"]].add(f"m_{m.group(1)}")
    return out


ANCHOR_RE = re.compile(r"((?:reference/)?(?:seed|src/deobf|deobf)/)?a\.java:(\d+)(?:\s*[-–~]\s*(\d+))?")


def port_files() -> list[str]:
    out = []
    for c in PORT_CRATES:
        out += glob.glob(str(ROOT / "crates" / c / "src/**/*.rs"), recursive=True)
    return sorted(out)


def method_aliases(names: list[str]) -> tuple[re.Pattern, dict[str, str], set[str]]:
    """点名识别：现名 ∪ 台账 ref 末段 ∪ 机械名 m_NNN（指向多个现名的歧义别名弃用）。"""
    alias = {n: {n} for n in names if not n.startswith("<")}
    for l in LEDGER.read_text(encoding="utf-8").splitlines():
        if not l.strip():
            continue
        d = json.loads(l)
        if d.get("kind") != "method" or d.get("new") not in alias:
            continue
        ref = d.get("ref", "")
        m = re.search(r"\bm_\d{3}\b", ref)
        if m:
            alias[d["new"]].add(m.group(0))
        tail = ref.split(".")[-1]
        if re.fullmatch(r"[A-Za-z_]\w*", tail) and tail != "a":
            alias[d["new"]].add(tail)
    al2cur = defaultdict(set)
    for cur, als in alias.items():
        for a in als:
            al2cur[a].add(cur)
    amb = {a for a, c in al2cur.items() if len(c) > 1}
    toks = sorted((a for a in al2cur if a not in amb), key=len, reverse=True)
    rx = re.compile(r"(?<![\w$])(" + "|".join(map(re.escape, toks)) + r")(?![\w$])")
    return rx, {a: next(iter(c)) for a, c in al2cur.items() if a not in amb}, amb


def rust_signals(jm: dict) -> tuple[dict, list]:
    """fn 名 / 行锚（v2.1 归属）/ 点名提及 / todo! 挡路点 / v2 口径朴素归属（对照用）。"""
    rx, al2cur, amb = method_aliases(jm["names"])
    fns = defaultdict(list)
    refs = defaultdict(list)        # 现名 → ["file:line〔名|稳|字段〕"]
    naive = defaultdict(list)       # v2 口径：锚区间与现行方法体区间相交即归属（不分方言/点名）
    mentions = defaultdict(list)
    prec = Counter()                # 名证锚精度自检：到被点名方法区间的最近距离分桶
    todos = []

    def dist(a: int, b: int, s: int, e: int) -> int:
        return 0 if (a <= e and b >= s) else min(abs(a - e), abs(b - s))

    files = port_files()
    v2_files = sorted(glob.glob(str(ROOT / "crates/*/src/**/*.rs"), recursive=True))
    for f in sorted(set(files) | set(v2_files)):
        rel = str(Path(f).relative_to(ROOT))
        is_port = f in files
        text = Path(f).read_text(encoding="utf-8")
        seed_file = bool(re.search(r"seed/a\.java:\d", text))
        for ln, line in enumerate(text.split("\n"), 1):
            loc = f"{rel}:{ln}"
            for m in ANCHOR_RE.finditer(line):  # v2 口径（含 game-oracle/src，与前任一致）
                a = int(m.group(2))
                b = max(a, int(m.group(3))) if m.group(3) else a
                if b - a <= 400:
                    for n, (s, e) in jm["v2"].items():
                        if a <= e and b >= s:
                            naive[n].append(loc)
            if not is_port:
                continue
            for m in re.finditer(r"\bfn ([A-Za-z_]\w*)", line):
                fns[m.group(1)].append(loc)
            named = {al2cur[m.group(1)] for m in rx.finditer(line)}
            for m in SIG_QUOTE_RE.finditer(line):  # 引用的 Java 签名（CFR seed 名 / deobf 名）
                key = (m.group(1).split(".")[-1], m.group(2), _param_types(m.group(3)))
                hit = jm["cur_by_sig"].get(key) or jm["seed_by_sig"].get(key)
                if hit:
                    named.add(hit)
            for n in named:
                mentions[n].append(loc)
            for m in ANCHOR_RE.finditer(line):
                a = int(m.group(2))
                b = max(a, int(m.group(3))) if m.group(3) else a
                if b - a > 400:
                    continue
                pre = m.group(1) or ""
                if "seed" in pre or (not pre and seed_file and b <= jm["seed_lines"]):
                    body = jm["seed"]
                    robust, fields, tag = jm["seed"], jm["seed_field"], "seed"
                else:
                    body = {n: [r] for n, r in jm["cur"].items()}
                    robust, fields, tag = jm["robust"], jm["field_robust"], ""
                if named:
                    best = None
                    for n in named:
                        d = min((dist(a, b, s, e) for s, e in body.get(n, [])), default=None)
                        if d is None:
                            continue
                        best = d if best is None else min(best, d)
                        if d <= NAMED_TOL:
                            refs[n].append(f"{loc}〔{tag}名〕")
                    if best is not None:
                        prec["0" if best == 0 else "1-5" if best <= 5 else "6-10" if best <= 10 else
                             "11-20" if best <= 20 else "21-50" if best <= 50 else ">50"] += 1
                    continue
                for n, rs in robust.items():
                    if any(s <= a - ROBUST_MARGIN and b + ROBUST_MARGIN <= e for s, e in rs):
                        refs[n].append(f"{loc}〔{tag}稳〕")
                for n, rs in fields.items():
                    if any(s <= a and b <= e for s, e in rs):
                        refs[n].append(f"{loc}〔{tag}字段〕")
            if re.search(r"\b(todo|unimplemented)!\(", line):
                code = line.split("//", 1)[0]
                if not re.search(r"\b(todo|unimplemented)!\(", code):
                    continue  # 注释散文提及（如"todo!() 作死码哨兵"）不是挡路点
                msg = re.search(r"(?:todo|unimplemented)!\(\s*\"([^\"]*)", line)
                if not msg:
                    for ahead in text.split("\n")[ln:ln + 2]:  # 多行调用 todo!(\n "…"
                        am = re.match(r"\s*\"([^\"]*)", ahead)
                        if am:
                            msg = am
                            break
                todos.append({"at": loc, "msg": msg.group(1) if msg else code.strip()})
    return {"fns": fns, "refs": refs, "naive": naive, "mentions": mentions, "prec": prec,
            "amb": amb}, todos


def port_evidence(name: str, sig: dict, mech: dict, v2: bool = False) -> dict:
    """移植证据。v2=True 复算前任 v2 口径（朴素行锚、无人工条目），只用于 §7 对照。"""
    cands = {name, snake(name)}
    fn_hits = sorted({p for c in cands for p in sig["fns"].get(c, [])})
    ids = set(mech.get(name, set()))
    if re.fullmatch(r"m_\d{3}", name):
        ids.add(name)
    for k, paths in sig["fns"].items():
        if any(re.search(rf"(^|_){i}(_|$)", k) for i in ids):
            fn_hits += [f"{p}（fn {k}）" for p in paths]
    if v2:
        refs = sig["naive"].get(name, [])
        return {"fn": fn_hits, "refs": refs, "note": None, "ported": bool(fn_hits or refs)}
    refs = sig["refs"].get(name, [])
    note = PORT_NOTES.get(name)
    if note and note[0] == "not-ported":
        ported = False
    elif note:
        ported = True
    else:
        ported = bool(fn_hits or refs)
    return {"fn": fn_hits, "refs": refs, "note": note, "ported": ported}


RUST_PAIR_RE = re.compile(r"(crates/[\w/.-]+\.rs)\s*`([^`]+)`")


def _bad_rust_pairs(tag: str, text: str) -> list[str]:
    bad = []
    for path, ident in RUST_PAIR_RE.findall(text):
        p = ROOT / path
        if not p.exists() or ident not in p.read_text(encoding="utf-8"):
            bad.append(f"{tag} 锚失效：{path} `{ident}`")
    return bad


def check_notes(rng: dict, n_lines: int) -> list[str]:
    """PORT_NOTES / TRIAGE 逐条校验（锚存在、类别合法、a.java 指针在界内）。"""
    bad = []
    for n, (kind, ev) in PORT_NOTES.items():
        tag = f"PORT_NOTES[{n}]"
        if n not in rng:
            bad.append(f"{tag} 方法不存在")
            continue
        if kind not in ("ported", "absorbed", "not-ported"):
            bad.append(f"{tag} 判定非法 {kind}")
        bad += _bad_rust_pairs(tag, ev)
        if kind == "ported" and not RUST_PAIR_RE.search(ev):
            bad.append(f"{tag} ported 缺 Rust 文件锚")
        if kind == "absorbed":
            s, e = rng[n]
            if not any(s <= int(x) <= e for x in re.findall(r"a\.java:(\d+)", ev)):
                bad.append(f"{tag} absorbed 缺落在方法体 a.java:{s}-{e} 内的行锚")
    for n, (cls, origin, ev) in TRIAGE.items():
        tag = f"TRIAGE[{n}]"
        if n not in rng:
            bad.append(f"{tag} 方法不存在")
            continue
        if cls not in ("a", "b", "c") or origin not in ("基线23", "修正暴露"):
            bad.append(f"{tag} 类/来源非法 {cls}/{origin}")
        ptrs = [int(x) for x in re.findall(r"a\.java:(\d+)", ev)]
        if not ptrs:
            bad.append(f"{tag} 缺 a.java 行锚")
        if any(x > n_lines for x in ptrs):
            bad.append(f"{tag} a.java 行锚越界")
        bad += _bad_rust_pairs(tag, ev)
    return bad


RULING_PARTS = ("主张", "证据", "可复现命令", "结论")


def ruling_sections() -> dict[str, str]:
    """裁决书 `## DC-n …` 节 → 节正文（至下一个同级/更高级标题）。"""
    if not RULINGS_DOC.exists():
        return {}
    text = RULINGS_DOC.read_text(encoding="utf-8")
    heads = list(re.finditer(r"^(#{2,3}) (DC-\d+)\b.*$", text, re.M))
    out = {}
    for i, m in enumerate(heads):
        lvl = len(m.group(1))
        end = len(text)
        for n in re.finditer(r"^(#{1,%d}) " % lvl, text[m.end():], re.M):
            end = m.end() + n.start()
            break
        out[m.group(2)] = text[m.start():end]
    return out


def check_rulings(drs: dict, hits: dict) -> tuple[dict, list[str]]:
    """TRIAGE b 条目的「裁决 DC-n」三重校验：① tools/dead_code_ruling.py 字节码不动点证死该方法；
    ② 裁决书含 DC-n 节且点名该方法、四段（主张/证据/可复现命令/结论）俱全；③ 动态一致：全部场景
    （JDI 实测）零触达。全过 ⇒ ☠️ 已裁死；否则 ⚠ 并退回「待裁」。返回 ({方法: DC-n}, 告警)。"""
    secs = ruling_sections()
    ruled, bad = {}, []
    for n, (cls, _, ev) in TRIAGE.items():
        if cls != "b":
            continue
        m = re.search(r"裁决 (DC-\d+)", ev)
        if not m:
            continue
        rid = m.group(1)
        tag = f"TRIAGE[{n}] 裁决 {rid}"
        r = drs.get(rid)
        ok = True
        if r is None or n not in r["methods"]:
            bad.append(f"{tag}：dead_code_ruling.RULINGS 无此裁决对象")
            ok = False
        elif not r["methods"][n].get("dead"):
            bad.append(f"{tag}：字节码复核未证死（{r['methods'][n].get('error', '有可达调用点')}）")
            ok = False
        sec = secs.get(rid)
        if sec is None:
            bad.append(f"{tag}：裁决书 {RULINGS_DOC.relative_to(ROOT)} 缺「## {rid}」节")
            ok = False
        else:
            # 裁决书以裁决时名字成文；改名批次落地后亦认台账旧名（不要求改写已定案文书）
            olds = {o for o, cur in ledger_renames().items() if cur == n}
            if not any(re.search(rf"(?<![\w$]){re.escape(x)}(?![\w$])", sec) for x in {n} | olds):
                bad.append(f"{tag}：裁决书 {rid} 节未点名 {n}")
                ok = False
            miss = [p for p in RULING_PARTS if p not in sec]
            if miss:
                bad.append(f"{tag}：裁决书 {rid} 节缺 {'/'.join(miss)}")
                ok = False
        if hits.get(n):
            bad.append(f"{tag}：与动态证据矛盾——场景触达 {hits[n][:2]}")
            ok = False
        if ok:
            ruled[n] = rid
    return ruled, bad


def _absent_violations(glob_pat: str, pattern: str) -> list[str]:
    rx = re.compile(pattern)
    out = []
    for p in sorted(ROOT.glob(glob_pat)):
        for ln, line in enumerate(p.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
            s = line.strip()
            if s.startswith(("*", "/*", "//")):
                continue
            if rx.search(line):
                out.append(f"{p.relative_to(ROOT)}:{ln}")
    return out


def _bad_body_anchor(tag: str, ev: str, rng: dict, n: str) -> list[str]:
    s, e = rng[n]
    if not any(s <= int(x) <= e for x in re.findall(r"a\.java:(\d+)", ev)):
        return [f"{tag} 缺落在方法体 a.java:{s}-{e} 内的行锚"]
    return []


def check_yellow(rng: dict, an: dict) -> tuple[set, list[str]]:
    """YELLOW_NOTES 机械校验。返回 (论证成立的方法集, 告警)。"""
    import dead_code_ruling as dcr
    ok, bad = set(), []
    for n, note in YELLOW_NOTES.items():
        tag = f"YELLOW_NOTES[{n}]"
        if n not in rng:
            bad.append(f"{tag} 方法不存在")
            continue
        b = _bad_body_anchor(tag, note["arg"], rng, n) + _bad_rust_pairs(tag, note["arg"])
        if "only_callers" in note:
            got = sorted({c for c, _ in dcr.callers_of(an, n)})
            if got != sorted(note["only_callers"]):
                b.append(f"{tag} 字节码调用者 {got} ≠ 声明 {note['only_callers']}")
        for g, pat in note.get("absent", []):
            v = _absent_violations(g, pat)
            if v:
                b.append(f"{tag} 驱动面出现 /{pat}/：{', '.join(v[:3])}")
        bad += b
        if not b:
            ok.add(n)
    return ok, bad


def check_gaps(rng: dict) -> list[str]:
    bad = []
    for n, ev in ARM_GAPS:
        tag = f"ARM_GAPS[{n}]"
        if n not in rng:
            bad.append(f"{tag} 方法不存在")
            continue
        bad += _bad_body_anchor(tag, ev, rng, n) + _bad_rust_pairs(tag, ev)
    return bad


# ---------------------------------------------------------------- ② 触达

def load_snapshot() -> dict:
    if not SNAPSHOT.exists():
        print(f"缺 {SNAPSHOT.relative_to(ROOT)} —— 先跑 python3 tools/method_reach.py", file=sys.stderr)
        sys.exit(2)
    return json.loads(SNAPSHOT.read_text(encoding="utf-8"))


def verified_hits(tests: list[dict], snap: dict) -> tuple[dict, list[dict]]:
    """方法 key → [(用例, 场景, t_hit)]：帧级用例窗口 ∩ 场景实测触达，且测得运行与
    用例消费的 A-trace FRAME 序列一致（equal；或窗口完全落在一致前缀内）。"""
    by_key = defaultdict(list)
    rows = []
    for t in tests:
        if t["kind"] != "frame":
            continue
        s = snap["scenarios"].get(t["trace_stem"])
        row = dict(t)
        if s is None or "phases" not in s:
            row["status"] = "无触达快照"
            rows.append(row)
            continue
        at = s.get("a_trace")
        start, end = t["window"]
        fx = t.get("fixture")
        if fx is not None:
            # 夹具帧用例：Rust 比对的期望帧 = 夹具；须与 JDI 实测运行的 FRAME 链逐字节相同
            # （帧数 + 链 sha256），测到的触达才是被 Rust 全帧验证的那次执行
            if end is None:
                end = fx["frames"]
            row["window_resolved"] = (start, end)
            ok = fx["frames"] == s["frames"] and fx["chain"] == s["frame_chain"]
            eff_end = min(end, s["frames"])
            row["status"] = "FRAME 一致（夹具帧链 = 实测）" if ok else "FRAME 失配（夹具帧链 ≠ 实测）"
        else:
            if end is None:
                end = at["frames"] if at else s["frames"]
            row["window_resolved"] = (start, end)
            # equal：测得运行 = 用例消费的 A-trace（逐帧）；prefix：一侧为另一侧前缀——只在
            # 公共前缀内计触达（保守：前缀外的首次命中不入账）
            ok = at is not None and at["match"] in ("equal", "prefix")
            eff_end = min(end, s["frames"], at["frames"]) if ok else end
            row["status"] = ("FRAME 一致" if ok and at["match"] == "equal" else
                             f"FRAME 前缀一致（计至 T{eff_end}）" if ok else
                             f"FRAME 失配（{at['match'] if at else '无 A-trace'}）")
        rows.append(row)
        if not ok:
            continue
        # 相位语义（MethodReach.java）：相位 p 在 p 处重布全部断点、只记 t_hit ≥ p 的首次命中，
        # 且**于下一相位起点截止**（旧断点被删）。窗口 [start, end] 内执行过的方法 =
        # 各相位 p ∈ [start, end] 记录的并集（同方法取最小 t_hit）——后一相位的首次命中
        # 必是 start 以来首次执行（否则前一相位已记到）。start 不是相位起点（快照陈旧）时
        # 退化为最大的 p < start 且只认 t_hit ≥ start（保守：可能漏记、不会多记）。
        first: dict[str, int] = {}
        ps = sorted(int(p) for p in s["phases"])
        base = max((p for p in ps if p <= start), default=None)
        if base is None:
            continue
        for p in ps:
            if p < base or p > eff_end:
                continue
            for key, th in s["phases"][str(p)].items():
                if th >= start and (key not in first or th < first[key]):
                    first[key] = th
        for key, th in first.items():
            if th <= eff_end:
                by_key[key].append((f"{Path(t['file']).stem}::{t['fn']}", t["trace_stem"], th))
    return by_key, rows


def full_run_hits(s: dict) -> dict[str, int]:
    """全程首次命中 = 各相位记录的并集取最小 t_hit（相位 1 只覆盖到下一相位起点前）。"""
    first: dict[str, int] = {}
    for ph in s.get("phases", {}).values():
        for key, th in ph.items():
            if key not in first or th < first[key]:
                first[key] = th
    return first


def any_hits(snap: dict) -> dict:
    out = defaultdict(list)
    for stem, s in snap["scenarios"].items():
        for key, th in full_run_hits(s).items():
            out[key].append((stem, th))
    return out


# ---------------------------------------------------------------- v1 复算

def legacy_status(pseudo: str | None) -> tuple[str, str]:
    if pseudo is None:
        return "absent", "v1 未收（<clinit>）"
    for sub, (st, names) in LEGACY_V1.items():
        if pseudo in names:
            return st, sub
    return "white", "未入账"


def v1_era_names(jm: dict) -> dict[str, str]:
    """现名 → v1 时（迭代 72，V1_REV）的方法名：按方法序数对齐 V1_REV 的 deobf（逐序参数类型须一致）。
    v1 伪名对 ≤1 参方法 = 方法名本身，随后续改名批次漂移；钉回 v1 时名字才能保证 v1 复算 44/1/58 不变
    （2026-10-15 dc470b2 批量改名前车；台账回溯不可行——部分改名早于 v1，回溯会越过 v1 时点）。"""
    r = git("show", f"{V1_REV}:{DEOBF_REL}")
    if r.returncode != 0:
        jm["warn"].append(f"取不到 {V1_REV}:{DEOBF_REL}——v1 伪名按现名计（v1 复算可能漂移）")
        return {}
    old = parse_java(r.stdout)["decls"]
    cur = parse_java(DEOBF.read_text(encoding="utf-8"))["decls"]
    if len(old) != len(cur) or any(a["types"] != b["types"] for a, b in zip(cur, old)):
        jm["warn"].append(f"deobf@{V1_REV} 与现行方法序数不对齐——v1 伪名按现名计")
        return {}
    return {a["name"]: b["name"] for a, b in zip(cur, old)}


# ---------------------------------------------------------------- 渲染

SYM = {"green": "绿", "yellow": "黄", "white": "白", "absent": "未收"}
CLS = {"a": "a 真未移植", "b": "b 不触达论证候选", "c": "c 判定存疑（工具方法论）"}


def classify(inv, sig, mech, vhits, v2: bool = False) -> dict[str, str]:
    out = {}
    for m in inv:
        pe = port_evidence(m["name"], sig, mech, v2=v2)
        out[m["name"]] = ("green" if vhits.get(m["key"]) else "yellow") if pe["ported"] else "white"
    return out


def render() -> tuple[str, dict]:
    inv = inventory()
    jm = java_model()
    v1n = v1_era_names(jm)
    for m in inv:  # ≤1 参方法的 v1 伪名 = 方法名 ⇒ 钉回 v1 时名字
        if m["legacy"] == m["name"]:
            m["legacy"] = v1n.get(m["name"], m["name"])
    rng = jm["cur"]
    names = [m["name"] for m in inv]
    moved = resolve_config(set(names))
    missing_rng = [n for n in names if n not in rng]
    sites = call_sites(rng, names)
    jcallers = javap_callers()
    tests = collect_tests()
    snap = load_snapshot()
    vhits, trows = verified_hits(tests, snap)
    ahits = any_hits(snap)
    sig, todos = rust_signals(jm)
    mech = ledger_mech_ids()
    n_lines = len(DEOBF.read_text(encoding="utf-8").split("\n"))
    bad_notes = check_notes(rng, n_lines)

    rows = []
    for m in inv:
        n, key = m["name"], m["key"]
        pe = port_evidence(n, sig, mech)
        v = sorted(vhits.get(key, []), key=lambda x: (x[2], x[0]))
        h = sorted(ahits.get(key, []), key=lambda x: (x[1], x[0]))
        color = ("green" if v else "yellow") if pe["ported"] else "white"
        lst, lsub = legacy_status(m["legacy"])
        rows.append({"name": n, "key": key, "range": rng.get(n), "port": pe, "verified": v,
                     "hits": h, "color": color, "legacy": (m["legacy"], lst, lsub),
                     "triage": TRIAGE.get(n), "callers_src": sites.get(n, []),
                     "callers_bc": sorted(jcallers.get(n, set())),
                     "mentions": sig["mentions"].get(n, [])})
    by = defaultdict(list)
    for r in rows:
        by[r["color"]].append(r)
    cnt = {c: len(by[c]) for c in ("green", "yellow", "white")}
    color_of = {r["name"]: r["color"] for r in rows}
    for r in rows:  # c 类须已解决（现非白）
        if r["triage"] and r["triage"][0] == "c" and r["color"] == "white":
            bad_notes.append(f"TRIAGE[{r['name']}] 判 c 但仍白——解决手段（PORT_NOTES/归属规则）未生效")
    v2c = classify(inv, sig, mech, vhits, v2=True)
    v2cnt = Counter(v2c.values())
    # 「修正暴露」= v2 口径有移植账、v2.1 机械信号（fn/行锚）全空者——须与 TRIAGE 登记一致。基线 23 白项按定义
    # 在 v2 口径下为白（来源即「基线23」），后续 Rust 注释的宽区间锚即便令 v2 朴素口径改判，也不属「修正暴露」
    # （2026-10-15 6a37155 `m_106 / m_108（a.java:8738-8904）` 区间锚覆盖 m_107 前车）
    base23 = {n for n, (_, o, _) in TRIAGE.items() if o == "基线23"}
    exposed = {r["name"] for r in rows if v2c[r["name"]] != "white"
               and not (r["port"]["fn"] or r["port"]["refs"])} - base23
    declared = {n for n, (_, o, _) in TRIAGE.items() if o == "修正暴露"}
    if exposed != declared:
        bad_notes.append(f"TRIAGE「修正暴露」与机械复算不符：漏登 {sorted(exposed - declared)}；"
                         f"多登 {sorted(declared - exposed)}")

    # v1 复算（伪名去重口径）
    leg_names = sorted({m["legacy"] for m in inv if m["legacy"] is not None})
    leg_cnt = defaultdict(int)
    for p in leg_names:
        leg_cnt[legacy_status(p)[0]] += 1

    wcls = Counter(r["triage"][0] if r["triage"] else "none" for r in by["white"])
    tri = Counter((o, c) for (c, o, _) in TRIAGE.values())
    base_now = Counter(color_of[n] for n, (c, o, _) in TRIAGE.items()
                       if o == "基线23" and c == "c" and n in color_of)

    # 死代码裁决（☠️）/ 黄项不触达论证 / 臂级缺口：逐条机械复核
    import dead_code_ruling as dcr
    an = dcr.analyze()
    drs = dcr.rulings(an)
    hits_by_name = {r["name"]: r["hits"] for r in rows}
    ruled, bad_r = check_rulings(drs, hits_by_name)
    argued, bad_y = check_yellow(rng, an)
    bad_notes += bad_r + bad_y + check_gaps(rng)
    b_white = [r for r in by["white"] if r["triage"] and r["triage"][0] == "b"]
    n_ruled = sum(1 for r in b_white if r["name"] in ruled)
    y_hit = [r for r in by["yellow"] if r["hits"]]
    y_nohit = [r for r in by["yellow"] if not r["hits"]]
    y_arg = [r for r in y_nohit if r["name"] in argued]
    deficit_w = cnt["white"] - n_ruled
    deficit_y = cnt["yellow"] - len(y_arg)

    L: list[str] = []
    a = L.append
    a("# 覆盖矩阵（层次一账本，机器生成）")
    a("")
    a("> 生成：`python3 tools/cover_matrix.py`（v2.1，2026-10-15 刷账：v2 重写 + v2.1 行锚归属治理；v1 = 迭代 72）。")
    a("> 触达数据：`python3 tools/method_reach.py` → `tools/method_reach/snapshot.json`。")
    a("> **请勿手改本文件**——改判定请改 tools/cover_matrix.py 的 PORT_NOTES/TRIAGE/YELLOW_NOTES/ARM_GAPS"
      "（逐条带证据，脚本校验锚点）。")
    a("> 判据文本见 [docs/spec/completeness.md](spec/completeness.md)「覆盖矩阵刷账（2026-10-15）」；"
      "白项 b 类死代码裁决书见 [docs/spec/dead-code-rulings.md](spec/dead-code-rulings.md)"
      "（`python3 tools/dead_code_ruling.py --check` 字节码机械复核）。")
    a("")
    a(f"## 汇总：绿 {cnt['green']} / 黄 {cnt['yellow']} / 白 {cnt['white']}（共 {len(rows)} 方法）")
    a("")
    a(f"- 白项分账（现白 {cnt['white']}）：{CLS['a']} {wcls['a']} / b 不触达 {wcls['b']}"
      f"（☠️ 已裁死 {n_ruled} / 待裁 {wcls['b'] - n_ruled}）"
      + (f" / c 未解决 {wcls['c']}" if wcls["c"] else "")
      + (f" / **未分账 {wcls['none']}**" if wcls["none"] else " / 未分账 0"))
    a_done = sum(1 for n, (c, o, _) in TRIAGE.items() if o == "基线23" and c == "a" and color_of.get(n) != "white")
    a(f"- 刷账三分账（§4.1）：基线 23 白 = a {tri[('基线23', 'a')]}（其后已移植 {a_done}）/ b {tri[('基线23', 'b')]} / "
      f"c {tri[('基线23', 'c')]}（c 已解决 → 现绿 {base_now['green']} / 黄 {base_now['yellow']}）；"
      f"v2.1 归属治理新暴露 {sum(v for (o, _), v in tri.items() if o == '修正暴露')} 项 = "
      f"b {tri[('修正暴露', 'b')]} / c {tri[('修正暴露', 'c')]}（c 经核验恢复移植账）")
    a(f"- 黄项：{cnt['yellow']}（被无对拍用例的场景触达 {len(y_hit)} = 补用例即可转绿；无任何场景触达 "
      f"{len(y_nohit)} = 带不触达论证 {len(y_arg)} / 待论证 {len(y_nohit) - len(y_arg)}——convergence-plan §0.1）")
    a(f"- **判据①赤字**（convergence-plan §0.1：白项清零、黄项带论证）：白（非 ☠️）{deficit_w} + "
      f"黄（无论证）{deficit_y} = {deficit_w + deficit_y}"
      + ("（白 = " + ", ".join(r["name"] for r in by["white"] if r["name"] not in ruled) + "）"
         if deficit_w else ""))
    a(f"- 口径复算（同一份 Rust 树）：v2 前任口径 绿 {v2cnt['green']} / 黄 {v2cnt['yellow']} / 白 {v2cnt['white']}"
      f"（差异见 §7）；v1 旧账（伪名口径，{len(leg_names)} 名）绿 {leg_cnt['green']} / 黄 {leg_cnt['yellow']} / "
      f"白 {leg_cnt['white']}（对照见 §6）")
    if missing_rng:
        a(f"- ⚠ deobf 区间缺失：{', '.join(missing_rng)}")
    for w in jm["warn"]:
        a(f"- ⚠ {w}")
    if moved:
        a(f"- ⚠ 人工条目键随台账改名就地映射（请把 tools/cover_matrix.py 条目键改为现名）：{', '.join(moved)}")
    if sig["amb"]:
        a(f"- ⚠ 歧义别名（不用于点名）：{', '.join(sorted(sig['amb']))}")
    for b in bad_notes:
        a(f"- ⚠ {b}")
    a("")
    a("三色判定（逐方法，机械）：**绿** = 有移植账 ∧ 被帧级对拍用例窗口内的 Java 执行触达；"
      "**黄** = 有移植账 ∧ 无帧级窗口触达；**白** = 无移植账。"
      "**Stage A 判据（convergence-plan §0.1）：白项清零、黄项带不触达论证**。"
      "方法级绿 ≠ 方法内每个分支已验——臂级缺口见 §5（Rust `todo!` 挡路点）与 type-closure.md；"
      "绿也 ≠ 副作用已验：帧 sha 不含 RMS/音频，此类方法的绿只证明 Java 执行落在已验窗口内。")
    a("")
    # §0 方法论自检
    a("## 0. 移植账归属规则与自检（v2.1）")
    a("")
    a(f"- 行锚方言：显式 `seed/a.java` 或 seed 方言文件（含显式 seed 锚的文件）的裸锚 → reference/seed/a.java；"
      f"其余 → deobf。seed 与 deobf 以方法序数对齐（运行时校验 163 个、逐序参数类型一致）。")
    a(f"- 名证锚：锚所在行点名 Java 方法（现名/台账别名/引用签名）⇒ 只归属被点名者，容差 ±{NAMED_TOL} 行；"
      f"稳健锚：未点名 ⇒ 锚外扩 {ROBUST_MARGIN} 行仍须落在 VF 时代 {jm['n_versions']} 个 deobf 版本"
      f"（{VF_SINCE} 起 + 工作区）方法区间的交集内；字段初始化器行归 <clinit>/<init>（不外扩）。")
    pk = ["0", "1-5", "6-10", "11-20", "21-50", ">50"]
    tot = sum(sig["prec"].values())
    a(f"- 行锚精度自检（名证锚 {tot} 条到被点名方法区间的最近距离）："
      + "，".join(f"{k} 行 {sig['prec'][k]}" for k in pk)
      + f"——>{NAMED_TOL} 行者多为「点名调用者、引用别处行」的语境锚，不予归属；"
      "v2 朴素口径（锚落区间即算）把漂移锚记给邻近小方法，§7 列出全部差异。")
    live = [c for c in PORT_CRATES if (ROOT / "crates" / c).exists()]
    a(f"- 移植扫描范围：crates/{{{','.join(live)}}}/src（game-oracle 为差分宿主，不计移植证据）。")
    a("")

    # §1 用例映射
    dirs = "、".join(str(d.relative_to(ROOT)) for d in TESTS_DIRS)
    a(f"## 1. 对拍用例 → 场景 → 窗口（{dirs} 机械解析，只认 git 已追踪文件）")
    a("")
    a("帧级用例两类：A-trace 用例（运行期读 `_out/A-<场景>/trace.txt`）与夹具帧用例（期望帧嵌入 "
      "`<场景>.frames.txt`，tools/frames_fixture.py 产出；夹具帧链须 = JDI 实测运行的帧链才计入验证）。")
    a("")
    a("| 用例 | 场景（A-trace） | 窗口 | FRAME 对照 | 验证触达方法数 |")
    a("|---|---|---|---|---|")
    vcount = defaultdict(int)
    for key, lst in vhits.items():
        for (tid, _, _) in lst:
            vcount[tid] += 1
    for t in trows:
        tid = f"{Path(t['file']).stem}::{t['fn']}"
        w = t.get("window_resolved") or t["window"]
        stem = t["trace_stem"] + ("（夹具）" if t.get("fixture") else "")
        a(f"| `{tid}` | {stem} | T{w[0]}-{w[1]} | {t['status']} | {vcount.get(tid, 0)} |")
    a("")
    others = [t for t in tests if t["kind"] != "frame"]
    if others:
        a("非帧级用例（不计入验证；列出以示归属）：")
        a("")
        oracle_dir = TESTS_DIRS[0]
        for t in others:
            if not t["path"].startswith(str(oracle_dir.relative_to(ROOT))):
                continue
            src = (f"，夹具 {t['trace_stem']}" if t.get("fixture") else
                   f"，读 A-{t['trace_stem']}" if t["trace_stem"] else "")
            a(f"- `{Path(t['file']).stem}::{t['fn']}`（{t['file']}:{t['line']}）：{t['kind']}{src}")
        per = defaultdict(Counter)
        for t in others:
            if not t["path"].startswith(str(oracle_dir.relative_to(ROOT))):
                per[t["path"]][t["kind"]] += 1
        for p, c in sorted(per.items()):
            a(f"- {p}：{sum(c.values())} 个（" + " / ".join(f"{k} {v}" for k, v in sorted(c.items())) + "）")
        a("")
    tested = {t["trace_stem"] for t in tests if t["kind"] == "frame"}
    a("场景触达（JDI 实测，全程 = 各相位并集取最小 t_hit；相位起点 = 1 ∪ 用例窗口起点）：")
    a("")
    a("| 场景 | 帧数 | 结束 | 相位 | 触达方法数 | A-trace 对照 | 帧级用例 |")
    a("|---|---|---|---|---|---|---|")
    for stem, s in snap["scenarios"].items():
        at = s.get("a_trace")
        ph = ",".join(sorted(s.get("phases", {}), key=int))
        src = "（夹具）" if at and at.get("source") == "fixture" else ""
        a(f"| {stem} | {s.get('frames')} | {s.get('end', '').replace('END ', '')} | {ph} | "
          f"{len(full_run_hits(s))} | {at['match'] + src if at else '无 A-trace'} | "
          f"{'✓' if stem in tested else '—'} |")
    a("")

    def anchor(r):
        return f"a.java:{r['range'][0]}-{r['range'][1]}" if r["range"] else "a.java:?"

    def port_str(r):
        pe = r["port"]
        parts = []
        if pe["note"]:
            kind = {"ported": "人工", "absorbed": "人工·吸收", "not-ported": "人工·压制"}[pe["note"][0]]
            parts.append(f"{kind}：{pe['note'][1]}")
        if pe["fn"]:
            parts.append("fn " + "; ".join(pe["fn"][:2]) + (" …" if len(pe["fn"]) > 2 else ""))
        if pe["refs"]:
            kinds = Counter(re.search(r"〔(.*)〕", x).group(1) for x in pe["refs"])
            ks = "/".join(f"{k}{v}" for k, v in sorted(kinds.items()))
            parts.append(f"行锚×{len(pe['refs'])}〔{ks}〕（{pe['refs'][0].split('〔')[0]}）")
        return "；".join(parts) or "—"

    # §2 绿
    a(f"## 2. 绿：已移植 + 帧级对拍窗口触达（{cnt['green']}）")
    a("")
    a("| 方法 | 区间 | 首个验证（用例 / 场景 @t_hit） | 验证用例数 | 移植证据 |")
    a("|---|---|---|---|---|")
    for r in sorted(by["green"], key=lambda r: r["range"] or (0, 0)):
        tid, st, th = r["verified"][0]
        n_t = len({x[0] for x in r["verified"]})
        a(f"| {r['name']} | {anchor(r)} | `{tid}` / {st} @T{th} | {n_t} | {port_str(r)} |")
    a("")

    # §3 黄
    a(f"## 3. 黄：已移植 + 无帧级窗口触达（{cnt['yellow']}）")
    a("")
    a("| 方法 | 区间 | 场景触达（无用例） | 移植证据 | 调用点（deobf） | 处置 |")
    a("|---|---|---|---|---|---|")
    for r in sorted(by["yellow"], key=lambda r: r["range"] or (0, 0)):
        h = "; ".join(f"{s}@T{t}" for s, t in r["hits"][:3]) or "**无场景触达**"
        cs = ", ".join(f"{c}:{ln}" for ln, c in r["callers_src"][:4])
        if r["hits"]:
            disp = "补帧级用例即转绿"
        elif r["name"] in argued:
            disp = "不触达论证（B，机械校验 ✓）：" + YELLOW_NOTES[r["name"]]["arg"]
        else:
            disp = "**待论证或新场景**"
        a(f"| {r['name']} | {anchor(r)} | {h} | {port_str(r)} | {cs} | {disp} |")
    a("")

    def cls_str(n: str, tr) -> str:
        if not tr:
            return "**未分账**"
        if tr[0] == "b":
            rid = ruled.get(n)
            if rid:
                v = drs[rid]["methods"][n]
                return f"b ☠️ 已裁死 {rid}（字节码复核 ✓：调用点 {len(v['sites'])} 处全不可达）"
            return "b 不触达（**待裁**）"
        if tr[0] == "a" and color_of.get(n) not in (None, "white"):
            return f"a 真未移植 → 已移植（现{SYM[color_of[n]]}）"
        return CLS.get(tr[0], tr[0])

    # §4 白
    a(f"## 4. 白：无移植账（{cnt['white']}）")
    a("")
    a("| 方法 | 区间 | 分账 | 场景触达 | Rust 点名提及 | 论证 / 依据 |")
    a("|---|---|---|---|---|---|")
    for r in sorted(by["white"], key=lambda r: r["range"] or (0, 0)):
        tr = r["triage"]
        h = "; ".join(f"{s}@T{t}" for s, t in r["hits"][:3]) or "无"
        mn = f"{len(r['mentions'])}（{r['mentions'][0]}）" if r["mentions"] else "0"
        a(f"| {r['name']} | {anchor(r)} | {cls_str(r['name'], tr)} | {h} | {mn} | {tr[2] if tr else '—'} |")
    a("")
    a("### 4.1 刷账三分账（2026-10-15：基线 23 白 + v2.1 归属治理新暴露）")
    a("")
    a("c 类 = 白色系工具方法论所致（移植存在但漏判 / 无可观测效应无「吸收」口径）；解决手段为"
      "归属规则修正或 PORT_NOTES 人工条目（逐条校验 Rust 锚），解决后现色随之变化。"
      "b 类 ☠️ = 死代码终裁（裁决书 + 字节码机械复核 + 全场景零触达三重校验；不计判据①赤字）。")
    a("")
    a("| 方法 | 区间 | 来源 | 分账 | 现色 | 场景触达 | 论证 / 依据 |")
    a("|---|---|---|---|---|---|---|")
    order = {"a": 0, "b": 1, "c": 2}
    by_name = {r["name"]: r for r in rows}
    known = {n: v for n, v in TRIAGE.items() if n in by_name}  # 未知键已由 check_notes 报 ⚠
    for n, (c, o, ev) in sorted(known.items(), key=lambda kv: (kv[1][1] != "基线23", order[kv[1][0]],
                                                               (by_name[kv[0]]["range"] or (0, 0)))):
        r = by_name[n]
        h = "; ".join(f"{s}@T{t}" for s, t in r["hits"][:2]) or "无"
        a(f"| {n} | {anchor(r)} | {o} | {cls_str(n, (c, o, ev))} | {SYM[r['color']]} | {h} | {ev} |")
    a("")

    # §5 todo
    a("## 5. 臂级缺口：Rust `todo!` 显式挡路点（方法级绿/黄内的未移植分支）")
    a("")
    for t in todos:
        a(f"- {t['at']}：{t['msg']}")
    a("")
    a("### 5.1 无 `todo!` 标记的臂级缺口（人工登记 ARM_GAPS，锚逐条校验；修复后锚失效即 ⚠ 提示销账）")
    a("")
    for n, ev in ARM_GAPS:
        r = by_name.get(n)
        a(f"- {n}（{anchor(r) if r else 'a.java:?'}，现色 {SYM[r['color']] if r else '?'}）：{ev}")
    a("")

    # §6 v1 对照
    a("## 6. 刷新前后对照（v1 伪名口径 → v2.1 逐方法）")
    a("")
    a("v1 取名 bug：`l.split()[-1].split(\"(\")[0]` ⇒ ≥2 参方法被吞成末参类型伪名，"
      "构造器成 `a`，`<clinit>` 漏收。下表按伪名列出其吞掉的真实方法与现色。")
    a("")
    a("| v1 名（v1 色） | 真实方法 → 现色 |")
    a("|---|---|")
    groups = defaultdict(list)
    for r in rows:
        groups[r["legacy"][0]].append(r)
    for p in sorted(groups, key=lambda x: (x is None, str(x))):
        st = legacy_status(p)[0]
        members = ", ".join(f"{r['name']}→{SYM[r['color']]}" for r in groups[p])
        a(f"| `{p}`（{SYM[st]}） | {members} |")
    a("")
    moves = defaultdict(int)
    for r in rows:
        moves[(r["legacy"][1], r["color"])] += 1
    a("逐方法迁移计数（v1 色 → 现色）：" + "，".join(
        f"{SYM[k[0]]}→{SYM[k[1]]} {v}" for k, v in sorted(moves.items())))
    a("")
    a("### 6.1 v1 白项去向（逐真实方法）")
    a("")
    b1 = [r for r in rows if r["legacy"][1] == "white" and r["color"] == "green"]
    b2 = [r for r in rows if r["legacy"][1] == "white" and r["color"] == "yellow"]
    ww = [r for r in rows if r["legacy"][1] == "white" and r["color"] == "white"]
    a(f"- **账面滞后 → 绿 {len(b1)}**：" + ", ".join(r["name"] for r in b1))
    a(f"- **账面滞后 → 黄 {len(b2)}**：" + ", ".join(r["name"] for r in b2))
    a(f"- **仍白 {len(ww)}**（分账见 §4/§4.1）：" + ", ".join(
        f"{r['name']}〔{r['triage'][0] if r['triage'] else '?'}〕" for r in ww))
    dn = [r for r in rows if r["legacy"][1] in ("green", "yellow") and r["color"] != r["legacy"][1]]
    if dn:
        a("")
        a("v1 绿/黄 → 现色变色（v1 过账或口径差，逐条）：" + ", ".join(
            f"{r['name']}（{SYM[r['legacy'][1]]}→{SYM[r['color']]}）" for r in dn))
    a("")

    # §7 v2 → v2.1 差异
    a("## 7. 前任 v2 口径 → v2.1 差异（同一份 Rust 树与触达快照复算）")
    a("")
    a("| 方法 | v2 色 | 现色 | v2 行锚（朴素归属） | v2.1 行锚 | 人工条目 |")
    a("|---|---|---|---|---|---|")
    for r in sorted(rows, key=lambda r: r["range"] or (0, 0)):
        old = v2c[r["name"]]
        n_old = len(sig["naive"].get(r["name"], []))
        n_new = len(r["port"]["refs"])
        if old == r["color"] and (n_old > 0) == (n_new > 0):
            continue
        note = r["port"]["note"][0] if r["port"]["note"] else "—"
        a(f"| {r['name']} | {SYM[old]} | {SYM[r['color']]} | {n_old} | {n_new} | {note} |")
    a("")
    text = "\n".join(L) + "\n"
    return text, {"counts": cnt, "legacy": dict(leg_cnt), "v2": dict(v2cnt), "white_cls": dict(wcls),
                  "bad": bad_notes, "ruled": n_ruled, "deficit": deficit_w + deficit_y}


def normalize(text: str) -> str:
    """账面归一化：抹去 Rust 文件行号与锚计数（并行改 crates/ 只会移动这些细节）。"""
    text = re.sub(r"(crates/[\w/.-]+\.rs):\d+", r"\1", text)
    text = re.sub(r"行锚×\d+〔[^〕]*〕", "行锚×N", text)
    text = re.sub(r"\| \d+（(crates/[\w/.-]+\.rs)）", r"| N（\1）", text)
    # §0 精度自检计数、§7 锚计数随 Rust 注释增删而动，只保留「有/无」
    text = re.sub(r"名证锚 \d+ 条", "名证锚 N 条", text)
    text = re.sub(r"(0|1-5|6-10|11-20|21-50|>50) 行 \d+", r"\1 行 N", text)
    return re.sub(r"^(\| \S+ \| . \| . \| )(\d+) \| (\d+) \|",
                  lambda m: f"{m.group(1)}{'N' if int(m.group(2)) else 0} | {'N' if int(m.group(3)) else 0} |",
                  text, flags=re.M)


def main() -> int:
    text, info = render()
    if "--check" in sys.argv:
        cur = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        strict = cur == text
        loose = normalize(cur) == normalize(text)
        print("覆盖矩阵 --check：" + ("逐字节一致" if strict else
                                     "账面一致（仅 Rust 行号/锚计数漂移）" if loose else
                                     "账面不一致（重跑 python3 tools/cover_matrix.py）"))
        return 0 if loose else 1
    OUT.write_text(text, encoding="utf-8")
    c, w = info["counts"], info["white_cls"]
    print(f"覆盖矩阵 → {OUT.relative_to(ROOT)}（绿 {c['green']} / 黄 {c['yellow']} / 白 {c['white']}；"
          f"白分账 a {w.get('a', 0)} / b {w.get('b', 0)}（☠️ {info['ruled']}）/ c {w.get('c', 0)} / "
          f"未分账 {w.get('none', 0)}；判据①赤字 {info['deficit']}；"
          f"v2 口径复算 {info['v2'].get('green', 0)}/{info['v2'].get('yellow', 0)}/{info['v2'].get('white', 0)}；"
          f"v1 复算 {info['legacy'].get('green', 0)}/{info['legacy'].get('yellow', 0)}/{info['legacy'].get('white', 0)}"
          + (f"；⚠ {len(info['bad'])} 条校验告警" if info["bad"] else "") + "）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
