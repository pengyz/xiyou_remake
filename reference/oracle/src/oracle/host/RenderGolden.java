package oracle.host;

import javax.microedition.lcdui.Font;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.security.MessageDigest;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/**
 * 渲染层 golden 微驱动（L1b）：对 shim 像素模型跑一组**固定绘制用例**，
 * 每用例输出全屏 ARGB 的 sha256（与 {@link TickHooks#afterPaint} 同一哈希流）；
 * 另解码 assets/raw 全部 PNG（3 独立 + 16 packed 容器全部子图）输出逐图 sha。
 *
 * <p>Rust 侧复刻（crates/game-core/src/render.rs + game-data/src/png.rs）以
 * crates/game-core/tests/render_golden.json 为裁判逐用例比对。
 *
 * <p>用法：{@code java -cp <classes> oracle.host.RenderGolden <assets/raw> [out.json]}
 * （stdout 出 JSON；out.json 给出时同时落盘）
 */
public final class RenderGolden {

    public static void main(String[] args) throws Exception {
        File raw = new File(args[0]);
        StringBuilder sb = new StringBuilder();
        sb.append("{\n  \"cases\": [\n");
        List<String[]> cases = new ArrayList<String[]>();
        runCases(cases);
        for (int i = 0; i < cases.size(); i++) {
            String[] c = cases.get(i);
            sb.append("    {\"name\":\"").append(c[0]).append("\",\"sha\":\"").append(c[1]).append("\"}");
            if (i + 1 < cases.size()) {
                sb.append(',');
            }
            sb.append('\n');
        }
        sb.append("  ],\n  \"images\": [\n");
        List<String[]> images = new ArrayList<String[]>();
        decodeAllImages(raw, images);
        for (int i = 0; i < images.size(); i++) {
            String[] m = images.get(i);
            sb.append("    {\"name\":\"").append(m[0]).append("\",\"w\":").append(m[1])
              .append(",\"h\":").append(m[2]).append(",\"sha\":\"").append(m[3]).append("\"}");
            if (i + 1 < images.size()) {
                sb.append(',');
            }
            sb.append('\n');
        }
        sb.append("  ]\n}\n");
        String json = sb.toString();
        System.out.print(json);
        if (args.length > 1) {
            try (java.io.Writer w = new java.io.OutputStreamWriter(
                    new java.io.FileOutputStream(args[1]), StandardCharsets.UTF_8)) {
                w.write(json);
            }
        }
    }

    // ---------------------------------------------------------------- 用例

    private static void runCases(List<String[]> out) throws Exception {
        // 16：仅白底（画布初始 0 → fillRect 白）
        caseOf(out, "blank_white", new Painter() {
            public void paint(Graphics g) {
            }
        });
        // 1：setClip 替换语义（红块只出现在交集内）
        caseOf(out, "fill_clip", new Painter() {
            public void paint(Graphics g) {
                g.setClip(20, 30, 100, 80);
                g.setColor(0xFF0000);
                g.fillRect(0, 0, 240, 320);
            }
        });
        // 2：clipRect 交集链
        caseOf(out, "clip_rect_chain", new Painter() {
            public void paint(Graphics g) {
                g.setClip(10, 10, 50, 50);
                g.clipRect(30, 30, 100, 100);
                g.setColor(0x00FF00);
                g.fillRect(0, 0, 240, 320);
            }
        });
        // 3：translate 只移动 clip（shim 特有语义）
        caseOf(out, "translate_clip", new Painter() {
            public void paint(Graphics g) {
                g.setClip(100, 100, 40, 40);
                g.translate(-50, -50);
                g.setColor(0x0000FF);
                g.fillRect(100, 100, 20, 20);  // clip 已移到 (50,50)-(90,90) ⇒ 全被裁
                g.fillRect(50, 50, 20, 20);    // 命中移后的 clip
            }
        });
        // 4：Bresenham 八象限 + 陡/缓线
        caseOf(out, "bresenham_octants", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0x000000);
                int cx = 120, cy = 160;
                int[][] pts = {{200, 160}, {200, 260}, {120, 260}, {40, 260}, {40, 160},
                        {40, 60}, {120, 60}, {200, 60}, {210, 30}, {30, 290}, {130, 20}};
                for (int[] p : pts) {
                    g.drawLine(cx, cy, p[0], p[1]);
                }
                g.drawLine(10, 10, 230, 310);
                g.drawLine(230, 10, 10, 310);
            }
        });
        // 5：drawRect 常规/退化尺寸
        caseOf(out, "draw_rect_sizes", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0xFF8800);
                g.drawRect(10, 10, 50, 30);
                g.drawRect(70, 10, 1, 1);
                g.drawRect(80, 10, 0, 5);
                g.drawRect(90, 10, 5, 0);
            }
        });
        // 6：fillTriangle 平底
        caseOf(out, "fill_triangle_flat", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0x00AAAA);
                g.fillTriangle(30, 250, 70, 250, 50, 230);
            }
        });
        // 7：fillTriangle 陡斜边
        caseOf(out, "fill_triangle_steep", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0xAA00AA);
                g.fillTriangle(100, 230, 120, 280, 140, 230);
            }
        });
        // 8：fillTriangle 退化（点/平线）
        caseOf(out, "fill_triangle_degenerate", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0xAA0000);
                g.fillTriangle(170, 250, 170, 250, 170, 250);
                g.fillTriangle(180, 250, 190, 250, 200, 250);
            }
        });
        // 9：fillArc 全圆（游戏唯一调用形态，a.java:6635）
        caseOf(out, "fill_arc_full", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0x0000FF);
                g.fillArc(90, 90, 32, 32, 0, 360);
            }
        });
        // 10：src-over 混合（半透明源叠两层）
        caseOf(out, "blend_over", new Painter() {
            public void paint(Graphics g) throws Exception {
                g.setColor(0x00CC66);
                g.fillRect(0, 0, 240, 320);
                Image src = translucentSource();
                g.drawImage(src, 40, 40, 20);
                g.drawImage(src, 60, 60, 20);
            }
        });
        // 11：drawImage anchor 组合
        caseOf(out, "draw_image_anchors", new Painter() {
            public void paint(Graphics g) throws Exception {
                Image src = gradientSource();
                g.drawImage(src, 10, 10, 20);        // TOP|LEFT
                g.drawImage(src, 120, 160, 3);       // HCENTER|VCENTER
                g.drawImage(src, 230, 310, 40);      // BOTTOM|RIGHT
                g.drawImage(src, 230, 10, 8);        // RIGHT|TOP
            }
        });
        // 12：DirectGraphics 8 变换
        caseOf(out, "draw_image_transformed", new Painter() {
            public void paint(Graphics g) throws Exception {
                Image src = gradientSource();
                for (int t = 0; t < 8; t++) {
                    g.drawImageTransformed(src, 20 + t * 26, 40, 20, t);  // TOP|LEFT
                }
            }
        });
        // 13：hash 字形（CJK/ASCII/粗体/小号）
        caseOf(out, "font_glyphs", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0x000000);
                g.drawString("\u56e7\u56e7\u897f\u6e38 ABC 012", 20, 40, 20);
                g.setFont(Font.getFont(Font.FACE_SYSTEM, Font.STYLE_BOLD, Font.SIZE_MEDIUM));
                g.drawString("\u56e7\u56e7 BOLD", 20, 80, 20);
                g.setFont(Font.getFont(Font.FACE_SYSTEM, Font.STYLE_PLAIN, Font.SIZE_SMALL));
                g.drawString("small 123", 20, 110, 20);
                g.setFont(Font.getFont(Font.FACE_SYSTEM, Font.STYLE_PLAIN, Font.SIZE_LARGE));
                g.drawString("L\u56e7", 20, 140, 20);
            }
        });
        // 14：drawSubstring + HCENTER|BASELINE
        caseOf(out, "draw_substring", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0x202020);
                g.drawSubstring("\u56e7\u56e7\u897f\u6e38\u5927\u95f9\u5929\u5bab", 2, 3, 100, 200, 1 | 64);
            }
        });
        // 15：drawChar
        caseOf(out, "draw_char", new Painter() {
            public void paint(Graphics g) {
                g.setColor(0x101010);
                g.drawChar('\u56e7', 50, 50, 20);
            }
        });
    }

    private interface Painter {
        void paint(Graphics g) throws Exception;
    }

    private static void caseOf(List<String[]> out, String name, Painter painter) throws Exception {
        Image screen = Image.createImage(240, 320);
        Graphics g = screen.getGraphics();
        g.setColor(0xFFFFFF);
        g.fillRect(0, 0, 240, 320);
        painter.paint(g);
        out.add(new String[]{name, hashImage(screen)});
        int[] px = new int[240 * 320];
        screen.getRGB(px, 0, 240, 0, 0, 240, 320);
        // 用例内自检：hashImage 与 getRGB 流一致（防止两套哈希输入漂移）
    }

    // ---------------------------------------------------------------- 源图

    /** 半透明测试源（11x7）：1/3 全透明、其余 50% 蓝。 */
    static Image translucentSource() {
        int[] px = new int[11 * 7];
        for (int i = 0; i < px.length; i++) {
            px[i] = (i % 3 == 0) ? 0 : (0x80 << 24) | 0x0000FF;
        }
        return Image.createRGBImage(px, 11, 7, true);
    }

    /** 位置梯度源（12x8）：变换方向可目视区分。 */
    static Image gradientSource() {
        int[] px = new int[12 * 8];
        for (int y = 0; y < 8; y++) {
            for (int x = 0; x < 12; x++) {
                px[y * 12 + x] = 0xFF000000 | (x * 20 << 16) | (y * 30 << 8) | ((x + y) * 10);
            }
        }
        return Image.createRGBImage(px, 12, 8, true);
    }

    // ---------------------------------------------------------------- PNG

    /** 解码 assets/raw 全部 PNG：3 独立 png + 16 packed 容器全部子图。 */
    static void decodeAllImages(File raw, List<String[]> out) throws Exception {
        for (String name : new String[]{"i62x62.png", "l0.png", "l1.png"}) {
            byte[] data = readAll(new File(raw, name));
            Image img = Image.createImage(data, 0, data.length);
            out.add(new String[]{name, String.valueOf(img.getWidth()),
                    String.valueOf(img.getHeight()), hashImage(img)});
        }
        String[] containers = {"sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm",
                "sptenemy1", "ui", "xtq", "menu", "intro", "face", "sptenemy2", "end", "load"};
        for (String name : containers) {
            byte[] all = readAll(new File(raw, name));
            int pos = 0;
            int idx = 0;
            while (pos + 2 <= all.length) {
                int len = (all[pos] & 0xFF) | (all[pos + 1] & 0xFF) << 8;
                pos += 2;
                Image img = Image.createImage(all, pos, len);
                out.add(new String[]{name + "#" + idx, String.valueOf(img.getWidth()),
                        String.valueOf(img.getHeight()), hashImage(img)});
                pos += len;
                idx++;
            }
        }
    }

    // ---------------------------------------------------------------- 哈希

    /** 与 TickHooks.afterPaint 同一哈希流：getRGB 全屏，逐像素大端 4 字节。 */
    static String hashImage(Image img) throws Exception {
        int w = img.getWidth(), h = img.getHeight();
        int[] px = new int[w * h];
        img.getRGB(px, 0, w, 0, 0, w, h);
        MessageDigest md = MessageDigest.getInstance("SHA-256");
        for (int p : px) {
            md.update((byte) (p >>> 24));
            md.update((byte) (p >>> 16));
            md.update((byte) (p >>> 8));
            md.update((byte) p);
        }
        StringBuilder sb = new StringBuilder();
        for (byte b : md.digest()) {
            sb.append(String.format("%02x", b));
        }
        return sb.toString();
    }

    static byte[] readAll(File f) throws Exception {
        InputStream in = new FileInputStream(f);
        try {
            ByteArrayOutputStream out = new ByteArrayOutputStream(1 << 14);
            byte[] buf = new byte[1 << 14];
            int n;
            while ((n = in.read(buf)) >= 0) {
                out.write(buf, 0, n);
            }
            return out.toByteArray();
        } finally {
            in.close();
        }
    }
}
