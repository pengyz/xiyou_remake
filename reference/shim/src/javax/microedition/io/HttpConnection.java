package javax.microedition.io;

/**
 * GCF {@code HttpConnection} shim。
 * 参考版引用：getResponseCode / getHeaderField(int) / getHeaderField(String) /
 * getHeaderFieldKey(int) / setRequestMethod(String) / setRequestProperty(String,String)
 * + 继承 openInputStream / openOutputStream / getLength / close。
 */
public interface HttpConnection extends ContentConnection {
    String GET = "GET";
    String POST = "POST";
    String HEAD = "HEAD";

    String getURL();

    String getProtocol();

    String getHost();

    String getFile();

    String getRef();

    String getQuery();

    int getPort();

    String getMethod();

    void setRequestMethod(String method);

    String getRequestProperty(String key);

    void setRequestProperty(String key, String value);

    int getResponseCode();

    String getResponseMessage();

    long getExpiration();

    long getDate();

    long getLastModified();

    String getHeaderField(String name);

    int getHeaderFieldInt(String name, int def);

    long getHeaderFieldDate(String name, long def);

    String getHeaderField(int n);

    String getHeaderFieldKey(int n);
}
