package javax.microedition.lcdui;

/**
 * MIDP-1.0 {@code javax.microedition.lcdui.Canvas} 自研 shim。
 *
 * <p>参考版实际引用面：&lt;init&gt; / hideNotify() / repaint() /
 * serviceRepaints() / setFullScreenMode(boolean) / showNotify()，
 * 以及子类覆盖的 paint(Graphics) / keyPressed(int) / keyReleased(int)。
 *
 * <p><b>tick 边界 = serviceRepaints()</b>：原版主循环每帧
 * {@code repaint(); serviceRepaints();}（a.java:2421-2422），本 shim 在
 * serviceRepaints() 内 ① 触发 paint() 到离屏画布 ② 采集帧哈希 ③ 在
 * {@link Hooks#preTick} 返回的输入事件上回调 keyPressed/keyReleased
 * ④ 调 {@link Hooks#postTick} 记录状态向量（oracle trace 宿主注入）。
 * 输入在 tick 边界投递 ⇒ 完全确定性（单线程协作）。
 */
public abstract class Canvas extends Displayable {
    /** 逻辑屏幕尺寸（原版硬编码 240x320，见 a.java:645）。 */
    public static final int SCREEN_W = 240;
    public static final int SCREEN_H = 320;

    /** oracle trace 宿主注入的钩子（shim 不依赖 oracle 包）。 */
    public interface Hooks {
        /** paint 完成后调用；screen 为离屏渲染结果。 */
        void afterPaint(Image screen);

        /** tick 开始（输入投递前）调用；返回 {code,type,code,type,...}，type:1=press 0=release。 */
        int[] preTick(Canvas c);

        /** tick 结束时调用（可抛 oracle StopSignal 终止运行）。 */
        void postTick(Canvas c);
    }

    public static Hooks hooks;

    final Image screen = Image.createImage(SCREEN_W, SCREEN_H);
    private boolean needRepaint;

    protected Canvas() {
    }

    protected void paint(Graphics g) {
    }

    protected void keyPressed(int keyCode) {
    }

    protected void keyReleased(int keyCode) {
    }

    protected void keyRepeated(int keyCode) {
    }

    protected void showNotify() {
    }

    protected void hideNotify() {
    }

    protected void sizeChanged(int w, int h) {
    }

    public void repaint() {
        needRepaint = true;
    }

    public void repaint(int x, int y, int w, int h) {
        needRepaint = true;
    }

    public void serviceRepaints() {
        if (needRepaint) {
            needRepaint = false;
            Graphics g = screen.getGraphics();
            g.setClip(0, 0, SCREEN_W, SCREEN_H);
            paint(g);
            if (hooks != null) {
                hooks.afterPaint(screen);
            }
        }
        if (hooks != null) {
            int[] ev = hooks.preTick(this);
            if (ev != null) {
                for (int i = 0; i + 1 < ev.length; i += 2) {
                    if (ev[i + 1] == 1) {
                        keyPressed(ev[i]);
                    } else {
                        keyReleased(ev[i]);
                    }
                }
            }
            hooks.postTick(this);
        }
    }

    public void setFullScreenMode(boolean mode) {
    }

    public int getWidth() {
        return SCREEN_W;
    }

    public int getHeight() {
        return SCREEN_H;
    }

    public int getGameAction(int keyCode) {
        return keyCode;
    }

    public int getKeyCode(int gameAction) {
        return gameAction;
    }
}
