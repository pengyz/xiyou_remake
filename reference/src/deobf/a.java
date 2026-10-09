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

public final class a
extends Canvas
implements Runnable {
    private Font f_Font_00 = Font.getFont((int)0, (int)0, (int)8);
    private int f_int_00 = this.f_Font_00.charWidth('\u56fd');
    private int f_int_01 = this.f_Font_00.getHeight();
    private int f_int_02 = 75;
    private String f_String_00 = "$Rev: 3289 $";
    private String[] f_String_arr_00;
    private boolean f_bool_00;
    private Graphics f_Graphics_00;
    private boolean f_bool_01;
    private int f_int_03;
    private byte gameMode;
    private byte f_byte_01;
    private byte f_byte_02;
    private DirectGraphics f_DirectGraphics_00;
    private int f_int_04;
    private Image f_Image_00;
    private boolean f_bool_02;
    private String[] f_String_arr_01;
    private int[] f_int_arr_00;
    private Image[][] f_Image_arr2_00;
    private static int[] f_int_arr_01 = new int[]{0, 8192, 16384, 24576, 8462, 270, 90, 8282};
    private int[][] f_int_arr2_00;
    private InputStream f_InputStream_00;
    private OutputStream f_OutputStream_00;
    private InputStream f_InputStream_01;
    private int f_int_05;
    private int f_int_06;
    private String[] f_String_arr_02;
    private int f_int_07;
    private int f_int_08;
    private int f_int_09;
    private byte[] f_byte_arr_00;
    private int f_int_10;
    private int f_int_11;
    private int f_int_12;
    private int f_int_13;
    private int f_int_14;
    private int f_int_15;
    private int f_int_16;
    private byte[] f_byte_arr_01;
    private byte[] f_byte_arr_02;
    private byte f_byte_03;
    private int f_int_17;
    private int f_int_18;
    private Image f_Image_01;
    private int f_int_19;
    private int f_int_20;
    private String f_String_01;
    private int f_int_21;
    private boolean f_bool_03;
    private byte f_byte_04;
    private int[] f_int_arr_02;
    private int f_int_22;
    private int f_int_23;
    private boolean f_bool_04;
    private boolean f_bool_05;
    private byte f_byte_05;
    private byte f_byte_06;
    private byte f_byte_07;
    private byte f_byte_08;
    private int f_int_24;
    private int f_int_25;
    private String f_String_02;
    private String f_String_03;
    private String[] f_String_arr_03;
    private int f_int_26;
    private int f_int_27;
    private int f_int_28;
    private int f_int_29;
    private int f_int_30;
    private int f_int_31;
    private int f_int_32;
    private int f_int_33;
    private int f_int_34;
    private int playerHp;
    private int playerAtk;
    private int playerDef;
    private byte equippedWeaponType;
    private byte equippedArmorType;
    private int[][] f_int_arr2_01;
    private int[] f_int_arr_03;
    private int f_int_38;
    private byte f_byte_11;
    private int playerPixelX;
    private int playerPixelY;
    private int playerCellX;
    private int playerCellY;
    private byte f_byte_12;
    private int f_int_43;
    private int f_int_44;
    private Image f_Image_02;
    private int[] f_int_arr_04;
    private int[] f_int_arr_05;
    private boolean f_bool_06;
    private boolean f_bool_07;
    private boolean f_bool_08;
    private byte f_byte_13;
    private byte f_byte_14;
    private String[] f_String_arr_04;
    private int[] entityPixelX;
    private int[] entityPixelY;
    private int[] f_int_arr_08;
    private int[] f_int_arr_09;
    private int[] f_int_arr_10;
    private Image[] f_Image_arr_00;
    private int[] entityType;
    private int[] f_int_arr_12;
    private int f_int_45;
    private boolean[] f_bool_arr_00;
    private boolean[] f_bool_arr_01;
    private byte[] f_byte_arr_03;
    private short[] entityParam;
    private boolean[] f_bool_arr_02;
    private byte[] f_byte_arr_04;
    private byte[] f_byte_arr_05;
    private int[] f_int_arr_13;
    private int[][] f_int_arr2_02;
    private byte[] f_byte_arr_06;
    private byte[] f_byte_arr_07;
    private boolean[] f_bool_arr_03;
    private int f_int_46;
    private boolean f_bool_09;
    private int f_int_47;
    private int f_int_48;
    private byte[][] f_byte_arr2_00;
    private byte[][] f_byte_arr2_01;
    private int f_int_49;
    private Image f_Image_03;
    private int mapCellsWide;
    private int mapCellsHigh;
    private int f_int_52;
    private int f_int_53;
    private byte[] mapTerrainGrid;
    private byte[] mapTransformGrid;
    private boolean[][] f_bool_arr2_00;
    private byte[][] f_byte_arr2_02;
    private byte[] f_byte_arr_10;
    private byte[][] f_byte_arr2_03;
    private byte f_byte_15;
    private boolean[] f_bool_arr_04;
    private int f_int_54;
    private int f_int_55;
    private int f_int_56;
    private int f_int_57;
    private int f_int_58;
    private int f_int_59;
    private int f_int_60;
    private int f_int_61;
    private int f_int_62;
    private int f_int_63;
    private boolean f_bool_10;
    private boolean f_bool_11;
    private int currentFloor;
    private int minFloorReached;
    private int maxFloorReached;
    private int f_int_67;
    private int f_int_68;
    private int alchemyUpgradeCount;
    private int f_int_70;
    private int f_int_71;
    private int f_int_72;
    private int f_int_73;
    private int f_int_74;
    private int f_int_75;
    private int alchemyPrice;
    private int f_int_77;
    private int f_int_78;
    private int f_int_79;
    private int f_int_80;
    private String[] f_String_arr_05;
    private int f_int_81;
    private byte[] f_byte_arr_11;
    private boolean[] f_bool_arr_05;
    private int f_int_82;
    private int f_int_83;
    private int f_int_84;
    private int f_int_85;
    private short[] f_short_arr_01;
    private short[] f_short_arr_02;
    private short[] f_short_arr_03;
    private int f_int_86;
    private String[] f_String_arr_06;
    private String[] f_String_arr_07;
    private byte[] f_byte_arr_12;
    private String[] f_String_arr_08;
    private String[] f_String_arr_09;
    private byte[] f_byte_arr_13;
    private int f_int_87;
    private int f_int_88;
    private byte[] f_byte_arr_14;
    private byte[] f_byte_arr_15;
    private int f_int_89;
    private int f_int_90;
    private String f_String_04;
    private String[] itemDescriptions;
    private String[] equipDescriptions;
    private byte[] itemUseCounts;
    private int yellowKeyCount;
    private int blueKeyCount;
    private int redKeyCount;
    private int goldAmount;
    private int itemStackSize;
    private byte[] itemStackTypes;
    private byte[] itemStackUses;
    private int f_int_96;
    private int[] equipTierBonuses;
    private byte[] equipTierTypes;
    private int f_int_97;
    private int f_int_98;
    private int f_int_99;
    private int f_int_100;
    private int f_int_101;
    private int f_int_102;
    private int f_int_103;
    private int f_int_104;
    private boolean f_bool_12;
    private boolean f_bool_13;
    private String[] objectTypeNames;
    private int[] enemyBaseHp;
    private int[] enemyBaseAtk;
    private int[] enemyBaseDef;
    private int[] enemyBaseGold;
    private int[] enemyAtkScaled;
    private int[] enemyDefScaled;
    private int[] enemyHpScaled;
    private int f_int_105;
    private int f_int_106;
    private int f_int_107;
    private int f_int_108;
    private int f_int_109;
    private int f_int_110;
    private int[] f_int_arr_22;
    private byte[] f_byte_arr_20;
    private int currentScriptIndex;
    private String[] levelScriptLines;
    private byte[] dialogueSpeakerType;
    private byte[] f_byte_arr_22;
    private String[] dialogueTexts;
    private int f_int_112;
    private int f_int_113;
    private boolean[] f_bool_arr_06;
    private int f_int_114;
    private int f_int_115;
    private byte f_byte_16;
    private String f_String_05;
    private int scriptCursor;
    private int f_int_117;
    private int f_int_118;
    private byte f_byte_17;
    private int f_int_119;
    private int f_int_120;
    private int f_int_121;
    private int f_int_122;
    private int f_int_123;
    private int f_int_124;
    private int f_int_125;
    private int f_int_126;
    private int f_int_127;
    private byte f_byte_18;
    private byte f_byte_19;
    private int f_int_128;
    private int f_int_129;
    private int f_int_130;
    private boolean f_bool_14;
    private short[] f_short_arr_04;
    private byte[][] bossEventSpawns;
    private byte[] bossTypeOrder;
    private boolean f_bool_15;
    private boolean[] f_bool_arr_07;
    private byte[] f_byte_arr_24;
    private int[] f_int_arr_23;
    private int[] f_int_arr_24;
    private int[] f_int_arr_25;
    private int[] f_int_arr_26;
    private int[] f_int_arr_27;
    private int[] f_int_arr_28;
    private int[] f_int_arr_29;
    private byte[] f_byte_arr_25;
    private byte[] f_byte_arr_26;
    private int f_int_131;
    private int f_int_132;
    private int f_int_133;
    private int f_int_134;
    private int f_int_135;
    private int f_int_136;
    private RecordStore f_RecordStore_00;
    private RecordEnumeration f_RecordEnumeration_00;
    private ByteArrayOutputStream f_ByteArrayOutputStream_00;
    private DataOutputStream f_DataOutputStream_00;
    private DataInputStream f_DataInputStream_00;
    private byte[] f_byte_arr_27;
    private byte[][] f_byte_arr2_05;
    private int f_int_137;
    private byte[] f_byte_arr_28;
    private int f_int_138;
    private byte[] f_byte_arr_29;
    private short[] f_short_arr_05;
    private short[] f_short_arr_06;
    private short[] f_short_arr_07;
    private short[] f_short_arr_08;
    private byte f_byte_20;
    private byte f_byte_21;
    private byte[] f_byte_arr_30;
    private int[] f_int_arr_30;
    private short[] f_short_arr_09;
    private short[] f_short_arr_10;
    private byte[] f_byte_arr_31;
    private int f_int_139;
    private int f_int_140;
    private boolean[] f_bool_arr_08;
    private byte[] f_byte_arr_32;
    private byte[] f_byte_arr_33;
    private byte[] f_byte_arr_34;
    private byte[] f_byte_arr_35;
    private byte[] f_byte_arr_36;
    private byte[] f_byte_arr_37;
    private short[] f_short_arr_11;
    private short[] f_short_arr_12;
    private byte[] f_byte_arr_38;
    private byte[] f_byte_arr_39;
    private byte[] f_byte_arr_40;
    private byte f_byte_22;
    private int[] f_int_arr_31;
    private boolean[] f_bool_arr_09;
    private boolean[] f_bool_arr_10;
    private int f_int_141;
    private boolean f_bool_16;
    private boolean f_bool_17;
    private int f_int_142;
    private byte f_byte_23;
    private boolean f_bool_18;
    private byte[] f_byte_arr_41;
    private int f_int_143;
    private int f_int_144;
    private int f_int_145;
    private byte f_byte_24;
    private boolean f_bool_19;
    private int f_int_146;
    private int f_int_147;
    private int[] f_int_arr_32;
    private byte f_byte_25;
    private byte[] f_byte_arr_42;
    private int f_int_148;
    private byte[][] f_byte_arr2_06;
    private short[][] f_short_arr2_00;
    private short f_short_00;
    private short f_short_01;
    private short f_short_02;
    private short f_short_03;
    private short[] f_short_arr_13;
    private short[] f_short_arr_14;
    private int f_int_149;
    private boolean f_bool_20;
    private boolean f_bool_21;
    private boolean f_bool_22;
    private byte f_byte_26;
    private int[] difficultyMultipliers;
    private int f_int_150;
    private int f_int_151;
    private boolean f_bool_23;
    private byte f_byte_27;
    private boolean f_bool_24;
    private boolean f_bool_25;
    private byte[] f_byte_arr_43;
    private byte[] f_byte_arr_44;
    private boolean f_bool_26;
    private boolean f_bool_27;
    private int[] f_int_arr_34;
    private int[] f_int_arr_35;
    private int f_int_152;
    private int f_int_153;
    private int f_int_154;
    private boolean f_bool_28;
    private String[] f_String_arr_15;
    private int f_int_155;
    private boolean f_bool_29;
    private byte f_byte_28;
    private static Player f_Player_00;
    private Random gameRandom;
    private final int[] f_int_arr_36;
    private int f_int_156;
    private final byte[] f_byte_arr_45;
    private int f_int_157;
    private int f_int_158;
    private int[] f_int_arr_37;
    private boolean f_bool_30;
    private final byte[] f_byte_arr_46;
    private byte f_byte_29;
    private int[][] f_int_arr2_03;
    private Image[] f_Image_arr_01;
    private String[] f_String_arr_16;
    private String[] f_String_arr_17;
    private String f_String_06;
    private int f_int_159;
    private byte[] f_byte_arr_47;
    private int f_int_160;
    private Object f_Object_00;
    private int f_int_161;
    private boolean f_bool_31;
    private boolean f_bool_32;
    private String f_String_07;
    private int f_int_162;
    private long f_long_00;
    private long f_long_01;
    private int f_int_163;
    private int f_int_164;
    private HttpConnection f_HttpConnection_00;
    private boolean f_bool_33;
    private byte[] f_byte_arr_48;
    private int f_int_165;
    private int f_int_166;

    public a() {
        new StringBuffer().append("(\u5185\u90e8\u7248\u672c").append(this.f_String_00.substring(4, this.f_String_00.length() - 1)).append(")");
        this.f_String_arr_00 = new String[]{"\u65e0\u6cd5\u518d\u4e0b\u4e00\u5c42\u4e86\uff0c\u8fd9\u662f\u4f60\u8fbe\u5230\u7684\u6700\u5e95\u5c42", "\u65e0\u6cd5\u518d\u4e0a\u4e00\u5c42\u4e86\uff0c\u8fd9\u662f\u4f60\u8fbe\u5230\u7684\u6700\u9ad8\u5c42", "\u65e0\u6cd5\u518d\u4e0b\u53bb\u4e86\u3002", "\u65e0\u6cd5\u518d\u4e0a\u53bb\u4e86\u3002", "\u4ffa\uff0c\u5f53\u4e16\u795e\u754c\u7b2c\u4e00\u6597\u8005\uff0c\u5b59!\u609f!\u7a7a! \u81ea\u4ece\u53d7\u5c01\u4e3a\u9f50\u5929\u5927\u5723\uff0c\u638c\u7ba1\u87e0\u6843\u56ed\u4ee5\u6765\uff0c\u4e00\u76f4\u900d\u9065\u5feb\u6d3b\uff0c\u65e0\u62d8\u675f\u2026\u2026", "\u4f60\u62e5\u6709\u66f4\u5f3a\u529b\u7684\u88c5\u5907\uff0c\u56e0\u6b64\u5c06\u6361\u5230\u7684\u4e22\u5f03\u4e86\u3002", "\u76f4\u5230\u90a3\u4e00\u5929\uff0c\u9047\u5230\u4e86\u5979\uff0c\u5728\u7b4b\u6597\u4e91\u4e0a\u7684\u6211\uff0c\u7adf\u7136\u7b2c\u4e00\u6b21\u5fc3\u6f6e\u8d77\u4f0f\uff0c\u6709\u4e86\u6655\u673a\u7684\u611f\u89c9\u2026\u2026", "\u795e\u4ed9\u52a8\u4e86\u611f\u60c5\uff0c\u5f80\u5f80\u4f1a\u4e07\u52ab\u4e0d\u590d\uff0c\n\u8fd9\u4e00\u6b21\uff0c\u8ba9\u6211\u4ed8\u51fa\u4e86\u4e94\u767e\u5e74\u7684\u65f6\u95f4\u53bb\u5fd8\u8bb0\u5979\u2026\u2026\n\u4e94\u6307\u5c71\u811a\u4e0b\u7684\u6c99\u5b50\uff0c\u63a0\u8fc7\u6211\u7684\u8138\u5e9e\u3002\n\u6c99\u5b50\uff0c\u8ddf\u65f6\u95f4\u4e00\u6837\uff0c\u540c\u6837\u968f\u98ce\u6d41\u901d\uff1b\u540c\u6837\u63a9\u57cb\u8fc7\u53bb\uff1b\n\u591a\u5c11\u6b21\u4f38\u624b\u60f3\u6293\u4f4f\uff0c\u5374\u4ece\u6307\u9699\u6e9c\u8d70\u2026\u2026\n\u770b\u591c\u7a7a\uff0c\u534a\u68a6\u534a\u9192\u95f4\uff0c\u5f80\u4e8b\u5386\u5386\u4e0a\u5fc3\u5934\u2026\u2026"};
        this.f_int_04 = 0;
        this.f_Image_00 = null;
        this.f_String_arr_01 = new String[]{"sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1", "ui", "xtq", "menu", "intro", "face", "sptenemy2", "end", "load"};
        this.f_int_arr_00 = new int[]{8, 1, 12, 4, 13, 23, 10, 20, 25, 6, 2, 2, 12, 20, 1, 2};
        this.f_Image_arr2_00 = new Image[this.f_int_arr_00.length][];
        this.f_int_arr2_00 = new int[][]{{0, 1, 2}, {3, 5}, {4, 5}};
        this.f_OutputStream_00 = null;
        this.f_InputStream_01 = null;
        this.f_String_arr_02 = new String[]{"\u65b0\u6e38\u620f", "\u7ee7\u7eed\u6e38\u620f", "\u8f7d\u5165\u8fdb\u5ea6", "\u4fdd\u5b58\u6e38\u620f", "\u8bbe\u7f6e", "\u5e2e\u52a9", "\u5173\u4e8e", "\u9000\u51fa", "\u8fd4\u56de\u83dc\u5355", "\u56de\u653e", "\u505c\u6b62\u56de\u653e", "\u5546\u5e97", "\u5341\u5168\u5927\u8865\u5305", "\u653b\u9632\u795e\u6cb9", "\u5f00\u95e8\u5929\u5929\u4e50", "\u5929\u5ead\u6d88\u8d39\u5238", "\u5370\u5ea6\u795e\u8840\u6cb9", "\u8df3\u8fc7\u6559\u7a0b"};
        this.f_int_16 = 25;
        this.f_byte_arr_01 = new byte[]{-1, 0, 1, 1, 1, 0, -1, -1};
        this.f_byte_arr_02 = new byte[]{-1, -1, -1, 0, 1, 1, 1, 0};
        this.f_byte_03 = 0;
        this.f_int_17 = 0;
        this.f_int_18 = 0;
        this.f_int_arr_02 = new int[]{0xF8F8F8, 0xAAAAAA, 0x888888, 0x444444, 0x111111};
        int[] nArray = new int[]{0, 12, 24, 36, 47, 57, 67, 75, 83, 89, 94, 97, 100, 97, 94, 89, 83, 75, 67, 57, 47, 36, 24, 12, 0};
        this.f_byte_07 = 0;
        this.f_String_arr_03 = new String[50];
        this.f_int_34 = this.f_int_01 + 4;
        this.playerHp = 498;
        this.playerAtk = 10;
        this.playerDef = 10;
        this.equippedWeaponType = 0;
        this.equippedArmorType = 0;
        this.f_int_arr2_01 = new int[][]{{0, 1, 0, 2}, new int[0], new int[0], new int[0]};
        this.playerPixelX = 192;
        this.playerPixelY = 352;
        this.playerCellX = 6;
        this.playerCellY = 11;
        this.f_int_arr_04 = new int[4];
        this.f_int_arr_05 = new int[4];
        this.f_String_arr_04 = new String[]{"\u65e0", "\u6728", "\u94c1", "\u94f6", "\u91d1", "\u5e03", "\u76ae", "\u9501", "\u91d1"};
        this.f_int_arr2_02 = new int[][]{{0}, {0, 1, 0, 2}, {0, 1, 2}, {0, 1, 2, 1}, {0, 1, 2, 2, 1, 0}, {0, 1, 2, 3, 2, 1}, {0, 1, 2, 3, 4}, {3, 4, 5, 6}, {4, 3, 2, 1, 0}, {2, 1, 0}, {2, 3, 4}};
        this.f_byte_arr_06 = new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 4, 2, 3, 3, 3, 3, 3, 3, 2, 3, 2, 2, 2, 3, 2, 3, 3, 3, 2, 3, 5, 3, 3, 3, 3, 3, 3, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0};
        this.f_byte_arr_07 = new byte[]{32, 32, 32, 32, 45, 32, 32, 32, 32, 104, 35, 45, 30, 32, 32, 30, 30, 24, 25, 21, 32, 25, 25, 27, 31, 27, 23, 23, 23, 32, 32, 32, 32, 27, 27, 27, 27, 22, 22, 16, 25, 15, 20, 28, 26, 23, 34, 35, 24, 33, 22, 22, 28, 36, 38, 29, 34, 29, 30, 31, 33, 26, 30, 36, 23, 38, 33, 42, 32, 78, 38, 36, 32, 39, 59, 36, 32, 29, 37, 27, 17, 32, 32, 32, 29, 27, 32, 36};
        this.f_bool_arr_03 = new boolean[]{true, false, false, false, false, true, true, true, true, false, true, false, false};
        this.f_int_47 = 0;
        this.f_int_48 = 270;
        this.f_byte_arr2_00 = new byte[][]{{0, 0, 0, 33, 56}, {0, 33, 0, 33, 56}, {0, 66, 0, 33, 56}, {2, 0, 0, 18, 26}, {3, 0, 0, 18, 34}, {1, 0, 0, 55, 61}, {4, 0, 0, 14, 26}, {4, 14, 0, 14, 26}, {4, 28, 0, 24, 26}};
        this.f_byte_arr2_01 = new byte[][]{{6, -1, -79, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -53, 0, 0, -16, -27, 0}, {7, -1, -80, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -54, 0, 2, -16, -26, 0}, {8, -1, -81, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -55, 0, 1, -16, -25, 0}};
        int[] nArray2 = new int[]{0, 0, 11, 1, 1, 11, 11, 11, 0, 0};
        int[] nArray3 = new int[]{0, 0, 0, 0, 1, 1, 1, 11, 11, 11};
        this.f_bool_arr_04 = new boolean[]{false, true, false, true, true, true, true, false, true, false, false, true, true, true, true, false, true, true, true, true, true, true, false, false, true, true, true, true, true, true, false, false, false, false, false, false, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, true, true, true, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false};
        this.currentFloor = 0;
        this.f_int_67 = 0;
        this.f_int_68 = 55;
        this.f_int_74 = this.f_int_01 + 90;
        this.f_int_75 = 115;
        this.f_String_arr_05 = new String[]{"\u58f0\u97f3", "\u5c0f\u5730\u56fe"};
        this.f_int_81 = this.f_int_01 + 8;
        this.f_byte_arr_11 = new byte[4];
        this.f_bool_arr_05 = new boolean[4];
        this.f_int_82 = 0;
        this.f_int_83 = 0;
        this.f_int_84 = 80;
        this.f_int_85 = 32 + this.f_int_81;
        this.f_short_arr_01 = new short[10];
        this.f_short_arr_02 = new short[10];
        this.f_short_arr_03 = new short[10];
        this.f_int_86 = 0;
        this.f_String_arr_06 = new String[]{"", "", "\u6709\u4e9b\u95e8\u4e0d\u80fd\u7528\u94a5\u5319\u6253\u5f00\uff0c\u53ea\u6709\u5f53\u4f60\u6253\u8d25\u5b83\u7684\u5b88\u536b\u540e\u624d\u4f1a\u81ea\u52a8\u6253\u5f00\u3002", "\u4f60\u8d2d\u4e70\u4e86\u793c\u7269\u540e\u518d\u4e0e\u5929\u5bab\u5546\u4eba\u5bf9\u8bdd\uff0c\u4ed6\u4f1a\u544a\u8bc9\u4f60\u4e00\u4e9b\u91cd\u8981\u7684\u6d88\u606f\u3002", "", "\u6211\u542c\u8bf4\u5728\u5929\u5bab\u4e2d\u67092\u628a\u9690\u85cf\u7684\u7ea2\u94a5\u5319\u3002", "\u5728\u8fd9\u4e2a\u533a\u57df\u4e0d\u591a\u6b21\u63d0\u5347\u653b\u51fb\u529b\uff0c\u5c31\u4e0d\u80fd\u6253\u8d25\u201c\u6768\u622c\u526f\u624b\u201d\u3002\u5207\u8bb0\u524d\u4eba\u6559\u8bad\uff01", "\u592a\u4e0a\u8001\u541b\u5c31\u572825\u697c\u3002\u4ee5\u4f60\u73b0\u5728\u7684\u72b6\u6001\u53bb\u653b\u51fb\u4ed6\u7b80\u76f4\u5c31\u662f\u81ea\u6740\u3002 \u4f60\u5e94\u5f53\u5728\u53d6\u5f97\u66f4\u9ad8\u7ea7\u522b\u7684\u9053\u5177\u540e\u518d\u53bb\u6253\u8d25\u4ed6\u3002", "\u4e0d\u627e\u5230\u6240\u6709\u7684\u6697\u589929\u697c\u7684\u6697\u9053\u662f\u4e0d\u4f1a\u6253\u5f00\u7684", "\u5982\u679c\u4f60\u523027\u697c\u65f6\u72b6\u6001\u4e3a\uff1a\u751f\u547d1500\u3001\u653b\u51fb80\u3001\u9632\u5fa198\u3001\u62e5\u67091\u628a\u84dd\u94a5\u5319\u30015\u628a\u9ec4\u94a5\u5319\u3002\u90a3\u4e48\u795d\u8d3a\u4f60\uff0c\u4f60\u7684\u524d\u671f\u662f\u6bd4\u8f83\u6210\u529f\u7684\u3002", "\u516d\u4e01\u516d\u7532\u7684\u653b\u51fb\u529b\u592a\u9ad8\u4e86\uff0c\u4f60\u6700\u597d\u5230\u80fd\u5bf9\u4ed6\u4e00\u51fb\u5fc5\u6740\u65f6\u518d\u4e0e\u4ed6\u6218\u6597\u3002", "\u522b\u5306\u5fd9\uff0c\u653e\u6162\u901f\u5ea6\u3002", "\u5982\u679c\u4f60\u80fd\u7528\u597d4\u79cd\u79fb\u52a8\u5b9d\u7269\uff0c\u4f60\u4e0d\u7528\u4e0e\u5f3a\u654c\u4f5c\u6218\u5c31\u80fd\u4e0a\u697c\u3002", "", "\u4f60\u9700\u8981\u7528\u201c\u7384\u660e\u77f3\u201d\u53d6\u51fa37\u697c\u4ed3\u5e93\u5185\u7684\u6240\u6709\u5b9d\u7269\u3002", "\u8c1c\u9898\uff1a\u201c\u57283\u70b9\uff0c\u62e5\u6709\u4f20\u9001\u529f\u80fd\u7684\u5bc6\u5b9d\u5c31\u4f1a\u51fa\u73b0\u3002\u201d", "\u201c\u5deb\u5e08\u201d\u4f1a\u7528\u9b54\u6cd5\u653b\u51fb\u8def\u8fc7\u7684\u4eba\uff0c\u57282\u4e2a\u201c\u592a\u4e0a\u8001\u541b\u62a4\u536b\u201d\u95f4\u901a\u8fc7\u4f1a\u4f7f\u4f60\u7684\u751f\u547d\u51cf\u5c11\u4e00\u534a\u3002", "44\u697c\uff0c\u88ab\u85cf\u5728\u5f02\u7a7a\u95f4\uff0c\u4f60\u53ea\u80fd\u7528\u5bc6\u5b9d\u624d\u80fd\u5230\u8fbe\u3002", "41\u697c\u4e8b\u5b9e\u4e0a\u662f\u5de6\u53f3\u5bf9\u79f0\u7684\u3002", "\u50cf\u9ab0\u5b50\u4e0a5\u7684\u5f62\u72b6\u662f\u4e00\u79cd\u5c01\u5370\u9b54\u6cd5\uff0c\u4f60\u6700\u597d\u8bb0\u4f4f\u5b83\u5728\u4f60\u4e0e49\u697c\u5047\u9b54\u738b\u6218\u6597\u65f6\u6709\u7528", "", "\u4f60\u597d\uff0c\u6211\u662f\u592a\u767d\u91d1\u661f\u3002\u4f60\u6700\u597d\u522b\u89c1\u654c\u4eba\u5c31\u6740\uff0c\u5148\u5f80\u4e0a\u8d70\uff0c\u62ff\u5230\u6b66\u5668\u548c\u9632\u5177\u518d\u505a\u6253\u7b97\u3002"};
        this.f_String_arr_07 = new String[]{"\u611f\u8c22\u4f60\u6551\u4e86\u6211\uff0c\u8fd9\u662f1000\u91d1\u5c31\u9001\u7ed9\u4f60\u5427\u3002", "\u8bd5\u4e0b\u706b\u773c\u91d1\u775b\u5427\uff0c\u4f60\u80fd\u770b\u5230\u602a\u7269\u7684\u4fe1\u606f\u548c\u6218\u6597\u635f\u5931\u7684\u8840\u91cf\uff0c\u4f60\u53ef\u4ee5\u5728\u7269\u54c1\u680f\u4e2d\u4f7f\u7528\u5b83\u3002", "", "", "\u5f88\u597d\uff0c\u4f60\u5c45\u7136\u627e\u5230\u4e86\u6211\uff0c\u4f5c\u4e3a\u5956\u52b1\u6211\u5c06\u7ed9\u4f60\u4e00\u74f6\u5343\u5e74\u6708\u6842\u9732\uff0c\u559d\u4e86\u5b83\u5c06\u6309\u4f60\u7684\u653b\u51fb\u529b\u548c\u9632\u5fa1\u529b\u7684\u7efc\u5408\u589e\u52a0\u7684\u4f60\u751f\u547d\u70b9\u6570\uff0c\u4f60\u8d8a\u665a\u4f7f\u7528\u5b83\u6548\u679c\u8d8a\u597d\u3002", "", "", "", "", "", "", "", "", "", "\u611f\u8c22\u4f60\u6551\u4e86\u6211\uff0c\u8fd9\u662f1000\u91d1\u5c31\u9001\u7ed9\u4f60\u5427\u3002", "", "", "", "", "", "\u54c8\u55bd\uff0c\u9001\u4f601000\u91d1\u4f5c\u4e3a\u89c1\u9762\u793c\uff0c\u8bb0\u5f97\u7ecf\u5e38\u6765\u627e\u6211\u54e6\u3002", ""};
        this.f_byte_arr_12 = new byte[]{2, 2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1};
        this.f_String_arr_08 = new String[]{"\u5927\u5723\u9976\u547d\uff0c\u6211\u53ef\u4ee5\u63d0\u53473%\u653b\u51fb\u529b\u548c\u9632\u5fa1\u529b\uff0c\u9700\u8981\u5c0f\u7684\u4e3a\u4f60\u6548\u52b3\u5417\uff1f", "", "\u5927\u5723\u7237\uff0c\u6211\u8fd9\u91cc\u67091\u628a\u84dd\u94a5\u5319\uff0c\u4f60\u7ed950\u91d1\u5e01\u5427\u3002", "\u6211\u67095\u628a\u9ec4\u94a5\u5319\uff0c\u4e00\u517150\u91d1\u5e01", "\u563f\u563f\uff0c\u6211\u6709\u5f88\u591a\u628a\u9ec4\u94a5\u5319\uff0c1\u628a1000\u91d1\u5e01", "\u6211\u67091\u628a\u7ea2\u94a5\u5319\uff0c\u53ea\u8981800\u91d1\u5e01", "\u6211\u67091\u628a\u84dd\u94a5\u5319\uff0c\u53ea\u8981200\u91d1\u5e01", "\u6211\u8ddf\u5176\u4ed6\u4eba\u4e0d\u4e00\u6837\uff0c\u4f60\u53ef\u4ee5\u628a\u591a\u4f59\u7684\u94a5\u5319\u5356\u7ed9\u6211\u3002100\u91d1\u5e01\u56de\u65361\u628a\u9ec4\u94a5\u5319", "\u6211\u67091\u628a\u9ec4\u94a5\u5319\uff0c1\u628a\u84dd\u94a5\u5319\uff0c\u4e00\u5171\u662f1000\u91d1\u5e01", "\u6211\u67093\u628a\u9ec4\u94a5\u5319\uff0c\u53ea\u8981200\u91d1\u5e01", "\u6211\u67093\u628a\u84dd\u94a5\u5319\uff0c\u6536\u4f602000\u91d1\u5e01", "\u6211\u53ef\u4ee5\u6062\u590d\u4f602000\u70b9\u8840\uff0c\u4e0d\u8fc7\u89811000\u91d1\u5e01", "\u6211\u6709\u4e2a\u5b9d\u7269\uff0c\u8981\u6536\u4f604000\u91d1\u5e01\uff0c\u662f\u4e2a\u7384\u660e\u77f3\u3002"};
        this.f_String_arr_09 = new String[]{"", "", "\u5929\u5bab\u4e00\u517150\u5c42\uff0c\u6bcf10\u5c42\u4e3a\u4e00\u4e2a\u533a\u57df\u3002\u5982\u679c\u4e0d\u6253\u8d25\u8be5\u533a\u57df\u7684\u5934\u76ee\u5c31\u4e0d\u80fd\u5230\u66f4\u9ad8\u7684\u5730\u65b9\u3002", "\u5728\u5546\u5e97\u91cc\u4f60\u6700\u597d\u9009\u62e9\u63d0\u5347\u9632\u5fa1\u529b\uff0c\u53ea\u6709\u5728\u653b\u51fb\u529b\u4f4e\u4e8e\u654c\u4eba\u7684\u9632\u5fa1\u529b\u65f6\u624d\u63d0\u5347\u653b\u51fb\u529b\u3002", "", "\u4f60\u662f\u5426\u6ce8\u610f\u52305\u30019\u300114\u300116\u300118\u697c\u6709\u7684\u5899\u4e0e\u4f17\u4e0d\u540c\uff1f", "\u5982\u679c\u4f60\u6301\u6709\u592a\u516c\u6756\uff0c\u9762\u5bf9\u5929\u795e\u529b\u58eb\u548c\u5de8\u7075\u795e\u65f6\u4f60\u7684\u653b\u51fb\u529b\u52a0\u500d\u3002\u5728\u6ca1\u6709\u592a\u516c\u6756\u7684\u60c5\u51b5\u4e0b\u4f60\u662f\u65e0\u6cd5\u6253\u8d25\u5de8\u7075\u795e\u7684\u3002\u592a\u516c\u6756\u88ab\u85cf\u572815\u697c\u4ee5\u4e0a\u7684\u5899\u5185\u3002", "", "\u5929\u5bab\u4e00\u5171\u670950\u5c42\uff0c\u4f4650\u697c\u5e76\u4e0d\u80fd\u76f4\u63a5\u4e0a\u53bb\u3002", "\u5b58\u653e\u4e4c\u91d1\u68cd\u7684\u623f\u95f4\u7684\u95e8\u574f\u4e86\uff0c\u4f60\u5fc5\u987b\u7528\u91d1\u52fa\u5b50\u7834\u5899\u800c\u5165\u3002", "\u5929\u5bab\u4e2d\u85cf\u6709\u6709\u4e2a\u201c\u5e78\u8fd0\u91d1\u5e01\u201d\u62e5\u6709\u5b83\u5728\u6253\u8d25\u654c\u4eba\u540e\u80fd\u591f\u83b7\u5f972\u500d\u7684\u91d1\u94b1\u3002", "\u201c\u7d2b\u91d1\u9f99\u9cde\u7532\u201d\u80fd\u9632\u5fa1\u201c\u592a\u4e0a\u8001\u541b\u62a4\u536b\u201d\u7684\u5939\u51fb\uff0c\u4f46\u5b83\u88ab\u6df1\u85cf\u5728\u795e\u79d8\u7684\u697c\u5c42\u4e2d\u3002", "\u5982\u679c\u8981\u6253\u8d25\u6768\u622c\u4f60\u9700\u8981\u201c\u4e4c\u91d1\u68cd\u201d\u3001\u201c\u94f6\u7f15\u9501\u7532\u201d\u3001\u201c\u6346\u4ed9\u7ef3\u201d\u6216\u66f4\u9ad8\u7b49\u7ea7\u7684\u5b9d\u7269\u3002"};
        this.f_byte_arr_13 = new byte[]{2, 3, 3, 3, 6, 3, 3, 6, 3, 3, 3, 3, 3};
        this.f_int_88 = this.f_String_arr_06.length + this.f_String_arr_09.length;
        this.f_byte_arr_14 = new byte[this.f_int_88];
        this.f_byte_arr_15 = new byte[this.f_int_88];
        this.itemDescriptions = new String[]{"\u80fd\u770b\u7834\u654c\u4eba\u5e95\u7ec6\uff0c\u663e\u793a\u654c\u4eba\u8be6\u7ec6\u4fe1\u606f\u3002\u5728\u6e38\u620f\u4e2d\u6309\u5feb\u6377\u952e5\u4e5f\u53ef\u4ee5\u67e5\u770b\u4f24\u5bb3\u91cf\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u8bb0\u5f55\u524d\u5c18\u5f80\u4e8b\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u5728\u697c\u68af\u8fb9\uff0c\u53ef\u4ee5\u77ac\u95f4\u4e0a\u4e0b\u5c42\uff0c\u7559\u795e\u6655\u673a\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u7184\u706d\\cFFCC33\u4e09\u6627\u771f\u706b\\r\u7684\u795e\u5668\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a\u65e0\u9650]", "\u6316\u6d1e\u5f00\u5899\u8d8a\u72f1\u7684\u5229\u5668,\u632b\u662f\u632b\u4e86\u70b9\uff0c\u4f46\u662f\u771f\u7684\u5f88\u597d\u7528\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u53ef\u4ee5\u9707\u5f00\u5f53\u524d\u5c42\u6240\u6709\u7684\u5899\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u559d\u4e0b\u540e\uff0c\u589e\u52a0\u76f8\u5f53\u4e8e\u5f53\u524d\\c00FFFF\u653b\u51fb\u529b\\cFFFFFF\u52a0\\c00FFFF\u9632\u5fa1\u529b\\cFFFFFF\u503c740%\u7684\\cFFCC00\u8840\u91cf\n[\u6708\u5bab\u51fa\u54c1\uff0c\u624b\u5de5\u917f\u5236\uff0c\u4e0d\u542b\u4e09\u805a\u6c30\u80fa\uff0c\u51b7\u85cf\u6548\u679c\u66f4\u4f73\uff0c\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u77ac\u79fb\u5230\u4ee5\u4e2d\u5fc3\u4e3a\u5bf9\u79f0\u70b9\u7684\u4f4d\u7f6e\u4e0a\u3002\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a3\u6b21]", "\u77ac\u79fb\u4e0a\u884c\u4e00\u5c42\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u77ac\u79fb\u4e0b\u884c\u4e00\u5c42\n\\c00ff00[\u4f7f\u7528\u6b21\u6570\uff1a1\u6b21]", "\u5f53\u5e74\u59dc\u5b50\u7259\u53d7\u5929\u547d\u5c01\u795e\uff0c\u4ed6\u7684\u9493\u9c7c\u7af9\u7aff\u88ab\u539f\u59cb\u5929\u5c0a\u9644\u4e0a\u4e86\u795e\u529b\uff0c\u53ef\u4ee5\u5f79\u4f7f\u5929\u795e\u529b\u58eb\u4f9b\u4ed6\u5dee\u9063\uff0c\u6b64\u6756\u53c8\u540d\u201c\u6253\u795e\u97ad\u201d\uff0c\u5bf9\u5929\u795e\u529b\u58eb\uff08\u5305\u62ec\u5de8\u7075\u795e\uff09\u5a01\u529b\u52a0\u500d\u3002\n\\cFFCC00[\u653e\u5728\u9053\u5177\u680f\u4e2d\u6709\u6548]", "\u5bf9\u67d0\u4e9b\u81ea\u604b\u7684\u795e\u4ed9\u4f24\u5bb3\u52a0\u500d\u3002\n\\cFFCC00[\u653e\u5728\u9053\u5177\u680f\u4e2d\u6709\u6548]", "\u6253\u602a\u5f97\u5230\u7684\u91d1\u94b1\u52a0\u500d\u3002\n\\cFFCC00[\u653e\u5728\u9053\u5177\u680f\u4e2d\u6709\u6548]", "\u53ef\u4ee5\u5f00\u542f\u9ec4\u95e8\u3002", "\u53ef\u4ee5\u5f00\u542f\u7ea2\u95e8\u3002", "\u53ef\u4ee5\u5f00\u542f\u84dd\u95e8\u3002", "\u52a0\u653b\u51fb\u3002", "\u52a0\u9632\u5fa1\u3002", "\u52a0\u8840\u3002", "\u52a0\u8840\u3002", "\u5f00\u542f\u5f53\u524d\u5c42\u6240\u6709\u9ec4\u95e8", "\u5982\u6765\u5f00\u201c\u6148\u60b2\u4e3a\u6000\u201d\u5de1\u56de\u4f5b\u7ecf\u6f14\u5531\u4f1a\u7684\u65f6\u5019\uff0c\u4f34\u594f\u7f57\u6c49\u7528\u7684\u4e50\u5668\uff0c\u9053\u884c\u6d45\u7684\u654c\u4eba\uff0c\u4f1a\u88ab\u5176\u68b5\u5929\u4f5b\u97f3\u77ac\u95f4\u5316\u4e3a\u7070\u98de\n\\cFFCC00\u4f7f\u7528\uff1a\u6740\u6b7b\u4e0a\u4e0b\u5de6\u53f3\u7684\u654c\u4eba\uff0c\u5bf9BOSS\u4e0d\u8d77\u4f5c\u7528\u3002"};
        this.equipDescriptions = new String[]{"", "\\cdddddd\u4e00\u6839\u76f8\u5f53\u957f\u7684\u6728\u5236\u957f\u68cd,\u65b0\u624b\u5fc5\u5907.\u6709\u4e86\u5b83\u6740\u4eba\u8d8a\u8d27\u4e0d\u614c\u4e0d\u6101.\n\\c00ff00\u88c5\u5907: \u653b\u51fb+10.\n\\cFFCC00\"\u770b\u4e0a\u53bb\u4f3c\u4e4e\u4f1a\u65ad\u6389\u3002\".", "\\cdddddd\u4e4c\u9ed1\u6cb9\u4eae\uff0c\u663e\u7136\u7ecf\u5386\u8fc7\u591a\u4eba\u4e4b\u624b\u3002\n\\c00ff00\u88c5\u5907: \u653b\u51fb+30.\n\\cFFCC00\"\u5f88\u7c97\u5f88\u7ed3\u5b9e\uff01\".", "\\cdddddd\u94f6\u68cd\uff0c\u6069\uff0c\u6709\u8fd9\u4e2a\u540d\u5b57\u5c31\u8db3\u591f\u4e86\u3002\n\\c00ff00\u88c5\u5907: \u653b\u51fb+70.\n\\cFFCC00\"\u53ea\u662f\u6839\u94f6\u68cd\".", "\\cdddddd\u56e0\u4e58\u5929\u5730\u4e4b\u7075\u6c14\uff0c\u96c6\u65e5\u6708\u4e4b\u7cbe\u534e\u4e43\u201c\u4e07\u6728\u4e4b\u7075\uff0c\u7075\u6728\u4e4b\u5c0a\u201d\u3002\n\\c00ff00\u88c5\u5907: \u653b\u51fb+120.\n\\cFFCC00\"\u6728\u4e4b\u7cbe\u534e\uff0c\u524a\u94c1\u65ad\u91d1\".", "\\cdddddd\u60a8\u7684\u9700\u8981\uff0c\u5b83\u77e5\u9053\uff1b\u60a8\u7684\u9700\u6c42\uff0c\u5b83\u6ee1\u8db3\u3002\u5b83\u597d\uff0c\u4f60\u4e5f\u597d\uff0c\u9f99\u738b\u540e\u5bab\uff0c\u9547\u5bab\u4e4b\u5b9d\uff01\n\\c00ff00\u88c5\u5907: \u653b\u51fb+220.\n\\cFFCC00\"\u4e0d\u8981\u8ff7\u604b\u5b83\uff0c\u5b83\u53ea\u662f\u4e00\u6839\u4f20\u8bf4\u3002\".", "", "\\cdddddd\u6ca1\u6709\u592a\u591a\u7684\u88c5\u9970\uff0c\u4e00\u4ef6\u975e\u5e38\u6734\u7d20\u3001\u8f7b\u4fbf\u7684\u5e03\u8863.\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+10.\n\\cFFCC00\"\u770b\u4e0a\u53bb\u6709\u4e0d\u5c11\u4eba\u7528\u8fc7\u4e86\u3002\".", "\\cdddddd\u4fdd\u6696\u5fa1\u5bd2\uff0c\u8170\u4e0d\u9178\uff0c\u817f\u4e0d\u75bc\uff0c\u8d70\u8def\u4e5f\u6709\u52b2\u4e86\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+30.\n\\cFFCC00\"\u8c79\u7eb9\uff0c\u6027\u611f\u53c8\u91ce\u6027\uff0c\u4eca\u5e74\u5929\u5bab\u6700\u6d41\u884c\u7684\u76ae\u8349\u6b3e\u5f0f\".", "\\cdddddd\u5982\u679c\u6ca1\u6709\u4e0a\u9762\u7684\u90a3\u884c\u5b57\uff0c\u5b83\u4e5f\u7b97\u662f\u4e2a\u6770\u4f5c\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+70.\n\\cFFCC00\"\u4e0a\u9762\u5199\u7740'\u529e\u56db\u7ea7\u795e\u4ed9\u8bc1\u4e66\uff0c\u56de\u6536\u4e8c\u624b\u83b2\u82b1\u5b9d\u5ea7'\".", "\\cdddddd\u534e\u4e3d\u7684\u88c5\u9970\uff0c\u5c31\u662f\u6709\u70b9\u65e7\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+120.\n\\cFFCC00\"\u522b\u4eba\u7a7f\u8fc7\u7684\u6781\u54c1\u3002\".", "\\cdddddd\u4e1c\u6d77\u9f99\u9cde\u7f16\u7ec7\u800c\u6210\uff0c\u9650\u91cf\u7248\uff0c\u5929\u4e0a\u5929\u4e0b\uff0c\u53ea\u6b64\u4e00\u6b3e\u3002\n\\c00ff00\u88c5\u5907: \u9632\u5fa1+220.\n\\cFFCC00\"\u66f4\u8f7b\u8584\uff0c\u66f4\u900f\u6c14\uff0c\u66f4\u591a\u9632\u62a4\uff0c\u66f4\u591a\u5b89\u5fc3\"."};
        this.itemUseCounts = new byte[]{-1, -1, -1, -1, 1, 1, 1, 3, 1, 1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1};
        this.yellowKeyCount = 0;
        this.blueKeyCount = 0;
        this.redKeyCount = 0;
        this.goldAmount = 0;
        this.itemStackTypes = new byte[32];
        this.itemStackUses = new byte[32];
        int[] nArray4 = new int[]{0, 1, 2, 4, 8, 16, 32};
        this.equipTierBonuses = new int[]{0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220};
        this.equipTierTypes = new byte[]{0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40};
        this.objectTypeNames = new String[]{"\u5b59\u609f\u7a7a", "\u9ec4\u95e8", "\u7ea2\u95e8", "\u84dd\u95e8", "\u5c01\u5370\u95e8", "\u95e8\u536b", "\u9690\u5f62\u8def\u5f84", "\u4e0a\u697c\u68af", "\u4e0b\u697c\u68af", "\u70bc\u4e39\u7089", "\u4e91\u96fe", "\u80fd\u6316\u7684\u5899", "\u9690\u5f62\u5899", "\u706b\u773c\u91d1\u775b", "\u751f\u6b7b\u7c3f", "\u7b4b\u6597\u4e91", "\u82ad\u8549\u6247", "\u91d1\u52fa\u5b50", "\u7384\u660e\u77f3", "\u5343\u5e74\u6708\u6842\u9732", "\u76f8\u5f62\u53d8\u4f4d", "\u4e0a\u884c\u7b26", "\u4e0b\u884c\u7b26", "\u592a\u516c\u6756", "\u6346\u4ed9\u7ef3", "\u5e78\u8fd0\u5e01", "\u9ec4\u94a5\u5319", "\u7ea2\u94a5\u5319", "\u84dd\u94a5\u5319", "\u5a01\u529b\u4e39", "\u91d1\u521a\u4e39", "\u56de\u6625\u4e38", "\u957f\u5bff\u4e39", "\u6843\u6728\u68d2", "\u7384\u94c1\u68d2", "\u771f\u94f6\u68d2", "\u91d1\u7b8d\u68d2", "\u9053\u888d", "\u94c1\u7532", "\u9501\u5b50\u7532", "\u7d2b\u91d1\u9f99\u9cde\u7532", "\u5929\u5bab\u5c0f\u72ac", "\u5929\u5bab\u5927\u72ac", "\u4f34\u7089\u7ae5\u5b50", "\u62a4\u5ead\u5c0f\u795e", "\u5b88\u56ed\u4ed9\u5a62", "\u62a4\u5ead\u6821\u5c09", "\u5de1\u5929\u536b\u58eb", "\u5de1\u5929\u536b\u58eb", "\u5de8\u529b\u58eb", "\u6267\u706b\u9053\u4eba", "\u594e\u6728\u72fc", "\u6267\u74f6\u4ed9\u4f8d", "\u91d1\u521a\u529b\u58eb", "\u62a4\u6301\u8fe6\u84dd", "\u4f8d\u6848\u4ed9\u5973", "\u62a4\u5ead\u795e\u5c06", "\u8d64\u529b\u97e6\u9a6e", "\u62a4\u4e39\u8001\u9053", "\u4f0f\u9b54\u97e6\u9a6e", "\u515c\u7387\u5bab\u661f\u541b", "\u65e0\u91cf\u62a4\u6cd5", "\u515c\u7387\u5bab\u8001\u4ed9", "\u4e8c\u90ce\u6267\u65d7\u5c06", "\u6740\u7834\u72fc", "\u4e8c\u90ce\u9a81\u9a91\u5c06", "\u5c0a\u5929\u97e6\u9a6e", "\u5578\u5929\u72ac", "\u592a\u4e0a\u8001\u541b", "\u6768\u622c", "\u7389\u7687\u5927\u5e1d", "\u7389\u7687\u5927\u5e1d", "\u725b\u9b54\u738b", "\u5929\u84ec\u5143\u5e05", "\u5de8\u7075\u795e", "\u54ea\u5412", "\u8def\u70b9", "\u592a\u767d\u91d1\u661f", "\u5929\u5bab\u5546\u4eba", "\u4e4c\u91d1\u68cd", "\u94f6\u7f15\u9501\u7532", "\u5c01\u5370\u95e8", "\u4f20\u602a\u70b9", "\u5267\u60c5\u70b9", "\u5ae6\u5a25", "\u9ec4\u91d1\u94a5\u5319", "\u65e5\u6708\u65e0\u6781\u94b9", "\u83e9\u63d0\u8001\u7956"};
        this.enemyBaseHp = new int[]{35, 45, 35, 50, 60, 55, 100, 50, 260, 60, 130, 100, 320, 20, 320, 100, 210, 220, 160, 200, 230, 220, 200, 360, 180, 180, 1200, 4500, 1500, 8000, 800, 5000, 120, 444, 100};
        this.enemyBaseAtk = new int[]{18, 20, 38, 42, 32, 52, 180, 48, 85, 100, 60, 95, 120, 100, 140, 680, 200, 180, 230, 380, 450, 370, 390, 310, 430, 460, 180, 560, 600, 5000, 500, 1580, 150, 199, 65};
        this.enemyBaseDef = new int[]{1, 2, 3, 6, 8, 12, 110, 22, 5, 8, 3, 30, 15, 68, 20, 50, 65, 30, 105, 130, 100, 110, 90, 20, 210, 360, 20, 310, 250, 1000, 100, 190, 50, 66, 15};
        this.enemyBaseGold = new int[]{1, 2, 3, 6, 5, 8, 100, 12, 18, 12, 8, 22, 30, 28, 30, 55, 45, 35, 65, 90, 100, 80, 50, 40, 120, 200, 100, 1000, 800, 500, 500, 500, 100, 144, 30};
        this.levelScriptLines = new String[]{"CES_84_6_11 MOV_0_5_11 TAK_8_9 CES_70_5_8 TAK_10_10 ROS_4_1 TAK_11_17 ROS_4_2 TAK_18_19 MOV_0_5_10 TAK_20_21 DES_70_5_8 LAY_2 ROS_1_4_7 ROS_2_0 ROS_2_6 RES_0 GUT_1 ", "TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8 ", "TAK_33_36 MOV_72_1_8_1_10 DES_72_1_10 ", "TAK_37_37 MOV_0_6_5 TAK_38_39 TAK_41_41 DES_44_1_3 DES_44_2_3 DES_44_3_3 DES_46_2_4 DES_44_9_3 DES_44_10_3 DES_44_11_3 DES_46_10_4 CES_44_5_4 CES_46_6_4 CES_44_7_4 CES_44_5_5 CES_44_7_5 CES_44_5_6 CES_46_6_6 CES_44_7_6 SWD TAK_42_42 ", "CES_72_1_11 TAK_43_44 MOV_0_6_3 MOV_72_1_11_6_2 ROS_4_1 TAK_45_48 MOV_72_6_2_6_1 DES_72_6_1 ", "TAK_49_52 MOV_72_9_1_7_1 DES_72_7_1 ", "TAK_53_58 ", "TAK_60_60 MOV_72_3_2_8_4 TAK_61_61 CES_47_8_3 CES_47_8_5 TAK_62_63 DES_72_8_4 DES_47_8_3 DES_47_8_5 ADD_2_72_11_10 ", "ROS_4_1 CES_73_10_1 MOV_73_10_1_6_9 TAK_68_74 MOV_73_6_9_6_10 ", "SWD ", "CES_72_3_10 MOV_72_3_10_2_10 MOV_72_2_10_4_9 TAK_76_78 DES_72_4_9 ", "SWD ", "TAK_84_87 MOV_58_5_4_6_8 MOV_58_4_4_6_8 MOV_58_3_4_6_8 MOV_57_7_4_6_8 MOV_57_8_4_6_8 MOV_57_9_4_6_8 MOV_56_4_2_6_8 MOV_56_3_2_6_8 MOV_56_2_2_6_8 MOV_59_8_2_6_8 MOV_59_9_2_6_8 MOV_59_10_2_6_8 TAK_88_90 MOV_73_6_2_6_8 ", "CES_6_10_2 CES_60_10_2 ", "ROS_4_1 TAK_92_97 DES_73_6_8 TAK_98_100 DES_70_6_7 ", "ROS_4_1 CES_61_5_2 CES_61_6_2 CES_61_7_2 CES_61_5_3 CES_70_6_3 CES_61_7_3 CES_61_5_4 CES_61_6_4 CES_61_7_4 TAK_118_121 ", "TAK_123_126 GUT_37 ", "TAK_68_74 ", "DES_1_4_4 CES_20_4_4 ", "CES_84_7_7 MOV_84_7_7_6_8 TAK_0_1 MOV_84_6_8_1_8 TAK_2_2 MOV_0_2_8 TAK_3_3 MOV_84_1_8_1_1 TAK_4_4 MOV_0_1_2 TAK_5_5 MOV_84_1_1_10_1 MOV_0_6_1 DES_84_10_1 TAK_6_7 MOV_0_11_1 MOV_0_1_11 ", "TAK_40_40 ", "TAK_59_59 ", "TAK_75_75 ", "TAK_84_87 GUT_12 ", "TAK_101_105 ", "CES_22_6_6 ", "TAK_129_132 DES_84_11_4 GIN_1_1000 ", "TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13 ", "TAK_136_143 DES_84_1_11 TAK_144_144 GIN_0_19 ", "TAK_145_146 DES_84_9_8 GIN_1_1000 ", "MOV_84_6_3_4_3 MOV_84_4_3_8_3 MOV_84_8_3_6_3 TAK_107_107 ", "TAK_108_108 DES_10_6_6 MOV_0_6_5 TAK_109_112 TAK_147_148 DES_84_6_3 TAK_149_149 ", "TAK_79_79 ", "TAK_155_160 DES_72_11_10 ", "TAK_150_154 ", "TAK_161_161 ", "TAK_162_162 ", "GLV_1 ", "CES_6_4_1 CES_61_4_1 ", "", "TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 ", "TAK_167_167 SEE_4_10_168_168_0 ROS_4_1 SEE_2_8_169_169_0 SEE_2_8_170_170_1 ", "TAK_171_171 SEE_7_10_172_172_0 SEE_7_10_173_173_1 ", "TAK_174_174 SEE_7_9_175_175_0 TAK_176_176 SEE_8_8_177_178_0 ", "TAK_179_180 ROS_4_3 SEE_8_6_181_182_0 ROS_4_1 SEE_10_4_183_183_0 ", "ROS_4_1 SEE_8_3_184_184_0 ROS_4_0 SEE_6_7_185_185_0 ", "SEE_4_4_186_186_0 ROS_4_1 SEE_6_2_187_187_1 ", "ROS_4_0 SEE_6_2_188_188_0 SEE_4_4_189_189_1 ", "TAK_190_190 ROS_4_1 SEE_2_2_191_192_0 ROS_4_0 SEE_2_6_193_196_0 SEE_2_6_197_198_1 ", "ROS_4_3 SEE_1_11_199_200_0 ", "ROS_4_2 SEE_6_11_201_202_0 ROS_4_3 SEE_3_8_203_204_0 CES_6_3_10 SEE_3_8_205_205_1 ", "TAK_206_209 ", "TAK_210_211 SEE_11_11_212_212_0 SEE_11_7_213_214_0 TAK_215_216 ", "TAK_217_218 ROS_5_510 ROS_6_510 CES_36_11_8 SEE_11_8_219_221_0 CES_40_11_9 SEE_11_9_222_222_0 SEE_8_9_224_225_0 SEE_10_9_223_223_0 DES_11_10_9 ", "TAK_224_224 SEE_6_2_225_225_0 ", "TAK_226_227 ROS_5_510 ROS_6_510 CES_36_5_1 CES_40_7_1 TAK_228_228 ", "TAK_229_231 MOV_0_6_7 TAK_232_232 DES_51_2_8 DES_51_1_8 DES_51_2_9 DES_51_1_9 CES_51_6_6 CES_51_5_7 CES_51_6_8 CES_51_7_7 GUT_57 ", "MOV_51_6_6_6_7 ROS_4_3 MOV_51_5_7_6_7 ROS_4_0 MOV_51_6_8_6_7 ROS_4_2 MOV_51_7_7_6_7 ROS_4_1 TAK_233_235 MOV_74_7_4_6_7 GUT_61 ", "", "", "", "TAK_236_238 MOV_75_5_4_6_7 TAK_239_239 GUT_62 ", "CES_69_5_5 CES_47_5_6 CES_47_5_7 CES_47_5_8 CES_56_7_6 CES_56_7_7 CES_56_7_8 TAK_240_245 GUT_63 ", "CES_77_6_6 TAK_246_250 DES_71_6_3 DES_77_6_6 DES_69_5_5 DES_-66_0 DES_-56_0 DES_-49_0 DES_-47_0 DES_-69_0 ", "TAK_253_254 ", "TAK_127_128 END_0 ", "TAK_261_263 SMS_0 ", "TAK_264_268 "};
        this.dialogueSpeakerType = new byte[]{0, 84, 0, 84, 0, 84, 0, 0, 84, 0, 70, 0, 70, 0, 70, 0, 70, 0, 84, 0, 0, 70, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 75, 0, 75, 75, 75, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 74, 0, 74, 0, 74, 0, 74, 72, 72, 47, 72, 0, 72, 72, 0, 73, 0, 73, 0, 73, 0, 73, 73, 72, 0, 72, 72, 0, 72, 72, 0, 73, 0, 73, 0, 73, 0, 73, 0, 70, 73, 70, 73, 70, 73, 73, 0, 70, 69, 0, 69, 0, 69, 84, 0, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 70, 0, 70, 0, 70, 72, 0, 72, 0, 0, -1, 0, 84, 0, 84, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 0, 0, 0, 69, 69, 0, 69, 0, 0, 72, 0, 72, 0, 72, 0, 0, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 0, 87, 87, 87, 87, 87, 87, 0, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 71, 71, 0, 71, 71, 74, 71, 75, 0, 71, 71, 69, 47, 69, 56, 69, 47, 77, 77, 77, 71, 0, 56, 71, 0, 0, 87, 0, 87, 0, 87, 0, 87, 0, 0, 87, 87, 87, 87, 87};
        this.f_byte_arr_22 = new byte[]{7, 5, -1, 3, -1, 5, -1, 1, 10, 0, -1, -1, -1, -1, -1, -1, -1, 1, 9, 7, 8, -1, -1, 5, 11, 2, 6, -1, -1, -1, -1, 3, 11, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 11, -1, 5, 0, 5, 9, 2, 10, 3, -1, -1, 11, -1, -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, 10, -1, -1, 9, 0, 1, -1, -1, -1, -1, -1, -1, 3, 11, -1, 0, 3, -1, -1, -1, -1, -1, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, -1, 5, -1, 1, 3, -1, 8, 9, 8, 9, -1, -1, -1, 8, 4, 5, 6, 10, 8, -1, -1, 5, 9, 1, -1, 0, 3, 7, -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, -1, -1, 0, -1, -1, -1, -1, -1, -1, 7, -1, 7, -1, 7, 2, 2, -1, 2, -1, -1, -1, 7, -1, -1, -1, -1, -1, -1, -1, -1, 9, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1};
        this.dialogueTexts = new String[]{"\u59d1\u5a18\uff0c\u60a8\u2026\u2026", "\u54ce\u5440\uff0c\u59d1\u5976\u5976\u6211\u597d\u4e0d\u5bb9\u6613\u6000\u63e32\u4e2a\u4ed9\u6843\u5939\u5e26\u51fa\u6765\uff0c\u5c45\u7136\u88ab\u53d1\u73b0\u4e86~\uff01\u5feb\u95ea\uff01\uff01", "\u2026\u2026\uff0c\u8dd1\u90fd\u8dd1\u7684\u8fd9\u4e48\u4f18\u96c5\u3002", "\u54ce\u5440\uff0c\u8dd1\u8fd9\u4e48\u5feb\u2026\u2026", "\u4fee\u9017\u5988\u5f85~\uff01\u2026\u2026", "\u6b7b\u7334\u5b50\uff0c\u7a77\u8ffd\u731b\u6253\uff0c\u8ffd\u7684\u4eba\u5bb6\u5c0f\u5fc3\u809d\u5657\u55f5\u5657\u55f5\u5730\u2026\u2026", "\u554a~~~\u59d1\u5a18\u54df~~\u59d1~~~\u5a18~~\uff01\uff01", "\u6b7b\u5f00\u6b7b\u5f00~\uff01\uff01\u597d\u72d7\u4e0d\u6321\u8def\u3002", "\u5c45\u7136\u8ffd\u5230\u8fd9\u91cc\u4e86\uff0c\u7b97\u4e86\u5427\uff0c\u8ba4\u4e86\u5427\u3002\n\\cFFCC00\u4e24\u4e2a\u6843\u5b50\u6eda\u843d\u5730\u4e0a", "\u54ce~~~~~~~", "\u6211\u54a4~~~~\uff01\u5929\u5ead\u5723\u5730\uff0c\u7981\u6b62\u55a7\u54d7\u3002", "\u6211~~~~", "\u6211\u4ec0\u4e48\u6211~\uff1f\uff1f\u6574\u5929\u8eab\u5c45\u4ed9\u4f4d\uff0c\u6e38\u624b\u597d\u95f2\u2026\u2026", "\u4f60~~~~~~~~~", "\u4f60\u4ec0\u4e48\u4f60~\uff1f\uff1f\u770b\u5ae6\u5a25\u8863\u6749\u4e0d\u6574\uff0c\u662f\u4e0d\u662f\u4f60\u975e\u793c~\uff1f\uff1f\u662f\u4e0d\u662f\u4f60\u662f\u4e0d\u662f\u4f60\u662f\u4e0d\u662f\u4f60\uff01\uff1f", "\u5979~~~~~~~~~~", "\u5979\u4ec0\u4e48\u5979~\uff1f\uff1f\u54e6~\uff01\u8fd8\u6eda\u51fa\u67652\u4e2a\u4ed9\u6843\uff01\uff01\u5047\u516c\u6d4e\u79c1\u662f\u5427~\uff01\uff01\u7334\u5b50\u5077\u6843\u662f\u5427\uff01\uff01", "\u4ffa\u6ca1~\uff01\u2026\u2026", "\u2026\u2026", "\u4f60\u2026\u2026\u53eb\u5ae6\u5a25\uff1f\u8fd9\u4e2a\u540d\u5b57\uff0c\u4ffa\uff0c\u8bb0\u3002\u4f4f\u3002\u4e86\u2026\u2026", "\u6ca1\u9519\uff0c\u6843\u5b50\u5c31\u662f\u4ffa\u5077\u7684\uff01\uff01\u4ffa\u8ba4\u7f5a\uff01", "\u5c06\u8fd9\u53ea\u5b7d\u755c\u5265\u4e0b\u94e0\u7532\u6253\u4e0b\u5929\u7262\uff01", "\u8d24\u5f1f\uff0c\u5feb\u9192\u9192\u2026\u2026", "\u54ce\u5466\u2026\u2026\u8001\u725b\uff0c\u9192\u6765\u89c1\u4f60\u90a3\u5f20\u8138\uff0c\u771f\u63d0\u795e\uff01\uff01", "\u6211\u5b81\u613f\u770b\u7740\u4f60\uff0c\u7761\u5f97\u5982\u6b64\u6c89\u9759\uff0c\u80dc\u8fc7\u4f60\u9192\u65f6\u51b3\u88c2\u822c\u65e0\u60c5~~", "\u9760\uff0c\u53d1\u6625\u5450\uff0c\u628a~\u624b~\u62ff~\u5f00~~\uff01", "\u54df~~\u633a\u6a2a\u5f97\u563f~\u725b\u54e5\u5b66\u5f97\u4e00\u624b\u597d\u6444\u5f71\u2026\u2026\u521a\u624d\u2026\u2026", "\u2026\u2026\u5927\u54e5\u4e45\u8fdd\u4e86\uff0c\u591a\u65e5\u4e0d\u89c1\uff0c\u53d7\u5c0f\u5f1f\u4e00\u62dc\uff01\uff01", "\u8bdd\u8bf4\u4f60\u4e00\u76f4\u5728\u68a6\u4e2d\u53eb\u7740\u4e00\u4e2a\u4eba\u7684\u540d\u5b57\uff0c\u5979\u4e00\u5b9a\u5077\u4e86\u4f60\u5f88\u591a\u6843\u5b50\u2026\u2026", "\u6ca1\u9519\uff0c\u5979\u662f\u5077\u4e86\u6211\u7684\u4e1c\u897f\uff0c\u4f46\u662f\u4e0d\u662f\u6843\u5b50\uff0c\u6211\u5f88\u60f3\u518d\u89c1\u5979\u2026\u2026", "\u6069\u6069\uff0c\u73b0\u5728\u5927\u54e5\u5e26\u4f60\u4ece\u5bc6\u9053\u51fa\u53bb\u2026\u2026", "\u2026\u2026\u8fd9\u4f60\u90fd\u80fd\u6316\u5f00~\uff01\uff01", "\u54e5\u5728\u9b54\u754c\u6709\u4e2a\u7ef0\u53f7\uff0c\u53eb\u201c\u6e9c\u5f97\u6ed1\u201d\uff0c\u6ca1\u6709\u4ec0\u4e48\u56da\u7262\u80fd\u56f0\u4f4f\u54e5\u2026\u2026", "\u5927\u54e5\uff01\uff01\u80fd\u4e0d\u80fd\u5206\u6211\u4ef6\u4e1c\u897f\u906e\u906e\u7f9e\u2026\u2026", "\u6709\uff0c\u6211\u8fd9\u6709\u628a\u521a\u6316\u5730\u9053\u7684\u7834\u52fa\uff0c\u4f60\u62ff\u53bb\u6321\u4f4f\u5148\uff01", "\u9760\u2026\u2026\u7b97\u4e86\uff0c\u603b\u6bd4\u6ca1\u6709\u597d\u3002\u2026\u2026", "\u6ca1\u529e\u6cd5\uff0c\u4f60\u5c31\u5c06\u5c31\u4e00\u4e0b\u5566\u3002\\cFFCC00\u6709\u4ef6\u9053\u7ae5\u7684\u65e7\u888d\u5b50\u57289\u697c\uff0c\u4f60\u4ed4\u7ec6\u627e\u627e\u5427\uff0c\\cF8F8F8\u6211\u5148\u64a4\u4e86\u2026\u2026", "\u5996\u7334\uff0c\u4f60\u53ef\u8ba4\u5f97\u672c\u5c11\u7237\uff01\uff01", "\u549d~~\u6211\u770b\u4f60\u9aa8\u9abc\u6e05\u5947\u4e09\u5934\u516d\u81c2\uff0c\u731c\u5f97\u4e0d\u9519\u7684\u8bdd\uff0c\u9601\u4e0b\u4e00\u5b9a\u662f\u7578\u5f62\u513f\uff01", "\u554a~\uff01\u5927\u54e5\u679c\u7136\u9ad8\u624b\uff01\u8fd9\u90fd\u88ab\u4f60\u770b\u7a7f\u4e86\uff01\u4e0d\u80fd\u7559\u4f60\u6d3b\u53e3\u4e86\uff01", "\u54ce\u5440\u5440~~\u6211\u56de\u53bb\u544a\u8bc9\u6211\u7239\u53bb\uff01", "\u6765\u4eba\uff01\u628a\u5996\u7334\u62ff\u4e0b\uff01", "\u54e6~~~\u55b3~\uff01\u674e\u5c0f\u7334\u8e22\u9986\uff01\uff01", "\u8d24\u5f1f\uff0c\u6162\u7740\uff01", "\u6e9c\u5f97\u6ed1\uff1f", "\u6211\u5077\u5077\u544a\u8bc9\u4f60\u54e6\uff0c\u6709\u4e2a\u94f6\u68cd\u572817\u5c42\uff0c\u4f60\u627e\u5230\u5b83\u4ee5\u540e\u5c31\u80fd\u5389\u5bb3\u5f88\u591a\uff1f", "\u4e86\u89e3\uff01", "\u7ea2\u4e86\u7ea2\u8138", "\u5c3d\u5feb\u53bb\u5427\uff0c\u6211\u95ea\u5148\u2026\u2026", "\u8d24\u5f1f\uff0c\u8fd9\u53ea\u5578\u5929\u72ac\u5728\u8fd9\u91cc\u6321\u9053\u592a\u5371\u9669\u4e86\uff0c\u8ddf\u54e5\u54e5\u7ed5\u5bc6\u9053\u5427\u3002", "\u4e00\u53ea\u770b\u95e8\u7684\u5ba0\u7269\uff0c\u80fd\u51f6\u5230\u54ea\u91cc\uff1f", "\u4e0d\u662f\uff0c\u81ea\u4ece\u8c03\u5230\u5929\u5ead\u5b88\u8def\uff0c\u5df2\u7ecf\u5f88\u591a\u5929\u6ca1\u6709\u5403\u8364\u8165\u4e86\u2026\u2026", "\u4e86\u89e3\uff0c\u7ed5\u9053\uff0c\u8d70~\uff01", "\u6211\u4e43\u6258\u5854\u5929\u738b\u9ebe\u4e0b\u5148\u950b\u5b98\uff0c\u5de8\u73b2\u795e\u662f\u4e5f\uff01\u4f60\u6b3a\u6211\u5b69\u513f\u2026\u2026", "\u6b3a\u4f60\u5b69\u513f~\uff1f", "\u5176\u5b9e\u2026\u2026\u54ea\u5412\u662f\u5974\u5bb6\u8ddf\u6258\u5854\u5929\u738b\u7684\u79c1\u751f\u5b50\u2026\u2026", "\u4f60\u662f\u5973\u5c06\uff01\uff1f", "\u5176\u5b9e\uff0c\u770b\u6211\u50cf\u8299\u84c9\u59d0\u59d0\u4e00\u6837\u7684\u597d\u8eab\u6bb5\uff0c\u4f60\u5c31\u5e94\u8be5\u4e86\u89e3\u2026\u2026", "\u2026\u2026\u549d~~\u6258\u5854\u5929\u738b\u53e3\u5473\u8fd9\u4e48\u504f\u2026\u2026", "\u554a~~~\u5929\u738b\uff0c\u5de8\u73b2\u513f\u4e0d\u80fd\u966a\u4f34\u4f60\u4e86\u2026\u2026", "\u770b\u6765\u4f60\u5df2\u7ecf\u6253\u901a\u6697\u5899\u4e86\uff0c\u770b\u54e5\u7684\u3002", "\u5662\uff0c\u901a\u5b8c\u6536\u5de5\uff0c\u8d70\uff01\uff01\u2026\u2026", "\u6211\u4eec\u662f\u5929\u5ead\u5e02\u5bb9\u7ba1\u7406\u961f\uff01\uff01\u554a\u54c8~\u4e71\u5806\u6e23\u571f\uff0c\u8fdd\u89c4\u65bd\u5de5\u7ec8\u4e8e\u88ab\u6293\u4e2a\u73b0\u884c\uff01", "\u54ce\u5440~\uff01\u5144\u5f1f\uff0c\u4e00\u5b9a\u8981\u67652\u5c42\u5929\u7262\u6551\u6211\u5440\uff01", "", "", "", "", "\u4f60\u5c31\u662f\u5b59\u609f\u7a7a\uff01\uff1f", "\u4ffa\u5c31\u662f\u3002", "\u6211\u5e38\u53bb\u5e7f\u5bd2\u5bab\u770b\u5979\uff0c\u5979\u8fd9\u51e0\u5929\u5e38\u5e38\u63d0\u5230\u4f60\u2026\u2026\u6211\u4ece\u6765\u4e0d\u77e5\u9053\u4ec0\u4e48\u53eb\u505a\u5ac9\u5992\uff0c\u4f46\u662f\u8fd9\u6b21\uff0c\u6211\u60f3\u8981\u4f60\u7684\u547d\u3002", "\u54fc\uff0c\u4f60\u54ea\u4f4d\uff1f", "\u6211\u4e43\u5929\u84ec\u5143\u5e05\uff0c\u6731\u521a\u9b23\uff01\uff01\u638c\u7ba1\u5929\u6cb3\u2026\u2026", "\u732a\u809b\u88c2\uff1f\uff1f\u597d\uff0c\u6ee1\u8db3\u4f60\u8fd9\u4e2a\u613f\u671b\u2026\u2026", "\u54ce\u2026\u2026\u6211\u8fd8\u6ca1\u62a5\u5b8c\u5462\u2026\u2026", "\u83ca\u82b1\u6b8b~~~\u6ee1\u5730\u4f24\u2026\u2026", "\u4f60\u8ddf\u6768\u622c\u4e00\u6218\uff0c\u5929\u5ead\u90fd\u5f00\u4e86\u76d8\u53e3\uff0c\u8d54\u7387\u662f1:5\uff0c\u54e5\u628a\u79c1\u623f\u94b1\u90fd\u62bc\u5230\u4f60\u5934\u4e0a\u4e86\uff0c\u8868\u8f9c\u8d1f\u54e5\u54e5\u54df~", "\u2026\u2026\u771f\u60f3\u80cc\u540e\u7ed9\u4f60\u4e00\u95f7\u68cd\u2026\u2026", "\u5929\u5ead\u7981\u6b62\u4e71\u5806\u6e23\u571f\uff0c\u54e5\u54e5\u628a\u6e23\u571f\u8fd0\u5230\u201c\u5929\u5ead\u57ce\u7ba1\u529e\u4e8b\u5904\u201d\u53bb\uff0c\u5c31\u4e0d\u7b97\u4e71\u5806\u4e86\uff0c\u634f\u563f\u563f~~", "\\cFFCC0023\u5c42\u4e43\u662f29\u5c42\u7684\u5730\u57fa\u6240\u5728\uff0c\u627e\u51fa\u6697\u85cf\u7684\u5899\uff0c\u5c31\u53ef\u4ee5\u8ba929\u5c42\u7684\u5899\u677e\u52a8\uff0c\u54e5\u54e5\u5c31\u53ef\u4ee5\u6316\u7a7f\u5b83\u3002", "\u2026\u2026\u4f60\u4e2a\u5047\u4ed7\u4e49\uff01", "\u5144\u5f1f\uff0c\u8d76\u5feb\u53bb\u51d1\u9f50\u88c5\u5907\uff0c\u6253\u8d25\u6768\u622c\uff0c", "\u54e5\u54e5\u5c31\u53d1\u8fbe\u4e86~\u563f\u563f~", "\u2026\u2026", "\u4e0a\u6b21\u8d81\u672c\u5143\u5e05\u81ea\u62a5\u5927\u540d\u7684\u65f6\u5019\uff0c\u7a81\u88ad\u672c\u5e05\uff0c\u672c\u5e05\u4e0d\u8ddf\u4f60\u8ba1\u8f83\uff0c\u5355\u6311\u8fd8\u662f\u7fa4\u6bb4\uff0c\u4f60\u81ea\u5df1\u9009\u3002", "\u6069\uff0c\u662f\u6761\u6c49\u5b50\uff0c\u4ffa\u5c31\u8ba4\u771f\u8ddf\u4f60\u6253\u4e00\u6b21\uff0c\u5355\u6311\uff01", "\u5355\u6311\u662f\u5427\uff0c\u4f60\u4e00\u4e2a\u5355\u6311\u6211\u4eec\u5168\u90e8\uff0c\u5f1f\u5144\u4eec\uff0c\u4e00\u8d77\u4e0a\uff01", "\u4f60\u4e2b\u4e0d\u5730\u9053\uff01", "\u4f60\u7684\u786e\u662f\u4e2a\u82f1\u96c4\uff0c\u96be\u602a\u5979\u4e00\u76f4\u5ff5\u5ff5\u4e0d\u5fd8\u2026\u2026", "\u8fc7\u5956\u8fc7\u5956\uff0c\u4f60\u7684\u90e8\u4e0b\u90fd\u8eba\u4e0b\u4e86\uff0c\u73b0\u5728\u8f6e\u5230\u4f60\u4e86\u2026\u2026", "\u6069~~\u8ba8\u538c\u6b7b\u4e86\uff0c\u6765\u4e86\u6765\u4e86\u2026\u2026", "\u4eba\u5bb6\u4eca\u5929\u8eab\u4f53\u4e0d\u65b9\u4fbf\uff0c\u6539\u5929\u518d\u6765\uff0c\u5148\u95ea\u4e86", "\u5929\u84ec\uff0c\u4f60\u6570\u6b21\u6218\u609f\u7a7a\u4e0d\u80dc\u5012\u7f62\u4e86\uff0c\u5e73\u65f6\u5e38\u5e38\u64c5\u81ea\u79bb\u5c97\uff0c\u53bb\u5e7f\u5bd2\u5bab\u628a\u599e\u2026\u2026\u6b7b\u7f6a\u53ef\u514d\uff0c\u6d3b\u7f6a\u96be\u9976\u3002", "\u542c\u8bf4\uff0c\u4e0b\u51e1\u6295\u80ce\uff0c\u5c31\u4f1a\u5815\u5165\u8f6e\u56de\uff0c\u5c31\u4f1a\u5fd8\u8bb0\u524d\u5c18\u5f80\u4e8b\u2026\u2026", "\u597d\uff0c\u6715\u5c31\u6210\u5168\u4f60\uff0c\u4e0b\u51e1\u4e4b\u524d\uff0c\u6709\u4ec0\u4e48\u8981\u6c42\u4e48\uff1f", "\u5929\u5929\u5927\u5403\u5927\u559d\uff0c\u5012\u5934\u7761\u89c9\uff0c\u751f\u6d3b\u5b89\u9038\u65e0\u8fb9\uff0c\u5fc3\u5bbd\u4f53\u80d6\u2026\u2026", "\u5f88\u597d\uff0c\u4f60\u7684\u5fc3\u610f\uff0c\u6715\u660e\u767d\u4e86\uff0c\u5b89\u5fc3\u53bb\u5427", "\u54c7\uff01\uff01\u6295\u80ce\u4e3a\u732a\uff1f\uff1f\uff01", "\u5929\u84ec\u5143\u5e05\u53d8\u6210\u4e86\u4e00\u53ea\u732a\uff0c\u88ab\u8d2c\u4e0b\u4e86\u51e1\u5c18", "\u2026\u2026\u771f\u9634\u9669\u2026\u2026", "\u54fc\uff0c\u54fc\uff0c\u5be1\u4eba\u5728\u56db\u5341\u4e5d\u5c42\u7b49\u4f60\uff0c\u54c7\u54c8\u54c8\u54c8\u54c8~\uff01\uff01", "\u672c\u6765\uff0c\u6218\u795e\u60c5\u5723\u7684\u540d\u53f7\u662f\u6211\u7684\uff1b\u5ae6\u5a25\u7684\u5fc3\uff0c\u8fdf\u65e9\u4e5f\u4f1a\u5f52\u5c5e\u4e8e\u6211\uff0c\u4f46\u662f\u4f60\u6765\u4e86\u4e4b\u540e\uff0c\u4e00\u5207\u90fd\u6539\u53d8\u4e86\u2026\u2026", "\u4f60\u559c\u6b22\u5979\uff0c\u8fd9\u4e48\u591a\u5e74\uff0c\u4f60\u4e3a\u4ec0\u4e48\u4e0d\u53bb\u627e\u5979\uff1f", "\u56e0\u4e3a\u6211\u662f\u6218\u795e\u60c5\u5723\uff0c\u662f\u4e0d\u80fd\u5931\u8d25\u7684\u2026\u2026", "\u4f60\u592a\u9a84\u50b2\u4e86\u2026\u2026", "\u65e0\u8bba\u5982\u4f55\uff0c\u6597\u795e\u548c\u6218\u795e\u8fd9\u4e00\u6218\uff0c\u662f\u6ce8\u5b9a\u7684\u2026\u2026", " ", "\u54c7\uff0c\u59d1\u5a18\u8eab\u9677\u4e09\u6627\u771f\u706b\u5f53\u4e2d\uff0c\u8981\u60f3\u529e\u6cd5\u5f00\u95e8\u706d\u706b\u2026\u2026", "\u59d1\u5a18\u9876\u4f4f\uff0c\u4ffa\u8001\u5b59\u6765\u6551\u4f60\uff01\uff01", "\u4e0d\u8981\u4e0d\u8981\u8fc7\u6765\uff01\uff01", "\u59d1\u5a18\u4f60\u6ca1\u4e8b\u5427\uff1f", "\u6b7b\u7334\u5b50\uff0c\u6708\u5bab\u9634\u51b7\uff0c\u59d1\u5976\u5976\u6211\u60f3\u84b8\u84b8\u6851\u62ff\uff0c\u6cbb\u591a\u5e74\u7684\u5173\u8282\u708e\u90fd\u4e0d\u884c\u2026\u2026", "\u2026\u2026", "\u6b7b\u7334\u5b50\uff0c\u4e0a\u6b21\u6843\u5b50\u7684\u4e8b\u60c5\u2026\u2026", "\u4ffa\u638c\u7ba1\u87e0\u6843\u56ed\uff0c\u5077\u5403\u4ed9\u6843\u4f55\u6b62\u5343\u767e\uff0c\u591a\u8ba42\u4e2a\uff0c\u7b97\u4ec0\u4e48\u2026\u2026", "\u5bb3\u4f60\u88ab\u9769\u9664\u4e86\u201c\u9f50\u5929\u5927\u5723\u201d\u7684\u4e0a\u4ed9\u4e4b\u4f4d\u2026\u2026", "\u4ffa\u8001\u5b59\u4e0d\u7a00\u7f55\u5929\u5bab\u7684\u4f4d\u5b50\uff0c~\u8d2c\u4e0b\u51e1\u5c18\u4ecd\u79f0\u738b\uff0c\u563f\u563f", "\u2026\u2026\u5728\u5929\u5bab\u51e0\u5343\u5e74\uff0c\u4ece\u6765\u6ca1\u6709\u4eba\u80af\u4e3a\u6211\u653e\u5f03\u4ed9\u4f4d\u2026\u2026\u5509\uff0c\u53ef\u60dc\u3002", "\u54fc\u54fc\uff0c\u5c45\u7136\u6253\u5230\u8fd9\u91cc\uff0c\u5b9e\u8bdd\u544a\u8bc9\u4f60\uff0c\u6240\u6709\u5929\u795e\u90fd\u5bf9\u4f60\u4e0d\u6ee1\uff0c\u8fd9\u6b21\u4f60\u88ab\u524a\u53bb\u4ed9\u7235\u6253\u5165\u5929\u7262\uff0c\u90fd\u662f\u8ba1\u5212\u4e4b\u4e2d\u3002", "\u90a3\u5ae6\u5a25\u5462\uff0c\u6843\u5b50\u5462\uff1f\u4e5f\u5728\u8ba1\u5212\u4e4b\u4e2d\uff1f\u4f60\u4eec\u6599\u5b9a\u4ffa\u4f1a\u7518\u5fc3\u9876\u7f6a\uff1f", "\u54c7\u54c8\u54c8\u54c8\u54c8~\uff01\u5929\u7f51\u6613\u9003\uff0c\u60c5\u4e1d\u96be\u65ad\uff0c\u4f60\u6709\u901a\u5929\u7684\u672c\u4e8b\uff0c\u4e5f\u96be\u8fc7\u8fd9\u4e00\u5173\u3002", "\u4e3a\u5979\u9876\u7f6a\uff0c\u4ffa\u4ece\u4e0d\u540e\u6094\uff0c\u73b0\u5728\uff0c\u662f\u4ffa\u4e86\u65ad\u6069\u6028\u7684\u65f6\u5019\u4e86\uff01\uff01", "\u5176\u5b9e\uff0c\u6715\u4e0d\u662f\u6253\u4e0d\u8fc7\u4f60\uff0c\u6715\u53ea\u4e0d\u8fc7\u79c1\u632a\u4e86\u56fd\u5e93\uff0c\u4e70\u4e86\u4f60\u7684\u76d8\u53e3\u2026\u2026", "\u4f60\u7ec8\u4e8e\u6253\u5230\u8fd9\u91cc\u4e86\u3002", "\u4f60\u5c45\u7136\u5728\u8fd9\u91cc\uff1f", "\u54c8\u54c8\u54c8\u54c8\uff0c\u8001\u592b\u4e00\u8def\u4fdd\u4f60\uff0c\u5c31\u662f\u4e3a\u4e86\u8ba9\u4f60\u5e2e\u6211\u626b\u6e05\u5929\u5ead\uff0c\u4f60\u7684\u6240\u505a\u6240\u4e3a\u2026", "\u4ffa\u6700\u6068\u7684\u5c31\u662f\u88ab\u4eba\u6b3a\u9a97\uff01\u6211\u2026\u2026\uff08\u609f\u7a7a\u4e45\u4e45\u5730\u9677\u5165\u4e86\u56de\u5fc6\uff09", "\u5929\u5ead\uff0c\u5929\u5ead\u53c8\u600e\u6837\uff1f\u5973\u4eba\u9a97\u6211\uff0c\u5144\u5f1f\u9a97\u6211\uff0c\u5982\u4eca\u4ffa\u8001\u5b59\u6ca1\u6709\u4ec0\u4e48\u53ef\u4ee5\u7559\u604b\u7684\uff0c\u56de\u82b1\u679c\u5c71\u7f62\u4e86\u3002", "\u5b7d\u755c\uff0c\u5929\u5ead\u5a01\u4eea\uff0c\u5c82\u80fd\u5bb9\u4f60\u5168\u8eab\u800c\u9000\uff01\uff01", "\u59d1\u5a18\uff0c\u4f60\u600e\u4e48\u4f1a\u88ab\u5173\u5728\u8fd9\u91cc\uff01", "\u5974\u5bb6\u6697\u4e2d\u52a9\u4f60\uff0c\u89e6\u72af\u5929\u6761\u2026\u2026", "\u7389\u5e1d\u8001\u513f\uff0c\u5f85\u6211\u6253\u70c2\u4f60\u7684\u91d1\u51a0\uff01\uff01\u59d1\u5a18\u4f60\u5148\u79bb\u5f00\uff0c\u7b49\u4ffa\u56de\u6765\uff01", "\u5509\u2026\u2026\u4f60\u53c8\u4f55\u82e6\u2026\u2026", "\u8fd9\u91cc\u6709\u74f6\u706b\u773c\u91d1\u775b\u724c\u773c\u5f71\u971c\uff0c\u53bb\u76b1\u6297\u8870\u8001\uff0c\u53ef\u4ee5\u770b\u6e05\u695a\u654c\u4eba\u7684\u672c\u8d28\uff0c\u91d1\u8272\u8d28\u611f\u8d34\u5408\u80a4\u8d28\uff0c\u6765\u81ea\u5df4\u9ece\uff0c\u4f60\u503c\u5f97\u62e5\u6709\u3002", "\u73b0\u5728\u6d82\u597d\u4e86\uff0c\u770b\u8d77\u6765\u55f2\u4e0d\u55f2~\uff1f", "\u6069\u2026\u2026\u672c\u6765\u662f\u53ea\u201c\u7334\u5996\u201d\uff0c\u73b0\u5728\u662f\u4e2a\u201c\u4eba\u5996\u201d\u3002", "\u5ae6\u5a25\u59d1\u5a18\uff0c\u60f3\u4e0d\u5230\u5728\u8fd9\u91cc\u9047\u5230\u4f60\u3002", "\u5927\u5723\uff0c\u8fd9\u662f\u6211\u4eb2\u624b\u917f\u5236\u7684\u5343\u5e74\u6708\u6842\u9732\uff0c\u559d\u4e0b\u5b83\uff0c\u72b9\u5982\u8131\u80ce\u6362\u9aa8\uff0c\u4f53\u529b\u5927\u589e\u3002", "\u54e6~\uff1f\u96be\u9053\u8fd9\u662f\u5b9a\u60c5\u4fe1\u7269\uff1f", "\u800c\u4e14\uff0c\u5b83\u8fd8\u53ef\u4ee5\u4f7f\u4eba\u5fd8\u8bb0\u7ea2\u5c18\u611f\u60c5\uff0c\u6211\u5e0c\u671b\u4f60\u80fd\u5fd8\u8bb0\u6211\u3002", "\u554a~\u54c8~\u7ed9\u6211\u4e00\u676f\u5fd8\u60c5\u6c34~\u6362\u6211\u4e00\u591c\u4e0d\u6d41\u6cea\u2026\u2026\u59d1\u5a18\uff0c\u4ffa\u51c6\u5907\u79bb\u5f00\u5929\u5ead\uff0c\u6211\u5e0c\u671b\u4f60\u8ddf\u6211\u4e00\u8d77\u8d70\u2026\u2026", "\u8fdd\u80cc\u5929\u6761\uff0c\u79c1\u5954\uff0c\u4f1a\u88ab\u6574\u4e2a\u5929\u754c\u4eba\u8089\u641c\u7d22\u7684\u2026\u2026", "\u79c1\u5954\uff1f\u4ffa\u8001\u5b59\u4e0d\u505a\u90a3\u7325\u7410\u4e4b\u4e8b\uff0c\u5f85\u4ffa\u6253\u4e0a\u7075\u9704\u5b9d\u6bbf\uff0c\u8ba9\u7389\u7687\u5927\u5e1d\u4eb2\u53e3\u7b54\u5e94\uff0c\u6574\u4e2a\u5929\u5ead\u8c01\u6562\u4e3a\u96be\u4f60\uff01\uff01", "\u5927\u5723\u4fdd\u91cd\uff0c\u6b64\u5730\u5974\u5bb6\u4e0d\u5b9c\u4e45\u7559\uff0c\u5974\u5bb6\u4e0d\u60f3\u8fde\u7d2f\u4f60\u2026\u2026", "\u59d1\u5a18\uff01\u59d1\u5a18\uff01", "\u5927\u5723\uff0c\u524d\u9762\u51f6\u9669\u96be\u6d4b\uff0c\u5974\u5bb6\u8fd9\u91cc\u6709\u70b9\u79c1\u623f\u94b1\uff0c\u9001\u7ed9\u4f60\u4e70\u70b9\u4ed9\u4e39\u6ecb\u8865\u8eab\u4f53\u5427\u2026\u2026", "\u2026\u2026\u59d1\u5a18\u5bf9\u6211\u4e00\u7247\u771f\u60c5\uff0c\u4ffa\u53d1\u8a93\u8981\u4e3a\u4f60\u6253\u4e0b\u4e00\u7247\u5929", "\u8ddf\u4ffa\u8d70\u5427\uff0c\u56de\u82b1\u679c\u5c71\u53bb\u2026\u2026", "\u8868\uff0c\u59d1\u5976\u5976\u6211\u4e3a\u4e86\u5929\u5bab\u62a4\u7167\uff0c\u629b\u5f03\u4e86\u524d\u592b\uff0c\u6211\u624d\u8868\u518d\u8ddf\u4f60\u4e0b\u51e1\uff0c\u4f60\u2026\u2026\u662f\u4e2a\u597d\u4eba\u2026\u2026(\u98d8\u8d70)", "......\u5973\u4eba\u5982\u8863\u670d\uff0c\u5144\u5f1f\u5982\u624b\u8db3\uff0c\u8001\u725b~\uff01\u4ffa\u6765\u5bfb\u4f60\uff01\uff01", "\u5b9d\u6247\u5b9d\u6247\u544a\u8bc9\u6211\uff0c\u8c01\u662f\u8fd9\u4e2a\u4e16\u754c\u4e0a\u6700\u578b\u6700\u731b\u7684\u7537\u4eba\uff1f", "\uff08\u6a21\u4eff\u6247\u5b50\u7684\u58f0\u97f3\uff09\u662f\u4f60~\u662f\u4f60~\u8fd8\u662f\u4f60", "\u771f\u81ea\u604b\u2026\u2026", "\u54c7~\uff01\u88ab\u4f60\u5077\u7aa5\u5230\u4e86\uff0c\u672c\u5c0a\u8be5\u6740\u4f60\u706d\u53e3\uff0c\u4f46\u662f\u73b0\u5728\u4f60\u8fd8\u4e0d\u914d\u672c\u5c0a\u51fa\u624b\u3002", "\u53ef\u6076\uff0c\u7b49\u4ffa\u8001\u5b59\u5148\u627e\u56de\u4ffa\u90a3\u6839\u5982\u610f\u68cd\u5b50\u518d\u6765\u6536\u62fe\u4f60\u2026\u2026\\cFFCC00\u5148\u53bb2\u5c42\u5929\u7262\u6551\u8001\u725b\uff0c\u8ba9\u4ed6\u66ff\u4ffa\u5f00\u6697\u5899\u7ed5\u8fc7\u53bb", "\u8001\u725b\uff0c\u4ffa\u6551\u4f60\u6765\u4e86~\uff01", "\u5e73\u65f6\u8ba9\u4f60\u5e2e\u5fd9\uff0c\u8001\u662f\u63a8\u4e09\u963b\u56db\uff0c\u8fd9\u6b21\u8fd9\u4e48\u723d\u5feb\uff0c\u4e00\u5b9a\u6709\u95ee\u9898~", "\u563f\u563f\uff0c35\u5c42\u6709\u4e2a\u4e09\u773c\u5c0f\u767d\u8138\u592a\u6076\u5fc3\uff0c\u66ff\u4ffa\u706d\u4e86\u4ed6~~", "\u4ffa\u5bf9\u5c0f\u767d\u8138\u6728\u6709\u5174\u8da3\u2026\u2026", "\u90a3\u5c31\u60f3\u529e\u6cd5\u5e2e\u4ffa\u7ed5\u8fc7\u53bb~~", "\u563f\u563f\uff0c\u5f00\u81ea\u5df1\u7684\u6d1e\uff0c\u8ba9\u522b\u4eba\u8bf4\u53bb\u5427~~~", "?\u6709\u6839\u6346\u4ed9\u7ef3\uff1f\u4f3c\u4e4e\u53ef\u4ee5\u514b\u5236\u4f4f\u90a3\u4e2a\u4e09\u773c\u5c0f\u767d\u8138\uff0c\u6069\uff0c\u641e\u5b9a\u4ed6\uff0c\u6346\u7ed1\u4ed6\uff0c\u62ff\u4ed6\u7684\u82ad\u8549\u6247\uff0c\u54e6\u4e5f~", "\u6709\u82ad\u8549\u6247\u53ef\u4ee5\u706d\u706b\u4e86\uff0c\u5ae6\u5a25\u59d1\u5a18\uff0c\u4ffa\u6765\u5566~~\uff01\uff01\u5bf9\u4e86\uff0c\u8fd8\u6709\u6211\u7684\u5982\u610f\u91d1\u7b8d\u68d2\u3002", "\u6b22\u8fce\u4f60\u6765\u5230\u5929\u5bab\u4e16\u754c\uff0c\u6211\u662f\u4f60\u7684\u5e08\u5085\u83e9\u63d0\u8001\u7956\u3002", "\u5728\u8fd9\u91cc\u6211\u4e0d\u4f1a\u6559\u4f60\u4e03\u5341\u4e8c\u53d8\uff0c\u4f46\u662f\u6211\u4f1a\u6559\u4f60\u600e\u4e48\u6e38\u5386\u5929\u5bab\u3002", "\u4e3a\u5e08\u77e5\u9053\u4f60\u8981\u5927\u95f9\u5929\u5bab\uff0c\u7279\u610f\u5343\u91cc\u4f20\u97f3\uff0c\u63d0\u4f9b\u8fdc\u7a0b\u89c6\u9891\u652f\u6301\uff0c\u5f53\u7136\uff0c\u5982\u679c\u4f60\u5acc\u4e3a\u5e08\u7f57\u55e6\uff0c\u4e5f\u53ef\u4ee5\u5728\u6e38\u620f\u83dc\u5355\u4e2d\u9009\u62e9\u8df3\u8fc7\u6559\u7a0b\u3002", "\u597d\u4e86\uff0c\u73b0\u5728\u8bf7\u8bd5\u7740\\cFFCC00\u6309\u65b9\u5411\u952e\u79fb\u52a8\u5230\u8fd9\u91cc\u3002", "\u5f88\u597d\uff0c\u4f60\u5df2\u7ecf\u5b66\u4f1a\u592a\u7a7a\u6b65\u4e86\u3002", "\u5728\u4f60\u9762\u524d\u6709\u4e00\u9053\u9ec4\u8272\u7684\u95e8\uff0c\u4f60\u65e0\u6cd5\u8fc7\u53bb\u3002", "\u4f60\u53ef\u4ee5\u770b\u5230\u8fd9\u91cc\u6709\u628a\u9ec4\u94a5\u5319\uff0c\u5b83\u53ef\u4ee5\u5f00\u542f\u8fd9\u9053\u95e8\u3002", "\u73b0\u5728\\cFFCC00\u79fb\u52a8\u5230\u8fd9\u91cc\uff0c\u518d\u56de\u6765\u5f00\u95e8\u3002", "\u7b49\u7b49\uff01", "\u524d\u9762\u6709\u53ea\u6321\u8def\u7684\u72d7\u3002\u4f60\u9700\u8981\u6253\u8d25\u5b83\u624d\u80fd\u8d70\u8fc7\u53bb\u3002", "\u73b0\u5728\uff0c\\cFFCC00\u8bf7\u8bd5\u7740\u79fb\u52a8\u5230\u5b83\u7684\u4f4d\u7f6e\u4e0a\uff0c\u4e0e\u5b83\u6218\u6597\u5427\u3002", "\u542c\u5230\u8f70\u9686\u58f0\u4e86\u5427\uff0c\u56e0\u4e3a\u4f60\u6253\u8d25\u4e86\\cFFCC00\u5b88\u536b\u5c01\u5370\u95e8\u7684\u654c\u4eba\u3002", "\u6240\u4ee5\u8fd9\u91cc\u7684\\cFFCC00\u5c01\u5370\u95e8\\cF8F8F8\u5c31\u88ab\u6253\u5f00\u4e86\u3002", "\u5728\u6218\u6597\u4e2d\u4f60\u53ef\u80fd\u4f1a\u635f\u5931\u8840\u91cf\u3002", "\u8fd9\u91cc\u6709\u4e2a\\cFFCC00\u5c0f\u4ed9\u6843\uff0c\u53ef\u4ee5\u56de\u590d\u4f60\u7684\u8840\u91cf\u3002", "\u5982\u679c\u8840\u91cf\u4e0d\u8db3\uff0c\u4f60\u5c06\u65e0\u6cd5\u6311\u6218\u654c\u4eba\u3002", "\u53c8\u5230\u4e86\u5b66\u4e60\u65f6\u95f4\u3002", "\u4f60\u7684\u80fd\u529b\u662f\u53ef\u4ee5\u63d0\u5347\u7684\uff0c\u5305\u62ec\u653b\u51fb\u3001\u9632\u5fa1\u3001\u8840\u91cf\u3002", "\u8fd9\u91cc\u6709\u4e2a\u84dd\u8272\u4ed9\u4e39\uff0c\u5b66\u540d\u662f\u201c\u9632\u5fa1\u4ed9\u4e39\u201d\uff0c\u670d\u4e0b\u5b83\uff0c\u53ef\u4ee5\u63d0\u5347\u4f60\u7684\u9632\u5fa1\u529b\uff0c\u8ba9\u4f60\u6218\u6597\u66f4\u6301\u4e45\u3002", "\u8bb0\u4f4f\uff0c\\cFFCC00\u5929\u5ead\u5c42\u6570\u8d8a\u9ad8\uff0c\u4ed9\u4e39\u836f\u6548\u8d8a\u5927\u3002", "\u73b0\u5728\uff0c\u5403\u4e86\u5b83\uff0c\u6251\u8fc7\u53bb\u505a\u6389\u524d\u9762\u90a3\u6761\u72d7\uff0c\u4f60\u4f1a\u53d1\u73b0\u635f\u8840\u5c11\u4e86\u3002", "\u770b\u5230\u4e0a\u9762\u7684\u84dd\u95e8\u4e86\u5417\uff0c\u5b83\u53ea\u80fd\u7528\u84dd\u8272\u7684\u94a5\u5319\u6253\u5f00\u3002", "\u5b83\u88ab\u85cf\u5728\u8fd9\u91cc\uff0c\\cFFCC00\u5148\u62ff\u5230\u5b83\u5427\u3002", "\u4f60\u53d1\u73b0\u4e86\u4e00\u9053\u7ea2\u95e8\u3002\u8fd9\u79cd\u95e8\u5f88\u5c11\u89c1\uff0c\u5fc5\u987b\u7528\u7ea2\u94a5\u5319\u624d\u80fd\u6253\u5f00\u3002", "\u5b83\u88ab\u85cf\u5728\u8fd9\u91cc\uff0c\\cFFCC00\u8bf7\u5148\u5f97\u5230\u5b83\uff0c\u518d\u56de\u6765\u5f00\u95e8\u3002", "\u4f60\u627e\u5230\u4e86\u4e00\u628a\u7ea2\u94a5\u5319\uff0c\u8fd9\u79cd\u94a5\u5319\u6bd4\u8f83\u7a00\u5c11\u3002", "\u8bd5\u7740\\cFFCC00\u7528\u5b83\u5f00\u542f\u8fd9\u91cc\u7684\u7ea2\u95e8\u3002", "\u5f88\u597d\uff0c\u8fd9\u5c42\u5df2\u7ecf\u63a5\u8fd1\u5c3d\u5934\u3002", "\u4f60\u4f1a\u53d1\u73b0\u8fd9\u6837\u7684\u7ea2\u8272\u4f20\u9001\u70b9\uff0c\u5b83\u53ef\u4ee5\u8ba9\u4f60\u5411\u4e0a\u4e00\u5c42\u697c\u3002", "\u4e0d\u8fc7\uff0c\u522b\u6025\u7740\u79bb\u5f00\u3002", "\u4f60\u662f\u4e0d\u662f\u5df2\u7ecf\u53d1\u73b0\u8fd9\u91cc\u6709\u4e2a\u9053\u5177\u4e86\u5417\uff1f", "\u8fd9\u91cc\u6709\u4e2a\u7ea2\u8272\u4ed9\u4e39\uff0c\u5b66\u540d\u662f\u201c\u653b\u51fb\u4ed9\u4e39\u201d\uff0c\u670d\u4e0b\u5b83\uff0c\u53ef\u4ee5\u63d0\u5347\u4f60\u7684\u653b\u51fb\u529b\uff0c\u8ba9\u4f60\u6218\u6597\u66f4\u72c2\u91ce\u3002", "\u4f46\u662f\u8fd9\u91cc\u597d\u8c61\u4e0d\u901a\u2026\u2026", "\u522b\u6025\uff01\u4fd7\u8bdd\u8bf4\u8f66\u5230\u5c71\u524d\u5fc5\u6709\u8def\uff0c\u5728\u5929\u5bab\u7684\u5f88\u591a\u5c42\u4e2d\u4f1a\u6709\u9690\u85cf\u7684\u8def\uff0c\u66f4\u591a\u60ca\u559c\u66f4\u591a\u6b22\u7b11\uff0c\u5c31\u5728\u9690\u85cf\u8def\u2026\u2026", "\u73b0\u5728\uff0c\u79fb\u52a8\u5230\u8fd9\u91cc\uff0c\u4f60\u5c31\u4f1a\u53d1\u73b0\u5b83\u3002", "\u8981\u8bb0\u4f4f\uff0c\\cFFCC00\u5f88\u591a\u5c42\u91cc\u90fd\u4f1a\u6709\u9690\u85cf\u7684\u4e1c\u897f\uff0c\u8bd5\u7740\u53bb\u63a2\u7d22\u5427\u3002", "\u770b\u5230\u4f60\u4e0a\u6765\u7684\u8def\u4e86\u5417\uff1f", "\u84dd\u8272\u7684\u4f20\u9001\u70b9\u53ef\u4ee5\u8ba9\u4f60\u5411\u4e0b\u4e00\u5c42\u697c\u3002", "\u4f60\u53ef\u80fd\u65e0\u6cd5\u51fb\u8d25\u8fd9\u4e2a\u654c\u4eba\uff0c\u7ed5\u9053\u4e5f\u662f\u524d\u8fdb\u7684\u529e\u6cd5\u3002", "\u90a3\u4e48\uff0c\u5982\u4f55\u5224\u65ad\u4e00\u4e2a\u654c\u4eba\u7684\u5f3a\u5f31\u5462\uff1f", "\u6e38\u620f\u4e2d\u4f60\u4f1a\u83b7\u5f97\u8fd9\u4ef6\u5b9d\u7269\uff0c\u5b83\u53eb\\cFFCC00\u706b\u773c\u91d1\u775b\u724c\u773c\u5f71\u818f\u3002", "\u6d82\u62b9\u4e00\u70b9\u5728\u773c\u76ae\u4e0a\uff0c\u4f60\u53ef\u4ee5\u770b\u7834\u654c\u60c5\uff0c\u8fd8\u53ef\u4ee5\u53bb\u9664\u773c\u89d2\u7eb9\u3002", "\u6211\u5e2e\u4f60\u5f00\u51fa\u4e86\u4e00\u6761\u8def\uff0c\u4f60\u53ef\u4ee5\u53bb\u53d6\u5b83\u4e86\u3002", "\u73b0\u5728\u4f60\u53ef\u4ee5\u53c2\u7167\u4f7f\u7528\u8bf4\u660e\u6765\u4f7f\u7528\u5b83\u4e86\u3002", "\u9664\u4e86\u63095/OK\u952e\u67e5\u770b\u654c\u4eba\u5bf9\u4f60\u9020\u6210\u7684\u4f24\u5bb3\u4ee5\u5916\u3002", "\u4f60\u8fd8\u53ef\u4ee5\u6309\u5de6\u8f6f\u952e\u6253\u5f00\u7269\u54c1\u680f\u3002", "\u9009\u62e9\u8be5\u7269\u54c1\uff0c\u6309\u786e\u8ba4\u952e\u67e5\u770b\u66f4\u8be6\u7ec6\u7684\u654c\u4eba\u4fe1\u606f\u3002", "\u4f60\u7ad9\u5728\u8fd9\u5341\u5b57\u8857\u5934\u4e0a\uff0c\u627e\u4e0d\u5230\u6765\u53bb\u7684\u65b9\u5411\u3002", "\u4e0d\u8981\u614c\u5f20\uff0c\u8bd5\u7740\u67e5\u770b\u4e0b\u8fd9\u91cc\u9053\u5177\u548c\u654c\u4eba\u7684\u5206\u5e03\u5f62\u52bf\u3002", "\u4e0a\u53bb\u7684\u4f20\u9001\u70b9\u5728\u8fd9\u91cc\u3002", "\u5982\u679c\u4f60\u65e0\u6cd5\u9a6c\u4e0a\u51fb\u8d25\u8fd9\u4e2a\u5b88\u536b\u3002", "\u5c31\u8bd5\u7740\u5c06\u5730\u56fe\u4e0a\u7684\u4ed9\u4e39\u548c\u4ed9\u6843\u5403\u6389\uff0c\u7136\u540e\u4f60\u5c31\u53ef\u4ee5\u6218\u80dc\u5b83\u4e86\u3002", "\u8bb0\u4f4f\uff0c\u5982\u679c\u524d\u65b9\u6709\u4e00\u7fa4\u654c\u4eba\u5728\u5411\u4f60\u6325\u624b\uff0c\u5343\u4e07\u522b\u51b2\u52a8\u3002", "\u7262\u8bb0\\cFFCC00\u201c\u5148\u5403\u4ed9\u4e39\u540e\u8089\u640f\u201d\\cF8F8F8\u662f\u51cf\u5c11\u635f\u8840\u7684\u7b2c\u4e00\u6cd5\u5219\u3002", "\u54e6\u563f\u563f~\u4e3a\u5e08\u8981\u7ee7\u7eed\u4eab\u53d7\u6e21\u5047\u5566~\u3002", "\u5728\u8fd9\u4e4b\u524d\u6211\u4f1a\u4f20\u6388\u4f60\u4e94\u767e\u5e74\u529f\u529b\uff0c\u518d\u9001\u4f60\u4e24\u4ef6\u4e1c\u897f\u9632\u8eab\u3002", "\u8fd9\u662f\u4e00\u628a\u6b66\u5668\uff0c\u80fd\u8ba9\u4f60\u63d0\u5347\u5f88\u9ad8\u7684\u653b\u51fb\u3002", "\u5728\u6e38\u620f\u7684\\cFFCC00\u6bcf10\u5c42\u90fd\u6709\u4e00\u628a\u65b0\u6b66\u5668\u3002", "\u5982\u679c\u4f60\u80fd\u65e9\u70b9\u83b7\u5f97\u5b83\uff0c\u5c31\u80fd\u8f7b\u677e\u5e94\u5bf9\u654c\u4eba\uff0c\u8d70\u5f97\u66f4\u8fdc\u3002", "\u540c\u6837\uff0c\u8fd9\u662f\u4e00\u4ef6\u9632\u5177\uff0c\u80fd\u63d0\u9ad8\u4f60\u7684\u9632\u5fa1\u3002", "\u6211\u73b0\u5728\u5e2e\u4f60\u6253\u5f00\u8fd9\u9053\u5899\uff0c\u5728\u4e00\u822c\u60c5\u51b5\u4e0b\uff0c\u5b83\u662f\u65e0\u6cd5\u51fb\u788e\u7684\u3002", "\u5bf9\u4e86\uff0c\u7ed9\u4f60\u4ecb\u7ecd\u4e00\u4e2a\u5929\u5bab\u4e0a\u7684\u670b\u53cb\u3002", "\u8fd9\u662f\u4e3a\u5e08\u7684\u8001\u670b\u53cb\uff0c\\cFFCC00\u592a\u767d\u91d1\u661f\\cF8F8F8\uff0c\u4ed6\u4f1a\u6697\u4e2d\u5e2e\u52a9\u4f60\u7684\u3002", "\u606d\u559c\uff0c\u4f60\u5df2\u7ecf\u6bd5\u4e1a\u4e86\uff0c\u6211\u518d\u4f20\u6388\u4f60\u4e94\u767e\u5e74\u7684\u529f\u529b\u3002", "\u8fd8\u7ed9\u4f60\u51c6\u5907\u4e86\u4e24\u4ef6\u795e\u5668\uff0c\u628a\u5b83\u4eec\u6536\u4e0b\u5427\u3002", "\u8981\u8bb0\u4f4f\uff0c\u5f80\u524d\u4f60\u5c06\u9762\u5bf9\u7684\u4e0d\u662f\u4e00\u4e2a\u654c\u4eba\uff0c\u800c\u662f\u6574\u4e2a\u5929\u5bab\u3002", "\u634f\u54c8\u54c8~\u4eca\u5929\u98ce\u548c\u65e5\u4e3d\uff0c\u6715\u5fc3\u60c5\u5f88\u597d~\uff01", "\u5440~\uff01\u54ea\u91cc\u94bb\u51fa\u4e00\u53ea\u679c\u5b50\u72f8\uff01\u9884\u9632\u975e\u5178\uff01\u5de6\u53f3\u4e0e\u6211\u62ff\u4e0b\uff01", "\u4ec0\u4e48\u7834\u7687\u5e1d\uff0c\u4e94\u8c37\u4e0d\u5206\uff0c\u516d\u755c\u4e0d\u8fa8\u2026\u2026\u7389\u5e1d\u8001\u513f\uff0c\u4ffa\u4e43\u82b1\u679c\u5c71\u7b2c\u4e00\u5c4a\u578b\u79c0\u51a0\u519b\uff0c\u7f8e\u7334\u738b\u5b59\u609f\u7a7a\uff01\uff01", "\u90fd\u7ed9\u6211\u4e0a~\uff01", "\u5de8\u73b2\u795e\uff0c\u6123\u5728\u90a3\u91cc\u505a\u4ec0\u4e48\uff1f", "\u965b\u4e0b\uff0c\u4eba\u5bb6\u662f\u5973\u5b69\u5b50\u561b\uff0c\u6700\u6015\u6bdb\u8338\u8338\u5730\u5c0f\u52a8\u7269\u4e86\u2026\u2026", "\u5c11\u5e9f\u8bdd\uff01\u60f3\u88ab\u780d\u5934\u554a\uff01", "\u965b\u4e0b\uff0c\u4e09\u592a\u5b50\u8bf7\u6218\uff01", "\u7389\u5e1d\u8001\u513f\uff0c\u4ffa\u8001\u5b59\u8981\u505a\u9f50\u5929\u5927\u5723\uff01\uff01", "\u55f7~~\u5b83\u8fc7\u6765\u4e86\u5b83\u8fc7\u6765\u4e86\uff0c\u8bf7\u4f60\u4e2a\u5934\u554a\uff0c\u8d76\u7d27\u9876\u4e0a\u5148~~", "\u62a4\u9a7e~\uff01\u62a4\u9a7e~\uff01", "\u8c01~\uff01\u662f\u5929\u5ead\u7b2c\u4e00\u578b\u7537~\uff01\uff01", "\u662f\u4f60~\uff01\u662f\u4f60~\uff01", "\u8c01~\uff01\u662f\u5929\u5ead\u7b2c\u4e00\u731b\u7537~\uff01\uff01", "\u662f\u4f60~\uff01\u662f\u4f60~\uff01", "\u8c01~\uff01\u662f\u5929\u5ead\u7b2c\u4e00\u660e\u661f\u6218\u795e~\uff01\uff01", "\u4f60\u662f\u7535\uff0c\u4f60\u662f\u5149\uff0c\u4f60\u662f\u552f\u4e00\u5730\u795e\u8bdd\uff0c\u4f60\u4e3b\u5bb0\uff0c\u6211\u5d07\u62dc~\u6ca1\u6709\u66f4\u597d\u7684\u529e\u6cd5~~\uff01", "\u965b\u4e0b\u2026\u2026\u5455\u2026\u2026", "\u81e3\u89c9\u5f97\u5427\u2026\u2026\u5455\u2026\u2026", "\u8fd8\u662f\u4e0d\u8981\u8ba9\u4e8c\u90ce\u795e\u7ee7\u7eed\u4e0b\u53bb\u4e86\uff0c\u5c3d\u5feb\u5e73\u606f\u8fd9\u6b21\u4e8b\u4ef6\uff0c\u7ed9\u5b59\u609f\u7a7a\u5c01\u4e2a\u5b98\u7b97\u4e86\uff01\uff01", "\u5455~~\u6715\u4e5f\u662f\u8fd9\u4e48\u60f3\u6ef4~~\u5594\u83b1\uff0c\u5c31\u5c01\u5b59\u609f\u7a7a\u4e3a\u9f50\u5929\u5927\u5723\uff0c\u638c\u7ba1\u87e0\u6843\u56ed\uff01", "\u563f\u563f\uff0c\u4ffa\u8001\u5b59\u5c31\u9886\u4e86~\uff01\u591a\u8c22~\uff01", "\u5996\u2026\u5996\u602a\uff0c\u4ed6\u7684\u773c\u775b\u95ea\u7740\u7ea2\u5149\u2026\u2026\u592a\u53ef\u6015\u4e86", "\u5c45\u7136\u9000\u7f29\uff0c\u4f60\u8fd9\u6ca1\u7528\u7684\u4e1c\u897f\u3002", "\u5662~~~~~~\u6ee1\u56ed\u4ed9\u6843\u6210\u719f\uff0c\u715e\u662f\u8bf1\u4eba\uff01\uff01", "\u8ba9\u4ffa\u8001\u5b59\u56db\u5904\u901b\u901b~~", "\u609f\u7a7a~\u609f\u7a7a~\uff01", "\u4e3a\u4ec0\u4e48\u6709\u53ea\u9171\u6cb9\u86e4\u87c6\u8ddf\u6211\u5343\u91cc\u4f20\u97f3\uff1f", "\u662f\u5e08\u7236\u6211\u554a\uff01", "\u5e08\u7236\uff1f\u4e3a\u4ec0\u4e48\u5316\u4e2a\u86e4\u87c6\u5986\uff1f", "\u4e3a\u5e08\u5728\u5370\u5ea6\uff0c\u6cd5\u672f\u4ea4\u6d41\u517c\u6e21\u5047\uff0c\u65e5\u5149\u6d74\u52a0\u987f\u987f\u5496\u55b1\u996d\uff0c\u5634\u5df4\u4e0a\u706b\u3002", "\u5982\u6765\u628a\u5b59\u609f\u7a7a\u5c01\u5370\u4e8e\u4e94\u6307\u5c71\u4e0b\uff0c\u4f34\u968f\u7740\u6240\u6709\u6069\u6028\u60c5\u4ec7\uff0c\u6b32\u77e5\u540e\u4e8b\u5982\u4f55\uff0c\u656c\u8bf7\u671f\u5f85\u300a\u897f\u6e38\u8bb0\u4e8c\u4e4b\u5927\u5723\u53d6\u7ecf\u300b", "\u7231\u5f92\u554a\uff0c\u4f60\u4e00\u4e2a\u4eba\u8981\u7ee7\u7eed\u6311\u6218\u5929\u5bab\uff0c\u4e3a\u5e08\u4e0d\u653e\u5fc3\uff0c\u7ed9\u4f60\u4e70\u4e86\u4efd\u4fdd\u9669\u3002", "\u5e08\u5085\u591f\u4e49\u6c14\u3002", "\uff08\u63a5\u8fc7\u4fdd\u5355\uff09\u53d7\u76ca\u4eba...\u201c\u83e9\u63d0\u8001\u7956\u201d", "\u6211\u7684\u5f92\u513f\u554a\uff0c\u4f60\u5df2\u7ecf\u9677\u5165\u6df7\u6c8c\u4e16\u754c\u3002\u4e5f\u5c31\u4eba\u4eec\u5e38\u8bf4\u7684\u7cbe\u795e\u5206\u88c2\u75c7\u3002", "\u8fd9\u662f\u7531\u4e8e\u79cd\u79cd\u611f\u60c5\u7ea0\u845b\u5f15\u53d1\u7684\uff0c\u5982\u679c\u4f60\u60f3\u51fa\u6765\uff0c\u5c31\u8981\u6218\u80dc\u4ed6\u4eec\u3002", "\u4e0d\uff0c\u662f\u6218\u80dc\u81ea\u5df1\u3002\u8981\u8ba9\u8fd9\u5929\u2026\u518d\u4e5f\u906e\u4e0d\u4f4f\u4f60\u7684\u773c\u3002", "\u6211\u8981\u63d0\u9192\u4f60\u7684\u662f\uff0c\u8fd9\u4e2a\u4e16\u754c\u91cc\uff0c\u6240\u6709\u7684\u654c\u4eba\u90fd\u4f1a\u6bd4\u539f\u6765\u66f4\u5f3a\uff0c\u5f53\u7136\u4f60\u7684\u80fd\u529b\u4e5f\u4f1a\u63d0\u5347\u66f4\u591a\u3002", "\u53bb\u5427\u2026\u2026\u52ab\u96be\u5728\u6240\u96be\u514d\u3002"};
        this.f_int_113 = this.levelScriptLines.length;
        this.f_bool_arr_06 = new boolean[this.f_int_113];
        this.f_int_118 = 0;
        this.bossEventSpawns = new byte[][]{{29, 1, 3, 0, 29, 2, 3, 0, 29, 3, 3, 0, 26, 1, 4, 0, 26, 2, 4, 0, 26, 3, 4, 0, 30, 9, 3, 0, 30, 10, 3, 0, 30, 11, 3, 0, 32, 9, 4, 0, 32, 10, 4, 0, 32, 11, 4, 0}, {29, 4, 4, 0, 29, 4, 5, 0, 29, 5, 4, 0, 26, 7, 4, 0, 26, 8, 4, 0, 26, 8, 5, 0, 30, 4, 7, 0, 30, 4, 8, 0, 30, 5, 8, 0, 32, 8, 8, 0, 32, 7, 8, 0, 32, 8, 7, 0}, {27, 6, 5, 0, 27, 5, 6, 0, 27, 7, 6, 0, 27, 6, 7, 0}, {17, 6, 4, 0}, {16, 5, 5, 0}, {24, 6, 2, 0, 32, 5, 1, 0, 32, 6, 1, 0, 32, 7, 1, 0, 27, 6, 6, 0}, {32, 2, 3, 0, 32, 3, 3, 0, 32, 4, 3, 0, 26, 8, 3, 0, 26, 9, 3, 0, 26, 10, 3, 0, 29, 3, 5, 0, 29, 4, 5, 0, 29, 5, 5, 0, 30, 7, 5, 0, 30, 8, 5, 0, 30, 9, 5, 0}, {32, 4, 4, 0, 32, 5, 4, 0, 32, 6, 4, 0}};
        this.bossTypeOrder = new byte[]{75, 74, 68, 67, 69, 71};
        this.f_bool_15 = false;
        this.f_RecordStore_00 = null;
        this.f_RecordEnumeration_00 = null;
        this.f_ByteArrayOutputStream_00 = null;
        this.f_DataOutputStream_00 = null;
        this.f_DataInputStream_00 = null;
        this.f_byte_arr_27 = null;
        this.f_byte_arr2_05 = new byte[56][];
        this.f_short_arr_05 = new short[64];
        this.f_short_arr_06 = new short[64];
        this.f_short_arr_07 = new short[64];
        this.f_short_arr_08 = new short[64];
        this.f_byte_arr_30 = new byte[30];
        this.f_int_arr_30 = new int[30];
        this.f_short_arr_09 = new short[30];
        this.f_short_arr_10 = new short[30];
        this.f_byte_arr_31 = new byte[30];
        this.f_bool_arr_08 = new boolean[30];
        this.f_byte_arr_32 = new byte[]{10, 6, -14, -26, -36, -38, -39, -41, -43};
        this.f_byte_arr_33 = new byte[]{16, 4, -10, 0, 14, 12, 14, 11, 13};
        this.f_byte_arr_34 = new byte[]{9, 22, 34, 41, 51, 52, 54, 55, 56};
        this.f_byte_arr_35 = new byte[]{15, -2, -19, -1, 21, 17, 21, 19, 21};
        this.f_byte_arr_36 = new byte[]{9, -1, -10, -22, -26, -27, -29, -30, -32};
        this.f_byte_arr_37 = new byte[]{20, 1, -15, 0, 24, 20, 24, 20, 24};
        this.f_bool_17 = true;
        String[] stringArray = new String[]{"\u754c\u9762", "\u83dc\u5355", "\u5730\u56fe", "\u80cc\u666f", "\u4eba\u7269", "\u7ec4\u4ef6", "\u8868\u60c5", "\u6548\u679c", "\u811a\u672c", "\u7f13\u5b58", "\u697c\u5c42", "\u89d2\u8272", "\u8bbe\u5b9a", "\u654c\u4eba", "\u5f15\u5b50", "\u7ed3\u5c40", "\u8f7d\u5165\u8fdb\u5ea6"};
        this.f_int_143 = 0;
        this.f_int_144 = 0;
        this.f_int_145 = 0;
        int[] nArray5 = new int[]{0xEEEEEE, 0xCCCCCC, 0xAAAAAA, 0x888888, 0x666666, 0x444444, 0x222222, 0x111111};
        this.f_int_arr_32 = new int[]{0xFF0000, 0xFF9000, 16776194, 1244928, 65478, 26367, 14156031};
        this.f_bool_20 = false;
        this.f_bool_21 = false;
        this.f_bool_22 = false;
        this.f_byte_26 = 0;
        this.difficultyMultipliers = new int[]{1, 30, 60, 80, 100};
        this.f_byte_arr_43 = new byte[]{29, 31, 39, 15, 0, 31, 29, 10, 0, 0, 37, 31, 36, 0, 32, 29};
        this.f_byte_arr_44 = new byte[]{1, 2, 0, 3, 4, 1, 2, 0};
        this.f_int_arr_34 = new int[]{4, 2, 2, 2, 2, 2, 2};
        this.f_int_arr_35 = new int[this.f_int_arr_34.length];
        this.f_String_arr_15 = new String[]{"logostart.mid", "logoquit.mid", "menu.mid", "game.mid"};
        this.f_int_155 = 60;
        this.f_bool_29 = true;
        this.f_byte_28 = (byte)-1;
        this.f_int_arr_36 = new int[]{4202520, -1};
        int[] nArray6 = new int[]{2555941, 6436695};
        int[] nArray7 = new int[]{2555941, 10126750};
        int[] nArray8 = new int[]{12342908, 15039118};
        int[] nArray9 = new int[]{0x222222, 0xFFDF7D};
        this.f_byte_arr_45 = new byte[]{87, 18, 9};
        this.f_int_arr_37 = new int[3];
        this.f_byte_arr_46 = new byte[]{0, 5, 5, 5, 10, 4, 14, 7, 21, 4, 25, 3, 28, 3, 31, 4, 35, 3, 38, 4, 42, 4, 46, 4, 50, 4, 54, 3, 57, 3, 60, 4, 64, 2, 66, 2, 68, 4, 72, 4, 76, 5};
        this.f_int_arr2_03 = new int[26][7];
        this.f_String_arr_16 = new String[]{"\u624b\u673a\u53f7", "\u5bc6\u7801", "\u8bd5\u73a9", "\u6ce8\u518c", "\u5e2e\u52a9", "\u6ce8\u518c\u5e10\u53f7", "\u6ce8\u518c\u5bc6\u7801", "\u6ce8\u518c\u624b\u673a\u53f7\u5fc5\u987b\u4e3a\u672c\u673a\uff0c\u9a8c\u8bc1\u6210\u529f\u540e\u5c06\u83b7\u5f97200\u6e38\u620f\u5e01", "\u6b22\u8fce\u767b\u9646\u96ea\u9ca4\u9c7c\u5e73\u53f0", "\u786e\u5b9a", "\u53d6\u6d88", "\u9000\u51fa", "\u8054\u7f51\u4e2d", "\u6b63\u5728\u91cd\u8bd5", "\u8054\u7f51\u8d85\u65f6", "\u670d\u52a1\u5668\u6ca1\u6709\u54cd\u5e94", "\u767b\u9646", "\u5269\u4f59\u70b9\u6570", "\u67e5\u8be2\u8bb0\u5f55", "\u5145\u503c", "\u4e2a\u4eba\u4fe1\u606f", "\u5e10\u53f7\u6216\u5bc6\u7801\u8f93\u5165\u6709\u8bef", "\u8bf7\u8f93\u516511\u4f4d\u624b\u673a\u53f7", "\u8bf7\u8f93\u51656-10\u4f4d\u5bc6\u7801", "\u65e5\u671f", "\u91d1\u989d", "\u672c\u6708\u4ed8\u8d39\u8bb0\u5f55", "\u8fdb\u5165\u6e38\u620f", "\u8bf7\u786e\u8ba4\u60a8\u586b\u5199\u7684\u662f\u5f53\u524d\u624b\u673a\u53f7\uff0c\u5426\u5219\u4f1a\u5bfc\u81f4\u6ce8\u518c\u5931\u8d25\u3002\u8d44\u8d392\u5143\uff0c\u9700\u8981\u53d1\u90011\u6761\u77ed\u4fe1\uff0c2\u5143/\u6761\uff0c\u4e0d\u542b\u901a\u4fe1\u8d39\u3002\u662f\u5426\u6ce8\u518c\uff1f", "\u5df2\u53d1\u9001\u6210\u529f\uff0c\u8bf7\u7b49\u5f85\u7cfb\u7edf\u9a8c\u8bc1\u540e\u624d\u80fd\u767b\u9646", "\u53d1\u9001\u5931\u8d25\uff0c\u8bf7\u91cd\u8bd5", "\u7c7b\u578b", "\u5e8f\u5217\u53f7", "\u5bc6\u7801", "\u91d1\u989d", "\u5143", "\u5df2\u63d0\u4ea4\u5145\u503c\u4fe1\u606f\uff0c\u8bf7\u5728\u5145\u503c\u8bb0\u5f55\u4e2d\u67e5\u770b\u5145\u503c\u7ed3\u679c\u3002", "\u8fd4\u56de", "\u72b6\u6001", "\u6700\u8fd1\u5145\u503c\u8bb0\u5f55", "\u63d0\u4ea4\u8fc7\u7a0b\u51fa\u73b0\u9519\u8bef\uff0c\u8bf7\u68c0\u67e5\u5145\u503c\u5185\u5bb9\u3002", "\uff1a", "\n", "\u662f\u5426\u786e\u5b9a\u9000\u51fa\u6e38\u620f\uff1f", "\u60a8\u5c1a\u672a\u767b\u9646\uff0c\u8bf7\u8f93\u5165\u60a8\u7684\u624b\u673a\u53f7\u548c\u5bc6\u7801\u8054\u7f51\u4ed8\u8d39", "\u5e8f\u5217\u53f7\u548c\u5bc6\u7801\u586b\u5199\u6709\u8bef", "\u77ed\u4fe1\u5145\u503c", "\u5df2\u53d1\u9001\u6210\u529f\u3002", "\u670d\u52a1\u5668\u54cd\u5e94\u9519\u8bef\uff01", "\u4f7f\u7528\u672c\u4ea7\u54c1\u5fc5\u987b\u5148\u767b\u5f55\uff0c\u4e0e\u96ea\u9ca4\u9c7c\u5176\u4ed6\u4ea7\u54c1\u4e2d\u6ce8\u518c\u7684\u624b\u673a\u53f7\uff0c\u5bc6\u7801\uff0c\u5e10\u53f7\u4e2d\u7684\u6e38\u620f\u5e01\u901a\u7528\u3002\u5982\u679c\u6ca1\u6709\u5e10\u53f7\u53ef\u4ee5\u6ce8\u518c\uff0c\u9001\u6e38\u620f\u5e01\u3002\u767b\u5f55\u540e\u4ed8\u8d39\u4fe1\u606f\u66f4\u5b89\u5168\uff0c\u907f\u514d\u6389\u5b58\u6863\u3002", "\u5145\u503c\u7684\u6e38\u620f\u5e01\u5728\u6240\u6709\u96ea\u9ca4\u9c7c\u76f8\u5173\u4ea7\u54c1\u4e2d\u901a\u7528\u3002", "\u8bf7\u8f93\u516511\u4f4d\u624b\u673a\u53f7", "\u8bf7\u8f93\u51656\u4f4d\u4ee5\u4e0a\u5bc6\u7801", "\u662f\u5426\u9a6c\u4e0a\u6ce8\u518c\uff1f\u9700\u8981\u786e\u8ba4\u60a8\u586b\u5199\u7684\u662f\u5f53\u524d\u624b\u673a\u53f7\uff0c\u5426\u5219\u4f1a\u5bfc\u81f4\u6ce8\u518c\u5931\u8d25\u3002\u8d44\u8d392\u5143\uff0c\u9700\u8981\u53d1\u90011\u6761\u77ed\u4fe1\uff0c2\u5143/\u6761\uff0c\u4e0d\u542b\u901a\u8baf\u8d39\u3002", "\u6ce8\u518c\u5931\u8d25\uff01\u5982\u679c\u8be5\u624b\u673a\u53f7\u662f\u60a8\u7684\u771f\u5b9e\u53f7\u7801\u3002\u5c06\u4f1a\u628a\u60a8\u7684\u5bc6\u7801\u6539\u4e3a\u5f53\u524d\u6ce8\u518c\u586b\u5199\u7684\u5bc6\u7801\uff0c\u8be5\u8fc7\u7a0b\u53ef\u80fd\u9700\u8981\u4e00\u6bb5\u65f6\u95f4\u3002", "\u5df2\u6ce8\u518c\u6210\u529f\u3002\u5f85\u9a8c\u8bc1\u6210\u529f\u540e\u5c31\u80fd\u7528\u8be5\u53f7\u7801\u5145\u503c\uff01\u5e76\u8d2d\u4e70\u6e38\u620f\u4e2d\u7684\u4ed8\u8d39\u5185\u5bb9\u3002\u5982\u679c\u518d\u6b21\u6ce8\u518c\u8fd8\u53ef\u4ee5\u53d1\u9001\u77ed\u4fe1\u4fee\u6539\u5bc6\u7801\u3002", "\u60a8\u5c1a\u672a\u9a8c\u8bc1\uff0c\u5982\u679c\u4e4b\u524d\u5df2\u53d1\u9001\u77ed\u4fe1\u9a8c\u8bc1\uff0c\u8bf7\u7a0d\u5019\u518d\u767b\u9646\u91cd\u8bd5\u3002", "\u5ba2\u670d\u7535\u8bdd\uff1a400 630 5518", "\u60a8\u7684\u624b\u673a\u65e0\u6cd5\u8fde\u63a5\u5230\u670d\u52a1\u5668\uff0c\u4e0d\u80fd\u8fdb\u884c\u6e38\u620f\u3002\u662f\u5426\u91cd\u65b0\u5c1d\u8bd5\u8054\u7f51\uff1f"};
        this.f_String_arr_17 = new String[]{"http://218.202.228.126:8880/nolander/server.php", "http://chatsrv0.ttutt.cn/dntk/server.php"};
        this.f_int_159 = 2048;
        this.f_byte_arr_47 = new byte[this.f_int_159];
        this.f_bool_31 = false;
        this.f_HttpConnection_00 = null;
        this.f_bool_33 = true;
        this.setFullScreenMode(true);
        a a2 = this;
        if (a2.gameRandom == null) {
            a2.gameRandom = new Random();
            a2.gameRandom.setSeed(System.currentTimeMillis());
        }
        this.f_bool_00 = true;
        this.gameMode = 0;
        this.m_000();
    }

    protected final void paint(Graphics object) {
        int n;
        int n2;
        int n3;
        int n4;
        int n5;
        this.f_DirectGraphics_00 = DirectUtils.getDirectGraphics((Graphics)object);
        this.f_Graphics_00 = object;
        this.f_Graphics_00.setFont(this.f_Font_00);
        switch (this.gameMode) {
            case 0: {
                if (this.f_int_04 < 2) {
                    this.f_Graphics_00.setColor(-1);
                    this.f_Graphics_00.fillRect(0, 0, 240, 320);
                    if (this.f_Image_00 == null) break;
                    this.f_Graphics_00.drawImage(this.f_Image_00, 240 - this.f_Image_00.getWidth() >> 1, 320 - this.f_Image_00.getHeight() >> 1, 0);
                    break;
                }
                this.m_144();
                break;
            }
            case 1: {
                object = this;
                n5 = 0;
                n5 = 320 - ((a)object).f_int_16 - ((a)object).f_Image_arr2_00[10][0].getHeight();
                if (n5 > 0) {
                    ((a)object).f_Graphics_00.setColor(6662375);
                    ((a)object).f_Graphics_00.fillRect(0, 0, 240, n5);
                }
                ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[10][0], 0, n5, 0);
                ((a)object).f_Graphics_00.setColor(3156024);
                ((a)object).f_Graphics_00.fillRect(0, 320 - ((a)object).f_int_16, 240, ((a)object).f_int_16);
                super.m_013(((a)object).f_Image_arr2_00[10][1], 86, 320 - ((a)object).f_int_16 + (((a)object).f_int_16 - 17 >> 1), 0, 17 * ((a)object).f_byte_arr_00[((a)object).f_int_10], 60, 17, 17);
                n5 = 320 - ((a)object).f_int_16 + (((a)object).f_int_16 - 13 >> 1);
                super.m_004(((a)object).f_Image_arr2_00[8][14], 60 - ((a)object).f_int_46, n5, 1);
                ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][14], 160 + ((a)object).f_int_46, n5, 0);
                this.f_Graphics_00.setClip(0, this.f_int_16, 240, 320 - (this.f_int_16 >> 1));
                this.m_130();
                this.f_Graphics_00.setClip(0, 0, 240, 320);
                break;
            }
            case 2: {
                object = this;
                ((a)object).f_Graphics_00.setColor(3156024);
                ((a)object).f_Graphics_00.fillRect(0, 0, 240, 320);
                ((a)object).f_int_147 = 148 * ((a)object).f_int_143 / 100;
                super.m_002(((a)object).f_Image_arr2_00[15][0], 108, 86, 0, 0, 24, ((a)object).f_int_147);
                super.m_002(((a)object).f_Image_arr2_00[15][1], 108, 86 + ((a)object).f_int_147, 0, ((a)object).f_int_147, 24, 148 - ((a)object).f_int_147);
                break;
            }
            case 11: {
                int n6;
                this.m_056(true);
                this.m_063(0, 20);
                this.m_053(this.f_int_56, this.f_int_57 + 20, true);
                this.m_126();
                this.m_035(0, 0);
                object = this;
                int n7 = 0;
                n4 = (((a)object).f_int_34 << 1) + 12;
                int n8 = 320 - n4;
                if (((a)object).f_String_02 == null || ((a)object).f_bool_05 || ((a)object).f_byte_16 == 4) {
                    super.m_037(0, ((a)object).f_int_48);
                    break;
                }
                Object var9_28 = null;
                super.m_039(0, n8, 80, n4);
                int n9 = n4;
                int n10 = 160;
                n3 = n8;
                n7 = 80;
                Object object2 = object;
                n7 = n3;
                for (n6 = 80; n6 < 214; n6 += 16) {
                    ((a)object2).f_Graphics_00.setClip(n6, n7, 16, 16);
                    ((a)object2).f_Graphics_00.drawImage(((a)object2).f_Image_arr2_00[8][0], n6 - 26, n7, 0);
                }
                ((a)object2).f_Graphics_00.setClip(0, 0, 240, 320);
                super.m_005(((a)object2).f_Image_arr2_00[8][0], 214, n7, 0, 0, 26, 16, 1);
                ((a)object2).f_Graphics_00.setColor(2699825);
                ((a)object2).f_Graphics_00.fillRect(80, n7 += 16, 149, n9 - 16);
                while (n7 < n3 + n9) {
                    ((a)object2).f_Graphics_00.setClip(229, n7, 11, 16);
                    ((a)object2).f_Graphics_00.drawImage(((a)object2).f_Image_arr2_00[8][0], 176, n7, 0);
                    n7 += 16;
                }
                ((a)object2).f_Graphics_00.setClip(0, 0, 240, 320);
                if (((a)object).f_byte_17 > 0) {
                    Image image = ((a)object).f_Image_arr_00[((a)object).f_byte_17];
                    if (((a)object).f_byte_17 != 72) {
                        n4 = image.getHeight();
                        super.m_002(image, 0 + (80 - ((a)object).f_byte_arr_07[((a)object).f_byte_17] >> 1), 320 - n4, 0, 0, ((a)object).f_byte_arr_07[((a)object).f_byte_17], n4);
                    } else {
                        super.m_002(image, 24, 275, 0, 0, 32, 45);
                    }
                } else if (((a)object).f_byte_17 == 0) {
                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[3][1], 40 - (((a)object).f_Image_arr2_00[3][1].getWidth() >> 1), 320 - ((a)object).f_Image_arr2_00[3][1].getHeight(), 0);
                } else {
                    ((a)object).f_Graphics_00.setColor(-1);
                    ((a)object).f_Graphics_00.drawString("?", 40, n8 + (n4 - ((a)object).f_int_01 >> 1), 17);
                }
                ((a)object).f_Graphics_00.setColor(10473684);
                super.m_019(((a)object).f_String_02, 85, n8 + 12, 129, n4 - 12, false);
                if (((a)object).f_int_26 > 0) {
                    super.m_002(((a)object).f_Image_arr2_00[8][15], 220, 320 - ((a)object).f_int_34 - 10, 0, 0, 7, 9);
                }
                if (((a)object).f_int_27 < ((a)object).f_int_28) {
                    super.m_002(((a)object).f_Image_arr2_00[8][15], 220, 320 - ((a)object).f_int_34 + 1, 7, 0, 7, 9);
                }
                if ((((a)object).f_byte_16 == 6 || ((a)object).f_byte_16 == 1) && (n4 = ((a)object).f_byte_arr_22[((a)object).f_int_112]) >= 0) {
                    if (((a)object).f_int_119 >= 0) {
                        n7 = ((a)object).entityPixelX[((a)object).f_int_119] + 16;
                        n8 = ((a)object).entityPixelY[((a)object).f_int_119] - 30;
                        if (((a)object).f_byte_17 == 84) {
                            n8 -= 20;
                        }
                    } else {
                        n7 = ((a)object).playerPixelX + 16;
                        n8 = ((a)object).playerPixelY - 30;
                    }
                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[12][n4], n7 + ((a)object).f_int_56, n8 + ((a)object).f_int_57 + 20, 0);
                }
                if (((a)object).f_byte_16 != 6) break;
                ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[2][11], ((a)object).f_int_56 + 5 + (((a)object).f_int_114 << 5), ((a)object).f_int_57 + (((a)object).f_int_115 << 5) - 12 + ((a)object).f_int_46, 0);
                break;
            }
            case 21: {
                object = this;
                ((a)object).f_Graphics_00.setColor(0);
                ((a)object).f_Graphics_00.fillRect(0, 0, 240, 320);
                ((a)object).f_Graphics_00.setColor(-1);
                ((a)object).f_Graphics_00.drawString("\u662f\u5426\u5f00\u542f\u58f0\u97f3\uff1f", 120, 160, 17);
                super.m_034();
                break;
            }
            case 3: {
                this.m_056(true);
                this.m_063(0, 20);
                this.m_053(this.f_int_56, this.f_int_57 + 20, true);
                n4 = 20;
                int n11 = 240 - (this.mapCellsWide + 1 << 2);
                object = this;
                int n12 = 0;
                int n13 = 0;
                if (((a)object).f_bool_arr_05[1] && ((a)object).f_Image_03 != null) {
                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_03, n11, 20, 0);
                    n12 = n11 + (((a)object).playerCellX << 2);
                    n13 = 20 + (((a)object).playerCellY << 2);
                    ((a)object).f_Graphics_00.setColor(1112072);
                    ((a)object).f_Graphics_00.fillRect(n12, n13, 4, 4);
                }
                this.m_037(0, this.f_int_48);
                this.m_035(0, 0);
                object = this;
                if (((a)object).f_bool_13) {
                    if (((a)object).f_int_70 >= 0) {
                        super.m_002(((a)object).f_Image_arr2_00[8][19], 104, 20, 0, 0, 16, 16);
                        super.m_002(((a)object).f_Image_arr2_00[8][19], 120, 20 + ((a)object).f_int_46, 16, 0, 15, 17);
                    }
                    if (((a)object).f_int_73 >= 0) {
                        super.m_002(((a)object).f_Image_arr2_00[8][19], 104, ((a)object).f_int_48 - 17, 0, 17, 16, 16);
                        super.m_002(((a)object).f_Image_arr2_00[8][19], 120, ((a)object).f_int_48 - 17 - ((a)object).f_int_46, 16, 17, 15, 17);
                    }
                }
                this.m_126();
                this.f_byte_13 = 1;
                this.f_byte_14 = (byte)3;
                this.m_034();
                break;
            }
            case 4: 
            case 19: {
                int n6;
                int n9;
                this.m_056(true);
                this.m_037(0, this.f_int_48);
                this.m_035(0, 0);
                n4 = 160;
                int n14 = 120;
                object = this;
                int n15 = ((a)object).f_int_08 + 22;
                int n16 = ((a)object).f_int_14 + 32;
                int n17 = 120 - (((a)object).f_int_08 + 22 >> 1);
                n2 = 160 - (((a)object).f_int_14 + 32 >> 1);
                n = ((a)object).f_int_15 - ((a)object).f_int_01 >> 1;
                boolean bl = false;
                super.m_040(n17, n2, n15, n16);
                n2 += 16 + n;
                int n18 = ((a)object).f_int_11;
                while (n18 < ((a)object).f_int_12) {
                    byte by = ((a)object).f_byte_arr_00[n18];
                    if (((a)object).f_int_10 == n18) {
                        ((a)object).f_Graphics_00.setColor(0);
                        ((a)object).f_Graphics_00.fillRect(120 - (((a)object).f_int_08 >> 1), n2 - n, ((a)object).f_int_08, ((a)object).f_int_15);
                        ((a)object).f_Graphics_00.setColor(16377897);
                    } else if (by != 11) {
                        ((a)object).f_Graphics_00.setColor(7574946);
                    } else {
                        ((a)object).f_Graphics_00.setColor(0xFF99FF);
                    }
                    if (by > 11 && by < 17) {
                        int n19 = 4;
                        n6 = n2;
                        n9 = 120;
                        int[] nArray = ((a)object).f_int_arr_32;
                        String string = ((a)object).f_String_arr_02[by];
                        Object object3 = object;
                        n4 = string.length();
                        int n20 = ((a)object3).f_byte_25;
                        int n21 = nArray.length;
                        char c = '\u0000';
                        n9 -= ((a)object3).f_Font_00.stringWidth(string) >> 1;
                        for (int i = 0; i < n4; ++i) {
                            c = string.charAt(i);
                            ((a)object3).f_Graphics_00.setColor(nArray[n20]);
                            ((a)object3).f_Graphics_00.drawChar(c, n9, n6, 0);
                            n9 += ((a)object3).f_Font_00.charWidth(c);
                            if (++n20 < n21) continue;
                            n20 = 0;
                        }
                        if ((((a)object3).f_int_03 & 3) == 0 && (((a)object3).f_byte_25 = (byte)(((a)object3).f_byte_25 + 1)) >= n21) {
                            ((a)object3).f_byte_25 = 0;
                        }
                    } else {
                        ((a)object).f_Graphics_00.drawString(((a)object).f_String_arr_02[by], 120, n2, 17);
                    }
                    ++n18;
                    n2 += ((a)object).f_int_15;
                }
                n3 = 120 + (n15 >> 1) - 23;
                n2 = 160 + (n16 >> 1) - 38;
                if (((a)object).f_int_11 > 0) {
                    super.m_002(((a)object).f_Image_arr2_00[8][15], n3, n2, 0, 0, 7, 9);
                }
                if (((a)object).f_int_12 < ((a)object).f_int_07) {
                    super.m_002(((a)object).f_Image_arr2_00[8][15], n3, n2 + 9, 7, 0, 7, 9);
                }
                this.m_034();
                break;
            }
            case 5: {
                this.m_056(false);
                object = this;
                int n22 = 240 - ((a)object).f_int_108 >> 1;
                n4 = 320 - ((a)object).f_int_107 >> 1;
                Image image = null;
                byte by = 0;
                n3 = 0;
                n2 = 0;
                n = 0;
                if (((a)object).f_int_110 - ((a)object).f_int_109 > 0) {
                    super.m_040(n22, n4, ((a)object).f_int_108, ((a)object).f_int_107);
                    if (((a)object).f_int_109 > 0) {
                        super.m_002(((a)object).f_Image_arr2_00[8][19], 112, n4 - 17 - (((a)object).f_int_03 & 1), 16, 0, 15, 17);
                    }
                    n4 += 16;
                    int n23 = ((a)object).f_int_109;
                    int n24 = n22 += 16;
                    while (n23 < ((a)object).f_int_110) {
                        by = ((a)object).f_byte_arr_20[n23];
                        image = ((a)object).f_Image_arr_00[((a)object).f_byte_arr_20[n23]];
                        n3 = ((a)object).f_int_arr_22[n23];
                        ((a)object).f_Graphics_00.setColor(13097429);
                        ((a)object).f_Graphics_00.drawString(((a)object).objectTypeNames[by], n22, n4 + 2, 0);
                        n22 = (240 + ((a)object).f_int_108 >> 1) - 75;
                        n = ((a)object).f_int_01 - 19 >> 1;
                        n4 += n;
                        if (n3 < 0) {
                            ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][17], n22 - 4, n4 + 2, 0);
                        } else {
                            ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][16], n22 - 12, n4 + 2, 0);
                            super.m_041((n22 += 15) - 1, n4 + 6, 48, 12);
                            super.m_042(((a)object).f_Image_arr2_00[8][2], n3, n22 + 45, n4 + 8);
                        }
                        n22 = n24;
                        super.m_041(n22, n4 += n + 19 + 4, 32, 32);
                        if (by > 40) {
                            n2 = 32 - ((a)object).f_byte_arr_07[by] >> 1;
                            n = image.getHeight();
                            if (n > 31) {
                                n = 31;
                            }
                            if (n2 < 0) {
                                super.m_002(image, n22, n4 + 31 - n, -n2, 0, 32, n);
                            } else {
                                super.m_002(image, n22 + n2, n4 + 31 - n, 0, 0, ((a)object).f_byte_arr_07[by], n);
                            }
                        } else {
                            ((a)object).f_Graphics_00.setClip(n22, n4, 32, 32);
                            ((a)object).f_Graphics_00.drawImage(image, n22 + (32 - image.getWidth() >> 1), n4 + (32 - image.getHeight() >> 1), 0);
                            ((a)object).f_Graphics_00.setClip(n22, n4, 240, 320);
                        }
                        super.m_002(((a)object).f_Image_arr2_00[8][7], n22 += 36, n4 += 4, 10, 0, 10, 13);
                        super.m_041(n22 += 15, n4, 48, 12);
                        super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).enemyAtkScaled[by - 41], n22 + 40, n4 + 2);
                        super.m_002(((a)object).f_Image_arr2_00[8][7], n22 += 52, n4, 20, 2, 10, 10);
                        ((a)object).f_Graphics_00.setColor(512);
                        super.m_041(n22 += 15, n4, 48, 12);
                        super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).enemyDefScaled[by - 41], n22 + 40, n4 + 2);
                        super.m_002(((a)object).f_Image_arr2_00[8][7], n22 -= 82, (n4 += 12) + 5, 0, 2, 10, 10);
                        super.m_041(n22 += 15, n4 + 5, 48, 12);
                        super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).enemyHpScaled[by - 41], n22 + 40, n4 + 7);
                        ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][1], n22 += 52, n4 + 5, 0);
                        super.m_041(n22 += 15, n4 + 5, 48, 12);
                        super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).enemyBaseGold[by - 41], n22 += 40, n4 + 7);
                        n4 += 18;
                        if (++n23 < ((a)object).f_int_110) {
                            n22 = n24 - 5;
                            ((a)object).f_Graphics_00.setColor(6435);
                            ((a)object).f_Graphics_00.drawLine(n22, n4, n22 + ((a)object).f_int_108 - 22, n4);
                            ((a)object).f_Graphics_00.setColor(4803902);
                            ((a)object).f_Graphics_00.drawLine(n22, ++n4, n22 + ((a)object).f_int_108 - 22, n4);
                        }
                        n4 += 2;
                        n22 = n24;
                    }
                    n4 += 16;
                    if (((a)object).f_int_110 < ((a)object).f_int_105) {
                        super.m_002(((a)object).f_Image_arr2_00[8][19], 112, n4 + (((a)object).f_int_03 & 1), 16, 17, 15, 17);
                    }
                }
                this.m_034();
                break;
            }
            case 7: {
                this.m_056(false);
                this.m_113(true);
                this.m_034();
                break;
            }
            case 8: {
                this.m_056(false);
                this.m_113(false);
                this.m_034();
                break;
            }
            case 9: {
                this.m_056(true);
                this.m_063(0, 20);
                this.m_053(this.f_int_56, this.f_int_57 + 20, true);
                this.m_037(0, this.f_int_48);
                this.m_035(0, 0);
                this.m_034();
                object = this;
                int n25 = 240 - ((a)object).f_int_75 - 22 >> 1;
                n4 = 320 - ((a)object).f_int_74 >> 1;
                int n26 = (((a)object).f_int_00 << 1) + 16 + 48;
                super.m_038(0, n25, n4, ((a)object).f_int_75 + 22, ((a)object).f_int_74);
                n25 = 240 - n26 >> 1;
                ((a)object).f_Graphics_00.setColor(7575203);
                ((a)object).f_Graphics_00.drawString("\u82b1\u8d39", n25, n4 += 21, 0);
                ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][1], n25 += (((a)object).f_int_00 << 1) + 4, n4 + (((a)object).f_int_01 - 10 >> 1), 0);
                super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).alchemyPrice, n25 += 60, n4 + 3 + (((a)object).f_int_01 - 10 >> 1));
                n25 = 240 - ((a)object).f_int_75 >> 1;
                ((a)object).f_Graphics_00.setColor(549016);
                ((a)object).f_Graphics_00.fillRect(n25, (n4 += ((a)object).f_int_01 + 5) + (((a)object).f_int_80 << 4), ((a)object).f_int_75, 16);
                ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][14], n25 + 10, n4 + 2 + (((a)object).f_int_80 << 4), 0);
                super.m_004(((a)object).f_Image_arr2_00[8][14], n25 + ((a)object).f_int_75 - 30, n4 + 2 + (((a)object).f_int_80 << 4), 1);
                super.m_002(((a)object).f_Image_arr2_00[8][7], 97, n4 + 4, 0, 2, 10, 10);
                super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).f_int_77, 142, n4 + 6);
                super.m_002(((a)object).f_Image_arr2_00[8][7], 97, (n4 += 16) + 4, 11, 0, 8, 13);
                super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).f_int_78, 142, n4 + 6);
                super.m_002(((a)object).f_Image_arr2_00[8][7], 97, (n4 += 16) + 4, 20, 2, 10, 11);
                super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).f_int_79, 142, n4 + 6);
                break;
            }
            case 10: {
                int n27 = 320;
                int n28 = 240;
                n4 = 0;
                int n29 = 0;
                object = this;
                for (n3 = 319; n3 >= 1; n3 -= 2) {
                    ((a)object).f_Graphics_00.drawLine(-1, -1 + (320 - n3), -1 + n3 - 1, 318);
                    ((a)object).f_Graphics_00.drawLine(239 - n3, -1, 239, -1 + n3);
                }
                object = this;
                n29 = 240 - ((a)object).f_int_103 >> 1;
                n4 = 320 - ((a)object).f_int_102 >> 1;
                n28 = 0;
                Image image = null;
                super.m_040(n29, n4, ((a)object).f_int_103, ((a)object).f_int_102);
                n29 += 20;
                n4 += 16;
                ((a)object).f_Graphics_00.setColor(13097429);
                if (((a)object).f_int_104 < ((a)object).itemStackSize) {
                    ((a)object).f_Graphics_00.drawString(((a)object).objectTypeNames[((a)object).itemStackTypes[((a)object).f_int_104]], n29, n4 + 2, 0);
                }
                ((a)object).f_Graphics_00.setColor(6178);
                ((a)object).f_Graphics_00.drawLine(n29 -= 9, n4 += ((a)object).f_int_01 + 4, n29 + ((a)object).f_int_103 - 23, n4);
                ((a)object).f_Graphics_00.setColor(3564144);
                ((a)object).f_Graphics_00.drawLine(n29, ++n4, n29 + ((a)object).f_int_103 - 23, n4);
                n4 += 10;
                n28 = n29 += 9;
                n3 = ((a)object).f_int_98;
                n2 = ((a)object).f_int_97 * ((a)object).f_int_98;
                while (n3 < ((a)object).f_int_99) {
                    n = 0;
                    while (n < ((a)object).f_int_97) {
                        super.m_041(n29, n4, 32, 32);
                        if (n2 == ((a)object).f_int_104) {
                            super.m_002(((a)object).f_Image_arr2_00[2][10], n29 - 4, n4 - 3, 38 * (((a)object).f_int_03 & 1), 0, 38, 38);
                        }
                        if (n2 < ((a)object).itemStackSize) {
                            image = ((a)object).f_Image_arr_00[((a)object).itemStackTypes[n2]];
                            ((a)object).f_Graphics_00.drawImage(image, n29 + (32 - image.getWidth() >> 1), n4 + (32 - image.getHeight() >> 1), 0);
                            if (((a)object).itemStackUses[n2] > 0) {
                                super.m_042(((a)object).f_Image_arr2_00[8][2], ((a)object).itemStackUses[n2], n29 + 32, n4 + 26);
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
                n29 = (240 + ((a)object).f_int_103 >> 1) - 22;
                n4 -= ((a)object).f_int_102 - 32 - ((a)object).f_int_01 - 4 >> 1;
                if (((a)object).f_int_98 > 0) {
                    super.m_002(((a)object).f_Image_arr2_00[8][15], n29, n4 - 15, 0, 0, 7, 9);
                }
                if (((a)object).f_int_99 < ((a)object).f_int_101) {
                    super.m_002(((a)object).f_Image_arr2_00[8][15], n29, n4 + 5, 7, 0, 7, 9);
                }
                this.m_034();
                break;
            }
            case 13: {
                this.m_056(false);
                break;
            }
            case 14: {
                object = this;
                int n30 = 0;
                if (((a)object).f_Image_01 != null) {
                    ((a)object).f_int_17 = ((a)object).f_Image_01.getHeight();
                    ((a)object).f_int_18 = 320 - ((a)object).f_int_17 - (((a)object).f_int_01 + 4) * 3 >> 1;
                    ((a)object).f_Graphics_00.setColor(0);
                    ((a)object).f_Graphics_00.fillRect(0, 0, 240, ((a)object).f_int_18);
                    n30 = ((a)object).f_int_18 + ((a)object).f_Image_01.getHeight();
                    ((a)object).f_Graphics_00.fillRect(0, n30, 240, 320 - n30);
                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_01, 120, ((a)object).f_int_18, 17);
                    if (!((a)object).f_bool_03) {
                        ((a)object).f_Graphics_00.setColor(-1);
                    } else {
                        ((a)object).f_Graphics_00.setColor(((a)object).f_int_arr_02[((a)object).f_byte_04]);
                    }
                    if (((a)object).f_int_19 < 3 && ((a)object).f_String_01 != null) {
                        super.m_019(((a)object).f_String_01, 10, ((a)object).f_int_18 + ((a)object).f_int_17 + ((a)object).f_int_01, 220, ((a)object).f_int_34 << 1, false);
                    }
                }
                if (((a)object).f_int_19 > 2) {
                    ((a)object).f_Graphics_00.setColor(-1);
                    ((a)object).f_Graphics_00.drawString("\u8bf7\u6309\u4efb\u610f\u952e", 120, 320 - ((a)object).f_int_01 - 2, 17);
                } else {
                    ((a)object).f_Graphics_00.setColor(-1);
                    ((a)object).f_Graphics_00.drawString("\u8df3\u8fc7", 240 - ((a)object).f_Font_00.stringWidth("\u8df3\u8fc7") - 5, 320 - ((a)object).f_int_01 - 2, 0);
                }
                this.f_Graphics_00.setClip(0, 0, 240, 320);
                this.m_130();
                this.f_Graphics_00.setClip(0, 0, 240, 320);
                break;
            }
            case 18: {
                object = this;
                ((a)object).f_Graphics_00.setColor(0);
                ((a)object).f_byte_21 = 0;
                ((a)object).f_short_arr_07 = new short[64];
                ((a)object).f_short_arr_08 = new short[64];
                int n31 = 0;
                n4 = 0;
                short s = 0;
                int n32 = ((a)object).f_byte_20;
                while (--n32 >= 0) {
                    n4 = ((a)object).f_short_arr_05[n32];
                    s = ((a)object).f_short_arr_06[n32];
                    ((a)object).f_Graphics_00.fillRect(n4, (int)s, 4, 4);
                    n31 = super.randomBelow(15);
                    if ((n31 & 1) != 0) {
                        super.m_124(n4, s - 4);
                    }
                    if ((n31 & 2) != 0) {
                        super.m_124(n4, s + 4);
                    }
                    if ((n31 & 4) != 0) {
                        super.m_124(n4 - 4, s);
                    }
                    if ((n31 & 8) == 0) continue;
                    super.m_124(n4 + 4, s);
                }
                ((a)object).f_byte_20 = ((a)object).f_byte_21;
                ((a)object).f_short_arr_05 = ((a)object).f_short_arr_07;
                ((a)object).f_short_arr_06 = ((a)object).f_short_arr_08;
                break;
            }
            case 12: {
                this.m_056(false);
                object = this;
                int n33 = 320 - ((a)object).f_int_01 - 4;
                super.m_142(((a)object).f_String_04, 120, n33 + 2, 17, ((a)object).f_int_arr_36);
                n33 += ((a)object).f_int_01 + 4 - 13 >> 1;
                if (((a)object).f_int_90 > 0) {
                    super.m_004(((a)object).f_Image_arr2_00[8][14], 60 - ((a)object).f_int_46, n33, 1);
                }
                if (((a)object).f_int_90 < ((a)object).f_int_89 - 1) {
                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][14], 160 + ((a)object).f_int_46, n33, 0);
                }
                this.m_034();
                break;
            }
            case 15: {
                this.m_056(false);
                this.m_013(this.f_Image_arr2_00[10][1], 86, 10, 0, 51, 68, 17, 17);
                this.m_034();
                break;
            }
            case 17: {
                this.m_056(false);
                this.m_013(this.f_Image_arr2_00[10][1], 86, 10, 0, 68, 68, 17, 17);
                this.m_034();
                break;
            }
            case 16: {
                this.m_056(false);
                object = this;
                int n34 = 240 - ((a)object).f_int_84 - 22 >> 1;
                n4 = 320 - ((a)object).f_int_85 >> 1;
                String string = null;
                super.m_013(((a)object).f_Image_arr2_00[10][1], 86, 10, 0, 34, 68, 17, 17);
                super.m_040(n34, n4, ((a)object).f_int_84 + 22, ((a)object).f_int_85);
                int n35 = (((a)object).f_int_01 + 8) * ((a)object).f_int_82;
                ((a)object).f_Graphics_00.setColor(549016);
                ((a)object).f_Graphics_00.fillRect(n34 += 11, (n4 += 16) + n35, ((a)object).f_int_84, ((a)object).f_int_01 + 8);
                ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[8][14], n34 + 10, n4 + 2 + n35 + (((a)object).f_int_01 - 5 >> 1), 0);
                super.m_004(((a)object).f_Image_arr2_00[8][14], n34 + ((a)object).f_int_84 - 30, n4 + 2 + n35 + (((a)object).f_int_01 - 5 >> 1), 1);
                n3 = 0;
                while (n3 < ((a)object).f_int_83) {
                    string = ((a)object).f_String_arr_05[((a)object).f_byte_arr_11[n3]];
                    n34 = 240 - ((a)object).f_Font_00.stringWidth(string) - 16 >> 1;
                    ((a)object).f_Graphics_00.setColor(-1);
                    ((a)object).f_Graphics_00.drawString(string, n34, n4 + 4, 0);
                    n34 += 2 + ((a)object).f_Font_00.stringWidth(string);
                    if (!((a)object).f_bool_arr_05[n3]) {
                        super.m_002(((a)object).f_Image_arr2_00[8][11], n34, n4 + (((a)object).f_int_01 - 2 >> 1), 12, 0, 12, 10);
                    } else {
                        super.m_002(((a)object).f_Image_arr2_00[8][11], n34, n4 + (((a)object).f_int_01 - 2 >> 1), 0, 0, 12, 10);
                    }
                    ++n3;
                    n4 += ((a)object).f_int_01 + 8;
                }
                this.m_034();
                break;
            }
            case 20: {
                object = this;
                int n36 = 320 - ((a)object).f_Image_arr2_00[14][0].getHeight() >> 1;
                ((a)object).f_int_23 += 2;
                if (!((a)object).f_bool_04 && (((a)object).f_int_03 & 1) != 0 && ++((a)object).f_int_22 > ((a)object).f_Image_arr2_00[14][0].getWidth() - 240) {
                    ((a)object).f_bool_04 = true;
                }
                if ((((a)object).f_int_03 & 3) >> 1 != 0 && ((a)object).f_int_33 < ((a)object).dialogueTexts[260].length()) {
                    ++((a)object).f_int_33;
                    super.m_018(((a)object).dialogueTexts[260], 209, (((a)object).f_int_34 << 1) + 12, ((a)object).f_int_33);
                    ((a)object).f_int_26 = ((a)object).f_int_28 - 1;
                    if (((a)object).f_int_26 < 0) {
                        ((a)object).f_int_26 = 0;
                    }
                    ((a)object).f_int_27 = ((a)object).f_int_28;
                }
                ((a)object).f_Graphics_00.setColor(3156024);
                ((a)object).f_Graphics_00.fillRect(0, 0, 240, n36);
                ((a)object).f_Graphics_00.fillRect(0, 320 - n36, 240, n36);
                ((a)object).f_Graphics_00.setClip(0, 0, 240, 320);
                ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[14][0], -((a)object).f_int_22, n36, 0);
                ((a)object).f_Graphics_00.setColor(-1);
                super.m_019(((a)object).dialogueTexts[260], 15, 320 - ((a)object).f_int_01 - 10, 210, ((a)object).f_int_01 + 10, false);
                break;
            }
            case 22: {
                this.m_144();
                break;
            }
            case 99: {
                object = this;
                int n37 = ((a)object).f_int_165 & 7;
                n4 = 20;
                n3 = 0;
                ((a)object).f_Graphics_00.setColor(0xF8F8F8);
                ((a)object).f_Graphics_00.fillRect(18, 138, 204, 44);
                ((a)object).f_Graphics_00.setColor(0);
                ((a)object).f_Graphics_00.drawRect(20, 140, 199, 39);
                ((a)object).f_Graphics_00.drawRect(18, 138, 203, 43);
                if (!((a)object).f_bool_32) {
                    if (n37 == 7) {
                        ((a)object).f_bool_32 = true;
                    }
                } else {
                    if (n37 == 7) {
                        ((a)object).f_bool_32 = false;
                    }
                    n37 = 7 - n37;
                }
                n2 = 0;
                while (n2 < 8) {
                    n3 = !((a)object).f_bool_32 ? n37 - n2 : n2 - n37;
                    switch (n3) {
                        case 0: {
                            ((a)object).f_Graphics_00.setColor(44527);
                            break;
                        }
                        case 1: {
                            ((a)object).f_Graphics_00.setColor(0x66CCFF);
                            break;
                        }
                        case 2: {
                            ((a)object).f_Graphics_00.setColor(11593215);
                            break;
                        }
                        default: {
                            ((a)object).f_Graphics_00.setColor(0xD9D9D9);
                        }
                    }
                    ((a)object).f_Graphics_00.fillRect(n4 + 2, 142, 21, 6);
                    ((a)object).f_Graphics_00.fillRect(215 - n4 + 2, 172, 21, 6);
                    ++n2;
                    n4 += 25;
                }
                ((a)object).f_Graphics_00.setColor(44527);
                ((a)object).f_Graphics_00.drawString(((a)object).f_String_07, 120, 320 - ((a)object).f_int_01 >> 1, 17);
            }
        }
        if (this.f_bool_16) {
            object = this;
            int n38 = 0;
            int n39 = 0;
            n3 = 0;
            if (((a)object).f_bool_17) {
                if (++((a)object).f_int_142 > 4) {
                    ((a)object).f_int_142 = 4;
                    ((a)object).f_bool_17 = false;
                    ((a)object).currentFloor = ((a)object).f_byte_23;
                    super.m_121(((a)object).currentFloor);
                    if (((a)object).currentFloor == 0) {
                        super.m_024(1, 2);
                        super.m_064((((a)object).f_int_58 - 32 >> 1) - ((a)object).playerPixelX, (((a)object).f_int_59 - 32 >> 1) - ((a)object).playerPixelY);
                    } else if (((a)object).currentFloor == 50) {
                        super.m_024(6, 7);
                        super.m_064((((a)object).f_int_58 - 32 >> 1) - ((a)object).playerPixelX, (((a)object).f_int_59 - 32 >> 1) - ((a)object).playerPixelY);
                    } else if (((a)object).currentFloor == 1 && !((a)object).f_bool_18) {
                        super.m_024(6, 11);
                        super.m_064((((a)object).f_int_58 - 32 >> 1) - ((a)object).playerPixelX, (((a)object).f_int_59 - 32 >> 1) - ((a)object).playerPixelY);
                    } else {
                        n2 = super.m_065(((a)object).f_bool_18);
                        if (n2 >= 0) {
                            super.m_031(((a)object).entityPixelX[n2] >> 5, ((a)object).entityPixelY[n2] >> 5);
                        }
                    }
                    super.m_057();
                    super.applyStepCellEffects();
                }
            } else if (--((a)object).f_int_142 <= 0) {
                ((a)object).f_bool_17 = true;
                ((a)object).f_bool_16 = false;
            }
            n39 = n38 = 4 - ((a)object).f_int_142;
            n3 = ((a)object).f_int_142 << 1;
            ((a)object).f_Graphics_00.setColor(0);
            n2 = 0;
            while (n2 < 40) {
                n = 0;
                while (n < 30) {
                    ((a)object).f_Graphics_00.fillRect(n38, n39, n3, n3);
                    ++n;
                    n38 += 8;
                }
                ++n2;
                n38 = 4 - ((a)object).f_int_142;
                n39 += 8;
            }
            return;
        }
        if (this.f_bool_05) {
            n4 = 320 - this.f_int_25 >> 1;
            n5 = 240 - this.f_int_24 >> 1;
            object = this;
            int n40 = n5;
            int n41 = n4;
            super.m_040(n5, n4, ((a)object).f_int_24, ((a)object).f_int_25);
            switch (((a)object).f_byte_08) {
                case 0: 
                case 2: 
                case 4: 
                case 5: {
                    ((a)object).f_Graphics_00.setColor(-1);
                    super.m_019(((a)object).f_String_02, n40 += 16, n41 += 16, 180, 240, true);
                    break;
                }
                case 1: 
                case 3: {
                    super.m_041(n40 += 16, n41 += 16, 32, 32);
                    Image image = ((a)object).f_Image_arr_00[((a)object).f_byte_07];
                    if (image != null) {
                        ((a)object).f_Graphics_00.drawImage(image, n40 + (32 - image.getWidth() >> 1), n41 + (32 - image.getHeight() >> 1), 0);
                    }
                    ((a)object).f_Graphics_00.setColor(16770173);
                    ((a)object).f_Graphics_00.drawString(((a)object).objectTypeNames[((a)object).f_byte_07], n40 += 42, n41 + (32 - ((a)object).f_int_01 >> 1), 0);
                    ((a)object).f_Graphics_00.setColor(-1);
                    n40 = n5 + 16;
                    super.m_019(((a)object).f_String_02, n40, n41 += 36, 180, ((a)object).f_int_25 - 32 - 36, true);
                }
            }
            n41 = n4 + ((a)object).f_int_25 - 12 - 18;
            if (((a)object).f_byte_05 != 0) {
                n40 = n5 + 13;
                super.m_002(((a)object).f_Image_arr2_00[8][11], n40, n41 + 6, (((a)object).f_byte_05 - 1) * 12, 0, 12, 10);
            }
            if (((a)object).f_byte_06 != 0) {
                n40 = n5 + ((a)object).f_int_24 - 25;
                super.m_002(((a)object).f_Image_arr2_00[8][11], n40, n41 + 6, (((a)object).f_byte_06 - 1) * 12, 0, 12, 10);
            }
        }
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     * Unable to fully structure code
     */
    public final void run() {
        if (this.f_bool_31) {
            var5_1 = this;
            while ((var6_4 = var5_1.m_151()) == 1) {
                Thread.yield();
            }
            var5_1.f_String_06 = null;
            var1_6 = var6_4;
            var2_8 = this.f_Object_00;
            synchronized (var2_8) {
                this.f_int_161 = var1_6;
                return;
            }
        }
        var1_7 = 0L;
        var3_10 = 0L;
        var5_2 = 0L;
        try {
            while (this.f_bool_00) {
                block274: {
                    block273: {
                        var1_7 = System.currentTimeMillis();
                        var3_10 = var1_7 + (long)this.f_int_02;
                        var5_3 = this;
                        if (!var5_3.f_bool_05) break block273;
                        block4 : switch (var5_3.f_byte_08) {
                            case 0: {
                                if (var5_3.gameMode == 15 || var5_3.gameMode == 17) {
                                    var6_5 = var5_3;
                                    switch (var6_5.f_int_05) {
                                        case -1: {
                                            super.m_020(var6_5.f_int_26 - var6_5.f_int_30);
                                            break;
                                        }
                                        case -2: {
                                            super.m_020(var6_5.f_int_26 + var6_5.f_int_30);
                                            break;
                                        }
                                        case -7: {
                                            var6_5.f_bool_05 = false;
                                            var6_5.gameMode = var6_5.f_byte_01;
                                        }
                                    }
                                    break;
                                }
                                super.m_016();
                                break;
                            }
                            case 4: {
                                switch (var5_3.f_int_05) {
                                    case -6: 
                                    case -5: {
                                        var5_3.f_bool_05 = super.applyMerchantOffer(true);
                                        break block4;
                                    }
                                    case -7: {
                                        var5_3.f_bool_05 = super.applyMerchantOffer(false);
                                        break block4;
                                    }
                                }
                                super.m_016();
                                break;
                            }
                            case 1: {
                                if (var5_3.f_byte_06 == 0 && var5_3.f_byte_05 == 0) {
                                    super.m_016();
                                    break;
                                }
                                block21 : switch (var5_3.f_byte_06) {
                                    case 2: 
                                    case 3: {
                                        switch (var5_3.f_int_05) {
                                            case 0: {
                                                break block21;
                                            }
                                            case -2: 
                                            case -1: {
                                                super.m_016();
                                                break block21;
                                            }
                                        }
                                        var5_3.f_bool_05 = false;
                                    }
                                }
                                break;
                            }
                            case 3: {
                                switch (var5_3.f_int_05) {
                                    case -6: {
                                        var5_3.f_bool_05 = false;
                                        super.useItemStack(var5_3.f_int_104);
                                        break block4;
                                    }
                                    case -7: {
                                        var5_3.gameMode = (byte)10;
                                        var5_3.f_bool_05 = false;
                                        break block4;
                                    }
                                }
                                super.m_016();
                                break;
                            }
                            case 2: {
                                break;
                            }
                            case 5: {
                                var6_5 = var5_3;
                                switch (var6_5.f_int_05) {
                                    case 0: {
                                        break block4;
                                    }
                                    case -1: {
                                        super.m_020(var6_5.f_int_26 - 1);
                                        break block4;
                                    }
                                    case -2: {
                                        super.m_020(var6_5.f_int_26 + 1);
                                        break block4;
                                    }
                                    case -3: {
                                        super.m_076(var6_5.f_int_90 - 1);
                                        break block4;
                                    }
                                    case -4: {
                                        super.m_076(var6_5.f_int_90 + 1);
                                        break block4;
                                    }
                                    case -7: 
                                    case -6: 
                                    case -5: {
                                        var6_5.f_bool_05 = false;
                                        var6_5.gameMode = (byte)10;
                                    }
                                }
                            }
                        }
                        var5_3.f_int_06 = 0;
                        var5_3.f_int_05 = 0;
                        break block274;
                    }
                    switch (var5_3.gameMode) {
                        case 0: {
                            var5_3.f_int_02 = 100;
                            if (var5_3.f_int_04 < 2) {
                                if (var5_3.f_int_156 == 0) {
                                    try {
                                        var5_3.f_Image_00 = Image.createImage((String)("/l" + var5_3.f_int_04 + ".png"));
                                    }
                                    catch (Exception v0) {}
                                    if (var5_3.f_Image_00 == null) {
                                        var5_3.f_int_156 = 0;
                                        ++var5_3.f_int_04;
                                        break;
                                    }
                                    var5_3.f_int_156 = 1;
                                    break;
                                }
                                if (++var5_3.f_int_156 <= 15) break;
                                ++var5_3.f_int_04;
                                var5_3.f_int_156 = 0;
                                break;
                            }
                            if (++var5_3.f_int_156 <= 35) {
                                super.m_143(0, var5_3.f_int_156);
                                break;
                            }
                            super.m_147(-1);
                            var5_3.f_bool_30 = false;
                            var5_3.f_int_156 = 0;
                            var5_3.f_Image_00 = null;
                            var5_3.f_Image_arr2_00[0] = null;
                            super.m_140();
                            var5_3.f_int_02 = 75;
                            var5_3.gameMode = (byte)21;
                            super.m_000();
                            break;
                        }
                        case 21: {
                            var6_5 = var5_3;
                            switch (var6_5.f_int_05) {
                                case -6: {
                                    var6_5.f_bool_29 = true;
                                    if (var6_5.f_int_155 == 0) {
                                        var6_5.f_int_155 = 60;
                                    }
                                    var6_5.gameMode = 1;
                                    super.m_000();
                                    break;
                                }
                                case -7: {
                                    var6_5.f_bool_29 = false;
                                    var6_5.f_int_155 = 0;
                                    var6_5.gameMode = 1;
                                    super.m_000();
                                }
                            }
                            break;
                        }
                        case 1: {
                            if ((var5_3.f_int_03 & 3) == 0) {
                                super.m_129(super.randomBelow(240), 320 - var5_3.f_int_16 - super.randomBelow(150), 2, -1);
                            }
                            super.m_052();
                            var6_5 = var5_3;
                            block66 : switch (var6_5.f_int_05) {
                                case -3: 
                                case -1: 
                                case 50: 
                                case 52: {
                                    if (var6_5.f_int_10 > 0) {
                                        --var6_5.f_int_10;
                                        break;
                                    }
                                    var6_5.f_int_10 = var6_5.f_int_07 - 1;
                                    break;
                                }
                                case -4: 
                                case -2: 
                                case 54: 
                                case 56: {
                                    if (var6_5.f_int_10 < var6_5.f_int_07 - 1) {
                                        ++var6_5.f_int_10;
                                        break;
                                    }
                                    var6_5.f_int_10 = 0;
                                    break;
                                }
                                case -5: 
                                case 53: {
                                    var7_11 = var6_5;
                                    switch (var7_11.f_byte_arr_00[var7_11.f_int_10]) {
                                        case 0: {
                                            var7_11.f_Image_arr2_00[10][0] = null;
                                            var7_11.gameMode = (byte)14;
                                            super.m_000();
                                            break block66;
                                        }
                                        case 1: {
                                            var7_11.gameMode = (byte)8;
                                            var7_11.f_byte_01 = 1;
                                            super.m_000();
                                            break block66;
                                        }
                                        case 2: {
                                            var7_11.gameMode = (byte)16;
                                            var7_11.f_byte_01 = 1;
                                            super.m_000();
                                            break block66;
                                        }
                                        case 3: {
                                            var7_11.gameMode = (byte)15;
                                            var7_11.f_byte_01 = 1;
                                            super.m_000();
                                            break block66;
                                        }
                                        case 4: {
                                            var7_11.gameMode = (byte)17;
                                            var7_11.f_byte_01 = 1;
                                            super.m_000();
                                            break block66;
                                        }
                                        case 5: {
                                            var7_11.gameMode = (byte)22;
                                            super.m_000();
                                        }
                                    }
                                }
                            }
                            break;
                        }
                        case 2: {
                            var6_5 = var5_3;
                            if (var6_5.f_int_143 < 100) {
                                if (var6_5.f_int_143 < var6_5.f_int_146) {
                                    var6_5.f_int_143 += 4;
                                    break;
                                }
                                var8_17 = var6_5.f_byte_arr_41[var6_5.f_int_145];
                                var7_12 = var6_5;
                                switch (var8_17) {
                                    case 0: {
                                        super.m_001(8);
                                        break;
                                    }
                                    case 1: {
                                        super.m_001(10);
                                        break;
                                    }
                                    case 2: {
                                        super.m_001(2);
                                        break;
                                    }
                                    case 3: {
                                        super.m_001(1);
                                        break;
                                    }
                                    case 4: {
                                        super.m_001(3);
                                        break;
                                    }
                                    case 5: {
                                        super.m_001(6);
                                        super.m_001(4);
                                        super.m_001(5);
                                        break;
                                    }
                                    case 13: {
                                        super.m_001(7);
                                        super.m_001(13);
                                        super.m_043();
                                        break;
                                    }
                                    case 6: {
                                        super.m_001(12);
                                        break;
                                    }
                                    case 7: {
                                        super.m_128();
                                        break;
                                    }
                                    case 8: {
                                        var7_12.minFloorReached = 51;
                                        v1.currentFloor = 51;
                                        var7_12.maxFloorReached = 51;
                                        var7_12.playerHp = 300;
                                        var7_12.playerAtk = 10;
                                        var7_12.playerDef = 10;
                                        var7_12.redKeyCount = 0;
                                        var7_12.blueKeyCount = 0;
                                        var7_12.yellowKeyCount = 0;
                                        var7_12.f_bool_21 = false;
                                        var7_12.f_bool_22 = false;
                                        var7_12.f_bool_20 = false;
                                        break;
                                    }
                                    case 12: {
                                        super.m_068();
                                        break;
                                    }
                                    case 10: {
                                        super.m_121(var7_12.currentFloor);
                                        super.m_057();
                                        break;
                                    }
                                    case 9: {
                                        super.m_118();
                                        break;
                                    }
                                    case 11: {
                                        super.m_001(3);
                                        var7_12.f_Image_02 = var7_12.f_Image_arr2_00[3][0];
                                        var7_12.f_int_arr_03 = var7_12.f_int_arr2_01[0];
                                        var7_12.f_int_38 = 0;
                                        var7_12.f_byte_12 = 0;
                                        super.m_064((var7_12.f_int_58 - 32 >> 1) - var7_12.playerPixelX, (var7_12.f_int_59 - 32 >> 1) - var7_12.playerPixelY);
                                        break;
                                    }
                                    case 14: {
                                        super.m_001(11);
                                        break;
                                    }
                                    case 15: {
                                        super.m_001(14);
                                        break;
                                    }
                                    case 16: {
                                        if (super.m_115(var7_12.f_int_134)) break;
                                        var7_12.f_int_144 = 0;
                                        var7_12.gameMode = (byte)8;
                                        super.m_000();
                                    }
                                }
                                ++var6_5.f_int_145;
                                var6_5.f_int_146 = var6_5.f_int_145 * 100 / var6_5.f_int_144;
                                if (var6_5.f_int_146 > var6_5.f_int_143) break;
                                var6_5.f_int_146 = var6_5.f_int_143 + 1;
                                break;
                            }
                            var6_5.f_int_143 = 0;
                            var6_5.f_int_146 = 0;
                            var6_5.f_Image_arr2_00[15] = null;
                            var6_5.gameMode = var6_5.f_byte_24;
                            if (!var6_5.f_bool_19) break;
                            super.m_000();
                            break;
                        }
                        case 3: {
                            var6_5 = var5_3;
                            switch (var6_5.f_int_05) {
                                case -6: {
                                    var6_5.gameMode = (byte)10;
                                    super.m_000();
                                    break;
                                }
                                case -7: {
                                    var6_5.f_byte_02 = var6_5.gameMode;
                                    var6_5.gameMode = (byte)4;
                                    super.m_000();
                                }
                            }
                            var6_5 = var5_3;
                            block102 : switch (var6_5.f_byte_11) {
                                case 5: {
                                    super.tickBattle(true);
                                    break;
                                }
                                case 0: {
                                    if (var6_5.f_bool_27) {
                                        super.m_032();
                                        break;
                                    }
                                    super.m_027();
                                    break;
                                }
                                case 1: {
                                    switch (var6_5.f_byte_12) {
                                        case 1: {
                                            var6_5.playerPixelY -= 8;
                                            if (var6_5.playerPixelY + var6_5.f_int_57 + 16 >= 106) break;
                                            super.m_064(var6_5.f_int_56, var6_5.f_int_57 + 8);
                                            break;
                                        }
                                        case 0: {
                                            var6_5.playerPixelY += 8;
                                            if (var6_5.playerPixelY + var6_5.f_int_57 + 16 <= var6_5.f_int_59 - 106) break;
                                            super.m_064(var6_5.f_int_56, var6_5.f_int_57 - 8);
                                            break;
                                        }
                                        case 3: {
                                            var6_5.playerPixelX -= 8;
                                            if (var6_5.playerPixelX + var6_5.f_int_56 + 16 >= 106) break;
                                            super.m_064(var6_5.f_int_56 + 8, var6_5.f_int_57);
                                            break;
                                        }
                                        case 2: {
                                            var6_5.playerPixelX += 8;
                                            if (var6_5.playerPixelX + var6_5.f_int_56 + 16 <= var6_5.f_int_58 - 106) break;
                                            super.m_064(var6_5.f_int_56 - 8, var6_5.f_int_57);
                                        }
                                    }
                                    var6_5.f_int_43 += 8;
                                    super.m_025();
                                    if (var6_5.f_int_43 < 32) break;
                                    var6_5.f_int_43 = 0;
                                    var6_5.f_int_38 = 0;
                                    super.applyStepCellEffects();
                                    break;
                                }
                                case 2: {
                                    super.m_026();
                                    if (!var6_5.f_bool_06) {
                                        switch (var6_5.f_int_06) {
                                            case -1: 
                                            case 50: {
                                                var6_5.f_byte_12 = 1;
                                                var6_5.f_int_57 += 16;
                                                super.m_064(var6_5.f_int_56, var6_5.f_int_57);
                                                break;
                                            }
                                            case -2: 
                                            case 56: {
                                                var6_5.f_byte_12 = 0;
                                                var6_5.f_int_57 -= 16;
                                                super.m_064(var6_5.f_int_56, var6_5.f_int_57);
                                                break;
                                            }
                                            case -3: 
                                            case 52: {
                                                var6_5.f_byte_12 = (byte)3;
                                                var6_5.f_int_56 += 16;
                                                super.m_064(var6_5.f_int_56, var6_5.f_int_57);
                                                break;
                                            }
                                            case -4: 
                                            case 54: {
                                                var6_5.f_byte_12 = (byte)2;
                                                var6_5.f_int_56 -= 16;
                                                super.m_064(var6_5.f_int_56, var6_5.f_int_57);
                                            }
                                        }
                                        switch (var6_5.f_int_05) {
                                            case -5: 
                                            case 53: {
                                                var6_5.f_bool_06 = true;
                                                super.m_104(0);
                                            }
                                        }
                                        break;
                                    }
                                    super.m_105();
                                    break;
                                }
                                case 3: {
                                    var7_13 = var6_5;
                                    switch (var7_13.f_int_05) {
                                        case -3: {
                                            if (var7_13.itemStackSize > 0) {
                                                if (var7_13.f_int_96 > 0) {
                                                    var7_13.f_int_96 = 0;
                                                    break block102;
                                                }
                                                var7_13.f_int_96 = var7_13.itemStackSize - 1;
                                                break block102;
                                            }
                                            ** GOTO lbl408
                                        }
                                        case -4: {
                                            if (var7_13.itemStackSize > 0) {
                                                if (var7_13.f_int_96 < var7_13.itemStackSize - 1) {
                                                    ++var7_13.f_int_96;
                                                    break block102;
                                                }
                                                var7_13.f_int_96 = 0;
                                                break block102;
                                            }
                                            ** GOTO lbl408
                                        }
                                        case -6: 
                                        case -5: {
                                            super.useItemStack(var7_13.f_int_96);
                                        }
                                        case -7: {
                                            var7_13.f_byte_11 = 0;
                                        }
                                    }
                                }
                            }
lbl408:
                            // 16 sources

                            super.m_055();
                            break;
                        }
                        case 4: {
                            var6_5 = var5_3;
                            switch (var6_5.f_int_05) {
                                case -1: {
                                    if (var6_5.f_int_10 > 0) {
                                        --var6_5.f_int_10;
                                        if (var6_5.f_int_10 >= var6_5.f_int_11) break;
                                        --var6_5.f_int_11;
                                        --var6_5.f_int_12;
                                        break;
                                    }
                                    var6_5.f_int_10 = var6_5.f_int_07 - 1;
                                    var6_5.f_int_12 = var6_5.f_int_07;
                                    var6_5.f_int_11 = var6_5.f_int_12 - var6_5.f_int_13;
                                    if (var6_5.f_int_11 >= 0) break;
                                    var6_5.f_int_11 = 0;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.f_int_10 < var6_5.f_int_07 - 1) {
                                        ++var6_5.f_int_10;
                                        if (var6_5.f_int_10 < var6_5.f_int_12) break;
                                        ++var6_5.f_int_12;
                                        ++var6_5.f_int_11;
                                        break;
                                    }
                                    var6_5.f_int_10 = 0;
                                    var6_5.f_int_11 = 0;
                                    var6_5.f_int_12 = var6_5.f_int_07;
                                    if (var6_5.f_int_07 <= var6_5.f_int_13) break;
                                    var6_5.f_int_12 = var6_5.f_int_13;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    super.m_012();
                                    break;
                                }
                                case -7: {
                                    var6_5.gameMode = var6_5.f_byte_02;
                                }
                            }
                            break;
                        }
                        case 5: {
                            var6_5 = var5_3;
                            switch (var6_5.f_int_05) {
                                case -1: {
                                    super.m_092(var6_5.f_int_109 - 1);
                                    break;
                                }
                                case -2: {
                                    super.m_092(var6_5.f_int_109 + 1);
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
                                    var6_5.gameMode = (byte)3;
                                }
                            }
                            break;
                        }
                        case 7: 
                        case 8: {
                            var6_5 = var5_3;
                            switch (var6_5.f_int_05) {
                                case -1: {
                                    if (var6_5.f_int_134 > 0) {
                                        if (--var6_5.f_int_134 >= var6_5.f_int_132) break;
                                        --var6_5.f_int_132;
                                        --var6_5.f_int_133;
                                        break;
                                    }
                                    var6_5.f_int_134 = 5;
                                    var6_5.f_int_133 = 6;
                                    var6_5.f_int_132 = 6 - var6_5.f_int_136;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.f_int_134 < 5) {
                                        if (++var6_5.f_int_134 < var6_5.f_int_133) break;
                                        ++var6_5.f_int_133;
                                        ++var6_5.f_int_132;
                                        break;
                                    }
                                    var6_5.f_int_134 = 0;
                                    var6_5.f_int_132 = 0;
                                    var6_5.f_int_133 = var6_5.f_int_136;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    if (var6_5.gameMode == 7) {
                                        super.m_119(var6_5.currentFloor);
                                        super.m_114(var6_5.f_int_134);
                                        super.m_015((byte)0, "\u4fdd\u5b58\u6210\u529f\uff01", (byte)0, (byte)0);
                                        break;
                                    }
                                    if (var6_5.gameMode != 8 || !var6_5.f_bool_arr_07[var6_5.f_int_134]) break;
                                    var6_5.f_int_06 = 0;
                                    var6_5.f_int_05 = 0;
                                    var6_5.f_Image_arr2_00[10][0] = null;
                                    super.m_131((byte)3, true);
                                    super.m_132(6);
                                    super.m_132(8);
                                    super.m_132(5);
                                    super.m_132(13);
                                    super.m_132(9);
                                    super.m_132(16);
                                    super.m_132(10);
                                    super.m_132(2);
                                    super.m_132(3);
                                    super.m_132(11);
                                    break;
                                }
                                case -7: {
                                    var6_5.gameMode = var6_5.f_byte_01;
                                }
                            }
                            break;
                        }
                        case 9: {
                            var6_5 = var5_3;
                            switch (var6_5.f_int_05) {
                                case -1: {
                                    if (var6_5.f_int_80 > 0) {
                                        --var6_5.f_int_80;
                                        break;
                                    }
                                    var6_5.f_int_80 = 2;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.f_int_80 < 2) {
                                        ++var6_5.f_int_80;
                                        break;
                                    }
                                    var6_5.f_int_80 = 0;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    if (var6_5.goldAmount >= var6_5.alchemyPrice) {
                                        var6_5.goldAmount -= var6_5.alchemyPrice;
                                        var7_14 = var6_5;
                                        switch (var7_14.f_int_80) {
                                            case 0: {
                                                var7_14.playerHp += var7_14.f_int_77;
                                                break;
                                            }
                                            case 1: {
                                                var7_14.playerAtk += var7_14.f_int_78;
                                                break;
                                            }
                                            case 2: {
                                                var7_14.playerDef += var7_14.f_int_79;
                                            }
                                        }
                                        ++var7_14.alchemyUpgradeCount;
                                        var7_14.alchemyPrice = a.alchemyPriceFor(var7_14.alchemyUpgradeCount + 1);
                                        break;
                                    }
                                    super.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                    break;
                                }
                                case -7: {
                                    var6_5.gameMode = (byte)3;
                                }
                            }
                            break;
                        }
                        case 10: {
                            var6_5 = var5_3;
                            block160 : switch (var6_5.f_int_05) {
                                case -1: {
                                    if (var6_5.f_int_104 >= var6_5.f_int_97) {
                                        var6_5.f_int_104 -= var6_5.f_int_97;
                                        if (var6_5.f_int_104 / var6_5.f_int_97 >= var6_5.f_int_98) break;
                                        --var6_5.f_int_98;
                                        --var6_5.f_int_99;
                                        break;
                                    }
                                    var6_5.f_int_104 += (var6_5.f_int_101 - 1) * var6_5.f_int_97;
                                    var6_5.f_int_99 = var6_5.f_int_101;
                                    var6_5.f_int_98 = var6_5.f_int_101 - var6_5.f_int_100;
                                    break;
                                }
                                case -2: {
                                    if (var6_5.f_int_104 < var6_5.f_int_97 * (var6_5.f_int_101 - 1)) {
                                        var6_5.f_int_104 += var6_5.f_int_97;
                                        if (var6_5.f_int_104 / var6_5.f_int_97 < var6_5.f_int_99) break;
                                        ++var6_5.f_int_98;
                                        ++var6_5.f_int_99;
                                        break;
                                    }
                                    var6_5.f_int_104 -= (var6_5.f_int_101 - 1) * var6_5.f_int_97;
                                    var6_5.f_int_98 = 0;
                                    var6_5.f_int_99 = var6_5.f_int_100;
                                    break;
                                }
                                case -3: {
                                    if (var6_5.f_int_104 % var6_5.f_int_97 > 0) {
                                        --var6_5.f_int_104;
                                        break;
                                    }
                                    var6_5.f_int_104 += var6_5.f_int_97 - 1;
                                    break;
                                }
                                case -4: {
                                    if (var6_5.f_int_104 % var6_5.f_int_97 < var6_5.f_int_97 - 1) {
                                        ++var6_5.f_int_104;
                                        break;
                                    }
                                    var6_5.f_int_104 -= var6_5.f_int_97 - 1;
                                    break;
                                }
                                case -6: 
                                case -5: {
                                    if (var6_5.f_int_104 >= var6_5.itemStackSize) break;
                                    var6_5.f_byte_07 = var6_5.itemStackTypes[var6_5.f_int_104];
                                    switch (var6_5.f_byte_07) {
                                        case 13: 
                                        case 14: {
                                            var6_5.gameMode = (byte)3;
                                            super.activateItem(var6_5.f_byte_07);
                                            break block160;
                                        }
                                        case 15: 
                                        case 23: 
                                        case 24: 
                                        case 25: {
                                            super.m_015((byte)3, var6_5.itemDescriptions[a.itemTypeToStackIndex(var6_5.f_byte_07)], (byte)0, (byte)3);
                                            break block160;
                                        }
                                    }
                                    super.m_015((byte)3, var6_5.itemDescriptions[a.itemTypeToStackIndex(var6_5.f_byte_07)], (byte)1, (byte)2);
                                    break;
                                }
                                case -7: {
                                    var6_5.gameMode = (byte)3;
                                }
                            }
                            break;
                        }
                        case 11: {
                            super.m_055();
                            super.m_105();
                            var6_5 = var5_3;
                            var7_15 = '\u0000';
                            block172 : switch (var6_5.f_byte_16) {
                                case 0: {
                                    if (var6_5.f_int_118 > 0) {
                                        --var6_5.f_int_118;
                                        break;
                                    }
                                    super.executeScriptInstruction(var6_5.f_String_05, var6_5.scriptCursor);
                                    break;
                                }
                                case 1: 
                                case 6: {
                                    if (var6_5.f_byte_16 == 6 && !super.m_097()) break;
                                    if (var6_5.f_int_33 >= var6_5.f_String_02.length()) ** GOTO lbl638
                                    if (var6_5.f_int_05 == 0) ** GOTO lbl623
                                    super.m_017(var6_5.f_String_02, 129, (var6_5.f_int_34 << 1) + 12);
                                    var6_5.f_int_33 = super.m_021(var6_5.f_int_33);
                                    ** GOTO lbl633
lbl623:
                                    // 1 sources

                                    if ((var6_5.f_int_03 & 1) == 0) ** GOTO lbl634
                                    ++var6_5.f_int_33;
                                    var7_15 = var6_5.f_String_02.charAt(var6_5.f_int_33 - 1);
                                    if (var7_15 == '\\') {
                                        var7_15 = var6_5.f_String_02.charAt(var6_5.f_int_33);
                                        if (var7_15 == 'c') {
                                            var6_5.f_int_33 += 8;
                                        } else if (var7_15 == 'r') {
                                            var6_5.f_int_33 += 2;
                                        }
                                    }
lbl633:
                                    // 7 sources

                                    super.m_018(var6_5.f_String_02, 129, (var6_5.f_int_34 << 1) + 12, var6_5.f_int_33);
lbl634:
                                    // 2 sources

                                    if (var6_5.f_int_28 <= var6_5.f_int_27) break;
                                    super.m_020(var6_5.f_int_28 - var6_5.f_int_30);
                                    break;
lbl638:
                                    // 1 sources

                                    switch (var6_5.f_int_05) {
                                        case 0: {
                                            break block172;
                                        }
                                        case -1: {
                                            super.m_020(var6_5.f_int_26 - 1);
                                            break block172;
                                        }
                                        case -2: {
                                            super.m_020(var6_5.f_int_26 + 1);
                                            break block172;
                                        }
                                    }
                                    var6_5.f_int_33 = 1;
                                    super.m_096(var6_5.f_int_112 + 1);
                                    if (var6_5.f_byte_16 != 6) break;
                                    super.m_103(var6_5.f_int_114, var6_5.f_int_115);
                                    break;
                                }
                                case 2: {
                                    super.m_106();
                                    break;
                                }
                                case 3: {
                                    if (var6_5.f_bool_16) break;
                                    var8_18 = var6_5;
                                    switch (var8_18.f_byte_11) {
                                        case 5: {
                                            super.tickBattle(var8_18.f_bool_14);
                                            break;
                                        }
                                        case 0: {
                                            if (var8_18.f_int_148 > 0) {
                                                var8_18.f_byte_12 = var8_18.f_byte_arr_42[--var8_18.f_int_148];
                                                var8_18.f_byte_11 = 1;
                                                super.tryStep(var8_18.f_byte_12);
                                                break;
                                            }
                                            var8_18.f_bool_27 = false;
                                            var8_18.f_byte_16 = 0;
                                            break;
                                        }
                                        case 1: {
                                            switch (var8_18.f_byte_12) {
                                                case 1: {
                                                    var8_18.playerPixelY -= 8;
                                                    break;
                                                }
                                                case 0: {
                                                    var8_18.playerPixelY += 8;
                                                    break;
                                                }
                                                case 3: {
                                                    var8_18.playerPixelX -= 8;
                                                    break;
                                                }
                                                case 2: {
                                                    var8_18.playerPixelX += 8;
                                                }
                                            }
                                            var8_18.f_int_43 += 8;
                                            var8_18.f_int_125 = var8_18.playerPixelX;
                                            var8_18.f_int_126 = var8_18.playerPixelY;
                                            super.m_025();
                                            if (var8_18.f_int_43 < 32) break;
                                            var8_18.f_int_43 = 0;
                                            var8_18.f_int_38 = 0;
                                            var8_18.f_byte_11 = 0;
                                            var8_18.playerCellX = var8_18.playerPixelX >> 5;
                                            var8_18.playerCellY = var8_18.playerPixelY >> 5;
                                            super.applyStepCellEffects();
                                        }
                                    }
                                    break;
                                }
                                case 4: {
                                    if (!super.m_097()) break;
                                    var6_5.gameMode = (byte)3;
                                    break;
                                }
                                case 5: {
                                    if (!var6_5.f_bool_16 || var6_5.f_bool_17) break;
                                    var6_5.f_byte_16 = 0;
                                }
                            }
                            break;
                        }
                        case 13: {
                            break;
                        }
                        case 14: {
                            super.m_014();
                            if ((var5_3.f_int_03 & 3) != 0) break;
                            super.m_129(super.randomBelow(240), 320 - var5_3.f_int_16 - super.randomBelow(150), 2, -1);
                            break;
                        }
                        case 12: {
                            super.m_052();
                            break;
                        }
                        case 19: {
                            var6_5 = var5_3;
                            switch (var6_5.f_int_05) {
                                case -3: 
                                case -1: 
                                case 50: 
                                case 52: {
                                    if (var6_5.f_int_10 > 0) {
                                        --var6_5.f_int_10;
                                        break;
                                    }
                                    var6_5.f_int_10 = var6_5.f_int_07 - 1;
                                    break;
                                }
                                case -4: 
                                case -2: 
                                case 54: 
                                case 56: {
                                    if (var6_5.f_int_10 < var6_5.f_int_07 - 1) {
                                        ++var6_5.f_int_10;
                                        break;
                                    }
                                    var6_5.f_int_10 = 0;
                                    break;
                                }
                                case -5: 
                                case 53: {
                                    break;
                                }
                                case -7: {
                                    var6_5.gameMode = (byte)4;
                                    super.m_000();
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
                            switch (var6_5.f_int_05) {
                                case -1: {
                                    if (--var6_5.f_int_82 >= 0) break;
                                    var6_5.f_int_82 = var6_5.f_int_83 - 1;
                                    break;
                                }
                                case -2: {
                                    if (++var6_5.f_int_82 <= var6_5.f_int_83 - 1) break;
                                    var6_5.f_int_82 = 0;
                                    break;
                                }
                                case -5: 
                                case -4: 
                                case -3: {
                                    var7_16 = var6_5;
                                    var7_16.f_bool_arr_05[var7_16.f_int_82] = var7_16.f_bool_arr_05[var7_16.f_int_82] == false;
                                    switch (var7_16.f_byte_arr_11[var7_16.f_int_82]) {
                                        case 0: {
                                            v2 = var7_16.f_bool_29 = var7_16.f_bool_29 == false;
                                            if (var7_16.f_bool_29) {
                                                var7_16.f_int_155 = 60;
                                                if (!var7_16.f_bool_28) {
                                                    super.m_139((byte)2, -1);
                                                } else {
                                                    super.m_139((byte)3, -1);
                                                }
                                            } else {
                                                super.m_140();
                                            }
                                            super.m_110();
                                            break;
                                        }
                                        case 1: {
                                            if (!var7_16.f_bool_arr_05[1]) {
                                                var7_16.f_Image_03 = null;
                                                break;
                                            }
                                            if (var7_16.f_byte_01 != 4) break;
                                            super.m_057();
                                        }
                                    }
                                    break;
                                }
                                case -7: 
                                case -6: {
                                    var6_5.gameMode = var6_5.f_byte_01;
                                }
                            }
                            break;
                        }
                        case 20: {
                            if (!var5_3.f_bool_04 || var5_3.f_int_05 == 0) break;
                            var5_3.gameMode = 1;
                            super.m_000();
                            break;
                        }
                        case 22: {
                            if (++var5_3.f_int_156 <= 70) {
                                super.m_143(1, var5_3.f_int_156);
                                break;
                            }
                            var5_3.f_bool_00 = false;
                            CMidlet.m_000();
                            break;
                        }
                        case 99: {
                            if (var5_3.f_bool_31) {
                                switch (var5_3.f_int_05) {
                                    case -7: {
                                        super.m_148();
                                        var5_3.f_bool_31 = false;
                                        var5_3.gameMode = 0;
                                    }
                                }
                                if (var5_3.f_int_165 < 8) {
                                    ++var5_3.f_int_165;
                                    break;
                                }
                                if (!super.m_153()) break;
                                var5_3.f_bool_31 = false;
                                break;
                            }
                            if (super.m_152() != 0 && super.m_152() != 3 && var5_3.f_int_05 == 0) break;
                            var5_3.gameMode = 0;
                        }
                    }
                    var5_3.f_int_05 = 0;
                }
                this.repaint();
                this.serviceRepaints();
                do {
                    Thread.yield();
                } while ((var5_2 = System.currentTimeMillis()) >= var1_7 && var5_2 < var3_10);
                ++this.f_int_03;
            }
            return;
        }
        catch (Exception v3) {
            this = v3;
            v3.printStackTrace();
            return;
        }
    }

    private void m_000() {
        switch (this.gameMode) {
            case 21: {
                a a2 = this;
                String string = "SKY_WAR";
                int n = 0;
                try {
                    int n2;
                    a2.f_RecordStore_00 = RecordStore.openRecordStore((String)string, (boolean)false);
                    a2.f_RecordEnumeration_00 = a2.f_RecordStore_00.enumerateRecords(null, null, false);
                    n = a2.f_RecordEnumeration_00.nextRecordId();
                    a2.f_byte_arr_27 = a2.f_RecordStore_00.getRecord(n);
                    a2.f_DataInputStream_00 = new DataInputStream(new ByteArrayInputStream(a2.f_byte_arr_27));
                    for (n2 = 0; n2 < 4; ++n2) {
                        a2.f_bool_arr_05[n2] = a2.f_DataInputStream_00.readBoolean();
                    }
                    n = a2.f_DataInputStream_00.readByte();
                    for (n2 = 0; n2 < n; ++n2) {
                        a2.f_int_arr_35[n2] = a2.f_DataInputStream_00.readInt();
                    }
                    a2.f_int_152 = a2.f_DataInputStream_00.readInt();
                    a2.f_int_153 = a2.f_DataInputStream_00.readInt();
                    a2.f_int_154 = a2.f_DataInputStream_00.readInt();
                }
                catch (Exception exception) {
                    a.m_116(string);
                    a2.f_bool_arr_05[1] = true;
                    a2.m_110();
                }
                this.m_001(8);
                this.f_byte_13 = 1;
                this.f_byte_14 = (byte)2;
                this.f_bool_29 = false;
                return;
            }
            case 1: {
                this.f_bool_28 = false;
                this.m_011();
                if (this.maxFloorReached < 51 && this.f_int_154 < this.maxFloorReached) {
                    this.f_int_154 = this.maxFloorReached;
                    this.m_110();
                }
                this.m_001(8);
                this.m_001(10);
                this.m_009();
                this.m_010(0);
                this.m_010(1);
                this.m_010(2);
                this.m_010(3);
                this.m_010(4);
                this.m_010(5);
                this.m_139((byte)2, -1);
                this.m_128();
                this.f_int_141 = -1;
                return;
            }
            case 14: {
                a a3 = this;
                this.f_int_19 = 0;
                a3.f_int_20 = 0;
                a3.f_int_21 = 0;
                a3.f_String_01 = null;
                a3.f_String_03 = null;
                a3.f_Image_01 = null;
                a3.f_bool_03 = false;
                return;
            }
            case 13: {
                return;
            }
            case 5: {
                this.m_091();
                this.f_byte_13 = 0;
                this.f_byte_14 = (byte)3;
                return;
            }
            case 4: {
                this.m_009();
                this.m_010(1);
                if (this.currentFloor > 50) {
                    this.m_010(17);
                }
                this.m_010(3);
                this.m_010(4);
                this.m_010(5);
                this.m_010(7);
                this.f_byte_13 = 1;
                this.f_byte_14 = (byte)3;
                return;
            }
            case 3: {
                this.f_bool_28 = true;
                this.f_byte_13 = 1;
                this.f_byte_14 = (byte)3;
                this.f_bool_05 = false;
                this.applyStepCellEffects();
                this.m_139((byte)3, -1);
                return;
            }
            case 7: 
            case 8: {
                if (this.f_byte_01 == 1) {
                    this.m_001(1);
                    this.m_001(8);
                    this.m_001(6);
                }
                this.f_byte_13 = 1;
                this.f_byte_14 = (byte)3;
                a a4 = this;
                a4.m_111();
                a4.f_int_136 = 208 / (a4.f_int_01 + 4);
                if (a4.f_int_136 > 6) {
                    a4.f_int_136 = 6;
                }
                a4.f_int_135 = a4.f_int_00 * 3 + 60;
                if (a4.f_int_135 < 148) {
                    a4.f_int_135 = 148;
                }
                a4.f_int_131 = 112 + (a4.f_int_01 + 4) * a4.f_int_136;
                a4.f_int_134 = 0;
                a4.f_int_132 = 0;
                a4.f_int_133 = a4.f_int_136;
                return;
            }
            case 9: {
                return;
            }
            case 10: {
                a a5 = this;
                this.f_int_97 = 4;
                a5.f_int_101 = 32 / a5.f_int_97;
                a5.f_int_100 = (268 - a5.f_int_01 - 10) / 40;
                if (a5.f_int_100 > a5.f_int_101) {
                    a5.f_int_100 = a5.f_int_101;
                }
                a5.f_int_98 = 0;
                a5.f_int_99 = a5.f_int_100;
                a5.f_int_103 = 42 + 40 * a5.f_int_97;
                a5.f_int_102 = 34 + a5.f_int_01 + 10 + 40 * a5.f_int_100;
                this.f_byte_13 = 1;
                this.f_byte_14 = (byte)3;
                return;
            }
            case 12: {
                a a6 = this;
                a6.m_076(0);
                this.f_byte_14 = (byte)3;
                return;
            }
            case 19: {
                this.m_009();
                this.m_010(12);
                this.m_010(13);
                this.m_010(14);
                this.m_010(15);
                this.m_010(16);
                return;
            }
            case 15: {
                this.m_001(8);
                this.m_001(1);
                this.m_015((byte)0, "\u6e38\u620f\u63cf\u8ff0\uff1a\n\\c99FFCC\u6709\u4eba\u7684\u5730\u65b9\u5c31\u6709\u6c5f\u6e56\uff0c\u6709\u795e\u4ed9\u7684\u5730\u65b9\u4f55\u5c1d\u4e0d\u662f\u6c5f\u6e56\uff1b\u767e\u6218\u767e\u80dc\u7684\u672c\u4e8b\uff0c\u6362\u4e0d\u56de\u5973\u4eba\u7684\u771f\u5fc3\uff0c\u5144\u5f1f\u7684\u771f\u4e49\uff1b\u9f50\u5929\u5927\u5723\u53c8\u5982\u4f55\uff0c\u6ca1\u6709\u771f\u60c5\u5b9e\u4e49\uff0c\u505a\u795e\u4ed9\u8ddf\u505a\u54b8\u9c7c\u6709\u4ec0\u4e48\u533a\u522b\uff1f\n\n\u64cd\u4f5c\u65b9\u5f0f\uff1a\u6309\u5de6\u8f6f\u952e\u8c03\u51fa\u7269\u54c1\u680f\uff0c\u5de6\u53f3\u9009\u62e9\u4e00\u4ef6\u9053\u5177\uff0c\u6309\u786e\u5b9a\u952e\u4f7f\u7528\u3002\n\u6e38\u620f\u64cd\u4f5c\uff1a\n\u4e0a\u65b9\u5411\u952e/2\uff1a\u5411\u4e0a\u884c\u8d70\n\u4e0b\u65b9\u5411\u952e/8\uff1a\u5411\u4e0b\u884c\u8d70\n\u5de6\u65b9\u5411\u952e/4\uff1a\u5411\u5de6\u884c\u8d70\n\u53f3\u65b9\u5411\u952e/6\uff1a\u5411\u53f3\u884c\u8d70\n\u786e\u5b9a\u952e/5:\u63a2\u7d22\u5730\u56fe\n\u5de6\u8f6f\u952e\uff1a\u6253\u5f00\u9053\u5177\u5217\u8868\n\u53f3\u8f6f\u952e\uff1a\u6253\u5f00\u6e38\u620f\u4e2d\u83dc\u5355\n\n\u4ee3\u7406\u53d1\u884c\uff1a\u5e7f\u5dde\u6613\u8bda\u8ba1\u7b97\u673a\u79d1\u6280\u6709\u9650\u516c\u53f8\n\u53d1\u884c\u5546\u7f51\u7ad9\uff1awww.9266.net\n\u5ba2\u670d\u7535\u8bdd\uff1a4006509913\n\u5ba2\u670d\u4fe1\u7bb1\uff1akefu@9266.net", (byte)0, (byte)0);
                this.f_byte_13 = 0;
                this.f_byte_14 = (byte)3;
                return;
            }
            case 17: {
                this.m_001(8);
                this.m_001(1);
                this.m_015((byte)0, "\u7248\u6743\u6240\u6709\uff1a\n\u4e0a\u6d77\u96ea\u9ca4\u9c7c\u8ba1\u7b97\u673a\u79d1\u6280\u6709\u9650\u516c\u53f8\nwww.kgame.com.cn\n\u624b\u673a\u4e0a\u7f51\uff1a\nwap.kgame.com.cn\n\u5236\u4f5c\u4eba\uff1a\u6881\u4e00\n\u7f16\u5267\uff1a\u738b\u4e4b\u6d63\n\u7b56\u5212\uff1a\u5b59\u60a6\n\u7a0b\u5e8f\uff1a\u6768\u653f\n\u7f8e\u672f\uff1a\u6881\u4e00\u3001\u738b\u4e4b\u6d63\u3001\u9ec4\u5409\u529b\n\u6d4b\u8bd5\uff1a\u91d1\u946b\uff0c\u738b\u6bc5\uff0c\u8ba1\u6210\u6bc5\n\u7248\u672c\uff1aV1.0\n\u5ba2\u670d\u7535\u8bdd\uff1a4006305518", (byte)0, (byte)0);
                this.f_byte_13 = 0;
                this.f_byte_14 = (byte)3;
                return;
            }
            case 16: {
                this.m_001(8);
                this.m_001(1);
                this.f_int_83 = 0;
                this.m_072(0);
                this.m_072(1);
                this.f_byte_13 = 0;
                this.f_byte_14 = (byte)3;
                return;
            }
            case 20: {
                this.m_011();
                this.m_001(14);
                this.f_String_03 = this.dialogueTexts[260];
                this.f_int_33 = 1;
                this.m_018(this.f_String_02, 209, (this.f_int_34 << 1) + 12, this.f_int_33);
                return;
            }
            case 22: {
                this.m_140();
            }
        }
    }

    protected final void showNotify() {
        if (this.f_bool_01) {
            return;
        }
        this.f_bool_01 = true;
        super.showNotify();
    }

    protected final void hideNotify() {
        if (!this.f_bool_01) {
            return;
        }
        if (this.gameMode == 3) {
            this.gameMode = (byte)4;
            this.m_000();
        }
        this.m_140();
        this.f_bool_02 = true;
        this.f_bool_01 = false;
        super.hideNotify();
    }

    private void m_001(int n) {
        int n2 = this.f_int_arr_00[n];
        int n3 = 0;
        if (this.f_Image_arr2_00[n] == null) {
            this.f_Image_arr2_00[n] = new Image[this.f_int_arr_00[n]];
        }
        this.f_InputStream_00 = this.getClass().getResourceAsStream(this.f_String_arr_01[n]);
        byte[] byArray = null;
        try {
            for (int i = 0; i < n2; ++i) {
                n3 = this.f_InputStream_00.read() & 0xFF | this.f_InputStream_00.read() << 8 & 0xFF00;
                if (this.f_Image_arr2_00[n][i] == null) {
                    if (byArray == null) {
                        byArray = new byte[n3];
                    } else if (byArray.length < n3) {
                        byArray = new byte[n3];
                    }
                    this.f_InputStream_00.read(byArray, 0, n3);
                    this.f_Image_arr2_00[n][i] = Image.createImage((byte[])byArray, (int)0, (int)n3);
                    continue;
                }
                this.f_InputStream_00.skip(n3);
            }
            return;
        }
        catch (Exception exception) {
            Exception exception2 = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.m_008();
        }
    }

    private void m_002(Image image, int n, int n2, int n3, int n4, int n5, int n6) {
        this.f_Graphics_00.setClip(n, n2, n5, n6);
        this.f_Graphics_00.drawImage(image, n - n3, n2 - n4, 0);
        this.f_Graphics_00.setClip(0, 0, 240, 320);
    }

    private static void m_003(Image image, Graphics graphics, int n, int n2, int n3, int n4, int n5, int n6) {
        graphics.setClip(n, n2, 5, 5);
        graphics.drawImage(image, n - n3, n2, 0);
        graphics.setClip(0, 0, 240, 320);
    }

    private void m_004(Image image, int n, int n2, int n3) {
        this.f_DirectGraphics_00.drawImage(image, n, n2, 0, f_int_arr_01[n3]);
    }

    private void m_005(Image image, int n, int n2, int n3, int n4, int n5, int n6, int n7) {
        this.f_Graphics_00.setClip(n, n2, n5, n6);
        if (n7 == 0) {
            this.f_Graphics_00.drawImage(image, n - n3, n2 - n4, 0);
        } else {
            int n8 = image.getWidth();
            int n9 = image.getHeight();
            switch (n7) {
                case 1: {
                    this.m_004(image, n - (n8 - n3 - n5), n2 - n4, n7);
                    break;
                }
                case 2: {
                    this.m_004(image, n - n3, n2 - (n9 - n4 - n6), n7);
                    break;
                }
                case 3: {
                    this.m_004(image, n - (n8 - n3 - n5), n2 - (n9 - n4 - n6), n7);
                    break;
                }
                case 4: {
                    this.f_Graphics_00.setClip(n, n2, n6, n5);
                    this.m_004(image, n - n4, n2 - n3, n7);
                    break;
                }
                case 5: {
                    this.f_Graphics_00.setClip(n, n2, n6, n5);
                    this.m_004(image, n - (n9 - n4 - n6), n2 - n3, n7);
                    break;
                }
                case 6: {
                    this.f_Graphics_00.setClip(n, n2, n6, n5);
                    this.m_004(image, n - n4, n2 - (n8 - n3 - n5), n7);
                    break;
                }
                case 7: {
                    this.f_Graphics_00.setClip(n, n2, n6, n5);
                    this.m_004(image, n - (n9 - n4 - n6), n2 - (n8 - n3 - n5), n7);
                }
            }
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
    }

    private static Image m_006(int n, int n2, int n3) {
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

    private static Image m_007(Image image, int n) {
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

    private void m_008() {
        if (this.f_InputStream_00 != null) {
            try {
                this.f_InputStream_00.close();
            }
            catch (Exception exception) {}
            this.f_InputStream_00 = null;
        }
        if (this.f_OutputStream_00 != null) {
            try {
                this.f_OutputStream_00.close();
            }
            catch (Exception exception) {}
            this.f_OutputStream_00 = null;
        }
        if (this.f_InputStream_01 != null) {
            try {
                this.f_InputStream_01.close();
            }
            catch (Exception exception) {}
            this.f_InputStream_01 = null;
        }
    }

    protected final void keyPressed(int n) {
        this.f_int_05 = this.f_int_06 = n;
        if (this.f_bool_02) {
            if (this.gameMode != 21 && this.gameMode != 0) {
                if (!this.f_bool_28) {
                    this.m_139((byte)2, -1);
                } else {
                    this.m_139((byte)3, -1);
                }
            }
            this.f_bool_02 = false;
        }
    }

    protected final void keyReleased(int n) {
        this.f_int_06 = 0;
    }

    private void m_009() {
        this.f_int_15 = this.f_int_01 + 6;
        this.f_int_13 = 238 / this.f_int_15;
        this.f_int_14 = this.f_int_15 * this.f_int_13;
        this.f_int_08 = this.f_int_00 << 1;
        this.f_int_09 = 0;
        this.f_int_07 = 0;
        if (this.f_byte_arr_00 == null) {
            this.f_byte_arr_00 = new byte[16];
        }
        this.f_int_11 = 0;
    }

    private void m_010(int n) {
        if (this.f_int_07 < 16) {
            int n2 = this.f_Font_00.stringWidth(this.f_String_arr_02[n]) + 60;
            this.f_byte_arr_00[this.f_int_07] = n;
            if (n2 > this.f_int_08) {
                this.f_int_08 = n2;
            }
            ++this.f_int_07;
            if (this.f_int_07 <= this.f_int_13) {
                this.f_int_12 = this.f_int_07;
            }
            this.f_int_09 += this.f_int_15;
            this.f_int_14 = (this.f_int_12 - this.f_int_11) * this.f_int_15;
        }
    }

    private void m_011() {
        this.f_Image_arr2_00[1] = null;
        this.f_Image_arr2_00[2] = null;
        this.f_Image_arr2_00[3] = null;
        this.f_Image_arr2_00[4] = null;
        this.f_Image_arr2_00[5] = null;
        this.f_Image_arr2_00[6] = null;
        this.f_Image_arr2_00[7] = null;
        this.f_Image_arr2_00[13] = null;
        this.f_Image_arr2_00[9] = null;
        this.f_Image_arr2_00[11] = null;
        this.f_Image_arr2_00[14] = null;
        this.f_Image_arr2_00[12] = null;
        this.f_Image_arr_00 = null;
    }

    private void m_012() {
        switch (this.f_byte_arr_00[this.f_int_10]) {
            case 0: {
                return;
            }
            case 1: {
                this.gameMode = (byte)3;
                return;
            }
            case 2: {
                this.gameMode = (byte)8;
                this.f_byte_01 = (byte)4;
                this.m_000();
                return;
            }
            case 3: {
                this.gameMode = (byte)7;
                this.f_byte_01 = (byte)4;
                this.m_000();
                return;
            }
            case 4: {
                this.gameMode = (byte)16;
                this.f_byte_01 = (byte)4;
                this.m_000();
                return;
            }
            case 5: {
                this.gameMode = (byte)15;
                this.f_byte_01 = (byte)4;
                this.m_000();
                return;
            }
            case 7: {
                this.m_011();
                System.gc();
                try {
                    Thread.sleep(100L);
                }
                catch (Exception exception) {}
                this.gameMode = 1;
                this.m_000();
                this.f_int_10 = 0;
                return;
            }
            case 9: {
                return;
            }
            case 10: {
                return;
            }
            case 11: {
                this.gameMode = (byte)19;
                this.m_000();
                return;
            }
            case 17: {
                this.changeFloor(1, false, false);
                this.gameMode = (byte)3;
                this.playerHp = 1000;
                this.playerAtk = 710;
                this.playerDef = 710;
            }
        }
    }

    private void m_013(Image image, int n, int n2, int n3, int n4, int n5, int n6, int n7) {
        n = 86;
        n7 = this.f_byte_03;
        int n8 = 0;
        while (n8 < n5) {
            n3 = n + this.f_byte_arr_01[n7];
            n6 = n2 + this.f_byte_arr_02[n7];
            this.f_Graphics_00.setClip(n3, n6, 17, 17);
            this.f_Graphics_00.drawImage(image, n3 - n8, n6 - n4, 0);
            if (++n7 > 7) {
                n7 = 0;
            }
            n8 += 17;
            n += 17;
        }
        this.f_byte_03 = (byte)(this.f_byte_03 + 1);
        if (this.f_byte_03 > 7) {
            this.f_byte_03 = 0;
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
    }

    /*
     * Enabled force condition propagation
     * Lifted jumps to return sites
     */
    private void m_014() {
        if (this.f_int_05 == -7) {
            this.f_Image_01 = null;
            this.f_Image_arr2_00[11] = null;
            this.f_Image_arr2_00[14] = null;
            this.f_String_01 = null;
            this.f_int_19 = 0;
            this.f_int_20 = 0;
            this.f_int_21 = 0;
            this.m_067();
            return;
        }
        if (this.f_int_20 == 0) {
            this.f_Image_01 = null;
            switch (this.f_int_19) {
                case 0: {
                    this.m_001(11);
                    this.f_Image_01 = a.m_007(this.f_Image_arr2_00[11][0], 34);
                    this.f_String_01 = this.f_String_arr_00[4];
                    break;
                }
                case 1: {
                    this.m_001(11);
                    this.f_Image_01 = a.m_007(this.f_Image_arr2_00[11][1], 34);
                    this.f_String_01 = this.f_String_arr_00[6];
                    break;
                }
                case 2: {
                    this.f_Image_arr2_00[11] = null;
                    this.m_001(14);
                    this.f_Image_01 = a.m_007(this.f_Image_arr2_00[14][0], 34);
                    this.f_String_01 = this.f_String_arr_00[7];
                }
            }
            if (this.f_Image_01 != null) {
                this.f_Image_01.getWidth();
            }
            this.f_bool_03 = false;
        }
        ++this.f_int_20;
        if (this.f_int_19 < 3) {
            if (this.f_String_03 == null) return;
            if (!this.f_bool_03) {
                if (this.f_int_21 < this.f_Font_00.stringWidth(this.f_String_arr_03[this.f_int_26]) + 10) {
                    this.f_int_21 += 4;
                    if (this.f_int_05 == 0) return;
                    this.f_int_21 = 240;
                    return;
                }
                this.f_bool_03 = true;
                this.f_byte_04 = 0;
                return;
            }
            this.f_byte_04 = (byte)(this.f_byte_04 + 1);
            if (this.f_byte_04 < this.f_int_arr_02.length) return;
            this.f_int_21 = 0;
            this.f_byte_04 = 0;
            this.f_bool_03 = false;
            if (this.f_int_26 < this.f_int_28 - 2) {
                this.m_020(this.f_int_26 + 2);
                return;
            }
            System.out.println("++~");
            ++this.f_int_19;
            this.f_int_20 = 0;
            this.m_014();
            return;
        }
        this.f_Image_01 = null;
        this.f_Image_arr2_00[11] = null;
        this.f_Image_arr2_00[14] = null;
        this.f_String_01 = null;
        this.f_int_19 = 0;
        this.m_067();
    }

    private void m_015(byte by, String string, byte by2, byte by3) {
        this.f_bool_05 = true;
        this.f_byte_08 = by;
        this.f_String_02 = string;
        switch (this.f_byte_08) {
            case 1: 
            case 3: {
                this.m_017(string, 180, 234);
                this.f_int_24 = 212;
                this.f_int_25 = 68 + (this.f_int_27 - this.f_int_26) * this.f_int_34;
                break;
            }
            case 0: 
            case 2: 
            case 4: 
            case 5: {
                this.m_017(string, 180, 240);
                this.f_int_24 = this.f_int_29 + 32;
                this.f_int_25 = 32 + (this.f_int_27 - this.f_int_26) * this.f_int_34;
            }
        }
        this.f_byte_05 = by2;
        this.f_byte_06 = by3;
        if (by2 != 0 || by3 != 0) {
            this.f_int_25 += 18;
        }
    }

    private void m_016() {
        switch (this.f_int_05) {
            case -1: {
                this.m_020(this.f_int_26 - 1);
                return;
            }
            case -2: {
                this.m_020(this.f_int_26 + 1);
                return;
            }
            case 0: {
                return;
            }
        }
        this.f_bool_05 = false;
    }

    private void m_017(String string, int n, int n2) {
        this.m_018(string, n, n2, string.length());
    }

    private void m_018(String string, int n, int n2, int n3) {
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
            n4 = this.f_Font_00.charWidth(c);
            if (c == '\n') {
                this.f_String_arr_03[n6++] = new String(cArray, n7, n8);
                n7 = i + 1;
                n5 = 0;
                n8 = 0;
                continue;
            }
            if (c != '\\') {
                n4 = this.f_Font_00.charWidth(c);
            } else if (++i < n3) {
                c = string.charAt(i);
                if (c == 'c') {
                    if (i + 6 < n3 && a.m_022(string.substring(i + 1, i + 7)) != -1) {
                        i += 6;
                        n8 += 7;
                        n4 = 0;
                    } else {
                        n4 = this.f_Font_00.charWidth('\\') + this.f_Font_00.charWidth(c);
                    }
                } else if (c == 'r') {
                    ++n8;
                    n4 = 0;
                }
            } else {
                n4 = this.f_Font_00.charWidth('\\');
                --i;
            }
            if ((n5 += n4) > n) {
                this.f_String_arr_03[n6++] = new String(cArray, n7, n8);
                n5 = n4;
                n7 = i;
                n8 = 1;
                continue;
            }
            ++n8;
        }
        if (n8 > 0) {
            this.f_String_arr_03[n6++] = new String(cArray, n7, n8);
        }
        this.f_int_26 = 0;
        this.f_int_28 = n6;
        this.f_int_30 = n2 / this.f_int_34;
        if (this.f_int_28 < this.f_int_30) {
            this.f_int_30 = this.f_int_28;
        }
        this.f_int_31 = this.f_int_34 * this.f_int_30;
        this.f_int_27 = n6 > this.f_int_30 ? this.f_int_30 : n6;
        if (this.f_int_28 == 1) {
            this.f_int_29 = this.f_Font_00.stringWidth(string);
            return;
        }
        this.f_int_29 = n;
    }

    private void m_019(String string, int n, int n2, int n3, int n4, boolean bl) {
        if (string == null) {
            return;
        }
        if (this.f_String_03 != string) {
            this.f_String_03 = string;
            this.m_017(string, n3, n4);
        }
        int n5 = n2 + 2;
        n4 = 0;
        n4 = 0;
        char c = '\u0000';
        this.f_int_32 = this.f_Graphics_00.getColor();
        int n6 = this.f_int_26;
        while (--n6 >= 0) {
            if (this.f_String_arr_03[n6] == null || (n4 = this.f_String_arr_03[n6].lastIndexOf(92)) < 0 || (c = this.f_String_arr_03[n6].charAt(n4 + 1)) != 'c') continue;
            n4 = a.m_022(this.f_String_arr_03[n6].substring(n4 + 2, n4 + 8));
            this.f_Graphics_00.setColor(n4);
            break;
        }
        n6 = this.f_int_26;
        while (n6 < this.f_int_27) {
            int n7 = n5;
            int n8 = n;
            String string2 = this.f_String_arr_03[n6];
            a a2 = this;
            int n9 = string2.length();
            int n10 = 0;
            int n11 = n8;
            char c2 = '\u0000';
            int n12 = 0;
            for (int i = 0; i < n9; ++i) {
                if (string2.charAt(i) != '\\') continue;
                if (i > 0) {
                    a2.f_Graphics_00.drawSubstring(string2, n10, i - n10, n11, n7, 0);
                    n11 += a2.f_Font_00.substringWidth(string2, n10, i - n10);
                }
                if (++i >= n9) continue;
                c2 = string2.charAt(i);
                if (c2 == 'c') {
                    if (i + 6 < n9) {
                        n12 = a.m_022(string2.substring(i + 1, i + 7));
                    }
                    if (n12 != -1) {
                        a2.f_int_32 = a2.f_Graphics_00.getColor();
                        a2.f_Graphics_00.setColor(n12);
                        n10 = (i += 6) + 1;
                        continue;
                    }
                    n10 = i - 1;
                    continue;
                }
                if (c2 == 'r') {
                    a2.f_Graphics_00.setColor(a2.f_int_32);
                    n10 = i + 1;
                    continue;
                }
                n10 = i;
            }
            if (n10 != 0) {
                if (n10 < n9) {
                    a2.f_Graphics_00.drawSubstring(string2, n10, n9 - n10, n11, n7, 0);
                }
            } else {
                a2.f_Graphics_00.drawString(string2, n8, n7, 0);
            }
            ++n6;
            n5 += this.f_int_34;
        }
        n6 = n + (n3 >> 1);
        if (bl) {
            this.f_Graphics_00.setColor(-1);
            if (this.f_int_26 > 0) {
                n5 = n2 - 8 + (this.f_int_03 & 1);
                this.f_Graphics_00.fillTriangle(n6, n5, n6 - 7, n5 + 7, n6 + 7, n5 + 7);
            }
            if (this.f_int_27 < this.f_int_28) {
                n5 = n2 + this.f_int_31 + 3 - (this.f_int_03 & 1);
                this.f_Graphics_00.fillTriangle(n6, n5, n6 - 6, n5 - 6, n6 + 6, n5 - 6);
            }
        }
    }

    private boolean m_020(int n) {
        if (n >= 0) {
            if (n >= this.f_int_28 - this.f_int_30 + 1) {
                n = this.f_int_28 - this.f_int_30;
            }
        } else {
            n = 0;
        }
        this.f_int_26 = n;
        this.f_int_27 = this.f_int_26 + this.f_int_30;
        if (this.f_int_26 < 0) {
            this.f_int_26 = 0;
        }
        if (this.f_int_27 > this.f_int_28) {
            this.f_int_27 = this.f_int_28;
        }
        return true;
    }

    private int m_021(int n) {
        int n2 = 0;
        int n3 = 0;
        int n4 = 0;
        if (this.f_String_arr_03 != null) {
            int n5 = 0;
            while (n5 < this.f_int_28) {
                n4 = this.f_String_arr_03[n5].length();
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

    private static int m_022(String string) {
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

    private void applyHpDelta(int n) {
        if (this.playerHp <= -n) {
            this.playerHp = 1;
            n = this.playerHp - 1;
        } else {
            this.playerHp += n;
        }
        if (n < 0) {
            this.m_125((byte)2, n, this.playerPixelX, this.playerPixelY);
            return;
        }
        if (n > 0) {
            if (this.f_byte_26 > 0) {
                n <<= 4;
            }
            this.m_125((byte)3, n, this.playerPixelX, this.playerPixelY);
        }
    }

    private void m_024(int n, int n2) {
        this.playerCellX = n;
        this.playerCellY = n2;
        this.playerPixelX = n << 5;
        this.playerPixelY = n2 << 5;
        this.f_int_43 = 0;
    }

    private void m_025() {
        if (this.f_int_38 < this.f_int_arr_03.length - 1) {
            ++this.f_int_38;
            return;
        }
        this.f_int_38 = 0;
    }

    private void m_026() {
        int n = 0;
        int n2 = 0;
        int n3 = 0;
        if (!this.f_bool_06) {
            n = (this.f_int_58 - 41 >> 1) - this.f_int_56;
            n2 = (this.f_int_59 >> 1) - this.f_int_57 + this.f_int_46;
        } else {
            n = this.playerPixelX;
            n2 = this.playerPixelY;
        }
        if (this.f_int_arr_04[3] != n || this.f_int_arr_05[3] != n2) {
            this.f_int_arr_04[0] = n;
            this.f_int_arr_05[0] = n2;
            for (int i = 1; i < 4; ++i) {
                n3 = 4 - i;
                if (i == 0) {
                    n3 += 5;
                }
                if (this.f_int_arr_04[i] < n) {
                    int n4 = i;
                    this.f_int_arr_04[n4] = this.f_int_arr_04[n4] + ((n - this.f_int_arr_04[i] >> 1) + n3);
                    if (this.f_int_arr_04[i] > n) {
                        this.f_int_arr_04[i] = n;
                    }
                } else if (this.f_int_arr_04[i] > n) {
                    int n5 = i;
                    this.f_int_arr_04[n5] = this.f_int_arr_04[n5] + ((n - this.f_int_arr_04[i] >> 1) - n3);
                    if (this.f_int_arr_04[i] < n) {
                        this.f_int_arr_04[i] = n;
                    }
                }
                if (this.f_int_arr_05[i] < n2) {
                    int n6 = i;
                    this.f_int_arr_05[n6] = this.f_int_arr_05[n6] + ((n2 - this.f_int_arr_05[i] >> 1) + n3);
                    if (this.f_int_arr_05[i] > n2) {
                        this.f_int_arr_05[i] = n2;
                    }
                } else if (this.f_int_arr_05[i] > n2) {
                    int n7 = i;
                    this.f_int_arr_05[n7] = this.f_int_arr_05[n7] + ((n2 - this.f_int_arr_05[i] >> 1) - n3);
                    if (this.f_int_arr_05[i] < n2) {
                        this.f_int_arr_05[i] = n2;
                    }
                }
                n = this.f_int_arr_04[i];
                n2 = this.f_int_arr_05[i];
            }
            return;
        }
        if (this.f_bool_06) {
            this.f_byte_11 = 0;
        }
    }

    private void m_027() {
        int n = 0;
        if (this.gameMode == 11) {
            this.m_032();
            return;
        }
        switch (this.f_int_05) {
            case -5: 
            case 53: {
                a a2 = this;
                for (int i = 0; i < 4; ++i) {
                    a2.f_int_arr_04[i] = a2.playerPixelX;
                    a2.f_int_arr_05[i] = a2.playerPixelY;
                }
                a2.f_bool_06 = false;
                a a3 = a2;
                int n2 = 0;
                int n3 = a3.f_int_45;
                while (--n3 >= 0) {
                    n2 = a3.entityType[n3];
                    if (a3.f_bool_arr_01[n3] || a3.f_byte_arr_03[n2] != 8) continue;
                    a3.f_int_arr_13[n2] = a3.predictHpLossVsType(n2, true);
                }
                a2.f_byte_11 = (byte)2;
                a2.f_bool_08 = a2.m_081(13) >= 0;
                break;
            }
            case 49: {
                this.f_int_05 = 0;
                if (!this.f_bool_13 || (n = this.m_065(true)) < 0) break;
                this.changeFloor(this.entityParam[n] & 0xFF, false, true);
                break;
            }
            case 55: {
                this.f_int_05 = 0;
                if (!this.f_bool_13 || (n = this.m_065(false)) < 0) break;
                this.changeFloor(this.entityParam[n] & 0xFF, true, true);
            }
        }
        switch (this.f_int_06) {
            case -1: 
            case 50: {
                this.f_byte_12 = 1;
                this.tryStep(this.f_byte_12);
                return;
            }
            case -2: 
            case 56: {
                this.f_byte_12 = 0;
                this.tryStep(this.f_byte_12);
                return;
            }
            case -3: 
            case 52: {
                this.f_byte_12 = (byte)3;
                this.tryStep(this.f_byte_12);
                return;
            }
            case -4: 
            case 54: {
                this.f_byte_12 = (byte)2;
                this.tryStep(this.f_byte_12);
            }
        }
    }

    /*
     * WARNING - void declaration
     */
    private boolean tryStep(byte by) {
        void var2_6;
        boolean bl = false;
        int n = this.playerPixelX >> 5;
        int n2 = this.playerPixelY >> 5;
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
        boolean bl2 = this.interactWithCell((int)var2_6, n2);
        if (bl2) {
            this.f_byte_11 = 1;
        } else if (this.f_bool_05 && this.f_byte_11 == 1) {
            this.f_bool_26 = false;
            this.f_int_148 = 0;
            this.f_byte_11 = 0;
        }
        return bl2;
    }

    /*
     * Enabled aggressive block sorting
     */
    private boolean interactWithCell(int n, int n2) {
        byte by = 0;
        int n3 = 0;
        int n4 = 0;
        n4 = 0;
        int n5 = 1;
        int n6 = 0;
        by = this.f_byte_arr2_02[n2][n];
        if (this.f_byte_arr_10[by] <= 0) {
            if (this.isCellWalkable(n, n2)) return n5 != 0;
            return 0 != 0;
        }
        int n7 = this.f_byte_arr_10[by];
        while (true) {
            if (--n7 < 0) {
                if (n5 == 0) return n5 != 0;
                if (n6 > 0) return n5 != 0;
                return this.isCellWalkable(n, n2) != 0;
            }
            n3 = this.f_byte_arr2_03[by][n7] - 1;
            n4 = this.entityType[n3];
            if (this.f_byte_arr_04[n3] == 1) continue;
            block0 : switch (this.f_byte_arr_03[n4]) {
                case 8: {
                    n4 = this.predictHpLossVsType(n4, true);
                    n5 = 0;
                    if (n4 < 0 || n4 >= this.playerHp) {
                        this.f_int_148 = 0;
                        this.m_015((byte)0, "\u4f60\u65e0\u6cd5\u6218\u80dc\u5b83", (byte)0, (byte)0);
                        break;
                    }
                    n4 = n3;
                    a a2 = this;
                    this.f_byte_11 = (byte)5;
                    a2.f_byte_arr_04[n4] = 3;
                    a2.f_int_44 = n4;
                    a2.f_int_150 = a2.enemyBaseHp[a2.entityType[n4] - 41];
                    a2.f_bool_14 = true;
                    a2.f_bool_25 = true;
                    break;
                }
                case 32: {
                    n5 = 0;
                    break;
                }
                case 16: {
                    n4 = n3;
                    a a3 = this;
                    this.f_int_87 = n4;
                    n5 = a3.entityParam[n4];
                    byte by2 = (byte)(n5 >>> 8);
                    n5 = (byte)n5;
                    int n8 = n4 = a3.entityType[n4] == 77 ? 0 : 1;
                    if (n5 > 0) {
                        n5 = (byte)(n5 - 1);
                        if ((by2 & 2) != 0) {
                            switch (n4) {
                                case 0: {
                                    boolean bl = false;
                                    n4 = n5;
                                    a3.m_015((byte)4, a3.f_String_arr_07[n4], (byte)0, (byte)0);
                                    break;
                                }
                                case 1: {
                                    a3.m_015((byte)4, a3.f_String_arr_08[n5], (byte)1, (byte)2);
                                }
                            }
                        } else {
                            a3.m_075(n4, n5);
                            switch (n4) {
                                case 0: {
                                    a3.m_015((byte)4, a3.f_String_arr_06[n5], (byte)0, (byte)0);
                                    break;
                                }
                                case 1: {
                                    a3.m_015((byte)4, a3.f_String_arr_09[n5], (byte)1, (byte)2);
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
                            if (!this.tryRunScene(this.entityParam[n3], false)) break;
                            this.f_bool_05 = false;
                            this.f_String_02 = null;
                            n5 = this.loadLevelScript(this.entityParam[n3]) ? 1 : 0;
                            n7 = 0;
                            break block0;
                        }
                        case 1: {
                            n5 = this.consumeKeyForDoor((byte)26);
                            if (n5 != 0) {
                                this.m_047(n3);
                                break block0;
                            }
                            this.f_int_148 = 0;
                            this.m_015((byte)0, "\u4f60\u6ca1\u6709\u9ec4\u94a5\u5319", (byte)0, (byte)0);
                            break block0;
                        }
                        case 3: {
                            n5 = this.consumeKeyForDoor((byte)28);
                            if (n5 != 0) {
                                this.m_047(n3);
                                break block0;
                            }
                            this.f_int_148 = 0;
                            this.m_015((byte)0, "\u4f60\u6ca1\u6709\u84dd\u94a5\u5319", (byte)0, (byte)0);
                            break block0;
                        }
                        case 2: {
                            n5 = this.consumeKeyForDoor((byte)27);
                            if (n5 != 0) {
                                this.m_047(n3);
                                break block0;
                            }
                            this.f_int_148 = 0;
                            this.m_015((byte)0, "\u4f60\u6ca1\u6709\u7ea2\u94a5\u5319", (byte)0, (byte)0);
                            break block0;
                        }
                        case 81: {
                            if (!this.f_bool_arr_00[n3]) break;
                            this.m_015((byte)0, "\u969c\u788d\u7269\uff1a\u5c01\u5370\u95e8\n\u9700\u8981\u6d88\u706d\u6307\u5b9a\u7684\u602a\u7269\u624d\u80fd\u6253\u5f00\u7684\u95e8\uff01", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 4: {
                            this.m_015((byte)0, "\u969c\u788d\u7269\uff1a\u5c01\u5370\u95e8\n\u9700\u8981\u6d88\u706d\u6307\u5b9a\u7684\u602a\u7269\u624d\u80fd\u6253\u5f00\u7684\u95e8\uff01", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 9: {
                            n5 = 0;
                            if (this.f_byte_12 != 1) break;
                            n7 = 0;
                            this.gameMode = (byte)9;
                            a a4 = this;
                            n4 = a4.currentFloor;
                            a4.alchemyPrice = a.alchemyPriceFor(a4.alchemyUpgradeCount + 1);
                            a4.f_int_77 = a.scaledByFloorTier(100, n4);
                            a4.f_int_78 = a.scaledByFloorTier(2, n4);
                            a4.f_int_79 = a.scaledByFloorTier(4, n4);
                            this.m_000();
                            break block0;
                        }
                        case 10: {
                            this.m_015((byte)0, "\u969c\u788d\u7269\uff1a\u4e09\u6627\u771f\u706b\n\u5fc5\u987b\u7528\u82ad\u8549\u6247\u624d\u80fd\u7184\u706d\u5b83. ", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 11: {
                            if (this.currentFloor != 23) {
                                this.m_015((byte)0, "\u969c\u788d\u7269\uff1a\u5899\n\u53ea\u6709\u91d1\u52fa\u5b50\u3001\u7384\u660e\u77f3\u53ef\u4ee5\u51ff\u5f00\u3002\u6216\u8005\u5267\u60c5\u6253\u5f00\uff01", (byte)0, (byte)0);
                            }
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 12: {
                            this.entityType[n3] = 11;
                            this.m_044(11, n3);
                            this.m_045(11, n3);
                            this.f_byte_arr_04[n3] = 2;
                            this.f_int_arr_12[n3] = 9;
                            this.f_bool_arr_00[n3] = true;
                            n5 = 0;
                            break block0;
                        }
                        case 6: {
                            n6 = (byte)(n6 + 1);
                            if (this.f_bool_arr_00[n3]) break;
                            this.f_bool_arr_00[n3] = true;
                            this.f_byte_arr_04[n3] = 2;
                            this.f_int_arr_12[n3] = 9;
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
    private void applyStepCellEffects() {
        block62: {
            block61: {
                block59: {
                    var1_1 = false;
                    this.playerCellX = this.playerPixelX >> 5;
                    this.playerCellY = this.playerPixelY >> 5;
                    var1_1 = false;
                    var2_6 = 0;
                    var3_7 = this.f_byte_arr2_02[this.playerCellY][this.playerCellX];
                    this.f_bool_07 = false;
                    var1_2 = this;
                    var2_6 = 0;
                    var6_8 = 0;
                    var7_9 = var1_2.equippedArmorType == 40 ? 1 : 0;
                    if (var7_9 == 0) {
                        var10_10 = var1_2.playerCellY;
                        var9_11 = var1_2.playerCellX;
                        var8_13 = var1_2;
                        if (var9_11 > 0 && var8_13.m_100(var9_11 - 1, var10_10, 61) >= 0 && var9_11 < var8_13.mapCellsWide - 1 && var8_13.m_100(var9_11 + 1, var10_10, 61) >= 0 ? true : var10_10 > 0 && var8_13.m_100(var9_11, var10_10 - 1, 61) >= 0 && var10_10 < var8_13.mapCellsHigh - 1 && var8_13.m_100(var9_11, var10_10 + 1, 61) >= 0) {
                            var1_2.applyHpDelta(-(var1_2.playerHp >> 1));
                        }
                        var11_16 = 62;
                        var10_10 = var1_2.playerCellY;
                        var9_11 = var1_2.playerCellX;
                        var8_13 = var1_2;
                        var12_17 = 0;
                        if (var9_11 > 0 && var8_13.m_100(var9_11 - 1, var10_10, 62) >= 0) {
                            ++var12_17;
                        }
                        if (var9_11 < var8_13.mapCellsHigh - 1 && var8_13.m_100(var9_11 + 1, var10_10, 62) >= 0) {
                            ++var12_17;
                        }
                        if (var10_10 > 0 && var8_13.m_100(var9_11, var10_10 - 1, 62) >= 0) {
                            ++var12_17;
                        }
                        if (var10_10 < var8_13.mapCellsHigh - 1 && var8_13.m_100(var9_11, var10_10 + 1, 62) >= 0) {
                            ++var12_17;
                        }
                        if ((var2_6 = var12_17) > 0) {
                            var1_2.applyHpDelta(-100);
                        }
                    }
                    var11_16 = var7_9;
                    var10_10 = var1_2.playerCellY;
                    var9_11 = var1_2.playerCellX;
                    var8_13 = var1_2;
                    var12_17 = 0;
                    var7_9 = 0;
                    if (var11_16 == 0) {
                        var7_9 = var8_13.m_100(var9_11 - 1, var10_10, 60);
                        if (var9_11 > 0 && var7_9 >= 0) {
                            ++var12_17;
                        }
                        var7_9 = var8_13.m_100(var9_11 + 1, var10_10, 60);
                        if (var9_11 < var8_13.mapCellsHigh - 1 && var7_9 >= 0) {
                            ++var12_17;
                        }
                        var7_9 = var8_13.m_100(var9_11, var10_10 - 1, 60);
                        if (var10_10 > 0 && var7_9 >= 0) {
                            ++var12_17;
                            var8_13.m_101(var9_11, var10_10 - 1, var7_9);
                        }
                        var7_9 = var8_13.m_100(var9_11, var10_10 + 1, 60);
                        if (var10_10 < var8_13.mapCellsHigh - 1 && var7_9 >= 0) {
                            ++var12_17;
                        }
                    } else {
                        var7_9 = var8_13.m_100(var9_11, var10_10 - 1, 60);
                        if (var10_10 > 0 && var7_9 >= 0) {
                            var8_13.m_101(var9_11, var10_10 - 1, var7_9);
                        }
                    }
                    if ((var2_6 = var12_17) > 0) {
                        var1_2.applyHpDelta(-200);
                        var6_8 = 1;
                    }
                    var4_18 = var6_8;
                    if (this.f_byte_arr_10[var3_7] <= 0) break block59;
                    var5_19 = this.f_byte_arr_10[var3_7];
                    while (--var5_19 >= 0) {
                        block60: {
                            var1_3 = this.f_byte_arr2_03[var3_7][var5_19] - 1;
                            var2_6 = this.entityType[var1_3];
                            switch (this.f_byte_arr_03[var2_6]) {
                                case 2: 
                                case 4: {
                                    this.pickupItemType(var2_6);
                                    this.m_047(var1_3);
                                    break;
                                }
                                case 1: {
                                    switch (var2_6) {
                                        case 83: {
                                            if (this.tryRunScene(this.entityParam[var1_3], true)) {
                                                this.f_bool_05 = false;
                                                this.loadLevelScript(this.entityParam[var1_3]);
                                                var4_18 = 1;
                                                var5_19 = 0;
                                            }
                                            break block60;
                                        }
                                        case 7: {
                                            var1_3 = this.entityParam[var1_3] & 255;
                                            this.changeFloor(var1_3, false, false);
                                            this.f_int_06 = 0;
                                            var5_19 = 0;
                                            break block60;
                                        }
                                        case 8: {
                                            var1_3 = this.entityParam[var1_3] & 255;
                                            this.changeFloor(var1_3, true, false);
                                            this.f_int_06 = 0;
                                            var5_19 = 0;
                                            break block60;
                                        }
                                        case 5: {
                                            var1_3 = this.entityParam[var1_3];
                                            this.m_050(this.playerCellX, this.playerCellY, var5_19);
                                            if (this.m_073(var1_3)) {
                                                this.f_byte_11 = (byte)4;
                                            }
                                            break block60;
                                        }
                                        case 76: {
                                            var1_3 = (this.entityParam[var1_3] & 255) + 1;
                                            this.m_050(this.playerCellX, this.playerCellY, var5_19);
                                            var2_6 = var1_3;
                                            var1_4 = this;
                                            var6_8 = 0;
                                            var1_4.f_int_86 = 0;
                                            var7_9 = 0;
                                            for (var8_14 = 0; var8_14 < var1_4.f_int_45; ++var8_14) {
                                                if (var1_4.f_bool_arr_01[var8_14]) continue;
                                                var7_9 = var1_4.entityParam[var8_14] & 255;
                                                var6_8 = var1_4.entityType[var8_14];
                                                if (var6_8 == 76) {
                                                    if (var7_9 + 1 != var2_6) continue;
                                                    var11_16 = var1_4.entityPixelY[var8_14] >> 5;
                                                    var10_10 = var1_4.entityPixelX[var8_14] >> 5;
                                                    var9_12 = var1_4;
                                                    var12_17 = var9_12.f_byte_arr2_02[var11_16][var10_10];
                                                    var7_9 = var9_12.f_byte_arr_10[var12_17];
                                                    if (var9_12.f_byte_arr_10[var12_17] <= 0) continue;
                                                    var6_8 = var7_9;
                                                    while (--var6_8 >= 0) {
                                                        var9_12.f_bool_arr_01[var9_12.f_byte_arr2_03[var12_17][var6_8] - 1] = true;
                                                    }
                                                    var9_12.f_byte_arr_10[var12_17] = 0;
                                                    continue;
                                                }
                                                if (var6_8 != 4 || var7_9 != var2_6) continue;
                                                var1_4.f_short_arr_03[var1_4.f_int_86] = var8_14;
                                                var1_4.f_short_arr_01[var1_4.f_int_86] = var1_4.entityPixelX[var8_14] >> 5;
                                                var1_4.f_short_arr_02[var1_4.f_int_86] = var1_4.entityPixelY[var8_14] >> 5;
                                                ++var1_4.f_int_86;
                                            }
                                            if (var1_4.f_int_86 > 0) {
                                                this.f_byte_11 = (byte)4;
                                            }
                                            break block60;
                                        }
                                        case 6: {
                                            this.f_bool_07 = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if (this.f_byte_11 != 4) {
                        this.f_byte_11 = 0;
                        if (var4_18 == 0) {
                            this.m_027();
                        }
                    }
                    break block61;
                }
                if (this.f_bool_27) {
                    if (var4_18 == 0) {
                        this.m_032();
                    } else {
                        this.f_byte_11 = 0;
                    }
                } else {
                    this.f_byte_11 = 0;
                    if (var4_18 == 0) {
                        this.m_027();
                    }
                }
            }
            if (!this.f_bool_26) break block62;
            if (this.f_int_148 <= 0) ** GOTO lbl163
            if (this.f_byte_arr_42[this.f_int_148 - 1] == this.f_byte_12) {
                --this.f_int_148;
            } else {
                this.f_int_148 = 0;
lbl163:
                // 2 sources

                this.f_bool_26 = false;
            }
        }
        var1_5 = this;
        var2_6 = var1_5.playerCellX;
        var6_8 = var1_5.playerCellY;
        var7_9 = var1_5.f_int_70 >> 5;
        var8_15 = var1_5.f_int_72 >> 5;
        var9_11 = var1_5.f_int_71 >> 5;
        var10_10 = var1_5.f_int_73 >> 5;
        var1_5.f_bool_13 = false;
        if (var1_5.f_bool_12) {
            if (var6_8 > 0) {
                if (var2_6 == var7_9 && var6_8 - 1 == var8_15) {
                    var1_5.f_bool_13 = true;
                } else if (var2_6 == var9_11 && var6_8 - 1 == var10_10) {
                    var1_5.f_bool_13 = true;
                }
            }
            if (var6_8 < var1_5.mapCellsHigh - 1) {
                if (var2_6 == var7_9 && var6_8 + 1 == var8_15) {
                    var1_5.f_bool_13 = true;
                } else if (var2_6 == var9_11 && var6_8 + 1 == var10_10) {
                    var1_5.f_bool_13 = true;
                }
            }
            if (var2_6 > 0) {
                if (var2_6 - 1 == var7_9 && var6_8 == var8_15) {
                    var1_5.f_bool_13 = true;
                } else if (var2_6 - 1 == var9_11 && var6_8 == var10_10) {
                    var1_5.f_bool_13 = true;
                }
            }
            if (var2_6 < var1_5.mapCellsWide - 1) {
                if (var2_6 + 1 == var7_9 && var6_8 == var8_15) {
                    var1_5.f_bool_13 = true;
                    return;
                }
                if (var2_6 + 1 == var9_11 && var6_8 == var10_10) {
                    var1_5.f_bool_13 = true;
                }
            }
        }
    }

    private void m_031(int n, int n2) {
        if (this.isCellWalkable(n, n2 - 1)) {
            --n2;
        } else if (this.isCellWalkable(n, n2 + 1)) {
            ++n2;
        } else if (this.isCellWalkable(n - 1, n2)) {
            --n;
        } else if (this.isCellWalkable(n + 1, n2)) {
            ++n;
        }
        this.m_024(n, n2);
        this.m_064((this.f_int_58 - 32 >> 1) - this.playerPixelX, (this.f_int_59 - 32 >> 1) - this.playerPixelY);
    }

    private void m_032() {
        if (this.f_int_148 > 0) {
            this.f_byte_12 = this.f_byte_arr_42[--this.f_int_148];
            boolean bl = this.tryStep(this.f_byte_12);
            if (this.f_int_148 == 0) {
                this.f_bool_27 = false;
                return;
            }
            if (!bl) {
                this.f_bool_27 = false;
                if (this.f_byte_11 != 5) {
                    this.f_byte_11 = 0;
                    this.f_int_148 = 0;
                }
            }
        }
    }

    private void m_033(int n, int n2) {
        int n3 = n + this.playerPixelX;
        int n4 = n2 + this.playerPixelY;
        int n5 = 0;
        if (this.f_bool_07) {
            n4 -= this.f_int_46;
        }
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], n3 + 8, n4 + 22, 0);
        switch (this.f_byte_12) {
            case 1: {
                this.m_002(this.f_Image_arr2_00[3][0], n3 - 8, n4 - 14, this.f_int_arr_03[this.f_int_38] * 41, 46, 41, 46);
                if (this.f_byte_11 != 2) break;
                n5 = 4;
                while (--n5 >= 0) {
                    this.m_002(this.f_Image_02, n + this.f_int_arr_04[n5] - 8, n2 + this.f_int_arr_05[n5] - 14, this.f_int_arr_03[this.f_int_38] * 41, 46, 41, 46);
                }
                break;
            }
            case 0: {
                if (this.f_byte_11 == 2) {
                    n5 = 4;
                    while (--n5 >= 0) {
                        this.m_002(this.f_Image_02, n + this.f_int_arr_04[n5] - 4, n2 + this.f_int_arr_05[n5] - 14, this.f_int_arr_03[this.f_int_38] * 41, 0, 41, 46);
                    }
                }
                this.m_002(this.f_Image_arr2_00[3][0], n3 - 3, n4 - 18, this.f_int_arr_03[this.f_int_38] * 41, 0, 41, 46);
                break;
            }
            case 3: {
                if (this.f_byte_11 == 2) {
                    n5 = 4;
                    while (--n5 >= 0) {
                        this.m_005(this.f_Image_02, n + this.f_int_arr_04[n5], n2 + this.f_int_arr_05[n5] - 14, this.f_int_arr_03[this.f_int_38] * 41, 92, 41, 46, 1);
                    }
                }
                this.m_005(this.f_Image_arr2_00[3][0], n3, n4 - 16, this.f_int_arr_03[this.f_int_38] * 41, 92, 41, 46, 1);
                break;
            }
            case 2: {
                if (this.f_byte_11 == 2) {
                    n5 = 4;
                    while (--n5 >= 0) {
                        this.m_002(this.f_Image_02, n + this.f_int_arr_04[n5] - 6, n2 + this.f_int_arr_05[n5] - 14, this.f_int_arr_03[this.f_int_38] * 41, 92, 41, 46);
                    }
                }
                this.m_002(this.f_Image_arr2_00[3][0], n3 - 8, n4 - 16, this.f_int_arr_03[this.f_int_38] * 41, 92, 41, 46);
            }
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        if (this.f_bool_26) {
            n3 = (this.playerCellX << 5) + n;
            n4 = (this.playerCellY << 5) + n2;
            n5 = this.f_int_148;
            while (--n5 >= 0) {
                this.f_Graphics_00.setColor(136);
                switch (this.f_byte_arr_42[n5]) {
                    case 1: {
                        this.m_002(this.f_Image_arr2_00[8][22], n3 + 5, (n4 -= 32) + 5 + this.f_int_46, 0, 0, 21, 22);
                        break;
                    }
                    case 0: {
                        this.m_002(this.f_Image_arr2_00[8][22], n3 + 5, (n4 += 32) + 5 + this.f_int_46, 21, 0, 21, 22);
                        break;
                    }
                    case 3: {
                        this.m_002(this.f_Image_arr2_00[8][22], (n3 -= 32) + 5 + this.f_int_46, n4 + 5, 42, 0, 21, 22);
                        break;
                    }
                    case 2: {
                        this.m_005(this.f_Image_arr2_00[8][22], (n3 += 32) + 5 + this.f_int_46, n4 + 5, 42, 0, 21, 22, 1);
                    }
                }
            }
        }
        n3 = n + this.playerPixelX;
        n4 = n2 + this.playerPixelY;
        switch (this.f_byte_11) {
            case 2: {
                n5 = this.f_int_46;
                this.m_002(this.f_Image_arr2_00[8][23], 5 - n5, 148, 44, 0, 22, 24);
                this.m_002(this.f_Image_arr2_00[8][23], 109, 25 - n5, 0, 0, 22, 24);
                this.m_002(this.f_Image_arr2_00[8][23], 109, this.f_int_48 - 29 + n5, 22, 0, 22, 24);
                this.m_005(this.f_Image_arr2_00[8][23], 215 + n5, 148, 44, 0, 22, 24, 1);
                return;
            }
            case 3: {
                n2 = n4 - 40;
                n = n3 + 16;
                n3 = 0;
                n4 = 0;
                if (this.f_int_96 < this.itemStackSize) {
                    n3 = this.itemStackTypes[this.f_int_96];
                    this.f_Graphics_00.drawImage(this.f_Image_arr_00[n3], n - 16, n2 + 4, 0);
                    if (this.f_int_96 > 0) {
                        this.f_Graphics_00.setColor(-1);
                        this.m_093(n - 18 - (this.f_int_03 & 1), n2 + 16, (byte)3);
                    }
                    if (this.f_int_96 < this.itemStackSize - 1) {
                        this.f_Graphics_00.setColor(-1);
                        this.m_093(n + 18 + (this.f_int_03 & 1), n2 + 16, (byte)3);
                    }
                    this.f_Graphics_00.setColor(-1);
                    n4 = this.f_Font_00.stringWidth(this.objectTypeNames[n3]);
                    this.f_Graphics_00.fillRect((n -= n4 >> 1) - 5, n2 -= this.f_int_01 + 4, n4 + 10, this.f_int_01 + 4);
                    this.f_Graphics_00.setColor(0);
                    this.f_Graphics_00.drawString(this.objectTypeNames[n3], n, n2 + 2, 0);
                    return;
                }
                this.f_Graphics_00.setColor(-1);
                this.f_Graphics_00.fillRect((n -= this.f_int_00 >> 1) - 5, n2 -= this.f_int_01 + 4 - 32, this.f_int_00 + 10, this.f_int_01 + 4);
                this.f_Graphics_00.setColor(0);
                this.f_Graphics_00.drawString("\u65e0", n, n2 + 2, 0);
            }
        }
    }

    private void m_034() {
        if (this.f_byte_13 != 0) {
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][10], 0, 302, 0);
            this.m_002(this.f_Image_arr2_00[8][11], 2, 307, (this.f_byte_13 - 1) * 12, 0, 12, 10);
        }
        if (this.f_byte_14 != 0) {
            this.m_004(this.f_Image_arr2_00[8][10], 222, 302, 1);
            this.m_002(this.f_Image_arr2_00[8][11], 226, 307, (this.f_byte_14 - 1) * 12, 0, 12, 10);
        }
    }

    private void m_035(int n, int n2) {
        n = n2;
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][3], 0, n, 0);
        if (this.currentFloor > 50) {
            this.f_Graphics_00.setColor(-1);
            this.m_142("\u5f15\u5b50", 32, n + 9 + (19 - this.f_int_01 >> 1), 17, this.f_int_arr_36);
            n += 4;
        } else {
            this.m_042(this.f_Image_arr2_00[8][18], this.currentFloor, 40, (n += 4) + 2);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][8], 41, n + 10, 0);
        }
        this.f_Graphics_00.setColor(2435368);
        this.f_Graphics_00.fillRect(65, n2, 175, 18);
        this.m_036(0, this.yellowKeyCount, 65, n -= 4);
        this.m_036(1, this.blueKeyCount, 107, n);
        this.m_036(2, this.redKeyCount, 149, n);
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][1], 191, n += 3, 0);
        this.m_042(this.f_Image_arr2_00[8][2], this.goldAmount, 237, n + 2);
    }

    private void m_036(int n, int n2, int n3, int n4) {
        this.f_Graphics_00.setClip(n3, n4, 18, 16);
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][6], n3 - n * 18, n4, 0);
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        this.m_042(this.f_Image_arr2_00[8][2], n2, n3 += 40, n4 += 5);
    }

    private void m_037(int n, int n2) {
        this.m_039(0, n2, 240, 50);
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[3][1], 40 - (this.f_Image_arr2_00[3][1].getWidth() >> 1), n2 + 50 - this.f_Image_arr2_00[3][1].getHeight(), 0);
        int n3 = n = 83;
        int n4 = n2 += 13;
        this.f_Graphics_00.setClip(n3, n4, 10, 10);
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][7], n3, n4 - 2, 0);
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        this.m_041(n3 += 16, n4 + 1, 58, 11);
        this.m_042(this.f_Image_arr2_00[8][2], this.playerHp, n3 + 52, n4 + 2);
        n3 = n;
        this.m_002(this.f_Image_arr2_00[8][7], n3, n4 += 12, 10, 0, 10, 13);
        this.f_Graphics_00.setColor(512);
        this.m_041(n3 += 16, n4 + 1, 58, 11);
        this.m_042(this.f_Image_arr2_00[8][2], this.playerAtk, n3 + 52, n4 + 2);
        n3 = n;
        this.m_002(this.f_Image_arr2_00[8][7], n3, n4 += 12, 20, 0, 10, 13);
        this.f_Graphics_00.setColor(512);
        this.m_041(n3 += 16, n4 + 1, 58, 11);
        this.m_042(this.f_Image_arr2_00[8][2], this.playerDef, n3 + 52, n4 + 2);
        n4 = n2 + 2;
        this.m_041(n3 += 60, n4, 32, 32);
        if (this.equippedWeaponType == 0) {
            this.f_Graphics_00.setColor(-1);
            this.f_Graphics_00.drawString(this.f_String_arr_04[this.equippedWeaponType], n3 + (32 - this.f_int_00 >> 1), n4 + (32 - this.f_int_01 >> 1), 0);
        } else {
            this.f_Graphics_00.drawImage(this.f_Image_arr_00[this.equippedWeaponType], n3 + (32 - this.f_Image_arr_00[this.equippedWeaponType].getWidth() >> 1), n4 + (32 - this.f_Image_arr_00[this.equippedWeaponType].getHeight() >> 1), 0);
        }
        this.m_041(n3 += 34, n4, 32, 32);
        if (this.equippedArmorType == 0) {
            this.f_Graphics_00.setColor(-1);
            this.f_Graphics_00.drawString(this.f_String_arr_04[this.equippedArmorType], n3 + (32 - this.f_int_00 >> 1), n4 + (32 - this.f_int_01 >> 1), 0);
            return;
        }
        this.f_Graphics_00.drawImage(this.f_Image_arr_00[this.equippedArmorType], n3 + (32 - this.f_Image_arr_00[this.equippedArmorType].getWidth() >> 1), n4 + (32 - this.f_Image_arr_00[this.equippedArmorType].getHeight() >> 1), 0);
    }

    private void m_038(int n, int n2, int n3, int n4, int n5) {
        this.m_040(n2, n3, n4, n5);
        n3 -= 2;
        n2 += 2;
        int[] nArray = this.f_int_arr2_00[n];
        n4 = nArray.length;
        n5 = 0;
        while (n5 < n4) {
            this.f_Graphics_00.setClip(n2, n3, 14, 16);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][13], n2 - nArray[n5] * 14, n3, 0);
            ++n5;
            n2 += 14;
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
    }

    private void m_039(int n, int n2, int n3, int n4) {
        int n5 = n;
        int n6 = n2;
        this.f_Graphics_00.setClip(n5, n6, 26, 16);
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n5, n6, 0);
        n5 += 26;
        while (n5 < n + n3 - 26) {
            this.f_Graphics_00.setClip(n5, n6, 16, 16);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n5 - 26, n6, 0);
            n5 += 16;
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        this.m_005(this.f_Image_arr2_00[8][0], n + n3 - 26, n6, 0, 0, 26, 16, 1);
        n5 = n + 11;
        this.f_Graphics_00.setColor(2699825);
        this.f_Graphics_00.fillRect(n5, n6 += 16, n3 - 22, n4 - 16);
        n5 = n + n3 - 11;
        while (n6 < n2 + n4) {
            this.f_Graphics_00.setClip(n, n6, 11, 16);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n - 42, n6, 0);
            this.f_Graphics_00.setClip(n5, n6, 11, 16);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n5 - 53, n6, 0);
            n6 += 16;
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
    }

    private void m_040(int n, int n2, int n3, int n4) {
        int n5 = n;
        int n6 = n2;
        this.f_Graphics_00.setClip(n5, n6, 26, 16);
        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n5, n6, 0);
        n5 += 26;
        while (n5 < n + n3 - 26) {
            this.f_Graphics_00.setClip(n5, n6, 16, 16);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n5 - 26, n6, 0);
            this.m_005(this.f_Image_arr2_00[8][0], n5, n6 + n4 - 16, 26, 0, 16, 16, 2);
            n5 += 16;
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        this.m_005(this.f_Image_arr2_00[8][0], n + n3 - 26, n6, 0, 0, 26, 16, 1);
        n5 = n + 11;
        this.f_Graphics_00.setColor(2699825);
        this.f_Graphics_00.fillRect(n5, n6 += 16, n3 - 22, n4 - 32);
        n5 = n + n3 - 11;
        while (n6 < n2 + n4 - 16) {
            this.f_Graphics_00.setClip(n, n6, 11, 16);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n - 42, n6, 0);
            this.f_Graphics_00.setClip(n5, n6, 11, 16);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], n5 - 53, n6, 0);
            n6 += 16;
        }
        n6 = n2 + n4 - 16;
        this.m_005(this.f_Image_arr2_00[8][0], n, n6, 0, 0, 26, 16, 2);
        this.m_005(this.f_Image_arr2_00[8][0], n + n3 - 26, n6, 0, 0, 26, 16, 3);
        this.f_Graphics_00.setClip(0, 0, 240, 320);
    }

    private void m_041(int n, int n2, int n3, int n4) {
        int n5 = n2 + n4 - 2;
        int n6 = n + n3 - 1;
        this.f_Graphics_00.setColor(4803902);
        this.f_Graphics_00.fillRect(n + 1, n2, n3 - 2, n4 - 1);
        this.f_Graphics_00.drawLine(n, n2 + 1, n, n5);
        this.f_Graphics_00.setColor(1645850);
        this.f_Graphics_00.drawLine(n6, n2 + 1, n6, n5);
        this.f_Graphics_00.drawLine(n + 1, n5 + 1, n6 - 1, n5 + 1);
    }

    private int m_042(Image image, int n, int n2, int n3) {
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
            this.f_Graphics_00.setClip(n2 -= n4 + 1, n3, n4, n5);
            this.f_Graphics_00.drawImage(image, n2 - n6 * n4, n3, 0);
            ++n7;
        } while ((n /= 10) > 0);
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        if (bl) {
            this.f_Graphics_00.setColor(15027533);
            this.f_Graphics_00.drawLine(n2 - n4 + 1, n3 + 3, n2 - 1, n3 + 3);
            ++n7;
        }
        return n7;
    }

    private void m_043() {
        this.f_Image_arr_00 = new Image[this.objectTypeNames.length];
        this.f_byte_arr_03 = new byte[this.objectTypeNames.length];
        this.f_bool_arr_02 = new boolean[this.objectTypeNames.length];
        int n = this.f_bool_arr_03.length;
        while (--n >= 0) {
            this.f_bool_arr_02[n] = this.f_bool_arr_03[n];
        }
        this.f_byte_arr_05 = new byte[this.objectTypeNames.length];
        this.f_int_arr_13 = new int[this.objectTypeNames.length];
        this.entityPixelX = new int[100];
        this.entityPixelY = new int[100];
        this.f_int_arr_08 = new int[100];
        this.f_int_arr_09 = new int[100];
        this.f_int_arr_10 = new int[100];
        this.entityType = new int[100];
        this.f_int_arr_12 = new int[100];
        this.f_bool_arr_00 = new boolean[100];
        this.f_bool_arr_01 = new boolean[100];
        this.entityParam = new short[100];
        this.f_byte_arr_04 = new byte[100];
        this.f_byte_arr_05[74] = 1;
        this.f_byte_arr_05[53] = 1;
        this.f_byte_arr_05[49] = 1;
        this.f_byte_arr_05[69] = 2;
        for (n = 1; n <= 12; ++n) {
            this.f_byte_arr_03[n] = 1;
        }
        for (n = 13; n <= 32; ++n) {
            this.f_byte_arr_03[n] = 2;
        }
        for (n = 33; n <= 40; ++n) {
            this.f_byte_arr_03[n] = 4;
        }
        for (n = 41; n < 79; ++n) {
            this.f_byte_arr_03[n] = 8;
        }
        this.f_byte_arr_03[76] = 1;
        this.f_byte_arr_03[77] = 16;
        this.f_byte_arr_03[78] = 16;
        this.f_byte_arr_03[79] = 4;
        this.f_byte_arr_03[80] = 4;
        this.f_byte_arr_03[81] = 1;
        this.f_byte_arr_03[83] = 1;
        this.f_byte_arr_03[72] = 32;
        this.f_byte_arr_03[84] = 32;
        this.f_byte_arr_03[87] = 32;
        this.f_byte_arr_03[85] = 2;
        this.f_byte_arr_03[86] = 2;
        this.f_byte_arr_03[82] = 1;
    }

    private void m_044(int n, int n2) {
        Image image = this.f_Image_arr_00[n];
        switch (n) {
            case 69: {
                this.m_001(9);
                this.f_int_arr_08[n2] = 96;
                this.f_int_arr_09[n2] = 32;
                return;
            }
            case 67: {
                this.m_001(9);
                break;
            }
            case 72: {
                this.f_int_arr_08[n2] = 32;
                this.f_int_arr_09[n2] = 45;
                return;
            }
            default: {
                if (image == null) break;
                this.f_int_arr_08[n2] = this.f_byte_arr_07[n];
                this.f_int_arr_09[n2] = image.getHeight();
                return;
            }
        }
        this.f_int_arr_08[n2] = 32;
        this.f_int_arr_09[n2] = 32;
    }

    private void m_045(int n, int n2) {
        this.f_int_arr_12[n2] = this.f_byte_arr_06[n];
        this.f_int_arr_10[n2] = 0;
    }

    private void m_046(int n) {
        this.f_byte_arr_04[n] = 2;
        int n2 = this.entityType[n];
        switch (this.f_byte_arr_03[n2]) {
            case 1: {
                return;
            }
            case 2: 
            case 8: 
            case 16: 
            case 32: {
                this.f_int_arr_08[n] = 27;
                this.f_int_arr_09[n] = 29;
                this.f_int_arr_12[n] = 8;
                this.f_int_arr_10[n] = 0;
            }
        }
    }

    private void m_047(int n) {
        this.f_byte_arr_04[n] = 1;
        int n2 = this.entityType[n];
        switch (this.f_byte_arr_03[n2]) {
            case 1: {
                switch (n2) {
                    case 1: 
                    case 2: 
                    case 3: {
                        this.f_int_arr_08[n] = 45;
                        this.f_int_arr_09[n] = 56;
                        this.f_int_arr_12[n] = 6;
                        this.f_int_arr_10[n] = 0;
                        break;
                    }
                    case 11: {
                        this.f_int_arr_08[n] = 45;
                        this.f_int_arr_09[n] = 57;
                        this.f_int_arr_12[n] = 2;
                        this.f_int_arr_10[n] = 0;
                    }
                }
                this.m_057();
                return;
            }
            case 2: 
            case 4: {
                this.f_int_arr_08[n] = 45;
                this.f_int_arr_09[n] = 56;
                this.f_int_arr_12[n] = 10;
                this.f_int_arr_10[n] = 0;
                return;
            }
            case 8: 
            case 16: 
            case 32: {
                this.f_int_arr_08[n] = 27;
                this.f_int_arr_09[n] = 29;
                this.f_int_arr_12[n] = 6;
                this.f_int_arr_10[n] = 0;
            }
        }
    }

    private int m_048(int n, int n2, int n3, int n4) {
        this.m_044(n, this.f_int_45);
        this.m_045(n, this.f_int_45);
        int n5 = this.f_int_arr_08[this.f_int_45] >> 5;
        byte by = 0;
        int n6 = n2 + 16 >> 5;
        int n7 = n3 + 16 >> 5;
        int n8 = 0;
        if (n5 <= 0) {
            n5 = 1;
        }
        n2 = n6 << 5;
        n3 = n7 << 5;
        this.entityType[this.f_int_45] = n;
        this.entityPixelX[this.f_int_45] = n2;
        this.entityPixelY[this.f_int_45] = n3;
        this.f_bool_arr_01[this.f_int_45] = false;
        this.f_bool_arr_00[this.f_int_45] = true;
        this.entityParam[this.f_int_45] = n4;
        this.f_byte_arr_04[this.f_int_45] = 0;
        ++this.f_int_45;
        for (n4 = 0; n4 < n5; ++n4) {
            by = this.f_byte_arr2_02[n7][n6 + n8];
            if (by == 0) {
                this.f_byte_arr2_02[n7][n6 + n8] = this.f_byte_15 = (byte)(this.f_byte_15 + 1);
                by = this.f_byte_15;
            }
            this.f_byte_arr2_03[by][this.f_byte_arr_10[by]] = this.f_int_45;
            byte by2 = by;
            this.f_byte_arr_10[by2] = this.f_byte_arr_10[by2] + 1;
            ++n8;
        }
        if (n == 7) {
            this.f_int_70 = n2;
            this.f_int_72 = n3;
        } else if (n == 8) {
            this.f_int_71 = n2;
            this.f_int_73 = n3;
        }
        return this.f_int_45 - 1;
    }

    private void m_049(int n, int n2, int n3) {
        n = this.f_byte_arr2_02[n2][n];
        n2 = this.f_byte_arr_10[n];
        byte by = 0;
        if (n2 > 0) {
            for (int i = 0; i < n2; ++i) {
                if (this.entityType[this.f_byte_arr2_03[n][i] - 1] != n3) {
                    byte by2 = by;
                    by = (byte)(by + 1);
                    this.f_byte_arr2_03[n][by2] = this.f_byte_arr2_03[n][i];
                    continue;
                }
                this.f_bool_arr_01[this.f_byte_arr2_03[n][i] - 1] = true;
            }
            this.f_byte_arr_10[n] = by;
        }
    }

    private void m_050(int n, int n2, int n3) {
        n = this.f_byte_arr2_02[n2][n];
        n2 = this.f_byte_arr_10[n];
        if (this.f_byte_arr_10[n] > 0) {
            this.f_bool_arr_01[this.f_byte_arr2_03[n][n3] - 1] = true;
            while (n3 < n2 - 1) {
                this.f_byte_arr2_03[n][n3] = this.f_byte_arr2_03[n][n3 + 1];
                ++n3;
            }
            int n4 = n;
            this.f_byte_arr_10[n4] = this.f_byte_arr_10[n4] - 1;
        }
    }

    private void m_051(int n) {
        byte by = this.f_byte_arr2_02[this.entityPixelY[n] >> 5][this.entityPixelX[n] >> 5];
        int n2 = this.f_byte_arr_10[by];
        this.f_bool_arr_01[n] = true;
        if (n2 > 0) {
            for (int i = 0; i < n2; ++i) {
                if (this.f_byte_arr2_03[by][i] - 1 != n) continue;
                for (n = i; n < n2 - 1; ++n) {
                    this.f_byte_arr2_03[by][n] = this.f_byte_arr2_03[by][n + 1];
                }
                byte by2 = by;
                this.f_byte_arr_10[by2] = this.f_byte_arr_10[by2] - 1;
                return;
            }
        }
    }

    private void m_052() {
        if (this.f_bool_09) {
            if (++this.f_int_46 > 1) {
                this.f_bool_09 = false;
                return;
            }
        } else if (--this.f_int_46 < -1) {
            this.f_bool_09 = true;
        }
    }

    /*
     * Enabled aggressive block sorting
     */
    private void m_053(int n, int n2, boolean bl) {
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
        this.m_052();
        bl = false;
        block12: for (int i = 0; i < this.f_int_45; ++i) {
            if (this.f_bool_arr_01[i] || !this.f_bool_arr_00[i]) continue;
            n8 = this.entityPixelX[i];
            n7 = this.entityPixelY[i];
            n6 = this.f_int_arr_08[i];
            n5 = this.f_int_arr_09[i];
            n10 = this.f_byte_arr_04[i];
            n4 = n + n8;
            n3 = n2 + n7;
            if (n4 < -n6 || n4 > this.f_int_58 || n3 < -12 || n3 > 20 + this.f_int_59) continue;
            n9 = this.entityType[i];
            Object object = this.f_Image_arr_00[n9];
            if (object != null) {
                by = this.f_byte_arr_03[n9];
                if (!bl && this.playerPixelX > n8 - 32 && this.playerPixelX < n8 + 32 && this.playerPixelY >= n7 - 32 && n7 > this.playerPixelY && !this.f_bool_arr_02[n9]) {
                    this.m_033(n, n2);
                    bl = true;
                }
                n8 = n6 * this.f_int_arr2_02[this.f_int_arr_12[i]][this.f_int_arr_10[i]];
                n7 = 0;
                switch (by) {
                    case 1: {
                        if (n9 == 6) {
                            n3 += 5 - this.f_int_46;
                        } else if (n9 == 9) {
                            n4 += 32;
                        }
                        if (this.f_byte_arr_04[i] != 1) {
                            this.m_002((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                            continue block12;
                        }
                        this.m_002(this.f_Image_arr2_00[4][12], n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                        continue block12;
                    }
                    case 4: {
                        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], n4 + 8, n3 + 22, 0);
                        this.m_002((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 24) + this.f_int_46, n8, 0, n6, n5);
                        continue block12;
                    }
                    case 16: {
                        if (n10 == 1 || n10 == 2) {
                            this.m_002(this.f_Image_arr2_00[2][9], n4 + 2, n3 + 2 - (this.f_int_arr_10[i] << 3), n8, 0, 27, 29);
                            continue block12;
                        }
                        this.f_Graphics_00.drawImage(this.f_Image_arr_00[15], n4 + 1, n3 + 8 + this.f_int_46, 0);
                        this.m_002((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 24) + this.f_int_46, n8, 0, n6, n5);
                        continue block12;
                    }
                    case 32: {
                        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], n4 + 8, n3 + 22, 0);
                        if (n10 == 1 || n10 == 2) {
                            this.m_002(this.f_Image_arr2_00[2][9], n4 + 2, n3 + 2 - (this.f_int_arr_10[i] << 3), n8, 0, 27, 29);
                            continue block12;
                        }
                        if (n9 != 72) {
                            this.m_002((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 16) + this.f_int_46, n8, 0, n6, n5);
                            continue block12;
                        }
                        if (i != this.f_int_127) {
                            this.m_002((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                            continue block12;
                        }
                        n7 = n5 * this.f_byte_19;
                        if (this.f_byte_19 == 3) {
                            this.m_005((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, n7 -= n5, n6, n5, 1);
                            continue block12;
                        }
                        this.m_002((Image)object, n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, n7, n6, n5);
                        continue block12;
                    }
                    case 8: {
                        if (n10 == 3) {
                            n4 += this.randomBelow(5) - 2;
                            n3 += this.randomBelow(5) - 2;
                        }
                        if (n10 == 1 || n10 == 2) {
                            this.m_002(this.f_Image_arr2_00[2][9], n4 + 2, n3 + 2 - (this.f_int_arr_10[i] << 3), n8, 0, 27, 29);
                        } else if (n9 == 67 || n9 == 69) {
                            n6 = n3;
                            n7 = n4;
                            n8 = i;
                            object = this;
                            n5 = ((a)object).entityType[n8];
                            n8 = ((a)object).f_int_arr2_02[((a)object).f_int_arr_12[n8]][((a)object).f_int_arr_10[n8]];
                            switch (n5) {
                                case 67: {
                                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[5][0], n7 + 8, n6 + 22, 0);
                                    int n11 = n7 + 16;
                                    int n12 = n6 + 16;
                                    n6 = n8;
                                    n7 = n12;
                                    n8 = n11;
                                    byte[] byArray = ((a)object).f_byte_arr2_01[n6];
                                    byte[] byArray2 = null;
                                    for (n10 = 0; n10 < byArray.length; n10 += 4) {
                                        byArray2 = ((a)object).f_byte_arr2_00[byArray[n10]];
                                        by = byArray2[0];
                                        super.m_002(((a)object).f_Image_arr2_00[9][by], n8 + byArray[n10 + 1], n7 + byArray[n10 + 2], byArray2[1], byArray2[2], byArray2[3], byArray2[4]);
                                    }
                                    break;
                                }
                                case 69: {
                                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr2_00[5][0], n7 + 40, n6 + 22, 0);
                                    n6 -= 49;
                                    ((a)object).f_Graphics_00.drawImage(((a)object).f_Image_arr_00[n5], n7 += 3, n6 += ((a)object).f_int_46, 0);
                                    if (n8 > 0) {
                                        super.m_002(((a)object).f_Image_arr2_00[9][5], n7 + 32, n6 + 27, 26 * (n8 - 1), 0, 26, 13);
                                    } else {
                                        break;
                                    }
                                }
                            }
                        } else {
                            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], n4 + 8, n3 + 22, 0);
                            this.m_002((Image)object, n4 + (32 - n6 >> 1), (n3 -= 4) - (n5 - 32), n8, 0, n6, n5);
                        }
                        if (this.f_byte_11 != 2 || !this.f_bool_08) break;
                        int n13 = this.f_int_arr_13[n9];
                        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[2][8], n4 - 3 + this.f_byte_arr_01[this.f_int_03 & 7], n3 - 24 + this.f_byte_arr_02[this.f_int_03 & 7], 0);
                        if (n13 >= 0) {
                            this.m_127(this.f_Image_arr2_00[2][7], this.f_int_arr_13[n9], n4 + 30 + this.f_byte_arr_01[this.f_int_03 & 7], n3 - 17 + this.f_byte_arr_02[this.f_int_03 & 7]);
                            continue block12;
                        }
                        this.m_002(this.f_Image_arr2_00[2][7], n4 + 14 + this.f_byte_arr_01[this.f_int_03 & 7], n3 - 17 + this.f_byte_arr_02[this.f_int_03 & 7], 70, 0, 7, 9);
                        continue block12;
                    }
                    case 2: {
                        if (n10 == 1) {
                            this.m_002(this.f_Image_arr2_00[4][12], n4 + (32 - n6 >> 1), n3 - (n5 - 32), n8, 0, n6, n5);
                            continue block12;
                        }
                        if (n10 == 2) {
                            this.m_002(this.f_Image_arr2_00[2][9], n4 + 2, n3 + 2 - (this.f_int_arr_10[i] << 3), n8, 0, 27, 29);
                            continue block12;
                        }
                        if (n9 < 26) {
                            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], n4 + 8, n3 + 22, 0);
                            this.f_Graphics_00.drawImage(this.f_Image_arr_00[n9], n4 + (32 - n6 >> 1), n3 - (n5 - 24) + this.f_int_46, 0);
                            continue block12;
                        }
                        this.f_Graphics_00.drawImage(this.f_Image_arr_00[n9], n4 + (32 - n6 >> 1), n3 - (n5 - 30), 0);
                    }
                }
                continue;
            }
            object = this.objectTypeNames[this.entityType[i]];
            int n14 = this.f_Font_00.stringWidth((String)object) + 8 >> 1;
            this.f_Graphics_00.setColor(-1);
            this.f_Graphics_00.fillArc(n4, n3, 32, 32, 0, 360);
            this.f_Graphics_00.fillRect(n4 -= n14 - 16, n3 += 32 - this.f_int_01 >> 1, n14 << 1, this.f_int_01);
            this.f_Graphics_00.setColor(0);
            this.f_Graphics_00.drawRect(n4, n3, (n14 << 1) - 1, this.f_int_01 - 1);
            this.f_Graphics_00.drawString(this.objectTypeNames[this.entityType[i]], n4 + 4, n3, 0);
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        if (!bl) {
            this.m_033(n, n2);
        }
        if (this.f_byte_11 == 5) {
            a a2 = this;
            if (a2.f_int_44 >= 0) {
                n8 = a2.f_int_56 + a2.entityPixelX[a2.f_int_44] + 16;
                n7 = a2.f_int_57 + a2.entityPixelY[a2.f_int_44] + 32;
                n6 = a2.f_int_151 & 7;
                n5 = a2.f_byte_arr_44[n6];
                if (n5 > 0) {
                    n5 = n5 - 1 << 2;
                    a2.m_002(a2.f_Image_arr2_00[3][2], n8 - (a2.f_byte_arr_43[n5 + 2] >> 1), n7 - (a2.f_byte_arr_43[n5 + 3] >> 1), a2.f_byte_arr_43[n5], a2.f_byte_arr_43[n5 + 1], a2.f_byte_arr_43[n5 + 2], a2.f_byte_arr_43[n5 + 3]);
                }
            }
        }
        if (this.f_int_70 >= 0) {
            n4 = n + this.f_int_70;
            n3 = n2 + this.f_int_72;
            if (n4 >= -32 && n4 <= this.f_int_58 && n3 >= -12 && n3 <= 20 + this.f_int_59) {
                this.f_Graphics_00.drawImage(this.f_Image_arr2_00[2][1], n4 + 5, n3 - 30 + this.f_int_46, 0);
            }
        }
        if (this.f_int_71 >= 0) {
            n4 = n + this.f_int_71;
            n3 = n2 + this.f_int_73;
            if (n4 >= -32 && n4 <= this.f_int_58 && n3 >= -12 && n3 <= 20 + this.f_int_59) {
                this.f_Graphics_00.drawImage(this.f_Image_arr2_00[2][2], n4, n3 - 30 + this.f_int_46, 0);
            }
        }
    }

    private void m_054() {
        int n = 0;
        int n2 = 0;
        if (this.f_int_45 > 0) {
            for (int i = this.f_int_45; i >= 1; --i) {
                n = this.entityPixelY[0];
                for (int j = 1; j < i; ++j) {
                    if (this.f_bool_arr_01[j]) continue;
                    n2 = this.entityPixelY[j];
                    if (n2 < n) {
                        int n3;
                        int n4;
                        int n5 = j;
                        n2 = j - 1;
                        a a2 = this;
                        int n6 = a2.entityType[n2];
                        int n7 = a2.entityPixelX[n2];
                        int n8 = a2.entityPixelY[n2];
                        int n9 = a2.f_int_arr_08[n2];
                        int n10 = a2.f_int_arr_09[n2];
                        byte by = a2.f_byte_arr_04[n2];
                        boolean bl = a2.f_bool_arr_01[n2];
                        boolean bl2 = a2.f_bool_arr_00[n2];
                        short s = a2.entityParam[n2];
                        int n11 = n9 >> 5;
                        if (n11 <= 0) {
                            n11 = 1;
                        }
                        int n12 = n8 >> 5;
                        int n13 = n7 >> 5;
                        byte by2 = 0;
                        int n14 = 0;
                        block2: for (n4 = 0; n4 < n11; ++n4) {
                            by2 = a2.f_byte_arr2_02[n12][n13 + n4];
                            n14 = a2.f_byte_arr_10[by2];
                            for (n3 = 0; n3 < n14; ++n3) {
                                if (a2.f_byte_arr2_03[by2][n3] != n2 + 1) continue;
                                a2.f_byte_arr2_03[by2][n3] = n5 + 1;
                                continue block2;
                            }
                        }
                        a2.entityType[n2] = a2.entityType[n5];
                        a2.entityPixelX[n2] = a2.entityPixelX[n5];
                        a2.entityPixelY[n2] = a2.entityPixelY[n5];
                        a2.f_int_arr_08[n2] = a2.f_int_arr_08[n5];
                        a2.f_int_arr_09[n2] = a2.f_int_arr_09[n5];
                        a2.f_byte_arr_04[n2] = a2.f_byte_arr_04[n5];
                        a2.f_bool_arr_01[n2] = a2.f_bool_arr_01[n5];
                        a2.f_bool_arr_00[n2] = a2.f_bool_arr_00[n5];
                        a2.entityParam[n2] = a2.entityParam[n5];
                        a2.m_045(a2.entityType[n2], n2);
                        a2.entityType[n5] = n6;
                        a2.entityPixelX[n5] = n7;
                        a2.entityPixelY[n5] = n8;
                        a2.f_int_arr_08[n5] = n9;
                        a2.f_int_arr_09[n5] = n10;
                        a2.f_byte_arr_04[n5] = by;
                        a2.f_bool_arr_01[n5] = bl;
                        a2.f_bool_arr_00[n5] = bl2;
                        a2.entityParam[n5] = s;
                        a2.m_045(a2.entityType[n5], n5);
                        n12 = a2.entityPixelY[n2] >> 5;
                        n13 = a2.entityPixelX[n2] >> 5;
                        n11 = a2.f_int_arr_08[n2] >> 5;
                        if (n11 <= 0) {
                            n11 = 1;
                        }
                        block4: for (n4 = 0; n4 < n11; ++n4) {
                            by2 = a2.f_byte_arr2_02[n12][n13 + n4];
                            n14 = a2.f_byte_arr_10[by2];
                            for (n3 = 0; n3 < n14; ++n3) {
                                if (a2.f_byte_arr2_03[by2][n3] != n5 + 1) continue;
                                a2.f_byte_arr2_03[by2][n3] = n2 + 1;
                                continue block4;
                            }
                        }
                        n2 = this.entityPixelY[j];
                    }
                    n = n2;
                }
            }
        }
    }

    private void m_055() {
        int n = 0;
        if (this.f_int_45 > 0 && (this.f_int_03 & 1) != 0) {
            int n2 = this.f_int_45;
            block4: while (--n2 >= 0) {
                if (this.f_bool_arr_01[n2] || !this.f_bool_arr_00[n2] || (n = this.f_int_arr_12[n2]) <= 0) continue;
                if (this.f_int_arr_10[n2] < this.f_int_arr2_02[n].length - 1) {
                    int n3 = n2;
                    this.f_int_arr_10[n3] = this.f_int_arr_10[n3] + 1;
                    continue;
                }
                switch (this.f_byte_arr_04[n2]) {
                    case 1: {
                        this.m_051(n2);
                        continue block4;
                    }
                    case 2: {
                        this.m_044(this.entityType[n2], n2);
                        this.m_045(this.entityType[n2], n2);
                        this.f_byte_arr_04[n2] = 0;
                        continue block4;
                    }
                }
                this.f_int_arr_10[n2] = 0;
            }
        }
    }

    private void m_056(boolean n) {
        int n2 = 0;
        int n3 = 0;
        if (--this.f_int_49 < -154) {
            this.f_int_49 = 0;
        }
        boolean bl = false;
        n3 = n == 0 ? 0 : this.f_int_48 - 320 + 6;
        for (n2 = this.f_int_49; n2 < 240; n2 += 77) {
            for (n = n3; n > -320; n -= 320) {
                if (bl) {
                    this.m_004(this.f_Image_arr2_00[1][0], n2, n, 2);
                } else {
                    this.f_Graphics_00.drawImage(this.f_Image_arr2_00[1][0], n2, n, 0);
                }
                bl = !bl;
            }
            bl = false;
        }
    }

    private void m_057() {
        this.f_Image_03 = null;
        if (this.f_bool_arr_05[1]) {
            Image image = Image.createImage((int)(this.mapCellsWide << 2), (int)(this.mapCellsHigh << 2));
            Graphics graphics = image.getGraphics();
            int n = 0;
            int n2 = 0;
            graphics.setColor(13097429);
            graphics.fillRect(0, 0, this.mapCellsWide << 2, this.mapCellsHigh << 2);
            graphics.setColor(7509153);
            byte by = 0;
            int n3 = 0;
            while (n3 < this.mapCellsHigh) {
                int n4 = 0;
                while (n4 < this.mapCellsWide) {
                    by = this.f_byte_arr2_02[n3][n4];
                    if (this.f_bool_arr2_00[n3][n4]) {
                        graphics.setColor(7509153);
                        graphics.fillRect(n, n2, 4, 4);
                    }
                    if (by > 0 && this.f_byte_arr_10[by] > 0 && !this.f_bool_arr_01[(by = this.f_byte_arr2_03[by][0]) - 1] && this.f_byte_arr_04[by - 1] != 1) {
                        switch (this.entityType[by - 1]) {
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
                                a.m_003(this.f_Image_arr2_00[8][12], graphics, n - 1, n2 - 1, 5, 0, 5, 5);
                                break;
                            }
                            case 8: {
                                a.m_003(this.f_Image_arr2_00[8][12], graphics, n - 1, n2 - 1, 0, 0, 5, 5);
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
            this.f_Image_03 = image = a.m_007(image, 170);
        }
    }

    private static short m_058(InputStream inputStream) throws IOException {
        return (short)(inputStream.read() & 0xFF | inputStream.read() << 8 & 0xFF00);
    }

    private void m_059() {
        this.f_byte_arr2_02 = new byte[this.mapCellsHigh][this.mapCellsWide];
        this.f_byte_arr_10 = new byte[128];
        this.f_byte_arr2_03 = new byte[128][16];
        this.f_byte_15 = 0;
    }

    private boolean isCellWalkable(int n, int n2) {
        if (n >= 0 && n < this.mapCellsWide && n2 >= 0 && n2 < this.mapCellsHigh) {
            return this.f_bool_arr2_00[n2][n];
        }
        return false;
    }

    private void m_061() {
        int n = 0;
        byte by = 0;
        int n2 = this.mapCellsWide;
        int n3 = this.mapCellsHigh;
        int n4 = 0;
        while (n4 < n3) {
            int n5 = 0;
            while (n5 < n2) {
                by = this.mapTerrainGrid[n];
                this.f_bool_arr2_00[n4][n5] = by < this.f_bool_arr_04.length ? this.f_bool_arr_04[by] : false;
                ++n5;
                n += 2;
            }
            ++n4;
            n += n2 << 1;
        }
    }

    private boolean m_062(int n, int n2) {
        if (n >= 0 && n < this.mapCellsWide && n2 >= 0 && n2 < this.mapCellsHigh) {
            return this.f_byte_arr_10[this.f_byte_arr2_02[n2][n]] > 0;
        }
        return false;
    }

    private void m_063(int n, int n2) {
        int n3;
        int n4;
        if (this.f_byte_11 == 4) {
            a a2 = this;
            ++a2.f_int_55;
            if (a2.f_int_55 == 18) {
                a a3 = a2;
                n4 = 0;
                n3 = a3.f_int_86;
                while (--n3 >= 0) {
                    n4 = a3.f_short_arr_03[n3];
                    a3.f_int_arr_12[n4] = 6;
                    a3.f_byte_arr_04[n4] = 1;
                }
            } else if (a2.f_int_55 >= 24) {
                a2.f_int_55 = 0;
                a a4 = a2;
                for (n4 = 0; n4 < a4.f_int_86; ++n4) {
                    a4.m_049(a4.f_short_arr_01[n4], a4.f_short_arr_02[n4], 4);
                }
                a4.f_int_86 = 0;
                a2.f_int_54 = 0;
                a2.f_byte_11 = 0;
            }
            a2.f_int_54 = a2.f_int_54 == -2 ? 2 : -2;
            n2 = 20 + this.f_int_54;
        }
        n = this.f_int_56 + (this.f_int_60 << 4);
        n2 = this.f_int_57 + n2 + (this.f_int_62 << 4);
        n4 = 0;
        int n5 = n;
        int n6 = n2;
        n2 = n3 = this.f_int_60 + (this.f_int_62 * this.mapCellsWide << 1);
        int n7 = this.f_int_62;
        while (n7 < this.f_int_63) {
            int n8 = this.f_int_60;
            while (n8 < this.f_int_61) {
                n4 = this.mapTerrainGrid[n2];
                if (n4 > 0) {
                    int n9 = (n4 & 7) << 4;
                    n4 = n4 >> 3 << 4;
                    this.m_005(this.f_Image_arr2_00[2][0], n5, n6, n9, n4, 16, 16, this.mapTransformGrid[n2]);
                }
                ++n8;
                ++n2;
                n5 += 16;
            }
            ++n7;
            n6 += 16;
            n5 = n;
            n2 = n3 += this.mapCellsWide << 1;
        }
        this.f_Graphics_00.setClip(0, 0, 240, 320);
    }

    private void m_064(int n, int n2) {
        if (!this.f_bool_10) {
            if (n > 64) {
                n = 64;
            } else if (n < -((this.mapCellsWide + 2 << 5) - this.f_int_58)) {
                n = -((this.mapCellsWide + 2 << 5) - this.f_int_58);
            }
            this.f_int_56 = n;
            if (n < 0) {
                this.f_int_60 = -n >> 4;
                this.f_int_61 = this.f_int_60 + (this.f_int_58 >> 4) + 1;
            } else {
                this.f_int_60 = 0;
                this.f_int_61 = (this.f_int_58 - n >> 4) + 1;
            }
            if (this.f_int_61 > this.mapCellsWide << 1) {
                this.f_int_61 = this.mapCellsWide << 1;
            }
            if (n2 > 64) {
                n2 = 64;
            } else if (n2 < -((this.mapCellsHigh + 2 << 5) - this.f_int_59)) {
                n2 = -((this.mapCellsHigh + 2 << 5) - this.f_int_59);
            }
        } else {
            this.f_int_56 = this.f_int_58 - this.f_int_52 >> 1;
            this.f_int_60 = 0;
            this.f_int_61 = this.mapCellsWide << 1;
        }
        if (!this.f_bool_11) {
            this.f_int_57 = n2;
            if (n2 < 0) {
                this.f_int_62 = -n2 >> 4;
                this.f_int_63 = this.f_int_62 + (this.f_int_59 >> 4) + 2;
            } else {
                this.f_int_62 = 0;
                this.f_int_63 = (this.f_int_59 - n2 >> 4) + 1;
            }
            if (this.f_int_63 > this.mapCellsHigh << 1) {
                this.f_int_63 = this.mapCellsHigh << 1;
                return;
            }
        } else {
            this.f_int_57 = this.f_int_59 - this.f_int_53 >> 1;
            this.f_int_62 = 0;
            this.f_int_63 = this.mapCellsHigh << 1;
        }
    }

    private int m_065(boolean bl) {
        if (bl) {
            for (int i = 0; i < this.f_int_45; ++i) {
                int n = this.entityParam[i] >> 9;
                if (this.entityType[i] != 7 || n != 0) continue;
                return i;
            }
        } else {
            for (int i = 0; i < this.f_int_45; ++i) {
                int n = this.entityParam[i] >> 9;
                if (this.entityType[i] != 8 || n != 0) continue;
                return i;
            }
        }
        return -1;
    }

    /*
     * Unable to fully structure code
     */
    private boolean changeFloor(int var1_1, boolean var2_2, boolean var3_3) {
        var4_4 = false;
        if (!var3_3) ** GOTO lbl-1000
        if (var1_1 < this.minFloorReached) {
            this.m_015((byte)0, this.f_String_arr_00[0], (byte)0, (byte)0);
            var4_4 = false;
        } else if (var1_1 > this.maxFloorReached) {
            var4_4 = false;
            this.m_015((byte)0, this.f_String_arr_00[1], (byte)0, (byte)0);
        } else lbl-1000:
        // 2 sources

        {
            var4_4 = true;
        }
        if (var4_4) {
            var4_4 = false;
            if (var1_1 < 0) {
                this.m_015((byte)0, this.f_String_arr_00[2], (byte)0, (byte)0);
            } else if (var1_1 > this.f_int_68) {
                this.m_015((byte)0, this.f_String_arr_00[3], (byte)0, (byte)0);
            } else {
                var4_4 = true;
                this.f_bool_16 = true;
                if (var1_1 < this.minFloorReached) {
                    this.minFloorReached = var1_1;
                } else if (var1_1 > this.maxFloorReached) {
                    this.maxFloorReached = var1_1;
                    if (this.maxFloorReached < 51 && this.maxFloorReached > this.f_int_154) {
                        this.f_int_154 = this.maxFloorReached;
                    }
                }
                this.m_119(this.currentFloor);
                this.f_byte_23 = (byte)var1_1;
                this.f_bool_18 = var2_2;
            }
        }
        return var4_4;
    }

    private void m_067() {
        this.m_131((byte)3, true);
        this.m_132(6);
        this.m_132(8);
        this.m_132(5);
        this.m_132(13);
        this.m_132(9);
        this.m_132(10);
        this.m_132(2);
        this.m_132(3);
        this.m_132(12);
        this.m_132(11);
    }

    private void m_068() {
        int n;
        this.equippedWeaponType = 0;
        this.equippedArmorType = 0;
        if (this.currentFloor == 1) {
            this.m_024(6, 11);
        } else if (this.currentFloor == 51) {
            this.m_024(1, 11);
        } else if (this.currentFloor == 50) {
            this.m_024(6, 6);
        } else {
            n = this.m_065(false);
            if (n >= 0) {
                this.m_031(this.entityPixelX[n] >> 5, this.entityPixelY[n] >> 5);
            }
        }
        n = this.f_int_113;
        while (--n >= 0) {
            this.f_bool_arr_06[n] = false;
        }
        this.f_int_89 = 0;
        this.goldAmount = 0;
        this.alchemyUpgradeCount = 0;
        this.itemStackSize = 0;
        this.f_byte_26 = 0;
        this.scaleEnemyStats(this.difficultyMultipliers[this.f_byte_26]);
    }

    private void scaleEnemyStats(int n) {
        if (this.enemyAtkScaled == null) {
            this.enemyAtkScaled = new int[this.enemyBaseAtk.length];
        }
        int n2 = this.enemyAtkScaled.length;
        while (--n2 >= 0) {
            this.enemyAtkScaled[n2] = this.enemyBaseAtk[n2] * n;
        }
        if (this.enemyDefScaled == null) {
            this.enemyDefScaled = new int[this.enemyBaseDef.length];
        }
        n2 = this.enemyDefScaled.length;
        while (--n2 >= 0) {
            this.enemyDefScaled[n2] = this.enemyBaseDef[n2] * n;
        }
        if (this.enemyHpScaled == null) {
            this.enemyHpScaled = new int[this.enemyBaseHp.length];
        }
        n2 = this.enemyHpScaled.length;
        while (--n2 >= 0) {
            this.enemyHpScaled[n2] = this.enemyBaseHp[n2] * n;
        }
    }

    private static int scaledByFloorTier(int n, int n2) {
        --n2;
        n2 /= 10;
        return n * ++n2;
    }

    private static int alchemyPriceFor(int n) {
        int n2 = 20;
        for (int i = 1; i < n; ++i) {
            n2 += 20 * i;
        }
        return n2;
    }

    private void m_072(int n) {
        int n2 = this.f_Font_00.stringWidth(this.f_String_arr_05[n]) + 80;
        this.f_byte_arr_11[this.f_int_83++] = n;
        if (this.f_int_83 == 1) {
            this.f_int_85 = 32 + this.f_int_81;
            this.f_int_84 = n2;
        } else {
            this.f_int_85 += this.f_int_81;
            if (this.f_int_84 < n2) {
                this.f_int_84 = n2;
            }
        }
        switch (n) {
            case 0: {
                this.f_bool_arr_05[this.f_int_83 - 1] = this.f_bool_29;
            }
        }
    }

    private boolean m_073(int n) {
        int n2 = 0;
        this.f_int_86 = 0;
        for (int i = 0; i < this.f_int_45; ++i) {
            if (this.f_bool_arr_01[i] || n != this.entityParam[i]) continue;
            n2 = this.entityType[i];
            if (n2 == 5) {
                return false;
            }
            if (n2 != 4) continue;
            this.f_short_arr_03[this.f_int_86] = i;
            this.f_short_arr_01[this.f_int_86] = this.entityPixelX[i] >> 5;
            this.f_short_arr_02[this.f_int_86] = this.entityPixelY[i] >> 5;
            ++this.f_int_86;
        }
        return this.f_int_86 > 0;
    }

    private boolean applyMerchantOffer(boolean bl) {
        short s = this.entityParam[this.f_int_87];
        byte by = (byte)(s >>> 8);
        s = (byte)s;
        int n = this.entityType[this.f_int_87] == 77 ? 0 : 1;
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
                                this.addItemToItemStack(13);
                                break;
                            }
                            case 1: 
                            case 15: 
                            case 21: {
                                this.gainGold(1000, this.playerPixelX, this.playerPixelY);
                                break;
                            }
                            case 6: {
                                this.addItemToItemStack(19);
                            }
                        }
                        break;
                    }
                    case 1: {
                        switch (s) {
                            case 1: {
                                this.playerAtk += this.playerAtk * 3 / 100;
                                this.playerDef += this.playerDef * 3 / 100;
                                break block0;
                            }
                            case 3: {
                                if (this.spendGold(50)) {
                                    ++this.blueKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 4: {
                                if (this.spendGold(50)) {
                                    this.yellowKeyCount += 5;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 5: {
                                if (this.spendGold(1000)) {
                                    ++this.yellowKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 6: {
                                if (this.spendGold(800)) {
                                    ++this.redKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 7: {
                                if (this.spendGold(200)) {
                                    ++this.blueKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 8: {
                                if (this.yellowKeyCount > 0) {
                                    --this.yellowKeyCount;
                                    this.goldAmount += 100;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u9ec4\u94a5\u5319", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 9: {
                                if (this.spendGold(1000)) {
                                    ++this.yellowKeyCount;
                                    ++this.blueKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 10: {
                                if (this.spendGold(200)) {
                                    this.yellowKeyCount += 3;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 11: {
                                if (this.spendGold(2000)) {
                                    this.blueKeyCount += 3;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 12: {
                                if (this.spendGold(1000)) {
                                    this.playerHp += 2000;
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 13: {
                                if (this.spendGold(4000)) {
                                    this.pickupItemType(18);
                                    break block0;
                                }
                                this.m_015((byte)0, "\u6ca1\u6709\u8db3\u591f\u7684\u91d1\u94b1", (byte)0, (byte)0);
                                bl2 = false;
                            }
                        }
                    }
                }
                if (bl2) {
                    if ((by & 4) == 0) {
                        int n2 = this.f_int_87;
                        this.entityParam[n2] = this.entityParam[n2] ^ 0x200;
                        if ((by & 1) != 0) {
                            this.m_075(n, s - 1);
                            if (n == 0) {
                                this.m_015((byte)4, this.f_String_arr_06[s - 1], (byte)0, (byte)0);
                            } else {
                                this.m_015((byte)4, this.f_String_arr_09[s - 1], (byte)0, (byte)0);
                            }
                            bl3 = true;
                        } else {
                            this.m_047(this.f_int_87);
                        }
                    }
                } else {
                    bl3 = true;
                }
            } else if ((by & 1) != 0) {
                int n3 = this.f_int_87;
                this.entityParam[n3] = this.entityParam[n3] ^ 0x100;
                if ((by & 4) == 0) {
                    this.m_047(this.f_int_87);
                }
            }
        } else if ((by & 2) == 0 && (by & 1) != 0) {
            int n4 = this.f_int_87;
            this.entityParam[n4] = this.entityParam[n4] ^ 0x100;
            if ((by & 4) == 0) {
                this.m_047(this.f_int_87);
            }
        }
        return bl3;
    }

    private void m_075(int n, int n2) {
        if (this.f_int_89 < this.f_int_88) {
            this.f_byte_arr_14[this.f_int_89] = n;
            this.f_byte_arr_15[this.f_int_89] = n2;
            ++this.f_int_89;
        }
    }

    private void m_076(int n) {
        byte by = 0;
        byte by2 = 0;
        if (this.f_int_89 > 0) {
            this.f_int_90 = n < 0 ? this.f_int_89 - 1 : (n >= this.f_int_89 ? 0 : n);
            by = this.f_byte_arr_14[this.f_int_90];
            by2 = this.f_byte_arr_15[this.f_int_90];
            if (by == 0) {
                this.m_015((byte)5, this.f_String_arr_06[by2], (byte)0, (byte)3);
            } else {
                this.m_015((byte)5, this.f_String_arr_09[by2], (byte)0, (byte)3);
            }
            this.f_String_04 = "" + (this.f_int_90 + 1) + " / " + this.f_int_89;
            return;
        }
        this.m_015((byte)0, "\u6ca1\u6709\u8bb0\u5f55", (byte)0, (byte)3);
        this.gameMode = (byte)10;
    }

    private boolean spendGold(int n) {
        boolean bl = true;
        if (this.goldAmount >= n) {
            this.goldAmount -= n;
        } else {
            bl = false;
        }
        return bl;
    }

    private void gainGold(int n, int n2, int n3) {
        this.goldAmount += n;
        if (n > 0) {
            this.m_125((byte)4, n, n2, n3);
        }
    }

    private void pickupItemType(int n) {
        int n2 = 0;
        n2 = this.m_083(n, 0, 12);
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
                if (this.m_082(n2, false)) {
                    this.f_byte_07 = (byte)n;
                    this.m_015((byte)1, this.equipDescriptions[n2], (byte)0, (byte)3);
                    return;
                }
                this.m_015((byte)0, this.f_String_arr_00[5], (byte)0, (byte)3);
                return;
            }
            case 15: {
                this.f_bool_12 = true;
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
                this.addItemToItemStack(n);
                return;
            }
            case 26: {
                ++this.yellowKeyCount;
                return;
            }
            case 27: {
                ++this.redKeyCount;
                return;
            }
            case 28: {
                ++this.blueKeyCount;
                return;
            }
            case 29: {
                if (this.currentFloor <= 10) {
                    ++this.playerAtk;
                    return;
                }
                this.playerAtk += this.floorTier();
                return;
            }
            case 30: {
                if (this.currentFloor <= 10) {
                    ++this.playerDef;
                    return;
                }
                this.playerDef += this.floorTier();
                return;
            }
            case 31: {
                n2 = this.floorTier();
                this.applyHpDelta(50 * n2);
                return;
            }
            case 32: {
                n2 = this.floorTier();
                this.applyHpDelta(200 * n2);
            }
        }
    }

    private int floorTier() {
        int n = this.currentFloor - 1;
        if ((n /= 10) < 0) {
            n = 0;
        }
        if (this.currentFloor > 50) {
            n = 0;
        }
        return n + 1;
    }

    private int m_081(int n) {
        int n2 = -1;
        for (int i = 0; i < this.itemStackSize; ++i) {
            if (this.itemStackTypes[i] != n) continue;
            n2 = i;
            break;
        }
        return n2;
    }

    private boolean m_082(int n, boolean bl) {
        boolean bl2 = true;
        int n2 = 0;
        if (n < 6) {
            n2 = this.m_083(this.equippedWeaponType, 0, 6);
            if (n2 < n || bl) {
                this.playerAtk -= this.equipTierBonuses[n2];
                this.playerAtk += this.equipTierBonuses[n];
                this.equippedWeaponType = this.equipTierTypes[n];
            } else {
                bl2 = false;
            }
        } else {
            n2 = this.m_083(this.equippedArmorType, 6, 12);
            if (n2 < n || bl) {
                this.playerDef -= this.equipTierBonuses[n2];
                this.playerDef += this.equipTierBonuses[n];
                this.equippedArmorType = this.equipTierTypes[n];
            } else {
                bl2 = false;
            }
        }
        return bl2;
    }

    private int m_083(int n, int n2, int n3) {
        while (--n3 >= n2) {
            if (this.equipTierTypes[n3] != n) continue;
            return n3;
        }
        return 0;
    }

    private static int itemTypeToStackIndex(int n) {
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

    private void addItemToItemStack(int n) {
        int n2 = this.m_081(n);
        int n3 = a.itemTypeToStackIndex(n);
        if (n2 < 0) {
            this.itemStackTypes[this.itemStackSize] = n;
            this.itemStackUses[this.itemStackSize] = this.itemUseCounts[n3];
            ++this.itemStackSize;
        } else {
            int n4 = n2;
            this.itemStackUses[n4] = this.itemStackUses[n4] + this.itemUseCounts[n3];
        }
        this.f_byte_07 = (byte)n;
        this.m_015((byte)1, this.itemDescriptions[n3], (byte)0, (byte)3);
    }

    private boolean consumeKeyForDoor(byte by) {
        boolean bl = false;
        switch (by) {
            case 26: {
                if (this.yellowKeyCount <= 0) break;
                this.m_125((byte)1, 2, this.playerPixelX, this.playerPixelY);
                --this.yellowKeyCount;
                bl = true;
                break;
            }
            case 28: {
                if (this.blueKeyCount <= 0) break;
                this.m_125((byte)1, 0, this.playerPixelX, this.playerPixelY);
                --this.blueKeyCount;
                bl = true;
                break;
            }
            case 27: {
                if (this.redKeyCount <= 0) break;
                this.m_125((byte)1, 1, this.playerPixelX, this.playerPixelY);
                --this.redKeyCount;
                bl = true;
            }
        }
        return bl;
    }

    private void useItemStack(int n) {
        if (this.itemStackUses[n] > 0) {
            if (this.activateItem(this.itemStackTypes[n])) {
                int n2 = n;
                this.itemStackUses[n2] = (byte)(this.itemStackUses[n2] - 1);
                if (this.itemStackUses[n2] == 0) {
                    while (n < this.itemStackSize - 1) {
                        this.itemStackTypes[n] = this.itemStackTypes[n + 1];
                        this.itemStackUses[n] = this.itemStackUses[n + 1];
                        ++n;
                    }
                    if (this.itemStackSize > 0) {
                        --this.itemStackSize;
                    }
                    return;
                }
            }
        } else {
            this.activateItem(this.itemStackTypes[n]);
        }
    }

    private boolean activateItem(byte by) {
        int n = 0;
        n = 0;
        switch (by) {
            case 13: {
                this.gameMode = (byte)5;
                this.m_000();
                break;
            }
            case 14: {
                this.gameMode = (byte)12;
                this.m_000();
                break;
            }
            case 15: {
                break;
            }
            case 16: {
                if (!this.m_089((byte)10)) {
                    this.m_015((byte)0, "\u4f60\u5fc5\u987b\u9762\u5bf9\u4e09\u6627\u771f\u706b\u518d\u4f7f\u7528\u5b83\u3002", (byte)0, (byte)0);
                    break;
                }
                this.gameMode = (byte)3;
                n = 1;
                break;
            }
            case 17: {
                if (!this.m_089((byte)11)) {
                    this.m_015((byte)0, "\u4f60\u5fc5\u987b\u9762\u5bf9\u4e00\u5835\u5899\u4f7f\u7528", (byte)0, (byte)0);
                    break;
                }
                this.gameMode = (byte)3;
                n = 1;
                break;
            }
            case 18: {
                by = (byte)this.f_int_45;
                while ((by = (byte)(by - 1)) >= 0) {
                    if (this.entityType[by] != 11) continue;
                    this.m_047(by);
                }
                this.gameMode = (byte)3;
                n = 1;
                break;
            }
            case 19: {
                n = (this.playerAtk + this.playerDef) * 74 / 10;
                this.applyHpDelta(n);
                this.m_015((byte)0, "\u589e\u52a0\u4e86" + n + "\u8840\u91cf", (byte)0, (byte)0);
                n = 1;
                break;
            }
            case 20: {
                if (this.currentFloor == 40) {
                    this.m_015((byte)0, "\u672c\u5c42\u4e0d\u80fd\u76f4\u63a5\u77ac\u79fb\u3002", (byte)0, (byte)0);
                    break;
                }
                a a2 = this;
                int n2 = a2.mapCellsWide - 1 - a2.playerCellX;
                int n3 = a2.mapCellsHigh - 1 - a2.playerCellY;
                boolean bl = a2.interactWithCell(n2, n3);
                if (bl) {
                    a2.m_024(n2, n3);
                    a2.m_064((a2.f_int_58 - 32 >> 1) - a2.playerPixelX, (a2.f_int_59 - 32 >> 1) - a2.playerPixelY);
                    a2.applyStepCellEffects();
                }
                if (bl) {
                    this.gameMode = (byte)3;
                    n = 1;
                    break;
                }
                if (this.f_bool_05) break;
                this.m_015((byte)0, "\u65e0\u6cd5\u79fb\u52a8\u5230\u8be5\u4f4d\u7f6e", (byte)0, (byte)0);
                break;
            }
            case 21: {
                n = this.changeFloor(this.currentFloor + 1, false, false);
                this.f_int_06 = 0;
                this.gameMode = (byte)3;
                break;
            }
            case 22: {
                n = this.changeFloor(this.currentFloor - 1, true, false);
                this.f_int_06 = 0;
                this.gameMode = (byte)3;
                break;
            }
            case 85: {
                by = (byte)this.f_int_45;
                while ((by = (byte)(by - 1)) >= 0) {
                    if (this.entityType[by] != 1) continue;
                    this.m_047(by);
                }
                this.gameMode = (byte)3;
                n = 1;
                break;
            }
            case 86: {
                a a3 = this;
                int n4 = a3.playerPixelX >> 5;
                int n5 = a3.playerPixelY >> 5;
                a3.killAdjacentAt(n4, n5 - 1);
                a3.killAdjacentAt(n4, n5 + 1);
                a3.killAdjacentAt(n4 - 1, n5);
                a3.killAdjacentAt(n4 + 1, n5);
                this.gameMode = (byte)3;
                n = 1;
            }
        }
        return n != 0;
    }

    private boolean m_089(byte by) {
        int n = this.playerPixelX >> 5;
        int n2 = this.playerPixelY >> 5;
        int n3 = this.m_100(n, n2 - 1, by);
        int n4 = this.m_100(n, n2 + 1, by);
        int n5 = this.m_100(n - 1, n2, by);
        by = (byte)this.m_100(n + 1, n2, by);
        n = 0;
        if (n3 >= 0) {
            this.m_047(n3);
            ++n;
        }
        if (n4 >= 0) {
            this.m_047(n4);
            ++n;
        }
        if (n5 >= 0) {
            this.m_047(n5);
            ++n;
        }
        if (by >= 0) {
            this.m_047(by);
            ++n;
        }
        return n > 0;
    }

    private void killAdjacentAt(int n, int n2) {
        try {
            byte by = this.f_byte_arr2_02[n2][n];
            int n3 = this.f_byte_arr_10[by];
            int n4 = 0;
            int n5 = 0;
            n4 = 0;
            if (n3 > 0) {
                for (int i = 0; i < n3; ++i) {
                    n4 = this.f_byte_arr2_03[by][i] - 1;
                    n5 = this.entityType[n4];
                    if (n5 == 5) {
                        n4 = this.entityParam[n4];
                        this.m_050(n, n2, i);
                        if (!this.m_073(n4)) continue;
                        this.f_byte_11 = (byte)4;
                        continue;
                    }
                    if (n5 >= 67 || this.f_byte_arr_03[n5] != 8 || this.f_byte_arr_04[n4] == 1) continue;
                    this.m_047(n4);
                    if (this.m_081(25) >= 0) {
                        this.gainGold(this.enemyBaseGold[n5 - 41] << 1, 0, 0);
                        continue;
                    }
                    this.gainGold(this.enemyBaseGold[n5 - 41], 0, 0);
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

    private void m_091() {
        int n;
        int n2;
        int n3;
        a a2;
        int n4;
        this.f_int_arr_22 = null;
        this.f_int_arr_22 = new int[128];
        this.f_byte_arr_20 = null;
        this.f_byte_arr_20 = new byte[128];
        this.f_int_108 = 208;
        this.f_int_105 = 0;
        block0: for (int i = 0; i < this.f_int_45; ++i) {
            if (this.f_bool_arr_01[i] || this.f_byte_arr_03[this.entityType[i]] != 8) continue;
            n4 = this.entityType[i];
            a2 = this;
            n3 = a2.f_int_105;
            while (--n3 >= 0) {
                if (n4 != a2.f_byte_arr_20[n3]) continue;
                continue block0;
            }
            n3 = a2.enemyHpScaled[n4 - 41];
            n2 = a2.enemyAtkScaled[n4 - 41];
            n = a2.enemyDefScaled[n4 - 41];
            a2.f_int_arr_22[a2.f_int_105] = a.predictBattleHpLoss(a2.effectiveAttackVsType(n4), a2.playerDef, n3, n2, n, true);
            a2.f_byte_arr_20[a2.f_int_105] = n4;
            ++a2.f_int_105;
        }
        a2 = this;
        n4 = 0;
        n3 = 0;
        for (n2 = 0; n2 < a2.f_int_105; ++n2) {
            n = a2.f_int_105;
            while (--n > n2) {
                if (a2.f_int_arr_22[n] < 0 || a2.f_int_arr_22[n] >= a2.f_int_arr_22[n - 1]) continue;
                n4 = a2.f_byte_arr_20[n];
                a2.f_byte_arr_20[n] = a2.f_byte_arr_20[n - 1];
                a2.f_byte_arr_20[n - 1] = n4;
                n3 = a2.f_int_arr_22[n];
                a2.f_int_arr_22[n] = a2.f_int_arr_22[n - 1];
                a2.f_int_arr_22[n - 1] = n3;
            }
        }
        this.f_int_106 = 258 / (this.f_int_01 + 4 + 36);
        if (this.f_int_106 > this.f_int_105) {
            this.f_int_106 = this.f_int_105;
        }
        this.f_int_107 = (this.f_int_01 + 4 + 36) * this.f_int_106 + 32;
        this.f_int_109 = 0;
        this.f_int_110 = this.f_int_106;
        if (this.f_int_110 > this.f_int_105) {
            this.f_int_110 = this.f_int_105;
        }
    }

    private void m_092(int n) {
        int n2 = this.f_int_105 - this.f_int_106;
        if (n2 < 0) {
            n2 = 0;
        }
        if (n >= 0 && n <= n2) {
            this.f_int_109 = n;
            this.f_int_110 = this.f_int_109 + this.f_int_106;
            if (this.f_int_110 > this.f_int_105) {
                this.f_int_110 = this.f_int_105;
            }
        }
    }

    private void m_093(int n, int n2, byte by) {
        this.f_Graphics_00.fillTriangle(n, n2 - 6, n, n2 + 6, n - 6, n2);
    }

    private boolean loadLevelScript(int n) {
        this.f_bool_27 = false;
        this.gameMode = (byte)11;
        this.currentScriptIndex = n;
        this.f_int_112 = 0;
        this.m_104(0);
        this.f_byte_16 = 0;
        this.f_String_05 = this.levelScriptLines[this.currentScriptIndex];
        this.f_String_02 = null;
        this.scriptCursor = 0;
        switch (n) {
            case 31: {
                this.m_119(this.currentFloor);
                this.m_122(24);
                n = this.f_int_45;
                while (--n >= 0) {
                    if (this.entityType[n] != 4) continue;
                    this.f_bool_arr_01[n] = true;
                }
                this.m_119(24);
                this.m_059();
                this.m_122(this.currentFloor);
            }
        }
        return false;
    }

    private boolean tryRunScene(int n, boolean bl) {
        if (n >= this.f_int_113) {
            return false;
        }
        boolean bl2 = !this.f_bool_arr_06[n];
        int n2 = 0;
        switch (n) {
            case 0: {
                if (!bl2) break;
                this.f_bool_arr_06[19] = true;
                break;
            }
            case 1: {
                bl2 &= this.f_bool_arr_06[0] & !bl;
                break;
            }
            case 2: {
                bl2 &= this.f_bool_arr_06[1] & !bl;
                break;
            }
            case 3: {
                bl2 &= bl;
                break;
            }
            case 4: {
                bl2 &= this.f_bool_arr_06[3] & bl;
                break;
            }
            case 7: {
                if (!(bl2 &= bl && !this.m_102(23, 12))) break;
                this.f_bool_arr_06[32] = true;
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
                bl2 &= bl & !this.f_bool_arr_06[47];
                break;
            }
            case 47: {
                bl2 &= bl & !this.f_bool_arr_06[46];
                break;
            }
            case 13: {
                break;
            }
            case 15: {
                if (this.m_102(this.currentFloor, 5)) {
                    bl2 = false;
                    break;
                }
                bl2 &= bl;
                break;
            }
            case 18: {
                if (!(bl2 &= bl) || this.m_100(2, 2, 1) >= 0 && this.m_100(2, 4, 1) >= 0 && this.m_100(2, 6, 1) >= 0 && this.m_100(6, 2, 1) >= 0 && this.m_100(4, 6, 1) >= 0 && this.m_100(6, 6, 1) >= 0 && this.m_100(4, 2, 1) < 0 && this.m_100(6, 4, 1) < 0) break;
                bl2 = false;
                this.f_bool_arr_06[18] = true;
                break;
            }
            case 25: {
                bl2 &= bl & this.f_bool_arr_06[13] & this.m_100(2, 2, 60) < 0 & this.m_100(10, 2, 60) < 0;
                break;
            }
            case 26: {
                if (this.m_100(11, 4, 84) >= 0) break;
                bl2 = false;
                break;
            }
            case 31: {
                bl2 &= bl;
                bl2 &= this.m_081(16) >= 0;
                break;
            }
            case 32: {
                if (this.f_bool_arr_06[7]) {
                    bl2 = false;
                    break;
                }
                bl2 &= !bl & this.m_102(23, 12);
                break;
            }
            case 33: {
                bl2 &= this.f_bool_arr_06[7];
                break;
            }
            case 34: {
                if (!(bl2 &= bl & !this.f_bool_arr_06[10])) break;
                n2 = this.predictHpLossVsType(69, true);
                bl2 &= n2 < 0 || n2 >= this.playerHp;
                break;
            }
            case 24: {
                if (!(bl2 &= !bl)) break;
                bl2 = true & 0 < this.playerHp;
                break;
            }
            case 16: {
                bl2 &= this.f_byte_26 == 0;
                break;
            }
            case 66: {
                bl2 = false;
                break;
            }
            case 67: {
                bl2 &= this.f_byte_26 > 0;
                break;
            }
            case 27: {
                bl2 &= bl & this.f_bool_arr_06[0];
            }
        }
        return bl2;
    }

    private void m_096(int n) {
        byte by = 0;
        if (n <= this.f_int_121) {
            this.f_int_112 = n;
            this.f_byte_17 = this.f_int_112 < 0 ? (byte)-1 : this.dialogueSpeakerType[this.f_int_112];
            this.f_int_119 = this.m_104(this.f_byte_17);
            by = this.f_byte_17;
            if (by < 0) {
                this.f_String_03 = this.f_String_02 = "?: \\cF8F8F8" + this.dialogueTexts[this.f_int_112];
                this.f_int_33 = 11;
            } else {
                this.f_String_03 = this.f_String_02 = this.objectTypeNames[by] + ": \\cF8F8F8" + this.dialogueTexts[this.f_int_112];
                this.f_int_33 = this.objectTypeNames[by].length() + 10;
            }
            this.m_018(this.f_String_02, 129, (this.f_int_34 << 1) + 12, this.f_int_33);
            return;
        }
        this.f_byte_16 = 0;
        this.f_int_33 = 0;
    }

    private boolean m_097() {
        return this.f_int_123 == this.f_int_125 && this.f_int_124 == this.f_int_126;
    }

    private void executeScriptInstruction(String object, int n) {
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
                this.f_int_120 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                this.f_int_121 = this.parseScriptInt((String)object, n + 1, " ");
                this.scriptCursor = this.f_int_122 + 1;
                this.f_byte_16 = 1;
                this.m_096(this.f_int_120);
            } else if (string.equals("MOV")) {
                int n5 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                if (n5 > 0) {
                    n3 = this.parseScriptInt((String)object, n + 1, "_");
                    n = this.f_int_122;
                    n4 = this.parseScriptInt((String)object, n + 1, "_");
                    n = this.f_int_122;
                }
                this.f_int_114 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                this.f_int_115 = this.parseScriptInt((String)object, n + 1, " ");
                this.scriptCursor = this.f_int_122 + 1;
                if (n5 > 0) {
                    this.f_int_127 = this.m_100(n3, n4, n5);
                    if (this.f_int_127 >= 0) {
                        this.f_byte_18 = 0;
                        object = this;
                        if (((a)object).entityType[((a)object).f_int_127] == 72) {
                            ((a)object).f_int_arr_12[((a)object).f_int_127] = 1;
                        }
                        ((a)object).f_int_129 = ((a)object).entityPixelX[((a)object).f_int_127] >> 5;
                        ((a)object).f_int_130 = ((a)object).entityPixelY[((a)object).f_int_127] >> 5;
                        super.m_133(((a)object).f_int_129, ((a)object).f_int_130, ((a)object).f_int_114, ((a)object).f_int_115);
                        this.f_byte_16 = (byte)2;
                    } else {
                        this.executeScriptInstruction(this.f_String_05, this.scriptCursor);
                    }
                } else if (this.m_133(this.playerCellX, this.playerCellY, this.f_int_114, this.f_int_115)) {
                    this.f_bool_27 = true;
                    this.f_byte_16 = (byte)3;
                    this.f_byte_11 = 0;
                    this.m_104(0);
                }
            } else if (string.equals("GUT")) {
                n2 = this.parseScriptInt((String)object, n + 1, " ");
                this.f_bool_arr_06[this.currentScriptIndex] = true;
                this.loadLevelScript(n2);
            } else if (string.equals("DES")) {
                int n6 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                if (n6 < 0) {
                    this.parseScriptInt((String)object, n + 1, " ");
                    n2 = -n6;
                    object = this;
                    n6 = ((a)object).f_int_45;
                    while (--n6 >= 0) {
                        if (((a)object).entityType[n6] != n2) continue;
                        super.m_047(n6);
                    }
                } else {
                    n3 = this.parseScriptInt((String)object, n + 1, "_");
                    n6 = this.m_100(n3, n4 = this.parseScriptInt((String)object, (n = this.f_int_122) + 1, " "), n6);
                    if (n6 >= 0) {
                        this.m_047(n6);
                    }
                }
                this.scriptCursor = this.f_int_122 + 1;
            } else if (string.equals("SWD")) {
                object = this;
                int n7 = ((a)object).f_int_45;
                while (--n7 >= 0) {
                    if (((a)object).f_bool_arr_01[n7] || (n2 = ((a)object).entityType[n7]) != 81) continue;
                    ((a)object).f_bool_arr_00[n7] = true;
                    ((a)object).entityType[n7] = 4;
                    super.m_044(4, n7);
                    ((a)object).f_byte_arr_04[n7] = 2;
                    ((a)object).f_int_arr_12[n7] = 8;
                }
                this.scriptCursor = n + 1;
            } else if (string.equals("MVS")) {
                int n8;
                object = this;
                this.f_short_arr_04 = null;
                ((a)object).f_short_arr_04 = new short[32];
                n2 = 0;
                int n9 = 0;
                n3 = 0;
                n4 = ((a)object).f_int_45;
                block22: while (--n4 >= 0) {
                    n2 = ((a)object).entityType[n4];
                    if (n2 != 82) continue;
                    n9 = ((a)object).entityParam[n4] & 0xFF;
                    for (n8 = 0; n8 < 32; n8 += 2) {
                        n3 = ((a)object).f_short_arr_04[n8] - 1;
                        if (n3 > 0) {
                            if (n9 != (((a)object).entityParam[n3] & 0xFF)) continue;
                            ((a)object).f_short_arr_04[n8 + 1] = n4 + 1;
                            continue block22;
                        }
                        ((a)object).f_short_arr_04[n8] = n4 + 1;
                        continue block22;
                    }
                }
                for (int i = 0; i < 32; i += 2) {
                    int n10;
                    int n11;
                    n3 = ((a)object).f_short_arr_04[i];
                    int n12 = ((a)object).f_short_arr_04[i + 1];
                    if (n3 <= 0) break;
                    if (n12 <= 0) continue;
                    --n12;
                    n8 = ((a)object).entityPixelX[--n3] >> 5;
                    n4 = ((a)object).entityPixelY[n3] >> 5;
                    n9 = ((a)object).f_byte_arr2_02[n4][n8];
                    int n13 = ((a)object).f_byte_arr_10[n9];
                    for (n11 = 0; n11 < n13; ++n11) {
                        n10 = ((a)object).f_byte_arr2_03[n9][n11] - 1;
                        n2 = ((a)object).entityType[n10];
                        if (((a)object).f_byte_arr_03[n2] != 8) continue;
                        super.m_049(((a)object).entityPixelX[n12] >> 5, ((a)object).entityPixelY[n12] >> 5, 82);
                        super.m_049(n8, n4, 82);
                        super.m_107(n10, ((a)object).entityPixelX[n12] >> 5, ((a)object).entityPixelY[n12] >> 5);
                    }
                    n9 = n3;
                    n3 = n12;
                    n12 = n9;
                    n8 = ((a)object).entityPixelX[n3] >> 5;
                    n4 = ((a)object).entityPixelY[n3] >> 5;
                    n9 = ((a)object).f_byte_arr2_02[n4][n8];
                    n13 = ((a)object).f_byte_arr_10[n9];
                    for (n11 = 0; n11 < n13; ++n11) {
                        n10 = ((a)object).f_byte_arr2_03[n9][n11] - 1;
                        n2 = ((a)object).entityType[n10];
                        if (((a)object).f_byte_arr_03[n2] != 8) continue;
                        super.m_049(((a)object).entityPixelX[n12] >> 5, ((a)object).entityPixelY[n12] >> 5, 82);
                        super.m_049(n8, n4, 82);
                        super.m_107(n10, ((a)object).entityPixelX[n12] >> 5, ((a)object).entityPixelY[n12] >> 5);
                    }
                }
                this.scriptCursor = n + 1;
            } else if (string.equals("LAY")) {
                this.m_119(this.currentFloor);
                this.f_byte_23 = (byte)this.parseScriptInt((String)object, n + 1, " ");
                this.scriptCursor = this.f_int_122 + 1;
                this.f_bool_16 = true;
                this.f_String_02 = null;
                this.f_byte_16 = (byte)5;
            } else if (string.equals("ROS")) {
                n2 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                switch (n2) {
                    case 1: {
                        this.playerPixelX = this.parseScriptInt((String)object, n + 1, "_") << 5;
                        n = this.f_int_122;
                        this.playerPixelY = this.parseScriptInt((String)object, n + 1, " ") << 5;
                        this.m_104(0);
                        break;
                    }
                    case 2: {
                        this.m_082(this.parseScriptInt((String)object, n + 1, " "), true);
                        break;
                    }
                    case 3: {
                        this.playerHp = this.parseScriptInt((String)object, n + 1, " ");
                        break;
                    }
                    case 4: {
                        this.f_byte_12 = (byte)this.parseScriptInt((String)object, n + 1, " ");
                        break;
                    }
                    case 5: {
                        this.playerAtk = this.parseScriptInt((String)object, n + 1, " ");
                        break;
                    }
                    case 6: {
                        this.playerDef = this.parseScriptInt((String)object, n + 1, " ");
                    }
                }
                this.scriptCursor = this.f_int_122 + 1;
            } else if (string.equals("CES")) {
                int n14;
                int n15 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                n3 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                n4 = this.parseScriptInt((String)object, n + 1, " ");
                n2 = this.m_048(n15, n3 << 5, n4 << 5, 0);
                this.m_046(n2);
                object = this;
                this.f_int_123 = (((a)object).f_int_58 - 32 >> 1) - ((a)object).f_int_56;
                ((a)object).f_int_124 = (((a)object).f_int_59 - 32 >> 1) - ((a)object).f_int_57;
                if (n2 >= 0) {
                    ((a)object).f_int_125 = ((a)object).entityPixelX[n2];
                    ((a)object).f_int_126 = ((a)object).entityPixelY[n2];
                }
                n2 = n15;
                object = this;
                switch (((a)object).f_byte_arr_03[n2]) {
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
                this.f_int_118 = n14;
                this.scriptCursor = this.f_int_122 + 1;
            } else if (string.equals("GIN")) {
                n2 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                int n16 = this.parseScriptInt((String)object, n + 1, " ");
                switch (n2) {
                    case 0: {
                        this.pickupItemType(n16);
                        break;
                    }
                    case 1: {
                        this.gainGold(n16, this.playerPixelX, this.playerPixelY);
                    }
                }
                this.scriptCursor = this.f_int_122 + 1;
                this.executeScriptInstruction(this.f_String_05, this.scriptCursor);
            } else if (string.equals("ADD")) {
                n2 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                int n17 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                n3 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                n4 = this.parseScriptInt((String)object, n + 1, " ");
                boolean bl2 = false;
                n4 <<= 5;
                n3 <<= 5;
                object = this;
                if (((a)object).currentFloor != n2) {
                    super.m_119(((a)object).currentFloor);
                    super.m_122(n2);
                }
                super.m_048(n17, n3, n4, 0);
                super.m_119(n2);
                if (((a)object).currentFloor != n2) {
                    super.m_059();
                    super.m_122(((a)object).currentFloor);
                }
                this.scriptCursor = this.f_int_122 + 1;
            } else if (string.equals("GLV")) {
                this.parseScriptInt((String)object, n + 1, " ");
                this.scriptCursor = this.f_int_122 + 1;
            } else if (string.equals("RES")) {
                this.parseScriptInt((String)object, n + 1, " ");
                if (this.f_byte_26 == 0) {
                    object = this;
                    ((a)object).blueKeyCount = 0;
                    ((a)object).redKeyCount = 0;
                    this.yellowKeyCount = 0;
                    ((a)object).itemStackSize = 0;
                    ((a)object).goldAmount = 4;
                    ((a)object).alchemyUpgradeCount = 0;
                    ((a)object).minFloorReached = 1;
                    ((a)object).maxFloorReached = 3;
                    ((a)object).playerAtk = 10;
                    ((a)object).playerDef = 10;
                    ((a)object).playerHp = 400;
                    ((a)object).f_int_89 = 0;
                }
                this.scriptCursor = this.f_int_122 + 1;
            } else if (string.equals("SEE")) {
                this.f_int_114 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                this.f_int_115 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                this.f_int_120 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                this.f_int_121 = this.parseScriptInt((String)object, n + 1, "_");
                n = this.f_int_122;
                this.f_int_117 = this.parseScriptInt((String)object, n + 1, " ");
                this.scriptCursor = this.f_int_122 + 1;
                this.m_096(this.f_int_120);
                switch (this.f_int_117) {
                    case 0: {
                        break;
                    }
                    case 1: {
                        if (!this.m_133(this.playerCellX, this.playerCellY, this.f_int_114, this.f_int_115)) break;
                        this.f_bool_26 = true;
                        this.f_byte_11 = 0;
                    }
                }
                this.f_byte_16 = (byte)6;
                this.m_103(this.f_int_114, this.f_int_115);
            } else if (string.equals("END")) {
                this.parseScriptInt((String)object, n + 1, " ");
                this.gameMode = (byte)20;
                this.m_000();
                this.scriptCursor = this.f_int_122 + 1;
            } else if (string.equals("SMS")) {
                this.parseScriptInt((String)object, n + 1, " ");
                this.scriptCursor = this.f_int_122 + 1;
            }
            if (this.scriptCursor < this.f_String_05.length()) {
                return;
            }
        } else {
            this.f_byte_16 = (byte)4;
            if (this.currentScriptIndex != 32) {
                this.f_bool_arr_06[this.currentScriptIndex] = true;
            }
            this.m_104(0);
        }
    }

    private int parseScriptInt(String string, int n, String string2) {
        int n2 = 0;
        n2 = string.indexOf(string2, n);
        string = string.substring(n, n2);
        this.f_int_122 = n2;
        n2 = Integer.parseInt(string);
        return n2;
    }

    private int m_100(int n, int n2, int n3) {
        n = this.f_byte_arr2_02[n2][n];
        n2 = this.f_byte_arr_10[n];
        int n4 = 0;
        if (n2 > 0) {
            for (int i = 0; i < n2; ++i) {
                n4 = this.f_byte_arr2_03[n][i] - 1;
                if (this.entityType[n4] != n3 || this.f_byte_arr_04[n4] == 1) continue;
                return n4;
            }
        }
        return -1;
    }

    private void m_101(int n, int n2, int n3) {
        if (this.isCellWalkable(n, --n2) && !this.m_062(n, n2) && (n != this.playerCellX || n2 != this.playerCellY)) {
            this.m_051(n3);
            n3 = this.m_048(60, n << 5, n2 << 5, 0);
            this.m_046(n3);
        }
    }

    private boolean m_102(int n, int n2) {
        boolean bl = false;
        if (this.currentFloor != n) {
            this.m_119(this.currentFloor);
            this.m_122(n);
        }
        for (int i = 0; i < this.f_int_45; ++i) {
            if (this.f_bool_arr_01[i] || this.entityType[i] != n2) continue;
            bl = true;
            break;
        }
        if (this.currentFloor != n) {
            this.m_059();
            this.m_122(this.currentFloor);
        }
        return bl;
    }

    private void m_103(int n, int n2) {
        this.f_int_123 = (this.f_int_58 - 32 >> 1) - this.f_int_56;
        this.f_int_124 = (this.f_int_59 - 32 >> 1) - this.f_int_57;
        this.f_int_125 = n << 5;
        this.f_int_126 = n2 << 5;
    }

    private int m_104(int n) {
        int n2;
        int n3;
        block4: {
            this.f_int_123 = (this.f_int_58 - 32 >> 1) - this.f_int_56;
            this.f_int_124 = (this.f_int_59 - 32 >> 1) - this.f_int_57;
            if (n == 0 || n == 87) {
                this.f_int_125 = this.playerPixelX;
                this.f_int_126 = this.playerPixelY;
                return -1;
            }
            int n4 = n;
            a a2 = this;
            int n5 = a2.f_int_45;
            while (--n5 >= 0) {
                if (a2.entityType[n5] != n4) continue;
                n3 = n5;
                break block4;
            }
            n3 = n2 = -1;
        }
        if (n3 >= 0) {
            this.f_int_125 = this.entityPixelX[n2];
            this.f_int_126 = this.entityPixelY[n2];
        }
        if (n == 69) {
            this.f_int_125 += 32;
        }
        return n2;
    }

    private void m_105() {
        if (this.f_int_123 < this.f_int_125) {
            this.f_int_123 += (this.f_int_125 - this.f_int_123 >> 2) + 2;
            if (this.f_int_123 > this.f_int_125) {
                this.f_int_123 = this.f_int_125;
            }
        } else if (this.f_int_123 > this.f_int_125) {
            this.f_int_123 += (this.f_int_125 - this.f_int_123 >> 2) - 2;
            if (this.f_int_123 < this.f_int_125) {
                this.f_int_123 = this.f_int_125;
            }
        }
        if (this.f_int_124 < this.f_int_126) {
            this.f_int_124 += (this.f_int_126 - this.f_int_124 >> 2) + 2;
            if (this.f_int_124 > this.f_int_126) {
                this.f_int_124 = this.f_int_126;
            }
        } else if (this.f_int_124 > this.f_int_126) {
            this.f_int_124 += (this.f_int_126 - this.f_int_124 >> 2) - 2;
            if (this.f_int_124 < this.f_int_126) {
                this.f_int_124 = this.f_int_126;
            }
        }
        this.m_064((this.f_int_58 - 32 >> 1) - this.f_int_123, (this.f_int_59 - 32 >> 1) - this.f_int_124);
    }

    private void m_106() {
        switch (this.f_byte_18) {
            case 5: {
                this.tickBattle(this.f_bool_14);
                if (this.f_bool_23 || !this.f_bool_arr_01[this.f_int_127] && this.f_byte_arr_04[this.f_int_127] != 1) break;
                this.f_byte_16 = 0;
                this.f_byte_18 = 0;
                this.f_int_127 = -1;
                return;
            }
            case 0: {
                if (this.f_int_148 > 0) {
                    this.f_byte_19 = this.f_byte_arr_42[--this.f_int_148];
                    a a2 = this;
                    int n = 0;
                    int n2 = 0;
                    int n3 = 0;
                    int n4 = 0;
                    n = a2.entityPixelX[a2.f_int_127] >> 5;
                    n2 = a2.entityPixelY[a2.f_int_127] >> 5;
                    switch (a2.f_byte_19) {
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
                    a2.f_byte_18 = 1;
                    if (a2.entityType[a2.f_int_127] != 72 && n == a2.playerCellX && n2 == a2.playerCellY) {
                        n3 = a2.entityType[a2.f_int_127];
                        n4 = a2.predictHpLossVsType(n3, false);
                        if (n4 >= 0 && n4 < a2.playerHp) {
                            a2.f_int_150 = a2.enemyHpScaled[n3 - 41];
                            a2.f_byte_11 = (byte)5;
                            a2.f_bool_14 = false;
                            a2.f_int_44 = a2.f_int_127;
                            a2.f_byte_arr_04[a2.f_int_127] = 5;
                            a2.f_byte_18 = (byte)5;
                            a2.f_bool_25 = false;
                            n4 = a2.enemyAtkScaled[n3 - 41] - a2.playerDef;
                            if (n4 > 0) {
                                a2.playerHp -= n4;
                            }
                        } else {
                            a2.f_byte_11 = 1;
                            a2.f_byte_12 = a2.f_byte_19;
                        }
                    }
                    if (a2.f_byte_arr_10[n = a2.f_byte_arr2_02[n2][n]] > 0) {
                        n4 = a2.f_byte_arr_10[n];
                        while (--n4 >= 0) {
                            n2 = a2.f_byte_arr2_03[n][n4] - 1;
                            n3 = a2.entityType[n2];
                            switch (a2.f_byte_arr_03[n3]) {
                                case 1: {
                                    switch (n3) {
                                        case 11: {
                                            a2.m_047(n2);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return;
                }
                this.m_108(this.f_int_127, this.f_int_129, this.f_int_130);
                this.f_byte_16 = 0;
                this.m_045(this.entityType[this.f_int_127], this.f_int_127);
                this.f_int_127 = -1;
                return;
            }
            case 1: {
                switch (this.f_byte_19) {
                    case 1: {
                        int n = this.f_int_127;
                        this.entityPixelY[n] = this.entityPixelY[n] - 8;
                        break;
                    }
                    case 0: {
                        int n = this.f_int_127;
                        this.entityPixelY[n] = this.entityPixelY[n] + 8;
                        break;
                    }
                    case 3: {
                        int n = this.f_int_127;
                        this.entityPixelX[n] = this.entityPixelX[n] - 8;
                        break;
                    }
                    case 2: {
                        int n = this.f_int_127;
                        this.entityPixelX[n] = this.entityPixelX[n] + 8;
                    }
                }
                this.f_int_125 = this.entityPixelX[this.f_int_127];
                this.f_int_126 = this.entityPixelY[this.f_int_127];
                this.f_int_128 += 8;
                int n = this.f_int_arr_12[this.f_int_127];
                if (n > 0) {
                    if (this.f_int_arr_10[this.f_int_127] < this.f_int_arr2_02[n].length - 1) {
                        int n5 = this.f_int_127;
                        this.f_int_arr_10[n5] = this.f_int_arr_10[n5] + 1;
                    } else {
                        this.f_int_arr_10[this.f_int_127] = 0;
                    }
                }
                if (this.f_int_128 != 32) break;
                this.f_int_128 = 0;
                this.f_byte_18 = 0;
                a a3 = this;
                int n6 = a3.entityPixelX[a3.f_int_127] >> 5;
                int n7 = a3.entityPixelY[a3.f_int_127] >> 5;
                int n8 = 0;
                if (n6 == a3.playerCellX && n7 == a3.playerCellY && a3.entityType[a3.f_int_127] != 72) {
                    n8 = a3.predictHpLossVsType(a3.entityType[a3.f_int_127], false);
                    if (n8 < 0 || n8 >= a3.playerHp) {
                        a3.f_bool_27 = false;
                        a3.gameMode = (byte)3;
                        a3.f_byte_11 = 1;
                        a3.f_byte_12 = a3.f_byte_19;
                    } else {
                        a3.playerHp -= n8;
                        a3.m_108(a3.f_int_127, a3.f_int_129, a3.f_int_130);
                        a3.m_049(n6, n7, a3.entityType[a3.f_int_127]);
                    }
                }
                this.m_106();
            }
        }
    }

    private void m_107(int n, int n2, int n3) {
        int n4 = this.entityPixelX[n];
        int n5 = this.entityPixelY[n];
        short s = this.entityParam[n];
        this.m_048(this.entityType[n], n2 << 5, n3 << 5, s);
        this.m_049(n4 >> 5, n5 >> 5, this.entityType[n]);
    }

    private void m_108(int n, int n2, int n3) {
        int n4;
        int n5;
        n2 = this.f_byte_arr2_02[n3][n2];
        n3 = this.f_byte_arr_10[n2];
        for (n5 = 0; n5 < n3; ++n5) {
            if (this.f_byte_arr2_03[n2][n5] - 1 != n) continue;
            for (n4 = n5; n4 < n3 - 1; ++n4) {
                this.f_byte_arr2_03[n2][n4] = this.f_byte_arr2_03[n2][n4 + 1];
            }
            int n6 = n2;
            this.f_byte_arr_10[n6] = this.f_byte_arr_10[n6] - 1;
            break;
        }
        if ((n2 = this.f_byte_arr2_02[n4 = this.entityPixelY[n] >> 5][n5 = this.entityPixelX[n] >> 5]) == 0) {
            this.f_byte_arr2_02[n4][n5] = this.f_byte_15 = (byte)(this.f_byte_15 + 1);
            n2 = this.f_byte_15;
        }
        this.f_byte_arr2_03[n2][this.f_byte_arr_10[n2]] = n + 1;
        int n7 = n2;
        this.f_byte_arr_10[n7] = this.f_byte_arr_10[n7] + 1;
    }

    private void spawnBossEvent(int n) {
        byte[] byArray = this.bossEventSpawns[n];
        int n2 = byArray.length;
        int n3 = 0;
        for (int i = 0; i < n2; i += 4) {
            n3 = this.m_048(byArray[i], byArray[i + 1] << 5, byArray[i + 2] << 5, byArray[i + 3]);
            this.m_046(n3);
        }
    }

    private void m_110() {
        String string = "SKY_WAR";
        boolean bl = false;
        try {
            int n;
            this.f_RecordStore_00 = RecordStore.openRecordStore((String)string, (boolean)true);
            if (this.f_RecordStore_00.getNumRecords() == 0) {
                bl = true;
            }
            this.f_ByteArrayOutputStream_00 = new ByteArrayOutputStream();
            this.f_DataOutputStream_00 = new DataOutputStream(this.f_ByteArrayOutputStream_00);
            for (n = 0; n < 4; ++n) {
                this.f_DataOutputStream_00.writeBoolean(this.f_bool_arr_05[n]);
            }
            this.f_DataOutputStream_00.writeByte(this.f_int_arr_34.length);
            for (n = 0; n < this.f_int_arr_34.length; ++n) {
                this.f_DataOutputStream_00.writeInt(this.f_int_arr_35[n]);
            }
            this.f_DataOutputStream_00.writeInt(this.f_int_152);
            this.f_DataOutputStream_00.writeInt(this.f_int_153);
            this.f_DataOutputStream_00.writeInt(this.f_int_154);
            if (!bl) {
                this.f_RecordStore_00.setRecord(1, this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
                return;
            }
            this.f_RecordStore_00.addRecord(this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
        }
        catch (Exception exception) {}
    }

    private void m_111() {
        String string = "MOT_IF";
        if (!this.f_bool_15) {
            this.f_bool_arr_07 = new boolean[6];
            this.f_byte_arr_24 = new byte[6];
            this.f_int_arr_23 = new int[6];
            this.f_int_arr_24 = new int[6];
            this.f_int_arr_25 = new int[6];
            this.f_int_arr_26 = new int[6];
            this.f_int_arr_27 = new int[6];
            this.f_int_arr_28 = new int[6];
            this.f_int_arr_29 = new int[6];
            this.f_byte_arr_25 = new byte[6];
            this.f_byte_arr_26 = new byte[6];
            this.f_bool_15 = true;
        }
        try {
            this.f_RecordStore_00 = RecordStore.openRecordStore((String)string, (boolean)false);
            this.f_RecordEnumeration_00 = this.f_RecordStore_00.enumerateRecords(null, null, false);
            int n = this.f_RecordEnumeration_00.nextRecordId();
            this.f_byte_arr_27 = this.f_RecordStore_00.getRecord(n);
            this.f_DataInputStream_00 = new DataInputStream(new ByteArrayInputStream(this.f_byte_arr_27));
            for (n = 0; n < 6; ++n) {
                boolean bl = this.f_bool_arr_07[n] = this.f_DataInputStream_00.readByte() != 0;
                if (!this.f_bool_arr_07[n]) continue;
                this.f_byte_arr_24[n] = this.f_DataInputStream_00.readByte();
                this.f_byte_arr_25[n] = this.f_DataInputStream_00.readByte();
                this.f_byte_arr_26[n] = this.f_DataInputStream_00.readByte();
                this.f_int_arr_23[n] = this.f_DataInputStream_00.readInt();
                this.f_int_arr_24[n] = this.f_DataInputStream_00.readInt();
                this.f_int_arr_25[n] = this.f_DataInputStream_00.readInt();
                this.f_int_arr_26[n] = this.f_DataInputStream_00.readInt();
                this.f_int_arr_27[n] = this.f_DataInputStream_00.readInt();
                this.f_int_arr_28[n] = this.f_DataInputStream_00.readInt();
                this.f_int_arr_29[n] = this.f_DataInputStream_00.readInt();
            }
            return;
        }
        catch (Exception exception) {
            this.m_112(-1);
            return;
        }
        finally {
            this.m_117();
        }
    }

    private void m_112(int n) {
        if (!this.f_bool_15) {
            this.m_111();
        }
        if (n >= 0 && n < 6) {
            this.f_bool_arr_07[n] = true;
            this.f_byte_arr_24[n] = this.currentFloor;
            this.f_int_arr_23[n] = this.playerHp;
            this.f_int_arr_24[n] = this.playerAtk;
            this.f_int_arr_25[n] = this.playerDef;
            this.f_int_arr_26[n] = this.yellowKeyCount;
            this.f_int_arr_27[n] = this.blueKeyCount;
            this.f_int_arr_28[n] = this.redKeyCount;
            this.f_int_arr_29[n] = this.goldAmount;
            this.f_byte_arr_25[n] = this.equippedWeaponType;
            this.f_byte_arr_26[n] = this.equippedArmorType;
        }
        Object object = "MOT_IF";
        a.m_116("MOT_IF");
        try {
            this.f_RecordStore_00 = RecordStore.openRecordStore((String)object, (boolean)true);
            this.f_ByteArrayOutputStream_00 = new ByteArrayOutputStream();
            this.f_DataOutputStream_00 = new DataOutputStream(this.f_ByteArrayOutputStream_00);
            for (int i = 0; i < 6; ++i) {
                if (this.f_bool_arr_07[i]) {
                    this.f_DataOutputStream_00.write(1);
                    this.f_DataOutputStream_00.writeByte(this.f_byte_arr_24[i]);
                    this.f_DataOutputStream_00.writeByte(this.f_byte_arr_25[i]);
                    this.f_DataOutputStream_00.writeByte(this.f_byte_arr_26[i]);
                    this.f_DataOutputStream_00.writeInt(this.f_int_arr_23[i]);
                    this.f_DataOutputStream_00.writeInt(this.f_int_arr_24[i]);
                    this.f_DataOutputStream_00.writeInt(this.f_int_arr_25[i]);
                    this.f_DataOutputStream_00.writeInt(this.f_int_arr_26[i]);
                    this.f_DataOutputStream_00.writeInt(this.f_int_arr_27[i]);
                    this.f_DataOutputStream_00.writeInt(this.f_int_arr_28[i]);
                    this.f_DataOutputStream_00.writeInt(this.f_int_arr_29[i]);
                    continue;
                }
                this.f_DataOutputStream_00.write(0);
            }
            this.f_RecordStore_00.addRecord(this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
            return;
        }
        catch (Exception exception) {
            object = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.m_117();
        }
    }

    private void m_113(boolean n) {
        n = n != 0 ? 2 : 1;
        int n2 = 240 - this.f_int_135 - 22 >> 1;
        int n3 = 320 - this.f_int_131 >> 1;
        Image image = null;
        this.m_038(n, n2, n3, this.f_int_135 + 22, this.f_int_131);
        n3 += 16;
        n = this.f_int_132;
        while (n < this.f_int_133) {
            if (n != this.f_int_134) {
                this.f_Graphics_00.setColor(7574946);
            } else {
                this.f_Graphics_00.setColor(3156024);
                this.f_Graphics_00.fillRect(120 - (this.f_int_135 >> 1), n3, this.f_int_135, this.f_int_01 + 4);
                this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][14], 120 - (this.f_int_135 >> 1) + 10, n3 + (this.f_int_01 - 8 >> 1), 0);
                this.m_004(this.f_Image_arr2_00[8][14], 120 + (this.f_int_135 >> 1) - 30, n3 + (this.f_int_01 - 8 >> 1), 1);
                this.f_Graphics_00.setColor(16377897);
            }
            if (!this.f_bool_arr_07[n]) {
                this.f_Graphics_00.drawString("---", 120, n3 + 2, 17);
            } else {
                this.f_Graphics_00.drawString("\u5b58\u6863" + n, 120, n3 + 2, 17);
            }
            ++n;
            n3 += this.f_int_01 + 4;
        }
        n2 = 120 + ((this.f_int_135 >> 1) - 10);
        if (this.f_int_132 > 0) {
            this.m_002(this.f_Image_arr2_00[8][15], n2, n3 - 20, 0, 0, 7, 9);
        }
        if (this.f_int_133 < 6) {
            this.m_002(this.f_Image_arr2_00[8][15], n2, n3 - 10, 7, 0, 7, 9);
        }
        this.f_Graphics_00.setColor(6178);
        this.f_Graphics_00.drawLine(n2 -= this.f_int_135 - 10, n3 += 2, n2 + this.f_int_135 - 1, n3);
        this.f_Graphics_00.setColor(3564144);
        this.f_Graphics_00.drawLine(n2, ++n3, n2 + this.f_int_135 - 1, n3);
        if (this.f_bool_arr_07[this.f_int_134]) {
            n2 += 6;
            n3 += 8;
            if (this.f_byte_arr_24[this.f_int_134] < 51) {
                this.m_042(this.f_Image_arr2_00[8][18], this.f_byte_arr_24[this.f_int_134], n2 + 48, n3);
                this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][8], n2 + 50, n3 + 8, 0);
            }
            n2 = 240 + this.f_int_135 >> 1;
            this.m_041(n2 -= 68, n3, 32, 32);
            if (this.f_byte_arr_25[this.f_int_134] == 0) {
                this.f_Graphics_00.setColor(-1);
                this.f_Graphics_00.drawString(this.f_String_arr_04[this.f_byte_arr_25[this.f_int_134]], n2 + (32 - this.f_int_00 >> 1), n3 + (32 - this.f_int_01 >> 1), 0);
            } else {
                n = this.f_byte_arr_25[this.f_int_134] - 33;
                if (n > 7) {
                    n = 8;
                }
                image = this.f_Image_arr2_00[6][n];
                this.f_Graphics_00.drawImage(image, n2 + (32 - image.getWidth() >> 1), n3 + (32 - image.getHeight() >> 1), 0);
            }
            this.m_041(n2 += 34, n3, 32, 32);
            if (this.f_byte_arr_26[this.f_int_134] == 0) {
                this.f_Graphics_00.setColor(-1);
                this.f_Graphics_00.drawString(this.f_String_arr_04[this.f_byte_arr_26[this.f_int_134]], n2 + (32 - this.f_int_00 >> 1), n3 + (32 - this.f_int_01 >> 1), 0);
            } else {
                n = this.f_byte_arr_26[this.f_int_134] - 33;
                if (n > 7) {
                    n = 9;
                }
                image = this.f_Image_arr2_00[6][n];
                this.f_Graphics_00.drawImage(image, n2 + (32 - image.getWidth() >> 1), n3 + (32 - image.getHeight() >> 1), 0);
            }
            n2 = 240 - this.f_int_135 + 10 >> 1;
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][1], n2, n3 += 20, 0);
            this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_29[this.f_int_134], n2 += 60, n3 + 2);
            this.m_002(this.f_Image_arr2_00[8][7], n2 -= 64, n3 += 16, 0, 2, 10, 10);
            this.m_041((n2 += 15) - 1, n3, 48, 12);
            this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_23[this.f_int_134], n2 + 45, n3 + 2);
            this.m_002(this.f_Image_arr2_00[8][7], n2 += 47, n3, 10, 0, 10, 13);
            this.f_Graphics_00.setColor(512);
            this.m_041(n2 += 11, n3, 30, 12);
            this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_24[this.f_int_134], n2 + 28, n3 + 2);
            this.m_002(this.f_Image_arr2_00[8][7], n2 += 32, n3, 20, 2, 10, 10);
            this.f_Graphics_00.setColor(512);
            this.m_041(n2 += 12, n3, 30, 12);
            this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_25[this.f_int_134], n2 + 28, n3 + 2);
            n2 = (240 - this.f_int_135 >> 1) + 6;
            this.m_036(0, this.f_int_arr_26[this.f_int_134], n2, n3 += 16);
            this.m_036(1, this.f_int_arr_27[this.f_int_134], n2 += 42, n3);
            this.m_036(2, this.f_int_arr_28[this.f_int_134], n2 += 42, n3);
        }
    }

    private void m_114(int n) {
        this.m_112(n);
        Object object = "MOT_L" + n;
        a.m_116((String)object);
        try {
            int n2;
            this.f_RecordStore_00 = RecordStore.openRecordStore((String)object, (boolean)true);
            this.f_ByteArrayOutputStream_00 = new ByteArrayOutputStream();
            this.f_DataOutputStream_00 = new DataOutputStream(this.f_ByteArrayOutputStream_00);
            this.f_DataOutputStream_00.writeByte(this.f_byte_26);
            this.f_DataOutputStream_00.writeByte(this.currentFloor);
            this.f_DataOutputStream_00.writeByte(this.minFloorReached);
            this.f_DataOutputStream_00.writeByte(this.maxFloorReached);
            this.f_DataOutputStream_00.writeByte(this.f_byte_12);
            this.f_DataOutputStream_00.writeByte(this.equippedWeaponType);
            this.f_DataOutputStream_00.writeByte(this.equippedArmorType);
            this.f_DataOutputStream_00.writeShort(this.playerCellX);
            this.f_DataOutputStream_00.writeShort(this.playerCellY);
            this.f_DataOutputStream_00.writeInt(this.playerHp);
            this.f_DataOutputStream_00.writeInt(this.playerAtk);
            this.f_DataOutputStream_00.writeInt(this.playerDef);
            this.f_DataOutputStream_00.writeShort(this.yellowKeyCount);
            this.f_DataOutputStream_00.writeShort(this.blueKeyCount);
            this.f_DataOutputStream_00.writeShort(this.redKeyCount);
            this.f_DataOutputStream_00.writeInt(this.goldAmount);
            this.f_DataOutputStream_00.writeShort(this.alchemyUpgradeCount);
            this.f_DataOutputStream_00.writeShort(this.f_int_89);
            for (n2 = 0; n2 < this.f_int_89; ++n2) {
                this.f_DataOutputStream_00.writeByte(this.f_byte_arr_14[n2]);
                this.f_DataOutputStream_00.writeByte(this.f_byte_arr_15[n2]);
            }
            for (n2 = 0; n2 < 128; ++n2) {
                if (n2 < this.f_int_113) {
                    this.f_DataOutputStream_00.writeBoolean(this.f_bool_arr_06[n2]);
                    continue;
                }
                this.f_DataOutputStream_00.writeBoolean(false);
            }
            this.f_DataOutputStream_00.writeByte(this.itemStackSize);
            for (n2 = 0; n2 < this.itemStackSize; ++n2) {
                this.f_DataOutputStream_00.writeByte(this.itemStackTypes[n2]);
                this.f_DataOutputStream_00.writeByte(this.itemStackUses[n2]);
            }
            for (n2 = 0; n2 < 56; ++n2) {
                if (this.f_byte_arr2_05[n2] != null) {
                    this.f_DataOutputStream_00.writeShort(this.f_byte_arr2_05[n2].length);
                    this.f_DataOutputStream_00.write(this.f_byte_arr2_05[n2], 0, this.f_byte_arr2_05[n2].length);
                    continue;
                }
                this.f_DataOutputStream_00.writeShort(-1);
            }
            this.f_RecordStore_00.addRecord(this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
            return;
        }
        catch (Exception exception) {
            object = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.m_117();
        }
    }

    private boolean m_115(int n) {
        boolean bl = true;
        short s = 0;
        Object object = "MOT_L" + n;
        try {
            this.f_RecordStore_00 = RecordStore.openRecordStore((String)object, (boolean)false);
            this.f_RecordEnumeration_00 = this.f_RecordStore_00.enumerateRecords(null, null, false);
            int n2 = this.f_RecordEnumeration_00.nextRecordId();
            this.f_byte_arr_27 = this.f_RecordStore_00.getRecord(n2);
            this.f_DataInputStream_00 = new DataInputStream(new ByteArrayInputStream(this.f_byte_arr_27));
            this.f_byte_26 = this.f_DataInputStream_00.readByte();
            this.currentFloor = this.f_DataInputStream_00.readByte();
            this.minFloorReached = this.f_DataInputStream_00.readByte();
            this.maxFloorReached = this.f_DataInputStream_00.readByte();
            this.f_byte_12 = this.f_DataInputStream_00.readByte();
            this.equippedWeaponType = this.f_DataInputStream_00.readByte();
            this.equippedArmorType = this.f_DataInputStream_00.readByte();
            this.playerCellX = this.f_DataInputStream_00.readShort();
            this.playerCellY = this.f_DataInputStream_00.readShort();
            this.playerPixelX = this.playerCellX << 5;
            this.playerPixelY = this.playerCellY << 5;
            this.playerHp = this.f_DataInputStream_00.readInt();
            this.playerAtk = this.f_DataInputStream_00.readInt();
            this.playerDef = this.f_DataInputStream_00.readInt();
            this.yellowKeyCount = this.f_DataInputStream_00.readShort();
            this.blueKeyCount = this.f_DataInputStream_00.readShort();
            this.redKeyCount = this.f_DataInputStream_00.readShort();
            this.goldAmount = this.f_DataInputStream_00.readInt();
            this.alchemyUpgradeCount = this.f_DataInputStream_00.readShort();
            this.f_int_43 = 0;
            this.f_int_89 = this.f_DataInputStream_00.readShort();
            for (n2 = 0; n2 < this.f_int_89; ++n2) {
                this.f_byte_arr_14[n2] = this.f_DataInputStream_00.readByte();
                this.f_byte_arr_15[n2] = this.f_DataInputStream_00.readByte();
            }
            for (n2 = 0; n2 < this.f_int_113; ++n2) {
                this.f_bool_arr_06[n2] = this.f_DataInputStream_00.readBoolean();
            }
            if (this.f_int_113 < 128) {
                this.f_DataInputStream_00.skip(128 - this.f_int_113);
            }
            this.f_bool_12 = false;
            this.itemStackSize = this.f_DataInputStream_00.readByte();
            for (n2 = 0; n2 < this.itemStackSize; ++n2) {
                this.itemStackTypes[n2] = this.f_DataInputStream_00.readByte();
                this.itemStackUses[n2] = this.f_DataInputStream_00.readByte();
                if (this.itemStackTypes[n2] != 15) continue;
                this.f_bool_12 = true;
            }
            for (n2 = 0; n2 < 56; ++n2) {
                s = this.f_DataInputStream_00.readShort();
                if (s > 0) {
                    this.f_byte_arr2_05[n2] = new byte[s];
                    this.f_DataInputStream_00.read(this.f_byte_arr2_05[n2], 0, s);
                    continue;
                }
                this.f_byte_arr2_05[n2] = null;
            }
            this.scaleEnemyStats(this.difficultyMultipliers[this.f_byte_26]);
        }
        catch (Exception exception) {
            object = exception;
            exception.printStackTrace();
            bl = false;
        }
        finally {
            this.m_117();
        }
        return bl;
    }

    private static void m_116(String string) {
        try {
            RecordStore.deleteRecordStore((String)string);
            return;
        }
        catch (Exception exception) {
            return;
        }
    }

    private void m_117() {
        if (this.f_RecordStore_00 != null) {
            try {
                this.f_RecordStore_00.closeRecordStore();
            }
            catch (Exception exception) {}
        }
        if (this.f_ByteArrayOutputStream_00 != null) {
            try {
                this.f_ByteArrayOutputStream_00.close();
            }
            catch (Exception exception) {}
        }
        if (this.f_DataOutputStream_00 != null) {
            try {
                this.f_DataOutputStream_00.close();
            }
            catch (Exception exception) {}
        }
        if (this.f_RecordEnumeration_00 != null) {
            this.f_RecordEnumeration_00.destroy();
        }
        this.f_RecordStore_00 = null;
        this.f_ByteArrayOutputStream_00 = null;
        this.f_DataOutputStream_00 = null;
        this.f_RecordEnumeration_00 = null;
        this.f_byte_arr_27 = null;
        System.gc();
    }

    private void m_118() {
        int n = 56;
        while (--n >= 0) {
            this.f_byte_arr2_05[n] = null;
        }
    }

    private void m_119(int n) {
        int n2 = this.f_int_45;
        int n3 = this.f_int_45;
        while (--n3 >= 0) {
            if (!this.f_bool_arr_01[n3] && this.f_byte_arr_04[n3] != 1) continue;
            --n2;
        }
        this.f_byte_arr2_05[n] = null;
        if (this.f_int_45 > 0) {
            this.f_byte_arr2_05[n] = new byte[(n2 << 3) + 2];
            this.f_byte_arr_28 = this.f_byte_arr2_05[n];
            this.f_int_137 = 0;
            this.m_120((short)n2);
            for (n3 = 0; n3 < this.f_int_45; ++n3) {
                if (this.f_bool_arr_01[n3] || this.f_byte_arr_04[n3] == 1) continue;
                this.f_byte_arr_28[this.f_int_137++] = this.entityType[n3];
                this.m_120((short)this.entityPixelX[n3]);
                this.m_120((short)this.entityPixelY[n3]);
                this.f_byte_arr_28[this.f_int_137++] = this.f_bool_arr_00[n3] ? (byte)1 : 0;
                this.m_120(this.entityParam[n3]);
            }
        }
    }

    private void m_120(short s) {
        this.f_byte_arr_28[this.f_int_137++] = (byte)(s >> 8);
        this.f_byte_arr_28[this.f_int_137++] = (byte)s;
    }

    /*
     * Loose catch block
     */
    private void m_121(int n) {
        a a2;
        block10: {
            int n2 = n;
            a2 = this;
            InputStream inputStream = a2.getClass().getResourceAsStream("maplv" + n2);
            a2.mapCellsWide = a.m_058(inputStream) >> 1;
            a2.mapCellsHigh = a.m_058(inputStream) >> 1;
            System.out.println("Width:" + a2.mapCellsWide + ",Height:" + a2.mapCellsHigh);
            int n3 = a2.mapCellsWide * a2.mapCellsHigh << 2;
            a2.f_int_52 = a2.mapCellsWide << 5;
            a2.f_int_53 = a2.mapCellsHigh << 5;
            a2.mapTerrainGrid = new byte[n3];
            a2.mapTransformGrid = new byte[n3];
            inputStream.read(a2.mapTerrainGrid, 0, n3);
            inputStream.read(a2.mapTransformGrid, 0, n3);
            a.m_006(32, 32, 0x55FF00FF);
            a.m_006(32, 32, 0x5500FF00);
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
        a2.f_int_58 = 240;
        a2.f_int_59 = 252;
        a2.f_bool_10 = a2.f_int_58 >= a2.f_int_52;
        a2.f_bool_11 = a2.f_int_59 >= a2.f_int_53;
        a2.m_059();
        v4.f_bool_arr2_00 = new boolean[a2.mapCellsHigh][a2.mapCellsWide];
        a2.m_064(0, 0);
        a2.m_061();
        this.f_int_45 = 0;
        this.f_int_73 = -1;
        this.f_int_71 = -1;
        this.f_int_72 = -1;
        this.f_int_70 = -1;
        this.m_122(n);
        this.m_054();
    }

    private void m_122(int n) {
        int n2;
        boolean bl = false;
        boolean bl2 = false;
        int n3 = 0;
        a a2 = this;
        this.f_int_45 = 0;
        a a3 = a2;
        for (n2 = 1; n2 <= 12; ++n2) {
            a3.f_Image_arr_00[n2] = a3.f_Image_arr2_00[4][n2 - 1];
        }
        for (n2 = 13; n2 <= 32; ++n2) {
            a3.f_Image_arr_00[n2] = a3.f_Image_arr2_00[5][n2 - 13 + 1];
        }
        for (n2 = 33; n2 <= 40; ++n2) {
            a3.f_Image_arr_00[n2] = a3.f_Image_arr2_00[6][n2 - 33];
        }
        for (n2 = 41; n2 < 61; ++n2) {
            a3.f_Image_arr_00[n2] = a3.f_Image_arr2_00[7][n2 - 41];
        }
        for (n2 = 61; n2 < 79; ++n2) {
            a3.f_Image_arr_00[n2] = a3.f_Image_arr2_00[13][n2 - 41 - 20];
        }
        a3.f_Image_arr_00[84] = a3.f_Image_arr2_00[13][18];
        a3.f_Image_arr_00[87] = a3.f_Image_arr2_00[13][19];
        a3.f_Image_arr_00[85] = a3.f_Image_arr2_00[5][21];
        a3.f_Image_arr_00[86] = a3.f_Image_arr2_00[5][22];
        a3.f_Image_arr_00[79] = a3.f_Image_arr2_00[6][8];
        a3.f_Image_arr_00[80] = a3.f_Image_arr2_00[6][9];
        a3.f_Image_arr_00[12] = a3.f_Image_arr_00[11];
        a3.f_Image_arr_00[81] = a3.f_Image_arr2_00[4][3];
        a3.f_Image_arr_00[71] = a3.f_Image_arr_00[70];
        this.f_int_45 = 0;
        if (this.f_byte_arr2_05[n] == null) {
            try {
                int n4 = n;
                a2 = this;
                InputStream inputStream = a2.getClass().getResourceAsStream("sprite" + n4);
                boolean bl3 = false;
                int n5 = 0;
                try {
                    InputStream inputStream2 = inputStream;
                    a a4 = a2;
                    int n6 = a.m_058(inputStream2) & 0xFFFF | a.m_058(inputStream2) << 16;
                    block27: for (int i = 0; i < n6; ++i) {
                        int n7 = inputStream.read();
                        n3 = a.m_058(inputStream);
                        short s = a.m_058(inputStream);
                        switch (n7) {
                            case 6: 
                            case 12: {
                                n7 = a2.m_048(n7, n3, s, 0);
                                a2.f_bool_arr_00[n7] = false;
                                continue block27;
                            }
                            case 9: {
                                n5 = inputStream.read();
                                a2.m_048(n7, n3, s, n5);
                                continue block27;
                            }
                            case 4: {
                                n5 = inputStream.read() + 1;
                                a2.m_048(n7, n3, s, n5);
                                continue block27;
                            }
                            case 5: 
                            case 81: {
                                n5 = inputStream.read() + 1;
                                n7 = a2.m_048(n7, n3, s, n5);
                                a2.f_bool_arr_00[n7] = false;
                                continue block27;
                            }
                            case 83: {
                                n5 = inputStream.read();
                                n7 = a2.m_048(n7, n3, s, n5);
                                a2.f_bool_arr_00[n7] = false;
                                continue block27;
                            }
                            case 7: 
                            case 8: {
                                n5 = inputStream.read() | inputStream.read() << 8;
                                a2.m_048(n7, n3, s, n5);
                                continue block27;
                            }
                            case 57: 
                            case 59: 
                            case 70: 
                            case 71: 
                            case 72: 
                            case 73: {
                                n5 = inputStream.read() + 1;
                                a2.m_048(n7, n3, s, n5);
                                continue block27;
                            }
                            case 77: {
                                n5 = inputStream.read();
                                n5 = n5 & 0xFF | a2.f_byte_arr_12[n5 - 1] << 8;
                                a2.m_048(n7, n3, s, n5);
                                continue block27;
                            }
                            case 78: {
                                n5 = inputStream.read();
                                n5 = n5 & 0xFF | a2.f_byte_arr_13[n5 - 1] << 8;
                                a2.m_048(n7, n3, s, n5);
                                continue block27;
                            }
                            case 76: 
                            case 82: {
                                n5 = inputStream.read() | inputStream.read() + 1 << 8;
                                n7 = a2.m_048(n7, n3, s, n5);
                                a2.f_bool_arr_00[n7] = false;
                                continue block27;
                            }
                            default: {
                                a2.m_048(n7, n3, s, 0);
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
            this.f_byte_arr_29 = this.f_byte_arr2_05[n];
            this.f_int_138 = 0;
            int n8 = this.m_123();
            for (int i = 0; i < n8; ++i) {
                byte by = this.f_byte_arr_29[this.f_int_138++];
                n2 = this.m_123();
                int n9 = this.m_123();
                boolean bl4 = this.f_byte_arr_29[this.f_int_138++] != 0;
                n3 = this.m_123();
                this.m_048(by, n2, n9, n3);
                this.f_bool_arr_00[i] = bl4;
                this.f_bool_arr_01[i] = false;
            }
            switch (this.currentFloor) {
                case 31: {
                    break;
                }
                case 12: {
                    n8 = this.m_100(1, 1, 78);
                    if (n8 >= 0) {
                        this.entityParam[n8] = 6 | this.f_byte_arr_13[5] << 8;
                    }
                    if ((n8 = this.m_100(11, 1, 78)) < 0) break;
                    this.entityParam[n8] = 5 | this.f_byte_arr_13[4] << 8;
                    break;
                }
                case 2: {
                    if (this.f_bool_arr_06[26] || this.m_100(11, 4, 84) >= 0) break;
                    this.m_048(84, 352, 128, 0);
                    break;
                }
                case 39: {
                    n8 = this.m_100(11, 1, 7);
                    if (n8 >= 0) {
                        this.m_049(11, 1, 7);
                        this.m_048(8, 11, 1, 38);
                    }
                    if ((n8 = this.m_100(11, 11, 8)) < 0) break;
                    this.m_049(11, 11, 8);
                    this.m_048(7, 11, 11, 40);
                }
            }
        }
        if (n != 50) {
            this.f_byte_arr_03[72] = 32;
            return;
        }
        this.f_byte_arr_03[72] = 8;
    }

    private int m_123() {
        return (this.f_byte_arr_29[this.f_int_138++] & 0xFF) << 8 | this.f_byte_arr_29[this.f_int_138++] & 0xFF;
    }

    private void m_124(int n, int n2) {
        if (this.f_byte_21 < 64) {
            this.f_short_arr_07[this.f_byte_21] = n;
            this.f_short_arr_08[this.f_byte_21] = n2;
            this.f_byte_21 = (byte)(this.f_byte_21 + 1);
        }
    }

    private void m_125(byte by, int n, int n2, int n3) {
        n2 += this.f_int_56;
        this.f_byte_arr_30[this.f_int_140] = by;
        this.f_int_arr_30[this.f_int_140] = n;
        this.f_bool_arr_08[this.f_int_140] = false;
        this.f_byte_arr_31[this.f_int_140] = 0;
        this.f_short_arr_10[this.f_int_140] = n3 += this.f_int_57;
        if (by == 4) {
            this.f_short_arr_09[this.f_int_140] = 240;
            this.f_short_arr_10[this.f_int_140] = 30;
        } else if (by >= 5 && by <= 7) {
            this.f_short_arr_09[this.f_int_140] = n2 + 16;
        } else if (by == 1) {
            this.f_short_arr_09[this.f_int_140] = n2 - 2;
        } else {
            int n4;
            int n5 = n2;
            Image image = this.f_Image_arr2_00[2][3];
            n2 = n4 = image.getWidth() / 11;
            if (n < 0) {
                n = -n;
            }
            do {
                n2 += n4;
            } while ((n /= 10) > 0);
            this.f_short_arr_09[this.f_int_140] = n5 + (32 + n2 >> 1);
        }
        if (++this.f_int_140 >= 30) {
            this.f_int_140 = 0;
        }
    }

    private void m_126() {
        byte by = 0;
        int n = this.f_int_139;
        while (true) {
            if (n >= 30) {
                n = 0;
            }
            if (n == this.f_int_140) break;
            if (!this.f_bool_arr_08[n]) {
                byte by2 = this.f_byte_arr_30[n];
                int n2 = this.f_int_arr_30[n];
                int n3 = 0 + this.f_short_arr_09[n];
                by = this.f_byte_arr_31[n];
                int n4 = 16 + this.f_short_arr_10[n];
                switch (by2) {
                    case 1: {
                        this.m_002(this.f_Image_arr2_00[2][6], n3, n4 -= this.f_byte_arr_31[n] << 2, 0, 19 * n2, 37, 19);
                        break;
                    }
                    case 2: {
                        this.m_127(this.f_Image_arr2_00[2][3], n2, n3, n4 -= this.f_byte_arr_31[n] << 2);
                        break;
                    }
                    case 3: {
                        this.m_127(this.f_Image_arr2_00[2][4], n2, n3, n4 -= this.f_byte_arr_31[n] << 2);
                        break;
                    }
                    case 4: {
                        this.m_127(this.f_Image_arr2_00[2][5], n2, n3, n4 -= this.f_byte_arr_31[n] << 2);
                        break;
                    }
                    case 5: {
                        this.m_042(this.f_Image_arr2_00[8][9], n2, n3 + this.f_byte_arr_32[by], 16 + this.f_short_arr_10[n] + this.f_byte_arr_33[by]);
                        break;
                    }
                    case 6: {
                        this.m_042(this.f_Image_arr2_00[8][9], n2, n3 + this.f_byte_arr_34[by], 16 + this.f_short_arr_10[n] + this.f_byte_arr_35[by]);
                        break;
                    }
                    case 7: {
                        this.m_042(this.f_Image_arr2_00[8][9], n2, n3 + this.f_byte_arr_36[by], 16 + this.f_short_arr_10[n] + this.f_byte_arr_37[by]);
                    }
                }
                if (by > 7) {
                    this.f_bool_arr_08[n] = true;
                } else {
                    int n5 = n;
                    this.f_byte_arr_31[n5] = this.f_byte_arr_31[n5] + 1;
                }
            }
            ++n;
        }
        if (this.f_int_139 != this.f_int_140 && this.f_bool_arr_08[this.f_int_139] && ++this.f_int_139 >= 30) {
            this.f_int_139 = 0;
        }
    }

    private int m_127(Image image, int n, int n2, int n3) {
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
            this.f_Graphics_00.setClip(n2 -= n4 + 1, n3, n4, n5);
            this.f_Graphics_00.drawImage(image, n2 - n6 * n4, n3, 0);
            ++n7;
        } while ((n /= 10) > 0);
        this.f_Graphics_00.setClip(n2 -= n4 + 1, n3, n4, n5);
        this.f_Graphics_00.drawImage(image, n2 - (image.getWidth() - n4), n3, 0);
        this.f_Graphics_00.setClip(0, 0, 240, 320);
        return n7 + 1;
    }

    private void m_128() {
        this.f_short_arr_11 = new short[32];
        this.f_short_arr_12 = new short[32];
        this.f_byte_arr_38 = new byte[32];
        this.f_byte_arr_39 = new byte[32];
        this.f_byte_arr_40 = new byte[32];
        this.f_byte_22 = 0;
        this.f_int_arr_31 = new int[32];
        this.f_bool_arr_09 = new boolean[32];
        this.f_bool_arr_10 = new boolean[32];
    }

    private void m_129(int n, int n2, int n3, int n4) {
        this.f_short_arr_11[this.f_byte_22] = n;
        this.f_short_arr_12[this.f_byte_22] = n2;
        this.f_byte_arr_38[this.f_byte_22] = -3;
        this.f_byte_arr_39[this.f_byte_22] = -4;
        this.f_byte_arr_40[this.f_byte_22] = 2;
        this.f_bool_arr_09[this.f_byte_22] = true;
        this.f_int_arr_31[this.f_byte_22] = -1;
        this.f_bool_arr_10[this.f_byte_22] = false;
        this.f_byte_22 = (byte)(this.f_byte_22 + 1);
        if (this.f_byte_22 > 31) {
            this.f_byte_22 = 0;
        }
    }

    private void m_130() {
        byte by = 0;
        int n = 0;
        this.f_Graphics_00.setColor(this.f_int_141);
        int n2 = 32;
        while (--n2 >= 0) {
            if (!this.f_bool_arr_09[n2]) continue;
            n = this.f_byte_arr_40[n2];
            this.f_Graphics_00.fillRect((int)this.f_short_arr_11[n2], (int)this.f_short_arr_12[n2], n, n);
            by = this.f_byte_arr_38[n2];
            int n3 = n2;
            this.f_short_arr_11[n3] = this.f_short_arr_11[n3] + by;
            n <<= 1;
            if (!this.f_bool_arr_10[n2]) {
                int n4 = n2;
                this.f_byte_arr_38[n4] = (byte)(this.f_byte_arr_38[n4] + 1);
                if (this.f_byte_arr_38[n4] == n) {
                    this.f_bool_arr_10[n2] = true;
                }
            } else {
                int n5 = n2;
                this.f_byte_arr_38[n5] = (byte)(this.f_byte_arr_38[n5] - 1);
                if (this.f_byte_arr_38[n5] == -n) {
                    this.f_bool_arr_10[n2] = false;
                    int n6 = n2;
                    this.f_byte_arr_40[n6] = (byte)(this.f_byte_arr_40[n6] + 1);
                    if (this.f_byte_arr_40[n6] > 5) {
                        this.f_bool_arr_09[n2] = false;
                    }
                }
            }
            int n7 = n2;
            this.f_short_arr_12[n7] = this.f_short_arr_12[n7] + this.f_byte_arr_39[n2];
        }
    }

    private void m_131(byte by, boolean bl) {
        this.f_byte_24 = (byte)3;
        this.f_bool_19 = true;
        if (this.f_byte_arr_41 == null) {
            this.f_byte_arr_41 = new byte[16];
        }
        this.f_int_145 = 0;
        this.f_int_144 = 0;
        this.gameMode = (byte)2;
        this.m_001(15);
    }

    private void m_132(int n) {
        this.f_byte_arr_41[this.f_int_144++] = n;
    }

    private boolean m_133(int n, int n2, int n3, int n4) {
        a a2 = this;
        this.f_byte_arr_42 = new byte[100];
        a2.f_int_148 = 0;
        a2.f_byte_arr2_06 = new byte[a2.mapCellsHigh][a2.mapCellsWide];
        a2.f_short_arr2_00 = new short[a2.mapCellsHigh][a2.mapCellsWide];
        a2.f_short_arr_13 = new short[100];
        a2.f_short_arr_14 = new short[100];
        a2.f_int_149 = 0;
        boolean bl = false;
        int n5 = n;
        int n6 = n2;
        int n7 = this.mapCellsWide;
        int n8 = n5 + n6 * n7;
        this.f_short_00 = (short)n;
        this.f_short_01 = (short)n2;
        this.f_short_02 = (short)n3;
        this.f_short_03 = (short)n4;
        this.f_int_149 = 1;
        n3 = 0;
        do {
            this.m_134(n5 - 1, n6, n8);
            this.m_134(n5 + 1, n6, n8);
            this.m_134(n5, n6 - 1, n8);
            this.m_134(n5, n6 + 1, n8);
            if (this.f_int_149 <= 0) break;
            n8 = this.f_short_arr_14[this.f_int_149 - 1];
            n5 = n8 % n7;
            n6 = n8 / n7;
            --this.f_int_149;
            this.f_byte_arr2_06[n6][n5] = 2;
            if (n5 != this.f_short_02 || n6 != this.f_short_03) continue;
            bl = true;
        } while (++n3 < 100 && !bl);
        n3 = 0;
        n += n2 * n7;
        n2 = 0;
        if (bl) {
            do {
                if ((n2 = (n3 = this.f_short_arr2_00[n6][n5]) - n8) == -1) {
                    this.f_byte_arr_42[this.f_int_148++] = 2;
                } else if (n2 == 1) {
                    this.f_byte_arr_42[this.f_int_148++] = 3;
                } else if (n2 == -n7) {
                    this.f_byte_arr_42[this.f_int_148++] = 0;
                } else if (n2 == n7) {
                    this.f_byte_arr_42[this.f_int_148++] = 1;
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

    private void m_134(int n, int n2, int n3) {
        short s = 0;
        if (this.f_byte_arr2_06[n2][n] == 0 && (this.isCellWalkable(n, n2) || this.m_062(n, n2))) {
            this.f_byte_arr2_06[n2][n] = 1;
            this.f_short_arr2_00[n2][n] = n3;
            s = (short)(Math.abs(n - this.f_short_02) + Math.abs(n2 - this.f_short_03) + Math.abs(n - this.f_short_00) + Math.abs(n2 - this.f_short_01));
            n3 = this.f_int_149;
            while (--n3 >= 0) {
                if (s >= this.f_short_arr_13[n3] && n3 != 0) continue;
                for (int i = this.f_int_149; i > n3 + 1; --i) {
                    this.f_short_arr_13[i] = this.f_short_arr_13[i - 1];
                    this.f_short_arr_14[i] = this.f_short_arr_14[i - 1];
                }
                this.f_short_arr_13[++n3] = s;
                this.f_short_arr_14[n3] = n + n2 * this.mapCellsWide;
                break;
            }
            ++this.f_int_149;
        }
    }

    private int predictHpLossVsType(int n, boolean bl) {
        int n2 = this.effectiveAttackVsType(n);
        return a.predictBattleHpLoss(n2, this.playerDef, this.enemyHpScaled[n -= 41], this.enemyAtkScaled[n], this.enemyDefScaled[n], bl);
    }

    private int effectiveAttackVsType(int n) {
        n = this.f_byte_arr_05[n];
        int n2 = 1;
        if ((n & 1) != 0 && this.m_081(23) >= 0) {
            n2 = 2;
        } else if ((n & 2) != 0 && this.m_081(24) >= 0) {
            n2 = 2;
        }
        return this.playerAtk * n2;
    }

    private static int predictBattleHpLoss(int n, int n2, int n3, int n4, int n5, boolean bl) {
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

    private void tickBattle(boolean bl) {
        if (!this.f_bool_23) {
            if (this.f_int_44 < 0) {
                this.f_byte_11 = 0;
                return;
            }
            int a2 = this.entityType[this.f_int_44];
            int n = this.effectiveAttackVsType(a2) - this.enemyDefScaled[a2 - 41];
            if (n > 0) {
                if ((this.f_int_151 & 3) == 0) {
                    this.f_int_150 -= n;
                    this.m_125((byte)(5 + this.f_int_151 % 3), n, this.entityPixelX[this.f_int_44], this.entityPixelY[this.f_int_44]);
                    if (this.f_int_150 > 0) {
                        n = this.enemyAtkScaled[a2 - 41] - this.playerDef;
                        if (n > 0) {
                            this.playerHp -= n;
                        }
                    } else {
                        boolean bl2;
                        n = this.predictHpLossVsType(a2, this.f_bool_25);
                        if (n > 0) {
                            this.m_125((byte)2, n, this.playerPixelX, this.playerPixelY);
                        }
                        if (this.m_081(25) >= 0) {
                            this.gainGold(this.enemyBaseGold[a2 - 41] << 1, this.entityPixelX[this.f_int_44], this.entityPixelY[this.f_int_44]);
                        } else {
                            this.gainGold(this.enemyBaseGold[a2 - 41], this.entityPixelX[this.f_int_44], this.entityPixelY[this.f_int_44]);
                        }
                        n = this.entityType[this.f_int_44];
                        a a3 = this;
                        boolean bl3 = false;
                        if (a3.currentFloor > 50) {
                            bl2 = false;
                        } else {
                            int n2 = a3.bossTypeOrder.length;
                            while (--n2 >= 0) {
                                if (n != a3.bossTypeOrder[n2]) continue;
                                a3.spawnBossEvent(n2);
                                break;
                            }
                            switch (n) {
                                case 73: {
                                    a3.f_bool_arr_06[a3.currentScriptIndex] = true;
                                    a3.loadLevelScript(22);
                                    if (a3.currentFloor != 40) break;
                                    a3.spawnBossEvent(6);
                                    break;
                                }
                                case 74: {
                                    a3.loadLevelScript(21);
                                    break;
                                }
                                case 75: {
                                    if (!a3.tryRunScene(20, true)) break;
                                    a3.loadLevelScript(20);
                                    bl3 = true;
                                }
                            }
                            bl2 = bl3;
                        }
                        this.f_bool_24 = bl2;
                        this.f_bool_23 = true;
                    }
                }
            } else {
                this.f_byte_11 = 0;
            }
            ++this.f_int_151;
            return;
        }
        if (this.f_byte_27 == 0) {
            this.m_047(this.f_int_44);
        }
        if ((this.f_byte_27 = (byte)(this.f_byte_27 + 1)) > 5) {
            this.f_byte_27 = 0;
            if (!this.f_bool_24 && bl) {
                this.tryStep(this.f_byte_12);
            }
            this.f_bool_23 = false;
            int n = this.entityType[this.f_int_44];
            a a2 = this;
            switch (n) {
                case 70: 
                case 71: {
                    if (a2.currentFloor != 49) break;
                    a2.loadLevelScript(35);
                    break;
                }
                case 61: {
                    if (a2.currentFloor != 49 || a2.m_100(6, 2, 61) >= 0 || a2.m_100(5, 3, 61) >= 0 || a2.m_100(7, 3, 61) >= 0 || a2.m_100(6, 4, 61) >= 0 || a2.m_100(5, 2, 61) < 0 || a2.m_100(7, 2, 61) < 0 || a2.m_100(5, 4, 61) < 0 || a2.m_100(7, 4, 61) < 0) break;
                    a2.m_049(6, 3, 70);
                    a2.m_048(71, 192, 96, 0);
                    break;
                }
                case 72: {
                    if (a2.currentFloor != 50) break;
                    a2.loadLevelScript(65);
                    break;
                }
                case 69: {
                    if (a2.currentFloor != 35) break;
                    a2.spawnBossEvent(7);
                    a2.loadLevelScript(36);
                }
            }
            this.f_int_44 = -1;
        }
    }

    private void m_139(byte by, int n) {
        if (!this.f_bool_29) {
            return;
        }
        this.f_InputStream_00 = null;
        try {
            if (this.f_byte_28 == by && f_Player_00 != null) {
                switch (f_Player_00.getState()) {
                    case 400: {
                        f_Player_00.stop();
                    }
                    case 300: {
                        try {
                            f_Player_00.setLoopCount(-1);
                        }
                        catch (Exception exception) {}
                        f_Player_00.start();
                        return;
                    }
                }
            }
            this.m_140();
            this.f_InputStream_00 = this.getClass().getResourceAsStream(this.f_String_arr_15[by]);
            f_Player_00 = Manager.createPlayer((InputStream)this.f_InputStream_00, (String)(this.f_String_arr_15[by].endsWith("wav") ? "audio/x-wav" : "audio/midi"));
            f_Player_00.realize();
            f_Player_00.prefetch();
            try {
                f_Player_00.setLoopCount(-1);
            }
            catch (Exception exception) {
                Exception exception2 = exception;
                exception.printStackTrace();
            }
            int n2 = this.f_int_155;
            a a2 = this;
            a2 = f_Player_00;
            if (a2 != null && ((a2.getState() & 0x12C) == 300 || (a2.getState() & 0x190) == 400)) {
                try {
                    ((VolumeControl)f_Player_00.getControl("VolumeControl")).setLevel(n2);
                }
                catch (Exception exception) {}
            }
            f_Player_00.start();
            this.f_byte_28 = by;
            return;
        }
        catch (Exception exception) {
            Exception exception3 = exception;
            exception.printStackTrace();
            return;
        }
        finally {
            this.m_008();
        }
    }

    private void m_140() {
        if (f_Player_00 != null) {
            f_Player_00.close();
        }
        f_Player_00 = null;
        this.f_byte_28 = (byte)-1;
    }

    private int randomBelow(int n) {
        return (this.gameRandom.nextInt() >>> 1) % n;
    }

    private void m_142(String string, int n, int n2, int n3, int[] nArray) {
        this.f_Graphics_00.setColor(nArray[0]);
        int n4 = --n;
        this.f_Graphics_00.drawString(string, n4, --n2, 17);
        int n5 = ++n;
        this.f_Graphics_00.drawString(string, n5, n2, 17);
        this.f_Graphics_00.drawString(string, ++n, n2++, 17);
        this.f_Graphics_00.drawString(string, n, n2++, 17);
        this.f_Graphics_00.drawString(string, n--, n2, 17);
        this.f_Graphics_00.drawString(string, n--, n2, 17);
        this.f_Graphics_00.drawString(string, n, n2--, 17);
        this.f_Graphics_00.drawString(string, n++, n2, 17);
        this.f_Graphics_00.setColor(nArray[1]);
        this.f_Graphics_00.drawString(string, n, n2, 17);
    }

    private void m_143(int n, int n2) {
        int n3;
        a a2;
        if (!this.f_bool_30) {
            a2 = this;
            a2.m_001(0);
            n3 = a2.f_byte_arr_45[0] + a2.f_byte_arr_45[1] + a2.f_byte_arr_45[2];
            a2.f_int_157 = Math.min(15, Math.abs(320 - n3 >> 2));
            n3 = 320 - n3 - (a2.f_int_157 << 1) >> 1;
            a2.f_int_arr_37[0] = n3 + (a2.f_byte_arr_45[0] >> 1);
            a2.f_int_arr_37[1] = n3 + a2.f_byte_arr_45[0] + a2.f_int_157;
            a2.f_int_arr_37[2] = a2.f_int_arr_37[1] + a2.f_byte_arr_45[1] + a2.f_int_157;
            a2.f_int_158 = 240 - a2.f_Image_arr2_00[0][7].getWidth() >> 1;
            this.f_bool_30 = true;
        }
        a2 = this;
        block33: for (n3 = a2.f_byte_29 - 1; n3 >= 0; --n3) {
            int[] nArray = a2.f_int_arr2_03[n3];
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
                    this.m_145(this.f_Image_arr2_00[0][1], 120, 0, 1, 12, 4, 120, this.f_int_arr_37[0], 0, -1);
                    break;
                }
                case 5: {
                    this.m_147(0);
                    this.m_145(this.f_Image_arr2_00[0][0], 120, this.f_int_arr_37[0], 2, 12, 4, 4, 0, 0, -1);
                    break;
                }
                case 8: {
                    this.m_145(this.f_Image_arr2_00[0][5], 120, this.f_int_arr_37[0] + 2, 0, 12, 0, 0, 0, 0, 0);
                    this.m_145(this.f_Image_arr2_00[0][2], 90, this.f_int_arr_37[1], 2, 4, 4, 4, 0, 0, -1);
                    this.m_145(this.f_Image_arr2_00[0][3], 120, this.f_int_arr_37[1], 2, 4, 4, 4, 0, 0, -1);
                    this.m_145(this.f_Image_arr2_00[0][4], 150, this.f_int_arr_37[1], 2, 4, 4, 4, 0, 0, -1);
                    this.m_145(this.f_Image_arr2_00[0][6], 15, this.f_int_arr_37[1], 1, 12, 3, this.f_int_158, this.f_int_arr_37[2], 0, -1);
                    break;
                }
                case 10: {
                    this.m_147(5);
                    this.m_145(null, this.f_int_158, this.f_int_arr_37[2], 3, 0, 0, 0, 0, 0, -1);
                    this.m_146(5, 4, 0, 1, 2, 0);
                    break;
                }
                case 14: {
                    this.m_146(5, 3, 0, 3, this.f_int_158 - 8, this.f_int_arr_37[2]);
                    this.m_146(6, 3, 0, 3, this.f_int_158 + this.f_byte_arr_46[2] - 4, this.f_int_arr_37[2]);
                    break;
                }
                case 15: {
                    this.m_146(7, 3, 0, 2, this.f_int_158 + this.f_byte_arr_46[4] - 2, this.f_int_arr_37[2]);
                    break;
                }
                case 17: {
                    this.m_146(5, 3, 0, 3, this.f_int_158, this.f_int_arr_37[2]);
                    this.m_146(6, 3, 0, 3, this.f_int_158 + this.f_byte_arr_46[2], this.f_int_arr_37[2]);
                    this.m_146(7, 3, 0, 2, this.f_int_158 + this.f_byte_arr_46[4], this.f_int_arr_37[2]);
                }
            }
            if (n2 >= 10 && n2 < 13) {
                this.m_145(null, 240, this.f_int_arr_37[2], 3, 0, 4, this.f_int_158 + this.f_byte_arr_46[n2 - 9 << 1], this.f_int_arr_37[2], n2 - 9, -1);
                return;
            }
            if (n2 >= 14 && n2 <= 17) {
                this.m_145(null, this.f_int_158 + this.f_byte_arr_46[n2 - 10 << 1], 320, 3, 0, 4, this.f_int_158 + this.f_byte_arr_46[n2 - 10 << 1], this.f_int_arr_37[2], n2 - 10, -1);
                return;
            }
            if (n2 >= 25 && n2 <= 31) {
                n = (n2 - 25 << 1) + 8;
                this.m_145(null, this.f_int_158 + this.f_byte_arr_46[n << 1], this.f_int_arr_37[2], 3, 0, 0, this.f_int_158 + this.f_byte_arr_46[n << 1], this.f_int_arr_37[2], n, -1);
                if (n2 < 31) {
                    this.m_145(null, this.f_int_158 + this.f_byte_arr_46[++n << 1], this.f_int_arr_37[2], 3, 0, 0, this.f_int_158 + this.f_byte_arr_46[n << 1], this.f_int_arr_37[2], n, -1);
                }
                return;
            }
        } else {
            if (n2 >= 18 && n2 < 31) {
                this.m_145(null, this.f_int_158 + this.f_byte_arr_46[n2 - 10 << 1], 320, 3, 0, 4, this.f_int_158 + this.f_byte_arr_46[n2 - 10 << 1], this.f_int_arr_37[2], n2 - 10, -1);
            }
            switch (n2) {
                case 50: {
                    this.m_146(2, 2, 4, 4, 4, 1);
                    break;
                }
                case 51: {
                    this.m_146(3, 2, 4, 4, 4, 1);
                    break;
                }
                case 52: {
                    this.m_146(4, 2, 4, 4, 4, 1);
                    break;
                }
                case 58: {
                    this.m_146(0, 1, 0, 1, 1000, 1000);
                    this.m_146(1, 2, 4, 4, 4, 1);
                    break;
                }
                case 63: {
                    this.m_146(15, 3, 0, 4, -10, this.f_int_arr_37[2]);
                }
            }
            if (n2 > 52 && n2 < 63) {
                n = n2 - 52;
                this.m_146(n + 4, 3, 0, 4, -10, this.f_int_arr_37[2]);
                this.m_146(26 - n, 3, 0, 4, 240, this.f_int_arr_37[2]);
            }
        }
    }

    private void m_144() {
        this.f_Graphics_00.setColor(0xFFFFFF);
        this.f_Graphics_00.fillRect(0, 0, 240, 320);
        for (int i = 0; i < this.f_byte_29; ++i) {
            int[] nArray = this.f_int_arr2_03[i];
            int n = nArray[0];
            int n2 = nArray[1];
            if (nArray[2] == 3 || nArray[2] == 4) {
                this.f_Graphics_00.setClip((n -= this.f_byte_arr_46[nArray[6] << 1]) + this.f_byte_arr_46[nArray[6] << 1], n2, (int)this.f_byte_arr_46[(nArray[6] << 1) + 1], 320);
                this.f_Graphics_00.drawImage(this.f_Image_arr2_00[0][7], n, n2, 0);
            } else if (nArray[2] == 2) {
                int n3 = nArray[3] - nArray[4];
                if (nArray[5] == 1) {
                    n3 = nArray[3] - n3 - 2;
                }
                this.f_Graphics_00.setClip((n -= nArray[6] * n3) + nArray[6] * n3, n2, nArray[6], 320);
            }
            if (nArray[2] != 3 && nArray[2] != 4) {
                this.f_Graphics_00.drawImage(this.f_Image_arr_01[i], n, n2, 0);
            }
            this.f_Graphics_00.setClip(0, 0, 240, 320);
        }
    }

    private void m_145(Image image, int n, int n2, int n3, int n4, int n5, int n6, int n7, int n8, int n9) {
        if (this.f_Image_arr_01 == null) {
            this.f_Image_arr_01 = new Image[26];
        }
        if (this.f_byte_29 < 26) {
            int[] nArray = this.f_int_arr2_03[this.f_byte_29];
            if (n9 >= 0 && n9 < this.f_byte_29) {
                this.f_int_arr2_03[this.f_byte_29] = this.f_int_arr2_03[n9];
                this.f_Image_arr_01[this.f_byte_29] = this.f_Image_arr_01[n9];
                this.f_Image_arr_01[n9] = image;
                this.f_int_arr2_03[n9] = nArray;
                nArray = this.f_int_arr2_03[n9];
            } else {
                this.f_Image_arr_01[this.f_byte_29] = image;
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
            this.f_byte_29 = (byte)(this.f_byte_29 + 1);
        }
    }

    private void m_146(int n, int n2, int n3, int n4, int n5, int n6) {
        if (n < this.f_byte_29) {
            int[] nArray = this.f_int_arr2_03[n];
            if (n2 == 1) {
                int n7 = this.f_Image_arr_01[n].getWidth();
                int n8 = this.f_Image_arr_01[n].getHeight();
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

    private void m_147(int n) {
        if (n >= 0) {
            if (n < this.f_byte_29) {
                int[] nArray = this.f_int_arr2_03[n];
                this.f_byte_29 = (byte)(this.f_byte_29 - 1);
                this.f_int_arr2_03[n] = this.f_int_arr2_03[this.f_byte_29];
                this.f_int_arr2_03[this.f_byte_29] = nArray;
                this.f_Image_arr_01[n] = null;
                this.f_Image_arr_01[n] = this.f_Image_arr_01[this.f_byte_29];
                this.f_Image_arr_01[this.f_byte_29] = null;
                return;
            }
        } else {
            this.f_byte_29 = 0;
            this.f_Image_arr_01 = null;
        }
    }

    private void m_148() {
        if (this.f_HttpConnection_00 != null) {
            try {
                this.f_HttpConnection_00.close();
            }
            catch (Exception exception) {}
            this.f_HttpConnection_00 = null;
        }
        if (this.f_OutputStream_00 != null) {
            try {
                this.f_OutputStream_00.close();
            }
            catch (Exception exception) {}
            this.f_OutputStream_00 = null;
        }
        if (this.f_InputStream_01 != null) {
            try {
                this.f_InputStream_01.close();
            }
            catch (Exception exception) {}
            this.f_InputStream_01 = null;
        }
        System.gc();
    }

    private int m_149() {
        return (this.f_byte_arr_48[this.f_int_162++] & 0xFF) << 8 | this.f_byte_arr_48[this.f_int_162++] & 0xFF;
    }

    private int m_150() {
        return this.m_149() << 16 | this.m_149() & 0xFFFF;
    }

    private int m_151() {
        int n = 0;
        try {
            String string;
            Object object;
            String string2;
            String string3 = string2 = this.f_String_06 == null ? this.f_String_arr_17[0] : this.f_String_06;
            Object object2 = this;
            if (((a)object2).f_bool_33) {
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
            this.f_HttpConnection_00 = (HttpConnection)Connector.open((String)object);
            this.f_HttpConnection_00.setRequestMethod("POST");
            this.f_HttpConnection_00.setRequestProperty("Content-Type", "application/octet-stream");
            if (this.f_bool_33) {
                this.f_HttpConnection_00.setRequestProperty("X-Online-Host", string2);
            }
            this.f_HttpConnection_00.setRequestProperty("Content-Length", "" + 0);
            if (this.f_byte_arr_47 != null) {
                this.f_OutputStream_00 = this.f_HttpConnection_00.openOutputStream();
                this.f_OutputStream_00.write(this.f_byte_arr_47, 0, 0);
                this.f_OutputStream_00.flush();
            }
            this.f_int_160 = this.f_HttpConnection_00.getResponseCode();
            if (this.f_int_160 >= 300 && this.f_int_160 < 400) {
                this.f_String_07 = "\u91cd\u5b9a\u5411!!!";
                this.f_String_06 = this.f_HttpConnection_00.getHeaderField("Location");
                n = 1;
            } else if (this.f_int_160 / 100 != 2) {
                n = -1;
            } else {
                n = 0;
                while (true) {
                    object = null;
                    object2 = this.f_HttpConnection_00.getHeaderFieldKey(n);
                    if (object2 == null) break;
                    this.f_HttpConnection_00.getHeaderField(n);
                    ++n;
                }
                this.f_InputStream_01 = this.f_HttpConnection_00.openInputStream();
                n = (int)this.f_HttpConnection_00.getLength();
                if (n > 0) {
                    this.f_byte_arr_48 = null;
                    this.f_byte_arr_48 = new byte[n];
                    int n4 = 0;
                    for (int i = 0; i < n; ++i) {
                        n4 = this.f_InputStream_01.read();
                        this.f_byte_arr_48[i] = (byte)n4;
                    }
                } else {
                    object = new ByteArrayOutputStream(2048);
                    boolean bl = false;
                    byte[] byArray = new byte[64];
                    while ((n = this.f_InputStream_01.read(byArray, 0, 64)) >= 0) {
                        ((ByteArrayOutputStream)object).write(byArray, 0, n);
                    }
                    ((ByteArrayOutputStream)object).close();
                    this.f_byte_arr_48 = ((ByteArrayOutputStream)object).toByteArray();
                }
                this.f_InputStream_01.close();
                this.f_InputStream_01 = null;
                n = this.f_byte_arr_48.length;
                Object object3 = new byte[n >> 1];
                int n5 = 0;
                while (n5 + 1 < n) {
                    object3[n5 >> 1] = a.m_154(this.f_byte_arr_48[n5]) << 4 | a.m_154(this.f_byte_arr_48[n5 + 1]);
                    n5 += 2;
                }
                this.f_byte_arr_48 = null;
                this.f_byte_arr_48 = object3;
                object3 = new StringBuffer();
                n >>= 1;
                for (n5 = 0; n5 < n; ++n5) {
                    ((StringBuffer)object3).append(Integer.toHexString(this.f_byte_arr_48[n5] & 0xFF));
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
            this.m_148();
        }
        return n;
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    private int m_152() {
        int n;
        Object object = this.f_Object_00;
        synchronized (object) {
            n = this.f_int_161;
        }
        if (n == 2) {
            Thread.yield();
        }
        return n;
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    private boolean m_153() {
        ++this.f_int_165;
        switch (this.m_152()) {
            case 0: {
                this.f_int_165 = 0;
                this.f_int_166 = 0;
                this.f_int_165 = 0;
                try {
                    if (this.f_int_160 == 200) {
                        a a2 = this;
                        long l = 0L;
                        a2.f_int_162 = 0;
                        a a3 = a2;
                        l = a3.m_150() << 32 | a3.m_150();
                        if (l > 0L) {
                            a2.f_int_164 = 0;
                            a2.f_long_01 = a2.f_long_00;
                            a2.f_long_00 = l;
                            a2.f_int_163 = a2.f_byte_arr_48[a2.f_int_162++];
                            for (int i = 0; i < a2.f_int_163; ++i) {
                                a3 = a2;
                                ++a3.f_int_162;
                                int n = a3.m_149();
                                if (n == 0) continue;
                                a3.f_int_162 += n;
                            }
                        } else {
                            if (++a2.f_int_164 >= 2 && a2.f_long_01 != a2.f_long_00) {
                                a2.f_long_00 = a2.f_long_01;
                            }
                            Object object = a2.f_Object_00;
                            synchronized (object) {
                                a2.f_int_161 = -1;
                            }
                            a2.f_String_07 = a2.f_String_arr_16[48];
                        }
                    }
                }
                catch (Exception exception) {
                    Exception exception2 = exception;
                    exception.printStackTrace();
                }
                finally {
                    this.m_148();
                }
                return true;
            }
            case -1: {
                if (this.f_int_166 < 3) {
                    ++this.f_int_166;
                    this.f_int_165 = 0;
                    this.f_String_07 = "\u6b63\u5728\u91cd\u8bd5(" + this.f_int_166 + ")";
                    a a4 = this;
                    a4.m_148();
                    a4.f_int_160 = 0;
                    a4.f_int_165 = 0;
                    if (a4.f_Object_00 == null) {
                        a4.f_Object_00 = new Object();
                    }
                    Object object = a4.f_Object_00;
                    synchronized (object) {
                        a4.f_int_161 = 2;
                    }
                    a4.f_bool_31 = true;
                    Thread thread = new Thread(a4);
                    thread.start();
                    break;
                }
                this.f_int_166 = 0;
                this.f_String_07 = this.f_String_arr_16[15];
                this.f_bool_33 = !this.f_bool_33;
                return true;
            }
        }
        if (this.f_int_165 > 300) {
            this.f_int_161 = 3;
            this.f_int_165 = 0;
            this.f_String_07 = "\u5df2\u8d85\u65f6\uff0c\u8bf7\u91cd\u8bd5\u3002";
            this.f_bool_33 = !this.f_bool_33;
            return true;
        }
        return false;
    }

    private static int m_154(int n) {
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
