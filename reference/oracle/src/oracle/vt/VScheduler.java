package oracle.vt;

import java.util.ArrayDeque;
import java.util.Deque;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeUnit;

/**
 * 虚拟线程协作调度器：全局单执行槽 + FIFO 就绪队列。
 *
 * <p>规则（全部确定性）：
 * <ul>
 *   <li>{@code start()} 只是登记（载波线程立即阻塞在自己的信号量上）；
 *       宿主线程跑完 MIDlet 启动（startApp/setCurrent/showNotify）后
 *       {@link #boot()} 才授权第一个虚拟线程；</li>
 *   <li>yield/sleep 处：若队列有其他就绪线程则 FIFO 轮转，否则继续持有；</li>
 *   <li>线程结束让出槽位给下一个；全部结束时 {@link #awaitCompletion} 返回。</li>
 * </ul>
 *
 * <p>已知限制（记录在 _diff/report.md）：若游戏代码在持有 Java monitor 时
 * 进入切换点，可能出现协作调度死锁；本游戏 yield/sleep 调用点核查过均在
 * synchronized 块外（a.java:1363/2424/2903/8349），且宿主有真实时间看门狗，
 * 死锁会显式失败而非产出错误 trace。
 */
public final class VScheduler {
    public interface Events {
        void onThreadStart(String name);

        void onThreadExit(String name);

        void onThreadFail(String name, Throwable t);
    }

    public static volatile Events events;

    private static final Object LOCK = new Object();
    private static final Deque<Carrier> READY = new ArrayDeque<Carrier>();
    private static final CountDownLatch DONE = new CountDownLatch(1);
    private static Carrier current;
    private static int alive;
    private static volatile boolean failed;
    private static volatile String failDetail = "";

    private VScheduler() {
    }

    static final class Carrier implements Runnable {
        final Semaphore turn = new Semaphore(0);
        final Runnable target;
        final String name;

        Carrier(Runnable target, String name) {
            this.target = target;
            this.name = name;
        }

        public void run() {
            Throwable err = null;
            try {
                turn.acquire();
                if (events != null) {
                    events.onThreadStart(name);
                }
                if (target != null) {
                    target.run();
                }
            } catch (StopSignal stop) {
                // 正常终止路径
            } catch (Throwable t) {
                err = t;
                failed = true;
                failDetail = name + ": " + t;
                if (events != null) {
                    events.onThreadFail(name, t);
                }
            } finally {
                if (err == null && events != null) {
                    events.onThreadExit(name);
                }
                death(this);
            }
        }
    }

    static void spawn(Runnable target, String name) {
        Carrier c = new Carrier(target, name);
        synchronized (LOCK) {
            alive++;
            READY.add(c);
        }
        Thread t = new Thread(c, name);
        t.setDaemon(true);
        t.start();
    }

    /** 宿主启动完成后授权执行（startApp 已返回）。 */
    public static void boot() {
        synchronized (LOCK) {
            grantNextLocked();
        }
    }

    public static boolean awaitCompletion(long timeoutMs) throws InterruptedException {
        return DONE.await(timeoutMs, TimeUnit.MILLISECONDS);
    }

    public static boolean hasFailed() {
        return failed;
    }

    public static String failDetail() {
        return failDetail;
    }

    static void yieldPoint() {
        Carrier me = current;
        if (me == null) {
            return; // 非虚拟线程上下文（宿主线程）：无切换语义
        }
        synchronized (LOCK) {
            if (READY.isEmpty()) {
                return; // 无其他就绪线程：继续持有槽位
            }
            Carrier next = READY.poll();
            READY.addLast(me);
            current = next;
            next.turn.release();
        }
        me.turn.acquireUninterruptibly();
    }

    private static void death(Carrier me) {
        synchronized (LOCK) {
            alive--;
            if (current == me) {
                current = null;
            }
            if (alive == 0) {
                DONE.countDown();
            } else {
                grantNextLocked();
            }
        }
    }

    private static void grantNextLocked() {
        if (current == null && !READY.isEmpty()) {
            current = READY.poll();
            current.turn.release();
        }
    }
}
