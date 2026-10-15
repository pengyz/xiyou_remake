//! RMS 存档格式解析（`loadFloorSave` a.java:9244-9300 + `saveFloorState` 对偶）
//! 与全局进度 SKY_WAR 的布局备忘（`dispatchModeEntry`（原 m_000）case 21 读档 a.java:4450-4485）。
//!
//! # MOT_L{n} 楼层存档字段序（DataInputStream 大端，a.java:9253-9287）
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
//! | 19 | bool×scriptLineCount + skip 至 128 | scriptLineFlags（a.java:9285-9291；`script_flags` 收 128 字节原样） |
//! | 20 | byte | itemStackSize + N×(type,uses) |
//! | 21 | 56×(short len + byte[len]) | 楼层触发状态 f_byte_arr2_05 |
//!
//! # SKY_WAR 全局进度（读 dispatchModeEntry case 21 a.java:4462-4474 / 写 saveGlobalProgress a.java:8931-8942）
//!
//! bool×4（optionChecked）+ byte 计数 N + N×int（f_int_arr_35）+ int×3
//! （f_int_152/153/154）。写侧计数取 `f_int_arr_34.length`、值取 f_int_arr_35
//! （a.java:8934-8938，原版两数组混用）。preset 实例：`00010000 | 07 | 7×int | 3×int`
//! = 45 字节。本模块对 SKY_WAR 既不编码也不解码（壳层赤字，completeness §3.4）。
//!
//! > 【2026-10-15 更正】原注"bool×4 + int×N + byte 计数 + N×int（尾段）+ int×3"
//! > 证伪——计数 byte 紧随 4 个 bool，其后只有一段 N×int（读写两侧字节码一致）。
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
    /// mode12 日志（f_byte_arr_14/15 对，a.java:9207-9211 写 / 9278-9281 读）
    pub journal: Vec<(u8, u8)>,
    /// scriptLineFlags 段原样（a.java:9285-9291 readBoolean = 字节≠0）：长度
    /// max(script_line_count,128)。Java 只读前 scriptLineCount（=levelScriptLines
    /// 行数 68）项、其余 skip——应用侧（load_floor_from_save）按此截取。
    pub script_flags: Vec<bool>,
    /// 道具栈（itemStackSize + 类型/次数对）
    pub item_stack_size: u8,
    pub item_stack_types: Vec<u8>,
    pub item_stack_uses: Vec<u8>,
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
/// `script_line_count` = 当前脚本行数：只决定旗标段的**字节跨度**
/// （max(count,128)，与 Java 读 count + skip(128-count) 等长；0 即按 128 读）。
/// 旗标值恒收进 [`FloorSave::script_flags`]（a.java:9285-9287）。
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
    // f_int_89 跟随对 = mode12 日志（kind, idx）
    let n_follow = c.i16();
    if n_follow < 0 {
        return Err("negative follow count".into());
    }
    let mut journal = Vec::with_capacity(n_follow as usize);
    for _ in 0..n_follow {
        journal.push((c.u8(), c.u8()));
    }
    // scriptLineFlags（a.java:9285-9291）：Java 读 scriptLineCount 个 bool 再 skip 至
    // 128——两段合计 max(count,128) 字节；值原样收进 script_flags（应用侧截前
    // scriptLineCount 项，与 `script_line_count` 实参无关——调用方普遍传 0）
    let flag_bytes = script_line_count.max(128);
    s.script_flags = (0..flag_bytes).map(|_| c.u8() != 0).collect();
    // 道具栏
    let stack = c.u8() as usize;
    s.journal = journal;
    s.item_stack_size = stack as u8;
    s.item_stack_types = Vec::with_capacity(stack);
    s.item_stack_uses = Vec::with_capacity(stack);
    for _ in 0..stack {
        s.item_stack_types.push(c.u8());
        s.item_stack_uses.push(c.u8());
    }
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
