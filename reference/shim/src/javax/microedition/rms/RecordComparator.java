package javax.microedition.rms;

/**
 * MIDP-1.0 {@code RecordComparator} 空接口 shim（enumerateRecords 描述符需要）。
 */
public interface RecordComparator {
    int compare(byte[] rec1, byte[] rec2);
}
