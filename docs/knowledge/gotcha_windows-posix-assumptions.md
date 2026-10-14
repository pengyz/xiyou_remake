---
name: windows-posix-assumptions
description: 仓库工具链 5 个 POSIX 假设在中文 Windows 全部落地即炸（python3 占位符/GBK 解码/classpath 冒号/line.separator/CRLF 检出），修法是 JVM+Python+git 三层钉死
type: gotcha
created: 2026-10-14
sources: ["2026-10-14 Windows 适配会话"]
---

工具链在中文 Windows 上有 5 个必踩坑：① `python3` 是微软商店占位符（在 PATH 里但执行即报错）；
② Python `subprocess text=True` 按 locale(GBK) 解码子进程输出，javap 的 UTF-8 中文字符串字面量直接
UnicodeDecodeError；③ `read_text()`/`write_text()` 不带 encoding 在 GBK locale 下读写 UTF-8 文件崩
（gates/cli.py 实测 traceback）；④ run.py classpath 用 `:` 拼接 ⇒ Windows 上
`ClassNotFoundException: oracle.host.Runner`；⑤ `core.autocrlf=true` 检出 CRLF ⇒ 按原始字节 sha256 的
门禁（reference-seed-integrity）恒红（LF 归一化后与基线一致，实测 21107ff3705a…）。

**为什么：** JVM 输出编码跟随**控制台**（sun.stdout.encoding）而非 file.encoding（JDK 18+ 已默认 UTF-8），
javap/trace/stderr 字节因此随平台漂移；`line.separator` 同理——TraceSink.println 的行尾跟随平台，
不强制则 Windows trace 为 CRLF、跨平台逐字节对拍失效。修法三层钉死，缺一层就有暗坑：
JVM 侧 `-Dfile.encoding/-Dsun.stdout.encoding/-Dsun.stderr.encoding`（javac/javap 加 `-J` 前缀）
+ `-Dline.separator=\n`；Python 侧 `subprocess(..., encoding="utf-8", errors="replace")`；
git 侧 `.gitattributes` `* text=auto eol=lf`。Windows 换算表：classpath 分隔符用 `os.pathsep`。

**何时使用：** ①新增任何 Python 工具/JVM 调用/按字节哈希的门禁时，按三层钉死模板写；
②Windows 跑门禁前先确认解释器真实可用（gates/hooks/pre-commit 已带 python3→python 探活回退，
install-hooks.sh 在无符号链接特权时退化为副本）；③诊断 classpath/编码/行尾类崩溃时，
先对照 reference/oracle/run.py 的 `JVM_UTF8`/`JVM_TOOL_FLAGS` 模板查缺哪层。
