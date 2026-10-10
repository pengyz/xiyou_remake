//! m_122 的 sprite 类型码生成 switch 忠实移植（a.java:9417-9579 区域）。
//!
//! 每条 sprite 记录 → (param: i16, visible: bool)，随后 spawnEntity 生成。
//! 参数形态（A 级，deobf a.java:9468-9579 逐分支）：
//! - 4：e0+1
//! - 5/81：e0+1，可见
//! - 6/12：0，可见
//! - 7/8：e0 | e1<<8
//! - 9：e0
//! - 10/11/13..58/60..69（default）：0
//! - 57/59/70..73：e0+1
//! - 76/82：e0 | (e1+1)<<8，可见
//! - 77：e0 | door12[e0-1]<<8（12 联动门参数表）
//! - 78：e0 | door13[e0-1]<<8（13 联动门参数表）
//! - 83：e0，可见

/// 门参数表（deobf a.java:871/902，夹具 door12/door13.txt）。
pub struct DoorTables {
    pub door12: Vec<i32>,
    pub door13: Vec<i32>,
}

pub fn spawn_param_and_visibility(type_code: u8, extra: &[u8], doors: &DoorTables) -> (i16, bool) {
    let e0 = extra.first().map(|b| *b as i32).unwrap_or(0);
    let e1 = extra.get(1).map(|b| *b as i32).unwrap_or(0);
    // hide_after_spawn = m_122 生成分支随后置 entityVisible=false（不参与 paintEntityLayer 绘制）
    let mut hide_after_spawn = false;
    let param = match type_code {
        4 => e0 + 1,
        // 5 / 81（封印门，entity_kind::SEAL_GATE）
        5 | 81 => {
            hide_after_spawn = true;
            e0 + 1
        }
        6 | 12 => {
            hide_after_spawn = true;
            0
        }
        7 | 8 => e0 | (e1 << 8),
        9 => e0,
        57 | 59 | 70 | 71 | 72 | 73 => e0 + 1,
        76 | 82 => {
            hide_after_spawn = true;
            e0 | ((e1 + 1) << 8)
        }
        77 => e0 | (doors.door12[(e0 - 1) as usize] << 8),
        78 => e0 | (doors.door13[(e0 - 1) as usize] << 8),
        83 => {
            hide_after_spawn = true;
            e0
        }
        _ => 0,
    };
    (param as i16, hide_after_spawn)
}
