package oracle.host;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;

/**
 * trace 文本输出：一行一记录，A/B 两变体逐字节可比（不含任何符号名/变体标识）。
 */
public final class TraceSink {
    private static PrintWriter writer;
    private static File outFile;

    private TraceSink() {
    }

    public static synchronized void open(File file) throws IOException {
        outFile = file;
        writer = new PrintWriter(new OutputStreamWriter(new FileOutputStream(file), StandardCharsets.UTF_8));
    }

    public static synchronized void line(String s) {
        if (writer != null) {
            writer.println(s);
        }
    }

    public static synchronized void close() {
        if (writer != null) {
            writer.flush();
            writer.close();
            writer = null;
        }
    }

    public static File file() {
        return outFile;
    }

    /** JSON 风格字符串转义（trace 内字符串值用）。 */
    public static String quote(String s) {
        if (s == null) {
            return "null";
        }
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

    public static String hex(byte[] b, int len) {
        StringBuilder sb = new StringBuilder(len * 2);
        for (int i = 0; i < len; i++) {
            sb.append(String.format("%02x", b[i]));
        }
        return sb.toString();
    }
}
