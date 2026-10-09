package javax.microedition.lcdui;

import java.util.ArrayList;
import java.util.List;

/**
 * MIDP-1.0 {@code javax.microedition.lcdui.Graphics} 自研 shim。
 *
 * <p>参考版实际引用的 API 面（constant-pool 扫描）：
 * drawChar / drawImage(Image,int,int,int) / drawLine / drawRect / drawString /
 * drawSubstring / fillArc / fillRect / fillTriangle / getColor / setClip /
 * setColor(int) / setFont(Font)。
 *
 * <p>像素模型：非预乘 ARGB int[]，src-over 混合（alpha=0 跳过，alpha=255 直写），
 * 全整数运算 ⇒ 逐像素确定性。字形渲染为 <b>确定性示意图案</b>（按字符码派生），
 * 非真实字库：文字位置/宽度证据看 {@code OP drawString(...)} 操作流，
 * 屏幕像素哈希用于差分，不用于设备视觉保真（L4 另行处理）。
 *
 * <p>所有绘制操作同时记入全局操作流（trace 用），单线程、确定顺序。
 */
public class Graphics {
    public static final int HCENTER = 1;
    public static final int VCENTER = 2;
    public static final int LEFT = 4;
    public static final int RIGHT = 8;
    public static final int TOP = 16;
    public static final int BOTTOM = 32;
    public static final int BASELINE = 64;

    public static final int SOLID = 0;
    public static final int DOTTED = 1;

    final Image target;
    private int color = 0x000000;
    private Font font;
    private int clipX, clipY, clipW, clipH;

    /** 全局绘制操作流（trace 证据）；由 oracle 钩子定期清空。 */
    private static final List<String> OPS = new ArrayList<String>();
    private static long opCount = 0;

    Graphics(Image target) {
        this.target = target;
        this.font = Font.getFont(Font.FACE_SYSTEM, Font.STYLE_PLAIN, Font.SIZE_MEDIUM);
        this.clipX = 0;
        this.clipY = 0;
        this.clipW = target.width;
        this.clipH = target.height;
    }

    // ---------------- trace 操作流 ----------------

    public static synchronized String flushOps() {
        StringBuilder sb = new StringBuilder();
        int n = Math.min(OPS.size(), 256);
        for (int i = 0; i < n; i++) {
            if (i > 0) {
                sb.append('\n');
            }
            sb.append(OPS.get(i));
        }
        String head = "OPS count=" + opCount;
        OPS.clear();
        opCount = 0;
        return head + "\n" + sb;
    }

    private static synchronized void op(String s) {
        opCount++;
        if (OPS.size() < 256) {
            OPS.add(s);
        }
    }

    private static String q(String s) {
        StringBuilder sb = new StringBuilder("\"");
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '"': sb.append("\\\""); break;
                case '\\': sb.append("\\\\"); break;
                case '\n': sb.append("\\n"); break;
                case '\r': sb.append("\\r"); break;
                case '\t': sb.append("\\t"); break;
                default:
                    if (c < 0x20 || c > 0x7E) {
                        sb.append(String.format("\\u%04x", (int) c));
                    } else {
                        sb.append(c);
                    }
            }
        }
        return sb.append('"').toString();
    }

    // ---------------- 状态 ----------------

    public int getColor() {
        return color;
    }

    public void setColor(int rgb) {
        this.color = rgb & 0xFFFFFF;
        op("setColor(#" + String.format("%06x", color) + ")");
    }

    public void setColor(int r, int g, int b) {
        setColor((r << 16) | (g << 8) | b);
    }

    public void setFont(Font f) {
        this.font = f;
        op("setFont(size=" + (f == null ? -1 : f.size) + ")");
    }

    public Font getFont() {
        return font;
    }

    public void setClip(int x, int y, int w, int h) {
        // MIDP 语义：setClip 取与画布的绝对交集，替换当前剪裁区
        int nx = Math.max(x, 0);
        int ny = Math.max(y, 0);
        int nx2 = Math.min(x + w, target.width);
        int ny2 = Math.min(y + h, target.height);
        clipX = nx;
        clipY = ny;
        clipW = Math.max(0, nx2 - nx);
        clipH = Math.max(0, ny2 - ny);
        op("setClip(" + x + "," + y + "," + w + "," + h + ")");
    }

    public void clipRect(int x, int y, int w, int h) {
        int x2 = Math.min(clipX + clipW, x + w);
        int y2 = Math.min(clipY + clipH, y + h);
        int nx = Math.max(clipX, x);
        int ny = Math.max(clipY, y);
        clipX = nx;
        clipY = ny;
        clipW = Math.max(0, x2 - nx);
        clipH = Math.max(0, y2 - ny);
        op("clipRect(" + x + "," + y + "," + w + "," + h + ")");
    }

    public void translate(int x, int y) {
        clipX += x;
        clipY += y;
        op("translate(" + x + "," + y + ")");
    }

    public int getTranslateX() {
        return 0;
    }

    public int getTranslateY() {
        return 0;
    }

    // ---------------- 基础图元 ----------------

    private void setPixel(int x, int y, int argb) {
        if (x < clipX || y < clipY || x >= clipX + clipW || y >= clipY + clipH) {
            return;
        }
        int a = (argb >>> 24) & 0xFF;
        int idx = y * target.width + x;
        if (a == 0) {
            return;
        }
        if (a == 255) {
            target.argb[idx] = argb;
            return;
        }
        int dst = target.argb[idx];
        int ia = 255 - a;
        int r = (((argb >>> 16) & 0xFF) * a + ((dst >>> 16) & 0xFF) * ia) / 255;
        int g = (((argb >>> 8) & 0xFF) * a + ((dst >>> 8) & 0xFF) * ia) / 255;
        int b = ((argb & 0xFF) * a + (dst & 0xFF) * ia) / 255;
        int da = (((dst >>> 24) & 0xFF) * ia) / 255 + a;
        target.argb[idx] = (da << 24) | (r << 16) | (g << 8) | b;
    }

    private int opaqueColor() {
        return 0xFF000000 | color;
    }

    public void fillRect(int x, int y, int w, int h) {
        op("fillRect(" + x + "," + y + "," + w + "," + h + ",#" + String.format("%06x", color) + ")");
        int c = opaqueColor();
        for (int yy = y; yy < y + h; yy++) {
            for (int xx = x; xx < x + w; xx++) {
                setPixel(xx, yy, c);
            }
        }
    }

    public void drawRect(int x, int y, int w, int h) {
        op("drawRect(" + x + "," + y + "," + w + "," + h + ",#" + String.format("%06x", color) + ")");
        drawLine(x, y, x + w - 1, y);
        drawLine(x, y + h - 1, x + w - 1, y + h - 1);
        drawLine(x, y, x, y + h - 1);
        drawLine(x + w - 1, y, x + w - 1, y + h - 1);
    }

    public void drawLine(int x1, int y1, int x2, int y2) {
        op("drawLine(" + x1 + "," + y1 + "," + x2 + "," + y2 + ",#" + String.format("%06x", color) + ")");
        int c = opaqueColor();
        int dx = Math.abs(x2 - x1), dy = Math.abs(y2 - y1);
        int sx = x1 < x2 ? 1 : -1, sy = y1 < y2 ? 1 : -1;
        int err = dx - dy;
        int x = x1, y = y1;
        while (true) {
            setPixel(x, y, c);
            if (x == x2 && y == y2) {
                break;
            }
            int e2 = 2 * err;
            if (e2 > -dy) {
                err -= dy;
                x += sx;
            }
            if (e2 < dx) {
                err += dx;
                y += sy;
            }
        }
    }

    public void fillTriangle(int x1, int y1, int x2, int y2, int x3, int y3) {
        op("fillTriangle(" + x1 + "," + y1 + "," + x2 + "," + y2 + "," + x3 + "," + y3
                + ",#" + String.format("%06x", color) + ")");
        int c = opaqueColor();
        // 顶点排序（确定性）
        if (y1 > y2) { int t = y1; y1 = y2; y2 = t; t = x1; x1 = x2; x2 = t; }
        if (y2 > y3) { int t = y2; y2 = y3; y3 = t; t = x2; x2 = x3; x3 = t; }
        if (y1 > y2) { int t = y1; y1 = y2; y2 = t; t = x1; x1 = x2; x2 = t; }
        for (int y = y1; y <= y3; y++) {
            double xl, xr;
            if (y < y2) {
                xl = edge(x1, y1, x3, y3, y);
                xr = edge(x1, y1, x2, y2, y);
            } else {
                xl = edge(x1, y1, x3, y3, y);
                xr = edge(x2, y2, x3, y3, y);
            }
            int lo = (int) Math.ceil(Math.min(xl, xr));
            int hi = (int) Math.floor(Math.max(xl, xr));
            for (int x = lo; x <= hi; x++) {
                setPixel(x, y, c);
            }
        }
    }

    private static double edge(int x1, int y1, int x2, int y2, int y) {
        if (y1 == y2) {
            return Math.min(x1, x2);
        }
        return x1 + (x2 - x1) * (double) (y - y1) / (double) (y2 - y1);
    }

    public void fillArc(int x, int y, int w, int h, int startAngle, int arcAngle) {
        op("fillArc(" + x + "," + y + "," + w + "," + h + "," + startAngle + "," + arcAngle
                + ",#" + String.format("%06x", color) + ")");
        int c = opaqueColor();
        double cx = x + w / 2.0, cy = y + h / 2.0;
        double rx = w / 2.0, ry = h / 2.0;
        int normStart = startAngle % 360;
        if (normStart < 0) {
            normStart += 360;
        }
        for (int yy = y; yy < y + h; yy++) {
            for (int xx = x; xx < x + w; xx++) {
                double dx = (xx + 0.5 - cx), dy = (yy + 0.5 - cy);
                if (rx <= 0 || ry <= 0) {
                    continue;
                }
                double nx = dx / rx, ny = dy / ry;
                if (nx * nx + ny * ny > 1.0) {
                    continue;
                }
                if (inArc(Math.atan2(ny, nx), normStart, arcAngle)) {
                    setPixel(xx, yy, c);
                }
            }
        }
    }

    public void drawArc(int x, int y, int w, int h, int startAngle, int arcAngle) {
        op("drawArc(" + x + "," + y + "," + w + "," + h + "," + startAngle + "," + arcAngle
                + ",#" + String.format("%06x", color) + ")");
        int c = opaqueColor();
        double cx = x + w / 2.0, cy = y + h / 2.0;
        double rx = w / 2.0, ry = h / 2.0;
        int normStart = startAngle % 360;
        if (normStart < 0) {
            normStart += 360;
        }
        for (int yy = y; yy < y + h; yy++) {
            for (int xx = x; xx < x + w; xx++) {
                double dx = (xx + 0.5 - cx), dy = (yy + 0.5 - cy);
                double nx = rx > 0 ? dx / rx : 0, ny = ry > 0 ? dy / ry : 0;
                double r2 = nx * nx + ny * ny;
                if (r2 < 0.72 || r2 > 1.28) {
                    continue;
                }
                if (inArc(Math.atan2(ny, nx), normStart, arcAngle)) {
                    setPixel(xx, yy, c);
                }
            }
        }
    }

    private static boolean inArc(double rad, int startDeg, int arcDeg) {
        double deg = Math.toDegrees(rad);
        if (deg < 0) {
            deg += 360.0;
        }
        if (arcDeg >= 360) {
            return true;
        }
        if (arcDeg < 0) {
            return !inArc(rad, (startDeg + arcDeg) % 360, -arcDeg);
        }
        double rel = deg - startDeg;
        while (rel < 0) {
            rel += 360.0;
        }
        while (rel >= 360.0) {
            rel -= 360.0;
        }
        return rel <= arcDeg;
    }

    // ---------------- 图像 ----------------

    public void drawImage(Image img, int x, int y, int anchor) {
        op("drawImage(" + img.width + "x" + img.height + "," + x + "," + y + "," + anchor + ")");
        int[] adj = anchorAdjust(anchor, img.width, img.height);
        blit(img, x - adj[0], y - adj[1], false);
    }

    /**
     * Nokia DirectGraphics 变换绘制（com.nokia.mid.ui.DirectGraphics.drawImage 的
     * shim 内部实现入口；transform 码沿用 Nokia DirectGraphics 常量）。
     */
    public void drawImageTransformed(Image img, int x, int y, int anchor, int transform) {
        op("drawImageT(" + img.width + "x" + img.height + "," + x + "," + y + "," + anchor
                + "," + transform + ")");
        boolean swap = transform == 4 || transform == 5 || transform == 6 || transform == 7;
        int bw = swap ? img.height : img.width;
        int bh = swap ? img.width : img.height;
        int[] adj = anchorAdjust(anchor, bw, bh);
        int ox = x - adj[0];
        int oy = y - adj[1];
        int w = img.width, h = img.height;
        for (int sy = 0; sy < h; sy++) {
            for (int sx = 0; sx < w; sx++) {
                int p = img.argb[sy * w + sx];
                if (((p >>> 24) & 0xFF) == 0) {
                    continue;
                }
                int dx, dy;
                switch (transform) {
                    case 1: dx = sx; dy = h - 1 - sy; break;             // MIRROR_ROT180
                    case 2: dx = w - 1 - sx; dy = sy; break;             // MIRROR
                    case 3: dx = w - 1 - sx; dy = h - 1 - sy; break;     // ROT180
                    case 4: dx = sy; dy = sx; break;                     // MIRROR_ROT270
                    case 5: dx = h - 1 - sy; dy = sx; break;             // ROT90
                    case 6: dx = sy; dy = w - 1 - sx; break;             // ROT270
                    case 7: dx = h - 1 - sy; dy = w - 1 - sx; break;     // MIRROR_ROT90
                    default: dx = sx; dy = sy; break;                    // NONE
                }
                setPixel(ox + dx, oy + dy, p);
            }
        }
    }

    void blit(Image img, int x, int y, boolean transpose) {
        for (int sy = 0; sy < img.height; sy++) {
            for (int sx = 0; sx < img.width; sx++) {
                int p = img.argb[sy * img.width + sx];
                if (((p >>> 24) & 0xFF) == 0) {
                    continue;
                }
                int dx = transpose ? x + sy : x + sx;
                int dy = transpose ? y + sx : y + sy;
                setPixel(dx, dy, p);
            }
        }
    }

    private static int[] anchorAdjust(int anchor, int w, int h) {
        int ax = 0, ay = 0;
        if ((anchor & HCENTER) != 0) {
            ax = w / 2;
        } else if ((anchor & RIGHT) != 0) {
            ax = w;
        }
        if ((anchor & VCENTER) != 0) {
            ay = h / 2;
        } else if ((anchor & BOTTOM) != 0) {
            ay = h;
        }
        return new int[]{ax, ay};
    }

    // ---------------- 文字 ----------------

    public void drawChar(char ch, int x, int y, int anchor) {
        op("drawChar(" + (int) ch + "," + x + "," + y + "," + anchor + ")");
        drawGlyph(ch, x, y, anchor);
    }

    public void drawString(String str, int x, int y, int anchor) {
        op("drawString(" + q(str) + "," + x + "," + y + "," + anchor + ")");
        drawText(str, 0, str.length(), x, y, anchor);
    }

    public void drawSubstring(String str, int offset, int len, int x, int y, int anchor) {
        op("drawSubstring(" + q(str) + "," + offset + "," + len + "," + x + "," + y + "," + anchor + ")");
        drawText(str, offset, offset + len, x, y, anchor);
    }

    private void drawText(String str, int from, int to, int x, int y, int anchor) {
        int w = font.substringWidth(str, from, to - from);
        int h = font.getHeight();
        int[] adj = anchorAdjust(anchor, w, h);
        if ((anchor & BASELINE) != 0) {
            adj[1] = font.getBaselinePosition();
        }
        int px = x - adj[0];
        int py = y - adj[1];
        for (int i = from; i < to; i++) {
            char ch = str.charAt(i);
            drawGlyph(ch, px, py, TOP | LEFT);
            px += font.charWidth(ch);
        }
    }

    private void drawGlyph(char ch, int x, int y, int anchor) {
        int cw = font.charWidth(ch);
        int chh = font.getHeight();
        int[] adj = anchorAdjust(anchor, cw, chh);
        if ((anchor & BASELINE) != 0) {
            adj[1] = font.getBaselinePosition();
        }
        int px = x - adj[0];
        int py = y - adj[1];
        int c = opaqueColor();
        // 确定性示意字形：字符码哈希派生 5x5 点阵，占字符格 80%
        int gw = Math.max(2, cw * 4 / 5);
        int gh = Math.max(3, chh * 3 / 5);
        int hash = (int) (ch * 2654435761L);
        for (int gy = 0; gy < gh; gy++) {
            for (int gx = 0; gx < gw; gx++) {
                int bit = (hash >>> ((gx + gy * 5) % 29)) & 1;
                if (bit != 0) {
                    setPixel(px + (cw - gw) / 2 + gx, py + (chh - gh) / 2 + gy, c);
                }
            }
        }
    }
}
