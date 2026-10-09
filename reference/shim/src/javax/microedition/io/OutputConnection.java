package javax.microedition.io;

import java.io.DataOutputStream;
import java.io.IOException;
import java.io.OutputStream;

/**
 * GCF {@code OutputConnection} shim（invokeinterface openOutputStream()）。
 */
public interface OutputConnection extends Connection {
    OutputStream openOutputStream() throws IOException;

    DataOutputStream openDataOutputStream() throws IOException;
}
