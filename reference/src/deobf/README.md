# reference/src/deobf/ — 无歧义版参考 Java（机械重映射产物）

本目录是 **CFR 对重映射后 class 的再生产物**，由

```bash
python3 tools/rename-pipeline/run.py
```

一键再生（幂等）。**请勿手改**：任何手改都会与字节码失联，也会在下次运行时被覆盖。

- 字段/方法名已按类型/签名唯一化（`f_<type>_<NN>` / `m_<NNN>`），
  旧名→新名映射表：[`data/naming/remap-table.json`](../../../data/naming/remap-table.json)；
- 等价性证据（javap 指令级 diff 仅符号名差异）：
  [`tools/rename-pipeline/_verify/report.md`](../../../tools/rename-pipeline/_verify/report.md)；
- 语义命名**未做**，将叠加在 `data/naming/ledger.jsonl`（现为空）；
- 不可变的 CFR 原始投影是 [`reference/seed/a.java`](../../seed/a.java)（D3 锚点，勿混淆）。
