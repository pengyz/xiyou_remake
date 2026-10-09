//! L2 时间线对拍：oracle trace（Java 实测）vs Rust 引擎重放。
//!
//! 夹具 = prologue-dialog 场景真实 trace 片段（tick 546-1210，采样格式）。
//!
//! 忠实性边界（P3.1）：phase 1/6 的打字机按键在 Java 里有「行完成/页推进」两段
//! 语义（m_017/m_021，行布局属渲染层 P3.2）；此处引擎按压 = 页推进的近似模型，
//! 因此断言的是**过渡序列**（cursor 12→24→43 与文本序列）而非逐 tick 值——
//! 该序列与 Java trace 的过渡序列逐项一致（tick 584/782/902 实测）。

use game_core::script::{DialogueTable, HostCtx, ScriptEngine};
use game_oracle::trace::parse;

fn table() -> DialogueTable {
    let scripts: Vec<String> = include_str!("../../game-core/tests/fixtures/level_scripts.txt")
        .split("\u{1e}\n")
        .map(|s| s.trim_end_matches('\u{1e}').to_string())
        .collect();
    let texts: Vec<String> = include_str!("../../game-core/tests/fixtures/dialogue_texts.txt")
        .split("\u{1e}\n")
        .map(|s| s.trim_end_matches('\u{1e}').to_string())
        .collect();
    let speakers: Vec<i8> = include_str!("../../game-core/tests/fixtures/speaker_types.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    let type_names: Vec<String> = include_str!("../../game-core/tests/fixtures/object_type_names.txt")
        .lines()
        .map(|s| s.to_string())
        .collect();
    let type_categories: Vec<u8> = include_str!("../../game-core/tests/fixtures/type_categories.txt")
        .lines()
        .filter_map(|l: &str| l.trim().parse::<u8>().ok())
        .collect();
    DialogueTable { scripts, texts, speakers, type_names, type_categories }
}

#[derive(Default)]
struct ReplayHost;
impl HostCtx for ReplayHost {
    fn find_path(&mut self, _: i32, _: i32, _: i32, _: i32) -> bool { false }
    fn cell_entity(&self, _: i32, _: i32, _: i32) -> i32 { -1 }
    fn remove_entity(&mut self, _: i32) {}
    fn remove_all_of_type(&mut self, _: i32) {}
    fn swd_convert_type81(&mut self) {}
    fn entity_pixel(&self, _: i32) -> (i32, i32) { (0, 0) }
    fn give_gold(&mut self, _: i32) {}
    fn pickup_item(&mut self, _: i32) {}
    fn snap_camera_to_player(&mut self) {}
    fn locate_speaker_entity(&mut self, _: i32) -> i32 { -1 }
    fn set_typewriter(&mut self, _: &str, _: i32, _: i32) {}
    fn reload_floor_entities(&mut self, _: i32) {}
    fn spawn_entity(&mut self, _: i32, _: i32, _: i32) -> i32 { -1 }
    fn ces_camera(&mut self, _: i32) {}
    fn set_camera_anchor_target(&mut self, _: i32, _: i32) {}
    fn exit_application(&mut self) {}
    fn mvs_exchange(&mut self) {}
    fn m_082_set_dialog_target(&mut self, _: i32) {}
    fn set_entity_route_flag(&mut self, _: i32) {}
    fn entity_type(&self, _idx: i32) -> i32 { -1 }
    fn snap_camera_after_ros1(&mut self, _px: i32, _py: i32) {}
}

fn press_key(input: &str) -> Option<i32> {
    let rest = input.strip_prefix("press(")?;
    rest.strip_suffix(')')?.parse().ok()
}

#[test]
fn l2_prologue_cursor_and_text_transitions_match_java() {
    let text = include_str!("fixtures/prologue_trace_excerpt.txt");
    let records = parse(text);
    assert!(records.len() > 100, "夹具过小: {}", records.len());

    let t = table();
    let mut host = ReplayHost;
    let mut eng = ScriptEngine::new(&t, 40); // floor-51：TAK_255_259 TAK_165_165 SEE_3_10_166_166_1

    // Java trace 过渡序列（实测：cursor 12@584 → 24@782 → 43@902；文本 6 句变化）
    let expect_cursor_transitions: [usize; 3] = [12, 24, 43];

    let mut got_cursor_transitions: Vec<usize> = vec![];
    let mut last_cursor = 0usize;
    let mut texts: Vec<String> = vec![];
    let mut last_text: Option<String> = None;
    let mut presses = 0usize;

    for rec in &records {
        if let Some(inp) = &rec.input {
            if press_key(inp).is_some() {
                presses += 1;
                if matches!(eng.state.dialog_phase, 1 | 6) {
                    eng.advance_dialog_page(&mut host);
                }
            }
        }
        eng.step(&mut host);

        if eng.state.script_cursor != last_cursor {
            last_cursor = eng.state.script_cursor;
            got_cursor_transitions.push(last_cursor);
        }
        if let Some(ov) = &eng.state.overlay_text {
            if last_text.as_deref() != Some(ov) {
                texts.push(ov.clone());
                last_text = Some(ov.clone());
            }
        }
    }

    assert_eq!(got_cursor_transitions, expect_cursor_transitions,
        "cursor 过渡序列（Java：t584/t782/t902）");
    // 7 个不同文本：255..259（五句）+ 165（为师千里传音）+ 166（SEE 打开的教学页）。
    // 注意 259/165/166 的合成文本同以"菩提老祖: "开头——此前"6 句"系 14 字符 head
    // 截断的测量假象（165→166 的变化不可见于 head）。
    assert_eq!(texts.len(), 7, "对白文本 7 句");
    assert!(texts[5].contains("千里传音"), "text[165] = 为师千里传音");
    assert!(texts[6].contains("好了"), "text[166] = BUG-006 冻结点教学页");
    assert!(eng.state.overlay_text.as_deref().unwrap().contains("好了"),
        "SEE 打开的页 166 = BUG-006 冻结点");
    assert!(presses >= 10, "夹具应含足够按键: {presses}");
}
