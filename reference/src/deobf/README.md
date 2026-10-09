# reference/src/deobf/ — 无歧义版参考 Java（可编译实体）

本目录是 **Vineflower 1.12.0 对重映射后 class 的再生产物 + 投影修复补丁**，由

```bash
python3 tools/rename-pipeline/run.py
```

一键再生（幂等，含 javac 零错编译门禁）。**请勿手改**：任何手改都会与字节码失联，
也会在下次运行时被覆盖。

- 字段/方法名已按类型/签名唯一化（`f_<type>_<NN>` / `m_<NNN>`），语义命名叠加见
  [`data/naming/ledger.jsonl`](../../../data/naming/ledger.jsonl)（映射机械层：
  [`data/naming/remap-table.json`](../../../data/naming/remap-table.json)）；
- 等价性证据：javap 指令级（仅符号名差异）见
  [`tools/rename-pipeline/_verify/report.md`](../../../tools/rename-pipeline/_verify/report.md)；
  运行时等价（oracle 三方对拍 A==B==C 逐 tick 一致）见
  [`reference/oracle/_diff/report.md`](../../oracle/_diff/report.md)；
- VF 直出残余的 110 处槽位类型投影缺陷由声明式补丁修复：
  [`data/patches/vf-projection/`](../../../data/patches/vf-projection/)（每条带 javap 证据）；
- 不可变的历史 CFR 原始投影是 [`reference/seed/a.java`](../../seed/a.java)
  （D3 锚点，冻结件，勿混淆）。

> 【2026-10-10 更正】本目录 v1 时代为 CFR 再生产物（8 处 `** GOTO` 控制流补丁路线）；
> 实测 CFR 路线修复面 1615 错不可行，引擎切换 Vineflower 后补丁层重写为
> vf-projection 18 条记录。沿革见 docs/knowledge/decision_decompiler-cfr-to-vineflower.md。
