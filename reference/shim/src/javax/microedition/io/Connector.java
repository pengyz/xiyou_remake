package javax.microedition.io;

import java.io.IOException;

/**
 * GCF {@code javax.microedition.io.Connector} shim。
 *
 * <p><b>oracle 策略（确定性）：网络永久关闭</b>——{@code open(String)} 一律抛
 * {@link IOException}，驱动参考版走其离线/失败分支（a.java:8321 catch → n=-1）。
 * 事件记入 {@link #EVENTS} 供 trace 采集。未来需要联网场景时再扩展为
 * 确定性假响应，属 oracle 策略层，不改参考版。
 */
public final class Connector {
    public static final java.util.List<String> EVENTS = new java.util.ArrayList<String>();

    public static final int READ = 1;
    public static final int WRITE = 2;
    public static final int READ_WRITE = 3;

    private Connector() {
    }

    public static Connection open(String name) throws IOException {
        EVENTS.add("open(" + name + ")");
        throw new IOException("oracle shim: network disabled");
    }

    public static Connection open(String name, int mode) throws IOException {
        return open(name);
    }

    public static Connection open(String name, int mode, boolean timeouts) throws IOException {
        return open(name);
    }

    public static java.io.InputStream openInputStream(String name) throws IOException {
        return ((InputConnection) open(name)).openInputStream();
    }

    public static java.io.OutputStream openOutputStream(String name) throws IOException {
        return ((OutputConnection) open(name)).openOutputStream();
    }
}
