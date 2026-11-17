# 快审·差分验证复核 — P1-a2 批 5-a 补正（commit 5edb8a2）

> 角色：独立对抗复核员（快审·差分验证员纪律，AGENTS.md 权威）。
> 复核对象：commit `5edb8a2`（批 5-a 补正——红 1 修复：method_reach 快照终态+三修+边界裁定落书，未推送）。
> 一轮复核：docs/findings/reviews/2026-11-04-p1a2-batch5a-review.md（红 1=--check exit=1 与贴件矛盾，过渡态贴件）。
> 复核时点：2026-11-04 09:46– +08:00。工作目录 /home/peng/workspace/xiyou_remake。

## 0. 复核范围与方法

- 对象=commit `5edb8a2`（未推送，HEAD）；前态=`cdbc7c1`（批5-a 原笔，一轮红 1 黄 3）。
  sha 贴原文：`5edb8a2cc8f08e29208bb406b08200591bd2cd2e`（`git log -1 --format=%H` 实测）。
- 方法=**独立重算**（不采信 commit 自贴）：工作树=提交树先证（`git diff 5edb8a2 --stat -- tools/ crates/ reference/
  docs/findings/2026-11-04-port-notes-anchor-debt.md` = 空；`git ls-files --others` 夹具/场景目录 = 空）
  ⇒ 本复核全部实跑即提交树实跑；`--check` 实跑取 exit 码；贴件段逐字对拍实文；snapshot diff 行型机械数
  （`grep '^-'`/`grep '^+'`）；本体零动=非注释行 sha 对拍（`git show cdbc7c1:…` vs `5edb8a2:…`）；
  落书面=`grep -rn` 全库实测（排除复核件自身）。判据均 A 级实测。
- 三纪律自查：sha 贴原文 ✓（上）；引锚实文 ✓（method_reach.py:75-84/315-335、fixtures 头 L1-L4、
  port-notes L38-56 均 sed 实读）；实证段禁手粘 ✓（本篇证据全部命令重跑产出）。

## 1. 攻击点 1 — 终态实跑：--check exit=0？贴件=提交树同树实证？🟢

**终态实跑（干净树=提交树，§0 已证同树）：**

```
$ python3 tools/method_reach.py --check
[method_reach] 快照一致
exit=0
```

exit=0 实跑 ✓——红 1（前态 exit=1「STALE fx-coin25a: 快照缺失」）确销。机理核
（tools/method_reach.py:315-335）：`--check` 校验集=SCEN ∪ tracked fixtures，fx-coin25a.txt
已入库 ⇒ 进校验集；快照现含 fx-coin25a 条目 ⇒ 通过。且 fx-item25a（provision stem）亦有条目
（`scenarios` 总数 86=85+1，`fx-item25a`/`fx-coin25a` 双在）⇒ 双家两 stem 均不 STALE。

**贴件三段逐字对拍（paste 与 commit 同树实证——终态纪律首例验收）：**

| 贴件段 | commit 贴 | 提交树实文 | 判 |
|---|---|---|---|
| ① `--check` | `[method_reach] 快照一致`+`exit=0` | 实跑同 | ✅ 逐字同 |
| ② preset 头 | `# preset: fx-coin25a.preset` | fixtures L2 实文同 | ✅ 逐字同 |
| ③ L4 再生锚 | `…A-fx-item25a/trace.txt --stem fx-coin25a…` | fixtures L4 实文同 | ✅ 逐字同 |

**终态纪律首例验收：通过**——贴件三段均取自提交树终态（本复核于同树实跑复现逐字同），
与前态"过渡态贴件病"（paste-time 真/commit-time 假）形成正反对照。paste=commit 同树实证成立。

## 2. 攻击点 2 — 三修核销：L4 锚实存 / preset 头正名 / 补测纯追加 🟢

| 修 | 核销内容 | 实测 | 判 |
|---|---|---|---|
| 修1 L4 锚→实存 | `reference/oracle/_out/A-fx-item25a/trace.txt` 存在（8,124,995B）；旧锚 `A-fx-coin25a/trace.txt` ls 报不存在（改名误伤实证） | ✅ | 🟢 |
| 修2 preset 头正名 | fixtures 头=`fx-coin25a.preset` 且该文件实存（759B）；旧名 `preset-item25a.preset` **测试侧不存在**；根因链实文核：`scenario_header`（method_reach.py:75-84）preset 路径=`f.parent / 头值` ⇒ 旧头解析到 fixtures/preset-item25a.preset（不存在）⇒ FileNotFoundError ✓ 机理吻合 | ✅ | 🟢 |
| 修3 补测纯追加 | snapshot.json diff 行型机械数：删除行=`grep '^-'`=1（仅 `--- a/…` 头）⇒ **零删**；追加 103 行（`+++` 头外）全为 fx-coin25a 新条目 ⇒ **纯追加** ✓；条目内 sha 对拍：`script_sha256`/`preset_sha256` 与实文件 sha256 全同、`preset`=`fx-coin25a.preset`、ticks 900/900、exit 0、frames 900 | ✅ | 🟢 |

**本体零动（fx-coin25a 夹具语义）**：`fx-coin25a.txt`/`fx-coin25a.frames.txt` 非注释行
（脚本体/帧数据）sha 对拍 cdbc7c1 vs 5edb8a2 = **全同**；diff 全量仅 2 行（frames L4 锚文 +
txt L2 preset 头）——语义零动 ✓。preset 双家字节同：fixtures fx-coin25a.preset ≡
scenarios preset-item25a.preset（sha256 全同）✓。

措辞小疵 2 处（非造假）：①commit 引文「"# preset: preset-item25a"」较实文旧值
`# preset: preset-item25a.preset` 少 `.preset` 后缀；②「旧异体名（从未存在）」字面过宽——
provision 侧 `scenarios/preset-item25a.preset` 实存（双家注记自家都写了），准确说法="测试侧
fixtures 目录下从未存在（解析落点）"。

## 3. 攻击点 3 — 黄②③：双家注记进 repo？grep -F 确切命令入档？🟢

- **黄②双家注记**：`grep -rn "双家" docs/ data/ tools/`（排除复核件）⇒
  `docs/findings/2026-11-04-port-notes-anchor-debt.md:38`「## fx-item25a/fx-coin25a 双家归属注记
  （批5-a 黄②·2026-11-04）」+ 双家路径表 + L4 锚改向说明——**进 repo 实证** ✓（前态
  「grep 空」缺口闭合）。表内 provision 侧 stem 注记「f-xitem25a 旧名保留——不改」与
  scenarios 实存面对上。
- **黄③ grep -F 确切命令入档**：同文件 L48-57 入档两命令
  `grep -F "fx_pickup_26" docs/coverage-matrix.md` / `grep -F "fx_pickup_33" docs/coverage-matrix.md`
  ✓；**复现验证**：两命令实跑输出 26 组 5 行+33 组 2 行=7 行，与批5-a 贴件 fx-pickup 行两段
  逐行对上（一轮复核 §4.1 的 7 行拼接形复现）——可复现性要件 (a) 闭合 ✓。
  附注：入档文中「26 组 5 行+33 组 2 行」计数与实跑一致 ✓。

## 4. 攻击点 4 — 边界裁定落书：三则+终态纪律惯例注记在 repo？🟡（部分落书）

`grep -rn` 全库（排除复核件）实测落书面：

| 条文 | repo 文档落点 | 判 |
|---|---|---|
| 三则**内容**（命令入档+不改字节+不隐匿不利输出） | port-notes L51 括号引述（随黄③入档文） | ✅ 简写形式在 |
| 三则**完整条文**（"命令侧整行选择/排序=合规加工"定义 + "手编辑/sed 改写行内容=违规"红线） | **零命中**（`grep "合规加工"` 仅 L51 引述、`grep "贴件边界"` = 0） | ❌ 仅存 commit message |
| **终态纪律**（"贴件=提交树终态（过渡态贴件=病）"） | **零命中**（`grep "提交树终态\|过渡态贴件"` = 0） | ❌ 仅存 commit message |

**判黄（轻）**：commit 自称「钦定惯例注记（随批落书）」——落书**部分兑现**：三则以简写引述
进了 port-notes，但完整裁定条文与终态纪律条文未入任何 docs 文件。按 AGENTS.md §7
（产出归宿=台账/spec/代码/知识库，非聊天记录）+ §8（惯例/知识类归 knowledge），且**本批自身
尺度**（黄③ 的"入档"=docs 文件入档，非 commit message）——两把尺子应一致：惯例注记要成为
"惯例"须落 `docs/knowledge/` 或 findings 可 grep 处。commit message 在 git 史可查，但非惯例
归宿。补法：裁定条文+终态纪律一句入 docs/knowledge（decision 类）或 port-notes 独立节，即闭合。
（若 lead 裁定 commit message 即落书载体，可升绿；快审从严先记黄。）

## 5. 攻击点 5 — 金样 70 全同+夹带净+本体零动 🟢

- **金样 70 全同**：实跑 `python3 tools/state_baseline.py --check` ⇒
  `[GREEN] 金样逐字节等价：70 场景 sha 全同`，exit=0（release 全跑 fixture_frames/
  gameplay_floor1/preset_cases 后比对）✓；`git show 5edb8a2 --stat | grep golden` = 0
  （本笔零触碰 golden）✓。
- **夹带净**：`git show 5edb8a2 --name-status` = 4 文件全在事内（fx-coin25a 两夹具头行+
  port-notes 注记+snapshot 补测）；未追踪面（排除本复核件）仅 `android/build/`（预存构建
  产物目录 reports/，非夹具/场景区，非本批引入）⇒ 零夹带 ✓。
- **本体零动**：fx-coin25a.txt/.frames.txt 非注释行 sha 对拍 cdbc7c1 vs 5edb8a2 全同（§2）；
  preset 双家字节同（759B sha 全同）——fx-coin25a 夹具语义零动 ✓。
- **批 1-4 未动坏**（实跑）：`cargo test -p game-oracle --test fixture_frames` ⇒
  **41 passed / 0 failed** ✓；`cargo test -p game-core --test mov_route_scene` ⇒
  **5 passed / 0 failed**（含 `fx_coin25a_lucky_coin_doubles_gold_frames_match` ✓）。

## 6. 总判

| # | 攻击点 | 判定 | 一句话 |
|---|---|---|---|
| 1 | 终态实跑+同树实证 | 🟢 | 干净树 --check exit=0；贴件三段与提交树实文逐字同——终态纪律首例验收**通过** |
| 2 | 三修核销 | 🟢 | L4 锚实存/旧锚不存在、preset 正名+FileNotFoundError 根因链实文核、snapshot 纯追加零删（附 2 处措辞小疵） |
| 3 | 黄②③ | 🟢 | 双家注记 grep 实证进 repo；grep -F 两命令入档且实跑复现 7 行贴件 |
| 4 | 边界裁定落书 | 🟡 | **部分落书**：三则简写引述在 port-notes L51，但完整条文+终态纪律 docs 零命中（仅存 commit message） |
| 5 | 金样/夹带/本体 | 🟢 | 金样 70 sha 全同 exit=0；夹带净；本体（非注释行）零动；批 1-4 测试 46/0 全绿 |

**红 0 / 黄 1 / 绿 4。总判：需修复（轻）**——核心红 1（--check 终态矛盾）已实证销账，
贴件=提交树终态纪律首例验收通过；唯黄④"随批落书"部分兑现：完整裁定条文（合规加工定义+
违规红线）与终态纪律一句（"贴件=提交树终态（过渡态贴件=病）"）在 docs 零命中（`grep -rn
"合规加工\|贴件边界\|提交树终态\|过渡态贴件"` 实测），仅存 commit message——按一轮复核
§1.3 同尺（"仅存 commit message ⇒ repo 内无注记"判黄）及 AGENTS.md §7 产出归宿纪律，
commit message 非落书载体；且本批自身尺度（黄③"入档"=docs 文件）两把尺子应一致。
**补法（3 行级）**：裁定条文+终态纪律一句入 docs/knowledge（decision 类）或 port-notes
独立节 → grep 闭合即绿，可推送。措辞小疵 2 处（§2）随下批注记即可，不阻断。

## 7. 三纪律自查

- sha 贴原文 ✓（§0）。引锚实文 ✓（method_reach.py:75-84/315-335、fixtures 头 L1-L4、
  port-notes L38-56、snapshot fx-coin25a 条目均 sed/python 实读）。实证段禁手粘 ✓
  （本篇全部证据命令重跑产出，含 --check 两次、state_baseline、cargo test ×2、
  grep ×5、sha 对拍脚本）。
