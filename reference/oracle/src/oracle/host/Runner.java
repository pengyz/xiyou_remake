package oracle.host;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;

import javax.microedition.lcdui.Canvas;

import oracle.vt.VScheduler;
import oracle.vt.VTime;

/**
 * 差分 trace 宿主（Java 侧入口）。
 *
 * <p>流程：装载变体 class（classpath 上排在原版 JAR 之前）→ 构造 CMidlet →
 * 反射调 startApp()（其内部 new Thread(a).start() 仅登记虚拟线程）→
 * {@link VScheduler#boot()} 授权主循环 → 等待 tick 预算耗尽
 * （{@link oracle.vt.StopSignal}）或 MIDlet 销毁/全部虚拟线程结束。
 *
 * <p>系统属性：oracle.out（输出目录）/ oracle.max（tick 预算）/
 * oracle.script（输入脚本）/ oracle.watchdog（真实时间看门狗毫秒）/
 * oracle.label（运行标识，只进 run-info，不进 trace）。
 */
public final class Runner {
    public static void main(String[] args) throws Exception {
        final File outDir = new File(System.getProperty("oracle.out", "reference/oracle/_out/run"));
        final int maxTicks = Integer.getInteger("oracle.max", 120);
        final long watchdog = Long.getLong("oracle.watchdog", 180000L);
        final String label = System.getProperty("oracle.label", "run");
        outDir.mkdirs();

        TraceSink.open(new File(outDir, "trace.txt"));
        VTime.reset();
        VTime.out = new PrintStream2(new LineForward());
        VScheduler.events = new VScheduler.Events() {
            public void onThreadStart(String name) {
                TraceSink.line("EVT thread-start");
            }

            public void onThreadExit(String name) {
                TraceSink.line("EVT thread-exit");
            }

            public void onThreadFail(String name, Throwable t) {
                TraceSink.line("EVT thread-fail " + t.getClass().getName().replace('.', '/') + ": " + t.getMessage());
            }
        };

        int[] script = loadScript(System.getProperty("oracle.script", ""));
        final TickHooks hooks = new TickHooks(outDir, maxTicks, script);
        Canvas.hooks = hooks;

        TraceSink.line("PRE max=" + maxTicks + " script-events=" + (script.length / 3) + " screen=240x320");

        long t0 = System.nanoTime();
        boolean ok = true;
        String failMsg = "";
        try {
            Class<?> cm = Class.forName("CMidlet");
            Object midlet = cm.getDeclaredConstructor().newInstance();
            Method startApp = cm.getDeclaredMethod("startApp");
            startApp.setAccessible(true);
            startApp.invoke(midlet);
            VScheduler.boot();
            boolean done = VScheduler.awaitCompletion(watchdog);
            if (!done) {
                ok = false;
                failMsg = "watchdog timeout (" + watchdog + "ms)";
            }
            if (VScheduler.hasFailed()) {
                ok = false;
                failMsg = "virtual thread failed: " + VScheduler.failDetail();
            }
        } catch (Throwable t) {
            ok = false;
            failMsg = "host failure: " + t;
        }

        String reason = hooks.stopReason() != null ? hooks.stopReason()
                : (ok ? "all-threads-exited" : "failed");
        TraceSink.line("END reason=" + reason + " ticks=" + hooks.tickCount());
        TraceSink.close();

        long ms = (System.nanoTime() - t0) / 1000000L;
        StringBuilder info = new StringBuilder();
        info.append("label=").append(label).append('\n');
        info.append("ok=").append(ok).append('\n');
        info.append("reason=").append(reason).append('\n');
        info.append("ticks=").append(hooks.tickCount()).append('\n');
        info.append("real_ms=").append(ms).append('\n');
        if (!ok) {
            info.append("fail=").append(failMsg).append('\n');
        }
        Files.write(new File(outDir, "run-info.txt").toPath(), info.toString().getBytes(StandardCharsets.UTF_8));

        System.exit(ok ? 0 : 2);
    }

    /** shim 事件流（Manager/Connector）与 System.out 一起按发生顺序并入 trace。 */
    static void flushEvents() {
        List<String> evts = new ArrayList<String>();
        synchronized (javax.microedition.media.Manager.EVENTS) {
            evts.addAll(javax.microedition.media.Manager.EVENTS);
            javax.microedition.media.Manager.EVENTS.clear();
        }
        synchronized (javax.microedition.io.Connector.EVENTS) {
            evts.addAll(javax.microedition.io.Connector.EVENTS);
            javax.microedition.io.Connector.EVENTS.clear();
        }
        for (String e : evts) {
            TraceSink.line("EVT " + e);
        }
    }

    private static int[] loadScript(String path) throws IOException {
        if (path == null || path.isEmpty()) {
            return new int[0];
        }
        List<Integer> ev = new ArrayList<Integer>();
        for (String line : Files.readAllLines(new File(path).toPath(), StandardCharsets.UTF_8)) {
            String s = line.trim();
            if (s.isEmpty() || s.startsWith("#")) {
                continue;
            }
            String[] parts = s.split("\\s+");
            // <tick> <keycode> <press|release>
            ev.add(Integer.parseInt(parts[0]));
            ev.add(Integer.parseInt(parts[1]));
            ev.add(parts[2].equals("press") ? 1 : 0);
        }
        int[] arr = new int[ev.size()];
        for (int i = 0; i < arr.length; i++) {
            arr[i] = ev.get(i);
        }
        return arr;
    }

    /** UTF-8 PrintStream（把游戏的 System.out.println 并入 trace）。 */
    static final class PrintStream2 extends java.io.PrintStream {
        PrintStream2(OutputStream out) throws java.io.UnsupportedEncodingException {
            super(out, true, "UTF-8");
        }
    }

    static final class LineForward extends OutputStream {
        private final ByteArrayOutputStream buf = new ByteArrayOutputStream();

        public synchronized void write(int b) {
            if (b == '\n') {
                TraceSink.line("OUT " + new String(buf.toByteArray(), StandardCharsets.UTF_8));
                buf.reset();
            } else if (b != '\r') {
                buf.write(b);
            }
        }

        public synchronized void write(byte[] b, int off, int len) {
            for (int i = 0; i < len; i++) {
                write(b[off + i]);
            }
        }
    }
}
