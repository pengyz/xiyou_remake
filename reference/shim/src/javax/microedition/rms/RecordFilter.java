package javax.microedition.rms;

/**
 * MIDP-1.0 {@code RecordFilter} 空接口 shim（enumerateRecords 描述符需要）。
 */
public interface RecordFilter {
    boolean matches(byte[] candidate);
}
