import javax.microedition.lcdui.Display;
import javax.microedition.midlet.MIDlet;

public class CMidlet extends MIDlet {
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

      Display.getDisplay(f_CMidlet_00).setCurrent(this.f_a_00);
      this.f_a_00.showNotify();
   }

   protected void pauseApp() {
      this.f_a_00.hideNotify();
   }

   protected void destroyApp(boolean var1) {
   }

   public static void m_000() {
      f_CMidlet_00.destroyApp(true);
      f_CMidlet_00.notifyDestroyed();
      f_CMidlet_00 = null;
   }
}
