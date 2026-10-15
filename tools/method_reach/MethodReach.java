import com.sun.jdi.Bootstrap;
import com.sun.jdi.Field;
import com.sun.jdi.IntegerValue;
import com.sun.jdi.Location;
import com.sun.jdi.Method;
import com.sun.jdi.ObjectReference;
import com.sun.jdi.ReferenceType;
import com.sun.jdi.Value;
import com.sun.jdi.VirtualMachine;
import com.sun.jdi.connect.Connector;
import com.sun.jdi.connect.ListeningConnector;
import com.sun.jdi.event.BreakpointEvent;
import com.sun.jdi.event.ClassPrepareEvent;
import com.sun.jdi.event.Event;
import com.sun.jdi.event.EventSet;
import com.sun.jdi.event.VMDeathEvent;
import com.sun.jdi.event.VMDisconnectEvent;
import com.sun.jdi.request.BreakpointRequest;
import com.sun.jdi.request.ClassPrepareRequest;
import com.sun.jdi.request.EventRequest;
import com.sun.jdi.request.EventRequestManager;

import java.io.File;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * MethodReach — 参考版 Java 方法触达测量（tools/method_reach.py 的 JDI 客户端）。
 *
 * <p>做法：以 JDWP 调试模式启动 oracle 宿主（同 run.py 的 Runner 命令行），
 * 在目标类（缺省 {@code a}）**每个方法的首个可执行位置**下一次性断点；命中即
 * 记录 {@code t_hit} 并删除该断点（每方法每相位至多 1 个事件 ⇒ 开销可忽略）。
 * 断点只观测不改写（字节码/输入/虚拟时钟均不变），产出的 trace 与无调试运行
 * 逐字节同构——由 method_reach.py 以 FRAME sha 序列对照现存 A-trace 复核。
 *
 * <p>t_hit 语义：命中时读 {@code Canvas.hooks.tick}（记为 k），t_hit = k + 1 =
 * **首个可能反映该次执行的 TICK 记录号**（shim Canvas.serviceRepaints 次序：
 * paint(k) → preTick(k→k+1) → keyPressed(k+1) → postTick 记 TICK k+1；
 * 逻辑在 tick 字段 = k 时运行、其效果首现于 TICK k+1 的帧）。
 *
 * <p>相位（--arm-at s1,s2…，TICK 号）：用于起点不在 T1 的对拍窗口。相位 s 在
 * {@code TickHooks.preTick} 入口且 tick 字段 == s-2 时重新布满断点 ⇒ 该相位只
 * 记录 t_hit ≥ s 的首次命中。s=1 = 类准备即布点。**前一相位于 s 处截止**（其未命中
 * 断点被删）——相位 1 不是全程；全程首次命中 = 各相位并集取最小 t_hit
 * （method_reach.py / cover_matrix.py 的 full_run_hits）。
 *
 * <p>用法：java MethodReach --out &lt;file&gt; [--class a] [--arm-at 1,240] -- &lt;目标 JVM 参数…&gt;
 * 输出（UTF-8 文本，确定性）：{@code PHASE <s>} / {@code HIT <t_hit> <name><signature>} /
 * {@code EXIT <code>} 行。
 */
public final class MethodReach {
    public static void main(String[] args) throws Exception {
        String outPath = null;
        String target = "a";
        List<Integer> arms = new ArrayList<Integer>(Arrays.asList(1));
        List<String> jvmArgs = new ArrayList<String>();
        for (int i = 0; i < args.length; i++) {
            if (args[i].equals("--")) {
                jvmArgs.addAll(Arrays.asList(args).subList(i + 1, args.length));
                break;
            } else if (args[i].equals("--out")) {
                outPath = args[++i];
            } else if (args[i].equals("--class")) {
                target = args[++i];
            } else if (args[i].equals("--arm-at")) {
                arms.clear();
                for (String s : args[++i].split(",")) {
                    arms.add(Integer.parseInt(s.trim()));
                }
            }
        }
        if (outPath == null || jvmArgs.isEmpty() || arms.get(0) != 1) {
            System.err.println("usage: MethodReach --out <file> [--class a] [--arm-at 1,...] -- <jvm args>");
            System.exit(2);
        }

        ListeningConnector lc = null;
        for (Connector c : Bootstrap.virtualMachineManager().listeningConnectors()) {
            if (c.name().equals("com.sun.jdi.SocketListen")) {
                lc = (ListeningConnector) c;
            }
        }
        Map<String, Connector.Argument> ca = lc.defaultArguments();
        ca.get("port").setValue("0");
        ca.get("localAddress").setValue("127.0.0.1");
        String addr = lc.startListening(ca);

        List<String> cmd = new ArrayList<String>();
        cmd.add(System.getProperty("java.home") + File.separator + "bin" + File.separator + "java");
        cmd.add("-agentlib:jdwp=transport=dt_socket,server=n,suspend=y,address=" + addr);
        cmd.addAll(jvmArgs);
        File outFile = new File(outPath);
        File dir = outFile.getAbsoluteFile().getParentFile();
        ProcessBuilder pb = new ProcessBuilder(cmd);
        pb.redirectOutput(new File(dir, "target-stdout.txt"));
        pb.redirectError(new File(dir, "target-stderr.txt"));
        Process proc = pb.start();
        VirtualMachine vm = lc.accept(ca);
        lc.stopListening(ca);

        EventRequestManager erm = vm.eventRequestManager();
        ClassPrepareRequest cpr = erm.createClassPrepareRequest();
        cpr.addClassFilter(target);
        cpr.setSuspendPolicy(EventRequest.SUSPEND_EVENT_THREAD);
        cpr.enable();
        ClassPrepareRequest cprHooks = null;
        if (arms.size() > 1) {
            cprHooks = erm.createClassPrepareRequest();
            cprHooks.addClassFilter("oracle.host.TickHooks");
            cprHooks.setSuspendPolicy(EventRequest.SUSPEND_EVENT_THREAD);
            cprHooks.enable();
        }

        List<String> lines = new ArrayList<String>();
        ReferenceType targetType = null;
        List<BreakpointRequest> live = new ArrayList<BreakpointRequest>();
        BreakpointRequest preTickBp = null;
        int phase = 0; // 当前相位下标（arms[phase]）
        Map<String, Integer> seen = new LinkedHashMap<String, Integer>();
        Field hooksField = null;
        Field tickField = null;
        ReferenceType canvasType = null;

        boolean done = false;
        vm.resume();
        while (!done) {
            EventSet set = vm.eventQueue().remove();
            for (Event ev : set) {
                if (ev instanceof ClassPrepareEvent) {
                    ReferenceType rt = ((ClassPrepareEvent) ev).referenceType();
                    if (rt.name().equals(target)) {
                        targetType = rt;
                        lines.add("PHASE " + arms.get(0));
                        armAll(erm, rt, live);
                    } else if (rt.name().equals("oracle.host.TickHooks")) {
                        for (Method m : rt.methodsByName("preTick")) {
                            preTickBp = erm.createBreakpointRequest(m.location());
                            preTickBp.setSuspendPolicy(EventRequest.SUSPEND_EVENT_THREAD);
                            preTickBp.enable();
                        }
                    }
                } else if (ev instanceof BreakpointEvent) {
                    BreakpointEvent be = (BreakpointEvent) ev;
                    if (canvasType == null) {
                        canvasType = vm.classesByName("javax.microedition.lcdui.Canvas").get(0);
                        hooksField = canvasType.fieldByName("hooks");
                    }
                    ObjectReference hooks = (ObjectReference) canvasType.getValue(hooksField);
                    if (tickField == null) {
                        tickField = hooks.referenceType().fieldByName("tick");
                    }
                    int tick = ((IntegerValue) hooks.getValue(tickField)).value();
                    if (be.request() == preTickBp) {
                        int next = phase + 1;
                        if (next < arms.size() && tick == arms.get(next) - 2) {
                            for (BreakpointRequest r : live) {
                                erm.deleteEventRequest(r);
                            }
                            live.clear();
                            seen.clear();
                            phase = next;
                            lines.add("PHASE " + arms.get(phase));
                            armAll(erm, targetType, live);
                            if (phase == arms.size() - 1) {
                                erm.deleteEventRequest(preTickBp);
                                preTickBp = null;
                            }
                        }
                    } else {
                        Method m = be.location().method();
                        String key = m.name() + m.signature();
                        if (!seen.containsKey(key)) {
                            seen.put(key, tick + 1);
                            lines.add("HIT " + (tick + 1) + " " + key);
                        }
                        erm.deleteEventRequest(be.request());
                        live.remove(be.request());
                    }
                } else if (ev instanceof VMDeathEvent || ev instanceof VMDisconnectEvent) {
                    done = true;
                }
            }
            if (!done) {
                set.resume();
            }
        }
        int code = proc.waitFor();
        lines.add("EXIT " + code);
        PrintWriter pw = new PrintWriter(outFile, StandardCharsets.UTF_8.name());
        for (String l : lines) {
            pw.println(l);
        }
        pw.close();
        System.exit(0);
    }

    /** 目标类每个有字节码的方法首位置下一次性断点。 */
    private static void armAll(EventRequestManager erm, ReferenceType rt, List<BreakpointRequest> live) {
        for (Method m : rt.methods()) {
            if (m.isAbstract() || m.isNative()) {
                continue;
            }
            Location loc = m.location();
            if (loc == null || loc.codeIndex() < 0) {
                continue;
            }
            BreakpointRequest r = erm.createBreakpointRequest(loc);
            r.setSuspendPolicy(EventRequest.SUSPEND_EVENT_THREAD);
            r.enable();
            live.add(r);
        }
    }
}
