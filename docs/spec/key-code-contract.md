# 键码契约（key_value 写侧+消费侧全量）——状态机件 C

> 2026-10-17 建档（state-machine-refactor-plan 件 C）。**零代码=零验证**教训（复核
> A2-件C）：原文档只写消费 26 处半边——本件补**写侧 41 写点**入契约 + gates 锚机械
> 校验（防注释漂移——本仓行号漂移前科两次）。数据源：state-machine-survey §4。

## 1. 消费模型（edge 语义，a.java:4270/4276 尾清锚）

```
press_key(code) ──置──▶ key_value = code（覆盖式取末键）
                        │
下一拍 tick ────────────┼──▶ 各态分支 match key_value 消费（26 处）
                        │
每消费臂尾 ─────────────┴──▶ key_value = 0（尾清：scene 17 + BootMachine 9）
```

- **held 语义分离**：`key_held_code` 独立字段（行走连续步进用），与 key_value 无耦合。
- **覆盖式取末键**：连续 press 只留末键（a.java:4270 语义）——键队列不存在（Stage A 不发明）。

## 2. 消费侧 26 处（state-machine-survey §4 表逐条）

| 归属 | 处数 | 键码面 | 复用性 |
|---|---|---|---|
| mode 3 子分派（tick 六段） | 6 | -7/-6（暂停/道具）、49-55（行走方向 split up/down）、-5/53（探索）、-1/-3（菜单） | -7/-6/-5 跨态复用 |
| 菜单族 tick_menu4/19/7/15/5/9/16 | 7 | -7 返回/-6 确认/-2/-4 下移/-1/-3 上移 | **键码语义跨态复用**（同码不同动作） |
| tick_item_panel | 1 | 同菜单族 + 使用 | 私有 |
| handle_overlay_key | 1 | ±1 翻页/±lines 滚动 | overlay 私有 |
| tick_dialog（:2600） | 1 | 对话推进键 | 私有 |
| BootMachine tick（paint 侧） | 6 | boot/title 键序 | BootMachine 私有 |

## 3. 写侧 41 写点（本件补全——原契约缺口）

**写侧语义面**（Rust 实测：`key_value = ` 45 处含初始化、其中 `key_value = 0` 尾清 26 处）：

| 写点类 | 语义 | 锚 |
|---|---|---|
| press_key 置值 | 覆盖式取末键（code 原样） | a.java:4270 |
| 尾清 `key_value = 0` | 每消费臂一处（26 处：scene 17 + BootMachine 9） | a.java:4270/4276 |
| 测试夹具直写 | #[cfg(test)] 合成驱动（非生产写点） | 测试面 |
| 初始化 | `key_value: 0`（结构体构造） | — |

**判据**：生产写点=press_key 置值 + 尾清两类；其余为测试夹具/初始化（gates 锚校验
核对分类计数）。

## 4. 跨态键码约定（⚠️ 同码不同动作——转移表化禁单维组织）

| 键码 | 通用约定 | 各态动作差异（例） |
|---|---|---|
| **-7** | 返回/退出 | mode3→暂停菜单；菜单族→menu_return_mode；对话→翻页/关闭 |
| **-6** | 确认/使用 | mode3→道具面板；菜单→项激活；道具面板→使用 |
| **-5/53** | 探索/确认键 | mode3 walkPhase=0 探索；标题菜单确认 |
| **-1/-3** | 上移 | 菜单光标上移（mode3 子分派另有语义） |
| **-2/-4** | 下移 | 菜单光标下移 |
| **49-55** | 行走方向 | mode3 walkPhase 行走（split up/down） |

⚠️ **(state, key) 二元组织**（survey §4 要点）：键码语义跨态复用——任何键码映射表/
转移表须按 (state, key) 而非 key 单维。

## 5. 锚校验表（gates `state-machine-anchors` 机械对照）

| 锚 | 期望 token | 语义 |
|---|---|---|
| a.java:4270 | `keyValue = 0` | 尾清锚 |
| a.java:4276 | `keyValue = 0` | 尾清锚（BootMachine） |
| a.java:3405-3425 | `gameMode = 17` | 菜单激活写点（mode 17 入口） |
| a.java:3415 | `gameMode = 17` | 同上逐字 |
| a.java:3420 | `gameMode = 22` | 菜单激活写点（mode 22 入口） |
| a.java:902 | `2, 3, 3, 3, 6, 3, 3, 6` | f_byte_arr_13 表常量（restore case12） |

（校验=gates/cli.py `check_state_machine_anchors`：锚行号在 deobf 范围内 + 期望
token 在锚行邻域命中——防行号漂移与注释漂移，本仓前科两次。）
