package javax.microedition.io;

/**
 * GCF {@code ContentConnection} shim（invokeinterface getLength()）。
 */
public interface ContentConnection extends InputConnection, OutputConnection {
    String getType();

    String getEncoding();

    long getLength();
}
