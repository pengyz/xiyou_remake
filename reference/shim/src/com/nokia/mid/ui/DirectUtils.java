package com.nokia.mid.ui;

import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

/**
 * Nokia UI API {@code com.nokia.mid.ui.DirectUtils} 最小 shim。
 * 参考版仅引用 getDirectGraphics(Graphics)。
 */
public final class DirectUtils {
    private DirectUtils() {
    }

    public static DirectGraphics getDirectGraphics(Graphics g) {
        return new DirectGraphicsImpl(g);
    }

    public static Image createImage(int[] argb, int width, int height, int alpha) {
        int[] buf = new int[argb.length];
        for (int i = 0; i < argb.length; i++) {
            int v = argb[i];
            buf[i] = (alpha & 0xFF) << 24 | v & 0xFFFFFF;
        }
        return Image.createRGBImage(buf, width, height, true);
    }

    private static final class DirectGraphicsImpl implements DirectGraphics {
        private final Graphics g;

        DirectGraphicsImpl(Graphics g) {
            this.g = g;
        }

        public void drawImage(Image image, int x, int y, int anchor, int transform) {
            g.drawImageTransformed(image, x, y, anchor, transform);
        }

        public Graphics getGraphics() {
            return g;
        }
    }
}
