# BUG-009 更正/补强稿（task-10 移交 lead 入库 docs/bug-ledger.md；本文件可删）

> 提交者：items-evidence（task-10 FX-BUG-AIOOBE 夹具取证）。按 AGENTS.md §3.6
> 「证伪更正文化」：BUG-009 原行不删，**原地加日期更正注** + 更新"复现/根因"列。

## 建议的原地更正注（贴在 BUG-009 行下方或行内）

【2026-10-16 更正/补强】根因已定位（A 级）：**applyStepCellEffects 型60/62 四邻扫描
「先调 findBlockingEntity 后判边界」**（a.java:5596-5616 型60 块、5575-5590 型62 块
x-1 调用同病）——四个方向的越界均可达：
- y=0/x=0：`findBlockingEntity(…-1…)` → `cellTypeGrid[y][-1]` ⇒ **Index -1**（= BUG-009
  原现象，preset-alchemy 初版 floor4 (5,0) 复现）；
- y=12/x=12：`findBlockingEntity(…+1…)` → `cellTypeGrid[13][…]`/`[…][13]` ⇒ **Index 13**
  （= 本次新复现，floor5 (9,12) 底边界格）。
崩溃点逐字（C 变体可读栈，A/B/C 三变体同 tick 同点同因）：
`findBlockingEntity(a.java:8616 `byte var7 = this.cellSlotCount[var6];` ← 上一行
`cellTypeGrid[var2][var1]` 即越界读) ← applyStepCellEffects(a.java:5617) ←
dispatchModeEntry(a.java:4320 `case 3: this.applyStepCellEffects();`) ← run`。
触发链：mode3 进场即执行（dispatchModeEntry case 3）⇒ **T236 载入完成首 tick 崩溃**、
主线程退出（trace 尾 `EVT thread-exit / END reason=all-threads-exited ticks=236`）。
对照：型61 块（a.java:5557-5565）与型62 块（5575-5590 除 x-1 外）**先判后调**——
同方法内三种写法并存，型60 块是唯一全四向先调后判者（x-1 一处同病在型62）。
Rust 侧差异：`entity::cell_entity` 越界返 -1（crates/game-core/src/entity.rs）⇒ **不崩**、
继续 tick——Java 可观测前缀（T1-236 帧）与 Rust 对拍一致，T237 起 Java 已死无参照帧。
Stage A：**保留原行为**（崩溃即原版语义，D4）；Stage B 修复 = 把四向调用挪到边界判定之后
（对齐型61/62 既有写法），或 cellTypeGrid 访问钳制。

## 复现（A 级，一键）

- 夹具：`crates/game-oracle/tests/fixtures/fx-bug-aioobe-border.{txt,preset,frames.txt}`
  （task-10 新增）；测试 `fx_bug_aioobe_border_frames_match`（crates/game-oracle/tests/fixture_frames.rs）
  钉死 T1-236 可观测前缀 + T236 未交场景（交场景拍 T237 = Java dispatch 崩溃拍）。
- 复现命令（无头）：
  `python3 tools/frames_fixture.py gen crates/game-oracle/tests/fixtures/fx-bug-aioobe-border.txt`
  ——A/B/C 三方 trace sha256 **含崩溃尾逐字节一致**（三方同点崩溃本身即忠实性证据）。
- 预置关键：`python3 tools/frames_fixture.py preset floor=5 x=9 y=12 facing=0 hp=500 atk=200 def=30 sky=5`
  （玩家钉 y=12 = mapCellsHigh-1；floor4 (5,0) 为 y=0 双子例）。

## 可达性（A 级，升级原行"疑似不可达"）

全 55 层 maplv 边界格（x=0/12 或 y=0/12）逐格 tile_walkability 复算：**无一可走**——
正常游玩不可达（玩家无法站上边界格），仅 preset/存档直钉出生可复现。复算命令（逐字可跑）：

```bash
python3 - <<'EOF'
import struct
walk = [int(l) for l in open('crates/game-core/tests/fixtures/tile_walkability.txt') if l.strip().lstrip('-').isdigit()]
for i in range(55):
    d = open(f'assets/raw/maplv{i}','rb').read()
    w = struct.unpack_from('<H',d,0)[0] >> 1; h = struct.unpack_from('<H',d,2)[0] >> 1
    terr = d[4:4+w*h*4]
    bb = [(x,y) for y in range(h) for x in range(w) if (x in (0,w-1) or y in (0,h-1))
          and terr[y*w*4+x*2] < len(walk) and walk[terr[y*w*4+x*2]] == 1]
    if bb: print(i, bb)
EOF
```
（输出为空 ⇒ 边界格全不可走。）

## 影响

无正常流程影响（原判保持）；测试装置触发。新增：FX-INT-6A 等夹具的出生点设计须避开
边界格（见 fx-int6a.txt cover 注记）。
