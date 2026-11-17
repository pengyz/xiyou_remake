# P2 批9 修复件复核（对抗复核 · 红1 行锚修复）

- 日期：2026-11-08
- 复核对象：`c2dbd9c`（amend 后，替 `5b8baf0`），分支 `modern`
- 前审：`docs/findings/reviews/2026-11-08-p2-batch9-review.md`（唯一红=面10 行锚）
- 总判：**可推送（1 黄，非阻塞）**

## 逐面结论

| 面 | 结论 | 关键证据 |
|---|---|---|
| 1. 13 行锚逐行核 | **绿** | 见 §1：13/13 行 `sed -n` 实文与 `evidence.effect` 语义（价/效/提示）全对 |
| 2. id12 (1,8) 一致性 | **绿** | JSON `a.java:7394-7402` = commit body 两处 cite 同值；实文 7394 `case 8:` / 7395 `yellowKeyCount > 0` / 7396 `--` / 7397 `goldAmount += 100` / 7399 `没有黄钥匙` / 7402 `break` |
| 3. 证伪更正格式 | **绿** | commit body 有 `【2026-11-08 红1 更正：原 7371-7376 等距推错，sed 实核修】`（历史错值留在标注内）+ `【复核红1 修（2026-11-08）】` 段；测试注释 `【2026-11-08 更正】` 在位并复述原失实主张 |
| 4. 断言界注 vs 实际 | **黄** | 注释称「到下一 fn replace_shell_rows 前止」，实测 `find("\n    fn ")` 落点=**2777 `fn camera_at_target`**（`replace_shell_rows` 是 `pub fn` 不匹配），判定面实际**含** replace_shell_rows(2758-2776)。断言仍 0 命中（更严），非弱化 |
| 5. 回归 | **绿** | p2b9 `6 passed; 0 failed`（本机实测）；`python3 gates/cli.py check` → `PASS=13 FAIL=0 SKIP=0`；state-baseline `[GREEN] 金样逐字节等价：74 场景 sha 全同`；`git status --porcelain` 非 `??` 行数=0 |
| 6. 夹带 | **绿** | `git show c2dbd9c --name-only --format=` = **恰 8 文件**；`git diff 5b8baf0 c2dbd9c` = 2 文件 `+18/-15`（净 +3）= 13 行 `evidence.effect` + 测试注释块，**零行为行** |

## §1 面1：13 行锚逐行 sed 实核（`reference/src/deobf/a.java`）

| id | (src,idx) | JSON 锚 | 锚内实文 | 对照 | 判 |
|---|---|---|---|---|---|
| 3 | (0,6) | 7342 | `this.addItemToItemStack(19);` | effect=addItemToItemStack(19) | ✓ |
| 6 | (1,1) | 7349-7350 | `playerAtk += playerAtk*3/100` / `playerDef += …*3/100` | atk/def 各+3% | ✓ |
| 7 | (1,3) | 7355-7360 | `spendGold(50)` → `blueKeyCount++` else `没有足够的金钱` | ¤50 蓝+1 | ✓ |
| 8 | (1,4) | 7363-7368 | `spendGold(50)` → `yellowKeyCount += 5` | ¤50 黄+5 | ✓ |
| 9 | (1,5) | 7371-7376 | `spendGold(1000)` → `yellowKeyCount++` | ¤1000 黄+1 | ✓ |
| 10 | (1,6) | 7379-7384 | `spendGold(800)` → `redKeyCount++` | ¤800 红+1 | ✓ |
| 11 | (1,7) | 7387-7392 | `spendGold(200)` → `blueKeyCount++` | ¤200 蓝+1 | ✓ |
| 12 | (1,8) | 7394-7402 | `case 8:` / `yellowKeyCount>0` / `--` / `goldAmount+=100` / `没有黄钥匙` / `break` | 黄-1→¤+100 键数门 | ✓ |
| 13 | (1,9) | 7404-7410 | `spendGold(1000)` → `yellowKeyCount++` + `blueKeyCount++` | ¤1000 黄+1蓝+1 | ✓ |
| 14 | (1,10) | 7413-7418 | `spendGold(200)` → `yellowKeyCount += 3` | ¤200 黄+3 | ✓ |
| 15 | (1,11) | 7421-7426 | `spendGold(2000)` → `blueKeyCount += 3` | ¤2000 蓝+3 | ✓ |
| 16 | (1,12) | 7429-7434 | `spendGold(1000)` → `playerHp += 2000` | ¤1000 hp+2000 | ✓ |
| 17 | (1,13) | 7437-7442 | `spendGold(4000)` → `pickupItemType(18)` | ¤4000 pickup18 | ✓ |

13/13 全对；`price`/`fail_hint` 与锚内数值/提示字面同值（id12 price=0、hint=没有黄钥匙）。旁证（非本批 13 行）：id1/4/5→7336 `gainGold(1000,playerPixelX/Y)`（7333-7335 `case 1/15/21:` 复用）✓、id2→7335-7340 内 7339 `addItemToItemStack(13)` ✓。

## §4 面4 黄：界注与实际判定面不一致（唯一未过面）

- 代码（`crates/game-core/src/scene_tests/shop_purchase_tests.rs:72-75`）：`scene_src[start..].find("\n    fn ")`
- 实测：`apply_merchant_offer` 起于 `crates/game-core/src/scene.rs:2639`；下一匹配 `"\n    fn "` = `scene.rs:2777 fn camera_at_target`（`scene.rs:2758` 是 `pub fn replace_shell_rows`，**不匹配**）⇒ body=2639..2776，**含** `replace_shell_rows` 全体及其 doc 注释。
- 新注释（同文件 64-65 行）称「到下一 fn replace_shell_rows 前止——超出体的后续 helper 不入判定面」：与实际相反。
- 影响面：断言对 8 字面实测 `count=0`（全 body），测试仍绿；实际判定面**比注释更宽（更严）**，无假绿风险。残留风险=未来若 `replace_shell_rows` 出现 `没有足够的金钱`（`scene_menu.rs:697` 同字面存在先例）会以「双源残留」误报。
- 建议（一行改）：注释改为「到 `fn camera_at_target`(scene.rs:2777) 前止——**含** replace_shell_rows(2758-2776)；体内注释亦须零字面=过扫更严」。属注释更正，不阻塞推送。

## §5 回归实测

```
cargo test -p game-core p2b9 → running 6 tests / test result: ok. 6 passed; 0 failed; 368 filtered out
python3 gates/cli.py check   → PASS=13 FAIL=0 SKIP=0
                              （state-baseline: [GREEN] 金样逐字节等价：74 场景 sha 全同）
git status --porcelain       → 非 ?? 行 = 0（仅未跟踪报告/android/build）
```

附注（非本修复引入的阻塞项）：`scene.rs:4632 mod shop_purchase_tests;` 缺 `#[cfg(test)]`（`gold_popup_tests` 有），非 test 构建产生 1 条 `unused import: super::scene_test_support` 警告；`test-ratchet` PASS 但提示新增二进制 `shop_goods_defs(test): 6` 待 ratchet-update。均不改判。

## §6 夹带实测

- 文件清单（恰 8）：`popup.rs` / `scene.rs` / `scene_tests/gold_popup_tests.rs` / `scene_tests/shop_purchase_tests.rs` / `script_ops.rs` / `game-data/src/defs.rs` / `game-data/tests/shop_goods_defs.rs` / `data/tables/shop_goods.json`
- 修前修后 delta（`git diff 5b8baf0 c2dbd9c`）：仅 2 文件 `+18/-15`（净 +3）= `shop_goods.json` 13 行 `evidence.effect` 置换 + 测试注释 5/2 行；**无任何代码/价格/效果/提示数据行混入**。

## 总判

**可推送**（面1/2/3/5/6 全绿，面4=黄·注释与实际判定面不符但断言更严、无假绿）。建议推送后随手修该一行界注。

Confidence: 92/100
