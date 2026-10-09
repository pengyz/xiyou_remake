package oracle.host;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.util.zip.CRC32;
import java.util.zip.Deflater;

/**
 * 确定性 PNG 编码器（RGBA8，filter 0，Deflater 固定级别）：截图产出用。
 * trace 只含像素 SHA-256，PNG 字节不参与比对，但同输入两次编码结果亦相同。
 */
public final class PngWriter {
    private PngWriter() {
    }

    public static void write(File file, int width, int height, int[] argb) throws IOException {
        byte[] raw = new byte[height * (1 + width * 4)];
        int p = 0;
        for (int y = 0; y < height; y++) {
            raw[p++] = 0; // filter: None
            for (int x = 0; x < width; x++) {
                int v = argb[y * width + x];
                raw[p++] = (byte) (v >>> 24);
                raw[p++] = (byte) (v >>> 16);
                raw[p++] = (byte) (v >>> 8);
                raw[p++] = (byte) v;
            }
        }
        Deflater def = new Deflater(Deflater.BEST_COMPRESSION, false);
        def.setInput(raw);
        def.finish();
        byte[] comp = new byte[raw.length + 256];
        int clen = 0;
        while (!def.finished()) {
            clen += def.deflate(comp, clen, comp.length - clen);
        }
        def.end();

        OutputStream out = new FileOutputStream(file);
        try {
            out.write(new byte[]{(byte) 0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n'});
            byte[] ihdr = new byte[13];
            put32(ihdr, 0, width);
            put32(ihdr, 4, height);
            ihdr[8] = 8;   // bit depth
            ihdr[9] = 6;   // color type RGBA
            ihdr[10] = 0;
            ihdr[11] = 0;
            ihdr[12] = 0;
            chunk(out, "IHDR", ihdr);
            chunk(out, "IDAT", java.util.Arrays.copyOf(comp, clen));
            chunk(out, "IEND", new byte[0]);
        } finally {
            out.close();
        }
    }

    private static void put32(byte[] b, int off, int v) {
        b[off] = (byte) (v >>> 24);
        b[off + 1] = (byte) (v >>> 16);
        b[off + 2] = (byte) (v >>> 8);
        b[off + 3] = (byte) v;
    }

    private static void chunk(OutputStream out, String type, byte[] data) throws IOException {
        byte[] head = new byte[4];
        put32(head, 0, data.length);
        out.write(head);
        byte[] t = type.getBytes(StandardCharsets.US_ASCII);
        out.write(t);
        out.write(data);
        CRC32 crc = new CRC32();
        crc.update(t);
        crc.update(data);
        byte[] c = new byte[4];
        put32(c, 0, (int) crc.getValue());
        out.write(c);
    }
}
