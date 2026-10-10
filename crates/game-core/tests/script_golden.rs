//! P3.1 脚本解释器 golden vectors（TDD）。
//!
//! 每个向量的期望值锚定 Java 侧证据：deobf 行号（reference/src/deobf/a.java）或
//! oracle trace 实测（prologue-dialog 场景）。数据来自编译期嵌入的 Java 提取夹具。
//! 证据索引见 docs/spec/p3-script-interpreter.md §3-4。

use game_core::enums::{DialogPhase, WalkPhase};
use game_core::script::{DialogueTable, HostCtx, ScriptEngine};

/// 从夹具构建对话表（\x1e 记录分隔，与提取脚本一致）。
fn table() -> DialogueTable {
    let scripts: Vec<String> = include_str!("fixtures/level_scripts.txt")
        .split("\u{1e}\n")
        .map(|s| s.trim_end_matches('\u{1e}').to_string())
        .collect();
    let texts: Vec<String> = include_str!("fixtures/dialogue_texts.txt")
        .split("\u{1e}\n")
        .map(|s| s.trim_end_matches('\u{1e}').to_string())
        .collect();
    let speakers: Vec<i8> = include_str!("fixtures/speaker_types.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    let type_names: Vec<String> = include_str!("fixtures/object_type_names.txt")
        .lines()
        .map(|s| s.to_string())
        .collect();
    let type_categories: Vec<u8> = include_str!("../../game-core/tests/fixtures/type_categories.txt")
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    DialogueTable { scripts, texts, speakers, type_names, type_categories }
}

/// 确定性 Host 桩：记录全部 hook 调用；find_path 结果可注入。
#[derive(Default)]
struct StubHost {
    path_result: bool,
    find_path_calls: Vec<(i32, i32, i32, i32)>,
    gold_calls: Vec<i32>,
    pickup_calls: Vec<i32>,
    remove_calls: Vec<i32>,
    exit_called: bool,
    camera_snaps: usize,
    reloaded_floors: Vec<i32>,
}

impl HostCtx for StubHost {
    fn find_path(&mut self, fx: i32, fy: i32, tx: i32, ty: i32) -> bool {
        self.find_path_calls.push((fx, fy, tx, ty));
        self.path_result
    }
    fn cell_entity(&self, _x: i32, _y: i32, _t: i32) -> i32 {
        -1
    }
    fn remove_entity(&mut self, idx: i32) {
        self.remove_calls.push(idx);
    }
    fn remove_all_of_type(&mut self, _t: i32) {}
    fn swd_convert_type81(&mut self) {}
    fn entity_pixel(&self, _idx: i32) -> (i32, i32) {
        (0, 0)
    }
    fn give_gold(&mut self, amount: i32) {
        self.gold_calls.push(amount);
    }
    fn pickup_item(&mut self, item: i32) {
        self.pickup_calls.push(item);
    }
    fn snap_camera_to_player(&mut self) {
        self.camera_snaps += 1;
    }
    fn locate_speaker_entity(&mut self, _speaker: i32) -> i32 {
        -1
    }
    fn set_typewriter(&mut self, _text: &str, _y: i32, _pos: i32) {}
    fn reload_floor_entities(&mut self, floor: i32) {
        self.reloaded_floors.push(floor);
    }
    fn spawn_entity(&mut self, _t: i32, _x: i32, _y: i32) -> i32 {
        -1
    }
    fn ces_camera(&mut self, _idx: i32) {}
    fn set_camera_anchor_target(&mut self, _x: i32, _y: i32) {}
    fn exit_application(&mut self) {
        self.exit_called = true;
    }
    fn mvs_exchange(&mut self) {
        panic!("MVS 在 P3.1 边界外（数据域零使用）");
    }
    fn m_082_set_dialog_target(&mut self, _v: i32) {}
    fn set_entity_route_flag(&mut self, _idx: i32) {}
    fn entity_type(&self, _idx: i32) -> i32 { -1 }
    fn snap_camera_after_ros1(&mut self, _px: i32, _py: i32) {}
}

/// 序章脚本 = 第 40 项（trace 实证 f_String_05，deobf a.java:1124 起第 41 项）。
const PROLOGUE: usize = 40;

#[test]
fn tak_opens_page255_cursor0_to_12() {
    // deobf a.java:8227-8233（TAK 分支）+ m_096:8185-8210；
    // trace 实证：tick578 scriptCursor=12、dialogPhase=1、overlayText="菩提老祖: \cF8F8F8悟空…"
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, PROLOGUE);
    eng.step(&mut host);

    assert_eq!(eng.state.script_cursor, 12, "trace t578");
    assert_eq!(eng.state.dialog_phase, DialogPhase::Typewriter);
    assert_eq!(eng.state.dialog_page, 255);
    assert_eq!(eng.state.tak_page_start, 255);
    assert_eq!(eng.state.tak_page_end, 259);
    let sp = t.speakers[255];
    assert_eq!(eng.state.speaker, sp as i32);
    let expect_prefix = format!("{}: \\cF8F8F8", t.type_names[sp as usize]);
    assert!(
        eng.state.overlay_text.as_deref().unwrap().starts_with(&expect_prefix),
        "overlay = {:?}",
        eng.state.overlay_text
    );
    // trace 实证说话人是菩提老祖（类型 87）：
    assert_eq!(expect_prefix, "菩提老祖: \\cF8F8F8");
}

#[test]
fn tak_page_advance_and_exhaust_resumes_script() {
    // m_096 else 分支（a.java:8206-8210）：page > range_end ⇒ phase=0；
    // phase=0 后下一次 step 执行下一条指令（trace t1002：cursor=24，第 6 句对白 text[165]）
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, PROLOGUE);
    eng.step(&mut host); // TAK_255_259 → cursor 12, page 255
    for page in 256..=259 {
        eng.advance_dialog_page(&mut host);
        assert_eq!(eng.state.dialog_page, page);
        assert_eq!(eng.state.dialog_phase, DialogPhase::Typewriter);
    }
    eng.advance_dialog_page(&mut host); // 260 > 259 → 榨干
    assert_eq!(eng.state.dialog_phase, DialogPhase::ScriptStep);
    eng.step(&mut host); // 恢复执行：TAK_165_165
    assert_eq!(eng.state.script_cursor, 24, "trace t1002");
    assert_eq!(eng.state.dialog_page, 165);
    assert_eq!(eng.state.dialog_phase, DialogPhase::Typewriter);
}


/// 驱动到 SEE 指令前的辅助：TAK_255_259 → 4 次翻页 → 榨干 → TAK_165_165 → 榨干
fn drive_to_see(eng: &mut ScriptEngine, host: &mut StubHost) {
    eng.step(host); // TAK_255_259
    for _ in 0..4 {
        eng.advance_dialog_page(host);
    }
    eng.advance_dialog_page(host); // 260 > 259 → phase 0
    eng.step(host); // TAK_165_165
    eng.advance_dialog_page(host); // 166 > 165 → phase 0
}

#[test]
fn see_bug006_replay_cursor_advances_walk_not_armed() {
    // BUG-006 回归锚：SEE_3_10_166_166_1 在 find_path=false 下 cursor 仍推进 43
    // （trace t1202），walk 不武装（script_walk_armed=false，f_int_148 死写）。
    // SEE 分支 deobf a.java:8449-8464：先 openDialogPage(c)，cursor 推进，phase=6。
    let t = table();
    let mut host = StubHost::default();
    host.path_result = false;
    let mut eng = ScriptEngine::new(&t, PROLOGUE);
    eng.step(&mut host); // TAK_255_259 → 12（trace t584）
    drive_to_see(&mut eng, &mut host);
    eng.step(&mut host); // SEE_3_10_166_166_1
    assert_eq!(eng.state.script_cursor, 43, "trace t1202");
    assert_eq!(eng.state.script_walk_armed, false);
    // SEE 置 phase=6，随后行尾收尾门（a.java:8551）覆写为 4——BUG-006 冻结态实证
    assert_eq!(eng.state.dialog_phase, DialogPhase::AwaitCamera);
    assert_eq!(eng.state.dialog_page, 166);
    assert_eq!(host.find_path_calls, vec![(1, 11, 3, 10)]);
}

#[test]
fn see_path_found_arms_walk() {
    // SEE e=1 分支（a.java:8456-8459）：find_path 成功 ⇒ walk_request_flag=true + walkPhase=0
    let t = table();
    let mut host = StubHost::default();
    host.path_result = true;
    let mut eng = ScriptEngine::new(&t, PROLOGUE);
    drive_to_see(&mut eng, &mut host);
    eng.step(&mut host);
    assert_eq!(eng.state.script_cursor, 43);
    assert!(eng.state.walk_request_flag);
    assert_eq!(eng.state.walk_phase, WalkPhase::Idle);
    assert_eq!(eng.state.dialog_phase, DialogPhase::AwaitCamera, "SEE 置 6 后行尾收尾门覆写为 4");
}

#[test]
fn gut_flags_and_jumps() {
    // GUT 分支 a.java:8269-8273 + loadLevelScript a.java:7997-8012：
    // flags[当前]=true；current=n；cursor=0；gameMode=11；phase=0；overlay=null
    let t = table();
    let gut_entry = t.scripts.iter().position(|s| s.contains("GUT_37")).unwrap();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, gut_entry);
    // 直接把 cursor 推到 GUT_37 的位置
    let pos = t.scripts[gut_entry].find("GUT_37").unwrap();
    eng.state.script_cursor = pos;
    eng.step(&mut host);
    assert!(eng.state.script_line_flags[gut_entry]);
    assert_eq!(eng.state.current_script_index, 37);
    assert_eq!(eng.state.script_cursor, 0);
    assert_eq!(eng.state.game_mode, 11);
    assert_eq!(eng.state.dialog_phase, DialogPhase::ScriptStep);
    assert_eq!(eng.state.overlay_text, None);
}

#[test]
fn res_resets_stats_when_difficulty_zero() {
    // RES 分支 a.java:8462-8479（difficultyIndex==0）：钥匙清零/金=4/攻防10/HP400/层1-3
    let t = table();
    let res_entry = t.scripts.iter().position(|s| s.starts_with("ROS_4_1 TAK_92_97")).unwrap();
    // 用含 RES_0 的条目（条目 0："… RES_0 GUT_1"）
    let entry0 = 0;
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, entry0);
    eng.state.difficulty_index = 0;
    eng.state.player_hp = 777;
    eng.state.gold = 500;
    eng.state.yellow_keys = 9;
    // 推进 cursor 到 RES_0
    let pos = t.scripts[entry0].find("RES_0").unwrap();
    eng.state.script_cursor = pos;
    eng.step(&mut host);
    assert_eq!(eng.state.player_hp, 400);
    assert_eq!(eng.state.player_atk, 10);
    assert_eq!(eng.state.player_def, 10);
    assert_eq!(eng.state.gold, 4);
    assert_eq!(eng.state.yellow_keys, 0);
    assert_eq!(eng.state.min_floor_reached, 1);
    assert_eq!(eng.state.max_floor_reached, 3);
    let _ = res_entry;
}

#[test]
fn ros_sets_player_fields() {
    // ROS 子命令 a.java:8340-8357：3=HP/4=朝向/5=ATK/6=DEF（1/2 走 hook，P3.2 实体）
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, 0);
    // 构造最小行：ROS_3_77 ROS_4_2
    eng.set_script_line_for_test("ROS_3_77 ROS_4_2 ");
    eng.step(&mut host);
    assert_eq!(eng.state.player_hp, 77);
    assert_eq!(eng.state.script_cursor, 9);
    eng.step(&mut host);
    assert_eq!(eng.state.facing_direction, 2);
    assert_eq!(eng.state.script_cursor, 17);
}

#[test]
fn gin_tail_recurses_into_next_instruction() {
    // GIN 分支 a.java:8395-8397：cursor 推进后**同 tick 显式尾递归**执行下一条
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, 0);
    // GIN 置于行中（行尾会触发 a.java:8551 收尾门 phase=4——Java 同款语义）
    eng.set_script_line_for_test("GIN_1_1000 TAK_8_9 X ");
    eng.step(&mut host);
    assert_eq!(host.gold_calls, vec![1000]);
    // 尾递归：TAK 同步执行完毕
    assert_eq!(eng.state.dialog_phase, DialogPhase::Typewriter);
    assert_eq!(eng.state.dialog_page, 8);
    assert_eq!(eng.state.script_cursor, 19);
}

#[test]
fn end_requests_exit() {
    // END 分支 a.java:8479-8483：gameMode=20 + m_000（exit hook）
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, 0);
    eng.set_script_line_for_test("END_0 ");
    eng.step(&mut host);
    assert_eq!(eng.state.game_mode, 20);
    assert!(host.exit_called);
}

#[test]
fn mov_player_path_gates_walk_phase() {
    // MOV 3 参分支 a.java:8246-8253：find_path 成功 ⇒ phase=3；失败 ⇒ 无动作
    let t = table();
    let mut host = StubHost::default();
    host.path_result = true;
    let mut eng = ScriptEngine::new(&t, 0);
    eng.set_script_line_for_test("MOV_0_5_11 GLV_0 ");
    eng.step(&mut host);
    assert_eq!(eng.state.dialog_phase, DialogPhase::Walk);
    assert_eq!(eng.state.script_walk_armed, true, "a.java:8265 三写点（复核 R2）");
    assert_eq!(eng.state.walk_phase, WalkPhase::Idle);
    assert!(host.camera_snaps > 0);
    assert_eq!(host.find_path_calls, vec![(1, 11, 5, 11)]); // playerCell(1,11)（gameplay-floor1 实测起点）
    host.path_result = false;
    eng.set_script_line_for_test("MOV_0_5_11 GLV_0 ");
    eng.step(&mut host);
    // Java 失败分支无 else：phase 不复位（保持 3）
    assert_eq!(eng.state.dialog_phase, DialogPhase::Walk, "失败分支不复位 phase");
}

#[test]
fn cursor_stops_at_line_end_bug006_frozen() {
    // 行尾门 a.java:8217（cursor+3 <= len）：cursor=43 冻结态步进 no-op（BUG-006）
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, PROLOGUE);
    eng.state.script_cursor = 43;
    eng.step(&mut host);
    assert_eq!(eng.state.script_cursor, 43);
}

#[test]
fn lay_sets_phase5_and_clears_overlay() {
    // LAY 分支 a.java:8314-8320
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, 0);
    eng.set_script_line_for_test("LAY_2 GLV_0 ");
    eng.state.overlay_text = Some("x".into());
    eng.step(&mut host);
    assert_eq!(eng.state.layer_byte, 2);
    assert_eq!(eng.state.overlay_text, None);
    assert_eq!(eng.state.dialog_phase, DialogPhase::LayCutscene);
    assert!(eng.state.flag_bool_16, "a.java:8317");
    assert_eq!(host.reloaded_floors, vec![eng.state.current_floor]);
}

#[test]
#[should_panic(expected = "MVS")]
fn mvs_is_p32_boundary() {
    // MVS 数据域零使用（68 条脚本 0 处）——P3.2 边界：panic 指引证据行号
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, 0);
    eng.set_script_line_for_test("MVS ");
    eng.step(&mut host);
}

#[test]
fn prologue_timeline_replays_trace() {
    // L2b 时间线级（prologue-dialog 场景 oracle trace 实测）：
    // cursor 0→12(t578)→24(t1002)→43(t1202)；对白文本序列 6 句（255..259 + 165）
    let t = table();
    let mut host = StubHost::default();
    let mut eng = ScriptEngine::new(&t, PROLOGUE);
    let mut timeline = vec![];
    let mut texts = vec![];

    eng.step(&mut host); // t578
    timeline.push(eng.state.script_cursor);
    texts.push(eng.state.overlay_text.clone().unwrap());
    for _ in 0..4 {
        // 玩家 -5 推进页 256..259
        eng.advance_dialog_page(&mut host);
        texts.push(eng.state.overlay_text.clone().unwrap());
    }
    eng.advance_dialog_page(&mut host); // 榨干 → phase 0
    eng.step(&mut host); // t1002: TAK_165_165
    timeline.push(eng.state.script_cursor);
    texts.push(eng.state.overlay_text.clone().unwrap());
    eng.advance_dialog_page(&mut host); // 166 > 165 → 榨干
    eng.step(&mut host); // t1202: SEE（path=false）
    timeline.push(eng.state.script_cursor);

    assert_eq!(timeline, vec![12, 24, 43], "trace 过渡序列（采样格式下过渡 tick 为 584/782/902）");
    assert_eq!(texts.len(), 6, "对白文本序列：255..259 五句 + TAK_165_165 一句");
    assert!(texts[0].starts_with("菩提老祖: \\cF8F8F8"), "首句说话人菩提老祖（类型 87）");
    assert!(texts[1].starts_with("孙悟空: \\cF8F8F8"), "第 2 句说话人悟空");
    // SEE_3_10_166_166_1 执行后打开第 166 页（BUG-006 冻结点文本"好了"）
    assert!(eng.state.overlay_text.as_deref().unwrap().contains("好了"),
        "SEE 打开的页 166 = 冻结点文本");
}
