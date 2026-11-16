# 2026-11-16 · 删 ScriptEngine 尾部收尾门 + 夹具剥离 `0x1E`（落地记录）

> 依据裁定：`docs/superpowers/proposals/2026-11-16-script-command-pipeline-proposal.md` §0′.4′
> （javap 直读 `analysis/rename-pipeline/renamed/a.class`，方法体 `executeScriptInstruction` 偏移 0–2431）。
> 用户授权：m00440「按裁决开始落地：先删 script.rs 尾门，再处理 0x1E」。

## 1. 结论（两句）

1. **`ScriptEngine` 尾门已删除**：Java 的行尾检查两分支都是 `return`、零字段写
   （`2397 if_icmpge 2431` / `2400 return` / `2431 return`），收尾状态只由入口门
   （偏移 22 的 `cursor+3 > len` ⇒ `2401 dialogPhase=4` + `flags[idx]` + `lockCameraOn(0)`）写入。
   旧尾门让「末命令即行尾」的行（如槽 6 `TAK_53_58 `）在**解析同拍**被收尾，整页对白被跳过。
2. **夹具行已剥 `U+001E` 且不再 `trim`**：`fixture_script_line` 只剥提取器追加的记录分隔符、
   保留行尾空格 ⇒ 生产口径的 `script_cursor` 终值与 Java 相等，槽 67 不再 panic。

## 2. 证据

| 事实 | 级别 | 指针 |
|---|---|---|
| 行尾检查零字段写（两分支均 `return`） | A（字节码） | javap 偏移 `2397`/`2400`/`2431`；裁定全文见提案 §0′.4′ |
| 收尾写只由入口门到达（`2401`） | A（字节码） | 同上；唯一跳 `2401` 的指令在偏移 `22` |
| Java 槽 67 字面量带行尾空格（12 字符） | A（反编译字面量） | `a.java:1192`；`docs/spec/evidence-boss.md:576`；`docs/spec/dead-code-rulings.md:177` |
| 夹具行尾 = `" \x1e"`（提取器追加） | A（字节计数） | `crates/game-core/tests/fixtures/level_scripts.txt` 3609 字节 / 67 个 `0x1E` / 68 槽 |
| 生产执行器本无尾门 | A（源码） | `crates/game-core/src/scene_script.rs:14` 只有入口门 ⇒ 本改动**不动生产行为** |

## 3. 改了什么

| 文件 | 改动 | 行为面 |
|---|---|---|
| `crates/game-core/src/script.rs` | 删除尾部收尾门（原 `:511-519`），原位留裁定注释 | `ScriptEngine` 路径（求解器/对拍 (b) 腿）：行尾收尾晚一拍，与 Java 一致 |
| `crates/game-core/src/scene.rs` | 新增 `pub fn fixture_script_line`；`from_fixtures` 改用它（剥 `0x1E`、不 `trim`） | 生产 `GameScene` 行长 −1（Java 口径）；末槽 67 尾空格保留 |
| `crates/game-core/tests/script_golden.rs` | 新增 2 个 TDD 测试；2 个既有断言改判（附更正注） | 测试语义对齐 Java 拍点 |
| `crates/game-core/tests/script_fixture_lines.rs` | 新增：无 `0x1E` / 行长对齐 / 生产与测试表同口径 | 新增 3 个测试 |
| `crates/game-core/src/scene_solver_api.rs` | `slot67_*_panic_parity` → `slot67_dc5_parse_parity` / `slot67_dc5_execute_parity`（state-parity） | 槽 67 正常解析（DC-5 不可达不变） |
| `crates/game-solver/src/script_abs.rs` | 槽 67 数据守卫改为「(a)(b) 同口径」；模块文档旧偏差段标记已闭合 | 对拍两侧语料一致 |
| 7 处注释 | 「行尾收尾门」→「下一拍入口门」（机制措辞更正） | 仅注释 |

## 4. TDD 记录（先红后绿）

新增测试先跑，红：

```
---- tail_check_is_noop_tak_at_line_end ----
assertion `left == right` failed: 同一拍不得被尾检查改写成 AwaitCamera（否则整页对白立即收尾）
  left: AwaitCamera   right: Typewriter
---- tail_check_writes_at_next_tick_entry_gate ----
assertion `left == right` failed: 页区间耗尽 ⇒ phase 0
  left: AwaitCamera   right: ScriptStep
```

删门后两测试转绿；同时暴露两处旧断言编码了错误语义（`script_golden.rs` SEE 后
`AwaitCamera`），按裁定改为「同拍保持 6，下一拍入口门才落 4」，并在原位留更正注。

夹具改动后暴露第三处：`slot67_dc5_unreachable_*_panic_parity` 断言的是**移植偏差**
（`.trim()` 吃尾空格 ⇒ panic），与 Java 字面量不符，已改判为 state-parity。

## 5. 门禁

```
python3 gates/cli.py check → PASS=13 FAIL=0 SKIP=0
   rust-tests ✓ / test-ratchet ✓(1089 vs 基线 1075) / state-baseline ✓ 74 场景 sha 全同
   table-content-hash ✓ / state-machine-anchors ✓ / asset-truth-manifest ✓
cargo test -p game-solver → 全绿（含 24 项对拍，586s）
```

## 6. 影响与遗留

- **求解器语义变严**：`ScriptEngine` 的行尾收尾从「同拍」改为「下一拍入口门」，
  与生产 `GameScene`、与 Java 三方一致 ⇒ 后续 `fast-forward` 分类表的拍点前提成立。
- **三查询未重跑**：`0x1E` 只改 cursor 终值，不改可达性；但批 6 的三查询结论
  （goals=0）与本次改动叠加后仍应以重跑为准（批 7 起点）。
- **未推送**：本轮改动的 commit 与既有 3 笔 docs（`dd38dca`/`b391099`/`5a85bb8`）都待用户点头。
- **S4a 未开工**：删门只是 S4a 的第一小步；`END`/`LAY`/`ROS_4`/`CES` 四处对齐仍待裁定。

## 7. 假设区

无新增假设。槽 67 的「Java 带尾空格」有 A 级字面量证据；DC-5 不可达裁定独立于本源码行为。
