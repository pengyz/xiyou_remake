//! RMS 存档格式解析（`loadFloorSave` a.java:9234-9296 + `saveFloorState` 对偶）
//! 与全局进度（`m_000` case 21 的 SKY_WAR 读档 a.java:4427-4470）。
//!
//! # MOT_L{n} 楼层存档字段序（DataInputStream 大端，a.java:9243-9283）
//!
//! | # | 类型 | 字段 |
//! |---|---|---|
//! | 1-7 | byte×7 | difficultyIndex / currentFloor / minFloorReached / maxFloorReached / facingDirection / equippedWeaponType / equippedArmorType |
//! | 8-9 | short×2 | playerCellX / playerCellY（pixel = cell<<5） |
//! | 10-12 | int×3 | playerHp / playerAtk / playerDef |
//! | 13-15 | short×3 | yellowKeyCount / blueKeyCount / redKeyCount |
//! | 16 | int | goldAmount |
//! | 17 | short | alchemyUpgradeCount |
//! | 18 | short | f_int_89（跟随对数）+ N×(byte,byte)（f_byte_arr_14/15） |
//! | 19 | bool×scriptLineCount + skip 至 128 | scriptLineFlags |
//! | 20 | byte | itemStackSize + N×(type,uses) |
//! | 21 | 56×(short len + byte[len]) | 楼层触发状态 f_byte_arr2_05 |
//!
//! # SKY_WAR 全局进度（a.java:4439-4451）
//!
//! bool×4（optionChecked）+ int×N（f_int_arr_35）+ byte 计数 + N×int
//! （f_int_arr_35 尾段）+ int×3（f_int_152/153/154）。
//!
//! oracle preset 注入（`oracle.preset` 属性 → RecordStore shim 预置，
//! `reference/oracle/scenarios/preset-floor1.preset`）：`库名@recordId=hex`。

/// 楼层存档字段（MOT_L{n}）。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FloorSave {
    pub difficulty_index: i8,
    pub current_floor: i8,
    pub min_floor_reached: i8,
    pub max_floor_reached: i8,
    pub facing_direction: i8,
    pub equipped_weapon_type: i8,
    pub equipped_armor_type: i8,
    pub player_cell_x: i16,
    pub player_cell_y: i16,
    pub player_hp: i32,
    pub player_atk: i32,
    pub player_def: i32,
    pub yellow_keys: i16,
    pub blue_keys: i16,
    pub red_keys: i16,
    pub gold: i32,
    pub alchemy_upgrades: i16,
    /// 楼层触发状态（56 层变长块）。
    pub floor_states: Vec<Option<Vec<u8>>>,
}

/// 大端读取游标。
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn u8(&mut self) -> u8 {
        let v = self.data[self.pos];
        self.pos += 1;
        v
    }
    fn i8(&mut self) -> i8 {
        self.u8() as i8
    }
    fn u16(&mut self) -> u16 {
        let v = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        v
    }
    fn i16(&mut self) -> i16 {
        self.u16() as i16
    }
    fn i32(&mut self) -> i32 {
        let v = i32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        v
    }
    fn skip(&mut self, n: usize) {
        self.pos += n;
    }
}

/// 解析 MOT_L{n} 记录（`loadFloorSave` 的读序）。
///
/// `script_line_count` = 当前脚本行数（skip 计算用；oracle 场景 0 即可）。
pub fn parse_floor_save(data: &[u8], script_line_count: usize) -> Result<FloorSave, String> {
    let mut c = Cursor { data, pos: 0 };
    let mut s = FloorSave::default();
    s.difficulty_index = c.i8();
    s.current_floor = c.i8();
    s.min_floor_reached = c.i8();
    s.max_floor_reached = c.i8();
    s.facing_direction = c.i8();
    s.equipped_weapon_type = c.i8();
    s.equipped_armor_type = c.i8();
    s.player_cell_x = c.i16();
    s.player_cell_y = c.i16();
    s.player_hp = c.i32();
    s.player_atk = c.i32();
    s.player_def = c.i32();
    s.yellow_keys = c.i16();
    s.blue_keys = c.i16();
    s.red_keys = c.i16();
    s.gold = c.i32();
    s.alchemy_upgrades = c.i16();
    // f_int_89 跟随对
    let n_follow = c.i16();
    if n_follow < 0 {
        return Err("negative follow count".into());
    }
    c.skip(n_follow as usize * 2);
    // scriptLineFlags + skip 至 128 字节
    c.skip(script_line_count);
    if script_line_count < 128 {
        c.skip(128 - script_line_count);
    }
    // 道具栏
    let stack = c.u8() as usize;
    c.skip(stack * 2);
    // 56 层触发状态
    for _ in 0..56 {
        let len = c.i16();
        if len > 0 {
            let start = c.pos;
            c.skip(len as usize);
            s.floor_states.push(Some(data[start..start + len as usize].to_vec()));
        } else {
            s.floor_states.push(None);
        }
    }
    Ok(s)
}

/// 解析 preset 文件文本（`<store>@<recordId>=<hex>` 行）。
pub fn parse_preset(text: &str) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, hex)) = line.split_once('=') {
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("bad hex"))
                .collect();
            out.push((key.to_string(), data));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// preset-floor1.preset 的 MOT_L0 首段逐字段复算（python struct 对照）：
    /// 00 01 01 01 00 00 00 | 0003 000a | 000001f4 0000001e 0000001e |
    /// 0001 0000 0000 | 00000064 | 0000 | ...
    #[test]
    fn mot_l0_head_fields() {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../reference/oracle/scenarios/preset-floor1.preset"),
        )
        .unwrap();
        let stores = parse_preset(&text);
        let mot_l0 = stores
            .iter()
            .find(|(k, _)| k.starts_with("MOT_L0"))
            .map(|(_, v)| v.clone())
            .expect("MOT_L0");
        let s = parse_floor_save(&mot_l0, 0).unwrap();
        // hex 首段：00 01 01 01 00 00 00 | 0003 000a | 000001f4 ...
        assert_eq!(s.difficulty_index, 0);
        assert_eq!(s.current_floor, 1);
        assert_eq!(s.min_floor_reached, 1);
        assert_eq!(s.max_floor_reached, 1);
        assert_eq!(s.facing_direction, 0);
        assert_eq!(s.equipped_weapon_type, 0);
        assert_eq!(s.equipped_armor_type, 0);
        assert_eq!((s.player_cell_x, s.player_cell_y), (3, 10));
        assert_eq!((s.player_hp, s.player_atk, s.player_def), (500, 30, 30));
        assert_eq!((s.yellow_keys, s.blue_keys, s.red_keys), (1, 0, 0));
        assert_eq!(s.gold, 100);
        assert_eq!(s.alchemy_upgrades, 0);
    }
}
