package com.nokia.mid.ui;

import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

/**
 * Nokia UI API {@code com.nokia.mid.ui.DirectGraphics} 最小 shim。
 * 参考版仅引用 drawImage(Image,int,int,int,int)（x,y,anchor,transform）。
 */
public interface DirectGraphics {
    int TRANS_NONE = 0;
    int TRANS_MIRROR_ROT180 = 1;
    int TRANS_MIRROR = 2;
    int TRANS_ROT180 = 3;
    int TRANS_MIRROR_ROT270 = 4;
    int TRANS_ROT90 = 5;
    int TRANS_ROT270 = 6;
    int TRANS_MIRROR_ROT90 = 7;

    void drawImage(Image image, int x, int y, int anchor, int transform);

    Graphics getGraphics();
}
