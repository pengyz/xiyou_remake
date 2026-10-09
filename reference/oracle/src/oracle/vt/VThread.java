package oracle.vt;

/**
 * {@code java.lang.Thread} 的 T-变换替身（字节码级 Class 项重定向，
 * 见 {@code reference/oracle/ttransform.py}）。
 *
 * <p>原版仅引用 &lt;init&gt;(Runnable) / start() / sleep(long) / yield()。
 * 语义：虚拟线程经 {@link VScheduler} 协作调度——同一时刻只有一个虚拟线程
 * 执行游戏代码，切换点只在 start 授权、yield、sleep、线程结束处。
 * 消除 JVM 线程调度不确定性，是 trace 逐字节确定的前提。
 */
public final class VThread implements Runnable {
    private static long seq = 0;

    private final Runnable target;
    private final String name;

    public VThread(Runnable target) {
        this.target = target;
        synchronized (VThread.class) {
            this.name = "vt-" + (++seq);
        }
    }

    public VThread() {
        this(null);
    }

    public void start() {
        VScheduler.spawn(target, name);
    }

    public static void yield() {
        VTime.advance(1);
        VScheduler.yieldPoint();
    }

    public static void sleep(long ms) {
        VTime.advance(ms > 0 ? ms : 0);
        VScheduler.yieldPoint();
    }

    public void run() {
        if (target != null) {
            target.run();
        }
    }
}
