package oracle.vt;

/**
 * 停机信号：tick 预算耗尽或 MIDlet 销毁时由 oracle 钩子抛出。
 * 继承 {@link Error}：穿过参考版 run() 的 {@code catch (Exception)} （a.java:2430）
 * 直达调度器，属预期正常终止路径。
 */
public class StopSignal extends Error {
    private static final long serialVersionUID = 1L;

    public StopSignal(String msg) {
        super(msg);
    }
}
