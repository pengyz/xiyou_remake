# 对抗复核报告：块接口表（commit a532855）

- 复核者：独立对抗复核 agent（未使用作者脚本 /tmp/block_iface.py，重算脚本自写，存 /tmp/review_recompute.py / /tmp/r2_real*.py）
- 被复核对象：`docs/findings/2026-11-20-block-interface-table.md`（157 行）+ `docs/findings/block-interfaces.json`（55 层）
- 复核日期：2026-11-20
- 计数：**绿 10 / 黄 5 / 红 3**

> 结论先行：坐标系 bug 是本表最严重的问题——作者解析 maplv 用 `h = raw_height`（26），把 tile 行数当 cell 行数，行 y∈[13,26) 读进的是翻转网格（flip codes）的字节。全塔 55/55 层的 walkable 总数因此虚增约 40-60%（f41 JSON 144 vs 真实 80）；JSON 内 879 块中 496 块含幻影 cell（2202 个幻影 cell vs 4115 真实 cell）。但关键缓冲事实：**幻影 cell 与真实 cell 从不混在同一块**（mixed=0），且幻影区无任何 cut_door 标注；把对比限制到真实 13×13 区域后，54/54 层割点标注与 JSON 完全一致。因此「门拓扑」结论幸存，被污染的是 walkable 总数、块数（867 虚高）、逐块 cell 几何与 §3 统计。

---

## G 绿（核实通过）

### G1 55 层块划分与割点在真实区域内与作者一致
独立重算（同一口径：零钥匙连通分量、ty6/7/8 豁免、割点门簇判整），在真实 13×13（f51 12×13）区域内：54/54 层 cut_door 标注 multiset 完全一致；门簇展开、标注计数一致。门拓扑这一核心交付物成立。

复现：
```bash
python3 /tmp/r2_real_diff.py   # 输出 "floors with REAL-vs-JSON annotation mismatch: 0 /54"
```

### G2 判据 3（割点 ≥ keydemand）真实成立
对 52 个可比层（f1/f29/f49/f50/f51 无 needed，不算违例也不算通过）：0 违例。md 写"54/54 全过"是虚标（52 可比），但实质结论成立。f40 needed=(y1,r1)，JSON f40 标注 d1:1 d2:1 d3:1 g:1，红门-脚本门链簇口径未把「链上各门独立需要钥匙」抹掉。f4 min_needed_y=2 vs 该层 d1 标注，一致。f46 min_needed_y=3，一致。

复现：
```bash
python3 /tmp/check_judge3.py   # 0 violations, floors comparable=52
cat /tmp/keydemand.json | python3 -c "import json,sys; d={r['floor']:r for r in json.load(sys.stdin)['floors']}; print(d[40])"
```

### G3 判据 5（105 楼梯边）真实成立
作者脚本内该判据是循环自证（assert 自己数=105），但我独立跑 `cargo run --release -p game-solver -- floors` 与 JSON edge_reconcile 做集合 diff：**diff 为空，105==105 真**。楼梯边来自 sprite 坐标，不经过 buggy 的 maplv 高度，所以坐标系 bug 不影响此判据。

复现：
```bash
cargo run --release -p game-solver -- floors
python3 /tmp/edge_reconcile.py   # set diff empty
```

### G4 判据 1/2（cells 和、归属唯一）通过
全塔 JSON 内 cells 总和与 walkable 之和一致、每 cell 恰属一块。这是自洽性判据，通过但注意它无法发现坐标系 bug（幻影 cell 也被计入）。

### G5 导出面三分类与 items.json 逐条一致（抽 20 条）
20 ChangeFloorUp→topology、85 RemoveAllType(target1)→topology、86 KillAdjacent4→topology、19 Heal→timepoint、25 Passive→timepoint、13-15/23/24/26-32/33-40/79-80→const。商店语义（id6 StatPct 免费可重复复利、item19 经商店进场）与 md §4 描述一致。数值抽查无错。

复现：
```bash
python3 /tmp/check_exports.py
python3 -c "import json; d=json.load(open('data/tables/items.json'))['items']; print([ (i['id'],i['name'],i['effect']) for i in d if i['id'] in (13,19,20,21,25,30,85,86)])"
```

### G6 f41 召火自使能链的块内展开正确
f41 门 13/25 的召火豁免（slot13 召 ty6 地形豁免 → 清双火 → slot25 出 item22）在块划分中被正确当作「格内豁免」处理——slot13/25 所在格 (10,2) 静态不可走，被作者当作块外格正确剔除，豁免链本身不改变块划分。这一点口径正确（但见 R3 的缺口）。

### G7 必杀怪与怪可通行口径自洽
怪（cat8）判可通行、不判割点，与 md §2 描述一致。必杀怪（ty41/45）在门/脚本门的割点口径下天然不产生割点——因为它们是怪不是门，f51 (7,10) / f52 (11,7) 的割点标注实际来自同格的 ty83 脚本门，标注来源正确。

### G8 f29 口径差异如实记录
f29 keydemand status='blocked_by_assumption'（ty72+ty11 硬阻挡、gates_ignored=2），md 如实写明本表 ok（9 块）与 keydemand unreachable 的归因（keydemand 硬阻挡假设），两表口径差异已显式写进文档，未含糊。

### G9 门簇口径修复 f40 bug
作者自述曾抓出 f40 红门-脚本门链漏计，修复后簇口径（门簇整体判割点、簇内各门独立需要钥匙的事实保留在标注计数里）经 G2 复算无违例，未抹掉链上独立钥匙需求。

### G10 commit 单笔 docs-only
`git show a532855 --stat`：仅改 2 个 docs/findings 文件，无代码/资产夹带。

复现：
```bash
git show a532855 --stat
```

---

## Y 黄（需修但不阻塞门拓扑）

### Y1 坐标系 bug：55/55 层 walkable 虚增（头号问题，见结论先行）
`h` 应为 `raw_h>>1`（=13），作者用 `raw_h`（=26）。复现：
```bash
python3 -c "
import struct
d=open('assets/raw/maplv41','rb').read()
print('raw_w,raw_h =',struct.unpack('<HH',d[:4]))   # (26,26) → cell 13×13
"
python3 /tmp/r2_real_walkable.py   # 55/55 层 json walkable ≠ 真实
```
后果（量化）：879 块中 496 块含幻影 cell（2202 幻影 cell）；块数虚高（f1..54 JSON 块数和 867，真实区域重算显著更低）；§3 全塔 walkable 统计不可用。**不影响**：割点标注（G1）、楼梯边（G3）、导出面（G5）。

### Y2 md §3 全塔统计与 JSON 矛盾
md 称 d1=231/d2=17/d3=42/s4=33/s81=5（共 328 去重格）、标注 428、脚本门 33 格；JSON 实际去重 d1=95/d2=13/d3=35/s4=27/s81=5（共 175）、标注 d1=254/d2=23/d3=44/s4=51/s81=5/script=48（共 425）。md 数字与任何口径都对不上，疑为旧版脚本残留。md 的「30 时点全为商店」亦与 JSON 的 31 条（含 f0 item25）矛盾，且 §4 自己列了 f0 item25。

### Y3 孤立块计数三处不一致
JSON 实际 34 个 singleton；md §5 称 35；md §5 逐层清单加总 37（f26 称 25，JSON 实 22）。三处互斥，必有两处错。

### Y4 f41/f48 两条脚本门链整条缺失
f41 (10,2) slot13+slot25、f48 (4,1) slot38 的 ty83 格静态不可走，被脚本静默丢弃，未进 anomalies，md/JSON 均零记录。这两格是 f41 著名门链（slot13 召火自使能 → slot25 出 item22 下行符，有 f41_gate_chain_tests.rs 为证）与 f48 CES_84 链的入口——S2 若直接消费本表会**丢失这两条链及其 spawn 的 item22（拓扑类道具）**。其余 3 条 ty83 落块外（f26/f36）已标 singleton，处理正确。

复现：
```bash
python3 /tmp/check_ty83_outside.py   # 列出 5 条落块外 ty83，其中 2 条完全无记录
grep -n "f41" docs/findings/block-interfaces.json | head  # f41 无 script_gate 条目
```

### Y5 判据 5 是循环自证
作者脚本 `assert len(edges)==105` 只证明「自己数出来是 105」，不证明与 floorgraph 一致。我独立对账通过（G3），但 md 应写明判据的对账对象而非自证。

---

## R 红（阻塞级，不修不能进 S2）

### R1 坐标系 bug 未被任何判据捕获，且未列入 anomalies
55/55 层全错，但判据 1/2（自洽）、判据 3（≥ 比较，幻影割点只会多不会少）、判据 5（楼梯独立于网格）全部照常通过——本表的判据体系对「网格维度」零防御。作者未自察，md 未提任何坐标维度假设。这是结构性风险：同样的解析 bug 可以再次无声进入后续层表。

### R2 md §3/§5 统计失真（见 Y2/Y3）
文档层面的数字与数据文件互相矛盾，违反 AGENTS.md「findings 必须区分已证实/假设」的精神——读者无法判断哪个数字是权威。块数 867 虚高直接误导 S2 的块图规模预期。

### R3 f41/f48 脚本门链整条缺失且无标记（见 Y4）
这两条链是塔内知名的自使能拓扑（有专门回归测试），丢失即 S2 的块图在 f41/f48 处行为性错误（玩家实际能过、块图判不能过），属于「静默错误假设」，必须补进表或 anomalies。

---

## 与 lead 攻击面逐条对应

| 攻击面 | 判定 |
|---|---|
| F1 独立重算（f4/f29/f40/f41/f51） | G1 通过（真实区域内标注一致），但叠加 Y1 坐标 bug |
| F2 割点口径 | G7 通过；必杀怪 2 格实为脚本门格标注，非怪本身割点，无逻辑错误 |
| F3 ty4/81 当割点 | engine 证实 ty4 恒拦截（scene.rs interact 顺序）、ty81 spawn 即隐形可穿（sprite_spawn.rs hide_after_spawn），作者按「恒拦截」标保守假设成立，需在表内注明 ty81 实际可穿（已在 §3 注明，无新问题） |
| F4 动态门处理 | G6：f41 豁免地形展开正确；缺口见 R3 |
| F5 导出面三分类 | G5 通过 |
| F6 一致性判据真实性 | G2/G3 真、G4 自洽但无维度防御（R1）、判据 5 自证（Y5） |
| F7 f29 口径 | G8 如实 |
| F8 夹带与诚实 | G10 docs-only；异常清单不完整（R3、Y2/Y3 计数矛盾） |

## 两句话结论

**S1 不能直接作为 S2 输入**：块划分的 cell 列表/块数/walkable 全塔被 13×26 幻影坐标系污染（496/879 块含幻影 cell），且 f41/f48 两条脚本门链整条静默丢失——但门拓扑（割点标注/楼梯边/导出面分类）经独立重算证实幸存，S1 的「接口结论」部分本身是对的。

**进 S2 前必修三件事**：① 解析器 `h` 改 `raw_h>>1` 重算全表并加「维度=13×13（f51 12×13）」判据进门禁；② f41 slot13/25、f48 slot38 两条 ty83 链补进表或 anomalies（含 item22 拓扑影响）；③ md §3/§5 全塔统计重算并与 JSON 对齐（去重格数、标注数、singleton 数三处矛盾消解）。

Confidence：坐标系 bug 与门拓扑幸存结论 A 级（字节级复现 + 独立重算 54/54）；必杀怪证伪、ty83 缺口 B 级；其余判定 B 级。

---

# 定向复核追加（a26e9a0，2026-11-20 18:59）

针对修复 commit `a26e9a0`（docs-only：md 重写 + JSON v2），只核三红是否真修。

**计数：绿 8 / 黄 1 / 红 0**

## R1 坐标系 — 真修（绿）

- ①维度：55/55 层 `width/height ∈ {(13,13),(12,13)}`，0 个 `y≥13` cell；f41 walkable=80 ✓、f37/f23=121 ✓（作者给的全部对照点命中）；f1..54 walkable 总和 4021、块数 382，与 commit message 一致。
- ②独立重算（重写脚本，仍非作者代码）：f26 walkable 62==62、blocks 25、singleton 22 ✓；f52 walkable 69==69 ✓；54/54 层 block-cell 并集与真实可走集合完全相等；割点标注 multiset 54/54 与我的修正口径重算完全一致。
- ③维度守卫已入 md §1（断言 `(w,h)∈{(13,13),(12,13)}` 越界即失败，每层 JSON 带 width/height），防回归口径成立。

复现：
```bash
python3 -c "import json;a=json.load(open('docs/findings/block-interfaces.json'));print([f['floor'] for f in a['floors'] if not (f['height']==13 and f['width'] in (12,13))])"   # []
# 割点标注独立重算 diff = 0/54（/tmp/r2_real_diff.py 已重建；log 见会话）
```

## R2 统计直出 — 真修（绿）

- md §3 割点标注 d1:254 / script_gate:48 / d3:44 / sealed4:51 / sealed81:5 / d2:23（合计 425）——与 JSON 逐 kind Counter **完全一致**。
- 导出面 448 const / 30 timepoint / 3 topology —— JSON 直出一致（旧 31 含 f0 item25，现口径 f1..54，矛盾消解）。
- 孤立阻断格 34（f26:22/f41:5/f24:2/f25:2/f13,f36,f38:1）——与 JSON 中带 `note` 的 singleton 块数一致；另有 14 个 1 格可走小袋（f3/f14/f31/f34…）是正常单格连通分量，与"34 孤立阻断"不矛盾（JSON 48 个 len==1 块 = 34+14，口径已分清）。
- 块数 382、分布表、逐层 walkable 表均与 JSON 一致；旧三处矛盾（§3 数字、singleton 34/35/37、时点 30/31）全部消解，且 md 保留了 v1→v2 修正记录（867→382、6188→4021、54/54 层下调表），符合"原地标注更正"纪律。

## R3 脚本门链补录 — 真修（绿）

- f41 (10,2) slot13+slot25、f48 (4,1) slot38 三条均已进：块 `exits[]`（带 `"outside_terrain": true`、spawns 完整）+ JSON `anomalies[]` + md §4 表格。
- item22 拓扑标注在：JSON anomaly 明写"spawns 含拓扑道具 [22]"，md §4 明写"spawn item22 下行符@f41(6,6)——拓扑类导出面，本层块图之外的关键通道"，并注明"S2 计代价时不得降级为不可达"。
- 「无 LAY 不入 105 条」说明如实（md §4）；edge_reconcile 106 行 = solver 范围 105（stair 104 + script(LAY) 1）+ f0 边 1 条另计，与 `cargo run -p game-solver -- floors` 独立对账仍成立（105==105）。

## 判据 3 改口「49 可比层」— 如实（绿，带一处黄）

- 重跑 `cargo run --release -p game-solver -- keydemand`：54 行，ok=**49**、no_entry=2（f1,f51）、no_up_exit=2（f49,f50）、blocked_by_assumption=1（f29）——"49 个可比层（keydemand ok 层）"计数精确，5 层排除理由如实（criterion 4 逐条列出）。
- **黄 1**：判据 3 的行内举例引用错 keydemand 数值——md 写"f4 需黄1"，实际 keydemand f4=min_y**2**；写"f46 需黄1+蓝2"，实际 f46=min_y**3**、b=0、r=0。PASS 判定本身不受影响（f4 cut d1:7≥2 ✓、f46 d1:8≥3 ✓），但引用数字必须改，否则违反"带证据指针"纪律。

复现：
```bash
cargo run --release -p game-solver -- keydemand --out /tmp/keydemand2.json
python3 -c "import json;k={r['floor']:r for r in json.load(open('/tmp/keydemand2.json'))['floors']};print(k[4]['min_needed_y'],k[46]['min_needed_y'],k[46]['min_needed_b'])"   # 2 3 0
```

## 定向复核结论（两句话）

**三红全部真修**：坐标系修正后 55/55 层维度、walkable、块数与我独立重算完全一致（割点标注 54/54、f26/f52 抽查通过），统计改为脚本直出后 md 与 JSON 零矛盾，三条门链补录完整且 item22 拓扑标注到位。

**a26e9a0 可以作为 S2 输入并提交入库**：唯一遗留是判据 3 行内两处 keydemand 数值引用笔误（f4 应为黄 2、f46 应为黄 3 蓝 0，PASS 结论不变），建议随下一个 docs commit 顺手更正，不阻塞。Confidence：本追加判定 A/B 级（维度与标注为独立重算字节级复现，keydemand 为 solver 重跑）。
