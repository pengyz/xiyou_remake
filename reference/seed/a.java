/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.nokia.mid.ui.DirectGraphics
 *  com.nokia.mid.ui.DirectUtils
 *  javax.microedition.io.Connector
 *  javax.microedition.io.HttpConnection
 *  javax.microedition.lcdui.Canvas
 *  javax.microedition.lcdui.Font
 *  javax.microedition.lcdui.Graphics
 *  javax.microedition.lcdui.Image
 *  javax.microedition.media.Manager
 *  javax.microedition.media.Player
 *  javax.microedition.media.control.VolumeControl
 *  javax.microedition.rms.RecordEnumeration
 *  javax.microedition.rms.RecordStore
 */
import com.nokia.mid.ui.DirectGraphics;
import com.nokia.mid.ui.DirectUtils;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.DataOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.Random;
import javax.microedition.io.Connector;
import javax.microedition.io.HttpConnection;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Font;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;
import javax.microedition.media.Manager;
import javax.microedition.media.Player;
import javax.microedition.media.control.VolumeControl;
import javax.microedition.rms.RecordEnumeration;
import javax.microedition.rms.RecordStore;

/*
 * Duplicate member names - consider using --renamedupmembers true
 */
public final class a
extends Canvas
implements Runnable {
    private Font a;
    private int a;
    private int b;
    private int c;
    private String a;
    private String[] a;
    private boolean a;
    private Graphics a;
    private boolean b;
    private int d;
    private byte a;
    private byte b;
    private byte c;
    private DirectGraphics a;
    private int e;
    private Image a;
    private boolean c;
    private String[] b;
    private int[] a;
    private Image[][] a;
    private static int[] b = new int[]{0, 8192, 16384, 24576, 8462, 270, 90, 8282};
    private int[][] a;
    private InputStream a;
    private OutputStream a;
    private InputStream b;
    private int f;
    private int g;
    private String[] c;
    private int h;
    private int i;
    private int j;
    private byte[] a;
    private int k;
    private int l;
    private int m;
    private int n;
    private int o;
    private int p;
    private int q;
    private byte[] b;
    private byte[] c;
    private byte d;
    private int r;
    private int s;
    private Image b;
    private int t;
    private int u;
    private String b;
    private int v;
    private boolean d;
    private byte e;
    private int[] c;
    private int w;
    private int x;
    private boolean e;
    private boolean f;
    private byte f;
    private byte g;
    private byte h;
    private byte i;
    private int y;
    private int z;
    private String c;
    private String d;
    private String[] d;
    private int A;
    private int B;
    private int C;
    private int D;
    private int E;
    private int F;
    private int G;
    private int H;
    private int I;
    private int J;
    private int K;
    private int L;
    private byte j;
    private byte k;
    private int[][] b;
    private int[] d;
    private int M;
    private byte l;
    private int N;
    private int O;
    private int P;
    private int Q;
    private byte m;
    private int R;
    private int S;
    private Image c;
    private int[] e;
    private int[] f;
    private boolean g;
    private boolean h;
    private boolean i;
    private byte n;
    private byte o;
    private String[] e;
    private int[] g;
    private int[] h;
    private int[] i;
    private int[] j;
    private int[] k;
    private Image[] a;
    private int[] l;
    private int[] m;
    private int T;
    private boolean[] a;
    private boolean[] b;
    private byte[] d;
    private short[] a;
    private boolean[] c;
    private byte[] e;
    private byte[] f;
    private int[] n;
    private int[][] c;
    private byte[] g;
    private byte[] h;
    private boolean[] d;
    private int U;
    private boolean j;
    private int V;
    private int W;
    private byte[][] a;
    private byte[][] b;
    private int X;
    private Image d;
    private int Y;
    private int Z;
    private int aa;
    private int ab;
    private byte[] i;
    private byte[] j;
    private boolean[][] a;
    private byte[][] c;
    private byte[] k;
    private byte[][] d;
    private byte p;
    private boolean[] e;
    private int ac;
    private int ad;
    private int ae;
    private int af;
    private int ag;
    private int ah;
    private int ai;
    private int aj;
    private int ak;
    private int al;
    private boolean k;
    private boolean l;
    private int am;
    private int an;
    private int ao;
    private int ap;
    private int aq;
    private int ar;
    private int as;
    private int at;
    private int au;
    private int av;
    private int aw;
    private int ax;
    private int ay;
    private int az;
    private int aA;
    private int aB;
    private int aC;
    private String[] f;
    private int aD;
    private byte[] l;
    private boolean[] f;
    private int aE;
    private int aF;
    private int aG;
    private int aH;
    private short[] b;
    private short[] c;
    private short[] d;
    private int aI;
    private String[] g;
    private String[] h;
    private byte[] m;
    private String[] i;
    private String[] j;
    private byte[] n;
    private int aJ;
    private int aK;
    private byte[] o;
    private byte[] p;
    private int aL;
    private int aM;
    private String e;
    private String[] k;
    private String[] l;
    private byte[] q;
    private int aN;
    private int aO;
    private int aP;
    private int aQ;
    private int aR;
    private byte[] r;
    private byte[] s;
    private int aS;
    private int[] o;
    private byte[] t;
    private int aT;
    private int aU;
    private int aV;
    private int aW;
    private int aX;
    private int aY;
    private int aZ;
    private int ba;
    private boolean m;
    private boolean n;
    private String[] m;
    private int[] p;
    private int[] q;
    private int[] r;
    private int[] s;
    private int[] t;
    private int[] u;
    private int[] v;
    private int bb;
    private int bc;
    private int bd;
    private int be;
    private int bf;
    private int bg;
    private int[] w;
    private byte[] u;
    private int bh;
    private String[] n;
    private byte[] v;
    private byte[] w;
    private String[] o;
    private int bi;
    private int bj;
    private boolean[] g;
    private int bk;
    private int bl;
    private byte q;
    private String f;
    private int bm;
    private int bn;
    private int bo;
    private byte r;
    private int bp;
    private int bq;
    private int br;
    private int bs;
    private int bt;
    private int bu;
    private int bv;
    private int bw;
    private int bx;
    private byte s;
    private byte t;
    private int by;
    private int bz;
    private int bA;
    private boolean o;
    private short[] e;
    private byte[][] e;
    private byte[] x;
    private boolean p;
    private boolean[] h;
    private byte[] y;
    private int[] x;
    private int[] y;
    private int[] z;
    private int[] A;
    private int[] B;
    private int[] C;
    private int[] D;
    private byte[] z;
    private byte[] A;
    private int bB;
    private int bC;
    private int bD;
    private int bE;
    private int bF;
    private int bG;
    private RecordStore a;
    private RecordEnumeration a;
    private ByteArrayOutputStream a;
    private DataOutputStream a;
    private DataInputStream a;
    private byte[] B;
    private byte[][] f;
    private int bH;
    private byte[] C;
    private int bI;
    private byte[] D;
    private short[] f;
    private short[] g;
    private short[] h;
    private short[] i;
    private byte u;
    private byte v;
    private byte[] E;
    private int[] E;
    private short[] j;
    private short[] k;
    private byte[] F;
    private int bJ;
    private int bK;
    private boolean[] i;
    private byte[] G;
    private byte[] H;
    private byte[] I;
    private byte[] J;
    private byte[] K;
    private byte[] L;
    private short[] l;
    private short[] m;
    private byte[] M;
    private byte[] N;
    private byte[] O;
    private byte w;
    private int[] F;
    private boolean[] j;
    private boolean[] k;
    private int bL;
    private boolean q;
    private boolean r;
    private int bM;
    private byte x;
    private boolean s;
    private byte[] P;
    private int bN;
    private int bO;
    private int bP;
    private byte y;
    private boolean t;
    private int bQ;
    private int bR;
    private int[] G;
    private byte z;
    private byte[] Q;
    private int bS;
    private byte[][] g;
    private short[][] a;
    private short a;
    private short b;
    private short c = 75;
    private short d;
    private short[] n;
    private short[] o;
    private int bT;
    private boolean u;
    private boolean v;
    private boolean w;
    private byte A;
    private int[] H;
    private int bU;
    private int bV;
    private boolean x;
    private byte B;
    private boolean y;
    private boolean z;
    private byte[] R;
    private byte[] S;
    private boolean A;
    private boolean B;
    private int[] I;
    private int[] J;
    private int bW;
    private int bX;
    private int bY;
    private boolean C;
    private String[] p;
    private int bZ;
    private boolean D;
    private byte C;
    private static Player a;
    private Random a;
    private final int[] K;
    private int ca;
    private final byte[] T;
    private int cb;
    private int cc;
    private int[] L;
    private boolean E;
    private final byte[] U;
    private byte D;
    private int[][] d;
    private Image[] b;
    private String[] q;
    private String[] r;
    private String g;
    private int cd;
    private byte[] V;
    private int ce;
    private Object a;
    private int cf;
    private boolean F;
    private boolean G;
    private String h;
    private int cg;
    private long a;
    private long b;
    private int ch;
    private int ci;
    private HttpConnection a = "$Rev: 3289 $";
    private boolean H;
    private byte[] W;
    private int cj;
    private int ck;

    public a() {
        this.b = this.a.getHeight();
        new StringBuffer().append("(\u5185\u90e8\u7248\u672c").append(this.a.substring(4, this.a.length() - 1)).append(")");
        this.a = new String[]{"\u65e0\u6cd5\u518d\u4e0b\u4e00\u5c42\u4e86\uff0c\u8fd9\u662f\u4f60\u8fbe\u5230\u7684\u6700\u5e95\u5c42", "\u65e0\u6cd5\u518d\u4e0a\u4e00\u5c42\u4e86\uff0c\u8fd9\u662f\u4f60\u8fbe\u5230\u7684\u6700\u9ad8\u5c42", "\u65e0\u6cd5\u518d\u4e0b\u53bb\u4e86\u3002", "\u65e0\u6cd5\u518d\u4e0a\u53bb\u4e86\u3002", "\u4ffa\uff0c\u5f53\u4e16\u795e\u754c\u7b2c\u4e00\u6597\u8005\uff0c\u5b59!\u609f!\u7a7a! \u81ea\u4ece\u53d7\u5c01\u4e3a\u9f50\u5929\u5927\u5723\uff0c\u638c\u7ba1\u87e0\u6843\u56ed\u4ee5\u6765\uff0c\u4e00\u76f4\u900d\u9065\u5feb\u6d3b\uff0c\u65e0\u62d8\u675f\u2026\u2026", "\u4f60\u62e5\u6709\u66f4\u5f3a\u529b\u7684\u88c5\u5907\uff0c\u56e0\u6b64\u5c06\u6361\u5230\u7684\u4e22\u5f03\u4e86\u3002", "\u76f4\u5230\u90a3\u4e00\u5929\uff0c\u9047\u5230\u4e86\u5979\uff0c\u5728\u7b4b\u6597\u4e91\u4e0a\u7684\u6211\uff0c\u7adf\u7136\u7b2c\u4e00\u6b21\u5fc3\u6f6e\u8d77\u4f0f\uff0c\u6709\u4e86\u6655\u673a\u7684\u611f\u89c9\u2026\u2026", "\u795e\u4ed9\u52a8\u4e86\u611f\u60c5\uff0c\u5f80\u5f80\u4f1a\u4e07\u52ab\u4e0d\u590d\uff0c\n\u8fd9\u4e00\u6b21\uff0c\u8ba9\u6211\u4ed8\u51fa\u4e86\u4e94\u767e\u5e74\u7684\u65f6\u95f4\u53bb\u5fd8\u8bb0\u5979\u2026\u2026\n\u4e94\u6307\u5c71\u811a\u4e0b\u7684\u6c99\u5b50\uff0c\u63a0\u8fc7\u6211\u7684\u8138\u5e9e\u3002\n\u6c99\u5b50\uff0c\u8ddf\u65f6\u95f4\u4e00\u6837\uff0c\u540c\u6837\u968f\u98ce\u6d41\u901d\uff1b\u540c\u6837\u63a9\u57cb\u8fc7\u53bb\uff1b\n\u591a\u5c11\u6b21\u4f38\u624b\u60f3\u6293\u4f4f\uff0c\u5374\u4ece\u6307\u9699\u6e9c\u8d70\u2026\u2026\n\u770b\u591c\u7a7a\uff0c\u534a\u68a6\u534a\u9192\u95f4\uff0c\u5f80\u4e8b\u5386\u5386\u4e0a\u5fc3\u5934\u2026\u2026"};
        this.e = 0;
        this.a = null;
        this.b = new String[]{"sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1", "ui", "xtq", "menu", "intro", "face", "sptenemy2", "end", "load"};
        this.a = new int[]{8, 1, 12, 4, 13, 23, 10, 20, 25, 6, 2, 2, 12, 20, 1, 2};
        this.a = new Image[this.a.length][];
        this.a = new int[][]{{0, 1, 2}, {3, 5}, {4, 5}};
        this.a = null;
        this.b = null;
        this.c = new String[]{"\u65b0\u6e38\u620f", "\u7ee7\u7eed\u6e38\u620f", "\u8f7d\u5165\u8fdb\u5ea6", "\u4fdd\u5b58\u6e38\u620f", "\u8bbe\u7f6e", "\u5e2e\u52a9", "\u5173\u4e8e", "\u9000\u51fa", "\u8fd4\u56de\u83dc\u5355", "\u56de\u653e", "\u505c\u6b62\u56de\u653e", "\u5546\u5e97", "\u5341\u5168\u5927\u8865\u5305", "\u653b\u9632\u795e\u6cb9", "\u5f00\u95e8\u5929\u5929\u4e50", "\u5929\u5ead\u6d88\u8d39\u5238", "\u5370\u5ea6\u795e\u8840\u6cb9", "\u8df3\u8fc7\u6559\u7a0b"};
        this.q = 25;
        this.b = new byte[]{-1, 0, 1, 1, 1, 0, -1, -1};
        this.c = new byte[]{-1, -1, -1, 0, 1, 1, 1, 0};
        this.d = 0;
        this.r = 0;
        this.s = 0;
        this.c = new int[]{0xF8F8F8, 0xAAAAAA, 0x888888, 0x444444, 0x111111};
        int[] nArray = new int[]{0, 12, 24, 36, 47, 57, 67, 75, 83, 89, 94, 97, 100, 97, 94, 89, 83, 75, 67, 57, 47, 36, 24, 12, 0};
        this.h = 0;
        this.d = new String[50];
        this.I = this.b + 4;
        this.J = 498;
        this.K = 10;
        this.L = 10;
        this.j = 0;
        this.k = 0;
        this.b = new int[][]{{0, 1, 0, 2}, new int[0], new int[0], new int[0]};
        this.N = 192;
        this.O = 352;
        this.P = 6;
        this.Q = 11;
        this.e = new int[4];
        this.f = new int[4];
        this.e = new String[]{"\u65e0", "\u6728", "\u94c1", "\u94f6", "\u91d1", "\u5e03", "\u76ae", "\u9501", "\u91d1"};
        this.c = new int[][]{{0}, {0, 1, 0, 2}, {0, 1, 2}, {0, 1, 2, 1}, {0, 1, 2, 2, 1, 0}, {0, 1, 2, 3, 2, 1}, {0, 1, 2, 3, 4}, {3, 4, 5, 6}, {4, 3, 2, 1, 0}, {2, 1, 0}, {2, 3, 4}};
        this.g = new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 4, 2, 3, 3, 3, 3, 3, 3, 2, 3, 2, 2, 2, 3, 2, 3, 3, 3, 2, 3, 5, 3, 3, 3, 3, 3, 3, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0};
        this.h = new byte[]{32, 32, 32, 32, 45, 32, 32, 32, 32, 104, 35, 45, 30, 32, 32, 30, 30, 24, 25, 21, 32, 25, 25, 27, 31, 27, 23, 23, 23, 32, 32, 32, 32, 27, 27, 27, 27, 22, 22, 16, 25, 15, 20, 28, 26, 23, 34, 35, 24, 33, 22, 22, 28, 36, 38, 29, 34, 29, 30, 31, 33, 26, 30, 36, 23, 38, 33, 42, 32, 78, 38, 36, 32, 39, 59, 36, 32, 29, 37, 27, 17, 32, 32, 32, 29, 27, 32, 36};
        this.d = new boolean[]{true, false, false, false, false, true, true, true, true, false, true, false, false};
        this.V = 0;
        this.W = 270;
        this.a = new byte[][]{{0, 0, 0, 33, 56}, {0, 33, 0, 33, 56}, {0, 66, 0, 33, 56}, {2, 0, 0, 18, 26}, {3, 0, 0, 18, 34}, {1, 0, 0, 55, 61}, {4, 0, 0, 14, 26}, {4, 14, 0, 14, 26}, {4, 28, 0, 24, 26}};
        this.b = new byte[][]{{6, -1, -79, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -53, 0, 0, -16, -27, 0}, {7, -1, -80, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -54, 0, 2, -16, -26, 0}, {8, -1, -81, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -55, 0, 1, -16, -25, 0}};
        int[] nArray2 = new int[]{0, 0, 11, 1, 1, 11, 11, 11, 0, 0};
        int[] nArray3 = new int[]{0, 0, 0, 0, 1, 1, 1, 11, 11, 11};
        this.e = new boolean[]{false, true, false, true, true, true, true, false, true, false, false, true, true, true, true, false, true, true, true, true, true, true, false, false, true, true, true, true, true, true, false, false, false, false, false, false, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false};
        this.am = 0;
        this.ap = 0;
        this.aq = 55;
        this.aw = this.b + 90;
        this.ax = 115;
        this.f = new String[]{"\u58f0\u97f3", "\u5c0f\u5730\u56fe"};
        this.aD = this.b + 8;
        this.l = new byte[4];
        this.f = new boolean[4];
        this.aE = 0;
        this.aF = 0;
        this.aG = 80;
        this.aH = 32 + this.aD;
        this.b = new short[10];
        this.c = new short[10];
        this.d = new short[10];
        this.aI = 0;
        this.g = new String[]{"", "", "\u6709\u4e9b\u95e8\u4e0d\u80fd\u7528\u94a5\u5319\u6253\u5f00\uff0c\u53ea\u6709\u5f53\u4f60\u6253\u8d25\u5b83\u7684\u5b88\u536b\u540e\u624d\u4f1a\u81ea\u52a8\u6253\u5f00\u3002", "\u4f60\u8d2d\u4e70\u4e86\u793c\u7269\u540e\u518d\u4e0e\u5929\u5bab\u5546\u4eba\u5bf9\u8bdd\uff0c\u4ed6\u4f1a\u544a\u8bc9\u4f60\u4e00\u4e9b\u91cd\u8981\u7684\u6d88\u606f\u3002", "", "\u6211\u542c\u8bf4\u5728\u5929\u5bab\u4e2d\u67092\u628a\u9690\u85cf\u7684\u7ea2\u94a5\u5319\u3002", "\u5728\u8fd9\u4e2a\u533a\u57df\u4e0d\u591a\u6b21\u63d0\u5347\u653b\u51fb\u529b\uff0c\u5c31\u4e0d\u80fd\u6253\u8d25\u201c\u6768\u622c\u526f\u624b\u201d\u3002\u5207\u8bb0\u524d\u4eba\u6559\u8bad\uff01", "\u592a\u4e0a\u8001\u541b\u5c31\u572825\u697c\u3002\u4ee5\u4f60\u73b0\u5728\u7684\u72b6\u6001\u53bb\u653b\u51fb\u4ed6\u7b80\u76f4\u5c31\u662f\u81ea\u6740\u3002 \u4f60\u5e94\u5f53\u5728\u53d6\u5f97\u66f4\u9ad8\u7ea7\u522b\u7684\u9053\u5177\u540e\u518d\u53bb\u6253\u8d25\u4ed6\u3002", "\u4e0d\u627e\u5230\u6240\u6709\u7684\u6697\u589929\u697c\u7684\u6697\u9053\u662f\u4e0d\u4f1a\u6253\u5f00\u7684", "\u5982\u679c\u4f60\u523027\u697c\u65f6\u72b6\u6001\u4e3a\uff1a\u751f\u547d1500\u3001\u653b\u51fb80\u3001\u9632\u5fa198\u3001\u62e5\u67091\u628a\u84dd\u94a5\u5319\u30015\u628a\u9ec4\u94a5\u5319\u3002\u90a3\u4e48\u795d\u8d3a\u4f60\uff0c\u4f60\u7684\u524d\u671f\u662f\u6bd4\u8f83\u6210\u529f\u7684\u3002", "\u516d\u4e01\u516d\u7532\u7684\u653b\u51fb\u529b\u592a\u9ad8\u4e86\uff0c\u4f60\u6700\u597d\u5230\u80fd\u5bf9\u4ed6\u4e00\u51fb\u5fc5\u6740\u65f6\u518d\u4e0e\u4ed6\u6218\u6597\u3002", "\u522b\u5306\u5fd9\uff0c\u653e\u6162\u901f\u5ea6\u3002", "\u5982\u679c\u4f60\u80fd\u7528\u597d4\u79cd\u79fb\u52a8\u5b9d\u7269\uff0c\u4f60\u4e0d\u7528\u4e0e\u5f3a\u654c\u4f5c\u6218\u5c31\u80fd\u4e0a\u697c\u3002", "", "\u4f60\u9700\u8981\u7528\u201c\u7384\u660e\u77f3\u201d\u53d6\u51fa37\u697c\u4ed3\u5e93\u5185\u7684\u6240\u6709\u5b9d\u7269\u3002", "\u8c1c\u9898\uff1a\u201c\u57283\u70b9\uff0c\u62e5\u6709\u4f20\u9001\u529f\u80fd\u7684\u5bc6\u5b9d\u5c31\u4f1a\u51fa\u73b0\u3002\u201d", "\u201c\u5deb\u5e08\u201d\u4f1a\u7528\u9b54\u6cd5\u653b\u51fb\u8def\u8fc7\u7684\u4eba\uff0c\u57282\u4e2a\u201c\u592a\u4e0a\u8001\u541b\u62a4\u536b\u201d\u95f4\u901a\u8fc7\u4f1a\u4f7f\u4f60\u7684\u751f\u547d\u51cf\u5c11\u4e00\u534a\u3002", "44\u697c\uff0c\u88ab\u85cf\u5728\u5f02\u7a7a\u95f4\uff0c\u4f60\u53ea\u80fd\u7528\u5bc6\u5b9d\u624d\u80fd\u5230\u8fbe\u3002", "41\u697c\u4e8b\u5b9e\u4e0a\u662f\u5de6\u53f3\u5bf9\u79f0\u7684\u3002", "\u50cf\u9ab0\u5b50\u4e0a5\u7684\u5f62\u72b6\u662f\u4e00\u79cd\u5c01\u5370\u9b54\u6cd5\uff0c\u4f60\u6700\u597d\u8bb0\u4f4f\u5b83\u5728\u4f60\u4e0e49\u697c\u5047\u9b54\u738b\u6218\u6597\u65f6\u6709\u7528", "", "\u4f60\u597d\uff0c\u6211\u662f\u592a\u767d\u91d1\u661f\u3002\u4f60\u6700\u597d\u522b\u89c1\u654c\u4eba\u5c31\u6740\uff0c\u5148\u5f80\u4e0a\u8d70\uff0c\u62ff\u5230\u6b66\u5668\u548c\u9632\u5177\u518d\u505a\u6253\u7b97\u3002"};
        this.h = new String[]{"\u611f\u8c22\u4f60\u6551\u4e86\u6211\uff0c\u8fd9\u662f1000\u91d1\u5c31\u9001\u7ed9\u4f60\u5427\u3002", "\u8bd5\u4e0b\u706b\u773c\u91d1\u775b\u5427\uff0c\u4f60\u80fd\u770b\u5230\u602a\u7269\u7684\u4fe1\u606f\u548c\u6218\u6597\u635f\u5931\u7684\u8840\u91cf\uff0c\u4f60\u53ef\u4ee5\u5728\u7269\u54c1\u680f\u4e2d\u4f7f\u7528\u5b83\u3002", "", "", "\u5f88\u597d\uff0c\u4f60\u5c45\u7136\u627e\u5230\u4e86\u6211\uff0c\u4f5c\u4e3a\u5956\u52b1\u6211\u5c06\u7ed9\u4f60\u4e00\u74f6\u5343\u5e74\u6708\u6842\u9732\uff0c\u559d\u4e86\u5b83\u5c06\u6309\u4f60\u7684\u653b\u51fb\u529b\u548c\u9632\u5fa1\u529b\u7684\u7efc\u5408\u589e\u52a0\u7684\u4f60\u751f\u547d\u70b9\u6570\uff0c\u4f60\u8d8a\u665a\u4f7f\u7528\u5b83\u6548\u679c\u8d8a\u597d\u3002", "", "", "", "", "", "", "", "", "", "\u611f\u8c22\u4f60\u6551\u4e86\u6211\uff0c\u8fd9\u662f1000\u91d1\u5c31\u9001\u7ed9\u4f60\u5427\u3002", "", "", "", "", "", "\u54c8\u55bd\uff0c\u9001\u4f601000\u91d1\u4f5c\u4e3a\u89c1\u9762\u793c\uff0c\u8bb0\u5f97\u7ecf\u5e38\u6765\u627e\u6211\u54e6\u3002", ""};
        this.m = new byte[]{2, 2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1};
        this.i = new String[]{"\u5927\u5723\u9976\u547d\uff0c\u6211\u53ef\u4ee5\u63d0\u53473%\u653b\u51fb\u529b\u548c\u9632\u5fa1\u529b\uff0c\u9700\u8981\u5c0f\u7684\u4e3a\u4f60\u6548\u52b3\u5417\uff1f", "", "\u5927\u5723\u7237\uff0c\u6211\u8fd9\u91cc\u67091\u628a\u84dd\u94a5\u5319\uff0c\u4f60\u7ed950\u91d1\u5e01\u5427\u3002", "\u6211\u67095\u628a\u9ec4\u94a5\u5319\uff0c\u4e00\u517150\u91d1\u5e01", "\u563f\u563f\uff0c\u6211\u6709\u5f88\u591a\u628a\u9ec4\u94a5\u5319\uff0c1\u628a1000\u91d1\u5e01", "\u6211\u67091\u628a\u7ea2\u94a5\u5319\uff0c\u53ea\u8981800\u91d1\u5e01", "\u6211\u67091\u628a\u84dd\u94a5\u5319\uff0c\u53ea\u8981200\u91d1\u5e01", "\u6211\u8ddf\u5176\u4ed6\u4eba\u4e0d\u4e00\u6837\uff0c\u4f60\u53ef\u4ee5\u628a\u591a\u4f59\u7684\u94a5\u5319\u5356\u7ed9\u6211\u3002100\u91d1\u5e01\u56de\u65361\u628a\u9ec4\u94a5\u5319", "\u6211\u67091\u628a\u9ec4\u94a5\u5319\uff0c1\u628a\u84dd\u94a5\u5319\uff0c\u4e00\u5171\u662f1000\u91d1\u5e01", "\u6211\u67093\u628a\u9ec4\u94a5\u5319\uff0c\u53ea\u8981200\u91d1\u5e01", "\u6211\u67093\u628a\u84dd\u94a5\u5319\uff0c\u6536\u4f602000\u91d1\u5e01", "\u6211\u53ef\u4ee5\u6062\u590d\u4f602000\u70b9\u8840\uff0c\u4e0d\u8fc7\u89811000\u91d1\u5e01", "\u6211\u6709\u4e2a\u5b9d\u7269\uff0c\u8981\u6536\u4f604000\u91d1\u5e01\uff0c\u662f\u4e2a\u7384\u660e\u77f3\u3002"};
        this.j = new String[]{"", "", "\u5929\u5bab\u4e00\u517150\u5c42\uff0c\u6bcf10\u5c42\u4e3a\u4e00\u4e2a\u533a\u57df\u3002\u5982\u679c\u4e0d\u6253\u8d25\u8be5\u533a\u57df\u7684\u5934\u76ee\u5c31\u4e0d\u80fd\u5230\u66f4\u9ad8\u7684\u5730\u65b9\u3002", "\u5728\u5546\u5e97\u91cc\u4f60\u6700\u597d\u9009\u62e9\u63d0\u5347\u9632\u5fa1\u529b\uff0c\u53ea\u6709\u5728\u653b\u51fb\u529b\u4f4e\u4e8e\u654c\u4eba\u7684\u9632\u5fa1\u529b\u65f6\u624d\u63d0\u5347\u653b\u51fb\u529b\u3002", "", "\u4f60\u662f\u5426\u6ce8\u610f\u52305\u30019\u300114\u300116\u300118\u697c\u6709\u7684\u5899\u4e0e\u4f17\u4e0d\u540c\uff1f", "\u5982\u679c\u4f60\u6301\u6709\u592a\u516c\u6756\uff0c\u9762\u5bf9\u5929\u795e\u529b\u58eb\u548c\u5de8\u7075\u795e\u65f6\u4f60\u7684\u653b\u51fb\u529b\u52a0\u500d\u3002\u5728\u6ca1\u6709\u592a\u516c\u6756\u7684\u60c5\u51b5\u4e0b\u4f60\u662f\u65e0\u6cd5\u6253\u8d25\u5de8\u7075\u795e\u7684\u3002\u592a\u516c\u6756\u88ab\u85cf\u572815\u697c\u4ee5\u4e0a\u7684\u5899\u5185\u3002", "", "\u5929\u5bab\u4e00\u5171\u670950\u5c42\uff0c\u4f4650\u697c\u5e76\u4e0d\u80fd\u76f4\u63a5\u4e0a\u53bb\u3002", "\u5b58\u653e\u4e4c\u91d1\u68cd\u7684\u623f\u95f4\u7684\u95e8\u574f\u4e86\uff0c\u4f60\u5fc5\u987b\u7528\u91d1\u52fa\u5b50\u7834\u5899\u800c\u5165\u3002", "\u5929\u5bab\u4e2d\u85cf\u6709\u6709\u4e2a\u201c\u5e78\u8fd0\u91d1\u5e01\u201d\u62e5\u6709\u5b83\u5728\u6253\u8d25\u654c\u4eba\u540e\u80fd\u591f\u83b7\u5f972\u500d\u7684\u91d1\u94b1\u3002", "\u201c\u7d2b\u91d1\u9f99\u9cde\u7532\u201d\u80fd\u9632\u5fa1\u201c\u592a\u4e0a\u8001\u541b\u62a4\u536b\u201d\u7684\u5939\u51fb\uff0c\u4f46\u5b83\u88ab\u6df1\u85cf\u5728\u795e\u79d8\u7684\u697c\u5c42\u4e2d\u3002", "\u5982\u679c\u8981\u6253\u8d25\u6768\u622c\u4f60\u9700\u8981\u201c\u4e4c\u91d1\u68cd\u201d\u3001\u201c\u94f6\u7f15\u9501\u7532\u201d\u3001\u201c\u6346\u4ed9\u7ef3\u201d\u6216\u66f4\u9ad8\u7b49\u7ea7\u7684\u5b9d\u7269\u3002"};
        this.n = new byte[]{2, 3, 3, 3, 6, 3, 3, 6, 3, 3, 3, 3, 3};
        this.aK = this.g.length + this.j.length;
        this.o = new byte[this.aK];
        this.p = new byte[this.aK];
        this.k = new String[]{"\u80fd\u770b\u7834\u654c\u4eba\u5e95\u7ec6\uff0c\u663e\u793a\u654c\u4eba\u8be6\u7ec6\u4fe1\u606f\u3002\u5728\u6e38\u620f\u4e2d\u6309\u5feb\u6377\u952e5\u4e5f\u53ef\u4ee5\u67e5\u770b\u4f24\u5bb3\u91cf\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u8bb0\u5f55\u524d\u5c18\u5f80\u4e8b\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u5728\u697c\u68af\u8fb9\uff0c\u53ef\u4ee5\u77ac\u95f4\u4e0a\u4e0b\u5c42\uff0c\u7559\u795e\u6655\u673a\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u7184\u706d\\cFFCC33\u4e09\u6627\u771f\u706b\\r\u7684\u795e\u5668\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u6316\u6d1e\u5f00\u5899\u8d8a\u72f1\u7684\u5229\u5668,\u632b\u662f\u632b\u4e86\u70b9\uff0c\u4f46\u662f\u771f\u7684\u5f88\u597d\u7528\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u53ef\u4ee5\u9707\u5f00\u5f53\u524d\u5c42\u6240\u6709\u7684\u5899\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u559d\u4e0b\u540e\uff0c\u589e\u52a0\u76f8\u5f53\u4e8e\u5f53\u524d\\c00FFFF\u653b\u51fb\u529b\\cFFFFFF\u52a0\\c00FFFF\u9632\u5fa1\u529b\\cFFFFFF\u503c740%\u7684\\cFFCC00\u8840\u91cf\n[\u6708\u5bab\u51fa\u54c1\uff0c\u624b\u5de5\u917f\u5236\uff0c\u4e0d\u542b\u4e09\u805a\u6c30\u80fa\uff0c\u51b7\u85cf\u6548\u679c\u66f4\u4f73\uff0c\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u77ac\u79fb\u5230\u4ee5\u4e2d\u5fc3\u4e3a\u5bf9\u79f0\u70b9\u7684\u4f4d\u7f6e\u4e0a\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a3\u6b21]", "\u77ac\u79fb\u4e0a\u884c\u4e00\u5c42\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u77ac\u79fb\u4e0b\u884c\u4e00\u5c42\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u5f53\u5e74\u59dc\u5b50\u7259\u53d7\u5929\u547d\u5c01\u795e\uff0c\u4ed6\u7684\u9493\u9c7c\u7af9\u7aff\u88ab\u539f\u59cb\u5929\u5c0a\u9644\u4e0a\u4e86\u795e\u529b\uff0c\u53ef\u4ee5\u5f79\u4f7f\u5929\u795e\u529b\u58eb\u4f9b\u4ed6\u5dee\u9063\uff0c\u6b64\u6756\u53c8\u540d\u201c\u6253\u795e\u97ad\u201d\uff0c\u5bf9\u5929\u795e\u529b\u58eb\uff08\u5305\u62ec\u5de8\u7075\u795e\uff09\u5a01\u529b\u52a0\u500d\u3002\n\\cFFCC00[\u653e\u5728\u9053\u5177\u680f\u4e2d\u6709\u6548]", "\u5bf9\u67d0\u4e9b\u81ea\u604b\u7684\u795e\u4ed9\u4f24\u5bb3\u52a0\u500d\u3002\n\\cFFCC00[\u653e\u5728\u9053\u5177\u680f\u4e2d\u6709\u6548]", "\u6253\u602a\u5f97\u5230\u7684\u91d1\u94b1\u52a0\u500d\u3002\n\\cFFCC00[\u653e\u5728\u9053\u5177\u680f\u4e2d\u6709\u6548]", "\u53ef\u4ee5\u5f00\u542f\u9ec4\u95e8\u3002", "\u53ef\u4ee5\u5f00\u542f\u7ea2\u95e8\u3002", "\u53ef\u4ee5\u5f00\u542f\u84dd\u95e8\u3002", "\u52a0\u653b\u51fb\u3002", "\u52a0\u9632\u5fa1\u3002", "\u52a0\u8840\u3002", "\u52a0\u8840\u3002", "\u5f00\u542f\u5f53\u524d\u5c42\u6240\u6709\u9ec4\u95e8", "\u5982\u6765\u5f00\u201c\u6148\u60b2\u4e3a\u6000\u201d\u5de1\u56de\u4f5b\u7ecf\u6f14\u5531\u4f1a\u7684\u65f6\u5019\uff0c\u4f34\u594f\u7f57\u6c49\u7528\u7684\u4e50\u5668\uff0c\u9053\u884c\u6d45\u7684\u654c\u4eba\uff0c\u4f1a\u88ab\u5176\u68b5\u5929\u4f5b\u97f3\u77ac\u95f4\u5316\u4e3a\u7070\u98de\n\\cFFCC00\u4f7f\u7528\uff1a\u6740\u6b7b\u4e0a\u4e0b\u5de6\u53f3\u7684\u654c\u4eba\uff0c\u5bf9BOSS\u4e0d\u8d77\u4f5c\u7528\u3002"};
        this.l = new String[]{"", "\\cdddddd\u4e00\u6839\u76f8\u5f53\u957f\u7684\u6728\u5236\u957f\u68cd,\u65b0\u624b\u5fc5\u5907.\u6709\u4e86\u5b83\u6740\u4eba\u8d8a\u8d27\u4e0d\u614c\u4e0d\u6101.\n\\c00ff00\u88c5\u5907: \u653b\u51fb+10.\n\\cFFCC00\"\u770b\u4e0a\u53bb\u4f3c\u4e4e\u4f1a\u65ad\u6389\u3002\".", "\\cdddddd\u4e4c\u9ed1\u6cb9\u4eae\uff0c\u663e\u7136\u7ecf\u5386\u8fc7\u591a\u4eba\u4e4b\u624b\u3002\n\\c00ff00\u88c5\u5907: \u653b\u51fb+30.\n\\cFFCC00\"\u5f88\u7c97\u5f88\u7ed3\u5b9e\uff01\".", "\\cdddddd\u94f6\u68cd\uff0c\u6069\uff0c\u6709\u8fd9\u4e2a\u540d\u5b57\u5c31\u8db3\u591f\u4e86\u3002\n\\c00ff00\u88c5\u5907: \u653b\u51fb+70.\n\\cFFCC00\"\u53ea\u662f\u6839\u94f6\u68cd\".", "\\cdddddd\u56e0\u4e58\u5929\u5730\u4e4b\u7075\u6c14\uff0c\u96c6\u65e5\u6708\u4e4b\u7cbe\u534e\u4e43\u201c\u4e07\u6728\u4e4b\u7075\uff0c\u7075\u6728\u4e4b\u5c0a\u201d\u3002\n\\c00ff00\u88c5\u5907: \u653b\u51fb+120.\n\\cFFCC00\"\u6728\u4e4b\u7cbe\u534e\uff0c\u524a\u94c1\u65ad\u91d1\".", "\\cdddddd\u60a8\u7684\u9700\u8981\uff0c\u5b83\u77e5\u9053\uff1b\u60a8\u7684\u9700\u6c42\uff0c\u5b83\u6ee1\u8db3\u3002\u5b83\u597d\uff0c\u4f60\u4e5f\u597d\uff0c\u9f99\u738b\u540e\u5bab\uff0c\u9547\u5bab\u4e4b\u5b9d\uff01\n\\c00ff00\u88c5\u5907: \u653b\u51fb+220.\n\\cFFCC00\"\u4e0d\u8981\u8ff7\u604b\u5b83\uff0c\u5b83\u53ea\u662f\u4e00\u6839\u4f20\u8bf4\u3002\".", "", "\\cdddddd\u6ca1\u6709\u592a\u591a\u7684\u88c5\u9970\uff0c\u4e00\u4ef6\u975e\u5e38\u6734\u7d20\u3001\u8f7b\u4fbf\u7684\u5e03\u8863.\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+10.\n\\cFFCC00\"\u770b\u4e0a\u53bb\u6709\u4e0d\u5c11\u4eba\u7528\u8fc7\u4e86\u3002\".", "\\cdddddd\u4fdd\u6696\u5fa1\u5bd2\uff0c\u8170\u4e0d\u9178\uff0c\u817f\u4e0d\u75bc\uff0c\u8d70\u8def\u4e5f\u6709\u52b2\u4e86\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+30.\n\\cFFCC00\"\u8c79\u7eb9\uff0c\u6027\u611f\u53c8\u91ce\u6027\uff0c\u4eca\u5e74\u5929\u5bab\u6700\u6d41\u884c\u7684\u76ae\u8349\u6b3e\u5f0f\".", "\\cdddddd\u5982\u679c\u6ca1\u6709\u4e0a\u9762\u7684\u90a3\u884c\u5b57\uff0c\u5b83\u4e5f\u7b97\u662f\u4e2a\u6770\u4f5c\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+70.\n\\cFFCC00\"\u4e0a\u9762\u5199\u7740'\u529e\u56db\u7ea7\u795e\u4ed9\u8bc1\u4e66\uff0c\u56de\u6536\u4e8c\u624b\u83b2\u82b1\u5b9d\u5ea7'\".", "\\cdddddd\u534e\u4e3d\u7684\u88c5\u9970\uff0c\u5c31\u662f\u6709\u70b9\u65e7\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+120.\n\\cFFCC00\"\u522b\u4eba\u7a7f\u8fc7\u7684\u6781\u54c1\u3002\".", "\\cdddddd\u4e1c\u6d77\u9f99\u9cde\u7f16\u7ec7\u800c\u6210\uff0c\u9650\u91cf\u7248\uff0c\u5929\u4e0a\u5929\u4e0b\uff0c\u53ea\u6b64\u4e00\u6b3e\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+220.\n\\cFFCC00\"\u66f4\u8f7b\u8584\uff0c\u66f4\u900f\u6c14\uff0c\u66f4\u591a\u9632\u62a4\uff0c\u66f4\u591a\u5b89\u5fc3\"."};
        this.q = new byte[]{-1, -1, -1, -1, 1, 1, 1, 3, 1, 1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1};
        this.aN = 0;
        this.aO = 0;
        this.aP = 0;
        this.aQ = 0;
        this.r = new byte[32];
        this.s = new byte[32];
        int[] nArray4 = new int[]{0, 1, 2, 4, 8, 16, 32};
        this.o = new int[]{0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220};
        this.t = new byte[]{0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40};
        this.m = new String[]{"\u5b59\u609f\u7a7a", "\u9ec4\u95e8", "\u7ea2\u95e8", "\u84dd\u95e8", "\u5c01\u5370\u95e8", "\u95e8\u536b", "\u9690\u5f62\u8def\u5f84", "\u4e0a\u697c\u68af", "\u4e0b\u697c\u68af", "\u70bc\u4e39\u7089", "\u4e91\u96fe", "\u80fd\u6316\u7684\u5899", "\u9690\u5f62\u5899", "\u706b\u773c\u91d1\u775b", "\u751f\u6b7b\u7c3f", "\u7b4b\u6597\u4e91", "\u82ad\u8549\u6247", "\u91d1\u52fa\u5b50", "\u7384\u660e\u77f3", "\u5343\u5e74\u6708\u6842\u9732", "\u76f8\u5f62\u53d8\u4f4d", "\u4e0a\u884c\u7b26", "\u4e0b\u884c\u7b26", "\u592a\u516c\u6756", "\u6346\u4ed9\u7ef3", "\u5e78\u8fd0\u5e01", "\u9ec4\u94a5\u5319", "\u7ea2\u94a5\u5319", "\u84dd\u94a5\u5319", "\u5a01\u529b\u4e39", "\u91d1\u521a\u4e39", "\u56de\u6625\u4e38", "\u957f\u5bff\u4e39", "\u6843\u6728\u68d2", "\u7384\u94c1\u68d2", "\u771f\u94f6\u68d2", "\u91d1\u7b8d\u68d2", "\u9053\u888d", "\u94c1\u7532", "\u9501\u5b50\u7532", "\u7d2b\u91d1\u9f99\u9cde\u7532", "\u5929\u5bab\u5c0f\u72ac", "\u5929\u5bab\u5927\u72ac", "\u4f34\u7089\u7ae5\u5b50", "\u62a4\u5ead\u5c0f\u795e", "\u5b88\u56ed\u4ed9\u5a62", "\u62a4\u5ead\u6821\u5c09", "\u5de1\u5929\u536b\u58eb", "\u5de1\u5929\u536b\u58eb", "\u5de8\u529b\u58eb", "\u6267\u706b\u9053\u4eba", "\u594e\u6728\u72fc", "\u6267\u74f6\u4ed9\u4f8d", "\u91d1\u521a\u529b\u58eb", "\u62a4\u6301\u8fe6\u84dd", "\u4f8d\u6848\u4ed9\u5973", "\u62a4\u5ead\u795e\u5c06", "\u8d64\u529b\u97e6\u9a6e", "\u62a4\u4e39\u8001\u9053", "\u4f0f\u9b54\u97e6\u9a6e", "\u515c\u7387\u5bab\u661f\u541b", "\u65e0\u91cf\u62a4\u6cd5", "\u515c\u7387\u5bab\u8001\u4ed9", "\u4e8c\u90ce\u6267\u65d7\u5c06", "\u6740\u7834\u72fc", "\u4e8c\u90ce\u9a81\u9a91\u5c06", "\u5c0a\u5929\u97e6\u9a6e", "\u5578\u5929\u72ac", "\u592a\u4e0a\u8001\u541b", "\u6768\u622c", "\u7389\u7687\u5927\u5e1d", "\u7389\u7687\u5927\u5e1d", "\u725b\u9b54\u738b", "\u5929\u84ec\u5143\u5e05", "\u5de8\u7075\u795e", "\u54ea\u5412", "\u8def\u70b9", "\u592a\u767d\u91d1\u661f", "\u5929\u5bab\u5546\u4eba", "\u4e4c\u91d1\u68cd", "\u94f6\u7f15\u9501\u7532", "\u5c01\u5370\u95e8", "\u4f20\u602a\u70b9", "\u5267\u60c5\u70b9", "\u5ae6\u5a25", "\u9ec4\u91d1\u94a5\u5319", "\u65e5\u6708\u65e0\u6781\u94b9", "\u83e9\u63d0\u8001\u7956"};
        this.p = new int[]{35, 45, 35, 50, 60, 55, 100, 50, 260, 60, 130, 100, 320, 20, 320, 100, 210, 220, 160, 200, 230, 220, 200, 360, 180, 180, 1200, 4500, 1500, 8000, 800, 5000, 120, 444, 100};
        this.q = new int[]{18, 20, 38, 42, 32, 52, 180, 48, 85, 100, 60, 95, 120, 100, 140, 680, 200, 180, 230, 380, 450, 370, 390, 310, 430, 460, 180, 560, 600, 5000, 500, 1580, 150, 199, 65};
        this.r = new int[]{1, 2, 3, 6, 8, 12, 110, 22, 5, 8, 3, 30, 15, 68, 20, 50, 65, 30, 105, 130, 100, 110, 90, 20, 210, 360, 20, 310, 250, 1000, 100, 190, 50, 66, 15};
        this.s = new int[]{1, 2, 3, 6, 5, 8, 100, 12, 18, 12, 8, 22, 30, 28, 30, 55, 45, 35, 65, 90, 100, 80, 50, 40, 120, 200, 100, 1000, 800, 500, 500, 500, 100, 144, 30};
        this.n = new String[]{"CES_84_6_11 MOV_0_5_11 TAK_8_9 CES_70_5_8 TAK_10_10 ROS_4_1 TAK_11_17 ROS_4_2 TAK_18_19 MOV_0_5_10 TAK_20_21 DES_70_5_8 LAY_2 ROS_1_4_7 ROS_2_0 ROS_2_6 RES_0 GUT_1 ", "TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8 ", "TAK_33_36 MOV_72_1_8_1_10 DES_72_1_10 ", "TAK_37_37 MOV_0_6_5 TAK_38_39 TAK_41_41 DES_44_1_3 DES_44_2_3 DES_44_3_3 DES_46_2_4 DES_44_9_3 DES_44_10_3 DES_44_11_3 DES_46_10_4 CES_44_5_4 CES_46_6_4 CES_44_7_4 CES_44_5_5 CES_44_7_5 CES_44_5_6 CES_46_6_6 CES_44_7_6 SWD TAK_42_42 ", "CES_72_1_11 TAK_43_44 MOV_0_6_3 MOV_72_1_11_6_2 ROS_4_1 TAK_45_48 MOV_72_6_2_6_1 DES_72_6_1 ", "TAK_49_52 MOV_72_9_1_7_1 DES_72_7_1 ", "TAK_53_58 ", "TAK_60_60 MOV_72_3_2_8_4 TAK_61_61 CES_47_8_3 CES_47_8_5 TAK_62_63 DES_72_8_4 DES_47_8_3 DES_47_8_5 ADD_2_72_11_10 ", "ROS_4_1 CES_73_10_1 MOV_73_10_1_6_9 TAK_68_74 MOV_73_6_9_6_10 ", "SWD ", "CES_72_3_10 MOV_72_3_10_2_10 MOV_72_2_10_4_9 TAK_76_78 DES_72_4_9 ", "SWD ", "TAK_84_87 MOV_58_5_4_6_8 MOV_58_4_4_6_8 MOV_58_3_4_6_8 MOV_57_7_4_6_8 MOV_57_8_4_6_8 MOV_57_9_4_6_8 MOV_56_4_2_6_8 MOV_56_3_2_6_8 MOV_56_2_2_6_8 MOV_59_8_2_6_8 MOV_59_9_2_6_8 MOV_59_10_2_6_8 TAK_88_90 MOV_73_6_2_6_8 ", "CES_6_10_2 CES_60_10_2 ", "ROS_4_1 TAK_92_97 DES_73_6_8 TAK_98_100 DES_70_6_7 ", "ROS_4_1 CES_61_5_2 CES_61_6_2 CES_61_7_2 CES_61_5_3 CES_70_6_3 CES_61_7_3 CES_61_5_4 CES_61_6_4 CES_61_7_4 TAK_118_121 ", "TAK_123_126 GUT_37 ", "TAK_68_74 ", "DES_1_4_4 CES_20_4_4 ", "CES_84_7_7 MOV_84_7_7_6_8 TAK_0_1 MOV_84_6_8_1_8 TAK_2_2 MOV_0_2_8 TAK_3_3 MOV_84_1_8_1_1 TAK_4_4 MOV_0_1_2 TAK_5_5 MOV_84_1_1_10_1 MOV_0_6_1 DES_84_10_1 TAK_6_7 MOV_0_11_1 MOV_0_1_11 ", "TAK_40_40 ", "TAK_59_59 ", "TAK_75_75 ", "TAK_84_87 GUT_12 ", "TAK_101_105 ", "CES_22_6_6 ", "TAK_129_132 DES_84_11_4 GIN_1_1000 ", "TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13 ", "TAK_136_143 DES_84_1_11 TAK_144_144 GIN_0_19 ", "TAK_145_146 DES_84_9_8 GIN_1_1000 ", "MOV_84_6_3_4_3 MOV_84_4_3_8_3 MOV_84_8_3_6_3 TAK_107_107 ", "TAK_108_108 DES_10_6_6 MOV_0_6_5 TAK_109_112 TAK_147_148 DES_84_6_3 TAK_149_149 ", "TAK_79_79 ", "TAK_155_160 DES_72_11_10 ", "TAK_150_154 ", "TAK_161_161 ", "TAK_162_162 ", "GLV_1 ", "CES_6_4_1 CES_61_4_1 ", "", "TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 ", "TAK_167_167 SEE_4_10_168_168_0 ROS_4_1 SEE_2_8_169_169_0 SEE_2_8_170_170_1 ", "TAK_171_171 SEE_7_10_172_172_0 SEE_7_10_173_173_1 ", "TAK_174_174 SEE_7_9_175_175_0 TAK_176_176 SEE_8_8_177_178_0 ", "TAK_179_180 ROS_4_3 SEE_8_6_181_182_0 ROS_4_1 SEE_10_4_183_183_0 ", "ROS_4_1 SEE_8_3_184_184_0 ROS_4_0 SEE_6_7_185_185_0 ", "SEE_4_4_186_186_0 ROS_4_1 SEE_6_2_187_187_1 ", "ROS_4_0 SEE_6_2_188_188_0 SEE_4_4_189_189_1 ", "TAK_190_190 ROS_4_1 SEE_2_2_191_192_0 ROS_4_0 SEE_2_6_193_196_0 SEE_2_6_197_198_1 ", "ROS_4_3 SEE_1_11_199_200_0 ", "ROS_4_2 SEE_6_11_201_202_0 ROS_4_3 SEE_3_8_203_204_0 CES_6_3_10 SEE_3_8_205_205_1 ", "TAK_206_209 ", "TAK_210_211 SEE_11_11_212_212_0 SEE_11_7_213_214_0 TAK_215_216 ", "TAK_217_218 ROS_5_510 ROS_6_510 CES_36_11_8 SEE_11_8_219_221_0 CES_40_11_9 SEE_11_9_222_222_0 SEE_8_9_224_225_0 SEE_10_9_223_223_0 DES_11_10_9 ", "TAK_224_224 SEE_6_2_225_225_0 ", "TAK_226_227 ROS_5_510 ROS_6_510 CES_36_5_1 CES_40_7_1 TAK_228_228 ", "TAK_229_231 MOV_0_6_7 TAK_232_232 DES_51_2_8 DES_51_1_8 DES_51_2_9 DES_51_1_9 CES_51_6_6 CES_51_5_7 CES_51_6_8 CES_51_7_7 GUT_57 ", "MOV_51_6_6_6_7 ROS_4_3 MOV_51_5_7_6_7 ROS_4_0 MOV_51_6_8_6_7 ROS_4_2 MOV_51_7_7_6_7 ROS_4_1 TAK_233_235 MOV_74_7_4_6_7 GUT_61 ", "", "", "", "TAK_236_238 MOV_75_5_4_6_7 TAK_239_239 GUT_62 ", "CES_69_5_5 CES_47_5_6 CES_47_5_7 CES_47_5_8 CES_56_7_6 CES_56_7_7 CES_56_7_8 TAK_240_245 GUT_63 ", "CES_77_6_6 TAK_246_250 DES_71_6_3 DES_77_6_6 DES_69_5_5 DES_-66_0 DES_-56_0 DES_-49_0 DES_-47_0 DES_-69_0 ", "TAK_253_254 ", "TAK_127_128 END_0 ", "TAK_261_263 SMS_0 ", "TAK_264_268 "};
        this.v = new byte[]{0, 84, 0, 84, 0, 84, 0, 0, 84, 0, 70, 0, 70, 0, 70, 0, 70, 0, 84, 0, 0, 70, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 75, 0, 75, 75, 75, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 74, 0, 74, 0, 74, 0, 74, 72, 72, 47, 72, 0, 72, 72, 0, 73, 0, 73, 0, 73, 0, 73, 73, 72, 0, 72, 72, 0, 72, 72, 0, 73, 0, 73, 0, 73, 0, 73, 0, 70, 73, 70, 73, 70, 73, 73, 0, 70, 69, 0, 69, 0, 69, 84, 0, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 70, 0, 70, 0, 70, 72, 0, 72, 0, 0, -1, 0, 84, 0, 84, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 0, 0, 0, 69, 69, 0, 69, 0, 0, 72, 0, 72, 0, 72, 0, 0, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 0, 87, 87, 87, 87, 87, 87, 0, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 71, 71, 0, 71, 71, 74, 71, 75, 0, 71, 71, 69, 47, 69, 56, 69, 47, 77, 77, 77, 71, 0, 56, 71, 0, 0, 87, 0, 87, 0, 87, 0, 87, 0, 0, 87, 87, 87, 87, 87};
        this.w = new byte[]{7, 5, -1, 3, -1, 5, -1, 1, 10, 0, -1, -1, -1, -1, -1, -1, -1, 1, 9, 7, 8, -1, -1, 5, 11, 2, 6, -1, -1, -1, -1, 3, 11, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 11, -1, 5, 0, 5, 9, 2, 10, 3, -1, -1, 11, -1, -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, 10, -1, -1, 9, 0, 1, -1, -1, -1, -1, -1, -1, 3, 11, -1, 0, 3, -1, -1, -1, -1, -1, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, -1, 5, -1, 1, 3, -1, 8, 9, 8, 9, -1, -1, -1, 8, 4, 5, 6, 10, 8, -1, -1, 5, 9, 1, -1, 0, 3, 7, -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, -1, -1, 0, -1, -1, -1, -1, -1, -1, 7, -1, 7, -1, 7, 2, 2, -1, 2, -1, -1, -1, 7, -1, -1, -1, -1, -1, -1, -1, -1, 9, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1};
        this.o = new String[]{"\u59d1\u5a18\uff0c\u60a8\u2026\u2026", "\u54ce\u5440\uff0c\u59d1\u5976\u5976\u6211\u597d\u4e0d\u5bb9\u6613\u6000\u63e32\u4e2a\u4ed9\u6843\u5939\u5e26\u51fa\u6765\uff0c\u5c45\u7136\u88ab\u53d1\u73b0\u4e86~\uff01\u5feb\u95ea\uff01\uff01", "\u2026\u2026\uff0c\u8dd1\u90fd\u8dd1\u7684\u8fd9\u4e48\u4f18\u96c5\u3002", "\u54ce\u5440\uff0c\u8dd1\u8fd9\u4e48\u5feb\u2026\u2026", "\u4fee\u9017\u5988\u5f85~\uff01\u2026\u2026", "\u6b7b\u7334\u5b50\uff0c\u7a77\u8ffd\u731b\u6253\uff0c\u8ffd\u7684\u4eba\u5bb6\u5c0f\u5fc3\u809d\u5657\u55f5\u5657\u55f5\u5730\u2026\u2026", "\u554a~~~\u59d1\u5a18\u54df~~\u59d1~~~\u5a18~~\uff01\uff01", "\u6b7b\u5f00\u6b7b\u5f00~\uff01\uff01\u597d\u72d7\u4e0d\u6321\u8def\u3002", "\u5c45\u7136\u8ffd\u5230\u8fd9\u91cc\u4e86\uff0c\u7b97\u4e86\u5427\uff0c\u8ba4\u4e86\u5427\u3002\n\\cFFCC00\u4e24\u4e2a\u6843\u5b50\u6eda\u843d\u5730\u4e0a", "\u54ce~~~~~~~", "\u6211\u54a4~~~~\uff01\u5929\u5ead\u5723\u5730\uff0c\u7981\u6b62\u55a7\u54d7\u3002", "\u6211~~~~", "\u6211\u4ec0\u4e48\u6211~\uff1f\uff1f\u6574\u5929\u8eab\u5c45\u4ed9\u4f4d\uff0c\u6e38\u624b\u597d\u95f2\u2026\u2026", "\u4f60~~~~~~~~~", "\u4f60\u4ec0\u4e48\u4f60~\uff1f\uff1f\u770b\u5ae6\u5a25\u8863\u6749\u4e0d\u6574\uff0c\u662f\u4e0d\u662f\u4f60\u975e\u793c~\uff1f\uff1f\u662f\u4e0d\u662f\u4f60\u662f\u4e0d\u662f\u4f60\u662f\u4e0d\u662f\u4f60\uff01\uff1f", "\u5979~~~~~~~~~~", "\u5979\u4ec0\u4e48\u5979~\uff1f\uff1f\u54e6~\uff01\u8fd8\u6eda\u51fa\u67652\u4e2a\u4ed9\u6843\uff01\uff01\u5047\u516c\u6d4e\u79c1\u662f\u5427~\uff01\uff01\u7334\u5b50\u5077\u6843\u662f\u5427\uff01\uff01", "\u4ffa\u6ca1~\uff01\u2026\u2026", "\u2026\u2026", "\u4f60\u2026\u2026\u53eb\u5ae6\u5a25\uff1f\u8fd9\u4e2a\u540d\u5b57\uff0c\u4ffa\uff0c\u8bb0\u3002\u4f4f\u3002\u4e86\u2026\u2026", "\u6ca1\u9519\uff0c\u6843\u5b50\u5c31\u662f\u4ffa\u5077\u7684\uff01\uff01\u4ffa\u8ba4\u7f5a\uff01", "\u5c06\u8fd9\u53ea\u5b7d\u755c\u5265\u4e0b\u94e0\u7532\u6253\u4e0b\u5929\u7262\uff01", "\u8d24\u5f1f\uff0c\u5feb\u9192\u9192\u2026\u2026", "\u54ce\u5466\u2026\u2026\u8001\u725b\uff0c\u9192\u6765\u89c1\u4f60\u90a3\u5f20\u8138\uff0c\u771f\u63d0\u795e\uff01\uff01", "\u6211\u5b81\u613f\u770b\u7740\u4f60\uff0c\u7761\u5f97\u5982\u6b64\u6c89\u9759\uff0c\u80dc\u8fc7\u4f60\u9192\u65f6\u51b3\u88c2\u822c\u65e0\u60c5~~", "\u9760\uff0c\u53d1\u6625\u5450\uff0c\u628a~\u624b~\u62ff~\u5f00~~\uff01", "\u54df~~\u633a\u6a2a\u5f97\u563f~\u725b\u54e5\u5b66\u5f97\u4e00\u624b\u597d\u6444\u5f71\u2026\u2026\u521a\u624d\u2026\u2026", "\u2026\u2026\u5927\u54e5\u4e45\u8fdd\u4e86\uff0c\u591a\u65e5\u4e0d\u89c1\uff0c\u53d7\u5c0f\u5f1f\u4e00\u62dc\uff01\uff01", "\u8bdd\u8bf4\u4f60\u4e00\u76f4\u5728\u68a6\u4e2d\u53eb\u7740\u4e00\u4e2a\u4eba\u7684\u540d\u5b57\uff0c\u5979\u4e00\u5b9a\u5077\u4e86\u4f60\u5f88\u591a\u6843\u5b50\u2026\u2026", "\u6ca1\u9519\uff0c\u5979\u662f\u5077\u4e86\u6211\u7684\u4e1c\u897f\uff0c\u4f46\u662f\u4e0d\u662f\u6843\u5b50\uff0c\u6211\u5f88\u60f3\u518d\u89c1\u5979\u2026\u2026", "\u6069\u6069\uff0c\u73b0\u5728\u5927\u54e5\u5e26\u4f60\u4ece\u5bc6\u9053\u51fa\u53bb\u2026\u2026", "\u2026\u2026\u8fd9\u4f60\u90fd\u80fd\u6316\u5f00~\uff01\uff01", "\u54e5\u5728\u9b54\u754c\u6709\u4e2a\u7ef0\u53f7\uff0c\u53eb\u201c\u6e9c\u5f97\u6ed1\u201d\uff0c\u6ca1\u6709\u4ec0\u4e48\u56da\u7262\u80fd\u56f0\u4f4f\u54e5\u2026\u2026", "\u5927\u54e5\uff01\uff01\u80fd\u4e0d\u80fd\u5206\u6211\u4ef6\u4e1c\u897f\u906e\u906e\u7f9e\u2026\u2026", "\u6709\uff0c\u6211\u8fd9\u6709\u628a\u521a\u6316\u5730\u9053\u7684\u7834\u52fa\uff0c\u4f60\u62ff\u53bb\u6321\u4f4f\u5148\uff01", "\u9760\u2026\u2026\u7b97\u4e86\uff0c\u603b\u6bd4\u6ca1\u6709\u597d\u3002\u2026\u2026", "\u6ca1\u529e\u6cd5\uff0c\u4f60\u5c31\u5c06\u5c31\u4e00\u4e0b\u5566\u3002\\cFFCC00\u6709\u4ef6\u9053\u7ae5\u7684\u65e7\u888d\u5b50\u57289\u697c\uff0c\u4f60\u4ed4\u7ec6\u627e\u627e\u5427\uff0c\\cF8F8F8\u6211\u5148\u64a4\u4e86\u2026\u2026", "\u5996\u7334\uff0c\u4f60\u53ef\u8ba4\u5f97\u672c\u5c11\u7237\uff01\uff01", "\u549d~~\u6211\u770b\u4f60\u9aa8\u9abc\u6e05\u5947\u4e09\u5934\u516d\u81c2\uff0c\u731c\u5f97\u4e0d\u9519\u7684\u8bdd\uff0c\u9601\u4e0b\u4e00\u5b9a\u662f\u7578\u5f62\u513f\uff01", "\u554a~\uff01\u5927\u54e5\u679c\u7136\u9ad8\u624b\uff01\u8fd9\u90fd\u88ab\u4f60\u770b\u7a7f\u4e86\uff01\u4e0d\u80fd\u7559\u4f60\u6d3b\u53e3\u4e86\uff01", "\u54ce\u5440\u5440~~\u6211\u56de\u53bb\u544a\u8bc9\u6211\u7239\u53bb\uff01", "\u6765\u4eba\uff01\u628a\u5996\u7334\u62ff\u4e0b\uff01", "\u54e6~~~\u55b3~\uff01\u674e\u5c0f\u7334\u8e22\u9986\uff01\uff01", "\u8d24\u5f1f\uff0c\u6162\u7740\uff01", "\u6e9c\u5f97\u6ed1\uff1f", "\u6211\u5077\u5077\u544a\u8bc9\u4f60\u54e6\uff0c\u6709\u4e2a\u94f6\u68cd\u572817\u5c42\uff0c\u4f60\u627e\u5230\u5b83\u4ee5\u540e\u5c31\u80fd\u5389\u5bb3\u5f88\u591a\uff1f", "\u4e86\u89e3\uff01", "\u7ea2\u4e86\u7ea2\u8138", "\u5c3d\u5feb\u53bb\u5427\uff0c\u6211\u95ea\u5148\u2026\u2026", "\u8d24\u5f1f\uff0c\u8fd9\u53ea\u5578\u5929\u72ac\u5728\u8fd9\u91cc\u6321\u9053\u592a\u5371\u9669\u4e86\uff0c\u8ddf\u54e5\u54e5\u7ed5\u5bc6\u9053\u5427\u3002", "\u4e00\u53ea\u770b\u95e8\u7684\u5ba0\u7269\uff0c\u80fd\u51f6\u5230\u54ea\u91cc\uff1f", "\u4e0d\u662f\uff0c\u81ea\u4ece\u8c03\u5230\u5929\u5ead\u5b88\u8def\uff0c\u5df2\u7ecf\u5f88\u591a\u5929\u6ca1\u6709\u5403\u8364\u8165\u4e86\u2026\u2026", "\u4e86\u89e3\uff0c\u7ed5\u9053\uff0c\u8d70~\uff01", "\u6211\u4e43\u6258\u5854\u5929\u738b\u9ebe\u4e0b\u5148\u950b\u5b98\uff0c\u5de8\u73b2\u795e\u662f\u4e5f\uff01\u4f60\u6b3a\u6211\u5b69\u513f\u2026\u2026", "\u6b3a\u4f60\u5b69\u513f~\uff1f", "\u5176\u5b9e\u2026\u2026\u54ea\u5412\u662f\u5974\u5bb6\u8ddf\u6258\u5854\u5929\u738b\u7684\u79c1\u751f\u5b50\u2026\u2026", "\u4f60\u662f\u5973\u5c06\uff01\uff1f", "\u5176\u5b9e\uff0c\u770b\u6211\u50cf\u8299\u84c9\u59d0\u59d0\u4e00\u6837\u7684\u597d\u8eab\u6bb5\uff0c\u4f60\u5c31\u5e94\u8be5\u4e86\u89e3\u2026\u2026", "\u2026\u2026\u549d~~\u6258\u5854\u5929\u738b\u53e3\u5473\u8fd9\u4e48\u504f\u2026\u2026", "\u554a~~~\u5929\u738b\uff0c\u5de8\u73b2\u513f\u4e0d\u80fd\u966a\u4f34\u4f60\u4e86\u2026\u2026", "\u770b\u6765\u4f60\u5df2\u7ecf\u6253\u901a\u6697\u5899\u4e86\uff0c\u770b\u54e5\u7684\u3002", "\u5662\uff0c\u901a\u5b8c\u6536\u5de5\uff0c\u8d70\uff01\uff01\u2026\u2026", "\u6211\u4eec\u662f\u5929\u5ead\u5e02\u5bb9\u7ba1\u7406\u961f\uff01\uff01\u554a\u54c8~\u4e71\u5806\u6e23\u571f\uff0c\u8fdd\u89c4\u65bd\u5de5\u7ec8\u4e8e\u88ab\u6293\u4e2a\u73b0\u884c\uff01", "\u54ce\u5440~\uff01\u5144\u5f1f\uff0c\u4e00\u5b9a\u8981\u67652\u5c42\u5929\u7262\u6551\u6211\u5440\uff01", "", "", "", "", "\u4f60\u5c31\u662f\u5b59\u609f\u7a7a\uff01\uff1f", "\u4ffa\u5c31\u662f\u3002", "\u6211\u5e38\u53bb\u5e7f\u5bd2\u5bab\u770b\u5979\uff0c\u5979\u8fd9\u51e0\u5929\u5e38\u5e38\u63d0\u5230\u4f60\u2026\u2026\u6211\u4ece\u6765\u4e0d\u77e5\u9053\u4ec0\u4e48\u53eb\u505a\u5ac9\u5992\uff0c\u4f46\u662f\u8fd9\u6b21\uff0c\u6211\u60f3\u8981\u4f60\u7684\u547d\u3002", "\u54fc\uff0c\u4f60\u54ea\u4f4d\uff1f", "\u6211\u4e43\u5929\u84ec\u5143\u5e05\uff0c\u6731\u521a\u9b23\uff01\uff01\u638c\u7ba1\u5929\u6cb3\u2026\u2026", "\u732a\u809b\u88c2\uff1f\uff1f\u597d\uff0c\u6ee1\u8db3\u4f60\u8fd9\u4e2a\u613f\u671b\u2026\u2026", "\u54ce\u2026\u2026\u6211\u8fd8\u6ca1\u62a5\u5b8c\u5462\u2026\u2026", "\u83ca\u82b1\u6b8b~~~\u6ee1\u5730\u4f24\u2026\u2026", "\u4f60\u8ddf\u6768\u622c\u4e00\u6218\uff0c\u5929\u5ead\u90fd\u5f00\u4e86\u76d8\u53e3\uff0c\u8d54\u7387\u662f1:5\uff0c\u54e5\u628a\u79c1\u623f\u94b1\u90fd\u62bc\u5230\u4f60\u5934\u4e0a\u4e86\uff0c\u8868\u8f9c\u8d1f\u54e5\u54e5\u54df~", "\u2026\u2026\u771f\u60f3\u80cc\u540e\u7ed9\u4f60\u4e00\u95f7\u68cd\u2026\u2026", "\u5929\u5ead\u7981\u6b62\u4e71\u5806\u6e23\u571f\uff0c\u54e5\u54e5\u628a\u6e23\u571f\u8fd0\u5230\u201c\u5929\u5ead\u57ce\u7ba1\u529e\u4e8b\u5904\u201d\u53bb\uff0c\u5c31\u4e0d\u7b97\u4e71\u5806\u4e86\uff0c\u634f\u563f\u563f~~", "\\cFFCC0023\u5c42\u4e43\u662f29\u5c42\u7684\u5730\u57fa\u6240\u5728\uff0c\u627e\u51fa\u6697\u85cf\u7684\u5899\uff0c\u5c31\u53ef\u4ee5\u8ba929\u5c42\u7684\u5899\u677e\u52a8\uff0c\u54e5\u54e5\u5c31\u53ef\u4ee5\u6316\u7a7f\u5b83\u3002", "\u2026\u2026\u4f60\u4e2a\u5047\u4ed7\u4e49\uff01", "\u5144\u5f1f\uff0c\u8d76\u5feb\u53bb\u51d1\u9f50\u88c5\u5907\uff0c\u6253\u8d25\u6768\u622c\uff0c", "\u54e5\u54e5\u5c31\u53d1\u8fbe\u4e86~\u563f\u563f~", "\u2026\u2026", "\u4e0a\u6b21\u8d81\u672c\u5143\u5e05\u81ea\u62a5\u5927\u540d\u7684\u65f6\u5019\uff0c\u7a81\u88ad\u672c\u5e05\uff0c\u672c\u5e05\u4e0d\u8ddf\u4f60\u8ba1\u8f83\uff0c\u5355\u6311\u8fd8\u662f\u7fa4\u6bb4\uff0c\u4f60\u81ea\u5df1\u9009\u3002", "\u6069\uff0c\u662f\u6761\u6c49\u5b50\uff0c\u4ffa\u5c31\u8ba4\u771f\u8ddf\u4f60\u6253\u4e00\u6b21\uff0c\u5355\u6311\uff01", "\u5355\u6311\u662f\u5427\uff0c\u4f60\u4e00\u4e2a\u5355\u6311\u6211\u4eec\u5168\u90e8\uff0c\u5f1f\u5144\u4eec\uff0c\u4e00\u8d77\u4e0a\uff01", "\u4f60\u4e2b\u4e0d\u5730\u9053\uff01", "\u4f60\u7684\u786e\u662f\u4e2a\u82f1\u96c4\uff0c\u96be\u602a\u5979\u4e00\u76f4\u5ff5\u5ff5\u4e0d\u5fd8\u2026\u2026", "\u8fc7\u5956\u8fc7\u5956\uff0c\u4f60\u7684\u90e8\u4e0b\u90fd\u8eba\u4e0b\u4e86\uff0c\u73b0\u5728\u8f6e\u5230\u4f60\u4e86\u2026\u2026", "\u6069~~\u8ba8\u538c\u6b7b\u4e86\uff0c\u6765\u4e86\u6765\u4e86\u2026\u2026", "\u4eba\u5bb6\u4eca\u5929\u8eab\u4f53\u4e0d\u65b9\u4fbf\uff0c\u6539\u5929\u518d\u6765\uff0c\u5148\u95ea\u4e86", "\u5929\u84ec\uff0c\u4f60\u6570\u6b21\u6218\u609f\u7a7a\u4e0d\u80dc\u5012\u7f62\u4e86\uff0c\u5e73\u65f6\u5e38\u5e38\u64c5\u81ea\u79bb\u5c97\uff0c\u53bb\u5e7f\u5bd2\u5bab\u628a\u599e\u2026\u2026\u6b7b\u7f6a\u53ef\u514d\uff0c\u6d3b\u7f6a\u96be\u9976\u3002", "\u542c\u8bf4\uff0c\u4e0b\u51e1\u6295\u80ce\uff0c\u5c31\u4f1a\u5815\u5165\u8f6e\u56de\uff0c\u5c31\u4f1a\u5fd8\u8bb0\u524d\u5c18\u5f80\u4e8b\u2026\u2026", "\u597d\uff0c\u6715\u5c31\u6210\u5168\u4f60\uff0c\u4e0b\u51e1\u4e4b\u524d\uff0c\u6709\u4ec0\u4e48\u8981\u6c42\u4e48\uff1f", "\u5929\u5929\u5927\u5403\u5927\u559d\uff0c\u5012\u5934\u7761\u89c9\uff0c\u751f\u6d3b\u5b89\u9038\u65e0\u8fb9\uff0c\u5fc3\u5bbd\u4f53\u80d6\u2026\u2026", "\u5f88\u597d\uff0c\u4f60\u7684\u5fc3\u610f\uff0c\u6715\u660e\u767d\u4e86\uff0c\u5b89\u5fc3\u53bb\u5427", "\u54c7\uff01\uff01\u6295\u80ce\u4e3a\u732a\uff1f\uff1f\uff01", "\u5929\u84ec\u5143\u5e05\u53d8\u6210\u4e86\u4e00\u53ea\u732a\uff0c\u88ab\u8d2c\u4e0b\u4e86\u51e1\u5c18", "\u2026\u2026\u771f\u9634\u9669\u2026\u2026", "\u54fc\uff0c\u54fc\uff0c\u5be1\u4eba\u5728\u56db\u5341\u4e5d\u5c42\u7b49\u4f60\uff0c\u54c7\u54c8\u54c8\u54c8\u54c8~\uff01\uff01", "\u672c\u6765\uff0c\u6218\u795e\u60c5\u5723\u7684\u540d\u53f7\u662f\u6211\u7684\uff1b\u5ae6\u5a25\u7684\u5fc3\uff0c\u8fdf\u65e9\u4e5f\u4f1a\u5f52\u5c5e\u4e8e\u6211\uff0c\u4f46\u662f\u4f60\u6765\u4e86\u4e4b\u540e\uff0c\u4e00\u5207\u90fd\u6539\u53d8\u4e86\u2026\u2026", "\u4f60\u559c\u6b22\u5979\uff0c\u8fd9\u4e48\u591a\u5e74\uff0c\u4f60\u4e3a\u4ec0\u4e48\u4e0d\u53bb\u627e\u5979\uff1f", "\u56e0\u4e3a\u6211\u662f\u6218\u795e\u60c5\u5723\uff0c\u662f\u4e0d\u80fd\u5931\u8d25\u7684\u2026\u2026", "\u4f60\u592a\u9a84\u50b2\u4e86\u2026\u2026", "\u65e0\u8bba\u5982\u4f55\uff0c\u6597\u795e\u548c\u6218\u795e\u8fd9\u4e00\u6218\uff0c\u662f\u6ce8\u5b9a\u7684\u2026\u2026", " ", "\u54c7\uff0c\u59d1\u5a18\u8eab\u9677\u4e09\u6627\u771f\u706b\u5f53\u4e2d\uff0c\u8981\u60f3\u529e\u6cd5\u5f00\u95e8\u706d\u706b\u2026\u2026", "\u59d1\u5a18\u9876\u4f4f\uff0c\u4ffa\u8001\u5b59\u6765\u6551\u4f60\uff01\uff01", "\u4e0d\u8981\u4e0d\u8981\u8fc7\u6765\uff01\uff01", "\u59d1\u5a18\u4f60\u6ca1\u4e8b\u5427\uff1f", "\u6b7b\u7334\u5b50\uff0c\u6708\u5bab\u9634\u51b7\uff0c\u59d1\u5976\u5976\u6211\u60f3\u84b8\u84b8\u6851\u62ff\uff0c\u6cbb\u591a\u5e74\u7684\u5173\u8282\u708e\u90fd\u4e0d\u884c\u2026\u2026", "\u2026\u2026", "\u6b7b\u7334\u5b50\uff0c\u4e0a\u6b21\u6843\u5b50\u7684\u4e8b\u60c5\u2026\u2026", "\u4ffa\u638c\u7ba1\u87e0\u6843\u56ed\uff0c\u5077\u5403\u4ed9\u6843\u4f55\u6b62\u5343\u767e\uff0c\u591a\u8ba42\u4e2a\uff0c\u7b97\u4ec0\u4e48\u2026\u2026", "\u5bb3\u4f60\u88ab\u9769\u9664\u4e86\u201c\u9f50\u5929\u5927\u5723\u201d\u7684\u4e0a\u4ed9\u4e4b\u4f4d\u2026\u2026", "\u4ffa\u8001\u5b59\u4e0d\u7a00\u7f55\u5929\u5bab\u7684\u4f4d\u5b50\uff0c~\u8d2c\u4e0b\u51e1\u5c18\u4ecd\u79f0\u738b\uff0c\u563f\u563f", "\u2026\u2026\u5728\u5929\u5bab\u51e0\u5343\u5e74\uff0c\u4ece\u6765\u6ca1\u6709\u4eba\u80af\u4e3a\u6211\u653e\u5f03\u4ed9\u4f4d\u2026\u2026\u5509\uff0c\u53ef\u60dc\u3002", "\u54fc\u54fc\uff0c\u5c45\u7136\u6253\u5230\u8fd9\u91cc\uff0c\u5b9e\u8bdd\u544a\u8bc9\u4f60\uff0c\u6240\u6709\u5929\u795e\u90fd\u5bf9\u4f60\u4e0d\u6ee1\uff0c\u8fd9\u6b21\u4f60\u88ab\u524a\u53bb\u4ed9\u7235\u6253\u5165\u5929\u7262\uff0c\u90fd\u662f\u8ba1\u5212\u4e4b\u4e2d\u3002", "\u90a3\u5ae6\u5a25\u5462\uff0c\u6843\u5b50\u5462\uff1f\u4e5f\u5728\u8ba1\u5212\u4e4b\u4e2d\uff1f\u4f60\u4eec\u6599\u5b9a\u4ffa\u4f1a\u7518\u5fc3\u9876\u7f6a\uff1f", "\u54c7\u54c8\u54c8\u54c8\u54c8~\uff01\u5929\u7f51\u6613\u9003\uff0c\u60c5\u4e1d\u96be\u65ad\uff0c\u4f60\u6709\u901a\u5929\u7684\u672c\u4e8b\uff0c\u4e5f\u96be\u8fc7\u8fd9\u4e00\u5173\u3002", "\u4e3a\u5979\u9876\u7f6a\uff0c\u4ffa\u4ece\u4e0d\u540e\u6094\uff0c\u73b0\u5728\uff0c\u662f\u4ffa\u4e86\u65ad\u6069\u6028\u7684\u65f6\u5019\u4e86\uff01\uff01", "\u5176\u5b9e\uff0c\u6715\u4e0d\u662f\u6253\u4e0d\u8fc7\u4f60\uff0c\u6715\u53ea\u4e0d\u8fc7\u79c1\u632a\u4e86\u56fd\u5e93\uff0c\u4e70\u4e86\u4f60\u7684\u76d8\u53e3\u2026\u2026", "\u4f60\u7ec8\u4e8e\u6253\u5230\u8fd9\u91cc\u4e86\u3002", "\u4f60\u5c45\u7136\u5728\u8fd9\u91cc\uff1f", "\u54c8\u54c8\u54c8\u54c8\uff0c\u8001\u592b\u4e00\u8def\u4fdd\u4f60\uff0c\u5c31\u662f\u4e3a\u4e86\u8ba9\u4f60\u5e2e\u6211\u626b\u6e05\u5929\u5ead\uff0c\u4f60\u7684\u6240\u505a\u6240\u4e3a\u2026", "\u4ffa\u6700\u6068\u7684\u5c31\u662f\u88ab\u4eba\u6b3a\u9a97\uff01\u6211\u2026\u2026\uff08\u609f\u7a7a\u4e45\u4e45\u5730\u9677\u5165\u4e86\u56de\u5fc6\uff09", "\u5929\u5ead\uff0c\u5929\u5ead\u53c8\u600e\u6837\uff1f\u5973\u4eba\u9a97\u6211\uff0c\u5144\u5f1f\u9a97\u6211\uff0c\u5982\u4eca\u4ffa\u8001\u5b59\u6ca1\u6709\u4ec0\u4e48\u53ef\u4ee5\u7559\u604b\u7684\uff0c\u56de\u82b1\u679c\u5c71\u7f62\u4e86\u3002", "\u5b7d\u755c\uff0c\u5929\u5ead\u5a01\u4eea\uff0c\u5c82\u80fd\u5bb9\u4f60\u5168\u8eab\u800c\u9000\uff01\uff01", "\u59d1\u5a18\uff0c\u4f60\u600e\u4e48\u4f1a\u88ab\u5173\u5728\u8fd9\u91cc\uff01", "\u5974\u5bb6\u6697\u4e2d\u52a9\u4f60\uff0c\u89e6\u72af\u5929\u6761\u2026\u2026", "\u7389\u5e1d\u8001\u513f\uff0c\u5f85\u6211\u6253\u70c2\u4f60\u7684\u91d1\u51a0\uff01\uff01\u59d1\u5a18\u4f60\u5148\u79bb\u5f00\uff0c\u7b49\u4ffa\u56de\u6765\uff01", "\u5509\u2026\u2026\u4f60\u53c8\u4f55\u82e6\u2026\u2026", "\u8fd9\u91cc\u6709\u74f6\u706b\u773c\u91d1\u775b\u724c\u773c\u5f71\u971c\uff0c\u53bb\u76b1\u6297\u8870\u8001\uff0c\u53ef\u4ee5\u770b\u6e05\u695a\u654c\u4eba\u7684\u672c\u8d28\uff0c\u91d1\u8272\u8d28\u611f\u8d34\u5408\u80a4\u8d28\uff0c\u6765\u81ea\u5df4\u9ece\uff0c\u4f60\u503c\u5f97\u62e5\u6709\u3002", "\u73b0\u5728\u6d82\u597d\u4e86\uff0c\u770b\u8d77\u6765\u55f2\u4e0d\u55f2~\uff1f", "\u6069\u2026\u2026\u672c\u6765\u662f\u53ea\u201c\u7334\u5996\u201d\uff0c\u73b0\u5728\u662f\u4e2a\u201c\u4eba\u5996\u201d\u3002", "\u5ae6\u5a25\u59d1\u5a18\uff0c\u60f3\u4e0d\u5230\u5728\u8fd9\u91cc\u9047\u5230\u4f60\u3002", "\u5927\u5723\uff0c\u8fd9\u662f\u6211\u4eb2\u624b\u917f\u5236\u7684\u5343\u5e74\u6708\u6842\u9732\uff0c\u559d\u4e0b\u5b83\uff0c\u72b9\u5982\u8131\u80ce\u6362\u9aa8\uff0c\u4f53\u529b\u5927\u589e\u3002", "\u54e6~\uff1f\u96be\u9053\u8fd9\u662f\u5b9a\u60c5\u4fe1\u7269\uff1f", "\u800c\u4e14\uff0c\u5b83\u8fd8\u53ef\u4ee5\u4f7f\u4eba\u5fd8\u8bb0\u7ea2\u5c18\u611f\u60c5\uff0c\u6211\u5e0c\u671b\u4f60\u80fd\u5fd8\u8bb0\u6211\u3002", "\u554a~\u54c8~\u7ed9\u6211\u4e00\u676f\u5fd8\u60c5\u6c34~\u6362\u6211\u4e00\u591c\u4e0d\u6d41\u6cea\u2026\u2026\u59d1\u5a18\uff0c\u4ffa\u51c6\u5907\u79bb\u5f00\u5929\u5ead\uff0c\u6211\u5e0c\u671b\u4f60\u8ddf\u6211\u4e00\u8d77\u8d70\u2026\u2026", "\u8fdd\u80cc\u5929\u6761\uff0c\u79c1\u5954\uff0c\u4f1a\u88ab\u6574\u4e2a\u5929\u754c\u4eba\u8089\u641c\u7d22\u7684\u2026\u2026", "\u79c1\u5954\uff1f\u4ffa\u8001\u5b59\u4e0d\u505a\u90a3\u7325\u7410\u4e4b\u4e8b\uff0c\u5f85\u4ffa\u6253\u4e0a\u7075\u9704\u5b9d\u6bbf\uff0c\u8ba9\u7389\u7687\u5927\u5e1d\u4eb2\u53e3\u7b54\u5e94\uff0c\u6574\u4e2a\u5929\u5ead\u8c01\u6562\u4e3a\u96be\u4f60\uff01\uff01", "\u5927\u5723\u4fdd\u91cd\uff0c\u6b64\u5730\u5974\u5bb6\u4e0d\u5b9c\u4e45\u7559\uff0c\u5974\u5bb6\u4e0d\u60f3\u8fde\u7d2f\u4f60\u2026\u2026", "\u59d1\u5a18\uff01\u59d1\u5a18\uff01", "\u5927\u5723\uff0c\u524d\u9762\u51f6\u9669\u96be\u6d4b\uff0c\u5974\u5bb6\u8fd9\u91cc\u6709\u70b9\u79c1\u623f\u94b1\uff0c\u9001\u7ed9\u4f60\u4e70\u70b9\u4ed9\u4e39\u6ecb\u8865\u8eab\u4f53\u5427\u2026\u2026", "\u2026\u2026\u59d1\u5a18\u5bf9\u6211\u4e00\u7247\u771f\u60c5\uff0c\u4ffa\u53d1\u8a93\u8981\u4e3a\u4f60\u6253\u4e0b\u4e00\u7247\u5929", "\u8ddf\u4ffa\u8d70\u5427\uff0c\u56de\u82b1\u679c\u5c71\u53bb\u2026\u2026", "\u8868\uff0c\u59d1\u5976\u5976\u6211\u4e3a\u4e86\u5929\u5bab\u62a4\u7167\uff0c\u629b\u5f03\u4e86\u524d\u592b\uff0c\u6211\u624d\u8868\u518d\u8ddf\u4f60\u4e0b\u51e1\uff0c\u4f60\u2026\u2026\u662f\u4e2a\u597d\u4eba\u2026\u2026(\u98d8\u8d70)", "......\u5973\u4eba\u5982\u8863\u670d\uff0c\u5144\u5f1f\u5982\u624b\u8db3\uff0c\u8001\u725b~\uff01\u4ffa\u6765\u5bfb\u4f60\uff01\uff01", "\u5b9d\u6247\u5b9d\u6247\u544a\u8bc9\u6211\uff0c\u8c01\u662f\u8fd9\u4e2a\u4e16\u754c\u4e0a\u6700\u578b\u6700\u731b\u7684\u7537\u4eba\uff1f", "\uff08\u6a21\u4eff\u6247\u5b50\u7684\u58f0\u97f3\uff09\u662f\u4f60~\u662f\u4f60~\u8fd8\u662f\u4f60", "\u771f\u81ea\u604b\u2026\u2026", "\u54c7~\uff01\u88ab\u4f60\u5077\u7aa5\u5230\u4e86\uff0c\u672c\u5c0a\u8be5\u6740\u4f60\u706d\u53e3\uff0c\u4f46\u662f\u73b0\u5728\u4f60\u8fd8\u4e0d\u914d\u672c\u5c0a\u51fa\u624b\u3002", "\u53ef\u6076\uff0c\u7b49\u4ffa\u8001\u5b59\u5148\u627e\u56de\u4ffa\u90a3\u6839\u5982\u610f\u68cd\u5b50\u518d\u6765\u6536\u62fe\u4f60\u2026\u2026\\cFFCC00\u5148\u53bb2\u5c42\u5929\u7262\u6551\u8001\u725b\uff0c\u8ba9\u4ed6\u66ff\u4ffa\u5f00\u6697\u5899\u7ed5\u8fc7\u53bb", "\u8001\u725b\uff0c\u4ffa\u6551\u4f60\u6765\u4e86~\uff01", "\u5e73\u65f6\u8ba9\u4f60\u5e2e\u5fd9\uff0c\u8001\u662f\u63a8\u4e09\u963b\u56db\uff0c\u8fd9\u6b21\u8fd9\u4e48\u723d\u5feb\uff0c\u4e00\u5b9a\u6709\u95ee\u9898~", "\u563f\u563f\uff0c35\u5c42\u6709\u4e2a\u4e09\u773c\u5c0f\u767d\u8138\u592a\u6076\u5fc3\uff0c\u66ff\u4ffa\u706d\u4e86\u4ed6~~", "\u4ffa\u5bf9\u5c0f\u767d\u8138\u6728\u6709\u5174\u8da3\u2026\u2026", "\u90a3\u5c31\u60f3\u529e\u6cd5\u5e2e\u4ffa\u7ed5\u8fc7\u53bb~~", "\u563f\u563f\uff0c\u5f00\u81ea\u5df1\u7684\u6d1e\uff0c\u8ba9\u522b\u4eba\u8bf4\u53bb\u5427~~~", "?\u6709\u6839\u6346\u4ed9\u7ef3\uff1f\u4f3c\u4e4e\u53ef\u4ee5\u514b\u5236\u4f4f\u90a3\u4e2a\u4e09\u773c\u5c0f\u767d\u8138\uff0c\u6069\uff0c\u641e\u5b9a\u4ed6\uff0c\u6346\u7ed1\u4ed6\uff0c\u62ff\u4ed6\u7684\u82ad\u8549\u6247\uff0c\u54e6\u4e5f~", "\u6709\u82ad\u8549\u6247\u53ef\u4ee5\u706d\u706b\u4e86\uff0c\u5ae6\u5a25\u59d1\u5a18\uff0c\u4ffa\u6765\u5566~~\uff01\uff01\u5bf9\u4e86\uff0c\u8fd8\u6709\u6211\u7684\u5982\u610f\u91d1\u7b8d\u68d2\u3002", "\u6b22\u8fce\u4f60\u6765\u5230\u5929\u5bab\u4e16\u754c\uff0c\u6211\u662f\u4f60\u7684\u5e08\u5085\u83e9\u63d0\u8001\u7956\u3002", "\u5728\u8fd9\u91cc\u6211\u4e0d\u4f1a\u6559\u4f60\u4e03\u5341\u4e8c\u53d8\uff0c\u4f46\u662f\u6211\u4f1a\u6559\u4f60\u600e\u4e48\u6e38\u5386\u5929\u5bab\u3002", "\u4e3a\u5e08\u77e5\u9053\u4f60\u8981\u5927\u95f9\u5929\u5bab\uff0c\u7279\u610f\u5343\u91cc\u4f20\u97f3\uff0c\u63d0\u4f9b\u8fdc\u7a0b\u89c6\u9891\u652f\u6301\uff0c\u5f53\u7136\uff0c\u5982\u679c\u4f60\u5acc\u4e3a\u5e08\u7f57\u55e6\uff0c\u4e5f\u53ef\u4ee5\u5728\u6e38\u620f\u83dc\u5355\u4e2d\u9009\u62e9\u8df3\u8fc7\u6559\u7a0b\u3002", "\u597d\u4e86\uff0c\u73b0\u5728\u8bf7\u8bd5\u7740\\cFFCC00\u6309\u65b9\u5411\u952e\u79fb\u52a8\u5230\u8fd9\u91cc\u3002", "\u5f88\u597d\uff0c\u4f60\u5df2\u7ecf\u5b66\u4f1a\u592a\u7a7a\u6b65\u4e86\u3002", "\u5728\u4f60\u9762\u524d\u6709\u4e00\u9053\u9ec4\u8272\u7684\u95e8\uff0c\u4f60\u65e0\u6cd5\u8fc7\u53bb\u3002", "\u4f60\u53ef\u4ee5\u770b\u5230\u8fd9\u91cc\u6709\u628a\u9ec4\u94a5\u5319\uff0c\u5b83\u53ef\u4ee5\u5f00\u542f\u8fd9\u9053\u95e8\u3002", "\u73b0\u5728\\cFFCC00\u79fb\u52a8\u5230\u8fd9\u91cc\uff0c\u518d\u56de\u6765\u5f00\u95e8\u3002", "\u7b49\u7b49\uff01", "\u524d\u9762\u6709\u53ea\u6321\u8def\u7684\u72d7\u3002\u4f60\u9700\u8981\u6253\u8d25\u5b83\u624d\u80fd\u8d70\u8fc7\u53bb\u3002", "\u73b0\u5728\uff0c\\cFFCC00\u8bf7\u8bd5\u7740\u79fb\u52a8\u5230\u5b83\u7684\u4f4d\u7f6e\u4e0a\uff0c\u4e0e\u5b83\u6218\u6597\u5427\u3002", "\u542c\u5230\u8f70\u9686\u58f0\u4e86\u5427\uff0c\u56e0\u4e3a\u4f60\u6253\u8d25\u4e86\\cFFCC00\u5b88\u536b\u5c01\u5370\u95e8\u7684\u654c\u4eba\u3002", "\u6240\u4ee5\u8fd9\u91cc\u7684\\cFFCC00\u5c01\u5370\u95e8\\cF8F8F8\u5c31\u88ab\u6253\u5f00\u4e86\u3002", "\u5728\u6218\u6597\u4e2d\u4f60\u53ef\u80fd\u4f1a\u635f\u5931\u8840\u91cf\u3002", "\u8fd9\u91cc\u6709\u4e2a\\cFFCC00\u5c0f\u4ed9\u6843\uff0c\u53ef\u4ee5\u56de\u590d\u4f60\u7684\u8840\u91cf\u3002", "\u5982\u679c\u8840\u91cf\u4e0d\u8db3\uff0c\u4f60\u5c06\u65e0\u6cd5\u6311\u6218\u654c\u4eba\u3002", "\u53c8\u5230\u4e86\u5b66\u4e60\u65f6\u95f4\u3002", "\u4f60\u7684\u80fd\u529b\u662f\u53ef\u4ee5\u63d0\u5347\u7684\uff0c\u5305\u62ec\u653b\u51fb\u3001\u9632\u5fa1\u3001\u8840\u91cf\u3002", "\u8fd9\u91cc\u6709\u4e2a\u84dd\u8272\u4ed9\u4e39\uff0c\u5b66\u540d\u662f\u201c\u9632\u5fa1\u4ed9\u4e39\u201d\uff0c\u670d\u4e0b\u5b83\uff0c\u53ef\u4ee5\u63d0\u5347\u4f60\u7684\u9632\u5fa1\u529b\uff0c\u8ba9\u4f60\u6218\u6597\u66f4\u6301\u4e45\u3002", "\u8bb0\u4f4f\uff0c\\cFFCC00\u5929\u5ead\u5c42\u6570\u8d8a\u9ad8\uff0c\u4ed9\u4e39\u836f\u6548\u8d8a\u5927\u3002", "\u73b0\u5728\uff0c\u5403\u4e86\u5b83\uff0c\u6251\u8fc7\u53bb\u505a\u6389\u524d\u9762\u90a3\u6761\u72d7\uff0c\u4f60\u4f1a\u53d1\u73b0\u635f\u8840\u5c11\u4e86\u3002", "\u770b\u5230\u4e0a\u9762\u7684\u84dd\u95e8\u4e86\u5417\uff0c\u5b83\u53ea\u80fd\u7528\u84dd\u8272\u7684\u94a5\u5319\u6253\u5f00\u3002", "\u5b83\u88ab\u85cf\u5728\u8fd9\u91cc\uff0c\\cFFCC00\u5148\u62ff\u5230\u5b83\u5427\u3002", "\u4f60\u53d1\u73b0\u4e86\u4e00\u9053\u7ea2\u95e8\u3002\u8fd9\u79cd\u95e8\u5f88\u5c11\u89c1\uff0c\u5fc5\u987b\u7528\u7ea2\u94a5\u5319\u624d\u80fd\u6253\u5f00\u3002", "\u5b83\u88ab\u85cf\u5728\u8fd9\u91cc\uff0c\\cFFCC00\u8bf7\u5148\u5f97\u5230\u5b83\uff0c\u518d\u56de\u6765\u5f00\u95e8\u3002", "\u4f60\u627e\u5230\u4e86\u4e00\u628a\u7ea2\u94a5\u5319\uff0c\u8fd9\u79cd\u94a5\u5319\u6bd4\u8f83\u7a00\u5c11\u3002", "\u8bd5\u7740\\cFFCC00\u7528\u5b83\u5f00\u542f\u8fd9\u91cc\u7684\u7ea2\u95e8\u3002", "\u5f88\u597d\uff0c\u8fd9\u5c42\u5df2\u7ecf\u63a5\u8fd1\u5c3d\u5934\u3002", "\u4f60\u4f1a\u53d1\u73b0\u8fd9\u6837\u7684\u7ea2\u8272\u4f20\u9001\u70b9\uff0c\u5b83\u53ef\u4ee5\u8ba9\u4f60\u5411\u4e0a\u4e00\u5c42\u697c\u3002", "\u4e0d\u8fc7\uff0c\u522b\u6025\u7740\u79bb\u5f00\u3002", "\u4f60\u662f\u4e0d\u662f\u5df2\u7ecf\u53d1\u73b0\u8fd9\u91cc\u6709\u4e2a\u9053\u5177\u4e86\u5417\uff1f", "\u8fd9\u91cc\u6709\u4e2a\u7ea2\u8272\u4ed9\u4e39\uff0c\u5b66\u540d\u662f\u201c\u653b\u51fb\u4ed9\u4e39\u201d\uff0c\u670d\u4e0b\u5b83\uff0c\u53ef\u4ee5\u63d0\u5347\u4f60\u7684\u653b\u51fb\u529b\uff0c\u8ba9\u4f60\u6218\u6597\u66f4\u72c2\u91ce\u3002", "\u4f46\u662f\u8fd9\u91cc\u597d\u8c61\u4e0d\u901a\u2026\u2026", "\u522b\u6025\uff01\u4fd7\u8bdd\u8bf4\u8f66\u5230\u5c71\u524d\u5fc5\u6709\u8def\uff0c\u5728\u5929\u5bab\u7684\u5f88\u591a\u5c42\u4e2d\u4f1a\u6709\u9690\u85cf\u7684\u8def\uff0c\u66f4\u591a\u60ca\u559c\u66f4\u591a\u6b22\u7b11\uff0c\u5c31\u5728\u9690\u85cf\u8def\u2026\u2026", "\u73b0\u5728\uff0c\u79fb\u52a8\u5230\u8fd9\u91cc\uff0c\u4f60\u5c31\u4f1a\u53d1\u73b0\u5b83\u3002", "\u8981\u8bb0\u4f4f\uff0c\\cFFCC00\u5f88\u591a\u5c42\u91cc\u90fd\u4f1a\u6709\u9690\u85cf\u7684\u4e1c\u897f\uff0c\u8bd5\u7740\u53bb\u63a2\u7d22\u5427\u3002", "\u770b\u5230\u4f60\u4e0a\u6765\u7684\u8def\u4e86\u5417\uff1f", "\u84dd\u8272\u7684\u4f20\u9001\u70b9\u53ef\u4ee5\u8ba9\u4f60\u5411\u4e0b\u4e00\u5c42\u697c\u3002", "\u4f60\u53ef\u80fd\u65e0\u6cd5\u51fb\u8d25\u8fd9\u4e2a\u654c\u4eba\uff0c\u7ed5\u9053\u4e5f\u662f\u524d\u8fdb\u7684\u529e\u6cd5\u3002", "\u90a3\u4e48\uff0c\u5982\u4f55\u5224\u65ad\u4e00\u4e2a\u654c\u4eba\u7684\u5f3a\u5f31\u5462\uff1f", "\u6e38\u620f\u4e2d\u4f60\u4f1a\u83b7\u5f97\u8fd9\u4ef6\u5b9d\u7269\uff0c\u5b83\u53eb\\cFFCC00\u706b\u773c\u91d1\u775b\u724c\u773c\u5f71\u818f\u3002", "\u6d82\u62b9\u4e00\u70b9\u5728\u773c\u76ae\u4e0a\uff0c\u4f60\u53ef\u4ee5\u770b\u7834\u654c\u60c5\uff0c\u8fd8\u53ef\u4ee5\u53bb\u9664\u773c\u89d2\u7eb9\u3002", "\u6211\u5e2e\u4f60\u5f00\u51fa\u4e86\u4e00\u6761\u8def\uff0c\u4f60\u53ef\u4ee5\u53bb\u53d6\u5b83\u4e86\u3002", "\u73b0\u5728\u4f60\u53ef\u4ee5\u53c2\u7167\u4f7f\u7528\u8bf4\u660e\u6765\u4f7f\u7528\u5b83\u4e86\u3002", "\u9664\u4e86\u63095/OK\u952e\u67e5\u770b\u654c\u4eba\u5bf9\u4f60\u9020\u6210\u7684\u4f24\u5bb3\u4ee5\u5916\u3002", "\u4f60\u8fd8\u53ef\u4ee5\u6309\u5de6\u8f6f\u952e\u6253\u5f00\u7269\u54c1\u680f\u3002", "\u9009\u62e9\u8be5\u7269\u54c1\uff0c\u6309\u786e\u8ba4\u952e\u67e5\u770b\u66f4\u8be6\u7ec6\u7684\u654c\u4eba\u4fe1\u606f\u3002", "\u4f60\u7ad9\u5728\u8fd9\u5341\u5b57\u8857\u5934\u4e0a\uff0c\u627e\u4e0d\u5230\u6765\u53bb\u7684\u65b9\u5411\u3002", "\u4e0d\u8981\u614c\u5f20\uff0c\u8bd5\u7740\u67e5\u770b\u4e0b\u8fd9\u91cc\u9053\u5177\u548c\u654c\u4eba\u7684\u5206\u5e03\u5f62\u52bf\u3002", "\u4e0a\u53bb\u7684\u4f20\u9001\u70b9\u5728\u8fd9\u91cc\u3002", "\u5982\u679c\u4f60\u65e0\u6cd5\u9a6c\u4e0a\u51fb\u8d25\u8fd9\u4e2a\u5b88\u536b\u3002", "\u5c31\u8bd5\u7740\u5c06\u5730\u56fe\u4e0a\u7684\u4ed9\u4e39\u548c\u4ed9\u6843\u5403\u6389\uff0c\u7136\u540e\u4f60\u5c31\u53ef\u4ee5\u6218\u80dc\u5b83\u4e86\u3002", "\u8bb0\u4f4f\uff0c\u5982\u679c\u524d\u65b9\u6709\u4e00\u7fa4\u654c\u4eba\u5728\u5411\u4f60\u6325\u624b\uff0c\u5343\u4e07\u522b\u51b2\u52a8\u3002", "\u7262\u8bb0\\cFFCC00\u201c\u5148\u5403\u4ed9\u4e39\u540e\u8089\u640f\u201d\\cF8F8F8\u662f\u51cf\u5c11\u635f\u8840\u7684\u7b2c\u4e00\u6cd5\u5219\u3002", "\u54e6\u563f\u563f~\u4e3a\u5e08\u8981\u7ee7\u7eed\u4eab\u53d7\u6e21\u5047\u5566~\u3002", "\u5728\u8fd9\u4e4b\u524d\u6211\u4f1a\u4f20\u6388\u4f60\u4e94\u767e\u5e74\u529f\u529b\uff0c\u518d\u9001\u4f60\u4e24\u4ef6\u4e1c\u897f\u9632\u8eab\u3002", "\u8fd9\u662f\u4e00\u628a\u6b66\u5668\uff0c\u80fd\u8ba9\u4f60\u63d0\u5347\u5f88\u9ad8\u7684\u653b\u51fb\u3002", "\u5728\u6e38\u620f\u7684\\cFFCC00\u6bcf10\u5c42\u90fd\u6709\u4e00\u628a\u65b0\u6b66\u5668\u3002", "\u5982\u679c\u4f60\u80fd\u65e9\u70b9\u83b7\u5f97\u5b83\uff0c\u5c31\u80fd\u8f7b\u677e\u5e94\u5bf9\u654c\u4eba\uff0c\u8d70\u5f97\u66f4\u8fdc\u3002", "\u540c\u6837\uff0c\u8fd9\u662f\u4e00\u4ef6\u9632\u5177\uff0c\u80fd\u63d0\u9ad8\u4f60\u7684\u9632\u5fa1\u3002", "\u6211\u73b0\u5728\u5e2e\u4f60\u6253\u5f00\u8fd9\u9053\u5899\uff0c\u5728\u4e00\u822c\u60c5\u51b5\u4e0b\uff0c\u5b83\u662f\u65e0\u6cd5\u51fb\u788e\u7684\u3002", "\u5bf9\u4e86\uff0c\u7ed9\u4f60\u4ecb\u7ecd\u4e00\u4e2a\u5929\u5bab\u4e0a\u7684\u670b\u53cb\u3002", "\u8fd9\u662f\u4e3a\u5e08\u7684\u8001\u670b\u53cb\uff0c\\cFFCC00\u592a\u767d\u91d1\u661f\\cF8F8F8\uff0c\u4ed6\u4f1a\u6697\u4e2d\u5e2e\u52a9\u4f60\u7684\u3002", "\u606d\u559c\uff0c\u4f60\u5df2\u7ecf\u6bd5\u4e1a\u4e86\uff0c\u6211\u518d\u4f20\u6388\u4f60\u4e94\u767e\u5e74\u7684\u529f\u529b\u3002", "\u8fd8\u7ed9\u4f60\u51c6\u5907\u4e86\u4e24\u4ef6\u795e\u5668\uff0c\u628a\u5b83\u4eec\u6536\u4e0b\u5427\u3002", "\u8981\u8bb0\u4f4f\uff0c\u5f80\u524d\u4f60\u5c06\u9762\u5bf9\u7684\u4e0d\u662f\u4e00\u4e2a\u654c\u4eba\uff0c\u800c\u662f\u6574\u4e2a\u5929\u5bab\u3002", "\u634f\u54c8\u54c8~\u4eca\u5929\u98ce\u548c\u65e5\u4e3d\uff0c\u6715\u5fc3\u60c5\u5f88\u597d~\uff01", "\u5440~\uff01\u54ea\u91cc\u94bb\u51fa\u4e00\u53ea\u679c\u5b50\u72f8\uff01\u9884\u9632\u975e\u5178\uff01\u5de6\u53f3\u4e0e\u6211\u62ff\u4e0b\uff01", "\u4ec0\u4e48\u7834\u7687\u5e1d\uff0c\u4e94\u8c37\u4e0d\u5206\uff0c\u516d\u755c\u4e0d\u8fa8\u2026\u2026\u7389\u5e1d\u8001\u513f\uff0c\u4ffa\u4e43\u82b1\u679c\u5c71\u7b2c\u4e00\u5c4a\u578b\u79c0\u51a0\u519b\uff0c\u7f8e\u7334\u738b\u5b59\u609f\u7a7a\uff01\uff01", "\u90fd\u7ed9\u6211\u4e0a~\uff01", "\u5de8\u73b2\u795e\uff0c\u6123\u5728\u90a3\u91cc\u505a\u4ec0\u4e48\uff1f", "\u965b\u4e0b\uff0c\u4eba\u5bb6\u662f\u5973\u5b69\u5b50\u561b\uff0c\u6700\u6015\u6bdb\u8338\u8338\u5730\u5c0f\u52a8\u7269\u4e86\u2026\u2026", "\u5c11\u5e9f\u8bdd\uff01\u60f3\u88ab\u780d\u5934\u554a\uff01", "\u965b\u4e0b\uff0c\u4e09\u592a\u5b50\u8bf7\u6218\uff01", "\u7389\u5e1d\u8001\u513f\uff0c\u4ffa\u8001\u5b59\u8981\u505a\u9f50\u5929\u5927\u5723\uff01\uff01", "\u55f7~~\u5b83\u8fc7\u6765\u4e86\u5b83\u8fc7\u6765\u4e86\uff0c\u8bf7\u4f60\u4e2a\u5934\u554a\uff0c\u8d76\u7d27\u9876\u4e0a\u5148~~", "\u62a4\u9a7e~\uff01\u62a4\u9a7e~\uff01", "\u8c01~\uff01\u662f\u5929\u5ead\u7b2c\u4e00\u578b\u7537~\uff01\uff01", "\u662f\u4f60~\uff01\u662f\u4f60~\uff01", "\u8c01~\uff01\u662f\u5929\u5ead\u7b2c\u4e00\u731b\u7537~\uff01\uff01", "\u662f\u4f60~\uff01\u662f\u4f60~\uff01", "\u8c01~\uff01\u662f\u5929\u5ead\u7b2c\u4e00\u660e\u661f\u6218\u795e~\uff01\uff01", "\u4f60\u662f\u7535\uff0c\u4f60\u662f\u5149\uff0c\u4f60\u662f\u552f\u4e00\u5730\u795e\u8bdd\uff0c\u4f60\u4e3b\u5bb0\uff0c\u6211\u5d07\u62dc~\u6ca1\u6709\u66f4\u597d\u7684\u529e\u6cd5~~\uff01", "\u965b\u4e0b\u2026\u2026\u5455\u2026\u2026", "\u81e3\u89c9\u5f97\u5427\u2026\u2026\u5455\u2026\u2026", "\u8fd8\u662f\u4e0d\u8981\u8ba9\u4e8c\u90ce\u795e\u7ee7\u7eed\u4e0b\u53bb\u4e86\uff0c\u5c3d\u5feb\u5e73\u606f\u8fd9\u6b21\u4e8b\u4ef6\uff0c\u7ed9\u5b59\u609f\u7a7a\u5c01\u4e2a\u5b98\u7b97\u4e86\uff01\uff01", "\u5455~~\u6715\u4e5f\u662f\u8fd9\u4e48\u60f3\u6ef4~~\u5594\u83b1\uff0c\u5c31\u5c01\u5b59\u609f\u7a7a\u4e3a\u9f50\u5929\u5927\u5723\uff0c\u638c\u7ba1\u87e0\u6843\u56ed\uff01", "\u563f\u563f\uff0c\u4ffa\u8001\u5b59\u5c31\u9886\u4e86~\uff01\u591a\u8c22~\uff01", "\u5996\u2026\u5996\u602a\uff0c\u4ed6\u7684\u773c\u775b\u95ea\u7740\u7ea2\u5149\u2026\u2026\u592a\u53ef\u6015\u4e86", "\u5c45\u7136\u9000\u7f29\uff0c\u4f60\u8fd9\u6ca1\u7528\u7684\u4e1c\u897f\u3002", "\u5662~~~~~~\u6ee1\u56ed\u4ed9\u6843\u6210\u719f\uff0c\u715e\u662f\u8bf1\u4eba\uff01\uff01", "\u8ba9\u4ffa\u8001\u5b59\u56db\u5904\u901b\u901b~~", "\u609f\u7a7a~\u609f\u7a7a~\uff01", "\u4e3a\u4ec0\u4e48\u6709\u53ea\u9171\u6cb9\u86e4\u87c6\u8ddf\u6211\u5343\u91cc\u4f20\u97f3\uff1f", "\u662f\u5e08\u7236\u6211\u554a\uff01", "\u5e08\u7236\uff1f\u4e3a\u4ec0\u4e48\u5316\u4e2a\u86e4\u87c6\u5986\uff1f", "\u4e3a\u5e08\u5728\u5370\u5ea6\uff0c\u6cd5\u672f\u4ea4\u6d41\u517c\u6e21\u5047\uff0c\u65e5\u5149\u6d74\u52a0\u987f\u987f\u5496\u55b1\u996d\uff0c\u5634\u5df4\u4e0a\u706b\u3002", "\u5982\u6765\u628a\u5b59\u609f\u7a7a\u5c01\u5370\u4e8e\u4e94\u6307\u5c71\u4e0b\uff0c\u4f34\u968f\u7740\u6240\u6709\u6069\u6028\u60c5\u4ec7\uff0c\u6b32\u77e5\u540e\u4e8b\u5982\u4f55\uff0c\u656c\u8bf7\u671f\u5f85\u300a\u897f\u6e38\u8bb0\u4e8c\u4e4b\u5927\u5723\u53d6\u7ecf\u300b", "\u7231\u5f92\u554a\uff0c\u4f60\u4e00\u4e2a\u4eba\u8981\u7ee7\u7eed\u6311\u6218\u5929\u5bab\uff0c\u4e3a\u5e08\u4e0d\u653e\u5fc3\uff0c\u7ed9\u4f60\u4e70\u4e86\u4efd\u4fdd\u9669\u3002", "\u5e08\u5085\u591f\u4e49\u6c14\u3002", "\uff08\u63a5\u8fc7\u4fdd\u5355\uff09\u53d7\u76ca\u4eba...\u201c\u83e9\u63d0\u8001\u7956\u201d", "\u6211\u7684\u5f92\u513f\u554a\uff0c\u4f60\u5df2\u7ecf\u9677\u5165\u6df7\u6c8c\u4e16\u754c\u3002\u4e5f\u5c31\u4eba\u4eec\u5e38\u8bf4\u7684\u7cbe\u795e\u5206\u88c2\u75c7\u3002", "\u8fd9\u662f\u7531\u4e8e\u79cd\u79cd\u611f\u60c5\u7ea0\u845b\u5f15\u53d1\u7684\uff0c\u5982\u679c\u4f60\u60f3\u51fa\u6765\uff0c\u5c31\u8981\u6218\u80dc\u4ed6\u4eec\u3002", "\u4e0d\uff0c\u662f\u6218\u80dc\u81ea\u5df1\u3002\u8981\u8ba9\u8fd9\u5929\u2026\u518d\u4e5f\u906e\u4e0d\u4f4f\u4f60\u7684\u773c\u3002", "\u6211\u8981\u63d0\u9192\u4f60\u7684\u662f\uff0c\u8fd9\u4e2a\u4e16\u754c\u91cc\uff0c\u6240\u6709\u7684\u654c\u4eba\u90fd\u4f1a\u6bd4\u539f\u6765\u66f4\u5f3a\uff0c\u5f53\u7136\u4f60\u7684\u80fd\u529b\u4e5f\u4f1a\u63d0\u5347\u66f4\u591a\u3002", "\u53bb\u5427\u2026\u2026\u52ab\u96be\u5728\u6240\u96be\u514d\u3002"};
        this.bj = this.n.length;
        this.g = new boolean[this.bj];
        this.bo = 0;
        this.e = new byte[][]{{29, 1, 3, 0, 29, 2, 3, 0, 29, 3, 3, 0, 26, 1, 4, 0, 26, 2, 4, 0, 26, 3, 4, 0, 30, 9, 3, 0, 30, 10, 3, 0, 30, 11, 3, 0, 32, 9, 4, 0, 32, 10, 4, 0, 32, 11, 4, 0}, {29, 4, 4, 0, 29, 4, 5, 0, 29, 5, 4, 0, 26, 7, 4, 0, 26, 8, 4, 0, 26, 8, 5, 0, 30, 4, 7, 0, 30, 4, 8, 0, 30, 5, 8, 0, 32, 8, 8, 0, 32, 7, 8, 0, 32, 8, 7, 0}, {27, 6, 5, 0, 27, 5, 6, 0, 27, 7, 6, 0, 27, 6, 7, 0}, {17, 6, 4, 0}, {16, 5, 5, 0}, {24, 6, 2, 0, 32, 5, 1, 0, 32, 6, 1, 0, 32, 7, 1, 0, 27, 6, 6, 0}, {32, 2, 3, 0, 32, 3, 3, 0, 32, 4, 3, 0, 26, 8, 3, 0, 26, 9, 3, 0, 26, 10, 3, 0, 29, 3, 5, 0, 29, 4, 5, 0, 29, 5, 5, 0, 30, 7, 5, 0, 30, 8, 5, 0, 30, 9, 5, 0}, {32, 4, 4, 0, 32, 5, 4, 0, 32, 6, 4, 0}};
        this.x = new byte[]{75, 74, 68, 67, 69, 71};
        this.p = false;
        this.a = null;
        this.a = null;
        this.a = null;
        this.a = null;
        this.a = null;
        this.B = null;
        this.f = new byte[56][];
        this.f = new short[64];
        this.g = new short[64];
        this.h = new short[64];
        this.i = new short[64];
        this.E = new byte[30];
        this.E = new int[30];
        this.j = new short[30];
        this.k = new short[30];
        this.F = new byte[30];
        this.i = new boolean[30];
        this.G = new byte[]{10, 6, -14, -26, -36, -38, -39, -41, -43};
        this.H = new byte[]{16, 4, -10, 0, 14, 12, 14, 11, 13};
        this.I = new byte[]{9, 22, 34, 41, 51, 52, 54, 55, 56};
        this.J = new byte[]{15, -2, -19, -1, 21, 17, 21, 19, 21};
        this.K = new byte[]{9, -1, -10, -22, -26, -27, -29, -30, -32};
        this.L = new byte[]{20, 1, -15, 0, 24, 20, 24, 20, 24};
        this.r = true;
        String[] stringArray = new String[]{"\u754c\u9762", "\u83dc\u5355", "\u5730\u56fe", "\u80cc\u666f", "\u4eba\u7269", "\u7ec4\u4ef6", "\u8868\u60c5", "\u6548\u679c", "\u811a\u672c", "\u7f13\u5b58", "\u697c\u5c42", "\u89d2\u8272", "\u8bbe\u5b9a", "\u654c\u4eba", "\u5f15\u5b50", "\u7ed3\u5c40", "\u8f7d\u5165\u8fdb\u5ea6"};
        this.bN = 0;
        this.bO = 0;
        this.bP = 0;
        int[] nArray5 = new int[]{0xEEEEEE, 0xCCCCCC, 0xAAAAAA, 0x888888, 0x666666, 0x444444, 0x222222, 0x111111};
        this.G = new int[]{0xFF0000, 0xFF9000, 16776194, 1244928, 65478, 26367, 14156031};
        this.u = false;
        this.v = false;
        this.w = false;
        this.A = 0;
        this.H = new int[]{1, 30, 60, 80, 100};
        this.R = new byte[]{29, 31, 39, 15, 0, 31, 29, 10, 0, 0, 37, 31, 36, 0, 32, 29};
        this.S = new byte[]{1, 2, 0, 3, 4, 1, 2, 0};
        this.I = new int[]{4, 2, 2, 2, 2, 2, 2};
        this.J = new int[this.I.length];
        this.p = new String[]{"logostart.mid", "logoquit.mid", "menu.mid", "game.mid"};
        this.bZ = 60;
        this.D = true;
        this.C = (byte)-1;
        this.K = new int[]{4202520, -1};
        int[] nArray6 = new int[]{2555941, 6436695};
        int[] nArray7 = new int[]{2555941, 10126750};
        int[] nArray8 = new int[]{12342908, 15039118};
        int[] nArray9 = new int[]{0x222222, 0xFFDF7D};
        this.T = new byte[]{87, 18, 9};
        this.L = new int[3];
        this.U = new byte[]{0, 5, 5, 5, 10, 4, 14, 7, 21, 4, 25, 3, 28, 3, 31, 4, 35, 3, 38, 4, 42, 4, 46, 4, 50, 4, 54, 3, 57, 3, 60, 4, 64, 2, 66, 2, 68, 4, 72, 4, 76, 5};
        this.d = new int[26][7];
        this.q = new String[]{"\u624b\u673a\u53f7", "\u5bc6\u7801", "\u8bd5\u73a9", "\u6ce8\u518c", "\u5e2e\u52a9", "\u6ce8\u518c\u5e10\u53f7", "\u6ce8\u518c\u5bc6\u7801", "\u6ce8\u518c\u624b\u673a\u53f7\u5fc5\u987b\u4e3a\u672c\u673a\uff0c\u9a8c\u8bc1\u6210\u529f\u540e\u5c06\u83b7\u5f97200\u6e38\u620f\u5e01", "\u6b22\u8fce\u767b\u9646\u96ea\u9ca4\u9c7c\u5e73\u53f0", "\u786e\u5b9a", "\u53d6\u6d88", "\u9000\u51fa", "\u8054\u7f51\u4e2d", "\u6b63\u5728\u91cd\u8bd5", "\u8054\u7f51\u8d85\u65f6", "\u670d\u52a1\u5668\u6ca1\u6709\u54cd\u5e94", "\u767b\u9646", "\u5269\u4f59\u70b9\u6570", "\u67e5\u8be2\u8bb0\u5f55", "\u5145\u503c", "\u4e2a\u4eba\u4fe1\u606f", "\u5e10\u53f7\u6216\u5bc6\u7801\u8f93\u5165\u6709\u8bef", "\u8bf7\u8f93\u516511\u4f4d\u624b\u673a\u53f7", "\u8bf7\u8f93\u51656-10\u4f4d\u5bc6\u7801", "\u65e5\u671f", "\u91d1\u989d", "\u672c\u6708\u4ed8\u8d39\u8bb0\u5f55", "\u8fdb\u5165\u6e38\u620f", "\u8bf7\u786e\u8ba4\u60a8\u586b\u5199\u7684\u662f\u5f53\u524d\u624b\u673a\u53f7\uff0c\u5426\u5219\u4f1a\u5bfc\u81f4\u6ce8\u518c\u5931\u8d25\u3002\u8d44\u8d392\u5143\uff0c\u9700\u8981\u53d1\u90011\u6761\u77ed\u4fe1\uff0c2\u5143/\u6761\uff0c\u4e0d\u542b\u901a\u4fe1\u8d39\u3002\u662f\u5426\u6ce8\u518c\uff1f", "\u5df2\u53d1\u9001\u6210\u529f\uff0c\u8bf7\u7b49\u5f85\u7cfb\u7edf\u9a8c\u8bc1\u540e\u624d\u80fd\u767b\u9646", "\u53d1\u9001\u5931\u8d25\uff0c\u8bf7\u91cd\u8bd5", "\u7c7b\u578b", "\u5e8f\u5217\u53f7", "\u5bc6\u7801", "\u91d1\u989d", "\u5143", "\u5df2\u63d0\u4ea4\u5145\u503c\u4fe1\u606f\uff0c\u8bf7\u5728\u5145\u503c\u8bb0\u5f55\u4e2d\u67e5\u770b\u5145\u503c\u7ed3\u679c\u3002", "\u8fd4\u56de", "\u72b6\u6001", "\u6700\u8fd1\u5145\u503c\u8bb0\u5f55", "\u63d0\u4ea4\u8fc7\u7a0b\u51fa\u73b0\u9519\u8bef\uff0c\u8bf7\u68c0\u67e5\u5145\u503c\u5185\u5bb9\u3002", "\uff1a", "\n", "\u662f\u5426\u786e\u5b9a\u9000\u51fa\u6e38\u620f\uff1f", "\u60a8\u5c1a\u672a\u767b\u9646\uff0c\u8bf7\u8f93\u5165\u60a8\u7684\u624b\u673a\u53f7\u548c\u5bc6\u7801\u8054\u7f51\u4ed8\u8d39", "\u5e8f\u5217\u53f7\u548c\u5bc6\u7801\u586b\u5199\u6709\u8bef", "\u77ed\u4fe1\u5145\u503c", "\u5df2\u53d1\u9001\u6210\u529f\u3002", "\u670d\u52a1\u5668\u54cd\u5e94\u9519\u8bef\uff01", "\u4f7f\u7528\u672c\u4ea7\u54c1\u5fc5\u987b\u5148\u767b\u5f55\uff0c\u4e0e\u96ea\u9ca4\u9c7c\u5176\u4ed6\u4ea7\u54c1\u4e2d\u6ce8\u518c\u7684\u624b\u673a\u53f7\uff0c\u5bc6\u7801\uff0c\u5e10\u53f7\u4e2d\u7684\u6e38\u620f\u5e01\u901a\u7528\u3002\u5982\u679c\u6ca1\u6709\u5e10\u53f7\u53ef\u4ee5\u6ce8\u518c\uff0c\u9001\u6e38\u620f\u5e01\u3002\u767b\u5f55\u540e\u4ed8\u8d39\u4fe1\u606f\u66f4\u5b89\u5168\uff0c\u907f\u514d\u6389\u5b58\u6863\u3002", "\u5145\u503c\u7684\u6e38\u620f\u5e01\u5728\u6240\u6709\u96ea\u9ca4\u9c7c\u76f8\u5173\u4ea7\u54c1\u4e2d\u901a\u7528\u3002", "\u8bf7\u8f93\u516511\u4f4d\u624b\u673a\u53f7", "\u8bf7\u8f93\u51656\u4f4d\u4ee5\u4e0a\u5bc6\u7801", "\u662f\u5426\u9a6c\u4e0a\u6ce8\u518c\uff1f\u9700\u8981\u786e\u8ba4\u60a8\u586b\u5199\u7684\u662f\u5f53\u524d\u624b\u673a\u53f7\uff0c\u5426\u5219\u4f1a\u5bfc\u81f4\u6ce8\u518c\u5931\u8d25\u3002\u8d44\u8d392\u5143\uff0c\u9700\u8981\u53d1\u90011\u6761\u77ed\u4fe1\uff0c2\u5143/\u6761\uff0c\u4e0d\u542b\u901a\u8baf\u8d39\u3002", "\u6ce8\u518c\u5931\u8d25\uff01\u5982\u679c\u8be5\u624b\u673a\u53f7\u662f\u60a8\u7684\u771f\u5b9e\u53f7\u7801\u3002\u5c06\u4f1a\u628a\u60a8\u7684\u5bc6\u7801\u6539\u4e3a\u5f53\u524d\u6ce8\u518c\u586b\u5199\u7684\u5bc6\u7801\uff0c\u8be5\u8fc7\u7a0b\u53ef\u80fd\u9700\u8981\u4e00\u6bb5\u65f6\u95f4\u3002", "\u5df2\u6ce8\u518c\u6210\u529f\u3002\u5f85\u9a8c\u8bc1\u6210\u529f\u540e\u5c31\u80fd\u7528\u8be5\u53f7\u7801\u5145\u503c\uff01\u5e76\u8d2d\u4e70\u6e38\u620f\u4e2d\u7684\u4ed8\u8d39\u5185\u5bb9\u3002\u5982\u679c\u518d\u6b21\u6ce8\u518c\u8fd8\u53ef\u4ee5\u53d1\u9001\u77ed\u4fe1\u4fee\u6539\u5bc6\u7801\u3002", "\u60a8\u5c1a\u672a\u9a8c\u8bc1\uff0c\u5982\u679c\u4e4b\u524d\u5df2\u53d1\u9001\u77ed\u4fe1\u9a8c\u8bc1\uff0c\u8bf7\u7a0d\u5019\u518d\u767b\u9646\u91cd\u8bd5\u3002", "\u5ba2\u670d\u7535\u8bdd\uff1a400 630 5518", "\u60a8\u7684\u624b\u673a\u65e0\u6cd5\u8fde\u63a5\u5230\u670d\u52a1\u5668\uff0c\u4e0d\u80fd\u8fdb\u884c\u6e38\u620f\u3002\u662f\u5426\u91cd\u65b0\u5c1d\u8bd5\u8054\u7f51\uff1f"};
        this.r = new String[]{"http://218.202.228.126:8880/nolander/server.php", "http://chatsrv0.ttutt.cn/dntk/server.php"};
        this.cd = 2048;
        this.V = new byte[this.cd];
        this.F = false;
        this.a = null;
        this.H = true;
        this.setFullScreenMode(true);
        a a2 = this;
        if (a2.a == null) {
            a2.a = new Random();
            a2.a.setSeed(System.currentTimeMillis());
        }
        this.a = true;
        this.a = 0;
        this.a();
    }

    protected final void paint(Graphics object) {
        int n;
        int n2;
        int n3;
        int n4;
        int n5;
        this.a = DirectUtils.getDirectGraphics((Graphics)object);
        this.a = object;
        this.a.setFont(this.a);
        switch (this.a) {
            case 0: {
                if (this.e < 2) {
                    this.a.setColor(-1);
                    this.a.fillRect(0, 0, 240, 320);
                    if (this.a == null) break;
                    this.a.drawImage(this.a, 240 - this.a.getWidth() >> 1, 320 - this.a.getHeight() >> 1, 0);
                    break;
                }
                this.H();
                break;
            }
            case 1: {
                object = this;
                n5 = 0;
                n5 = 320 - ((a)object).q - ((a)object).a[10][0].getHeight();
                if (n5 > 0) {
                    ((a)object).a.setColor(6662375);
                    ((a)object).a.fillRect(0, 0, 240, n5);
                }
                ((a)object).a.drawImage(((a)object).a[10][0], 0, n5, 0);
                ((a)object).a.setColor(3156024);
                ((a)object).a.fillRect(0, 320 - ((a)object).q, 240, ((a)object).q);
                super.b(((a)object).a[10][1], 86, 320 - ((a)object).q + (((a)object).q - 17 >> 1), 0, 17 * ((a)object).a[((a)object).k], 60, 17, 17);
                n5 = 320 - ((a)object).q + (((a)object).q - 13 >> 1);
                super.a(((a)object).a[8][14], 60 - ((a)object).U, n5, 1);
                ((a)object).a.drawImage(((a)object).a[8][14], 160 + ((a)object).U, n5, 0);
                this.a.setClip(0, this.q, 240, 320 - (this.q >> 1));
                this.F();
                this.a.setClip(0, 0, 240, 320);
                break;
            }
            case 2: {
                object = this;
                ((a)object).a.setColor(3156024);
                ((a)object).a.fillRect(0, 0, 240, 320);
                ((a)object).bR = 148 * ((a)object).bN / 100;
                super.a(((a)object).a[15][0], 108, 86, 0, 0, 24, ((a)object).bR);
                super.a(((a)object).a[15][1], 108, 86 + ((a)object).bR, 0, ((a)object).bR, 24, 148 - ((a)object).bR);
                break;
            }
            case 11: {
                int n6;
                this.a(true);
                this.h(0, 20);
                this.a(this.ae, this.af + 20, true);
                this.D();
                this.d(0, 0);
                object = this;
                int n7 = 0;
                n4 = (((a)object).I << 1) + 12;
                int n8 = 320 - n4;
                if (((a)object).c == null || ((a)object).f || ((a)object).q == 4) {
                    super.e(0, ((a)object).W);
                    break;
                }
                Object var9_28 = null;
                super.b(0, n8, 80, n4);
                int n9 = n4;
                int n10 = 160;
                n3 = n8;
                n7 = 80;
                Object object2 = object;
                n7 = n3;
                for (n6 = 80; n6 < 214; n6 += 16) {
                    ((a)object2).a.setClip(n6, n7, 16, 16);
                    ((a)object2).a.drawImage(((a)object2).a[8][0], n6 - 26, n7, 0);
                }
                ((a)object2).a.setClip(0, 0, 240, 320);
                super.a(((a)object2).a[8][0], 214, n7, 0, 0, 26, 16, 1);
                ((a)object2).a.setColor(2699825);
                ((a)object2).a.fillRect(80, n7 += 16, 149, n9 - 16);
                while (n7 < n3 + n9) {
                    ((a)object2).a.setClip(229, n7, 11, 16);
                    ((a)object2).a.drawImage(((a)object2).a[8][0], 176, n7, 0);
                    n7 += 16;
                }
                ((a)object2).a.setClip(0, 0, 240, 320);
                if (((a)object).r > 0) {
                    Image image = ((a)object).a[((a)object).r];
                    if (((a)object).r != 72) {
                        n4 = image.getHeight();
                        super.a(image, 0 + (80 - ((a)object).h[((a)object).r] >> 1), 320 - n4, 0, 0, ((a)object).h[((a)object).r], n4);
                    } else {
                        super.a(image, 24, 275, 0, 0, 32, 45);
                    }
                } else if (((a)object).r == 0) {
                    ((a)object).a.drawImage(((a)object).a[3][1], 40 - (((a)object).a[3][1].getWidth() >> 1), 320 - ((a)object).a[3][1].getHeight(), 0);
                } else {
                    ((a)object).a.setColor(-1);
                    ((a)object).a.drawString("?", 40, n8 + (n4 - ((a)object).b >> 1), 17);
                }
                ((a)object).a.setColor(10473684);
                super.a(((a)object).c, 85, n8 + 12, 129, n4 - 12, false);
                if (((a)object).A > 0) {
                    super.a(((a)object).a[8][15], 220, 320 - ((a)object).I - 10, 0, 0, 7, 9);
                }
                if (((a)object).B < ((a)object).C) {
                    super.a(((a)object).a[8][15], 220, 320 - ((a)object).I + 1, 7, 0, 7, 9);
                }
                if ((((a)object).q == 6 || ((a)object).q == 1) && (n4 = ((a)object).w[((a)object).bi]) >= 0) {
                    if (((a)object).bp >= 0) {
                        n7 = ((a)object).g[((a)object).bp] + 16;
                        n8 = ((a)object).h[((a)object).bp] - 30;
                        if (((a)object).r == 84) {
                            n8 -= 20;
                        }
                    } else {
                        n7 = ((a)object).N + 16;
                        n8 = ((a)object).O - 30;
                    }
                    ((a)object).a.drawImage(((a)object).a[12][n4], n7 + ((a)object).ae, n8 + ((a)object).af + 20, 0);
                }
                if (((a)object).q != 6) break;
                ((a)object).a.drawImage(((a)object).a[2][11], ((a)object).ae + 5 + (((a)object).bk << 5), ((a)object).af + (((a)object).bl << 5) - 12 + ((a)object).U, 0);
                break;
            }
            case 21: {
                object = this;
                ((a)object).a.setColor(0);
                ((a)object).a.fillRect(0, 0, 240, 320);
                ((a)object).a.setColor(-1);
                ((a)object).a.drawString("\u662f\u5426\u5f00\u542f\u58f0\u97f3\uff1f", 120, 160, 17);
                super.m();
                break;
            }
            case 3: {
                this.a(true);
                this.h(0, 20);
                this.a(this.ae, this.af + 20, true);
                n4 = 20;
                int n11 = 240 - (this.Y + 1 << 2);
                object = this;
                int n12 = 0;
                int n13 = 0;
                if (((a)object).f[1] && ((a)object).d != null) {
                    ((a)object).a.drawImage(((a)object).d, n11, 20, 0);
                    n12 = n11 + (((a)object).P << 2);
                    n13 = 20 + (((a)object).Q << 2);
                    ((a)object).a.setColor(1112072);
                    ((a)object).a.fillRect(n12, n13, 4, 4);
                }
                this.e(0, this.W);
                this.d(0, 0);
                object = this;
                if (((a)object).n) {
                    if (((a)object).as >= 0) {
                        super.a(((a)object).a[8][19], 104, 20, 0, 0, 16, 16);
                        super.a(((a)object).a[8][19], 120, 20 + ((a)object).U, 16, 0, 15, 17);
                    }
                    if (((a)object).av >= 0) {
                        super.a(((a)object).a[8][19], 104, ((a)object).W - 17, 0, 17, 16, 16);
                        super.a(((a)object).a[8][19], 120, ((a)object).W - 17 - ((a)object).U, 16, 17, 15, 17);
                    }
                }
                this.D();
                this.n = 1;
                this.o = (byte)3;
                this.m();
                break;
            }
            case 4: 
            case 19: {
                int n6;
                int n9;
                this.a(true);
                this.e(0, this.W);
                this.d(0, 0);
                n4 = 160;
                int n14 = 120;
                object = this;
                int n15 = ((a)object).i + 22;
                int n16 = ((a)object).o + 32;
                int n17 = 120 - (((a)object).i + 22 >> 1);
                n2 = 160 - (((a)object).o + 32 >> 1);
                n = ((a)object).p - ((a)object).b >> 1;
                boolean bl = false;
                super.c(n17, n2, n15, n16);
                n2 += 16 + n;
                int n18 = ((a)object).l;
                while (n18 < ((a)object).m) {
                    byte by = ((a)object).a[n18];
                    if (((a)object).k == n18) {
                        ((a)object).a.setColor(0);
                        ((a)object).a.fillRect(120 - (((a)object).i >> 1), n2 - n, ((a)object).i, ((a)object).p);
                        ((a)object).a.setColor(16377897);
                    } else if (by != 11) {
                        ((a)object).a.setColor(7574946);
                    } else {
                        ((a)object).a.setColor(0xFF99FF);
                    }
                    if (by > 11 && by < 17) {
                        int n19 = 4;
                        n6 = n2;
                        n9 = 120;
                        int[] nArray = ((a)object).G;
                        String string = ((a)object).c[by];
                        Object object3 = object;
                        n4 = string.length();
                        int n20 = ((a)object3).z;
                        int n21 = nArray.length;
                        char c = '\u0000';
                        n9 -= ((a)object3).a.stringWidth(string) >> 1;
                        for (int i = 0; i < n4; ++i) {
                            c = string.charAt(i);
                            ((a)object3).a.setColor(nArray[n20]);
                            ((a)object3).a.drawChar(c, n9, n6, 0);
                            n9 += ((a)object3).a.charWidth(c);
                            if (++n20 < n21) continue;
                            n20 = 0;
                        }
                        if ((((a)object3).d & 3) == 0 && (((a)object3).z = (byte)(((a)object3).z + 1)) >= n21) {
                            ((a)object3).z = 0;
                        }
                    } else {
                        ((a)object).a.drawString(((a)object).c[by], 120, n2, 17);
                    }
                    ++n18;
                    n2 += ((a)object).p;
                }
                n3 = 120 + (n15 >> 1) - 23;
                n2 = 160 + (n16 >> 1) - 38;
                if (((a)object).l > 0) {
                    super.a(((a)object).a[8][15], n3, n2, 0, 0, 7, 9);
                }
                if (((a)object).m < ((a)object).h) {
                    super.a(((a)object).a[8][15], n3, n2 + 9, 7, 0, 7, 9);
                }
                this.m();
                break;
            }
            case 5: {
                this.a(false);
                object = this;
                int n22 = 240 - ((a)object).be >> 1;
                n4 = 320 - ((a)object).bd >> 1;
                Image image = null;
                byte by = 0;
                n3 = 0;
                n2 = 0;
                n = 0;
                if (((a)object).bg - ((a)object).bf > 0) {
                    super.c(n22, n4, ((a)object).be, ((a)object).bd);
                    if (((a)object).bf > 0) {
                        super.a(((a)object).a[8][19], 112, n4 - 17 - (((a)object).d & 1), 16, 0, 15, 17);
                    }
                    n4 += 16;
                    int n23 = ((a)object).bf;
                    int n24 = n22 += 16;
                    while (n23 < ((a)object).bg) {
                        by = ((a)object).u[n23];
                        image = ((a)object).a[((a)object).u[n23]];
                        n3 = ((a)object).w[n23];
                        ((a)object).a.setColor(13097429);
                        ((a)object).a.drawString(((a)object).m[by], n22, n4 + 2, 0);
                        n22 = (240 + ((a)object).be >> 1) - 75;
                        n = ((a)object).b - 19 >> 1;
                        n4 += n;
                        if (n3 < 0) {
                            ((a)object).a.drawImage(((a)object).a[8][17], n22 - 4, n4 + 2, 0);
                        } else {
                            ((a)object).a.drawImage(((a)object).a[8][16], n22 - 12, n4 + 2, 0);
                            super.d((n22 += 15) - 1, n4 + 6, 48, 12);
                            super.a(((a)object).a[8][2], n3, n22 + 45, n4 + 8);
                        }
                        n22 = n24;
                        super.d(n22, n4 += n + 19 + 4, 32, 32);
                        if (by > 40) {
                            n2 = 32 - ((a)object).h[by] >> 1;
                            n = image.getHeight();
                            if (n > 31) {
                                n = 31;
                            }
                            if (n2 < 0) {
                                super.a(image, n22, n4 + 31 - n, -n2, 0, 32, n);
                            } else {
                                super.a(image, n22 + n2, n4 + 31 - n, 0, 0, ((a)object).h[by], n);
                            }
                        } else {
                            ((a)object).a.setClip(n22, n4, 32, 32);
                            ((a)object).a.drawImage(image, n22 + (32 - image.getWidth() >> 1), n4 + (32 - image.getHeight() >> 1), 0);
                            ((a)object).a.setClip(n22, n4, 240, 320);
                        }
                        super.a(((a)object).a[8][7], n22 += 36, n4 += 4, 10, 0, 10, 13);
                        super.d(n22 += 15, n4, 48, 12);
                        super.a(((a)object).a[8][2], ((a)object).t[by - 41], n22 + 40, n4 + 2);
                        super.a(((a)object).a[8][7], n22 += 52, n4, 20, 2, 10, 10);
                        ((a)object).a.setColor(512);
                        super.d(n22 += 15, n4, 48, 12);
                        super.a(((a)object).a[8][2], ((a)object).u[by - 41], n22 + 40, n4 + 2);
                        super.a(((a)object).a[8][7], n22 -= 82, (n4 += 12) + 5, 0, 2, 10, 10);
                        super.d(n22 += 15, n4 + 5, 48, 12);
                        super.a(((a)object).a[8][2], ((a)object).v[by - 41], n22 + 40, n4 + 7);
                        ((a)object).a.drawImage(((a)object).a[8][1], n22 += 52, n4 + 5, 0);
                        super.d(n22 += 15, n4 + 5, 48, 12);
                        super.a(((a)object).a[8][2], ((a)object).s[by - 41], n22 += 40, n4 + 7);
                        n4 += 18;
                        if (++n23 < ((a)object).bg) {
                            n22 = n24 - 5;
                            ((a)object).a.setColor(6435);
                            ((a)object).a.drawLine(n22, n4, n22 + ((a)object).be - 22, n4);
                            ((a)object).a.setColor(4803902);
                            ((a)object).a.drawLine(n22, ++n4, n22 + ((a)object).be - 22, n4);
                        }
                        n4 += 2;
                        n22 = n24;
                    }
                    n4 += 16;
                    if (((a)object).bg < ((a)object).bb) {
                        super.a(((a)object).a[8][19], 112, n4 + (((a)object).d & 1), 16, 17, 15, 17);
                    }
                }
                this.m();
                break;
            }
            case 7: {
                this.a(false);
                this.b(true);
                this.m();
                break;
            }
            case 8: {
                this.a(false);
                this.b(false);
                this.m();
                break;
            }
            case 9: {
                this.a(true);
                this.h(0, 20);
                this.a(this.ae, this.af + 20, true);
                this.e(0, this.W);
                this.d(0, 0);
                this.m();
                object = this;
                int n25 = 240 - ((a)object).ax - 22 >> 1;
                n4 = 320 - ((a)object).aw >> 1;
                int n26 = (((a)object).a << 1) + 16 + 48;
                super.a(0, n25, n4, ((a)object).ax + 22, ((a)object).aw);
                n25 = 240 - n26 >> 1;
                ((a)object).a.setColor(7575203);
                ((a)object).a.drawString("\u82b1\u8d39", n25, n4 += 21, 0);
                ((a)object).a.drawImage(((a)object).a[8][1], n25 += (((a)object).a << 1) + 4, n4 + (((a)object).b - 10 >> 1), 0);
                super.a(((a)object).a[8][2], ((a)object).ay, n25 += 60, n4 + 3 + (((a)object).b - 10 >> 1));
                n25 = 240 - ((a)object).ax >> 1;
                ((a)object).a.setColor(549016);
                ((a)object).a.fillRect(n25, (n4 += ((a)object).b + 5) + (((a)object).aC << 4), ((a)object).ax, 16);
                ((a)object).a.drawImage(((a)object).a[8][14], n25 + 10, n4 + 2 + (((a)object).aC << 4), 0);
                super.a(((a)object).a[8][14], n25 + ((a)object).ax - 30, n4 + 2 + (((a)object).aC << 4), 1);
                super.a(((a)object).a[8][7], 97, n4 + 4, 0, 2, 10, 10);
                super.a(((a)object).a[8][2], ((a)object).az, 142, n4 + 6);
                super.a(((a)object).a[8][7], 97, (n4 += 16) + 4, 11, 0, 8, 13);
                super.a(((a)object).a[8][2], ((a)object).aA, 142, n4 + 6);
                super.a(((a)object).a[8][7], 97, (n4 += 16) + 4, 20, 2, 10, 11);
                super.a(((a)object).a[8][2], ((a)object).aB, 142, n4 + 6);
                break;
            }
            case 10: {
                int n27 = 320;
                int n28 = 240;
                n4 = 0;
                int n29 = 0;
                object = this;
                for (n3 = 319; n3 >= 1; n3 -= 2) {
                    ((a)object).a.drawLine(-1, -1 + (320 - n3), -1 + n3 - 1, 318);
                    ((a)object).a.drawLine(239 - n3, -1, 239, -1 + n3);
                }
                object = this;
                n29 = 240 - ((a)object).aZ >> 1;
                n4 = 320 - ((a)object).aY >> 1;
                n28 = 0;
                Image image = null;
                super.c(n29, n4, ((a)object).aZ, ((a)object).aY);
                n29 += 20;
                n4 += 16;
                ((a)object).a.setColor(13097429);
                if (((a)object).ba < ((a)object).aR) {
                    ((a)object).a.drawString(((a)object).m[((a)object).r[((a)object).ba]], n29, n4 + 2, 0);
                }
                ((a)object).a.setColor(6178);
                ((a)object).a.drawLine(n29 -= 9, n4 += ((a)object).b + 4, n29 + ((a)object).aZ - 23, n4);
                ((a)object).a.setColor(3564144);
                ((a)object).a.drawLine(n29, ++n4, n29 + ((a)object).aZ - 23, n4);
                n4 += 10;
                n28 = n29 += 9;
                n3 = ((a)object).aU;
                n2 = ((a)object).aT * ((a)object).aU;
                while (n3 < ((a)object).aV) {
                    n = 0;
                    while (n < ((a)object).aT) {
                        super.d(n29, n4, 32, 32);
                        if (n2 == ((a)object).ba) {
                            super.a(((a)object).a[2][10], n29 - 4, n4 - 3, 38 * (((a)object).d & 1), 0, 38, 38);
                        }
                        if (n2 < ((a)object).aR) {
                            image = ((a)object).a[((a)object).r[n2]];
                            ((a)object).a.drawImage(image, n29 + (32 - image.getWidth() >> 1), n4 + (32 - image.getHeight() >> 1), 0);
                            if (((a)object).s[n2] > 0) {
                                super.a(((a)object).a[8][2], (int)((a)object).s[n2], n29 + 32, n4 + 26);
                            }
                        }
                        ++n;
                        n29 += 40;
                        ++n2;
                    }
                    ++n3;
                    n29 = n28;
                    n4 += 40;
                }
                n29 = (240 + ((a)object).aZ >> 1) - 22;
                n4 -= ((a)object).aY - 32 - ((a)object).b - 4 >> 1;
                if (((a)object).aU > 0) {
                    super.a(((a)object).a[8][15], n29, n4 - 15, 0, 0, 7, 9);
                }
                if (((a)object).aV < ((a)object).aX) {
                    super.a(((a)object).a[8][15], n29, n4 + 5, 7, 0, 7, 9);
                }
                this.m();
                break;
            }
            case 13: {
                this.a(false);
                break;
            }
            case 14: {
                object = this;
                int n30 = 0;
                if (((a)object).b != null) {
                    ((a)object).r = ((a)object).b.getHeight();
                    ((a)object).s = 320 - ((a)object).r - (((a)object).b + 4) * 3 >> 1;
                    ((a)object).a.setColor(0);
                    ((a)object).a.fillRect(0, 0, 240, ((a)object).s);
                    n30 = ((a)object).s + ((a)object).b.getHeight();
                    ((a)object).a.fillRect(0, n30, 240, 320 - n30);
                    ((a)object).a.drawImage(((a)object).b, 120, ((a)object).s, 17);
                    if (!((a)object).d) {
                        ((a)object).a.setColor(-1);
                    } else {
                        ((a)object).a.setColor(((a)object).c[((a)object).e]);
                    }
                    if (((a)object).t < 3 && ((a)object).b != null) {
                        super.a(((a)object).b, 10, ((a)object).s + ((a)object).r + ((a)object).b, 220, ((a)object).I << 1, false);
                    }
                }
                if (((a)object).t > 2) {
                    ((a)object).a.setColor(-1);
                    ((a)object).a.drawString("\u8bf7\u6309\u4efb\u610f\u952e", 120, 320 - ((a)object).b - 2, 17);
                } else {
                    ((a)object).a.setColor(-1);
                    ((a)object).a.drawString("\u8df3\u8fc7", 240 - ((a)object).a.stringWidth("\u8df3\u8fc7") - 5, 320 - ((a)object).b - 2, 0);
                }
                this.a.setClip(0, 0, 240, 320);
                this.F();
                this.a.setClip(0, 0, 240, 320);
                break;
            }
            case 18: {
                object = this;
                ((a)object).a.setColor(0);
                ((a)object).v = 0;
                ((a)object).h = new short[64];
                ((a)object).i = new short[64];
                int n31 = 0;
                n4 = 0;
                short s = 0;
                int n32 = ((a)object).u;
                while (--n32 >= 0) {
                    n4 = ((a)object).f[n32];
                    s = ((a)object).g[n32];
                    ((a)object).a.fillRect(n4, (int)s, 4, 4);
                    n31 = super.g(15);
                    if ((n31 & 1) != 0) {
                        super.m(n4, s - 4);
                    }
                    if ((n31 & 2) != 0) {
                        super.m(n4, s + 4);
                    }
                    if ((n31 & 4) != 0) {
                        super.m(n4 - 4, s);
                    }
                    if ((n31 & 8) == 0) continue;
                    super.m(n4 + 4, s);
                }
                ((a)object).u = ((a)object).v;
                ((a)object).f = ((a)object).h;
                ((a)object).g = ((a)object).i;
                break;
            }
            case 12: {
                this.a(false);
                object = this;
                int n33 = 320 - ((a)object).b - 4;
                super.a(((a)object).e, 120, n33 + 2, 17, ((a)object).K);
                n33 += ((a)object).b + 4 - 13 >> 1;
                if (((a)object).aM > 0) {
                    super.a(((a)object).a[8][14], 60 - ((a)object).U, n33, 1);
                }
                if (((a)object).aM < ((a)object).aL - 1) {
                    ((a)object).a.drawImage(((a)object).a[8][14], 160 + ((a)object).U, n33, 0);
                }
                this.m();
                break;
            }
            case 15: {
                this.a(false);
                this.b(this.a[10][1], 86, 10, 0, 51, 68, 17, 17);
                this.m();
                break;
            }
            case 17: {
                this.a(false);
                this.b(this.a[10][1], 86, 10, 0, 68, 68, 17, 17);
                this.m();
                break;
            }
            case 16: {
                this.a(false);
                object = this;
                int n34 = 240 - ((a)object).aG - 22 >> 1;
                n4 = 320 - ((a)object).aH >> 1;
                String string = null;
                super.b(((a)object).a[10][1], 86, 10, 0, 34, 68, 17, 17);
                super.c(n34, n4, ((a)object).aG + 22, ((a)object).aH);
                int n35 = (((a)object).b + 8) * ((a)object).aE;
                ((a)object).a.setColor(549016);
                ((a)object).a.fillRect(n34 += 11, (n4 += 16) + n35, ((a)object).aG, ((a)object).b + 8);
                ((a)object).a.drawImage(((a)object).a[8][14], n34 + 10, n4 + 2 + n35 + (((a)object).b - 5 >> 1), 0);
                super.a(((a)object).a[8][14], n34 + ((a)object).aG - 30, n4 + 2 + n35 + (((a)object).b - 5 >> 1), 1);
                n3 = 0;
                while (n3 < ((a)object).aF) {
                    string = ((a)object).f[((a)object).l[n3]];
                    n34 = 240 - ((a)object).a.stringWidth(string) - 16 >> 1;
                    ((a)object).a.setColor(-1);
                    ((a)object).a.drawString(string, n34, n4 + 4, 0);
                    n34 += 2 + ((a)object).a.stringWidth(string);
                    if (!((a)object).f[n3]) {
                        super.a(((a)object).a[8][11], n34, n4 + (((a)object).b - 2 >> 1), 12, 0, 12, 10);
                    } else {
                        super.a(((a)object).a[8][11], n34, n4 + (((a)object).b - 2 >> 1), 0, 0, 12, 10);
                    }
                    ++n3;
                    n4 += ((a)object).b + 8;
                }
                this.m();
                break;
            }
            case 20: {
                object = this;
                int n36 = 320 - ((a)object).a[14][0].getHeight() >> 1;
                ((a)object).x += 2;
                if (!((a)object).e && (((a)object).d & 1) != 0 && ++((a)object).w > ((a)object).a[14][0].getWidth() - 240) {
                    ((a)object).e = true;
                }
                if ((((a)object).d & 3) >> 1 != 0 && ((a)object).H < ((a)object).o[260].length()) {
                    ++((a)object).H;
                    super.a(((a)object).o[260], 209, (((a)object).I << 1) + 12, ((a)object).H);
                    ((a)object).A = ((a)object).C - 1;
                    if (((a)object).A < 0) {
                        ((a)object).A = 0;
                    }
                    ((a)object).B = ((a)object).C;
                }
                ((a)object).a.setColor(3156024);
                ((a)object).a.fillRect(0, 0, 240, n36);
                ((a)object).a.fillRect(0, 320 - n36, 240, n36);
                ((a)object).a.setClip(0, 0, 240, 320);
                ((a)object).a.drawImage(((a)object).a[14][0], -((a)object).w, n36, 0);
                ((a)object).a.setColor(-1);
                super.a(((a)object).o[260], 15, 320 - ((a)object).b - 10, 210, ((a)object).b + 10, false);
                break;
            }
            case 22: {
                this.H();
                break;
            }
            case 99: {
                object = this;
                int n37 = ((a)object).cj & 7;
                n4 = 20;
                n3 = 0;
                ((a)object).a.setColor(0xF8F8F8);
                ((a)object).a.fillRect(18, 138, 204, 44);
                ((a)object).a.setColor(0);
                ((a)object).a.drawRect(20, 140, 199, 39);
                ((a)object).a.drawRect(18, 138, 203, 43);
                if (!((a)object).G) {
                    if (n37 == 7) {
                        ((a)object).G = true;
                    }
                } else {
                    if (n37 == 7) {
                        ((a)object).G = false;
                    }
                    n37 = 7 - n37;
                }
                n2 = 0;
                while (n2 < 8) {
                    n3 = !((a)object).G ? n37 - n2 : n2 - n37;
                    switch (n3) {
                        case 0: {
                            ((a)object).a.setColor(44527);
                            break;
                        }
                        case 1: {
                            ((a)object).a.setColor(0x66CCFF);
                            break;
                        }
                        case 2: {
                            ((a)object).a.setColor(11593215);
                            break;
                        }
                        default: {
                            ((a)object).a.setColor(0xD9D9D9);
                        }
                    }
                    ((a)object).a.fillRect(n4 + 2, 142, 21, 6);
                    ((a)object).a.fillRect(215 - n4 + 2, 172, 21, 6);
                    ++n2;
                    n4 += 25;
                }
                ((a)object).a.setColor(44527);
                ((a)object).a.drawString(((a)object).h, 120, 320 - ((a)object).b >> 1, 17);
            }
        }
        if (this.q) {
            object = this;
            int n38 = 0;
            int n39 = 0;
            n3 = 0;
            if (((a)object).r) {
                if (++((a)object).bM > 4) {
                    ((a)object).bM = 4;
                    ((a)object).r = false;
                    ((a)object).am = ((a)object).x;
                    super.s(((a)object).am);
                    if (((a)object).am == 0) {
                        super.a(1, 2);
                        super.i((((a)object).ag - 32 >> 1) - ((a)object).N, (((a)object).ah - 32 >> 1) - ((a)object).O);
                    } else if (((a)object).am == 50) {
                        super.a(6, 7);
                        super.i((((a)object).ag - 32 >> 1) - ((a)object).N, (((a)object).ah - 32 >> 1) - ((a)object).O);
                    } else if (((a)object).am == 1 && !((a)object).s) {
                        super.a(6, 11);
                        super.i((((a)object).ag - 32 >> 1) - ((a)object).N, (((a)object).ah - 32 >> 1) - ((a)object).O);
                    } else {
                        n2 = super.a(((a)object).s);
                        if (n2 >= 0) {
                            super.b(((a)object).g[n2] >> 5, ((a)object).h[n2] >> 5);
                        }
                    }
                    super.r();
                    super.k();
                }
            } else if (--((a)object).bM <= 0) {
                ((a)object).r = true;
                ((a)object).q = false;
            }
            n39 = n38 = 4 - ((a)object).bM;
            n3 = ((a)object).bM << 1;
            ((a)object).a.setColor(0);
            n2 = 0;
            while (n2 < 40) {
                n = 0;
                while (n < 30) {
                    ((a)object).a.fillRect(n38, n39, n3, n3);
                    ++n;
                    n38 += 8;
                }
                ++n2;
                n38 = 4 - ((a)object).bM;
                n39 += 8;
            }
            return;
        }
        if (this.f) {
            n4 = 320 - this.z >> 1;
            n5 = 240 - this.y >> 1;
            object = this;
            int n40 = n5;
            int n41 = n4;
            super.c(n5, n4, ((a)object).y, ((a)object).z);
            switch (((a)object).i) {
                case 0: 
                case 2: 
                case 4: 
                case 5: {
                    ((a)object).a.setColor(-1);
                    super.a(((a)object).c, n40 += 16, n41 += 16, 180, 240, true);
                    break;
                }
                case 1: 
                case 3: {
                    super.d(n40 += 16, n41 += 16, 32, 32);
                    Image image = ((a)object).a[((a)object).h];
                    if (image != null) {
                        ((a)object).a.drawImage(image, n40 + (32 - image.getWidth() >> 1), n41 + (32 - image.getHeight() >> 1), 0);
                    }
                    ((a)object).a.setColor(16770173);
                    ((a)object).a.drawString(((a)object).m[((a)object).h], n40 += 42, n41 + (32 - ((a)object).b >> 1), 0);
                    ((a)object).a.setColor(-1);
                    n40 = n5 + 16;
                    super.a(((a)object).c, n40, n41 += 36, 180, ((a)object).z - 32 - 36, true);
                }
            }
            n41 = n4 + ((a)object).z - 12 - 18;
            if (((a)object).f != 0) {
                n40 = n5 + 13;
                super.a(((a)object).a[8][11], n40, n41 + 6, (((a)object).f - 1) * 12, 0, 12, 10);
            }
            if (((a)object).g != 0) {
                n40 = n5 + ((a)object).y - 25;
                super.a(((a)object).a[8][11], n40, n41 + 6, (((a)object).g - 1) * 12, 0, 12, 10);
            }
        }
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     * Unable to fully structure code
     */
    public final void run() {
        if (this.F) {
            var5_1 = this;
            while ((var6_4 = var5_1.e()) == 1) {
                Thread.yield();
            }
            var5_1.g = null;
            var1_6 = var6_4;
            var2_8 = this.a;
            synchronized (var2_8) {
                this.cf = var1_6;
                return;
            }
        }
        var1_7 = 0L;
        var3_10 = 0L;
        var5_2 = 0L;
        try {
            while (this.a) {
                block274: {
                    block273: {
                        var1_7 = System.currentTimeMillis();
                        var3_10 = var1_7 + (long)this.c;
                        var5_3 = this;
                        if (!var5_3.f) break block273;
                        block4 : switch (var5_3.i) {
                            case 0: {
                                if (var5_3.a == 15 || var5_3.a == 17) {
                                    var6_5 = var5_3;
                                    switch (var6_5.f) {
                                        case -1: {
                                            super.a(var6_5.A - var6_5.E);
                                            break;
                                        }
                                        case -2: {
                                            super.a(var6_5.A + var6_5.E);
                                            break;
                                        }
                                        case -7: {
                                            var6_5.f = false;
                                            var6_5.a = var6_5.b;
                                        }
                                    }
                                    break;
                                }
                                super.g();
                                break;
                            }
                            case 4: {
                                switch (var5_3.f) {
                                    case -6: 
                                    case -5: {
                                        var5_3.f = super.a(true);
                                        break block4;
                                    }
                                    case -7: {
                                        var5_3.f = super.a(false);
                                        break block4;
                                    }
                                }
                                super.g();
                                break;
                            }
                            case 1: {
                                if (var5_3.g == 0 && var5_3.f == 0) {
                                    super.g();
                                    break;
                                }
                                block21 : switch (var5_3.g) {
                                    case 2: 
                                    case 3: {
                                        switch (var5_3.f) {
                                            case 0: {
                                                break block21;
                                            }
                                            case -2: 
                                            case -1: {
                                                super.g();
                                                break block21;
                                            }
                                        }
                                        var5_3.f = false;
                                    }
                                }
                                break;
                            }
                            case 3: {
                                switch (var5_3.f) {
                                    case -6: {
                                        var5_3.f = false;
                                        super.l(var5_3.ba);
                                        break block4;
                                    }
                                    case -7: {
                                        var5_3.a = (byte)10;
                                        var5_3.f = false;
                                        break block4;
                                    }
                                }
                                super.g();
                                break;
                            }
                            case 2: {
                                break;
                            }
                            case 5: {
                                var6_5 = var5_3;
                                switch (var6_5.f) {
                                    case 0: {
                                        break block4;
                                    }
                                    case -1: {
                                        super.a(var6_5.A - 1);
                                        break block4;
                                    }
                                    case -2: {
                                        super.a(var6_5.A + 1);
                                        break block4;
                                    }
                                    case -3: {
                                        super.i(var6_5.aM - 1);
                                        break block4;
                                    }
                                    case -4: {
                                        super.i(var6_5.aM + 1);
                                        break block4;
                                    }
                                    case -7: 
                                    case -6: 
                                    case -5: {
                                        var6_5.f = false;
                                        var6_5.a = (byte)10;
                                    }
                                }
                            }
                        }
                        var5_3.g = 0;
                        var5_3.f = 0;
                        break block274;
                    }
                    switch (var5_3.a) {
                        case 0: {
                            var5_3.c = 100;
                            if (var5_3.e < 2) {
                                if (var5_3.ca == 0) {
                                    try {
                                        var5_3.a = Image.createImage((String)("/l" + var5_3.e + ".png"));
                                    }
                                    catch (Exception v0) {}
                                    if (var5_3.a == null) {
                                        var5_3.ca = 0;
                                        ++var5_3.e;
                                        break;
                                    }
                                    var5_3.ca = 1;
                                    break;
                                }
                                if (++var5_3.ca <= 15) break;
                                ++var5_3.e;
                                var5_3.ca = 0;
                                break;
                            }
                            if (++var5_3.ca <= 35) {
                                super.n(0, var5_3.ca);
                                break;
                            }
                            super.v(-1);
                            var5_3.E = false;
                            var5_3.ca = 0;
                            var5_3.a = null;
                            var5_3.a[0] = null;
                            super.G();
                            var5_3.c = 75;
                            var5_3.a = (byte)21;
                            super.a();
                            break;
                        }
                        case 21: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -6: {
                                    var6_5.D = true;
                                    if (var6_5.bZ == 0) {
                                        var6_5.bZ = 60;
                                    }
                                    var6_5.a = 1;
                                    super.a();
                                    break;
                                }
                                case -7: {
                                    var6_5.D = false;
                                    var6_5.bZ = 0;
                                    var6_5.a = 1;
                                    super.a();
                                }
                            }
                            break;
                        }
                        case 1: {
                            if ((var5_3.d & 3) == 0) {
                                super.e(super.g(240), 320 - var5_3.q - super.g(150), 2, -1);
                            }
                            super.o();
                            var6_5 = var5_3;
                            block66 : switch (var6_5.f) {
                                case -3: 
                                case -1: 
                                case 50: 
                                case 52: {
                                    if (var6_5.k > 0) {
                                        --var6_5.k;
                                        break;
                                    }
                                    var6_5.k = var6_5.h - 1;
                                    break;
                                }
                                case -4: 
                                case -2: 
                                case 54: 
                                case 56: {
                                    if (var6_5.k < var6_5.h - 1) {
                                        ++var6_5.k;
                                        break;
                                    }
                                    var6_5.k = 0;
                                    break;
                                }
                                case -5: 
                                case 53: {
                                    var7_11 = var6_5;
                                    switch (var7_11.a[var7_11.k]) {
                                        case 0: {
                                            var7_11.a[10][0] = null;
                                            var7_11.a = (byte)14;
                                            super.a();
                                            break block66;
                                        }
                                        case 1: {
                                            var7_11.a = (byte)8;
                                            var7_11.b = 1;
                                            super.a();
                                            break block66;
                                        }
                                        case 2: {
                                            var7_11.a = (byte)16;
                                            var7_11.b = 1;
                                            super.a();
                                            break block66;
                                        }
                                        case 3: {
                                            var7_11.a = (byte)15;
                                            var7_11.b = 1;
                                            super.a();
                                            break block66;
                                        }
                                        case 4: {
                                            var7_11.a = (byte)17;
                                            var7_11.b = 1;
                                            super.a();
                                            break block66;
                                        }
                                        case 5: {
                                            var7_11.a = (byte)22;
                                            super.a();
                                        }
                                    }
                                }
                            }
                            break;
                        }
                        case 2: {
                            var6_5 = var5_3;
                            if (var6_5.bN < 100) {
                                if (var6_5.bN < var6_5.bQ) {
                                    var6_5.bN += 4;
                                    break;
                                }
                                var8_17 = var6_5.P[var6_5.bP];
                                var7_12 = var6_5;
                                switch (var8_17) {
                                    case 0: {
                                        super.a(8);
                                        break;
                                    }
                                    case 1: {
                                        super.a(10);
                                        break;
                                    }
                                    case 2: {
                                        super.a(2);
                                        break;
                                    }
                                    case 3: {
                                        super.a(1);
                                        break;
                                    }
                                    case 4: {
                                        super.a(3);
                                        break;
                                    }
                                    case 5: {
                                        super.a(6);
                                        super.a(4);
                                        super.a(5);
                                        break;
                                    }
                                    case 13: {
                                        super.a(7);
                                        super.a(13);
                                        super.n();
                                        break;
                                    }
                                    case 6: {
                                        super.a(12);
                                        break;
                                    }
                                    case 7: {
                                        super.E();
                                        break;
                                    }
                                    case 8: {
                                        var7_12.an = 51;
                                        v1.am = 51;
                                        var7_12.ao = 51;
                                        var7_12.J = 300;
                                        var7_12.K = 10;
                                        var7_12.L = 10;
                                        var7_12.aP = 0;
                                        var7_12.aO = 0;
                                        var7_12.aN = 0;
                                        var7_12.v = false;
                                        var7_12.w = false;
                                        var7_12.u = false;
                                        break;
                                    }
                                    case 12: {
                                        super.v();
                                        break;
                                    }
                                    case 10: {
                                        super.s(var7_12.am);
                                        super.r();
                                        break;
                                    }
                                    case 9: {
                                        super.C();
                                        break;
                                    }
                                    case 11: {
                                        super.a(3);
                                        var7_12.c = var7_12.a[3][0];
                                        var7_12.d = var7_12.b[0];
                                        var7_12.M = 0;
                                        var7_12.m = 0;
                                        super.i((var7_12.ag - 32 >> 1) - var7_12.N, (var7_12.ah - 32 >> 1) - var7_12.O);
                                        break;
                                    }
                                    case 14: {
                                        super.a(11);
                                        break;
                                    }
                                    case 15: {
                                        super.a(14);
                                        break;
                                    }
                                    case 16: {
                                        if (super.e(var7_12.bE)) break;
                                        var7_12.bO = 0;
                                        var7_12.a = (byte)8;
                                        super.a();
                                    }
                                }
                                ++var6_5.bP;
                                var6_5.bQ = var6_5.bP * 100 / var6_5.bO;
                                if (var6_5.bQ > var6_5.bN) break;
                                var6_5.bQ = var6_5.bN + 1;
                                break;
                            }
                            var6_5.bN = 0;
                            var6_5.bQ = 0;
                            var6_5.a[15] = null;
                            var6_5.a = var6_5.y;
                            if (!var6_5.t) break;
                            super.a();
                            break;
                        }
                        case 3: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -6: {
                                    var6_5.a = (byte)10;
                                    super.a();
                                    break;
                                }
                                case -7: {
                                    var6_5.c = var6_5.a;
                                    var6_5.a = (byte)4;
                                    super.a();
                                }
                            }
                            var6_5 = var5_3;
                            block102 : switch (var6_5.l) {
                                case 5: {
                                    super.c(true);
                                    break;
                                }
                                case 0: {
                                    if (var6_5.B) {
                                        super.l();
                                        break;
                                    }
                                    super.j();
                                    break;
                                }
                                case 1: {
                                    switch (var6_5.m) {
                                        case 1: {
                                            var6_5.O -= 8;
                                            if (var6_5.O + var6_5.af + 16 >= 106) break;
                                            super.i(var6_5.ae, var6_5.af + 8);
                                            break;
                                        }
                                        case 0: {
                                            var6_5.O += 8;
                                            if (var6_5.O + var6_5.af + 16 <= var6_5.ah - 106) break;
                                            super.i(var6_5.ae, var6_5.af - 8);
                                            break;
                                        }
                                        case 3: {
                                            var6_5.N -= 8;
                                            if (var6_5.N + var6_5.ae + 16 >= 106) break;
                                            super.i(var6_5.ae + 8, var6_5.af);
                                            break;
                                        }
                                        case 2: {
                                            var6_5.N += 8;
                                            if (var6_5.N + var6_5.ae + 16 <= var6_5.ag - 106) break;
                                            super.i(var6_5.ae - 8, var6_5.af);
                                        }
                                    }
                                    var6_5.R += 8;
                                    super.h();
                                    if (var6_5.R < 32) break;
                                    var6_5.R = 0;
                                    var6_5.M = 0;
                                    super.k();
                                    break;
                                }
                                case 2: {
                                    super.i();
                                    if (!var6_5.g) {
                                        switch (var6_5.g) {
                                            case -1: 
                                            case 50: {
                                                var6_5.m = 1;
                                                var6_5.af += 16;
                                                super.i(var6_5.ae, var6_5.af);
                                                break;
                                            }
                                            case -2: 
                                            case 56: {
                                                var6_5.m = 0;
                                                var6_5.af -= 16;
                                                super.i(var6_5.ae, var6_5.af);
                                                break;
                                            }
                                            case -3: 
                                            case 52: {
                                                var6_5.m = (byte)3;
                                                var6_5.ae += 16;
                                                super.i(var6_5.ae, var6_5.af);
                                                break;
                                            }
                                            case -4: 
                                            case 54: {
                                                var6_5.m = (byte)2;
                                                var6_5.ae -= 16;
                                                super.i(var6_5.ae, var6_5.af);
                                            }
                                        }
                                        switch (var6_5.f) {
                                            case -5: 
                                            case 53: {
                                                var6_5.g = true;
                                                super.e(0);
                                            }
                                        }
                                        break;
                                    }
                                    super.x();
                                    break;
                                }
                                case 3: {
                                    var7_13 = var6_5;
                                    switch (var7_13.f) {
                                        case -3: {
                                            if (var7_13.aR > 0) {
                                                if (var7_13.aS > 0) {
                                                    var7_13.aS = 0;
                                                    break block102;
                                                }
                                                var7_13.aS = var7_13.aR - 1;
                                                break block102;
                                            }
                                            ** GOTO lbl408
                                        }
                                        case -4: {
                                            if (var7_13.aR > 0) {
                                                if (var7_13.aS < var7_13.aR - 1) {
                                                    ++var7_13.aS;
                                                    break block102;
                                                }
                                                var7_13.aS = 0;
                                                break block102;
                                            }
                                            ** GOTO lbl408
                                        }
                                        case -6: 
                                        case -5: {
                                            super.l(var7_13.aS);
                                        }
                                        case -7: {
                                            var7_13.l = 0;
                                        }
                                    }
                                }
                            }
lbl408:
                            // 16 sources

                            super.q();
                            break;
                        }
                        case 4: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -1: {
                                    if (var6_5.k > 0) {
                                        --var6_5.k;
                                        if (var6_5.k >= var6_5.l) break;
                                        --var6_5.l;
                                        --var6_5.m;
                                        break;
                                    }
                                    var6_5.k = var6_5.h - 1;
                                    var6_5.m = var6_5.h;
                                    var6_5.l = var6_5.m - var6_5.n;
                                    if (var6_5.l >= 0) break;
                                    var6_5.l = 0;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.k < var6_5.h - 1) {
                                        ++var6_5.k;
                                        if (var6_5.k < var6_5.m) break;
                                        ++var6_5.m;
                                        ++var6_5.l;
                                        break;
                                    }
                                    var6_5.k = 0;
                                    var6_5.l = 0;
                                    var6_5.m = var6_5.h;
                                    if (var6_5.h <= var6_5.n) break;
                                    var6_5.m = var6_5.n;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    super.e();
                                    break;
                                }
                                case -7: {
                                    var6_5.a = var6_5.c;
                                }
                            }
                            break;
                        }
                        case 5: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -1: {
                                    super.m(var6_5.bf - 1);
                                    break;
                                }
                                case -2: {
                                    super.m(var6_5.bf + 1);
                                    break;
                                }
                                case -3: {
                                    break;
                                }
                                case -4: {
                                    break;
                                }
                                case -7: 
                                case -6: {
                                    var6_5.a = (byte)3;
                                }
                            }
                            break;
                        }
                        case 7: 
                        case 8: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -1: {
                                    if (var6_5.bE > 0) {
                                        if (--var6_5.bE >= var6_5.bC) break;
                                        --var6_5.bC;
                                        --var6_5.bD;
                                        break;
                                    }
                                    var6_5.bE = 5;
                                    var6_5.bD = 6;
                                    var6_5.bC = 6 - var6_5.bG;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.bE < 5) {
                                        if (++var6_5.bE < var6_5.bD) break;
                                        ++var6_5.bD;
                                        ++var6_5.bC;
                                        break;
                                    }
                                    var6_5.bE = 0;
                                    var6_5.bC = 0;
                                    var6_5.bD = var6_5.bG;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    if (var6_5.a == 7) {
                                        super.r(var6_5.am);
                                        super.q(var6_5.bE);
                                        super.a((byte)0, "\u4fdd\u5b58\u6210\u529f\uff01", (byte)0, (byte)0);
                                        break;
                                    }
                                    if (var6_5.a != 8 || !var6_5.h[var6_5.bE]) break;
                                    var6_5.g = 0;
                                    var6_5.f = 0;
                                    var6_5.a[10][0] = null;
                                    super.a((byte)3, true);
                                    super.u(6);
                                    super.u(8);
                                    super.u(5);
                                    super.u(13);
                                    super.u(9);
                                    super.u(16);
                                    super.u(10);
                                    super.u(2);
                                    super.u(3);
                                    super.u(11);
                                    break;
                                }
                                case -7: {
                                    var6_5.a = var6_5.b;
                                }
                            }
                            break;
                        }
                        case 9: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -1: {
                                    if (var6_5.aC > 0) {
                                        --var6_5.aC;
                                        break;
                                    }
                                    var6_5.aC = 2;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.aC < 2) {
                                        ++var6_5.aC;
                                        break;
                                    }
                                    var6_5.aC = 0;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    if (var6_5.aQ >= var6_5.ay) {
                                        var6_5.aQ -= var6_5.ay;
                                        var7_14 = var6_5;
                                        switch (var7_14.aC) {
                                            case 0: {
                                                var7_14.J += var7_14.az;
                                                break;
                                            }
                                            case 1: {
                                                var7_14.K += var7_14.aA;
                                                break;
                                            }
                                            case 2: {
                                                var7_14.L += var7_14.aB;
                                            }
                                        }
                                        ++var7_14.ar;
                                        var7_14.ay = a.b(var7_14.ar + 1);
                                        break;
                                    }
                                    super.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                    break;
                                }
                                case -7: {
                                    var6_5.a = (byte)3;
                                }
                            }
                            break;
                        }
                        case 10: {
                            var6_5 = var5_3;
                            block160 : switch (var6_5.f) {
                                case -1: {
                                    if (var6_5.ba >= var6_5.aT) {
                                        var6_5.ba -= var6_5.aT;
                                        if (var6_5.ba / var6_5.aT >= var6_5.aU) break;
                                        --var6_5.aU;
                                        --var6_5.aV;
                                        break;
                                    }
                                    var6_5.ba += (var6_5.aX - 1) * var6_5.aT;
                                    var6_5.aV = var6_5.aX;
                                    var6_5.aU = var6_5.aX - var6_5.aW;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.ba < var6_5.aT * (var6_5.aX - 1)) {
                                        var6_5.ba += var6_5.aT;
                                        if (var6_5.ba / var6_5.aT < var6_5.aV) break;
                                        ++var6_5.aU;
                                        ++var6_5.aV;
                                        break;
                                    }
                                    var6_5.ba -= (var6_5.aX - 1) * var6_5.aT;
                                    var6_5.aU = 0;
                                    var6_5.aV = var6_5.aW;
                                    break;
                                }
                                case -3: {
                                    if (var6_5.ba % var6_5.aT > 0) {
                                        --var6_5.ba;
                                        break;
                                    }
                                    var6_5.ba += var6_5.aT - 1;
                                    break;
                                }
                                case -4: {
                                    if (var6_5.ba % var6_5.aT < var6_5.aT - 1) {
                                        ++var6_5.ba;
                                        break;
                                    }
                                    var6_5.ba -= var6_5.aT - 1;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    if (var6_5.ba >= var6_5.aR) break;
                                    var6_5.h = var6_5.r[var6_5.ba];
                                    switch (var6_5.h) {
                                        case 13: 
                                        case 14: {
                                            var6_5.a = (byte)3;
                                            super.c(var6_5.h);
                                            break block160;
                                        }
                                        case 15: 
                                        case 23: 
                                        case 24: 
                                        case 25: {
                                            super.a((byte)3, var6_5.k[a.d(var6_5.h)], (byte)0, (byte)3);
                                            break block160;
                                        }
                                    }
                                    super.a((byte)3, var6_5.k[a.d(var6_5.h)], (byte)1, (byte)2);
                                    break;
                                }
                                case -7: {
                                    var6_5.a = (byte)3;
                                }
                            }
                            break;
                        }
                        case 11: {
                            super.q();
                            super.x();
                            var6_5 = var5_3;
                            var7_15 = '\u0000';
                            block172 : switch (var6_5.q) {
                                case 0: {
                                    if (var6_5.bo > 0) {
                                        --var6_5.bo;
                                        break;
                                    }
                                    super.a(var6_5.f, var6_5.bm);
                                    break;
                                }
                                case 1: 
                                case 6: {
                                    if (var6_5.q == 6 && !super.a()) break;
                                    if (var6_5.H >= var6_5.c.length()) ** GOTO lbl638
                                    if (var6_5.f == 0) ** GOTO lbl623
                                    super.a(var6_5.c, 129, (var6_5.I << 1) + 12);
                                    var6_5.H = super.a(var6_5.H);
                                    ** GOTO lbl633
lbl623:
                                    // 1 sources

                                    if ((var6_5.d & 1) == 0) ** GOTO lbl634
                                    ++var6_5.H;
                                    var7_15 = var6_5.c.charAt(var6_5.H - 1);
                                    if (var7_15 == '\\') {
                                        var7_15 = var6_5.c.charAt(var6_5.H);
                                        if (var7_15 == 'c') {
                                            var6_5.H += 8;
                                        } else if (var7_15 == 'r') {
                                            var6_5.H += 2;
                                        }
                                    }
lbl633:
                                    // 7 sources

                                    super.a(var6_5.c, 129, (var6_5.I << 1) + 12, var6_5.H);
lbl634:
                                    // 2 sources

                                    if (var6_5.C <= var6_5.B) break;
                                    super.a(var6_5.C - var6_5.E);
                                    break;
lbl638:
                                    // 1 sources

                                    switch (var6_5.f) {
                                        case 0: {
                                            break block172;
                                        }
                                        case -1: {
                                            super.a(var6_5.A - 1);
                                            break block172;
                                        }
                                        case -2: {
                                            super.a(var6_5.A + 1);
                                            break block172;
                                        }
                                    }
                                    var6_5.H = 1;
                                    super.n(var6_5.bi + 1);
                                    if (var6_5.q != 6) break;
                                    super.l(var6_5.bk, var6_5.bl);
                                    break;
                                }
                                case 2: {
                                    super.y();
                                    break;
                                }
                                case 3: {
                                    if (var6_5.q) break;
                                    var8_18 = var6_5;
                                    switch (var8_18.l) {
                                        case 5: {
                                            super.c(var8_18.o);
                                            break;
                                        }
                                        case 0: {
                                            if (var8_18.bS > 0) {
                                                var8_18.m = var8_18.Q[--var8_18.bS];
                                                var8_18.l = 1;
                                                super.a(var8_18.m);
                                                break;
                                            }
                                            var8_18.B = false;
                                            var8_18.q = 0;
                                            break;
                                        }
                                        case 1: {
                                            switch (var8_18.m) {
                                                case 1: {
                                                    var8_18.O -= 8;
                                                    break;
                                                }
                                                case 0: {
                                                    var8_18.O += 8;
                                                    break;
                                                }
                                                case 3: {
                                                    var8_18.N -= 8;
                                                    break;
                                                }
                                                case 2: {
                                                    var8_18.N += 8;
                                                }
                                            }
                                            var8_18.R += 8;
                                            var8_18.bv = var8_18.N;
                                            var8_18.bw = var8_18.O;
                                            super.h();
                                            if (var8_18.R < 32) break;
                                            var8_18.R = 0;
                                            var8_18.M = 0;
                                            var8_18.l = 0;
                                            var8_18.P = var8_18.N >> 5;
                                            var8_18.Q = var8_18.O >> 5;
                                            super.k();
                                        }
                                    }
                                    break;
                                }
                                case 4: {
                                    if (!super.a()) break;
                                    var6_5.a = (byte)3;
                                    break;
                                }
                                case 5: {
                                    if (!var6_5.q || var6_5.r) break;
                                    var6_5.q = 0;
                                }
                            }
                            break;
                        }
                        case 13: {
                            break;
                        }
                        case 14: {
                            super.f();
                            if ((var5_3.d & 3) != 0) break;
                            super.e(super.g(240), 320 - var5_3.q - super.g(150), 2, -1);
                            break;
                        }
                        case 12: {
                            super.o();
                            break;
                        }
                        case 19: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -3: 
                                case -1: 
                                case 50: 
                                case 52: {
                                    if (var6_5.k > 0) {
                                        --var6_5.k;
                                        break;
                                    }
                                    var6_5.k = var6_5.h - 1;
                                    break;
                                }
                                case -4: 
                                case -2: 
                                case 54: 
                                case 56: {
                                    if (var6_5.k < var6_5.h - 1) {
                                        ++var6_5.k;
                                        break;
                                    }
                                    var6_5.k = 0;
                                    break;
                                }
                                case -5: 
                                case 53: {
                                    break;
                                }
                                case -7: {
                                    var6_5.a = (byte)4;
                                    super.a();
                                }
                            }
                            break;
                        }
                        case 15: 
                        case 17: {
                            break;
                        }
                        case 16: {
                            var6_5 = var5_3;
                            switch (var6_5.f) {
                                case -1: {
                                    if (--var6_5.aE >= 0) break;
                                    var6_5.aE = var6_5.aF - 1;
                                    break;
                                }
                                case -2: {
                                    if (++var6_5.aE <= var6_5.aF - 1) break;
                                    var6_5.aE = 0;
                                    break;
                                }
                                case -5: 
                                case -4: 
                                case -3: {
                                    var7_16 = var6_5;
                                    var7_16.f[var7_16.aE] = var7_16.f[var7_16.aE] == false;
                                    switch (var7_16.l[var7_16.aE]) {
                                        case 0: {
                                            v2 = var7_16.D = var7_16.D == false;
                                            if (var7_16.D) {
                                                var7_16.bZ = 60;
                                                if (!var7_16.C) {
                                                    super.a((byte)2, -1);
                                                } else {
                                                    super.a((byte)3, -1);
                                                }
                                            } else {
                                                super.G();
                                            }
                                            super.z();
                                            break;
                                        }
                                        case 1: {
                                            if (!var7_16.f[1]) {
                                                var7_16.d = null;
                                                break;
                                            }
                                            if (var7_16.b != 4) break;
                                            super.r();
                                        }
                                    }
                                    break;
                                }
                                case -7: 
                                case -6: {
                                    var6_5.a = var6_5.b;
                                }
                            }
                            break;
                        }
                        case 20: {
                            if (!var5_3.e || var5_3.f == 0) break;
                            var5_3.a = 1;
                            super.a();
                            break;
                        }
                        case 22: {
                            if (++var5_3.ca <= 70) {
                                super.n(1, var5_3.ca);
                                break;
                            }
                            var5_3.a = false;
                            CMidlet.a();
                            break;
                        }
                        case 99: {
                            if (var5_3.F) {
                                switch (var5_3.f) {
                                    case -7: {
                                        super.I();
                                        var5_3.F = false;
                                        var5_3.a = 0;
                                    }
                                }
                                if (var5_3.cj < 8) {
                                    ++var5_3.cj;
                                    break;
                                }
                                if (!super.b()) break;
                                var5_3.F = false;
                                break;
                            }
                            if (super.f() != 0 && super.f() != 3 && var5_3.f == 0) break;
                            var5_3.a = 0;
                        }
                    }
                    var5_3.f = 0;
                }
                this.repaint();
                this.serviceRepaints();
                do {
                    Thread.yield();
                } while ((var5_2 = System.currentTimeMillis()) >= var1_7 && var5_2 < var3_10);
                ++this.d;
            }
            return;
        }
        catch (Exception v3) {
            this = v3;
            v3.printStackTrace();
            return;
        }
    }

    private void a() {
        switch (this.a) {
            case 21: {
                a a2 = this;
                String string = "SKY_WAR";
                int n = 0;
                try {
                    int n2;
                    a2.a = RecordStore.openRecordStore((String)string, (boolean)false);
                    a2.a = a2.a.enumerateRecords(null, null, false);
                    n = a2.a.nextRecordId();
                    a2.B = a2.a.getRecord(n);
                    a2.a = new DataInputStream(new ByteArrayInputStream(a2.B));
                    for (n2 = 0; n2 < 4; ++n2) {
                        a2.f[n2] = a2.a.readBoolean();
                    }
                    n = a2.a.readByte();
                    for (n2 = 0; n2 < n; ++n2) {
                        a2.J[n2] = a2.a.readInt();
                    }
                    a2.bW = a2.a.readInt();
                    a2.bX = a2.a.readInt();
                    a2.bY = a2.a.readInt();
                }
                catch (Exception exception) {
                    a.a(string);
                    a2.f[1] = true;
                    a2.z();
                }
                this.a(8);
                this.n = 1;
                this.o = (byte)2;
                this.D = false;
                return;
            }
            case 1: {
                this.C = false;
                this.d();
                if (this.ao < 51 && this.bY < this.ao) {
                    this.bY = this.ao;
                    this.z();
                }
                this.a(8);
                this.a(10);
                this.c();
                this.b(0);
                this.b(1);
                this.b(2);
                this.b(3);
                this.b(4);
                this.b(5);
                this.a((byte)2, -1);
                this.E();
                this.bL = -1;
                return;
            }
            case 14: {
                a a3 = this;
                this.t = 0;
                a3.u = 0;
                a3.v = 0;
                a3.b = null;
                a3.d = null;
                a3.b = null;
                a3.d = false;
                return;
            }
            case 13: {
                return;
            }
            case 5: {
                this.w();
                this.n = 0;
                this.o = (byte)3;
                return;
            }
            case 4: {
                this.c();
                this.b(1);
                if (this.am > 50) {
                    this.b(17);
                }
                this.b(3);
                this.b(4);
                this.b(5);
                this.b(7);
                this.n = 1;
                this.o = (byte)3;
                return;
            }
            case 3: {
                this.C = true;
                this.n = 1;
                this.o = (byte)3;
                this.f = false;
                this.k();
                this.a((byte)3, -1);
                return;
            }
            case 7: 
            case 8: {
                if (this.b == 1) {
                    this.a(1);
                    this.a(8);
                    this.a(6);
                }
                this.n = 1;
                this.o = (byte)3;
                a a4 = this;
                a4.A();
                a4.bG = 208 / (a4.b + 4);
                if (a4.bG > 6) {
                    a4.bG = 6;
                }
                a4.bF = a4.a * 3 + 60;
                if (a4.bF < 148) {
                    a4.bF = 148;
                }
                a4.bB = 112 + (a4.b + 4) * a4.bG;
                a4.bE = 0;
                a4.bC = 0;
                a4.bD = a4.bG;
                return;
            }
            case 9: {
                return;
            }
            case 10: {
                a a5 = this;
                this.aT = 4;
                a5.aX = 32 / a5.aT;
                a5.aW = (268 - a5.b - 10) / 40;
                if (a5.aW > a5.aX) {
                    a5.aW = a5.aX;
                }
                a5.aU = 0;
                a5.aV = a5.aW;
                a5.aZ = 42 + 40 * a5.aT;
                a5.aY = 34 + a5.b + 10 + 40 * a5.aW;
                this.n = 1;
                this.o = (byte)3;
                return;
            }
            case 12: {
                a a6 = this;
                a6.i(0);
                this.o = (byte)3;
                return;
            }
            case 19: {
                this.c();
                this.b(12);
                this.b(13);
                this.b(14);
                this.b(15);
                this.b(16);
                return;
            }
            case 15: {
                this.a(8);
                this.a(1);
                this.a((byte)0, "\u6e38\u620f\u63cf\u8ff0\uff1a\n\\c99FFCC\u6709\u4eba\u7684\u5730\u65b9\u5c31\u6709\u6c5f\u6e56\uff0c\u6709\u795e\u4ed9\u7684\u5730\u65b9\u4f55\u5c1d\u4e0d\u662f\u6c5f\u6e56\uff1b\u767e\u6218\u767e\u80dc\u7684\u672c\u4e8b\uff0c\u6362\u4e0d\u56de\u5973\u4eba\u7684\u771f\u5fc3\uff0c\u5144\u5f1f\u7684\u771f\u4e49\uff1b\u9f50\u5929\u5927\u5723\u53c8\u5982\u4f55\uff0c\u6ca1\u6709\u771f\u60c5\u5b9e\u4e49\uff0c\u505a\u795e\u4ed9\u8ddf\u505a\u54b8\u9c7c\u6709\u4ec0\u4e48\u533a\u522b\uff1f\n\n\u64cd\u4f5c\u65b9\u5f0f\uff1a\u6309\u5de6\u8f6f\u952e\u8c03\u51fa\u7269\u54c1\u680f\uff0c\u5de6\u53f3\u9009\u62e9\u4e00\u4ef6\u9053\u5177\uff0c\u6309\u786e\u5b9a\u952e\u4f7f\u7528\u3002\n\u6e38\u620f\u64cd\u4f5c\uff1a\n\u4e0a\u65b9\u5411\u952e/2\uff1a\u5411\u4e0a\u884c\u8d70\n\u4e0b\u65b9\u5411\u952e/8\uff1a\u5411\u4e0b\u884c\u8d70\n\u5de6\u65b9\u5411\u952e/4\uff1a\u5411\u5de6\u884c\u8d70\n\u53f3\u65b9\u5411\u952e/6\uff1a\u5411\u53f3\u884c\u8d70\n\u786e\u5b9a\u952e/5:\u63a2\u7d22\u5730\u56fe\n\u5de6\u8f6f\u952e\uff1a\u6253\u5f00\u9053\u5177\u5217\u8868\n\u53f3\u8f6f\u952e\uff1a\u6253\u5f00\u6e38\u620f\u4e2d\u83dc\u5355\n\n\u4ee3\u7406\u53d1\u884c\uff1a\u5e7f\u5dde\u6613\u8bda\u8ba1\u7b97\u673a\u79d1\u6280\u6709\u9650\u516c\u53f8\n\u53d1\u884c\u5546\u7f51\u7ad9\uff1awww.9266.net\n\u5ba2\u670d\u7535\u8bdd\uff1a4006509913\n\u5ba2\u670d\u4fe1\u7bb1\uff1akefu@9266.net", (byte)0, (byte)0);
                this.n = 0;
                this.o = (byte)3;
                return;
            }
            case 17: {
                this.a(8);
                this.a(1);
                this.a((byte)0, "\u7248\u6743\u6240\u6709\uff1a\n\u4e0a\u6d77\u96ea\u9ca4\u9c7c\u8ba1\u7b97\u673a\u79d1\u6280\u6709\u9650\u516c\u53f8\nwww.kgame.com.cn\n\u624b\u673a\u4e0a\u7f51\uff1a\nwap.kgame.com.cn\n\u5236\u4f5c\u4eba\uff1a\u6881\u4e00\n\u7f16\u5267\uff1a\u738b\u4e4b\u6d63\n\u7b56\u5212\uff1a\u5b59\u60a6\n\u7a0b\u5e8f\uff1a\u6768\u653f\n\u7f8e\u672f\uff1a\u6881\u4e00\u3001\u738b\u4e4b\u6d63\u3001\u9ec4\u5409\u529b\n\u6d4b\u8bd5\uff1a\u91d1\u946b\uff0c\u738b\u6bc5\uff0c\u8ba1\u6210\u6bc5\n\u7248\u672c\uff1aV1.0\n\u5ba2\u670d\u7535\u8bdd\uff1a4006305518", (byte)0, (byte)0);
                this.n = 0;
                this.o = (byte)3;
                return;
            }
            case 16: {
                this.a(8);
                this.a(1);
                this.aF = 0;
                this.h(0);
                this.h(1);
                this.n = 0;
                this.o = (byte)3;
                return;
            }
            case 20: {
                this.d();
                this.a(14);
                this.d = this.o[260];
                this.H = 1;
                this.a(this.c, 209, (this.I << 1) + 12, this.H);
                return;
            }
            case 22: {
                this.G();
            }
        }
    }

    protected final void showNotify() {
        if (this.b) {
            return;
        }
        this.b = true;
        super.showNotify();
    }

    protected final void hideNotify() {
        if (!this.b) {
            return;
        }
        if (this.a == 3) {
            this.a = (byte)4;
            this.a();
        }
        this.G();
        this.c = true;
        this.b = false;
        super.hideNotify();
    }

    private void a(int n) {
        int n2 = this.a[n];
        int n3 = 0;
        if (this.a[n] == null) {
            this.a[n] = new Image[this.a[n]];
        }
        this.a = this.getClass().getResourceAsStream(this.b[n]);
        byte[] byArray = null;
        try {
            for (int i = 0; i < n2; ++i) {
                n3 = this.a.read() & 0xFF | this.a.read() << 8 & 0xFF00;
                if (this.a[n][i] == null) {
                    if (byArray == null) {
                        byArray = new byte[n3];
                    } else if (byArray.length < n3) {
                        byArray = new byte[n3];
                    }
                    this.a.read(byArray, 0, n3);
                    this.a[n][i] = Image.createImage((byte[])byArray, (int)0, (int)n3);
                    continue;
                }
                this.a.skip(n3);
            }
            return;
        }
        catch (Exception exception) {
            Exception exception2 = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.b();
        }
    }

    private void a(Image image, int n, int n2, int n3, int n4, int n5, int n6) {
        this.a.setClip(n, n2, n5, n6);
        this.a.drawImage(image, n - n3, n2 - n4, 0);
        this.a.setClip(0, 0, 240, 320);
    }

    private static void a(Image image, Graphics graphics, int n, int n2, int n3, int n4, int n5, int n6) {
        graphics.setClip(n, n2, 5, 5);
        graphics.drawImage(image, n - n3, n2, 0);
        graphics.setClip(0, 0, 240, 320);
    }

    private void a(Image image, int n, int n2, int n3) {
        this.a.drawImage(image, n, n2, 0, b[n3]);
    }

    private void a(Image image, int n, int n2, int n3, int n4, int n5, int n6, int n7) {
        this.a.setClip(n, n2, n5, n6);
        if (n7 == 0) {
            this.a.drawImage(image, n - n3, n2 - n4, 0);
        } else {
            int n8 = image.getWidth();
            int n9 = image.getHeight();
            switch (n7) {
                case 1: {
                    this.a(image, n - (n8 - n3 - n5), n2 - n4, n7);
                    break;
                }
                case 2: {
                    this.a(image, n - n3, n2 - (n9 - n4 - n6), n7);
                    break;
                }
                case 3: {
                    this.a(image, n - (n8 - n3 - n5), n2 - (n9 - n4 - n6), n7);
                    break;
                }
                case 4: {
                    this.a.setClip(n, n2, n6, n5);
                    this.a(image, n - n4, n2 - n3, n7);
                    break;
                }
                case 5: {
                    this.a.setClip(n, n2, n6, n5);
                    this.a(image, n - (n9 - n4 - n6), n2 - n3, n7);
                    break;
                }
                case 6: {
                    this.a.setClip(n, n2, n6, n5);
                    this.a(image, n - n4, n2 - (n8 - n3 - n5), n7);
                    break;
                }
                case 7: {
                    this.a.setClip(n, n2, n6, n5);
                    this.a(image, n - (n9 - n4 - n6), n2 - (n8 - n3 - n5), n7);
                }
            }
        }
        this.a.setClip(0, 0, 240, 320);
    }

    private static Image a(int n, int n2, int n3) {
        Object object = null;
        object = new int[1024];
        n2 = ((int[])object).length;
        while (--n2 >= 0) {
            object[n2] = n3;
        }
        Image image = Image.createRGBImage((int[])object, (int)32, (int)32, (boolean)true);
        object = image;
        return image;
    }

    private static Image a(Image image, int n) {
        int n2 = image.getWidth();
        int n3 = image.getHeight();
        int[] nArray = new int[n2 * n3];
        image.getRGB(nArray, 0, n2, 0, 0, n2, n3);
        n <<= 24;
        int n4 = n2 * n3;
        while (--n4 >= 0) {
            int n5 = nArray[n4];
            if (n5 == -1 || (n5 & 0xFF000000) == 0) continue;
            nArray[n4] = nArray[n4] & 0xFFFFFF | n;
        }
        return Image.createRGBImage((int[])nArray, (int)n2, (int)n3, (boolean)true);
    }

    private void b() {
        if (this.a != null) {
            try {
                this.a.close();
            }
            catch (Exception exception) {}
            this.a = null;
        }
        if (this.a != null) {
            try {
                this.a.close();
            }
            catch (Exception exception) {}
            this.a = null;
        }
        if (this.b != null) {
            try {
                this.b.close();
            }
            catch (Exception exception) {}
            this.b = null;
        }
    }

    protected final void keyPressed(int n) {
        this.f = this.g = n;
        if (this.c) {
            if (this.a != 21 && this.a != 0) {
                if (!this.C) {
                    this.a((byte)2, -1);
                } else {
                    this.a((byte)3, -1);
                }
            }
            this.c = false;
        }
    }

    protected final void keyReleased(int n) {
        this.g = 0;
    }

    private void c() {
        this.p = this.b + 6;
        this.n = 238 / this.p;
        this.o = this.p * this.n;
        this.i = this.a << 1;
        this.j = 0;
        this.h = 0;
        if (this.a == null) {
            this.a = new byte[16];
        }
        this.l = 0;
    }

    private void b(int n) {
        if (this.h < 16) {
            int n2 = this.a.stringWidth(this.c[n]) + 60;
            this.a[this.h] = n;
            if (n2 > this.i) {
                this.i = n2;
            }
            ++this.h;
            if (this.h <= this.n) {
                this.m = this.h;
            }
            this.j += this.p;
            this.o = (this.m - this.l) * this.p;
        }
    }

    private void d() {
        this.a[1] = null;
        this.a[2] = null;
        this.a[3] = null;
        this.a[4] = null;
        this.a[5] = null;
        this.a[6] = null;
        this.a[7] = null;
        this.a[13] = null;
        this.a[9] = null;
        this.a[11] = null;
        this.a[14] = null;
        this.a[12] = null;
        this.a = null;
    }

    private void e() {
        switch (this.a[this.k]) {
            case 0: {
                return;
            }
            case 1: {
                this.a = (byte)3;
                return;
            }
            case 2: {
                this.a = (byte)8;
                this.b = (byte)4;
                this.a();
                return;
            }
            case 3: {
                this.a = (byte)7;
                this.b = (byte)4;
                this.a();
                return;
            }
            case 4: {
                this.a = (byte)16;
                this.b = (byte)4;
                this.a();
                return;
            }
            case 5: {
                this.a = (byte)15;
                this.b = (byte)4;
                this.a();
                return;
            }
            case 7: {
                this.d();
                System.gc();
                try {
                    Thread.sleep(100L);
                }
                catch (Exception exception) {}
                this.a = 1;
                this.a();
                this.k = 0;
                return;
            }
            case 9: {
                return;
            }
            case 10: {
                return;
            }
            case 11: {
                this.a = (byte)19;
                this.a();
                return;
            }
            case 17: {
                this.a(1, false, false);
                this.a = (byte)3;
                this.J = 1000;
                this.K = 710;
                this.L = 710;
            }
        }
    }

    private void b(Image image, int n, int n2, int n3, int n4, int n5, int n6, int n7) {
        n = 86;
        n7 = this.d;
        int n8 = 0;
        while (n8 < n5) {
            n3 = n + this.b[n7];
            n6 = n2 + this.c[n7];
            this.a.setClip(n3, n6, 17, 17);
            this.a.drawImage(image, n3 - n8, n6 - n4, 0);
            if (++n7 > 7) {
                n7 = 0;
            }
            n8 += 17;
            n += 17;
        }
        this.d = (byte)(this.d + 1);
        if (this.d > 7) {
            this.d = 0;
        }
        this.a.setClip(0, 0, 240, 320);
    }

    /*
     * Enabled force condition propagation
     * Lifted jumps to return sites
     */
    private void f() {
        if (this.f == -7) {
            this.b = null;
            this.a[11] = null;
            this.a[14] = null;
            this.b = null;
            this.t = 0;
            this.u = 0;
            this.v = 0;
            this.u();
            return;
        }
        if (this.u == 0) {
            this.b = null;
            switch (this.t) {
                case 0: {
                    this.a(11);
                    this.b = a.a(this.a[11][0], 34);
                    this.b = this.a[4];
                    break;
                }
                case 1: {
                    this.a(11);
                    this.b = a.a(this.a[11][1], 34);
                    this.b = this.a[6];
                    break;
                }
                case 2: {
                    this.a[11] = null;
                    this.a(14);
                    this.b = a.a(this.a[14][0], 34);
                    this.b = this.a[7];
                }
            }
            if (this.b != null) {
                this.b.getWidth();
            }
            this.d = false;
        }
        ++this.u;
        if (this.t < 3) {
            if (this.d == null) return;
            if (!this.d) {
                if (this.v < this.a.stringWidth(this.d[this.A]) + 10) {
                    this.v += 4;
                    if (this.f == 0) return;
                    this.v = 240;
                    return;
                }
                this.d = true;
                this.e = 0;
                return;
            }
            this.e = (byte)(this.e + 1);
            if (this.e < this.c.length) return;
            this.v = 0;
            this.e = 0;
            this.d = false;
            if (this.A < this.C - 2) {
                this.a(this.A + 2);
                return;
            }
            System.out.println("++~");
            ++this.t;
            this.u = 0;
            this.f();
            return;
        }
        this.b = null;
        this.a[11] = null;
        this.a[14] = null;
        this.b = null;
        this.t = 0;
        this.u();
    }

    private void a(byte by, String string, byte by2, byte by3) {
        this.f = true;
        this.i = by;
        this.c = string;
        switch (this.i) {
            case 1: 
            case 3: {
                this.a(string, 180, 234);
                this.y = 212;
                this.z = 68 + (this.B - this.A) * this.I;
                break;
            }
            case 0: 
            case 2: 
            case 4: 
            case 5: {
                this.a(string, 180, 240);
                this.y = this.D + 32;
                this.z = 32 + (this.B - this.A) * this.I;
            }
        }
        this.f = by2;
        this.g = by3;
        if (by2 != 0 || by3 != 0) {
            this.z += 18;
        }
    }

    private void g() {
        switch (this.f) {
            case -1: {
                this.a(this.A - 1);
                return;
            }
            case -2: {
                this.a(this.A + 1);
                return;
            }
            case 0: {
                return;
            }
        }
        this.f = false;
    }

    private void a(String string, int n, int n2) {
        this.a(string, n, n2, string.length());
    }

    private void a(String string, int n, int n2, int n3) {
        char[] cArray = null;
        int n4 = 0;
        char c = '\u0000';
        int n5 = 0;
        int n6 = 0;
        int n7 = 0;
        int n8 = 0;
        if (string == null || string.length() == 0) {
            return;
        }
        cArray = string.toCharArray();
        for (int i = 0; i < n3; ++i) {
            c = cArray[i];
            n4 = this.a.charWidth(c);
            if (c == '\n') {
                this.d[n6++] = new String(cArray, n7, n8);
                n7 = i + 1;
                n5 = 0;
                n8 = 0;
                continue;
            }
            if (c != '\\') {
                n4 = this.a.charWidth(c);
            } else if (++i < n3) {
                c = string.charAt(i);
                if (c == 'c') {
                    if (i + 6 < n3 && a.a(string.substring(i + 1, i + 7)) != -1) {
                        i += 6;
                        n8 += 7;
                        n4 = 0;
                    } else {
                        n4 = this.a.charWidth('\\') + this.a.charWidth(c);
                    }
                } else if (c == 'r') {
                    ++n8;
                    n4 = 0;
                }
            } else {
                n4 = this.a.charWidth('\\');
                --i;
            }
            if ((n5 += n4) > n) {
                this.d[n6++] = new String(cArray, n7, n8);
                n5 = n4;
                n7 = i;
                n8 = 1;
                continue;
            }
            ++n8;
        }
        if (n8 > 0) {
            this.d[n6++] = new String(cArray, n7, n8);
        }
        this.A = 0;
        this.C = n6;
        this.E = n2 / this.I;
        if (this.C < this.E) {
            this.E = this.C;
        }
        this.F = this.I * this.E;
        this.B = n6 > this.E ? this.E : n6;
        if (this.C == 1) {
            this.D = this.a.stringWidth(string);
            return;
        }
        this.D = n;
    }

    private void a(String string, int n, int n2, int n3, int n4, boolean bl) {
        if (string == null) {
            return;
        }
        if (this.d != string) {
            this.d = string;
            this.a(string, n3, n4);
        }
        int n5 = n2 + 2;
        n4 = 0;
        n4 = 0;
        char c = '\u0000';
        this.G = this.a.getColor();
        int n6 = this.A;
        while (--n6 >= 0) {
            if (this.d[n6] == null || (n4 = this.d[n6].lastIndexOf(92)) < 0 || (c = this.d[n6].charAt(n4 + 1)) != 'c') continue;
            n4 = a.a(this.d[n6].substring(n4 + 2, n4 + 8));
            this.a.setColor(n4);
            break;
        }
        n6 = this.A;
        while (n6 < this.B) {
            int n7 = n5;
            int n8 = n;
            String string2 = this.d[n6];
            a a2 = this;
            int n9 = string2.length();
            int n10 = 0;
            int n11 = n8;
            char c2 = '\u0000';
            int n12 = 0;
            for (int i = 0; i < n9; ++i) {
                if (string2.charAt(i) != '\\') continue;
                if (i > 0) {
                    a2.a.drawSubstring(string2, n10, i - n10, n11, n7, 0);
                    n11 += a2.a.substringWidth(string2, n10, i - n10);
                }
                if (++i >= n9) continue;
                c2 = string2.charAt(i);
                if (c2 == 'c') {
                    if (i + 6 < n9) {
                        n12 = a.a(string2.substring(i + 1, i + 7));
                    }
                    if (n12 != -1) {
                        a2.G = a2.a.getColor();
                        a2.a.setColor(n12);
                        n10 = (i += 6) + 1;
                        continue;
                    }
                    n10 = i - 1;
                    continue;
                }
                if (c2 == 'r') {
                    a2.a.setColor(a2.G);
                    n10 = i + 1;
                    continue;
                }
                n10 = i;
            }
            if (n10 != 0) {
                if (n10 < n9) {
                    a2.a.drawSubstring(string2, n10, n9 - n10, n11, n7, 0);
                }
            } else {
                a2.a.drawString(string2, n8, n7, 0);
            }
            ++n6;
            n5 += this.I;
        }
        n6 = n + (n3 >> 1);
        if (bl) {
            this.a.setColor(-1);
            if (this.A > 0) {
                n5 = n2 - 8 + (this.d & 1);
                this.a.fillTriangle(n6, n5, n6 - 7, n5 + 7, n6 + 7, n5 + 7);
            }
            if (this.B < this.C) {
                n5 = n2 + this.F + 3 - (this.d & 1);
                this.a.fillTriangle(n6, n5, n6 - 6, n5 - 6, n6 + 6, n5 - 6);
            }
        }
    }

    private boolean a(int n) {
        if (n >= 0) {
            if (n >= this.C - this.E + 1) {
                n = this.C - this.E;
            }
        } else {
            n = 0;
        }
        this.A = n;
        this.B = this.A + this.E;
        if (this.A < 0) {
            this.A = 0;
        }
        if (this.B > this.C) {
            this.B = this.C;
        }
        return true;
    }

    private int a(int n) {
        int n2 = 0;
        int n3 = 0;
        int n4 = 0;
        if (this.d != null) {
            int n5 = 0;
            while (n5 < this.C) {
                n4 = this.d[n5].length();
                if (n >= n2 && n < (n3 += n4)) {
                    return n3;
                }
                ++n5;
                n2 += n4;
            }
        } else {
            n3 = 0;
        }
        return n3;
    }

    private static int a(String string) {
        int n = 0;
        try {
            n = Integer.parseInt(string, 16);
        }
        catch (Exception exception) {
            n = 0x2B2B2B;
            exception.printStackTrace();
        }
        return n;
    }

    private void c(int n) {
        if (this.J <= -n) {
            this.J = 1;
            n = this.J - 1;
        } else {
            this.J += n;
        }
        if (n < 0) {
            this.a((byte)2, n, this.N, this.O);
            return;
        }
        if (n > 0) {
            if (this.A > 0) {
                n <<= 4;
            }
            this.a((byte)3, n, this.N, this.O);
        }
    }

    private void a(int n, int n2) {
        this.P = n;
        this.Q = n2;
        this.N = n << 5;
        this.O = n2 << 5;
        this.R = 0;
    }

    private void h() {
        if (this.M < this.d.length - 1) {
            ++this.M;
            return;
        }
        this.M = 0;
    }

    private void i() {
        int n = 0;
        int n2 = 0;
        int n3 = 0;
        if (!this.g) {
            n = (this.ag - 41 >> 1) - this.ae;
            n2 = (this.ah >> 1) - this.af + this.U;
        } else {
            n = this.N;
            n2 = this.O;
        }
        if (this.e[3] != n || this.f[3] != n2) {
            this.e[0] = n;
            this.f[0] = n2;
            for (int i = 1; i < 4; ++i) {
                n3 = 4 - i;
                if (i == 0) {
                    n3 += 5;
                }
                if (this.e[i] < n) {
                    int n4 = i;
                    this.e[n4] = this.e[n4] + ((n - this.e[i] >> 1) + n3);
                    if (this.e[i] > n) {
                        this.e[i] = n;
                    }
                } else if (this.e[i] > n) {
                    int n5 = i;
                    this.e[n5] = this.e[n5] + ((n - this.e[i] >> 1) - n3);
                    if (this.e[i] < n) {
                        this.e[i] = n;
                    }
                }
                if (this.f[i] < n2) {
                    int n6 = i;
                    this.f[n6] = this.f[n6] + ((n2 - this.f[i] >> 1) + n3);
                    if (this.f[i] > n2) {
                        this.f[i] = n2;
                    }
                } else if (this.f[i] > n2) {
                    int n7 = i;
                    this.f[n7] = this.f[n7] + ((n2 - this.f[i] >> 1) - n3);
                    if (this.f[i] < n2) {
                        this.f[i] = n2;
                    }
                }
                n = this.e[i];
                n2 = this.f[i];
            }
            return;
        }
        if (this.g) {
            this.l = 0;
        }
    }

    private void j() {
        int n = 0;
        if (this.a == 11) {
            this.l();
            return;
        }
        switch (this.f) {
            case -5: 
            case 53: {
                a a2 = this;
                for (int i = 0; i < 4; ++i) {
                    a2.e[i] = a2.N;
                    a2.f[i] = a2.O;
                }
                a2.g = false;
                a a3 = a2;
                int n2 = 0;
                int n3 = a3.T;
                while (--n3 >= 0) {
                    n2 = a3.l[n3];
                    if (a3.b[n3] || a3.d[n2] != 8) continue;
                    a3.n[n2] = a3.a(n2, true);
                }
                a2.l = (byte)2;
                a2.i = a2.c(13) >= 0;
                break;
            }
            case 49: {
                this.f = 0;
                if (!this.n || (n = this.a(true)) < 0) break;
                this.a(this.a[n] & 0xFF, false, true);
                break;
            }
            case 55: {
                this.f = 0;
                if (!this.n || (n = this.a(false)) < 0) break;
                this.a(this.a[n] & 0xFF, true, true);
            }
        }
        switch (this.g) {
            case -1: 
            case 50: {
                this.m = 1;
                this.a(this.m);
                return;
            }
            case -2: 
            case 56: {
                this.m = 0;
                this.a(this.m);
                return;
            }
            case -3: 
            case 52: {
                this.m = (byte)3;
                this.a(this.m);
                return;
            }
            case -4: 
            case 54: {
                this.m = (byte)2;
                this.a(this.m);
            }
        }
    }

    /*
     * WARNING - void declaration
     */
    private boolean a(byte by) {
        void var2_6;
        boolean bl = false;
        int n = this.N >> 5;
        int n2 = this.O >> 5;
        switch (by) {
            case 1: {
                --n2;
                break;
            }
            case 0: {
                ++n2;
                break;
            }
            case 3: {
                void var2_4;
                --var2_4;
                break;
            }
            case 2: {
                void var2_5;
                ++var2_5;
            }
        }
        boolean bl2 = this.a((int)var2_6, n2);
        if (bl2) {
            this.l = 1;
        } else if (this.f && this.l == 1) {
            this.A = false;
            this.bS = 0;
            this.l = 0;
        }
        return bl2;
    }

    /*
     * Enabled aggressive block sorting
     */
    private boolean a(int n, int n2) {
        byte by = 0;
        int n3 = 0;
        int n4 = 0;
        n4 = 0;
        int n5 = 1;
        int n6 = 0;
        by = this.c[n2][n];
        if (this.k[by] <= 0) {
            if (this.b(n, n2)) return n5 != 0;
            return 0 != 0;
        }
        int n7 = this.k[by];
        while (true) {
            if (--n7 < 0) {
                if (n5 == 0) return n5 != 0;
                if (n6 > 0) return n5 != 0;
                return this.b(n, n2) != 0;
            }
            n3 = this.d[by][n7] - 1;
            n4 = this.l[n3];
            if (this.e[n3] == 1) continue;
            block0 : switch (this.d[n4]) {
                case 8: {
                    n4 = this.a(n4, true);
                    n5 = 0;
                    if (n4 < 0 || n4 >= this.J) {
                        this.bS = 0;
                        this.a((byte)0, "\u4f60\u65e0\u6cd5\u6218\u80dc\u5b83", (byte)0, (byte)0);
                        break;
                    }
                    n4 = n3;
                    a a2 = this;
                    this.l = (byte)5;
                    a2.e[n4] = 3;
                    a2.S = n4;
                    a2.bU = a2.p[a2.l[n4] - 41];
                    a2.o = true;
                    a2.z = true;
                    break;
                }
                case 32: {
                    n5 = 0;
                    break;
                }
                case 16: {
                    n4 = n3;
                    a a3 = this;
                    this.aJ = n4;
                    n5 = a3.a[n4];
                    byte by2 = (byte)(n5 >>> 8);
                    n5 = (byte)n5;
                    int n8 = n4 = a3.l[n4] == 77 ? 0 : 1;
                    if (n5 > 0) {
                        n5 = (byte)(n5 - 1);
                        if ((by2 & 2) != 0) {
                            switch (n4) {
                                case 0: {
                                    boolean bl = false;
                                    n4 = n5;
                                    a3.a((byte)4, a3.h[n4], (byte)0, (byte)0);
                                    break;
                                }
                                case 1: {
                                    a3.a((byte)4, a3.i[n5], (byte)1, (byte)2);
                                }
                            }
                        } else {
                            a3.j(n4, n5);
                            switch (n4) {
                                case 0: {
                                    a3.a((byte)4, a3.g[n5], (byte)0, (byte)0);
                                    break;
                                }
                                case 1: {
                                    a3.a((byte)4, a3.j[n5], (byte)1, (byte)2);
                                    break;
                                }
                            }
                        }
                    }
                    n5 = 0;
                    break;
                }
                case 1: {
                    switch (n4) {
                        case 7: 
                        case 8: {
                            n6 = (byte)(n6 + 1);
                            break block0;
                        }
                        case 83: {
                            if (!this.b((int)this.a[n3], false)) break;
                            this.f = false;
                            this.c = null;
                            n5 = this.d(this.a[n3]) ? 1 : 0;
                            n7 = 0;
                            break block0;
                        }
                        case 1: {
                            n5 = this.b((byte)26);
                            if (n5 != 0) {
                                this.e(n3);
                                break block0;
                            }
                            this.bS = 0;
                            this.a((byte)0, "\u4f60\u6ca1\u6709\u9ec4\u94a5\u5319", (byte)0, (byte)0);
                            break block0;
                        }
                        case 3: {
                            n5 = this.b((byte)28);
                            if (n5 != 0) {
                                this.e(n3);
                                break block0;
                            }
                            this.bS = 0;
                            this.a((byte)0, "\u4f60\u6ca1\u6709\u84dd\u94a5\u5319", (byte)0, (byte)0);
                            break block0;
                        }
                        case 2: {
                            n5 = this.b((byte)27);
                            if (n5 != 0) {
                                this.e(n3);
                                break block0;
                            }
                            this.bS = 0;
                            this.a((byte)0, "\u4f60\u6ca1\u6709\u7ea2\u94a5\u5319", (byte)0, (byte)0);
                            break block0;
                        }
                        case 81: {
                            if (!this.a[n3]) break;
                            this.a((byte)0, "\u969c\u788d\u7269\uff1a\u5c01\u5370\u95e8\n\u9700\u8981\u6d88\u706d\u6307\u5b9a\u7684\u602a\u7269\u624d\u80fd\u6253\u5f00\u7684\u95e8\uff01", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 4: {
                            this.a((byte)0, "\u969c\u788d\u7269\uff1a\u5c01\u5370\u95e8\n\u9700\u8981\u6d88\u706d\u6307\u5b9a\u7684\u602a\u7269\u624d\u80fd\u6253\u5f00\u7684\u95e8\uff01", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 9: {
                            n5 = 0;
                            if (this.m != 1) break;
                            n7 = 0;
                            this.a = (byte)9;
                            a a4 = this;
                            n4 = a4.am;
                            a4.ay = a.b(a4.ar + 1);
                            a4.az = a.a(100, n4);
                            a4.aA = a.a(2, n4);
                            a4.aB = a.a(4, n4);
                            this.a();
                            break block0;
                        }
                        case 10: {
                            this.a((byte)0, "\u969c\u788d\u7269\uff1a\u4e09\u6627\u771f\u706b\n\u5fc5\u987b\u7528\u82ad\u8549\u6247\u624d\u80fd\u7184\u706d\u5b83. ", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 11: {
                            if (this.am != 23) {
                                this.a((byte)0, "\u969c\u788d\u7269\uff1a\u5899\n\u53ea\u6709\u91d1\u52fa\u5b50\u3001\u7384\u660e\u77f3\u53ef\u4ee5\u51ff\u5f00\u3002\u6216\u8005\u5267\u60c5\u6253\u5f00\uff01", (byte)0, (byte)0);
                            }
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 12: {
                            this.l[n3] = 11;
                            this.f(11, n3);
                            this.g(11, n3);
                            this.e[n3] = 2;
                            this.m[n3] = 9;
                            this.a[n3] = true;
                            n5 = 0;
                            break block0;
                        }
                        case 6: {
                            n6 = (byte)(n6 + 1);
                            if (this.a[n3]) break;
                            this.a[n3] = true;
                            this.e[n3] = 2;
                            this.m[n3] = 9;
                        }
                    }
                    break;
                }
            }
        }
    }

    /*
     * Unable to fully structure code
     */
    private void k() {
        block62: {
            block61: {
                block59: {
                    var1_1 = false;
                    this.P = this.N >> 5;
                    this.Q = this.O >> 5;
                    var1_1 = false;
                    var2_6 = 0;
                    var3_7 = this.c[this.Q][this.P];
                    this.h = false;
                    var1_2 = this;
                    var2_6 = 0;
                    var6_8 = 0;
                    var7_9 = var1_2.k == 40 ? 1 : 0;
                    if (var7_9 == 0) {
                        var10_10 = var1_2.Q;
                        var9_11 = var1_2.P;
                        var8_13 = var1_2;
                        if (var9_11 > 0 && var8_13.b(var9_11 - 1, var10_10, 61) >= 0 && var9_11 < var8_13.Y - 1 && var8_13.b(var9_11 + 1, var10_10, 61) >= 0 ? true : var10_10 > 0 && var8_13.b(var9_11, var10_10 - 1, 61) >= 0 && var10_10 < var8_13.Z - 1 && var8_13.b(var9_11, var10_10 + 1, 61) >= 0) {
                            var1_2.c(-(var1_2.J >> 1));
                        }
                        var11_16 = 62;
                        var10_10 = var1_2.Q;
                        var9_11 = var1_2.P;
                        var8_13 = var1_2;
                        var12_17 = 0;
                        if (var9_11 > 0 && var8_13.b(var9_11 - 1, var10_10, 62) >= 0) {
                            ++var12_17;
                        }
                        if (var9_11 < var8_13.Z - 1 && var8_13.b(var9_11 + 1, var10_10, 62) >= 0) {
                            ++var12_17;
                        }
                        if (var10_10 > 0 && var8_13.b(var9_11, var10_10 - 1, 62) >= 0) {
                            ++var12_17;
                        }
                        if (var10_10 < var8_13.Z - 1 && var8_13.b(var9_11, var10_10 + 1, 62) >= 0) {
                            ++var12_17;
                        }
                        if ((var2_6 = var12_17) > 0) {
                            var1_2.c(-100);
                        }
                    }
                    var11_16 = var7_9;
                    var10_10 = var1_2.Q;
                    var9_11 = var1_2.P;
                    var8_13 = var1_2;
                    var12_17 = 0;
                    var7_9 = 0;
                    if (var11_16 == 0) {
                        var7_9 = var8_13.b(var9_11 - 1, var10_10, 60);
                        if (var9_11 > 0 && var7_9 >= 0) {
                            ++var12_17;
                        }
                        var7_9 = var8_13.b(var9_11 + 1, var10_10, 60);
                        if (var9_11 < var8_13.Z - 1 && var7_9 >= 0) {
                            ++var12_17;
                        }
                        var7_9 = var8_13.b(var9_11, var10_10 - 1, 60);
                        if (var10_10 > 0 && var7_9 >= 0) {
                            ++var12_17;
                            var8_13.d(var9_11, var10_10 - 1, var7_9);
                        }
                        var7_9 = var8_13.b(var9_11, var10_10 + 1, 60);
                        if (var10_10 < var8_13.Z - 1 && var7_9 >= 0) {
                            ++var12_17;
                        }
                    } else {
                        var7_9 = var8_13.b(var9_11, var10_10 - 1, 60);
                        if (var10_10 > 0 && var7_9 >= 0) {
                            var8_13.d(var9_11, var10_10 - 1, var7_9);
                        }
                    }
                    if ((var2_6 = var12_17) > 0) {
                        var1_2.c(-200);
                        var6_8 = 1;
                    }
                    var4_18 = var6_8;
                    if (this.k[var3_7] <= 0) break block59;
                    var5_19 = this.k[var3_7];
                    while (--var5_19 >= 0) {
                        block60: {
                            var1_3 = this.d[var3_7][var5_19] - 1;
                            var2_6 = this.l[var1_3];
                            switch (this.d[var2_6]) {
                                case 2: 
                                case 4: {
                                    this.j(var2_6);
                                    this.e(var1_3);
                                    break;
                                }
                                case 1: {
                                    switch (var2_6) {
                                        case 83: {
                                            if (this.b((int)this.a[var1_3], true)) {
                                                this.f = false;
                                                this.d(this.a[var1_3]);
                                                var4_18 = 1;
                                                var5_19 = 0;
                                            }
                                            break block60;
                                        }
                                        case 7: {
                                            var1_3 = this.a[var1_3] & 255;
                                            this.a(var1_3, false, false);
                                            this.g = 0;
                                            var5_19 = 0;
                                            break block60;
                                        }
                                        case 8: {
                                            var1_3 = this.a[var1_3] & 255;
                                            this.a(var1_3, true, false);
                                            this.g = 0;
                                            var5_19 = 0;
                                            break block60;
                                        }
                                        case 5: {
                                            var1_3 = this.a[var1_3];
                                            this.b(this.P, this.Q, var5_19);
                                            if (this.b(var1_3)) {
                                                this.l = (byte)4;
                                            }
                                            break block60;
                                        }
                                        case 76: {
                                            var1_3 = (this.a[var1_3] & 255) + 1;
                                            this.b(this.P, this.Q, var5_19);
                                            var2_6 = var1_3;
                                            var1_4 = this;
                                            var6_8 = 0;
                                            var1_4.aI = 0;
                                            var7_9 = 0;
                                            for (var8_14 = 0; var8_14 < var1_4.T; ++var8_14) {
                                                if (var1_4.b[var8_14]) continue;
                                                var7_9 = var1_4.a[var8_14] & 255;
                                                var6_8 = var1_4.l[var8_14];
                                                if (var6_8 == 76) {
                                                    if (var7_9 + 1 != var2_6) continue;
                                                    var11_16 = var1_4.h[var8_14] >> 5;
                                                    var10_10 = var1_4.g[var8_14] >> 5;
                                                    var9_12 = var1_4;
                                                    var12_17 = var9_12.c[var11_16][var10_10];
                                                    var7_9 = var9_12.k[var12_17];
                                                    if (var9_12.k[var12_17] <= 0) continue;
                                                    var6_8 = var7_9;
                                                    while (--var6_8 >= 0) {
                                                        var9_12.b[var9_12.d[var12_17][var6_8] - 1] = true;
                                                    }
                                                    var9_12.k[var12_17] = 0;
                                                    continue;
                                                }
                                                if (var6_8 != 4 || var7_9 != var2_6) continue;
                                                var1_4.d[var1_4.aI] = var8_14;
                                                var1_4.b[var1_4.aI] = var1_4.g[var8_14] >> 5;
                                                var1_4.c[var1_4.aI] = var1_4.h[var8_14] >> 5;
                                                ++var1_4.aI;
                                            }
                                            if (var1_4.aI > 0) {
                                                this.l = (byte)4;
                                            }
                                            break block60;
                                        }
                                        case 6: {
                                            this.h = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if (this.l != 4) {
                        this.l = 0;
                        if (var4_18 == 0) {
                            this.j();
                        }
                    }
                    break block61;
                }
                if (this.B) {
                    if (var4_18 == 0) {
                        this.l();
                    } else {
                        this.l = 0;
                    }
                } else {
                    this.l = 0;
                    if (var4_18 == 0) {
                        this.j();
                    }
                }
            }
            if (!this.A) break block62;
            if (this.bS <= 0) ** GOTO lbl163
            if (this.Q[this.bS - 1] == this.m) {
                --this.bS;
            } else {
                this.bS = 0;
lbl163:
                // 2 sources

                this.A = false;
            }
        }
        var1_5 = this;
        var2_6 = var1_5.P;
        var6_8 = var1_5.Q;
        var7_9 = var1_5.as >> 5;
        var8_15 = var1_5.au >> 5;
        var9_11 = var1_5.at >> 5;
        var10_10 = var1_5.av >> 5;
        var1_5.n = false;
        if (var1_5.m) {
            if (var6_8 > 0) {
                if (var2_6 == var7_9 && var6_8 - 1 == var8_15) {
                    var1_5.n = true;
                } else if (var2_6 == var9_11 && var6_8 - 1 == var10_10) {
                    var1_5.n = true;
                }
            }
            if (var6_8 < var1_5.Z - 1) {
                if (var2_6 == var7_9 && var6_8 + 1 == var8_15) {
                    var1_5.n = true;
                } else if (var2_6 == var9_11 && var6_8 + 1 == var10_10) {
                    var1_5.n = true;
                }
            }
            if (var2_6 > 0) {
                if (var2_6 - 1 == var7_9 && var6_8 == var8_15) {
                    var1_5.n = true;
                } else if (var2_6 - 1 == var9_11 && var6_8 == var10_10) {
                    var1_5.n = true;
                }
            }
            if (var2_6 < var1_5.Y - 1) {
                if (var2_6 + 1 == var7_9 && var6_8 == var8_15) {
                    var1_5.n = true;
                    return;
                }
                if (var2_6 + 1 == var9_11 && var6_8 == var10_10) {
                    var1_5.n = true;
                }
            }
        }
    }

    private void b(int n, int n2) {
        if (this.b(n, n2 - 1)) {
            --n2;
        } else if (this.b(n, n2 + 1)) {
            ++n2;
        } else if (this.b(n - 1, n2)) {
            --n;
        } else if (this.b(n + 1, n2)) {
            ++n;
        }
        this.a(n, n2);
        this.i((this.ag - 32 >> 1) - this.N, (this.ah - 32 >> 1) - this.O);
    }

    private void l() {
        if (this.bS > 0) {
            this.m = this.Q[--this.bS];
            boolean bl = this.a(this.m);
            if (this.bS == 0) {
                this.B = false;
                return;
            }
            if (!bl) {
                this.B = false;
                if (this.l != 5) {
                    this.l = 0;
                    this.bS = 0;
                }
            }
        }
    }

    private void c(int n, int n2) {
        int n3 = n + this.N;
        int n4 = n2 + this.O;
        int n5 = 0;
        if (this.h) {
            n4 -= this.U;
        }
        this.a.drawImage(this.a[5][0], n3 + 8, n4 + 22, 0);
        switch (this.m) {
            case 1: {
                this.a(this.a[3][0], n3 - 8, n4 - 14, this.d[this.M] * 41, 46, 41, 46);
                if (this.l != 2) break;
                n5 = 4;
                while (--n5 >= 0) {
                    this.a(this.c, n + this.e[n5] - 8, n2 + this.f[n5] - 14, this.d[this.M] * 41, 46, 41, 46);
                }
                break;
            }
            case 0: {
                if (this.l == 2) {
                    n5 = 4;
                    while (--n5 >= 0) {
                        this.a(this.c, n + this.e[n5] - 4, n2 + this.f[n5] - 14, this.d[this.M] * 41, 0, 41, 46);
                    }
                }
                this.a(this.a[3][0], n3 - 3, n4 - 18, this.d[this.M] * 41, 0, 41, 46);
                break;
            }
            case 3: {
                if (this.l == 2) {
                    n5 = 4;
                    while (--n5 >= 0) {
                        this.a(this.c, n + this.e[n5], n2 + this.f[n5] - 14, this.d[this.M] * 41, 92, 41, 46, 1);
                    }
                }
                this.a(this.a[3][0], n3, n4 - 16, this.d[this.M] * 41, 92, 41, 46, 1);
                break;
            }
            case 2: {
                if (this.l == 2) {
                    n5 = 4;
                    while (--n5 >= 0) {
                        this.a(this.c, n + this.e[n5] - 6, n2 + this.f[n5] - 14, this.d[this.M] * 41, 92, 41, 46);
                    }
                }
                this.a(this.a[3][0], n3 - 8, n4 - 16, this.d[this.M] * 41, 92, 41, 46);
            }
        }
        this.a.setClip(0, 0, 240, 320);
        if (this.A) {
            n3 = (this.P << 5) + n;
            n4 = (this.Q << 5) + n2;
            n5 = this.bS;
            while (--n5 >= 0) {
                this.a.setColor(136);
                switch (this.Q[n5]) {
                    case 1: {
                        this.a(this.a[8][22], n3 + 5, (n4 -= 32) + 5 + this.U, 0, 0, 21, 22);
                        break;
                    }
                    case 0: {
                        this.a(this.a[8][22], n3 + 5, (n4 += 32) + 5 + this.U, 21, 0, 21, 22);
                        break;
                    }
                    case 3: {
                        this.a(this.a[8][22], (n3 -= 32) + 5 + this.U, n4 + 5, 42, 0, 21, 22);
                        break;
                    }
                    case 2: {
                        this.a(this.a[8][22], (n3 += 32) + 5 + this.U, n4 + 5, 42, 0, 21, 22, 1);
                    }
                }
            }
        }
        n3 = n + this.N;
        n4 = n2 + this.O;
        switch (this.l) {
            case 2: {
                n5 = this.U;
                this.a(this.a[8][23], 5 - n5, 148, 44, 0, 22, 24);
                this.a(this.a[8][23], 109, 25 - n5, 0, 0, 22, 24);
                this.a(this.a[8][23], 109, this.W - 29 + n5, 22, 0, 22, 24);
                this.a(this.a[8][23], 215 + n5, 148, 44, 0, 22, 24, 1);
                return;
            }
            case 3: {
                n2 = n4 - 40;
                n = n3 + 16;
                n3 = 0;
                n4 = 0;
                if (this.aS < this.aR) {
                    n3 = this.r[this.aS];
                    this.a.drawImage(this.a[n3], n - 16, n2 + 4, 0);
                    if (this.aS > 0) {
                        this.a.setColor(-1);
                        this.a(n - 18 - (this.d & 1), n2 + 16, (byte)3);
                    }
                    if (this.aS < this.aR - 1) {
                        this.a.setColor(-1);
                        this.a(n + 18 + (this.d & 1), n2 + 16, (byte)3);
                    }
                    this.a.setColor(-1);
                    n4 = this.a.stringWidth(this.m[n3]);
                    this.a.fillRect((n -= n4 >> 1) - 5, n2 -= this.b + 4, n4 + 10, this.b + 4);
                    this.a.setColor(0);
                    this.a.drawString(this.m[n3], n, n2 + 2, 0);
                    return;
                }
                this.a.setColor(-1);
                this.a.fillRect((n -= this.a >> 1) - 5, n2 -= this.b + 4 - 32, this.a + 10, this.b + 4);
                this.a.setColor(0);
                this.a.drawString("\u65e0", n, n2 + 2, 0);
            }
        }
    }

    private void m() {
        if (this.n != 0) {
            this.a.drawImage(this.a[8][10], 0, 302, 0);
            this.a(this.a[8][11], 2, 307, (this.n - 1) * 12, 0, 12, 10);
        }
        if (this.o != 0) {
            this.a(this.a[8][10], 222, 302, 1);
            this.a(this.a[8][11], 226, 307, (this.o - 1) * 12, 0, 12, 10);
        }
    }

    private void d(int n, int n2) {
        n = n2;
        this.a.setClip(0, 0, 240, 320);
        this.a.drawImage(this.a[8][3], 0, n, 0);
        if (this.am > 50) {
            this.a.setColor(-1);
            this.a("\u5f15\u5b50", 32, n + 9 + (19 - this.b >> 1), 17, this.K);
            n += 4;
        } else {
            this.a(this.a[8][18], this.am, 40, (n += 4) + 2);
            this.a.drawImage(this.a[8][8], 41, n + 10, 0);
        }
        this.a.setColor(2435368);
        this.a.fillRect(65, n2, 175, 18);
        this.a(0, this.aN, 65, n -= 4);
        this.a(1, this.aO, 107, n);
        this.a(2, this.aP, 149, n);
        this.a.drawImage(this.a[8][1], 191, n += 3, 0);
        this.a(this.a[8][2], this.aQ, 237, n + 2);
    }

    private void a(int n, int n2, int n3, int n4) {
        this.a.setClip(n3, n4, 18, 16);
        this.a.drawImage(this.a[8][6], n3 - n * 18, n4, 0);
        this.a.setClip(0, 0, 240, 320);
        this.a(this.a[8][2], n2, n3 += 40, n4 += 5);
    }

    private void e(int n, int n2) {
        this.b(0, n2, 240, 50);
        this.a.drawImage(this.a[3][1], 40 - (this.a[3][1].getWidth() >> 1), n2 + 50 - this.a[3][1].getHeight(), 0);
        int n3 = n = 83;
        int n4 = n2 += 13;
        this.a.setClip(n3, n4, 10, 10);
        this.a.drawImage(this.a[8][7], n3, n4 - 2, 0);
        this.a.setClip(0, 0, 240, 320);
        this.d(n3 += 16, n4 + 1, 58, 11);
        this.a(this.a[8][2], this.J, n3 + 52, n4 + 2);
        n3 = n;
        this.a(this.a[8][7], n3, n4 += 12, 10, 0, 10, 13);
        this.a.setColor(512);
        this.d(n3 += 16, n4 + 1, 58, 11);
        this.a(this.a[8][2], this.K, n3 + 52, n4 + 2);
        n3 = n;
        this.a(this.a[8][7], n3, n4 += 12, 20, 0, 10, 13);
        this.a.setColor(512);
        this.d(n3 += 16, n4 + 1, 58, 11);
        this.a(this.a[8][2], this.L, n3 + 52, n4 + 2);
        n4 = n2 + 2;
        this.d(n3 += 60, n4, 32, 32);
        if (this.j == 0) {
            this.a.setColor(-1);
            this.a.drawString(this.e[this.j], n3 + (32 - this.a >> 1), n4 + (32 - this.b >> 1), 0);
        } else {
            this.a.drawImage(this.a[this.j], n3 + (32 - this.a[this.j].getWidth() >> 1), n4 + (32 - this.a[this.j].getHeight() >> 1), 0);
        }
        this.d(n3 += 34, n4, 32, 32);
        if (this.k == 0) {
            this.a.setColor(-1);
            this.a.drawString(this.e[this.k], n3 + (32 - this.a >> 1), n4 + (32 - this.b >> 1), 0);
            return;
        }
        this.a.drawImage(this.a[this.k], n3 + (32 - this.a[this.k].getWidth() >> 1), n4 + (32 - this.a[this.k].getHeight() >> 1), 0);
    }

    private void a(int n, int n2, int n3, int n4, int n5) {
        this.c(n2, n3, n4, n5);
        n3 -= 2;
        n2 += 2;
        int[] nArray = this.a[n];
        n4 = nArray.length;
        n5 = 0;
        while (n5 < n4) {
            this.a.setClip(n2, n3, 14, 16);
            this.a.drawImage(this.a[8][13], n2 - nArray[n5] * 14, n3, 0);
            ++n5;
            n2 += 14;
        }
        this.a.setClip(0, 0, 240, 320);
    }

    private void b(int n, int n2, int n3, int n4) {
        int n5 = n;
        int n6 = n2;
        this.a.setClip(n5, n6, 26, 16);
        this.a.drawImage(this.a[8][0], n5, n6, 0);
        n5 += 26;
        while (n5 < n + n3 - 26) {
            this.a.setClip(n5, n6, 16, 16);
            this.a.drawImage(this.a[8][0], n5 - 26, n6, 0);
            n5 += 16;
        }
        this.a.setClip(0, 0, 240, 320);
        this.a(this.a[8][0], n + n3 - 26, n6, 0, 0, 26, 16, 1);
        n5 = n + 11;
        this.a.setColor(2699825);
        this.a.fillRect(n5, n6 += 16, n3 - 22, n4 - 16);
        n5 = n + n3 - 11;
        while (n6 < n2 + n4) {
            this.a.setClip(n, n6, 11, 16);
            this.a.drawImage(this.a[8][0], n - 42, n6, 0);
            this.a.setClip(n5, n6, 11, 16);
            this.a.drawImage(this.a[8][0], n5 - 53, n6, 0);
            n6 += 16;
        }
        this.a.setClip(0, 0, 240, 320);
    }

    private void c(int n, int n2, int n3, int n4) {
        int n5 = n;
        int n6 = n2;
        this.a.setClip(n5, n6, 26, 16);
        this.a.drawImage(this.a[8][0], n5, n6, 0);
        n5 += 26;
        while (n5 < n + n3 - 26) {
            this.a.setClip(n5, n6, 16, 16);
            this.a.drawImage(this.a[8][0], n5 - 26, n6, 0);
            this.a(this.a[8][0], n5, n6 + n4 - 16, 26, 0, 16, 16, 2);
            n5 += 16;
        }
        this.a.setClip(0, 0, 240, 320);
        this.a(this.a[8][0], n + n3 - 26, n6, 0, 0, 26, 16, 1);
        n5 = n + 11;
        this.a.setColor(2699825);
        this.a.fillRect(n5, n6 += 16, n3 - 22, n4 - 32);
        n5 = n + n3 - 11;
        while (n6 < n2 + n4 - 16) {
            this.a.setClip(n, n6, 11, 16);
            this.a.drawImage(this.a[8][0], n - 42, n6, 0);
            this.a.setClip(n5, n6, 11, 16);
            this.a.drawImage(this.a[8][0], n5 - 53, n6, 0);
            n6 += 16;
        }
        n6 = n2 + n4 - 16;
        this.a(this.a[8][0], n, n6, 0, 0, 26, 16, 2);
        this.a(this.a[8][0], n + n3 - 26, n6, 0, 0, 26, 16, 3);
        this.a.setClip(0, 0, 240, 320);
    }

    private void d(int n, int n2, int n3, int n4) {
        int n5 = n2 + n4 - 2;
        int n6 = n + n3 - 1;
        this.a.setColor(4803902);
        this.a.fillRect(n + 1, n2, n3 - 2, n4 - 1);
        this.a.drawLine(n, n2 + 1, n, n5);
        this.a.setColor(1645850);
        this.a.drawLine(n6, n2 + 1, n6, n5);
        this.a.drawLine(n + 1, n5 + 1, n6 - 1, n5 + 1);
    }

    private int a(Image image, int n, int n2, int n3) {
        boolean bl = n < 0;
        if (bl) {
            n = -n;
        }
        int n4 = image.getWidth() / 10;
        int n5 = image.getHeight();
        int n6 = 0;
        int n7 = 0;
        do {
            n6 = n % 10;
            this.a.setClip(n2 -= n4 + 1, n3, n4, n5);
            this.a.drawImage(image, n2 - n6 * n4, n3, 0);
            ++n7;
        } while ((n /= 10) > 0);
        this.a.setClip(0, 0, 240, 320);
        if (bl) {
            this.a.setColor(15027533);
            this.a.drawLine(n2 - n4 + 1, n3 + 3, n2 - 1, n3 + 3);
            ++n7;
        }
        return n7;
    }

    private void n() {
        this.a = new Image[this.m.length];
        this.d = new byte[this.m.length];
        this.c = new boolean[this.m.length];
        int n = this.d.length;
        while (--n >= 0) {
            this.c[n] = this.d[n];
        }
        this.f = new byte[this.m.length];
        this.n = new int[this.m.length];
        this.g = new int[100];
        this.h = new int[100];
        this.i = new int[100];
        this.j = new int[100];
        this.k = new int[100];
        this.l = new int[100];
        this.m = new int[100];
        this.a = new boolean[100];
        this.b = new boolean[100];
        this.a = new short[100];
        this.e = new byte[100];
        this.f[74] = 1;
        this.f[53] = 1;
        this.f[49] = 1;
        this.f[69] = 2;
        for (n = 1; n <= 12; ++n) {
            this.d[n] = 1;
        }
        for (n = 13; n <= 32; ++n) {
            this.d[n] = 2;
        }
        for (n = 33; n <= 40; ++n) {
            this.d[n] = 4;
        }
        for (n = 41; n < 79; ++n) {
            this.d[n] = 8;
        }
        this.d[76] = 1;
        this.d[77] = 16;
        this.d[78] = 16;
        this.d[79] = 4;
        this.d[80] = 4;
        this.d[81] = 1;
        this.d[83] = 1;
        this.d[72] = 32;
        this.d[84] = 32;
        this.d[87] = 32;
        this.d[85] = 2;
        this.d[86] = 2;
        this.d[82] = 1;
    }

    private void f(int n, int n2) {
        Image image = this.a[n];
        switch (n) {
            case 69: {
                this.a(9);
                this.i[n2] = 96;
                this.j[n2] = 32;
                return;
            }
            case 67: {
                this.a(9);
                break;
            }
            case 72: {
                this.i[n2] = 32;
                this.j[n2] = 45;
                return;
            }
            default: {
                if (image == null) break;
                this.i[n2] = this.h[n];
                this.j[n2] = image.getHeight();
                return;
            }
        }
        this.i[n2] = 32;
        this.j[n2] = 32;
    }

    private void g(int n, int n2) {
        this.m[n2] = this.g[n];
        this.k[n2] = 0;
    }

    private void d(int n) {
        this.e[n] = 2;
        int n2 = this.l[n];
        switch (this.d[n2]) {
            case 1: {
                return;
            }
            case 2: 
            case 8: 
            case 16: 
            case 32: {
                this.i[n] = 27;
                this.j[n] = 29;
                this.m[n] = 8;
                this.k[n] = 0;
            }
        }
    }

    private void e(int n) {
        this.e[n] = 1;
        int n2 = this.l[n];
        switch (this.d[n2]) {
            case 1: {
                switch (n2) {
                    case 1: 
                    case 2: 
                    case 3: {
                        this.i[n] = 45;
                        this.j[n] = 56;
                        this.m[n] = 6;
                        this.k[n] = 0;
                        break;
                    }
                    case 11: {
                        this.i[n] = 45;
                        this.j[n] = 57;
                        this.m[n] = 2;
                        this.k[n] = 0;
                    }
                }
                this.r();
                return;
            }
            case 2: 
            case 4: {
                this.i[n] = 45;
                this.j[n] = 56;
                this.m[n] = 10;
                this.k[n] = 0;
                return;
            }
            case 8: 
            case 16: 
            case 32: {
                this.i[n] = 27;
                this.j[n] = 29;
                this.m[n] = 6;
                this.k[n] = 0;
            }
        }
    }

    private int a(int n, int n2, int n3, int n4) {
        this.f(n, this.T);
        this.g(n, this.T);
        int n5 = this.i[this.T] >> 5;
        byte by = 0;
        int n6 = n2 + 16 >> 5;
        int n7 = n3 + 16 >> 5;
        int n8 = 0;
        if (n5 <= 0) {
            n5 = 1;
        }
        n2 = n6 << 5;
        n3 = n7 << 5;
        this.l[this.T] = n;
        this.g[this.T] = n2;
        this.h[this.T] = n3;
        this.b[this.T] = false;
        this.a[this.T] = true;
        this.a[this.T] = n4;
        this.e[this.T] = 0;
        ++this.T;
        for (n4 = 0; n4 < n5; ++n4) {
            by = this.c[n7][n6 + n8];
            if (by == 0) {
                this.c[n7][n6 + n8] = this.p = (byte)(this.p + 1);
                by = this.p;
            }
            this.d[by][this.k[by]] = this.T;
            byte by2 = by;
            this.k[by2] = this.k[by2] + 1;
            ++n8;
        }
        if (n == 7) {
            this.as = n2;
            this.au = n3;
        } else if (n == 8) {
            this.at = n2;
            this.av = n3;
        }
        return this.T - 1;
    }

    private void a(int n, int n2, int n3) {
        n = this.c[n2][n];
        n2 = this.k[n];
        byte by = 0;
        if (n2 > 0) {
            for (int i = 0; i < n2; ++i) {
                if (this.l[this.d[n][i] - 1] != n3) {
                    byte by2 = by;
                    by = (byte)(by + 1);
                    this.d[n][by2] = this.d[n][i];
                    continue;
                }
                this.b[this.d[n][i] - 1] = true;
            }
            this.k[n] = by;
        }
    }

    private void b(int n, int n2, int n3) {
        n = this.c[n2][n];
        n2 = this.k[n];
        if (this.k[n] > 0) {
            this.b[this.d[n][n3] - 1] = true;
            while (n3 < n2 - 1) {
                this.d[n][n3] = this.d[n][n3 + 1];
                ++n3;
            }
            int n4 = n;
            this.k[n4] = this.k[n4] - 1;
        }
    }

    private void f(int n) {
        byte by = this.c[this.h[n] >> 5][this.g[n] >> 5];
        int n2 = this.k[by];
        this.b[n] = true;
        if (n2 > 0) {
            for (int i = 0; i < n2; ++i) {
                if (this.d[by][i] - 1 != n) continue;
                for (n = i; n < n2 - 1; ++n) {
                    this.d[by][n] = this.d[by][n + 1];
                }
                byte by2 = by;
                this.k[by2] = this.k[by2] - 1;
                return;
            }
        }
    }

    private void o() {
        if (this.j) {
            if (++this.U > 1) {
                this.j = false;
                return;
            }
        } else if (--this.U < -1) {
            this.j = true;
        }
    }

    /*
     * Enabled aggressive block sorting
     */
    private void a(int n, int n2, boolean bl) {
        int n3;
        int n4;
        int n5;
        int n6;
        int n7;
        int n8;
        Object var4_4 = null;
        var4_4 = null;
        boolean bl2 = false;
        int n9 = 0;
        byte by = 0;
        bl2 = false;
        int n10 = 0;
        this.o();
        bl = false;
        block12: for (int i = 0; i < this.T; ++i) {
            if (this.b[i] || !this.a[i]) continue;
            n8 = this.g[i];
            n7 = this.h[i];
            n6 = this.i[i];
            n5 = this.j[i];
            n10 = this.e[i];
            n4 = n + n8;
            n3 = n2 + n7;
            if (n4 < -n6 || n4 > this.ag || n3 < -12 || n3 > 20 + this.ah) continue;
            n9 = this.l[i];
            Object object = this.a[n9];
            if (object != null) {
                by = this.d[n9];
                if (!bl && this.N > n8 - 32 && this.N < n8 + 32 && this.O >= n7 - 32 && n7 > this.O && !this.c[n9]) {
                    this.c(n, n2);
                    bl = true;
                }
                n8 = n6 * this.c[this.m[i]][this.k[i]];
                n7 = 0;
                switch (by) {
                    case 1: {
                        if (n9 == 6) {
                            n3 += 5 - this.U;
                        } else if (n9 == 9) {
                            n4 += 32;
                        }
                        if (this.e[i] != 1) {
                            this.a((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                            continue block12;
                        }
                        this.a(this.a[4][12], n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                        continue block12;
                    }
                    case 4: {
                        this.a.drawImage(this.a[5][0], n4 + 8, n3 + 22, 0);
                        this.a((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 24) + this.U, n8, 0, n6, n5);
                        continue block12;
                    }
                    case 16: {
                        if (n10 == 1 || n10 == 2) {
                            this.a(this.a[2][9], n4 + 2, n3 + 2 - (this.k[i] << 3), n8, 0, 27, 29);
                            continue block12;
                        }
                        this.a.drawImage(this.a[15], n4 + 1, n3 + 8 + this.U, 0);
                        this.a((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 24) + this.U, n8, 0, n6, n5);
                        continue block12;
                    }
                    case 32: {
                        this.a.drawImage(this.a[5][0], n4 + 8, n3 + 22, 0);
                        if (n10 == 1 || n10 == 2) {
                            this.a(this.a[2][9], n4 + 2, n3 + 2 - (this.k[i] << 3), n8, 0, 27, 29);
                            continue block12;
                        }
                        if (n9 != 72) {
                            this.a((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 16) + this.U, n8, 0, n6, n5);
                            continue block12;
                        }
                        if (i != this.bx) {
                            this.a((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                            continue block12;
                        }
                        n7 = n5 * this.t;
                        if (this.t == 3) {
                            this.a((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, n7 -= n5, n6, n5, 1);
                            continue block12;
                        }
                        this.a((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, n7, n6, n5);
                        continue block12;
                    }
                    case 8: {
                        if (n10 == 3) {
                            n4 += this.g(5) - 2;
                            n3 += this.g(5) - 2;
                        }
                        if (n10 == 1 || n10 == 2) {
                            this.a(this.a[2][9], n4 + 2, n3 + 2 - (this.k[i] << 3), n8, 0, 27, 29);
                        } else if (n9 == 67 || n9 == 69) {
                            n6 = n3;
                            n7 = n4;
                            n8 = i;
                            object = this;
                            n5 = ((a)object).l[n8];
                            n8 = ((a)object).c[((a)object).m[n8]][((a)object).k[n8]];
                            switch (n5) {
                                case 67: {
                                    ((a)object).a.drawImage(((a)object).a[5][0], n7 + 8, n6 + 22, 0);
                                    int n11 = n7 + 16;
                                    int n12 = n6 + 16;
                                    n6 = n8;
                                    n7 = n12;
                                    n8 = n11;
                                    byte[] byArray = ((a)object).b[n6];
                                    byte[] byArray2 = null;
                                    for (n10 = 0; n10 < byArray.length; n10 += 4) {
                                        byArray2 = ((a)object).a[byArray[n10]];
                                        by = byArray2[0];
                                        super.a(((a)object).a[9][by], n8 + byArray[n10 + 1], n7 + byArray[n10 + 2], byArray2[1], byArray2[2], byArray2[3], byArray2[4]);
                                    }
                                    break;
                                }
                                case 69: {
                                    ((a)object).a.drawImage(((a)object).a[5][0], n7 + 40, n6 + 22, 0);
                                    n6 -= 49;
                                    ((a)object).a.drawImage(((a)object).a[n5], n7 += 3, n6 += ((a)object).U, 0);
                                    if (n8 > 0) {
                                        super.a(((a)object).a[9][5], n7 + 32, n6 + 27, 26 * (n8 - 1), 0, 26, 13);
                                    } else {
                                        break;
                                    }
                                }
                            }
                        } else {
                            this.a.drawImage(this.a[5][0], n4 + 8, n3 + 22, 0);
                            this.a((Image)object, n4 + (32 - n6 >> 1), (n3 -= 4) - (n5 - 32), n8, 0, n6, n5);
                        }
                        if (this.l != 2 || !this.i) break;
                        int n13 = this.n[n9];
                        this.a.drawImage(this.a[2][8], n4 - 3 + this.b[this.d & 7], n3 - 24 + this.c[this.d & 7], 0);
                        if (n13 >= 0) {
                            this.b(this.a[2][7], this.n[n9], n4 + 30 + this.b[this.d & 7], n3 - 17 + this.c[this.d & 7]);
                            continue block12;
                        }
                        this.a(this.a[2][7], n4 + 14 + this.b[this.d & 7], n3 - 17 + this.c[this.d & 7], 70, 0, 7, 9);
                        continue block12;
                    }
                    case 2: {
                        if (n10 == 1) {
                            this.a(this.a[4][12], n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                            continue block12;
                        }
                        if (n10 == 2) {
                            this.a(this.a[2][9], n4 + 2, n3 + 2 - (this.k[i] << 3), n8, 0, 27, 29);
                            continue block12;
                        }
                        if (n9 < 26) {
                            this.a.drawImage(this.a[5][0], n4 + 8, n3 + 22, 0);
                            this.a.drawImage(this.a[n9], n4 + (32 - n6 >> 1), n3 - (n5 - 24) + this.U, 0);
                            continue block12;
                        }
                        this.a.drawImage(this.a[n9], n4 + (32 - n6 >> 1), n3 - (n5 - 30), 0);
                    }
                }
                continue;
            }
            object = this.m[this.l[i]];
            int n14 = this.a.stringWidth((String)object) + 8 >> 1;
            this.a.setColor(-1);
            this.a.fillArc(n4, n3, 32, 32, 0, 360);
            this.a.fillRect(n4 -= n14 - 16, n3 += 32 - this.b >> 1, n14 << 1, this.b);
            this.a.setColor(0);
            this.a.drawRect(n4, n3, (n14 << 1) - 1, this.b - 1);
            this.a.drawString(this.m[this.l[i]], n4 + 4, n3, 0);
        }
        this.a.setClip(0, 0, 240, 320);
        if (!bl) {
            this.c(n, n2);
        }
        if (this.l == 5) {
            a a2 = this;
            if (a2.S >= 0) {
                n8 = a2.ae + a2.g[a2.S] + 16;
                n7 = a2.af + a2.h[a2.S] + 32;
                n6 = a2.bV & 7;
                n5 = a2.S[n6];
                if (n5 > 0) {
                    n5 = n5 - 1 << 2;
                    a2.a(a2.a[3][2], n8 - (a2.R[n5 + 2] >> 1), n7 - (a2.R[n5 + 3] >> 1), a2.R[n5], a2.R[n5 + 1], a2.R[n5 + 2], a2.R[n5 + 3]);
                }
            }
        }
        if (this.as >= 0) {
            n4 = n + this.as;
            n3 = n2 + this.au;
            if (n4 >= -32 && n4 <= this.ag && n3 >= -12 && n3 <= 20 + this.ah) {
                this.a.drawImage(this.a[2][1], n4 + 5, n3 - 30 + this.U, 0);
            }
        }
        if (this.at >= 0) {
            n4 = n + this.at;
            n3 = n2 + this.av;
            if (n4 >= -32 && n4 <= this.ag && n3 >= -12 && n3 <= 20 + this.ah) {
                this.a.drawImage(this.a[2][2], n4, n3 - 30 + this.U, 0);
            }
        }
    }

    private void p() {
        int n = 0;
        int n2 = 0;
        if (this.T > 0) {
            for (int i = this.T; i >= 1; --i) {
                n = this.h[0];
                for (int j = 1; j < i; ++j) {
                    if (this.b[j]) continue;
                    n2 = this.h[j];
                    if (n2 < n) {
                        int n3;
                        int n4;
                        int n5 = j;
                        n2 = j - 1;
                        a a2 = this;
                        int n6 = a2.l[n2];
                        int n7 = a2.g[n2];
                        int n8 = a2.h[n2];
                        int n9 = a2.i[n2];
                        int n10 = a2.j[n2];
                        byte by = a2.e[n2];
                        boolean bl = a2.b[n2];
                        boolean bl2 = a2.a[n2];
                        short s = a2.a[n2];
                        int n11 = n9 >> 5;
                        if (n11 <= 0) {
                            n11 = 1;
                        }
                        int n12 = n8 >> 5;
                        int n13 = n7 >> 5;
                        byte by2 = 0;
                        int n14 = 0;
                        block2: for (n4 = 0; n4 < n11; ++n4) {
                            by2 = a2.c[n12][n13 + n4];
                            n14 = a2.k[by2];
                            for (n3 = 0; n3 < n14; ++n3) {
                                if (a2.d[by2][n3] != n2 + 1) continue;
                                a2.d[by2][n3] = n5 + 1;
                                continue block2;
                            }
                        }
                        a2.l[n2] = a2.l[n5];
                        a2.g[n2] = a2.g[n5];
                        a2.h[n2] = a2.h[n5];
                        a2.i[n2] = a2.i[n5];
                        a2.j[n2] = a2.j[n5];
                        a2.e[n2] = a2.e[n5];
                        a2.b[n2] = a2.b[n5];
                        a2.a[n2] = a2.a[n5];
                        a2.a[n2] = a2.a[n5];
                        a2.g(a2.l[n2], n2);
                        a2.l[n5] = n6;
                        a2.g[n5] = n7;
                        a2.h[n5] = n8;
                        a2.i[n5] = n9;
                        a2.j[n5] = n10;
                        a2.e[n5] = by;
                        a2.b[n5] = bl;
                        a2.a[n5] = bl2;
                        a2.a[n5] = s;
                        a2.g(a2.l[n5], n5);
                        n12 = a2.h[n2] >> 5;
                        n13 = a2.g[n2] >> 5;
                        n11 = a2.i[n2] >> 5;
                        if (n11 <= 0) {
                            n11 = 1;
                        }
                        block4: for (n4 = 0; n4 < n11; ++n4) {
                            by2 = a2.c[n12][n13 + n4];
                            n14 = a2.k[by2];
                            for (n3 = 0; n3 < n14; ++n3) {
                                if (a2.d[by2][n3] != n5 + 1) continue;
                                a2.d[by2][n3] = n2 + 1;
                                continue block4;
                            }
                        }
                        n2 = this.h[j];
                    }
                    n = n2;
                }
            }
        }
    }

    private void q() {
        int n = 0;
        if (this.T > 0 && (this.d & 1) != 0) {
            int n2 = this.T;
            block4: while (--n2 >= 0) {
                if (this.b[n2] || !this.a[n2] || (n = this.m[n2]) <= 0) continue;
                if (this.k[n2] < this.c[n].length - 1) {
                    int n3 = n2;
                    this.k[n3] = this.k[n3] + 1;
                    continue;
                }
                switch (this.e[n2]) {
                    case 1: {
                        this.f(n2);
                        continue block4;
                    }
                    case 2: {
                        this.f(this.l[n2], n2);
                        this.g(this.l[n2], n2);
                        this.e[n2] = 0;
                        continue block4;
                    }
                }
                this.k[n2] = 0;
            }
        }
    }

    private void a(boolean n) {
        int n2 = 0;
        int n3 = 0;
        if (--this.X < -154) {
            this.X = 0;
        }
        boolean bl = false;
        n3 = n == 0 ? 0 : this.W - 320 + 6;
        for (n2 = this.X; n2 < 240; n2 += 77) {
            for (n = n3; n > -320; n -= 320) {
                if (bl) {
                    this.a(this.a[1][0], n2, n, 2);
                } else {
                    this.a.drawImage(this.a[1][0], n2, n, 0);
                }
                bl = !bl;
            }
            bl = false;
        }
    }

    private void r() {
        this.d = null;
        if (this.f[1]) {
            Image image = Image.createImage((int)(this.Y << 2), (int)(this.Z << 2));
            Graphics graphics = image.getGraphics();
            int n = 0;
            int n2 = 0;
            graphics.setColor(13097429);
            graphics.fillRect(0, 0, this.Y << 2, this.Z << 2);
            graphics.setColor(7509153);
            byte by = 0;
            int n3 = 0;
            while (n3 < this.Z) {
                int n4 = 0;
                while (n4 < this.Y) {
                    by = this.c[n3][n4];
                    if (this.a[n3][n4]) {
                        graphics.setColor(7509153);
                        graphics.fillRect(n, n2, 4, 4);
                    }
                    if (by > 0 && this.k[by] > 0 && !this.b[(by = this.d[by][0]) - 1] && this.e[by - 1] != 1) {
                        switch (this.l[by - 1]) {
                            case 1: {
                                graphics.setColor(0xEBEB4E);
                                graphics.fillRect(n, n2, 3, 3);
                                graphics.setColor(9794048);
                                graphics.drawLine(n + 3, n2, n + 3, n2 + 3);
                                graphics.drawLine(n, n2 + 3, n + 3, n2 + 3);
                                break;
                            }
                            case 3: {
                                graphics.setColor(4767984);
                                graphics.fillRect(n, n2, 3, 3);
                                graphics.setColor(549016);
                                graphics.drawLine(n + 3, n2, n + 3, n2 + 3);
                                graphics.drawLine(n, n2 + 3, n + 3, n2 + 3);
                                break;
                            }
                            case 2: {
                                graphics.setColor(16273480);
                                graphics.fillRect(n, n2, 4, 4);
                                graphics.setColor(0x800000);
                                graphics.drawLine(n + 3, n2, n + 3, n2 + 3);
                                graphics.drawLine(n, n2 + 3, n + 3, n2 + 3);
                                break;
                            }
                            case 7: {
                                a.a(this.a[8][12], graphics, n - 1, n2 - 1, 5, 0, 5, 5);
                                break;
                            }
                            case 8: {
                                a.a(this.a[8][12], graphics, n - 1, n2 - 1, 0, 0, 5, 5);
                            }
                        }
                    }
                    ++n4;
                    n += 4;
                }
                ++n3;
                n2 += 4;
                n = 0;
            }
            this.d = image = a.a(image, 170);
        }
    }

    private static short a(InputStream inputStream) throws IOException {
        return (short)(inputStream.read() & 0xFF | inputStream.read() << 8 & 0xFF00);
    }

    private void s() {
        this.c = new byte[this.Z][this.Y];
        this.k = new byte[128];
        this.d = new byte[128][16];
        this.p = 0;
    }

    private boolean b(int n, int n2) {
        if (n >= 0 && n < this.Y && n2 >= 0 && n2 < this.Z) {
            return this.a[n2][n];
        }
        return false;
    }

    private void t() {
        int n = 0;
        byte by = 0;
        int n2 = this.Y;
        int n3 = this.Z;
        int n4 = 0;
        while (n4 < n3) {
            int n5 = 0;
            while (n5 < n2) {
                by = this.i[n];
                this.a[n4][n5] = by < this.e.length ? this.e[by] : false;
                ++n5;
                n += 2;
            }
            ++n4;
            n += n2 << 1;
        }
    }

    private boolean c(int n, int n2) {
        if (n >= 0 && n < this.Y && n2 >= 0 && n2 < this.Z) {
            return this.k[this.c[n2][n]] > 0;
        }
        return false;
    }

    private void h(int n, int n2) {
        int n3;
        int n4;
        if (this.l == 4) {
            a a2 = this;
            ++a2.ad;
            if (a2.ad == 18) {
                a a3 = a2;
                n4 = 0;
                n3 = a3.aI;
                while (--n3 >= 0) {
                    n4 = a3.d[n3];
                    a3.m[n4] = 6;
                    a3.e[n4] = 1;
                }
            } else if (a2.ad >= 24) {
                a2.ad = 0;
                a a4 = a2;
                for (n4 = 0; n4 < a4.aI; ++n4) {
                    a4.a((int)a4.b[n4], (int)a4.c[n4], 4);
                }
                a4.aI = 0;
                a2.ac = 0;
                a2.l = 0;
            }
            a2.ac = a2.ac == -2 ? 2 : -2;
            n2 = 20 + this.ac;
        }
        n = this.ae + (this.ai << 4);
        n2 = this.af + n2 + (this.ak << 4);
        n4 = 0;
        int n5 = n;
        int n6 = n2;
        n2 = n3 = this.ai + (this.ak * this.Y << 1);
        int n7 = this.ak;
        while (n7 < this.al) {
            int n8 = this.ai;
            while (n8 < this.aj) {
                n4 = this.i[n2];
                if (n4 > 0) {
                    int n9 = (n4 & 7) << 4;
                    n4 = n4 >> 3 << 4;
                    this.a(this.a[2][0], n5, n6, n9, n4, 16, 16, this.j[n2]);
                }
                ++n8;
                ++n2;
                n5 += 16;
            }
            ++n7;
            n6 += 16;
            n5 = n;
            n2 = n3 += this.Y << 1;
        }
        this.a.setClip(0, 0, 240, 320);
    }

    private void i(int n, int n2) {
        if (!this.k) {
            if (n > 64) {
                n = 64;
            } else if (n < -((this.Y + 2 << 5) - this.ag)) {
                n = -((this.Y + 2 << 5) - this.ag);
            }
            this.ae = n;
            if (n < 0) {
                this.ai = -n >> 4;
                this.aj = this.ai + (this.ag >> 4) + 1;
            } else {
                this.ai = 0;
                this.aj = (this.ag - n >> 4) + 1;
            }
            if (this.aj > this.Y << 1) {
                this.aj = this.Y << 1;
            }
            if (n2 > 64) {
                n2 = 64;
            } else if (n2 < -((this.Z + 2 << 5) - this.ah)) {
                n2 = -((this.Z + 2 << 5) - this.ah);
            }
        } else {
            this.ae = this.ag - this.aa >> 1;
            this.ai = 0;
            this.aj = this.Y << 1;
        }
        if (!this.l) {
            this.af = n2;
            if (n2 < 0) {
                this.ak = -n2 >> 4;
                this.al = this.ak + (this.ah >> 4) + 2;
            } else {
                this.ak = 0;
                this.al = (this.ah - n2 >> 4) + 1;
            }
            if (this.al > this.Z << 1) {
                this.al = this.Z << 1;
                return;
            }
        } else {
            this.af = this.ah - this.ab >> 1;
            this.ak = 0;
            this.al = this.Z << 1;
        }
    }

    private int a(boolean bl) {
        if (bl) {
            for (int i = 0; i < this.T; ++i) {
                int n = this.a[i] >> 9;
                if (this.l[i] != 7 || n != 0) continue;
                return i;
            }
        } else {
            for (int i = 0; i < this.T; ++i) {
                int n = this.a[i] >> 9;
                if (this.l[i] != 8 || n != 0) continue;
                return i;
            }
        }
        return -1;
    }

    /*
     * Unable to fully structure code
     */
    private boolean a(int var1_1, boolean var2_2, boolean var3_3) {
        var4_4 = false;
        if (!var3_3) ** GOTO lbl-1000
        if (var1_1 < this.an) {
            this.a((byte)0, this.a[0], (byte)0, (byte)0);
            var4_4 = false;
        } else if (var1_1 > this.ao) {
            var4_4 = false;
            this.a((byte)0, this.a[1], (byte)0, (byte)0);
        } else lbl-1000:
        // 2 sources

        {
            var4_4 = true;
        }
        if (var4_4) {
            var4_4 = false;
            if (var1_1 < 0) {
                this.a((byte)0, this.a[2], (byte)0, (byte)0);
            } else if (var1_1 > this.aq) {
                this.a((byte)0, this.a[3], (byte)0, (byte)0);
            } else {
                var4_4 = true;
                this.q = true;
                if (var1_1 < this.an) {
                    this.an = var1_1;
                } else if (var1_1 > this.ao) {
                    this.ao = var1_1;
                    if (this.ao < 51 && this.ao > this.bY) {
                        this.bY = this.ao;
                    }
                }
                this.r(this.am);
                this.x = (byte)var1_1;
                this.s = var2_2;
            }
        }
        return var4_4;
    }

    private void u() {
        this.a((byte)3, true);
        this.u(6);
        this.u(8);
        this.u(5);
        this.u(13);
        this.u(9);
        this.u(10);
        this.u(2);
        this.u(3);
        this.u(12);
        this.u(11);
    }

    private void v() {
        int n;
        this.j = 0;
        this.k = 0;
        if (this.am == 1) {
            this.a(6, 11);
        } else if (this.am == 51) {
            this.a(1, 11);
        } else if (this.am == 50) {
            this.a(6, 6);
        } else {
            n = this.a(false);
            if (n >= 0) {
                this.b(this.g[n] >> 5, this.h[n] >> 5);
            }
        }
        n = this.bj;
        while (--n >= 0) {
            this.g[n] = false;
        }
        this.aL = 0;
        this.aQ = 0;
        this.ar = 0;
        this.aR = 0;
        this.A = 0;
        this.g(this.H[this.A]);
    }

    private void g(int n) {
        if (this.t == null) {
            this.t = new int[this.q.length];
        }
        int n2 = this.t.length;
        while (--n2 >= 0) {
            this.t[n2] = this.q[n2] * n;
        }
        if (this.u == null) {
            this.u = new int[this.r.length];
        }
        n2 = this.u.length;
        while (--n2 >= 0) {
            this.u[n2] = this.r[n2] * n;
        }
        if (this.v == null) {
            this.v = new int[this.p.length];
        }
        n2 = this.v.length;
        while (--n2 >= 0) {
            this.v[n2] = this.p[n2] * n;
        }
    }

    private static int a(int n, int n2) {
        --n2;
        n2 /= 10;
        return n * ++n2;
    }

    private static int b(int n) {
        int n2 = 20;
        for (int i = 1; i < n; ++i) {
            n2 += 20 * i;
        }
        return n2;
    }

    private void h(int n) {
        int n2 = this.a.stringWidth(this.f[n]) + 80;
        this.l[this.aF++] = n;
        if (this.aF == 1) {
            this.aH = 32 + this.aD;
            this.aG = n2;
        } else {
            this.aH += this.aD;
            if (this.aG < n2) {
                this.aG = n2;
            }
        }
        switch (n) {
            case 0: {
                this.f[this.aF - 1] = this.D;
            }
        }
    }

    private boolean b(int n) {
        int n2 = 0;
        this.aI = 0;
        for (int i = 0; i < this.T; ++i) {
            if (this.b[i] || n != this.a[i]) continue;
            n2 = this.l[i];
            if (n2 == 5) {
                return false;
            }
            if (n2 != 4) continue;
            this.d[this.aI] = i;
            this.b[this.aI] = this.g[i] >> 5;
            this.c[this.aI] = this.h[i] >> 5;
            ++this.aI;
        }
        return this.aI > 0;
    }

    private boolean a(boolean bl) {
        short s = this.a[this.aJ];
        byte by = (byte)(s >>> 8);
        s = (byte)s;
        int n = this.l[this.aJ] == 77 ? 0 : 1;
        boolean bl2 = true;
        boolean bl3 = false;
        if (s <= 0) {
            return false;
        }
        if (bl) {
            if ((by & 2) != 0) {
                block0 : switch (n) {
                    case 0: {
                        switch (s) {
                            case 2: {
                                this.k(13);
                                break;
                            }
                            case 1: 
                            case 15: 
                            case 21: {
                                this.c(1000, this.N, this.O);
                                break;
                            }
                            case 6: {
                                this.k(19);
                            }
                        }
                        break;
                    }
                    case 1: {
                        switch (s) {
                            case 1: {
                                this.K += this.K * 3 / 100;
                                this.L += this.L * 3 / 100;
                                break block0;
                            }
                            case 3: {
                                if (this.c(50)) {
                                    ++this.aO;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 4: {
                                if (this.c(50)) {
                                    this.aN += 5;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 5: {
                                if (this.c(1000)) {
                                    ++this.aN;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 6: {
                                if (this.c(800)) {
                                    ++this.aP;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 7: {
                                if (this.c(200)) {
                                    ++this.aO;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 8: {
                                if (this.aN > 0) {
                                    --this.aN;
                                    this.aQ += 100;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u9ec4\u94a5\u5319", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 9: {
                                if (this.c(1000)) {
                                    ++this.aN;
                                    ++this.aO;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 10: {
                                if (this.c(200)) {
                                    this.aN += 3;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 11: {
                                if (this.c(2000)) {
                                    this.aO += 3;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 12: {
                                if (this.c(1000)) {
                                    this.J += 2000;
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 13: {
                                if (this.c(4000)) {
                                    this.j(18);
                                    break block0;
                                }
                                this.a((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                            }
                        }
                    }
                }
                if (bl2) {
                    if ((by & 4) == 0) {
                        int n2 = this.aJ;
                        this.a[n2] = this.a[n2] ^ 0x200;
                        if ((by & 1) != 0) {
                            this.j(n, s - 1);
                            if (n == 0) {
                                this.a((byte)4, this.g[s - 1], (byte)0, (byte)0);
                            } else {
                                this.a((byte)4, this.j[s - 1], (byte)0, (byte)0);
                            }
                            bl3 = true;
                        } else {
                            this.e(this.aJ);
                        }
                    }
                } else {
                    bl3 = true;
                }
            } else if ((by & 1) != 0) {
                int n3 = this.aJ;
                this.a[n3] = this.a[n3] ^ 0x100;
                if ((by & 4) == 0) {
                    this.e(this.aJ);
                }
            }
        } else if ((by & 2) == 0 && (by & 1) != 0) {
            int n4 = this.aJ;
            this.a[n4] = this.a[n4] ^ 0x100;
            if ((by & 4) == 0) {
                this.e(this.aJ);
            }
        }
        return bl3;
    }

    private void j(int n, int n2) {
        if (this.aL < this.aK) {
            this.o[this.aL] = n;
            this.p[this.aL] = n2;
            ++this.aL;
        }
    }

    private void i(int n) {
        byte by = 0;
        byte by2 = 0;
        if (this.aL > 0) {
            this.aM = n < 0 ? this.aL - 1 : (n >= this.aL ? 0 : n);
            by = this.o[this.aM];
            by2 = this.p[this.aM];
            if (by == 0) {
                this.a((byte)5, this.g[by2], (byte)0, (byte)3);
            } else {
                this.a((byte)5, this.j[by2], (byte)0, (byte)3);
            }
            this.e = "" + (this.aM + 1) + " / " + this.aL;
            return;
        }
        this.a((byte)0, "\u6ca1\u6709\u8bb0\u5f55", (byte)0, (byte)3);
        this.a = (byte)10;
    }

    private boolean c(int n) {
        boolean bl = true;
        if (this.aQ >= n) {
            this.aQ -= n;
        } else {
            bl = false;
        }
        return bl;
    }

    private void c(int n, int n2, int n3) {
        this.aQ += n;
        if (n > 0) {
            this.a((byte)4, n, n2, n3);
        }
    }

    private void j(int n) {
        int n2 = 0;
        n2 = this.a(n, 0, 12);
        switch (n) {
            case 33: 
            case 34: 
            case 35: 
            case 36: 
            case 37: 
            case 38: 
            case 39: 
            case 40: 
            case 79: 
            case 80: {
                if (this.a(n2, false)) {
                    this.h = (byte)n;
                    this.a((byte)1, this.l[n2], (byte)0, (byte)3);
                    return;
                }
                this.a((byte)0, this.a[5], (byte)0, (byte)3);
                return;
            }
            case 15: {
                this.m = true;
            }
            case 13: 
            case 14: 
            case 16: 
            case 17: 
            case 18: 
            case 19: 
            case 20: 
            case 21: 
            case 22: 
            case 23: 
            case 24: 
            case 25: 
            case 85: 
            case 86: {
                this.k(n);
                return;
            }
            case 26: {
                ++this.aN;
                return;
            }
            case 27: {
                ++this.aP;
                return;
            }
            case 28: {
                ++this.aO;
                return;
            }
            case 29: {
                if (this.am <= 10) {
                    ++this.K;
                    return;
                }
                this.K += this.a();
                return;
            }
            case 30: {
                if (this.am <= 10) {
                    ++this.L;
                    return;
                }
                this.L += this.a();
                return;
            }
            case 31: {
                n2 = this.a();
                this.c(50 * n2);
                return;
            }
            case 32: {
                n2 = this.a();
                this.c(200 * n2);
            }
        }
    }

    private int a() {
        int n = this.am - 1;
        if ((n /= 10) < 0) {
            n = 0;
        }
        if (this.am > 50) {
            n = 0;
        }
        return n + 1;
    }

    private int c(int n) {
        int n2 = -1;
        for (int i = 0; i < this.aR; ++i) {
            if (this.r[i] != n) continue;
            n2 = i;
            break;
        }
        return n2;
    }

    private boolean a(int n, boolean bl) {
        boolean bl2 = true;
        int n2 = 0;
        if (n < 6) {
            n2 = this.a((int)this.j, 0, 6);
            if (n2 < n || bl) {
                this.K -= this.o[n2];
                this.K += this.o[n];
                this.j = this.t[n];
            } else {
                bl2 = false;
            }
        } else {
            n2 = this.a((int)this.k, 6, 12);
            if (n2 < n || bl) {
                this.L -= this.o[n2];
                this.L += this.o[n];
                this.k = this.t[n];
            } else {
                bl2 = false;
            }
        }
        return bl2;
    }

    private int a(int n, int n2, int n3) {
        while (--n3 >= n2) {
            if (this.t[n3] != n) continue;
            return n3;
        }
        return 0;
    }

    private static int d(int n) {
        int n2 = 0;
        switch (n) {
            case 85: {
                n2 = 20;
                break;
            }
            case 86: {
                n2 = 21;
                break;
            }
            default: {
                n2 = n - 13;
            }
        }
        return n2;
    }

    private void k(int n) {
        int n2 = this.c(n);
        int n3 = a.d(n);
        if (n2 < 0) {
            this.r[this.aR] = n;
            this.s[this.aR] = this.q[n3];
            ++this.aR;
        } else {
            int n4 = n2;
            this.s[n4] = this.s[n4] + this.q[n3];
        }
        this.h = (byte)n;
        this.a((byte)1, this.k[n3], (byte)0, (byte)3);
    }

    private boolean b(byte by) {
        boolean bl = false;
        switch (by) {
            case 26: {
                if (this.aN <= 0) break;
                this.a((byte)1, 2, this.N, this.O);
                --this.aN;
                bl = true;
                break;
            }
            case 28: {
                if (this.aO <= 0) break;
                this.a((byte)1, 0, this.N, this.O);
                --this.aO;
                bl = true;
                break;
            }
            case 27: {
                if (this.aP <= 0) break;
                this.a((byte)1, 1, this.N, this.O);
                --this.aP;
                bl = true;
            }
        }
        return bl;
    }

    private void l(int n) {
        if (this.s[n] > 0) {
            if (this.c(this.r[n])) {
                int n2 = n;
                this.s[n2] = (byte)(this.s[n2] - 1);
                if (this.s[n2] == 0) {
                    while (n < this.aR - 1) {
                        this.r[n] = this.r[n + 1];
                        this.s[n] = this.s[n + 1];
                        ++n;
                    }
                    if (this.aR > 0) {
                        --this.aR;
                    }
                    return;
                }
            }
        } else {
            this.c(this.r[n]);
        }
    }

    private boolean c(byte by) {
        int n = 0;
        n = 0;
        switch (by) {
            case 13: {
                this.a = (byte)5;
                this.a();
                break;
            }
            case 14: {
                this.a = (byte)12;
                this.a();
                break;
            }
            case 15: {
                break;
            }
            case 16: {
                if (!this.d((byte)10)) {
                    this.a((byte)0, "\u4f60\u5fc5\u987b\u9762\u5bf9\u4e09\u6627\u771f\u706b\u518d\u4f7f\u7528\u5b83\u3002", (byte)0, (byte)0);
                    break;
                }
                this.a = (byte)3;
                n = 1;
                break;
            }
            case 17: {
                if (!this.d((byte)11)) {
                    this.a((byte)0, "\u4f60\u5fc5\u987b\u9762\u5bf9\u4e00\u5835\u5899\u4f7f\u7528", (byte)0, (byte)0);
                    break;
                }
                this.a = (byte)3;
                n = 1;
                break;
            }
            case 18: {
                by = (byte)this.T;
                while ((by = (byte)(by - 1)) >= 0) {
                    if (this.l[by] != 11) continue;
                    this.e(by);
                }
                this.a = (byte)3;
                n = 1;
                break;
            }
            case 19: {
                n = (this.K + this.L) * 74 / 10;
                this.c(n);
                this.a((byte)0, "\u589e\u52a0\u4e86" + n + "\u8840\u91cf", (byte)0, (byte)0);
                n = 1;
                break;
            }
            case 20: {
                if (this.am == 40) {
                    this.a((byte)0, "\u672c\u5c42\u4e0d\u80fd\u76f4\u63a5\u77ac\u79fb\u3002", (byte)0, (byte)0);
                    break;
                }
                a a2 = this;
                int n2 = a2.Y - 1 - a2.P;
                int n3 = a2.Z - 1 - a2.Q;
                boolean bl = a2.a(n2, n3);
                if (bl) {
                    a2.a(n2, n3);
                    a2.i((a2.ag - 32 >> 1) - a2.N, (a2.ah - 32 >> 1) - a2.O);
                    a2.k();
                }
                if (bl) {
                    this.a = (byte)3;
                    n = 1;
                    break;
                }
                if (this.f) break;
                this.a((byte)0, "\u65e0\u6cd5\u79fb\u52a8\u5230\u8be5\u4f4d\u7f6e", (byte)0, (byte)0);
                break;
            }
            case 21: {
                n = this.a(this.am + 1, false, false);
                this.g = 0;
                this.a = (byte)3;
                break;
            }
            case 22: {
                n = this.a(this.am - 1, true, false);
                this.g = 0;
                this.a = (byte)3;
                break;
            }
            case 85: {
                by = (byte)this.T;
                while ((by = (byte)(by - 1)) >= 0) {
                    if (this.l[by] != 1) continue;
                    this.e(by);
                }
                this.a = (byte)3;
                n = 1;
                break;
            }
            case 86: {
                a a3 = this;
                int n4 = a3.N >> 5;
                int n5 = a3.O >> 5;
                a3.k(n4, n5 - 1);
                a3.k(n4, n5 + 1);
                a3.k(n4 - 1, n5);
                a3.k(n4 + 1, n5);
                this.a = (byte)3;
                n = 1;
            }
        }
        return n != 0;
    }

    private boolean d(byte by) {
        int n = this.N >> 5;
        int n2 = this.O >> 5;
        int n3 = this.b(n, n2 - 1, by);
        int n4 = this.b(n, n2 + 1, by);
        int n5 = this.b(n - 1, n2, by);
        by = (byte)this.b(n + 1, n2, by);
        n = 0;
        if (n3 >= 0) {
            this.e(n3);
            ++n;
        }
        if (n4 >= 0) {
            this.e(n4);
            ++n;
        }
        if (n5 >= 0) {
            this.e(n5);
            ++n;
        }
        if (by >= 0) {
            this.e(by);
            ++n;
        }
        return n > 0;
    }

    private void k(int n, int n2) {
        try {
            byte by = this.c[n2][n];
            int n3 = this.k[by];
            int n4 = 0;
            int n5 = 0;
            n4 = 0;
            if (n3 > 0) {
                for (int i = 0; i < n3; ++i) {
                    n4 = this.d[by][i] - 1;
                    n5 = this.l[n4];
                    if (n5 == 5) {
                        n4 = this.a[n4];
                        this.b(n, n2, i);
                        if (!this.b(n4)) continue;
                        this.l = (byte)4;
                        continue;
                    }
                    if (n5 >= 67 || this.d[n5] != 8 || this.e[n4] == 1) continue;
                    this.e(n4);
                    if (this.c(25) >= 0) {
                        this.c(this.s[n5 - 41] << 1, 0, 0);
                        continue;
                    }
                    this.c(this.s[n5 - 41], 0, 0);
                }
            }
            return;
        }
        catch (Exception exception) {
            Exception exception2 = exception;
            exception.printStackTrace();
            return;
        }
    }

    private void w() {
        int n;
        int n2;
        int n3;
        a a2;
        int n4;
        this.w = null;
        this.w = new int[128];
        this.u = null;
        this.u = new byte[128];
        this.be = 208;
        this.bb = 0;
        block0: for (int i = 0; i < this.T; ++i) {
            if (this.b[i] || this.d[this.l[i]] != 8) continue;
            n4 = this.l[i];
            a2 = this;
            n3 = a2.bb;
            while (--n3 >= 0) {
                if (n4 != a2.u[n3]) continue;
                continue block0;
            }
            n3 = a2.v[n4 - 41];
            n2 = a2.t[n4 - 41];
            n = a2.u[n4 - 41];
            a2.w[a2.bb] = a.a(a2.f(n4), a2.L, n3, n2, n, true);
            a2.u[a2.bb] = n4;
            ++a2.bb;
        }
        a2 = this;
        n4 = 0;
        n3 = 0;
        for (n2 = 0; n2 < a2.bb; ++n2) {
            n = a2.bb;
            while (--n > n2) {
                if (a2.w[n] < 0 || a2.w[n] >= a2.w[n - 1]) continue;
                n4 = a2.u[n];
                a2.u[n] = a2.u[n - 1];
                a2.u[n - 1] = n4;
                n3 = a2.w[n];
                a2.w[n] = a2.w[n - 1];
                a2.w[n - 1] = n3;
            }
        }
        this.bc = 258 / (this.b + 4 + 36);
        if (this.bc > this.bb) {
            this.bc = this.bb;
        }
        this.bd = (this.b + 4 + 36) * this.bc + 32;
        this.bf = 0;
        this.bg = this.bc;
        if (this.bg > this.bb) {
            this.bg = this.bb;
        }
    }

    private void m(int n) {
        int n2 = this.bb - this.bc;
        if (n2 < 0) {
            n2 = 0;
        }
        if (n >= 0 && n <= n2) {
            this.bf = n;
            this.bg = this.bf + this.bc;
            if (this.bg > this.bb) {
                this.bg = this.bb;
            }
        }
    }

    private void a(int n, int n2, byte by) {
        this.a.fillTriangle(n, n2 - 6, n, n2 + 6, n - 6, n2);
    }

    private boolean d(int n) {
        this.B = false;
        this.a = (byte)11;
        this.bh = n;
        this.bi = 0;
        this.e(0);
        this.q = 0;
        this.f = this.n[this.bh];
        this.c = null;
        this.bm = 0;
        switch (n) {
            case 31: {
                this.r(this.am);
                this.t(24);
                n = this.T;
                while (--n >= 0) {
                    if (this.l[n] != 4) continue;
                    this.b[n] = true;
                }
                this.r(24);
                this.s();
                this.t(this.am);
            }
        }
        return false;
    }

    private boolean b(int n, boolean bl) {
        if (n >= this.bj) {
            return false;
        }
        boolean bl2 = !this.g[n];
        int n2 = 0;
        switch (n) {
            case 0: {
                if (!bl2) break;
                this.g[19] = true;
                break;
            }
            case 1: {
                bl2 &= this.g[0] & !bl;
                break;
            }
            case 2: {
                bl2 &= this.g[1] & !bl;
                break;
            }
            case 3: {
                bl2 &= bl;
                break;
            }
            case 4: {
                bl2 &= this.g[3] & bl;
                break;
            }
            case 7: {
                if (!(bl2 &= bl && !this.d(23, 12))) break;
                this.g[32] = true;
                break;
            }
            case 8: 
            case 9: 
            case 11: 
            case 12: 
            case 19: 
            case 30: 
            case 41: 
            case 42: 
            case 43: 
            case 44: 
            case 45: 
            case 48: 
            case 49: 
            case 50: 
            case 52: 
            case 53: 
            case 55: 
            case 56: {
                bl2 &= bl;
                break;
            }
            case 46: {
                bl2 &= bl & !this.g[47];
                break;
            }
            case 47: {
                bl2 &= bl & !this.g[46];
                break;
            }
            case 13: {
                break;
            }
            case 15: {
                if (this.d(this.am, 5)) {
                    bl2 = false;
                    break;
                }
                bl2 &= bl;
                break;
            }
            case 18: {
                if (!(bl2 &= bl) || this.b(2, 2, 1) >= 0 && this.b(2, 4, 1) >= 0 && this.b(2, 6, 1) >= 0 && this.b(6, 2, 1) >= 0 && this.b(4, 6, 1) >= 0 && this.b(6, 6, 1) >= 0 && this.b(4, 2, 1) < 0 && this.b(6, 4, 1) < 0) break;
                bl2 = false;
                this.g[18] = true;
                break;
            }
            case 25: {
                bl2 &= bl & this.g[13] & this.b(2, 2, 60) < 0 & this.b(10, 2, 60) < 0;
                break;
            }
            case 26: {
                if (this.b(11, 4, 84) >= 0) break;
                bl2 = false;
                break;
            }
            case 31: {
                bl2 &= bl;
                bl2 &= this.c(16) >= 0;
                break;
            }
            case 32: {
                if (this.g[7]) {
                    bl2 = false;
                    break;
                }
                bl2 &= !bl & this.d(23, 12);
                break;
            }
            case 33: {
                bl2 &= this.g[7];
                break;
            }
            case 34: {
                if (!(bl2 &= bl & !this.g[10])) break;
                n2 = this.a(69, true);
                bl2 &= n2 < 0 || n2 >= this.J;
                break;
            }
            case 24: {
                if (!(bl2 &= !bl)) break;
                bl2 = true & 0 < this.J;
                break;
            }
            case 16: {
                bl2 &= this.A == 0;
                break;
            }
            case 66: {
                bl2 = false;
                break;
            }
            case 67: {
                bl2 &= this.A > 0;
                break;
            }
            case 27: {
                bl2 &= bl & this.g[0];
            }
        }
        return bl2;
    }

    private void n(int n) {
        byte by = 0;
        if (n <= this.br) {
            this.bi = n;
            this.r = this.bi < 0 ? (byte)-1 : this.v[this.bi];
            this.bp = this.e(this.r);
            by = this.r;
            if (by < 0) {
                this.d = this.c = "?: \\cF8F8F8" + this.o[this.bi];
                this.H = 11;
            } else {
                this.d = this.c = this.m[by] + ": \\cF8F8F8" + this.o[this.bi];
                this.H = this.m[by].length() + 10;
            }
            this.a(this.c, 129, (this.I << 1) + 12, this.H);
            return;
        }
        this.q = 0;
        this.H = 0;
    }

    private boolean a() {
        return this.bt == this.bv && this.bu == this.bw;
    }

    private void a(String object, int n) {
        Object var3_3 = null;
        boolean bl = false;
        bl = false;
        int n2 = 0;
        int n3 = 0;
        int n4 = 0;
        if (n + 3 <= ((String)object).length()) {
            String string = ((String)object).substring(n, n + 3);
            n += 3;
            if (string.equals("TAK")) {
                this.bq = this.a((String)object, n + 1, "_");
                n = this.bs;
                this.br = this.a((String)object, n + 1, " ");
                this.bm = this.bs + 1;
                this.q = 1;
                this.n(this.bq);
            } else if (string.equals("MOV")) {
                int n5 = this.a((String)object, n + 1, "_");
                n = this.bs;
                if (n5 > 0) {
                    n3 = this.a((String)object, n + 1, "_");
                    n = this.bs;
                    n4 = this.a((String)object, n + 1, "_");
                    n = this.bs;
                }
                this.bk = this.a((String)object, n + 1, "_");
                n = this.bs;
                this.bl = this.a((String)object, n + 1, " ");
                this.bm = this.bs + 1;
                if (n5 > 0) {
                    this.bx = this.b(n3, n4, n5);
                    if (this.bx >= 0) {
                        this.s = 0;
                        object = this;
                        if (((a)object).l[((a)object).bx] == 72) {
                            ((a)object).m[((a)object).bx] = 1;
                        }
                        ((a)object).bz = ((a)object).g[((a)object).bx] >> 5;
                        ((a)object).bA = ((a)object).h[((a)object).bx] >> 5;
                        super.a(((a)object).bz, ((a)object).bA, ((a)object).bk, ((a)object).bl);
                        this.q = (byte)2;
                    } else {
                        this.a(this.f, this.bm);
                    }
                } else if (this.a(this.P, this.Q, this.bk, this.bl)) {
                    this.B = true;
                    this.q = (byte)3;
                    this.l = 0;
                    this.e(0);
                }
            } else if (string.equals("GUT")) {
                n2 = this.a((String)object, n + 1, " ");
                this.g[this.bh] = true;
                this.d(n2);
            } else if (string.equals("DES")) {
                int n6 = this.a((String)object, n + 1, "_");
                n = this.bs;
                if (n6 < 0) {
                    this.a((String)object, n + 1, " ");
                    n2 = -n6;
                    object = this;
                    n6 = ((a)object).T;
                    while (--n6 >= 0) {
                        if (((a)object).l[n6] != n2) continue;
                        super.e(n6);
                    }
                } else {
                    n3 = this.a((String)object, n + 1, "_");
                    n6 = this.b(n3, n4 = this.a((String)object, (n = this.bs) + 1, " "), n6);
                    if (n6 >= 0) {
                        this.e(n6);
                    }
                }
                this.bm = this.bs + 1;
            } else if (string.equals("SWD")) {
                object = this;
                int n7 = ((a)object).T;
                while (--n7 >= 0) {
                    if (((a)object).b[n7] || (n2 = ((a)object).l[n7]) != 81) continue;
                    ((a)object).a[n7] = true;
                    ((a)object).l[n7] = 4;
                    super.f(4, n7);
                    ((a)object).e[n7] = 2;
                    ((a)object).m[n7] = 8;
                }
                this.bm = n + 1;
            } else if (string.equals("MVS")) {
                int n8;
                object = this;
                this.e = null;
                ((a)object).e = new short[32];
                n2 = 0;
                int n9 = 0;
                n3 = 0;
                n4 = ((a)object).T;
                block22: while (--n4 >= 0) {
                    n2 = ((a)object).l[n4];
                    if (n2 != 82) continue;
                    n9 = ((a)object).a[n4] & 0xFF;
                    for (n8 = 0; n8 < 32; n8 += 2) {
                        n3 = ((a)object).e[n8] - 1;
                        if (n3 > 0) {
                            if (n9 != (((a)object).a[n3] & 0xFF)) continue;
                            ((a)object).e[n8 + 1] = n4 + 1;
                            continue block22;
                        }
                        ((a)object).e[n8] = n4 + 1;
                        continue block22;
                    }
                }
                for (int i = 0; i < 32; i += 2) {
                    int n10;
                    int n11;
                    n3 = ((a)object).e[i];
                    int n12 = ((a)object).e[i + 1];
                    if (n3 <= 0) break;
                    if (n12 <= 0) continue;
                    --n12;
                    n8 = ((a)object).g[--n3] >> 5;
                    n4 = ((a)object).h[n3] >> 5;
                    n9 = ((a)object).c[n4][n8];
                    int n13 = ((a)object).k[n9];
                    for (n11 = 0; n11 < n13; ++n11) {
                        n10 = ((a)object).d[n9][n11] - 1;
                        n2 = ((a)object).l[n10];
                        if (((a)object).d[n2] != 8) continue;
                        super.a(((a)object).g[n12] >> 5, ((a)object).h[n12] >> 5, 82);
                        super.a(n8, n4, 82);
                        super.e(n10, ((a)object).g[n12] >> 5, ((a)object).h[n12] >> 5);
                    }
                    n9 = n3;
                    n3 = n12;
                    n12 = n9;
                    n8 = ((a)object).g[n3] >> 5;
                    n4 = ((a)object).h[n3] >> 5;
                    n9 = ((a)object).c[n4][n8];
                    n13 = ((a)object).k[n9];
                    for (n11 = 0; n11 < n13; ++n11) {
                        n10 = ((a)object).d[n9][n11] - 1;
                        n2 = ((a)object).l[n10];
                        if (((a)object).d[n2] != 8) continue;
                        super.a(((a)object).g[n12] >> 5, ((a)object).h[n12] >> 5, 82);
                        super.a(n8, n4, 82);
                        super.e(n10, ((a)object).g[n12] >> 5, ((a)object).h[n12] >> 5);
                    }
                }
                this.bm = n + 1;
            } else if (string.equals("LAY")) {
                this.r(this.am);
                this.x = (byte)this.a((String)object, n + 1, " ");
                this.bm = this.bs + 1;
                this.q = true;
                this.c = null;
                this.q = (byte)5;
            } else if (string.equals("ROS")) {
                n2 = this.a((String)object, n + 1, "_");
                n = this.bs;
                switch (n2) {
                    case 1: {
                        this.N = this.a((String)object, n + 1, "_") << 5;
                        n = this.bs;
                        this.O = this.a((String)object, n + 1, " ") << 5;
                        this.e(0);
                        break;
                    }
                    case 2: {
                        this.a(this.a((String)object, n + 1, " "), true);
                        break;
                    }
                    case 3: {
                        this.J = this.a((String)object, n + 1, " ");
                        break;
                    }
                    case 4: {
                        this.m = (byte)this.a((String)object, n + 1, " ");
                        break;
                    }
                    case 5: {
                        this.K = this.a((String)object, n + 1, " ");
                        break;
                    }
                    case 6: {
                        this.L = this.a((String)object, n + 1, " ");
                    }
                }
                this.bm = this.bs + 1;
            } else if (string.equals("CES")) {
                int n14;
                int n15 = this.a((String)object, n + 1, "_");
                n = this.bs;
                n3 = this.a((String)object, n + 1, "_");
                n = this.bs;
                n4 = this.a((String)object, n + 1, " ");
                n2 = this.a(n15, n3 << 5, n4 << 5, 0);
                this.d(n2);
                object = this;
                this.bt = (((a)object).ag - 32 >> 1) - ((a)object).ae;
                ((a)object).bu = (((a)object).ah - 32 >> 1) - ((a)object).af;
                if (n2 >= 0) {
                    ((a)object).bv = ((a)object).g[n2];
                    ((a)object).bw = ((a)object).h[n2];
                }
                n2 = n15;
                object = this;
                switch (((a)object).d[n2]) {
                    case 1: 
                    case 8: 
                    case 16: {
                        n14 = 5;
                        break;
                    }
                    case 2: 
                    case 4: {
                        n14 = 8;
                        break;
                    }
                    default: {
                        n14 = 0;
                    }
                }
                this.bo = n14;
                this.bm = this.bs + 1;
            } else if (string.equals("GIN")) {
                n2 = this.a((String)object, n + 1, "_");
                n = this.bs;
                int n16 = this.a((String)object, n + 1, " ");
                switch (n2) {
                    case 0: {
                        this.j(n16);
                        break;
                    }
                    case 1: {
                        this.c(n16, this.N, this.O);
                    }
                }
                this.bm = this.bs + 1;
                this.a(this.f, this.bm);
            } else if (string.equals("ADD")) {
                n2 = this.a((String)object, n + 1, "_");
                n = this.bs;
                int n17 = this.a((String)object, n + 1, "_");
                n = this.bs;
                n3 = this.a((String)object, n + 1, "_");
                n = this.bs;
                n4 = this.a((String)object, n + 1, " ");
                boolean bl2 = false;
                n4 <<= 5;
                n3 <<= 5;
                object = this;
                if (((a)object).am != n2) {
                    super.r(((a)object).am);
                    super.t(n2);
                }
                super.a(n17, n3, n4, 0);
                super.r(n2);
                if (((a)object).am != n2) {
                    super.s();
                    super.t(((a)object).am);
                }
                this.bm = this.bs + 1;
            } else if (string.equals("GLV")) {
                this.a((String)object, n + 1, " ");
                this.bm = this.bs + 1;
            } else if (string.equals("RES")) {
                this.a((String)object, n + 1, " ");
                if (this.A == 0) {
                    object = this;
                    ((a)object).aO = 0;
                    ((a)object).aP = 0;
                    this.aN = 0;
                    ((a)object).aR = 0;
                    ((a)object).aQ = 4;
                    ((a)object).ar = 0;
                    ((a)object).an = 1;
                    ((a)object).ao = 3;
                    ((a)object).K = 10;
                    ((a)object).L = 10;
                    ((a)object).J = 400;
                    ((a)object).aL = 0;
                }
                this.bm = this.bs + 1;
            } else if (string.equals("SEE")) {
                this.bk = this.a((String)object, n + 1, "_");
                n = this.bs;
                this.bl = this.a((String)object, n + 1, "_");
                n = this.bs;
                this.bq = this.a((String)object, n + 1, "_");
                n = this.bs;
                this.br = this.a((String)object, n + 1, "_");
                n = this.bs;
                this.bn = this.a((String)object, n + 1, " ");
                this.bm = this.bs + 1;
                this.n(this.bq);
                switch (this.bn) {
                    case 0: {
                        break;
                    }
                    case 1: {
                        if (!this.a(this.P, this.Q, this.bk, this.bl)) break;
                        this.A = true;
                        this.l = 0;
                    }
                }
                this.q = (byte)6;
                this.l(this.bk, this.bl);
            } else if (string.equals("END")) {
                this.a((String)object, n + 1, " ");
                this.a = (byte)20;
                this.a();
                this.bm = this.bs + 1;
            } else if (string.equals("SMS")) {
                this.a((String)object, n + 1, " ");
                this.bm = this.bs + 1;
            }
            if (this.bm < this.f.length()) {
                return;
            }
        } else {
            this.q = (byte)4;
            if (this.bh != 32) {
                this.g[this.bh] = true;
            }
            this.e(0);
        }
    }

    private int a(String string, int n, String string2) {
        int n2 = 0;
        n2 = string.indexOf(string2, n);
        string = string.substring(n, n2);
        this.bs = n2;
        n2 = Integer.parseInt(string);
        return n2;
    }

    private int b(int n, int n2, int n3) {
        n = this.c[n2][n];
        n2 = this.k[n];
        int n4 = 0;
        if (n2 > 0) {
            for (int i = 0; i < n2; ++i) {
                n4 = this.d[n][i] - 1;
                if (this.l[n4] != n3 || this.e[n4] == 1) continue;
                return n4;
            }
        }
        return -1;
    }

    private void d(int n, int n2, int n3) {
        if (this.b(n, --n2) && !this.c(n, n2) && (n != this.P || n2 != this.Q)) {
            this.f(n3);
            n3 = this.a(60, n << 5, n2 << 5, 0);
            this.d(n3);
        }
    }

    private boolean d(int n, int n2) {
        boolean bl = false;
        if (this.am != n) {
            this.r(this.am);
            this.t(n);
        }
        for (int i = 0; i < this.T; ++i) {
            if (this.b[i] || this.l[i] != n2) continue;
            bl = true;
            break;
        }
        if (this.am != n) {
            this.s();
            this.t(this.am);
        }
        return bl;
    }

    private void l(int n, int n2) {
        this.bt = (this.ag - 32 >> 1) - this.ae;
        this.bu = (this.ah - 32 >> 1) - this.af;
        this.bv = n << 5;
        this.bw = n2 << 5;
    }

    private int e(int n) {
        int n2;
        int n3;
        block4: {
            this.bt = (this.ag - 32 >> 1) - this.ae;
            this.bu = (this.ah - 32 >> 1) - this.af;
            if (n == 0 || n == 87) {
                this.bv = this.N;
                this.bw = this.O;
                return -1;
            }
            int n4 = n;
            a a2 = this;
            int n5 = a2.T;
            while (--n5 >= 0) {
                if (a2.l[n5] != n4) continue;
                n3 = n5;
                break block4;
            }
            n3 = n2 = -1;
        }
        if (n3 >= 0) {
            this.bv = this.g[n2];
            this.bw = this.h[n2];
        }
        if (n == 69) {
            this.bv += 32;
        }
        return n2;
    }

    private void x() {
        if (this.bt < this.bv) {
            this.bt += (this.bv - this.bt >> 2) + 2;
            if (this.bt > this.bv) {
                this.bt = this.bv;
            }
        } else if (this.bt > this.bv) {
            this.bt += (this.bv - this.bt >> 2) - 2;
            if (this.bt < this.bv) {
                this.bt = this.bv;
            }
        }
        if (this.bu < this.bw) {
            this.bu += (this.bw - this.bu >> 2) + 2;
            if (this.bu > this.bw) {
                this.bu = this.bw;
            }
        } else if (this.bu > this.bw) {
            this.bu += (this.bw - this.bu >> 2) - 2;
            if (this.bu < this.bw) {
                this.bu = this.bw;
            }
        }
        this.i((this.ag - 32 >> 1) - this.bt, (this.ah - 32 >> 1) - this.bu);
    }

    private void y() {
        switch (this.s) {
            case 5: {
                this.c(this.o);
                if (this.x || !this.b[this.bx] && this.e[this.bx] != 1) break;
                this.q = 0;
                this.s = 0;
                this.bx = -1;
                return;
            }
            case 0: {
                if (this.bS > 0) {
                    this.t = this.Q[--this.bS];
                    a a2 = this;
                    int n = 0;
                    int n2 = 0;
                    int n3 = 0;
                    int n4 = 0;
                    n = a2.g[a2.bx] >> 5;
                    n2 = a2.h[a2.bx] >> 5;
                    switch (a2.t) {
                        case 1: {
                            --n2;
                            break;
                        }
                        case 0: {
                            ++n2;
                            break;
                        }
                        case 3: {
                            --n;
                            break;
                        }
                        case 2: {
                            ++n;
                        }
                    }
                    a2.s = 1;
                    if (a2.l[a2.bx] != 72 && n == a2.P && n2 == a2.Q) {
                        n3 = a2.l[a2.bx];
                        n4 = a2.a(n3, false);
                        if (n4 >= 0 && n4 < a2.J) {
                            a2.bU = a2.v[n3 - 41];
                            a2.l = (byte)5;
                            a2.o = false;
                            a2.S = a2.bx;
                            a2.e[a2.bx] = 5;
                            a2.s = (byte)5;
                            a2.z = false;
                            n4 = a2.t[n3 - 41] - a2.L;
                            if (n4 > 0) {
                                a2.J -= n4;
                            }
                        } else {
                            a2.l = 1;
                            a2.m = a2.t;
                        }
                    }
                    if (a2.k[n = a2.c[n2][n]] > 0) {
                        n4 = a2.k[n];
                        while (--n4 >= 0) {
                            n2 = a2.d[n][n4] - 1;
                            n3 = a2.l[n2];
                            switch (a2.d[n3]) {
                                case 1: {
                                    switch (n3) {
                                        case 11: {
                                            a2.e(n2);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return;
                }
                this.f(this.bx, this.bz, this.bA);
                this.q = 0;
                this.g(this.l[this.bx], this.bx);
                this.bx = -1;
                return;
            }
            case 1: {
                switch (this.t) {
                    case 1: {
                        int n = this.bx;
                        this.h[n] = this.h[n] - 8;
                        break;
                    }
                    case 0: {
                        int n = this.bx;
                        this.h[n] = this.h[n] + 8;
                        break;
                    }
                    case 3: {
                        int n = this.bx;
                        this.g[n] = this.g[n] - 8;
                        break;
                    }
                    case 2: {
                        int n = this.bx;
                        this.g[n] = this.g[n] + 8;
                    }
                }
                this.bv = this.g[this.bx];
                this.bw = this.h[this.bx];
                this.by += 8;
                int n = this.m[this.bx];
                if (n > 0) {
                    if (this.k[this.bx] < this.c[n].length - 1) {
                        int n5 = this.bx;
                        this.k[n5] = this.k[n5] + 1;
                    } else {
                        this.k[this.bx] = 0;
                    }
                }
                if (this.by != 32) break;
                this.by = 0;
                this.s = 0;
                a a3 = this;
                int n6 = a3.g[a3.bx] >> 5;
                int n7 = a3.h[a3.bx] >> 5;
                int n8 = 0;
                if (n6 == a3.P && n7 == a3.Q && a3.l[a3.bx] != 72) {
                    n8 = a3.a(a3.l[a3.bx], false);
                    if (n8 < 0 || n8 >= a3.J) {
                        a3.B = false;
                        a3.a = (byte)3;
                        a3.l = 1;
                        a3.m = a3.t;
                    } else {
                        a3.J -= n8;
                        a3.f(a3.bx, a3.bz, a3.bA);
                        a3.a(n6, n7, a3.l[a3.bx]);
                    }
                }
                this.y();
            }
        }
    }

    private void e(int n, int n2, int n3) {
        int n4 = this.g[n];
        int n5 = this.h[n];
        short s = this.a[n];
        this.a(this.l[n], n2 << 5, n3 << 5, (int)s);
        this.a(n4 >> 5, n5 >> 5, this.l[n]);
    }

    private void f(int n, int n2, int n3) {
        int n4;
        int n5;
        n2 = this.c[n3][n2];
        n3 = this.k[n2];
        for (n5 = 0; n5 < n3; ++n5) {
            if (this.d[n2][n5] - 1 != n) continue;
            for (n4 = n5; n4 < n3 - 1; ++n4) {
                this.d[n2][n4] = this.d[n2][n4 + 1];
            }
            int n6 = n2;
            this.k[n6] = this.k[n6] - 1;
            break;
        }
        if ((n2 = this.c[n4 = this.h[n] >> 5][n5 = this.g[n] >> 5]) == 0) {
            this.c[n4][n5] = this.p = (byte)(this.p + 1);
            n2 = this.p;
        }
        this.d[n2][this.k[n2]] = n + 1;
        int n7 = n2;
        this.k[n7] = this.k[n7] + 1;
    }

    private void o(int n) {
        byte[] byArray = this.e[n];
        int n2 = byArray.length;
        int n3 = 0;
        for (int i = 0; i < n2; i += 4) {
            n3 = this.a((int)byArray[i], byArray[i + 1] << 5, byArray[i + 2] << 5, (int)byArray[i + 3]);
            this.d(n3);
        }
    }

    private void z() {
        String string = "SKY_WAR";
        boolean bl = false;
        try {
            int n;
            this.a = RecordStore.openRecordStore((String)string, (boolean)true);
            if (this.a.getNumRecords() == 0) {
                bl = true;
            }
            this.a = new ByteArrayOutputStream();
            this.a = new DataOutputStream(this.a);
            for (n = 0; n < 4; ++n) {
                this.a.writeBoolean(this.f[n]);
            }
            this.a.writeByte(this.I.length);
            for (n = 0; n < this.I.length; ++n) {
                this.a.writeInt(this.J[n]);
            }
            this.a.writeInt(this.bW);
            this.a.writeInt(this.bX);
            this.a.writeInt(this.bY);
            if (!bl) {
                this.a.setRecord(1, this.a.toByteArray(), 0, this.a.size());
                return;
            }
            this.a.addRecord(this.a.toByteArray(), 0, this.a.size());
        }
        catch (Exception exception) {}
    }

    private void A() {
        String string = "MOT_IF";
        if (!this.p) {
            this.h = new boolean[6];
            this.y = new byte[6];
            this.x = new int[6];
            this.y = new int[6];
            this.z = new int[6];
            this.A = new int[6];
            this.B = new int[6];
            this.C = new int[6];
            this.D = new int[6];
            this.z = new byte[6];
            this.A = new byte[6];
            this.p = true;
        }
        try {
            this.a = RecordStore.openRecordStore((String)string, (boolean)false);
            this.a = this.a.enumerateRecords(null, null, false);
            int n = this.a.nextRecordId();
            this.B = this.a.getRecord(n);
            this.a = new DataInputStream(new ByteArrayInputStream(this.B));
            for (n = 0; n < 6; ++n) {
                boolean bl = this.h[n] = this.a.readByte() != 0;
                if (!this.h[n]) continue;
                this.y[n] = this.a.readByte();
                this.z[n] = this.a.readByte();
                this.A[n] = this.a.readByte();
                this.x[n] = this.a.readInt();
                this.y[n] = this.a.readInt();
                this.z[n] = this.a.readInt();
                this.A[n] = this.a.readInt();
                this.B[n] = this.a.readInt();
                this.C[n] = this.a.readInt();
                this.D[n] = this.a.readInt();
            }
            return;
        }
        catch (Exception exception) {
            this.p(-1);
            return;
        }
        finally {
            this.B();
        }
    }

    private void p(int n) {
        if (!this.p) {
            this.A();
        }
        if (n >= 0 && n < 6) {
            this.h[n] = true;
            this.y[n] = this.am;
            this.x[n] = this.J;
            this.y[n] = this.K;
            this.z[n] = this.L;
            this.A[n] = this.aN;
            this.B[n] = this.aO;
            this.C[n] = this.aP;
            this.D[n] = this.aQ;
            this.z[n] = this.j;
            this.A[n] = this.k;
        }
        Object object = "MOT_IF";
        a.a("MOT_IF");
        try {
            this.a = RecordStore.openRecordStore((String)object, (boolean)true);
            this.a = new ByteArrayOutputStream();
            this.a = new DataOutputStream(this.a);
            for (int i = 0; i < 6; ++i) {
                if (this.h[i]) {
                    this.a.write(1);
                    this.a.writeByte(this.y[i]);
                    this.a.writeByte(this.z[i]);
                    this.a.writeByte(this.A[i]);
                    this.a.writeInt(this.x[i]);
                    this.a.writeInt(this.y[i]);
                    this.a.writeInt(this.z[i]);
                    this.a.writeInt(this.A[i]);
                    this.a.writeInt(this.B[i]);
                    this.a.writeInt(this.C[i]);
                    this.a.writeInt(this.D[i]);
                    continue;
                }
                this.a.write(0);
            }
            this.a.addRecord(this.a.toByteArray(), 0, this.a.size());
            return;
        }
        catch (Exception exception) {
            object = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.B();
        }
    }

    private void b(boolean n) {
        n = n != 0 ? 2 : 1;
        int n2 = 240 - this.bF - 22 >> 1;
        int n3 = 320 - this.bB >> 1;
        Image image = null;
        this.a(n, n2, n3, this.bF + 22, this.bB);
        n3 += 16;
        n = this.bC;
        while (n < this.bD) {
            if (n != this.bE) {
                this.a.setColor(7574946);
            } else {
                this.a.setColor(3156024);
                this.a.fillRect(120 - (this.bF >> 1), n3, this.bF, this.b + 4);
                this.a.drawImage(this.a[8][14], 120 - (this.bF >> 1) + 10, n3 + (this.b - 8 >> 1), 0);
                this.a(this.a[8][14], 120 + (this.bF >> 1) - 30, n3 + (this.b - 8 >> 1), 1);
                this.a.setColor(16377897);
            }
            if (!this.h[n]) {
                this.a.drawString("---", 120, n3 + 2, 17);
            } else {
                this.a.drawString("\u5b58\u6863" + n, 120, n3 + 2, 17);
            }
            ++n;
            n3 += this.b + 4;
        }
        n2 = 120 + ((this.bF >> 1) - 10);
        if (this.bC > 0) {
            this.a(this.a[8][15], n2, n3 - 20, 0, 0, 7, 9);
        }
        if (this.bD < 6) {
            this.a(this.a[8][15], n2, n3 - 10, 7, 0, 7, 9);
        }
        this.a.setColor(6178);
        this.a.drawLine(n2 -= this.bF - 10, n3 += 2, n2 + this.bF - 1, n3);
        this.a.setColor(3564144);
        this.a.drawLine(n2, ++n3, n2 + this.bF - 1, n3);
        if (this.h[this.bE]) {
            n2 += 6;
            n3 += 8;
            if (this.y[this.bE] < 51) {
                this.a(this.a[8][18], (int)this.y[this.bE], n2 + 48, n3);
                this.a.drawImage(this.a[8][8], n2 + 50, n3 + 8, 0);
            }
            n2 = 240 + this.bF >> 1;
            this.d(n2 -= 68, n3, 32, 32);
            if (this.z[this.bE] == 0) {
                this.a.setColor(-1);
                this.a.drawString(this.e[this.z[this.bE]], n2 + (32 - this.a >> 1), n3 + (32 - this.b >> 1), 0);
            } else {
                n = this.z[this.bE] - 33;
                if (n > 7) {
                    n = 8;
                }
                image = this.a[6][n];
                this.a.drawImage(image, n2 + (32 - image.getWidth() >> 1), n3 + (32 - image.getHeight() >> 1), 0);
            }
            this.d(n2 += 34, n3, 32, 32);
            if (this.A[this.bE] == 0) {
                this.a.setColor(-1);
                this.a.drawString(this.e[this.A[this.bE]], n2 + (32 - this.a >> 1), n3 + (32 - this.b >> 1), 0);
            } else {
                n = this.A[this.bE] - 33;
                if (n > 7) {
                    n = 9;
                }
                image = this.a[6][n];
                this.a.drawImage(image, n2 + (32 - image.getWidth() >> 1), n3 + (32 - image.getHeight() >> 1), 0);
            }
            n2 = 240 - this.bF + 10 >> 1;
            this.a.drawImage(this.a[8][1], n2, n3 += 20, 0);
            this.a(this.a[8][2], this.D[this.bE], n2 += 60, n3 + 2);
            this.a(this.a[8][7], n2 -= 64, n3 += 16, 0, 2, 10, 10);
            this.d((n2 += 15) - 1, n3, 48, 12);
            this.a(this.a[8][2], this.x[this.bE], n2 + 45, n3 + 2);
            this.a(this.a[8][7], n2 += 47, n3, 10, 0, 10, 13);
            this.a.setColor(512);
            this.d(n2 += 11, n3, 30, 12);
            this.a(this.a[8][2], this.y[this.bE], n2 + 28, n3 + 2);
            this.a(this.a[8][7], n2 += 32, n3, 20, 2, 10, 10);
            this.a.setColor(512);
            this.d(n2 += 12, n3, 30, 12);
            this.a(this.a[8][2], this.z[this.bE], n2 + 28, n3 + 2);
            n2 = (240 - this.bF >> 1) + 6;
            this.a(0, this.A[this.bE], n2, n3 += 16);
            this.a(1, this.B[this.bE], n2 += 42, n3);
            this.a(2, this.C[this.bE], n2 += 42, n3);
        }
    }

    private void q(int n) {
        this.p(n);
        Object object = "MOT_L" + n;
        a.a((String)object);
        try {
            int n2;
            this.a = RecordStore.openRecordStore((String)object, (boolean)true);
            this.a = new ByteArrayOutputStream();
            this.a = new DataOutputStream(this.a);
            this.a.writeByte(this.A);
            this.a.writeByte(this.am);
            this.a.writeByte(this.an);
            this.a.writeByte(this.ao);
            this.a.writeByte(this.m);
            this.a.writeByte(this.j);
            this.a.writeByte(this.k);
            this.a.writeShort(this.P);
            this.a.writeShort(this.Q);
            this.a.writeInt(this.J);
            this.a.writeInt(this.K);
            this.a.writeInt(this.L);
            this.a.writeShort(this.aN);
            this.a.writeShort(this.aO);
            this.a.writeShort(this.aP);
            this.a.writeInt(this.aQ);
            this.a.writeShort(this.ar);
            this.a.writeShort(this.aL);
            for (n2 = 0; n2 < this.aL; ++n2) {
                this.a.writeByte(this.o[n2]);
                this.a.writeByte(this.p[n2]);
            }
            for (n2 = 0; n2 < 128; ++n2) {
                if (n2 < this.bj) {
                    this.a.writeBoolean(this.g[n2]);
                    continue;
                }
                this.a.writeBoolean(false);
            }
            this.a.writeByte(this.aR);
            for (n2 = 0; n2 < this.aR; ++n2) {
                this.a.writeByte(this.r[n2]);
                this.a.writeByte(this.s[n2]);
            }
            for (n2 = 0; n2 < 56; ++n2) {
                if (this.f[n2] != null) {
                    this.a.writeShort(this.f[n2].length);
                    this.a.write(this.f[n2], 0, this.f[n2].length);
                    continue;
                }
                this.a.writeShort(-1);
            }
            this.a.addRecord(this.a.toByteArray(), 0, this.a.size());
            return;
        }
        catch (Exception exception) {
            object = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.B();
        }
    }

    private boolean e(int n) {
        boolean bl = true;
        short s = 0;
        Object object = "MOT_L" + n;
        try {
            this.a = RecordStore.openRecordStore((String)object, (boolean)false);
            this.a = this.a.enumerateRecords(null, null, false);
            int n2 = this.a.nextRecordId();
            this.B = this.a.getRecord(n2);
            this.a = new DataInputStream(new ByteArrayInputStream(this.B));
            this.A = this.a.readByte();
            this.am = this.a.readByte();
            this.an = this.a.readByte();
            this.ao = this.a.readByte();
            this.m = this.a.readByte();
            this.j = this.a.readByte();
            this.k = this.a.readByte();
            this.P = this.a.readShort();
            this.Q = this.a.readShort();
            this.N = this.P << 5;
            this.O = this.Q << 5;
            this.J = this.a.readInt();
            this.K = this.a.readInt();
            this.L = this.a.readInt();
            this.aN = this.a.readShort();
            this.aO = this.a.readShort();
            this.aP = this.a.readShort();
            this.aQ = this.a.readInt();
            this.ar = this.a.readShort();
            this.R = 0;
            this.aL = this.a.readShort();
            for (n2 = 0; n2 < this.aL; ++n2) {
                this.o[n2] = this.a.readByte();
                this.p[n2] = this.a.readByte();
            }
            for (n2 = 0; n2 < this.bj; ++n2) {
                this.g[n2] = this.a.readBoolean();
            }
            if (this.bj < 128) {
                this.a.skip(128 - this.bj);
            }
            this.m = false;
            this.aR = this.a.readByte();
            for (n2 = 0; n2 < this.aR; ++n2) {
                this.r[n2] = this.a.readByte();
                this.s[n2] = this.a.readByte();
                if (this.r[n2] != 15) continue;
                this.m = true;
            }
            for (n2 = 0; n2 < 56; ++n2) {
                s = this.a.readShort();
                if (s > 0) {
                    this.f[n2] = new byte[s];
                    this.a.read(this.f[n2], 0, s);
                    continue;
                }
                this.f[n2] = null;
            }
            this.g(this.H[this.A]);
        }
        catch (Exception exception) {
            object = exception;
            exception.printStackTrace();
            bl = false;
        }
        finally {
            this.B();
        }
        return bl;
    }

    private static void a(String string) {
        try {
            RecordStore.deleteRecordStore((String)string);
            return;
        }
        catch (Exception exception) {
            return;
        }
    }

    private void B() {
        if (this.a != null) {
            try {
                this.a.closeRecordStore();
            }
            catch (Exception exception) {}
        }
        if (this.a != null) {
            try {
                this.a.close();
            }
            catch (Exception exception) {}
        }
        if (this.a != null) {
            try {
                this.a.close();
            }
            catch (Exception exception) {}
        }
        if (this.a != null) {
            this.a.destroy();
        }
        this.a = null;
        this.a = null;
        this.a = null;
        this.a = null;
        this.B = null;
        System.gc();
    }

    private void C() {
        int n = 56;
        while (--n >= 0) {
            this.f[n] = null;
        }
    }

    private void r(int n) {
        int n2 = this.T;
        int n3 = this.T;
        while (--n3 >= 0) {
            if (!this.b[n3] && this.e[n3] != 1) continue;
            --n2;
        }
        this.f[n] = null;
        if (this.T > 0) {
            this.f[n] = new byte[(n2 << 3) + 2];
            this.C = this.f[n];
            this.bH = 0;
            this.a((short)n2);
            for (n3 = 0; n3 < this.T; ++n3) {
                if (this.b[n3] || this.e[n3] == 1) continue;
                this.C[this.bH++] = this.l[n3];
                this.a((short)this.g[n3]);
                this.a((short)this.h[n3]);
                this.C[this.bH++] = this.a[n3] ? (byte)1 : 0;
                this.a(this.a[n3]);
            }
        }
    }

    private void a(short s) {
        this.C[this.bH++] = (byte)(s >> 8);
        this.C[this.bH++] = (byte)s;
    }

    /*
     * Loose catch block
     */
    private void s(int n) {
        a a2;
        block10: {
            int n2 = n;
            a2 = this;
            InputStream inputStream = a2.getClass().getResourceAsStream("maplv" + n2);
            a2.Y = a.a(inputStream) >> 1;
            a2.Z = a.a(inputStream) >> 1;
            System.out.println("Width:" + a2.Y + ",Height:" + a2.Z);
            int n3 = a2.Y * a2.Z << 2;
            a2.aa = a2.Y << 5;
            a2.ab = a2.Z << 5;
            a2.i = new byte[n3];
            a2.j = new byte[n3];
            inputStream.read(a2.i, 0, n3);
            inputStream.read(a2.j, 0, n3);
            a.a(32, 32, 0x55FF00FF);
            a.a(32, 32, 0x5500FF00);
            try {
                inputStream.close();
            }
            catch (Exception exception) {}
            break block10;
            catch (Exception exception) {
                try {
                    Exception exception2 = exception;
                    exception.printStackTrace();
                }
                catch (Throwable throwable) {
                    try {
                        inputStream.close();
                    }
                    catch (Exception exception3) {}
                    throw throwable;
                }
                try {
                    inputStream.close();
                }
                catch (Exception exception4) {}
            }
        }
        a2.ag = 240;
        a2.ah = 252;
        a2.k = a2.ag >= a2.aa;
        a2.l = a2.ah >= a2.ab;
        a2.s();
        v4.a = new boolean[a2.Z][a2.Y];
        a2.i(0, 0);
        a2.t();
        this.T = 0;
        this.av = -1;
        this.at = -1;
        this.au = -1;
        this.as = -1;
        this.t(n);
        this.p();
    }

    private void t(int n) {
        int n2;
        boolean bl = false;
        boolean bl2 = false;
        int n3 = 0;
        a a2 = this;
        this.T = 0;
        a a3 = a2;
        for (n2 = 1; n2 <= 12; ++n2) {
            a3.a[n2] = a3.a[4][n2 - 1];
        }
        for (n2 = 13; n2 <= 32; ++n2) {
            a3.a[n2] = a3.a[5][n2 - 13 + 1];
        }
        for (n2 = 33; n2 <= 40; ++n2) {
            a3.a[n2] = a3.a[6][n2 - 33];
        }
        for (n2 = 41; n2 < 61; ++n2) {
            a3.a[n2] = a3.a[7][n2 - 41];
        }
        for (n2 = 61; n2 < 79; ++n2) {
            a3.a[n2] = a3.a[13][n2 - 41 - 20];
        }
        a3.a[84] = a3.a[13][18];
        a3.a[87] = a3.a[13][19];
        a3.a[85] = a3.a[5][21];
        a3.a[86] = a3.a[5][22];
        a3.a[79] = a3.a[6][8];
        a3.a[80] = a3.a[6][9];
        a3.a[12] = a3.a[11];
        a3.a[81] = a3.a[4][3];
        a3.a[71] = a3.a[70];
        this.T = 0;
        if (this.f[n] == null) {
            try {
                int n4 = n;
                a2 = this;
                InputStream inputStream = a2.getClass().getResourceAsStream("sprite" + n4);
                boolean bl3 = false;
                int n5 = 0;
                try {
                    InputStream inputStream2 = inputStream;
                    a a4 = a2;
                    int n6 = a.a(inputStream2) & 0xFFFF | a.a(inputStream2) << 16;
                    block27: for (int i = 0; i < n6; ++i) {
                        int n7 = inputStream.read();
                        n3 = a.a(inputStream);
                        short s = a.a(inputStream);
                        switch (n7) {
                            case 6: 
                            case 12: {
                                n7 = a2.a(n7, n3, (int)s, 0);
                                a2.a[n7] = false;
                                continue block27;
                            }
                            case 9: {
                                n5 = inputStream.read();
                                a2.a(n7, n3, (int)s, n5);
                                continue block27;
                            }
                            case 4: {
                                n5 = inputStream.read() + 1;
                                a2.a(n7, n3, (int)s, n5);
                                continue block27;
                            }
                            case 5: 
                            case 81: {
                                n5 = inputStream.read() + 1;
                                n7 = a2.a(n7, n3, (int)s, n5);
                                a2.a[n7] = false;
                                continue block27;
                            }
                            case 83: {
                                n5 = inputStream.read();
                                n7 = a2.a(n7, n3, (int)s, n5);
                                a2.a[n7] = false;
                                continue block27;
                            }
                            case 7: 
                            case 8: {
                                n5 = inputStream.read() | inputStream.read() << 8;
                                a2.a(n7, n3, (int)s, n5);
                                continue block27;
                            }
                            case 57: 
                            case 59: 
                            case 70: 
                            case 71: 
                            case 72: 
                            case 73: {
                                n5 = inputStream.read() + 1;
                                a2.a(n7, n3, (int)s, n5);
                                continue block27;
                            }
                            case 77: {
                                n5 = inputStream.read();
                                n5 = n5 & 0xFF | a2.m[n5 - 1] << 8;
                                a2.a(n7, n3, (int)s, n5);
                                continue block27;
                            }
                            case 78: {
                                n5 = inputStream.read();
                                n5 = n5 & 0xFF | a2.n[n5 - 1] << 8;
                                a2.a(n7, n3, (int)s, n5);
                                continue block27;
                            }
                            case 76: 
                            case 82: {
                                n5 = inputStream.read() | inputStream.read() + 1 << 8;
                                n7 = a2.a(n7, n3, (int)s, n5);
                                a2.a[n7] = false;
                                continue block27;
                            }
                            default: {
                                a2.a(n7, n3, (int)s, 0);
                            }
                        }
                    }
                }
                catch (Exception exception) {
                    Exception exception2 = exception;
                    exception.printStackTrace();
                }
            }
            catch (Exception exception) {}
        } else {
            this.D = this.f[n];
            this.bI = 0;
            int n8 = this.b();
            for (int i = 0; i < n8; ++i) {
                byte by = this.D[this.bI++];
                n2 = this.b();
                int n9 = this.b();
                boolean bl4 = this.D[this.bI++] != 0;
                n3 = this.b();
                this.a((int)by, n2, n9, n3);
                this.a[i] = bl4;
                this.b[i] = false;
            }
            switch (this.am) {
                case 31: {
                    break;
                }
                case 12: {
                    n8 = this.b(1, 1, 78);
                    if (n8 >= 0) {
                        this.a[n8] = 6 | this.n[5] << 8;
                    }
                    if ((n8 = this.b(11, 1, 78)) < 0) break;
                    this.a[n8] = 5 | this.n[4] << 8;
                    break;
                }
                case 2: {
                    if (this.g[26] || this.b(11, 4, 84) >= 0) break;
                    this.a(84, 352, 128, 0);
                    break;
                }
                case 39: {
                    n8 = this.b(11, 1, 7);
                    if (n8 >= 0) {
                        this.a(11, 1, 7);
                        this.a(8, 11, 1, 38);
                    }
                    if ((n8 = this.b(11, 11, 8)) < 0) break;
                    this.a(11, 11, 8);
                    this.a(7, 11, 11, 40);
                }
            }
        }
        if (n != 50) {
            this.d[72] = 32;
            return;
        }
        this.d[72] = 8;
    }

    private int b() {
        return (this.D[this.bI++] & 0xFF) << 8 | this.D[this.bI++] & 0xFF;
    }

    private void m(int n, int n2) {
        if (this.v < 64) {
            this.h[this.v] = n;
            this.i[this.v] = n2;
            this.v = (byte)(this.v + 1);
        }
    }

    private void a(byte by, int n, int n2, int n3) {
        n2 += this.ae;
        this.E[this.bK] = by;
        this.E[this.bK] = n;
        this.i[this.bK] = false;
        this.F[this.bK] = 0;
        this.k[this.bK] = n3 += this.af;
        if (by == 4) {
            this.j[this.bK] = 240;
            this.k[this.bK] = 30;
        } else if (by >= 5 && by <= 7) {
            this.j[this.bK] = n2 + 16;
        } else if (by == 1) {
            this.j[this.bK] = n2 - 2;
        } else {
            int n4;
            int n5 = n2;
            Image image = this.a[2][3];
            n2 = n4 = image.getWidth() / 11;
            if (n < 0) {
                n = -n;
            }
            do {
                n2 += n4;
            } while ((n /= 10) > 0);
            this.j[this.bK] = n5 + (32 + n2 >> 1);
        }
        if (++this.bK >= 30) {
            this.bK = 0;
        }
    }

    private void D() {
        byte by = 0;
        int n = this.bJ;
        while (true) {
            if (n >= 30) {
                n = 0;
            }
            if (n == this.bK) break;
            if (!this.i[n]) {
                byte by2 = this.E[n];
                int n2 = this.E[n];
                int n3 = 0 + this.j[n];
                by = this.F[n];
                int n4 = 16 + this.k[n];
                switch (by2) {
                    case 1: {
                        this.a(this.a[2][6], n3, n4 -= this.F[n] << 2, 0, 19 * n2, 37, 19);
                        break;
                    }
                    case 2: {
                        this.b(this.a[2][3], n2, n3, n4 -= this.F[n] << 2);
                        break;
                    }
                    case 3: {
                        this.b(this.a[2][4], n2, n3, n4 -= this.F[n] << 2);
                        break;
                    }
                    case 4: {
                        this.b(this.a[2][5], n2, n3, n4 -= this.F[n] << 2);
                        break;
                    }
                    case 5: {
                        this.a(this.a[8][9], n2, n3 + this.G[by], 16 + this.k[n] + this.H[by]);
                        break;
                    }
                    case 6: {
                        this.a(this.a[8][9], n2, n3 + this.I[by], 16 + this.k[n] + this.J[by]);
                        break;
                    }
                    case 7: {
                        this.a(this.a[8][9], n2, n3 + this.K[by], 16 + this.k[n] + this.L[by]);
                    }
                }
                if (by > 7) {
                    this.i[n] = true;
                } else {
                    int n5 = n;
                    this.F[n5] = this.F[n5] + 1;
                }
            }
            ++n;
        }
        if (this.bJ != this.bK && this.i[this.bJ] && ++this.bJ >= 30) {
            this.bJ = 0;
        }
    }

    private int b(Image image, int n, int n2, int n3) {
        int n4 = n < 0 ? 1 : 0;
        if (n4 != 0) {
            n = -n;
        }
        n4 = image.getWidth() / 11;
        int n5 = image.getHeight();
        int n6 = 0;
        int n7 = 0;
        do {
            n6 = n % 10;
            this.a.setClip(n2 -= n4 + 1, n3, n4, n5);
            this.a.drawImage(image, n2 - n6 * n4, n3, 0);
            ++n7;
        } while ((n /= 10) > 0);
        this.a.setClip(n2 -= n4 + 1, n3, n4, n5);
        this.a.drawImage(image, n2 - (image.getWidth() - n4), n3, 0);
        this.a.setClip(0, 0, 240, 320);
        return n7 + 1;
    }

    private void E() {
        this.l = new short[32];
        this.m = new short[32];
        this.M = new byte[32];
        this.N = new byte[32];
        this.O = new byte[32];
        this.w = 0;
        this.F = new int[32];
        this.j = new boolean[32];
        this.k = new boolean[32];
    }

    private void e(int n, int n2, int n3, int n4) {
        this.l[this.w] = n;
        this.m[this.w] = n2;
        this.M[this.w] = -3;
        this.N[this.w] = -4;
        this.O[this.w] = 2;
        this.j[this.w] = true;
        this.F[this.w] = -1;
        this.k[this.w] = false;
        this.w = (byte)(this.w + 1);
        if (this.w > 31) {
            this.w = 0;
        }
    }

    private void F() {
        byte by = 0;
        int n = 0;
        this.a.setColor(this.bL);
        int n2 = 32;
        while (--n2 >= 0) {
            if (!this.j[n2]) continue;
            n = this.O[n2];
            this.a.fillRect((int)this.l[n2], (int)this.m[n2], n, n);
            by = this.M[n2];
            int n3 = n2;
            this.l[n3] = this.l[n3] + by;
            n <<= 1;
            if (!this.k[n2]) {
                int n4 = n2;
                this.M[n4] = (byte)(this.M[n4] + 1);
                if (this.M[n4] == n) {
                    this.k[n2] = true;
                }
            } else {
                int n5 = n2;
                this.M[n5] = (byte)(this.M[n5] - 1);
                if (this.M[n5] == -n) {
                    this.k[n2] = false;
                    int n6 = n2;
                    this.O[n6] = (byte)(this.O[n6] + 1);
                    if (this.O[n6] > 5) {
                        this.j[n2] = false;
                    }
                }
            }
            int n7 = n2;
            this.m[n7] = this.m[n7] + this.N[n2];
        }
    }

    private void a(byte by, boolean bl) {
        this.y = (byte)3;
        this.t = true;
        if (this.P == null) {
            this.P = new byte[16];
        }
        this.bP = 0;
        this.bO = 0;
        this.a = (byte)2;
        this.a(15);
    }

    private void u(int n) {
        this.P[this.bO++] = n;
    }

    private boolean a(int n, int n2, int n3, int n4) {
        a a2 = this;
        this.Q = new byte[100];
        a2.bS = 0;
        a2.g = new byte[a2.Z][a2.Y];
        a2.a = new short[a2.Z][a2.Y];
        a2.n = new short[100];
        a2.o = new short[100];
        a2.bT = 0;
        boolean bl = false;
        int n5 = n;
        int n6 = n2;
        int n7 = this.Y;
        int n8 = n5 + n6 * n7;
        this.a = (short)n;
        this.b = (short)n2;
        this.c = (short)n3;
        this.d = (short)n4;
        this.bT = 1;
        n3 = 0;
        do {
            this.g(n5 - 1, n6, n8);
            this.g(n5 + 1, n6, n8);
            this.g(n5, n6 - 1, n8);
            this.g(n5, n6 + 1, n8);
            if (this.bT <= 0) break;
            n8 = this.o[this.bT - 1];
            n5 = n8 % n7;
            n6 = n8 / n7;
            --this.bT;
            this.g[n6][n5] = 2;
            if (n5 != this.c || n6 != this.d) continue;
            bl = true;
        } while (++n3 < 100 && !bl);
        n3 = 0;
        n += n2 * n7;
        n2 = 0;
        if (bl) {
            do {
                if ((n2 = (n3 = this.a[n6][n5]) - n8) == -1) {
                    this.Q[this.bS++] = 2;
                } else if (n2 == 1) {
                    this.Q[this.bS++] = 3;
                } else if (n2 == -n7) {
                    this.Q[this.bS++] = 0;
                } else if (n2 == n7) {
                    this.Q[this.bS++] = 1;
                } else {
                    return false;
                }
                n8 = n3;
                n5 = n3 % n7;
                n6 = n3 / n7;
            } while (n3 != n);
        }
        return bl;
    }

    private void g(int n, int n2, int n3) {
        short s = 0;
        if (this.g[n2][n] == 0 && (this.b(n, n2) || this.c(n, n2))) {
            this.g[n2][n] = 1;
            this.a[n2][n] = n3;
            s = (short)(Math.abs(n - this.c) + Math.abs(n2 - this.d) + Math.abs(n - this.a) + Math.abs(n2 - this.b));
            n3 = this.bT;
            while (--n3 >= 0) {
                if (s >= this.n[n3] && n3 != 0) continue;
                for (int i = this.bT; i > n3 + 1; --i) {
                    this.n[i] = this.n[i - 1];
                    this.o[i] = this.o[i - 1];
                }
                this.n[++n3] = s;
                this.o[n3] = n + n2 * this.Y;
                break;
            }
            ++this.bT;
        }
    }

    private int a(int n, boolean bl) {
        int n2 = this.f(n);
        return a.a(n2, this.L, this.v[n -= 41], this.t[n], this.u[n], bl);
    }

    private int f(int n) {
        n = this.f[n];
        int n2 = 1;
        if ((n & 1) != 0 && this.c(23) >= 0) {
            n2 = 2;
        } else if ((n & 2) != 0 && this.c(24) >= 0) {
            n2 = 2;
        }
        return this.K * n2;
    }

    private static int a(int n, int n2, int n3, int n4, int n5, boolean bl) {
        int n6 = -1;
        int n7 = 0;
        if (n > n5) {
            n7 = n3 / (n - n5);
            if (n7 * (n - n5) < n3) {
                ++n7;
            }
            n6 = n7 > 1 && n4 > n2 ? (n4 - n2) * (n7 - 1) : 0;
            if (!bl && n4 > n2) {
                n6 += n4 - n2;
            }
        }
        return n6;
    }

    private void c(boolean bl) {
        if (!this.x) {
            if (this.S < 0) {
                this.l = 0;
                return;
            }
            int a2 = this.l[this.S];
            int n = this.f(a2) - this.u[a2 - 41];
            if (n > 0) {
                if ((this.bV & 3) == 0) {
                    this.bU -= n;
                    this.a((byte)(5 + this.bV % 3), n, this.g[this.S], this.h[this.S]);
                    if (this.bU > 0) {
                        n = this.t[a2 - 41] - this.L;
                        if (n > 0) {
                            this.J -= n;
                        }
                    } else {
                        boolean bl2;
                        n = this.a(a2, this.z);
                        if (n > 0) {
                            this.a((byte)2, n, this.N, this.O);
                        }
                        if (this.c(25) >= 0) {
                            this.c(this.s[a2 - 41] << 1, this.g[this.S], this.h[this.S]);
                        } else {
                            this.c(this.s[a2 - 41], this.g[this.S], this.h[this.S]);
                        }
                        n = this.l[this.S];
                        a a3 = this;
                        boolean bl3 = false;
                        if (a3.am > 50) {
                            bl2 = false;
                        } else {
                            int n2 = a3.x.length;
                            while (--n2 >= 0) {
                                if (n != a3.x[n2]) continue;
                                a3.o(n2);
                                break;
                            }
                            switch (n) {
                                case 73: {
                                    a3.g[a3.bh] = true;
                                    a3.d(22);
                                    if (a3.am != 40) break;
                                    a3.o(6);
                                    break;
                                }
                                case 74: {
                                    a3.d(21);
                                    break;
                                }
                                case 75: {
                                    if (!a3.b(20, true)) break;
                                    a3.d(20);
                                    bl3 = true;
                                }
                            }
                            bl2 = bl3;
                        }
                        this.y = bl2;
                        this.x = true;
                    }
                }
            } else {
                this.l = 0;
            }
            ++this.bV;
            return;
        }
        if (this.B == 0) {
            this.e(this.S);
        }
        if ((this.B = (byte)(this.B + 1)) > 5) {
            this.B = 0;
            if (!this.y && bl) {
                this.a(this.m);
            }
            this.x = false;
            int n = this.l[this.S];
            a a2 = this;
            switch (n) {
                case 70: 
                case 71: {
                    if (a2.am != 49) break;
                    a2.d(35);
                    break;
                }
                case 61: {
                    if (a2.am != 49 || a2.b(6, 2, 61) >= 0 || a2.b(5, 3, 61) >= 0 || a2.b(7, 3, 61) >= 0 || a2.b(6, 4, 61) >= 0 || a2.b(5, 2, 61) < 0 || a2.b(7, 2, 61) < 0 || a2.b(5, 4, 61) < 0 || a2.b(7, 4, 61) < 0) break;
                    a2.a(6, 3, 70);
                    a2.a(71, 192, 96, 0);
                    break;
                }
                case 72: {
                    if (a2.am != 50) break;
                    a2.d(65);
                    break;
                }
                case 69: {
                    if (a2.am != 35) break;
                    a2.o(7);
                    a2.d(36);
                }
            }
            this.S = -1;
        }
    }

    private void a(byte by, int n) {
        if (!this.D) {
            return;
        }
        this.a = null;
        try {
            if (this.C == by && a != null) {
                switch (a.getState()) {
                    case 400: {
                        a.stop();
                    }
                    case 300: {
                        try {
                            a.setLoopCount(-1);
                        }
                        catch (Exception exception) {}
                        a.start();
                        return;
                    }
                }
            }
            this.G();
            this.a = this.getClass().getResourceAsStream(this.p[by]);
            a = Manager.createPlayer((InputStream)this.a, (String)(this.p[by].endsWith("wav") ? "audio/x-wav" : "audio/midi"));
            a.realize();
            a.prefetch();
            try {
                a.setLoopCount(-1);
            }
            catch (Exception exception) {
                Exception exception2 = exception;
                exception.printStackTrace();
            }
            int n2 = this.bZ;
            a a2 = this;
            a2 = a;
            if (a2 != null && ((a2.getState() & 0x12C) == 300 || (a2.getState() & 0x190) == 400)) {
                try {
                    ((VolumeControl)a.getControl("VolumeControl")).setLevel(n2);
                }
                catch (Exception exception) {}
            }
            a.start();
            this.C = by;
            return;
        }
        catch (Exception exception) {
            Exception exception3 = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.b();
        }
    }

    private void G() {
        if (a != null) {
            a.close();
        }
        a = null;
        this.C = (byte)-1;
    }

    private int g(int n) {
        return (this.a.nextInt() >>> 1) % n;
    }

    private void a(String string, int n, int n2, int n3, int[] nArray) {
        this.a.setColor(nArray[0]);
        int n4 = --n;
        this.a.drawString(string, n4, --n2, 17);
        int n5 = ++n;
        this.a.drawString(string, n5, n2, 17);
        this.a.drawString(string, ++n, n2++, 17);
        this.a.drawString(string, n, n2++, 17);
        this.a.drawString(string, n--, n2, 17);
        this.a.drawString(string, n--, n2, 17);
        this.a.drawString(string, n, n2--, 17);
        this.a.drawString(string, n++, n2, 17);
        this.a.setColor(nArray[1]);
        this.a.drawString(string, n, n2, 17);
    }

    private void n(int n, int n2) {
        int n3;
        a a2;
        if (!this.E) {
            a2 = this;
            a2.a(0);
            n3 = a2.T[0] + a2.T[1] + a2.T[2];
            a2.cb = Math.min(15, Math.abs(320 - n3 >> 2));
            n3 = 320 - n3 - (a2.cb << 1) >> 1;
            a2.L[0] = n3 + (a2.T[0] >> 1);
            a2.L[1] = n3 + a2.T[0] + a2.cb;
            a2.L[2] = a2.L[1] + a2.T[1] + a2.cb;
            a2.cc = 240 - a2.a[0][7].getWidth() >> 1;
            this.E = true;
        }
        a2 = this;
        block33: for (n3 = a2.D - 1; n3 >= 0; --n3) {
            int[] nArray = a2.d[n3];
            switch (nArray[2]) {
                case 1: {
                    if (nArray[3] <= 0) continue block33;
                    nArray[0] = nArray[0] + (nArray[4] - nArray[0]) / nArray[3];
                    nArray[1] = nArray[1] + (nArray[5] - nArray[1]) / nArray[3];
                    nArray[3] = nArray[3] - 1;
                    continue block33;
                }
                case 3: {
                    if (nArray[3] > 0) {
                        int n4 = (nArray[4] - nArray[0]) / nArray[3];
                        int n5 = (nArray[5] - nArray[1]) / nArray[3];
                        nArray[0] = nArray[0] + n4;
                        nArray[1] = nArray[1] + n5;
                        nArray[3] = nArray[3] - 1;
                        if (nArray[3] != 0) continue block33;
                        int n6 = n4 > 0 ? 8 : (nArray[4] = n4 < 0 ? -8 : 0);
                        nArray[5] = n5 > 0 ? 8 : (n5 < 0 ? -8 : 0);
                        continue block33;
                    }
                    if (nArray[4] != 0) continue block33;
                    switch (nArray[3]) {
                        case -3: 
                        case 0: {
                            nArray[0] = nArray[0] + nArray[4];
                            nArray[1] = nArray[1] + nArray[5];
                            break;
                        }
                        case -2: {
                            nArray[4] = nArray[4] >> 1;
                            nArray[5] = nArray[5] >> 1;
                        }
                        case -1: {
                            nArray[0] = nArray[0] - nArray[4];
                            nArray[1] = nArray[1] - nArray[5];
                        }
                    }
                    nArray[3] = nArray[3] - 1;
                    continue block33;
                }
                case 2: {
                    if (nArray[4] <= 1) continue block33;
                    nArray[4] = nArray[4] - 1;
                    continue block33;
                }
                case 4: {
                    if (nArray[3] <= 0) continue block33;
                    switch (nArray[5]) {
                        case 0: {
                            nArray[0] = nArray[0] - nArray[4];
                            nArray[1] = nArray[1] + nArray[4];
                            break;
                        }
                        case 1: {
                            nArray[0] = nArray[0] - nArray[4];
                            nArray[1] = nArray[1] - nArray[4];
                            break;
                        }
                        case 2: {
                            nArray[0] = nArray[0] + nArray[4];
                            nArray[1] = nArray[1] - nArray[4];
                            break;
                        }
                        case 3: {
                            nArray[0] = nArray[0] + nArray[4];
                            nArray[1] = nArray[1] + nArray[4];
                        }
                    }
                    nArray[5] = nArray[5] + 1;
                    if (nArray[5] < 4) continue block33;
                    nArray[5] = 0;
                    nArray[3] = nArray[3] - 1;
                }
            }
        }
        if (n == 0 || n2 < 18) {
            switch (n2) {
                case 1: {
                    this.a(this.a[0][1], 120, 0, 1, 12, 4, 120, this.L[0], 0, -1);
                    break;
                }
                case 5: {
                    this.v(0);
                    this.a(this.a[0][0], 120, this.L[0], 2, 12, 4, 4, 0, 0, -1);
                    break;
                }
                case 8: {
                    this.a(this.a[0][5], 120, this.L[0] + 2, 0, 12, 0, 0, 0, 0, 0);
                    this.a(this.a[0][2], 90, this.L[1], 2, 4, 4, 4, 0, 0, -1);
                    this.a(this.a[0][3], 120, this.L[1], 2, 4, 4, 4, 0, 0, -1);
                    this.a(this.a[0][4], 150, this.L[1], 2, 4, 4, 4, 0, 0, -1);
                    this.a(this.a[0][6], 15, this.L[1], 1, 12, 3, this.cc, this.L[2], 0, -1);
                    break;
                }
                case 10: {
                    this.v(5);
                    this.a(null, this.cc, this.L[2], 3, 0, 0, 0, 0, 0, -1);
                    this.a(5, 4, 0, 1, 2, 0);
                    break;
                }
                case 14: {
                    this.a(5, 3, 0, 3, this.cc - 8, this.L[2]);
                    this.a(6, 3, 0, 3, this.cc + this.U[2] - 4, this.L[2]);
                    break;
                }
                case 15: {
                    this.a(7, 3, 0, 2, this.cc + this.U[4] - 2, this.L[2]);
                    break;
                }
                case 17: {
                    this.a(5, 3, 0, 3, this.cc, this.L[2]);
                    this.a(6, 3, 0, 3, this.cc + this.U[2], this.L[2]);
                    this.a(7, 3, 0, 2, this.cc + this.U[4], this.L[2]);
                }
            }
            if (n2 >= 10 && n2 < 13) {
                this.a(null, 240, this.L[2], 3, 0, 4, this.cc + this.U[n2 - 9 << 1], this.L[2], n2 - 9, -1);
                return;
            }
            if (n2 >= 14 && n2 <= 17) {
                this.a(null, this.cc + this.U[n2 - 10 << 1], 320, 3, 0, 4, this.cc + this.U[n2 - 10 << 1], this.L[2], n2 - 10, -1);
                return;
            }
            if (n2 >= 25 && n2 <= 31) {
                n = (n2 - 25 << 1) + 8;
                this.a(null, this.cc + this.U[n << 1], this.L[2], 3, 0, 0, this.cc + this.U[n << 1], this.L[2], n, -1);
                if (n2 < 31) {
                    this.a(null, this.cc + this.U[++n << 1], this.L[2], 3, 0, 0, this.cc + this.U[n << 1], this.L[2], n, -1);
                }
                return;
            }
        } else {
            if (n2 >= 18 && n2 < 31) {
                this.a(null, this.cc + this.U[n2 - 10 << 1], 320, 3, 0, 4, this.cc + this.U[n2 - 10 << 1], this.L[2], n2 - 10, -1);
            }
            switch (n2) {
                case 50: {
                    this.a(2, 2, 4, 4, 4, 1);
                    break;
                }
                case 51: {
                    this.a(3, 2, 4, 4, 4, 1);
                    break;
                }
                case 52: {
                    this.a(4, 2, 4, 4, 4, 1);
                    break;
                }
                case 58: {
                    this.a(0, 1, 0, 1, 1000, 1000);
                    this.a(1, 2, 4, 4, 4, 1);
                    break;
                }
                case 63: {
                    this.a(15, 3, 0, 4, -10, this.L[2]);
                }
            }
            if (n2 > 52 && n2 < 63) {
                n = n2 - 52;
                this.a(n + 4, 3, 0, 4, -10, this.L[2]);
                this.a(26 - n, 3, 0, 4, 240, this.L[2]);
            }
        }
    }

    private void H() {
        this.a.setColor(0xFFFFFF);
        this.a.fillRect(0, 0, 240, 320);
        for (int i = 0; i < this.D; ++i) {
            int[] nArray = this.d[i];
            int n = nArray[0];
            int n2 = nArray[1];
            if (nArray[2] == 3 || nArray[2] == 4) {
                this.a.setClip((n -= this.U[nArray[6] << 1]) + this.U[nArray[6] << 1], n2, (int)this.U[(nArray[6] << 1) + 1], 320);
                this.a.drawImage(this.a[0][7], n, n2, 0);
            } else if (nArray[2] == 2) {
                int n3 = nArray[3] - nArray[4];
                if (nArray[5] == 1) {
                    n3 = nArray[3] - n3 - 2;
                }
                this.a.setClip((n -= nArray[6] * n3) + nArray[6] * n3, n2, nArray[6], 320);
            }
            if (nArray[2] != 3 && nArray[2] != 4) {
                this.a.drawImage(this.b[i], n, n2, 0);
            }
            this.a.setClip(0, 0, 240, 320);
        }
    }

    private void a(Image image, int n, int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9) {
        if (this.b == null) {
            this.b = new Image[26];
        }
        if (this.D < 26) {
            int[] nArray = this.d[this.D];
            if (n9 >= 0 && n9 < this.D) {
                this.d[this.D] = this.d[n9];
                this.b[this.D] = this.b[n9];
                this.b[n9] = image;
                this.d[n9] = nArray;
                nArray = this.d[n9];
            } else {
                this.b[this.D] = image;
            }
            if (n3 == 3) {
                nArray[6] = n8;
            } else {
                n8 = image.getWidth();
                int n10 = image.getHeight();
                if (n3 == 2) {
                    nArray[6] = n8 /= n5;
                }
                n9 = 0;
                int n11 = 0;
                if ((n4 & 1) != 0) {
                    n9 = 0 - n8;
                } else if ((n4 & 4) != 0) {
                    n9 = 0 - (n8 >> 1);
                }
                if ((n4 & 2) != 0) {
                    n11 = 0 - (n10 - 1);
                } else if ((n4 & 8) != 0) {
                    n11 = 0 - ((n10 >> 1) - 1);
                }
                n += n9;
                n2 += n11;
                if (n3 == 1) {
                    n6 += n9;
                    n7 += n11;
                }
            }
            nArray[0] = n;
            nArray[1] = n2;
            nArray[2] = n3;
            nArray[3] = n5;
            nArray[4] = n6;
            nArray[5] = n7;
            this.D = (byte)(this.D + 1);
        }
    }

    private void a(int n, int n2, int n3, int n4, int n5, int n6) {
        if (n < this.D) {
            int[] nArray = this.d[n];
            if (n2 == 1) {
                int n7 = this.b[n].getWidth();
                int n8 = this.b[n].getHeight();
                if ((n3 & 1) != 0) {
                    n5 -= n7;
                } else if ((n3 & 4) != 0) {
                    n5 -= n7 >> 1;
                }
                if ((n3 & 2) != 0) {
                    n6 -= n8 - 1;
                } else if ((n3 & 8) != 0) {
                    n6 -= (n8 >> 1) - 1;
                }
            }
            nArray[2] = n2;
            nArray[3] = n4;
            nArray[4] = n5;
            nArray[5] = n6;
        }
    }

    private void v(int n) {
        if (n >= 0) {
            if (n < this.D) {
                int[] nArray = this.d[n];
                this.D = (byte)(this.D - 1);
                this.d[n] = this.d[this.D];
                this.d[this.D] = nArray;
                this.b[n] = null;
                this.b[n] = this.b[this.D];
                this.b[this.D] = null;
                return;
            }
        } else {
            this.D = 0;
            this.b = null;
        }
    }

    private void I() {
        if (this.a != null) {
            try {
                this.a.close();
            }
            catch (Exception exception) {}
            this.a = null;
        }
        if (this.a != null) {
            try {
                this.a.close();
            }
            catch (Exception exception) {}
            this.a = null;
        }
        if (this.b != null) {
            try {
                this.b.close();
            }
            catch (Exception exception) {}
            this.b = null;
        }
        System.gc();
    }

    private int c() {
        return (this.W[this.cg++] & 0xFF) << 8 | this.W[this.cg++] & 0xFF;
    }

    private int d() {
        return this.c() << 16 | this.c() & 0xFFFF;
    }

    private int e() {
        int n = 0;
        try {
            String string;
            Object object;
            String string2;
            String string3 = string2 = this.g == null ? this.r[0] : this.g;
            Object object2 = this;
            if (((a)object2).H) {
                int n2 = string3.indexOf(47, 7);
                object = string3.substring(n2);
                string = "http://10.0.0.172:80" + (String)object;
            } else {
                string = string3;
            }
            object = string;
            object2 = string2;
            int n3 = ((String)object2).indexOf(47, 7);
            string2 = n3 < 7 ? ((String)object2).substring(7, ((String)object2).length()) : ((String)object2).substring(7, n3);
            this.a = (HttpConnection)Connector.open((String)object);
            this.a.setRequestMethod("POST");
            this.a.setRequestProperty("Content-Type", "application/octet-stream");
            if (this.H) {
                this.a.setRequestProperty("X-Online-Host", string2);
            }
            this.a.setRequestProperty("Content-Length", "" + 0);
            if (this.V != null) {
                this.a = this.a.openOutputStream();
                this.a.write(this.V, 0, 0);
                this.a.flush();
            }
            this.ce = this.a.getResponseCode();
            if (this.ce >= 300 && this.ce < 400) {
                this.h = "\u91cd\u5b9a\u5411!!!";
                this.g = this.a.getHeaderField("Location");
                n = 1;
            } else if (this.ce / 100 != 2) {
                n = -1;
            } else {
                n = 0;
                while (true) {
                    object = null;
                    object2 = this.a.getHeaderFieldKey(n);
                    if (object2 == null) break;
                    this.a.getHeaderField(n);
                    ++n;
                }
                this.b = this.a.openInputStream();
                n = (int)this.a.getLength();
                if (n > 0) {
                    this.W = null;
                    this.W = new byte[n];
                    int n4 = 0;
                    for (int i = 0; i < n; ++i) {
                        n4 = this.b.read();
                        this.W[i] = (byte)n4;
                    }
                } else {
                    object = new ByteArrayOutputStream(2048);
                    boolean bl = false;
                    byte[] byArray = new byte[64];
                    while ((n = this.b.read(byArray, 0, 64)) >= 0) {
                        ((ByteArrayOutputStream)object).write(byArray, 0, n);
                    }
                    ((ByteArrayOutputStream)object).close();
                    this.W = ((ByteArrayOutputStream)object).toByteArray();
                }
                this.b.close();
                this.b = null;
                n = this.W.length;
                Object object3 = new byte[n >> 1];
                int n5 = 0;
                while (n5 + 1 < n) {
                    object3[n5 >> 1] = a.h(this.W[n5]) << 4 | a.h(this.W[n5 + 1]);
                    n5 += 2;
                }
                this.W = null;
                this.W = object3;
                object3 = new StringBuffer();
                n >>= 1;
                for (n5 = 0; n5 < n; ++n5) {
                    ((StringBuffer)object3).append(Integer.toHexString(this.W[n5] & 0xFF));
                    ((StringBuffer)object3).append(" ");
                }
                System.out.println("\u4e0b\u884c\u6570\u636e\u957f\u5ea6\uff1a" + n + "\uff0c\u5185\u5bb9\uff1a" + object3);
                n = 0;
            }
        }
        catch (Exception exception) {
            n = -1;
        }
        finally {
            this.I();
        }
        return n;
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    private int f() {
        int n;
        Object object = this.a;
        synchronized (object) {
            n = this.cf;
        }
        if (n == 2) {
            Thread.yield();
        }
        return n;
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    private boolean b() {
        ++this.cj;
        switch (this.f()) {
            case 0: {
                this.cj = 0;
                this.ck = 0;
                this.cj = 0;
                try {
                    if (this.ce == 200) {
                        a a2 = this;
                        long l = 0L;
                        a2.cg = 0;
                        a a3 = a2;
                        l = a3.d() << 32 | a3.d();
                        if (l > 0L) {
                            a2.ci = 0;
                            a2.b = a2.a;
                            a2.a = l;
                            a2.ch = a2.W[a2.cg++];
                            for (int i = 0; i < a2.ch; ++i) {
                                a3 = a2;
                                ++a3.cg;
                                int n = a3.c();
                                if (n == 0) continue;
                                a3.cg += n;
                            }
                        } else {
                            if (++a2.ci >= 2 && a2.b != a2.a) {
                                a2.a = a2.b;
                            }
                            Object object = a2.a;
                            synchronized (object) {
                                a2.cf = -1;
                            }
                            a2.h = a2.q[48];
                        }
                    }
                }
                catch (Exception exception) {
                    Exception exception2 = exception;
                    exception.printStackTrace();
                }
                finally {
                    this.I();
                }
                return true;
            }
            case -1: {
                if (this.ck < 3) {
                    ++this.ck;
                    this.cj = 0;
                    this.h = "\u6b63\u5728\u91cd\u8bd5(" + this.ck + ")";
                    a a4 = this;
                    a4.I();
                    a4.ce = 0;
                    a4.cj = 0;
                    if (a4.a == null) {
                        a4.a = new Object();
                    }
                    Object object = a4.a;
                    synchronized (object) {
                        a4.cf = 2;
                    }
                    a4.F = true;
                    Thread thread = new Thread(a4);
                    thread.start();
                    break;
                }
                this.ck = 0;
                this.h = this.q[15];
                this.H = !this.H;
                return true;
            }
        }
        if (this.cj > 300) {
            this.cf = 3;
            this.cj = 0;
            this.h = "\u5df2\u8d85\u65f6\uff0c\u8bf7\u91cd\u8bd5\u3002";
            this.H = !this.H;
            return true;
        }
        return false;
    }

    private static int h(int n) {
        if (n >= 48 && n <= 57) {
            return n - 48;
        }
        if (n >= 97 && n <= 122) {
            return n - 97 + 10;
        }
        if (n >= 65 && n <= 90) {
            return n - 65 + 10;
        }
        return -n;
    }

    static {
        int[] nArray = new int[]{0, 2, 1, 3, 4, 5, 6, 7};
    }
}
