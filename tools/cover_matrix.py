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
  ③ 用例 → 场景 → 窗口：crates/game-oracle/tests/*.rs 机械解析（collect_tests；
     帧级 = 含 `assert_eq!(&sha, expect…)` 且非 #[ignore]；窗口由 checked 断言派生）
  ④ 移植账（只扫移植目标 crate：PORT_CRATES；game-oracle 是差分宿主不计）：
     Rust fn 直证（同名/蛇形/台账机械名 m_NNN）∪ Rust 注释 a.java 行锚（v2.1 归属规则，
     见下）∪ PORT_NOTES 人工条目（ported/absorbed/not-ported，逐条校验锚点）
  ⑤ 白项分账：TRIAGE 人工条目（a/b/c + 论证 + a.java 行锚，逐条校验）；v1 旧账
     （LEGACY_V1）原样保留只用于"刷新前后对照"复算（44/1/58 可复现）

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
TESTS_DIR = ROOT / "crates/game-oracle/tests"
SNAPSHOT = ROOT / "tools/method_reach/snapshot.json"
OUT = ROOT / "docs" / "coverage-matrix.md"

# 移植目标 crate（AGENTS.md §2：game-oracle 是差分宿主，不是移植目标）
PORT_CRATES = ("game-core", "game-data", "game-platform", "game-desktop", "game-wasm")
# VF 时代起点（反编译引擎 CFR→Vineflower，deobf 行号体系自此稳定；game-core 首提
# 41d8fce 晚于此，deobf 方言锚全部写于 VF 时代）
VF_SINCE = "996a29d"
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
    "m_059": ("ported",
              "crates/game-core/src/entity.rs `pub fn new(cols: i32, rows: i32) -> Self`"
              "（CellGrid::new 四件初始化 = a.java:6954-6957；doc 点名「m_059 初始化」）"),
    "findItemStackIndex": ("ported",
                           "crates/game-core/src/combat.rs `pub fn find_item(&self, item_type: u8) -> i32`"
                           "（doc「findItemStackIndex 忠实移植」，首个同型槽/无则 -1 = a.java:7656-7665；"
                           "其 Rust 行锚 8485 陈旧，落入 executeScriptInstruction）"),
    "addItemToItemStack": ("ported",
                           "crates/game-core/src/scene.rs `fn add_item_to_stack(&mut self, t: i32)`"
                           "（追加/叠加次数 + showOverlayMessage = a.java:7719-7728/7730；行锚 7702-7714 漂移落入 "
                           "itemTypeToStackIndex）。未移植 itemUseTargetType 写（a.java:7729）——无可见效应：拾取路径调用前 "
                           "已置 overlay_kind_item（crates/game-core/src/scene.rs `self.overlay_kind_item = t`），"
                           "商人路径浮层同调用内被关闭/替换（2026-10-15 独立审计核）"),
    "m_103": ("ported",
              "crates/game-core/src/scene.rs `// m_103：ease 相机目标 = 目标格`（SEE 臂）+ "
              "crates/game-core/src/scene.rs `// m_103：ease 目标 = SEE 目标格`（翻页臂）两处内联，"
              "四赋值 = a.java:8660-8663"),
    "m_118": ("ported",
              "crates/game-core/src/scene.rs `saved_floor_states: vec![None; 56]`（新游戏构造即 56 槽空 = "
              "a.java:9363-9367；读档路径 crates/game-core/src/scene.rs "
              "`scene.saved_floor_states = save.floor_states.clone()`）"),
    "m_120": ("ported",
              "crates/game-core/src/scene.rs `fn save_floor_state(&mut self)`（u16 BE 写内联为 to_be_bytes = "
              "a.java:9400-9401）"),
    "m_123": ("ported",
              "crates/game-core/src/scene.rs `u16::from_be_bytes([d[*p], d[*p + 1]])`（层存档恢复 read_u16 "
              "闭包 = a.java:9677，m_122 恢复臂 a.java:9623-9635 内联）"),
    "m_008": ("absorbed",
              "流关闭 + 三字段置 null（a.java:4635-4660）；javap 字段普查：f_InputStream_00/"
              "f_OutputStream_00/f_InputStream_01 读写者仅 m_001/m_008/m_139/m_148/m_151/<init>（I/O 方法）"
              "⇒ 无游戏态效应；Rust 以内存切片解码无流可关（crates/game-data/src/packed_png.rs "
              "`pub fn parse(data: &'a [u8], image_count: usize) -> Result<Self, Error>`）"),
    # —— 方法论修正（v2.1 行锚归属）暴露、经核验确有移植/吸收者 ——
    "m_006": ("absorbed",
              "静态纯函数（造 32×32 纯色 RGB 图，a.java:4606-4613）；仅有的两个调用点 loadFloorData "
              "a.java:9420-9421 结果被丢弃（javap：invokestatic m_006 后紧跟 pop）⇒ 无可观测效应"),
    "releaseSceneImages": ("absorbed",
                           "图像容器/entityTypeImage 置 null 释放内存（a.java:4715-4727）；读者在读前经 "
                           "m_001 判空重载（a.java:4519）或 initEntityTables 重建 entityTypeImage（javap："
                           "put 仅 initEntityTables/releaseSceneImages）；Rust 构造期全量装载不释放"
                           "（crates/game-core/src/scene.rs `懒加载资源容器（Rust 构造期全量装载，等价 no-op）`）"),
    "m_068": ("ported",
              "新游戏态复位分散移植：crates/game-desktop/src/main.rs `GameScene::load_floor(` 以 (32,352) "
              "落点 = placePlayerAtCell(1,11)（a.java:7194 floor51 臂，新游戏 step8 已置 51 层）+ "
              "crates/game-core/src/scene.rs `difficulty_index: 0`（装备/跟随/金/道具栈/炼丹/难度构造默认 0 = "
              "a.java:7189-7190、7210-7214）+ crates/game-desktop/src/main.rs `scale_enemy_stats(&tables, 1)`"
              "（= a.java:7215 scaleEnemyStats(multipliers[0])）"),
    "m_072": ("ported",
              "crates/game-core/src/menu_family.rs `fn add(&mut self, label_idx: usize, checked: bool)`"
              "（OptionList::add = addOption：stringWidth+80 定框宽、首项 32+行高 = a.java:7267-7277；"
              "原 28 条行锚系 game-data seed 方言锚误归属）"),
    "itemTypeToStackIndex": ("ported",
                             "crates/game-core/src/scene.rs `fn item_desc_index(&self, t: i32) -> usize`"
                             "（85→20 / 86→21 / 其余 t-13 = a.java:7704-7713；add_item_to_stack 内联同式）"),
    "deleteRecordStoreQuietly": ("absorbed",
                                 "删库后重建（a.java:9322-9325）；三调用点 a.java:4476（SKY_WAR 读失败→删→"
                                 "saveGlobalProgress 重建）/9018（MOT_IF）/9181（MOT_L{n}）均为删后即写；"
                                 "壳 RMS 以覆盖写吸收删+写（crates/game-desktop/src/main.rs "
                                 "`std::fs::write(&path, v).ok();`）；SKY_WAR 路径的持久化缺口归 saveGlobalProgress（a）"),
    "m_117": ("absorbed",
              "RecordStore/流关闭 + 五字段置 null + gc（a.java:9329-9359）；f_byte_arr_27 读者"
              "（m_000/loadSlotTable/loadFloorSave）均先 getRecord 赋值再读（a.java:4459/8971/9253）"
              "⇒ 无可观测效应"),
    "m_128": ("ported",
              "crates/game-core/src/title.rs `pub fn new() -> Particles`（32 槽粒子数组清零 + 游标 0 = "
              "a.java:9819-9827）"),
    # m_131：v2 唯一行锚（pathfind.rs findPath 注）系误归属；v2.1 另有合法名证锚（intro.rs:274
    # 「m_131(target, call_m000)」），机械仍判移植——此条只补记真实移植位置，不改色
    "m_131": ("ported",
              "crates/game-core/src/intro.rs `pub fn new(target: i32, call_m000: bool, steps: &[i8]) -> LoadProgress`"
              "（loadTargetMode/loadCallInit/步表/计数复位 = a.java:9874-9881；m_001(15) 由构造期装载吸收）"),
}

# TRIAGE：白项三分账。值 = (类, 来源, 论证)。类：
#   "a" 真未移植（后续铺层/壳层赤字）；"b" 不触达论证候选（死内容/冷路径，B 级以上论证）；
#   "c" 判定存疑（工具方法论问题；须有 PORT_NOTES 或归属规则修正作为解决，现色随之变化）。
# 来源："基线23" = 前任 v2 刷账输出的 23 白项；"修正暴露" = v2.1 行锚归属治理后新暴露者。
# 复现命令约定：JAVAP = `javap -p -c analysis/rename-pipeline/renamed/a.class`
TRIAGE: dict[str, tuple[str, str, str]] = {
    # ---- a 真未移植 ----
    "m_106": ("a", "基线23",
              "MOV 五参实体路径缓动（a.java:8735-8867）：run() dialogPhase 2 臂调用（a.java:4073）；dialogPhase=2 "
              "由 MOV 五参臂在 m_100 命中实体时置（a.java:8285-8296）；levelScriptLines（a.java:1124-1193，68 条）"
              "含五参 MOV 36 处（脚本 1/2/4/5/7/8/10/12/19/30/57/61），其中 20 处起点（型,格）在 jar sprite{n} "
              "初始布置中有对应实体（如脚本 1 `MOV_72_3_7_1_8` = sprite2 型 72@(3,7)；脚本 12 的 13 处全落 "
              "sprite40）⇒ 正常游玩可达。Rust：crates/game-core/src/scene.rs "
              "`todo!(\"m_106 实体路径缓动（MOV 对拍批）\")`；即 convergence-plan P1 挂起项 dp2=m_106"),
    "m_108": ("a", "基线23",
              "实体跨格槽表重登记（a.java:8877-8901）：唯一调用者 m_106（a.java:8803/8845；javap 调用图同）"
              "⇒ 随 m_106 可达、随 m_106 一并移植；Rust 无对应实现"),
    "saveGlobalProgress": ("a", "基线23",
                           "SKY_WAR 全局进度写（a.java:8914-8948：optionChecked×4 + f_int_arr_35 + "
                           "f_int_152..154）；调用点 m_000 a.java:4299/4478、run a.java:4169（选项切换后）。"
                           "Rust：save.rs 对 SKY_WAR 既不编码也不解码（只有 parse_floor_save/parse_preset）；game-core 无 I/O"
                           "（crates/game-core/src/scene.rs `saveGlobalProgress（RMS 侧，无像素效应；game-core 无 I/O）`），"
                           "boot 恒走 SKY_WAR 读失败回退（crates/game-core/src/paint.rs `RMS \"SKY_WAR\" 读档失败路径`）；"
                           "壳 Rms 只改写 MOT_L*/MOT_IF（crates/game-desktop/src/main.rs "
                           "`stores.push((\"MOT_IF@1\".into(), mot_if.clone()));`）⇒ 选项/全局旗标跨会话不持久"
                           "（读写双缺，读臂在 m_000 case 21 a.java:4450-4479）——壳层赤字（会话内帧不可见，非铺层赤字）"),
    # ---- b 不触达论证候选 ----
    "m_093": ("b", "基线23",
              "左向三角箭头（a.java:8031-8033）：唯一调用点 paintPlayerSprite walkPhase case 3 道具轮盘"
              "（a.java:5977/5982）；JAVAP 中 putfield walkPhase 共 23 处，前导常量 ∈{0,1,2,4,5}（无 3，"
              "亦无计算值写入）⇒ walkPhase 永不为 3，case 3 绘制臂与 run() `switch (walkPhase)`（a.java:3565）的 case 3 键处理臂（a.java:3647-3673）皆死代码"),
    "m_107": ("b", "基线23",
              "实体换格（a.java:8869-8875）：唯一调用点 executeScriptInstruction 的 MVS 臂（a.java:8516-8589，"
              "`!var3.equals(\"MVS\")` 之 else）；levelScriptLines（a.java:1124-1193）68 条脚本 MVS 出现 0 次；"
              "executeScriptInstruction 三调用点均传 dialogScriptLine（a.java:4021/8299/8434）；JAVAP：dialogScriptLine "
              "唯一写点 loadLevelScript（aaload levelScriptLines），levelScriptLines 仅 <init> 赋值且无元素写 ⇒ "
              "脚本文本源封闭、MVS 死指令（crates/game-core/src/script.rs `数据域 68 条脚本零使用`）"),
    "m_148": ("b", "基线23",
              "联网簇·连接关闭（a.java:10607-10636）：调用点 run mode99 臂 a.java:4261、m_151/m_153（簇内）。"
              "mode 99 不可达：JAVAP putfield gameMode 54 处，常量 ∈{0-5,7-12,14-17,19-22}，计算来源 "
              "menuReturnMode∈{1,4}/loadTargetMode∈{3}/inGameMenuReturnMode←gameMode ⇒ 无 99；"
              "另 master-plan §4 范围裁剪（HttpConnection 联网整体剔除）"),
    "m_149": ("b", "基线23", "联网簇·响应体 u16 读（a.java:10638-10640）：仅 m_150/m_153 调用（簇内）；簇不可达论证同 m_148"),
    "m_150": ("b", "基线23", "联网簇·响应体 u32 读（a.java:10642-10644）：仅 m_153 调用（簇内）；同 m_148"),
    "m_151": ("b", "基线23",
              "联网簇·HTTP POST（a.java:10648-10766）：仅 run() f_bool_31 臂调用（a.java:3229-3242）；JAVAP "
              "putfield f_bool_31：<init>/run×2 置 0、唯一置 1 者 m_153（mode99 内）⇒ 不可达；同 m_148"),
    "m_152": ("b", "基线23", "联网簇·状态读（a.java:10768-10779）：调用点 run mode99 臂 a.java:4271 与 m_153；同 m_148"),
    "m_153": ("b", "基线23",
              "联网簇·轮询/重试状态机（a.java:10781-10865）：唯一调用点 run mode99 臂 a.java:4267，且该臂先要求 "
              "f_bool_31 已真（a.java:4258），而 f_bool_31 唯一置真点就在 m_153 内（自门控）⇒ 即便 mode99 可达也永不执行；"
              "同 m_148（2026-10-15 独立审计补强）"),
    "m_154": ("b", "基线23", "联网簇·hex 字符解码（a.java:10867-10875）：仅 m_151 调用（a.java:10727）；同 m_148"),
    "addSpark": ("b", "修正暴露",
                 "火花登记（a.java:9680-9686）：唯一调用点 paint `switch (gameMode)` case 18（a.java:3003-3031）；"
                 "gameMode 写点无 18（论证同 m_148 的 JAVAP 普查）⇒ 死代码（completeness 迭代 60 mode18 火花 ☠️）。"
                 "v2 的唯一移植证据系漂移锚（combat.rs 注「spawnPopup kind 常量（…9679-9686…）」）"),
    # ---- c 判定存疑（工具方法论），解决方式见 PORT_NOTES / 归属规则 ----
    "m_001": ("c", "基线23",
              "packed 容器解码（a.java:4516-4549）已移植于 game-data packed_png.rs：其 seed 方言锚 "
              "`reference/seed/a.java:2657-2676` = seed `private void a(int n)`（序数对齐 = m_001），v2 按 deobf "
              "行号读成 paint 区间。解决：v2.1 方言规则（无需人工条目）"),
    "readU16BE": ("c", "基线23",
                  "u16 读原语（a.java:6949-6951）已移植于 game-data lib.rs read::u16_le：其 seed 锚引 "
                  "`private static short a(InputStream)`（序数对齐 = readU16BE）。解决：v2.1 方言 + CFR 签名点名规则。"
                  "注：方法体为 LE（低字节先），名称 BE 不符——命名台账疑误（data/naming 不在本任务写入范围）"),
    "<clinit>": ("c", "基线23",
                 "static{}（a.java:10877-10879）：JAVAP static{} 0-51 putstatic f_int_arr_01 = "
                 "{0,8192,16384,24576,8462,270,90,8282}，54-93 另造一个 int[8] 后丢弃（无 putstatic）；VF 将前者"
                 "投影为字段初始化器 a.java:44，v2 不把字段初始化器行归属 <clinit>。Rust："
                 "crates/game-core/src/paint.rs `pub const NOKIA_TRANSFORM_TABLE: [i32; 8] = [0, 8192, 16384, 24576, 8462, 270, 90, 8282];`。"
                 "解决：v2.1 字段初始化器归属规则"),
    "m_059": ("c", "基线23", "CellGrid::new 已移植（a.java:6953-6958；见 PORT_NOTES）：fn 异名且无行锚"),
    "findItemStackIndex": ("c", "基线23", "PlayerCombat::find_item 已移植（a.java:7655-7666；见 PORT_NOTES）：fn 异名 + 行锚陈旧"),
    "addItemToItemStack": ("c", "基线23", "add_item_to_stack 已移植（a.java:7718-7731；见 PORT_NOTES）：fn 异名 + 行锚漂移"),
    "m_103": ("c", "基线23", "两处内联已移植（a.java:8659-8664；见 PORT_NOTES）：无 fn、注释点名无行锚"),
    "m_118": ("c", "基线23", "构造期吸收（a.java:9362-9368；见 PORT_NOTES）"),
    "m_120": ("c", "基线23", "内联于 save_floor_state（a.java:9399-9402；见 PORT_NOTES）"),
    "m_123": ("c", "基线23", "内联于层存档恢复（a.java:9676-9678；见 PORT_NOTES）"),
    "m_008": ("c", "基线23", "I/O 清理无可观测效应（a.java:4634-4661；见 PORT_NOTES absorbed）：v2 无「吸收」口径"),
    "m_006": ("c", "修正暴露", "结果被丢弃的纯函数（a.java:4605-4614；见 PORT_NOTES absorbed）；v2 证据系漂移锚"),
    "releaseSceneImages": ("c", "修正暴露", "内存释放被构造期装载吸收（a.java:4714-4728；见 PORT_NOTES）；v2 证据系漂移锚"),
    "m_068": ("c", "修正暴露", "新游戏复位分散移植（a.java:7188-7216；见 PORT_NOTES）；v2 证据系漂移锚 + seed 误读"),
    "m_072": ("c", "修正暴露", "OptionList::add 已移植（a.java:7266-7283；见 PORT_NOTES）；v2 证据系 28 条 seed 误读"),
    "itemTypeToStackIndex": ("c", "修正暴露", "item_desc_index 已移植（a.java:7702-7716；见 PORT_NOTES）；v2 证据系漂移锚"),
    "deleteRecordStoreQuietly": ("c", "修正暴露", "删后即写被壳覆盖写吸收（a.java:9321-9326；见 PORT_NOTES）；v2 证据系漂移锚"),
    "m_117": ("c", "修正暴露", "RMS 清理无可观测效应（a.java:9328-9360；见 PORT_NOTES）；v2 证据系漂移锚"),
    "m_128": ("c", "修正暴露", "Particles::new 已移植（a.java:9818-9828；见 PORT_NOTES）；v2 证据系漂移锚"),
}

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
    return {"names": names, "cur": cur_rng, "cur_field": cur_field, "robust": robust,
            "field_robust": field_robust, "seed": seed_rng, "seed_field": seed_field,
            "seed_by_sig": seed_by_sig,
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

def collect_tests() -> list[dict]:
    """crates/game-oracle/tests/*.rs 机械解析。帧级用例 = 非 #[ignore] + 读 _out/A-*/trace.txt
    + `assert_eq!(&sha, expect…)`；窗口：
      `assert_eq!(checked, records.len())`          → [1, A-trace 帧数]（full）
      `if rec.tick >= S` + `assert_eq!(checked, C)` → [S, S+C-1]
      `assert_eq!(checked, C)`                      → [1, C]
      `assert!(checked >= C)`                       → [1, C]（保守下界）"""
    tests = []
    for f in sorted(TESTS_DIR.glob("*.rs")):
        text = f.read_text(encoding="utf-8")
        # 属性块允许夹 /// 文档注释（preset_floor35/10/46/42/43 的 #[test] 与 fn 间有文档）；
        # 只认以 `#[` 起首的行为属性（文档正文里出现 "#[ignore]" 字样不算）
        heads = list(re.finditer(r"((?:^[ \t]*(?:#\[[^\n]*\]|//[^\n]*)[ \t]*\n)+)[ \t]*fn (\w+)\(",
                                 text, re.M))
        for i, m in enumerate(heads):
            attrs = [l.strip() for l in m.group(1).splitlines() if l.strip().startswith("#[")]
            name = m.group(2)
            if "#[test]" not in attrs:
                continue
            body = text[m.end():heads[i + 1].start() if i + 1 < len(heads) else len(text)]
            t = {"file": f.name, "fn": name, "line": text[:m.start(2)].count("\n") + 1,
                 "ignored": any(x.startswith("#[ignore") for x in attrs),
                 "trace_stem": None, "kind": None, "window": None}
            tr = re.search(r"_out/A-([\w-]+)/trace\.txt", body)
            if tr:
                t["trace_stem"] = tr.group(1)
            frame = bool(re.search(r"assert_eq!\(\s*&sha,\s*expect", body))
            if t["ignored"]:
                t["kind"] = "ignored"
            elif tr and frame:
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
                else:
                    t["kind"] = "frame-unbounded"  # 帧断言但无 checked 计数：不计入验证
            elif tr:
                t["kind"] = "trace-partial"   # 读 A-trace 但非逐帧（ops 前缀/锚点）
            elif "include_str!(\"fixtures/" in body:
                t["kind"] = "fixture"
            else:
                t["kind"] = "structural"
            tests.append(t)
    return tests


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
                    for n, (s, e) in jm["cur"].items():
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
                msg = re.search(r"(?:todo|unimplemented)!\(\s*\"([^\"]*)", line)
                todos.append({"at": loc, "msg": msg.group(1) if msg else line.strip()})
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
    rng = jm["cur"]
    names = [m["name"] for m in inv]
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
    # 「修正暴露」= v2 口径有移植账、v2.1 机械信号（fn/行锚）全空者——须与 TRIAGE 登记一致
    exposed = {r["name"] for r in rows if v2c[r["name"]] != "white"
               and not (r["port"]["fn"] or r["port"]["refs"])}
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
    base_now = Counter(color_of[n] for n, (c, o, _) in TRIAGE.items() if o == "基线23" and c == "c")

    L: list[str] = []
    a = L.append
    a("# 覆盖矩阵（层次一账本，机器生成）")
    a("")
    a("> 生成：`python3 tools/cover_matrix.py`（v2.1，2026-10-15 刷账：v2 重写 + v2.1 行锚归属治理；v1 = 迭代 72）。")
    a("> 触达数据：`python3 tools/method_reach.py` → `tools/method_reach/snapshot.json`。")
    a("> **请勿手改本文件**——改判定请改 tools/cover_matrix.py 的 PORT_NOTES/TRIAGE（逐条带证据，脚本校验锚点）。")
    a("> 判据文本见 [docs/spec/completeness.md](spec/completeness.md)「覆盖矩阵刷账（2026-10-15）」。")
    a("")
    a(f"## 汇总：绿 {cnt['green']} / 黄 {cnt['yellow']} / 白 {cnt['white']}（共 {len(rows)} 方法）")
    a("")
    a(f"- 白项分账（现白 {cnt['white']}）：{CLS['a']} {wcls['a']} / {CLS['b']} {wcls['b']}"
      + (f" / c 未解决 {wcls['c']}" if wcls["c"] else "")
      + (f" / **未分账 {wcls['none']}**" if wcls["none"] else " / 未分账 0"))
    a(f"- 刷账三分账（§4.1）：基线 23 白 = a {tri[('基线23', 'a')]} / b {tri[('基线23', 'b')]} / "
      f"c {tri[('基线23', 'c')]}（c 已解决 → 现绿 {base_now['green']} / 黄 {base_now['yellow']}）；"
      f"v2.1 归属治理新暴露 {sum(v for (o, _), v in tri.items() if o == '修正暴露')} 项 = "
      f"b {tri[('修正暴露', 'b')]} / c {tri[('修正暴露', 'c')]}（c 经核验恢复移植账）")
    yr = sum(1 for r in by["yellow"] if r["hits"])
    a(f"- 黄项：{cnt['yellow']}（其中 {yr} 项被无对拍用例的场景触达 = 补用例即可转绿；"
      f"{cnt['yellow'] - yr} 项无任何场景触达，待不触达论证——convergence-plan §0.1）")
    a(f"- 口径复算（同一份 Rust 树）：v2 前任口径 绿 {v2cnt['green']} / 黄 {v2cnt['yellow']} / 白 {v2cnt['white']}"
      f"（差异见 §7）；v1 旧账（伪名口径，{len(leg_names)} 名）绿 {leg_cnt['green']} / 黄 {leg_cnt['yellow']} / "
      f"白 {leg_cnt['white']}（对照见 §6）")
    if missing_rng:
        a(f"- ⚠ deobf 区间缺失：{', '.join(missing_rng)}")
    for w in jm["warn"]:
        a(f"- ⚠ {w}")
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
    a("## 1. 对拍用例 → 场景 → 窗口（crates/game-oracle/tests 机械解析）")
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
        a(f"| `{tid}` | {t['trace_stem']} | T{w[0]}-{w[1]} | {t['status']} | {vcount.get(tid, 0)} |")
    a("")
    others = [t for t in tests if t["kind"] != "frame"]
    if others:
        a("非帧级用例（不计入验证；列出以示归属）：")
        a("")
        for t in others:
            a(f"- `{Path(t['file']).stem}::{t['fn']}`（{t['file']}:{t['line']}）：{t['kind']}"
              + (f"，读 A-{t['trace_stem']}" if t["trace_stem"] else ""))
        a("")
    tested = {t["trace_stem"] for t in tests if t["kind"] == "frame"}
    a("场景触达（JDI 实测，全程 = 各相位并集取最小 t_hit；相位起点 = 1 ∪ 用例窗口起点）：")
    a("")
    a("| 场景 | 帧数 | 结束 | 相位 | 触达方法数 | A-trace 对照 | 帧级用例 |")
    a("|---|---|---|---|---|---|---|")
    for stem, s in snap["scenarios"].items():
        at = s.get("a_trace")
        ph = ",".join(sorted(s.get("phases", {}), key=int))
        a(f"| {stem} | {s.get('frames')} | {s.get('end', '').replace('END ', '')} | {ph} | "
          f"{len(full_run_hits(s))} | {at['match'] if at else '无 A-trace'} | "
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
    a("| 方法 | 区间 | 场景触达（无用例） | 移植证据 | 调用点（deobf） |")
    a("|---|---|---|---|---|")
    for r in sorted(by["yellow"], key=lambda r: r["range"] or (0, 0)):
        h = "; ".join(f"{s}@T{t}" for s, t in r["hits"][:3]) or "**无场景触达**"
        cs = ", ".join(f"{c}:{ln}" for ln, c in r["callers_src"][:4])
        a(f"| {r['name']} | {anchor(r)} | {h} | {port_str(r)} | {cs} |")
    a("")

    # §4 白
    a(f"## 4. 白：无移植账（{cnt['white']}）")
    a("")
    a("| 方法 | 区间 | 分账 | 场景触达 | Rust 点名提及 | 论证 / 依据 |")
    a("|---|---|---|---|---|---|")
    for r in sorted(by["white"], key=lambda r: r["range"] or (0, 0)):
        tr = r["triage"]
        cls = CLS.get(tr[0], tr[0]) if tr else "**未分账**"
        h = "; ".join(f"{s}@T{t}" for s, t in r["hits"][:3]) or "无"
        mn = f"{len(r['mentions'])}（{r['mentions'][0]}）" if r["mentions"] else "0"
        a(f"| {r['name']} | {anchor(r)} | {cls} | {h} | {mn} | {tr[2] if tr else '—'} |")
    a("")
    a("### 4.1 刷账三分账（2026-10-15：基线 23 白 + v2.1 归属治理新暴露）")
    a("")
    a("c 类 = 白色系工具方法论所致（移植存在但漏判 / 无可观测效应无「吸收」口径）；解决手段为"
      "归属规则修正或 PORT_NOTES 人工条目（逐条校验 Rust 锚），解决后现色随之变化。")
    a("")
    a("| 方法 | 区间 | 来源 | 分账 | 现色 | 场景触达 | 论证 / 依据 |")
    a("|---|---|---|---|---|---|---|")
    order = {"a": 0, "b": 1, "c": 2}
    by_name = {r["name"]: r for r in rows}
    for n, (c, o, ev) in sorted(TRIAGE.items(), key=lambda kv: (kv[1][1] != "基线23", order[kv[1][0]],
                                                                 (by_name[kv[0]]["range"] or (0, 0)))):
        r = by_name[n]
        h = "; ".join(f"{s}@T{t}" for s, t in r["hits"][:2]) or "无"
        a(f"| {n} | {anchor(r)} | {o} | {CLS[c]} | {SYM[r['color']]} | {h} | {ev} |")
    a("")

    # §5 todo
    a("## 5. 臂级缺口：Rust `todo!` 显式挡路点（方法级绿/黄内的未移植分支）")
    a("")
    for t in todos:
        a(f"- {t['at']}：{t['msg']}")
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
                  "bad": bad_notes}


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
          f"白分账 a {w.get('a', 0)} / b {w.get('b', 0)} / c {w.get('c', 0)} / 未分账 {w.get('none', 0)}；"
          f"v2 口径复算 {info['v2'].get('green', 0)}/{info['v2'].get('yellow', 0)}/{info['v2'].get('white', 0)}；"
          f"v1 复算 {info['legacy'].get('green', 0)}/{info['legacy'].get('yellow', 0)}/{info['legacy'].get('white', 0)}"
          + (f"；⚠ {len(info['bad'])} 条校验告警" if info["bad"] else "") + "）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
