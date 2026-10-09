package javax.microedition.media;

import java.io.InputStream;

import javax.microedition.media.control.VolumeControl;

/**
 * MMAPI {@code javax.microedition.media.Manager} shim。
 * 参考版仅引用 createPlayer(InputStream,String)。
 *
 * <p>策略（oracle 确定性约定）：createPlayer 返回可变状态机的 Player stub
 * （UNREALIZED→REALIZED→PREFETCHED→STARTED），<b>不产生任何音频输出</b>；
 * 音频请求记入 oracle trace 事件流。输入流不消费（由调用方自行关闭）。
 */
public final class Manager {
    /** oracle 诊断事件收集（确定性、按发生顺序）。 */
    public static final java.util.List<String> EVENTS = new java.util.ArrayList<String>();

    private Manager() {
    }

    public static Player createPlayer(InputStream stream, String type) {
        EVENTS.add("createPlayer(" + (type == null ? "null" : type) + ")");
        return new PlayerStub();
    }

    public static Player createPlayer(String locator) {
        EVENTS.add("createPlayer(locator=" + locator + ")");
        return new PlayerStub();
    }

    static final class PlayerStub implements Player {
        private int state = UNREALIZED;
        private int loopCount = 1;
        private final VolumeControlStub volume = new VolumeControlStub();

        public void realize() {
            EVENTS.add("realize->" + Math.max(state, REALIZED));
            state = Math.max(state, REALIZED);
        }

        public void prefetch() {
            EVENTS.add("prefetch->" + Math.max(state, PREFETCHED));
            state = Math.max(state, PREFETCHED);
        }

        public void start() {
            EVENTS.add("start(state=" + state + ")");
            state = STARTED;
        }

        public void stop() {
            EVENTS.add("stop(state=" + state + ")");
            state = state == CLOSED ? CLOSED : PREFETCHED;
        }

        public void close() {
            EVENTS.add("close(state=" + state + ")");
            state = CLOSED;
        }

        public int getState() {
            return state;
        }

        public void setLoopCount(int count) {
            EVENTS.add("setLoopCount(" + count + ")");
            this.loopCount = count;
        }

        public Control getControl(String controlType) {
            if (controlType != null && controlType.endsWith("VolumeControl")) {
                return volume;
            }
            EVENTS.add("getControl(" + controlType + ")=null");
            return null;
        }
    }

    static final class VolumeControlStub implements VolumeControl {
        private int level = 80;
        private boolean mute;

        public int setLevel(int level) {
            EVENTS.add("setLevel(" + level + ")");
            this.level = Math.max(0, Math.min(100, level));
            return this.level;
        }

        public int getLevel() {
            return level;
        }

        public boolean isMuted() {
            return mute;
        }

        public void setMute(boolean mute) {
            this.mute = mute;
        }
    }
}
