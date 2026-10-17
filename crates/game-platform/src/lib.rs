//! game-platform —— 平台 trait 与确定性类型（AGENTS §2 红线：只放 trait 与类型；
//! 真实实现（桌面/wasm 时钟、OS 熵源）在 game-desktop/game-wasm，此处仅确定性桩）。

pub mod hash;

/// 游戏时钟（对齐 oracle VTime 语义：虚拟毫秒随 tick 推进）。
pub trait GameClock {
    fn now_ms(&self) -> u64;
}

/// 75ms/tick 的虚拟时钟——与 reference/oracle 的 T-变换（System→VTime）同参数。
#[derive(Debug, Clone)]
pub struct VirtualClock {
    pub tick_ms: u64,
    pub ticks: u64,
}

impl VirtualClock {
    pub fn new(tick_ms: u64) -> Self {
        Self { tick_ms, ticks: 0 }
    }

    pub fn advance_tick(&mut self) {
        self.ticks += 1;
    }
}

impl GameClock for VirtualClock {
    fn now_ms(&self) -> u64 {
        self.ticks * self.tick_ms
    }
}

/// 游戏随机数 trait。合同：战斗零随机（docs/spec/gameplay.md R-battle-*）；
/// 唯一播种点 = 原版 `Random.setSeed(currentTimeMillis)`（state-machine.md §6），
/// 差分模式下由 VirtualClock 提供种子。
pub trait GameRng {
    fn next_i32(&mut self) -> i32;
}

/// `java.util.Random` 的 48-bit LCG 逐位复刻（JDK 规范算法）。
///
/// 差分权威：oracle T-变换把 `setSeed(System.currentTimeMillis())` 指向
/// VTime（构造时 = 0），A1/A2 门禁 trace 逐字节一致证明种子确定。
/// 消费链实测锚（A-menu-sweep）：种子 0 首对 `(>>>1)%240/(>>>1)%150` =
/// (0,148)——T73/T77 spawn 位 (0,147)/(109,248)，T80 帧 (0,119)/(103,236)
/// 逐字节吻合（`reference/oracle/_out/A-menu-sweep/trace.txt` TICK 0080）。
#[derive(Debug, Clone)]
pub struct JavaRandom {
    /// LCG 原始状态（低 48 位有效，JDK `Random.seed` 字段的位模式）。
    seed: u64,
}

const MULTIPLIER: u64 = 0x5DEECE66D;
const ADDEND: u64 = 0xB;
const MASK: u64 = (1u64 << 48) - 1;

impl JavaRandom {
    /// `new Random(); setSeed(seed)`。
    pub fn new_seeded(seed: u64) -> JavaRandom {
        JavaRandom { seed: (seed ^ MULTIPLIER) & MASK }
    }

    /// `Random.next(bits)`（protected 语义）。
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
        (self.seed >> (48 - bits)) as i32
    }

    /// `nextInt()`（= next(32)，可为负）。
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    /// 游戏侧唯一消费形态 `randomBelow(n)`：`(nextInt() >>> 1) % n`
    /// （a.java:10181；参数求值序 L2R——spawn 先 x 后 y）。
    pub fn random_below(&mut self, n: i32) -> i32 {
        ((self.next_int() as u32) >> 1) as i32 % n
    }
}

/// SplitMix64 —— 确定性、无依赖、分布均匀的最小实现（仅作差分桩；
/// 与原版 java.util.Random 算法的一致性在需要时另行考证移植）。
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    pub state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl GameRng for SplitMix64 {
    fn next_i32(&mut self) -> i32 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        z as i32
    }
}

// ---------------------------------------------------------------- RMS 记录模型（Storage 契约）

/// RMS 记录键格式 `<store>@<record_id>` 的记录号：本游戏每店单记录恒 1（docs/spec/
/// platform-contract.md §1.3）。Java 读取恒取**首条枚举记录**、从不按记录号读
/// （a.java:4457-4459 / 8978-8980 / 9260-9262）⇒ 记录号语义惰性，收敛为常量。
pub const RMS_RECORD_ID: u32 = 1;

/// 游戏 RMS 记录域（Java 原版硬编码店名：a.java:4452 "SKY_WAR"、8960 "MOT_IF"、
/// 9190 "MOT_L"+槽号 0..=5）。键域进契约而非各壳自抄，防双移植面漂移。
pub const RMS_STORES: [&str; 8] =
    ["MOT_L0", "MOT_L1", "MOT_L2", "MOT_L3", "MOT_L4", "MOT_L5", "MOT_IF", "SKY_WAR"];

/// store 名 → 记录键（`rms_key("MOT_L0") == "MOT_L0@1"`）。
pub fn rms_key(store: &str) -> String {
    format!("{store}@{RMS_RECORD_ID}")
}

/// 记录键 → store 名：取首个 `@` 之前的部分；键无 `@` 原样返回；空串入空串出。
/// 全输入域不 panic（与 shell/win `k.split('@').next().unwrap()` 同构——split 恒返回首段）。
pub fn rms_store_name(key: &str) -> &str {
    key.split('@').next().unwrap_or(key)
}

/// RMS 持久接缝：具名 store 的字节记录（docs/spec/platform-contract.md §1）。
/// 实现只在壳层（shell/win 文件版 RMS、shell/android 私有目录版）——本 crate
/// 只放 trait 与类型（AGENTS §2 红线）。与 Java RMS 的等价性论证见 spec §1.3
/// （读=首条枚举记录、写=整店替换 ⇒ 观测上"具名 store 单 blob 覆写"）。
pub trait Storage {
    /// 载入记录域内**已持久**的全部记录（缺失 store 静默缺席）。
    /// 只返回 [`RMS_STORES`] 域内记录、键 = [`rms_key`]；平台上的外来数据
    /// （如 save/ 下杂散 .bin）**不得**出现在结果里。
    fn load_all(&self) -> Vec<(String, Vec<u8>)>;

    /// 持久化一批记录：同键覆盖、新键创建（upsert）；未提及的键不动。
    /// 传入域外键 = 契约违约 ⇒ 该键丢弃 + 可记日志，**不 panic**。
    /// 任何失败 best-effort 吞掉（实现可自记日志），与 Java `catch {}` 同口径
    /// （a.java:8955-8956）；任何输入都不得 panic。
    fn persist(&self, records: &[(String, Vec<u8>)]);
}

// ---------------------------------------------------------------- ProcessControl 契约

/// 退出执行接缝（docs/spec/platform-contract.md §2）：**游戏层只发信号**
/// （`BootMachine::finished`：game-core `paint.rs:221/585`，mode 22 退场 70 拍后置位
/// = CMidlet.m_000；脚本侧 `game-core/src/script.rs:157-158` 的 `exit_application`
/// 同为信号），**执行归 shell**（消费链 shell/win `Shell::finished` →
/// `about_to_wait` 消费点）。`finished` 判定**不入 trait**——那属 game-core 信号语义。
pub trait ProcessControl {
    /// 请求平台终止应用。**正常返回**（置退出标志/退事件循环标志，进程可在随后的
    /// 壳收尾中结束；调用方不得假设进程立即消失）。幂等；不得 panic。
    /// 桌面 = 退出事件循环（进程随 main 返回而结束）；Android = Activity.finish() /
    /// ANativeActivity_finish（**不杀进程**）。
    /// 无 `'static`/Send 约束：winit 的 `el.exit()` 只能在事件循环回调内调用，
    /// 实现可为作用域薄适配器或 exit-flag 信箱（spec §2.2）。
    fn request_exit(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_clock_matches_oracle_vtime_cadence() {
        // oracle trace：TICK 0002 vt=75（reference/oracle/_out/A1/trace.txt 首行区）
        let mut c = VirtualClock::new(75);
        assert_eq!(c.now_ms(), 0);
        c.advance_tick();
        assert_eq!(c.now_ms(), 75);
        c.advance_tick();
        assert_eq!(c.now_ms(), 150);
    }

    /// java.util.Random(0) 规范向量：首 nextInt = -1155484576
    /// （JDK 实测 + `RND(seed=…)` 探针；A-menu-sweep T73 粒子反推一致）。
    #[test]
    fn java_random_seed0_vector() {
        let mut r = JavaRandom::new_seeded(0);
        assert_eq!(r.next_int(), -1155484576);
        assert_eq!(r.next_int(), -723955400);
        // randomBelow 序列（A-menu-sweep 实测锚）
        let mut r = JavaRandom::new_seeded(0);
        assert_eq!(r.random_below(240), 0);
        assert_eq!(r.random_below(150), 148);
        assert_eq!(r.random_below(240), 109);
        assert_eq!(r.random_below(150), 47);
    }

    #[test]
    fn splitmix_is_deterministic() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..16 {
            assert_eq!(a.next_i32(), b.next_i32());
        }
    }
}
