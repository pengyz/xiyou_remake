package javax.microedition.lcdui;

/**
 * MIDP-1.0 {@code Displayable} 最小 shim（被 Canvas 继承）。
 */
public abstract class Displayable {
    String title;

    protected Displayable() {
    }

    public String getTitle() {
        return title;
    }

    public void setTitle(String t) {
        this.title = t;
    }

    public boolean isShown() {
        return true;
    }
}
