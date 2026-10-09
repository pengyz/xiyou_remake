#!/usr/bin/env python3
"""patches.py — Vineflower 投影修复补丁 apply 引擎（v2）。

取代 t_cf1 的 CFR `** GOTO` 补丁引擎（CFR 路线 2026-10-10 退役，见
docs/knowledge/decision_decompiler-cfr-to-vineflower.md）。本引擎只做一件事：
读 `data/patches/vf-projection/VF-*.json`，把声明式锚定替换机械套用到
Vineflower 直出的源码文本上。**不手改生成物**：补丁记录才是可审计的真相源。

补丁记录格式（每文件一条）：
{
  "id": "VF-IWC",
  "target": "a.java",                    // 作用的源文件名
  "kind": "method-replace" | "scoped" | "global",
  "method": "   private boolean interactWithCell(...",  // scoped/method-replace 的方法签名前缀
  "body": "...",                          // method-replace：整方法新文本（3 空格缩进，配平大括号）
  "replacements": [                       // scoped/global：锚定替换列表
    {"old": "...", "new": "...", "count": 1}   // count=null 表示作用域内全部
  ],
  "evidence": "javap 依据（A/B 级）",
  "confidence": 92
}

硬校验（任一失败 ⇒ raise PatchError，run.py 捕获后整体 FAIL，拒绝落盘）：
  1. 记录的 target 必须命中本次产物之一；
  2. method 锚点在全文**唯一命中**且可配平大括号（method-replace/scoped）；
  3. 每条 replacement 的 old 在其作用域（方法内或全文）出现次数 == count（或 ≥1 当 count=null）；
  4. 全部套用后由 run.py 的 javac 步骤终审（编译零错），本引擎不自行判定语义。
"""
from __future__ import annotations

import json
from pathlib import Path


class PatchError(RuntimeError):
    pass


def load_patches(patches_dir: Path) -> list:
    """按文件名序读取 VF-*.json（应用顺序稳定 ⇒ 确定性）。"""
    records = []
    for p in sorted(patches_dir.glob("VF-*.json")):
        rec = json.loads(p.read_text(encoding="utf-8"))
        rec["_source_file"] = p.name
        records.append(rec)
    return records


def method_span(src: str, sig: str):
    """定位方法（签名前缀唯一命中）→ (start, end)，end 含配平的 `}` 与其后一个换行。"""
    n = src.count(sig)
    if n != 1:
        raise PatchError(f"方法锚点命中 {n} 次（须唯一）: {sig[:60]!r}")
    i = src.find(sig)
    start = src.rfind("\n", 0, i) + 1
    depth, j = 0, i
    while True:
        if j >= len(src):
            raise PatchError(f"方法大括号未配平: {sig[:60]!r}")
        c = src[j]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    end = j + 1
    if end < len(src) and src[end] == "\n":
        end += 1
    return start, end


def _apply_replacements(seg: str, reps: list, where: str) -> str:
    for r in reps:
        old, new, cnt = r["old"], r["new"], r.get("count")
        n = seg.count(old)
        if cnt is None:
            if n < 1:
                raise PatchError(f"{where}: 锚点未命中: {old[:60]!r}")
        elif n != cnt:
            raise PatchError(f"{where}: 锚点计数 {n} != 期望 {cnt}: {old[:60]!r}")
        seg = seg.replace(old, new)
    return seg


def apply_to_file(src_path: Path, patches_dir: Path) -> dict:
    """把全部补丁套用到单个源文件，返回报告 dict；失败即 raise（拒绝落盘）。"""
    records = load_patches(patches_dir)
    target = src_path.name
    src = src_path.read_text(encoding="utf-8")
    applied, report = [], {"target": target, "applied": [], "records": len(records)}
    for rec in records:
        if rec.get("target") != target:
            continue
        rid, kind = rec["id"], rec["kind"]
        if kind == "global":
            src = _apply_replacements(src, rec["replacements"], rid)
        elif kind in ("scoped", "method-replace"):
            s, e = method_span(src, rec["method"])
            if kind == "method-replace":
                src = src[:s] + rec["body"] + src[e:]
            else:
                seg = _apply_replacements(src[s:e], rec["replacements"], rid)
                src = src[:s] + seg + src[e:]
        else:
            raise PatchError(f"{rid}: 未知 kind={kind!r}")
        applied.append(rid)
    report["applied"] = applied
    if applied:
        src_path.write_text(src, encoding="utf-8")
    return report


def apply_all(sources: list, patches_dir: Path) -> list:
    """套用到产物列表；校验每条记录的 target 都被消费过（防补丁数据指向已不存在的产物）。"""
    records = load_patches(patches_dir)
    targets = {r["target"] for r in records}
    names = {p.name for p in sources}
    missing = targets - names
    if missing:
        raise PatchError(f"补丁 target 无产物可套用: {sorted(missing)}")
    reports = [apply_to_file(p, patches_dir) for p in sources]
    # 每条记录至少命中一个文件
    hit = {rid for r in reports for rid in r["applied"]}
    for rec in records:
        if rec["id"] not in hit:
            raise PatchError(f"{rec['id']}: 未能应用（target={rec['target']} 但锚点未命中？）")
    return reports


def format_report_section(reports: list) -> str:
    """报告章节：确定性内容（无时间戳），列每条记录的 id/kind/替换数。"""
    from collections import defaultdict
    lines = ["## 投影修复补丁（data/patches/vf-projection/）", ""]
    lines.append("Vineflower 直出的 110 处槽位类型投影缺陷（boolean/int/byte 转世、"
                 "byte 累加器 iinc 语义、this 自存等）由声明式补丁修复；"
                 "每条记录的 javap 证据与置信度见对应 JSON 文件。"
                 "语义终审 = oracle 三方对拍（A==B==C，见 reference/oracle/_diff/report.md）。")
    lines.append("")
    lines.append("| target | 记录 |")
    lines.append("|---|---|")
    for r in reports:
        ids = ", ".join(r["applied"]) or "（无）"
        lines.append(f"| {r['target']} | {ids} |")
    lines.append("")
    return "\n".join(lines)
