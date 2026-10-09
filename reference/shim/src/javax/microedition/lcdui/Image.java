package javax.microedition.lcdui;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.zip.Inflater;

/**
 * MIDP-1.0 {@code javax.microedition.lcdui.Image} 自研 shim。
 *
 * <p>仅实现参考版字节码实际引用的 API 面（constant-pool 扫描见
 * {@code reference/oracle/_diff/api-surface.md}）：
 * createImage(int,int) / createImage(String) / createImage(byte[],int,int) /
 * createRGBImage(int[],int,int,boolean) / getGraphics() / getWidth() /
 * getHeight() / getRGB(int[],int,int,int,int,int,int)。
 *
 * <p>像素模型：非预乘 ARGB {@code int[]}（0xAARRGGBB）。PNG 解码器为纯确定性实现
 * （java.util.zip.Inflater 仅做 zlib 解压）：支持原版 JAR 中实际出现的
 * color-type 3（调色板）bit depth 1/2/4/8 + tRNS，以及防御性的
 * color-type 0/2/4/6 depth 8，非隔行扫描。资源盘点证据见
 * {@code reference/oracle/_diff/report.md}（164 个 PNG 签名全部为 (depth,3,0)）。
 */
public class Image {
    /** 像素缓冲：width*height，0xAARRGGBB。null 表示可变离屏图尚未分配（见 mutable）。 */
    final int[] argb;
    final int width;
    final int height;

    private Image(int w, int h, int[] buf) {
        this.width = w;
        this.height = h;
        this.argb = buf;
    }

    public static Image createImage(int width, int height) {
        return new Image(width, height, new int[width * height]);
    }

    public static Image createImage(String name) throws IOException {
        InputStream in = Image.class.getResourceAsStream(name);
        if (in == null) {
            in = Image.class.getResourceAsStream("/" + name);
        }
        if (in == null) {
            throw new IOException("resource not found: " + name);
        }
        try {
            byte[] data = readAll(in);
            return decodePng(data, 0, data.length);
        } finally {
            in.close();
        }
    }

    public static Image createImage(byte[] imageData, int imageOffset, int imageLength) {
        return decodePng(imageData, imageOffset, imageLength);
    }

    public static Image createRGBImage(int[] rgb, int width, int height, boolean processAlpha) {
        int[] buf = new int[width * height];
        for (int i = 0; i < buf.length; i++) {
            int v = rgb[i];
            buf[i] = processAlpha ? v : (v | 0xFF000000);
        }
        return new Image(width, height, buf);
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public Graphics getGraphics() {
        return new Graphics(this);
    }

    public void getRGB(int[] rgbData, int offset, int scanlength, int x, int y, int width, int height) {
        for (int row = 0; row < height; row++) {
            int src = (y + row) * this.width + x;
            int dst = offset + row * scanlength;
            System.arraycopy(argb, src, rgbData, dst, width);
        }
    }

    // ------------------------------------------------------------------
    // PNG 解码
    // ------------------------------------------------------------------

    private static Image decodePng(byte[] data, int off, int len) {
        if (off < 0 || len < 8 || off + len > data.length) {
            throw new IllegalArgumentException("bad png range");
        }
        int p = off;
        int end = off + len;
        byte[] sig = {(byte) 0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n'};
        for (int i = 0; i < 8; i++) {
            if (data[p + i] != sig[i]) {
                throw new IllegalArgumentException("not a PNG");
            }
        }
        p += 8;
        int w = 0, h = 0, bitDepth = 0, colorType = 0, interlace = 0;
        byte[] plte = null;
        byte[] trns = null;
        ByteArrayOutputStream idat = new ByteArrayOutputStream(1 << 16);
        while (p + 8 <= end) {
            int chunkLen = be32(data, p);
            String type = new String(data, p + 4, 4, java.nio.charset.StandardCharsets.US_ASCII);
            int body = p + 8;
            if (body + chunkLen + 4 > end) {
                throw new IllegalArgumentException("png chunk overrun: " + type);
            }
            if (type.equals("IHDR")) {
                w = be32(data, body);
                h = be32(data, body + 4);
                bitDepth = data[body + 8] & 0xFF;
                colorType = data[body + 9] & 0xFF;
                interlace = data[body + 12] & 0xFF;
                if (interlace != 0) {
                    throw new IllegalArgumentException("interlaced PNG unsupported");
                }
            } else if (type.equals("PLTE")) {
                plte = new byte[chunkLen];
                System.arraycopy(data, body, plte, 0, chunkLen);
            } else if (type.equals("tRNS")) {
                trns = new byte[chunkLen];
                System.arraycopy(data, body, trns, 0, chunkLen);
            } else if (type.equals("IDAT")) {
                idat.write(data, body, chunkLen);
            } else if (type.equals("IEND")) {
                break;
            }
            p = body + chunkLen + 4;
        }
        if (w <= 0 || h <= 0) {
            throw new IllegalArgumentException("png missing IHDR");
        }
        byte[] raw = zlibInflate(idat.toByteArray());
        int[] buf = new int[w * h];
        unfilterDecode(raw, buf, w, h, bitDepth, colorType, plte, trns);
        return new Image(w, h, buf);
    }

    private static void unfilterDecode(byte[] raw, int[] out, int w, int h, int bitDepth,
                                      int colorType, byte[] plte, byte[] trns) {
        int channels;
        switch (colorType) {
            case 0: channels = 1; break;   // 灰度
            case 2: channels = 3; break;   // 真彩
            case 3: channels = 1; break;   // 调色板
            case 4: channels = 2; break;   // 灰度+alpha
            case 6: channels = 4; break;   // 真彩+alpha
            default: throw new IllegalArgumentException("png color-type " + colorType);
        }
        if (colorType == 3 && plte == null) {
            throw new IllegalArgumentException("palette PNG missing PLTE");
        }
        int bpp = Math.max(1, (channels * bitDepth) / 8);   // filter 字节步长（按字节）
        int rowBytes = (w * channels * bitDepth + 7) / 8;
        byte[] prev = new byte[rowBytes];
        byte[] cur = new byte[rowBytes];
        int pos = 0;
        for (int y = 0; y < h; y++) {
            if (pos + 1 + rowBytes > raw.length) {
                throw new IllegalArgumentException("png data truncated");
            }
            int filter = raw[pos++] & 0xFF;
            System.arraycopy(raw, pos, cur, 0, rowBytes);
            pos += rowBytes;
            switch (filter) {
                case 0: break;
                case 1:
                    for (int i = bpp; i < rowBytes; i++) cur[i] = (byte) (cur[i] + cur[i - bpp]);
                    break;
                case 2:
                    for (int i = 0; i < rowBytes; i++) cur[i] = (byte) (cur[i] + prev[i]);
                    break;
                case 3:
                    for (int i = 0; i < rowBytes; i++) {
                        int left = i >= bpp ? (cur[i - bpp] & 0xFF) : 0;
                        cur[i] = (byte) (cur[i] + ((left + (prev[i] & 0xFF)) >> 1));
                    }
                    break;
                case 4:
                    for (int i = 0; i < rowBytes; i++) {
                        int a = i >= bpp ? (cur[i - bpp] & 0xFF) : 0;
                        int b = prev[i] & 0xFF;
                        int c = i >= bpp ? (prev[i - bpp] & 0xFF) : 0;
                        int pp = a + b - c;
                        int pa = Math.abs(pp - a), pb = Math.abs(pp - b), pc = Math.abs(pp - c);
                        int pred = (pa <= pb && pa <= pc) ? a : (pb <= pc ? b : c);
                        cur[i] = (byte) (cur[i] + pred);
                    }
                    break;
                default:
                    throw new IllegalArgumentException("png filter " + filter);
            }
            for (int x = 0; x < w; x++) {
                out[y * w + x] = pixelAt(cur, x, bitDepth, colorType, plte, trns);
            }
            byte[] tmp = prev;
            prev = cur;
            cur = tmp;
        }
    }

    private static int pixelAt(byte[] row, int x, int bitDepth, int colorType,
                               byte[] plte, byte[] trns) {
        if (colorType == 3) {
            int idx = sampleIndex(row, x, bitDepth);
            if (idx * 3 + 2 >= plte.length) {
                throw new IllegalArgumentException("palette index out of range");
            }
            int r = plte[idx * 3] & 0xFF, g = plte[idx * 3 + 1] & 0xFF, b = plte[idx * 3 + 2] & 0xFF;
            int a = (trns != null && idx < trns.length) ? (trns[idx] & 0xFF) : 0xFF;
            return (a << 24) | (r << 16) | (g << 8) | b;
        }
        if (bitDepth != 8) {
            throw new IllegalArgumentException("png bit-depth " + bitDepth + " unsupported for color-type " + colorType);
        }
        switch (colorType) {
            case 0: {
                int g = row[x] & 0xFF;
                int a = (trns != null && trns.length >= 2 && ((trns[0] & 0xFF) << 8 | (trns[1] & 0xFF)) == g) ? 0 : 0xFF;
                return (a << 24) | (g << 16) | (g << 8) | g;
            }
            case 2: {
                int o = x * 3;
                int r = row[o] & 0xFF, g = row[o + 1] & 0xFF, b = row[o + 2] & 0xFF;
                int a = 0xFF;
                if (trns != null && trns.length >= 6) {
                    int tr = (trns[0] & 0xFF) << 8 | (trns[1] & 0xFF);
                    int tg = (trns[2] & 0xFF) << 8 | (trns[3] & 0xFF);
                    int tb = (trns[4] & 0xFF) << 8 | (trns[5] & 0xFF);
                    if (tr == r && tg == g && tb == b) a = 0;
                }
                return (a << 24) | (r << 16) | (g << 8) | b;
            }
            case 4: {
                int o = x * 2;
                int g = row[o] & 0xFF, a = row[o + 1] & 0xFF;
                return (a << 24) | (g << 16) | (g << 8) | g;
            }
            case 6: {
                int o = x * 4;
                int r = row[o] & 0xFF, g = row[o + 1] & 0xFF, b = row[o + 2] & 0xFF, a = row[o + 3] & 0xFF;
                return (a << 24) | (r << 16) | (g << 8) | b;
            }
            default:
                throw new IllegalArgumentException("png color-type " + colorType);
        }
    }

    private static int sampleIndex(byte[] row, int x, int bitDepth) {
        switch (bitDepth) {
            case 8: return row[x] & 0xFF;
            case 4: return (row[x >> 1] >> ((x & 1) == 0 ? 4 : 0)) & 0x0F;
            case 2: return (row[x >> 2] >> (6 - 2 * (x & 3))) & 0x03;
            case 1: return (row[x >> 3] >> (7 - (x & 7))) & 0x01;
            default: throw new IllegalArgumentException("png bit-depth " + bitDepth);
        }
    }

    private static byte[] zlibInflate(byte[] compressed) {
        Inflater inf = new Inflater();
        inf.setInput(compressed);
        ByteArrayOutputStream out = new ByteArrayOutputStream(compressed.length * 4 + 64);
        byte[] buf = new byte[1 << 15];
        try {
            while (!inf.finished()) {
                int n = inf.inflate(buf);
                if (n == 0) {
                    if (inf.needsInput() || inf.needsDictionary()) {
                        throw new IllegalArgumentException("png zlib data truncated");
                    }
                } else {
                    out.write(buf, 0, n);
                }
            }
        } catch (java.util.zip.DataFormatException e) {
            throw new IllegalArgumentException("png zlib corrupt: " + e.getMessage());
        } finally {
            inf.end();
        }
        return out.toByteArray();
    }

    private static int be32(byte[] b, int off) {
        return ((b[off] & 0xFF) << 24) | ((b[off + 1] & 0xFF) << 16)
                | ((b[off + 2] & 0xFF) << 8) | (b[off + 3] & 0xFF);
    }

    private static byte[] readAll(InputStream in) throws IOException {
        ByteArrayOutputStream out = new ByteArrayOutputStream(1 << 14);
        byte[] buf = new byte[1 << 14];
        int n;
        while ((n = in.read(buf)) >= 0) {
            out.write(buf, 0, n);
        }
        return out.toByteArray();
    }
}
