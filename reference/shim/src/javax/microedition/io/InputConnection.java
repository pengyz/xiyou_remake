package javax.microedition.io;

import java.io.DataInputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * GCF {@code InputConnection} shim（invokeinterface openInputStream()）。
 */
public interface InputConnection extends Connection {
    InputStream openInputStream() throws IOException;

    DataInputStream openDataInputStream() throws IOException;
}
