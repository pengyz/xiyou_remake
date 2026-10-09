# original/ — 原始 JAR 只读归档

**本目录是全部游戏资产的唯一真相源（single source of truth）。**

| 文件 | 说明 |
|---|---|
| `囧囧西游-大闹天宫.jar` | 原始 MIDlet（319527 字节）。来源：用户提供，2010-01-20 SNOWFISH 发行 |
| `SHA256SUMS` | 完整性校验（`sha256sum -c SHA256SUMS`），门禁 `gates/cli.py original` 自动检查 |

## 红线（AGENTS.md §2）

- **任何字节修改 = 严重违规**。本目录只允许追加新的只读归档，不允许改写既有文件。
- 所有派生资源（解包、转格式、提取）一律进 `analysis/` 或 `assets/`（不入库），
  且必须可用 `tools/` 中的工具从本目录一键再生。
- 需要"修正资源"时：修正逻辑写进转换工具，**绝不修改本目录**。

## 快速考证命令（证据纪律 §3 的复现入口）

```bash
unzip -l "original/囧囧西游-大闹天宫.jar"          # 138 个条目
unzip -p "original/囧囧西游-大闹天宫.jar" maplv0 | xxd | head   # 关卡格式考证
```
