package oracle.host;

import java.io.File;
import java.io.IOException;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.List;

import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

import oracle.vt.StopSignal;
import oracle.vt.VTime;

/**
 * tick 边界钩子（注入 {@link Canvas#hooks}）：
 * serviceRepaints() 内先 paint→{@link #afterPaint}（帧哈希+截图+绘制操作流），
 * 再 {@link #preTick}（输入脚本投递），最后 {@link #postTick}（TICK 记录 +
 * 状态向量 dump，tick 预算耗尽时抛 {@link StopSignal}）。
 */
public final class TickHooks implements Canvas.Hooks {
    private final File outDir;
    private final int maxTicks;
    /** 扁平脚本：{tick, code, type, tick, code, type, ...}，type 1=press 0=release。 */
    private final int[] script;

    private int tick;
    private int scriptPos;
    private String frameLine = "FRAME none";
    private String opsBlock = "OPS count=0";
    private volatile String stopReason;

    public TickHooks(File outDir, int maxTicks, int[] script) {
        this.outDir = outDir;
        this.maxTicks = maxTicks;
        this.script = script;
    }

    public int tickCount() {
        return tick;
    }

    public String stopReason() {
        return stopReason;
    }

    public void afterPaint(Image screen) {
        try {
            int w = screen.getWidth(), h = screen.getHeight();
            int[] px = new int[w * h];
            screen.getRGB(px, 0, w, 0, 0, w, h);
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            for (int p : px) {
                md.update((byte) (p >>> 24));
                md.update((byte) (p >>> 16));
                md.update((byte) (p >>> 8));
                md.update((byte) p);
            }
            String sha = TraceSink.hex(md.digest(), 16);
            String name = String.format("frame-%04d.png", tick + 1);
            // oracle.frames=all（默认）逐帧落盘；off 只保留 trace 内的 FRAME sha（对拍内容不变，
            // 省去每 tick PNG 编码+写盘——深场景实测为主要耗时；FRAME sha 由同一像素流计算，验证力不变）
            if (!"off".equals(System.getProperty("oracle.frames", "all"))) {
                PngWriter.write(new File(outDir, name), w, h, px);
            }
            frameLine = "FRAME sha=" + sha + " w=" + w + " h=" + h + " png=" + name;
            opsBlock = Graphics.flushOps();
        } catch (IOException e) {
            frameLine = "FRAME error=" + e;
        } catch (Exception e) {
            frameLine = "FRAME error=" + e;
        }
    }

    public int[] preTick(Canvas c) {
        tick++;
        List<Integer> ev = new ArrayList<Integer>();
        while (scriptPos + 2 < script.length && script[scriptPos] <= tick) {
            if (script[scriptPos] == tick) {
                ev.add(script[scriptPos + 1]);
                ev.add(script[scriptPos + 2]);
            }
            scriptPos += 3;
        }
        int[] arr = new int[ev.size()];
        for (int i = 0; i < arr.length; i++) {
            arr[i] = ev.get(i);
        }
        return arr;
    }

    public void postTick(Canvas c) {
        Runner.flushEvents();
        StringBuilder in = new StringBuilder();
        for (int i = 0; i + 1 < script.length; i += 3) {
            if (script[i] == tick) {
                if (in.length() > 0) {
                    in.append(' ');
                }
                in.append(script[i + 2] == 1 ? "press" : "release").append('(').append(script[i + 1]).append(')');
            }
        }
        // 转储采样（oracle.dumpStride，默认 1=每 tick 全量）：全量字段转储只在
        // 采样 tick / 输入事件 tick / 末 tick 执行；FRAME sha 每 tick 恒有（帧级
        // 对拍主证据不降级）。A/B/C 同 stride ⇒ 对拍语义不变，确定性与 tick 序一致。
        int stride = Integer.getInteger("oracle.dumpStride", 1);
        boolean hadEvents = in.length() > 0;
        boolean full = stride <= 1 || hadEvents || tick % stride == 0 || tick >= maxTicks;
        TraceSink.line(String.format("TICK %04d vt=%d", tick, VTime.currentTimeMillis()));
        TraceSink.line(frameLine);
        if (full) {
            for (String l : opsBlock.split("\n")) {
                TraceSink.line("  " + l);
            }
        }
        if (in.length() > 0) {
            TraceSink.line("INPUT " + in);
        }
        if (full) {
            for (String l : StateDump.dump(c)) {
                TraceSink.line("  " + l);
            }
        }
        if (tick >= maxTicks) {
            stopReason = "max-ticks";
            throw new StopSignal("max-ticks " + maxTicks);
        }
    }
}
