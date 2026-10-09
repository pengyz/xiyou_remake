//! game-platform —— 平台 trait 与确定性类型（AGENTS §2 红线：只放 trait 与类型；
//! 真实实现（桌面/wasm 时钟、OS 熵源）在 game-desktop/game-wasm，此处仅确定性桩）。

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

    #[test]
    fn splitmix_is_deterministic() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..16 {
            assert_eq!(a.next_i32(), b.next_i32());
        }
    }
}
