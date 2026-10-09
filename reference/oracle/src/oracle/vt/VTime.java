package oracle.vt;

import java.io.PrintStream;

/**
 * 虚拟时钟（T-变换目标：字节码把 {@code java/lang/System} 的
 * currentTimeMillis/gc/out 引用重定向到本类，见
 * {@code reference/oracle/ttransform.py}）。
 *
 * <p>确定性约定：
 * <ul>
 *   <li>{@link #currentTimeMillis()} 纯读，返回虚拟毫秒；</li>
 *   <li>虚拟时间只在 {@link VThread#yield()}（+1ms）与 {@link VThread#sleep(long)}
 *       （+ms）处推进 —— 原版帧限速忙等
 *       {@code do { Thread.yield(); } while (now < t0+frameBudget);}
 *       （a.java:2424-2425）因此每帧确定性地空转 frameBudget 次后退出；</li>
 *   <li>{@link #gc()} 为 no-op（消除 GC 时序噪声，行为无语义影响）。</li>
 * </ul>
 */
public final class VTime {
    private static long now = 0;

    /** T-变换目标：System.out 的替身（打印进入 trace 事件流）。 */
    public static PrintStream out;

    private VTime() {
    }

    public static synchronized long currentTimeMillis() {
        return now;
    }

    public static synchronized void advance(long ms) {
        now += ms;
    }

    public static synchronized void reset() {
        now = 0;
    }

    public static void gc() {
        // no-op（oracle 确定性约定）
    }
}
