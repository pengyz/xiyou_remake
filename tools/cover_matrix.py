#!/usr/bin/env python3
"""cover_matrix.py — 覆盖矩阵生成器（层次一工具，收敛计划 P0，迭代 72）。

从改名类（analysis/rename-pipeline/renamed/a.class）提取**全方法清单**，
按子系统标注三色状态，产出 docs/coverage-matrix.md：

  绿 = 已移植 + 场景全帧已验（列出覆盖场景）
  黄 = 已移植 + 场景未验（或验证范围有已知缺口）
  白 = 无移植账（未移植/待考）

v1 语义：覆盖标注为**子系统注解表**（人工维护于本脚本 COVERAGE，数据源
= completeness.md 各层收敛记录）。v2 计划：字节码调用图插桩做真覆盖。
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RENAMED = ROOT / "analysis/rename-pipeline/renamed/a.class"
OUT = ROOT / "docs" / "coverage-matrix.md"

# 子系统 → (状态, 覆盖场景, 方法名清单)。状态：green/yellow/white。
# 方法名以改名类 javap 名为准（精确匹配；前缀 * 表示 startswith）。
COVERAGE: dict[str, tuple[str, str, list[str]]] = {
    "boot/载入/存读档": ("green", "boot_menu、prologue、preset_save、preset_load_menu、floor_crossing（T1-9800/全帧）",
        ["m_000", "m_001", "m_006", "m_091", "m_093", "m_117", "m_118", "loadSlotTable",
         "loadFloorSave", "saveGlobalProgress", "m_119", "m_122", "m_059", "m_068",
         "loadProgressTarget", "bootLoadPhase", "loadStepIndex", "loadStepCount",
         "loadTargetMode", "loadCallInit", "slotCursor"]),
    "行走/相机": ("green", "全部已铺层（wp1 8px 步进、边缘跟随、m_025 帧循环）",
        ["tryStep", "handleFieldInput", "m_055", "m_025", "isCellWalkable", "m_062",
         "rebuildWalkability", "findPath", "m_134", "m_103", "easeAfterimages",
         "lockCameraOn", "setCameraClamped", "stepCameraTowardTarget", "cameraAtTarget",
         "walkPhase", "walkStepCount", "walkPathBuffer", "stepProgressPx"]),
    "交互/门/钥匙": ("green", "floor1/2/10/35/42/49（黄红蓝门、封印门81/4、83 剧情点）",
        ["interactWithCell", "m_100", "m_101", "removeSlotAt", "consumeKeyForDoor",
         "spawnPopup", "pullEntityFromCell", "removeGroupAtCellByType", "m_062",
         "dispatchSpriteSize", "dispatchAnimFields", "spawnEntity", "markEntityRemoved",
         "markEntityDesRemoved", "m_107", "m_108", "predictHpLossVsType"]),
    "战斗/赏金": ("green", "floor35/40/42/43/49（一拍斩/多拍、boss 钩子 69/70/72/73/75）",
        ["tickBattle", "spawnBossEvent", "predictHpLossVsType", "scaleEnemyStats",
         "enemyAtkScaled", "enemyHpScaled", "gainGold", "enemyBaseGold",
         "battleSceneRerun", "battleFinishing", "battleAnimTick", "removeLogoItem"]),
    "脚本 DSL": ("green", "floor35/42/46/49/10（TAK/SEE/MOV/DES/SWD/CES/GUT/ROS/RES/GLV/END/SMS 实战）",
        ["executeScriptInstruction", "tryRunScene", "loadLevelScript", "openDialogPage",
         "flipOverlayPage", "layoutWrappedText", "layoutFullText", "layoutWrappedTextLimited",
         "skipTextToLineEnd", "advanceScriptWalk", "scriptLineFlags", "dialogScriptLine",
         "parseScriptInt", "scriptCursor", "dialogPhase", "scriptStepDelay"]),
    "绘制/状态栏/HUD": ("green", "全部已铺层（paint case 0/2/3/7/9/10/11/12/15/16/18/20/21）",
        ["paintEntityLayer", "paintTileLayer", "drawParallaxBackdrop", "paintPlayerSprite",
         "drawIconStrip", "paintMiniFrame", "paintNumber", "paintHud", "paintKeySlot",
         "paintSoftkeyBar", "drawEdgePatch", "buildMinimap", "m_003", "m_004", "m_142",
         "updateAndDrawParticles", "spawnParticle", "addSpark", "drawString"]),
    "道具/装备/炼丹": ("green", "floor1/5/10/12/43（13-25 道具、26-28 钥匙、29-32 药、79/80 装备、85/86）",
        ["pickupItemType", "tryEquip", "m_083", "equipTierTypes", "equipTierBonuses",
         "addItemToItemStack", "itemTypeToStackIndex", "findItemStackIndex", "useItemStack",
         "m_075", "m_076", "alchemyPrice", "alchemyUpgradeCount", "m_082"]),
    "mode/菜单族": ("green", "boot_menu、preset_help、preset_save、floor51_menu、preset_item13、merchant（mode 2/4/7/8/10/12/15/16/17/18/19/20/21）",
        ["m_092", "m_093", "m_094", "m_095", "m_096", "m_097", "m_098", "m_099", "m_102",
         "menuCursorIndex", "menuItemKinds", "optionChecked", "m_091"]),
    "已知挂起（黄）": ("yellow", "floor46 白烟纹理帧序、floor43 状态栏数字/盒尾、dp2 实体缓动、m_102 跨层快照",
        ["m_106", "m_107"]),
}

# 已知白项（未移植/待考，显式列出——来自 completeness 挂起队列）
KNOWN_WHITE = [
    "m_131",   # loadTargetMode/载入目标态（mode 14/15 路径）
]


def methods() -> list[str]:
    out = subprocess.run(["javap", "-p", str(RENAMED)], capture_output=True, text=True)
    if out.returncode != 0:
        print(f"javap 失败: {out.stderr}", file=sys.stderr)
        sys.exit(2)
    names = []
    for line in out.stdout.splitlines():
        l = line.strip()
        if "(" not in l or l.startswith(("Compiled from", "public final class", "class ")):
            continue
        names.append(l.split()[-1].split("(")[0])
    return names


def classify(name: str) -> tuple[str, str]:
    for subsystem, (status, scenes, names) in COVERAGE.items():
        for n in names:
            if n == name or (n.startswith("*") and name.startswith(n[1:])):
                return status, subsystem
    return "white", "未入账"


def main() -> int:
    inv = methods()
    # 同名重载去重保留一份
    uniq = sorted(set(inv))
    rows: dict[str, list[tuple[str, str]]] = {"green": [], "yellow": [], "white": []}
    for m in uniq:
        st, sub = classify(m)
        rows[st].append((m, sub))
    counts = {k: len(v) for k, v in rows.items()}
    total = len(uniq)

    lines = [
        "# 覆盖矩阵（层次一账本，机器生成）",
        "",
        f"> 生成：`python3 tools/cover_matrix.py`（迭代 72 起）。方法清单 = 改名类",
        f"> javap 全量（{total} 方法，含构造/重载去重）。三色：绿=已移植+场景已验、",
        "> 黄=已移植+场景未验、白=无移植账。**Stage A 判据：白项清零、黄项带论证**。",
        "",
        f"## 汇总：绿 {counts['green']} / 黄 {counts['yellow']} / 白 {counts['white']}（共 {total}）",
        "",
        "> v1 粒度 = 子系统注解（人工维护于 cover_matrix.py COVERAGE 表，数据源",
        "> completeness.md）。v2 = 字节码调用图插桩真覆盖。differ 层（状态向量）",
        "> 见 tools/statediff.py。",
        "",
    ]
    for st, title in (("green", "绿：已移植+场景已验"), ("yellow", "黄：已移植+未验证/挂起"), ("white", "白：无移植账")):
        lines.append(f"## {title}（{counts[st]}）\n")
        cur_sub = None
        for m, sub in sorted(rows[st], key=lambda x: (x[1], x[0])):
            if sub != cur_sub:
                lines.append(f"### {sub}")
                cur_sub = sub
            lines.append(f"- {m}")
        lines.append("")
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"覆盖矩阵 → {OUT.relative_to(ROOT)}（绿 {counts['green']} / 黄 {counts['yellow']} / 白 {counts['white']}）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
