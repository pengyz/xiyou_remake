//! 商店购买规则（S3-C：§5.7 遗留验收 3——StatPct 饱和阈值闭式）。
//!
//! 提案 §3.5 口径：StatPct 为 **+pct% 整数复利**，每次购买
//! `new = atk + floor(atk * pct / 100)`（整数除法逐次模拟，不可除法折算）。
//! H6 已拍板：每次购买成本 = 1 步（步数成本，solver 侧按行动计）。
//!
//! 饱和点闭式：单次购买增益 `floor(atk*pct/100)` 为 0 当且仅当
//! `atk*pct < 100`，即 `atk <= ceil(100/pct) - 1`。
//! pct=3 ⇒ 阈值 33：atk ≤ 33 时购买是纯浪费（+0 属性仍付 1 步）。

#[cfg(test)]
mod tests {
    use super::super::shop::{stat_pct_compound_step, stat_pct_zero_gain_threshold};

    #[test]
    fn zero_gain_threshold_closed_form() {
        assert_eq!(stat_pct_zero_gain_threshold(3), 33);
        assert_eq!(stat_pct_zero_gain_threshold(50), 1);
        assert_eq!(stat_pct_zero_gain_threshold(100), 0);
        assert_eq!(stat_pct_zero_gain_threshold(1), 99);
    }

    #[test]
    fn compound_step_matches_floor_semantics() {
        assert_eq!(stat_pct_compound_step(10, 3), 10); // 30/100=0 ⇒ 无增益
        assert_eq!(stat_pct_compound_step(33, 3), 33); // 99/100=0 ⇒ 阈值内
        assert_eq!(stat_pct_compound_step(34, 3), 35); // 102/100=1 ⇒ +1
        assert_eq!(stat_pct_compound_step(5000, 3), 5150);
    }

    #[test]
    fn saturates_at_i32_max() {
        assert_eq!(stat_pct_compound_step(i32::MAX, 3), i32::MAX);
        // 中间溢出边界：saturating_mul 先饱和再 /100（与 S2 solver 侧继承语义逐位一致）
        assert_eq!(stat_pct_compound_step(i32::MAX / 2, 100), 1_095_216_659);
    }
}

/// 单次整数复利步：`atk + floor(atk * pct / 100)`，i32 饱和防溢出。
pub fn stat_pct_compound_step(atk: i32, pct: i32) -> i32 {
    atk.saturating_add(atk.saturating_mul(pct) / 100)
}

/// 零增益阈值闭式：`ceil(100 / pct) - 1`。atk ≤ 阈值时每次购买增益为 0。
pub fn stat_pct_zero_gain_threshold(pct: i32) -> i32 {
    (100 + pct - 1) / pct - 1
}
