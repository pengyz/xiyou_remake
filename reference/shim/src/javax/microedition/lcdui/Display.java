package javax.microedition.lcdui;

import javax.microedition.midlet.MIDlet;

/**
 * MIDP-1.0 {@code javax.microedition.lcdui.Display} 最小 shim。
 * 参考版仅引用 getDisplay(MIDlet) / setCurrent(Displayable)。
 */
public final class Display {
    private static Display instance;
    private Displayable current;

    private Display() {
    }

    public static synchronized Display getDisplay(MIDlet midlet) {
        if (instance == null) {
            instance = new Display();
        }
        return instance;
    }

    public synchronized void setCurrent(Displayable next) {
        this.current = next;
    }

    public synchronized Displayable getCurrent() {
        return current;
    }

    public boolean isColor() {
        return true;
    }

    public int numColors() {
        return 65536;
    }

    public void callSerially(Runnable r) {
        r.run();
    }
}
