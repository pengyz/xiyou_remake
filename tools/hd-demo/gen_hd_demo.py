#!/usr/bin/env python3
"""gen_hd_demo — 弃件② 3x demo 图集生成器（高清素材探针 HD-asset probe，2026-10-21）。

设计件：docs/superpowers/specs/2026-10-21-hd-asset-probe-design.md（R-4 图集切片、R-7 弃件隔离）。
**弃件纪律**：demo 图集只证管线（帧序/帧数/列数/密度换算），画质不作证据（C 级）；
产物固定 `assets/hd-demo/`（gitignored），不入 original/、不入保真表、不进金样运行。

原图集来源（探明结论，A 级）：
  - `assets/raw/map` = packed_png 容器（N×[u16 LE 长度][PNG]，帧序 map_00..map_11，共 12 帧；
    张数常量表 `crates/game-data/src/packed_png.rs` RESOURCE_IMAGE_COUNTS["map"]=12）。
  - 帧 00（128×208）= **地图图块图集 tileset**（paint_tiles 消费，game_view.rs:226-233
    `sx=(v&7)<<4`、`sy=(v>>3)<<4`、16×16 切片；asset-hardcode-audit 陷阱①：`&7`/`>>3` 的 8 =
    图集列数索引语义，legacy 值断言不动）。
  - 帧切分：帧 = 16×16 tile 槽位，列数 8，行数 13（208/16），帧数 104；
    帧序 = tile id 升序 = 行优先（id&7=列、id>>3=行，maplv::tile_atlas_col/row 同式）。

demo 图集（输出）：
  - `assets/hd-demo/map_00_3x.png`：384×624 = 源 128×208 ×3（density d=3，帧 48×48 = 16×3）。
  - `assets/hd-demo/frames/tile_XXX.png`：104 个 48×48 帧序 PNG 清单
    （与 packed_png 中间态"帧序 PNG 清单"格式对齐，asset_pipeline/mod.rs 头部）。
  - `tools/hd-demo/frame-hashes.txt`：帧哈希清单（入库；生成器产出，供确定性/漂移复核）。

混合质量（量产"分批换素材、混密度并存"预演）：
  - **restyle**（程序化精绘）：maplv0-54（55 关）实际 tile 用量 top16 高频帧
    （草/石/墙/水等常见地图图块）——调色板压平 + 3x 硬边描线 + 类别纹理，
    纯像素画风格（无抗锯齿、无渐变、无随机源）。
  - **nn**（最近邻放大垫底）：其余 88 帧逐像素 ×3。

硬断言（生成期即跑，失败即崩）：
  1. 容器帧数 12、帧 00 尺寸 128×208（源锚）；
  2. 列数 legacy 断言：src_width//16 == 8（陷阱① 索引语义）；
  3. R-4 宽度型断言换算：demo_width//(16*3) == 8；
  4. demo 尺寸 == 源 ×3，帧网格一一对应（8×13 = 104 帧）；
  5. 帧序恒等：NN 帧 = 源帧 NN×3 逐字节（帧不串位）；restyle 帧保留同槽位 +
     alpha 掩码 == 源帧 alpha NN×3；manifest 行序 = tile id 升序、(id&7,id>>3)==(col,row)；
  6. maplv 全部 tile id 可被图集寻址（id>>3 < 13，即 id ≤ 103）；
  7. 高频帧选取 == 频次排序 top16（EXPECTED_HAND_IDS 钉死，源漂移即红）。

用法：
  python3 tools/hd-demo/gen_hd_demo.py           # 生成 assets/hd-demo/ + frame-hashes.txt
  python3 tools/hd-demo/gen_hd_demo.py --check   # 复核在盘产物与清单一致（不写盘）

确定性：全部整数运算、无随机源（禁 PYTHONHASHSEED 敏感的 hash()）、PNG 编码参数固定；
同输入 ⇒ 同输出字节（跑两次 sha256 相同是验收判据之一）。
"""
from __future__ import annotations

import argparse
import hashlib
import struct
import sys
from collections import Counter
from pathlib import Path

from PIL import Image

# ---------------------------------------------------------------- 常量（legacy 锚）
ROOT = Path(__file__).resolve().parent.parent.parent
RAW_MAP = ROOT / "assets" / "raw" / "map"
RAW_DIR = ROOT / "assets" / "raw"
OUT_DIR = ROOT / "assets" / "hd-demo"
FRAMES_DIR = OUT_DIR / "frames"
HASHES_TXT = Path(__file__).resolve().parent / "frame-hashes.txt"

TILE_PX = 16            # 帧像素（图集切片，maplv::tile_atlas_* ×16）
DENSITY = 3             # demo 密度 d=3
ATLAS_COLS = 8          # 陷阱① 索引语义 legacy 值（改读 JSON 亦须断言 8）
CONTAINER_FRAMES = 12   # RESOURCE_IMAGE_COUNTS["map"]
MAPLV_LEVELS = 55       # maplv0..maplv54
HAND_RANK_LIMIT = 16    # 高频精绘帧数
# 频次 top16（键 = (-用量, tile_id)，A 级：assets/raw/maplv0-54 实测，覆盖 51.87% 格）
EXPECTED_HAND_IDS = (1, 22, 17, 10, 24, 46, 47, 2, 30, 15, 9, 45, 49, 38, 39, 44)

PNG_MAGIC = b"\x89PNG\r\n\x1a\n"


# ---------------------------------------------------------------- packed_png 容器读取
def read_map_container(path: Path) -> list[bytes]:
    """N × [u16 LE 长度][PNG 字节] → 帧序字节清单（逐字节保留策略，对齐 asset_pipeline 中间态）。"""
    data = path.read_bytes()
    frames: list[bytes] = []
    pos = 0
    while pos < len(data):
        if pos + 2 > len(data):
            raise AssertionError(f"{path.name}: 帧 {len(frames)} 长度前缀截断 @ {pos}")
        ln = struct.unpack_from("<H", data, pos)[0]
        pos += 2
        png = data[pos : pos + ln]
        assert len(png) == ln, f"{path.name}: 帧 {len(frames)} 数据截断"
        assert png[:8] == PNG_MAGIC, f"{path.name}: 帧 {len(frames)} 非 PNG 魔数 @ {pos - 2}"
        frames.append(png)
        pos += ln
    return frames


# ---------------------------------------------------------------- maplv tile 用量
def maplv_tile_counts() -> Counter:
    """maplv0-54（55 关）terrain 网格 tile id 用量（v>0；v==0 = 空格，paint_tiles 跳过）。"""
    files = sorted(RAW_DIR.glob("maplv*"), key=lambda p: int(p.name[len("maplv") :]))
    assert len(files) == MAPLV_LEVELS, f"maplv 关数漂移: {len(files)} != {MAPLV_LEVELS}"
    counts: Counter = Counter()
    for p in files:
        data = p.read_bytes()
        y = struct.unpack_from("<H", data, 0)[0] >> 1
        z = struct.unpack_from("<H", data, 2)[0] >> 1
        grid = y * z * 4
        assert len(data) == 4 + 2 * grid, f"{p.name}: 尺寸与网格不符（{len(data)} != {4 + 2 * grid}）"
        counts.update(v for v in data[4 : 4 + grid] if v > 0)
    return counts


# ---------------------------------------------------------------- 3x 精绘（像素画）
def _lum(c: tuple[int, int, int]) -> int:
    return (c[0] * 299 + c[1] * 587 + c[2] * 114) // 1000


def _hash2(x: int, y: int, seed: int) -> int:
    """纯整数确定性哈希（禁内置 hash()：PYTHONHASHSEED 不稳）。"""
    h = (x * 0x9E3779B1) ^ (y * 0x85EBCA6B) ^ (seed * 0xC2B2AE35)
    return (h ^ (h >> 16)) & 0xFFFFFFFF


def _classify(colors: list[tuple[int, int, int]]) -> str:
    """C 级启发式类别（草/石/墙/水等）——只影响纹理笔触，不作任何证据。"""
    best = max(colors, key=lambda c: (max(c) - min(c), _lum(c)))
    sat = max(best) - min(best)
    if sat < 30:
        return "stone" if _lum(best) >= 90 else "stone"
    r, g, b = best
    if b > g and b - g >= 15:
        return "water"
    if g >= r and g - b >= 10:
        return "grass"
    if r >= g:
        return "wall"
    return "stone"


def restyle_tile(src: list[tuple[int, int, int, int]], tile_id: int) -> list[tuple[int, int, int, int]]:
    """16×16 RGBA → 48×48 RGBA：调色板压平 + 硬边描线 + 类别纹理（无抗锯齿、无渐变）。"""
    px = TILE_PX * DENSITY
    opaque = [p[:3] for p in src if p[3] > 0]
    if not opaque:
        return [(0, 0, 0, 0)] * (px * px)
    cnt = Counter(opaque)
    palette = sorted(cnt, key=lambda c: (-cnt[c], c))[:4]
    dark = min(palette, key=lambda c: (_lum(c), c))
    light = max(palette, key=lambda c: (_lum(c), c))
    kind = _classify(palette)

    def nearest(c: tuple[int, int, int]) -> tuple[int, int, int]:
        return min(palette, key=lambda k: ((k[0] - c[0]) ** 2 + (k[1] - c[1]) ** 2 + (k[2] - c[2]) ** 2, k))

    # 逻辑像素 → 主色（None = 透明，透明掩码 == 源 alpha NN×3）
    flat: list[tuple[int, int, int] | None] = [nearest(p[:3]) if p[3] > 0 else None for p in src]
    out: list[tuple[int, int, int, int]] = [(0, 0, 0, 0)] * (px * px)
    for ly in range(TILE_PX):
        for lx in range(TILE_PX):
            c = flat[ly * TILE_PX + lx]
            if c is None:
                continue
            for dy in range(DENSITY):
                for dx in range(DENSITY):
                    out[(ly * DENSITY + dy) * px + lx * DENSITY + dx] = (c[0], c[1], c[2], 255)

    def set_px(x: int, y: int, c: tuple[int, int, int]) -> None:
        if 0 <= x < px and 0 <= y < px and out[y * px + x][3] > 0:
            out[y * px + x] = (c[0], c[1], c[2], 255)

    def at(lx: int, ly: int) -> tuple[int, int, int] | None:
        if 0 <= lx < TILE_PX and 0 <= ly < TILE_PX:
            return flat[ly * TILE_PX + lx]
        return None

    # 硬边描线：相邻不透明异色 ⇒ 本块右列/下行 1 物理 px 暗色轮廓（不越透明边界）
    for ly in range(TILE_PX):
        for lx in range(TILE_PX):
            c = at(lx, ly)
            if c is None or c == dark:
                continue
            right, down = at(lx + 1, ly), at(lx, ly + 1)
            x0, y0 = lx * DENSITY, ly * DENSITY
            if right is not None and right != c:
                for dy in range(DENSITY):
                    set_px(x0 + DENSITY - 1, y0 + dy, dark)
            if down is not None and down != c:
                for dx in range(DENSITY):
                    set_px(x0 + dx, y0 + DENSITY - 1, dark)

    # 类别纹理（整数哈希噪点/条纹，仅用主色板颜色，硬边）
    seed = tile_id
    for y in range(px):
        for x in range(px):
            h = _hash2(x, y, seed)
            if out[y * px + x][3] == 0:
                continue
            if kind == "water" and (y + (x // 6)) % 9 == 0 and h % 5 < 2:
                set_px(x, y, light)
            elif kind == "grass" and x % 7 == h % 7 and (y + h // 8) % 11 < 3:
                set_px(x, y, light if h % 3 else dark)
            elif kind == "wall" and (y % 12 == 11 or (x + (y // 12) * 6) % 12 == 11):
                set_px(x, y, dark)
            elif kind == "stone" and h % 23 == 0:
                set_px(x, y, light if h % 2 else dark)
            elif kind == "wall" and h % 29 == 0:
                set_px(x, y, light)
    return out


def nn3_tile(src: list[tuple[int, int, int, int]]) -> list[tuple[int, int, int, int]]:
    """16×16 RGBA → 48×48 RGBA：最近邻 ×3（垫底帧）。"""
    px = TILE_PX * DENSITY
    out: list[tuple[int, int, int, int]] = [(0, 0, 0, 0)] * (px * px)
    for ly in range(TILE_PX):
        for lx in range(TILE_PX):
            c = src[ly * TILE_PX + lx]
            for dy in range(DENSITY):
                for dx in range(DENSITY):
                    out[(ly * DENSITY + dy) * px + lx * DENSITY + dx] = c
    return out


# ---------------------------------------------------------------- 生成 / 校验
def load_source():
    frames = read_map_container(RAW_MAP)
    assert len(frames) == CONTAINER_FRAMES, f"容器帧数漂移: {len(frames)} != {CONTAINER_FRAMES}"
    import io

    # 帧 00 PNG 字节（= tileset；容器是 N×[u16 长度][PNG]，非裸 PNG，须走帧序解包）
    sheet = Image.open(io.BytesIO(frames[0])).convert("RGBA")
    w, h = sheet.size
    assert (w, h) == (128, 208), f"源图集尺寸漂移: {(w, h)} != (128, 208)"
    assert w // TILE_PX == ATLAS_COLS, f"列数 legacy 断言失败: {w // TILE_PX} != {ATLAS_COLS}（陷阱①）"
    cols, rows = w // TILE_PX, h // TILE_PX
    assert cols * rows == 104, f"帧数漂移: {cols * rows} != 104"
    tiles = []
    for row in range(rows):
        for col in range(cols):
            t = sheet.crop((col * TILE_PX, row * TILE_PX, (col + 1) * TILE_PX, (row + 1) * TILE_PX))
            tiles.append(list(t.getdata()))
    return tiles, cols, rows


def build():
    tiles, cols, rows = load_source()
    counts = maplv_tile_counts()
    assert max(counts) <= cols * rows - 1, f"maplv tile id 越出图集寻址: max={max(counts)}"
    hand = [t for t, _ in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))[:HAND_RANK_LIMIT]]
    assert tuple(hand) == EXPECTED_HAND_IDS, f"高频帧选取漂移: {hand} != {list(EXPECTED_HAND_IDS)}"
    hand_set = set(hand)

    frames_out: list[list[tuple[int, int, int, int]]] = []
    modes: list[str] = []
    for tid in range(cols * rows):
        src = tiles[tid]
        if tid in hand_set:
            frames_out.append(restyle_tile(src, tid))
            modes.append("restyle")
        else:
            frames_out.append(nn3_tile(src))
            modes.append("nn")
    return tiles, frames_out, modes, counts, cols, rows, hand_set


def raw_sha(px: list[tuple[int, int, int, int]]) -> str:
    return hashlib.sha256(b"".join(bytes(p) for p in px)).hexdigest()


def compose_sheet(frames_out, cols, rows):
    px = TILE_PX * DENSITY
    img = Image.new("RGBA", (cols * px, rows * px))
    for tid, fr in enumerate(frames_out):
        col, row = tid & 7, tid >> 3
        assert col < cols and row < rows, f"帧 {tid} 槽位越界"
        tile = Image.new("RGBA", (px, px))
        tile.putdata(fr)
        img.paste(tile, (col * px, row * px))
    return img


def manifest_text(tiles, frames_out, modes, counts, cols, rows, sheet_img, src_frame0_sha) -> str:
    px = TILE_PX * DENSITY
    lines = [
        "# hd-demo 3x 图集帧哈希清单（弃件②：高清素材探针 demo 图集——只证管线，画质不作证据）",
        "# 生成器: tools/hd-demo/gen_hd_demo.py（确定性输出=同输入同哈希，无随机源）",
        "# 源图集: assets/raw/map 容器帧 00（packed_png 帧序 map_00 = tileset，帧=16×16 tile）",
        "#   源尺寸 128×208；列数 8（asset-hardcode-audit 陷阱① 索引语义，legacy 断言不变）；",
        "#   帧数 104 = 8×13；帧序 = tile id 升序（id&7=列、id>>3=行，maplv::tile_atlas_col/row 同式）",
        f"#   源帧 00 PNG sha256 = {src_frame0_sha}",
        f"# demo 图集: assets/hd-demo/map_00_3x.png（{cols * px}×{rows * px} = 源×3，帧 {px}×{px} = 16×3）",
        "#   帧序 PNG 清单: assets/hd-demo/frames/tile_XXX.png（与 packed_png 中间态帧序清单对齐）",
        "# mode: restyle=程序化精绘（maplv0-54 用量 top16 高频 tile，像素硬边）| nn=最近邻放大垫底",
        "# sha256 = 帧 48×48 RGBA 原始像素字节（与 PNG 编码器无关）",
        "# use_55 = maplv0-54（55 关）terrain 总用量（v>0；v==0 空格不计）",
        "# columns: frame_id row col mode use_55 sha256_rgba48",
    ]
    for tid, fr in enumerate(frames_out):
        row, col = tid >> 3, tid & 7
        lines.append(
            f"{tid:03d} {row} {col} {modes[tid]} {counts.get(tid, 0)} {raw_sha(fr)}"
        )
    sheet_raw = raw_sha(list(sheet_img.getdata()))
    sheet_png_sha = hashlib.sha256(sheet_img_to_png_bytes(sheet_img)).hexdigest()
    lines.append(
        f"# sheet map_00_3x.png {cols * px}x{rows * px} raw_rgba_sha256={sheet_raw} png_sha256={sheet_png_sha}"
    )
    # 恒等锚（NN 帧逐字节 == 源帧 NN×3；帧序恒等的生成期证明）
    nn_ok = sum(
        1
        for tid, fr in enumerate(frames_out)
        if modes[tid] == "nn" and fr == nn3_tile(tiles[tid])
    )
    lines.append(f"# nn_byte_identity frames={nn_ok} (= nn 模式帧数，逐帧 = 源帧 NN×3)")
    return "\n".join(lines) + "\n"


PNG_SAVE_KW = dict(format="PNG", optimize=False, compress_level=6)


def sheet_img_to_png_bytes(img: Image.Image) -> bytes:
    import io

    buf = io.BytesIO()
    img.save(buf, **PNG_SAVE_KW)
    return buf.getvalue()


def generate() -> None:
    tiles, frames_out, modes, counts, cols, rows, hand_set = build()
    px = TILE_PX * DENSITY
    # 硬断言 3：R-4 宽度型断言换算 width/(16*d) == 8
    demo_w, demo_h = cols * px, rows * px
    assert demo_w // (TILE_PX * DENSITY) == ATLAS_COLS, "R-4 列数断言失败"
    assert (demo_w, demo_h) == (128 * DENSITY, 208 * DENSITY), "demo 尺寸 != 源 ×3"

    # 硬断言 5：帧序恒等（NN 帧 == 源帧 NN×3 逐字节；restyle 帧 alpha 掩码 == 源 alpha NN×3）
    for tid, fr in enumerate(frames_out):
        src = tiles[tid]
        if modes[tid] == "nn":
            assert fr == nn3_tile(src), f"NN 帧 {tid} 与源帧 NN×3 不恒等（帧串位？）"
        else:
            exp_mask = [p[3] > 0 for p in nn3_tile(src)]
            got_mask = [p[3] > 0 for p in fr]
            assert got_mask == exp_mask, f"restyle 帧 {tid} alpha 掩码漂移"

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    FRAMES_DIR.mkdir(parents=True, exist_ok=True)
    sheet = compose_sheet(frames_out, cols, rows)
    sheet.save(OUT_DIR / "map_00_3x.png", **PNG_SAVE_KW)
    for tid, fr in enumerate(frames_out):
        tile = Image.new("RGBA", (px, px))
        tile.putdata(fr)
        tile.save(FRAMES_DIR / f"tile_{tid:03d}.png", **PNG_SAVE_KW)
    import io

    src_frame0 = read_map_container(RAW_MAP)[0]
    src_frame0_sha = hashlib.sha256(src_frame0).hexdigest()
    HASHES_TXT.write_text(
        manifest_text(tiles, frames_out, modes, counts, cols, rows, sheet, src_frame0_sha),
        encoding="utf-8",
    )
    print(
        f"hd-demo: 帧数 {len(frames_out)}（restyle {sum(1 for m in modes if m == 'restyle')} / "
        f"nn {sum(1 for m in modes if m == 'nn')}），图集 {demo_w}×{demo_h}，"
        f"输出 {OUT_DIR.relative_to(ROOT)} + {HASHES_TXT.relative_to(ROOT)}"
    )


def check() -> None:
    """复核在盘产物：解码 PNG 重算 RGBA 哈希与清单逐行对齐 + 全部硬断言重跑。"""
    tiles, frames_out, modes, counts, cols, rows, hand_set = build()
    px = TILE_PX * DENSITY
    lines = HASHES_TXT.read_text(encoding="utf-8").splitlines()
    rows_txt = [ln for ln in lines if ln and not ln.startswith("#")]
    assert len(rows_txt) == cols * rows, f"清单行数漂移: {len(rows_txt)}"
    for tid, ln in enumerate(rows_txt):
        fid, row, col, mode, use, sha = ln.split()
        assert int(fid) == tid, f"帧序漂移 @{tid}: {fid}"
        assert (int(col), int(row)) == (tid & 7, tid >> 3), f"帧 {tid} 槽位与 id 不符（陷阱①）"
        assert mode == modes[tid], f"帧 {tid} mode 漂移"
        assert int(use) == counts.get(tid, 0), f"帧 {tid} 用量漂移"
        disk = Image.open(FRAMES_DIR / f"tile_{tid:03d}.png").convert("RGBA")
        assert disk.size == (px, px), f"帧 {tid} 尺寸漂移"
        assert raw_sha(list(disk.getdata())) == sha, f"帧 {tid} 哈希漂移（在盘 vs 清单）"
        assert raw_sha(frames_out[tid]) == sha, f"帧 {tid} 哈希漂移（重算 vs 清单）"
    sheet = Image.open(OUT_DIR / "map_00_3x.png").convert("RGBA")
    assert sheet.size == (cols * px, rows * px), f"demo 图集尺寸漂移: {sheet.size}"
    assert sheet.size == (128 * DENSITY, 208 * DENSITY), "demo 尺寸 != 源 ×3"
    assert sheet.size[0] // (TILE_PX * DENSITY) == ATLAS_COLS, "R-4 列数断言失败"
    sheet_line = next(ln for ln in lines if ln.startswith("# sheet "))
    raw_sha_exp = sheet_line.split("raw_rgba_sha256=")[1].split()[0]
    png_sha_exp = sheet_line.split("png_sha256=")[1].split()[0]
    assert raw_sha(list(sheet.getdata())) == raw_sha_exp, "图集 raw RGBA 哈希漂移"
    assert hashlib.sha256((OUT_DIR / "map_00_3x.png").read_bytes()).hexdigest() == png_sha_exp, "图集 PNG 字节哈希漂移"
    nn_line = next(ln for ln in lines if ln.startswith("# nn_byte_identity "))
    nn_exp = int(nn_line.split("frames=")[1].split()[0])
    assert nn_exp == sum(1 for m in modes if m == "nn"), "NN 帧计数漂移"
    print(f"hd-demo --check: {cols * rows} 帧 + 图集 {sheet.size[0]}×{sheet.size[1]} 全部一致")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true", help="复核在盘产物与 frame-hashes.txt 一致（不写盘）")
    args = ap.parse_args()
    if args.check:
        check()
    else:
        generate()
    return 0


if __name__ == "__main__":
    sys.exit(main())
