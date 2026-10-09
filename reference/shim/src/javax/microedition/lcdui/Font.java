package javax.microedition.lcdui;

/**
 * MIDP-1.0 {@code javax.microedition.lcdui.Font} 自研 shim。
 *
 * <p>参考版仅引用：getFont(int,int,int) / charWidth(char) / getHeight() /
 * stringWidth(String) / substringWidth(String,int,int)。
 *
 * <p><b>度量为 shim 自定义常量（非设备保真）</b>：文字排版宽度会影响游戏内
 * 布局坐标，但差分比对的两端使用同一 shim，故不影响等价性判定；设备级
 * 视觉保真是 L4 视觉门禁（P4）的事。常量值记录于
 * {@code reference/oracle/_diff/report.md}。
 */
public final class Font {
    public static final int STYLE_PLAIN = 0;
    public static final int STYLE_BOLD = 1;
    public static final int STYLE_ITALIC = 2;
    public static final int STYLE_UNDERLINED = 4;

    public static final int SIZE_SMALL = 8;
    public static final int SIZE_MEDIUM = 16;
    public static final int SIZE_LARGE = 32;

    public static final int FACE_SYSTEM = 0;
    public static final int FACE_MONOSPACE = 32;
    public static final int FACE_PROPORTIONAL = 64;

    final int face;
    final int style;
    final int size;
    /** ASCII 字符步进宽（像素）。 */
    final int asciiWidth;
    /** 行高（像素）。 */
    final int height;
    /** 基线距顶部（像素）。 */
    final int baseline;

    private Font(int face, int style, int size, int asciiWidth, int height, int baseline) {
        this.face = face;
        this.style = style;
        this.size = size;
        this.asciiWidth = asciiWidth;
        this.height = height;
        this.baseline = baseline;
    }

    public static Font getFont(int face, int style, int size) {
        int aw, h, bl;
        switch (size) {
            case SIZE_SMALL:  aw = 6;  h = 10; bl = 8;  break;
            case SIZE_LARGE:  aw = 12; h = 20; bl = 16; break;
            case SIZE_MEDIUM:
            default:          aw = 8;  h = 14; bl = 11; break;
        }
        if (face == FACE_MONOSPACE) {
            aw = aw; // 与系统字体同度量（shim 简化）
        }
        if ((style & STYLE_BOLD) != 0) {
            aw += 1;
        }
        return new Font(face, style, size, aw, h, bl);
    }

    public int charWidth(char ch) {
        return isWide(ch) ? asciiWidth * 2 : asciiWidth;
    }

    public int getHeight() {
        return height;
    }

    public int getBaselinePosition() {
        return baseline;
    }

    public int stringWidth(String str) {
        if (str == null) {
            throw new NullPointerException();
        }
        int w = 0;
        for (int i = 0; i < str.length(); i++) {
            w += charWidth(str.charAt(i));
        }
        return w;
    }

    public int substringWidth(String str, int offset, int len) {
        if (str == null) {
            throw new NullPointerException();
        }
        int w = 0;
        for (int i = offset; i < offset + len; i++) {
            w += charWidth(str.charAt(i));
        }
        return w;
    }

    /** 宽字符判定：CJK / 全角区域按 2 倍 ASCII 步进。 */
    static boolean isWide(char ch) {
        return ch >= 0x2E80 && ch <= 0x9FFF      // CJK 部首/符号/统一表意
                || ch >= 0xA960 && ch <= 0xA97F
                || ch >= 0xAC00 && ch <= 0xD7FF  // 韩文音节（防御）
                || ch >= 0xF900 && ch <= 0xFAFF  // 兼容表意
                || ch >= 0xFF00 && ch <= 0xFF60  // 全角形式
                || ch >= 0xFFE0 && ch <= 0xFFE6;
    }
}
