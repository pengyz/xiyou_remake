package javax.microedition.midlet;

/**
 * MIDP-1.0 {@code javax.microedition.midlet.MIDlet} 最小 shim。
 * 参考版仅引用 &lt;init&gt; / notifyDestroyed()，并覆盖 startApp/pauseApp/destroyApp。
 */
public abstract class MIDlet {
    /** notifyDestroyed() 被调用的次数（oracle 用作终止条件之一）。 */
    public static volatile boolean destroyed;
    public static volatile int destroyCount;

    protected MIDlet() {
    }

    protected abstract void startApp();

    protected abstract void pauseApp();

    protected abstract void destroyApp(boolean unconditional);

    public final void notifyDestroyed() {
        destroyed = true;
        destroyCount++;
    }

    public final void notifyPaused() {
    }

    public final void resumeRequest() {
    }

    public final String getAppProperty(String key) {
        return null;
    }

    public final boolean platformRequest(String url) {
        return false;
    }

    public final int checkPermission(String name) {
        return 1;
    }
}
