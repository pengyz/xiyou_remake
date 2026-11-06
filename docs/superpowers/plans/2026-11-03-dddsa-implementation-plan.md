# DDDSA 实施计划（2026-11-03）——每提交对抗复核制

> 用户指令：实现 spec（docs/superpowers/specs/2026-11-03-dddsa-architecture-design.md **v3 终稿**）；
> **每次提交前必须：独立子 agent 对抗复核 + 测试通过 + 推送**（逐提交纪律，非逐批）。

## 0. 逐提交工作流（本计划核心纪律）

```
成员实现 commit（gates 13/13 本地绿）→ 报 lead
  → lead 发**独立对抗复核件**（子 agent 攻 diff：伪数/锚/语义/红线）
  → 红项回派修复（复验）｜绿=裁定
  → 测试全量通过（--check 金样 60+归属+gates）
  → **推送 origin**
```
- 复核件独立性：每 commit 一个新子 agent（不复用），骨架先行防中断；
- 复核红=该 commit 不推送（回派修复后重新复核）；
- 终稿 spec 的 §8 抽验表=复核件的核对清单来源。

## 1. P1 物品注册表化（4-7d，spec §2/§5）

### 前置件（B 裁定六组+臂面——先行）
| # | 内容 | 判据 |
|---|---|---|
| P1-a1 | **dump 补面 6 组**（道具栈 item_types/uses/stack_size、itemUseTargetType、tutorialSeen、scriptLineFlags、overlayText）+A2 重冻结 | state_baseline --check 全同（新面）+statediff 名集相等 |
| P1-a2 | **21 臂组夹具全量**（禁抽样，净补≈11 条+型 32 销账） | 每臂 Java 帧锚夹具绿 |

### 主体（每件一 commit 一复核一推送）
| # | 内容 | 判据 |
|---|---|---|
| P1-1 | items.json schema+强类型 Params 定义+加载期校验（五类拒载复用） | 拒载负例+schema 生成单源 |
| P1-2 | ItemEffect trait+内建效果注册（Rust 67 行 match 逐字抽象，a.java:7534-7649 锚） | 逐臂等价三层（锚+夹具+金样） |
| P1-3 | pickup/use dispatch 化（match 删除与表行**同 commit 一次性切**——禁双轨） | 金样 60+新 dump 面全同 |
| P1-4 | 魔法区间表化（13..=25/26..40/79..85 全转行）+保真表哈希闸扩 FIDELITY_TABLES | 哈希闸+勘误流程注记 |
| P1-5 | C1 升级演练：新物品=表加行零改码（含未知 effect 加载期拒载+运行期 no-op 测试） | zero-code-drill 断言 |
| P1-6 | EnemyDef 聚合：敌人 SoA→per-type 记录（用户数据模型总则；lead 裁定排 P1-5 后） | EnemyDef 表+对账 |

## 2. P2 子系统 trait 化（8-15d，spec §3）——P1 后
八 trait 面+Ctx 三宪+构造面收敛+item/effect 编排收敛——每件同纪律。

## 3. P3 编辑器 MVP（5-8d items 单表，spec §4）——P2 后
server.py+schema 单源生成+四段机器化演练。

## 4. 估算与验证预算
每 commit=复核件（~10-20min）+测试（单测 13s/步 4min/里程碑 18min）+推送；
P1 合计 4-7d（含前置件 +1-2d）。
