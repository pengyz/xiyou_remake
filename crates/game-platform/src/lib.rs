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
    /// （a.java:10177；参数求值序 L2R——spawn 先 x 后 y）。
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
