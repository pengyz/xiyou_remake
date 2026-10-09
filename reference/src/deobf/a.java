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
    private int f_int_00 = this.f_Font_00.charWidth('国');
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
        new StringBuffer().append("(内部版本").append(this.f_String_00.substring(4, this.f_String_00.length() - 1)).append(")");
        this.f_String_arr_00 = new String[]{"无法再下一层了，这是你达到的最底层", "无法再上一层了，这是你达到的最高层", "无法再下去了。", "无法再上去了。", "俺，当世神界第一斗者，孙!悟!空! 自从受封为齐天大圣，掌管蟠桃园以来，一直逍遥快活，无拘束……", "你拥有更强力的装备，因此将捡到的丢弃了。", "直到那一天，遇到了她，在筋斗云上的我，竟然第一次心潮起伏，有了晕机的感觉……", "神仙动了感情，往往会万劫不复，\n这一次，让我付出了五百年的时间去忘记她……\n五指山脚下的沙子，掠过我的脸庞。\n沙子，跟时间一样，同样随风流逝；同样掩埋过去；\n多少次伸手想抓住，却从指隙溜走……\n看夜空，半梦半醒间，往事历历上心头……"};
        this.f_int_04 = 0;
        this.f_Image_00 = null;
        this.f_String_arr_01 = new String[]{"sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1", "ui", "xtq", "menu", "intro", "face", "sptenemy2", "end", "load"};
        this.f_int_arr_00 = new int[]{8, 1, 12, 4, 13, 23, 10, 20, 25, 6, 2, 2, 12, 20, 1, 2};
        this.f_Image_arr2_00 = new Image[this.f_int_arr_00.length][];
        this.f_int_arr2_00 = new int[][]{{0, 1, 2}, {3, 5}, {4, 5}};
        this.f_OutputStream_00 = null;
        this.f_InputStream_01 = null;
        this.f_String_arr_02 = new String[]{"新游戏", "继续游戏", "载入进度", "保存游戏", "设置", "帮助", "关于", "退出", "返回菜单", "回放", "停止回放", "商店", "十全大补包", "攻防神油", "开门天天乐", "天庭消费券", "印度神血油", "跳过教程"};
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
        this.f_String_arr_04 = new String[]{"无", "木", "铁", "银", "金", "布", "皮", "锁", "金"};
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
        this.f_String_arr_05 = new String[]{"声音", "小地图"};
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
        this.f_String_arr_06 = new String[]{"", "", "有些门不能用钥匙打开，只有当你打败它的守卫后才会自动打开。", "你购买了礼物后再与天宫商人对话，他会告诉你一些重要的消息。", "", "我听说在天宫中有2把隐藏的红钥匙。", "在这个区域不多次提升攻击力，就不能打败“杨戬副手”。切记前人教训！", "太上老君就在25楼。以你现在的状态去攻击他简直就是自杀。 你应当在取得更高级别的道具后再去打败他。", "不找到所有的暗墙29楼的暗道是不会打开的", "如果你到27楼时状态为：生命1500、攻击80、防御98、拥有1把蓝钥匙、5把黄钥匙。那么祝贺你，你的前期是比较成功的。", "六丁六甲的攻击力太高了，你最好到能对他一击必杀时再与他战斗。", "别匆忙，放慢速度。", "如果你能用好4种移动宝物，你不用与强敌作战就能上楼。", "", "你需要用“玄明石”取出37楼仓库内的所有宝物。", "谜题：“在3点，拥有传送功能的密宝就会出现。”", "“巫师”会用魔法攻击路过的人，在2个“太上老君护卫”间通过会使你的生命减少一半。", "44楼，被藏在异空间，你只能用密宝才能到达。", "41楼事实上是左右对称的。", "像骰子上5的形状是一种封印魔法，你最好记住它在你与49楼假魔王战斗时有用", "", "你好，我是太白金星。你最好别见敌人就杀，先往上走，拿到武器和防具再做打算。"};
        this.f_String_arr_07 = new String[]{"感谢你救了我，这是1000金就送给你吧。", "试下火眼金睛吧，你能看到怪物的信息和战斗损失的血量，你可以在物品栏中使用它。", "", "", "很好，你居然找到了我，作为奖励我将给你一瓶千年月桂露，喝了它将按你的攻击力和防御力的综合增加的你生命点数，你越晚使用它效果越好。", "", "", "", "", "", "", "", "", "", "感谢你救了我，这是1000金就送给你吧。", "", "", "", "", "", "哈喽，送你1000金作为见面礼，记得经常来找我哦。", ""};
        this.f_byte_arr_12 = new byte[]{2, 2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1};
        this.f_String_arr_08 = new String[]{"大圣饶命，我可以提升3%攻击力和防御力，需要小的为你效劳吗？", "", "大圣爷，我这里有1把蓝钥匙，你给50金币吧。", "我有5把黄钥匙，一共50金币", "嘿嘿，我有很多把黄钥匙，1把1000金币", "我有1把红钥匙，只要800金币", "我有1把蓝钥匙，只要200金币", "我跟其他人不一样，你可以把多余的钥匙卖给我。100金币回收1把黄钥匙", "我有1把黄钥匙，1把蓝钥匙，一共是1000金币", "我有3把黄钥匙，只要200金币", "我有3把蓝钥匙，收你2000金币", "我可以恢复你2000点血，不过要1000金币", "我有个宝物，要收你4000金币，是个玄明石。"};
        this.f_String_arr_09 = new String[]{"", "", "天宫一共50层，每10层为一个区域。如果不打败该区域的头目就不能到更高的地方。", "在商店里你最好选择提升防御力，只有在攻击力低于敌人的防御力时才提升攻击力。", "", "你是否注意到5、9、14、16、18楼有的墙与众不同？", "如果你持有太公杖，面对天神力士和巨灵神时你的攻击力加倍。在没有太公杖的情况下你是无法打败巨灵神的。太公杖被藏在15楼以上的墙内。", "", "天宫一共有50层，但50楼并不能直接上去。", "存放乌金棍的房间的门坏了，你必须用金勺子破墙而入。", "天宫中藏有有个“幸运金币”拥有它在打败敌人后能够获得2倍的金钱。", "“紫金龙鳞甲”能防御“太上老君护卫”的夹击，但它被深藏在神秘的楼层中。", "如果要打败杨戬你需要“乌金棍”、“银缕锁甲”、“捆仙绳”或更高等级的宝物。"};
        this.f_byte_arr_13 = new byte[]{2, 3, 3, 3, 6, 3, 3, 6, 3, 3, 3, 3, 3};
        this.f_int_88 = this.f_String_arr_06.length + this.f_String_arr_09.length;
        this.f_byte_arr_14 = new byte[this.f_int_88];
        this.f_byte_arr_15 = new byte[this.f_int_88];
        this.itemDescriptions = new String[]{"能看破敌人底细，显示敌人详细信息。在游戏中按快捷键5也可以查看伤害量。\n\\c00ff00[使用次数：无限]", "记录前尘往事。\n\\c00ff00[使用次数：无限]", "在楼梯边，可以瞬间上下层，留神晕机。\n\\c00ff00[使用次数：无限]", "熄灭\\cFFCC33三昧真火\\r的神器。\n\\c00ff00[使用次数：无限]", "挖洞开墙越狱的利器,挫是挫了点，但是真的很好用。\n\\c00ff00[使用次数：1次]", "可以震开当前层所有的墙\n\\c00ff00[使用次数：1次]", "喝下后，增加相当于当前\\c00FFFF攻击力\\cFFFFFF加\\c00FFFF防御力\\cFFFFFF值740%的\\cFFCC00血量\n[月宫出品，手工酿制，不含三聚氰胺，冷藏效果更佳，使用次数：1次]", "瞬移到以中心为对称点的位置上。\n\\c00ff00[使用次数：3次]", "瞬移上行一层\n\\c00ff00[使用次数：1次]", "瞬移下行一层\n\\c00ff00[使用次数：1次]", "当年姜子牙受天命封神，他的钓鱼竹竿被原始天尊附上了神力，可以役使天神力士供他差遣，此杖又名“打神鞭”，对天神力士（包括巨灵神）威力加倍。\n\\cFFCC00[放在道具栏中有效]", "对某些自恋的神仙伤害加倍。\n\\cFFCC00[放在道具栏中有效]", "打怪得到的金钱加倍。\n\\cFFCC00[放在道具栏中有效]", "可以开启黄门。", "可以开启红门。", "可以开启蓝门。", "加攻击。", "加防御。", "加血。", "加血。", "开启当前层所有黄门", "如来开“慈悲为怀”巡回佛经演唱会的时候，伴奏罗汉用的乐器，道行浅的敌人，会被其梵天佛音瞬间化为灰飞\n\\cFFCC00使用：杀死上下左右的敌人，对BOSS不起作用。"};
        this.equipDescriptions = new String[]{"", "\\cdddddd一根相当长的木制长棍,新手必备.有了它杀人越货不慌不愁.\n\\c00ff00装备: 攻击+10.\n\\cFFCC00\"看上去似乎会断掉。\".", "\\cdddddd乌黑油亮，显然经历过多人之手。\n\\c00ff00装备: 攻击+30.\n\\cFFCC00\"很粗很结实！\".", "\\cdddddd银棍，恩，有这个名字就足够了。\n\\c00ff00装备: 攻击+70.\n\\cFFCC00\"只是根银棍\".", "\\cdddddd因乘天地之灵气，集日月之精华乃“万木之灵，灵木之尊”。\n\\c00ff00装备: 攻击+120.\n\\cFFCC00\"木之精华，削铁断金\".", "\\cdddddd您的需要，它知道；您的需求，它满足。它好，你也好，龙王后宫，镇宫之宝！\n\\c00ff00装备: 攻击+220.\n\\cFFCC00\"不要迷恋它，它只是一根传说。\".", "", "\\cdddddd没有太多的装饰，一件非常朴素、轻便的布衣.\n\\c00ff00装备: 防御+10.\n\\cFFCC00\"看上去有不少人用过了。\".", "\\cdddddd保暖御寒，腰不酸，腿不疼，走路也有劲了。\n\\c00ff00装备: 防御+30.\n\\cFFCC00\"豹纹，性感又野性，今年天宫最流行的皮草款式\".", "\\cdddddd如果没有上面的那行字，它也算是个杰作。\n\\c00ff00装备: 防御+70.\n\\cFFCC00\"上面写着'办四级神仙证书，回收二手莲花宝座'\".", "\\cdddddd华丽的装饰，就是有点旧。\n\\c00ff00装备: 防御+120.\n\\cFFCC00\"别人穿过的极品。\".", "\\cdddddd东海龙鳞编织而成，限量版，天上天下，只此一款。\n\\c00ff00装备: 防御+220.\n\\cFFCC00\"更轻薄，更透气，更多防护，更多安心\"."};
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
        this.objectTypeNames = new String[]{"孙悟空", "黄门", "红门", "蓝门", "封印门", "门卫", "隐形路径", "上楼梯", "下楼梯", "炼丹炉", "云雾", "能挖的墙", "隐形墙", "火眼金睛", "生死簿", "筋斗云", "芭蕉扇", "金勺子", "玄明石", "千年月桂露", "相形变位", "上行符", "下行符", "太公杖", "捆仙绳", "幸运币", "黄钥匙", "红钥匙", "蓝钥匙", "威力丹", "金刚丹", "回春丸", "长寿丹", "桃木棒", "玄铁棒", "真银棒", "金箍棒", "道袍", "铁甲", "锁子甲", "紫金龙鳞甲", "天宫小犬", "天宫大犬", "伴炉童子", "护庭小神", "守园仙婢", "护庭校尉", "巡天卫士", "巡天卫士", "巨力士", "执火道人", "奎木狼", "执瓶仙侍", "金刚力士", "护持迦蓝", "侍案仙女", "护庭神将", "赤力韦驮", "护丹老道", "伏魔韦驮", "兜率宫星君", "无量护法", "兜率宫老仙", "二郎执旗将", "杀破狼", "二郎骁骑将", "尊天韦驮", "啸天犬", "太上老君", "杨戬", "玉皇大帝", "玉皇大帝", "牛魔王", "天蓬元帅", "巨灵神", "哪吒", "路点", "太白金星", "天宫商人", "乌金棍", "银缕锁甲", "封印门", "传怪点", "剧情点", "嫦娥", "黄金钥匙", "日月无极钹", "菩提老祖"};
        this.enemyBaseHp = new int[]{35, 45, 35, 50, 60, 55, 100, 50, 260, 60, 130, 100, 320, 20, 320, 100, 210, 220, 160, 200, 230, 220, 200, 360, 180, 180, 1200, 4500, 1500, 8000, 800, 5000, 120, 444, 100};
        this.enemyBaseAtk = new int[]{18, 20, 38, 42, 32, 52, 180, 48, 85, 100, 60, 95, 120, 100, 140, 680, 200, 180, 230, 380, 450, 370, 390, 310, 430, 460, 180, 560, 600, 5000, 500, 1580, 150, 199, 65};
        this.enemyBaseDef = new int[]{1, 2, 3, 6, 8, 12, 110, 22, 5, 8, 3, 30, 15, 68, 20, 50, 65, 30, 105, 130, 100, 110, 90, 20, 210, 360, 20, 310, 250, 1000, 100, 190, 50, 66, 15};
        this.enemyBaseGold = new int[]{1, 2, 3, 6, 5, 8, 100, 12, 18, 12, 8, 22, 30, 28, 30, 55, 45, 35, 65, 90, 100, 80, 50, 40, 120, 200, 100, 1000, 800, 500, 500, 500, 100, 144, 30};
        this.levelScriptLines = new String[]{"CES_84_6_11 MOV_0_5_11 TAK_8_9 CES_70_5_8 TAK_10_10 ROS_4_1 TAK_11_17 ROS_4_2 TAK_18_19 MOV_0_5_10 TAK_20_21 DES_70_5_8 LAY_2 ROS_1_4_7 ROS_2_0 ROS_2_6 RES_0 GUT_1 ", "TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8 ", "TAK_33_36 MOV_72_1_8_1_10 DES_72_1_10 ", "TAK_37_37 MOV_0_6_5 TAK_38_39 TAK_41_41 DES_44_1_3 DES_44_2_3 DES_44_3_3 DES_46_2_4 DES_44_9_3 DES_44_10_3 DES_44_11_3 DES_46_10_4 CES_44_5_4 CES_46_6_4 CES_44_7_4 CES_44_5_5 CES_44_7_5 CES_44_5_6 CES_46_6_6 CES_44_7_6 SWD TAK_42_42 ", "CES_72_1_11 TAK_43_44 MOV_0_6_3 MOV_72_1_11_6_2 ROS_4_1 TAK_45_48 MOV_72_6_2_6_1 DES_72_6_1 ", "TAK_49_52 MOV_72_9_1_7_1 DES_72_7_1 ", "TAK_53_58 ", "TAK_60_60 MOV_72_3_2_8_4 TAK_61_61 CES_47_8_3 CES_47_8_5 TAK_62_63 DES_72_8_4 DES_47_8_3 DES_47_8_5 ADD_2_72_11_10 ", "ROS_4_1 CES_73_10_1 MOV_73_10_1_6_9 TAK_68_74 MOV_73_6_9_6_10 ", "SWD ", "CES_72_3_10 MOV_72_3_10_2_10 MOV_72_2_10_4_9 TAK_76_78 DES_72_4_9 ", "SWD ", "TAK_84_87 MOV_58_5_4_6_8 MOV_58_4_4_6_8 MOV_58_3_4_6_8 MOV_57_7_4_6_8 MOV_57_8_4_6_8 MOV_57_9_4_6_8 MOV_56_4_2_6_8 MOV_56_3_2_6_8 MOV_56_2_2_6_8 MOV_59_8_2_6_8 MOV_59_9_2_6_8 MOV_59_10_2_6_8 TAK_88_90 MOV_73_6_2_6_8 ", "CES_6_10_2 CES_60_10_2 ", "ROS_4_1 TAK_92_97 DES_73_6_8 TAK_98_100 DES_70_6_7 ", "ROS_4_1 CES_61_5_2 CES_61_6_2 CES_61_7_2 CES_61_5_3 CES_70_6_3 CES_61_7_3 CES_61_5_4 CES_61_6_4 CES_61_7_4 TAK_118_121 ", "TAK_123_126 GUT_37 ", "TAK_68_74 ", "DES_1_4_4 CES_20_4_4 ", "CES_84_7_7 MOV_84_7_7_6_8 TAK_0_1 MOV_84_6_8_1_8 TAK_2_2 MOV_0_2_8 TAK_3_3 MOV_84_1_8_1_1 TAK_4_4 MOV_0_1_2 TAK_5_5 MOV_84_1_1_10_1 MOV_0_6_1 DES_84_10_1 TAK_6_7 MOV_0_11_1 MOV_0_1_11 ", "TAK_40_40 ", "TAK_59_59 ", "TAK_75_75 ", "TAK_84_87 GUT_12 ", "TAK_101_105 ", "CES_22_6_6 ", "TAK_129_132 DES_84_11_4 GIN_1_1000 ", "TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13 ", "TAK_136_143 DES_84_1_11 TAK_144_144 GIN_0_19 ", "TAK_145_146 DES_84_9_8 GIN_1_1000 ", "MOV_84_6_3_4_3 MOV_84_4_3_8_3 MOV_84_8_3_6_3 TAK_107_107 ", "TAK_108_108 DES_10_6_6 MOV_0_6_5 TAK_109_112 TAK_147_148 DES_84_6_3 TAK_149_149 ", "TAK_79_79 ", "TAK_155_160 DES_72_11_10 ", "TAK_150_154 ", "TAK_161_161 ", "TAK_162_162 ", "GLV_1 ", "CES_6_4_1 CES_61_4_1 ", "", "TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 ", "TAK_167_167 SEE_4_10_168_168_0 ROS_4_1 SEE_2_8_169_169_0 SEE_2_8_170_170_1 ", "TAK_171_171 SEE_7_10_172_172_0 SEE_7_10_173_173_1 ", "TAK_174_174 SEE_7_9_175_175_0 TAK_176_176 SEE_8_8_177_178_0 ", "TAK_179_180 ROS_4_3 SEE_8_6_181_182_0 ROS_4_1 SEE_10_4_183_183_0 ", "ROS_4_1 SEE_8_3_184_184_0 ROS_4_0 SEE_6_7_185_185_0 ", "SEE_4_4_186_186_0 ROS_4_1 SEE_6_2_187_187_1 ", "ROS_4_0 SEE_6_2_188_188_0 SEE_4_4_189_189_1 ", "TAK_190_190 ROS_4_1 SEE_2_2_191_192_0 ROS_4_0 SEE_2_6_193_196_0 SEE_2_6_197_198_1 ", "ROS_4_3 SEE_1_11_199_200_0 ", "ROS_4_2 SEE_6_11_201_202_0 ROS_4_3 SEE_3_8_203_204_0 CES_6_3_10 SEE_3_8_205_205_1 ", "TAK_206_209 ", "TAK_210_211 SEE_11_11_212_212_0 SEE_11_7_213_214_0 TAK_215_216 ", "TAK_217_218 ROS_5_510 ROS_6_510 CES_36_11_8 SEE_11_8_219_221_0 CES_40_11_9 SEE_11_9_222_222_0 SEE_8_9_224_225_0 SEE_10_9_223_223_0 DES_11_10_9 ", "TAK_224_224 SEE_6_2_225_225_0 ", "TAK_226_227 ROS_5_510 ROS_6_510 CES_36_5_1 CES_40_7_1 TAK_228_228 ", "TAK_229_231 MOV_0_6_7 TAK_232_232 DES_51_2_8 DES_51_1_8 DES_51_2_9 DES_51_1_9 CES_51_6_6 CES_51_5_7 CES_51_6_8 CES_51_7_7 GUT_57 ", "MOV_51_6_6_6_7 ROS_4_3 MOV_51_5_7_6_7 ROS_4_0 MOV_51_6_8_6_7 ROS_4_2 MOV_51_7_7_6_7 ROS_4_1 TAK_233_235 MOV_74_7_4_6_7 GUT_61 ", "", "", "", "TAK_236_238 MOV_75_5_4_6_7 TAK_239_239 GUT_62 ", "CES_69_5_5 CES_47_5_6 CES_47_5_7 CES_47_5_8 CES_56_7_6 CES_56_7_7 CES_56_7_8 TAK_240_245 GUT_63 ", "CES_77_6_6 TAK_246_250 DES_71_6_3 DES_77_6_6 DES_69_5_5 DES_-66_0 DES_-56_0 DES_-49_0 DES_-47_0 DES_-69_0 ", "TAK_253_254 ", "TAK_127_128 END_0 ", "TAK_261_263 SMS_0 ", "TAK_264_268 "};
        this.dialogueSpeakerType = new byte[]{0, 84, 0, 84, 0, 84, 0, 0, 84, 0, 70, 0, 70, 0, 70, 0, 70, 0, 84, 0, 0, 70, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 75, 0, 75, 75, 75, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 74, 0, 74, 0, 74, 0, 74, 72, 72, 47, 72, 0, 72, 72, 0, 73, 0, 73, 0, 73, 0, 73, 73, 72, 0, 72, 72, 0, 72, 72, 0, 73, 0, 73, 0, 73, 0, 73, 0, 70, 73, 70, 73, 70, 73, 73, 0, 70, 69, 0, 69, 0, 69, 84, 0, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 70, 0, 70, 0, 70, 72, 0, 72, 0, 0, -1, 0, 84, 0, 84, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 84, 0, 0, 0, 0, 69, 69, 0, 69, 0, 0, 72, 0, 72, 0, 72, 0, 0, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 0, 87, 87, 87, 87, 87, 87, 0, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 87, 71, 71, 0, 71, 71, 74, 71, 75, 0, 71, 71, 69, 47, 69, 56, 69, 47, 77, 77, 77, 71, 0, 56, 71, 0, 0, 87, 0, 87, 0, 87, 0, 87, 0, 0, 87, 87, 87, 87, 87};
        this.f_byte_arr_22 = new byte[]{7, 5, -1, 3, -1, 5, -1, 1, 10, 0, -1, -1, -1, -1, -1, -1, -1, 1, 9, 7, 8, -1, -1, 5, 11, 2, 6, -1, -1, -1, -1, 3, 11, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 11, -1, 5, 0, 5, 9, 2, 10, 3, -1, -1, 11, -1, -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, 10, -1, -1, 9, 0, 1, -1, -1, -1, -1, -1, -1, 3, 11, -1, 0, 3, -1, -1, -1, -1, -1, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, 5, -1, 5, -1, 1, 3, -1, 8, 9, 8, 9, -1, -1, -1, 8, 4, 5, 6, 10, 8, -1, -1, 5, 9, 1, -1, 0, 3, 7, -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, -1, -1, 0, -1, -1, -1, -1, -1, -1, 7, -1, 7, -1, 7, 2, 2, -1, 2, -1, -1, -1, 7, -1, -1, -1, -1, -1, -1, -1, -1, 9, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1};
        this.dialogueTexts = new String[]{"姑娘，您……", "哎呀，姑奶奶我好不容易怀揣2个仙桃夹带出来，居然被发现了~！快闪！！", "……，跑都跑的这么优雅。", "哎呀，跑这么快……", "修逗妈待~！……", "死猴子，穷追猛打，追的人家小心肝噗嗵噗嗵地……", "啊~~~姑娘哟~~姑~~~娘~~！！", "死开死开~！！好狗不挡路。", "居然追到这里了，算了吧，认了吧。\n\\cFFCC00两个桃子滚落地上", "哎~~~~~~~", "我咤~~~~！天庭圣地，禁止喧哗。", "我~~~~", "我什么我~？？整天身居仙位，游手好闲……", "你~~~~~~~~~", "你什么你~？？看嫦娥衣杉不整，是不是你非礼~？？是不是你是不是你是不是你！？", "她~~~~~~~~~~", "她什么她~？？哦~！还滚出来2个仙桃！！假公济私是吧~！！猴子偷桃是吧！！", "俺没~！……", "……", "你……叫嫦娥？这个名字，俺，记。住。了……", "没错，桃子就是俺偷的！！俺认罚！", "将这只孽畜剥下铠甲打下天牢！", "贤弟，快醒醒……", "哎呦……老牛，醒来见你那张脸，真提神！！", "我宁愿看着你，睡得如此沉静，胜过你醒时决裂般无情~~", "靠，发春呐，把~手~拿~开~~！", "哟~~挺横得嘿~牛哥学得一手好摄影……刚才……", "……大哥久违了，多日不见，受小弟一拜！！", "话说你一直在梦中叫着一个人的名字，她一定偷了你很多桃子……", "没错，她是偷了我的东西，但是不是桃子，我很想再见她……", "恩恩，现在大哥带你从密道出去……", "……这你都能挖开~！！", "哥在魔界有个绰号，叫“溜得滑”，没有什么囚牢能困住哥……", "大哥！！能不能分我件东西遮遮羞……", "有，我这有把刚挖地道的破勺，你拿去挡住先！", "靠……算了，总比没有好。……", "没办法，你就将就一下啦。\\cFFCC00有件道童的旧袍子在9楼，你仔细找找吧，\\cF8F8F8我先撤了……", "妖猴，你可认得本少爷！！", "咝~~我看你骨骼清奇三头六臂，猜得不错的话，阁下一定是畸形儿！", "啊~！大哥果然高手！这都被你看穿了！不能留你活口了！", "哎呀呀~~我回去告诉我爹去！", "来人！把妖猴拿下！", "哦~~~喳~！李小猴踢馆！！", "贤弟，慢着！", "溜得滑？", "我偷偷告诉你哦，有个银棍在17层，你找到它以后就能厉害很多？", "了解！", "红了红脸", "尽快去吧，我闪先……", "贤弟，这只啸天犬在这里挡道太危险了，跟哥哥绕密道吧。", "一只看门的宠物，能凶到哪里？", "不是，自从调到天庭守路，已经很多天没有吃荤腥了……", "了解，绕道，走~！", "我乃托塔天王麾下先锋官，巨玲神是也！你欺我孩儿……", "欺你孩儿~？", "其实……哪吒是奴家跟托塔天王的私生子……", "你是女将！？", "其实，看我像芙蓉姐姐一样的好身段，你就应该了解……", "……咝~~托塔天王口味这么偏……", "啊~~~天王，巨玲儿不能陪伴你了……", "看来你已经打通暗墙了，看哥的。", "噢，通完收工，走！！……", "我们是天庭市容管理队！！啊哈~乱堆渣土，违规施工终于被抓个现行！", "哎呀~！兄弟，一定要来2层天牢救我呀！", "", "", "", "", "你就是孙悟空！？", "俺就是。", "我常去广寒宫看她，她这几天常常提到你……我从来不知道什么叫做嫉妒，但是这次，我想要你的命。", "哼，你哪位？", "我乃天蓬元帅，朱刚鬣！！掌管天河……", "猪肛裂？？好，满足你这个愿望……", "哎……我还没报完呢……", "菊花残~~~满地伤……", "你跟杨戬一战，天庭都开了盘口，赔率是1:5，哥把私房钱都押到你头上了，表辜负哥哥哟~", "……真想背后给你一闷棍……", "天庭禁止乱堆渣土，哥哥把渣土运到“天庭城管办事处”去，就不算乱堆了，捏嘿嘿~~", "\\cFFCC0023层乃是29层的地基所在，找出暗藏的墙，就可以让29层的墙松动，哥哥就可以挖穿它。", "……你个假仗义！", "兄弟，赶快去凑齐装备，打败杨戬，", "哥哥就发达了~嘿嘿~", "……", "上次趁本元帅自报大名的时候，突袭本帅，本帅不跟你计较，单挑还是群殴，你自己选。", "恩，是条汉子，俺就认真跟你打一次，单挑！", "单挑是吧，你一个单挑我们全部，弟兄们，一起上！", "你丫不地道！", "你的确是个英雄，难怪她一直念念不忘……", "过奖过奖，你的部下都躺下了，现在轮到你了……", "恩~~讨厌死了，来了来了……", "人家今天身体不方便，改天再来，先闪了", "天蓬，你数次战悟空不胜倒罢了，平时常常擅自离岗，去广寒宫把妞……死罪可免，活罪难饶。", "听说，下凡投胎，就会堕入轮回，就会忘记前尘往事……", "好，朕就成全你，下凡之前，有什么要求么？", "天天大吃大喝，倒头睡觉，生活安逸无边，心宽体胖……", "很好，你的心意，朕明白了，安心去吧", "哇！！投胎为猪？？！", "天蓬元帅变成了一只猪，被贬下了凡尘", "……真阴险……", "哼，哼，寡人在四十九层等你，哇哈哈哈哈~！！", "本来，战神情圣的名号是我的；嫦娥的心，迟早也会归属于我，但是你来了之后，一切都改变了……", "你喜欢她，这么多年，你为什么不去找她？", "因为我是战神情圣，是不能失败的……", "你太骄傲了……", "无论如何，斗神和战神这一战，是注定的……", " ", "哇，姑娘身陷三昧真火当中，要想办法开门灭火……", "姑娘顶住，俺老孙来救你！！", "不要不要过来！！", "姑娘你没事吧？", "死猴子，月宫阴冷，姑奶奶我想蒸蒸桑拿，治多年的关节炎都不行……", "……", "死猴子，上次桃子的事情……", "俺掌管蟠桃园，偷吃仙桃何止千百，多认2个，算什么……", "害你被革除了“齐天大圣”的上仙之位……", "俺老孙不稀罕天宫的位子，~贬下凡尘仍称王，嘿嘿", "……在天宫几千年，从来没有人肯为我放弃仙位……唉，可惜。", "哼哼，居然打到这里，实话告诉你，所有天神都对你不满，这次你被削去仙爵打入天牢，都是计划之中。", "那嫦娥呢，桃子呢？也在计划之中？你们料定俺会甘心顶罪？", "哇哈哈哈哈~！天网易逃，情丝难断，你有通天的本事，也难过这一关。", "为她顶罪，俺从不后悔，现在，是俺了断恩怨的时候了！！", "其实，朕不是打不过你，朕只不过私挪了国库，买了你的盘口……", "你终于打到这里了。", "你居然在这里？", "哈哈哈哈，老夫一路保你，就是为了让你帮我扫清天庭，你的所做所为…", "俺最恨的就是被人欺骗！我……（悟空久久地陷入了回忆）", "天庭，天庭又怎样？女人骗我，兄弟骗我，如今俺老孙没有什么可以留恋的，回花果山罢了。", "孽畜，天庭威仪，岂能容你全身而退！！", "姑娘，你怎么会被关在这里！", "奴家暗中助你，触犯天条……", "玉帝老儿，待我打烂你的金冠！！姑娘你先离开，等俺回来！", "唉……你又何苦……", "这里有瓶火眼金睛牌眼影霜，去皱抗衰老，可以看清楚敌人的本质，金色质感贴合肤质，来自巴黎，你值得拥有。", "现在涂好了，看起来嗲不嗲~？", "恩……本来是只“猴妖”，现在是个“人妖”。", "嫦娥姑娘，想不到在这里遇到你。", "大圣，这是我亲手酿制的千年月桂露，喝下它，犹如脱胎换骨，体力大增。", "哦~？难道这是定情信物？", "而且，它还可以使人忘记红尘感情，我希望你能忘记我。", "啊~哈~给我一杯忘情水~换我一夜不流泪……姑娘，俺准备离开天庭，我希望你跟我一起走……", "违背天条，私奔，会被整个天界人肉搜索的……", "私奔？俺老孙不做那猥琐之事，待俺打上灵霄宝殿，让玉皇大帝亲口答应，整个天庭谁敢为难你！！", "大圣保重，此地奴家不宜久留，奴家不想连累你……", "姑娘！姑娘！", "大圣，前面凶险难测，奴家这里有点私房钱，送给你买点仙丹滋补身体吧……", "……姑娘对我一片真情，俺发誓要为你打下一片天", "跟俺走吧，回花果山去……", "表，姑奶奶我为了天宫护照，抛弃了前夫，我才表再跟你下凡，你……是个好人……(飘走)", "......女人如衣服，兄弟如手足，老牛~！俺来寻你！！", "宝扇宝扇告诉我，谁是这个世界上最型最猛的男人？", "（模仿扇子的声音）是你~是你~还是你", "真自恋……", "哇~！被你偷窥到了，本尊该杀你灭口，但是现在你还不配本尊出手。", "可恶，等俺老孙先找回俺那根如意棍子再来收拾你……\\cFFCC00先去2层天牢救老牛，让他替俺开暗墙绕过去", "老牛，俺救你来了~！", "平时让你帮忙，老是推三阻四，这次这么爽快，一定有问题~", "嘿嘿，35层有个三眼小白脸太恶心，替俺灭了他~~", "俺对小白脸木有兴趣……", "那就想办法帮俺绕过去~~", "嘿嘿，开自己的洞，让别人说去吧~~~", "?有根捆仙绳？似乎可以克制住那个三眼小白脸，恩，搞定他，捆绑他，拿他的芭蕉扇，哦也~", "有芭蕉扇可以灭火了，嫦娥姑娘，俺来啦~~！！对了，还有我的如意金箍棒。", "欢迎你来到天宫世界，我是你的师傅菩提老祖。", "在这里我不会教你七十二变，但是我会教你怎么游历天宫。", "为师知道你要大闹天宫，特意千里传音，提供远程视频支持，当然，如果你嫌为师罗嗦，也可以在游戏菜单中选择跳过教程。", "好了，现在请试着\\cFFCC00按方向键移动到这里。", "很好，你已经学会太空步了。", "在你面前有一道黄色的门，你无法过去。", "你可以看到这里有把黄钥匙，它可以开启这道门。", "现在\\cFFCC00移动到这里，再回来开门。", "等等！", "前面有只挡路的狗。你需要打败它才能走过去。", "现在，\\cFFCC00请试着移动到它的位置上，与它战斗吧。", "听到轰隆声了吧，因为你打败了\\cFFCC00守卫封印门的敌人。", "所以这里的\\cFFCC00封印门\\cF8F8F8就被打开了。", "在战斗中你可能会损失血量。", "这里有个\\cFFCC00小仙桃，可以回复你的血量。", "如果血量不足，你将无法挑战敌人。", "又到了学习时间。", "你的能力是可以提升的，包括攻击、防御、血量。", "这里有个蓝色仙丹，学名是“防御仙丹”，服下它，可以提升你的防御力，让你战斗更持久。", "记住，\\cFFCC00天庭层数越高，仙丹药效越大。", "现在，吃了它，扑过去做掉前面那条狗，你会发现损血少了。", "看到上面的蓝门了吗，它只能用蓝色的钥匙打开。", "它被藏在这里，\\cFFCC00先拿到它吧。", "你发现了一道红门。这种门很少见，必须用红钥匙才能打开。", "它被藏在这里，\\cFFCC00请先得到它，再回来开门。", "你找到了一把红钥匙，这种钥匙比较稀少。", "试着\\cFFCC00用它开启这里的红门。", "很好，这层已经接近尽头。", "你会发现这样的红色传送点，它可以让你向上一层楼。", "不过，别急着离开。", "你是不是已经发现这里有个道具了吗？", "这里有个红色仙丹，学名是“攻击仙丹”，服下它，可以提升你的攻击力，让你战斗更狂野。", "但是这里好象不通……", "别急！俗话说车到山前必有路，在天宫的很多层中会有隐藏的路，更多惊喜更多欢笑，就在隐藏路……", "现在，移动到这里，你就会发现它。", "要记住，\\cFFCC00很多层里都会有隐藏的东西，试着去探索吧。", "看到你上来的路了吗？", "蓝色的传送点可以让你向下一层楼。", "你可能无法击败这个敌人，绕道也是前进的办法。", "那么，如何判断一个敌人的强弱呢？", "游戏中你会获得这件宝物，它叫\\cFFCC00火眼金睛牌眼影膏。", "涂抹一点在眼皮上，你可以看破敌情，还可以去除眼角纹。", "我帮你开出了一条路，你可以去取它了。", "现在你可以参照使用说明来使用它了。", "除了按5/OK键查看敌人对你造成的伤害以外。", "你还可以按左软键打开物品栏。", "选择该物品，按确认键查看更详细的敌人信息。", "你站在这十字街头上，找不到来去的方向。", "不要慌张，试着查看下这里道具和敌人的分布形势。", "上去的传送点在这里。", "如果你无法马上击败这个守卫。", "就试着将地图上的仙丹和仙桃吃掉，然后你就可以战胜它了。", "记住，如果前方有一群敌人在向你挥手，千万别冲动。", "牢记\\cFFCC00“先吃仙丹后肉搏”\\cF8F8F8是减少损血的第一法则。", "哦嘿嘿~为师要继续享受渡假啦~。", "在这之前我会传授你五百年功力，再送你两件东西防身。", "这是一把武器，能让你提升很高的攻击。", "在游戏的\\cFFCC00每10层都有一把新武器。", "如果你能早点获得它，就能轻松应对敌人，走得更远。", "同样，这是一件防具，能提高你的防御。", "我现在帮你打开这道墙，在一般情况下，它是无法击碎的。", "对了，给你介绍一个天宫上的朋友。", "这是为师的老朋友，\\cFFCC00太白金星\\cF8F8F8，他会暗中帮助你的。", "恭喜，你已经毕业了，我再传授你五百年的功力。", "还给你准备了两件神器，把它们收下吧。", "要记住，往前你将面对的不是一个敌人，而是整个天宫。", "捏哈哈~今天风和日丽，朕心情很好~！", "呀~！哪里钻出一只果子狸！预防非典！左右与我拿下！", "什么破皇帝，五谷不分，六畜不辨……玉帝老儿，俺乃花果山第一届型秀冠军，美猴王孙悟空！！", "都给我上~！", "巨玲神，愣在那里做什么？", "陛下，人家是女孩子嘛，最怕毛茸茸地小动物了……", "少废话！想被砍头啊！", "陛下，三太子请战！", "玉帝老儿，俺老孙要做齐天大圣！！", "嗷~~它过来了它过来了，请你个头啊，赶紧顶上先~~", "护驾~！护驾~！", "谁~！是天庭第一型男~！！", "是你~！是你~！", "谁~！是天庭第一猛男~！！", "是你~！是你~！", "谁~！是天庭第一明星战神~！！", "你是电，你是光，你是唯一地神话，你主宰，我崇拜~没有更好的办法~~！", "陛下……呕……", "臣觉得吧……呕……", "还是不要让二郎神继续下去了，尽快平息这次事件，给孙悟空封个官算了！！", "呕~~朕也是这么想滴~~喔莱，就封孙悟空为齐天大圣，掌管蟠桃园！", "嘿嘿，俺老孙就领了~！多谢~！", "妖…妖怪，他的眼睛闪着红光……太可怕了", "居然退缩，你这没用的东西。", "噢~~~~~~满园仙桃成熟，煞是诱人！！", "让俺老孙四处逛逛~~", "悟空~悟空~！", "为什么有只酱油蛤蟆跟我千里传音？", "是师父我啊！", "师父？为什么化个蛤蟆妆？", "为师在印度，法术交流兼渡假，日光浴加顿顿咖喱饭，嘴巴上火。", "如来把孙悟空封印于五指山下，伴随着所有恩怨情仇，欲知后事如何，敬请期待《西游记二之大圣取经》", "爱徒啊，你一个人要继续挑战天宫，为师不放心，给你买了份保险。", "师傅够义气。", "（接过保单）受益人...“菩提老祖”", "我的徒儿啊，你已经陷入混沌世界。也就人们常说的精神分裂症。", "这是由于种种感情纠葛引发的，如果你想出来，就要战胜他们。", "不，是战胜自己。要让这天…再也遮不住你的眼。", "我要提醒你的是，这个世界里，所有的敌人都会比原来更强，当然你的能力也会提升更多。", "去吧……劫难在所难免。"};
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
        String[] stringArray = new String[]{"界面", "菜单", "地图", "背景", "人物", "组件", "表情", "效果", "脚本", "缓存", "楼层", "角色", "设定", "敌人", "引子", "结局", "载入进度"};
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
        this.f_String_arr_16 = new String[]{"手机号", "密码", "试玩", "注册", "帮助", "注册帐号", "注册密码", "注册手机号必须为本机，验证成功后将获得200游戏币", "欢迎登陆雪鲤鱼平台", "确定", "取消", "退出", "联网中", "正在重试", "联网超时", "服务器没有响应", "登陆", "剩余点数", "查询记录", "充值", "个人信息", "帐号或密码输入有误", "请输入11位手机号", "请输入6-10位密码", "日期", "金额", "本月付费记录", "进入游戏", "请确认您填写的是当前手机号，否则会导致注册失败。资费2元，需要发送1条短信，2元/条，不含通信费。是否注册？", "已发送成功，请等待系统验证后才能登陆", "发送失败，请重试", "类型", "序列号", "密码", "金额", "元", "已提交充值信息，请在充值记录中查看充值结果。", "返回", "状态", "最近充值记录", "提交过程出现错误，请检查充值内容。", "：", "\n", "是否确定退出游戏？", "您尚未登陆，请输入您的手机号和密码联网付费", "序列号和密码填写有误", "短信充值", "已发送成功。", "服务器响应错误！", "使用本产品必须先登录，与雪鲤鱼其他产品中注册的手机号，密码，帐号中的游戏币通用。如果没有帐号可以注册，送游戏币。登录后付费信息更安全，避免掉存档。", "充值的游戏币在所有雪鲤鱼相关产品中通用。", "请输入11位手机号", "请输入6位以上密码", "是否马上注册？需要确认您填写的是当前手机号，否则会导致注册失败。资费2元，需要发送1条短信，2元/条，不含通讯费。", "注册失败！如果该手机号是您的真实号码。将会把您的密码改为当前注册填写的密码，该过程可能需要一段时间。", "已注册成功。待验证成功后就能用该号码充值！并购买游戏中的付费内容。如果再次注册还可以发送短信修改密码。", "您尚未验证，如果之前已发送短信验证，请稍候再登陆重试。", "客服电话：400 630 5518", "您的手机无法连接到服务器，不能进行游戏。是否重新尝试联网？"};
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
                ((a)object).f_Graphics_00.drawString("是否开启声音？", 120, 160, 17);
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
                ((a)object).f_Graphics_00.drawString("花费", n25, n4 += 21, 0);
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
                    ((a)object).f_Graphics_00.drawString("请按任意键", 120, 320 - ((a)object).f_int_01 - 2, 17);
                } else {
                    ((a)object).f_Graphics_00.setColor(-1);
                    ((a)object).f_Graphics_00.drawString("跳过", 240 - ((a)object).f_Font_00.stringWidth("跳过") - 5, 320 - ((a)object).f_int_01 - 2, 0);
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
                                        super.m_015((byte)0, "保存成功！", (byte)0, (byte)0);
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
                                    super.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
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
                this.m_015((byte)0, "游戏描述：\n\\c99FFCC有人的地方就有江湖，有神仙的地方何尝不是江湖；百战百胜的本事，换不回女人的真心，兄弟的真义；齐天大圣又如何，没有真情实义，做神仙跟做咸鱼有什么区别？\n\n操作方式：按左软键调出物品栏，左右选择一件道具，按确定键使用。\n游戏操作：\n上方向键/2：向上行走\n下方向键/8：向下行走\n左方向键/4：向左行走\n右方向键/6：向右行走\n确定键/5:探索地图\n左软键：打开道具列表\n右软键：打开游戏中菜单\n\n代理发行：广州易诚计算机科技有限公司\n发行商网站：www.9266.net\n客服电话：4006509913\n客服信箱：kefu@9266.net", (byte)0, (byte)0);
                this.f_byte_13 = 0;
                this.f_byte_14 = (byte)3;
                return;
            }
            case 17: {
                this.m_001(8);
                this.m_001(1);
                this.m_015((byte)0, "版权所有：\n上海雪鲤鱼计算机科技有限公司\nwww.kgame.com.cn\n手机上网：\nwap.kgame.com.cn\n制作人：梁一\n编剧：王之浣\n策划：孙悦\n程序：杨政\n美术：梁一、王之浣、黄吉力\n测试：金鑫，王毅，计成毅\n版本：V1.0\n客服电话：4006305518", (byte)0, (byte)0);
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
                        this.m_015((byte)0, "你无法战胜它", (byte)0, (byte)0);
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
                            this.m_015((byte)0, "你没有黄钥匙", (byte)0, (byte)0);
                            break block0;
                        }
                        case 3: {
                            n5 = this.consumeKeyForDoor((byte)28);
                            if (n5 != 0) {
                                this.m_047(n3);
                                break block0;
                            }
                            this.f_int_148 = 0;
                            this.m_015((byte)0, "你没有蓝钥匙", (byte)0, (byte)0);
                            break block0;
                        }
                        case 2: {
                            n5 = this.consumeKeyForDoor((byte)27);
                            if (n5 != 0) {
                                this.m_047(n3);
                                break block0;
                            }
                            this.f_int_148 = 0;
                            this.m_015((byte)0, "你没有红钥匙", (byte)0, (byte)0);
                            break block0;
                        }
                        case 81: {
                            if (!this.f_bool_arr_00[n3]) break;
                            this.m_015((byte)0, "障碍物：封印门\n需要消灭指定的怪物才能打开的门！", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 4: {
                            this.m_015((byte)0, "障碍物：封印门\n需要消灭指定的怪物才能打开的门！", (byte)0, (byte)0);
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
                            this.m_015((byte)0, "障碍物：三昧真火\n必须用芭蕉扇才能熄灭它. ", (byte)0, (byte)0);
                            n5 = 0;
                            n7 = 0;
                            break block0;
                        }
                        case 11: {
                            if (this.currentFloor != 23) {
                                this.m_015((byte)0, "障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！", (byte)0, (byte)0);
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
                this.f_Graphics_00.drawString("无", n, n2 + 2, 0);
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
            this.m_142("引子", 32, n + 9 + (19 - this.f_int_01 >> 1), 17, this.f_int_arr_36);
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
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 4: {
                                if (this.spendGold(50)) {
                                    this.yellowKeyCount += 5;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 5: {
                                if (this.spendGold(1000)) {
                                    ++this.yellowKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 6: {
                                if (this.spendGold(800)) {
                                    ++this.redKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 7: {
                                if (this.spendGold(200)) {
                                    ++this.blueKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 8: {
                                if (this.yellowKeyCount > 0) {
                                    --this.yellowKeyCount;
                                    this.goldAmount += 100;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有黄钥匙", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 9: {
                                if (this.spendGold(1000)) {
                                    ++this.yellowKeyCount;
                                    ++this.blueKeyCount;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 10: {
                                if (this.spendGold(200)) {
                                    this.yellowKeyCount += 3;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 11: {
                                if (this.spendGold(2000)) {
                                    this.blueKeyCount += 3;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 12: {
                                if (this.spendGold(1000)) {
                                    this.playerHp += 2000;
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                                bl2 = false;
                                break block0;
                            }
                            case 13: {
                                if (this.spendGold(4000)) {
                                    this.pickupItemType(18);
                                    break block0;
                                }
                                this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
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
        this.m_015((byte)0, "没有记录", (byte)0, (byte)3);
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
                    this.m_015((byte)0, "你必须面对三昧真火再使用它。", (byte)0, (byte)0);
                    break;
                }
                this.gameMode = (byte)3;
                n = 1;
                break;
            }
            case 17: {
                if (!this.m_089((byte)11)) {
                    this.m_015((byte)0, "你必须面对一堵墙使用", (byte)0, (byte)0);
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
                this.m_015((byte)0, "增加了" + n + "血量", (byte)0, (byte)0);
                n = 1;
                break;
            }
            case 20: {
                if (this.currentFloor == 40) {
                    this.m_015((byte)0, "本层不能直接瞬移。", (byte)0, (byte)0);
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
                this.m_015((byte)0, "无法移动到该位置", (byte)0, (byte)0);
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
                this.f_Graphics_00.drawString("存档" + n, 120, n3 + 2, 17);
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
                this.f_String_07 = "重定向!!!";
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
                System.out.println("下行数据长度：" + n + "，内容：" + object3);
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
                    this.f_String_07 = "正在重试(" + this.f_int_166 + ")";
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
            this.f_String_07 = "已超时，请重试。";
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
