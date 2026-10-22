# tools/zero-code-drill — 零改码链演练实跑（survey §3 步 5=验收闸）

> 命题（survey §3 v2 降格口径）：**同图集复用、消费替换完成后域内零改码**。
> 加一个假想新敌人 = 纯表+资源行（sprite_defs 加行 + anim_profile 复用 + 资源行复用
> 保真图集容器/槽位），断言 **零 Rust 改动**（`git status --porcelain crates/` 为空）。

## 两份制下的加行位置（§2.2）

保真三表被内容哈希闸冻结（gate `table-content-hash`，变更=勘误流程+金样重过——
非荣誉制）⇒ 新内容进 **mod 通道**（[`../mod-skeleton/`](../mod-skeleton/README.md)：
型号 id ≥ 1000 命名空间），保真表一字不动、金样运行禁 mod 表（规则 2）。

## 演练步骤（`drill.py` 内断言，任一失败 ⇒ 退出码 1）

| 步 | 内容 | 断言 |
|---|---|---|
| [0] | 规则 3 护航 | 保真三表 sha256 == `gates/table_content_hash.json` 基线 |
| [0b] | 零改码基线 | `git diff/status -- crates/` 为空 |
| [1] | **正例** | 假想新敌人 mod 行（id 1000；anim_profile 复用 `anim_row_0`；资源行复用 `sptenemy1`/3）加载通过；真解析器保真三表原样可载 |
| [2] | 失败面 A **域外型** | 保真 sprite_defs 直接加行 id=89 ⇒ 真解析器**域溢出拒载**（0..=88，域外三界①）；mod 通道 id=89 ⇒ 命名空间拒载 |
| [3] | 失败面 B **坏外键** | anim_profile 孤儿 ⇒ mod 通道**坏外键拒载**；保真侧同错 ⇒ 真解析器坏外键拒载 |
| [4] | 失败面 C **quirk 型** | frame_h=null（运行时图高，Q-001/Q-003 族）无登记 ⇒ **QuirkPathRequired**（走 Q-00x 路径，§2.1 禁静默留码）；登记 Q-101 四字段+quirk_ref ⇒ 通过 |
| [5] | 规则 2 | 金样运行清单混入 mod 表 ⇒ AssertionError |
| [6] | 规则 4 | mod 独立基线立账/复验；保真哈希基线不含 mod 条目 |
| [7] | 收尾 | 保真三表哈希仍绿 + `crates/` 零改动 = 零 Rust 改动实证 |

## 真解析器对照（防镜像自证）

[2]/[3] 的保真侧拒载以 **真加载层** 为准：`drill.py` 在临时目录搭 harness
（path-dep `crates/game-data`，零依赖），调 `defs::parse_sprite_defs` /
`DefBundle::load`（五类拒载），断言错误信息含「域溢出拒载（…0..=88）」/
「坏外键拒载」。harness 不入仓、演练不触 `crates/`（[0b]/[7] 双查）。

## 域外三界（survey §3，演练的边界证词）

| 界 | 域外后果 | 演练落点 |
|---|---|---|
| ① ANIM_TABLE_IDX 89 索引域外新动画行/新型号 | panic 边=扩表改码 | 失败面 A |
| ② 图集容器 16 常量域外=新容器 | 改码 | loader `BadForeignKey`（container） |
| ③ 战斗表/张数表数组索引扩张 | 需重锚 | loader `FrameIndexOutOfBounds`（槽位超表长） |

## 用法

```bash
python3 tools/zero-code-drill/drill.py     # 全量演练（含真解析器 harness 编译，首次 ~1 分钟）
```

判据：尾行 `== 演练汇总：N/N 通过 —— PASS`，退出码 0。
