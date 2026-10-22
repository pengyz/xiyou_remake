#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""mod 骨架加载通道——两份制四规则（survey §2.2）的最小实现。

> 骨架边界（plan §Phase 3）：只做**加载通道 + 四规则断言**，不做完整 mod 系统
> （无覆盖合并、无运行时装配、无打包）。保真面解析/跨表校验的真值宿主是
> `crates/game-data/src/defs.rs`（五类拒载），本文件是 mod 侧通道的骨架镜像，
> 错误 kind 命名与 defs.rs `Reject` 变体一一对应（BadJson/SchemaVersion/
> UnknownField/BadForeignKey/DomainOverflow/FrameIndexOutOfBounds）。

两份制（survey §2.2 四规则写死）：
  规则 1  mod 禁覆盖保真 id——id 命名空间隔离：mod 型号 ≥ 1000（[`MOD_TYPE_ID_MIN`]）
          或前缀（mod 动画 id 必须 `mod:` 前缀）；保真 id 冻结（type 0..=88、
          anim_row_*）不得被 mod 覆盖；
  规则 2  金样运行禁 mod 表——[`assert_golden_run_clean`] 断言金样运行清单只含保真表；
  规则 3  保真表内容哈希闸护航——[`verify_fidelity_hashes`] 对照
          `gates/table_content_hash.json`（gate `table-content-hash` 同源基线），
          哈希不匹配=拒载（非荣誉制：变更=勘误流程+金样重过）；
  规则 4  mod 独立基线——[`ModBaseline`] 单独记录 mod 表哈希，**不进金样闸**。

五类拒载镜像（§2.3）+ quirk 路由（§2.1 禁静默留码）：
  quirk 型行（运行时图高/fallback 尺寸/SEG 加载副作用等纯表规则装不下的形状）必须
  `quirk_ref` 指向已登记 quirk（quirk_id/reason/src/red_test_id 四字段留痕），
  否则 [`ModReject`] kind=`QuirkPathRequired`——quirk 型走 Q-00x 路径，不许冒充纯表行。
"""
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# ---------------------------------------------------------------- 契约常量

#: mod 表 schema 版本 = `crates/game-data/src/defs.rs` `SCHEMA_VERSION`
SCHEMA_VERSION = 2
#: mod 型号命名空间下界（survey §2.2 规则 1：mod 型号 ≥ 1000）
MOD_TYPE_ID_MIN = 1000
#: mod 动画 id 前缀（survey §2.2 规则 1：……或前缀）
MOD_ANIM_PREFIX = "mod:"
#: 保真型号域上界（defs.rs `TYPE_ID_MAX`；冻结，禁 mod 覆盖）
FIDELITY_TYPE_ID_MAX = 88
#: 保真表内容哈希闸基线（gate `table-content-hash` 同源）
HASH_BASELINE = ROOT / "gates" / "table_content_hash.json"

#: mod 表封套键（第 5 类 unknown-field=拒载的键集）
MOD_ENVELOPE_KEYS = ("schema_version", "mod_id", "sprite_defs", "anim_defs",
                     "quirks_in_code", "resource_rows")
#: mod 精灵行键 = 保真 SPRITE_KEYS + `quirk_ref`（quirk 路由，§2.1）
MOD_SPRITE_KEYS = ("type_id", "sheet", "sheet_index", "frame_w", "frame_h",
                   "render_category", "anim_row", "type_frame_width", "anim_profile",
                   "assembly_profile", "insert_block", "floor_category_override",
                   "transition_overrides", "density", "evidence", "quirk_ref")
MOD_ANIM_KEYS = ("id", "frames", "step_ticks", "loop_mode", "evidence")
MOD_QUIRK_KEYS = ("quirk_id", "reason", "src", "red_test_id")
MOD_RESOURCE_KEYS = ("container", "slot", "source")
MOD_FLOOR_KEYS = ("floor", "category")
MOD_TRANSITION_KEYS = ("on", "by_category", "evidence")
MOD_ANCHOR_KEYS = ("index", "value", "src")

_QUIRK_ID_RE = re.compile(r"^Q-\d{3}$")


class ModReject(Exception):
    """mod 通道拒载/护航失败。`kind` 与 defs.rs `Reject` 变体同名镜像
    （另有 QuirkPathRequired / FidelityHashDrift / ModBaselineDrift 三个骨架级 kind）。
    每条信息含定位（`path`）——§2.3 错误信息含定位。"""

    def __init__(self, kind: str, path: str, detail: str):
        self.kind = kind
        self.path = path
        self.detail = detail
        super().__init__(f"{kind}（路径 {path}）：{detail}")


def _reject(kind: str, path: str, detail: str) -> ModReject:
    return ModReject(kind, path, detail)


# ---------------------------------------------------------------- 规则 3：保真表哈希闸护航

def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_fidelity_hashes(tables_dir: Path, baseline: Path) -> dict:
    """规则 3 护航：保真三表 sha256 == 闸基线（非荣誉制）。

    基线形：`{"tables": {"sprite_defs.json": <sha256>, ...}}`
    （gate `table-content-hash` 读同一文件——两处一份真相）。
    """
    if not baseline.exists():
        raise _reject("FidelityHashDrift", str(baseline),
                      "保真表哈希闸基线缺失（gate table-content-hash 基线件）")
    payload = json.loads(baseline.read_text(encoding="utf-8"))
    expected = payload.get("tables")
    if not isinstance(expected, dict) or not expected:
        raise _reject("FidelityHashDrift", str(baseline), "基线无 tables 条目")
    bad = []
    for name, want in sorted(expected.items()):
        p = Path(tables_dir) / name
        if not p.exists():
            bad.append(f"{name} 缺失")
        elif _sha256(p) != want:
            bad.append(f"{name} sha256 不匹配（期望 {want[:12]}…）")
    if bad:
        raise _reject("FidelityHashDrift", str(baseline),
                      "；".join(bad) + "——保真表变更=勘误流程+金样重过（非荣誉制）")
    return dict(expected)


class FidelityNamespace:
    """保真面 id 命名空间事实（只读投影；保真表真值宿主=defs.rs 解析器）。"""

    def __init__(self, type_ids, anim_ids, containers, container_counts,
                 assembly_ids, quirk_ids, hashes=None):
        self.type_ids = set(type_ids)
        self.anim_ids = set(anim_ids)
        self.containers = list(containers)
        self.container_counts = list(container_counts)
        self.assembly_ids = set(assembly_ids)
        self.quirk_ids = set(quirk_ids)
        self.hashes = hashes or {}

    def count_of(self, container: str):
        return self.container_counts[self.containers.index(container)]


def load_fidelity(tables_dir: Path, hash_baseline: Path | None = None) -> FidelityNamespace:
    """加载保真命名空间事实。给了 `hash_baseline` 就先过规则 3 护航（硬前置）。"""
    hashes = {}
    if hash_baseline is not None:
        hashes = verify_fidelity_hashes(tables_dir, hash_baseline)
    tables_dir = Path(tables_dir)
    sprite = json.loads((tables_dir / "sprite_defs.json").read_text(encoding="utf-8"))
    anim = json.loads((tables_dir / "anim_defs.json").read_text(encoding="utf-8"))
    sheet = json.loads((tables_dir / "sheet_index.json").read_text(encoding="utf-8"))
    type_ids = [r["type_id"] for r in sprite.get("sprite_defs", [])]
    quirk_ids = [q.get("quirk_id") for q in sprite.get("quirks_in_code", [])]
    anim_ids = [r["id"] for r in anim.get("anim_defs", [])]
    asm = sheet.get("assembly_profiles", {})
    return FidelityNamespace(
        type_ids=type_ids,
        anim_ids=anim_ids,
        containers=sheet.get("containers", []),
        container_counts=sheet.get("container_counts", []),
        assembly_ids=list(asm.keys()) if isinstance(asm, dict) else [],
        quirk_ids=[q for q in quirk_ids if q],
        hashes=hashes,
    )


# ---------------------------------------------------------------- 规则 4：mod 独立基线

class ModBaseline:
    """mod 独立基线（规则 4：不进金样闸）——mod 表哈希单独立账。

    基线形：`{"comment": ..., "mod_tables": {<文件名>: <sha256>}}`。
    保真哈希闸（规则 3）**不覆盖**本基线条目；本基线也**不得**记保真三表。
    """

    def __init__(self, path: Path):
        self.path = Path(path)

    def _read(self) -> dict:
        if not self.path.exists():
            return {"comment": "mod 独立基线（两份制规则 4）：mod 表哈希单独立账，不进金样闸。",
                    "mod_tables": {}}
        return json.loads(self.path.read_text(encoding="utf-8"))

    def update(self, mod_table_paths) -> dict:
        entries = {}
        for p in mod_table_paths:
            p = Path(p)
            entries[p.name] = _sha256(p)
        payload = self._read()
        payload["mod_tables"] = dict(sorted(entries.items()))
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
                             encoding="utf-8")
        return payload["mod_tables"]

    def verify(self, mod_table_paths) -> None:
        payload = self._read()
        entries = payload.get("mod_tables", {})
        bad = []
        for p in mod_table_paths:
            p = Path(p)
            want = entries.get(p.name)
            if want is None:
                bad.append(f"{p.name} 未入 mod 基线")
            elif _sha256(p) != want:
                bad.append(f"{p.name} sha256 不匹配")
        if bad:
            raise _reject("ModBaselineDrift", str(self.path), "；".join(bad))


# ---------------------------------------------------------------- 规则 2：金样运行禁 mod 表

def _is_mod_table(path: Path) -> bool:
    """mod 表标记 = 封套含 `mod_id`（两份制：保真表封套无此键）。"""
    try:
        doc = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    return isinstance(doc, dict) and "mod_id" in doc


def assert_golden_run_clean(table_paths, fidelity: FidelityNamespace | None = None) -> None:
    """规则 2 断言：金样运行清单只许含保真表——混入 mod 表 ⇒ AssertionError。"""
    for p in table_paths:
        p = Path(p)
        if _is_mod_table(p):
            raise AssertionError(
                f"金样运行禁 mod 表（两份制规则 2）：{p.name} 封套含 mod_id，"
                f"保真面运行=纯保真表（survey §2.2）")


# ---------------------------------------------------------------- mod 表加载通道（规则 1 + 五类镜像 + quirk 路由）

class ModBundle:
    def __init__(self, mod_id, sprite_rows, anim_rows, quirk_rows, resource_rows):
        self.mod_id = mod_id
        self.sprite_rows = sprite_rows
        self.anim_rows = anim_rows
        self.quirk_rows = quirk_rows
        self.resource_rows = resource_rows


def _check_keys(rec: dict, path: str, allowed) -> None:
    for k in rec:
        if k not in allowed:
            raise _reject("UnknownField", path, f"未知字段 {k!r}（防拼写静默）")


def _need(rec: dict, path: str, key: str, typ):
    if key not in rec:
        raise _reject("Schema", path, f"缺失 {key}")
    v = rec[key]
    if typ is int and not (isinstance(v, int) and not isinstance(v, bool)):
        raise _reject("Schema", f"{path}.{key}", "非整数")
    if typ is str and not isinstance(v, str):
        raise _reject("Schema", f"{path}.{key}", "非字符串")
    return v


def quirk_reason(row: dict) -> str | None:
    """quirk 型判定（§2.1 禁静默留码）：纯表规则装不下的形状 ⇒ 返回原因。

    判定形状与保真 quirks_in_code Q-001..Q-003 同族：
      - frame_h 为空 = 型高由运行时定（Q-001/Q-003 族，a.java:6318）；
      - type_frame_width 为空 = fallback 32×32 特例（Q-003，a.java:6322-6324）；
      - assembly_profile 非空 = SEG 片件+加载副作用（Q-002，a.java:6304-6307）。
    """
    if row.get("frame_h") is None:
        return "frame_h=null=运行时图高（Q-001/Q-003 族，a.java:6318）"
    if row.get("type_frame_width") is None:
        return "type_frame_width=null=fallback 32×32 特例（Q-003，a.java:6322-6324）"
    if row.get("assembly_profile"):
        return "assembly_profile=SEG 片件加载副作用（Q-002，a.java:6304-6307）"
    return None


def load_mod_table(path: Path, fidelity: FidelityNamespace) -> ModBundle:
    """mod 表加载通道：严格封套 + 规则 1 命名空间 + 外键/帧序域 + quirk 路由。

    保真命名空间由 [`load_fidelity`] 提供（给了 hash_baseline 即已过规则 3 护航）；
    本通道拒载语义与 defs.rs 五类对齐，quirk 型行必须走 Q-00x 登记路径。
    """
    path = Path(path)
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        raise _reject("BadJson", str(path), f"{e}") from e
    if not isinstance(doc, dict):
        raise _reject("Schema", str(path), "封套非对象")
    _check_keys(doc, str(path), MOD_ENVELOPE_KEYS)
    ver = doc.get("schema_version")
    if ver != SCHEMA_VERSION:
        raise _reject("SchemaVersion", f"{path}:schema_version",
                      f"found {ver!r}，期望 {SCHEMA_VERSION}")
    mod_id = _need(doc, str(path), "mod_id", str)
    sprite_rows = doc.get("sprite_defs", [])
    anim_rows = doc.get("anim_defs", [])
    quirk_rows = doc.get("quirks_in_code", [])
    resource_rows = doc.get("resource_rows", [])
    for name, bucket in (("sprite_defs", sprite_rows), ("anim_defs", anim_rows),
                         ("quirks_in_code", quirk_rows), ("resource_rows", resource_rows)):
        if not isinstance(bucket, list):
            raise _reject("Schema", f"{path}:{name}", "非数组")

    # ---- quirk 登记（§2.1 四字段留痕）----
    mod_quirk_ids = set()
    for i, q in enumerate(quirk_rows):
        p = f"{path}:quirks_in_code[{i}]"
        if not isinstance(q, dict):
            raise _reject("Schema", p, "非对象")
        _check_keys(q, p, MOD_QUIRK_KEYS)
        qid = _need(q, p, "quirk_id", str)
        if not _QUIRK_ID_RE.match(qid):
            raise _reject("Schema", f"{p}.quirk_id", f"quirk_id {qid!r} 形不符 Q-NNN")
        if qid in fidelity.quirk_ids or qid in mod_quirk_ids:
            raise _reject("DomainOverflow", f"{p}.quirk_id",
                          f"quirk_id {qid} 重复/覆盖保真 quirk（禁覆盖保真 id）")
        for f in MOD_QUIRK_KEYS:
            _need(q, p, f, str)
        mod_quirk_ids.add(qid)

    # ---- mod 动画档（规则 1：id 前缀命名空间）----
    mod_anim_ids = set()
    for i, a in enumerate(anim_rows):
        p = f"{path}:anim_defs[{i}]"
        if not isinstance(a, dict):
            raise _reject("Schema", p, "非对象")
        _check_keys(a, p, MOD_ANIM_KEYS)
        aid = _need(a, p, "id", str)
        if aid in fidelity.anim_ids:
            raise _reject("DomainOverflow", f"{p}.id",
                          f"mod 动画 id {aid!r} 禁覆盖保真 id（两份制规则 1）")
        if not aid.startswith(MOD_ANIM_PREFIX):
            raise _reject("DomainOverflow", f"{p}.id",
                          f"mod 动画 id {aid!r} 须前缀 {MOD_ANIM_PREFIX!r}（id 命名空间隔离）")
        if aid in mod_anim_ids:
            raise _reject("DomainOverflow", f"{p}.id", f"mod 动画 id {aid!r} 重复（id 稳定性）")
        frames = a.get("frames")
        if not isinstance(frames, list) or not all(
                isinstance(x, int) and not isinstance(x, bool) for x in frames):
            raise _reject("Schema", f"{p}.frames", "缺失或非整数数组")
        mod_anim_ids.add(aid)

    # ---- 资源行（纯表+资源行：域内=复用保真图集容器/槽位）----
    for i, r in enumerate(resource_rows):
        p = f"{path}:resource_rows[{i}]"
        if not isinstance(r, dict):
            raise _reject("Schema", p, "非对象")
        _check_keys(r, p, MOD_RESOURCE_KEYS)
        container = _need(r, p, "container", str)
        if container not in fidelity.containers:
            raise _reject("BadForeignKey", f"{p}.container",
                          f"container {container!r} 不在保真 containers"
                          f"（新容器=域外三界②，扩容器=改码）")
        slot = _need(r, p, "slot", int)
        table_len = fidelity.count_of(container)
        if slot < 0 or slot >= table_len:
            raise _reject("FrameIndexOutOfBounds", f"{p}.slot",
                          f"槽位 {slot} 超表长 {table_len}（域外三界③，扩张数=重锚）")

    # ---- mod 精灵行（规则 1 型号命名空间 + 外键 + quirk 路由）----
    seen_type_ids = set()
    for i, row in enumerate(sprite_rows):
        p = f"{path}:sprite_defs[{i}]"
        if not isinstance(row, dict):
            raise _reject("Schema", p, "非对象")
        _check_keys(row, p, MOD_SPRITE_KEYS)
        type_id = _need(row, p, "type_id", int)
        if type_id in fidelity.type_ids or type_id <= FIDELITY_TYPE_ID_MAX:
            raise _reject("DomainOverflow", f"{p}.type_id",
                          f"mod 型号 id {type_id} 禁覆盖保真 id（保真域 0..={FIDELITY_TYPE_ID_MAX} 冻结）")
        if type_id < MOD_TYPE_ID_MIN:
            raise _reject("DomainOverflow", f"{p}.type_id",
                          f"域外型 id {type_id}：既非保真域 0..={FIDELITY_TYPE_ID_MAX}"
                          f" 也非 mod 命名空间 ≥{MOD_TYPE_ID_MIN}（域外三界①，扩域=改码）")
        if type_id in seen_type_ids:
            raise _reject("DomainOverflow", f"{p}.type_id", f"mod 型号 id {type_id} 重复（id 稳定性）")
        seen_type_ids.add(type_id)
        anim_profile = _need(row, p, "anim_profile", str)
        known_anims = fidelity.anim_ids | mod_anim_ids
        if anim_profile not in known_anims:
            raise _reject("BadForeignKey", f"{p}.anim_profile",
                          f"anim_profile {anim_profile!r} 不存在（孤儿拒载）")
        sheet = row.get("sheet")
        if sheet is not None:
            if not isinstance(sheet, str):
                raise _reject("Schema", f"{p}.sheet", "非字符串/非 null")
            if sheet not in fidelity.containers:
                raise _reject("BadForeignKey", f"{p}.sheet",
                              f"sheet {sheet!r} 不在 sheet_index.containers（跨表悬空拒载）")
        slot = row.get("sheet_index")
        if slot is not None:
            if not isinstance(slot, int) or isinstance(slot, bool):
                raise _reject("Schema", f"{p}.sheet_index", "非整数/非 null")
            if sheet is None:
                raise _reject("Schema", f"{p}.sheet_index", "sheet_index 与 sheet 须同生共死")
            table_len = fidelity.count_of(sheet)
            if slot < 0 or slot >= table_len:
                raise _reject("FrameIndexOutOfBounds", f"{p}.sheet_index",
                              f"槽位 {slot} 超表长 {table_len}")
        asm = row.get("assembly_profile")
        if asm is not None and asm not in fidelity.assembly_ids:
            raise _reject("BadForeignKey", f"{p}.assembly_profile",
                          f"assembly_profile {asm!r} 不存在（跨表悬空拒载）")
        for key in ("floor_category_override", "transition_overrides"):
            if key in row and not isinstance(row[key], list):
                raise _reject("Schema", f"{p}.{key}", "非数组")
        # quirk 路由（§2.1 禁静默留码）：quirk 型必须走 Q-00x 登记路径
        reason = quirk_reason(row)
        if reason is not None:
            qref = row.get("quirk_ref")
            if qref is None:
                raise _reject("QuirkPathRequired", f"{p}.quirk_ref",
                              f"quirk 型走 Q-00x 路径登记（禁静默留码）：{reason}")
            if qref not in fidelity.quirk_ids and qref not in mod_quirk_ids:
                raise _reject("BadForeignKey", f"{p}.quirk_ref",
                              f"quirk_ref {qref!r} 未登记（quirks_in_code 缺席）")
        elif row.get("quirk_ref") is not None:
            raise _reject("Schema", f"{p}.quirk_ref", "非 quirk 型行不得带 quirk_ref（防伪登记）")

    return ModBundle(mod_id, sprite_rows, anim_rows, quirk_rows, resource_rows)
