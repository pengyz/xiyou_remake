package javax.microedition.rms;

/**
 * MIDP-1.0 {@code RecordEnumeration} shim。
 * 参考版仅引用 nextRecordId() / destroy()。
 */
public interface RecordEnumeration {
    int numRecords();

    boolean hasNextElement();

    boolean hasPreviousElement();

    int nextRecordId();

    int previousRecordId();

    void reset();

    void destroy();
}
