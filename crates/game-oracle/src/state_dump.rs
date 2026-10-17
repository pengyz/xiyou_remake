//! 状态向量镜像（迭代 72，层次二工具）：把 GameScene 的状态按 **oracle
//! trace 同名同格式** 导出为 `name = value` 行，供 tools/statediff.py 与
//! A trace 逐 tick 逐字段差分。
//!
//! 值格式对齐 a 侧 StateDump.java：
//! - 标量：Java toString（byte/short 有符号十进制、boolean true/false）
//! - 原始数组：`ARR[len] sha=#XXXXXXXXXXXXXXXX head=e0,...,e15`
//!   （sha = 每元素按**符号扩展 long** 取 8 字节大端拼接后 SHA-256 前 8 字节；
//!   head = 前 16 元素，byte/short 有符号十进制、boolean true/false）
//!
//! 已知不 对齐项（differ 会标记，属预期）：walkPathBuffer Java 恒 100 槽
//! 含陈旧尾，Rust 为活跃路径变长Vec。
use game_core::scene::GameScene;
use game_platform::hash::sha256_hex;

/// 一条导出项：语义名（= 改名类字段名，fields.py 可映射）+ 值。Java 类型由构造函数区分
/// （e_int=int / e_byte=byte / e_bool=boolean / e_int_arr=[I / e_short_arr=[S /
/// e_byte_arr=[B / e_bool_arr=[Z），只决定值的格式化，不进导出行。
struct Entry {
    name: &'static str,
    value: String,
}

fn e_int(name: &'static str, v: i32) -> Entry {
    Entry { name, value: v.to_string() }
}
fn e_byte(name: &'static str, v: i8) -> Entry {
    Entry { name, value: v.to_string() }
}
fn e_bool(name: &'static str, v: bool) -> Entry {
    Entry { name, value: if v { "true" } else { "false" }.into() }
}
fn e_int_arr(name: &'static str, v: &[i32]) -> Entry {
    Entry { name, value: arr_str(v.iter().map(|e| (*e as i64, e.to_string()))) }
}
fn e_short_arr(name: &'static str, v: &[i16]) -> Entry {
    Entry { name, value: arr_str(v.iter().map(|e| (*e as i64, e.to_string()))) }
}
fn e_byte_arr(name: &'static str, v: &[u8]) -> Entry {
    // Java byte[] 元素经 Array.get → Byte（有符号）
    Entry {
        name,
        value: arr_str(v.iter().map(|e| (*e as i8 as i64, (*e as i8).to_string()))),
    }
}
fn e_bool_arr(name: &'static str, v: &[bool]) -> Entry {
    Entry {
        name,
        value: arr_str(v.iter().map(|e| (*e as i64, if *e { "true" } else { "false" }.into()))),
    }
}

/// sha = 元素符号扩展 long 的 8 字节大端拼接 SHA-256 前 16 hex；head 前 16 项。
fn arr_str(iter: impl Iterator<Item = (i64, String)>) -> String {
    let elems: Vec<(i64, String)> = iter.collect();
    let mut bytes = Vec::with_capacity(elems.len() * 8);
    for (lv, _) in &elems {
        bytes.extend_from_slice(&lv.to_be_bytes());
    }
    let mut head = String::new();
    for (i, (_, s)) in elems.iter().enumerate().take(16) {
        if i > 0 {
            head.push(',');
        }
        head.push_str(s);
    }
    format!("ARR[{}] sha=#{} head={}", elems.len(), &sha256_hex(&bytes)[..16], head)
}

/// 核心 60 项标量 + 实体/路径关键数组。字段序无关（按名对拍）。
pub fn dump_scene(sc: &GameScene) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |e: Entry| out.push(format!("{} = {}", e.name, e.value));

    push(e_int("frameCounter", sc.frame_counter as i32));
    push(e_byte("gameMode", sc.game_mode.raw() as i8)); // 序列化边界 raw()（件 A 边界契约）
    push(e_int("keyValue", sc.key_value));
    push(e_int("keyHeldCode", sc.key_held));
    push(e_bool("overlayActive", sc.overlay_active));
    push(e_byte("overlayKind", sc.overlay_kind as i8));
    push(e_int("playerHp", sc.player.hp));
    push(e_int("playerAtk", sc.player.atk));
    push(e_int("playerDef", sc.player.def));
    push(e_int("goldAmount", sc.player.gold));
    push(e_int("equippedWeaponType", sc.equipped_weapon));
    push(e_int("equippedArmorType", sc.equipped_armor));
    push(e_byte("walkPhase", sc.walk_phase as i8));
    push(e_int("playerPixelX", sc.view.player_px));
    push(e_int("playerPixelY", sc.view.player_py));
    push(e_int("playerCellX", sc.player_cell_x));
    push(e_int("playerCellY", sc.player_cell_y));
    push(e_byte("facingDirection", sc.facing as i8));
    push(e_int("stepProgressPx", sc.step_progress));
    push(e_int("cameraPixelX", sc.view.cam_x));
    push(e_int("cameraPixelY", sc.view.cam_y));
    push(e_int("viewWidthPx", sc.view.view_w));
    push(e_int("viewHeightPx", sc.view.view_h));
    push(e_int("mapCellsWide", sc.view.wide));
    push(e_int("mapCellsHigh", sc.view.high));
    push(e_int("currentFloor", sc.floor));
    push(e_int("entityCount", sc.entities.count as i32));
    push(e_int("yellowKeyCount", sc.player.yellow_keys));
    push(e_int("blueKeyCount", sc.player.blue_keys));
    push(e_int("redKeyCount", sc.player.red_keys));
    push(e_int("tileBob", sc.tile_bob));
    push(e_int("openAnimTick", sc.open_anim_tick));
    push(e_int("currentScriptIndex", sc.current_script_index as i32));
    push(e_int("dialogPage", sc.dialog_page));
    push(e_byte("dialogPhase", sc.dialog_phase as i8));
    push(e_int("walkStepCount", sc.walk_step_count));
    push(e_bool("scriptWalkArmed", sc.script_walk_armed));
    push(e_bool("battleActiveFlag", sc.battle_active));

    // 实体数组（前 count 项有效；Java 恒 100 槽，尾部 0 填充在两侧等价）。
    // SoA→AoS（第 1 步）后按 items 映射重建整表引用：**push 序/值序/尾默认逐字节
    // 保持**（A2 行序冻结；Entity::default=旧 Vec 预填值 ⇒ 金样不变）。
    let ea_i32 = |f: fn(&game_core::entity::Entity) -> i32| -> Vec<i32> {
        sc.entities.items.iter().map(|e| f(e)).collect()
    };
    let ea_i16 = |f: fn(&game_core::entity::Entity) -> i16| -> Vec<i16> {
        sc.entities.items.iter().map(|e| f(e)).collect()
    };
    let ea_u8 = |f: fn(&game_core::entity::Entity) -> u8| -> Vec<u8> {
        sc.entities.items.iter().map(|e| f(e)).collect()
    };
    let ea_bool = |f: fn(&game_core::entity::Entity) -> bool| -> Vec<bool> {
        sc.entities.items.iter().map(|e| f(e)).collect()
    };
    push(e_int_arr("entityType", &ea_i32(|e| e.type_)));
    push(e_int_arr("entityPixelX", &ea_i32(|e| e.px)));
    push(e_int_arr("entityPixelY", &ea_i32(|e| e.py)));
    push(e_int_arr("entityAnimRow", &ea_i32(|e| e.anim_row)));
    push(e_int_arr("entityAnimFrame", &ea_i32(|e| e.anim_frame)));
    push(e_short_arr("entityParam", &ea_i16(|e| e.param)));
    push(e_byte_arr("entityState", &ea_u8(|e| e.state)));
    push(e_bool_arr("entityVisible", &ea_bool(|e| e.visible)));
    push(e_bool_arr("entityRemoved", &ea_bool(|e| e.removed)));
    push(e_int_arr("enemyBaseGold", &sc.enemy_base_gold));
    push(e_byte_arr("walkPathBuffer", &sc.walk_path_buffer));

    out
}
