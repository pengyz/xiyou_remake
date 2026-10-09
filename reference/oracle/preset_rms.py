#!/usr/bin/env python3
"""preset_rms.py — RMS 存档预置生成器（绕过序章直达任意层）。

格式依据（A 级证据，deobf a.java）：
- MOT_L{n}（m_114 写 / m_115 读，逐字段顺序一致）：
  byte f_byte_26(难度), byte currentFloor, byte minFloorReached, byte maxFloorReached,
  byte f_byte_12(朝向), byte equippedWeaponType, byte equippedArmorType,
  short playerCellX, short playerCellY,
  int playerHp, int playerAtk, int playerDef,
  short yellowKeyCount, short blueKeyCount, short redKeyCount,
  int goldAmount, short alchemyUpgradeCount, short f_int_89(背包种类数),
  f_int_89 × (byte 类型, byte 次数),
  128 × boolean（f_bool_arr_06，脚本已触发标记）,
  byte itemStackSize, itemStackSize × (byte 类型, byte 次数),
  56 × (short 长度 + 长度×byte)（f_byte_arr2_05，各层脚本缓存）
- SKY_WAR（m_110 写 / run 内 case21 读）：4×boolean + byte N + N×int + 3×int。

shim RecordStore 的读取入口（Runner 启动时）见 --preset 接线：预置数据为
`<store>@<recordId>=<hex>` 行式文本，经 -Doracle.preset=<file> 注入。
"""
from __future__ import annotations

import struct
import sys


def mot_l(*, difficulty=0, current_floor=1, min_floor=1, max_floor=1, facing=0,
          weapon=0, armor=0, cell_x=3, cell_y=10, hp=500, atk=30, deff=30,
          yellow=1, blue=0, red=0, gold=100, alchemy=0,
          items=(), script_seen=(), layer_caches=(), f_int_113=128,
          f_byte_26_init=None) -> bytes:
    out = bytearray()
    out += struct.pack('>b', f_byte_26_init if f_byte_26_init is not None else difficulty)
    out += struct.pack('>b', current_floor)
    out += struct.pack('>b', min_floor)
    out += struct.pack('>b', max_floor)
    out += struct.pack('>b', facing)
    out += struct.pack('>b', weapon)
    out += struct.pack('>b', armor)
    out += struct.pack('>h', cell_x)
    out += struct.pack('>h', cell_y)
    out += struct.pack('>i', hp)
    out += struct.pack('>i', atk)
    out += struct.pack('>i', deff)
    out += struct.pack('>h', yellow)
    out += struct.pack('>h', blue)
    out += struct.pack('>h', red)
    out += struct.pack('>i', gold)
    out += struct.pack('>h', alchemy)
    out += struct.pack('>h', len(items))
    for t, n in items:
        out += struct.pack('>bb', t, n)
    seen = set(script_seen)
    for i in range(128):
        out += struct.pack('>B', 1 if i in seen else 0)
    out += struct.pack('>b', 0)  # itemStackSize 由 items 数量决定？——不：物品栈在上方 f_int_89 段，
    # m_114 里 itemStackSize 段是独立的（栈内物品 = 类型+次数）——这里保持 0（背包为空走最简路径）
    # 修正：上面 f_int_89 段即背包；itemStackSize 段写 0
    for i in range(56):
        cache = layer_caches.get(i) if isinstance(layer_caches, dict) else None
        if cache:
            out += struct.pack('>h', len(cache))
            out += cache
        else:
            out += struct.pack('>h', 0)
    return bytes(out)


def mot_if(*, slots=()) -> bytes:
    """6 槽位表（m_111 读）：每槽 1B 有效位；有效槽再写 1B 层号 + 2B 装备 + 7×int 数值。
    slots = [(slot_index, floor, weapon, armor, hp, atk, def, i6, i7, i8, i9)]。"""
    out = bytearray()
    valid = {s[0]: s for s in slots}
    for i in range(6):
        if i in valid:
            s = valid[i]
            out += struct.pack('>b', 1)
            out += struct.pack('>b', s[1])   # f_byte_arr_24[i] = 层号
            out += struct.pack('>b', s[2])   # f_byte_arr_25[i]
            out += struct.pack('>b', s[3])   # f_byte_arr_26[i]
            for v in s[4:11]:
                out += struct.pack('>i', v)
        else:
            out += struct.pack('>b', 0)
    return bytes(out)


def sky_war(*, flags=(False, True, False, False), floors=(4, 2, 2, 2, 2, 2, 2),
            max_reached=1, min_reached=1, current=1) -> bytes:
    out = bytearray()
    for f in flags:
        out += struct.pack('>B', 1 if f else 0)
    out += struct.pack('>b', len(floors))
    for v in floors:
        out += struct.pack('>i', v)
    out += struct.pack('>i', max_reached)
    out += struct.pack('>i', min_reached)
    out += struct.pack('>i', current)
    return bytes(out)


def preset_text(stores: dict) -> str:
    """stores = {name: {recordId: bytes}} → 行式 `<name>@<id>=<hexlower>`。"""
    lines = []
    for name in sorted(stores):
        for rid in sorted(stores[name]):
            lines.append(f"{name}@{rid}={stores[name][rid].hex()}")
    return "\n".join(lines) + "\n"


if __name__ == '__main__':
    # f_int_134（存档位）默认 0 ⇒ 预置 MOT_L0（m_115 读 "MOT_L"+f_int_134）
    stores = {
        "MOT_L0": {1: mot_l(current_floor=1, min_floor=1, max_floor=1,
                            cell_x=3, cell_y=10, hp=500, atk=30, deff=30,
                            yellow=1, gold=100)},
        # MOT_IF：槽位有效性表（f_bool_arr_07 来源，m_111 启动时读）——
        # 槽 0 有效（层 1、HP 500/ATK 30/DEF 30 与 MOT_L0 一致）
        "MOT_IF": {1: mot_if(slots=[(0, 1, 0, 0, 500, 30, 30, 0, 0, 0, 0)])},
        "SKY_WAR": {1: sky_war(max_reached=1, min_reached=1, current=1)},
    }
    sys.stdout.write(preset_text(stores))
