# tools/mod-skeleton — 两份制四规则骨架（survey §2.2）

> **骨架边界**（plan §Phase 3）：只做 **mod 表加载通道 + 四规则断言**，不做完整 mod
> 系统（无覆盖合并、无运行时装配、无打包）。保真面解析真值宿主=crates/game-data
> `defs.rs`（五类拒载）；本目录是 **mod 侧通道** 的骨架，拒载 kind 与 `defs.rs`
> `Reject` 变体同名镜像（BadJson/SchemaVersion/UnknownField/BadForeignKey/
> DomainOverflow/FrameIndexOutOfBounds）。

## 两份制四规则（survey §2.2 写死）

| 规则 | 实现 | 断言 |
|---|---|---|
| 1 mod 禁覆盖保真 id | `load_mod_table`：型号 id ≥ `MOD_TYPE_ID_MIN`(1000)；动画 id 必 `mod:` 前缀；保真 id（type 0..=88、`anim_row_*`）冻结禁覆盖 | `test_rule1_*` |
| 2 金样运行禁 mod 表 | `assert_golden_run_clean`：金样运行清单混入封套含 `mod_id` 的表 ⇒ **AssertionError** | `test_rule2_*` |
| 3 保真表哈希闸护航 | `verify_fidelity_hashes`：对照 `gates/table_content_hash.json`（gate `table-content-hash` **同源基线**）；漂移=拒载（非荣誉制：变更=勘误流程+金样重过） | `test_rule3_*` |
| 4 mod 独立基线 | `ModBaseline`（本目录 `mod_baseline.json`）：mod 表哈希单独立账，**不进金样闸**、保真哈希闸不背 mod | `test_rule4_*` |

## 加载通道语义（五类拒载镜像 + quirk 路由）

- **坏外键拒载**：`anim_profile`→保真∪mod anim、`sheet`→保真 containers（新容器=域外
  三界②）、`assembly_profile`→SEG 片件表、`quirk_ref`→quirks_in_code 登记；
- **域溢出拒载**：型号 id 命名空间（<89=覆盖保真；89..999=域外型；≥1000=mod）；
- **帧序越界拒载**：资源槽位/图集槽位 ≥ 所引容器张数（域外三界③，扩张数=重锚；
  张数表锚=`a.java:453` **2026-10-22 勘误：非 451**——名序声明 450/名序值行 451/张数
  赋值行 453，见 `../table-evidence/README.md` 勘误节）；
- **unknown-field=拒载**：封套/记录严格键集（防拼写静默）；
- **quirk 路由（§2.1 禁静默留码）**：quirk 型行（`frame_h=null` 运行时图高、
  `type_frame_width=null` fallback 32×32、`assembly_profile` SEG 加载副作用——与保真
  Q-001..Q-003 同族）必须 `quirk_ref` 指向已登记 quirk（quirk_id/reason/src/
  red_test_id 四字段留痕），否则 `QuirkPathRequired`——**quirk 型走 Q-00x 路径**，
  不许冒充纯表行混进零改码链。

## 用法

```bash
python3 -m unittest discover -s tools/mod-skeleton/tests -v   # 四规则最小测试（16 例）
```

```python
from loader import load_fidelity, load_mod_table, assert_golden_run_clean, ModBaseline

fid = load_fidelity(Path("data/tables"),
                    hash_baseline=Path("gates/table_content_hash.json"))  # 规则 3 护航硬前置
bundle = load_mod_table(Path("my_mod.json"), fid)                         # 规则 1 + 五类 + quirk
assert_golden_run_clean([Path("data/tables/sprite_defs.json")], fid)      # 规则 2
ModBaseline(Path("tools/mod-skeleton/mod_baseline.json")).update([...])   # 规则 4
```

## 关联

- 演练实跑：[`../zero-code-drill/`](../zero-code-drill/README.md)（survey §3 步 5=验收闸）；
- 保真表内容哈希闸：gate `table-content-hash`（`gates/cli.py`）；
- 机械对照器：[`../table-evidence/`](../table-evidence/README.md)（gate `table-evidence`）。
