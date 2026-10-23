#!/usr/bin/env python3
"""原图精修放大（保设计超分）：RealESRGAN x4 anime 模型，设计逐像素不动。

用户裁定（2026-10-22）：形象迁移废弃——AI 重绘会捏造设计（三尖枪/错脸谱）。
改为「AI 给原图精修，提升分辨率」：超分模型只放大/清理，不改设计。
模型：RealESRGAN_x4plus_anime_6B（平涂/动画风适配）。输入 assets/png/actor.png
正面帧 00（41×46）→ 4x → 164×184 高清帧（density 4，直接进 HD 密度通路）。
"""
import os
import numpy as np
import torch
from PIL import Image
from spandrel import ModelLoader

MODEL = "tools/style-sample/models/RealESRGAN_x4plus_anime_6B.pth"
SRC = "assets/png/actor.png"
OUT_DIR = "assets/hd-demo/style-sample"


def to_tensor(img: Image.Image) -> torch.Tensor:
    a = np.asarray(img.convert("RGB"), dtype=np.float32) / 255.0
    return torch.from_numpy(a).permute(2, 0, 1).unsqueeze(0)


def to_image(t: torch.Tensor) -> Image.Image:
    a = t.squeeze(0).permute(1, 2, 0).clamp(0, 1).cpu().numpy()
    return Image.fromarray((a * 255).round().astype(np.uint8))


def main():
    desc = ModelLoader().load_from_file(MODEL)
    model = desc.model.eval()
    orig = Image.open(SRC).convert("RGB")
    fw, fh = orig.width // 3, orig.height // 3
    frame = orig.crop((0, 0, fw, fh))  # 正面帧 00（41×46）
    print(f"input frame: {frame.size}")
    with torch.no_grad():
        out = model(to_tensor(frame))
    up = to_image(out)
    print(f"upscaled: {up.size}")
    os.makedirs(OUT_DIR, exist_ok=True)
    up.save(f"{OUT_DIR}/wukong_esrgan_x4.png")
    # 对照：原帧 6x NN | ESRGAN 4x（同显高）
    H = 460
    nn = frame.resize((fw * 6, fh * 6), Image.NEAREST)
    o1 = nn.resize((int(nn.width * H / nn.height), H), Image.NEAREST)
    u1 = up.resize((int(up.width * H / up.height), H), Image.LANCZOS)
    canvas = Image.new("RGB", (o1.width + 24 + u1.width, H + 36), (242, 232, 212))
    canvas.paste(o1, (6, 30))
    canvas.paste(u1, (o1.width + 18, 30))
    from PIL import ImageDraw
    d = ImageDraw.Draw(canvas)
    d.text((8, 8), "ORIGINAL 41x46 (6x NN)", fill=(40, 30, 24))
    d.text((o1.width + 20, 8), "RealESRGAN x4 anime (design preserved)", fill=(40, 30, 24))
    canvas.save(f"{OUT_DIR}/compare_esrgan_x4.png")
    print("done")


if __name__ == "__main__":
    main()
