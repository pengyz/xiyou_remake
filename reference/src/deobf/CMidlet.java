/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  javax.microedition.lcdui.Display
 *  javax.microedition.lcdui.Displayable
 *  javax.microedition.midlet.MIDlet
 */
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Displayable;
import javax.microedition.midlet.MIDlet;

public class CMidlet
extends MIDlet {
    private static CMidlet f_CMidlet_00;
    private a f_a_00;

    public CMidlet() {
        f_CMidlet_00 = this;
    }

    protected void startApp() {
        if (this.f_a_00 == null) {
            this.f_a_00 = new a();
            new Thread(this.f_a_00).start();
        }
        Display.getDisplay((MIDlet)f_CMidlet_00).setCurrent((Displayable)this.f_a_00);
        this.f_a_00.showNotify();
    }

    protected void pauseApp() {
        this.f_a_00.hideNotify();
    }

    protected void destroyApp(boolean bl) {
    }

    public static void m_000() {
        f_CMidlet_00.destroyApp(true);
        f_CMidlet_00.notifyDestroyed();
        f_CMidlet_00 = null;
    }
}
