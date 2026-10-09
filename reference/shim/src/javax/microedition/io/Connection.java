package javax.microedition.io;

import java.io.IOException;

/**
 * GCF {@code javax.microedition.io.Connection} shim（invokeinterface close()）。
 */
public interface Connection {
    void close() throws IOException;
}
