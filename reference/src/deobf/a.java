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

public final class a extends Canvas implements Runnable {
   private Font f_Font_00 = Font.getFont(0, 0, 8);
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
      this.f_String_arr_00 = new String[]{
         "无法再下一层了，这是你达到的最底层",
         "无法再上一层了，这是你达到的最高层",
         "无法再下去了。",
         "无法再上去了。",
         "俺，当世神界第一斗者，孙!悟!空! 自从受封为齐天大圣，掌管蟠桃园以来，一直逍遥快活，无拘束……",
         "你拥有更强力的装备，因此将捡到的丢弃了。",
         "直到那一天，遇到了她，在筋斗云上的我，竟然第一次心潮起伏，有了晕机的感觉……",
         "神仙动了感情，往往会万劫不复，\n这一次，让我付出了五百年的时间去忘记她……\n五指山脚下的沙子，掠过我的脸庞。\n沙子，跟时间一样，同样随风流逝；同样掩埋过去；\n多少次伸手想抓住，却从指隙溜走……\n看夜空，半梦半醒间，往事历历上心头……"
      };
      this.f_int_04 = 0;
      this.f_Image_00 = null;
      this.f_String_arr_01 = new String[]{
         "sflogo", "mapbg", "map", "actor", "sptmap", "sptprop", "sptarm", "sptenemy1", "ui", "xtq", "menu", "intro", "face", "sptenemy2", "end", "load"
      };
      this.f_int_arr_00 = new int[]{8, 1, 12, 4, 13, 23, 10, 20, 25, 6, 2, 2, 12, 20, 1, 2};
      this.f_Image_arr2_00 = new Image[this.f_int_arr_00.length][];
      this.f_int_arr2_00 = new int[][]{{0, 1, 2}, {3, 5}, {4, 5}};
      this.f_OutputStream_00 = null;
      this.f_InputStream_01 = null;
      this.f_String_arr_02 = new String[]{
         "新游戏", "继续游戏", "载入进度", "保存游戏", "设置", "帮助", "关于", "退出", "返回菜单", "回放", "停止回放", "商店", "十全大补包", "攻防神油", "开门天天乐", "天庭消费券", "印度神血油", "跳过教程"
      };
      this.f_int_16 = 25;
      this.f_byte_arr_01 = new byte[]{-1, 0, 1, 1, 1, 0, -1, -1};
      this.f_byte_arr_02 = new byte[]{-1, -1, -1, 0, 1, 1, 1, 0};
      this.f_byte_03 = 0;
      this.f_int_17 = 0;
      this.f_int_18 = 0;
      this.f_int_arr_02 = new int[]{16316664, 11184810, 8947848, 4473924, 1118481};
      int[] var10000 = new int[]{0, 12, 24, 36, 47, 57, 67, 75, 83, 89, 94, 97, 100, 97, 94, 89, 83, 75, 67, 57, 47, 36, 24, 12, 0};
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
      this.f_int_arr2_02 = new int[][]{
         {0},
         {0, 1, 0, 2},
         {0, 1, 2},
         {0, 1, 2, 1},
         {0, 1, 2, 2, 1, 0},
         {0, 1, 2, 3, 2, 1},
         {0, 1, 2, 3, 4},
         {3, 4, 5, 6},
         {4, 3, 2, 1, 0},
         {2, 1, 0},
         {2, 3, 4}
      };
      this.f_byte_arr_06 = new byte[]{
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         2,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         3,
         3,
         3,
         4,
         2,
         3,
         3,
         3,
         3,
         3,
         3,
         2,
         3,
         2,
         2,
         2,
         3,
         2,
         3,
         3,
         3,
         2,
         3,
         5,
         3,
         3,
         3,
         3,
         3,
         3,
         0,
         0,
         0,
         2,
         2,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         0,
         2,
         0,
         0,
         0,
         0
      };
      this.f_byte_arr_07 = new byte[]{
         32,
         32,
         32,
         32,
         45,
         32,
         32,
         32,
         32,
         104,
         35,
         45,
         30,
         32,
         32,
         30,
         30,
         24,
         25,
         21,
         32,
         25,
         25,
         27,
         31,
         27,
         23,
         23,
         23,
         32,
         32,
         32,
         32,
         27,
         27,
         27,
         27,
         22,
         22,
         16,
         25,
         15,
         20,
         28,
         26,
         23,
         34,
         35,
         24,
         33,
         22,
         22,
         28,
         36,
         38,
         29,
         34,
         29,
         30,
         31,
         33,
         26,
         30,
         36,
         23,
         38,
         33,
         42,
         32,
         78,
         38,
         36,
         32,
         39,
         59,
         36,
         32,
         29,
         37,
         27,
         17,
         32,
         32,
         32,
         29,
         27,
         32,
         36
      };
      this.f_bool_arr_03 = new boolean[]{true, false, false, false, false, true, true, true, true, false, true, false, false};
      this.f_int_47 = 0;
      this.f_int_48 = 270;
      this.f_byte_arr2_00 = new byte[][]{
         {0, 0, 0, 33, 56},
         {0, 33, 0, 33, 56},
         {0, 66, 0, 33, 56},
         {2, 0, 0, 18, 26},
         {3, 0, 0, 18, 34},
         {1, 0, 0, 55, 61},
         {4, 0, 0, 14, 26},
         {4, 14, 0, 14, 26},
         {4, 28, 0, 24, 26}
      };
      this.f_byte_arr2_01 = new byte[][]{
         {6, -1, -79, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -53, 0, 0, -16, -27, 0},
         {7, -1, -80, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -54, 0, 2, -16, -26, 0},
         {8, -1, -81, 0, 3, -26, -5, 0, 4, 11, 6, 0, 5, -25, -55, 0, 1, -16, -25, 0}
      };
      var10000 = new int[]{0, 0, 11, 1, 1, 11, 11, 11, 0, 0};
      var10000 = new int[]{0, 0, 0, 0, 1, 1, 1, 11, 11, 11};
      this.f_bool_arr_04 = new boolean[]{
         false,
         true,
         false,
         true,
         true,
         true,
         true,
         false,
         true,
         false,
         false,
         true,
         true,
         true,
         true,
         false,
         true,
         true,
         true,
         true,
         true,
         true,
         false,
         false,
         true,
         true,
         true,
         true,
         true,
         true,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         true,
         true,
         true,
         true,
         false,
         false,
         false,
         false,
         true,
         true,
         true,
         true,
         false,
         false,
         false,
         false,
         true,
         true,
         true,
         true,
         false,
         false,
         false,
         false,
         true,
         true,
         true,
         true,
         false,
         false,
         false,
         false,
         true,
         true,
         true,
         true,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         false,
         true,
         false
      };
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
      this.f_String_arr_06 = new String[]{
         "",
         "",
         "有些门不能用钥匙打开，只有当你打败它的守卫后才会自动打开。",
         "你购买了礼物后再与天宫商人对话，他会告诉你一些重要的消息。",
         "",
         "我听说在天宫中有2把隐藏的红钥匙。",
         "在这个区域不多次提升攻击力，就不能打败“杨戬副手”。切记前人教训！",
         "太上老君就在25楼。以你现在的状态去攻击他简直就是自杀。 你应当在取得更高级别的道具后再去打败他。",
         "不找到所有的暗墙29楼的暗道是不会打开的",
         "如果你到27楼时状态为：生命1500、攻击80、防御98、拥有1把蓝钥匙、5把黄钥匙。那么祝贺你，你的前期是比较成功的。",
         "六丁六甲的攻击力太高了，你最好到能对他一击必杀时再与他战斗。",
         "别匆忙，放慢速度。",
         "如果你能用好4种移动宝物，你不用与强敌作战就能上楼。",
         "",
         "你需要用“玄明石”取出37楼仓库内的所有宝物。",
         "谜题：“在3点，拥有传送功能的密宝就会出现。”",
         "“巫师”会用魔法攻击路过的人，在2个“太上老君护卫”间通过会使你的生命减少一半。",
         "44楼，被藏在异空间，你只能用密宝才能到达。",
         "41楼事实上是左右对称的。",
         "像骰子上5的形状是一种封印魔法，你最好记住它在你与49楼假魔王战斗时有用",
         "",
         "你好，我是太白金星。你最好别见敌人就杀，先往上走，拿到武器和防具再做打算。"
      };
      this.f_String_arr_07 = new String[]{
         "感谢你救了我，这是1000金就送给你吧。",
         "试下火眼金睛吧，你能看到怪物的信息和战斗损失的血量，你可以在物品栏中使用它。",
         "",
         "",
         "很好，你居然找到了我，作为奖励我将给你一瓶千年月桂露，喝了它将按你的攻击力和防御力的综合增加的你生命点数，你越晚使用它效果越好。",
         "",
         "",
         "",
         "",
         "",
         "",
         "",
         "",
         "",
         "感谢你救了我，这是1000金就送给你吧。",
         "",
         "",
         "",
         "",
         "",
         "哈喽，送你1000金作为见面礼，记得经常来找我哦。",
         ""
      };
      this.f_byte_arr_12 = new byte[]{2, 2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1};
      this.f_String_arr_08 = new String[]{
         "大圣饶命，我可以提升3%攻击力和防御力，需要小的为你效劳吗？",
         "",
         "大圣爷，我这里有1把蓝钥匙，你给50金币吧。",
         "我有5把黄钥匙，一共50金币",
         "嘿嘿，我有很多把黄钥匙，1把1000金币",
         "我有1把红钥匙，只要800金币",
         "我有1把蓝钥匙，只要200金币",
         "我跟其他人不一样，你可以把多余的钥匙卖给我。100金币回收1把黄钥匙",
         "我有1把黄钥匙，1把蓝钥匙，一共是1000金币",
         "我有3把黄钥匙，只要200金币",
         "我有3把蓝钥匙，收你2000金币",
         "我可以恢复你2000点血，不过要1000金币",
         "我有个宝物，要收你4000金币，是个玄明石。"
      };
      this.f_String_arr_09 = new String[]{
         "",
         "",
         "天宫一共50层，每10层为一个区域。如果不打败该区域的头目就不能到更高的地方。",
         "在商店里你最好选择提升防御力，只有在攻击力低于敌人的防御力时才提升攻击力。",
         "",
         "你是否注意到5、9、14、16、18楼有的墙与众不同？",
         "如果你持有太公杖，面对天神力士和巨灵神时你的攻击力加倍。在没有太公杖的情况下你是无法打败巨灵神的。太公杖被藏在15楼以上的墙内。",
         "",
         "天宫一共有50层，但50楼并不能直接上去。",
         "存放乌金棍的房间的门坏了，你必须用金勺子破墙而入。",
         "天宫中藏有有个“幸运金币”拥有它在打败敌人后能够获得2倍的金钱。",
         "“紫金龙鳞甲”能防御“太上老君护卫”的夹击，但它被深藏在神秘的楼层中。",
         "如果要打败杨戬你需要“乌金棍”、“银缕锁甲”、“捆仙绳”或更高等级的宝物。"
      };
      this.f_byte_arr_13 = new byte[]{2, 3, 3, 3, 6, 3, 3, 6, 3, 3, 3, 3, 3};
      this.f_int_88 = this.f_String_arr_06.length + this.f_String_arr_09.length;
      this.f_byte_arr_14 = new byte[this.f_int_88];
      this.f_byte_arr_15 = new byte[this.f_int_88];
      this.itemDescriptions = new String[]{
         "能看破敌人底细，显示敌人详细信息。在游戏中按快捷键5也可以查看伤害量。\n\\c00ff00[使用次数：无限]",
         "记录前尘往事。\n\\c00ff00[使用次数：无限]",
         "在楼梯边，可以瞬间上下层，留神晕机。\n\\c00ff00[使用次数：无限]",
         "熄灭\\cFFCC33三昧真火\\r的神器。\n\\c00ff00[使用次数：无限]",
         "挖洞开墙越狱的利器,挫是挫了点，但是真的很好用。\n\\c00ff00[使用次数：1次]",
         "可以震开当前层所有的墙\n\\c00ff00[使用次数：1次]",
         "喝下后，增加相当于当前\\c00FFFF攻击力\\cFFFFFF加\\c00FFFF防御力\\cFFFFFF值740%的\\cFFCC00血量\n[月宫出品，手工酿制，不含三聚氰胺，冷藏效果更佳，使用次数：1次]",
         "瞬移到以中心为对称点的位置上。\n\\c00ff00[使用次数：3次]",
         "瞬移上行一层\n\\c00ff00[使用次数：1次]",
         "瞬移下行一层\n\\c00ff00[使用次数：1次]",
         "当年姜子牙受天命封神，他的钓鱼竹竿被原始天尊附上了神力，可以役使天神力士供他差遣，此杖又名“打神鞭”，对天神力士（包括巨灵神）威力加倍。\n\\cFFCC00[放在道具栏中有效]",
         "对某些自恋的神仙伤害加倍。\n\\cFFCC00[放在道具栏中有效]",
         "打怪得到的金钱加倍。\n\\cFFCC00[放在道具栏中有效]",
         "可以开启黄门。",
         "可以开启红门。",
         "可以开启蓝门。",
         "加攻击。",
         "加防御。",
         "加血。",
         "加血。",
         "开启当前层所有黄门",
         "如来开“慈悲为怀”巡回佛经演唱会的时候，伴奏罗汉用的乐器，道行浅的敌人，会被其梵天佛音瞬间化为灰飞\n\\cFFCC00使用：杀死上下左右的敌人，对BOSS不起作用。"
      };
      this.equipDescriptions = new String[]{
         "",
         "\\cdddddd一根相当长的木制长棍,新手必备.有了它杀人越货不慌不愁.\n\\c00ff00装备: 攻击+10.\n\\cFFCC00\"看上去似乎会断掉。\".",
         "\\cdddddd乌黑油亮，显然经历过多人之手。\n\\c00ff00装备: 攻击+30.\n\\cFFCC00\"很粗很结实！\".",
         "\\cdddddd银棍，恩，有这个名字就足够了。\n\\c00ff00装备: 攻击+70.\n\\cFFCC00\"只是根银棍\".",
         "\\cdddddd因乘天地之灵气，集日月之精华乃“万木之灵，灵木之尊”。\n\\c00ff00装备: 攻击+120.\n\\cFFCC00\"木之精华，削铁断金\".",
         "\\cdddddd您的需要，它知道；您的需求，它满足。它好，你也好，龙王后宫，镇宫之宝！\n\\c00ff00装备: 攻击+220.\n\\cFFCC00\"不要迷恋它，它只是一根传说。\".",
         "",
         "\\cdddddd没有太多的装饰，一件非常朴素、轻便的布衣.\n\\c00ff00装备: 防御+10.\n\\cFFCC00\"看上去有不少人用过了。\".",
         "\\cdddddd保暖御寒，腰不酸，腿不疼，走路也有劲了。\n\\c00ff00装备: 防御+30.\n\\cFFCC00\"豹纹，性感又野性，今年天宫最流行的皮草款式\".",
         "\\cdddddd如果没有上面的那行字，它也算是个杰作。\n\\c00ff00装备: 防御+70.\n\\cFFCC00\"上面写着'办四级神仙证书，回收二手莲花宝座'\".",
         "\\cdddddd华丽的装饰，就是有点旧。\n\\c00ff00装备: 防御+120.\n\\cFFCC00\"别人穿过的极品。\".",
         "\\cdddddd东海龙鳞编织而成，限量版，天上天下，只此一款。\n\\c00ff00装备: 防御+220.\n\\cFFCC00\"更轻薄，更透气，更多防护，更多安心\"."
      };
      this.itemUseCounts = new byte[]{-1, -1, -1, -1, 1, 1, 1, 3, 1, 1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1};
      this.yellowKeyCount = 0;
      this.blueKeyCount = 0;
      this.redKeyCount = 0;
      this.goldAmount = 0;
      this.itemStackTypes = new byte[32];
      this.itemStackUses = new byte[32];
      var10000 = new int[]{0, 1, 2, 4, 8, 16, 32};
      this.equipTierBonuses = new int[]{0, 10, 30, 70, 120, 220, 0, 10, 30, 70, 120, 220};
      this.equipTierTypes = new byte[]{0, 33, 34, 35, 79, 36, 0, 37, 38, 39, 80, 40};
      this.objectTypeNames = new String[]{
         "孙悟空",
         "黄门",
         "红门",
         "蓝门",
         "封印门",
         "门卫",
         "隐形路径",
         "上楼梯",
         "下楼梯",
         "炼丹炉",
         "云雾",
         "能挖的墙",
         "隐形墙",
         "火眼金睛",
         "生死簿",
         "筋斗云",
         "芭蕉扇",
         "金勺子",
         "玄明石",
         "千年月桂露",
         "相形变位",
         "上行符",
         "下行符",
         "太公杖",
         "捆仙绳",
         "幸运币",
         "黄钥匙",
         "红钥匙",
         "蓝钥匙",
         "威力丹",
         "金刚丹",
         "回春丸",
         "长寿丹",
         "桃木棒",
         "玄铁棒",
         "真银棒",
         "金箍棒",
         "道袍",
         "铁甲",
         "锁子甲",
         "紫金龙鳞甲",
         "天宫小犬",
         "天宫大犬",
         "伴炉童子",
         "护庭小神",
         "守园仙婢",
         "护庭校尉",
         "巡天卫士",
         "巡天卫士",
         "巨力士",
         "执火道人",
         "奎木狼",
         "执瓶仙侍",
         "金刚力士",
         "护持迦蓝",
         "侍案仙女",
         "护庭神将",
         "赤力韦驮",
         "护丹老道",
         "伏魔韦驮",
         "兜率宫星君",
         "无量护法",
         "兜率宫老仙",
         "二郎执旗将",
         "杀破狼",
         "二郎骁骑将",
         "尊天韦驮",
         "啸天犬",
         "太上老君",
         "杨戬",
         "玉皇大帝",
         "玉皇大帝",
         "牛魔王",
         "天蓬元帅",
         "巨灵神",
         "哪吒",
         "路点",
         "太白金星",
         "天宫商人",
         "乌金棍",
         "银缕锁甲",
         "封印门",
         "传怪点",
         "剧情点",
         "嫦娥",
         "黄金钥匙",
         "日月无极钹",
         "菩提老祖"
      };
      this.enemyBaseHp = new int[]{
         35,
         45,
         35,
         50,
         60,
         55,
         100,
         50,
         260,
         60,
         130,
         100,
         320,
         20,
         320,
         100,
         210,
         220,
         160,
         200,
         230,
         220,
         200,
         360,
         180,
         180,
         1200,
         4500,
         1500,
         8000,
         800,
         5000,
         120,
         444,
         100
      };
      this.enemyBaseAtk = new int[]{
         18,
         20,
         38,
         42,
         32,
         52,
         180,
         48,
         85,
         100,
         60,
         95,
         120,
         100,
         140,
         680,
         200,
         180,
         230,
         380,
         450,
         370,
         390,
         310,
         430,
         460,
         180,
         560,
         600,
         5000,
         500,
         1580,
         150,
         199,
         65
      };
      this.enemyBaseDef = new int[]{
         1, 2, 3, 6, 8, 12, 110, 22, 5, 8, 3, 30, 15, 68, 20, 50, 65, 30, 105, 130, 100, 110, 90, 20, 210, 360, 20, 310, 250, 1000, 100, 190, 50, 66, 15
      };
      this.enemyBaseGold = new int[]{
         1, 2, 3, 6, 5, 8, 100, 12, 18, 12, 8, 22, 30, 28, 30, 55, 45, 35, 65, 90, 100, 80, 50, 40, 120, 200, 100, 1000, 800, 500, 500, 500, 100, 144, 30
      };
      this.levelScriptLines = new String[]{
         "CES_84_6_11 MOV_0_5_11 TAK_8_9 CES_70_5_8 TAK_10_10 ROS_4_1 TAK_11_17 ROS_4_2 TAK_18_19 MOV_0_5_10 TAK_20_21 DES_70_5_8 LAY_2 ROS_1_4_7 ROS_2_0 ROS_2_6 RES_0 GUT_1 ",
         "TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8 ",
         "TAK_33_36 MOV_72_1_8_1_10 DES_72_1_10 ",
         "TAK_37_37 MOV_0_6_5 TAK_38_39 TAK_41_41 DES_44_1_3 DES_44_2_3 DES_44_3_3 DES_46_2_4 DES_44_9_3 DES_44_10_3 DES_44_11_3 DES_46_10_4 CES_44_5_4 CES_46_6_4 CES_44_7_4 CES_44_5_5 CES_44_7_5 CES_44_5_6 CES_46_6_6 CES_44_7_6 SWD TAK_42_42 ",
         "CES_72_1_11 TAK_43_44 MOV_0_6_3 MOV_72_1_11_6_2 ROS_4_1 TAK_45_48 MOV_72_6_2_6_1 DES_72_6_1 ",
         "TAK_49_52 MOV_72_9_1_7_1 DES_72_7_1 ",
         "TAK_53_58 ",
         "TAK_60_60 MOV_72_3_2_8_4 TAK_61_61 CES_47_8_3 CES_47_8_5 TAK_62_63 DES_72_8_4 DES_47_8_3 DES_47_8_5 ADD_2_72_11_10 ",
         "ROS_4_1 CES_73_10_1 MOV_73_10_1_6_9 TAK_68_74 MOV_73_6_9_6_10 ",
         "SWD ",
         "CES_72_3_10 MOV_72_3_10_2_10 MOV_72_2_10_4_9 TAK_76_78 DES_72_4_9 ",
         "SWD ",
         "TAK_84_87 MOV_58_5_4_6_8 MOV_58_4_4_6_8 MOV_58_3_4_6_8 MOV_57_7_4_6_8 MOV_57_8_4_6_8 MOV_57_9_4_6_8 MOV_56_4_2_6_8 MOV_56_3_2_6_8 MOV_56_2_2_6_8 MOV_59_8_2_6_8 MOV_59_9_2_6_8 MOV_59_10_2_6_8 TAK_88_90 MOV_73_6_2_6_8 ",
         "CES_6_10_2 CES_60_10_2 ",
         "ROS_4_1 TAK_92_97 DES_73_6_8 TAK_98_100 DES_70_6_7 ",
         "ROS_4_1 CES_61_5_2 CES_61_6_2 CES_61_7_2 CES_61_5_3 CES_70_6_3 CES_61_7_3 CES_61_5_4 CES_61_6_4 CES_61_7_4 TAK_118_121 ",
         "TAK_123_126 GUT_37 ",
         "TAK_68_74 ",
         "DES_1_4_4 CES_20_4_4 ",
         "CES_84_7_7 MOV_84_7_7_6_8 TAK_0_1 MOV_84_6_8_1_8 TAK_2_2 MOV_0_2_8 TAK_3_3 MOV_84_1_8_1_1 TAK_4_4 MOV_0_1_2 TAK_5_5 MOV_84_1_1_10_1 MOV_0_6_1 DES_84_10_1 TAK_6_7 MOV_0_11_1 MOV_0_1_11 ",
         "TAK_40_40 ",
         "TAK_59_59 ",
         "TAK_75_75 ",
         "TAK_84_87 GUT_12 ",
         "TAK_101_105 ",
         "CES_22_6_6 ",
         "TAK_129_132 DES_84_11_4 GIN_1_1000 ",
         "TAK_113_117 TAK_133_135 DES_84_6_11 GIN_0_13 ",
         "TAK_136_143 DES_84_1_11 TAK_144_144 GIN_0_19 ",
         "TAK_145_146 DES_84_9_8 GIN_1_1000 ",
         "MOV_84_6_3_4_3 MOV_84_4_3_8_3 MOV_84_8_3_6_3 TAK_107_107 ",
         "TAK_108_108 DES_10_6_6 MOV_0_6_5 TAK_109_112 TAK_147_148 DES_84_6_3 TAK_149_149 ",
         "TAK_79_79 ",
         "TAK_155_160 DES_72_11_10 ",
         "TAK_150_154 ",
         "TAK_161_161 ",
         "TAK_162_162 ",
         "GLV_1 ",
         "CES_6_4_1 CES_61_4_1 ",
         "",
         "TAK_255_259 TAK_165_165 SEE_3_10_166_166_1 ",
         "TAK_167_167 SEE_4_10_168_168_0 ROS_4_1 SEE_2_8_169_169_0 SEE_2_8_170_170_1 ",
         "TAK_171_171 SEE_7_10_172_172_0 SEE_7_10_173_173_1 ",
         "TAK_174_174 SEE_7_9_175_175_0 TAK_176_176 SEE_8_8_177_178_0 ",
         "TAK_179_180 ROS_4_3 SEE_8_6_181_182_0 ROS_4_1 SEE_10_4_183_183_0 ",
         "ROS_4_1 SEE_8_3_184_184_0 ROS_4_0 SEE_6_7_185_185_0 ",
         "SEE_4_4_186_186_0 ROS_4_1 SEE_6_2_187_187_1 ",
         "ROS_4_0 SEE_6_2_188_188_0 SEE_4_4_189_189_1 ",
         "TAK_190_190 ROS_4_1 SEE_2_2_191_192_0 ROS_4_0 SEE_2_6_193_196_0 SEE_2_6_197_198_1 ",
         "ROS_4_3 SEE_1_11_199_200_0 ",
         "ROS_4_2 SEE_6_11_201_202_0 ROS_4_3 SEE_3_8_203_204_0 CES_6_3_10 SEE_3_8_205_205_1 ",
         "TAK_206_209 ",
         "TAK_210_211 SEE_11_11_212_212_0 SEE_11_7_213_214_0 TAK_215_216 ",
         "TAK_217_218 ROS_5_510 ROS_6_510 CES_36_11_8 SEE_11_8_219_221_0 CES_40_11_9 SEE_11_9_222_222_0 SEE_8_9_224_225_0 SEE_10_9_223_223_0 DES_11_10_9 ",
         "TAK_224_224 SEE_6_2_225_225_0 ",
         "TAK_226_227 ROS_5_510 ROS_6_510 CES_36_5_1 CES_40_7_1 TAK_228_228 ",
         "TAK_229_231 MOV_0_6_7 TAK_232_232 DES_51_2_8 DES_51_1_8 DES_51_2_9 DES_51_1_9 CES_51_6_6 CES_51_5_7 CES_51_6_8 CES_51_7_7 GUT_57 ",
         "MOV_51_6_6_6_7 ROS_4_3 MOV_51_5_7_6_7 ROS_4_0 MOV_51_6_8_6_7 ROS_4_2 MOV_51_7_7_6_7 ROS_4_1 TAK_233_235 MOV_74_7_4_6_7 GUT_61 ",
         "",
         "",
         "",
         "TAK_236_238 MOV_75_5_4_6_7 TAK_239_239 GUT_62 ",
         "CES_69_5_5 CES_47_5_6 CES_47_5_7 CES_47_5_8 CES_56_7_6 CES_56_7_7 CES_56_7_8 TAK_240_245 GUT_63 ",
         "CES_77_6_6 TAK_246_250 DES_71_6_3 DES_77_6_6 DES_69_5_5 DES_-66_0 DES_-56_0 DES_-49_0 DES_-47_0 DES_-69_0 ",
         "TAK_253_254 ",
         "TAK_127_128 END_0 ",
         "TAK_261_263 SMS_0 ",
         "TAK_264_268 "
      };
      this.dialogueSpeakerType = new byte[]{
         0,
         84,
         0,
         84,
         0,
         84,
         0,
         0,
         84,
         0,
         70,
         0,
         70,
         0,
         70,
         0,
         70,
         0,
         84,
         0,
         0,
         70,
         72,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         72,
         75,
         0,
         75,
         75,
         75,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         74,
         0,
         74,
         0,
         74,
         0,
         74,
         72,
         72,
         47,
         72,
         0,
         72,
         72,
         0,
         73,
         0,
         73,
         0,
         73,
         0,
         73,
         73,
         72,
         0,
         72,
         72,
         0,
         72,
         72,
         0,
         73,
         0,
         73,
         0,
         73,
         0,
         73,
         0,
         70,
         73,
         70,
         73,
         70,
         73,
         73,
         0,
         70,
         69,
         0,
         69,
         0,
         69,
         84,
         0,
         0,
         84,
         0,
         84,
         0,
         84,
         0,
         84,
         0,
         84,
         70,
         0,
         70,
         0,
         70,
         72,
         0,
         72,
         0,
         0,
         -1,
         0,
         84,
         0,
         84,
         84,
         0,
         84,
         0,
         84,
         0,
         84,
         0,
         84,
         0,
         84,
         0,
         84,
         0,
         0,
         0,
         0,
         69,
         69,
         0,
         69,
         0,
         0,
         72,
         0,
         72,
         0,
         72,
         0,
         0,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         0,
         87,
         87,
         87,
         87,
         87,
         87,
         0,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         87,
         71,
         71,
         0,
         71,
         71,
         74,
         71,
         75,
         0,
         71,
         71,
         69,
         47,
         69,
         56,
         69,
         47,
         77,
         77,
         77,
         71,
         0,
         56,
         71,
         0,
         0,
         87,
         0,
         87,
         0,
         87,
         0,
         87,
         0,
         0,
         87,
         87,
         87,
         87,
         87
      };
      this.f_byte_arr_22 = new byte[]{
         7,
         5,
         -1,
         3,
         -1,
         5,
         -1,
         1,
         10,
         0,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         1,
         9,
         7,
         8,
         -1,
         -1,
         5,
         11,
         2,
         6,
         -1,
         -1,
         -1,
         -1,
         3,
         11,
         0,
         8,
         3,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         2,
         -1,
         11,
         -1,
         5,
         0,
         5,
         9,
         2,
         10,
         3,
         -1,
         -1,
         11,
         -1,
         -1,
         0,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         10,
         -1,
         -1,
         9,
         0,
         1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         3,
         11,
         -1,
         0,
         3,
         -1,
         -1,
         -1,
         -1,
         -1,
         5,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         5,
         -1,
         5,
         -1,
         1,
         3,
         -1,
         8,
         9,
         8,
         9,
         -1,
         -1,
         -1,
         8,
         4,
         5,
         6,
         10,
         8,
         -1,
         -1,
         5,
         9,
         1,
         -1,
         0,
         3,
         7,
         -1,
         0,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         3,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         3,
         -1,
         -1,
         0,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         7,
         -1,
         7,
         -1,
         7,
         2,
         2,
         -1,
         2,
         -1,
         -1,
         -1,
         7,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         9,
         3,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1,
         -1
      };
      this.dialogueTexts = new String[]{
         "姑娘，您……",
         "哎呀，姑奶奶我好不容易怀揣2个仙桃夹带出来，居然被发现了~！快闪！！",
         "……，跑都跑的这么优雅。",
         "哎呀，跑这么快……",
         "修逗妈待~！……",
         "死猴子，穷追猛打，追的人家小心肝噗嗵噗嗵地……",
         "啊~~~姑娘哟~~姑~~~娘~~！！",
         "死开死开~！！好狗不挡路。",
         "居然追到这里了，算了吧，认了吧。\n\\cFFCC00两个桃子滚落地上",
         "哎~~~~~~~",
         "我咤~~~~！天庭圣地，禁止喧哗。",
         "我~~~~",
         "我什么我~？？整天身居仙位，游手好闲……",
         "你~~~~~~~~~",
         "你什么你~？？看嫦娥衣杉不整，是不是你非礼~？？是不是你是不是你是不是你！？",
         "她~~~~~~~~~~",
         "她什么她~？？哦~！还滚出来2个仙桃！！假公济私是吧~！！猴子偷桃是吧！！",
         "俺没~！……",
         "……",
         "你……叫嫦娥？这个名字，俺，记。住。了……",
         "没错，桃子就是俺偷的！！俺认罚！",
         "将这只孽畜剥下铠甲打下天牢！",
         "贤弟，快醒醒……",
         "哎呦……老牛，醒来见你那张脸，真提神！！",
         "我宁愿看着你，睡得如此沉静，胜过你醒时决裂般无情~~",
         "靠，发春呐，把~手~拿~开~~！",
         "哟~~挺横得嘿~牛哥学得一手好摄影……刚才……",
         "……大哥久违了，多日不见，受小弟一拜！！",
         "话说你一直在梦中叫着一个人的名字，她一定偷了你很多桃子……",
         "没错，她是偷了我的东西，但是不是桃子，我很想再见她……",
         "恩恩，现在大哥带你从密道出去……",
         "……这你都能挖开~！！",
         "哥在魔界有个绰号，叫“溜得滑”，没有什么囚牢能困住哥……",
         "大哥！！能不能分我件东西遮遮羞……",
         "有，我这有把刚挖地道的破勺，你拿去挡住先！",
         "靠……算了，总比没有好。……",
         "没办法，你就将就一下啦。\\cFFCC00有件道童的旧袍子在9楼，你仔细找找吧，\\cF8F8F8我先撤了……",
         "妖猴，你可认得本少爷！！",
         "咝~~我看你骨骼清奇三头六臂，猜得不错的话，阁下一定是畸形儿！",
         "啊~！大哥果然高手！这都被你看穿了！不能留你活口了！",
         "哎呀呀~~我回去告诉我爹去！",
         "来人！把妖猴拿下！",
         "哦~~~喳~！李小猴踢馆！！",
         "贤弟，慢着！",
         "溜得滑？",
         "我偷偷告诉你哦，有个银棍在17层，你找到它以后就能厉害很多？",
         "了解！",
         "红了红脸",
         "尽快去吧，我闪先……",
         "贤弟，这只啸天犬在这里挡道太危险了，跟哥哥绕密道吧。",
         "一只看门的宠物，能凶到哪里？",
         "不是，自从调到天庭守路，已经很多天没有吃荤腥了……",
         "了解，绕道，走~！",
         "我乃托塔天王麾下先锋官，巨玲神是也！你欺我孩儿……",
         "欺你孩儿~？",
         "其实……哪吒是奴家跟托塔天王的私生子……",
         "你是女将！？",
         "其实，看我像芙蓉姐姐一样的好身段，你就应该了解……",
         "……咝~~托塔天王口味这么偏……",
         "啊~~~天王，巨玲儿不能陪伴你了……",
         "看来你已经打通暗墙了，看哥的。",
         "噢，通完收工，走！！……",
         "我们是天庭市容管理队！！啊哈~乱堆渣土，违规施工终于被抓个现行！",
         "哎呀~！兄弟，一定要来2层天牢救我呀！",
         "",
         "",
         "",
         "",
         "你就是孙悟空！？",
         "俺就是。",
         "我常去广寒宫看她，她这几天常常提到你……我从来不知道什么叫做嫉妒，但是这次，我想要你的命。",
         "哼，你哪位？",
         "我乃天蓬元帅，朱刚鬣！！掌管天河……",
         "猪肛裂？？好，满足你这个愿望……",
         "哎……我还没报完呢……",
         "菊花残~~~满地伤……",
         "你跟杨戬一战，天庭都开了盘口，赔率是1:5，哥把私房钱都押到你头上了，表辜负哥哥哟~",
         "……真想背后给你一闷棍……",
         "天庭禁止乱堆渣土，哥哥把渣土运到“天庭城管办事处”去，就不算乱堆了，捏嘿嘿~~",
         "\\cFFCC0023层乃是29层的地基所在，找出暗藏的墙，就可以让29层的墙松动，哥哥就可以挖穿它。",
         "……你个假仗义！",
         "兄弟，赶快去凑齐装备，打败杨戬，",
         "哥哥就发达了~嘿嘿~",
         "……",
         "上次趁本元帅自报大名的时候，突袭本帅，本帅不跟你计较，单挑还是群殴，你自己选。",
         "恩，是条汉子，俺就认真跟你打一次，单挑！",
         "单挑是吧，你一个单挑我们全部，弟兄们，一起上！",
         "你丫不地道！",
         "你的确是个英雄，难怪她一直念念不忘……",
         "过奖过奖，你的部下都躺下了，现在轮到你了……",
         "恩~~讨厌死了，来了来了……",
         "人家今天身体不方便，改天再来，先闪了",
         "天蓬，你数次战悟空不胜倒罢了，平时常常擅自离岗，去广寒宫把妞……死罪可免，活罪难饶。",
         "听说，下凡投胎，就会堕入轮回，就会忘记前尘往事……",
         "好，朕就成全你，下凡之前，有什么要求么？",
         "天天大吃大喝，倒头睡觉，生活安逸无边，心宽体胖……",
         "很好，你的心意，朕明白了，安心去吧",
         "哇！！投胎为猪？？！",
         "天蓬元帅变成了一只猪，被贬下了凡尘",
         "……真阴险……",
         "哼，哼，寡人在四十九层等你，哇哈哈哈哈~！！",
         "本来，战神情圣的名号是我的；嫦娥的心，迟早也会归属于我，但是你来了之后，一切都改变了……",
         "你喜欢她，这么多年，你为什么不去找她？",
         "因为我是战神情圣，是不能失败的……",
         "你太骄傲了……",
         "无论如何，斗神和战神这一战，是注定的……",
         " ",
         "哇，姑娘身陷三昧真火当中，要想办法开门灭火……",
         "姑娘顶住，俺老孙来救你！！",
         "不要不要过来！！",
         "姑娘你没事吧？",
         "死猴子，月宫阴冷，姑奶奶我想蒸蒸桑拿，治多年的关节炎都不行……",
         "……",
         "死猴子，上次桃子的事情……",
         "俺掌管蟠桃园，偷吃仙桃何止千百，多认2个，算什么……",
         "害你被革除了“齐天大圣”的上仙之位……",
         "俺老孙不稀罕天宫的位子，~贬下凡尘仍称王，嘿嘿",
         "……在天宫几千年，从来没有人肯为我放弃仙位……唉，可惜。",
         "哼哼，居然打到这里，实话告诉你，所有天神都对你不满，这次你被削去仙爵打入天牢，都是计划之中。",
         "那嫦娥呢，桃子呢？也在计划之中？你们料定俺会甘心顶罪？",
         "哇哈哈哈哈~！天网易逃，情丝难断，你有通天的本事，也难过这一关。",
         "为她顶罪，俺从不后悔，现在，是俺了断恩怨的时候了！！",
         "其实，朕不是打不过你，朕只不过私挪了国库，买了你的盘口……",
         "你终于打到这里了。",
         "你居然在这里？",
         "哈哈哈哈，老夫一路保你，就是为了让你帮我扫清天庭，你的所做所为…",
         "俺最恨的就是被人欺骗！我……（悟空久久地陷入了回忆）",
         "天庭，天庭又怎样？女人骗我，兄弟骗我，如今俺老孙没有什么可以留恋的，回花果山罢了。",
         "孽畜，天庭威仪，岂能容你全身而退！！",
         "姑娘，你怎么会被关在这里！",
         "奴家暗中助你，触犯天条……",
         "玉帝老儿，待我打烂你的金冠！！姑娘你先离开，等俺回来！",
         "唉……你又何苦……",
         "这里有瓶火眼金睛牌眼影霜，去皱抗衰老，可以看清楚敌人的本质，金色质感贴合肤质，来自巴黎，你值得拥有。",
         "现在涂好了，看起来嗲不嗲~？",
         "恩……本来是只“猴妖”，现在是个“人妖”。",
         "嫦娥姑娘，想不到在这里遇到你。",
         "大圣，这是我亲手酿制的千年月桂露，喝下它，犹如脱胎换骨，体力大增。",
         "哦~？难道这是定情信物？",
         "而且，它还可以使人忘记红尘感情，我希望你能忘记我。",
         "啊~哈~给我一杯忘情水~换我一夜不流泪……姑娘，俺准备离开天庭，我希望你跟我一起走……",
         "违背天条，私奔，会被整个天界人肉搜索的……",
         "私奔？俺老孙不做那猥琐之事，待俺打上灵霄宝殿，让玉皇大帝亲口答应，整个天庭谁敢为难你！！",
         "大圣保重，此地奴家不宜久留，奴家不想连累你……",
         "姑娘！姑娘！",
         "大圣，前面凶险难测，奴家这里有点私房钱，送给你买点仙丹滋补身体吧……",
         "……姑娘对我一片真情，俺发誓要为你打下一片天",
         "跟俺走吧，回花果山去……",
         "表，姑奶奶我为了天宫护照，抛弃了前夫，我才表再跟你下凡，你……是个好人……(飘走)",
         "......女人如衣服，兄弟如手足，老牛~！俺来寻你！！",
         "宝扇宝扇告诉我，谁是这个世界上最型最猛的男人？",
         "（模仿扇子的声音）是你~是你~还是你",
         "真自恋……",
         "哇~！被你偷窥到了，本尊该杀你灭口，但是现在你还不配本尊出手。",
         "可恶，等俺老孙先找回俺那根如意棍子再来收拾你……\\cFFCC00先去2层天牢救老牛，让他替俺开暗墙绕过去",
         "老牛，俺救你来了~！",
         "平时让你帮忙，老是推三阻四，这次这么爽快，一定有问题~",
         "嘿嘿，35层有个三眼小白脸太恶心，替俺灭了他~~",
         "俺对小白脸木有兴趣……",
         "那就想办法帮俺绕过去~~",
         "嘿嘿，开自己的洞，让别人说去吧~~~",
         "?有根捆仙绳？似乎可以克制住那个三眼小白脸，恩，搞定他，捆绑他，拿他的芭蕉扇，哦也~",
         "有芭蕉扇可以灭火了，嫦娥姑娘，俺来啦~~！！对了，还有我的如意金箍棒。",
         "欢迎你来到天宫世界，我是你的师傅菩提老祖。",
         "在这里我不会教你七十二变，但是我会教你怎么游历天宫。",
         "为师知道你要大闹天宫，特意千里传音，提供远程视频支持，当然，如果你嫌为师罗嗦，也可以在游戏菜单中选择跳过教程。",
         "好了，现在请试着\\cFFCC00按方向键移动到这里。",
         "很好，你已经学会太空步了。",
         "在你面前有一道黄色的门，你无法过去。",
         "你可以看到这里有把黄钥匙，它可以开启这道门。",
         "现在\\cFFCC00移动到这里，再回来开门。",
         "等等！",
         "前面有只挡路的狗。你需要打败它才能走过去。",
         "现在，\\cFFCC00请试着移动到它的位置上，与它战斗吧。",
         "听到轰隆声了吧，因为你打败了\\cFFCC00守卫封印门的敌人。",
         "所以这里的\\cFFCC00封印门\\cF8F8F8就被打开了。",
         "在战斗中你可能会损失血量。",
         "这里有个\\cFFCC00小仙桃，可以回复你的血量。",
         "如果血量不足，你将无法挑战敌人。",
         "又到了学习时间。",
         "你的能力是可以提升的，包括攻击、防御、血量。",
         "这里有个蓝色仙丹，学名是“防御仙丹”，服下它，可以提升你的防御力，让你战斗更持久。",
         "记住，\\cFFCC00天庭层数越高，仙丹药效越大。",
         "现在，吃了它，扑过去做掉前面那条狗，你会发现损血少了。",
         "看到上面的蓝门了吗，它只能用蓝色的钥匙打开。",
         "它被藏在这里，\\cFFCC00先拿到它吧。",
         "你发现了一道红门。这种门很少见，必须用红钥匙才能打开。",
         "它被藏在这里，\\cFFCC00请先得到它，再回来开门。",
         "你找到了一把红钥匙，这种钥匙比较稀少。",
         "试着\\cFFCC00用它开启这里的红门。",
         "很好，这层已经接近尽头。",
         "你会发现这样的红色传送点，它可以让你向上一层楼。",
         "不过，别急着离开。",
         "你是不是已经发现这里有个道具了吗？",
         "这里有个红色仙丹，学名是“攻击仙丹”，服下它，可以提升你的攻击力，让你战斗更狂野。",
         "但是这里好象不通……",
         "别急！俗话说车到山前必有路，在天宫的很多层中会有隐藏的路，更多惊喜更多欢笑，就在隐藏路……",
         "现在，移动到这里，你就会发现它。",
         "要记住，\\cFFCC00很多层里都会有隐藏的东西，试着去探索吧。",
         "看到你上来的路了吗？",
         "蓝色的传送点可以让你向下一层楼。",
         "你可能无法击败这个敌人，绕道也是前进的办法。",
         "那么，如何判断一个敌人的强弱呢？",
         "游戏中你会获得这件宝物，它叫\\cFFCC00火眼金睛牌眼影膏。",
         "涂抹一点在眼皮上，你可以看破敌情，还可以去除眼角纹。",
         "我帮你开出了一条路，你可以去取它了。",
         "现在你可以参照使用说明来使用它了。",
         "除了按5/OK键查看敌人对你造成的伤害以外。",
         "你还可以按左软键打开物品栏。",
         "选择该物品，按确认键查看更详细的敌人信息。",
         "你站在这十字街头上，找不到来去的方向。",
         "不要慌张，试着查看下这里道具和敌人的分布形势。",
         "上去的传送点在这里。",
         "如果你无法马上击败这个守卫。",
         "就试着将地图上的仙丹和仙桃吃掉，然后你就可以战胜它了。",
         "记住，如果前方有一群敌人在向你挥手，千万别冲动。",
         "牢记\\cFFCC00“先吃仙丹后肉搏”\\cF8F8F8是减少损血的第一法则。",
         "哦嘿嘿~为师要继续享受渡假啦~。",
         "在这之前我会传授你五百年功力，再送你两件东西防身。",
         "这是一把武器，能让你提升很高的攻击。",
         "在游戏的\\cFFCC00每10层都有一把新武器。",
         "如果你能早点获得它，就能轻松应对敌人，走得更远。",
         "同样，这是一件防具，能提高你的防御。",
         "我现在帮你打开这道墙，在一般情况下，它是无法击碎的。",
         "对了，给你介绍一个天宫上的朋友。",
         "这是为师的老朋友，\\cFFCC00太白金星\\cF8F8F8，他会暗中帮助你的。",
         "恭喜，你已经毕业了，我再传授你五百年的功力。",
         "还给你准备了两件神器，把它们收下吧。",
         "要记住，往前你将面对的不是一个敌人，而是整个天宫。",
         "捏哈哈~今天风和日丽，朕心情很好~！",
         "呀~！哪里钻出一只果子狸！预防非典！左右与我拿下！",
         "什么破皇帝，五谷不分，六畜不辨……玉帝老儿，俺乃花果山第一届型秀冠军，美猴王孙悟空！！",
         "都给我上~！",
         "巨玲神，愣在那里做什么？",
         "陛下，人家是女孩子嘛，最怕毛茸茸地小动物了……",
         "少废话！想被砍头啊！",
         "陛下，三太子请战！",
         "玉帝老儿，俺老孙要做齐天大圣！！",
         "嗷~~它过来了它过来了，请你个头啊，赶紧顶上先~~",
         "护驾~！护驾~！",
         "谁~！是天庭第一型男~！！",
         "是你~！是你~！",
         "谁~！是天庭第一猛男~！！",
         "是你~！是你~！",
         "谁~！是天庭第一明星战神~！！",
         "你是电，你是光，你是唯一地神话，你主宰，我崇拜~没有更好的办法~~！",
         "陛下……呕……",
         "臣觉得吧……呕……",
         "还是不要让二郎神继续下去了，尽快平息这次事件，给孙悟空封个官算了！！",
         "呕~~朕也是这么想滴~~喔莱，就封孙悟空为齐天大圣，掌管蟠桃园！",
         "嘿嘿，俺老孙就领了~！多谢~！",
         "妖…妖怪，他的眼睛闪着红光……太可怕了",
         "居然退缩，你这没用的东西。",
         "噢~~~~~~满园仙桃成熟，煞是诱人！！",
         "让俺老孙四处逛逛~~",
         "悟空~悟空~！",
         "为什么有只酱油蛤蟆跟我千里传音？",
         "是师父我啊！",
         "师父？为什么化个蛤蟆妆？",
         "为师在印度，法术交流兼渡假，日光浴加顿顿咖喱饭，嘴巴上火。",
         "如来把孙悟空封印于五指山下，伴随着所有恩怨情仇，欲知后事如何，敬请期待《西游记二之大圣取经》",
         "爱徒啊，你一个人要继续挑战天宫，为师不放心，给你买了份保险。",
         "师傅够义气。",
         "（接过保单）受益人...“菩提老祖”",
         "我的徒儿啊，你已经陷入混沌世界。也就人们常说的精神分裂症。",
         "这是由于种种感情纠葛引发的，如果你想出来，就要战胜他们。",
         "不，是战胜自己。要让这天…再也遮不住你的眼。",
         "我要提醒你的是，这个世界里，所有的敌人都会比原来更强，当然你的能力也会提升更多。",
         "去吧……劫难在所难免。"
      };
      this.f_int_113 = this.levelScriptLines.length;
      this.f_bool_arr_06 = new boolean[this.f_int_113];
      this.f_int_118 = 0;
      this.bossEventSpawns = new byte[][]{
         {
               29,
               1,
               3,
               0,
               29,
               2,
               3,
               0,
               29,
               3,
               3,
               0,
               26,
               1,
               4,
               0,
               26,
               2,
               4,
               0,
               26,
               3,
               4,
               0,
               30,
               9,
               3,
               0,
               30,
               10,
               3,
               0,
               30,
               11,
               3,
               0,
               32,
               9,
               4,
               0,
               32,
               10,
               4,
               0,
               32,
               11,
               4,
               0
         },
         {
               29,
               4,
               4,
               0,
               29,
               4,
               5,
               0,
               29,
               5,
               4,
               0,
               26,
               7,
               4,
               0,
               26,
               8,
               4,
               0,
               26,
               8,
               5,
               0,
               30,
               4,
               7,
               0,
               30,
               4,
               8,
               0,
               30,
               5,
               8,
               0,
               32,
               8,
               8,
               0,
               32,
               7,
               8,
               0,
               32,
               8,
               7,
               0
         },
         {27, 6, 5, 0, 27, 5, 6, 0, 27, 7, 6, 0, 27, 6, 7, 0},
         {17, 6, 4, 0},
         {16, 5, 5, 0},
         {24, 6, 2, 0, 32, 5, 1, 0, 32, 6, 1, 0, 32, 7, 1, 0, 27, 6, 6, 0},
         {
               32,
               2,
               3,
               0,
               32,
               3,
               3,
               0,
               32,
               4,
               3,
               0,
               26,
               8,
               3,
               0,
               26,
               9,
               3,
               0,
               26,
               10,
               3,
               0,
               29,
               3,
               5,
               0,
               29,
               4,
               5,
               0,
               29,
               5,
               5,
               0,
               30,
               7,
               5,
               0,
               30,
               8,
               5,
               0,
               30,
               9,
               5,
               0
         },
         {32, 4, 4, 0, 32, 5, 4, 0, 32, 6, 4, 0}
      };
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
      String[] var5 = new String[]{"界面", "菜单", "地图", "背景", "人物", "组件", "表情", "效果", "脚本", "缓存", "楼层", "角色", "设定", "敌人", "引子", "结局", "载入进度"};
      this.f_int_143 = 0;
      this.f_int_144 = 0;
      this.f_int_145 = 0;
      var10000 = new int[]{15658734, 13421772, 11184810, 8947848, 6710886, 4473924, 2236962, 1118481};
      this.f_int_arr_32 = new int[]{16711680, 16748544, 16776194, 1244928, 65478, 26367, 14156031};
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
      this.f_byte_28 = -1;
      this.f_int_arr_36 = new int[]{4202520, -1};
      var10000 = new int[]{2555941, 6436695};
      var10000 = new int[]{2555941, 10126750};
      var10000 = new int[]{12342908, 15039118};
      var10000 = new int[]{2236962, 16768893};
      this.f_byte_arr_45 = new byte[]{87, 18, 9};
      this.f_int_arr_37 = new int[3];
      this.f_byte_arr_46 = new byte[]{
         0, 5, 5, 5, 10, 4, 14, 7, 21, 4, 25, 3, 28, 3, 31, 4, 35, 3, 38, 4, 42, 4, 46, 4, 50, 4, 54, 3, 57, 3, 60, 4, 64, 2, 66, 2, 68, 4, 72, 4, 76, 5
      };
      this.f_int_arr2_03 = new int[26][7];
      this.f_String_arr_16 = new String[]{
         "手机号",
         "密码",
         "试玩",
         "注册",
         "帮助",
         "注册帐号",
         "注册密码",
         "注册手机号必须为本机，验证成功后将获得200游戏币",
         "欢迎登陆雪鲤鱼平台",
         "确定",
         "取消",
         "退出",
         "联网中",
         "正在重试",
         "联网超时",
         "服务器没有响应",
         "登陆",
         "剩余点数",
         "查询记录",
         "充值",
         "个人信息",
         "帐号或密码输入有误",
         "请输入11位手机号",
         "请输入6-10位密码",
         "日期",
         "金额",
         "本月付费记录",
         "进入游戏",
         "请确认您填写的是当前手机号，否则会导致注册失败。资费2元，需要发送1条短信，2元/条，不含通信费。是否注册？",
         "已发送成功，请等待系统验证后才能登陆",
         "发送失败，请重试",
         "类型",
         "序列号",
         "密码",
         "金额",
         "元",
         "已提交充值信息，请在充值记录中查看充值结果。",
         "返回",
         "状态",
         "最近充值记录",
         "提交过程出现错误，请检查充值内容。",
         "：",
         "\n",
         "是否确定退出游戏？",
         "您尚未登陆，请输入您的手机号和密码联网付费",
         "序列号和密码填写有误",
         "短信充值",
         "已发送成功。",
         "服务器响应错误！",
         "使用本产品必须先登录，与雪鲤鱼其他产品中注册的手机号，密码，帐号中的游戏币通用。如果没有帐号可以注册，送游戏币。登录后付费信息更安全，避免掉存档。",
         "充值的游戏币在所有雪鲤鱼相关产品中通用。",
         "请输入11位手机号",
         "请输入6位以上密码",
         "是否马上注册？需要确认您填写的是当前手机号，否则会导致注册失败。资费2元，需要发送1条短信，2元/条，不含通讯费。",
         "注册失败！如果该手机号是您的真实号码。将会把您的密码改为当前注册填写的密码，该过程可能需要一段时间。",
         "已注册成功。待验证成功后就能用该号码充值！并购买游戏中的付费内容。如果再次注册还可以发送短信修改密码。",
         "您尚未验证，如果之前已发送短信验证，请稍候再登陆重试。",
         "客服电话：400 630 5518",
         "您的手机无法连接到服务器，不能进行游戏。是否重新尝试联网？"
      };
      this.f_String_arr_17 = new String[]{"http://218.202.228.126:8880/nolander/server.php", "http://chatsrv0.ttutt.cn/dntk/server.php"};
      this.f_int_159 = 2048;
      this.f_byte_arr_47 = new byte[this.f_int_159];
      this.f_bool_31 = false;
      this.f_HttpConnection_00 = null;
      this.f_bool_33 = true;
      this.setFullScreenMode(true);
      a var1 = this;
      if (this.gameRandom == null) {
         var1.gameRandom = new Random();
         var1.gameRandom.setSeed(System.currentTimeMillis());
      }

      this.f_bool_00 = true;
      this.gameMode = 0;
      this.m_000();
   }

   protected final void paint(Graphics var1) {
      this.f_DirectGraphics_00 = DirectUtils.getDirectGraphics(var1);
      this.f_Graphics_00 = var1;
      this.f_Graphics_00.setFont(this.f_Font_00);
      switch (this.gameMode) {
         case 0:
            if (this.f_int_04 < 2) {
               this.f_Graphics_00.setColor(-1);
               this.f_Graphics_00.fillRect(0, 0, 240, 320);
               if (this.f_Image_00 != null) {
                  this.f_Graphics_00.drawImage(this.f_Image_00, 240 - this.f_Image_00.getWidth() >> 1, 320 - this.f_Image_00.getHeight() >> 1, 0);
               }
            } else {
               this.m_144();
            }
            break;
         case 1:
            a var34 = this;
            int var83 = 0;
            if ((var83 = 320 - var34.f_int_16 - var34.f_Image_arr2_00[10][0].getHeight()) > 0) {
               var34.f_Graphics_00.setColor(6662375);
               var34.f_Graphics_00.fillRect(0, 0, 240, var83);
            }

            var34.f_Graphics_00.drawImage(var34.f_Image_arr2_00[10][0], 0, var83, 0);
            var34.f_Graphics_00.setColor(3156024);
            var34.f_Graphics_00.fillRect(0, 320 - var34.f_int_16, 240, var34.f_int_16);
            var34.m_013(
               var34.f_Image_arr2_00[10][1], 86, 320 - var34.f_int_16 + (var34.f_int_16 - 17 >> 1), 0, 17 * var34.f_byte_arr_00[var34.f_int_10], 60, 17, 17
            );
            var83 = 320 - var34.f_int_16 + (var34.f_int_16 - 13 >> 1);
            var34.m_004(var34.f_Image_arr2_00[8][14], 60 - var34.f_int_46, var83, 1);
            var34.f_Graphics_00.drawImage(var34.f_Image_arr2_00[8][14], 160 + var34.f_int_46, var83, 0);
            this.f_Graphics_00.setClip(0, this.f_int_16, 240, 320 - (this.f_int_16 >> 1));
            this.m_130();
            this.f_Graphics_00.setClip(0, 0, 240, 320);
            break;
         case 2:
            a var33 = this;
            this.f_Graphics_00.setColor(3156024);
            var33.f_Graphics_00.fillRect(0, 0, 240, 320);
            var33.f_int_147 = 148 * var33.f_int_143 / 100;
            var33.m_002(var33.f_Image_arr2_00[15][0], 108, 86, 0, 0, 24, var33.f_int_147);
            var33.m_002(var33.f_Image_arr2_00[15][1], 108, 86 + var33.f_int_147, 0, var33.f_int_147, 24, 148 - var33.f_int_147);
            break;
         case 3:
            this.m_056(true);
            this.m_063(0, 20);
            this.m_053(this.f_int_56, this.f_int_57 + 20, true);
            int var194 = 240 - (this.mapCellsWide + 1 << 2);
            byte var115 = 20;
            int var82 = var194;
            a var31 = this;
            int var129 = 0;
            int var147 = 0;
            if (var31.f_bool_arr_05[1] && var31.f_Image_03 != null) {
               var31.f_Graphics_00.drawImage(var31.f_Image_03, var82, 20, 0);
               var129 = var82 + (var31.playerCellX << 2);
               var147 = 20 + (var31.playerCellY << 2);
               var31.f_Graphics_00.setColor(1112072);
               var31.f_Graphics_00.fillRect(var129, var147, 4, 4);
            }

            this.m_037(0, this.f_int_48);
            this.m_035(0, 0);
            a var32 = this;
            if (this.f_bool_13) {
               if (var32.f_int_70 >= 0) {
                  var32.m_002(var32.f_Image_arr2_00[8][19], 104, 20, 0, 0, 16, 16);
                  var32.m_002(var32.f_Image_arr2_00[8][19], 120, 20 + var32.f_int_46, 16, 0, 15, 17);
               }

               if (var32.f_int_73 >= 0) {
                  var32.m_002(var32.f_Image_arr2_00[8][19], 104, var32.f_int_48 - 17, 0, 17, 16, 16);
                  var32.m_002(var32.f_Image_arr2_00[8][19], 120, var32.f_int_48 - 17 - var32.f_int_46, 16, 17, 15, 17);
               }
            }

            this.m_126();
            this.f_byte_13 = 1;
            this.f_byte_14 = 3;
            this.m_034();
            break;
         case 4:
         case 19:
            this.m_056(true);
            this.m_037(0, this.f_int_48);
            this.m_035(0, 0);
            int var113 = 160;
            byte var79 = 120;
            a var30 = this;
            int var128 = this.f_int_08 + 22;
            int var146 = var30.f_int_14 + 32;
            int var163 = 120 - (var30.f_int_08 + 22 >> 1);
            int var172 = 160 - (var30.f_int_14 + 32 >> 1);
            int var180 = var30.f_int_15 - var30.f_int_01 >> 1;
            byte var185 = 0;
            var30.m_040(var163, var172, var128, var146);
            var172 += 16 + var180;

            for (int var189 = var30.f_int_11; var189 < var30.f_int_12; var172 += var30.f_int_15) {
               var185 = var30.f_byte_arr_00[var189];
               if (var30.f_int_10 == var189) {
                  var30.f_Graphics_00.setColor(0);
                  var30.f_Graphics_00.fillRect(120 - (var30.f_int_08 >> 1), var172 - var180, var30.f_int_08, var30.f_int_15);
                  var30.f_Graphics_00.setColor(16377897);
               } else if (var185 != 11) {
                  var30.f_Graphics_00.setColor(7574946);
               } else {
                  var30.f_Graphics_00.setColor(16751103);
               }

               if (var185 > 11 && var185 < 17) {
                  String var10001 = var30.f_String_arr_02[var185];
                  var79 = 4;
                  int var192 = var172;
                  int var190 = 120;
                  int[] var187 = var30.f_int_arr_32;
                  String var164 = var10001;
                  a var81 = var30;
                  var113 = var164.length();
                  int var13 = var81.f_byte_25;
                  int var14 = var187.length;
                  char var15 = '\u0000';
                  var190 -= var81.f_Font_00.stringWidth(var164) >> 1;

                  for (int var16 = 0; var16 < var113; var16++) {
                     var15 = var164.charAt(var16);
                     var81.f_Graphics_00.setColor(var187[var13]);
                     var81.f_Graphics_00.drawChar(var15, var190, var192, 0);
                     var190 += var81.f_Font_00.charWidth(var15);
                     if (++var13 >= var14) {
                        var13 = 0;
                     }
                  }

                  if ((var81.f_int_03 & 3) == 0 && ++var81.f_byte_25 >= var14) {
                     var81.f_byte_25 = 0;
                  }
               } else {
                  var30.f_Graphics_00.drawString(var30.f_String_arr_02[var185], 120, var172, 17);
               }

               var189++;
            }

            var163 = 120 + (var128 >> 1) - 23;
            var172 = 160 + (var146 >> 1) - 38;
            if (var30.f_int_11 > 0) {
               var30.m_002(var30.f_Image_arr2_00[8][15], var163, var172, 0, 0, 7, 9);
            }

            if (var30.f_int_12 < var30.f_int_07) {
               var30.m_002(var30.f_Image_arr2_00[8][15], var163, var172 + 9, 7, 0, 7, 9);
            }

            this.m_034();
            break;
         case 5:
            this.m_056(false);
            a var29 = this;
            int var64 = 240 - var29.f_int_108 >> 1;
            int var105 = 320 - var29.f_int_107 >> 1;
            Image var126 = null;
            byte var144 = 0;
            int var161 = 0;
            int var170 = 0;
            int var177 = 0;
            if (var29.f_int_110 - var29.f_int_109 > 0) {
               var29.m_040(var64, var105, var29.f_int_108, var29.f_int_107);
               if (var29.f_int_109 > 0) {
                  var29.m_002(var29.f_Image_arr2_00[8][19], 112, var105 - 17 - (var29.f_int_03 & 1), 16, 0, 15, 17);
               }

               var105 += 16;
               var64 += 16;
               int var184 = var29.f_int_109;

               for (int var188 = var64; var184 < var29.f_int_110; var64 = var188) {
                  var144 = var29.f_byte_arr_20[var184];
                  var126 = var29.f_Image_arr_00[var29.f_byte_arr_20[var184]];
                  var161 = var29.f_int_arr_22[var184];
                  var29.f_Graphics_00.setColor(13097429);
                  var29.f_Graphics_00.drawString(var29.objectTypeNames[var144], var64, var105 + 2, 0);
                  var64 = (240 + var29.f_int_108 >> 1) - 75;
                  var177 = var29.f_int_01 - 19 >> 1;
                  var105 += var177;
                  if (var161 < 0) {
                     var29.f_Graphics_00.drawImage(var29.f_Image_arr2_00[8][17], var64 - 4, var105 + 2, 0);
                  } else {
                     var29.f_Graphics_00.drawImage(var29.f_Image_arr2_00[8][16], var64 - 12, var105 + 2, 0);
                     var64 += 15;
                     var29.m_041(var64 - 1, var105 + 6, 48, 12);
                     var29.m_042(var29.f_Image_arr2_00[8][2], var161, var64 + 45, var105 + 8);
                  }

                  var64 = var188;
                  var105 += var177 + 19 + 4;
                  var29.m_041(var64, var105, 32, 32);
                  if (var144 > 40) {
                     var170 = 32 - var29.f_byte_arr_07[var144] >> 1;
                     if ((var177 = var126.getHeight()) > 31) {
                        var177 = 31;
                     }

                     if (var170 < 0) {
                        var29.m_002(var126, var64, var105 + 31 - var177, -var170, 0, 32, var177);
                     } else {
                        var29.m_002(var126, var64 + var170, var105 + 31 - var177, 0, 0, var29.f_byte_arr_07[var144], var177);
                     }
                  } else {
                     var29.f_Graphics_00.setClip(var64, var105, 32, 32);
                     var29.f_Graphics_00.drawImage(var126, var64 + (32 - var126.getWidth() >> 1), var105 + (32 - var126.getHeight() >> 1), 0);
                     var29.f_Graphics_00.setClip(var64, var105, 240, 320);
                  }

                  var105 += 4;
                  var64 += 36;
                  var29.m_002(var29.f_Image_arr2_00[8][7], var64, var105, 10, 0, 10, 13);
                  var64 += 15;
                  var29.m_041(var64, var105, 48, 12);
                  var29.m_042(var29.f_Image_arr2_00[8][2], var29.enemyAtkScaled[var144 - 41], var64 + 40, var105 + 2);
                  var64 += 52;
                  var29.m_002(var29.f_Image_arr2_00[8][7], var64, var105, 20, 2, 10, 10);
                  var64 += 15;
                  var29.f_Graphics_00.setColor(512);
                  var29.m_041(var64, var105, 48, 12);
                  var29.m_042(var29.f_Image_arr2_00[8][2], var29.enemyDefScaled[var144 - 41], var64 + 40, var105 + 2);
                  var64 -= 82;
                  var105 += 12;
                  var29.m_002(var29.f_Image_arr2_00[8][7], var64, var105 + 5, 0, 2, 10, 10);
                  var64 += 15;
                  var29.m_041(var64, var105 + 5, 48, 12);
                  var29.m_042(var29.f_Image_arr2_00[8][2], var29.enemyHpScaled[var144 - 41], var64 + 40, var105 + 7);
                  var64 += 52;
                  var29.f_Graphics_00.drawImage(var29.f_Image_arr2_00[8][1], var64, var105 + 5, 0);
                  var64 += 15;
                  var29.m_041(var64, var105 + 5, 48, 12);
                  var64 += 40;
                  var29.m_042(var29.f_Image_arr2_00[8][2], var29.enemyBaseGold[var144 - 41], var64, var105 + 7);
                  var105 += 18;
                  if (++var184 < var29.f_int_110) {
                     var64 = var188 - 5;
                     var29.f_Graphics_00.setColor(6435);
                     var29.f_Graphics_00.drawLine(var64, var105, var64 + var29.f_int_108 - 22, var105);
                     var105++;
                     var29.f_Graphics_00.setColor(4803902);
                     var29.f_Graphics_00.drawLine(var64, var105, var64 + var29.f_int_108 - 22, var105);
                  }

                  var105 += 2;
               }

               var105 += 16;
               if (var29.f_int_110 < var29.f_int_105) {
                  var29.m_002(var29.f_Image_arr2_00[8][19], 112, var105 + (var29.f_int_03 & 1), 16, 17, 15, 17);
               }
            }

            this.m_034();
         case 6:
         case 23:
         case 24:
         case 25:
         case 26:
         case 27:
         case 28:
         case 29:
         case 30:
         case 31:
         case 32:
         case 33:
         case 34:
         case 35:
         case 36:
         case 37:
         case 38:
         case 39:
         case 40:
         case 41:
         case 42:
         case 43:
         case 44:
         case 45:
         case 46:
         case 47:
         case 48:
         case 49:
         case 50:
         case 51:
         case 52:
         case 53:
         case 54:
         case 55:
         case 56:
         case 57:
         case 58:
         case 59:
         case 60:
         case 61:
         case 62:
         case 63:
         case 64:
         case 65:
         case 66:
         case 67:
         case 68:
         case 69:
         case 70:
         case 71:
         case 72:
         case 73:
         case 74:
         case 75:
         case 76:
         case 77:
         case 78:
         case 79:
         case 80:
         case 81:
         case 82:
         case 83:
         case 84:
         case 85:
         case 86:
         case 87:
         case 88:
         case 89:
         case 90:
         case 91:
         case 92:
         case 93:
         case 94:
         case 95:
         case 96:
         case 97:
         case 98:
         default:
            break;
         case 7:
            this.m_056(false);
            this.m_113(true);
            this.m_034();
            break;
         case 8:
            this.m_056(false);
            this.m_113(false);
            this.m_034();
            break;
         case 9:
            this.m_056(true);
            this.m_063(0, 20);
            this.m_053(this.f_int_56, this.f_int_57 + 20, true);
            this.m_037(0, this.f_int_48);
            this.m_035(0, 0);
            this.m_034();
            a var28 = this;
            int var59 = 240 - var28.f_int_75 - 22 >> 1;
            int var100 = 320 - var28.f_int_74 >> 1;
            int var125 = (var28.f_int_00 << 1) + 16 + 48;
            var28.m_038(0, var59, var100, var28.f_int_75 + 22, var28.f_int_74);
            var59 = 240 - var125 >> 1;
            var100 += 21;
            var28.f_Graphics_00.setColor(7575203);
            var28.f_Graphics_00.drawString("花费", var59, var100, 0);
            var59 += (var28.f_int_00 << 1) + 4;
            var28.f_Graphics_00.drawImage(var28.f_Image_arr2_00[8][1], var59, var100 + (var28.f_int_01 - 10 >> 1), 0);
            var59 += 60;
            var28.m_042(var28.f_Image_arr2_00[8][2], var28.alchemyPrice, var59, var100 + 3 + (var28.f_int_01 - 10 >> 1));
            var100 += var28.f_int_01 + 5;
            var59 = 240 - var28.f_int_75 >> 1;
            var28.f_Graphics_00.setColor(549016);
            var28.f_Graphics_00.fillRect(var59, var100 + (var28.f_int_80 << 4), var28.f_int_75, 16);
            var28.f_Graphics_00.drawImage(var28.f_Image_arr2_00[8][14], var59 + 10, var100 + 2 + (var28.f_int_80 << 4), 0);
            var28.m_004(var28.f_Image_arr2_00[8][14], var59 + var28.f_int_75 - 30, var100 + 2 + (var28.f_int_80 << 4), 1);
            var28.m_002(var28.f_Image_arr2_00[8][7], 97, var100 + 4, 0, 2, 10, 10);
            var28.m_042(var28.f_Image_arr2_00[8][2], var28.f_int_77, 142, var100 + 6);
            var100 += 16;
            var28.m_002(var28.f_Image_arr2_00[8][7], 97, var100 + 4, 11, 0, 8, 13);
            var28.m_042(var28.f_Image_arr2_00[8][2], var28.f_int_78, 142, var100 + 6);
            var100 += 16;
            var28.m_002(var28.f_Image_arr2_00[8][7], 97, var100 + 4, 20, 2, 10, 11);
            var28.m_042(var28.f_Image_arr2_00[8][2], var28.f_int_79, 142, var100 + 6);
            break;
         case 10:
            short var141 = 320;
            short var122 = 240;
            int var93 = 0;
            int var53 = 0;
            a var26 = this;

            for (int var159 = 319; var159 >= 1; var159 -= 2) {
               var26.f_Graphics_00.drawLine(-1, -1 + (320 - var159), -1 + var159 - 1, 318);
               var26.f_Graphics_00.drawLine(239 - var159, -1, 239, -1 + var159);
            }

            a var27 = this;
            var53 = 240 - var27.f_int_103 >> 1;
            var93 = 320 - var27.f_int_102 >> 1;
            int var123 = 0;
            Object var141n = null;
            var27.m_040(var53, var93, var27.f_int_103, var27.f_int_102);
            var53 += 20;
            var93 += 16;
            var27.f_Graphics_00.setColor(13097429);
            if (var27.f_int_104 < var27.itemStackSize) {
               var27.f_Graphics_00.drawString(var27.objectTypeNames[var27.itemStackTypes[var27.f_int_104]], var53, var93 + 2, 0);
            }

            var93 += var27.f_int_01 + 4;
            var53 -= 9;
            var27.f_Graphics_00.setColor(6178);
            var27.f_Graphics_00.drawLine(var53, var93, var53 + var27.f_int_103 - 23, var93);
            var93++;
            var27.f_Graphics_00.setColor(3564144);
            var27.f_Graphics_00.drawLine(var53, var93, var53 + var27.f_int_103 - 23, var93);
            var93 += 10;
            var53 += 9;
            var123 = var53;
            int var160 = var27.f_int_98;
            int var169 = var27.f_int_97 * var27.f_int_98;

            while (var160 < var27.f_int_99) {
               for (int var8 = 0; var8 < var27.f_int_97; var169++) {
                  var27.m_041(var53, var93, 32, 32);
                  if (var169 == var27.f_int_104) {
                     var27.m_002(var27.f_Image_arr2_00[2][10], var53 - 4, var93 - 3, 38 * (var27.f_int_03 & 1), 0, 38, 38);
                  }

                  if (var169 < var27.itemStackSize) {
                     Image var143 = var27.f_Image_arr_00[var27.itemStackTypes[var169]];
                     var27.f_Graphics_00.drawImage(var143, var53 + (32 - var143.getWidth() >> 1), var93 + (32 - var143.getHeight() >> 1), 0);
                     if (var27.itemStackUses[var169] > 0) {
                        var27.m_042(var27.f_Image_arr2_00[8][2], var27.itemStackUses[var169], var53 + 32, var93 + 26);
                     }
                  }

                  var8++;
                  var53 += 40;
               }

               var160++;
               var53 = var123;
               var93 += 40;
            }

            var53 = (240 + var27.f_int_103 >> 1) - 22;
            var93 -= var27.f_int_102 - 32 - var27.f_int_01 - 4 >> 1;
            if (var27.f_int_98 > 0) {
               var27.m_002(var27.f_Image_arr2_00[8][15], var53, var93 - 15, 0, 0, 7, 9);
            }

            if (var27.f_int_99 < var27.f_int_101) {
               var27.m_002(var27.f_Image_arr2_00[8][15], var53, var93 + 5, 7, 0, 7, 9);
            }

            this.m_034();
            break;
         case 11:
            this.m_056(true);
            this.m_063(0, 20);
            this.m_053(this.f_int_56, this.f_int_57 + 20, true);
            this.m_126();
            this.m_035(0, 0);
            a var25 = this;
            int var48 = 0;
            int var91 = (var25.f_int_34 << 1) + 12;
            int var120 = 320 - var91;
            if (var25.f_String_02 != null && !var25.f_bool_05 && var25.f_byte_16 != 4) {
               Image var9 = null;
               var25.m_039(0, var120, 80, var91);
               int var11 = var91;
               int var9i = 160;
               int var158 = var120;
               var48 = (byte)80;
               a var10 = var25;
               int var12 = 80;
               var48 = var158;

               while (var12 < 214) {
                  var10.f_Graphics_00.setClip(var12, var48, 16, 16);
                  var10.f_Graphics_00.drawImage(var10.f_Image_arr2_00[8][0], var12 - 26, var48, 0);
                  var12 += 16;
               }

               var10.f_Graphics_00.setClip(0, 0, 240, 320);
               var10.m_005(var10.f_Image_arr2_00[8][0], 214, var48, 0, 0, 26, 16, 1);
               var48 += 16;
               var10.f_Graphics_00.setColor(2699825);
               var10.f_Graphics_00.fillRect(80, var48, 149, var11 - 16);

               while (var48 < var158 + var11) {
                  var10.f_Graphics_00.setClip(229, var48, 11, 16);
                  var10.f_Graphics_00.drawImage(var10.f_Image_arr2_00[8][0], 176, var48, 0);
                  var48 += 16;
               }

               var10.f_Graphics_00.setClip(0, 0, 240, 320);
               if (var25.f_byte_17 > 0) {
                  var9 = var25.f_Image_arr_00[var25.f_byte_17];
                  if (var25.f_byte_17 != 72) {
                     var91 = var9.getHeight();
                     var25.m_002(var9, 0 + (80 - var25.f_byte_arr_07[var25.f_byte_17] >> 1), 320 - var91, 0, 0, var25.f_byte_arr_07[var25.f_byte_17], var91);
                  } else {
                     var25.m_002(var9, 24, 275, 0, 0, 32, 45);
                  }
               } else if (var25.f_byte_17 == 0) {
                  var25.f_Graphics_00
                     .drawImage(
                        var25.f_Image_arr2_00[3][1], 40 - (var25.f_Image_arr2_00[3][1].getWidth() >> 1), 320 - var25.f_Image_arr2_00[3][1].getHeight(), 0
                     );
               } else {
                  var25.f_Graphics_00.setColor(-1);
                  var25.f_Graphics_00.drawString("?", 40, var120 + (var91 - var25.f_int_01 >> 1), 17);
               }

               var25.f_Graphics_00.setColor(10473684);
               var25.m_019(var25.f_String_02, 85, var120 + 12, 129, var91 - 12, false);
               if (var25.f_int_26 > 0) {
                  var25.m_002(var25.f_Image_arr2_00[8][15], 220, 320 - var25.f_int_34 - 10, 0, 0, 7, 9);
               }

               if (var25.f_int_27 < var25.f_int_28) {
                  var25.m_002(var25.f_Image_arr2_00[8][15], 220, 320 - var25.f_int_34 + 1, 7, 0, 7, 9);
               }

               byte var92;
               if ((var25.f_byte_16 == 6 || var25.f_byte_16 == 1) && (var92 = var25.f_byte_arr_22[var25.f_int_112]) >= 0) {
                  if (var25.f_int_119 >= 0) {
                     var48 = var25.entityPixelX[var25.f_int_119] + 16;
                     var120 = var25.entityPixelY[var25.f_int_119] - 30;
                     if (var25.f_byte_17 == 84) {
                        var120 -= 20;
                     }
                  } else {
                     var48 = var25.playerPixelX + 16;
                     var120 = var25.playerPixelY - 30;
                  }

                  var25.f_Graphics_00.drawImage(var25.f_Image_arr2_00[12][var92], var48 + var25.f_int_56, var120 + var25.f_int_57 + 20, 0);
               }

               if (var25.f_byte_16 == 6) {
                  var25.f_Graphics_00
                     .drawImage(
                        var25.f_Image_arr2_00[2][11],
                        var25.f_int_56 + 5 + (var25.f_int_114 << 5),
                        var25.f_int_57 + (var25.f_int_115 << 5) - 12 + var25.f_int_46,
                        0
                     );
               }
            } else {
               var25.m_037(0, var25.f_int_48);
            }
            break;
         case 12:
            this.m_056(false);
            a var24 = this;
            int var46 = 320 - var24.f_int_01 - 4;
            var24.m_142(var24.f_String_04, 120, var46 + 2, 17, var24.f_int_arr_36);
            var46 += var24.f_int_01 + 4 - 13 >> 1;
            if (var24.f_int_90 > 0) {
               var24.m_004(var24.f_Image_arr2_00[8][14], 60 - var24.f_int_46, var46, 1);
            }

            if (var24.f_int_90 < var24.f_int_89 - 1) {
               var24.f_Graphics_00.drawImage(var24.f_Image_arr2_00[8][14], 160 + var24.f_int_46, var46, 0);
            }

            this.m_034();
            break;
         case 13:
            this.m_056(false);
            break;
         case 14:
            a var23 = this;
            int var44 = 0;
            if (var23.f_Image_01 != null) {
               var23.f_int_17 = var23.f_Image_01.getHeight();
               var23.f_int_18 = 320 - var23.f_int_17 - (var23.f_int_01 + 4) * 3 >> 1;
               var23.f_Graphics_00.setColor(0);
               var23.f_Graphics_00.fillRect(0, 0, 240, var23.f_int_18);
               var44 = var23.f_int_18 + var23.f_Image_01.getHeight();
               var23.f_Graphics_00.fillRect(0, var44, 240, 320 - var44);
               var23.f_Graphics_00.drawImage(var23.f_Image_01, 120, var23.f_int_18, 17);
               if (!var23.f_bool_03) {
                  var23.f_Graphics_00.setColor(-1);
               } else {
                  var23.f_Graphics_00.setColor(var23.f_int_arr_02[var23.f_byte_04]);
               }

               if (var23.f_int_19 < 3 && var23.f_String_01 != null) {
                  var23.m_019(var23.f_String_01, 10, var23.f_int_18 + var23.f_int_17 + var23.f_int_01, 220, var23.f_int_34 << 1, false);
               }
            }

            if (var23.f_int_19 > 2) {
               var23.f_Graphics_00.setColor(-1);
               var23.f_Graphics_00.drawString("请按任意键", 120, 320 - var23.f_int_01 - 2, 17);
            } else {
               var23.f_Graphics_00.setColor(-1);
               var23.f_Graphics_00.drawString("跳过", 240 - var23.f_Font_00.stringWidth("跳过") - 5, 320 - var23.f_int_01 - 2, 0);
            }

            this.f_Graphics_00.setClip(0, 0, 240, 320);
            this.m_130();
            this.f_Graphics_00.setClip(0, 0, 240, 320);
            break;
         case 15:
            this.m_056(false);
            this.m_013(this.f_Image_arr2_00[10][1], 86, 10, 0, 51, 68, 17, 17);
            this.m_034();
            break;
         case 16:
            this.m_056(false);
            a var22 = this;
            int var40 = 240 - var22.f_int_84 - 22 >> 1;
            int var89 = 320 - var22.f_int_85 >> 1;
            String var118 = null;
            var22.m_013(var22.f_Image_arr2_00[10][1], 86, 10, 0, 34, 68, 17, 17);
            var22.m_040(var40, var89, var22.f_int_84 + 22, var22.f_int_85);
            var40 += 11;
            var89 += 16;
            int var140 = (var22.f_int_01 + 8) * var22.f_int_82;
            var22.f_Graphics_00.setColor(549016);
            var22.f_Graphics_00.fillRect(var40, var89 + var140, var22.f_int_84, var22.f_int_01 + 8);
            var22.f_Graphics_00.drawImage(var22.f_Image_arr2_00[8][14], var40 + 10, var89 + 2 + var140 + (var22.f_int_01 - 5 >> 1), 0);
            var22.m_004(var22.f_Image_arr2_00[8][14], var40 + var22.f_int_84 - 30, var89 + 2 + var140 + (var22.f_int_01 - 5 >> 1), 1);

            for (int var157 = 0; var157 < var22.f_int_83; var89 += var22.f_int_01 + 8) {
               var118 = var22.f_String_arr_05[var22.f_byte_arr_11[var157]];
               var40 = 240 - var22.f_Font_00.stringWidth(var118) - 16 >> 1;
               var22.f_Graphics_00.setColor(-1);
               var22.f_Graphics_00.drawString(var118, var40, var89 + 4, 0);
               var40 += 2 + var22.f_Font_00.stringWidth(var118);
               if (!var22.f_bool_arr_05[var157]) {
                  var22.m_002(var22.f_Image_arr2_00[8][11], var40, var89 + (var22.f_int_01 - 2 >> 1), 12, 0, 12, 10);
               } else {
                  var22.m_002(var22.f_Image_arr2_00[8][11], var40, var89 + (var22.f_int_01 - 2 >> 1), 0, 0, 12, 10);
               }

               var157++;
            }

            this.m_034();
            break;
         case 17:
            this.m_056(false);
            this.m_013(this.f_Image_arr2_00[10][1], 86, 10, 0, 68, 68, 17, 17);
            this.m_034();
            break;
         case 18:
            a var21 = this;
            this.f_Graphics_00.setColor(0);
            var21.f_byte_21 = 0;
            var21.f_short_arr_07 = new short[64];
            var21.f_short_arr_08 = new short[64];
            int var38 = 0;
            short var87 = 0;
            short var4 = 0;
            int var5 = var21.f_byte_20;

            while (--var5 >= 0) {
               var87 = var21.f_short_arr_05[var5];
               var4 = var21.f_short_arr_06[var5];
               var21.f_Graphics_00.fillRect(var87, var4, 4, 4);
               if (((var38 = var21.randomBelow(15)) & 1) != 0) {
                  var21.m_124(var87, var4 - 4);
               }

               if ((var38 & 2) != 0) {
                  var21.m_124(var87, var4 + 4);
               }

               if ((var38 & 4) != 0) {
                  var21.m_124(var87 - 4, var4);
               }

               if ((var38 & 8) != 0) {
                  var21.m_124(var87 + 4, var4);
               }
            }

            var21.f_byte_20 = var21.f_byte_21;
            var21.f_short_arr_05 = var21.f_short_arr_07;
            var21.f_short_arr_06 = var21.f_short_arr_08;
            break;
         case 20:
            a var20 = this;
            int var37 = 320 - var20.f_Image_arr2_00[14][0].getHeight() >> 1;
            var20.f_int_23 += 2;
            if (!var20.f_bool_04 && (var20.f_int_03 & 1) != 0 && ++var20.f_int_22 > var20.f_Image_arr2_00[14][0].getWidth() - 240) {
               var20.f_bool_04 = true;
            }

            if ((var20.f_int_03 & 3) >> 1 != 0 && var20.f_int_33 < var20.dialogueTexts[260].length()) {
               var20.f_int_33++;
               var20.m_018(var20.dialogueTexts[260], 209, (var20.f_int_34 << 1) + 12, var20.f_int_33);
               var20.f_int_26 = var20.f_int_28 - 1;
               if (var20.f_int_26 < 0) {
                  var20.f_int_26 = 0;
               }

               var20.f_int_27 = var20.f_int_28;
            }

            var20.f_Graphics_00.setColor(3156024);
            var20.f_Graphics_00.fillRect(0, 0, 240, var37);
            var20.f_Graphics_00.fillRect(0, 320 - var37, 240, var37);
            var20.f_Graphics_00.setClip(0, 0, 240, 320);
            var20.f_Graphics_00.drawImage(var20.f_Image_arr2_00[14][0], -var20.f_int_22, var37, 0);
            var20.f_Graphics_00.setColor(-1);
            var20.m_019(var20.dialogueTexts[260], 15, 320 - var20.f_int_01 - 10, 210, var20.f_int_01 + 10, false);
            break;
         case 21:
            a var19 = this;
            this.f_Graphics_00.setColor(0);
            var19.f_Graphics_00.fillRect(0, 0, 240, 320);
            var19.f_Graphics_00.setColor(-1);
            var19.f_Graphics_00.drawString("是否开启声音？", 120, 160, 17);
            var19.m_034();
            break;
         case 22:
            this.m_144();
            break;
         case 99:
            a var18 = this;
            int var2 = this.f_int_165 & 7;
            byte var3 = 20;
            int var6 = 0;
            var18.f_Graphics_00.setColor(16316664);
            var18.f_Graphics_00.fillRect(18, 138, 204, 44);
            var18.f_Graphics_00.setColor(0);
            var18.f_Graphics_00.drawRect(20, 140, 199, 39);
            var18.f_Graphics_00.drawRect(18, 138, 203, 43);
            if (!var18.f_bool_32) {
               if (var2 == 7) {
                  var18.f_bool_32 = true;
               }
            } else {
               if (var2 == 7) {
                  var18.f_bool_32 = false;
               }

               var2 = 7 - var2;
            }

            for (int var7 = 0; var7 < 8; var3 += 25) {
               if (!var18.f_bool_32) {
                  var6 = var2 - var7;
               } else {
                  var6 = var7 - var2;
               }

               switch (var6) {
                  case 0:
                     var18.f_Graphics_00.setColor(44527);
                     break;
                  case 1:
                     var18.f_Graphics_00.setColor(6737151);
                     break;
                  case 2:
                     var18.f_Graphics_00.setColor(11593215);
                     break;
                  default:
                     var18.f_Graphics_00.setColor(14277081);
               }

               var18.f_Graphics_00.fillRect(var3 + 2, 142, 21, 6);
               var18.f_Graphics_00.fillRect(215 - var3 + 2, 172, 21, 6);
               var7++;
            }

            var18.f_Graphics_00.setColor(44527);
            var18.f_Graphics_00.drawString(var18.f_String_07, 120, 320 - var18.f_int_01 >> 1, 17);
      }

      if (!this.f_bool_16) {
         if (this.f_bool_05) {
            int var195 = 240 - this.f_int_24 >> 1;
            int var116 = 320 - this.f_int_25 >> 1;
            int var86 = var195;
            a var36 = this;
            int var133 = var86;
            int var151 = var116;
            var36.m_040(var86, var116, var36.f_int_24, var36.f_int_25);
            switch (var36.f_byte_08) {
               case 0:
               case 2:
               case 4:
               case 5:
                  var133 += 16;
                  var151 += 16;
                  var36.f_Graphics_00.setColor(-1);
                  var36.m_019(var36.f_String_02, var133, var151, 180, 240, true);
                  break;
               case 1:
               case 3:
                  var133 += 16;
                  var151 += 16;
                  var36.m_041(var133, var151, 32, 32);
                  Image var168;
                  if ((var168 = var36.f_Image_arr_00[var36.f_byte_07]) != null) {
                     var36.f_Graphics_00.drawImage(var168, var133 + (32 - var168.getWidth() >> 1), var151 + (32 - var168.getHeight() >> 1), 0);
                  }

                  var133 += 42;
                  var36.f_Graphics_00.setColor(16770173);
                  var36.f_Graphics_00.drawString(var36.objectTypeNames[var36.f_byte_07], var133, var151 + (32 - var36.f_int_01 >> 1), 0);
                  var36.f_Graphics_00.setColor(-1);
                  var133 = var86 + 16;
                  var151 += 36;
                  var36.m_019(var36.f_String_02, var133, var151, 180, var36.f_int_25 - 32 - 36, true);
            }

            var151 = var116 + var36.f_int_25 - 12 - 18;
            if (var36.f_byte_05 != 0) {
               var133 = var86 + 13;
               var36.m_002(var36.f_Image_arr2_00[8][11], var133, var151 + 6, (var36.f_byte_05 - 1) * 12, 0, 12, 10);
            }

            if (var36.f_byte_06 != 0) {
               var133 = var86 + var36.f_int_24 - 25;
               var36.m_002(var36.f_Image_arr2_00[8][11], var133, var151 + 6, (var36.f_byte_06 - 1) * 12, 0, 12, 10);
            }
         }
      } else {
         a var35 = this;
         int var131 = 0;
         int var149 = 0;
         int var166 = 0;
         if (var35.f_bool_17) {
            if (++var35.f_int_142 > 4) {
               var35.f_int_142 = 4;
               var35.f_bool_17 = false;
               var35.currentFloor = var35.f_byte_23;
               var35.m_121(var35.currentFloor);
               if (var35.currentFloor == 0) {
                  var35.m_024(1, 2);
                  var35.m_064((var35.f_int_58 - 32 >> 1) - var35.playerPixelX, (var35.f_int_59 - 32 >> 1) - var35.playerPixelY);
               } else if (var35.currentFloor == 50) {
                  var35.m_024(6, 7);
                  var35.m_064((var35.f_int_58 - 32 >> 1) - var35.playerPixelX, (var35.f_int_59 - 32 >> 1) - var35.playerPixelY);
               } else if (var35.currentFloor == 1 && !var35.f_bool_18) {
                  var35.m_024(6, 11);
                  var35.m_064((var35.f_int_58 - 32 >> 1) - var35.playerPixelX, (var35.f_int_59 - 32 >> 1) - var35.playerPixelY);
               } else {
                  int var175;
                  if ((var175 = var35.m_065(var35.f_bool_18)) >= 0) {
                     var35.m_031(var35.entityPixelX[var175] >> 5, var35.entityPixelY[var175] >> 5);
                  }
               }

               var35.m_057();
               var35.applyStepCellEffects();
            }
         } else if (--var35.f_int_142 <= 0) {
            var35.f_bool_17 = true;
            var35.f_bool_16 = false;
         }

         var149 = var131 = 4 - var35.f_int_142;
         var166 = var35.f_int_142 << 1;
         var35.f_Graphics_00.setColor(0);

         for (int var176 = 0; var176 < 40; var149 += 8) {
            for (int var181 = 0; var181 < 30; var131 += 8) {
               var35.f_Graphics_00.fillRect(var131, var149, var166, var166);
               var181++;
            }

            var176++;
            var131 = 4 - var35.f_int_142;
         }
      }
   }

   public final void run() {
      if (this.f_bool_31) {
         a var17 = this;

         int var32;
         while ((var32 = var17.m_151()) == 1) {
            Thread.yield();
         }

         var17.f_String_06 = null;
         int var13 = var32;
         synchronized (this.f_Object_00) {
            this.f_int_161 = var13;
         }
      } else {
         long var1 = 0L;
         long var3 = 0L;
         long var5 = 0L;

         try {
            for (; this.f_bool_00; this.f_int_03++) {
               var3 = (var1 = System.currentTimeMillis()) + this.f_int_02;
               a var15 = this;
               if (this.f_bool_05) {
                  label510:
                  switch (var15.f_byte_08) {
                     case 0:
                        if (var15.gameMode != 15 && var15.gameMode != 17) {
                           var15.m_016();
                        } else {
                           a var31 = var15;
                           switch (var15.f_int_05) {
                              case -7:
                                 var31.f_bool_05 = false;
                                 var31.gameMode = var31.f_byte_01;
                                 break label510;
                              case -2:
                                 var31.m_020(var31.f_int_26 + var31.f_int_30);
                                 break label510;
                              case -1:
                                 var31.m_020(var31.f_int_26 - var31.f_int_30);
                           }
                        }
                        break;
                     case 1:
                        if (var15.f_byte_06 == 0 && var15.f_byte_05 == 0) {
                           var15.m_016();
                        } else {
                           switch (var15.f_byte_06) {
                              case 2:
                              case 3:
                                 switch (var15.f_int_05) {
                                    case -2:
                                    case -1:
                                       var15.m_016();
                                    case 0:
                                       break;
                                    default:
                                       var15.f_bool_05 = false;
                                 }
                           }
                        }
                     case 2:
                     default:
                        break;
                     case 3:
                        switch (var15.f_int_05) {
                           case -7:
                              var15.gameMode = 10;
                              var15.f_bool_05 = false;
                              break label510;
                           case -6:
                              var15.f_bool_05 = false;
                              var15.useItemStack(var15.f_int_104);
                              break label510;
                           default:
                              var15.m_016();
                              break label510;
                        }
                     case 4:
                        switch (var15.f_int_05) {
                           case -7:
                              var15.f_bool_05 = var15.applyMerchantOffer(false);
                              break label510;
                           case -6:
                           case -5:
                              var15.f_bool_05 = var15.applyMerchantOffer(true);
                              break label510;
                           default:
                              var15.m_016();
                              break label510;
                        }
                     case 5:
                        a var30 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                           case -6:
                           case -5:
                              var30.f_bool_05 = false;
                              var30.gameMode = 10;
                              break;
                           case -4:
                              var30.m_076(var30.f_int_90 + 1);
                              break;
                           case -3:
                              var30.m_076(var30.f_int_90 - 1);
                              break;
                           case -2:
                              var30.m_020(var30.f_int_26 + 1);
                              break;
                           case -1:
                              var30.m_020(var30.f_int_26 - 1);
                           case 0:
                        }
                  }

                  var15.f_int_05 = var15.f_int_06 = 0;
               } else {
                  label534:
                  switch (var15.gameMode) {
                     case 0:
                        var15.f_int_02 = 100;
                        if (var15.f_int_04 < 2) {
                           if (var15.f_int_156 == 0) {
                              try {
                                 var15.f_Image_00 = Image.createImage("/l" + var15.f_int_04 + ".png");
                              } catch (Exception var10) {
                              }

                              if (var15.f_Image_00 == null) {
                                 var15.f_int_156 = 0;
                                 var15.f_int_04++;
                              } else {
                                 var15.f_int_156 = 1;
                              }
                           } else if (++var15.f_int_156 > 15) {
                              var15.f_int_04++;
                              var15.f_int_156 = 0;
                           }
                        } else if (++var15.f_int_156 <= 35) {
                           var15.m_143(0, var15.f_int_156);
                        } else {
                           var15.m_147(-1);
                           var15.f_bool_30 = false;
                           var15.f_int_156 = 0;
                           var15.f_Image_00 = null;
                           var15.f_Image_arr2_00[0] = null;
                           var15.m_140();
                           var15.f_int_02 = 75;
                           var15.gameMode = 21;
                           var15.m_000();
                        }
                        break;
                     case 1:
                        if ((var15.f_int_03 & 3) == 0) {
                           var15.m_129(var15.randomBelow(240), 320 - var15.f_int_16 - var15.randomBelow(150), 2, -1);
                        }

                        var15.m_052();
                        a var29 = var15;
                        switch (var15.f_int_05) {
                           case -5:
                           case 53:
                              a var40 = var29;
                              switch (var29.f_byte_arr_00[var40.f_int_10]) {
                                 case 0:
                                    var40.f_Image_arr2_00[10][0] = null;
                                    var40.gameMode = 14;
                                    var40.m_000();
                                    break label534;
                                 case 1:
                                    var40.gameMode = 8;
                                    var40.f_byte_01 = 1;
                                    var40.m_000();
                                    break label534;
                                 case 2:
                                    var40.gameMode = 16;
                                    var40.f_byte_01 = 1;
                                    var40.m_000();
                                    break label534;
                                 case 3:
                                    var40.gameMode = 15;
                                    var40.f_byte_01 = 1;
                                    var40.m_000();
                                    break label534;
                                 case 4:
                                    var40.gameMode = 17;
                                    var40.f_byte_01 = 1;
                                    var40.m_000();
                                    break label534;
                                 case 5:
                                    var40.gameMode = 22;
                                    var40.m_000();
                                 default:
                                    break label534;
                              }
                           case -4:
                           case -2:
                           case 54:
                           case 56:
                              if (var29.f_int_10 < var29.f_int_07 - 1) {
                                 var29.f_int_10++;
                              } else {
                                 var29.f_int_10 = 0;
                              }
                              break label534;
                           case -3:
                           case -1:
                           case 50:
                           case 52:
                              if (var29.f_int_10 > 0) {
                                 var29.f_int_10--;
                              } else {
                                 var29.f_int_10 = var29.f_int_07 - 1;
                              }
                           default:
                              break label534;
                        }
                     case 2:
                        a var28 = var15;
                        if (var15.f_int_143 < 100) {
                           if (var28.f_int_143 < var28.f_int_146) {
                              var28.f_int_143 += 4;
                           } else {
                              byte var41 = var28.f_byte_arr_41[var28.f_int_145];
                              a var37 = var28;
                              switch (var41) {
                                 case 0:
                                    var37.m_001(8);
                                    break;
                                 case 1:
                                    var37.m_001(10);
                                    break;
                                 case 2:
                                    var37.m_001(2);
                                    break;
                                 case 3:
                                    var37.m_001(1);
                                    break;
                                 case 4:
                                    var37.m_001(3);
                                    break;
                                 case 5:
                                    var37.m_001(6);
                                    var37.m_001(4);
                                    var37.m_001(5);
                                    break;
                                 case 6:
                                    var37.m_001(12);
                                    break;
                                 case 7:
                                    var37.m_128();
                                    break;
                                 case 8:
                                    a var39;
                                    (var39 = var37).currentFloor = var39.minFloorReached = 51;
                                    var39.maxFloorReached = 51;
                                    var39.playerHp = 300;
                                    var39.playerAtk = 10;
                                    var39.playerDef = 10;
                                    var39.redKeyCount = 0;
                                    var39.blueKeyCount = 0;
                                    var39.yellowKeyCount = 0;
                                    var39.f_bool_21 = false;
                                    var39.f_bool_22 = false;
                                    var39.f_bool_20 = false;
                                    break;
                                 case 9:
                                    var37.m_118();
                                    break;
                                 case 10:
                                    var37.m_121(var37.currentFloor);
                                    var37.m_057();
                                    break;
                                 case 11:
                                    a var38;
                                    (var38 = var37).m_001(3);
                                    var38.f_Image_02 = var38.f_Image_arr2_00[3][0];
                                    var38.f_int_arr_03 = var38.f_int_arr2_01[0];
                                    var38.f_int_38 = 0;
                                    var38.f_byte_12 = 0;
                                    var38.m_064((var38.f_int_58 - 32 >> 1) - var38.playerPixelX, (var38.f_int_59 - 32 >> 1) - var38.playerPixelY);
                                    break;
                                 case 12:
                                    var37.m_068();
                                    break;
                                 case 13:
                                    var37.m_001(7);
                                    var37.m_001(13);
                                    var37.m_043();
                                    break;
                                 case 14:
                                    var37.m_001(11);
                                    break;
                                 case 15:
                                    var37.m_001(14);
                                    break;
                                 case 16:
                                    if (!var37.m_115(var37.f_int_134)) {
                                       var37.f_int_144 = 0;
                                       var37.gameMode = 8;
                                       var37.m_000();
                                    }
                              }

                              var28.f_int_145++;
                              var28.f_int_146 = var28.f_int_145 * 100 / var28.f_int_144;
                              if (var28.f_int_146 <= var28.f_int_143) {
                                 var28.f_int_146 = var28.f_int_143 + 1;
                              }
                           }
                        } else {
                           var28.f_int_146 = var28.f_int_143 = 0;
                           var28.f_Image_arr2_00[15] = null;
                           var28.gameMode = var28.f_byte_24;
                           if (var28.f_bool_19) {
                              var28.m_000();
                           }
                        }
                        break;
                     case 3:
                        a var26 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                              var26.f_byte_02 = var26.gameMode;
                              var26.gameMode = 4;
                              var26.m_000();
                              break;
                           case -6:
                              var26.gameMode = 10;
                              var26.m_000();
                        }

                        var26 = var15;
                        switch (var15.f_byte_11) {
                           case 0:
                              if (var26.f_bool_27) {
                                 var26.m_032();
                              } else {
                                 var26.m_027();
                              }
                              break;
                           case 1:
                              switch (var26.f_byte_12) {
                                 case 0:
                                    var26.playerPixelY += 8;
                                    if (var26.playerPixelY + var26.f_int_57 + 16 > var26.f_int_59 - 106) {
                                       var26.m_064(var26.f_int_56, var26.f_int_57 - 8);
                                    }
                                    break;
                                 case 1:
                                    var26.playerPixelY -= 8;
                                    if (var26.playerPixelY + var26.f_int_57 + 16 < 106) {
                                       var26.m_064(var26.f_int_56, var26.f_int_57 + 8);
                                    }
                                    break;
                                 case 2:
                                    var26.playerPixelX += 8;
                                    if (var26.playerPixelX + var26.f_int_56 + 16 > var26.f_int_58 - 106) {
                                       var26.m_064(var26.f_int_56 - 8, var26.f_int_57);
                                    }
                                    break;
                                 case 3:
                                    var26.playerPixelX -= 8;
                                    if (var26.playerPixelX + var26.f_int_56 + 16 < 106) {
                                       var26.m_064(var26.f_int_56 + 8, var26.f_int_57);
                                    }
                              }

                              var26.f_int_43 += 8;
                              var26.m_025();
                              if (var26.f_int_43 >= 32) {
                                 var26.f_int_43 = 0;
                                 var26.f_int_38 = 0;
                                 var26.applyStepCellEffects();
                              }
                              break;
                           case 2:
                              var26.m_026();
                              if (!var26.f_bool_06) {
                                 switch (var26.f_int_06) {
                                    case -4:
                                    case 54:
                                       var26.f_byte_12 = 2;
                                       var26.f_int_56 -= 16;
                                       var26.m_064(var26.f_int_56, var26.f_int_57);
                                       break;
                                    case -3:
                                    case 52:
                                       var26.f_byte_12 = 3;
                                       var26.f_int_56 += 16;
                                       var26.m_064(var26.f_int_56, var26.f_int_57);
                                       break;
                                    case -2:
                                    case 56:
                                       var26.f_byte_12 = 0;
                                       var26.f_int_57 -= 16;
                                       var26.m_064(var26.f_int_56, var26.f_int_57);
                                       break;
                                    case -1:
                                    case 50:
                                       var26.f_byte_12 = 1;
                                       var26.f_int_57 += 16;
                                       var26.m_064(var26.f_int_56, var26.f_int_57);
                                 }

                                 switch (var26.f_int_05) {
                                    case -5:
                                    case 53:
                                       var26.f_bool_06 = true;
                                       var26.m_104(0);
                                 }
                              } else {
                                 var26.m_105();
                              }
                              break;
                           case 3:
                              a var36 = var26;
                              switch (var26.f_int_05) {
                                 case -6:
                                 case -5:
                                    var36.useItemStack(var36.f_int_96);
                                 case -7:
                                    var36.f_byte_11 = 0;
                                    break;
                                 case -4:
                                    if (var36.itemStackSize > 0) {
                                       if (var36.f_int_96 < var36.itemStackSize - 1) {
                                          var36.f_int_96++;
                                       } else {
                                          var36.f_int_96 = 0;
                                       }
                                    }
                                    break;
                                 case -3:
                                    if (var36.itemStackSize > 0) {
                                       if (var36.f_int_96 > 0) {
                                          var36.f_int_96 = 0;
                                       } else {
                                          var36.f_int_96 = var36.itemStackSize - 1;
                                       }
                                    }
                              }
                           case 4:
                           default:
                              break;
                           case 5:
                              var26.tickBattle(true);
                        }

                        var15.m_055();
                        break;
                     case 4:
                        a var25 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                              var25.gameMode = var25.f_byte_02;
                              break label534;
                           case -6:
                           case -5:
                              var25.m_012();
                           case -4:
                           case -3:
                           default:
                              break label534;
                           case -2:
                              if (var25.f_int_10 < var25.f_int_07 - 1) {
                                 var25.f_int_10++;
                                 if (var25.f_int_10 >= var25.f_int_12) {
                                    var25.f_int_12++;
                                    var25.f_int_11++;
                                 }
                              } else {
                                 var25.f_int_10 = 0;
                                 var25.f_int_11 = 0;
                                 var25.f_int_12 = var25.f_int_07;
                                 if (var25.f_int_07 > var25.f_int_13) {
                                    var25.f_int_12 = var25.f_int_13;
                                 }
                              }
                              break label534;
                           case -1:
                              if (var25.f_int_10 > 0) {
                                 var25.f_int_10--;
                                 if (var25.f_int_10 < var25.f_int_11) {
                                    var25.f_int_11--;
                                    var25.f_int_12--;
                                 }
                              } else {
                                 var25.f_int_10 = var25.f_int_07 - 1;
                                 var25.f_int_12 = var25.f_int_07;
                                 var25.f_int_11 = var25.f_int_12 - var25.f_int_13;
                                 if (var25.f_int_11 < 0) {
                                    var25.f_int_11 = 0;
                                 }
                              }
                              break label534;
                        }
                     case 5:
                        a var24 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                           case -6:
                              var24.gameMode = 3;
                           case -5:
                           case -4:
                           case -3:
                           default:
                              break;
                           case -2:
                              var24.m_092(var24.f_int_109 + 1);
                              break;
                           case -1:
                              var24.m_092(var24.f_int_109 - 1);
                        }
                     case 6:
                     case 13:
                     case 15:
                     case 17:
                     case 18:
                     case 23:
                     case 24:
                     case 25:
                     case 26:
                     case 27:
                     case 28:
                     case 29:
                     case 30:
                     case 31:
                     case 32:
                     case 33:
                     case 34:
                     case 35:
                     case 36:
                     case 37:
                     case 38:
                     case 39:
                     case 40:
                     case 41:
                     case 42:
                     case 43:
                     case 44:
                     case 45:
                     case 46:
                     case 47:
                     case 48:
                     case 49:
                     case 50:
                     case 51:
                     case 52:
                     case 53:
                     case 54:
                     case 55:
                     case 56:
                     case 57:
                     case 58:
                     case 59:
                     case 60:
                     case 61:
                     case 62:
                     case 63:
                     case 64:
                     case 65:
                     case 66:
                     case 67:
                     case 68:
                     case 69:
                     case 70:
                     case 71:
                     case 72:
                     case 73:
                     case 74:
                     case 75:
                     case 76:
                     case 77:
                     case 78:
                     case 79:
                     case 80:
                     case 81:
                     case 82:
                     case 83:
                     case 84:
                     case 85:
                     case 86:
                     case 87:
                     case 88:
                     case 89:
                     case 90:
                     case 91:
                     case 92:
                     case 93:
                     case 94:
                     case 95:
                     case 96:
                     case 97:
                     case 98:
                     default:
                        break;
                     case 7:
                     case 8:
                        a var23 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                              var23.gameMode = var23.f_byte_01;
                              break label534;
                           case -6:
                           case -5:
                              if (var23.gameMode == 7) {
                                 var23.m_119(var23.currentFloor);
                                 var23.m_114(var23.f_int_134);
                                 var23.m_015((byte)0, "保存成功！", (byte)0, (byte)0);
                              } else if (var23.gameMode == 8 && var23.f_bool_arr_07[var23.f_int_134]) {
                                 var23.f_int_05 = var23.f_int_06 = 0;
                                 var23.f_Image_arr2_00[10][0] = null;
                                 var23.m_131((byte)3, true);
                                 var23.m_132(6);
                                 var23.m_132(8);
                                 var23.m_132(5);
                                 var23.m_132(13);
                                 var23.m_132(9);
                                 var23.m_132(16);
                                 var23.m_132(10);
                                 var23.m_132(2);
                                 var23.m_132(3);
                                 var23.m_132(11);
                              }
                           case -4:
                           case -3:
                           default:
                              break label534;
                           case -2:
                              if (var23.f_int_134 < 5) {
                                 if (++var23.f_int_134 >= var23.f_int_133) {
                                    var23.f_int_133++;
                                    var23.f_int_132++;
                                 }
                              } else {
                                 var23.f_int_132 = var23.f_int_134 = 0;
                                 var23.f_int_133 = var23.f_int_136;
                              }
                              break label534;
                           case -1:
                              if (var23.f_int_134 > 0) {
                                 if (--var23.f_int_134 < var23.f_int_132) {
                                    var23.f_int_132--;
                                    var23.f_int_133--;
                                 }
                              } else {
                                 var23.f_int_134 = 5;
                                 var23.f_int_133 = 6;
                                 var23.f_int_132 = 6 - var23.f_int_136;
                              }
                              break label534;
                        }
                     case 9:
                        a var22 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                              var22.gameMode = 3;
                              break label534;
                           case -6:
                           case -5:
                              if (var22.goldAmount >= var22.alchemyPrice) {
                                 var22.goldAmount = var22.goldAmount - var22.alchemyPrice;
                                 a var35 = var22;
                                 switch (var22.f_int_80) {
                                    case 0:
                                       var35.playerHp = var35.playerHp + var35.f_int_77;
                                       break;
                                    case 1:
                                       var35.playerAtk = var35.playerAtk + var35.f_int_78;
                                       break;
                                    case 2:
                                       var35.playerDef = var35.playerDef + var35.f_int_79;
                                 }

                                 var35.alchemyUpgradeCount++;
                                 var35.alchemyPrice = alchemyPriceFor(var35.alchemyUpgradeCount + 1);
                              } else {
                                 var22.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                              }
                           case -4:
                           case -3:
                           default:
                              break label534;
                           case -2:
                              if (var22.f_int_80 < 2) {
                                 var22.f_int_80++;
                              } else {
                                 var22.f_int_80 = 0;
                              }
                              break label534;
                           case -1:
                              if (var22.f_int_80 > 0) {
                                 var22.f_int_80--;
                              } else {
                                 var22.f_int_80 = 2;
                              }
                              break label534;
                        }
                     case 10:
                        a var21 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                              var21.gameMode = 3;
                              break label534;
                           case -6:
                           case -5:
                              if (var21.f_int_104 < var21.itemStackSize) {
                                 var21.f_byte_07 = var21.itemStackTypes[var21.f_int_104];
                                 switch (var21.f_byte_07) {
                                    case 13:
                                    case 14:
                                       var21.gameMode = 3;
                                       var21.activateItem(var21.f_byte_07);
                                       break label534;
                                    case 15:
                                    case 23:
                                    case 24:
                                    case 25:
                                       var21.m_015((byte)3, var21.itemDescriptions[itemTypeToStackIndex(var21.f_byte_07)], (byte)0, (byte)3);
                                       break label534;
                                    case 16:
                                    case 17:
                                    case 18:
                                    case 19:
                                    case 20:
                                    case 21:
                                    case 22:
                                    default:
                                       var21.m_015((byte)3, var21.itemDescriptions[itemTypeToStackIndex(var21.f_byte_07)], (byte)1, (byte)2);
                                 }
                              }
                              break label534;
                           case -4:
                              if (var21.f_int_104 % var21.f_int_97 < var21.f_int_97 - 1) {
                                 var21.f_int_104++;
                              } else {
                                 var21.f_int_104 = var21.f_int_104 - (var21.f_int_97 - 1);
                              }
                              break label534;
                           case -3:
                              if (var21.f_int_104 % var21.f_int_97 > 0) {
                                 var21.f_int_104--;
                              } else {
                                 var21.f_int_104 = var21.f_int_104 + (var21.f_int_97 - 1);
                              }
                              break label534;
                           case -2:
                              if (var21.f_int_104 < var21.f_int_97 * (var21.f_int_101 - 1)) {
                                 var21.f_int_104 = var21.f_int_104 + var21.f_int_97;
                                 if (var21.f_int_104 / var21.f_int_97 >= var21.f_int_99) {
                                    var21.f_int_98++;
                                    var21.f_int_99++;
                                 }
                              } else {
                                 var21.f_int_104 = var21.f_int_104 - (var21.f_int_101 - 1) * var21.f_int_97;
                                 var21.f_int_98 = 0;
                                 var21.f_int_99 = var21.f_int_100;
                              }
                              break label534;
                           case -1:
                              if (var21.f_int_104 >= var21.f_int_97) {
                                 var21.f_int_104 = var21.f_int_104 - var21.f_int_97;
                                 if (var21.f_int_104 / var21.f_int_97 < var21.f_int_98) {
                                    var21.f_int_98--;
                                    var21.f_int_99--;
                                 }
                              } else {
                                 var21.f_int_104 = var21.f_int_104 + (var21.f_int_101 - 1) * var21.f_int_97;
                                 var21.f_int_99 = var21.f_int_101;
                                 var21.f_int_98 = var21.f_int_101 - var21.f_int_100;
                              }
                           default:
                              break label534;
                        }
                     case 11:
                        var15.m_055();
                        var15.m_105();
                        a var20 = var15;
                        char var33 = '\u0000';
                        switch (var20.f_byte_16) {
                           case 0:
                              if (var20.f_int_118 > 0) {
                                 var20.f_int_118--;
                              } else {
                                 var20.executeScriptInstruction(var20.f_String_05, var20.scriptCursor);
                              }
                              break label534;
                           case 1:
                           case 6:
                              if (var20.f_byte_16 != 6 || var20.m_097()) {
                                 if (var20.f_int_33 < var20.f_String_02.length()) {
                                    label519: {
                                       if (var20.f_int_05 != 0) {
                                          var20.m_017(var20.f_String_02, 129, (var20.f_int_34 << 1) + 12);
                                          var20.f_int_33 = var20.m_021(var20.f_int_33);
                                       } else {
                                          if ((var20.f_int_03 & 1) == 0) {
                                             break label519;
                                          }

                                          var20.f_int_33++;
                                          if (var20.f_String_02.charAt(var20.f_int_33 - 1) == '\\') {
                                             if ((var33 = var20.f_String_02.charAt(var20.f_int_33)) == 'c') {
                                                var20.f_int_33 += 8;
                                             } else if (var33 == 'r') {
                                                var20.f_int_33 += 2;
                                             }
                                          }
                                       }

                                       var20.m_018(var20.f_String_02, 129, (var20.f_int_34 << 1) + 12, var20.f_int_33);
                                    }

                                    if (var20.f_int_28 > var20.f_int_27) {
                                       var20.m_020(var20.f_int_28 - var20.f_int_30);
                                    }
                                 } else {
                                    switch (var20.f_int_05) {
                                       case -2:
                                          var20.m_020(var20.f_int_26 + 1);
                                          break label534;
                                       case -1:
                                          var20.m_020(var20.f_int_26 - 1);
                                       case 0:
                                          break label534;
                                       default:
                                          var20.f_int_33 = 1;
                                          var20.m_096(var20.f_int_112 + 1);
                                          if (var20.f_byte_16 == 6) {
                                             var20.m_103(var20.f_int_114, var20.f_int_115);
                                          }
                                    }
                                 }
                              }
                              break label534;
                           case 2:
                              var20.m_106();
                              break label534;
                           case 3:
                              if (!var20.f_bool_16) {
                                 a var8 = var20;
                                 switch (var20.f_byte_11) {
                                    case 0:
                                       if (var8.f_int_148 > 0) {
                                          var8.f_byte_12 = var8.f_byte_arr_42[--var8.f_int_148];
                                          var8.f_byte_11 = 1;
                                          var8.tryStep(var8.f_byte_12);
                                       } else {
                                          var8.f_bool_27 = false;
                                          var8.f_byte_16 = 0;
                                       }
                                       break label534;
                                    case 1:
                                       switch (var8.f_byte_12) {
                                          case 0:
                                             var8.playerPixelY += 8;
                                             break;
                                          case 1:
                                             var8.playerPixelY -= 8;
                                             break;
                                          case 2:
                                             var8.playerPixelX += 8;
                                             break;
                                          case 3:
                                             var8.playerPixelX -= 8;
                                       }

                                       var8.f_int_43 += 8;
                                       var8.f_int_125 = var8.playerPixelX;
                                       var8.f_int_126 = var8.playerPixelY;
                                       var8.m_025();
                                       if (var8.f_int_43 >= 32) {
                                          var8.f_int_43 = 0;
                                          var8.f_int_38 = 0;
                                          var8.f_byte_11 = 0;
                                          var8.playerCellX = var8.playerPixelX >> 5;
                                          var8.playerCellY = var8.playerPixelY >> 5;
                                          var8.applyStepCellEffects();
                                       }
                                       break label534;
                                    case 5:
                                       var8.tickBattle(var8.f_bool_14);
                                 }
                              }
                              break label534;
                           case 4:
                              if (var20.m_097()) {
                                 var20.gameMode = 3;
                              }
                              break label534;
                           case 5:
                              if (var20.f_bool_16 && !var20.f_bool_17) {
                                 var20.f_byte_16 = 0;
                              }
                           default:
                              break label534;
                        }
                     case 12:
                        var15.m_052();
                        break;
                     case 14:
                        var15.m_014();
                        if ((var15.f_int_03 & 3) == 0) {
                           var15.m_129(var15.randomBelow(240), 320 - var15.f_int_16 - var15.randomBelow(150), 2, -1);
                        }
                        break;
                     case 16:
                        a var19 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                           case -6:
                              var19.gameMode = var19.f_byte_01;
                              break label534;
                           case -5:
                           case -4:
                           case -3:
                              a var7 = var19;
                              var19.f_bool_arr_05[var7.f_int_82] = !var7.f_bool_arr_05[var7.f_int_82];
                              switch (var7.f_byte_arr_11[var7.f_int_82]) {
                                 case 0:
                                    var7.f_bool_29 = !var7.f_bool_29;
                                    if (var7.f_bool_29) {
                                       var7.f_int_155 = 60;
                                       if (!var7.f_bool_28) {
                                          var7.m_139((byte)2, -1);
                                       } else {
                                          var7.m_139((byte)3, -1);
                                       }
                                    } else {
                                       var7.m_140();
                                    }

                                    var7.m_110();
                                    break label534;
                                 case 1:
                                    if (!var7.f_bool_arr_05[1]) {
                                       var7.f_Image_03 = null;
                                    } else if (var7.f_byte_01 == 4) {
                                       var7.m_057();
                                    }
                                 default:
                                    break label534;
                              }
                           case -2:
                              if (++var19.f_int_82 > var19.f_int_83 - 1) {
                                 var19.f_int_82 = 0;
                              }
                              break label534;
                           case -1:
                              if (--var19.f_int_82 < 0) {
                                 var19.f_int_82 = var19.f_int_83 - 1;
                              }
                           default:
                              break label534;
                        }
                     case 19:
                        a var18 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                              var18.gameMode = 4;
                              var18.m_000();
                           case -5:
                           case 53:
                           default:
                              break label534;
                           case -4:
                           case -2:
                           case 54:
                           case 56:
                              if (var18.f_int_10 < var18.f_int_07 - 1) {
                                 var18.f_int_10++;
                              } else {
                                 var18.f_int_10 = 0;
                              }
                              break label534;
                           case -3:
                           case -1:
                           case 50:
                           case 52:
                              if (var18.f_int_10 > 0) {
                                 var18.f_int_10--;
                              } else {
                                 var18.f_int_10 = var18.f_int_07 - 1;
                              }
                              break label534;
                        }
                     case 20:
                        if (var15.f_bool_04 && var15.f_int_05 != 0) {
                           var15.gameMode = 1;
                           var15.m_000();
                        }
                        break;
                     case 21:
                        a var6 = var15;
                        switch (var15.f_int_05) {
                           case -7:
                              var6.f_bool_29 = false;
                              var6.f_int_155 = 0;
                              var6.gameMode = 1;
                              var6.m_000();
                              break label534;
                           case -6:
                              var6.f_bool_29 = true;
                              if (var6.f_int_155 == 0) {
                                 var6.f_int_155 = 60;
                              }

                              var6.gameMode = 1;
                              var6.m_000();
                           default:
                              break label534;
                        }
                     case 22:
                        if (++var15.f_int_156 <= 70) {
                           var15.m_143(1, var15.f_int_156);
                        } else {
                           var15.f_bool_00 = false;
                           CMidlet.m_000();
                        }
                        break;
                     case 99:
                        if (var15.f_bool_31) {
                           switch (var15.f_int_05) {
                              case -7:
                                 var15.m_148();
                                 var15.f_bool_31 = false;
                                 var15.gameMode = 0;
                              default:
                                 if (var15.f_int_165 < 8) {
                                    var15.f_int_165++;
                                 } else if (var15.m_153()) {
                                    var15.f_bool_31 = false;
                                 }
                           }
                        } else if (var15.m_152() == 0 || var15.m_152() == 3 || var15.f_int_05 != 0) {
                           var15.gameMode = 0;
                        }
                  }

                  var15.f_int_05 = 0;
               }

               this.repaint();
               this.serviceRepaints();

               do {
                  Thread.yield();
               } while ((var5 = System.currentTimeMillis()) >= var1 && var5 < var3);
            }
         } catch (Exception var11) {
            var11.printStackTrace();
         }
      }
   }

   private void m_000() {
      switch (this.gameMode) {
         case 1:
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
         case 3:
            this.f_bool_28 = true;
            this.f_byte_13 = 1;
            this.f_byte_14 = 3;
            this.f_bool_05 = false;
            this.applyStepCellEffects();
            this.m_139((byte)3, -1);
            return;
         case 4:
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
            this.f_byte_14 = 3;
            return;
         case 5:
            this.m_091();
            this.f_byte_13 = 0;
            this.f_byte_14 = 3;
            return;
         case 7:
         case 8:
            if (this.f_byte_01 == 1) {
               this.m_001(1);
               this.m_001(8);
               this.m_001(6);
            }

            this.f_byte_13 = 1;
            this.f_byte_14 = 3;
            a var8 = this;
            this.m_111();
            var8.f_int_136 = 208 / (var8.f_int_01 + 4);
            if (var8.f_int_136 > 6) {
               var8.f_int_136 = 6;
            }

            var8.f_int_135 = var8.f_int_00 * 3 + 60;
            if (var8.f_int_135 < 148) {
               var8.f_int_135 = 148;
            }

            var8.f_int_131 = 112 + (var8.f_int_01 + 4) * var8.f_int_136;
            var8.f_int_132 = var8.f_int_134 = 0;
            var8.f_int_133 = var8.f_int_136;
            return;
         case 9:
            return;
         case 10:
            a var7 = this;
            this.f_int_97 = 4;
            var7.f_int_101 = 32 / var7.f_int_97;
            var7.f_int_100 = (268 - var7.f_int_01 - 10) / 40;
            if (var7.f_int_100 > var7.f_int_101) {
               var7.f_int_100 = var7.f_int_101;
            }

            var7.f_int_98 = 0;
            var7.f_int_99 = var7.f_int_100;
            var7.f_int_103 = 42 + 40 * var7.f_int_97;
            var7.f_int_102 = 34 + var7.f_int_01 + 10 + 40 * var7.f_int_100;
            this.f_byte_13 = 1;
            this.f_byte_14 = 3;
            return;
         case 12:
            this.m_076(0);
            this.f_byte_14 = 3;
            return;
         case 13:
            return;
         case 14:
            a var6 = this;
            this.f_int_19 = 0;
            var6.f_int_20 = 0;
            var6.f_int_21 = 0;
            var6.f_String_01 = null;
            var6.f_String_03 = null;
            var6.f_Image_01 = null;
            var6.f_bool_03 = false;
            return;
         case 15:
            this.m_001(8);
            this.m_001(1);
            this.m_015(
               (byte)0,
               "游戏描述：\n\\c99FFCC有人的地方就有江湖，有神仙的地方何尝不是江湖；百战百胜的本事，换不回女人的真心，兄弟的真义；齐天大圣又如何，没有真情实义，做神仙跟做咸鱼有什么区别？\n\n操作方式：按左软键调出物品栏，左右选择一件道具，按确定键使用。\n游戏操作：\n上方向键/2：向上行走\n下方向键/8：向下行走\n左方向键/4：向左行走\n右方向键/6：向右行走\n确定键/5:探索地图\n左软键：打开道具列表\n右软键：打开游戏中菜单\n\n代理发行：广州易诚计算机科技有限公司\n发行商网站：www.9266.net\n客服电话：4006509913\n客服信箱：kefu@9266.net",
               (byte)0,
               (byte)0
            );
            this.f_byte_13 = 0;
            this.f_byte_14 = 3;
            return;
         case 16:
            this.m_001(8);
            this.m_001(1);
            this.f_int_83 = 0;
            this.m_072(0);
            this.m_072(1);
            this.f_byte_13 = 0;
            this.f_byte_14 = 3;
            return;
         case 17:
            this.m_001(8);
            this.m_001(1);
            this.m_015(
               (byte)0,
               "版权所有：\n上海雪鲤鱼计算机科技有限公司\nwww.kgame.com.cn\n手机上网：\nwap.kgame.com.cn\n制作人：梁一\n编剧：王之浣\n策划：孙悦\n程序：杨政\n美术：梁一、王之浣、黄吉力\n测试：金鑫，王毅，计成毅\n版本：V1.0\n客服电话：4006305518",
               (byte)0,
               (byte)0
            );
            this.f_byte_13 = 0;
            this.f_byte_14 = 3;
            return;
         case 19:
            this.m_009();
            this.m_010(12);
            this.m_010(13);
            this.m_010(14);
            this.m_010(15);
            this.m_010(16);
            return;
         case 20:
            this.m_011();
            this.m_001(14);
            this.f_String_03 = this.dialogueTexts[260];
            this.f_int_33 = 1;
            this.m_018(this.f_String_02, 209, (this.f_int_34 << 1) + 12, this.f_int_33);
            return;
         case 21:
            a var1 = this;
            String var2 = "SKY_WAR";
            int var3 = 0;

            try {
               var1.f_RecordStore_00 = RecordStore.openRecordStore(var2, false);
               var1.f_RecordEnumeration_00 = var1.f_RecordStore_00.enumerateRecords(null, null, false);
               var3 = var1.f_RecordEnumeration_00.nextRecordId();
               var1.f_byte_arr_27 = var1.f_RecordStore_00.getRecord(var3);
               var1.f_DataInputStream_00 = new DataInputStream(new ByteArrayInputStream(var1.f_byte_arr_27));

               for (int var4 = 0; var4 < 4; var4++) {
                  var1.f_bool_arr_05[var4] = var1.f_DataInputStream_00.readBoolean();
               }

               byte var10 = var1.f_DataInputStream_00.readByte();

               for (int var11 = 0; var11 < var10; var11++) {
                  var1.f_int_arr_35[var11] = var1.f_DataInputStream_00.readInt();
               }

               var1.f_int_152 = var1.f_DataInputStream_00.readInt();
               var1.f_int_153 = var1.f_DataInputStream_00.readInt();
               var1.f_int_154 = var1.f_DataInputStream_00.readInt();
            } catch (Exception var5) {
               m_116(var2);
               var1.f_bool_arr_05[1] = true;
               var1.m_110();
            }

            this.m_001(8);
            this.f_byte_13 = 1;
            this.f_byte_14 = 2;
            this.f_bool_29 = false;
            return;
         case 22:
            this.m_140();
         case 2:
         case 6:
         case 11:
         case 18:
      }
   }

   protected final void showNotify() {
      if (!this.f_bool_01) {
         this.f_bool_01 = true;
         super.showNotify();
      }
   }

   protected final void hideNotify() {
      if (this.f_bool_01) {
         if (this.gameMode == 3) {
            this.gameMode = 4;
            this.m_000();
         }

         this.m_140();
         this.f_bool_02 = true;
         this.f_bool_01 = false;
         super.hideNotify();
      }
   }

   private void m_001(int var1) {
      int var2 = this.f_int_arr_00[var1];
      int var3 = 0;
      if (this.f_Image_arr2_00[var1] == null) {
         this.f_Image_arr2_00[var1] = new Image[this.f_int_arr_00[var1]];
      }

      this.f_InputStream_00 = this.getClass().getResourceAsStream(this.f_String_arr_01[var1]);
      byte[] var4 = null;

      try {
         for (int var5 = 0; var5 < var2; var5++) {
            var3 = this.f_InputStream_00.read() & 0xFF | this.f_InputStream_00.read() << 8 & 0xFF00;
            if (this.f_Image_arr2_00[var1][var5] == null) {
               if (var4 == null) {
                  var4 = new byte[var3];
               } else if (var4.length < var3) {
                  var4 = new byte[var3];
               }

               this.f_InputStream_00.read(var4, 0, var3);
               this.f_Image_arr2_00[var1][var5] = Image.createImage(var4, 0, var3);
            } else {
               this.f_InputStream_00.skip(var3);
            }
         }

         return;
      } catch (Exception var8) {
         var8.printStackTrace();
      } finally {
         this.m_008();
      }
   }

   private void m_002(Image var1, int var2, int var3, int var4, int var5, int var6, int var7) {
      this.f_Graphics_00.setClip(var2, var3, var6, var7);
      this.f_Graphics_00.drawImage(var1, var2 - var4, var3 - var5, 0);
      this.f_Graphics_00.setClip(0, 0, 240, 320);
   }

   private static void m_003(Image var0, Graphics var1, int var2, int var3, int var4, int var5, int var6, int var7) {
      var1.setClip(var2, var3, 5, 5);
      var1.drawImage(var0, var2 - var4, var3, 0);
      var1.setClip(0, 0, 240, 320);
   }

   private void m_004(Image var1, int var2, int var3, int var4) {
      this.f_DirectGraphics_00.drawImage(var1, var2, var3, 0, f_int_arr_01[var4]);
   }

   private void m_005(Image var1, int var2, int var3, int var4, int var5, int var6, int var7, int var8) {
      this.f_Graphics_00.setClip(var2, var3, var6, var7);
      if (var8 == 0) {
         this.f_Graphics_00.drawImage(var1, var2 - var4, var3 - var5, 0);
      } else {
         int var9 = var1.getWidth();
         int var10 = var1.getHeight();
         switch (var8) {
            case 1:
               this.m_004(var1, var2 - (var9 - var4 - var6), var3 - var5, var8);
               break;
            case 2:
               this.m_004(var1, var2 - var4, var3 - (var10 - var5 - var7), var8);
               break;
            case 3:
               this.m_004(var1, var2 - (var9 - var4 - var6), var3 - (var10 - var5 - var7), var8);
               break;
            case 4:
               this.f_Graphics_00.setClip(var2, var3, var7, var6);
               this.m_004(var1, var2 - var5, var3 - var4, var8);
               break;
            case 5:
               this.f_Graphics_00.setClip(var2, var3, var7, var6);
               this.m_004(var1, var2 - (var10 - var5 - var7), var3 - var4, var8);
               break;
            case 6:
               this.f_Graphics_00.setClip(var2, var3, var7, var6);
               this.m_004(var1, var2 - var5, var3 - (var9 - var4 - var6), var8);
               break;
            case 7:
               this.f_Graphics_00.setClip(var2, var3, var7, var6);
               this.m_004(var1, var2 - (var10 - var5 - var7), var3 - (var9 - var4 - var6), var8);
         }
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
   }

   private static Image m_006(int var0, int var1, int var2) {
      int[] var3 = null;
      var1 = (var3 = new int[1024]).length;

      while (--var1 >= 0) {
         var3[var1] = var2;
      }

      return Image.createRGBImage(var3, 32, 32, true);
   }

   private static Image m_007(Image var0, int var1) {
      int var2 = var0.getWidth();
      int var3 = var0.getHeight();
      int[] var4 = new int[var2 * var3];
      var0.getRGB(var4, 0, var2, 0, 0, var2, var3);
      var1 <<= 24;
      int var5 = var2 * var3;

      while (--var5 >= 0) {
         int var6;
         if ((var6 = var4[var5]) != -1 && (var6 & 0xFF000000) != 0) {
            var4[var5] = var4[var5] & 16777215 | var1;
         }
      }

      return Image.createRGBImage(var4, var2, var3, true);
   }

   private void m_008() {
      if (this.f_InputStream_00 != null) {
         try {
            this.f_InputStream_00.close();
         } catch (Exception var3) {
         }

         this.f_InputStream_00 = null;
      }

      if (this.f_OutputStream_00 != null) {
         try {
            this.f_OutputStream_00.close();
         } catch (Exception var2) {
         }

         this.f_OutputStream_00 = null;
      }

      if (this.f_InputStream_01 != null) {
         try {
            this.f_InputStream_01.close();
         } catch (Exception var1) {
         }

         this.f_InputStream_01 = null;
      }
   }

   protected final void keyPressed(int var1) {
      this.f_int_05 = this.f_int_06 = var1;
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

   protected final void keyReleased(int var1) {
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

   private void m_010(int var1) {
      if (this.f_int_07 < 16) {
         int var2 = this.f_Font_00.stringWidth(this.f_String_arr_02[var1]) + 60;
         this.f_byte_arr_00[this.f_int_07] = (byte)var1;
         if (var2 > this.f_int_08) {
            this.f_int_08 = var2;
         }

         this.f_int_07++;
         if (this.f_int_07 <= this.f_int_13) {
            this.f_int_12 = this.f_int_07;
         }

         this.f_int_09 = this.f_int_09 + this.f_int_15;
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
         case 0:
            return;
         case 1:
            this.gameMode = 3;
            return;
         case 2:
            this.gameMode = 8;
            this.f_byte_01 = 4;
            this.m_000();
            return;
         case 3:
            this.gameMode = 7;
            this.f_byte_01 = 4;
            this.m_000();
            return;
         case 4:
            this.gameMode = 16;
            this.f_byte_01 = 4;
            this.m_000();
            return;
         case 5:
            this.gameMode = 15;
            this.f_byte_01 = 4;
            this.m_000();
            return;
         case 7:
            this.m_011();
            System.gc();

            try {
               Thread.sleep(100L);
            } catch (Exception var1) {
            }

            this.gameMode = 1;
            this.m_000();
            this.f_int_10 = 0;
            return;
         case 9:
            return;
         case 10:
            return;
         case 11:
            this.gameMode = 19;
            this.m_000();
            return;
         case 17:
            this.changeFloor(1, false, false);
            this.gameMode = 3;
            this.playerHp = 1000;
            this.playerAtk = 710;
            this.playerDef = 710;
         case 6:
         case 8:
         case 12:
         case 13:
         case 14:
         case 15:
         case 16:
      }
   }

   private void m_013(Image var1, int var2, int var3, int var4, int var5, int var6, int var7, int var8) {
      int var10 = 86;
      var6 = var6;
      var8 = this.f_byte_03;

      for (int var9 = 0; var9 < var6; var10 += 17) {
         var4 = var10 + this.f_byte_arr_01[var8];
         var7 = var3 + this.f_byte_arr_02[var8];
         this.f_Graphics_00.setClip(var4, var7, 17, 17);
         this.f_Graphics_00.drawImage(var1, var4 - var9, var7 - var5, 0);
         if (++var8 > 7) {
            var8 = 0;
         }

         var9 += 17;
      }

      if (++this.f_byte_03 > 7) {
         this.f_byte_03 = 0;
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
   }

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
      } else {
         if (this.f_int_20 == 0) {
            this.f_Image_01 = null;
            switch (this.f_int_19) {
               case 0:
                  this.m_001(11);
                  this.f_Image_01 = m_007(this.f_Image_arr2_00[11][0], 34);
                  this.f_String_01 = this.f_String_arr_00[4];
                  break;
               case 1:
                  this.m_001(11);
                  this.f_Image_01 = m_007(this.f_Image_arr2_00[11][1], 34);
                  this.f_String_01 = this.f_String_arr_00[6];
                  break;
               case 2:
                  this.f_Image_arr2_00[11] = null;
                  this.m_001(14);
                  this.f_Image_01 = m_007(this.f_Image_arr2_00[14][0], 34);
                  this.f_String_01 = this.f_String_arr_00[7];
            }

            if (this.f_Image_01 != null) {
               this.f_Image_01.getWidth();
            }

            this.f_bool_03 = false;
         }

         this.f_int_20++;
         if (this.f_int_19 < 3) {
            if (this.f_String_03 != null) {
               if (!this.f_bool_03) {
                  if (this.f_int_21 >= this.f_Font_00.stringWidth(this.f_String_arr_03[this.f_int_26]) + 10) {
                     this.f_bool_03 = true;
                     this.f_byte_04 = 0;
                     return;
                  }

                  this.f_int_21 += 4;
                  if (this.f_int_05 != 0) {
                     this.f_int_21 = 240;
                     return;
                  }
               } else if (++this.f_byte_04 >= this.f_int_arr_02.length) {
                  this.f_int_21 = 0;
                  this.f_byte_04 = 0;
                  this.f_bool_03 = false;
                  if (this.f_int_26 < this.f_int_28 - 2) {
                     this.m_020(this.f_int_26 + 2);
                     return;
                  }

                  System.out.println("++~");
                  this.f_int_19++;
                  this.f_int_20 = 0;
                  this.m_014();
                  return;
               }
            }
         } else {
            this.f_Image_01 = null;
            this.f_Image_arr2_00[11] = null;
            this.f_Image_arr2_00[14] = null;
            this.f_String_01 = null;
            this.f_int_19 = 0;
            this.m_067();
         }
      }
   }

   private void m_015(byte var1, String var2, byte var3, byte var4) {
      this.f_bool_05 = true;
      this.f_byte_08 = var1;
      this.f_String_02 = var2;
      switch (this.f_byte_08) {
         case 0:
         case 2:
         case 4:
         case 5:
            this.m_017(var2, 180, 240);
            this.f_int_24 = this.f_int_29 + 32;
            this.f_int_25 = 32 + (this.f_int_27 - this.f_int_26) * this.f_int_34;
            break;
         case 1:
         case 3:
            this.m_017(var2, 180, 234);
            this.f_int_24 = 212;
            this.f_int_25 = 68 + (this.f_int_27 - this.f_int_26) * this.f_int_34;
      }

      this.f_byte_05 = var3;
      this.f_byte_06 = var4;
      if (var3 != 0 || var4 != 0) {
         this.f_int_25 += 18;
      }
   }

   private void m_016() {
      switch (this.f_int_05) {
         case -2:
            this.m_020(this.f_int_26 + 1);
            return;
         case -1:
            this.m_020(this.f_int_26 - 1);
            return;
         case 0:
            return;
         default:
            this.f_bool_05 = false;
      }
   }

   private void m_017(String var1, int var2, int var3) {
      this.m_018(var1, var2, var3, var1.length());
   }

   private void m_018(String var1, int var2, int var3, int var4) {
      char[] var5 = null;
      int var6 = 0;
      char var7 = '\u0000';
      int var8 = 0;
      int var9 = 0;
      int var10 = 0;
      int var11 = 0;
      if (var1 != null && var1.length() != 0) {
         var5 = var1.toCharArray();

         for (int var12 = 0; var12 < var4; var12++) {
            var7 = var5[var12];
            var6 = this.f_Font_00.charWidth(var7);
            if (var7 == '\n') {
               this.f_String_arr_03[var9++] = new String(var5, var10, var11);
               var10 = var12 + 1;
               var8 = 0;
               var11 = 0;
            } else {
               if (var7 != '\\') {
                  var6 = this.f_Font_00.charWidth(var7);
               } else if (++var12 < var4) {
                  if ((var7 = var1.charAt(var12)) == 'c') {
                     if (var12 + 6 < var4 && m_022(var1.substring(var12 + 1, var12 + 7)) != -1) {
                        var12 += 6;
                        var11 += 7;
                        var6 = 0;
                     } else {
                        var6 = this.f_Font_00.charWidth('\\') + this.f_Font_00.charWidth(var7);
                     }
                  } else if (var7 == 'r') {
                     var11++;
                     var6 = 0;
                  }
               } else {
                  var6 = this.f_Font_00.charWidth('\\');
                  var12--;
               }

               if ((var8 += var6) > var2) {
                  this.f_String_arr_03[var9++] = new String(var5, var10, var11);
                  var8 = var6;
                  var10 = var12;
                  var11 = 1;
               } else {
                  var11++;
               }
            }
         }

         if (var11 > 0) {
            this.f_String_arr_03[var9++] = new String(var5, var10, var11);
         }

         this.f_int_26 = 0;
         this.f_int_28 = var9;
         this.f_int_30 = var3 / this.f_int_34;
         if (this.f_int_28 < this.f_int_30) {
            this.f_int_30 = this.f_int_28;
         }

         this.f_int_31 = this.f_int_34 * this.f_int_30;
         if (var9 > this.f_int_30) {
            this.f_int_27 = this.f_int_30;
         } else {
            this.f_int_27 = var9;
         }

         if (this.f_int_28 == 1) {
            this.f_int_29 = this.f_Font_00.stringWidth(var1);
         } else {
            this.f_int_29 = var2;
         }
      }
   }

   private void m_019(String var1, int var2, int var3, int var4, int var5, boolean var6) {
      if (var1 != null) {
         if (this.f_String_03 != var1) {
            this.f_String_03 = var1;
            this.m_017(var1, var4, var5);
         }

         int var17 = var3 + 2;
         int var20 = 0;
         var20 = 0;
         boolean var7 = false;
         this.f_int_32 = this.f_Graphics_00.getColor();
         int var8 = this.f_int_26;

         while (--var8 >= 0) {
            if (this.f_String_arr_03[var8] != null
               && (var20 = this.f_String_arr_03[var8].lastIndexOf(92)) >= 0
               && this.f_String_arr_03[var8].charAt(var20 + 1) == 'c') {
               var20 = m_022(this.f_String_arr_03[var8].substring(var20 + 2, var20 + 8));
               this.f_Graphics_00.setColor(var20);
               break;
            }
         }

         for (int var26 = this.f_int_26; var26 < this.f_int_27; var17 += this.f_int_34) {
            String var10001 = this.f_String_arr_03[var26];
            int var10 = var17;
            int var9 = var2;
            String var25 = var10001;
            a var24 = this;
            int var11 = var25.length();
            int var12 = 0;
            int var13 = var9;
            char var14 = '\u0000';
            int var15 = 0;

            for (int var16 = 0; var16 < var11; var16++) {
               if (var25.charAt(var16) == '\\') {
                  if (var16 > 0) {
                     var24.f_Graphics_00.drawSubstring(var25, var12, var16 - var12, var13, var10, 0);
                     var13 += var24.f_Font_00.substringWidth(var25, var12, var16 - var12);
                  }

                  if (++var16 < var11) {
                     if ((var14 = var25.charAt(var16)) == 'c') {
                        if (var16 + 6 < var11) {
                           var15 = m_022(var25.substring(var16 + 1, var16 + 7));
                        }

                        if (var15 != -1) {
                           var24.f_int_32 = var24.f_Graphics_00.getColor();
                           var24.f_Graphics_00.setColor(var15);
                           var16 += 6;
                           var12 = var16 + 1;
                        } else {
                           var12 = var16 - 1;
                        }
                     } else if (var14 == 'r') {
                        var24.f_Graphics_00.setColor(var24.f_int_32);
                        var12 = var16 + 1;
                     } else {
                        var12 = var16;
                     }
                  }
               }
            }

            if (var12 != 0) {
               if (var12 < var11) {
                  var24.f_Graphics_00.drawSubstring(var25, var12, var11 - var12, var13, var10, 0);
               }
            } else {
               var24.f_Graphics_00.drawString(var25, var9, var10, 0);
            }

            var26++;
         }

         var8 = var2 + (var4 >> 1);
         if (var6) {
            this.f_Graphics_00.setColor(-1);
            if (this.f_int_26 > 0) {
               int var18 = var3 - 8 + (this.f_int_03 & 1);
               this.f_Graphics_00.fillTriangle(var8, var18, var8 - 7, var18 + 7, var8 + 7, var18 + 7);
            }

            if (this.f_int_27 < this.f_int_28) {
               int var19 = var3 + this.f_int_31 + 3 - (this.f_int_03 & 1);
               this.f_Graphics_00.fillTriangle(var8, var19, var8 - 6, var19 - 6, var8 + 6, var19 - 6);
            }
         }
      }
   }

   private boolean m_020(int var1) {
      if (var1 >= 0) {
         if (var1 >= this.f_int_28 - this.f_int_30 + 1) {
            var1 = this.f_int_28 - this.f_int_30;
         }
      } else {
         var1 = 0;
      }

      this.f_int_26 = var1;
      this.f_int_27 = this.f_int_26 + this.f_int_30;
      if (this.f_int_26 < 0) {
         this.f_int_26 = 0;
      }

      if (this.f_int_27 > this.f_int_28) {
         this.f_int_27 = this.f_int_28;
      }

      return true;
   }

   private int m_021(int var1) {
      int var2 = 0;
      int var3 = 0;
      int var4 = 0;
      if (this.f_String_arr_03 != null) {
         int var5 = 0;

         while (var5 < this.f_int_28) {
            var4 = this.f_String_arr_03[var5].length();
            var3 += var4;
            if (var1 >= var2 && var1 < var3) {
               return var3;
            }

            var5++;
            var2 += var4;
         }
      } else {
         var3 = 0;
      }

      return var3;
   }

   private static int m_022(String var0) {
      int var1 = 0;

      try {
         var1 = Integer.parseInt(var0, 16);
      } catch (Exception var2) {
         var1 = 2829099;
         var2.printStackTrace();
      }

      return var1;
   }

   private void applyHpDelta(int var1) {
      if (this.playerHp <= -var1) {
         this.playerHp = 1;
         var1 = this.playerHp - 1;
      } else {
         this.playerHp += var1;
      }

      if (var1 < 0) {
         this.m_125((byte)2, var1, this.playerPixelX, this.playerPixelY);
      } else {
         if (var1 > 0) {
            if (this.f_byte_26 > 0) {
               var1 <<= 4;
            }

            this.m_125((byte)3, var1, this.playerPixelX, this.playerPixelY);
         }
      }
   }

   private void m_024(int var1, int var2) {
      this.playerCellX = var1;
      this.playerCellY = var2;
      this.playerPixelX = var1 << 5;
      this.playerPixelY = var2 << 5;
      this.f_int_43 = 0;
   }

   private void m_025() {
      if (this.f_int_38 < this.f_int_arr_03.length - 1) {
         this.f_int_38++;
      } else {
         this.f_int_38 = 0;
      }
   }

   private void m_026() {
      int var1 = 0;
      int var2 = 0;
      int var3 = 0;
      if (!this.f_bool_06) {
         var1 = (this.f_int_58 - 41 >> 1) - this.f_int_56;
         var2 = (this.f_int_59 >> 1) - this.f_int_57 + this.f_int_46;
      } else {
         var1 = this.playerPixelX;
         var2 = this.playerPixelY;
      }

      if (this.f_int_arr_04[3] == var1 && this.f_int_arr_05[3] == var2) {
         if (this.f_bool_06) {
            this.f_byte_11 = 0;
         }
      } else {
         this.f_int_arr_04[0] = var1;
         this.f_int_arr_05[0] = var2;

         for (int var4 = 1; var4 < 4; var4++) {
            var3 = 4 - var4;
            if (var4 == 0) {
               var3 += 5;
            }

            if (this.f_int_arr_04[var4] < var1) {
               this.f_int_arr_04[var4] = this.f_int_arr_04[var4] + (var1 - this.f_int_arr_04[var4] >> 1) + var3;
               if (this.f_int_arr_04[var4] > var1) {
                  this.f_int_arr_04[var4] = var1;
               }
            } else if (this.f_int_arr_04[var4] > var1) {
               this.f_int_arr_04[var4] = this.f_int_arr_04[var4] + ((var1 - this.f_int_arr_04[var4] >> 1) - var3);
               if (this.f_int_arr_04[var4] < var1) {
                  this.f_int_arr_04[var4] = var1;
               }
            }

            if (this.f_int_arr_05[var4] < var2) {
               this.f_int_arr_05[var4] = this.f_int_arr_05[var4] + (var2 - this.f_int_arr_05[var4] >> 1) + var3;
               if (this.f_int_arr_05[var4] > var2) {
                  this.f_int_arr_05[var4] = var2;
               }
            } else if (this.f_int_arr_05[var4] > var2) {
               this.f_int_arr_05[var4] = this.f_int_arr_05[var4] + ((var2 - this.f_int_arr_05[var4] >> 1) - var3);
               if (this.f_int_arr_05[var4] < var2) {
                  this.f_int_arr_05[var4] = var2;
               }
            }

            var1 = this.f_int_arr_04[var4];
            var2 = this.f_int_arr_05[var4];
         }
      }
   }

   private void m_027() {
      int var1 = 0;
      if (this.gameMode == 11) {
         this.m_032();
      } else {
         switch (this.f_int_05) {
            case -5:
            case 53:
               a var7 = this;

               for (int var2 = 0; var2 < 4; var2++) {
                  var7.f_int_arr_04[var2] = var7.playerPixelX;
                  var7.f_int_arr_05[var2] = var7.playerPixelY;
               }

               var7.f_bool_06 = false;
               a var8 = var7;
               int var3 = 0;
               int var4 = var8.f_int_45;

               while (--var4 >= 0) {
                  var3 = var8.entityType[var4];
                  if (!var8.f_bool_arr_01[var4] && var8.f_byte_arr_03[var3] == 8) {
                     var8.f_int_arr_13[var3] = var8.predictHpLossVsType(var3, true);
                  }
               }

               var7.f_byte_11 = 2;
               var7.f_bool_08 = var7.m_081(13) >= 0;
               break;
            case 49:
               this.f_int_05 = 0;
               if (this.f_bool_13 && (var1 = this.m_065(true)) >= 0) {
                  this.changeFloor(this.entityParam[var1] & 255, false, true);
               }
               break;
            case 55:
               this.f_int_05 = 0;
               if (this.f_bool_13 && (var1 = this.m_065(false)) >= 0) {
                  this.changeFloor(this.entityParam[var1] & 255, true, true);
               }
         }

         switch (this.f_int_06) {
            case -4:
            case 54:
               this.f_byte_12 = 2;
               this.tryStep(this.f_byte_12);
            default:
               return;
            case -3:
            case 52:
               this.f_byte_12 = 3;
               this.tryStep(this.f_byte_12);
               return;
            case -2:
            case 56:
               this.f_byte_12 = 0;
               this.tryStep(this.f_byte_12);
               return;
            case -1:
            case 50:
               this.f_byte_12 = 1;
               this.tryStep(this.f_byte_12);
         }
      }
   }

   private boolean tryStep(byte var1) {
      int var2 = 0;
      var2 = this.playerPixelX >> 5;
      int var3 = this.playerPixelY >> 5;
      switch (var1) {
         case 0:
            var3++;
            break;
         case 1:
            var3--;
            break;
         case 2:
            var2++;
            break;
         case 3:
            var2--;
      }

      boolean var5;
      if (var5 = this.interactWithCell(var2, var3)) {
         this.f_byte_11 = 1;
      } else if (this.f_bool_05 && this.f_byte_11 == 1) {
         this.f_bool_26 = false;
         this.f_int_148 = 0;
         this.f_byte_11 = 0;
      }

      return var5;
   }

   private boolean interactWithCell(int var1, int var2) {
      byte var3 = 0;
      int var4 = 0;
      int var5 = 0;
      var5 = 0;
      boolean var6 = true;
      byte var7 = 0;
      var3 = this.f_byte_arr2_02[var2][var1];
      if (this.f_byte_arr_10[var3] > 0) {
         int var8 = this.f_byte_arr_10[var3];

         while (--var8 >= 0) {
            var4 = this.f_byte_arr2_03[var3][var8] - 1;
            var5 = this.entityType[var4];
            if (this.f_byte_arr_04[var4] != 1) {
               switch (this.f_byte_arr_03[var5]) {
                  case 1:
                     switch (var5) {
                        case 1:
                           if (var6 = this.consumeKeyForDoor((byte)26)) {
                              this.m_047(var4);
                           } else {
                              this.f_int_148 = 0;
                              this.m_015((byte)0, "你没有黄钥匙", (byte)0, (byte)0);
                           }
                           continue;
                        case 2:
                           if (var6 = this.consumeKeyForDoor((byte)27)) {
                              this.m_047(var4);
                           } else {
                              this.f_int_148 = 0;
                              this.m_015((byte)0, "你没有红钥匙", (byte)0, (byte)0);
                           }
                           continue;
                        case 3:
                           if (var6 = this.consumeKeyForDoor((byte)28)) {
                              this.m_047(var4);
                           } else {
                              this.f_int_148 = 0;
                              this.m_015((byte)0, "你没有蓝钥匙", (byte)0, (byte)0);
                           }
                           continue;
                        case 4:
                           this.m_015((byte)0, "障碍物：封印门\n需要消灭指定的怪物才能打开的门！", (byte)0, (byte)0);
                           var6 = false;
                           var8 = 0;
                           continue;
                        case 6:
                           var7++;
                           if (!this.f_bool_arr_00[var4]) {
                              this.f_bool_arr_00[var4] = true;
                              this.f_byte_arr_04[var4] = 2;
                              this.f_int_arr_12[var4] = 9;
                           }
                           continue;
                        case 7:
                        case 8:
                           var7++;
                           continue;
                        case 9:
                           var6 = false;
                           if (this.f_byte_12 == 1) {
                              var8 = 0;
                              this.gameMode = 9;
                              a var16 = this;
                              var5 = this.currentFloor;
                              var16.alchemyPrice = alchemyPriceFor(var16.alchemyUpgradeCount + 1);
                              var16.f_int_77 = scaledByFloorTier(100, var5);
                              var16.f_int_78 = scaledByFloorTier(2, var5);
                              var16.f_int_79 = scaledByFloorTier(4, var5);
                              this.m_000();
                           }
                           continue;
                        case 10:
                           this.m_015((byte)0, "障碍物：三昧真火\n必须用芭蕉扇才能熄灭它. ", (byte)0, (byte)0);
                           var6 = false;
                           var8 = 0;
                           continue;
                        case 11:
                           if (this.currentFloor != 23) {
                              this.m_015((byte)0, "障碍物：墙\n只有金勺子、玄明石可以凿开。或者剧情打开！", (byte)0, (byte)0);
                           }

                           var6 = false;
                           var8 = 0;
                           continue;
                        case 12:
                           this.entityType[var4] = 11;
                           this.m_044(11, var4);
                           this.m_045(11, var4);
                           this.f_byte_arr_04[var4] = 2;
                           this.f_int_arr_12[var4] = 9;
                           this.f_bool_arr_00[var4] = true;
                           var6 = false;
                           continue;
                        case 81:
                           if (this.f_bool_arr_00[var4]) {
                              this.m_015((byte)0, "障碍物：封印门\n需要消灭指定的怪物才能打开的门！", (byte)0, (byte)0);
                              var6 = false;
                              var8 = 0;
                           }
                           continue;
                        case 83:
                           if (this.tryRunScene(this.entityParam[var4], false)) {
                              this.f_bool_05 = false;
                              this.f_String_02 = null;
                              var6 = this.loadLevelScript(this.entityParam[var4]);
                              var8 = 0;
                           }
                        default:
                           continue;
                     }
                  case 8:
                     var5 = this.predictHpLossVsType(var5, true);
                     var6 = false;
                     if (var5 >= 0 && var5 < this.playerHp) {
                        var5 = var4;
                        a var15 = this;
                        this.f_byte_11 = 5;
                        var15.f_byte_arr_04[var5] = 3;
                        var15.f_int_44 = var5;
                        var15.f_int_150 = var15.enemyBaseHp[var15.entityType[var5] - 41];
                        var15.f_bool_14 = true;
                        var15.f_bool_25 = true;
                     } else {
                        this.f_int_148 = 0;
                        this.m_015((byte)0, "你无法战胜它", (byte)0, (byte)0);
                     }
                     break;
                  case 16:
                     var5 = var4;
                     a var12 = this;
                     this.f_int_87 = var5;
                     int var6p = var12.entityParam[var5];
                        byte var9 = (byte)(var6p >>> 8);
                     byte var26 = (byte)var6p;
                     var5 = var12.entityType[var5] == 77 ? 0 : 1;
                     if (var26 > 0) {
                        var26--;
                        if ((var9 & 2) != 0) {
                           switch (var5) {
                              case 0:
                                 boolean var13 = false;
                                 byte var21 = var26;
                                 a var14 = var12;
                                 var12.m_015((byte)4, var14.f_String_arr_07[var21], (byte)0, (byte)0);
                                 break;
                              case 1:
                                 var12.m_015((byte)4, var12.f_String_arr_08[var26], (byte)1, (byte)2);
                           }
                        } else {
                           var12.m_075(var5, var26);
                           switch (var5) {
                              case 0:
                                 var12.m_015((byte)4, var12.f_String_arr_06[var26], (byte)0, (byte)0);
                                 break;
                              case 1:
                                 var12.m_015((byte)4, var12.f_String_arr_09[var26], (byte)1, (byte)2);
                           }
                        }
                     }

                     var6 = false;
                     break;
                  case 32:
                     var6 = false;
               }
            }
         }

         if (var6 && var7 <= 0) {
            var6 = this.isCellWalkable(var1, var2);
         }
      } else if (!this.isCellWalkable(var1, var2)) {
         var6 = false;
      }

      return (boolean)var6;
   }

   private void applyStepCellEffects() {
      int var1 = 0;
      this.playerCellX = this.playerPixelX >> 5;
      this.playerCellY = this.playerPixelY >> 5;
      var1 = 0;
      int var2 = 0;
      byte var3 = this.f_byte_arr2_02[this.playerCellY][this.playerCellX];
      this.f_bool_07 = false;
      a var14 = this;
      var2 = 0;
      int var6 = 0;
      boolean var7;
      if (!(var7 = var14.equippedArmorType == 40)) {
         int var10 = var14.playerCellY;
         int var9 = var14.playerCellX;
         a var8 = var14;
         if (var9 > 0 && var8.m_100(var9 - 1, var10, 61) >= 0 && var9 < var8.mapCellsWide - 1 && var8.m_100(var9 + 1, var10, 61) >= 0
            ? true
            : var10 > 0 && var8.m_100(var9, var10 - 1, 61) >= 0 && var10 < var8.mapCellsHigh - 1 && var8.m_100(var9, var10 + 1, 61) >= 0) {
            var14.applyHpDelta(-(var14.playerHp >> 1));
         }

         byte var11 = 62;
         var10 = var14.playerCellY;
         var9 = var14.playerCellX;
         var8 = var14;
         int var12 = 0;
         if (var9 > 0 && var8.m_100(var9 - 1, var10, 62) >= 0) {
            var12++;
         }

         if (var9 < var8.mapCellsHigh - 1 && var8.m_100(var9 + 1, var10, 62) >= 0) {
            var12++;
         }

         if (var10 > 0 && var8.m_100(var9, var10 - 1, 62) >= 0) {
            var12++;
         }

         if (var10 < var8.mapCellsHigh - 1 && var8.m_100(var9, var10 + 1, 62) >= 0) {
            var12++;
         }

         if (var12 > 0) {
            var14.applyHpDelta(-100);
         }
      }

      boolean var52b = var7;
      int var49 = var14.playerCellY;
      int var45 = var14.playerCellX;
      a var41 = var14;
      int var54 = 0;
      int var7n = 0;
      if (!var52b) {
         var7n = var41.m_100(var45 - 1, var49, 60);
         if (var45 > 0 && var7n >= 0) {
            var54++;
         }

         var7n = var41.m_100(var45 + 1, var49, 60);
         if (var45 < var41.mapCellsHigh - 1 && var7n >= 0) {
            var54++;
         }

         var7n = var41.m_100(var45, var49 - 1, 60);
         if (var49 > 0 && var7n >= 0) {
            var54++;
            var41.m_101(var45, var49 - 1, var7n);
         }

         var7n = var41.m_100(var45, var49 + 1, 60);
         if (var49 < var41.mapCellsHigh - 1 && var7n >= 0) {
            var54++;
         }
      } else {
         var7n = var41.m_100(var45, var49 - 1, 60);
         if (var49 > 0 && var7n >= 0) {
            var41.m_101(var45, var49 - 1, var7n);
         }
      }

      if (var54 > 0) {
         var14.applyHpDelta(-200);
         var6 = 1;
      }

      boolean var4 = var6 != 0;
      if (this.f_byte_arr_10[var3] > 0) {
         int var5 = this.f_byte_arr_10[var3];

         while (--var5 >= 0) {
            var1 = this.f_byte_arr2_03[var3][var5] - 1;
            var2 = this.entityType[var1];
            switch (this.f_byte_arr_03[var2]) {
               case 1:
                  switch (var2) {
                     case 5:
                        short var20 = this.entityParam[var1];
                        this.m_050(this.playerCellX, this.playerCellY, var5);
                        if (this.m_073(var20)) {
                           this.f_byte_11 = 4;
                        }
                        continue;
                     case 6:
                        this.f_bool_07 = true;
                        continue;
                     case 7:
                        var1 = this.entityParam[var1] & 255;
                        this.changeFloor(var1, false, false);
                        this.f_int_06 = 0;
                        var5 = 0;
                        continue;
                     case 8:
                        var1 = this.entityParam[var1] & 255;
                        this.changeFloor(var1, true, false);
                        this.f_int_06 = 0;
                        var5 = 0;
                        continue;
                     case 76:
                        var1 = (this.entityParam[var1] & 255) + 1;
                        this.m_050(this.playerCellX, this.playerCellY, var5);
                        var2 = var1;
                        a var17 = this;
                        var6 = 0;
                        var17.f_int_86 = 0;
                        int var36 = 0;
                        int var42 = 0;

                        for (; var42 < var17.f_int_45; var42++) {
                           if (!var17.f_bool_arr_01[var42]) {
                              var36 = var17.entityParam[var42] & 255;
                              if ((var6 = var17.entityType[var42]) != 76) {
                                 if (var6 == 4 && var36 == var2) {
                                    var17.f_short_arr_03[var17.f_int_86] = (short)var42;
                                    var17.f_short_arr_01[var17.f_int_86] = (short)(var17.entityPixelX[var42] >> 5);
                                    var17.f_short_arr_02[var17.f_int_86] = (short)(var17.entityPixelY[var42] >> 5);
                                    var17.f_int_86++;
                                 }
                              } else if (var36 + 1 == var2) {
                                 int var10001 = var17.entityPixelX[var42] >> 5;
                                 int var52 = var17.entityPixelY[var42] >> 5;
                                 var49 = var10001;
                                 a var46 = var17;
                                 byte var55 = var17.f_byte_arr2_02[var52][var49];
                                 byte var38 = var46.f_byte_arr_10[var55];
                                 if (var46.f_byte_arr_10[var55] > 0) {
                                    var6 = var38;

                                    while (--var6 >= 0) {
                                       var46.f_bool_arr_01[var46.f_byte_arr2_03[var55][var6] - 1] = true;
                                    }

                                    var46.f_byte_arr_10[var55] = 0;
                                 }
                              }
                           }
                        }

                        if (var17.f_int_86 > 0) {
                           this.f_byte_11 = 4;
                        }
                        continue;
                     case 83:
                        if (this.tryRunScene(this.entityParam[var1], true)) {
                           this.f_bool_05 = false;
                           this.loadLevelScript(this.entityParam[var1]);
                           var4 = true;
                           var5 = 0;
                        }
                     default:
                        continue;
                  }
               case 2:
               case 4:
                  this.pickupItemType(var2);
                  this.m_047(var1);
               case 3:
               case 5:
               case 6:
               case 7:
               case 8:
            }
         }

         if (this.f_byte_11 != 4) {
            this.f_byte_11 = 0;
            if (!var4) {
               this.m_027();
            }
         }
      } else if (this.f_bool_27) {
         if (!var4) {
            this.m_032();
         } else {
            this.f_byte_11 = 0;
         }
      } else {
         this.f_byte_11 = 0;
         if (!var4) {
            this.m_027();
         }
      }

      label216:
      if (this.f_bool_26) {
         if (this.f_int_148 > 0) {
            if (this.f_byte_arr_42[this.f_int_148 - 1] == this.f_byte_12) {
               this.f_int_148--;
               break label216;
            }

            this.f_int_148 = 0;
         }

         this.f_bool_26 = false;
      }

      a var21 = this;
      var2 = this.playerCellX;
      var6 = var21.playerCellY;
      int var7f = var21.f_int_70 >> 5;
      int var43 = var21.f_int_72 >> 5;
      var45 = var21.f_int_71 >> 5;
      var49 = var21.f_int_73 >> 5;
      var21.f_bool_13 = false;
      if (var21.f_bool_12) {
         if (var6 > 0) {
            if (var2 == var7f && var6 - 1 == var43) {
               var21.f_bool_13 = true;
            } else if (var2 == var45 && var6 - 1 == var49) {
               var21.f_bool_13 = true;
            }
         }

         if (var6 < var21.mapCellsHigh - 1) {
            if (var2 == var7f && var6 + 1 == var43) {
               var21.f_bool_13 = true;
            } else if (var2 == var45 && var6 + 1 == var49) {
               var21.f_bool_13 = true;
            }
         }

         if (var2 > 0) {
            if (var2 - 1 == var7f && var6 == var43) {
               var21.f_bool_13 = true;
            } else if (var2 - 1 == var45 && var6 == var49) {
               var21.f_bool_13 = true;
            }
         }

         if (var2 < var21.mapCellsWide - 1) {
            if (var2 + 1 == var7f && var6 == var43) {
               var21.f_bool_13 = true;
               return;
            }

            if (var2 + 1 == var45 && var6 == var49) {
               var21.f_bool_13 = true;
            }
         }
      }
   }

   private void m_031(int var1, int var2) {
      if (this.isCellWalkable(var1, var2 - 1)) {
         var2--;
      } else if (this.isCellWalkable(var1, var2 + 1)) {
         var2++;
      } else if (this.isCellWalkable(var1 - 1, var2)) {
         var1--;
      } else if (this.isCellWalkable(var1 + 1, var2)) {
         var1++;
      }

      this.m_024(var1, var2);
      this.m_064((this.f_int_58 - 32 >> 1) - this.playerPixelX, (this.f_int_59 - 32 >> 1) - this.playerPixelY);
   }

   private void m_032() {
      if (this.f_int_148 > 0) {
         this.f_byte_12 = this.f_byte_arr_42[--this.f_int_148];
         boolean var1 = this.tryStep(this.f_byte_12);
         if (this.f_int_148 == 0) {
            this.f_bool_27 = false;
            return;
         }

         if (!var1) {
            this.f_bool_27 = false;
            if (this.f_byte_11 != 5) {
               this.f_byte_11 = 0;
               this.f_int_148 = 0;
            }
         }
      }
   }

   private void m_033(int var1, int var2) {
      int var3 = var1 + this.playerPixelX;
      int var4 = var2 + this.playerPixelY;
      int var5 = 0;
      if (this.f_bool_07) {
         var4 -= this.f_int_46;
      }

      this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], var3 + 8, var4 + 22, 0);
      switch (this.f_byte_12) {
         case 0:
            if (this.f_byte_11 == 2) {
               var5 = 4;

               while (--var5 >= 0) {
                  this.m_002(
                     this.f_Image_02, var1 + this.f_int_arr_04[var5] - 4, var2 + this.f_int_arr_05[var5] - 14, this.f_int_arr_03[this.f_int_38] * 41, 0, 41, 46
                  );
               }
            }

            this.m_002(this.f_Image_arr2_00[3][0], var3 - 3, var4 - 18, this.f_int_arr_03[this.f_int_38] * 41, 0, 41, 46);
            break;
         case 1:
            this.m_002(this.f_Image_arr2_00[3][0], var3 - 8, var4 - 14, this.f_int_arr_03[this.f_int_38] * 41, 46, 41, 46);
            if (this.f_byte_11 == 2) {
               var5 = 4;

               while (--var5 >= 0) {
                  this.m_002(
                     this.f_Image_02,
                     var1 + this.f_int_arr_04[var5] - 8,
                     var2 + this.f_int_arr_05[var5] - 14,
                     this.f_int_arr_03[this.f_int_38] * 41,
                     46,
                     41,
                     46
                  );
               }
            }
            break;
         case 2:
            if (this.f_byte_11 == 2) {
               var5 = 4;

               while (--var5 >= 0) {
                  this.m_002(
                     this.f_Image_02,
                     var1 + this.f_int_arr_04[var5] - 6,
                     var2 + this.f_int_arr_05[var5] - 14,
                     this.f_int_arr_03[this.f_int_38] * 41,
                     92,
                     41,
                     46
                  );
               }
            }

            this.m_002(this.f_Image_arr2_00[3][0], var3 - 8, var4 - 16, this.f_int_arr_03[this.f_int_38] * 41, 92, 41, 46);
            break;
         case 3:
            if (this.f_byte_11 == 2) {
               var5 = 4;

               while (--var5 >= 0) {
                  this.m_005(
                     this.f_Image_02, var1 + this.f_int_arr_04[var5], var2 + this.f_int_arr_05[var5] - 14, this.f_int_arr_03[this.f_int_38] * 41, 92, 41, 46, 1
                  );
               }
            }

            this.m_005(this.f_Image_arr2_00[3][0], var3, var4 - 16, this.f_int_arr_03[this.f_int_38] * 41, 92, 41, 46, 1);
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
      if (this.f_bool_26) {
         var3 = (this.playerCellX << 5) + var1;
         var4 = (this.playerCellY << 5) + var2;
         var5 = this.f_int_148;

         while (--var5 >= 0) {
            this.f_Graphics_00.setColor(136);
            switch (this.f_byte_arr_42[var5]) {
               case 0:
                  var4 += 32;
                  this.m_002(this.f_Image_arr2_00[8][22], var3 + 5, var4 + 5 + this.f_int_46, 21, 0, 21, 22);
                  break;
               case 1:
                  var4 -= 32;
                  this.m_002(this.f_Image_arr2_00[8][22], var3 + 5, var4 + 5 + this.f_int_46, 0, 0, 21, 22);
                  break;
               case 2:
                  var3 += 32;
                  this.m_005(this.f_Image_arr2_00[8][22], var3 + 5 + this.f_int_46, var4 + 5, 42, 0, 21, 22, 1);
                  break;
               case 3:
                  var3 -= 32;
                  this.m_002(this.f_Image_arr2_00[8][22], var3 + 5 + this.f_int_46, var4 + 5, 42, 0, 21, 22);
            }
         }
      }

      var3 = var1 + this.playerPixelX;
      var4 = var2 + this.playerPixelY;
      switch (this.f_byte_11) {
         case 2:
            var5 = this.f_int_46;
            this.m_002(this.f_Image_arr2_00[8][23], 5 - var5, 148, 44, 0, 22, 24);
            this.m_002(this.f_Image_arr2_00[8][23], 109, 25 - var5, 0, 0, 22, 24);
            this.m_002(this.f_Image_arr2_00[8][23], 109, this.f_int_48 - 29 + var5, 22, 0, 22, 24);
            this.m_005(this.f_Image_arr2_00[8][23], 215 + var5, 148, 44, 0, 22, 24, 1);
            return;
         case 3:
            int var10001 = var3 + 16;
            var2 = var4 - 40;
            var1 = var3 + 16;
            byte var15 = 0;
            int var19 = 0;
            if (this.f_int_96 < this.itemStackSize) {
               var15 = this.itemStackTypes[this.f_int_96];
               this.f_Graphics_00.drawImage(this.f_Image_arr_00[var15], var1 - 16, var2 + 4, 0);
               if (this.f_int_96 > 0) {
                  this.f_Graphics_00.setColor(-1);
                  this.m_093(var1 - 18 - (this.f_int_03 & 1), var2 + 16, (byte)3);
               }

               if (this.f_int_96 < this.itemStackSize - 1) {
                  this.f_Graphics_00.setColor(-1);
                  this.m_093(var1 + 18 + (this.f_int_03 & 1), var2 + 16, (byte)3);
               }

               this.f_Graphics_00.setColor(-1);
               var19 = this.f_Font_00.stringWidth(this.objectTypeNames[var15]);
               var1 -= var19 >> 1;
               var2 -= this.f_int_01 + 4;
               this.f_Graphics_00.fillRect(var1 - 5, var2, var19 + 10, this.f_int_01 + 4);
               this.f_Graphics_00.setColor(0);
               this.f_Graphics_00.drawString(this.objectTypeNames[var15], var1, var2 + 2, 0);
               return;
            } else {
               var1 -= this.f_int_00 >> 1;
               this.f_Graphics_00.setColor(-1);
               var2 -= this.f_int_01 + 4 - 32;
               this.f_Graphics_00.fillRect(var1 - 5, var2, this.f_int_00 + 10, this.f_int_01 + 4);
               this.f_Graphics_00.setColor(0);
               this.f_Graphics_00.drawString("无", var1, var2 + 2, 0);
            }
         case 4:
         case 5:
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

   private void m_035(int var1, int var2) {
      var1 = var2;
      this.f_Graphics_00.setClip(0, 0, 240, 320);
      this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][3], 0, var1, 0);
      if (this.currentFloor > 50) {
         this.f_Graphics_00.setColor(-1);
         this.m_142("引子", 32, var1 + 9 + (19 - this.f_int_01 >> 1), 17, this.f_int_arr_36);
         var1 += 4;
      } else {
         var1 += 4;
         this.m_042(this.f_Image_arr2_00[8][18], this.currentFloor, 40, var1 + 2);
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][8], 41, var1 + 10, 0);
      }

      this.f_Graphics_00.setColor(2435368);
      this.f_Graphics_00.fillRect(65, var2, 175, 18);
      var1 -= 4;
      this.m_036(0, this.yellowKeyCount, 65, var1);
      this.m_036(1, this.blueKeyCount, 107, var1);
      this.m_036(2, this.redKeyCount, 149, var1);
      var1 += 3;
      this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][1], 191, var1, 0);
      this.m_042(this.f_Image_arr2_00[8][2], this.goldAmount, 237, var1 + 2);
   }

   private void m_036(int var1, int var2, int var3, int var4) {
      this.f_Graphics_00.setClip(var3, var4, 18, 16);
      this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][6], var3 - var1 * 18, var4, 0);
      this.f_Graphics_00.setClip(0, 0, 240, 320);
      var4 += 5;
      var3 += 40;
      this.m_042(this.f_Image_arr2_00[8][2], var2, var3, var4);
   }

   private void m_037(int var1, int var2) {
      this.m_039(0, var2, 240, 50);
      this.f_Graphics_00
         .drawImage(this.f_Image_arr2_00[3][1], 40 - (this.f_Image_arr2_00[3][1].getWidth() >> 1), var2 + 50 - this.f_Image_arr2_00[3][1].getHeight(), 0);
      var2 += 13;
      byte var6 = 83;
      int var3 = var6;
      int var4 = var2;
      this.f_Graphics_00.setClip(var3, var4, 10, 10);
      this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][7], var3, var4 - 2, 0);
      this.f_Graphics_00.setClip(0, 0, 240, 320);
      var3 += 16;
      this.m_041(var3, var4 + 1, 58, 11);
      this.m_042(this.f_Image_arr2_00[8][2], this.playerHp, var3 + 52, var4 + 2);
      int var9 = var6;
      var4 += 12;
      this.m_002(this.f_Image_arr2_00[8][7], var9, var4, 10, 0, 10, 13);
      var9 += 16;
      this.f_Graphics_00.setColor(512);
      this.m_041(var9, var4 + 1, 58, 11);
      this.m_042(this.f_Image_arr2_00[8][2], this.playerAtk, var9 + 52, var4 + 2);
      var4 += 12;
      int var11 = var6;
      this.m_002(this.f_Image_arr2_00[8][7], var11, var4, 20, 0, 10, 13);
      var11 += 16;
      this.f_Graphics_00.setColor(512);
      this.m_041(var11, var4 + 1, 58, 11);
      this.m_042(this.f_Image_arr2_00[8][2], this.playerDef, var11 + 52, var4 + 2);
      var11 += 60;
      var4 = var2 + 2;
      this.m_041(var11, var4, 32, 32);
      if (this.equippedWeaponType == 0) {
         this.f_Graphics_00.setColor(-1);
         this.f_Graphics_00.drawString(this.f_String_arr_04[this.equippedWeaponType], var11 + (32 - this.f_int_00 >> 1), var4 + (32 - this.f_int_01 >> 1), 0);
      } else {
         this.f_Graphics_00
            .drawImage(
               this.f_Image_arr_00[this.equippedWeaponType],
               var11 + (32 - this.f_Image_arr_00[this.equippedWeaponType].getWidth() >> 1),
               var4 + (32 - this.f_Image_arr_00[this.equippedWeaponType].getHeight() >> 1),
               0
            );
      }

      var11 += 34;
      this.m_041(var11, var4, 32, 32);
      if (this.equippedArmorType == 0) {
         this.f_Graphics_00.setColor(-1);
         this.f_Graphics_00.drawString(this.f_String_arr_04[this.equippedArmorType], var11 + (32 - this.f_int_00 >> 1), var4 + (32 - this.f_int_01 >> 1), 0);
      } else {
         this.f_Graphics_00
            .drawImage(
               this.f_Image_arr_00[this.equippedArmorType],
               var11 + (32 - this.f_Image_arr_00[this.equippedArmorType].getWidth() >> 1),
               var4 + (32 - this.f_Image_arr_00[this.equippedArmorType].getHeight() >> 1),
               0
            );
      }
   }

   private void m_038(int var1, int var2, int var3, int var4, int var5) {
      this.m_040(var2, var3, var4, var5);
      int var10002 = var2 + 2;
      var3 -= 2;
      var2 = var10002;
      var1 = var1;
      a var6 = this;
      int[] var8;
      var4 = (var8 = this.f_int_arr2_00[var1]).length;
      var2 = var2;

      for (int var13 = 0; var13 < var4; var2 += 14) {
         var6.f_Graphics_00.setClip(var2, var3, 14, 16);
         var6.f_Graphics_00.drawImage(var6.f_Image_arr2_00[8][13], var2 - var8[var13] * 14, var3, 0);
         var13++;
      }

      var6.f_Graphics_00.setClip(0, 0, 240, 320);
   }

   private void m_039(int var1, int var2, int var3, int var4) {
      int var5 = var1;
      int var6 = var2;
      this.f_Graphics_00.setClip(var5, var6, 26, 16);
      this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var5, var6, 0);

      for (int var7 = var5 + 26; var7 < var1 + var3 - 26; var7 += 16) {
         this.f_Graphics_00.setClip(var7, var6, 16, 16);
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var7 - 26, var6, 0);
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
      this.m_005(this.f_Image_arr2_00[8][0], var1 + var3 - 26, var6, 0, 0, 26, 16, 1);
      var5 = var1 + 11;
      var6 += 16;
      this.f_Graphics_00.setColor(2699825);
      this.f_Graphics_00.fillRect(var5, var6, var3 - 22, var4 - 16);
      var5 = var1 + var3 - 11;

      while (var6 < var2 + var4) {
         this.f_Graphics_00.setClip(var1, var6, 11, 16);
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var1 - 42, var6, 0);
         this.f_Graphics_00.setClip(var5, var6, 11, 16);
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var5 - 53, var6, 0);
         var6 += 16;
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
   }

   private void m_040(int var1, int var2, int var3, int var4) {
      int var5 = var1;
      int var6 = var2;
      this.f_Graphics_00.setClip(var5, var6, 26, 16);
      this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var5, var6, 0);

      for (int var7 = var5 + 26; var7 < var1 + var3 - 26; var7 += 16) {
         this.f_Graphics_00.setClip(var7, var6, 16, 16);
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var7 - 26, var6, 0);
         this.m_005(this.f_Image_arr2_00[8][0], var7, var6 + var4 - 16, 26, 0, 16, 16, 2);
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
      this.m_005(this.f_Image_arr2_00[8][0], var1 + var3 - 26, var6, 0, 0, 26, 16, 1);
      var5 = var1 + 11;
      var6 += 16;
      this.f_Graphics_00.setColor(2699825);
      this.f_Graphics_00.fillRect(var5, var6, var3 - 22, var4 - 32);
      var5 = var1 + var3 - 11;

      while (var6 < var2 + var4 - 16) {
         this.f_Graphics_00.setClip(var1, var6, 11, 16);
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var1 - 42, var6, 0);
         this.f_Graphics_00.setClip(var5, var6, 11, 16);
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][0], var5 - 53, var6, 0);
         var6 += 16;
      }

      var6 = var2 + var4 - 16;
      this.m_005(this.f_Image_arr2_00[8][0], var1, var6, 0, 0, 26, 16, 2);
      this.m_005(this.f_Image_arr2_00[8][0], var1 + var3 - 26, var6, 0, 0, 26, 16, 3);
      this.f_Graphics_00.setClip(0, 0, 240, 320);
   }

   private void m_041(int var1, int var2, int var3, int var4) {
      int var5 = var2 + var4 - 2;
      int var6 = var1 + var3 - 1;
      this.f_Graphics_00.setColor(4803902);
      this.f_Graphics_00.fillRect(var1 + 1, var2, var3 - 2, var4 - 1);
      this.f_Graphics_00.drawLine(var1, var2 + 1, var1, var5);
      this.f_Graphics_00.setColor(1645850);
      this.f_Graphics_00.drawLine(var6, var2 + 1, var6, var5);
      this.f_Graphics_00.drawLine(var1 + 1, var5 + 1, var6 - 1, var5 + 1);
   }

   private int m_042(Image var1, int var2, int var3, int var4) {
      boolean var5;
      if (var5 = var2 < 0) {
         var2 = -var2;
      }

      int var6 = var1.getWidth() / 10;
      int var7 = var1.getHeight();
      var3 = var3;
      int var8 = 0;
      int var9 = 0;

      do {
         var3 -= var6 + 1;
         var8 = var2 % 10;
         this.f_Graphics_00.setClip(var3, var4, var6, var7);
         this.f_Graphics_00.drawImage(var1, var3 - var8 * var6, var4, 0);
         var2 /= 10;
         var9++;
      } while (var2 > 0);

      this.f_Graphics_00.setClip(0, 0, 240, 320);
      if (var5) {
         this.f_Graphics_00.setColor(15027533);
         this.f_Graphics_00.drawLine(var3 - var6 + 1, var4 + 3, var3 - 1, var4 + 3);
         var9++;
      }

      return var9;
   }

   private void m_043() {
      this.f_Image_arr_00 = new Image[this.objectTypeNames.length];
      this.f_byte_arr_03 = new byte[this.objectTypeNames.length];
      this.f_bool_arr_02 = new boolean[this.objectTypeNames.length];
      int var1 = this.f_bool_arr_03.length;

      while (--var1 >= 0) {
         this.f_bool_arr_02[var1] = this.f_bool_arr_03[var1];
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
      this.f_byte_arr_05[49] = this.f_byte_arr_05[53] = this.f_byte_arr_05[74] = 1;
      this.f_byte_arr_05[69] = 2;

      for (int var2 = 1; var2 <= 12; var2++) {
         this.f_byte_arr_03[var2] = 1;
      }

      for (int var3 = 13; var3 <= 32; var3++) {
         this.f_byte_arr_03[var3] = 2;
      }

      for (int var4 = 33; var4 <= 40; var4++) {
         this.f_byte_arr_03[var4] = 4;
      }

      for (int var5 = 41; var5 < 79; var5++) {
         this.f_byte_arr_03[var5] = 8;
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

   private void m_044(int var1, int var2) {
      Image var3 = this.f_Image_arr_00[var1];
      switch (var1) {
         case 67:
            this.m_001(9);
            break;
         case 69:
            this.m_001(9);
            this.f_int_arr_08[var2] = 96;
            this.f_int_arr_09[var2] = 32;
            return;
         case 72:
            this.f_int_arr_08[var2] = 32;
            this.f_int_arr_09[var2] = 45;
            return;
         default:
            if (var3 != null) {
               this.f_int_arr_08[var2] = this.f_byte_arr_07[var1];
               this.f_int_arr_09[var2] = var3.getHeight();
               return;
            }
      }

      this.f_int_arr_08[var2] = 32;
      this.f_int_arr_09[var2] = 32;
   }

   private void m_045(int var1, int var2) {
      this.f_int_arr_12[var2] = this.f_byte_arr_06[var1];
      this.f_int_arr_10[var2] = 0;
   }

   private void m_046(int var1) {
      this.f_byte_arr_04[var1] = 2;
      int var2 = this.entityType[var1];
      switch (this.f_byte_arr_03[var2]) {
         case 1:
            return;
         case 2:
         case 8:
         case 16:
         case 32:
            this.f_int_arr_08[var1] = 27;
            this.f_int_arr_09[var1] = 29;
            this.f_int_arr_12[var1] = 8;
            this.f_int_arr_10[var1] = 0;
      }
   }

   private void m_047(int var1) {
      this.f_byte_arr_04[var1] = 1;
      int var2 = this.entityType[var1];
      switch (this.f_byte_arr_03[var2]) {
         case 1:
            switch (var2) {
               case 1:
               case 2:
               case 3:
                  this.f_int_arr_08[var1] = 45;
                  this.f_int_arr_09[var1] = 56;
                  this.f_int_arr_12[var1] = 6;
                  this.f_int_arr_10[var1] = 0;
                  break;
               case 11:
                  this.f_int_arr_08[var1] = 45;
                  this.f_int_arr_09[var1] = 57;
                  this.f_int_arr_12[var1] = 2;
                  this.f_int_arr_10[var1] = 0;
            }

            this.m_057();
            return;
         case 2:
         case 4:
            this.f_int_arr_08[var1] = 45;
            this.f_int_arr_09[var1] = 56;
            this.f_int_arr_12[var1] = 10;
            this.f_int_arr_10[var1] = 0;
            return;
         case 8:
         case 16:
         case 32:
            this.f_int_arr_08[var1] = 27;
            this.f_int_arr_09[var1] = 29;
            this.f_int_arr_12[var1] = 6;
            this.f_int_arr_10[var1] = 0;
      }
   }

   private int m_048(int var1, int var2, int var3, int var4) {
      this.m_044(var1, this.f_int_45);
      this.m_045(var1, this.f_int_45);
      int var5 = this.f_int_arr_08[this.f_int_45] >> 5;
      byte var6 = 0;
      int var7 = var2 + 16 >> 5;
      int var8 = var3 + 16 >> 5;
      int var9 = 0;
      if (var5 <= 0) {
         var5 = 1;
      }

      var2 = var7 << 5;
      var3 = var8 << 5;
      this.entityType[this.f_int_45] = var1;
      this.entityPixelX[this.f_int_45] = var2;
      this.entityPixelY[this.f_int_45] = var3;
      this.f_bool_arr_01[this.f_int_45] = false;
      this.f_bool_arr_00[this.f_int_45] = true;
      this.entityParam[this.f_int_45] = (short)var4;
      this.f_byte_arr_04[this.f_int_45] = 0;
      this.f_int_45++;

      for (int var12 = 0; var12 < var5; var12++) {
         if ((var6 = this.f_byte_arr2_02[var8][var7 + var9]) == 0) {
            this.f_byte_arr2_02[var8][var7 + var9] = ++this.f_byte_15;
            var6 = this.f_byte_15;
         }

         this.f_byte_arr2_03[var6][this.f_byte_arr_10[var6]] = (byte)this.f_int_45;
         this.f_byte_arr_10[var6]++;
         var9++;
      }

      if (var1 == 7) {
         this.f_int_70 = var2;
         this.f_int_72 = var3;
      } else if (var1 == 8) {
         this.f_int_71 = var2;
         this.f_int_73 = var3;
      }

      return this.f_int_45 - 1;
   }

   private void m_049(int var1, int var2, int var3) {
      byte var6 = this.f_byte_arr2_02[var2][var1];
      byte var7 = this.f_byte_arr_10[var6];
      int var4 = 0;
      if (var7 > 0) {
         for (int var5 = 0; var5 < var7; var5++) {
            if (this.entityType[this.f_byte_arr2_03[var6][var5] - 1] != var3) {
               this.f_byte_arr2_03[var6][var4++] = this.f_byte_arr2_03[var6][var5];
            } else {
               this.f_bool_arr_01[this.f_byte_arr2_03[var6][var5] - 1] = true;
            }
         }

         this.f_byte_arr_10[var6] = (byte)var4;
      }
   }

   private void m_050(int var1, int var2, int var3) {
      byte var4 = this.f_byte_arr2_02[var2][var1];
      byte var5 = this.f_byte_arr_10[var4];
      if (this.f_byte_arr_10[var4] > 0) {
         this.f_bool_arr_01[this.f_byte_arr2_03[var4][var3] - 1] = true;

         for (int var6 = var3; var6 < var5 - 1; var6++) {
            this.f_byte_arr2_03[var4][var6] = this.f_byte_arr2_03[var4][var6 + 1];
         }

         this.f_byte_arr_10[var4]--;
      }
   }

   private void m_051(int var1) {
      byte var2 = this.f_byte_arr2_02[this.entityPixelY[var1] >> 5][this.entityPixelX[var1] >> 5];
      byte var3 = this.f_byte_arr_10[var2];
      this.f_bool_arr_01[var1] = true;
      if (var3 > 0) {
         for (int var4 = 0; var4 < var3; var4++) {
            if (this.f_byte_arr2_03[var2][var4] - 1 == var1) {
               for (int var5 = var4; var5 < var3 - 1; var5++) {
                  this.f_byte_arr2_03[var2][var5] = this.f_byte_arr2_03[var2][var5 + 1];
               }

               this.f_byte_arr_10[var2]--;
               return;
            }
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

   private void m_053(int var1, int var2, boolean var3) {
      Image var4 = null;
      var4 = null;
      var4 = null;
      int var11 = 0;
      byte var12 = 0;
      var4 = null;
      byte var13 = 0;
      this.m_052();
      var3 = false;

      for (int var14 = 0; var14 < this.f_int_45; var14++) {
         if (!this.f_bool_arr_01[var14] && this.f_bool_arr_00[var14]) {
            int var5 = this.entityPixelX[var14];
            int var6 = this.entityPixelY[var14];
            int var7 = this.f_int_arr_08[var14];
            int var8 = this.f_int_arr_09[var14];
            var13 = this.f_byte_arr_04[var14];
            int var9 = var1 + var5;
            int var10 = var2 + var6;
            if (var9 >= -var7 && var9 <= this.f_int_58 && var10 >= -12 && var10 <= 20 + this.f_int_59) {
               var11 = this.entityType[var14];
               if ((var4 = this.f_Image_arr_00[var11]) != null) {
                  var12 = this.f_byte_arr_03[var11];
                  if (!var3
                     && this.playerPixelX > var5 - 32
                     && this.playerPixelX < var5 + 32
                     && this.playerPixelY >= var6 - 32
                     && var6 > this.playerPixelY
                     && !this.f_bool_arr_02[var11]) {
                     this.m_033(var1, var2);
                     var3 = true;
                  }

                  var5 = var7 * this.f_int_arr2_02[this.f_int_arr_12[var14]][this.f_int_arr_10[var14]];
                  int var31 = 0;
                  switch (var12) {
                     case 1:
                        if (var11 == 6) {
                           var10 += 5 - this.f_int_46;
                        } else if (var11 == 9) {
                           var9 += 32;
                        }

                        if (this.f_byte_arr_04[var14] != 1) {
                           this.m_002(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, 0, var7, var8);
                        } else {
                           this.m_002(this.f_Image_arr2_00[4][12], var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, 0, var7, var8);
                        }
                        break;
                     case 2:
                        if (var13 == 1) {
                           this.m_002(this.f_Image_arr2_00[4][12], var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, 0, var7, var8);
                        } else if (var13 == 2) {
                           this.m_002(this.f_Image_arr2_00[2][9], var9 + 2, var10 + 2 - (this.f_int_arr_10[var14] << 3), var5, 0, 27, 29);
                        } else if (var11 < 26) {
                           this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], var9 + 8, var10 + 22, 0);
                           this.f_Graphics_00.drawImage(this.f_Image_arr_00[var11], var9 + (32 - var7 >> 1), var10 - (var8 - 24) + this.f_int_46, 0);
                        } else {
                           this.f_Graphics_00.drawImage(this.f_Image_arr_00[var11], var9 + (32 - var7 >> 1), var10 - (var8 - 30), 0);
                        }
                        break;
                     case 4:
                        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], var9 + 8, var10 + 22, 0);
                        this.m_002(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 24) + this.f_int_46, var5, 0, var7, var8);
                        break;
                     case 8:
                        if (var13 == 3) {
                           var9 += this.randomBelow(5) - 2;
                           var10 += this.randomBelow(5) - 2;
                        }

                        if (var13 == 1 || var13 == 2) {
                           this.m_002(this.f_Image_arr2_00[2][9], var9 + 2, var10 + 2 - (this.f_int_arr_10[var14] << 3), var5, 0, 27, 29);
                        } else if (var11 != 67 && var11 != 69) {
                           this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], var9 + 8, var10 + 22, 0);
                           var10 -= 4;
                           this.m_002(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, 0, var7, var8);
                        } else {
                           var7 = var10;
                           var31 = var9;
                           var5 = var14;
                           a var22 = this;
                           var8 = this.entityType[var5];
                           var5 = var22.f_int_arr2_02[var22.f_int_arr_12[var5]][var22.f_int_arr_10[var5]];
                           switch (var8) {
                              case 67:
                                 var22.f_Graphics_00.drawImage(var22.f_Image_arr2_00[5][0], var31 + 8, var7 + 22, 0);
                                 int var10001 = var31 + 16;
                                 int var10002 = var7 + 16;
                                 var7 = var5;
                                 var31 = var10002;
                                 var5 = var10001;
                                 a var23 = var22;
                                 byte[] var42 = var22.f_byte_arr2_01[var7];
                                 Object var45 = null;

                                 for (byte var59 = 0; var59 < var42.length; var59 += 4) {
                                    var12 = (byte)((Object[])(var45 = var23.f_byte_arr2_00[var42[var59]]))[0];
                                    var23.m_002(
                                       var23.f_Image_arr2_00[9][var12],
                                       var5 + var42[var59 + 1],
                                       var31 + var42[var59 + 2],
                                       (int)((Object[])var45)[1],
                                       (int)((Object[])var45)[2],
                                       (int)((Object[])var45)[3],
                                       (int)((Object[])var45)[4]
                                    );
                                 }
                                 break;
                              case 69:
                                 var22.f_Graphics_00.drawImage(var22.f_Image_arr2_00[5][0], var31 + 40, var7 + 22, 0);
                                 var31 += 3;
                                 var7 -= 49;
                                 var7 += var22.f_int_46;
                                 var22.f_Graphics_00.drawImage(var22.f_Image_arr_00[var8], var31, var7, 0);
                                 if (var5 > 0) {
                                    var22.m_002(var22.f_Image_arr2_00[9][5], var31 + 32, var7 + 27, 26 * (var5 - 1), 0, 26, 13);
                                 }
                           }
                        }

                        if (this.f_byte_11 == 2 && this.f_bool_08) {
                           int var24 = this.f_int_arr_13[var11];
                           this.f_Graphics_00
                              .drawImage(
                                 this.f_Image_arr2_00[2][8],
                                 var9 - 3 + this.f_byte_arr_01[this.f_int_03 & 7],
                                 var10 - 24 + this.f_byte_arr_02[this.f_int_03 & 7],
                                 0
                              );
                           if (var24 >= 0) {
                              this.m_127(
                                 this.f_Image_arr2_00[2][7],
                                 this.f_int_arr_13[var11],
                                 var9 + 30 + this.f_byte_arr_01[this.f_int_03 & 7],
                                 var10 - 17 + this.f_byte_arr_02[this.f_int_03 & 7]
                              );
                           } else {
                              this.m_002(
                                 this.f_Image_arr2_00[2][7],
                                 var9 + 14 + this.f_byte_arr_01[this.f_int_03 & 7],
                                 var10 - 17 + this.f_byte_arr_02[this.f_int_03 & 7],
                                 70,
                                 0,
                                 7,
                                 9
                              );
                           }
                        }
                        break;
                     case 16:
                        if (var13 != 1 && var13 != 2) {
                           this.f_Graphics_00.drawImage(this.f_Image_arr_00[15], var9 + 1, var10 + 8 + this.f_int_46, 0);
                           this.m_002(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 24) + this.f_int_46, var5, 0, var7, var8);
                           break;
                        }

                        this.m_002(this.f_Image_arr2_00[2][9], var9 + 2, var10 + 2 - (this.f_int_arr_10[var14] << 3), var5, 0, 27, 29);
                        break;
                     case 32:
                        this.f_Graphics_00.drawImage(this.f_Image_arr2_00[5][0], var9 + 8, var10 + 22, 0);
                        if (var13 == 1 || var13 == 2) {
                           this.m_002(this.f_Image_arr2_00[2][9], var9 + 2, var10 + 2 - (this.f_int_arr_10[var14] << 3), var5, 0, 27, 29);
                        } else if (var11 != 72) {
                           this.m_002(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 16) + this.f_int_46, var5, 0, var7, var8);
                        } else if (var14 != this.f_int_127) {
                           this.m_002(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, 0, var7, var8);
                        } else {
                           var31 = var8 * this.f_byte_19;
                           if (this.f_byte_19 == 3) {
                              var31 -= var8;
                              this.m_005(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, var31, var7, var8, 1);
                           } else {
                              this.m_002(var4, var9 + (32 - var7 >> 1), var10 - (var8 - 32), var5, var31, var7, var8);
                           }
                        }
                  }
               } else {
                  String var20 = this.objectTypeNames[this.entityType[var14]];
                  int var21 = this.f_Font_00.stringWidth(var20) + 8 >> 1;
                  this.f_Graphics_00.setColor(-1);
                  this.f_Graphics_00.fillArc(var9, var10, 32, 32, 0, 360);
                  var10 += 32 - this.f_int_01 >> 1;
                  var9 -= var21 - 16;
                  this.f_Graphics_00.fillRect(var9, var10, var21 << 1, this.f_int_01);
                  this.f_Graphics_00.setColor(0);
                  this.f_Graphics_00.drawRect(var9, var10, (var21 << 1) - 1, this.f_int_01 - 1);
                  this.f_Graphics_00.drawString(this.objectTypeNames[this.entityType[var14]], var9 + 4, var10, 0);
               }
            }
         }
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
      if (!var3) {
         this.m_033(var1, var2);
      }

      if (this.f_byte_11 == 5) {
         a var25 = this;
         if (this.f_int_44 >= 0) {
            int var30 = var25.f_int_56 + var25.entityPixelX[var25.f_int_44] + 16;
            int var37 = var25.f_int_57 + var25.entityPixelY[var25.f_int_44] + 32;
            int var43 = var25.f_int_151 & 7;
            int var47;
            if ((var47 = var25.f_byte_arr_44[var43]) > 0) {
               var47 = var47 - 1 << 2;
               var25.m_002(
                  var25.f_Image_arr2_00[3][2],
                  var30 - (var25.f_byte_arr_43[var47 + 2] >> 1),
                  var37 - (var25.f_byte_arr_43[var47 + 3] >> 1),
                  var25.f_byte_arr_43[var47],
                  var25.f_byte_arr_43[var47 + 1],
                  var25.f_byte_arr_43[var47 + 2],
                  var25.f_byte_arr_43[var47 + 3]
               );
            }
         }
      }

      if (this.f_int_70 >= 0) {
         int var50 = var1 + this.f_int_70;
         int var53 = var2 + this.f_int_72;
         if (var50 >= -32 && var50 <= this.f_int_58 && var53 >= -12 && var53 <= 20 + this.f_int_59) {
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[2][1], var50 + 5, var53 - 30 + this.f_int_46, 0);
         }
      }

      if (this.f_int_71 >= 0) {
         int var51 = var1 + this.f_int_71;
         int var54 = var2 + this.f_int_73;
         if (var51 >= -32 && var51 <= this.f_int_58 && var54 >= -12 && var54 <= 20 + this.f_int_59) {
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[2][2], var51, var54 - 30 + this.f_int_46, 0);
         }
      }
   }

   private void m_054() {
      int var1 = 0;
      int var2 = 0;
      if (this.f_int_45 > 0) {
         for (int var3 = this.f_int_45; var3 >= 1; var3--) {
            var1 = this.entityPixelY[0];

            for (int var4 = 1; var4 < var3; var4++) {
               if (!this.f_bool_arr_01[var4]) {
                  if ((var2 = this.entityPixelY[var4]) < var1) {
                     int var10001 = var4 - 1;
                     int var5 = var4;
                     var2 = var10001;
                     a var23 = this;
                     int var6 = this.entityType[var2];
                     int var7 = var23.entityPixelX[var2];
                     int var8 = var23.entityPixelY[var2];
                     int var9 = var23.f_int_arr_08[var2];
                     int var10 = var23.f_int_arr_09[var2];
                     byte var11 = var23.f_byte_arr_04[var2];
                     boolean var12 = var23.f_bool_arr_01[var2];
                     boolean var13 = var23.f_bool_arr_00[var2];
                     short var14 = var23.entityParam[var2];
                     int var15;
                     if ((var15 = var9 >> 5) <= 0) {
                        var15 = 1;
                     }

                     int var16 = var8 >> 5;
                     int var17 = var7 >> 5;
                     byte var18 = 0;
                     byte var19 = 0;

                     for (int var20 = 0; var20 < var15; var20++) {
                        var18 = var23.f_byte_arr2_02[var16][var17 + var20];
                        var19 = var23.f_byte_arr_10[var18];

                        for (int var21 = 0; var21 < var19; var21++) {
                           if (var23.f_byte_arr2_03[var18][var21] == var2 + 1) {
                              var23.f_byte_arr2_03[var18][var21] = (byte)(var5 + 1);
                              break;
                           }
                        }
                     }

                     var23.entityType[var2] = var23.entityType[var5];
                     var23.entityPixelX[var2] = var23.entityPixelX[var5];
                     var23.entityPixelY[var2] = var23.entityPixelY[var5];
                     var23.f_int_arr_08[var2] = var23.f_int_arr_08[var5];
                     var23.f_int_arr_09[var2] = var23.f_int_arr_09[var5];
                     var23.f_byte_arr_04[var2] = var23.f_byte_arr_04[var5];
                     var23.f_bool_arr_01[var2] = var23.f_bool_arr_01[var5];
                     var23.f_bool_arr_00[var2] = var23.f_bool_arr_00[var5];
                     var23.entityParam[var2] = var23.entityParam[var5];
                     var23.m_045(var23.entityType[var2], var2);
                     var23.entityType[var5] = var6;
                     var23.entityPixelX[var5] = var7;
                     var23.entityPixelY[var5] = var8;
                     var23.f_int_arr_08[var5] = var9;
                     var23.f_int_arr_09[var5] = var10;
                     var23.f_byte_arr_04[var5] = var11;
                     var23.f_bool_arr_01[var5] = var12;
                     var23.f_bool_arr_00[var5] = var13;
                     var23.entityParam[var5] = var14;
                     var23.m_045(var23.entityType[var5], var5);
                     var16 = var23.entityPixelY[var2] >> 5;
                     var17 = var23.entityPixelX[var2] >> 5;
                     if ((var15 = var23.f_int_arr_08[var2] >> 5) <= 0) {
                        var15 = 1;
                     }

                     for (int var33 = 0; var33 < var15; var33++) {
                        var18 = var23.f_byte_arr2_02[var16][var17 + var33];
                        var19 = var23.f_byte_arr_10[var18];

                        for (int var34 = 0; var34 < var19; var34++) {
                           if (var23.f_byte_arr2_03[var18][var34] == var5 + 1) {
                              var23.f_byte_arr2_03[var18][var34] = (byte)(var2 + 1);
                              break;
                           }
                        }
                     }

                     var2 = this.entityPixelY[var4];
                  }

                  var1 = var2;
               }
            }
         }
      }
   }

   private void m_055() {
      int var1 = 0;
      if (this.f_int_45 > 0 && (this.f_int_03 & 1) != 0) {
         int var2 = this.f_int_45;

         while (--var2 >= 0) {
            if (!this.f_bool_arr_01[var2] && this.f_bool_arr_00[var2] && (var1 = this.f_int_arr_12[var2]) > 0) {
               if (this.f_int_arr_10[var2] < this.f_int_arr2_02[var1].length - 1) {
                  this.f_int_arr_10[var2]++;
               } else {
                  switch (this.f_byte_arr_04[var2]) {
                     case 1:
                        this.m_051(var2);
                        break;
                     case 2:
                        this.m_044(this.entityType[var2], var2);
                        this.m_045(this.entityType[var2], var2);
                        this.f_byte_arr_04[var2] = 0;
                        break;
                     default:
                        this.f_int_arr_10[var2] = 0;
                  }
               }
            }
         }
      }
   }

   private void m_056(boolean var1) {
      boolean var2 = false;
      int var3 = 0;
      if (--this.f_int_49 < -154) {
         this.f_int_49 = 0;
      }

      boolean var4 = false;
      if (!var1) {
         var3 = 0;
      } else {
         var3 = this.f_int_48 - 320 + 6;
      }

      for (int var6 = this.f_int_49; var6 < 240; var4 = false) {
         for (int var5 = var3; var5 > -320; var5 -= 320) {
            if (var4) {
               this.m_004(this.f_Image_arr2_00[1][0], var6, var5, 2);
            } else {
               this.f_Graphics_00.drawImage(this.f_Image_arr2_00[1][0], var6, var5, 0);
            }

            var4 = !var4;
         }

         var6 += 77;
      }
   }

   private void m_057() {
      this.f_Image_03 = null;
      if (this.f_bool_arr_05[1]) {
         Image var1;
         a var8;
         Graphics var2 = (var1 = Image.createImage((var8 = this).mapCellsWide << 2, var8.mapCellsHigh << 2)).getGraphics();
         int var3 = 0;
         int var4 = 0;
         var2.setColor(13097429);
         var2.fillRect(0, 0, var8.mapCellsWide << 2, var8.mapCellsHigh << 2);
         var2.setColor(7509153);
         byte var5 = 0;

         for (int var6 = 0; var6 < var8.mapCellsHigh; var3 = 0) {
            for (int var7 = 0; var7 < var8.mapCellsWide; var3 += 4) {
               var5 = var8.f_byte_arr2_02[var6][var7];
               if (var8.f_bool_arr2_00[var6][var7]) {
                  var2.setColor(7509153);
                  var2.fillRect(var3, var4, 4, 4);
               }

               if (var5 > 0 && var8.f_byte_arr_10[var5] > 0) {
                  var5 = var8.f_byte_arr2_03[var5][0];
                  if (!var8.f_bool_arr_01[var5 - 1] && var8.f_byte_arr_04[var5 - 1] != 1) {
                     switch (var8.entityType[var5 - 1]) {
                        case 1:
                           var2.setColor(15461198);
                           var2.fillRect(var3, var4, 3, 3);
                           var2.setColor(9794048);
                           var2.drawLine(var3 + 3, var4, var3 + 3, var4 + 3);
                           var2.drawLine(var3, var4 + 3, var3 + 3, var4 + 3);
                           break;
                        case 2:
                           var2.setColor(16273480);
                           var2.fillRect(var3, var4, 4, 4);
                           var2.setColor(8388608);
                           var2.drawLine(var3 + 3, var4, var3 + 3, var4 + 3);
                           var2.drawLine(var3, var4 + 3, var3 + 3, var4 + 3);
                           break;
                        case 3:
                           var2.setColor(4767984);
                           var2.fillRect(var3, var4, 3, 3);
                           var2.setColor(549016);
                           var2.drawLine(var3 + 3, var4, var3 + 3, var4 + 3);
                           var2.drawLine(var3, var4 + 3, var3 + 3, var4 + 3);
                        case 4:
                        case 5:
                        case 6:
                        default:
                           break;
                        case 7:
                           m_003(var8.f_Image_arr2_00[8][12], var2, var3 - 1, var4 - 1, 5, 0, 5, 5);
                           break;
                        case 8:
                           m_003(var8.f_Image_arr2_00[8][12], var2, var3 - 1, var4 - 1, 0, 0, 5, 5);
                     }
                  }
               }

               var7++;
            }

            var6++;
            var4 += 4;
         }

         this.f_Image_03 = m_007(var1, 170);
      }
   }

   private static short m_058(InputStream var0) throws IOException {
      return (short)(var0.read() & 0xFF | var0.read() << 8 & 0xFF00);
   }

   private void m_059() {
      this.f_byte_arr2_02 = new byte[this.mapCellsHigh][this.mapCellsWide];
      this.f_byte_arr_10 = new byte[128];
      this.f_byte_arr2_03 = new byte[128][16];
      this.f_byte_15 = 0;
   }

   private boolean isCellWalkable(int var1, int var2) {
      return var1 >= 0 && var1 < this.mapCellsWide && var2 >= 0 && var2 < this.mapCellsHigh ? this.f_bool_arr2_00[var2][var1] : false;
   }

   private void m_061() {
      int var1 = 0;
      byte var2 = 0;
      int var3 = this.mapCellsWide;
      int var4 = this.mapCellsHigh;

      for (int var5 = 0; var5 < var4; var1 += var3 << 1) {
         for (int var6 = 0; var6 < var3; var1 += 2) {
            if ((var2 = this.mapTerrainGrid[var1]) < this.f_bool_arr_04.length) {
               this.f_bool_arr2_00[var5][var6] = this.f_bool_arr_04[var2];
            } else {
               this.f_bool_arr2_00[var5][var6] = false;
            }

            var6++;
         }

         var5++;
      }
   }

   private boolean m_062(int var1, int var2) {
      return var1 >= 0 && var1 < this.mapCellsWide && var2 >= 0 && var2 < this.mapCellsHigh ? this.f_byte_arr_10[this.f_byte_arr2_02[var2][var1]] > 0 : false;
   }

   private void m_063(int var1, int var2) {
      if (this.f_byte_11 == 4) {
         a var10 = this;
         this.f_int_55++;
         if (var10.f_int_55 == 18) {
            a var13 = var10;
            short var16 = 0;
            int var4 = var13.f_int_86;

            while (--var4 >= 0) {
               var16 = var13.f_short_arr_03[var4];
               var13.f_int_arr_12[var16] = 6;
               var13.f_byte_arr_04[var16] = 1;
            }
         } else if (var10.f_int_55 >= 24) {
            var10.f_int_55 = 0;
            a var12 = var10;

            for (int var3 = 0; var3 < var12.f_int_86; var3++) {
               var12.m_049(var12.f_short_arr_01[var3], var12.f_short_arr_02[var3], 4);
            }

            var12.f_int_86 = 0;
            var10.f_int_54 = 0;
            var10.f_byte_11 = 0;
         }

         if (var10.f_int_54 == -2) {
            var10.f_int_54 = 2;
         } else {
            var10.f_int_54 = -2;
         }

         var2 = 20 + this.f_int_54;
      }

      var1 = this.f_int_56 + (this.f_int_60 << 4);
      var2 = this.f_int_57 + var2 + (this.f_int_62 << 4);
      int var18 = 0;
      int var6 = var1;
      int var7 = var2;
      int var21;
      var2 = var21 = this.f_int_60 + (this.f_int_62 * this.mapCellsWide << 1);

      for (int var8 = this.f_int_62; var8 < this.f_int_63; var2 = var21 += this.mapCellsWide << 1) {
         for (int var9 = this.f_int_60; var9 < this.f_int_61; var6 += 16) {
            if ((var18 = this.mapTerrainGrid[var2]) > 0) {
               int var5 = (var18 & 7) << 4;
               var18 = var18 >> 3 << 4;
               this.m_005(this.f_Image_arr2_00[2][0], var6, var7, var5, var18, 16, 16, this.mapTransformGrid[var2]);
            }

            var9++;
            var2++;
         }

         var8++;
         var7 += 16;
         var6 = var1;
      }

      this.f_Graphics_00.setClip(0, 0, 240, 320);
   }

   private void m_064(int var1, int var2) {
      if (!this.f_bool_10) {
         if (var1 > 64) {
            var1 = 64;
         } else if (var1 < -((this.mapCellsWide + 2 << 5) - this.f_int_58)) {
            var1 = -((this.mapCellsWide + 2 << 5) - this.f_int_58);
         }

         this.f_int_56 = var1;
         if (var1 < 0) {
            this.f_int_60 = -var1 >> 4;
            this.f_int_61 = this.f_int_60 + (this.f_int_58 >> 4) + 1;
         } else {
            this.f_int_60 = 0;
            this.f_int_61 = (this.f_int_58 - var1 >> 4) + 1;
         }

         if (this.f_int_61 > this.mapCellsWide << 1) {
            this.f_int_61 = this.mapCellsWide << 1;
         }

         if (var2 > 64) {
            var2 = 64;
         } else if (var2 < -((this.mapCellsHigh + 2 << 5) - this.f_int_59)) {
            var2 = -((this.mapCellsHigh + 2 << 5) - this.f_int_59);
         }
      } else {
         this.f_int_56 = this.f_int_58 - this.f_int_52 >> 1;
         this.f_int_60 = 0;
         this.f_int_61 = this.mapCellsWide << 1;
      }

      if (!this.f_bool_11) {
         this.f_int_57 = var2;
         if (var2 < 0) {
            this.f_int_62 = -var2 >> 4;
            this.f_int_63 = this.f_int_62 + (this.f_int_59 >> 4) + 2;
         } else {
            this.f_int_62 = 0;
            this.f_int_63 = (this.f_int_59 - var2 >> 4) + 1;
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

   private int m_065(boolean var1) {
      if (var1) {
         for (int var2 = 0; var2 < this.f_int_45; var2++) {
            int var3 = this.entityParam[var2] >> 9;
            if (this.entityType[var2] == 7 && var3 == 0) {
               return var2;
            }
         }
      } else {
         for (int var5 = 0; var5 < this.f_int_45; var5++) {
            int var6 = this.entityParam[var5] >> 9;
            if (this.entityType[var5] == 8 && var6 == 0) {
               return var5;
            }
         }
      }

      return -1;
   }

   private boolean changeFloor(int var1, boolean var2, boolean var3) {
      boolean var5;
      label37: {
         var5 = false;
         if (var3) {
            if (var1 < this.minFloorReached) {
               this.m_015((byte)0, this.f_String_arr_00[0], (byte)0, (byte)0);
               var5 = false;
               break label37;
            }

            if (var1 > this.maxFloorReached) {
               var5 = false;
               this.m_015((byte)0, this.f_String_arr_00[1], (byte)0, (byte)0);
               break label37;
            }
         }

         var5 = true;
      }

      if (var5) {
         var5 = false;
         if (var1 < 0) {
            this.m_015((byte)0, this.f_String_arr_00[2], (byte)0, (byte)0);
         } else if (var1 > this.f_int_68) {
            this.m_015((byte)0, this.f_String_arr_00[3], (byte)0, (byte)0);
         } else {
            var5 = true;
            this.f_bool_16 = true;
            if (var1 < this.minFloorReached) {
               this.minFloorReached = var1;
            } else if (var1 > this.maxFloorReached) {
               this.maxFloorReached = var1;
               if (this.maxFloorReached < 51 && this.maxFloorReached > this.f_int_154) {
                  this.f_int_154 = this.maxFloorReached;
               }
            }

            this.m_119(this.currentFloor);
            this.f_byte_23 = (byte)var1;
            this.f_bool_18 = var2;
         }
      }

      return var5;
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
      this.equippedWeaponType = 0;
      this.equippedArmorType = 0;
      if (this.currentFloor == 1) {
         this.m_024(6, 11);
      } else if (this.currentFloor == 51) {
         this.m_024(1, 11);
      } else if (this.currentFloor == 50) {
         this.m_024(6, 6);
      } else {
         int var1;
         if ((var1 = this.m_065(false)) >= 0) {
            this.m_031(this.entityPixelX[var1] >> 5, this.entityPixelY[var1] >> 5);
         }
      }

      int var2 = this.f_int_113;

      while (--var2 >= 0) {
         this.f_bool_arr_06[var2] = false;
      }

      this.f_int_89 = 0;
      this.goldAmount = 0;
      this.alchemyUpgradeCount = 0;
      this.itemStackSize = 0;
      this.f_byte_26 = 0;
      this.scaleEnemyStats(this.difficultyMultipliers[this.f_byte_26]);
   }

   private void scaleEnemyStats(int var1) {
      if (this.enemyAtkScaled == null) {
         this.enemyAtkScaled = new int[this.enemyBaseAtk.length];
      }

      int var2 = this.enemyAtkScaled.length;

      while (--var2 >= 0) {
         this.enemyAtkScaled[var2] = this.enemyBaseAtk[var2] * var1;
      }

      if (this.enemyDefScaled == null) {
         this.enemyDefScaled = new int[this.enemyBaseDef.length];
      }

      var2 = this.enemyDefScaled.length;

      while (--var2 >= 0) {
         this.enemyDefScaled[var2] = this.enemyBaseDef[var2] * var1;
      }

      if (this.enemyHpScaled == null) {
         this.enemyHpScaled = new int[this.enemyBaseHp.length];
      }

      var2 = this.enemyHpScaled.length;

      while (--var2 >= 0) {
         this.enemyHpScaled[var2] = this.enemyBaseHp[var2] * var1;
      }
   }

   private static int scaledByFloorTier(int var0, int var1) {
      var1--;
      var1 /= 10;
      return var0 * ++var1;
   }

   private static int alchemyPriceFor(int var0) {
      int var1 = 20;

      for (int var2 = 1; var2 < var0; var2++) {
         var1 += 20 * var2;
      }

      return var1;
   }

   private void m_072(int var1) {
      int var2 = this.f_Font_00.stringWidth(this.f_String_arr_05[var1]) + 80;
      this.f_byte_arr_11[this.f_int_83++] = (byte)var1;
      if (this.f_int_83 == 1) {
         this.f_int_85 = 32 + this.f_int_81;
         this.f_int_84 = var2;
      } else {
         this.f_int_85 = this.f_int_85 + this.f_int_81;
         if (this.f_int_84 < var2) {
            this.f_int_84 = var2;
         }
      }

      switch (var1) {
         case 0:
            this.f_bool_arr_05[this.f_int_83 - 1] = this.f_bool_29;
      }
   }

   private boolean m_073(int var1) {
      int var2 = 0;
      this.f_int_86 = 0;

      for (int var3 = 0; var3 < this.f_int_45; var3++) {
         if (!this.f_bool_arr_01[var3] && var1 == this.entityParam[var3]) {
            if ((var2 = this.entityType[var3]) == 5) {
               return false;
            }

            if (var2 == 4) {
               this.f_short_arr_03[this.f_int_86] = (short)var3;
               this.f_short_arr_01[this.f_int_86] = (short)(this.entityPixelX[var3] >> 5);
               this.f_short_arr_02[this.f_int_86] = (short)(this.entityPixelY[var3] >> 5);
               this.f_int_86++;
            }
         }
      }

      return this.f_int_86 > 0;
   }

   private boolean applyMerchantOffer(boolean var1) {
      short var2;
      byte var3 = (byte)((var2 = this.entityParam[this.f_int_87]) >>> 8);
      byte var7 = (byte)var2;
      int var4 = this.entityType[this.f_int_87] == 77 ? 0 : 1;
      boolean var5 = true;
      boolean var6 = false;
      if (var7 <= 0) {
         return false;
      }

      if (var1) {
         if ((var3 & 2) != 0) {
            label94:
            switch (var4) {
               case 0:
                  switch (var7) {
                     case 1:
                     case 15:
                     case 21:
                        this.gainGold(1000, this.playerPixelX, this.playerPixelY);
                        break label94;
                     case 2:
                        this.addItemToItemStack(13);
                        break label94;
                     case 6:
                        this.addItemToItemStack(19);
                     default:
                        break label94;
                  }
               case 1:
                  switch (var7) {
                     case 1:
                        this.playerAtk = this.playerAtk + this.playerAtk * 3 / 100;
                        this.playerDef = this.playerDef + this.playerDef * 3 / 100;
                     case 2:
                     default:
                        break;
                     case 3:
                        if (this.spendGold(50)) {
                           this.blueKeyCount++;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 4:
                        if (this.spendGold(50)) {
                           this.yellowKeyCount += 5;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 5:
                        if (this.spendGold(1000)) {
                           this.yellowKeyCount++;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 6:
                        if (this.spendGold(800)) {
                           this.redKeyCount++;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 7:
                        if (this.spendGold(200)) {
                           this.blueKeyCount++;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 8:
                        if (this.yellowKeyCount > 0) {
                           this.yellowKeyCount--;
                           this.goldAmount += 100;
                        } else {
                           this.m_015((byte)0, "没有黄钥匙", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 9:
                        if (this.spendGold(1000)) {
                           this.yellowKeyCount++;
                           this.blueKeyCount++;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 10:
                        if (this.spendGold(200)) {
                           this.yellowKeyCount += 3;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 11:
                        if (this.spendGold(2000)) {
                           this.blueKeyCount += 3;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 12:
                        if (this.spendGold(1000)) {
                           this.playerHp += 2000;
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                        break;
                     case 13:
                        if (this.spendGold(4000)) {
                           this.pickupItemType(18);
                        } else {
                           this.m_015((byte)0, "没有足够的金钱", (byte)0, (byte)0);
                           var5 = false;
                        }
                  }
            }

            if (var5) {
               if ((var3 & 4) == 0) {
                  this.entityParam[this.f_int_87] = (short)(this.entityParam[this.f_int_87] ^ 512);
                  if ((var3 & 1) != 0) {
                     this.m_075(var4, var7 - 1);
                     if (var4 == 0) {
                        this.m_015((byte)4, this.f_String_arr_06[var7 - 1], (byte)0, (byte)0);
                     } else {
                        this.m_015((byte)4, this.f_String_arr_09[var7 - 1], (byte)0, (byte)0);
                     }

                     var6 = true;
                  } else {
                     this.m_047(this.f_int_87);
                  }
               }
            } else {
               var6 = true;
            }
         } else if ((var3 & 1) != 0) {
            this.entityParam[this.f_int_87] = (short)(this.entityParam[this.f_int_87] ^ 256);
            if ((var3 & 4) == 0) {
               this.m_047(this.f_int_87);
            }
         }
      } else if ((var3 & 2) == 0 && (var3 & 1) != 0) {
         this.entityParam[this.f_int_87] = (short)(this.entityParam[this.f_int_87] ^ 256);
         if ((var3 & 4) == 0) {
            this.m_047(this.f_int_87);
         }
      }

      return var6;
   }

   private void m_075(int var1, int var2) {
      if (this.f_int_89 < this.f_int_88) {
         this.f_byte_arr_14[this.f_int_89] = (byte)var1;
         this.f_byte_arr_15[this.f_int_89] = (byte)var2;
         this.f_int_89++;
      }
   }

   private void m_076(int var1) {
      byte var2 = 0;
      byte var3 = 0;
      if (this.f_int_89 > 0) {
         if (var1 < 0) {
            this.f_int_90 = this.f_int_89 - 1;
         } else if (var1 >= this.f_int_89) {
            this.f_int_90 = 0;
         } else {
            this.f_int_90 = var1;
         }

         var2 = this.f_byte_arr_14[this.f_int_90];
         var3 = this.f_byte_arr_15[this.f_int_90];
         if (var2 == 0) {
            this.m_015((byte)5, this.f_String_arr_06[var3], (byte)0, (byte)3);
         } else {
            this.m_015((byte)5, this.f_String_arr_09[var3], (byte)0, (byte)3);
         }

         this.f_String_04 = "" + (this.f_int_90 + 1) + " / " + this.f_int_89;
      } else {
         this.m_015((byte)0, "没有记录", (byte)0, (byte)3);
         this.gameMode = 10;
      }
   }

   private boolean spendGold(int var1) {
      boolean var2 = true;
      if (this.goldAmount >= var1) {
         this.goldAmount -= var1;
      } else {
         var2 = false;
      }

      return var2;
   }

   private void gainGold(int var1, int var2, int var3) {
      this.goldAmount += var1;
      if (var1 > 0) {
         this.m_125((byte)4, var1, var2, var3);
      }
   }

   private void pickupItemType(int var1) {
      int var2 = 0;
      var2 = this.m_083(var1, 0, 12);
      switch (var1) {
         case 15:
            this.f_bool_12 = true;
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
         case 86:
            this.addItemToItemStack(var1);
            return;
         case 26:
            this.yellowKeyCount++;
            return;
         case 27:
            this.redKeyCount++;
            return;
         case 28:
            this.blueKeyCount++;
            return;
         case 29:
            if (this.currentFloor <= 10) {
               this.playerAtk++;
               return;
            }

            this.playerAtk = this.playerAtk + this.floorTier();
            return;
         case 30:
            if (this.currentFloor <= 10) {
               this.playerDef++;
               return;
            }

            this.playerDef = this.playerDef + this.floorTier();
            return;
         case 31:
            var2 = this.floorTier();
            this.applyHpDelta(50 * var2);
            return;
         case 32:
            var2 = this.floorTier();
            this.applyHpDelta(200 * var2);
         case 41:
         case 42:
         case 43:
         case 44:
         case 45:
         case 46:
         case 47:
         case 48:
         case 49:
         case 50:
         case 51:
         case 52:
         case 53:
         case 54:
         case 55:
         case 56:
         case 57:
         case 58:
         case 59:
         case 60:
         case 61:
         case 62:
         case 63:
         case 64:
         case 65:
         case 66:
         case 67:
         case 68:
         case 69:
         case 70:
         case 71:
         case 72:
         case 73:
         case 74:
         case 75:
         case 76:
         case 77:
         case 78:
         case 81:
         case 82:
         case 83:
         case 84:
         default:
            return;
         case 33:
         case 34:
         case 35:
         case 36:
         case 37:
         case 38:
         case 39:
         case 40:
         case 79:
         case 80:
            if (this.m_082(var2, false)) {
               this.f_byte_07 = (byte)var1;
               this.m_015((byte)1, this.equipDescriptions[var2], (byte)0, (byte)3);
            } else {
               this.m_015((byte)0, this.f_String_arr_00[5], (byte)0, (byte)3);
            }
      }
   }

   private int floorTier() {
      int var1;
      if ((var1 = (var1 = this.currentFloor - 1) / 10) < 0) {
         var1 = 0;
      }

      if (this.currentFloor > 50) {
         var1 = 0;
      }

      return var1 + 1;
   }

   private int m_081(int var1) {
      int var2 = -1;

      for (int var3 = 0; var3 < this.itemStackSize; var3++) {
         if (this.itemStackTypes[var3] == var1) {
            var2 = var3;
            break;
         }
      }

      return var2;
   }

   private boolean m_082(int var1, boolean var2) {
      boolean var3 = true;
      int var4 = 0;
      if (var1 < 6) {
         if ((var4 = this.m_083(this.equippedWeaponType, 0, 6)) >= var1 && !var2) {
            var3 = false;
         } else {
            this.playerAtk = this.playerAtk - this.equipTierBonuses[var4];
            this.playerAtk = this.playerAtk + this.equipTierBonuses[var1];
            this.equippedWeaponType = this.equipTierTypes[var1];
         }
      } else if ((var4 = this.m_083(this.equippedArmorType, 6, 12)) >= var1 && !var2) {
         var3 = false;
      } else {
         this.playerDef = this.playerDef - this.equipTierBonuses[var4];
         this.playerDef = this.playerDef + this.equipTierBonuses[var1];
         this.equippedArmorType = this.equipTierTypes[var1];
      }

      return var3;
   }

   private int m_083(int var1, int var2, int var3) {
      var3 = var3;

      while (--var3 >= var2) {
         if (this.equipTierTypes[var3] == var1) {
            return var3;
         }
      }

      return 0;
   }

   private static int itemTypeToStackIndex(int var0) {
      int var1 = 0;
      switch (var0) {
         case 85:
            var1 = 20;
            break;
         case 86:
            var1 = 21;
            break;
         default:
            var1 = var0 - 13;
      }

      return var1;
   }

   private void addItemToItemStack(int var1) {
      int var2 = this.m_081(var1);
      int var3 = itemTypeToStackIndex(var1);
      if (var2 < 0) {
         this.itemStackTypes[this.itemStackSize] = (byte)var1;
         this.itemStackUses[this.itemStackSize] = this.itemUseCounts[var3];
         this.itemStackSize++;
      } else {
         this.itemStackUses[var2] = (byte)(this.itemStackUses[var2] + this.itemUseCounts[var3]);
      }

      this.f_byte_07 = (byte)var1;
      this.m_015((byte)1, this.itemDescriptions[var3], (byte)0, (byte)3);
   }

   private boolean consumeKeyForDoor(byte var1) {
      boolean var2 = false;
      switch (var1) {
         case 26:
            if (this.yellowKeyCount > 0) {
               this.m_125((byte)1, 2, this.playerPixelX, this.playerPixelY);
               this.yellowKeyCount--;
               var2 = true;
            }
            break;
         case 27:
            if (this.redKeyCount > 0) {
               this.m_125((byte)1, 1, this.playerPixelX, this.playerPixelY);
               this.redKeyCount--;
               var2 = true;
            }
            break;
         case 28:
            if (this.blueKeyCount > 0) {
               this.m_125((byte)1, 0, this.playerPixelX, this.playerPixelY);
               this.blueKeyCount--;
               var2 = true;
            }
      }

      return var2;
   }

   private void useItemStack(int var1) {
      if (this.itemStackUses[var1] > 0) {
         if (this.activateItem(this.itemStackTypes[var1]) && --this.itemStackUses[var1] == 0) {
            
            for (int var4 = var1; var4 < this.itemStackSize - 1; var4++) {
               this.itemStackTypes[var4] = this.itemStackTypes[var4 + 1];
               this.itemStackUses[var4] = this.itemStackUses[var4 + 1];
            }

            if (this.itemStackSize > 0) {
               this.itemStackSize--;
            }

            return;
         }
      } else {
         this.activateItem(this.itemStackTypes[var1]);
      }
   }

   private boolean activateItem(byte var1) {
      boolean var2 = false;
      var2 = false;
      switch (var1) {
         case 13:
            this.gameMode = 5;
            this.m_000();
            break;
         case 14:
            this.gameMode = 12;
            this.m_000();
         case 15:
         case 23:
         case 24:
         case 25:
         default:
            break;
         case 16:
            if (!this.m_089((byte)10)) {
               this.m_015((byte)0, "你必须面对三昧真火再使用它。", (byte)0, (byte)0);
            } else {
               this.gameMode = 3;
               var2 = true;
            }
            break;
         case 17:
            if (!this.m_089((byte)11)) {
               this.m_015((byte)0, "你必须面对一堵墙使用", (byte)0, (byte)0);
            } else {
               this.gameMode = 3;
               var2 = true;
            }
            break;
         case 18:
            int var1c = this.f_int_45;

            while (--var1c >= 0) {
               if (this.entityType[var1c] == 11) {
                  this.m_047(var1c);
               }
            }

            this.gameMode = 3;
            var2 = true;
            break;
         case 19:
            int var2hp = (this.playerAtk + this.playerDef) * 74 / 10;
            this.applyHpDelta(var2hp);
            this.m_015((byte)0, "增加了" + var2hp + "血量", (byte)0, (byte)0);
            var2 = true;
            break;
         case 20:
            if (this.currentFloor == 40) {
               this.m_015((byte)0, "本层不能直接瞬移。", (byte)0, (byte)0);
            } else {
               a var8 = this;
               int var12 = this.mapCellsWide - 1 - var8.playerCellX;
               int var13 = var8.mapCellsHigh - 1 - var8.playerCellY;
               boolean var5;
               if (var5 = var8.interactWithCell(var12, var13)) {
                  var8.m_024(var12, var13);
                  var8.m_064((var8.f_int_58 - 32 >> 1) - var8.playerPixelX, (var8.f_int_59 - 32 >> 1) - var8.playerPixelY);
                  var8.applyStepCellEffects();
               }

               if (var5) {
                  this.gameMode = 3;
                  var2 = true;
               } else if (!this.f_bool_05) {
                  this.m_015((byte)0, "无法移动到该位置", (byte)0, (byte)0);
               }
            }
            break;
         case 21:
            var2 = this.changeFloor(this.currentFloor + 1, false, false);
            this.f_int_06 = 0;
            this.gameMode = 3;
            break;
         case 22:
            var2 = this.changeFloor(this.currentFloor - 1, true, false);
            this.f_int_06 = 0;
            this.gameMode = 3;
            break;
         case 85:
            var1c = this.f_int_45;

            while (--var1c >= 0) {
               if (this.entityType[var1c] == 1) {
                  this.m_047(var1c);
               }
            }

            this.gameMode = 3;
            var2 = true;
            break;
         case 86:
            a var6 = this;
            int var3 = this.playerPixelX >> 5;
            int var4 = var6.playerPixelY >> 5;
            var6.killAdjacentAt(var3, var4 - 1);
            var6.killAdjacentAt(var3, var4 + 1);
            var6.killAdjacentAt(var3 - 1, var4);
            var6.killAdjacentAt(var3 + 1, var4);
            boolean var10000 = true;
            this.gameMode = 3;
            var2 = true;
      }

      return var2;
   }

   private boolean m_089(byte var1) {
      int var2 = this.playerPixelX >> 5;
      int var3 = this.playerPixelY >> 5;
      int var4 = this.m_100(var2, var3 - 1, var1);
      int var5 = this.m_100(var2, var3 + 1, var1);
      int var6 = this.m_100(var2 - 1, var3, var1);
      int var7 = this.m_100(var2 + 1, var3, var1);
      var2 = 0;
      if (var4 >= 0) {
         this.m_047(var4);
         var2++;
      }

      if (var5 >= 0) {
         this.m_047(var5);
         var2++;
      }

      if (var6 >= 0) {
         this.m_047(var6);
         var2++;
      }

      if (var7 >= 0) {
         this.m_047(var7);
         var2++;
      }

      return var2 > 0;
   }

   private void killAdjacentAt(int var1, int var2) {
      try {
         byte var3 = this.f_byte_arr2_02[var2][var1];
         byte var4 = this.f_byte_arr_10[var3];
         int var5 = 0;
         int var6 = 0;
         var5 = 0;
         if (var4 > 0) {
            for (int var7 = 0; var7 < var4; var7++) {
               var5 = this.f_byte_arr2_03[var3][var7] - 1;
               if ((var6 = this.entityType[var5]) == 5) {
                  short var11 = this.entityParam[var5];
                  this.m_050(var1, var2, var7);
                  if (this.m_073(var11)) {
                     this.f_byte_11 = 4;
                  }
               } else if (var6 < 67 && this.f_byte_arr_03[var6] == 8 && this.f_byte_arr_04[var5] != 1) {
                  this.m_047(var5);
                  if (this.m_081(25) >= 0) {
                     this.gainGold(this.enemyBaseGold[var6 - 41] << 1, 0, 0);
                  } else {
                     this.gainGold(this.enemyBaseGold[var6 - 41], 0, 0);
                  }
               }
            }
         }
      } catch (Exception var8) {
         var8.printStackTrace();
      }
   }

   private void m_091() {
      this.f_int_arr_22 = null;
      this.f_int_arr_22 = new int[128];
      this.f_byte_arr_20 = null;
      this.f_byte_arr_20 = new byte[128];
      this.f_int_108 = 208;
      this.f_int_105 = 0;

      label56:
      for (int var1 = 0; var1 < this.f_int_45; var1++) {
         if (!this.f_bool_arr_01[var1] && this.f_byte_arr_03[this.entityType[var1]] == 8) {
            int var3 = this.entityType[var1];
            a var2 = this;
            int var4 = this.f_int_105;

            while (--var4 >= 0) {
               if (var3 == var2.f_byte_arr_20[var4]) {
                  continue label56;
               }
            }

            var4 = var2.enemyHpScaled[var3 - 41];
            int var5 = var2.enemyAtkScaled[var3 - 41];
            int var6 = var2.enemyDefScaled[var3 - 41];
            var2.f_int_arr_22[var2.f_int_105] = predictBattleHpLoss(var2.effectiveAttackVsType(var3), var2.playerDef, var4, var5, var6, true);
            var2.f_byte_arr_20[var2.f_int_105] = (byte)var3;
            var2.f_int_105++;
         }
      }

      a var7 = this;
      byte var8 = 0;
      int var11 = 0;

      for (int var13 = 0; var13 < var7.f_int_105; var13++) {
         int var14 = var7.f_int_105;

         while (--var14 > var13) {
            if (var7.f_int_arr_22[var14] >= 0 && var7.f_int_arr_22[var14] < var7.f_int_arr_22[var14 - 1]) {
               var8 = var7.f_byte_arr_20[var14];
               var7.f_byte_arr_20[var14] = var7.f_byte_arr_20[var14 - 1];
               var7.f_byte_arr_20[var14 - 1] = var8;
               var11 = var7.f_int_arr_22[var14];
               var7.f_int_arr_22[var14] = var7.f_int_arr_22[var14 - 1];
               var7.f_int_arr_22[var14 - 1] = var11;
            }
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

   private void m_092(int var1) {
      int var2;
      if ((var2 = this.f_int_105 - this.f_int_106) < 0) {
         var2 = 0;
      }

      if (var1 >= 0 && var1 <= var2) {
         this.f_int_109 = var1;
         this.f_int_110 = this.f_int_109 + this.f_int_106;
         if (this.f_int_110 > this.f_int_105) {
            this.f_int_110 = this.f_int_105;
         }
      }
   }

   private void m_093(int var1, int var2, byte var3) {
      this.f_Graphics_00.fillTriangle(var1, var2 - 6, var1, var2 + 6, var1 - 6, var2);
   }

   private boolean loadLevelScript(int var1) {
      this.f_bool_27 = false;
      this.gameMode = 11;
      this.currentScriptIndex = var1;
      this.f_int_112 = 0;
      this.m_104(0);
      this.f_byte_16 = 0;
      this.f_String_05 = this.levelScriptLines[this.currentScriptIndex];
      this.f_String_02 = null;
      this.scriptCursor = 0;
      switch (var1) {
         case 31:
            this.m_119(this.currentFloor);
            this.m_122(24);
            var1 = this.f_int_45;

            while (--var1 >= 0) {
               if (this.entityType[var1] == 4) {
                  this.f_bool_arr_01[var1] = true;
               }
            }

            this.m_119(24);
            this.m_059();
            this.m_122(this.currentFloor);
         default:
            return false;
      }
   }

   private boolean tryRunScene(int var1, boolean var2) {
      if (var1 >= this.f_int_113) {
         return false;
      }

      boolean var3 = !this.f_bool_arr_06[var1];
      int var4 = 0;
      switch (var1) {
         case 0:
            if (var3) {
               this.f_bool_arr_06[19] = true;
            }
            break;
         case 1:
            var3 &= this.f_bool_arr_06[0] & !var2;
            break;
         case 2:
            var3 &= this.f_bool_arr_06[1] & !var2;
            break;
         case 3:
            var3 &= var2;
            break;
         case 4:
            var3 &= this.f_bool_arr_06[3] & var2;
         case 5:
         case 6:
         case 10:
         case 13:
         case 14:
         case 17:
         case 20:
         case 21:
         case 22:
         case 23:
         case 28:
         case 29:
         case 35:
         case 36:
         case 37:
         case 38:
         case 39:
         case 40:
         case 51:
         case 54:
         case 57:
         case 58:
         case 59:
         case 60:
         case 61:
         case 62:
         case 63:
         case 64:
         case 65:
         default:
            break;
         case 7:
            if (var3 &= var2 && !this.m_102(23, 12)) {
               this.f_bool_arr_06[32] = true;
            }
            break;
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
         case 56:
            var3 &= var2;
            break;
         case 15:
            if (this.m_102(this.currentFloor, 5)) {
               var3 = false;
            } else {
               var3 &= var2;
            }
            break;
         case 16:
            var3 &= this.f_byte_26 == 0;
            break;
         case 18:
            if ((var3 &= var2)
               && (
                  this.m_100(2, 2, 1) < 0
                     || this.m_100(2, 4, 1) < 0
                     || this.m_100(2, 6, 1) < 0
                     || this.m_100(6, 2, 1) < 0
                     || this.m_100(4, 6, 1) < 0
                     || this.m_100(6, 6, 1) < 0
                     || this.m_100(4, 2, 1) >= 0
                     || this.m_100(6, 4, 1) >= 0
               )) {
               var3 = false;
               this.f_bool_arr_06[18] = true;
            }
            break;
         case 24:
            if (var3 &= !var2) {
               var3 = true & 0 < this.playerHp;
            }
            break;
         case 25:
            var3 &= var2 & this.f_bool_arr_06[13] & this.m_100(2, 2, 60) < 0 & this.m_100(10, 2, 60) < 0;
            break;
         case 26:
            if (this.m_100(11, 4, 84) < 0) {
               var3 = false;
            }
            break;
         case 27:
            var3 &= var2 & this.f_bool_arr_06[0];
            break;
         case 31:
            boolean var5;
            var3 = (var5 = var3 & var2) & this.m_081(16) >= 0;
            break;
         case 32:
            if (this.f_bool_arr_06[7]) {
               var3 = false;
            } else {
               var3 &= !var2 & this.m_102(23, 12);
            }
            break;
         case 33:
            var3 &= this.f_bool_arr_06[7];
            break;
         case 34:
            if (var3 &= var2 & !this.f_bool_arr_06[10]) {
               var4 = this.predictHpLossVsType(69, true);
               var3 &= var4 < 0 || var4 >= this.playerHp;
            }
            break;
         case 46:
            var3 &= var2 & !this.f_bool_arr_06[47];
            break;
         case 47:
            var3 &= var2 & !this.f_bool_arr_06[46];
            break;
         case 66:
            var3 = false;
            break;
         case 67:
            var3 &= this.f_byte_26 > 0;
      }

      return var3;
   }

   private void m_096(int var1) {
      byte var2 = 0;
      if (var1 <= this.f_int_121) {
         this.f_int_112 = var1;
         if (this.f_int_112 < 0) {
            this.f_byte_17 = -1;
         } else {
            this.f_byte_17 = this.dialogueSpeakerType[this.f_int_112];
         }

         this.f_int_119 = this.m_104(this.f_byte_17);
         var2 = this.f_byte_17;
         if (this.f_byte_17 < 0) {
            this.f_String_03 = this.f_String_02 = "?: \\cF8F8F8" + this.dialogueTexts[this.f_int_112];
            this.f_int_33 = 11;
         } else {
            this.f_String_03 = this.f_String_02 = this.objectTypeNames[var2] + ": \\cF8F8F8" + this.dialogueTexts[this.f_int_112];
            this.f_int_33 = this.objectTypeNames[var2].length() + 10;
         }

         this.m_018(this.f_String_02, 129, (this.f_int_34 << 1) + 12, this.f_int_33);
      } else {
         this.f_byte_16 = 0;
         this.f_int_33 = 0;
      }
   }

   private boolean m_097() {
      return this.f_int_123 == this.f_int_125 && this.f_int_124 == this.f_int_126;
   }

   private void executeScriptInstruction(String var1, int var2) {
      String var3 = null;
      var3 = null;
      var3 = null;
      int var4 = 0;
      int var5 = 0;
      int var6 = 0;
      if (var2 + 3 <= var1.length()) {
         var3 = var1.substring(var2, var2 + 3);
         var2 += 3;
         if (var3.equals("TAK")) {
            this.f_int_120 = this.parseScriptInt(var1, var2 + 1, "_");
            var2 = this.f_int_122;
            this.f_int_121 = this.parseScriptInt(var1, var2 + 1, " ");
            this.scriptCursor = this.f_int_122 + 1;
            this.f_byte_16 = 1;
            this.m_096(this.f_int_120);
         } else if (var3.equals("MOV")) {
            int var55 = this.parseScriptInt(var1, var2 + 1, "_");
            var2 = this.f_int_122;
            if (var55 > 0) {
               var5 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               var6 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
            }

            this.f_int_114 = this.parseScriptInt(var1, var2 + 1, "_");
            var2 = this.f_int_122;
            this.f_int_115 = this.parseScriptInt(var1, var2 + 1, " ");
            this.scriptCursor = this.f_int_122 + 1;
            if (var55 > 0) {
               this.f_int_127 = this.m_100(var5, var6, var55);
               if (this.f_int_127 >= 0) {
                  this.f_byte_18 = 0;
                  a var19 = this;
                  if (this.entityType[var19.f_int_127] == 72) {
                     var19.f_int_arr_12[var19.f_int_127] = 1;
                  }

                  var19.f_int_129 = var19.entityPixelX[var19.f_int_127] >> 5;
                  var19.f_int_130 = var19.entityPixelY[var19.f_int_127] >> 5;
                  var19.m_133(var19.f_int_129, var19.f_int_130, var19.f_int_114, var19.f_int_115);
                  this.f_byte_16 = 2;
               } else {
                  this.executeScriptInstruction(this.f_String_05, this.scriptCursor);
               }
            } else if (this.m_133(this.playerCellX, this.playerCellY, this.f_int_114, this.f_int_115)) {
               this.f_bool_27 = true;
               this.f_byte_16 = 3;
               this.f_byte_11 = 0;
               this.m_104(0);
            }
         } else if (var3.equals("GUT")) {
            var4 = this.parseScriptInt(var1, var2 + 1, " ");
            this.f_bool_arr_06[this.currentScriptIndex] = true;
            this.loadLevelScript(var4);
         } else if (var3.equals("DES")) {
            int var52 = this.parseScriptInt(var1, var2 + 1, "_");
            var2 = this.f_int_122;
            if (var52 < 0) {
               this.parseScriptInt(var1, var2 + 1, " ");
               var4 = -var52;
               a var18 = this;
               int var53 = this.f_int_45;

               while (--var53 >= 0) {
                  if (var18.entityType[var53] == var4) {
                     var18.m_047(var53);
                  }
               }
            } else {
               var5 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               var6 = this.parseScriptInt(var1, var2 + 1, " ");
               int var54;
               if ((var54 = this.m_100(var5, var6, var52)) >= 0) {
                  this.m_047(var54);
               }
            }

            this.scriptCursor = this.f_int_122 + 1;
         } else if (var3.equals("SWD")) {
            a var17 = this;
            int var51 = this.f_int_45;

            while (--var51 >= 0) {
               if (!var17.f_bool_arr_01[var51] && (var4 = var17.entityType[var51]) == 81) {
                  var17.f_bool_arr_00[var51] = true;
                  var17.entityType[var51] = 4;
                  var17.m_044(4, var51);
                  var17.f_byte_arr_04[var51] = 2;
                  var17.f_int_arr_12[var51] = 8;
               }
            }

            this.scriptCursor = var2 + 1;
         } else if (!var3.equals("MVS")) {
            if (var3.equals("LAY")) {
               this.m_119(this.currentFloor);
               this.f_byte_23 = (byte)this.parseScriptInt(var1, var2 + 1, " ");
               this.scriptCursor = this.f_int_122 + 1;
               this.f_bool_16 = true;
               this.f_String_02 = null;
               this.f_byte_16 = 5;
            } else if (var3.equals("ROS")) {
               var4 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               switch (var4) {
                  case 1:
                     this.playerPixelX = this.parseScriptInt(var1, var2 + 1, "_") << 5;
                     var2 = this.f_int_122;
                     this.playerPixelY = this.parseScriptInt(var1, var2 + 1, " ") << 5;
                     this.m_104(0);
                     break;
                  case 2:
                     this.m_082(this.parseScriptInt(var1, var2 + 1, " "), true);
                     break;
                  case 3:
                     this.playerHp = this.parseScriptInt(var1, var2 + 1, " ");
                     break;
                  case 4:
                     this.f_byte_12 = (byte)this.parseScriptInt(var1, var2 + 1, " ");
                     break;
                  case 5:
                     this.playerAtk = this.parseScriptInt(var1, var2 + 1, " ");
                     break;
                  case 6:
                     this.playerDef = this.parseScriptInt(var1, var2 + 1, " ");
               }

               this.scriptCursor = this.f_int_122 + 1;
            } else if (var3.equals("CES")) {
               int var47 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               var5 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               var6 = this.parseScriptInt(var1, var2 + 1, " ");
               var4 = this.m_048(var47, var5 << 5, var6 << 5, 0);
               this.m_046(var4);
               var4 = var4;
               a var14 = this;
               this.f_int_123 = (var14.f_int_58 - 32 >> 1) - var14.f_int_56;
               var14.f_int_124 = (var14.f_int_59 - 32 >> 1) - var14.f_int_57;
               if (var4 >= 0) {
                  var14.f_int_125 = var14.entityPixelX[var4];
                  var14.f_int_126 = var14.entityPixelY[var4];
               }

               var4 = var47;
               byte var10001;
               switch (this.f_byte_arr_03[var4]) {
                  case 1:
                  case 8:
                  case 16:
                     var10001 = 5;
                     break;
                  case 2:
                  case 4:
                     var10001 = 8;
                     break;
                  default:
                     var10001 = 0;
               }

               this.f_int_118 = var10001;
               this.scriptCursor = this.f_int_122 + 1;
            } else if (var3.equals("GIN")) {
               var4 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               int var48 = this.parseScriptInt(var1, var2 + 1, " ");
               switch (var4) {
                  case 0:
                     this.pickupItemType(var48);
                     break;
                  case 1:
                     this.gainGold(var48, this.playerPixelX, this.playerPixelY);
               }

               this.scriptCursor = this.f_int_122 + 1;
               this.executeScriptInstruction(this.f_String_05, this.scriptCursor);
            } else if (var3.equals("ADD")) {
               var4 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               int var49 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               var5 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               var6 = this.parseScriptInt(var1, var2 + 1, " ");
               int var10003 = var5 << 5;
               int var10004 = var6 << 5;
               boolean var88 = false;
               var6 = var10004;
               var5 = var10003;
               int var50 = var49;
               var4 = var4;
               a var15 = this;
               if (this.currentFloor != var4) {
                  var15.m_119(var15.currentFloor);
                  var15.m_122(var4);
               }

               var15.m_048(var50, var5, var6, 0);
               var15.m_119(var4);
               if (var15.currentFloor != var4) {
                  var15.m_059();
                  var15.m_122(var15.currentFloor);
               }

               this.scriptCursor = this.f_int_122 + 1;
            } else if (var3.equals("GLV")) {
               this.parseScriptInt(var1, var2 + 1, " ");
               this.scriptCursor = this.f_int_122 + 1;
            } else if (var3.equals("RES")) {
               this.parseScriptInt(var1, var2 + 1, " ");
               if (this.f_byte_26 == 0) {
                  a var16 = this;
                  this.yellowKeyCount = var16.redKeyCount = var16.blueKeyCount = 0;
                  var16.itemStackSize = 0;
                  var16.goldAmount = 4;
                  var16.alchemyUpgradeCount = 0;
                  var16.minFloorReached = 1;
                  var16.maxFloorReached = 3;
                  var16.playerAtk = 10;
                  var16.playerDef = 10;
                  var16.playerHp = 400;
                  var16.f_int_89 = 0;
               }

               this.scriptCursor = this.f_int_122 + 1;
            } else if (var3.equals("SEE")) {
               this.f_int_114 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               this.f_int_115 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               this.f_int_120 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               this.f_int_121 = this.parseScriptInt(var1, var2 + 1, "_");
               var2 = this.f_int_122;
               this.f_int_117 = this.parseScriptInt(var1, var2 + 1, " ");
               this.scriptCursor = this.f_int_122 + 1;
               this.m_096(this.f_int_120);
               switch (this.f_int_117) {
                  case 1:
                     if (this.m_133(this.playerCellX, this.playerCellY, this.f_int_114, this.f_int_115)) {
                        this.f_bool_26 = true;
                        this.f_byte_11 = 0;
                     }
                  case 0:
                  default:
                     this.f_byte_16 = 6;
                     this.m_103(this.f_int_114, this.f_int_115);
               }
            } else if (var3.equals("END")) {
               this.parseScriptInt(var1, var2 + 1, " ");
               this.gameMode = 20;
               this.m_000();
               this.scriptCursor = this.f_int_122 + 1;
            } else if (var3.equals("SMS")) {
               this.parseScriptInt(var1, var2 + 1, " ");
               this.scriptCursor = this.f_int_122 + 1;
            }
         } else {
            a var13 = this;
            this.f_short_arr_04 = null;
            var13.f_short_arr_04 = new short[32];
            int var56 = 0;
            boolean var42 = false;
            int var70 = 0;
            var6 = var13.f_int_45;

            while (--var6 >= 0) {
               if ((var56 = var13.entityType[var6]) == 82) {
                  int var43 = var13.entityParam[var6] & 255;

                  for (int var7 = 0; var7 < 32; var7 += 2) {
                     if ((var70 = var13.f_short_arr_04[var7] - 1) <= 0) {
                        var13.f_short_arr_04[var7] = (short)(var6 + 1);
                        break;
                     }

                     if (var43 == (var13.entityParam[var70] & 255)) {
                        var13.f_short_arr_04[var7 + 1] = (short)(var6 + 1);
                        break;
                     }
                  }
               }
            }

            for (int var11 = 0; var11 < 32; var11 += 2) {
               var70 = var13.f_short_arr_04[var11];
               int var9 = var13.f_short_arr_04[var11 + 1];
               if (var70 <= 0) {
                  break;
               }

               if (var9 > 0) {
                  var70--;
                  var9--;
                  int var86 = var13.entityPixelX[var70] >> 5;
                  var6 = var13.entityPixelY[var70] >> 5;
                  byte var44 = var13.f_byte_arr2_02[var6][var86];
                  byte var8 = var13.f_byte_arr_10[var44];

                  for (int var12 = 0; var12 < var8; var12++) {
                     int var10 = var13.f_byte_arr2_03[var44][var12] - 1;
                     var56 = var13.entityType[var10];
                     if (var13.f_byte_arr_03[var56] == 8) {
                        var13.m_049(var13.entityPixelX[var9] >> 5, var13.entityPixelY[var9] >> 5, 82);
                        var13.m_049(var86, var6, 82);
                        var13.m_107(var10, var13.entityPixelX[var9] >> 5, var13.entityPixelY[var9] >> 5);
                     }
                  }

                  int var45 = var70;
                  var70 = var9;
                  var9 = var45;
                  var86 = var13.entityPixelX[var70] >> 5;
                  var6 = var13.entityPixelY[var70] >> 5;
                  byte var46 = var13.f_byte_arr2_02[var6][var86];
                  var8 = var13.f_byte_arr_10[var46];

                  for (int var93 = 0; var93 < var8; var93++) {
                     int var92 = var13.f_byte_arr2_03[var46][var93] - 1;
                     var56 = var13.entityType[var92];
                     if (var13.f_byte_arr_03[var56] == 8) {
                        var13.m_049(var13.entityPixelX[var9] >> 5, var13.entityPixelY[var9] >> 5, 82);
                        var13.m_049(var86, var6, 82);
                        var13.m_107(var92, var13.entityPixelX[var9] >> 5, var13.entityPixelY[var9] >> 5);
                     }
                  }
               }
            }

            this.scriptCursor = var2 + 1;
         }

         if (this.scriptCursor < this.f_String_05.length()) {
            return;
         }
      } else {
         this.f_byte_16 = 4;
         if (this.currentScriptIndex != 32) {
            this.f_bool_arr_06[this.currentScriptIndex] = true;
         }

         this.m_104(0);
      }
   }

   private int parseScriptInt(String var1, int var2, String var3) {
      int var4 = 0;
      var4 = var1.indexOf(var3, var2);
      var1 = var1.substring(var2, var4);
      this.f_int_122 = var4;
      return Integer.parseInt(var1);
   }

   private int m_100(int var1, int var2, int var3) {
      byte var6 = this.f_byte_arr2_02[var2][var1];
      byte var7 = this.f_byte_arr_10[var6];
      int var4 = 0;
      if (var7 > 0) {
         for (int var5 = 0; var5 < var7; var5++) {
            var4 = this.f_byte_arr2_03[var6][var5] - 1;
            if (this.entityType[var4] == var3 && this.f_byte_arr_04[var4] != 1) {
               return var4;
            }
         }
      }

      return -1;
   }

   private void m_101(int var1, int var2, int var3) {
      var2--;
      if (this.isCellWalkable(var1, var2) && !this.m_062(var1, var2) && (var1 != this.playerCellX || var2 != this.playerCellY)) {
         this.m_051(var3);
         var3 = this.m_048(60, var1 << 5, var2 << 5, 0);
         this.m_046(var3);
      }
   }

   private boolean m_102(int var1, int var2) {
      boolean var3 = false;
      if (this.currentFloor != var1) {
         this.m_119(this.currentFloor);
         this.m_122(var1);
      }

      for (int var4 = 0; var4 < this.f_int_45; var4++) {
         if (!this.f_bool_arr_01[var4] && this.entityType[var4] == var2) {
            var3 = true;
            break;
         }
      }

      if (this.currentFloor != var1) {
         this.m_059();
         this.m_122(this.currentFloor);
      }

      return var3;
   }

   private void m_103(int var1, int var2) {
      this.f_int_123 = (this.f_int_58 - 32 >> 1) - this.f_int_56;
      this.f_int_124 = (this.f_int_59 - 32 >> 1) - this.f_int_57;
      this.f_int_125 = var1 << 5;
      this.f_int_126 = var2 << 5;
   }

   private int m_104(int var1) {
      this.f_int_123 = (this.f_int_58 - 32 >> 1) - this.f_int_56;
      this.f_int_124 = (this.f_int_59 - 32 >> 1) - this.f_int_57;
      if (var1 != 0 && var1 != 87) {
         int var3 = var1;
         a var2 = this;
         int var4 = this.f_int_45;

         int var10000;
         while (true) {
            if (--var4 >= 0) {
               if (var2.entityType[var4] != var3) {
                  continue;
               }

               var10000 = var4;
               break;
            }

            var10000 = -1;
            break;
         }

         int var5 = var10000;
         if (var10000 >= 0) {
            this.f_int_125 = this.entityPixelX[var5];
            this.f_int_126 = this.entityPixelY[var5];
         }

         if (var1 == 69) {
            this.f_int_125 += 32;
         }

         return var5;
      } else {
         this.f_int_125 = this.playerPixelX;
         this.f_int_126 = this.playerPixelY;
         return -1;
      }
   }

   private void m_105() {
      if (this.f_int_123 < this.f_int_125) {
         this.f_int_123 = this.f_int_123 + (this.f_int_125 - this.f_int_123 >> 2) + 2;
         if (this.f_int_123 > this.f_int_125) {
            this.f_int_123 = this.f_int_125;
         }
      } else if (this.f_int_123 > this.f_int_125) {
         this.f_int_123 = this.f_int_123 + ((this.f_int_125 - this.f_int_123 >> 2) - 2);
         if (this.f_int_123 < this.f_int_125) {
            this.f_int_123 = this.f_int_125;
         }
      }

      if (this.f_int_124 < this.f_int_126) {
         this.f_int_124 = this.f_int_124 + (this.f_int_126 - this.f_int_124 >> 2) + 2;
         if (this.f_int_124 > this.f_int_126) {
            this.f_int_124 = this.f_int_126;
         }
      } else if (this.f_int_124 > this.f_int_126) {
         this.f_int_124 = this.f_int_124 + ((this.f_int_126 - this.f_int_124 >> 2) - 2);
         if (this.f_int_124 < this.f_int_126) {
            this.f_int_124 = this.f_int_126;
         }
      }

      this.m_064((this.f_int_58 - 32 >> 1) - this.f_int_123, (this.f_int_59 - 32 >> 1) - this.f_int_124);
   }

   private void m_106() {
      switch (this.f_byte_18) {
         case 0:
            if (this.f_int_148 > 0) {
               this.f_byte_19 = this.f_byte_arr_42[--this.f_int_148];
               a var10 = this;
               int var11 = 0;
               int var14 = 0;
               int var18 = 0;
               boolean var5 = false;
               var11 = var10.entityPixelX[var10.f_int_127] >> 5;
               var14 = var10.entityPixelY[var10.f_int_127] >> 5;
               switch (var10.f_byte_19) {
                  case 0:
                     var14++;
                     break;
                  case 1:
                     var14--;
                     break;
                  case 2:
                     var11++;
                     break;
                  case 3:
                     var11--;
               }

               var10.f_byte_18 = 1;
               if (var10.entityType[var10.f_int_127] != 72 && var11 == var10.playerCellX && var14 == var10.playerCellY) {
                  var18 = var10.entityType[var10.f_int_127];
                  int var6;
                  if ((var6 = var10.predictHpLossVsType(var18, false)) >= 0 && var6 < var10.playerHp) {
                     var10.f_int_150 = var10.enemyHpScaled[var18 - 41];
                     var10.f_byte_11 = 5;
                     var10.f_bool_14 = false;
                     var10.f_int_44 = var10.f_int_127;
                     var10.f_byte_arr_04[var10.f_int_127] = 5;
                     var10.f_byte_18 = 5;
                     var10.f_bool_25 = false;
                     int var7;
                     if ((var7 = var10.enemyAtkScaled[var18 - 41] - var10.playerDef) > 0) {
                        var10.playerHp -= var7;
                     }
                  } else {
                     var10.f_byte_11 = 1;
                     var10.f_byte_12 = var10.f_byte_19;
                  }
               }

               byte var13 = var10.f_byte_arr2_02[var14][var11];
               if (var10.f_byte_arr_10[var13] > 0) {
                  int var8 = var10.f_byte_arr_10[var13];

                  while (--var8 >= 0) {
                     var14 = var10.f_byte_arr2_03[var13][var8] - 1;
                     var18 = var10.entityType[var14];
                     switch (var10.f_byte_arr_03[var18]) {
                        case 1:
                           switch (var18) {
                              case 11:
                                 var10.m_047(var14);
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
         case 1:
            switch (this.f_byte_19) {
               case 0:
                  this.entityPixelY[this.f_int_127] = this.entityPixelY[this.f_int_127] + 8;
                  break;
               case 1:
                  this.entityPixelY[this.f_int_127] = this.entityPixelY[this.f_int_127] - 8;
                  break;
               case 2:
                  this.entityPixelX[this.f_int_127] = this.entityPixelX[this.f_int_127] + 8;
                  break;
               case 3:
                  this.entityPixelX[this.f_int_127] = this.entityPixelX[this.f_int_127] - 8;
            }

            this.f_int_125 = this.entityPixelX[this.f_int_127];
            this.f_int_126 = this.entityPixelY[this.f_int_127];
            this.f_int_128 += 8;
            int var1;
            if ((var1 = this.f_int_arr_12[this.f_int_127]) > 0) {
               if (this.f_int_arr_10[this.f_int_127] < this.f_int_arr2_02[var1].length - 1) {
                  this.f_int_arr_10[this.f_int_127]++;
               } else {
                  this.f_int_arr_10[this.f_int_127] = 0;
               }
            }

            if (this.f_int_128 == 32) {
               this.f_int_128 = 0;
               this.f_byte_18 = 0;
               a var9 = this;
               int var2 = this.entityPixelX[var9.f_int_127] >> 5;
               int var3 = var9.entityPixelY[var9.f_int_127] >> 5;
               int var4 = 0;
               if (var2 == var9.playerCellX && var3 == var9.playerCellY && var9.entityType[var9.f_int_127] != 72) {
                  if ((var4 = var9.predictHpLossVsType(var9.entityType[var9.f_int_127], false)) >= 0 && var4 < var9.playerHp) {
                     var9.playerHp -= var4;
                     var9.m_108(var9.f_int_127, var9.f_int_129, var9.f_int_130);
                     var9.m_049(var2, var3, var9.entityType[var9.f_int_127]);
                  } else {
                     var9.f_bool_27 = false;
                     var9.gameMode = 3;
                     var9.f_byte_11 = 1;
                     var9.f_byte_12 = var9.f_byte_19;
                  }
               }

               this.m_106();
            }
            break;
         case 5:
            this.tickBattle(this.f_bool_14);
            if (!this.f_bool_23 && (this.f_bool_arr_01[this.f_int_127] || this.f_byte_arr_04[this.f_int_127] == 1)) {
               this.f_byte_16 = 0;
               this.f_byte_18 = 0;
               this.f_int_127 = -1;
               return;
            }
      }
   }

   private void m_107(int var1, int var2, int var3) {
      int var4 = this.entityPixelX[var1];
      int var5 = this.entityPixelY[var1];
      short var6 = this.entityParam[var1];
      this.m_048(this.entityType[var1], var2 << 5, var3 << 5, var6);
      this.m_049(var4 >> 5, var5 >> 5, this.entityType[var1]);
   }

   private void m_108(int var1, int var2, int var3) {
      byte var6 = this.f_byte_arr2_02[var3][var2];
      byte var8 = this.f_byte_arr_10[var6];

      for (int var4 = 0; var4 < var8; var4++) {
         if (this.f_byte_arr2_03[var6][var4] - 1 == var1) {
            for (int var5 = var4; var5 < var8 - 1; var5++) {
               this.f_byte_arr2_03[var6][var5] = this.f_byte_arr2_03[var6][var5 + 1];
            }

            this.f_byte_arr_10[var6]--;
            break;
         }
      }

      int var9 = this.entityPixelX[var1] >> 5;
      int var10 = this.entityPixelY[var1] >> 5;
      if ((var6 = this.f_byte_arr2_02[var10][var9]) == 0) {
         this.f_byte_arr2_02[var10][var9] = ++this.f_byte_15;
         var6 = this.f_byte_15;
      }

      this.f_byte_arr2_03[var6][this.f_byte_arr_10[var6]] = (byte)(var1 + 1);
      this.f_byte_arr_10[var6]++;
   }

   private void spawnBossEvent(int var1) {
      byte[] var5;
      int var2 = (var5 = this.bossEventSpawns[var1]).length;
      int var3 = 0;

      for (int var4 = 0; var4 < var2; var4 += 4) {
         var3 = this.m_048(var5[var4], var5[var4 + 1] << 5, var5[var4 + 2] << 5, var5[var4 + 3]);
         this.m_046(var3);
      }
   }

   private void m_110() {
      String var1 = "SKY_WAR";
      boolean var2 = false;

      try {
         this.f_RecordStore_00 = RecordStore.openRecordStore(var1, true);
         if (this.f_RecordStore_00.getNumRecords() == 0) {
            var2 = true;
         }

         this.f_ByteArrayOutputStream_00 = new ByteArrayOutputStream();
         this.f_DataOutputStream_00 = new DataOutputStream(this.f_ByteArrayOutputStream_00);

         for (int var4 = 0; var4 < 4; var4++) {
            this.f_DataOutputStream_00.writeBoolean(this.f_bool_arr_05[var4]);
         }

         this.f_DataOutputStream_00.writeByte(this.f_int_arr_34.length);

         for (int var5 = 0; var5 < this.f_int_arr_34.length; var5++) {
            this.f_DataOutputStream_00.writeInt(this.f_int_arr_35[var5]);
         }

         this.f_DataOutputStream_00.writeInt(this.f_int_152);
         this.f_DataOutputStream_00.writeInt(this.f_int_153);
         this.f_DataOutputStream_00.writeInt(this.f_int_154);
         if (!var2) {
            this.f_RecordStore_00.setRecord(1, this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
            return;
         }

         this.f_RecordStore_00.addRecord(this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
      } catch (Exception var3) {
      }
   }

   private void m_111() {
      String var1 = "MOT_IF";
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
         this.f_RecordStore_00 = RecordStore.openRecordStore(var1, false);
         this.f_RecordEnumeration_00 = this.f_RecordStore_00.enumerateRecords(null, null, false);
         int var6 = this.f_RecordEnumeration_00.nextRecordId();
         this.f_byte_arr_27 = this.f_RecordStore_00.getRecord(var6);
         this.f_DataInputStream_00 = new DataInputStream(new ByteArrayInputStream(this.f_byte_arr_27));

         for (int var7 = 0; var7 < 6; var7++) {
            this.f_bool_arr_07[var7] = this.f_DataInputStream_00.readByte() != 0;
            if (this.f_bool_arr_07[var7]) {
               this.f_byte_arr_24[var7] = this.f_DataInputStream_00.readByte();
               this.f_byte_arr_25[var7] = this.f_DataInputStream_00.readByte();
               this.f_byte_arr_26[var7] = this.f_DataInputStream_00.readByte();
               this.f_int_arr_23[var7] = this.f_DataInputStream_00.readInt();
               this.f_int_arr_24[var7] = this.f_DataInputStream_00.readInt();
               this.f_int_arr_25[var7] = this.f_DataInputStream_00.readInt();
               this.f_int_arr_26[var7] = this.f_DataInputStream_00.readInt();
               this.f_int_arr_27[var7] = this.f_DataInputStream_00.readInt();
               this.f_int_arr_28[var7] = this.f_DataInputStream_00.readInt();
               this.f_int_arr_29[var7] = this.f_DataInputStream_00.readInt();
            }
         }

         return;
      } catch (Exception var4) {
         this.m_112(-1);
      } finally {
         this.m_117();
      }
   }

   private void m_112(int var1) {
      if (!this.f_bool_15) {
         this.m_111();
      }

      if (var1 >= 0 && var1 < 6) {
         this.f_bool_arr_07[var1] = true;
         this.f_byte_arr_24[var1] = (byte)this.currentFloor;
         this.f_int_arr_23[var1] = this.playerHp;
         this.f_int_arr_24[var1] = this.playerAtk;
         this.f_int_arr_25[var1] = this.playerDef;
         this.f_int_arr_26[var1] = this.yellowKeyCount;
         this.f_int_arr_27[var1] = this.blueKeyCount;
         this.f_int_arr_28[var1] = this.redKeyCount;
         this.f_int_arr_29[var1] = this.goldAmount;
         this.f_byte_arr_25[var1] = this.equippedWeaponType;
         this.f_byte_arr_26[var1] = this.equippedArmorType;
      }

      String var6 = "MOT_IF";
      m_116("MOT_IF");

      try {
         this.f_RecordStore_00 = RecordStore.openRecordStore(var6, true);
         this.f_ByteArrayOutputStream_00 = new ByteArrayOutputStream();
         this.f_DataOutputStream_00 = new DataOutputStream(this.f_ByteArrayOutputStream_00);

         for (int var7 = 0; var7 < 6; var7++) {
            if (this.f_bool_arr_07[var7]) {
               this.f_DataOutputStream_00.write(1);
               this.f_DataOutputStream_00.writeByte(this.f_byte_arr_24[var7]);
               this.f_DataOutputStream_00.writeByte(this.f_byte_arr_25[var7]);
               this.f_DataOutputStream_00.writeByte(this.f_byte_arr_26[var7]);
               this.f_DataOutputStream_00.writeInt(this.f_int_arr_23[var7]);
               this.f_DataOutputStream_00.writeInt(this.f_int_arr_24[var7]);
               this.f_DataOutputStream_00.writeInt(this.f_int_arr_25[var7]);
               this.f_DataOutputStream_00.writeInt(this.f_int_arr_26[var7]);
               this.f_DataOutputStream_00.writeInt(this.f_int_arr_27[var7]);
               this.f_DataOutputStream_00.writeInt(this.f_int_arr_28[var7]);
               this.f_DataOutputStream_00.writeInt(this.f_int_arr_29[var7]);
            } else {
               this.f_DataOutputStream_00.write(0);
            }
         }

         this.f_RecordStore_00.addRecord(this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
         return;
      } catch (Exception var4) {
         var4.printStackTrace();
      } finally {
         this.m_117();
      }
   }

   private void m_113(boolean var1) {
      int var1i = var1 ? 2 : 1;
      int var2 = 240 - this.f_int_135 - 22 >> 1;
      int var3 = 320 - this.f_int_131 >> 1;
      Image var4 = null;
      this.m_038(var1i, var2, var3, this.f_int_135 + 22, this.f_int_131);
      var3 += 16;

      for (int var6 = this.f_int_132; var6 < this.f_int_133; var3 += this.f_int_01 + 4) {
         if (var6 != this.f_int_134) {
            this.f_Graphics_00.setColor(7574946);
         } else {
            this.f_Graphics_00.setColor(3156024);
            this.f_Graphics_00.fillRect(120 - (this.f_int_135 >> 1), var3, this.f_int_135, this.f_int_01 + 4);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][14], 120 - (this.f_int_135 >> 1) + 10, var3 + (this.f_int_01 - 8 >> 1), 0);
            this.m_004(this.f_Image_arr2_00[8][14], 120 + (this.f_int_135 >> 1) - 30, var3 + (this.f_int_01 - 8 >> 1), 1);
            this.f_Graphics_00.setColor(16377897);
         }

         if (!this.f_bool_arr_07[var6]) {
            this.f_Graphics_00.drawString("---", 120, var3 + 2, 17);
         } else {
            this.f_Graphics_00.drawString("存档" + var6, 120, var3 + 2, 17);
         }

         var6++;
      }

      var2 = 120 + ((this.f_int_135 >> 1) - 10);
      if (this.f_int_132 > 0) {
         this.m_002(this.f_Image_arr2_00[8][15], var2, var3 - 20, 0, 0, 7, 9);
      }

      if (this.f_int_133 < 6) {
         this.m_002(this.f_Image_arr2_00[8][15], var2, var3 - 10, 7, 0, 7, 9);
      }

      var3 += 2;
      var2 -= this.f_int_135 - 10;
      this.f_Graphics_00.setColor(6178);
      this.f_Graphics_00.drawLine(var2, var3, var2 + this.f_int_135 - 1, var3);
      var3++;
      this.f_Graphics_00.setColor(3564144);
      this.f_Graphics_00.drawLine(var2, var3, var2 + this.f_int_135 - 1, var3);
      if (this.f_bool_arr_07[this.f_int_134]) {
         var2 += 6;
         var3 += 8;
         if (this.f_byte_arr_24[this.f_int_134] < 51) {
            this.m_042(this.f_Image_arr2_00[8][18], this.f_byte_arr_24[this.f_int_134], var2 + 48, var3);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][8], var2 + 50, var3 + 8, 0);
         }

         var2 = 240 + this.f_int_135 >> 1;
         var2 -= 68;
         this.m_041(var2, var3, 32, 32);
         if (this.f_byte_arr_25[this.f_int_134] == 0) {
            this.f_Graphics_00.setColor(-1);
            this.f_Graphics_00
               .drawString(this.f_String_arr_04[this.f_byte_arr_25[this.f_int_134]], var2 + (32 - this.f_int_00 >> 1), var3 + (32 - this.f_int_01 >> 1), 0);
         } else {
            if ((var1i = this.f_byte_arr_25[this.f_int_134] - 33) > 7) {
               var1i = 8;
            }

            var4 = this.f_Image_arr2_00[6][var1i];
            this.f_Graphics_00.drawImage(var4, var2 + (32 - var4.getWidth() >> 1), var3 + (32 - var4.getHeight() >> 1), 0);
         }

         var2 += 34;
         this.m_041(var2, var3, 32, 32);
         if (this.f_byte_arr_26[this.f_int_134] == 0) {
            this.f_Graphics_00.setColor(-1);
            this.f_Graphics_00
               .drawString(this.f_String_arr_04[this.f_byte_arr_26[this.f_int_134]], var2 + (32 - this.f_int_00 >> 1), var3 + (32 - this.f_int_01 >> 1), 0);
         } else {
            if ((var1i = this.f_byte_arr_26[this.f_int_134] - 33) > 7) {
               var1i = 9;
            }

            var4 = this.f_Image_arr2_00[6][var1i];
            this.f_Graphics_00.drawImage(var4, var2 + (32 - var4.getWidth() >> 1), var3 + (32 - var4.getHeight() >> 1), 0);
         }

         var2 = 240 - this.f_int_135 + 10 >> 1;
         var3 += 20;
         this.f_Graphics_00.drawImage(this.f_Image_arr2_00[8][1], var2, var3, 0);
         var2 += 60;
         this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_29[this.f_int_134], var2, var3 + 2);
         var2 -= 64;
         var3 += 16;
         this.m_002(this.f_Image_arr2_00[8][7], var2, var3, 0, 2, 10, 10);
         var2 += 15;
         this.m_041(var2 - 1, var3, 48, 12);
         this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_23[this.f_int_134], var2 + 45, var3 + 2);
         var2 += 47;
         this.m_002(this.f_Image_arr2_00[8][7], var2, var3, 10, 0, 10, 13);
         var2 += 11;
         this.f_Graphics_00.setColor(512);
         this.m_041(var2, var3, 30, 12);
         this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_24[this.f_int_134], var2 + 28, var3 + 2);
         var2 += 32;
         this.m_002(this.f_Image_arr2_00[8][7], var2, var3, 20, 2, 10, 10);
         var2 += 12;
         this.f_Graphics_00.setColor(512);
         this.m_041(var2, var3, 30, 12);
         this.m_042(this.f_Image_arr2_00[8][2], this.f_int_arr_25[this.f_int_134], var2 + 28, var3 + 2);
         var2 = (240 - this.f_int_135 >> 1) + 6;
         var3 += 16;
         this.m_036(0, this.f_int_arr_26[this.f_int_134], var2, var3);
         var2 += 42;
         this.m_036(1, this.f_int_arr_27[this.f_int_134], var2, var3);
         var2 += 42;
         this.m_036(2, this.f_int_arr_28[this.f_int_134], var2, var3);
      }
   }

   private void m_114(int var1) {
      this.m_112(var1);
      String var6;
      m_116(var6 = "MOT_L" + var1);

      try {
         this.f_RecordStore_00 = RecordStore.openRecordStore(var6, true);
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

         for (int var7 = 0; var7 < this.f_int_89; var7++) {
            this.f_DataOutputStream_00.writeByte(this.f_byte_arr_14[var7]);
            this.f_DataOutputStream_00.writeByte(this.f_byte_arr_15[var7]);
         }

         for (int var8 = 0; var8 < 128; var8++) {
            if (var8 < this.f_int_113) {
               this.f_DataOutputStream_00.writeBoolean(this.f_bool_arr_06[var8]);
            } else {
               this.f_DataOutputStream_00.writeBoolean(false);
            }
         }

         this.f_DataOutputStream_00.writeByte(this.itemStackSize);

         for (int var9 = 0; var9 < this.itemStackSize; var9++) {
            this.f_DataOutputStream_00.writeByte(this.itemStackTypes[var9]);
            this.f_DataOutputStream_00.writeByte(this.itemStackUses[var9]);
         }

         for (int var10 = 0; var10 < 56; var10++) {
            if (this.f_byte_arr2_05[var10] != null) {
               this.f_DataOutputStream_00.writeShort(this.f_byte_arr2_05[var10].length);
               this.f_DataOutputStream_00.write(this.f_byte_arr2_05[var10], 0, this.f_byte_arr2_05[var10].length);
            } else {
               this.f_DataOutputStream_00.writeShort(-1);
            }
         }

         this.f_RecordStore_00.addRecord(this.f_ByteArrayOutputStream_00.toByteArray(), 0, this.f_ByteArrayOutputStream_00.size());
         return;
      } catch (Exception var4) {
         var4.printStackTrace();
      } finally {
         this.m_117();
      }
   }

   private boolean m_115(int var1) {
      boolean var2 = true;
      short var3 = 0;
      String var8 = "MOT_L" + var1;

      try {
         this.f_RecordStore_00 = RecordStore.openRecordStore(var8, false);
         this.f_RecordEnumeration_00 = this.f_RecordStore_00.enumerateRecords(null, null, false);
         var1 = this.f_RecordEnumeration_00.nextRecordId();
         this.f_byte_arr_27 = this.f_RecordStore_00.getRecord(var1);
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

         for (int var10 = 0; var10 < this.f_int_89; var10++) {
            this.f_byte_arr_14[var10] = this.f_DataInputStream_00.readByte();
            this.f_byte_arr_15[var10] = this.f_DataInputStream_00.readByte();
         }

         for (int var11 = 0; var11 < this.f_int_113; var11++) {
            this.f_bool_arr_06[var11] = this.f_DataInputStream_00.readBoolean();
         }

         if (this.f_int_113 < 128) {
            this.f_DataInputStream_00.skip(128 - this.f_int_113);
         }

         this.f_bool_12 = false;
         this.itemStackSize = this.f_DataInputStream_00.readByte();

         for (int var12 = 0; var12 < this.itemStackSize; var12++) {
            this.itemStackTypes[var12] = this.f_DataInputStream_00.readByte();
            this.itemStackUses[var12] = this.f_DataInputStream_00.readByte();
            if (this.itemStackTypes[var12] == 15) {
               this.f_bool_12 = true;
            }
         }

         for (int var13 = 0; var13 < 56; var13++) {
            if ((var3 = this.f_DataInputStream_00.readShort()) > 0) {
               this.f_byte_arr2_05[var13] = new byte[var3];
               this.f_DataInputStream_00.read(this.f_byte_arr2_05[var13], 0, var3);
            } else {
               this.f_byte_arr2_05[var13] = null;
            }
         }

         this.scaleEnemyStats(this.difficultyMultipliers[this.f_byte_26]);
      } catch (Exception var6) {
         var6.printStackTrace();
         var2 = false;
      } finally {
         this.m_117();
      }

      return var2;
   }

   private static void m_116(String var0) {
      try {
         RecordStore.deleteRecordStore(var0);
      } catch (Exception var1) {
      }
   }

   private void m_117() {
      if (this.f_RecordStore_00 != null) {
         try {
            this.f_RecordStore_00.closeRecordStore();
         } catch (Exception var3) {
         }
      }

      if (this.f_ByteArrayOutputStream_00 != null) {
         try {
            this.f_ByteArrayOutputStream_00.close();
         } catch (Exception var2) {
         }
      }

      if (this.f_DataOutputStream_00 != null) {
         try {
            this.f_DataOutputStream_00.close();
         } catch (Exception var1) {
         }
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
      int var1 = 56;

      while (--var1 >= 0) {
         this.f_byte_arr2_05[var1] = null;
      }
   }

   private void m_119(int var1) {
      int var2 = this.f_int_45;
      int var3 = this.f_int_45;

      while (--var3 >= 0) {
         if (this.f_bool_arr_01[var3] || this.f_byte_arr_04[var3] == 1) {
            var2--;
         }
      }

      this.f_byte_arr2_05[var1] = null;
      if (this.f_int_45 > 0) {
         this.f_byte_arr2_05[var1] = new byte[(var2 << 3) + 2];
         this.f_byte_arr_28 = this.f_byte_arr2_05[var1];
         this.f_int_137 = 0;
         this.m_120((short)var2);

         for (int var4 = 0; var4 < this.f_int_45; var4++) {
            if (!this.f_bool_arr_01[var4] && this.f_byte_arr_04[var4] != 1) {
               this.f_byte_arr_28[this.f_int_137++] = (byte)this.entityType[var4];
               this.m_120((short)this.entityPixelX[var4]);
               this.m_120((short)this.entityPixelY[var4]);
               this.f_byte_arr_28[this.f_int_137++] = (byte)(this.f_bool_arr_00[var4] ? 1 : 0);
               this.m_120(this.entityParam[var4]);
            }
         }
      }
   }

   private void m_120(short var1) {
      this.f_byte_arr_28[this.f_int_137++] = (byte)(var1 >> 8);
      this.f_byte_arr_28[this.f_int_137++] = (byte)var1;
   }

   private void m_121(int var1) {
      int var3 = var1;
      a var2 = this;
      InputStream var14 = this.getClass().getResourceAsStream("maplv" + var3);

      try {
         var2.mapCellsWide = m_058(var14) >> 1;
         var2.mapCellsHigh = m_058(var14) >> 1;
         System.out.println("Width:" + var2.mapCellsWide + ",Height:" + var2.mapCellsHigh);
         int var4 = var2.mapCellsWide * var2.mapCellsHigh << 2;
         var2.f_int_52 = var2.mapCellsWide << 5;
         var2.f_int_53 = var2.mapCellsHigh << 5;
         var2.mapTerrainGrid = new byte[var4];
         var2.mapTransformGrid = new byte[var4];
         var14.read(var2.mapTerrainGrid, 0, var4);
         var14.read(var2.mapTransformGrid, 0, var4);
         m_006(32, 32, 1442775295);
         m_006(32, 32, 1426128640);
      } catch (Exception var11) {
         var11.printStackTrace();
      } finally {
         try {
            var14.close();
         } catch (Exception var10) {
         }
      }

      var2.f_int_58 = 240;
      var2.f_int_59 = 252;
      var2.f_bool_10 = var2.f_int_58 >= var2.f_int_52;
      var2.f_bool_11 = var2.f_int_59 >= var2.f_int_53;
      var2.m_059();
      a var13;
      (var13 = var2).f_bool_arr2_00 = new boolean[var13.mapCellsHigh][var13.mapCellsWide];
      var13.m_064(0, 0);
      var13.m_061();
      this.f_int_45 = 0;
      this.f_int_70 = this.f_int_72 = this.f_int_71 = this.f_int_73 = -1;
      this.m_122(var1);
      this.m_054();
   }

   private void m_122(int var1) {
      int var2 = 0;
      int var3 = 0;
      int var6 = 0;
      a var12 = this;
      this.f_int_45 = 0;
      a var17 = var12;

      for (int var4 = 1; var4 <= 12; var4++) {
         var17.f_Image_arr_00[var4] = var17.f_Image_arr2_00[4][var4 - 1];
      }

      for (int var21 = 13; var21 <= 32; var21++) {
         var17.f_Image_arr_00[var21] = var17.f_Image_arr2_00[5][var21 - 13 + 1];
      }

      for (int var22 = 33; var22 <= 40; var22++) {
         var17.f_Image_arr_00[var22] = var17.f_Image_arr2_00[6][var22 - 33];
      }

      for (int var23 = 41; var23 < 61; var23++) {
         var17.f_Image_arr_00[var23] = var17.f_Image_arr2_00[7][var23 - 41];
      }

      for (int var24 = 61; var24 < 79; var24++) {
         var17.f_Image_arr_00[var24] = var17.f_Image_arr2_00[13][var24 - 41 - 20];
      }

      var17.f_Image_arr_00[84] = var17.f_Image_arr2_00[13][18];
      var17.f_Image_arr_00[87] = var17.f_Image_arr2_00[13][19];
      var17.f_Image_arr_00[85] = var17.f_Image_arr2_00[5][21];
      var17.f_Image_arr_00[86] = var17.f_Image_arr2_00[5][22];
      var17.f_Image_arr_00[79] = var17.f_Image_arr2_00[6][8];
      var17.f_Image_arr_00[80] = var17.f_Image_arr2_00[6][9];
      var17.f_Image_arr_00[12] = var17.f_Image_arr_00[11];
      var17.f_Image_arr_00[81] = var17.f_Image_arr2_00[4][3];
      var17.f_Image_arr_00[71] = var17.f_Image_arr_00[70];
      this.f_int_45 = 0;
      if (this.f_byte_arr2_05[var1] == null) {
         try {
            var3 = var1;
            a var13 = this;
            InputStream var25 = this.getClass().getResourceAsStream("sprite" + var3);
            int var5 = 0;
            int var8 = 0;

            try {
               InputStream var27 = var25;
               var3 = m_058(var27) & '\uffff' | m_058(var27) << 16;

               for (int var9 = 0; var9 < var3; var9++) {
                  var5 = var25.read();
                  var6 = m_058(var25);
                  short var7 = m_058(var25);
                  switch (var5) {
                     case 4:
                        var8 = var25.read() + 1;
                        var13.m_048(var5, var6, var7, var8);
                        break;
                     case 5:
                     case 81:
                        var8 = var25.read() + 1;
                        var5 = var13.m_048(var5, var6, var7, var8);
                        var13.f_bool_arr_00[var5] = false;
                        break;
                     case 6:
                     case 12:
                        var5 = var13.m_048(var5, var6, var7, 0);
                        var13.f_bool_arr_00[var5] = false;
                        break;
                     case 7:
                     case 8:
                        var8 = var25.read() | var25.read() << 8;
                        var13.m_048(var5, var6, var7, var8);
                        break;
                     case 9:
                        var8 = var25.read();
                        var13.m_048(var5, var6, var7, var8);
                        break;
                     case 10:
                     case 11:
                     case 13:
                     case 14:
                     case 15:
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
                     case 26:
                     case 27:
                     case 28:
                     case 29:
                     case 30:
                     case 31:
                     case 32:
                     case 33:
                     case 34:
                     case 35:
                     case 36:
                     case 37:
                     case 38:
                     case 39:
                     case 40:
                     case 41:
                     case 42:
                     case 43:
                     case 44:
                     case 45:
                     case 46:
                     case 47:
                     case 48:
                     case 49:
                     case 50:
                     case 51:
                     case 52:
                     case 53:
                     case 54:
                     case 55:
                     case 56:
                     case 58:
                     case 60:
                     case 61:
                     case 62:
                     case 63:
                     case 64:
                     case 65:
                     case 66:
                     case 67:
                     case 68:
                     case 69:
                     case 74:
                     case 75:
                     case 79:
                     case 80:
                     default:
                        var13.m_048(var5, var6, var7, 0);
                        break;
                     case 57:
                     case 59:
                     case 70:
                     case 71:
                     case 72:
                     case 73:
                        var8 = var25.read() + 1;
                        var13.m_048(var5, var6, var7, var8);
                        break;
                     case 76:
                     case 82:
                        var8 = var25.read() | var25.read() + 1 << 8;
                        var5 = var13.m_048(var5, var6, var7, var8);
                        var13.f_bool_arr_00[var5] = false;
                        break;
                     case 77:
                        var8 = (var8 = var25.read()) & 0xFF | var13.f_byte_arr_12[var8 - 1] << 8;
                        var13.m_048(var5, var6, var7, var8);
                        break;
                     case 78:
                        var8 = (var8 = var25.read()) & 0xFF | var13.f_byte_arr_13[var8 - 1] << 8;
                        var13.m_048(var5, var6, var7, var8);
                        break;
                     case 83:
                        var8 = var25.read();
                        var5 = var13.m_048(var5, var6, var7, var8);
                        var13.f_bool_arr_00[var5] = false;
                  }
               }
            } catch (Exception var10) {
               var10.printStackTrace();
            }
         } catch (Exception var11) {
         }
      } else {
         this.f_byte_arr_29 = this.f_byte_arr2_05[var1];
         this.f_int_138 = 0;
         var2 = this.m_123();

         for (int var48 = 0; var48 < var2; var48++) {
            byte var20 = this.f_byte_arr_29[this.f_int_138++];
            int var26 = this.m_123();
            int var33 = this.m_123();
            boolean var36 = this.f_byte_arr_29[this.f_int_138++] != 0;
            var6 = this.m_123();
            this.m_048(var20, var26, var33, var6);
            this.f_bool_arr_00[var48] = var36;
            this.f_bool_arr_01[var48] = false;
         }

         switch (this.currentFloor) {
            case 2:
               if (!this.f_bool_arr_06[26] && this.m_100(11, 4, 84) < 0) {
                  this.m_048(84, 352, 128, 0);
               }
               break;
            case 12:
               if ((var2 = this.m_100(1, 1, 78)) >= 0) {
                  this.entityParam[var2] = (short)(6 | this.f_byte_arr_13[5] << 8);
               }

               if ((var2 = this.m_100(11, 1, 78)) >= 0) {
                  this.entityParam[var2] = (short)(5 | this.f_byte_arr_13[4] << 8);
               }
            case 31:
            default:
               break;
            case 39:
               if (this.m_100(11, 1, 7) >= 0) {
                  this.m_049(11, 1, 7);
                  this.m_048(8, 11, 1, 38);
               }

               if (this.m_100(11, 11, 8) >= 0) {
                  this.m_049(11, 11, 8);
                  this.m_048(7, 11, 11, 40);
               }
         }
      }

      if (var1 != 50) {
         this.f_byte_arr_03[72] = 32;
      } else {
         this.f_byte_arr_03[72] = 8;
      }
   }

   private int m_123() {
      return (this.f_byte_arr_29[this.f_int_138++] & 0xFF) << 8 | this.f_byte_arr_29[this.f_int_138++] & 0xFF;
   }

   private void m_124(int var1, int var2) {
      if (this.f_byte_21 < 64) {
         this.f_short_arr_07[this.f_byte_21] = (short)var1;
         this.f_short_arr_08[this.f_byte_21] = (short)var2;
         this.f_byte_21++;
      }
   }

   private void m_125(byte var1, int var2, int var3, int var4) {
      var3 += this.f_int_56;
      var4 += this.f_int_57;
      this.f_byte_arr_30[this.f_int_140] = (byte)var1;
      this.f_int_arr_30[this.f_int_140] = var2;
      this.f_bool_arr_08[this.f_int_140] = false;
      this.f_byte_arr_31[this.f_int_140] = 0;
      this.f_short_arr_10[this.f_int_140] = (short)var4;
      if (var1 == 4) {
         this.f_short_arr_09[this.f_int_140] = 240;
         this.f_short_arr_10[this.f_int_140] = 30;
      } else if (var1 >= 5 && var1 <= 7) {
         this.f_short_arr_09[this.f_int_140] = (short)(var3 + 16);
      } else if (var1 == 1) {
         this.f_short_arr_09[this.f_int_140] = (short)(var3 - 2);
      } else {
         short[] var10000 = this.f_short_arr_09;
         int var10001 = this.f_int_140;
         Image var10004 = this.f_Image_arr2_00[2][3];
         var2 = var2;
         int w125 = var10004.getWidth() / 11;
            var3 = w125;
         if (var2 < 0) {
            var2 = -var2;
         }

         do {
            var3 += w125;
         } while ((var2 /= 10) > 0);

         var10000[var10001] = (short)(var3 + (32 + var3 >> 1));
      }

      if (++this.f_int_140 >= 30) {
         this.f_int_140 = 0;
      }
   }

   private void m_126() {
      byte var3 = 0;
      int var4 = this.f_int_139;

      while (true) {
         if (var4 >= 30) {
            var4 = 0;
         }

         if (var4 == this.f_int_140) {
            if (this.f_int_139 != this.f_int_140 && this.f_bool_arr_08[this.f_int_139] && ++this.f_int_139 >= 30) {
               this.f_int_139 = 0;
            }

            return;
         }

         if (!this.f_bool_arr_08[var4]) {
            byte var5 = this.f_byte_arr_30[var4];
            int var6 = this.f_int_arr_30[var4];
            int var1 = 0 + this.f_short_arr_09[var4];
            var3 = this.f_byte_arr_31[var4];
            int var2 = 16 + this.f_short_arr_10[var4];
            switch (var5) {
               case 1:
                  var2 -= this.f_byte_arr_31[var4] << 2;
                  this.m_002(this.f_Image_arr2_00[2][6], var1, var2, 0, 19 * var6, 37, 19);
                  break;
               case 2:
                  var2 -= this.f_byte_arr_31[var4] << 2;
                  this.m_127(this.f_Image_arr2_00[2][3], var6, var1, var2);
                  break;
               case 3:
                  var2 -= this.f_byte_arr_31[var4] << 2;
                  this.m_127(this.f_Image_arr2_00[2][4], var6, var1, var2);
                  break;
               case 4:
                  var2 -= this.f_byte_arr_31[var4] << 2;
                  this.m_127(this.f_Image_arr2_00[2][5], var6, var1, var2);
                  break;
               case 5:
                  this.m_042(this.f_Image_arr2_00[8][9], var6, var1 + this.f_byte_arr_32[var3], 16 + this.f_short_arr_10[var4] + this.f_byte_arr_33[var3]);
                  break;
               case 6:
                  this.m_042(this.f_Image_arr2_00[8][9], var6, var1 + this.f_byte_arr_34[var3], 16 + this.f_short_arr_10[var4] + this.f_byte_arr_35[var3]);
                  break;
               case 7:
                  this.m_042(this.f_Image_arr2_00[8][9], var6, var1 + this.f_byte_arr_36[var3], 16 + this.f_short_arr_10[var4] + this.f_byte_arr_37[var3]);
            }

            if (var3 > 7) {
               this.f_bool_arr_08[var4] = true;
            } else {
               this.f_byte_arr_31[var4]++;
            }
         }

         var4++;
      }
   }

   private int m_127(Image var1, int var2, int var3, int var4) {
      boolean var5;
      if (var5 = var2 < 0) {
         var2 = -var2;
      }

      int digitW = var1.getWidth() / 11;
      int var6 = var1.getHeight();
      var3 = var3;
      int var7 = 0;
      int var8 = 0;

      do {
         var3 -= digitW + 1;
         var7 = var2 % 10;
         this.f_Graphics_00.setClip(var3, var4, digitW, var6);
         this.f_Graphics_00.drawImage(var1, var3 - var7 * digitW, var4, 0);
         var2 /= 10;
         var8++;
      } while (var2 > 0);

      var3 -= digitW + 1;
      this.f_Graphics_00.setClip(var3, var4, digitW, var6);
      this.f_Graphics_00.drawImage(var1, var3 - (var1.getWidth() - digitW), var4, 0);
      this.f_Graphics_00.setClip(0, 0, 240, 320);
      return var8 + 1;
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

   private void m_129(int var1, int var2, int var3, int var4) {
      this.f_short_arr_11[this.f_byte_22] = (short)var1;
      this.f_short_arr_12[this.f_byte_22] = (short)var2;
      this.f_byte_arr_38[this.f_byte_22] = -3;
      this.f_byte_arr_39[this.f_byte_22] = -4;
      this.f_byte_arr_40[this.f_byte_22] = 2;
      this.f_bool_arr_09[this.f_byte_22] = true;
      this.f_int_arr_31[this.f_byte_22] = -1;
      this.f_bool_arr_10[this.f_byte_22] = false;
      if (++this.f_byte_22 > 31) {
         this.f_byte_22 = 0;
      }
   }

   private void m_130() {
      byte var1 = 0;
      int var2 = 0;
      this.f_Graphics_00.setColor(this.f_int_141);
      int var3 = 32;

      while (--var3 >= 0) {
         if (this.f_bool_arr_09[var3]) {
            var2 = this.f_byte_arr_40[var3];
            this.f_Graphics_00.fillRect(this.f_short_arr_11[var3], this.f_short_arr_12[var3], var2, var2);
            var1 = this.f_byte_arr_38[var3];
            this.f_short_arr_11[var3] = (short)(this.f_short_arr_11[var3] + var1);
            var2 <<= 1;
            if (!this.f_bool_arr_10[var3]) {
               if (++this.f_byte_arr_38[var3] == var2) {
                  this.f_bool_arr_10[var3] = true;
               }
            } else if (--this.f_byte_arr_38[var3] == -var2) {
               this.f_bool_arr_10[var3] = false;
               if (++this.f_byte_arr_40[var3] > 5) {
                  this.f_bool_arr_09[var3] = false;
               }
            }

            this.f_short_arr_12[var3] = (short)(this.f_short_arr_12[var3] + this.f_byte_arr_39[var3]);
         }
      }
   }

   private void m_131(byte var1, boolean var2) {
      this.f_byte_24 = 3;
      this.f_bool_19 = true;
      if (this.f_byte_arr_41 == null) {
         this.f_byte_arr_41 = new byte[16];
      }

      this.f_int_144 = this.f_int_145 = 0;
      this.gameMode = 2;
      this.m_001(15);
   }

   private void m_132(int var1) {
      this.f_byte_arr_41[this.f_int_144++] = (byte)var1;
   }

   private boolean m_133(int var1, int var2, int var3, int var4) {
      a var5 = this;
      this.f_byte_arr_42 = new byte[100];
      var5.f_int_148 = 0;
      var5.f_byte_arr2_06 = new byte[var5.mapCellsHigh][var5.mapCellsWide];
      var5.f_short_arr2_00 = new short[var5.mapCellsHigh][var5.mapCellsWide];
      var5.f_short_arr_13 = new short[100];
      var5.f_short_arr_14 = new short[100];
      var5.f_int_149 = 0;
      boolean var16 = false;
      int var6 = var1;
      int var7 = var2;
      int var8 = this.mapCellsWide;
      int var9 = var6 + var7 * var8;
      this.f_short_00 = (short)var1;
      this.f_short_01 = (short)var2;
      this.f_short_02 = (short)var3;
      this.f_short_03 = (short)var4;
      this.f_int_149 = 1;
      var3 = 0;

      do {
         this.m_134(var6 - 1, var7, var9);
         this.m_134(var6 + 1, var7, var9);
         this.m_134(var6, var7 - 1, var9);
         this.m_134(var6, var7 + 1, var9);
         if (this.f_int_149 <= 0) {
            break;
         }

         var6 = (var9 = this.f_short_arr_14[this.f_int_149 - 1]) % var8;
         var7 = var9 / var8;
         this.f_int_149--;
         this.f_byte_arr2_06[var7][var6] = 2;
         if (var6 == this.f_short_02 && var7 == this.f_short_03) {
            var16 = true;
         }
      } while (++var3 < 100 && !var16);

      short var14 = 0;
      var1 += var2 * var8;
      int var11 = 0;
      if (var16) {
         do {
            if ((var11 = (var14 = this.f_short_arr2_00[var7][var6]) - var9) == -1) {
               this.f_byte_arr_42[this.f_int_148++] = 2;
            } else if (var11 == 1) {
               this.f_byte_arr_42[this.f_int_148++] = 3;
            } else if (var11 == -var8) {
               this.f_byte_arr_42[this.f_int_148++] = 0;
            } else {
               if (var11 != var8) {
                  return false;
               }

               this.f_byte_arr_42[this.f_int_148++] = 1;
            }

            var9 = var14;
            var6 = var14 % var8;
            var7 = var14 / var8;
         } while (var14 != var1);
      }

      return var16;
   }

   private void m_134(int var1, int var2, int var3) {
      short var4 = 0;
      if (this.f_byte_arr2_06[var2][var1] == 0 && (this.isCellWalkable(var1, var2) || this.m_062(var1, var2))) {
         this.f_byte_arr2_06[var2][var1] = 1;
         this.f_short_arr2_00[var2][var1] = (short)var3;
         var4 = (short)(
            Math.abs(var1 - this.f_short_02) + Math.abs(var2 - this.f_short_03) + Math.abs(var1 - this.f_short_00) + Math.abs(var2 - this.f_short_01)
         );
         var3 = this.f_int_149;

         while (--var3 >= 0) {
            if (var4 < this.f_short_arr_13[var3] || var3 == 0) {
               for (int var5 = this.f_int_149; var5 > var3 + 1; var5--) {
                  this.f_short_arr_13[var5] = this.f_short_arr_13[var5 - 1];
                  this.f_short_arr_14[var5] = this.f_short_arr_14[var5 - 1];
               }

               this.f_short_arr_13[++var3] = var4;
               this.f_short_arr_14[var3] = (short)(var1 + var2 * this.mapCellsWide);
               break;
            }
         }

         this.f_int_149++;
      }
   }

   private int predictHpLossVsType(int var1, boolean var2) {
      int var3 = this.effectiveAttackVsType(var1);
      var1 -= 41;
      return predictBattleHpLoss(var3, this.playerDef, this.enemyHpScaled[var1], this.enemyAtkScaled[var1], this.enemyDefScaled[var1], var2);
   }

   private int effectiveAttackVsType(int var1) {
      byte var3 = this.f_byte_arr_05[var1];
      byte var2 = 1;
      if ((var3 & 1) != 0 && this.m_081(23) >= 0) {
         var2 = 2;
      } else if ((var3 & 2) != 0 && this.m_081(24) >= 0) {
         var2 = 2;
      }

      return this.playerAtk * var2;
   }

   private static int predictBattleHpLoss(int var0, int var1, int var2, int var3, int var4, boolean var5) {
      int var6 = -1;
      int var7 = 0;
      if (var0 > var4) {
         if ((var7 = var2 / (var0 - var4)) * (var0 - var4) < var2) {
            var7++;
         }

         if (var7 > 1 && var3 > var1) {
            var6 = (var3 - var1) * (var7 - 1);
         } else {
            var6 = 0;
         }

         if (!var5 && var3 > var1) {
            var6 += var3 - var1;
         }
      }

      return var6;
   }

   private void tickBattle(boolean var1) {
      if (!this.f_bool_23) {
         if (this.f_int_44 < 0) {
            this.f_byte_11 = 0;
         } else {
            int var1t = this.entityType[this.f_int_44];
            int var8;
            if ((var8 = this.effectiveAttackVsType(var1t) - this.enemyDefScaled[var1t - 41]) > 0) {
               if ((this.f_int_151 & 3) == 0) {
                  this.f_int_150 -= var8;
                  this.m_125((byte)(5 + this.f_int_151 % 3), var8, this.entityPixelX[this.f_int_44], this.entityPixelY[this.f_int_44]);
                  if (this.f_int_150 > 0) {
                     if ((var8 = this.enemyAtkScaled[var1t - 41] - this.playerDef) > 0) {
                        this.playerHp -= var8;
                     }
                  } else {
                     if ((var8 = this.predictHpLossVsType(var1t, this.f_bool_25)) > 0) {
                        this.m_125((byte)2, var8, this.playerPixelX, this.playerPixelY);
                     }

                     if (this.m_081(25) >= 0) {
                        this.gainGold(this.enemyBaseGold[var1t - 41] << 1, this.entityPixelX[this.f_int_44], this.entityPixelY[this.f_int_44]);
                     } else {
                        this.gainGold(this.enemyBaseGold[var1t - 41], this.entityPixelX[this.f_int_44], this.entityPixelY[this.f_int_44]);
                     }

                     var8 = this.entityType[this.f_int_44];
                     a var7 = this;
                     boolean var3 = false;
                     boolean var10001;
                     if (var7.currentFloor > 50) {
                        var10001 = false;
                     } else {
                        int var4 = var7.bossTypeOrder.length;

                        while (--var4 >= 0) {
                           if (var8 == var7.bossTypeOrder[var4]) {
                              var7.spawnBossEvent(var4);
                              break;
                           }
                        }

                        switch (var8) {
                           case 73:
                              var7.f_bool_arr_06[var7.currentScriptIndex] = true;
                              var7.loadLevelScript(22);
                              if (var7.currentFloor == 40) {
                                 var7.spawnBossEvent(6);
                              }
                              break;
                           case 74:
                              var7.loadLevelScript(21);
                              break;
                           case 75:
                              if (var7.tryRunScene(20, true)) {
                                 var7.loadLevelScript(20);
                                 var3 = true;
                              }
                        }

                        var10001 = var3;
                     }

                     this.f_bool_24 = var10001;
                     this.f_bool_23 = true;
                  }
               }
            } else {
               this.f_byte_11 = 0;
            }

            this.f_int_151++;
         }
      } else {
         if (this.f_byte_27 == 0) {
            this.m_047(this.f_int_44);
         }

         if (++this.f_byte_27 > 5) {
            this.f_byte_27 = 0;
            if (!this.f_bool_24 && var1) {
               this.tryStep(this.f_byte_12);
            }

            this.f_bool_23 = false;
            int var2 = this.entityType[this.f_int_44];
            a var5 = this;
            switch (var2) {
               case 61:
                  if (var5.currentFloor == 49
                     && var5.m_100(6, 2, 61) < 0
                     && var5.m_100(5, 3, 61) < 0
                     && var5.m_100(7, 3, 61) < 0
                     && var5.m_100(6, 4, 61) < 0
                     && var5.m_100(5, 2, 61) >= 0
                     && var5.m_100(7, 2, 61) >= 0
                     && var5.m_100(5, 4, 61) >= 0
                     && var5.m_100(7, 4, 61) >= 0) {
                     var5.m_049(6, 3, 70);
                     var5.m_048(71, 192, 96, 0);
                  }
               case 62:
               case 63:
               case 64:
               case 65:
               case 66:
               case 67:
               case 68:
               default:
                  break;
               case 69:
                  if (var5.currentFloor == 35) {
                     var5.spawnBossEvent(7);
                     var5.loadLevelScript(36);
                  }
                  break;
               case 70:
               case 71:
                  if (var5.currentFloor == 49) {
                     var5.loadLevelScript(35);
                  }
                  break;
               case 72:
                  if (var5.currentFloor == 50) {
                     var5.loadLevelScript(65);
                  }
            }

            this.f_int_44 = -1;
         }
      }
   }

   private void m_139(byte var1, int var2) {
      if (this.f_bool_29) {
         this.f_InputStream_00 = null;

         try {
            if (this.f_byte_28 == var1 && f_Player_00 != null) {
               switch (f_Player_00.getState()) {
                  case 400:
                     f_Player_00.stop();
                  case 300:
                     try {
                        f_Player_00.setLoopCount(-1);
                     } catch (Exception var9) {
                     }

                     f_Player_00.start();
                     return;
               }
            }

            this.m_140();
            this.f_InputStream_00 = this.getClass().getResourceAsStream(this.f_String_arr_15[var1]);
            (f_Player_00 = Manager.createPlayer(this.f_InputStream_00, this.f_String_arr_15[var1].endsWith("wav") ? "audio/x-wav" : "audio/midi")).realize();
            f_Player_00.prefetch();

            try {
               f_Player_00.setLoopCount(-1);
            } catch (Exception var11) {
               var11.printStackTrace();
            }

            int var3 = this.f_int_155;
            Player var14 = f_Player_00;
            if (f_Player_00 != null && ((var14.getState() & 300) == 300 || (var14.getState() & 400) == 400)) {
               try {
                  ((VolumeControl)f_Player_00.getControl("VolumeControl")).setLevel(var3);
               } catch (Exception var10) {
               }
            }

            f_Player_00.start();
            this.f_byte_28 = var1;
         } catch (Exception var12) {
            var12.printStackTrace();
         } finally {
            this.m_008();
         }
      }
   }

   private void m_140() {
      if (f_Player_00 != null) {
         f_Player_00.close();
      }

      f_Player_00 = null;
      this.f_byte_28 = -1;
   }

   private int randomBelow(int var1) {
      return (this.gameRandom.nextInt() >>> 1) % var1;
   }

   private void m_142(String var1, int var2, int var3, int var4, int[] var5) {
      var2--;
      var3--;
      this.f_Graphics_00.setColor(var5[0]);
      this.f_Graphics_00.drawString(var1, var2++, var3, 17);
      this.f_Graphics_00.drawString(var1, var2++, var3, 17);
      this.f_Graphics_00.drawString(var1, var2, var3++, 17);
      this.f_Graphics_00.drawString(var1, var2, var3++, 17);
      this.f_Graphics_00.drawString(var1, var2--, var3, 17);
      this.f_Graphics_00.drawString(var1, var2--, var3, 17);
      this.f_Graphics_00.drawString(var1, var2, var3--, 17);
      this.f_Graphics_00.drawString(var1, var2++, var3, 17);
      this.f_Graphics_00.setColor(var5[1]);
      this.f_Graphics_00.drawString(var1, var2, var3, 17);
   }

   private void m_143(int var1, int var2) {
      if (!this.f_bool_30) {
         a var3 = this;
         this.m_001(0);
         int var4 = var3.f_byte_arr_45[0] + var3.f_byte_arr_45[1] + var3.f_byte_arr_45[2];
         var3.f_int_157 = Math.min(15, Math.abs(320 - var4 >> 2));
         var4 = 320 - var4 - (var3.f_int_157 << 1) >> 1;
         var3.f_int_arr_37[0] = var4 + (var3.f_byte_arr_45[0] >> 1);
         var3.f_int_arr_37[1] = var4 + var3.f_byte_arr_45[0] + var3.f_int_157;
         var3.f_int_arr_37[2] = var3.f_int_arr_37[1] + var3.f_byte_arr_45[1] + var3.f_int_157;
         var3.f_int_158 = 240 - var3.f_Image_arr2_00[0][7].getWidth() >> 1;
         this.f_bool_30 = true;
      }

      a var11 = this;

      for (int var13 = this.f_byte_29 - 1; var13 >= 0; var13--) {
         int[] var5;
         switch ((var5 = var11.f_int_arr2_03[var13])[2]) {
            case 1:
               if (var5[3] > 0) {
                  var5[0] += (var5[4] - var5[0]) / var5[3];
                  var5[1] += (var5[5] - var5[1]) / var5[3];
                  var5[3]--;
               }
               break;
            case 2:
               if (var5[4] > 1) {
                  var5[4]--;
               }
               break;
            case 3:
               if (var5[3] > 0) {
                  int var6 = (var5[4] - var5[0]) / var5[3];
                  int var7 = (var5[5] - var5[1]) / var5[3];
                  var5[0] += var6;
                  var5[1] += var7;
                  if (--var5[3] == 0) {
                     var5[4] = var6 > 0 ? 8 : (var6 < 0 ? -8 : 0);
                     var5[5] = var7 > 0 ? 8 : (var7 < 0 ? -8 : 0);
                  }
               } else if (var5[4] == 0) {
                  switch (var5[3]) {
                     case -3:
                     case 0:
                        var5[0] += var5[4];
                        var5[1] += var5[5];
                        break;
                     case -2:
                        var5[4] >>= 1;
                        var5[5] >>= 1;
                     case -1:
                        var5[0] -= var5[4];
                        var5[1] -= var5[5];
                  }

                  var5[3]--;
               }
               break;
            case 4:
               if (var5[3] > 0) {
                  switch (var5[5]) {
                     case 0:
                        var5[0] -= var5[4];
                        var5[1] += var5[4];
                        break;
                     case 1:
                        var5[0] -= var5[4];
                        var5[1] -= var5[4];
                        break;
                     case 2:
                        var5[0] += var5[4];
                        var5[1] -= var5[4];
                        break;
                     case 3:
                        var5[0] += var5[4];
                        var5[1] += var5[4];
                  }

                  if (++var5[5] >= 4) {
                     var5[5] = 0;
                     var5[3]--;
                  }
               }
         }
      }

      if (var1 != 0 && var2 >= 18) {
         if (var2 >= 18 && var2 < 31) {
            this.m_145(
               null,
               this.f_int_158 + this.f_byte_arr_46[var2 - 10 << 1],
               320,
               3,
               0,
               4,
               this.f_int_158 + this.f_byte_arr_46[var2 - 10 << 1],
               this.f_int_arr_37[2],
               var2 - 10,
               -1
            );
         }

         switch (var2) {
            case 50:
               this.m_146(2, 2, 4, 4, 4, 1);
               break;
            case 51:
               this.m_146(3, 2, 4, 4, 4, 1);
               break;
            case 52:
               this.m_146(4, 2, 4, 4, 4, 1);
            case 53:
            case 54:
            case 55:
            case 56:
            case 57:
            case 59:
            case 60:
            case 61:
            case 62:
            default:
               break;
            case 58:
               this.m_146(0, 1, 0, 1, 1000, 1000);
               this.m_146(1, 2, 4, 4, 4, 1);
               break;
            case 63:
               this.m_146(15, 3, 0, 4, -10, this.f_int_arr_37[2]);
         }

         if (var2 > 52 && var2 < 63) {
            var1 = var2 - 52;
            this.m_146(var1 + 4, 3, 0, 4, -10, this.f_int_arr_37[2]);
            this.m_146(26 - var1, 3, 0, 4, 240, this.f_int_arr_37[2]);
         }
      } else {
         switch (var2) {
            case 1:
               this.m_145(this.f_Image_arr2_00[0][1], 120, 0, 1, 12, 4, 120, this.f_int_arr_37[0], 0, -1);
            case 2:
            case 3:
            case 4:
            case 6:
            case 7:
            case 9:
            case 11:
            case 12:
            case 13:
            case 16:
            default:
               break;
            case 5:
               this.m_147(0);
               this.m_145(this.f_Image_arr2_00[0][0], 120, this.f_int_arr_37[0], 2, 12, 4, 4, 0, 0, -1);
               break;
            case 8:
               this.m_145(this.f_Image_arr2_00[0][5], 120, this.f_int_arr_37[0] + 2, 0, 12, 0, 0, 0, 0, 0);
               this.m_145(this.f_Image_arr2_00[0][2], 90, this.f_int_arr_37[1], 2, 4, 4, 4, 0, 0, -1);
               this.m_145(this.f_Image_arr2_00[0][3], 120, this.f_int_arr_37[1], 2, 4, 4, 4, 0, 0, -1);
               this.m_145(this.f_Image_arr2_00[0][4], 150, this.f_int_arr_37[1], 2, 4, 4, 4, 0, 0, -1);
               this.m_145(this.f_Image_arr2_00[0][6], 15, this.f_int_arr_37[1], 1, 12, 3, this.f_int_158, this.f_int_arr_37[2], 0, -1);
               break;
            case 10:
               this.m_147(5);
               this.m_145(null, this.f_int_158, this.f_int_arr_37[2], 3, 0, 0, 0, 0, 0, -1);
               this.m_146(5, 4, 0, 1, 2, 0);
               break;
            case 14:
               this.m_146(5, 3, 0, 3, this.f_int_158 - 8, this.f_int_arr_37[2]);
               this.m_146(6, 3, 0, 3, this.f_int_158 + this.f_byte_arr_46[2] - 4, this.f_int_arr_37[2]);
               break;
            case 15:
               this.m_146(7, 3, 0, 2, this.f_int_158 + this.f_byte_arr_46[4] - 2, this.f_int_arr_37[2]);
               break;
            case 17:
               this.m_146(5, 3, 0, 3, this.f_int_158, this.f_int_arr_37[2]);
               this.m_146(6, 3, 0, 3, this.f_int_158 + this.f_byte_arr_46[2], this.f_int_arr_37[2]);
               this.m_146(7, 3, 0, 2, this.f_int_158 + this.f_byte_arr_46[4], this.f_int_arr_37[2]);
         }

         if (var2 >= 10 && var2 < 13) {
            this.m_145(null, 240, this.f_int_arr_37[2], 3, 0, 4, this.f_int_158 + this.f_byte_arr_46[var2 - 9 << 1], this.f_int_arr_37[2], var2 - 9, -1);
            return;
         }

         if (var2 >= 14 && var2 <= 17) {
            this.m_145(
               null,
               this.f_int_158 + this.f_byte_arr_46[var2 - 10 << 1],
               320,
               3,
               0,
               4,
               this.f_int_158 + this.f_byte_arr_46[var2 - 10 << 1],
               this.f_int_arr_37[2],
               var2 - 10,
               -1
            );
            return;
         }

         if (var2 >= 25 && var2 <= 31) {
            var1 = (var2 - 25 << 1) + 8;
            this.m_145(
               null,
               this.f_int_158 + this.f_byte_arr_46[var1 << 1],
               this.f_int_arr_37[2],
               3,
               0,
               0,
               this.f_int_158 + this.f_byte_arr_46[var1 << 1],
               this.f_int_arr_37[2],
               var1,
               -1
            );
            if (var2 < 31) {
               this.m_145(
                  null,
                  this.f_int_158 + this.f_byte_arr_46[++var1 << 1],
                  this.f_int_arr_37[2],
                  3,
                  0,
                  0,
                  this.f_int_158 + this.f_byte_arr_46[var1 << 1],
                  this.f_int_arr_37[2],
                  var1,
                  -1
               );
            }

            return;
         }
      }
   }

   private void m_144() {
      this.f_Graphics_00.setColor(16777215);
      this.f_Graphics_00.fillRect(0, 0, 240, 320);

      for (int var4 = 0; var4 < this.f_byte_29; var4++) {
         int[] var5;
         int var1 = (var5 = this.f_int_arr2_03[var4])[0];
         int var2 = var5[1];
         if (var5[2] == 3 || var5[2] == 4) {
            var1 -= this.f_byte_arr_46[var5[6] << 1];
            this.f_Graphics_00.setClip(var1 + this.f_byte_arr_46[var5[6] << 1], var2, this.f_byte_arr_46[(var5[6] << 1) + 1], 320);
            this.f_Graphics_00.drawImage(this.f_Image_arr2_00[0][7], var1, var2, 0);
         } else if (var5[2] == 2) {
            int var3 = var5[3] - var5[4];
            if (var5[5] == 1) {
               var3 = var5[3] - var3 - 2;
            }

            var1 -= var5[6] * var3;
            this.f_Graphics_00.setClip(var1 + var5[6] * var3, var2, var5[6], 320);
         }

         if (var5[2] != 3 && var5[2] != 4) {
            this.f_Graphics_00.drawImage(this.f_Image_arr_01[var4], var1, var2, 0);
         }

         this.f_Graphics_00.setClip(0, 0, 240, 320);
      }
   }

   private void m_145(Image var1, int var2, int var3, int var4, int var5, int var6, int var7, int var8, int var9, int var10) {
      if (this.f_Image_arr_01 == null) {
         this.f_Image_arr_01 = new Image[26];
      }

      if (this.f_byte_29 < 26) {
         int[] var11 = this.f_int_arr2_03[this.f_byte_29];
         if (var10 >= 0 && var10 < this.f_byte_29) {
            this.f_int_arr2_03[this.f_byte_29] = this.f_int_arr2_03[var10];
            this.f_Image_arr_01[this.f_byte_29] = this.f_Image_arr_01[var10];
            this.f_Image_arr_01[var10] = var1;
            this.f_int_arr2_03[var10] = var11;
            var11 = this.f_int_arr2_03[var10];
         } else {
            this.f_Image_arr_01[this.f_byte_29] = var1;
         }

         if (var4 == 3) {
            var11[6] = var9;
         } else {
            var9 = var1.getWidth();
            int var13 = var1.getHeight();
            if (var4 == 2) {
               var9 /= var6;
               var11[6] = var9;
            }

            var10 = 0;
            int var12 = 0;
            if ((var5 & 1) != 0) {
               var10 = 0 - var9;
            } else if ((var5 & 4) != 0) {
               var10 = 0 - (var9 >> 1);
            }

            if ((var5 & 2) != 0) {
               var12 = 0 - (var13 - 1);
            } else if ((var5 & 8) != 0) {
               var12 = 0 - ((var13 >> 1) - 1);
            }

            var2 += var10;
            var3 += var12;
            if (var4 == 1) {
               var7 += var10;
               var8 += var12;
            }
         }

         var11[0] = var2;
         var11[1] = var3;
         var11[2] = var4;
         var11[3] = var6;
         var11[4] = var7;
         var11[5] = var8;
         this.f_byte_29++;
      }
   }

   private void m_146(int var1, int var2, int var3, int var4, int var5, int var6) {
      if (var1 < this.f_byte_29) {
         int[] var7 = this.f_int_arr2_03[var1];
         if (var2 == 1) {
            int var8 = this.f_Image_arr_01[var1].getWidth();
            int var9 = this.f_Image_arr_01[var1].getHeight();
            if ((var3 & 1) != 0) {
               var5 -= var8;
            } else if ((var3 & 4) != 0) {
               var5 -= var8 >> 1;
            }

            if ((var3 & 2) != 0) {
               var6 -= var9 - 1;
            } else if ((var3 & 8) != 0) {
               var6 -= (var9 >> 1) - 1;
            }
         }

         var7[2] = var2;
         var7[3] = var4;
         var7[4] = var5;
         var7[5] = var6;
      }
   }

   private void m_147(int var1) {
      if (var1 >= 0) {
         if (var1 < this.f_byte_29) {
            int[] var2 = this.f_int_arr2_03[var1];
            this.f_int_arr2_03[var1] = this.f_int_arr2_03[--this.f_byte_29];
            this.f_int_arr2_03[this.f_byte_29] = var2;
            this.f_Image_arr_01[var1] = null;
            this.f_Image_arr_01[var1] = this.f_Image_arr_01[this.f_byte_29];
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
         } catch (Exception var3) {
         }

         this.f_HttpConnection_00 = null;
      }

      if (this.f_OutputStream_00 != null) {
         try {
            this.f_OutputStream_00.close();
         } catch (Exception var2) {
         }

         this.f_OutputStream_00 = null;
      }

      if (this.f_InputStream_01 != null) {
         try {
            this.f_InputStream_01.close();
         } catch (Exception var1) {
         }

         this.f_InputStream_01 = null;
      }

      System.gc();
   }

   private int m_149() {
      return (this.f_byte_arr_48[this.f_int_162++] & 0xFF) << 8 | this.f_byte_arr_48[this.f_int_162++] & 0xFF;
   }

   private int m_150() {
      return this.m_149() << 16 | this.m_149() & 65535;
   }

   // $VF: Could not verify finally blocks. A semaphore variable has been added to preserve control flow.
   // Please report this to the Vineflower issue tracker, at https://github.com/Vineflower/vineflower/issues with a copy of the class file (if you have the rights to distribute it!)
   private int m_151() {
      int var1 = 0;
      boolean var7 = false /* VF: Semaphore variable */;

      label183: {
         try {
            var7 = true;
            String var11 = this.f_String_06 == null ? this.f_String_arr_17[0] : this.f_String_06;
            String var4 = var11;
            String var10000;
            if (this.f_bool_33) {
               int var2 = var4.indexOf(47, 7);
               String var18 = var4.substring(var2);
               var10000 = "http://10.0.0.172:80" + var18;
            } else {
               var10000 = var4;
            }

            String var19 = var10000;
            String var3 = var11;
            int var31;
            String var12 = (var31 = var11.indexOf(47, 7)) < 7 ? var3.substring(7, var3.length()) : var3.substring(7, var31);
            this.f_HttpConnection_00 = (HttpConnection)Connector.open(var19);
            this.f_HttpConnection_00.setRequestMethod("POST");
            this.f_HttpConnection_00.setRequestProperty("Content-Type", "application/octet-stream");
            if (this.f_bool_33) {
               this.f_HttpConnection_00.setRequestProperty("X-Online-Host", var12);
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
               var1 = (byte)1;
               var7 = false;
            } else if (this.f_int_160 / 100 != 2) {
               var1 = (byte)-1;
               var7 = false;
            } else {
               var1 = 0;

               while (true) {
                  Object var20ref = null;
                  if (this.f_HttpConnection_00.getHeaderFieldKey(var1) == null) {
                     this.f_InputStream_01 = this.f_HttpConnection_00.openInputStream();
                     if ((var1 = (int)this.f_HttpConnection_00.getLength()) > 0) {
                        this.f_byte_arr_48 = null;
                        this.f_byte_arr_48 = new byte[var1];
                        int var20 = 0;

                        for (int var26 = 0; var26 < var1; var26++) {
                           var20 = this.f_InputStream_01.read();
                           this.f_byte_arr_48[var26] = (byte)var20;
                        }
                     } else {
                        ByteArrayOutputStream var23 = new ByteArrayOutputStream(2048);
                        boolean var27 = false;
                        byte[] var28 = new byte[64];

                        while ((var1 = this.f_InputStream_01.read(var28, 0, 64)) >= 0) {
                           var23.write(var28, 0, var1);
                        }

                        var23.close();
                        this.f_byte_arr_48 = var23.toByteArray();
                     }

                     this.f_InputStream_01.close();
                     this.f_InputStream_01 = null;
                     byte[] var29 = new byte[(var1 = this.f_byte_arr_48.length) >> 1];

                     for (int var24 = 0; var24 + 1 < var1; var24 += 2) {
                        var29[var24 >> 1] = (byte)(m_154(this.f_byte_arr_48[var24]) << 4 | m_154(this.f_byte_arr_48[var24 + 1]));
                     }

                     this.f_byte_arr_48 = null;
                     this.f_byte_arr_48 = var29;
                     StringBuffer var30 = new StringBuffer();
                     var1 >>= 1;

                     for (int var25 = 0; var25 < var1; var25++) {
                        var30.append(Integer.toHexString(this.f_byte_arr_48[var25] & 255));
                        var30.append(" ");
                     }

                     System.out.println("下行数据长度：" + var1 + "，内容：" + var30);
                     var1 = (byte)0;
                     var7 = false;
                     break label183;
                  }

                  this.f_HttpConnection_00.getHeaderField(var1);
                  var1++;
               }
            }
            break label183;
         } catch (Exception var8) {
            var7 = false;
         } finally {
            if (var7) {
               this.m_148();
            }
         }

         byte var10 = -1;
         this.m_148();
         return var10;
      }

      this.m_148();
      return var1;
   }

   private int m_152() {
      int var3;
      synchronized (this.f_Object_00) {
         var3 = this.f_int_161;
      }

      if (var3 == 2) {
         Thread.yield();
      }

      return var3;
   }

   private boolean m_153() {
      this.f_int_165++;
      switch (this.m_152()) {
         case -1:
            if (this.f_int_166 >= 3) {
               this.f_int_166 = 0;
               this.f_String_07 = this.f_String_arr_16[15];
               this.f_bool_33 = !this.f_bool_33;
               return true;
            } else {
               this.f_int_166++;
               this.f_int_165 = 0;
               this.f_String_07 = "正在重试(" + this.f_int_166 + ")";
               a var14 = this;
               this.m_148();
               var14.f_int_160 = 0;
               var14.f_int_165 = 0;
               if (var14.f_Object_00 == null) {
                  var14.f_Object_00 = new Object();
               }

               synchronized (var14.f_Object_00) {
                  var14.f_int_161 = 2;
               }

               var14.f_bool_31 = true;
               new Thread(var14).start();
            }
         default:
            if (this.f_int_165 > 300) {
               this.f_int_161 = 3;
               this.f_int_165 = 0;
               this.f_String_07 = "已超时，请重试。";
               this.f_bool_33 = !this.f_bool_33;
               return true;
            }

            return false;
         case 0:
            this.f_int_165 = 0;
            this.f_int_166 = 0;
            this.f_int_165 = 0;

            try {
               if (this.f_int_160 == 200) {
                  a var1 = this;
                  long var4 = 0L;
                  var1.f_int_162 = 0;
                  a var3 = var1;
                  if ((var4 = var1.m_150() << 32 | var3.m_150()) > 0L) {
                     var1.f_int_164 = 0;
                     var1.f_long_01 = var1.f_long_00;
                     var1.f_long_00 = var4;
                     var1.f_int_163 = var1.f_byte_arr_48[var1.f_int_162++];

                     for (int var2 = 0; var2 < var1.f_int_163; var2++) {
                        var3 = var1;
                        var1.f_int_162++;
                        int var18;
                        if ((var18 = var3.m_149()) != 0) {
                           int var19 = var18;
                           var3.f_int_162 += var19;
                        }
                     }
                  } else {
                     if (++var1.f_int_164 >= 2 && var1.f_long_01 != var1.f_long_00) {
                        var1.f_long_00 = var1.f_long_01;
                     }

                     synchronized (var1.f_Object_00) {
                        var1.f_int_161 = -1;
                     }

                     var1.f_String_07 = var1.f_String_arr_16[48];
                  }
               }
            } catch (Exception var12) {
               var12.printStackTrace();
            } finally {
               this.m_148();
            }

            return true;
      }
   }

   private static int m_154(int var0) {
      if (var0 >= 48 && var0 <= 57) {
         return var0 - 48;
      } else if (var0 >= 97 && var0 <= 122) {
         return var0 - 97 + 10;
      } else {
         return var0 >= 65 && var0 <= 90 ? var0 - 65 + 10 : -var0;
      }
   }

   static {
      int[] var10000 = new int[]{0, 2, 1, 3, 4, 5, 6, 7};
   }
}
