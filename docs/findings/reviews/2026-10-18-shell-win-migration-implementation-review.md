# 对抗审查：shell-port 四缝迁移 + shell/common 抽取（task-3/task-5 对账件）

> 审查人：adversary。对象：fdba990（①Storage）/ eabb815（②AudioOut）/ 28e4916（③
> ProcessControl）/ 5f541f0（④AssetSource），9f63a63 行为锚已过拆分终验（git-audit G5）。
> 手段：断言保全逐字比对（UTF-8 python）、四 impl grep、抽取 5 判据核、RED 符号缺席
> 证明、GREEN 独立复跑、三对账逐 commit。

## 判定：**通过**（断言零改动 145/145 逐字保全、GREEN 42/42、抽取 5 判据全中）；2 条备忘

## 1. 断言改写 diff（硬条件②）—— ✅ 0 行，且豁免未动用

- **逐字保全证明**：9f63a63 的 38 锚全部 assert 行（去空白后 145 条唯一）与当前树
  （shell/win + shell/common）做 UTF-8 全文包含比对 ⇒ **missing = 0**。
  "断言文本与期望值改写 = 0 行"成立（字节级）。
- **硬条件①未动用豁免**：键序断言保持**字面量**——shell/common/src/lib.rs:67-68
  期望 `["MOT_L0@1", …, "SKY_WAR@1"]` 字面数组（非 RMS_STORES 驱动）；
  game-platform 自 efb29bb 起**零改动**（`git diff efb29bb HEAD -- crates/game-platform`
  空），storage_contract.rs:54 字面量锚原样 ✓。
- 【方法更正】本项首测我用 PowerShell 比对得"76 缺失"——**我方 GBK 控制台编码伪差**
  （中文串被毁），换 UTF-8 python 后 0 缺失。与 task-7 复审的 temp 伪差同记：我的
  取证脚本涉中文必须走 python utf-8。

## 2. 计数迁移总账 —— ✅ 精确闭合

38 原锚 = 12 随迁（5 storage + 7 audio）+ 26 留 win；终态 game-desktop 29
（26 + 78ac8cf 并发 1 + B3/B4 新 2）+ game-shell-common 13（12 随迁 + B① 新 1）。
**原 38 条零丢失**（§1 包含比对硬证）；其"38→33→26→28→29 / 0→6→13"账与我实测
（29/13，cargo test 清点）逐项吻合。

## 3. RED 抽查 —— ✅（符号缺席证明 + 逐字输出族吻合）

`git grep` 亲验父树符号缺席：`struct FileRms` ∉ fdba990^、`fn consume_exit` ∉
28e4916^、`struct DirAssets` ∉ 5f541f0^ ⇒ 其 E0432/E0599 编译红族输出必然成立
（方法限界注明：本单元按符号缺席+message 逐字证明，全量 temp-archive 复现已在
485f542/b587d11 两单元执行、可复现性已建立）。78ac8cf 回放法红前轮已核。

## 4. 抽取 5 判据 —— ✅ 全中

| # | 判据 | 实况 |
|---|---|---|
| 1 | 共享音频面纯字节零 std::fs | shell/common/src/audio.rs API 面零 std::fs（仅 #[cfg(test)] fixture 读 ogg + 文档提及；模块 doc 明引本判据）；track_path/REGEN_CMD 留 win ✓ |
| 2 | BgmController/BgmSnapshot/level_to_gain 升 common | audio.rs:35/:70/:65 ✓ |
| 3 | 键模型走契约、无第二套白名单 | rms.rs:16 use 契约三件 + :5 注明"不自带第二套白名单" ✓ |
| 4 | 测试随迁断言零字节 + MOT_IF 字段级对偶落 common | §1 保全 ✓；`serialize_slot_if_round_trips_every_field`（rms.rs:102-127 全字段回读镜像 load_slot_table 读序）✓ |
| 5 | glob/lewton | shell/common/Cargo.toml:13 `lewton = "=0.10.2"` ✓；members glob 自动纳入 ✓ |

## 5. 四 impl grep + exit 计数 —— ✅

`impl Storage for FileRms`（common/rms.rs:62）、`impl AudioOut for WinMmPcm/AlsaPcm`
（win/audio.rs:141/308）、`impl ProcessControl for WinitExit`（main.rs:992）、
`impl AssetSource for DirAssets`（main.rs:74）+ 测试替身 MockDev/FailingDev/Probe ✓；
dyn 消费 6 处（load_container/load_assets/Box Storage×2/consume_exit/Box AudioSink）✓。
exit 链：`el.exit()` 生产面 **1 处**（WinitExit::request_exit :994，2→1 达成）；
consume_exit（:652-653）= `if self.finished()` + request_exit，与现状逐字等价、
**无消费位**（(a) 口径）；B3 MockProcess 锁"finished 前零请求/置位后每轮恰一次"✓；
注释还回引了我 F4 的口径更正（:648"循环先退的巧合"）——好。

## 6. 三对账（逐 commit）—— ✅

fdba990（common 3 文件 + win 2）、eabb815（common 2 + win 2 + 夹具物理随迁）、
28e4916（main.rs 1）、5f541f0（main.rs 1）——内容/声称/写范围（shell/win +
shell/common，F-4 裁决扩域内）全对，全 pathspec 无卷入；并发 78ac8cf 落于 2-3 间
与其声明一致（git 链亲验）。panic 三类文案/资产 fnv 摘要锚随断言保全隐含 ✓
（"缺 assets/raw/{name}" quirk 锚含其中）。

## 7. GREEN 独立复现

`cargo test -p game-shell-common -p game-desktop`（HEAD）⇒ **13/13 + 29/29 全绿**，
EXIT=0。

## 8. 备忘（不卡闸）

1. test-ratchet 基线 ratchet-update → task-6（约定不变）。
2. RED 窗口连坐：共享总闸跑全 workspace ⇒ 任一成员红期全员拒提交（其三次连坐重试
   留痕）——团队流程观察项，若复发建议 knowledge 记"并行 TDD 红期协调"条。
