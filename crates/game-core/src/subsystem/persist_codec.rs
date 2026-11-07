//! persist_codec face（P2 批4 终形）——PersistCodec：存档格式编解码面（spec §3.1 注④=只管格式编解码，行为不入）。
//!
//! 形照 spec §3.1：`fn serialize(&self, st: &SceneState) -> Blob` + `fn deserialize(&self, blob: &Blob) -> SceneState`
//! （deserialize 双向=plan 批4/批0 F2 注记①）。
//!
//! 【SceneState 终形】= `save::FloorSave`（MOT_L 槽位记录的域切片状态集 23 字段族）——批0 占位文档
//! 「PersistState 族切片集」实指此族：跨域切片（menu difficulty/persist floor 族/entity facing·cell/
//! player 数值·道具栈/item 装备/alchemy/script 旗标/journal/56 层态），与 PersistState **行为面**
//! （scene_persist.rs load_floor/change_floor 等）不入本面（plan 批4 明文）。
//!
//! 【字节布局】=serialize_slot_blob 旧写序（a.java:9182-9244 saveFloorState）逐字保真：
//! 7×u8 头 → cell_x/cell_y i16 BE → hp/atk/def i32 BE → 三钥匙 i16 BE → gold i32 BE → alchemy i16 BE
//! → journal 计数 i16+对 → 128×u8 旗标 → 栈计数 u8+对 → 56 层态（i16 长+块/None=-1i16）。
//!
//! 【HostCtx 无 persist 面】（批4 披露）：HostCtx 余 10 方法无存档消费，界收窄/实现者拆 N/A。
//!
//! 【盲区清单】见 [`PERSIST_BLIND_ZONES`]（plan:69 点名项 min/maxFloor/pendingFloor/wentDown 逐条对账）。

/// RMS 字节 blob（Vec<u8> newtype——save.rs RMS 字节 B 级）。
pub struct Blob(pub Vec<u8>);

/// 域切片状态集（MOT_L 槽位记录）——终形=`save::FloorSave` 别名（23 字段族，单源在 save.rs）。
pub type SceneState = crate::save::FloorSave;

/// spec §3.1：持久化编解码（格式面——行为不入）。
pub trait PersistCodec {
    fn serialize(&self, st: &SceneState) -> Blob;
    fn deserialize(&self, blob: &Blob) -> SceneState;
}

/// 持久化编解码器（无状态 ZST——格式纯函数；批10 前 Coordinator 调用面内联构造）。
pub struct PersistCoder;

impl PersistCodec for PersistCoder {
    /// MOT_L 槽位记录编码（writeFloorSave 旧写序 a.java:9182-9244——逐字节保真）。
    fn serialize(&self, st: &SceneState) -> Blob {
        let mut o = Vec::new();
        o.push(st.difficulty_index as u8);
        o.push(st.current_floor as u8);
        o.push(st.min_floor_reached as u8);
        o.push(st.max_floor_reached as u8);
        o.push(st.facing_direction as u8);
        o.push(st.equipped_weapon_type as u8); // equippedWeaponType（a.java:9192 写真实值）
        o.push(st.equipped_armor_type as u8); // equippedArmorType（a.java:9193）
        // writeShort(playerCellX/Y)（a.java:9197-9198）：**2 字节**——旧写法 i16 BE
        o.extend_from_slice(&st.player_cell_x.to_be_bytes());
        o.extend_from_slice(&st.player_cell_y.to_be_bytes());
        for v in [st.player_hp, st.player_atk, st.player_def] {
            o.extend_from_slice(&v.to_be_bytes());
        }
        for v in [st.yellow_keys, st.blue_keys, st.red_keys] {
            o.extend_from_slice(&v.to_be_bytes());
        }
        o.extend_from_slice(&st.gold.to_be_bytes());
        // writeShort(alchemyUpgradeCount)（a.java:9206；旧写法恒 0）
        o.extend_from_slice(&st.alchemy_upgrades.to_be_bytes());
        // writeShort(inventoryEntryCount) + f_byte_arr_14/15 对（a.java:9207-9212）= mode12 日志
        o.extend_from_slice(&(st.journal.len() as i16).to_be_bytes());
        for &(kind, idx) in &st.journal {
            o.push(kind);
            o.push(idx);
        }
        // 128 字节脚本旗标（不足补 false）
        for k in 0..128 {
            let v = st.script_flags.get(k).copied().unwrap_or(false);
            o.push(v as u8);
        }
        // 道具栈
        o.push(st.item_stack_size);
        for k in 0..st.item_stack_size as usize {
            o.push(st.item_stack_types[k]);
            o.push(st.item_stack_uses[k]);
        }
        // 56 层触发状态
        for k in 0..56usize {
            match st.floor_states.get(k) {
                Some(Some(d)) => {
                    o.extend_from_slice(&(d.len() as i16).to_be_bytes());
                    o.extend_from_slice(d);
                }
                _ => o.extend_from_slice(&(-1i16).to_be_bytes()),
            }
        }
        Blob(o)
    }

    /// MOT_L 槽位记录解码——读侧单源=`save::parse_floor_save`（a.java:9260-9308 loadFloorSave 读序）。
    /// 畸形 blob panic（现状 handoff.rs:192 unwrap 同语义；serialize 侧布局保证对称）。
    fn deserialize(&self, blob: &Blob) -> SceneState {
        crate::save::parse_floor_save(&blob.0, 0).expect("MOT_L 槽位记录可解析")
    }
}

/// persist 盲区清单（dump 盲区——state_dump 无对应 FLD；plan:69 点名项逐条对账）。
///
/// - minFloor：写点 scene_persist.rs:381 change_floor 直写/:35 setter（script_ops.rs:95 调用）/scene.rs:581 载入面——dump 盲区（state_dump 无 minFloor）
/// - maxFloor：scene_persist.rs:383 change_floor 直写（:384 note_max_floor 水位 f_int_154 a.java:7163-7164）/:36 setter（script_ops.rs:96）/scene.rs:582——dump 盲区
/// - pendingFloor：scene_persist.rs:386 change_floor 直写/:37 setter（script_ops.rs:67）——dump 盲区
/// - wentDown：scene_persist.rs:387 change_floor（down 形参——型 7 上楼传 false）——dump 盲区
/// - 【评估其余】currentFloor=state_dump FLD currentFloor 覆盖非盲区；global/global_dirty/floor_assets/
///   saved_slot_blobs/preset_mot_if/saved_floor_states/journal=存档/资产面状态（floor_states·journal 已入
///   本面序列化非空转）——dump 盲区登记此备查。
pub const PERSIST_BLIND_ZONES: &[&str] = &[
    "minFloor（min_floor）：change_floor 直写 scene_persist.rs:381+setter :35（script_ops.rs:95）+载入 scene.rs:581——dump 盲区",
    "maxFloor（max_floor）：change_floor 直写 :383（:384 note_max_floor 水位 a.java:7163-7164）+setter :36（script_ops.rs:96）+载入 scene.rs:582——dump 盲区",
    "pendingFloor（pending_floor）：change_floor 直写 :386+setter :37（script_ops.rs:67）——dump 盲区",
    "wentDown（went_down）：change_floor :387（down 形参——型 7 上楼传 false）——dump 盲区",
    "评估其余（global/global_dirty/floor_assets/saved_slot_blobs/preset_mot_if/saved_floor_states/journal）：存档/资产面状态（floor_states·journal 已入序列化面非空转）——dump 盲区备查；currentFloor=FLD 覆盖非盲区",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// 【P2 批4 红测试】存读档往返夹具（plan 批4 判据「存读档往返夹具+save.rs 联测」）：
    /// face.serialize(st) → save::parse_floor_save（读侧单源）逐字段相等。
    #[test]
    fn p2b4_roundtrip_floor_save() {
        let st = crate::save::FloorSave {
            difficulty_index: 3,
            current_floor: 12,
            min_floor_reached: 1,
            max_floor_reached: 40,
            facing_direction: 2,
            equipped_weapon_type: 11,
            equipped_armor_type: 6,
            player_cell_x: 168,
            player_cell_y: -7,
            player_hp: 500,
            player_atk: 120,
            player_def: 80,
            yellow_keys: 2,
            blue_keys: 1,
            red_keys: 0,
            gold: 12345,
            alchemy_upgrades: 4,
            journal: vec![(1, 2), (3, 255)],
            script_flags: (0..128).map(|k| k % 3 == 0).collect(),
            item_stack_size: 3,
            item_stack_types: vec![25, 41, 42],
            item_stack_uses: vec![7, 0, 3],
            floor_states: (0..56)
                .map(|k| if k % 5 == 0 { Some(vec![k as u8, 0xAB]) } else { None })
                .collect(),
        };
        let blob = PersistCoder.serialize(&st);
        let back = crate::save::parse_floor_save(&blob.0, 0).expect("blob 可解析");
        assert_eq!(back, st, "存读档往返夹具：serialize→save::parse_floor_save 逐字段相等");
    }

    /// 【P2 批4 红测试】字节布局钉（serialize_slot_blob 旧写布局逐字保真——a.java:9182-9244）。
    #[test]
    fn p2b4_layout_offsets() {
        let st = SceneState {
            difficulty_index: 0x11,
            current_floor: 0x22,
            min_floor_reached: 0x33,
            max_floor_reached: 0x44,
            facing_direction: 0x55,
            equipped_weapon_type: 0x66,
            equipped_armor_type: 0x77,
            player_cell_x: 0x1234,
            player_cell_y: -2,
            player_hp: 0x01020304,
            player_atk: 5,
            player_def: 6,
            yellow_keys: 7,
            blue_keys: 8,
            red_keys: 9,
            gold: 10,
            alchemy_upgrades: 0x0A0B,
            journal: vec![],
            script_flags: vec![false; 128],
            item_stack_size: 0,
            item_stack_types: vec![],
            item_stack_uses: vec![],
            floor_states: vec![None; 56],
        };
        let b = PersistCoder.serialize(&st).0;
        // 7×u8 头
        assert_eq!(&b[0..7], &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77], "7×u8 头序");
        // cell_x/cell_y i16 BE（a.java:9197-9198 writeShort 旧写）
        assert_eq!(&b[7..9], &0x1234i16.to_be_bytes(), "cell_x BE i16");
        assert_eq!(&b[9..11], &(-2i16).to_be_bytes(), "cell_y BE i16");
        // hp i32 BE @11
        assert_eq!(&b[11..15], &0x01020304i32.to_be_bytes(), "hp BE i32");
        // alchemy i16 BE @33
        assert_eq!(&b[33..35], &0x0A0Bi16.to_be_bytes(), "alchemy BE i16");
        // journal 计数 @35（空）
        assert_eq!(&b[35..37], &0i16.to_be_bytes(), "journal 计数=0");
        // 128 旗标段 @37..165 全 0
        assert!(b[37..165].iter().all(|&v| v == 0), "128 旗标段");
        // 栈计数 @165=0
        assert_eq!(b[165], 0, "item_stack_size=0");
        // 56 层态全 None=56×(-1i16 BE)
        assert_eq!(b.len(), 278, "最小记录长=7+2+2+12+6+2+2+2+128+1+56×2=278");
        for k in 0..56 {
            assert_eq!(&b[166 + k * 2..168 + k * 2], &(-1i16).to_be_bytes(), "层态 None=-1i16");
        }
    }

    /// 【P2 批4 红测试】盲区登记 plan:69 点名项逐条对账（批3 红1 教训——点名项机器断言）。
    #[test]
    fn p2b4_blind_zones_cover_plan_named() {
        let joined = PERSIST_BLIND_ZONES.join("\n");
        for name in ["minFloor", "maxFloor", "pendingFloor", "wentDown"] {
            assert!(joined.contains(name), "persist 盲区漏登记 plan:69 点名项：{name}");
        }
    }
}
