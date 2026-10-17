//! BUG-021 截断回归夹具（考证：docs/findings/reviews/2026-10-17-mot-if-truncation-fidelity.md）。
//!
//! MOT_IF 槽表截断/损坏读失败对拍 Java 真值语义（a.java:8983-9005 渐进解析 +
//! 9000-9001 catch → markSlotEntry(-1)）：
//! ① **半解析保留**：字段整段在界内才提交（Java readX RHS 先求值，EOFException 不赋值），
//!    该字段及其后全部留**旧值**；不清表（markSlotEntry(-1) 不清数组，a.java:9007-9024）；
//! ② **无 panic**：旧守卫 `pos+2+28` 只保证 30 字节、实读 31（3×u8+7×i32）⇒
//!    末字段 gold 的 i32 切片越界硬崩；Java 同窗是 EOFException 优雅降级；
//! ③ **parse_dirty**：读失败 = markSlotEntry(-1) 落盘触发器（壳消费回写 MOT_IF）。
//!
//! 期望模型独立于解析器（**静态偏移表**推导，非镜像实现）：槽 k 基偏移 32k；
//! valid@b(1B)、floor@b+1、weapon@b+2、armor@b+3、hp@b+4..8、atk@b+8..12、
//! def@b+12..16、钥y@b+16..20、钥b@b+20..24、钥r@b+24..28、gold@b+28..32（BE i32）；
//! valid=0 的槽只有 valid 字节。本夹具 blob：槽0/槽1 有效（字段值互异可辨识）、
//! 槽2-5 无效 ⇒ 全长 68。
//!
//! TDD RED 复现（A 强化协议）：本文件单独应用到父树 ⇒
//! `cargo test -p game-core --test mot_if_truncation` 编译失败即红（旧
//! `load_slot_table` 无返回值/无 parse_dirty/take_parse_dirty/reset_entry_layout）。

use game_core::menu_family::SlotSelect;

const FULL_LEN: usize = 68;

/// 夹具槽数据（10 字段：floor/weapon/armor/hp/atk/def/钥y/钥b/钥r/gold）。
const S0: [i32; 10] = [11, 12, 13, 0x11111111, 0x22222222, 0x33333333, 0x44444444, 0x55555555, 0x66666666, 0x77777777];
const S1: [i32; 10] = [21, 22, 23, 0x12121212, 0x23232323, 0x34343434, 0x45454545, 0x56565656, 0x67676767, 0x78787878];

/// 全长 68 的夹具 blob：槽0/槽1 有效 + 槽2-5 无效。
fn blob() -> Vec<u8> {
    let mut b = Vec::new();
    for s in [S0, S1] {
        b.push(1); // valid
        b.push(s[0] as u8); // floor
        b.push(s[1] as u8); // weapon
        b.push(s[2] as u8); // armor
        for v in &s[3..] {
            b.extend_from_slice(&v.to_be_bytes());
        }
    }
    b.extend_from_slice(&[0, 0, 0, 0]); // 槽2-5 无效
    assert_eq!(b.len(), FULL_LEN);
    b
}

/// 字段静态偏移表（独立推导）：(start, end, 槽号, 字段序号)。
/// 字段序号 0=floor 1=weapon 2=armor 3=hp 4=atk 5=def 6=钥y 7=钥b 8=钥r 9=gold，
/// 10=valid 字节。槽0/1 全字段；槽2-5 仅 valid 字节（64..68）。
fn field_spans() -> Vec<(usize, usize, usize, usize)> {
    let mut v = Vec::new();
    for (base, k) in [(0usize, 0usize), (32, 1)] {
        v.push((base, base + 1, k, 10));
        let offs = [1usize, 2, 3, 4, 8, 12, 16, 20, 24, 28];
        let lens = [1usize, 1, 1, 4, 4, 4, 4, 4, 4, 4];
        for f in 0..10 {
            v.push((base + offs[f], base + offs[f] + lens[f], k, f));
        }
    }
    for (i, k) in (2usize..6).enumerate() {
        v.push((64 + i, 65 + i, k, 10));
    }
    v
}

/// 期望态：字段前缀提交模型（首个放不下的字段即停，其后留 base 旧值）。
fn expected(len: usize, base: &SlotSelect) -> SlotSelect {
    let mut e = base.clone();
    for (start, end, k, f) in field_spans() {
        if end > len {
            break;
        }
        let _ = start;
        let s = match k {
            0 => S0,
            1 => S1,
            _ => [0; 10],
        };
        match f {
            // valid 字节的期望值 = blob 中该字节（槽0/1=1、槽2-5=0）
            10 => e.valid[k] = k < 2,
            0 => e.floor_num[k] = s[0] as i8,
            1 => e.weapon[k] = s[1] as i8,
            2 => e.armor[k] = s[2] as i8,
            3 => e.hp[k] = s[3],
            4 => e.atk[k] = s[4],
            5 => e.def[k] = s[5],
            6 => e.keys_yellow[k] = s[6] as i16,
            7 => e.keys_blue[k] = s[7] as i16,
            8 => e.keys_red[k] = s[8] as i16,
            9 => e.gold[k] = s[9],
            _ => unreachable!(),
        }
    }
    e
}

fn assert_slots_eq(got: &SlotSelect, want: &SlotSelect, at: &str) {
    for k in 0..6 {
        assert_eq!(got.valid[k], want.valid[k], "{at}: valid[{k}]");
        assert_eq!(got.floor_num[k], want.floor_num[k], "{at}: floor_num[{k}]");
        assert_eq!(got.weapon[k], want.weapon[k], "{at}: weapon[{k}]");
        assert_eq!(got.armor[k], want.armor[k], "{at}: armor[{k}]");
        assert_eq!(got.hp[k], want.hp[k], "{at}: hp[{k}]");
        assert_eq!(got.atk[k], want.atk[k], "{at}: atk[{k}]");
        assert_eq!(got.def[k], want.def[k], "{at}: def[{k}]");
        assert_eq!(got.keys_yellow[k], want.keys_yellow[k], "{at}: keys_yellow[{k}]");
        assert_eq!(got.keys_blue[k], want.keys_blue[k], "{at}: keys_blue[{k}]");
        assert_eq!(got.keys_red[k], want.keys_red[k], "{at}: keys_red[{k}]");
        assert_eq!(got.gold[k], want.gold[k], "{at}: gold[{k}]");
    }
}

/// ① 半解析保留 + 返回值 + parse_dirty：**各截断长度**全网格对拍（含旧 panic 窗 L=31）。
#[test]
fn truncation_grid_matches_java_partial_parse_on_zero_base() {
    let b = blob();
    let zeros = SlotSelect::new();
    for len in 0..=FULL_LEN {
        let mut sl = zeros.clone();
        let ok = sl.load_slot_table(Some(&b[..len]));
        assert_eq!(ok, len == FULL_LEN, "L={len}: 完整解析判定");
        assert_eq!(sl.take_parse_dirty(), len != FULL_LEN, "L={len}: markSlotEntry(-1) 触发器");
        assert_slots_eq(&sl, &expected(len, &zeros), &format!("L={len} 零基"));
    }
}

/// ① 半解析保留 = 留**旧值**：预置 marker 基态，截断后未及字段保留 marker
/// （Java 数组跨进入复用，旧值 = 上次表值）。
#[test]
fn truncation_retains_old_values_beyond_eof() {
    let b = blob();
    let mut base = SlotSelect::new();
    for k in 0..6 {
        base.valid[k] = true;
        base.floor_num[k] = 90 + k as i8;
        base.weapon[k] = 80 + k as i8;
        base.armor[k] = 70 + k as i8;
        base.hp[k] = 0x0BAD_0000 + k as i32;
        base.atk[k] = 0x0BAD_1000 + k as i32;
        base.def[k] = 0x0BAD_2000 + k as i32;
        base.keys_yellow[k] = 60 + k as i16;
        base.keys_blue[k] = 50 + k as i16;
        base.keys_red[k] = 40 + k as i16;
        base.gold[k] = 0x0BAD_3000 + k as i32;
    }
    for len in 0..=FULL_LEN {
        let mut sl = base.clone();
        sl.load_slot_table(Some(&b[..len]));
        assert_slots_eq(&sl, &expected(len, &base), &format!("L={len} marker 基"));
    }
}

/// ② 旧 panic 窗命名回归：valid 字节 + 恰 30 字节字段区（L=31）——旧守卫放行后
/// 末字段 gold 的 i32 切片越界 panic；Java 同窗是 EOFException 优雅降级。
#[test]
fn panic_window_30_byte_remainder_degrades_gracefully() {
    let b = blob();
    let mut sl = SlotSelect::new();
    let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sl.load_slot_table(Some(&b[..31]))
    }));
    let ok = ok.expect("L=31（字段区恰 30 字节）不得 panic——守卫须 ≥31");
    assert!(!ok, "截断解析失败");
    // 槽0：floor..钥r 提交（3+4×6=27 字节），gold 保留旧值 0（Java readInt EOF 不赋值）
    assert!(sl.valid[0]);
    assert_eq!((sl.floor_num[0], sl.weapon[0], sl.armor[0]), (11, 12, 13));
    assert_eq!(sl.hp[0], 0x11111111);
    assert_eq!(sl.atk[0], 0x22222222);
    assert_eq!(sl.def[0], 0x33333333);
    assert_eq!((sl.keys_yellow[0], sl.keys_blue[0], sl.keys_red[0]), (0x4444, 0x5555, 0x6666));
    assert_eq!(sl.gold[0], 0, "gold（第 28..32 字节）未提交、留旧值");
    assert!(!sl.valid[1], "其后字段/槽全部未达");
}

/// ③ 缺店/空记录同为读失败（Java openRecordStore/getRecord 抛 ⇒ markSlotEntry(-1)）：
/// 不写数组、parse_dirty 置位。
#[test]
fn missing_or_empty_blob_marks_dirty_without_writes() {
    let mut base = SlotSelect::new();
    base.hp[3] = 0x0DDC_AFE0;
    for mot_if in [None, Some(&[][..])] {
        let mut sl = base.clone();
        let ok = sl.load_slot_table(mot_if);
        assert!(!ok);
        assert!(sl.take_parse_dirty(), "markSlotEntry(-1) 触发器");
        assert_slots_eq(&sl, &base, "缺店/空记录不动数组");
    }
}

/// 入口语义（a.java:4354-4366）：度量随入口重算 + 光标/滚动复位，**数据数组不动**
/// （Java 数组仅 !f_bool_15 首分配 ⇒ 跨进入复用，半截解析的旧值即上次表值）。
#[test]
fn reset_entry_layout_keeps_slot_data() {
    let mut sl = SlotSelect::new();
    sl.valid[2] = true;
    sl.hp[2] = 777;
    sl.floor_num[2] = 55;
    sl.cursor = 4;
    sl.scroll_top = 2;
    sl.scroll_bottom = 6;
    sl.rows_per_page = 999; // 污染度量，验证入口重算
    sl.content_w = 999;
    sl.content_h = 999;
    sl.reset_entry_layout();
    assert_eq!((sl.cursor, sl.scroll_top), (0, 0), "slotCursor/slotScrollTop 复位（a.java:4365）");
    assert_eq!(sl.scroll_bottom, sl.rows_per_page, "slotScrollBottom 复位（a.java:4366）");
    // 度量公式（a.java:4354-4364）：per=min(6, 208/lh)、w=max(148, wide*3+60)、h=112+lh*per
    let font = game_core::paint::paint_font();
    let lh = font.height + 4;
    let per = (game_core::layout::SLOT_LIST_MAX_H / lh).min(6);
    assert_eq!(sl.rows_per_page, per, "slotsPerPage 重算（a.java:4354-4357）");
    assert_eq!(sl.content_w, (font.char_width('国' as u16) * 3 + 60).max(148), "slotListWidth（4359-4362）");
    assert_eq!(sl.content_h, 112 + lh * per, "slotListHeight（4364）");
    assert_eq!((sl.valid[2], sl.hp[2], sl.floor_num[2]), (true, 777, 55), "数据数组不动");
}

/// 完整解析：返回 true、parse_dirty 不置位、逐字段吻合。
#[test]
fn full_parse_succeeds_and_clears_dirty() {
    let b = blob();
    let mut sl = SlotSelect::new();
    assert!(sl.load_slot_table(Some(&b)));
    assert!(!sl.take_parse_dirty(), "完整解析不触发 markSlotEntry(-1)");
    let zeros = SlotSelect::new();
    assert_slots_eq(&sl, &expected(FULL_LEN, &zeros), "全长解析");
}
