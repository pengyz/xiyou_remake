package javax.microedition.media.control;

import javax.microedition.media.Control;

/**
 * MMAPI {@code VolumeControl} shim（参考版引用 setLevel(int)）。
 */
public interface VolumeControl extends Control {
    int setLevel(int level);

    int getLevel();

    boolean isMuted();

    void setMute(boolean mute);
}
