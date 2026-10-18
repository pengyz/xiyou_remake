#!/usr/bin/env python3
"""asset_manifest —— 真相源哈希清单（STAGE-B-003 复核硬 4，task-10 半边）。

AGENTS.md §2：`original/` 是**唯一资产真相源**；`assets/`（raw/png/art/audio）一律
gitignored 再生成物。复核硬 4 的禁第三态要求：**入库且无生成锚**的状态不存在——
本清单即"生成关系锚"（源 jar sha256 + 每资产源偏移 + 产物 sha256），入库随
`data/status/baselines/` 走，gates 校验防漂移（挂接在 task-8 落地后做，届时跑
门禁自测 + 基线影响说明，AGENTS §5）。

清单结构（data/status/baselines/asset-truth-manifest.json）：
  source    真相源 jar：路径 / sha256 / size
  entries   jar 内每条目：name / header_offset（zip 本地头偏移）/ data_offset
            （条目数据偏移）/ size / sha256（条目字节）/ products（再生成物路径→sha256）
  artifacts PNG+JSON 中间产物（assets/art，task-008 unpack 生成）：
            route = gitignored-generated；products = 产物相对路径→sha256
  counts    口径计数（与 data/status/baselines/baseline.json 对账面）

三态核（verify）：
  S1 清单空 ∧ art 目录缺      = 一致（task-8 前的现状）
  S2 清单满 ∧ art 目录在 ∧ 全部产物哈希命中 = 一致（生成物再生后）
  S3 其余任何组合（清单空但目录在 / 哈希漂移 / 有未登记产物）= **第三态 ⇒ exit 1**

用法：
  python3 tools/asset_manifest.py build    # 重算并写清单
  python3 tools/asset_manifest.py verify   # 校验（gates 挂接口）：漂移 ⇒ exit 1
  python3 tools/asset_manifest.py selftest # 工具自检（build→verify 干跑）

红线：本工具只读 original/ 与 assets/，绝不写原始归档（AGENTS §2）。
"""
import hashlib
import json
import struct
import sys
import zipfile
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JAR = ROOT / "original" / "囧囧西游-大闹天宫.jar"
MANIFEST = ROOT / "data" / "status" / "baselines" / "asset-truth-manifest.json"
ASSET_ART = ROOT / "assets" / "art"
VERSION = 1


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def entry_offsets(jar_bytes: bytes, info: zipfile.ZipInfo) -> tuple[int, int]:
    """(header_offset, data_offset)：zip 本地头 30B + 文件名 + extra 后即数据起点。
    注意必须读 jar **原始字节**（ZipExtFile 读出的是解压数据，不是本地头）。"""
    header = info.header_offset
    head = jar_bytes[header : header + 30]
    if len(head) < 30 or head[:4] != b"PK\x03\x04":
        raise ValueError(f"{info.filename}: 非法 zip 本地头 @ {header}")
    name_len, extra_len = struct.unpack("<HH", head[26:30])
    return header, header + 30 + name_len + extra_len


def iter_entries():
    """逐条目产出 (name, header_offset, data_offset, size, sha256, bytes)。"""
    jar_bytes = JAR.read_bytes()
    with zipfile.ZipFile(JAR) as z:
        for info in z.infolist():
            data = z.read(info.filename)
            hoff, doff = entry_offsets(jar_bytes, info)
            yield info.filename, hoff, doff, len(data), sha256(data), data


def build_manifest() -> dict:
    entries = []
    counts = {"entries": 0, "maplv": 0, "sprite": 0, "classes": 0}
    for name, hoff, doff, size, digest, _data in iter_entries():
        counts["entries"] += 1
        if name.startswith("maplv"):
            counts["maplv"] += 1
        if name.startswith("sprite"):
            counts["sprite"] += 1
        if name.endswith(".class"):
            counts["classes"] += 1
        # 再生成物：assets/raw/<name> = 条目字节原样（extract.py 口径）
        products = {}
        raw = ROOT / "assets" / "raw" / name
        if raw.is_file():
            products[f"assets/raw/{name}"] = sha256(raw.read_bytes())
        entries.append(
            {
                "name": name,
                "header_offset": hoff,
                "data_offset": doff,
                "size": size,
                "sha256": digest,
                "products": products,
            }
        )
    art_products = {}
    if ASSET_ART.is_dir():
        for p in sorted(ASSET_ART.rglob("*")):
            if p.is_file():
                art_products[str(p.relative_to(ROOT)).replace("\\", "/")] = sha256(p.read_bytes())
    return {
        "version": VERSION,
        "generated_by": "tools/asset_manifest.py",
        "note": "真相源生成关系锚（STAGE-B-003 硬4 禁第三态）：源 jar+偏移+产物哈希；"
        "assets/ 全部 gitignored 再生成物，本清单钉生成结果防 '改产物不改 original' 无声漂移",
        "source": {
            "path": "original/囧囧西游-大闹天宫.jar",
            "sha256": sha256(JAR.read_bytes()),
            "size": JAR.stat().st_size,
        },
        "entries": entries,
        "artifacts": {
            "route": "gitignored-generated",
            "note": "assets/art = unpack 生成物（task-008 落地后填充 products）；"
            "三态核：清单空∧目录缺 或 清单满∧哈希全命中，其余=第三态拒绝",
            "products": art_products,
        },
        "counts": counts,
    }


def verify(manifest: dict) -> list[str]:
    problems: list[str] = []
    # 真相源
    if not JAR.is_file():
        return [f"真相源缺失: {JAR}"]
    if sha256(JAR.read_bytes()) != manifest["source"]["sha256"]:
        problems.append("源 jar sha256 漂移（original/ 变更？只读红线）")
    if JAR.stat().st_size != manifest["source"]["size"]:
        problems.append("源 jar size 漂移")
    # 条目：偏移/字节/产物
    jar_bytes = JAR.read_bytes()
    seen = set()
    with zipfile.ZipFile(JAR) as z:
        infos = {i.filename: i for i in z.infolist()}
    for name, hoff, doff, size, digest, data in iter_entries():
        seen.add(name)
        # 偏移独立实证：data_offset 起的压缩流独立解压 == 条目字节
        # （jar 条目多为 DEFLATE——切片是压缩流，直接比字节是比错对象；
        #   stored 条目切片即数据）
        info = infos[name]
        stream = jar_bytes[doff : doff + info.compress_size]
        if info.compress_type == zipfile.ZIP_STORED:
            ok = stream == data
        elif info.compress_type == zipfile.ZIP_DEFLATED:
            ok = zlib.decompress(stream, -15) == data
        else:
            ok = False
        if not ok:
            problems.append(f"{name}: data_offset={doff} 独立解压≠条目字节（偏移数学错）")
        row = next((e for e in manifest["entries"] if e["name"] == name), None)
        if row is None:
            problems.append(f"清单缺条目: {name}")
            continue
        for field, got in (("header_offset", hoff), ("data_offset", doff), ("size", size), ("sha256", digest)):
            if row[field] != got:
                problems.append(f"{name}: {field} 漂移（清单 {row[field]} vs 实 {got}）")
        for rel, want in row["products"].items():
            p = ROOT / rel
            if not p.is_file():
                problems.append(f"{name}: 产物缺失 {rel}")
            elif sha256(p.read_bytes()) != want:
                problems.append(f"{name}: 产物哈希漂移 {rel}")
    for row in manifest["entries"]:
        if row["name"] not in seen:
            problems.append(f"清单多余条目（jar 中已无）: {row['name']}")
    # 三态核（artifacts）
    art_dir = ASSET_ART.is_dir() and any(ASSET_ART.rglob("*"))
    art_products = manifest["artifacts"]["products"]
    if art_products and not art_dir:
        problems.append("第三态：清单有 art 产物记录但 assets/art 缺失")
    elif art_dir and not art_products:
        problems.append("第三态：assets/art 存在但清单零登记（入库且无生成锚=禁）")
    elif art_products:
        for rel, want in art_products.items():
            p = ROOT / rel
            if not p.is_file():
                problems.append(f"art 产物缺失: {rel}")
            elif sha256(p.read_bytes()) != want:
                problems.append(f"art 产物哈希漂移: {rel}")
        if ASSET_ART.is_dir():
            for p in sorted(ASSET_ART.rglob("*")):
                if p.is_file():
                    rel = str(p.relative_to(ROOT)).replace("\\", "/")
                    if rel not in art_products:
                        problems.append(f"第三态：未登记 art 产物 {rel}")
    # assets/ 必须不入库（第三态的入库面）
    return problems


def main() -> int:
    try:
        sys.stdout.reconfigure(encoding="utf-8")  # Windows GBK 控制台防乱码
    except Exception:
        pass
    mode = sys.argv[1] if len(sys.argv) > 1 else "verify"
    if not JAR.is_file():
        print(f"真相源缺失: {JAR}", file=sys.stderr)
        return 1
    if mode == "build":
        manifest = build_manifest()
        MANIFEST.parent.mkdir(parents=True, exist_ok=True)
        MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=False) + "\n", encoding="utf-8")
        print(f"清单已写: {MANIFEST.relative_to(ROOT)}（entries={len(manifest['entries'])}）")
        return 0
    if mode == "verify":
        if not MANIFEST.is_file():
            print(f"清单缺失: {MANIFEST.relative_to(ROOT)}（先 build）", file=sys.stderr)
            return 1
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        problems = verify(manifest)
        if problems:
            print(f"真相源清单校验 FAIL（{len(problems)} 项）:", file=sys.stderr)
            for p in problems:
                print(f"  - {p}", file=sys.stderr)
            return 1
        print(f"真相源清单校验 PASS（entries={len(manifest['entries'])}，art 三态核通过）")
        return 0
    if mode == "selftest":
        # 干跑：build 结果直接 verify（不落盘），自证两半一致
        manifest = build_manifest()
        problems = verify(manifest)
        if problems:
            print(f"selftest FAIL（{len(problems)} 项）:", file=sys.stderr)
            for p in problems:
                print(f"  - {p}", file=sys.stderr)
            return 1
        print(f"selftest PASS（entries={len(manifest['entries'])}）")
        return 0
    print(f"未知模式: {mode}（用法见文件头）", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
