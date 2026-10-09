# API 面核对（自动生成，确定性）

扫描对象：`a.class` + `CMidlet.class` 常量池全部 Fieldref/Methodref/InterfaceMethodref；shim 覆盖判定 = 类存在且成员沿继承链可解析（解析的是 reference/shim/ 编译产物）。

- 引用总数（去重）：707
- MIDP/Nokia API（shim 提供）：69，缺失 0
- JDK API（java/*，未列入 shim 责任）：638

## shim 覆盖清单（MIDP/Nokia）

| # | owner | kind | name | desc |
|---|-------|------|------|------|
| 1 | `com/nokia/mid/ui/DirectGraphics` | imethod | `drawImage` | `(Ljavax/microedition/lcdui/Image;IIII)V` |
| 2 | `com/nokia/mid/ui/DirectUtils` | method | `getDirectGraphics` | `(Ljavax/microedition/lcdui/Graphics;)Lcom/nokia/mid/ui/DirectGraphics;` |
| 3 | `javax/microedition/io/Connection` | imethod | `close` | `()V` |
| 4 | `javax/microedition/io/Connector` | method | `open` | `(Ljava/lang/String;)Ljavax/microedition/io/Connection;` |
| 5 | `javax/microedition/io/ContentConnection` | imethod | `getLength` | `()J` |
| 6 | `javax/microedition/io/HttpConnection` | imethod | `getHeaderField` | `(I)Ljava/lang/String;` |
| 7 | `javax/microedition/io/HttpConnection` | imethod | `getHeaderField` | `(Ljava/lang/String;)Ljava/lang/String;` |
| 8 | `javax/microedition/io/HttpConnection` | imethod | `getHeaderFieldKey` | `(I)Ljava/lang/String;` |
| 9 | `javax/microedition/io/HttpConnection` | imethod | `getResponseCode` | `()I` |
| 10 | `javax/microedition/io/HttpConnection` | imethod | `setRequestMethod` | `(Ljava/lang/String;)V` |
| 11 | `javax/microedition/io/HttpConnection` | imethod | `setRequestProperty` | `(Ljava/lang/String;Ljava/lang/String;)V` |
| 12 | `javax/microedition/io/InputConnection` | imethod | `openInputStream` | `()Ljava/io/InputStream;` |
| 13 | `javax/microedition/io/OutputConnection` | imethod | `openOutputStream` | `()Ljava/io/OutputStream;` |
| 14 | `javax/microedition/lcdui/Canvas` | method | `<init>` | `()V` |
| 15 | `javax/microedition/lcdui/Canvas` | method | `hideNotify` | `()V` |
| 16 | `javax/microedition/lcdui/Canvas` | method | `repaint` | `()V` |
| 17 | `javax/microedition/lcdui/Canvas` | method | `serviceRepaints` | `()V` |
| 18 | `javax/microedition/lcdui/Canvas` | method | `setFullScreenMode` | `(Z)V` |
| 19 | `javax/microedition/lcdui/Canvas` | method | `showNotify` | `()V` |
| 20 | `javax/microedition/lcdui/Display` | method | `getDisplay` | `(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;` |
| 21 | `javax/microedition/lcdui/Display` | method | `setCurrent` | `(Ljavax/microedition/lcdui/Displayable;)V` |
| 22 | `javax/microedition/lcdui/Font` | method | `charWidth` | `(C)I` |
| 23 | `javax/microedition/lcdui/Font` | method | `getFont` | `(III)Ljavax/microedition/lcdui/Font;` |
| 24 | `javax/microedition/lcdui/Font` | method | `getHeight` | `()I` |
| 25 | `javax/microedition/lcdui/Font` | method | `stringWidth` | `(Ljava/lang/String;)I` |
| 26 | `javax/microedition/lcdui/Font` | method | `substringWidth` | `(Ljava/lang/String;II)I` |
| 27 | `javax/microedition/lcdui/Graphics` | method | `drawChar` | `(CIII)V` |
| 28 | `javax/microedition/lcdui/Graphics` | method | `drawImage` | `(Ljavax/microedition/lcdui/Image;III)V` |
| 29 | `javax/microedition/lcdui/Graphics` | method | `drawLine` | `(IIII)V` |
| 30 | `javax/microedition/lcdui/Graphics` | method | `drawRect` | `(IIII)V` |
| 31 | `javax/microedition/lcdui/Graphics` | method | `drawString` | `(Ljava/lang/String;III)V` |
| 32 | `javax/microedition/lcdui/Graphics` | method | `drawSubstring` | `(Ljava/lang/String;IIIII)V` |
| 33 | `javax/microedition/lcdui/Graphics` | method | `fillArc` | `(IIIIII)V` |
| 34 | `javax/microedition/lcdui/Graphics` | method | `fillRect` | `(IIII)V` |
| 35 | `javax/microedition/lcdui/Graphics` | method | `fillTriangle` | `(IIIIII)V` |
| 36 | `javax/microedition/lcdui/Graphics` | method | `getColor` | `()I` |
| 37 | `javax/microedition/lcdui/Graphics` | method | `setClip` | `(IIII)V` |
| 38 | `javax/microedition/lcdui/Graphics` | method | `setColor` | `(I)V` |
| 39 | `javax/microedition/lcdui/Graphics` | method | `setFont` | `(Ljavax/microedition/lcdui/Font;)V` |
| 40 | `javax/microedition/lcdui/Image` | method | `createImage` | `(II)Ljavax/microedition/lcdui/Image;` |
| 41 | `javax/microedition/lcdui/Image` | method | `createImage` | `(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;` |
| 42 | `javax/microedition/lcdui/Image` | method | `createImage` | `([BII)Ljavax/microedition/lcdui/Image;` |
| 43 | `javax/microedition/lcdui/Image` | method | `createRGBImage` | `([IIIZ)Ljavax/microedition/lcdui/Image;` |
| 44 | `javax/microedition/lcdui/Image` | method | `getGraphics` | `()Ljavax/microedition/lcdui/Graphics;` |
| 45 | `javax/microedition/lcdui/Image` | method | `getHeight` | `()I` |
| 46 | `javax/microedition/lcdui/Image` | method | `getRGB` | `([IIIIIII)V` |
| 47 | `javax/microedition/lcdui/Image` | method | `getWidth` | `()I` |
| 48 | `javax/microedition/media/Controllable` | imethod | `getControl` | `(Ljava/lang/String;)Ljavax/microedition/media/Control;` |
| 49 | `javax/microedition/media/Manager` | method | `createPlayer` | `(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;` |
| 50 | `javax/microedition/media/Player` | imethod | `close` | `()V` |
| 51 | `javax/microedition/media/Player` | imethod | `getState` | `()I` |
| 52 | `javax/microedition/media/Player` | imethod | `prefetch` | `()V` |
| 53 | `javax/microedition/media/Player` | imethod | `realize` | `()V` |
| 54 | `javax/microedition/media/Player` | imethod | `setLoopCount` | `(I)V` |
| 55 | `javax/microedition/media/Player` | imethod | `start` | `()V` |
| 56 | `javax/microedition/media/Player` | imethod | `stop` | `()V` |
| 57 | `javax/microedition/media/control/VolumeControl` | imethod | `setLevel` | `(I)I` |
| 58 | `javax/microedition/midlet/MIDlet` | method | `<init>` | `()V` |
| 59 | `javax/microedition/midlet/MIDlet` | method | `notifyDestroyed` | `()V` |
| 60 | `javax/microedition/rms/RecordEnumeration` | imethod | `destroy` | `()V` |
| 61 | `javax/microedition/rms/RecordEnumeration` | imethod | `nextRecordId` | `()I` |
| 62 | `javax/microedition/rms/RecordStore` | method | `addRecord` | `([BII)I` |
| 63 | `javax/microedition/rms/RecordStore` | method | `closeRecordStore` | `()V` |
| 64 | `javax/microedition/rms/RecordStore` | method | `deleteRecordStore` | `(Ljava/lang/String;)V` |
| 65 | `javax/microedition/rms/RecordStore` | method | `enumerateRecords` | `(Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)Ljavax/microedition/rms/RecordEnumeration;` |
| 66 | `javax/microedition/rms/RecordStore` | method | `getNumRecords` | `()I` |
| 67 | `javax/microedition/rms/RecordStore` | method | `getRecord` | `(I)[B` |
| 68 | `javax/microedition/rms/RecordStore` | method | `openRecordStore` | `(Ljava/lang/String;Z)Ljavax/microedition/rms/RecordStore;` |
| 69 | `javax/microedition/rms/RecordStore` | method | `setRecord` | `(I[BII)V` |

## ✓ 缺失 0 条

## JDK 引用（存档）

| owner | kind | name | desc |
|-------|------|------|------|
| `CMidlet` | method | `a` | `()V` |
| `CMidlet` | field | `a` | `LCMidlet;` |
| `CMidlet` | field | `a` | `La;` |
| `CMidlet` | method | `destroyApp` | `(Z)V` |
| `a` | method | `<init>` | `()V` |
| `a` | method | `A` | `()V` |
| `a` | field | `A` | `B` |
| `a` | field | `A` | `I` |
| `a` | field | `A` | `Z` |
| `a` | field | `A` | `[B` |
| `a` | field | `A` | `[I` |
| `a` | method | `B` | `()V` |
| `a` | field | `B` | `B` |
| `a` | field | `B` | `I` |
| `a` | field | `B` | `Z` |
| `a` | field | `B` | `[B` |
| `a` | field | `B` | `[I` |
| `a` | method | `C` | `()V` |
| `a` | field | `C` | `B` |
| `a` | field | `C` | `I` |
| `a` | field | `C` | `Z` |
| `a` | field | `C` | `[B` |
| `a` | field | `C` | `[I` |
| `a` | method | `D` | `()V` |
| `a` | field | `D` | `B` |
| `a` | field | `D` | `I` |
| `a` | field | `D` | `Z` |
| `a` | field | `D` | `[B` |
| `a` | field | `D` | `[I` |
| `a` | method | `E` | `()V` |
| `a` | field | `E` | `I` |
| `a` | field | `E` | `Z` |
| `a` | field | `E` | `[B` |
| `a` | field | `E` | `[I` |
| `a` | method | `F` | `()V` |
| `a` | field | `F` | `I` |
| `a` | field | `F` | `Z` |
| `a` | field | `F` | `[B` |
| `a` | field | `F` | `[I` |
| `a` | method | `G` | `()V` |
| `a` | field | `G` | `I` |
| `a` | field | `G` | `Z` |
| `a` | field | `G` | `[B` |
| `a` | field | `G` | `[I` |
| `a` | method | `H` | `()V` |
| `a` | field | `H` | `I` |
| `a` | field | `H` | `Z` |
| `a` | field | `H` | `[B` |
| `a` | field | `H` | `[I` |
| `a` | method | `I` | `()V` |
| `a` | field | `I` | `I` |
| `a` | field | `I` | `[B` |
| `a` | field | `I` | `[I` |
| `a` | field | `J` | `I` |
| `a` | field | `J` | `[B` |
| `a` | field | `J` | `[I` |
| `a` | field | `K` | `I` |
| `a` | field | `K` | `[B` |
| `a` | field | `K` | `[I` |
| `a` | field | `L` | `I` |
| `a` | field | `L` | `[B` |
| `a` | field | `L` | `[I` |
| `a` | field | `M` | `I` |
| `a` | field | `M` | `[B` |
| `a` | field | `N` | `I` |
| `a` | field | `N` | `[B` |
| `a` | field | `O` | `I` |
| `a` | field | `O` | `[B` |
| `a` | field | `P` | `I` |
| `a` | field | `P` | `[B` |
| `a` | field | `Q` | `I` |
| `a` | field | `Q` | `[B` |
| `a` | field | `R` | `I` |
| `a` | field | `R` | `[B` |
| `a` | field | `S` | `I` |
| `a` | field | `S` | `[B` |
| `a` | field | `T` | `I` |
| `a` | field | `T` | `[B` |
| `a` | field | `U` | `I` |
| `a` | field | `U` | `[B` |
| `a` | field | `V` | `I` |
| `a` | field | `V` | `[B` |
| `a` | field | `W` | `I` |
| `a` | field | `W` | `[B` |
| `a` | field | `X` | `I` |
| `a` | field | `Y` | `I` |
| `a` | field | `Z` | `I` |
| `a` | method | `a` | `()I` |
| `a` | method | `a` | `()V` |
| `a` | method | `a` | `()Z` |
| `a` | method | `a` | `(B)Z` |
| `a` | method | `a` | `(BI)V` |
| `a` | method | `a` | `(BIII)V` |
| `a` | method | `a` | `(BLjava/lang/String;BB)V` |
| `a` | method | `a` | `(BZ)V` |
| `a` | method | `a` | `(I)I` |
| `a` | method | `a` | `(I)V` |
| `a` | method | `a` | `(I)Z` |
| `a` | method | `a` | `(II)I` |
| `a` | method | `a` | `(II)V` |
| `a` | method | `a` | `(II)Z` |
| `a` | method | `a` | `(IIB)V` |
| `a` | method | `a` | `(III)I` |
| `a` | method | `a` | `(III)Ljavax/microedition/lcdui/Image;` |
| `a` | method | `a` | `(III)V` |
| `a` | method | `a` | `(IIII)I` |
| `a` | method | `a` | `(IIII)V` |
| `a` | method | `a` | `(IIII)Z` |
| `a` | method | `a` | `(IIIII)V` |
| `a` | method | `a` | `(IIIIII)V` |
| `a` | method | `a` | `(IIIIIZ)I` |
| `a` | method | `a` | `(IIZ)V` |
| `a` | method | `a` | `(IZ)I` |
| `a` | method | `a` | `(IZ)Z` |
| `a` | method | `a` | `(IZZ)Z` |
| `a` | method | `a` | `(Ljava/io/InputStream;)S` |
| `a` | method | `a` | `(Ljava/lang/String;)I` |
| `a` | method | `a` | `(Ljava/lang/String;)V` |
| `a` | method | `a` | `(Ljava/lang/String;I)V` |
| `a` | method | `a` | `(Ljava/lang/String;II)V` |
| `a` | method | `a` | `(Ljava/lang/String;III)V` |
| `a` | method | `a` | `(Ljava/lang/String;IIIIZ)V` |
| `a` | method | `a` | `(Ljava/lang/String;III[I)V` |
| `a` | method | `a` | `(Ljava/lang/String;ILjava/lang/String;)I` |
| `a` | method | `a` | `(Ljavax/microedition/lcdui/Image;I)Ljavax/microedition/lcdui/Image;` |
| `a` | method | `a` | `(Ljavax/microedition/lcdui/Image;III)I` |
| `a` | method | `a` | `(Ljavax/microedition/lcdui/Image;III)V` |
| `a` | method | `a` | `(Ljavax/microedition/lcdui/Image;IIIIII)V` |
| `a` | method | `a` | `(Ljavax/microedition/lcdui/Image;IIIIIII)V` |
| `a` | method | `a` | `(Ljavax/microedition/lcdui/Image;IIIIIIIII)V` |
| `a` | method | `a` | `(Ljavax/microedition/lcdui/Image;Ljavax/microedition/lcdui/Graphics;IIIIII)V` |
| `a` | method | `a` | `(S)V` |
| `a` | method | `a` | `(Z)I` |
| `a` | method | `a` | `(Z)V` |
| `a` | method | `a` | `(Z)Z` |
| `a` | field | `a` | `B` |
| `a` | field | `a` | `I` |
| `a` | field | `a` | `J` |
| `a` | field | `a` | `Lcom/nokia/mid/ui/DirectGraphics;` |
| `a` | field | `a` | `Ljava/io/ByteArrayOutputStream;` |
| `a` | field | `a` | `Ljava/io/DataInputStream;` |
| `a` | field | `a` | `Ljava/io/DataOutputStream;` |
| `a` | field | `a` | `Ljava/io/InputStream;` |
| `a` | field | `a` | `Ljava/io/OutputStream;` |
| `a` | field | `a` | `Ljava/lang/Object;` |
| `a` | field | `a` | `Ljava/lang/String;` |
| `a` | field | `a` | `Ljava/util/Random;` |
| `a` | field | `a` | `Ljavax/microedition/io/HttpConnection;` |
| `a` | field | `a` | `Ljavax/microedition/lcdui/Font;` |
| `a` | field | `a` | `Ljavax/microedition/lcdui/Graphics;` |
| `a` | field | `a` | `Ljavax/microedition/lcdui/Image;` |
| `a` | field | `a` | `Ljavax/microedition/media/Player;` |
| `a` | field | `a` | `Ljavax/microedition/rms/RecordEnumeration;` |
| `a` | field | `a` | `Ljavax/microedition/rms/RecordStore;` |
| `a` | field | `a` | `S` |
| `a` | field | `a` | `Z` |
| `a` | field | `a` | `[B` |
| `a` | field | `a` | `[I` |
| `a` | field | `a` | `[Ljava/lang/String;` |
| `a` | field | `a` | `[Ljavax/microedition/lcdui/Image;` |
| `a` | field | `a` | `[S` |
| `a` | field | `a` | `[Z` |
| `a` | field | `a` | `[[B` |
| `a` | field | `a` | `[[I` |
| `a` | field | `a` | `[[Ljavax/microedition/lcdui/Image;` |
| `a` | field | `a` | `[[S` |
| `a` | field | `a` | `[[Z` |
| `a` | field | `aA` | `I` |
| `a` | field | `aB` | `I` |
| `a` | field | `aC` | `I` |
| `a` | field | `aD` | `I` |
| `a` | field | `aE` | `I` |
| `a` | field | `aF` | `I` |
| `a` | field | `aG` | `I` |
| `a` | field | `aH` | `I` |
| `a` | field | `aI` | `I` |
| `a` | field | `aJ` | `I` |
| `a` | field | `aK` | `I` |
| `a` | field | `aL` | `I` |
| `a` | field | `aM` | `I` |
| `a` | field | `aN` | `I` |
| `a` | field | `aO` | `I` |
| `a` | field | `aP` | `I` |
| `a` | field | `aQ` | `I` |
| `a` | field | `aR` | `I` |
| `a` | field | `aS` | `I` |
| `a` | field | `aT` | `I` |
| `a` | field | `aU` | `I` |
| `a` | field | `aV` | `I` |
| `a` | field | `aW` | `I` |
| `a` | field | `aX` | `I` |
| `a` | field | `aY` | `I` |
| `a` | field | `aZ` | `I` |
| `a` | field | `aa` | `I` |
| `a` | field | `ab` | `I` |
| `a` | field | `ac` | `I` |
| `a` | field | `ad` | `I` |
| `a` | field | `ae` | `I` |
| `a` | field | `af` | `I` |
| `a` | field | `ag` | `I` |
| `a` | field | `ah` | `I` |
| `a` | field | `ai` | `I` |
| `a` | field | `aj` | `I` |
| `a` | field | `ak` | `I` |
| `a` | field | `al` | `I` |
| `a` | field | `am` | `I` |
| `a` | field | `an` | `I` |
| `a` | field | `ao` | `I` |
| `a` | field | `ap` | `I` |
| `a` | field | `aq` | `I` |
| `a` | field | `ar` | `I` |
| `a` | field | `as` | `I` |
| `a` | field | `at` | `I` |
| `a` | field | `au` | `I` |
| `a` | field | `av` | `I` |
| `a` | field | `aw` | `I` |
| `a` | field | `ax` | `I` |
| `a` | field | `ay` | `I` |
| `a` | field | `az` | `I` |
| `a` | method | `b` | `()I` |
| `a` | method | `b` | `()V` |
| `a` | method | `b` | `()Z` |
| `a` | method | `b` | `(B)Z` |
| `a` | method | `b` | `(I)I` |
| `a` | method | `b` | `(I)V` |
| `a` | method | `b` | `(I)Z` |
| `a` | method | `b` | `(II)V` |
| `a` | method | `b` | `(II)Z` |
| `a` | method | `b` | `(III)I` |
| `a` | method | `b` | `(III)V` |
| `a` | method | `b` | `(IIII)V` |
| `a` | method | `b` | `(IZ)Z` |
| `a` | method | `b` | `(Ljavax/microedition/lcdui/Image;III)I` |
| `a` | method | `b` | `(Ljavax/microedition/lcdui/Image;IIIIIII)V` |
| `a` | method | `b` | `(Z)V` |
| `a` | field | `b` | `B` |
| `a` | field | `b` | `I` |
| `a` | field | `b` | `J` |
| `a` | field | `b` | `Ljava/io/InputStream;` |
| `a` | field | `b` | `Ljava/lang/String;` |
| `a` | field | `b` | `Ljavax/microedition/lcdui/Image;` |
| `a` | field | `b` | `S` |
| `a` | field | `b` | `Z` |
| `a` | field | `b` | `[B` |
| `a` | field | `b` | `[I` |
| `a` | field | `b` | `[Ljava/lang/String;` |
| `a` | field | `b` | `[Ljavax/microedition/lcdui/Image;` |
| `a` | field | `b` | `[S` |
| `a` | field | `b` | `[Z` |
| `a` | field | `b` | `[[B` |
| `a` | field | `b` | `[[I` |
| `a` | field | `bA` | `I` |
| `a` | field | `bB` | `I` |
| `a` | field | `bC` | `I` |
| `a` | field | `bD` | `I` |
| `a` | field | `bE` | `I` |
| `a` | field | `bF` | `I` |
| `a` | field | `bG` | `I` |
| `a` | field | `bH` | `I` |
| `a` | field | `bI` | `I` |
| `a` | field | `bJ` | `I` |
| `a` | field | `bK` | `I` |
| `a` | field | `bL` | `I` |
| `a` | field | `bM` | `I` |
| `a` | field | `bN` | `I` |
| `a` | field | `bO` | `I` |
| `a` | field | `bP` | `I` |
| `a` | field | `bQ` | `I` |
| `a` | field | `bR` | `I` |
| `a` | field | `bS` | `I` |
| `a` | field | `bT` | `I` |
| `a` | field | `bU` | `I` |
| `a` | field | `bV` | `I` |
| `a` | field | `bW` | `I` |
| `a` | field | `bX` | `I` |
| `a` | field | `bY` | `I` |
| `a` | field | `bZ` | `I` |
| `a` | field | `ba` | `I` |
| `a` | field | `bb` | `I` |
| `a` | field | `bc` | `I` |
| `a` | field | `bd` | `I` |
| `a` | field | `be` | `I` |
| `a` | field | `bf` | `I` |
| `a` | field | `bg` | `I` |
| `a` | field | `bh` | `I` |
| `a` | field | `bi` | `I` |
| `a` | field | `bj` | `I` |
| `a` | field | `bk` | `I` |
| `a` | field | `bl` | `I` |
| `a` | field | `bm` | `I` |
| `a` | field | `bn` | `I` |
| `a` | field | `bo` | `I` |
| `a` | field | `bp` | `I` |
| `a` | field | `bq` | `I` |
| `a` | field | `br` | `I` |
| `a` | field | `bs` | `I` |
| `a` | field | `bt` | `I` |
| `a` | field | `bu` | `I` |
| `a` | field | `bv` | `I` |
| `a` | field | `bw` | `I` |
| `a` | field | `bx` | `I` |
| `a` | field | `by` | `I` |
| `a` | field | `bz` | `I` |
| `a` | method | `c` | `()I` |
| `a` | method | `c` | `()V` |
| `a` | method | `c` | `(B)Z` |
| `a` | method | `c` | `(I)I` |
| `a` | method | `c` | `(I)V` |
| `a` | method | `c` | `(I)Z` |
| `a` | method | `c` | `(II)V` |
| `a` | method | `c` | `(II)Z` |
| `a` | method | `c` | `(III)V` |
| `a` | method | `c` | `(IIII)V` |
| `a` | method | `c` | `(Z)V` |
| `a` | field | `c` | `B` |
| `a` | field | `c` | `I` |
| `a` | field | `c` | `Ljava/lang/String;` |
| `a` | field | `c` | `Ljavax/microedition/lcdui/Image;` |
| `a` | field | `c` | `S` |
| `a` | field | `c` | `Z` |
| `a` | field | `c` | `[B` |
| `a` | field | `c` | `[I` |
| `a` | field | `c` | `[Ljava/lang/String;` |
| `a` | field | `c` | `[S` |
| `a` | field | `c` | `[Z` |
| `a` | field | `c` | `[[B` |
| `a` | field | `c` | `[[I` |
| `a` | field | `ca` | `I` |
| `a` | field | `cb` | `I` |
| `a` | field | `cc` | `I` |
| `a` | field | `cd` | `I` |
| `a` | field | `ce` | `I` |
| `a` | field | `cf` | `I` |
| `a` | field | `cg` | `I` |
| `a` | field | `ch` | `I` |
| `a` | field | `ci` | `I` |
| `a` | field | `cj` | `I` |
| `a` | field | `ck` | `I` |
| `a` | method | `d` | `()I` |
| `a` | method | `d` | `()V` |
| `a` | method | `d` | `(B)Z` |
| `a` | method | `d` | `(I)I` |
| `a` | method | `d` | `(I)V` |
| `a` | method | `d` | `(I)Z` |
| `a` | method | `d` | `(II)V` |
| `a` | method | `d` | `(II)Z` |
| `a` | method | `d` | `(III)V` |
| `a` | method | `d` | `(IIII)V` |
| `a` | field | `d` | `B` |
| `a` | field | `d` | `I` |
| `a` | field | `d` | `Ljava/lang/String;` |
| `a` | field | `d` | `Ljavax/microedition/lcdui/Image;` |
| `a` | field | `d` | `S` |
| `a` | field | `d` | `Z` |
| `a` | field | `d` | `[B` |
| `a` | field | `d` | `[I` |
| `a` | field | `d` | `[Ljava/lang/String;` |
| `a` | field | `d` | `[S` |
| `a` | field | `d` | `[Z` |
| `a` | field | `d` | `[[B` |
| `a` | field | `d` | `[[I` |
| `a` | method | `e` | `()I` |
| `a` | method | `e` | `()V` |
| `a` | method | `e` | `(I)I` |
| `a` | method | `e` | `(I)V` |
| `a` | method | `e` | `(I)Z` |
| `a` | method | `e` | `(II)V` |
| `a` | method | `e` | `(III)V` |
| `a` | method | `e` | `(IIII)V` |
| `a` | field | `e` | `B` |
| `a` | field | `e` | `I` |
| `a` | field | `e` | `Ljava/lang/String;` |
| `a` | field | `e` | `Z` |
| `a` | field | `e` | `[B` |
| `a` | field | `e` | `[I` |
| `a` | field | `e` | `[Ljava/lang/String;` |
| `a` | field | `e` | `[S` |
| `a` | field | `e` | `[Z` |
| `a` | field | `e` | `[[B` |
| `a` | method | `f` | `()I` |
| `a` | method | `f` | `()V` |
| `a` | method | `f` | `(I)I` |
| `a` | method | `f` | `(I)V` |
| `a` | method | `f` | `(II)V` |
| `a` | method | `f` | `(III)V` |
| `a` | field | `f` | `B` |
| `a` | field | `f` | `I` |
| `a` | field | `f` | `Ljava/lang/String;` |
| `a` | field | `f` | `Z` |
| `a` | field | `f` | `[B` |
| `a` | field | `f` | `[I` |
| `a` | field | `f` | `[Ljava/lang/String;` |
| `a` | field | `f` | `[S` |
| `a` | field | `f` | `[Z` |
| `a` | field | `f` | `[[B` |
| `a` | method | `g` | `()V` |
| `a` | method | `g` | `(I)I` |
| `a` | method | `g` | `(I)V` |
| `a` | method | `g` | `(II)V` |
| `a` | method | `g` | `(III)V` |
| `a` | field | `g` | `B` |
| `a` | field | `g` | `I` |
| `a` | field | `g` | `Ljava/lang/String;` |
| `a` | field | `g` | `Z` |
| `a` | field | `g` | `[B` |
| `a` | field | `g` | `[I` |
| `a` | field | `g` | `[Ljava/lang/String;` |
| `a` | field | `g` | `[S` |
| `a` | field | `g` | `[Z` |
| `a` | field | `g` | `[[B` |
| `a` | method | `h` | `()V` |
| `a` | method | `h` | `(I)I` |
| `a` | method | `h` | `(I)V` |
| `a` | method | `h` | `(II)V` |
| `a` | field | `h` | `B` |
| `a` | field | `h` | `I` |
| `a` | field | `h` | `Ljava/lang/String;` |
| `a` | field | `h` | `Z` |
| `a` | field | `h` | `[B` |
| `a` | field | `h` | `[I` |
| `a` | field | `h` | `[Ljava/lang/String;` |
| `a` | field | `h` | `[S` |
| `a` | field | `h` | `[Z` |
| `a` | method | `hideNotify` | `()V` |
| `a` | method | `i` | `()V` |
| `a` | method | `i` | `(I)V` |
| `a` | method | `i` | `(II)V` |
| `a` | field | `i` | `B` |
| `a` | field | `i` | `I` |
| `a` | field | `i` | `Z` |
| `a` | field | `i` | `[B` |
| `a` | field | `i` | `[I` |
| `a` | field | `i` | `[Ljava/lang/String;` |
| `a` | field | `i` | `[S` |
| `a` | field | `i` | `[Z` |
| `a` | method | `j` | `()V` |
| `a` | method | `j` | `(I)V` |
| `a` | method | `j` | `(II)V` |
| `a` | field | `j` | `B` |
| `a` | field | `j` | `I` |
| `a` | field | `j` | `Z` |
| `a` | field | `j` | `[B` |
| `a` | field | `j` | `[I` |
| `a` | field | `j` | `[Ljava/lang/String;` |
| `a` | field | `j` | `[S` |
| `a` | field | `j` | `[Z` |
| `a` | method | `k` | `()V` |
| `a` | method | `k` | `(I)V` |
| `a` | method | `k` | `(II)V` |
| `a` | field | `k` | `B` |
| `a` | field | `k` | `I` |
| `a` | field | `k` | `Z` |
| `a` | field | `k` | `[B` |
| `a` | field | `k` | `[I` |
| `a` | field | `k` | `[Ljava/lang/String;` |
| `a` | field | `k` | `[S` |
| `a` | field | `k` | `[Z` |
| `a` | method | `l` | `()V` |
| `a` | method | `l` | `(I)V` |
| `a` | method | `l` | `(II)V` |
| `a` | field | `l` | `B` |
| `a` | field | `l` | `I` |
| `a` | field | `l` | `Z` |
| `a` | field | `l` | `[B` |
| `a` | field | `l` | `[I` |
| `a` | field | `l` | `[Ljava/lang/String;` |
| `a` | field | `l` | `[S` |
| `a` | method | `m` | `()V` |
| `a` | method | `m` | `(I)V` |
| `a` | method | `m` | `(II)V` |
| `a` | field | `m` | `B` |
| `a` | field | `m` | `I` |
| `a` | field | `m` | `Z` |
| `a` | field | `m` | `[B` |
| `a` | field | `m` | `[I` |
| `a` | field | `m` | `[Ljava/lang/String;` |
| `a` | field | `m` | `[S` |
| `a` | method | `n` | `()V` |
| `a` | method | `n` | `(I)V` |
| `a` | method | `n` | `(II)V` |
| `a` | field | `n` | `B` |
| `a` | field | `n` | `I` |
| `a` | field | `n` | `Z` |
| `a` | field | `n` | `[B` |
| `a` | field | `n` | `[I` |
| `a` | field | `n` | `[Ljava/lang/String;` |
| `a` | field | `n` | `[S` |
| `a` | method | `o` | `()V` |
| `a` | method | `o` | `(I)V` |
| `a` | field | `o` | `B` |
| `a` | field | `o` | `I` |
| `a` | field | `o` | `Z` |
| `a` | field | `o` | `[B` |
| `a` | field | `o` | `[I` |
| `a` | field | `o` | `[Ljava/lang/String;` |
| `a` | field | `o` | `[S` |
| `a` | method | `p` | `()V` |
| `a` | method | `p` | `(I)V` |
| `a` | field | `p` | `B` |
| `a` | field | `p` | `I` |
| `a` | field | `p` | `Z` |
| `a` | field | `p` | `[B` |
| `a` | field | `p` | `[I` |
| `a` | field | `p` | `[Ljava/lang/String;` |
| `a` | method | `q` | `()V` |
| `a` | method | `q` | `(I)V` |
| `a` | field | `q` | `B` |
| `a` | field | `q` | `I` |
| `a` | field | `q` | `Z` |
| `a` | field | `q` | `[B` |
| `a` | field | `q` | `[I` |
| `a` | field | `q` | `[Ljava/lang/String;` |
| `a` | method | `r` | `()V` |
| `a` | method | `r` | `(I)V` |
| `a` | field | `r` | `B` |
| `a` | field | `r` | `I` |
| `a` | field | `r` | `Z` |
| `a` | field | `r` | `[B` |
| `a` | field | `r` | `[I` |
| `a` | field | `r` | `[Ljava/lang/String;` |
| `a` | method | `s` | `()V` |
| `a` | method | `s` | `(I)V` |
| `a` | field | `s` | `B` |
| `a` | field | `s` | `I` |
| `a` | field | `s` | `Z` |
| `a` | field | `s` | `[B` |
| `a` | field | `s` | `[I` |
| `a` | method | `showNotify` | `()V` |
| `a` | method | `t` | `()V` |
| `a` | method | `t` | `(I)V` |
| `a` | field | `t` | `B` |
| `a` | field | `t` | `I` |
| `a` | field | `t` | `Z` |
| `a` | field | `t` | `[B` |
| `a` | field | `t` | `[I` |
| `a` | method | `u` | `()V` |
| `a` | method | `u` | `(I)V` |
| `a` | field | `u` | `B` |
| `a` | field | `u` | `I` |
| `a` | field | `u` | `Z` |
| `a` | field | `u` | `[B` |
| `a` | field | `u` | `[I` |
| `a` | method | `v` | `()V` |
| `a` | method | `v` | `(I)V` |
| `a` | field | `v` | `B` |
| `a` | field | `v` | `I` |
| `a` | field | `v` | `Z` |
| `a` | field | `v` | `[B` |
| `a` | field | `v` | `[I` |
| `a` | method | `w` | `()V` |
| `a` | field | `w` | `B` |
| `a` | field | `w` | `I` |
| `a` | field | `w` | `Z` |
| `a` | field | `w` | `[B` |
| `a` | field | `w` | `[I` |
| `a` | method | `x` | `()V` |
| `a` | field | `x` | `B` |
| `a` | field | `x` | `I` |
| `a` | field | `x` | `Z` |
| `a` | field | `x` | `[B` |
| `a` | field | `x` | `[I` |
| `a` | method | `y` | `()V` |
| `a` | field | `y` | `B` |
| `a` | field | `y` | `I` |
| `a` | field | `y` | `Z` |
| `a` | field | `y` | `[B` |
| `a` | field | `y` | `[I` |
| `a` | method | `z` | `()V` |
| `a` | field | `z` | `B` |
| `a` | field | `z` | `I` |
| `a` | field | `z` | `Z` |
| `a` | field | `z` | `[B` |
| `a` | field | `z` | `[I` |
| `java/io/ByteArrayInputStream` | method | `<init>` | `([B)V` |
| `java/io/ByteArrayOutputStream` | method | `<init>` | `()V` |
| `java/io/ByteArrayOutputStream` | method | `<init>` | `(I)V` |
| `java/io/ByteArrayOutputStream` | method | `close` | `()V` |
| `java/io/ByteArrayOutputStream` | method | `size` | `()I` |
| `java/io/ByteArrayOutputStream` | method | `toByteArray` | `()[B` |
| `java/io/ByteArrayOutputStream` | method | `write` | `([BII)V` |
| `java/io/DataInputStream` | method | `<init>` | `(Ljava/io/InputStream;)V` |
| `java/io/DataInputStream` | method | `read` | `([BII)I` |
| `java/io/DataInputStream` | method | `readBoolean` | `()Z` |
| `java/io/DataInputStream` | method | `readByte` | `()B` |
| `java/io/DataInputStream` | method | `readInt` | `()I` |
| `java/io/DataInputStream` | method | `readShort` | `()S` |
| `java/io/DataInputStream` | method | `skip` | `(J)J` |
| `java/io/DataOutputStream` | method | `<init>` | `(Ljava/io/OutputStream;)V` |
| `java/io/DataOutputStream` | method | `close` | `()V` |
| `java/io/DataOutputStream` | method | `write` | `(I)V` |
| `java/io/DataOutputStream` | method | `write` | `([BII)V` |
| `java/io/DataOutputStream` | method | `writeBoolean` | `(Z)V` |
| `java/io/DataOutputStream` | method | `writeByte` | `(I)V` |
| `java/io/DataOutputStream` | method | `writeInt` | `(I)V` |
| `java/io/DataOutputStream` | method | `writeShort` | `(I)V` |
| `java/io/InputStream` | method | `close` | `()V` |
| `java/io/InputStream` | method | `read` | `()I` |
| `java/io/InputStream` | method | `read` | `([BII)I` |
| `java/io/InputStream` | method | `skip` | `(J)J` |
| `java/io/OutputStream` | method | `close` | `()V` |
| `java/io/OutputStream` | method | `flush` | `()V` |
| `java/io/OutputStream` | method | `write` | `([BII)V` |
| `java/io/PrintStream` | method | `println` | `(Ljava/lang/String;)V` |
| `java/lang/Class` | method | `getResourceAsStream` | `(Ljava/lang/String;)Ljava/io/InputStream;` |
| `java/lang/Integer` | method | `parseInt` | `(Ljava/lang/String;)I` |
| `java/lang/Integer` | method | `parseInt` | `(Ljava/lang/String;I)I` |
| `java/lang/Integer` | method | `toHexString` | `(I)Ljava/lang/String;` |
| `java/lang/Math` | method | `abs` | `(I)I` |
| `java/lang/Math` | method | `min` | `(II)I` |
| `java/lang/Object` | method | `<init>` | `()V` |
| `java/lang/Object` | method | `getClass` | `()Ljava/lang/Class;` |
| `java/lang/String` | method | `<init>` | `([CII)V` |
| `java/lang/String` | method | `charAt` | `(I)C` |
| `java/lang/String` | method | `endsWith` | `(Ljava/lang/String;)Z` |
| `java/lang/String` | method | `equals` | `(Ljava/lang/Object;)Z` |
| `java/lang/String` | method | `indexOf` | `(II)I` |
| `java/lang/String` | method | `indexOf` | `(Ljava/lang/String;I)I` |
| `java/lang/String` | method | `lastIndexOf` | `(I)I` |
| `java/lang/String` | method | `length` | `()I` |
| `java/lang/String` | method | `substring` | `(I)Ljava/lang/String;` |
| `java/lang/String` | method | `substring` | `(II)Ljava/lang/String;` |
| `java/lang/String` | method | `toCharArray` | `()[C` |
| `java/lang/StringBuffer` | method | `<init>` | `()V` |
| `java/lang/StringBuffer` | method | `append` | `(I)Ljava/lang/StringBuffer;` |
| `java/lang/StringBuffer` | method | `append` | `(Ljava/lang/Object;)Ljava/lang/StringBuffer;` |
| `java/lang/StringBuffer` | method | `append` | `(Ljava/lang/String;)Ljava/lang/StringBuffer;` |
| `java/lang/StringBuffer` | method | `toString` | `()Ljava/lang/String;` |
| `java/lang/System` | method | `currentTimeMillis` | `()J` |
| `java/lang/System` | method | `gc` | `()V` |
| `java/lang/System` | field | `out` | `Ljava/io/PrintStream;` |
| `java/lang/Thread` | method | `<init>` | `(Ljava/lang/Runnable;)V` |
| `java/lang/Thread` | method | `sleep` | `(J)V` |
| `java/lang/Thread` | method | `start` | `()V` |
| `java/lang/Thread` | method | `yield` | `()V` |
| `java/lang/Throwable` | method | `printStackTrace` | `()V` |
| `java/util/Random` | method | `<init>` | `()V` |
| `java/util/Random` | method | `nextInt` | `()I` |
| `java/util/Random` | method | `setSeed` | `(J)V` |

