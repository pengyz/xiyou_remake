package oracle.host;

import java.lang.reflect.Array;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.List;

import javax.microedition.lcdui.Image;

/**
 * 状态向量反射导出：<b>只导出值，不导出符号名</b>（字段按 class 文件声明序编号
 * + 类型描述符），因此「原始版」与「无歧义版」（成员符号重命名）的 dump
 * 可以逐字节比对。声明序由 t1 rename-pipeline 保证不变（常量池 name_index
 * 原地替换，成员顺序不动）。
 */
public final class StateDump {
    private static final int MAX_STRINGS = 64;
    private static final int MAX_ARRAY_ELEMS = 32;

    private StateDump() {
    }

    public static List<String> dump(Object root) {
        List<String> out = new ArrayList<String>();
        Class<?> cls = root.getClass();
        Field[] fields = cls.getDeclaredFields();
        int inst = 0, stat = 0;
        for (Field f : fields) {
            if (Modifier.isStatic(f.getModifiers())) {
                stat++;
            } else {
                inst++;
            }
        }
        out.add("FIELDS inst=" + inst + " static=" + stat);
        int idx = 0;
        for (Field f : fields) {
            f.setAccessible(true);
            Object v;
            try {
                v = Modifier.isStatic(f.getModifiers()) ? f.get(null) : f.get(root);
            } catch (IllegalAccessException e) {
                v = "?";
            }
            String kind = Modifier.isStatic(f.getModifiers()) ? "SFLD" : "FLD";
            out.add(kind + " " + String.format("%03d", idx++) + " " + f.getType().getName().replace('.', '/')
                    + " " + value(v, root, 0));
        }
        return out;
    }

    private static String value(Object v, Object root, int depth) {
        if (v == null) {
            return "null";
        }
        if (v == root) {
            return "SELF";
        }
        Class<?> c = v.getClass();
        if (v instanceof String) {
            return TraceSink.quote((String) v);
        }
        if (v instanceof Boolean || v instanceof Byte || v instanceof Short || v instanceof Integer
                || v instanceof Long) {
            return v.toString();
        }
        if (v instanceof Float) {
            return "F:" + v.toString();
        }
        if (v instanceof Double) {
            return "D:" + v.toString();
        }
        if (v instanceof Character) {
            return "C:" + (int) ((Character) v).charValue();
        }
        if (v instanceof Image) {
            Image img = (Image) v;
            return "IMG(" + img.getWidth() + "x" + img.getHeight() + ",#" + imgHash(img) + ")";
        }
        if (c.isArray()) {
            return array(v, root, depth);
        }
        return "OBJ:" + c.getName().replace('.', '/');
    }

    private static String array(Object v, Object root, int depth) {
        int len = Array.getLength(v);
        Class<?> comp = v.getClass().getComponentType();
        StringBuilder sb = new StringBuilder("ARR[").append(len).append(']');
        if (comp.isPrimitive()) {
            if (comp == int.class || comp == byte.class || comp == short.class
                    || comp == long.class || comp == boolean.class || comp == char.class) {
                sb.append(" sha=#").append(primitiveHash(v));
            } else {
                sb.append(" sha=#").append(primitiveHash(v));
            }
            sb.append(" head=");
            int n = Math.min(len, 16);
            for (int i = 0; i < n; i++) {
                if (i > 0) {
                    sb.append(',');
                }
                sb.append(Array.get(v, i));
            }
            return sb.toString();
        }
        // 对象数组：有限递归
        sb.append('{');
        int n = Math.min(len, MAX_ARRAY_ELEMS);
        for (int i = 0; i < n; i++) {
            if (i > 0) {
                sb.append(',');
            }
            Object e = Array.get(v, i);
            if (e == null) {
                sb.append("null");
            } else if (depth >= 2) {
                sb.append("OBJ:").append(e.getClass().getName().replace('.', '/'));
            } else {
                sb.append(value(e, root, depth + 1));
            }
        }
        if (len > n) {
            sb.append(",+").append(len - n).append("more");
        }
        return sb.append('}').toString();
    }

    private static String primitiveHash(Object v) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            int len = Array.getLength(v);
            Class<?> comp = v.getClass().getComponentType();
            for (int i = 0; i < len; i++) {
                Object e = Array.get(v, i);
                long lv;
                if (comp == boolean.class) {
                    lv = ((Boolean) e) ? 1 : 0;
                } else if (comp == char.class) {
                    lv = ((Character) e).charValue();
                } else if (comp == float.class) {
                    lv = Float.floatToRawIntBits((Float) e);
                } else if (comp == double.class) {
                    lv = Double.doubleToRawLongBits((Double) e);
                } else {
                    lv = ((Number) e).longValue();
                }
                for (int s = 56; s >= 0; s -= 8) {
                    md.update((byte) (lv >>> s));
                }
            }
            return TraceSink.hex(md.digest(), 8);
        } catch (Exception e) {
            return "err";
        }
    }

    /** 图像内容哈希：全部 ARGB 像素（大端 4 字节/像素），取前 8 字节 hex。 */
    public static String imgHash(Image img) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            int[] px = new int[img.getWidth() * img.getHeight()];
            img.getRGB(px, 0, img.getWidth(), 0, 0, img.getWidth(), img.getHeight());
            for (int p : px) {
                md.update((byte) (p >>> 24));
                md.update((byte) (p >>> 16));
                md.update((byte) (p >>> 8));
                md.update((byte) p);
            }
            return TraceSink.hex(md.digest(), 8);
        } catch (Exception e) {
            return "err";
        }
    }
}
