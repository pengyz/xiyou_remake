package javax.microedition.media;

/**
 * MMAPI {@code javax.microedition.media.Player} shim。
 * 参考版引用：close() / getState() / prefetch() / realize() / setLoopCount(int) /
 * start() / stop() + 继承 getControl(String)。
 */
public interface Player extends Controllable {
    int UNREALIZED = 100;
    int REALIZED = 200;
    int PREFETCHED = 300;
    int STARTED = 400;
    int CLOSED = 0;

    void realize();

    void prefetch();

    void start();

    void stop();

    void close();

    int getState();

    void setLoopCount(int count);
}
