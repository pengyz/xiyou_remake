# JAR 实测考证（P0）

日期：2026-10-09。工具：unzip / javap / CFR 0.152。工作副本 `analysis/game.jar`，解包 `analysis/ex/`。

## 1. 基本盘

- MIDlet：`囧囧西游之大闹天宫`，SNOWFISH（雪鲤鱼），2010-01-20，"解谜游戏"，Media-Price 8 元
- 配置：**MIDP-1.0 / CLDC-1.0**（无 GameCanvas/Sprite/TiledLayer/MMAPI 强制依赖，引擎全部自研）
- class 文件版本 45.3（Java 1.1 时代字节码）
- 138 个资源条目，仅 **2 个 class**：
  - `a.class` 154KB：`extends Canvas implements Runnable`，**411 字段 + 163 方法**，名字全部是 `a/b/c` 型同名重载
  - `CMidlet.class` 823B：MIDlet 入口（起线程跑 `a`，接管 show/hideNotify）

## 2. 混淆等级评估（关键结论）

| 混淆手段 | 是否存在 | 证据 |
|---|---|---|
| 类合并 | ✅ 全部逻辑并入单类 `a` | javap |
| 标识符混淆 | ✅ 同名重载（字段 `a` 按类型共存 20+ 次） | javap -p |
| 字符串加密 | ❌ 无 | 剧情中文、提示、DSL 脚本全部明文 |
| 控制流平坦化 | ❌ 无 | CFR 直接还原正常方法体 |
| 反射/动态加载 | ❌ 无 | API 引用表无相关项 |

→ **真实成本 = 语义命名 + 拆 god class**，不需要重型解密/去平坦化工具。
CFR 0.152 反编译成功：`analysis/decomp/a.java`，8457 行，质量干净（建议 P1 用 Vineflower 交叉比对）。

## 3. 依赖的外部 API（移植映射输入）

- `javax.microedition.lcdui.*`：Canvas/Graphics/Image/Font（drawString/charWidth/stringWidth）
- `com.nokia.mid.ui.DirectGraphics / DirectUtils`：Nokia 像素级绘制/半透明
- `javax.microedition.media.Manager/Player/VolumeControl`：播放 `.mid`（game/menu/logo*.mid）
- `javax.microedition.rms.RecordStore`（含 Enumeration/Filter/Comparator）：存档
- `javax.microedition.io.HttpConnection`：**雪鲤鱼平台登录/短信充值/查询**（服务已死 → 剔除）
- `java.util.Random`（setSeed(currentTimeMillis)）、`System.currentTimeMillis`（busy-wait 帧循环）

## 4. 屏幕与分辨率

- `setFullScreenMode(true)`，画布硬编码 **240×320**（fillRect(0,0,240,320) 等）
- 文本渲染走 MIDP 系统 Font（stringWidth/charWidth）—— 像素复刻的字体风险（R2）
- 对话字符串内嵌格式控制码：`\cFFCC00`（颜色）、`\r`（换行）→ 需要自研文本排版解析

## 5. 资源格式（已考证部分）

### 5.1 packed PNG（大部分图形资源）
`actor/face/load/end/intro/menu/ui/xtq/map/mapbg/spt*/sflogo` = **2 字节头 + 标准 PNG**。
PNG 从 offset 2 开始（魔数 89 50 4E 47）。2 字节头含义未定：
- 部分吻合 PNG 数据长度（end: LE=16694=pngsize；mapbg: 9910=pngsize），多数不吻合
- 不是宽高（与 IHDR 不符）。假设：历史长度字段/资源索引，P2 用代码考证

| 文件 | IHDR | 头 LE16 | 备注 |
|---|---|---|---|
| map | 128×208 | 11406 | 大地图 |
| mapbg | 77×320 | 9910 | 地图背景（卷轴） |
| menu | 240×295 | 24498 | 主菜单 |
| end/intro | 240×145 | – | 结局/开场 |
| ui | 64×16 | 521 | UI 元件 |
| sptmap/sptprop/sptarm/sptenemy1/sptenemy2 | 32×36/16×10/27×25/45×31/78×45 | – | 精灵图集 |

### 5.2 maplv0–54（55 层关卡，1356B/个）
`uint16 宽(LE)=26, uint16 高(LE)=26` + **26×26×2 = 1352 字节**（两张网格，各 676B）。
字节值为 tile id（出现 0,1,2,5…）。两层语义（地形/物件？）待 P2 考证。
（26×26 网格 → 屏幕 240×320 中的可视窗口 12×16 格左右，地图可卷动。）

### 5.3 sprite0–54（每层精灵描述，70–270B）
二进制，首 u16(LE)=条目数（sprite1=35，共 184B → 每条 5 字节：`07 60 01 20 00` 型）。
疑似帧矩形/图集索引表。P2 对照 `a.java` 中 `getResourceAsStream("sprite"+n)` 解析代码考证。

### 5.4 script（3861B，**明文文本**）
CRLF 文本 DSL：
```
GAME:
STLAYER=51  EDLAYER=51  HP=300  ATK=10  DEF=10  RKEY=0 BKEY=0 YKEY=0
HOT=0  SMS=0  WHOSYOURDADDY=0
END

GUTS:
0 CES_84_6_11 MOV_0_5_11 TAK_8_9 ... GUT_1
1 TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8
...
```
- `GAME:` 段 = 全局初始数值（含作弊开关 WHOSYOURDADDY）
- `GUTS:` 段 = **每层事件脚本**，编号行 = 事件；指令流由空格分隔
- 指令词表（另在 `a.java` 常量池明文出现，同源）：
  `CES_/MOV_/TAK_/DES_/ROS_/SEE_/MVS/LAY_/RES_/GUT_/GIN_/GLV_/MOT_IF/MOT_L/ADD_/SWD/END_/SMS_/POST/ADD`
- 参数形态：`TAK_8_9`（对话 8–9 号文本）、`MOV_72_3_7_1_8`（移动）、`DES_72_6_1`（消除）、
  `SEE_6_2_188_188_0`（观察/触发，带文本区间）、`GIN_1_1000`（给予）、`GUT_1`（跳层？）等 —— 语义表 P2 产出
- `a.java` 内另有大量同源明文数组：每层脚本（this.n）、全部对话文本（this.o）、教程文本、
  道具说明（带 `\c` 颜色码）、雪鲤鱼登录 UI 文案（this.q）、数值表（this.p：35 个 int）

## 6. 游戏形态（从文本可读出的玩法，供规格书核对）

55 层天宫塔（"41楼左右对称"、"44楼藏在异空间"、"太上老君在25楼"…），推箱子/解谜 + RPG 战斗
（HP/ATK/DEF、仙丹加属性、武器每10层更新），钥匙开门（红/黄/蓝）、隐藏墙/暗道、道具
（捆仙绳、芭蕉扇、玄明石、破勾、打神鞭…）、封印门需击败守卫、BOSS（杨戬、假魔王、二郎神、
天蓬元帅、玉帝），NPC 商人买卖钥匙/回血，剧情对话 + 教程关。移动宝物 4 种可不战上楼。

## 7. 计费/联网子系统（范围裁剪对象）

明文 UI 文案证实完整"雪鲤鱼平台"账号体系：手机号/密码登录、试玩、注册（短信 2 元/条）、
充值、查询记录、客服电话。原版支持**免登录试玩** → 剔除整套联网/计费后游戏本体不受影响。
