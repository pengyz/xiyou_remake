package javax.microedition.rms;

import java.util.Map;
import java.util.TreeMap;

/**
 * MIDP-1.0 {@code javax.microedition.rms.RecordStore} 自研 shim：纯内存实现。
 *
 * <p>参考版实际引用面：openRecordStore(String,boolean) / closeRecordStore() /
 * deleteRecordStore(String) / enumerateRecords(RecordFilter,RecordComparator,boolean) /
 * addRecord(byte[],int,int) / setRecord(int,byte[],int,int) / getRecord(int) /
 * getNumRecords()。
 *
 * <p>存储为进程内静态 Map（同一 JVM 运行期持久，跨运行为空）：确定性、无 I/O。
 * 记录 id 从 1 起顺序分配。异常类型使用 {@link RecordStoreException}（unchecked，
 * 与 MIDP 的 checked 异常不同——shim 简化，游戏只捕获 Exception）。
 */
public class RecordStore {
    public static final class RecordStoreException extends RuntimeException {
        public RecordStoreException(String msg) {
            super(msg);
        }
    }

    private static final Map<String, TreeMap<Integer, byte[]>> STORES = new TreeMap<String, TreeMap<Integer, byte[]>>();
    private static final Map<String, Integer> OPEN_COUNT = new TreeMap<String, Integer>();

    static {
        applyPreset(System.getProperty("oracle.preset"));
    }

    /**
     * oracle 深场景支持：启动前灌入 RMS 预置（-Doracle.preset=<file>）。
     * 文件格式（preset_rms.py 产出，确定性）：每行 `<store>@<recordId>=<hexlower>`。
     * 只在 JVM 启动时执行一次、早于任何游戏代码 ⇒ 对游戏而言与真机历史存档无差别。
     * 无该属性 ⇒ 什么都不做（既有行为完全不变）。
     */
    static void applyPreset(String path) {
        if (path == null || path.length() == 0) {
            return;
        }
        try {
            java.io.BufferedReader r = new java.io.BufferedReader(new java.io.FileReader(path));
            String line;
            while ((line = r.readLine()) != null) {
                line = line.trim();
                if (line.length() == 0 || line.charAt(0) == '#') {
                    continue;
                }
                int at = line.indexOf('@');
                int eq = line.indexOf('=', at);
                if (at <= 0 || eq <= at) {
                    throw new RecordStoreException("bad preset line: " + line);
                }
                String name = line.substring(0, at);
                int rid = Integer.parseInt(line.substring(at + 1, eq));
                String hex = line.substring(eq + 1);
                byte[] data = new byte[hex.length() / 2];
                for (int i = 0; i < data.length; i++) {
                    data[i] = (byte) Integer.parseInt(hex.substring(i * 2, i * 2 + 2), 16);
                }
                TreeMap<Integer, byte[]> recs = STORES.get(name);
                if (recs == null) {
                    recs = new TreeMap<Integer, byte[]>();
                    STORES.put(name, recs);
                }
                recs.put(rid, data);
            }
            r.close();
        } catch (RecordStoreException e) {
            throw e;
        } catch (Exception e) {
            throw new RecordStoreException("preset load failed: " + e);
        }
    }

    private final String name;
    private final TreeMap<Integer, byte[]> records;
    private int nextId = 1;

    private RecordStore(String name, TreeMap<Integer, byte[]> records) {
        this.name = name;
        this.records = records;
        if (!records.isEmpty()) {
            this.nextId = records.lastKey() + 1;
        }
    }

    public static synchronized RecordStore openRecordStore(String name, boolean createIfNecessary) {
        TreeMap<Integer, byte[]> recs = STORES.get(name);
        if (recs == null) {
            if (!createIfNecessary) {
                throw new RecordStoreException("no such record store: " + name);
            }
            recs = new TreeMap<Integer, byte[]>();
            STORES.put(name, recs);
        }
        Integer c = OPEN_COUNT.get(name);
        OPEN_COUNT.put(name, c == null ? 1 : c + 1);
        return new RecordStore(name, recs);
    }

    public synchronized void closeRecordStore() {
        Integer c = OPEN_COUNT.get(name);
        if (c != null && c > 1) {
            OPEN_COUNT.put(name, c - 1);
        } else {
            OPEN_COUNT.remove(name);
        }
    }

    public static synchronized void deleteRecordStore(String name) {
        if (OPEN_COUNT.containsKey(name)) {
            throw new RecordStoreException("store open: " + name);
        }
        STORES.remove(name);
    }

    public synchronized int addRecord(byte[] data, int offset, int numBytes) {
        byte[] copy = new byte[numBytes];
        System.arraycopy(data, offset, copy, 0, numBytes);
        int id = nextId++;
        records.put(id, copy);
        return id;
    }

    public synchronized void setRecord(int recordId, byte[] data, int offset, int numBytes) {
        if (!records.containsKey(recordId)) {
            throw new RecordStoreException("no such record: " + recordId);
        }
        byte[] copy = new byte[numBytes];
        System.arraycopy(data, offset, copy, 0, numBytes);
        records.put(recordId, copy);
    }

    public synchronized byte[] getRecord(int recordId) {
        byte[] r = records.get(recordId);
        if (r == null) {
            throw new RecordStoreException("no such record: " + recordId);
        }
        byte[] copy = new byte[r.length];
        System.arraycopy(r, 0, copy, 0, r.length);
        return copy;
    }

    public synchronized int getNumRecords() {
        return records.size();
    }

    public synchronized RecordEnumeration enumerateRecords(RecordFilter filter, RecordComparator comparator,
                                                           boolean keepUpdated) {
        return new EnumImpl(new java.util.ArrayList<Integer>(records.keySet()));
    }

    public String getName() {
        return name;
    }

    /** oracle 诊断：导出全部 store 内容摘要（确定性）。 */
    public static synchronized String dumpAll() {
        StringBuilder sb = new StringBuilder();
        for (Map.Entry<String, TreeMap<Integer, byte[]>> e : STORES.entrySet()) {
            sb.append(e.getKey()).append(':').append(e.getValue().size()).append("recs");
            for (Map.Entry<Integer, byte[]> r : e.getValue().entrySet()) {
                sb.append(' ').append(r.getKey()).append('=').append(r.getValue().length).append('b');
            }
            sb.append('\n');
        }
        return sb.toString();
    }

    private static final class EnumImpl implements RecordEnumeration {
        private final java.util.List<Integer> ids;
        private int pos = 0;

        EnumImpl(java.util.List<Integer> ids) {
            this.ids = ids;
        }

        public int numRecords() {
            return ids.size();
        }

        public boolean hasNextElement() {
            return pos < ids.size();
        }

        public boolean hasPreviousElement() {
            return pos > 0;
        }

        public int nextRecordId() {
            return ids.get(pos++);
        }

        public int previousRecordId() {
            return ids.get(--pos);
        }

        public void reset() {
            pos = 0;
        }

        public void destroy() {
        }
    }
}
