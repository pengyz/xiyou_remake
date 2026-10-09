package javax.microedition.media;

/**
 * MMAPI {@code Controllable} shim（参考版 invokeinterface getControl(String)）。
 */
public interface Controllable {
    Control getControl(String controlType);
}
