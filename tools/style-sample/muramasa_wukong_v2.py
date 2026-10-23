#!/usr/bin/env python3
"""风格样张 v2：Q 版悟空 × 胧村正"华丽画工"（风格迁移，非换比例）。

用户裁定（2026-10-22）：**Q 版卡通形象保持不动**，迁移的是香草社的华丽画风——
柔喷薄影（遮罩高斯影）、鎏金质感（多层高光）、饱和石色、丰润光感、极细暖色勾线。
高清分辨率素材（密度通路已验）。

输出 assets/hd-demo/style-sample/：wukong_v2_hos（独立帧）+ compare_v2（对照）。
仅风格样张 demo；量产由画师/AI 精修。
"""
import os
from PIL import Image, ImageDraw, ImageFilter

S = 2  # 超采样（绘后 LANCZOS 降采样保边）
W, H = 246, 276  # 逻辑尺寸（≈原帧 41×46 的 6x 高清）
CW, CH = W * S, H * S

# —— 胧村正华丽配色（饱和石色系 + 鎏金光感）——
GOLD_HI, GOLD_LT, GOLD_MID, GOLD_DP = (255, 236, 168), (247, 214, 122), (216, 156, 58), (160, 98, 28)
VERM, VERM_DP = (216, 69, 46), (132, 36, 24)
TEAL, TEAL_LT, TEAL_DP = (46, 110, 107), (95, 168, 162), (28, 66, 64)
CREAM, CREAM_SH = (247, 238, 219), (228, 193, 158)
FACE_RED, FACE_DP = (201, 58, 43), (146, 36, 26)
LINE = (66, 38, 28)  # 极细暖色勾线（非纯黑）
BOOT, BOOT_LT = (64, 40, 30), (110, 72, 48)
SKIN, SKIN_SH = (242, 214, 172), (214, 174, 128)

base = Image.new("RGBA", (CW, CH), (0, 0, 0, 0))
d = ImageDraw.Draw(base)
shade = Image.new("RGBA", (CW, CH), (0, 0, 0, 0))   # 柔影层（后模糊合成）
sd = ImageDraw.Draw(shade)
lite = Image.new("RGBA", (CW, CH), (0, 0, 0, 0))    # 高光层（后模糊合成）
ld = ImageDraw.Draw(lite)


def P(pts):
    return [(x * S, y * S) for x, y in pts]


def poly(pts, fill, line=LINE, w=2):
    d.polygon(P(pts), fill=fill)
    if line:
        d.line(P(pts) + [P(pts)[0]], fill=line, width=max(w * S // 2, 1), joint="curve")


def ell(cx, cy, rx, ry, fill, line=LINE, w=2):
    box = [(cx - rx) * S, (cy - ry) * S, (cx + rx) * S, (cy + ry) * S]
    d.ellipse(box, fill=fill)
    if line:
        d.ellipse(box, outline=line, width=max(w * S // 2, 1))


def soft_shade(pts_list, color, alpha, blur=14):
    """柔喷薄影：形状画进影层→高斯模糊→以 alpha 合成（胧村正柔影关键）。"""
    m = Image.new("L", (CW, CH), 0)
    md = ImageDraw.Draw(m)
    for pts in pts_list:
        md.polygon(P(pts), fill=alpha)
    m = m.filter(ImageFilter.GaussianBlur(blur * S // 2))
    layer = Image.new("RGBA", (CW, CH), color + (0,))
    layer.putalpha(m)
    shade.alpha_composite(layer)


def soft_lite(ellipses, color, alpha, blur=10):
    m = Image.new("L", (CW, CH), 0)
    md = ImageDraw.Draw(m)
    for (cx, cy, rx, ry) in ellipses:
        md.ellipse([(cx - rx) * S, (cy - ry) * S, (cx + rx) * S, (cy + ry) * S], fill=alpha)
    m = m.filter(ImageFilter.GaussianBlur(blur * S // 2))
    layer = Image.new("RGBA", (CW, CH), color + (0,))
    layer.putalpha(m)
    lite.alpha_composite(layer)


def curve(pts, color, w0, w1):
    n = len(pts)
    for i in range(n - 1):
        t = i / max(n - 2, 1)
        w = int((w0 + (w1 - w0) * t) * S / 2)
        d.line([(pts[i][0] * S, pts[i][1] * S), (pts[i + 1][0] * S, pts[i + 1][1] * S)],
               fill=color, width=max(w, 1))


# ================= Q 版身形（头身比照原版：大头小身）=================
# —— 雉鸡翎（鎏金渐变长翎，先画衬后）——
fL = [(104, 52), (72, 30), (40, 16), (16, 12), (4, 22)]
fR = [(142, 52), (174, 30), (206, 16), (230, 12), (242, 22)]
for f in (fL, fR):
    curve(f, GOLD_DP, 11, 3)
    curve(f, GOLD_MID, 8, 2)
    curve([(x, y - 2) for x, y in f], GOLD_LT, 4, 1)
curve([fL[0], (60, 26), (28, 14)], GOLD_HI, 2, 1)
curve([fR[0], (186, 26), (218, 14)], GOLD_HI, 2, 1)

# —— 金箍棒（身后斜倚·鎏金+朱砂箍）——
poly([(46, 252), (62, 258), (196, 118), (180, 112)], GOLD_DP)
poly([(50, 250), (60, 254), (192, 120), (182, 116)], GOLD_MID)
poly([(53, 249), (58, 251), (188, 122), (183, 119)], GOLD_HI)
poly([(42, 248), (66, 257), (70, 268), (46, 259)], VERM, LINE, 2)
poly([(184, 106), (206, 96), (212, 108), (190, 118)], VERM, LINE, 2)

# —— 靴 ——
poly([(96, 232), (120, 232), (122, 262), (94, 262)], BOOT, LINE, 2)
poly([(126, 232), (150, 232), (152, 262), (124, 262)], BOOT, LINE, 2)
poly([(92, 256), (124, 256), (126, 270), (90, 270)], BOOT_LT, LINE, 2)
poly([(122, 256), (154, 256), (156, 270), (120, 270)], BOOT_LT, LINE, 2)
soft_shade([[(96, 232), (120, 232), (122, 250), (94, 250)]], VERM_DP, 110, 6)

# —— 腿（红裤）——
poly([(96, 196), (120, 196), (121, 238), (95, 238)], VERM, LINE, 2)
poly([(126, 196), (150, 196), (151, 238), (125, 238)], VERM, LINE, 2)
soft_shade([[(96, 196), (120, 196), (121, 238), (95, 238)], [(126, 196), (150, 196), (151, 238), (125, 238)]],
           VERM_DP, 120, 7)
soft_lite([(104, 206, 5, 16), (134, 206, 5, 16)], VERM, 130, 7)

# —— 战裙（金·飘摆+石青缘）——
poly([(80, 178), (166, 178), (176, 216), (158, 238), (123, 226), (88, 238), (70, 216)],
     GOLD_MID, LINE, 2)
poly([(82, 178), (164, 178), (167, 196), (79, 196)], GOLD_LT)
curve([(92, 196), (86, 216), (96, 232)], GOLD_DP, 4, 2)
curve([(154, 196), (160, 216), (150, 232)], GOLD_DP, 4, 2)
poly([(70, 216), (88, 238), (80, 244), (62, 220)], TEAL, LINE, 2)
poly([(176, 216), (158, 238), (166, 244), (184, 220)], TEAL, LINE, 2)
soft_shade([[(80, 178), (166, 178), (170, 210), (76, 210)]], GOLD_DP, 110, 10)
soft_lite([(100, 188, 18, 6), (146, 188, 18, 6)], GOLD_HI, 150, 8)

# —— 甲身（黄金甲·鎏金）——
poly([(84, 118), (162, 118), (170, 156), (160, 190), (86, 190), (76, 156)], GOLD_MID, LINE, 2)
poly([(90, 122), (156, 122), (161, 152), (123, 162), (85, 152)], GOLD_LT)
curve([(123, 128), (113, 148), (123, 164), (133, 148), (123, 128)], VERM_DP, 3, 2)  # 胸纹
curve([(100, 168), (123, 178), (146, 168)], GOLD_DP, 3, 2)
poly([(82, 182), (164, 182), (164, 198), (82, 198)], VERM, LINE, 2)   # 腰带
poly([(112, 182), (134, 182), (134, 198), (112, 198)], GOLD_HI, LINE, 2)
ell(80, 128, 20, 16, GOLD_MID, LINE, 2)   # 护肩
ell(166, 128, 20, 16, GOLD_MID, LINE, 2)
ell(80, 125, 13, 9, GOLD_LT)
ell(166, 125, 13, 9, GOLD_LT)
soft_shade([[(84, 118), (162, 118), (168, 160), (80, 160)]], GOLD_DP, 100, 11)
soft_lite([(100, 134, 14, 8), (146, 134, 14, 8), (123, 150, 20, 6)], GOLD_HI, 170, 8)

# —— 手臂（Q 版短臂）——
poly([(68, 136), (88, 136), (86, 176), (66, 172)], GOLD_MID, LINE, 2)
poly([(158, 136), (178, 136), (180, 172), (160, 176)], GOLD_MID, LINE, 2)
poly([(64, 168), (88, 172), (85, 186), (61, 182)], TEAL, LINE, 2)
poly([(158, 172), (182, 168), (185, 182), (161, 186)], TEAL, LINE, 2)
ell(72, 192, 13, 11, SKIN, LINE, 2)
ell(174, 192, 13, 11, SKIN, LINE, 2)
soft_shade([[(68, 136), (88, 136), (86, 176), (66, 172)], [(158, 136), (178, 136), (180, 172), (160, 176)]],
           GOLD_DP, 110, 7)
soft_shade([[(72, 186), (84, 188), (84, 198), (72, 198)]], SKIN_SH, 120, 5)

# —— 头（Q 版大头·京剧红面膛）——
poly([(108, 100), (138, 100), (136, 122), (110, 122)], SKIN, LINE, 2)  # 颈
ell(123, 74, 58, 52, CREAM, LINE, 2)                                     # 白脸基底
poly([(123, 40), (158, 54), (170, 78), (156, 106), (123, 118), (90, 106), (76, 78), (88, 54)],
     FACE_RED, LINE, 2)                                                  # 红面膛
curve([(100, 52), (118, 44), (128, 48)], CREAM, 8, 4)                    # 白眉
curve([(146, 52), (128, 44), (118, 48)], CREAM, 8, 4)
poly([(98, 66), (116, 60), (118, 74), (100, 76)], CREAM, LINE, 2)        # 凤眼
poly([(148, 66), (130, 60), (128, 74), (146, 76)], CREAM, LINE, 2)
ell(110, 68, 5, 6, LINE)
ell(136, 68, 5, 6, LINE)
curve([(118, 84), (123, 92), (128, 84)], VERM_DP, 3, 2)                  # 鼻
curve([(104, 100), (123, 108), (142, 100)], CREAM, 8, 4)                 # 嘴（白胡）
curve([(108, 101), (123, 105), (138, 101)], VERM_DP, 2, 2)
ell(68, 80, 11, 16, FACE_RED, LINE, 2)                                   # 耳+金环
ell(178, 80, 11, 16, FACE_RED, LINE, 2)
ell(68, 102, 6, 6, GOLD_LT, LINE, 1)
ell(178, 102, 6, 6, GOLD_LT, LINE, 1)
soft_shade([[(123, 40), (160, 60), (168, 88), (150, 110), (123, 118)]], FACE_DP, 110, 12)  # 右侧柔影
soft_lite([(104, 58, 16, 12), (96, 88, 10, 12)], CREAM, 130, 10)         # 面部柔光

# —— 头冠（凤翅紫金冠·鎏金华丽）——
poly([(82, 52), (90, 24), (110, 12), (123, 8), (136, 12), (156, 24), (164, 52),
      (152, 38), (123, 30), (94, 38)], GOLD_MID, LINE, 2)
poly([(96, 28), (123, 16), (150, 28), (146, 40), (123, 32), (100, 40)], GOLD_LT)
poly([(62, 46), (88, 36), (92, 52), (66, 60)], GOLD_MID, LINE, 2)         # 左翅
poly([(184, 46), (158, 36), (154, 52), (180, 60)], GOLD_MID, LINE, 2)     # 右翅
poly([(64, 45), (86, 38), (88, 46), (66, 52)], GOLD_HI)
poly([(182, 45), (160, 38), (158, 46), (180, 52)], GOLD_HI)
ell(123, 18, 9, 9, VERM, LINE, 2)                                        # 宝珠
poly([(116, 6), (123, -6), (130, 6)], VERM, LINE, 2)                     # 珠缨
poly([(92, 42), (104, 38), (106, 50), (94, 54)], GOLD_DP, LINE, 2)       # 翎座
poly([(142, 38), (154, 42), (152, 54), (140, 50)], GOLD_DP, LINE, 2)
soft_shade([[(82, 52), (94, 22), (123, 10), (152, 22), (164, 52), (123, 34)]], GOLD_DP, 110, 9)
soft_lite([(106, 22, 10, 6), (140, 22, 10, 6), (123, 14, 8, 4)], GOLD_HI, 190, 6)

# ================= 合成：柔影/高光 → 极细勾线复勾 → 降采样 =================
out = base.copy()
out.alpha_composite(shade.filter(ImageFilter.GaussianBlur(2 * S)))
out.alpha_composite(lite.filter(ImageFilter.GaussianBlur(1.5 * S)))

img = out.resize((W, H), Image.LANCZOS)
os.makedirs("assets/hd-demo/style-sample", exist_ok=True)
img.save("assets/hd-demo/style-sample/wukong_v2_hos.png")

# —— 对照：原帧 | v2 ——
orig = Image.open("assets/png/actor.png").convert("RGBA")
fw, fh = orig.width // 3, orig.height // 3
frame = orig.crop((0, 0, fw, fh))
disp_h = 520
o2 = frame.resize((int(fw * disp_h / fh), disp_h), Image.NEAREST)
n2 = img.resize((int(W * disp_h / H), disp_h), Image.LANCZOS)
canvas = Image.new("RGBA", (o2.width + 32 + n2.width, disp_h + 44), (242, 232, 212, 255))
canvas.paste(o2, (8, 36), o2)
canvas.paste(n2, (o2.width + 28, 36), n2)
dd = ImageDraw.Draw(canvas)
dd.text((10, 10), "ORIGINAL pixel art", fill=(40, 30, 24))
dd.text((o2.width + 30, 10), "v2 Q-BODY x MURAMASA PAINTING (demo)", fill=(40, 30, 24))
canvas.save("assets/hd-demo/style-sample/compare_v2.png")
print("done: wukong_v2_hos.png + compare_v2.png")
