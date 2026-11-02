    use crate::render::FontMetrics;
    use super::scene_test_support::scene;
    use super::{GameMode, PointerHit};
    use crate::layout::{SCREEN_H, SCREEN_W};

    fn menu_scene() -> super::GameScene {
        let mut sc = scene((2, 2), &[(43, 5, 5)]);
        sc.game_mode = GameMode::InGameMenu;
        sc.menu_kinds = vec![0, 1, 2, 3, 4, 5];
        sc.menu_item_count = 6;
        sc.menu_scroll_top = 0;
        sc.menu_visible_bottom = 6;
        sc.menu_line_height = 18;
        sc.menu_box_width = 148;
        sc.menu_list_height = 112;
        sc.menu_cursor = 0;
        sc
    }

    /// (d)+(a)：行带几何与 paint_menu_list 同式逐行锚 + 命中带定死 + confirm_key=-6
    #[test]
    fn menu_pointer_hit_geometry_boundaries_and_confirm_key() {
        let sc = menu_scene();
        let (y0, h) = sc.menu_row_band(0);
        assert_eq!(sc.pointer_hit(120, y0 + h / 2), PointerHit::Row { index: 0, confirm_key: -6 });
        let (y2, h2) = sc.menu_row_band(2);
        assert_eq!(sc.pointer_hit(120, y2 + h2 / 2), PointerHit::Row { index: 2, confirm_key: -6 });
        // 命中带边界（③-2/c）：行尾-1 归本行、下一像素归下行
        assert_eq!(sc.pointer_hit(120, y0 + h - 1), PointerHit::Row { index: 0, confirm_key: -6 });
        assert_eq!(sc.pointer_hit(120, y0 + h), PointerHit::Row { index: 1, confirm_key: -6 });
        // 盒外/空洞 = None
        assert_eq!(sc.pointer_hit(120, y0 - 3), PointerHit::None);
        // 行几何 = paint 同式（居中盒 16+pad、行距 18）
        assert_eq!(y0, 160 - ((112 + 32) >> 1) + 16, "行带 y 与 paint_menu_list 同式锚（pad 相消）");
        // 玩法/对话态恒 None（不误触）
        let mut s2 = menu_scene();
        s2.game_mode = GameMode::Map;
        assert_eq!(s2.pointer_hit(120, y0 + 5), PointerHit::None);
    }

    /// (b)：select_row 只动光标+滚窗同步，其他状态零变化
    #[test]
    fn select_row_only_cursor_and_scroll() {
        let mut sc = menu_scene();
        let hp = sc.player.hp;
        let gold = sc.player.gold;
        sc.select_row(2);
        assert_eq!(sc.menu_cursor, 2);
        assert_eq!(sc.menu_scroll_top, 0, "可视窗内不滚");
        assert_eq!(sc.player.hp, hp);
        assert_eq!(sc.player.gold, gold);
        sc.menu_scroll_top = 0;
        sc.menu_visible_bottom = 4;
        sc.select_row(5);
        assert_eq!(sc.menu_cursor, 5);
        assert_eq!(sc.menu_visible_bottom, 6, "滚窗跟随下缘");
        assert_eq!(sc.menu_scroll_top, 2);
        sc.select_row(0);
        assert_eq!(sc.menu_scroll_top, 0, "滚窗跟随上缘");
        assert_eq!(sc.menu_visible_bottom, 4);
    }

    /// 设置面板（OptionList，2026-10-18 用户反馈面）：行带单源 + 命中 ±1px +
    /// confirm_key=-5（run case 16 翻转键）+ select_row 只动高亮
    #[test]
    fn options_panel_pointer_hit_and_select() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::About;
        let list = crate::menu_family::OptionList::new(true, false);
        sc.options = Some(list);
        let font = crate::paint::paint_font();
        let row_h = font.height() + 8;
        // 期望=字面常量锚（F-1 口径：禁被测自身作期望）——OptionList::add 累加式：
        // box_h=32+2*row_h（a.java:9911-9926）；行首=盒顶+16（paint case 16）
        // > 【2026-10-26 更正】盒顶取 **layout 画布**高（本测 Modern 18×18）——旧期望取
        // > SCREEN_H/W 把"Modern 面板留在经典左上区"钉成了正确（竖屏实机截图实证，用户报
        // > "存档UI没有居中"；menu_family::modern_center_tests）。
        let (lw, lh) = (sc.layout.screen_w, sc.layout.screen_h);
        assert_eq!((lw, lh), (18 * crate::layout::CELL_PX, 18 * crate::layout::CELL_PX), "Modern 18×18 格");
        let box_h = 32 + 2 * row_h;
        let y0 = (lh - box_h) / 2 + 16;
        let x = lw / 2; // 盒内任一 x（行带命中与 x 无关，盒外另测）
        assert_eq!(
            sc.pointer_hit(x, y0 + 1),
            PointerHit::Row { index: 0, confirm_key: -5 },
            "行0 顶像素归本行"
        );
        assert_eq!(sc.pointer_hit(x, y0 - 1), PointerHit::None, "行0 顶-1 不命中");
        assert_eq!(
            sc.pointer_hit(x, y0 + row_h),
            PointerHit::Row { index: 1, confirm_key: -5 },
            "行界像素归下行"
        );
        // select_row 只动高亮
        let hp = sc.player.hp;
        sc.select_row(1);
        if let Some(l) = sc.options.as_ref() {
            assert_eq!(l.highlight_index(), 1);
        } else {
            panic!();
        }
        assert_eq!(sc.player.hp, hp, "他态零变化");
    }

    /// 【STAGE-B-006】游戏内设置页：壳层附加行排在第 3-5 行；切换只改 `shell_options`（声音 / 小地图 / SKY_WAR 快照 /
    /// 脏位不变）；高亮在 5 行内环绕；-5 / -4 / -3 与"点行 → 选中 → 确认"都只翻附加行（复核 T1-F2）。
    #[test]
    fn in_game_settings_shell_rows_toggle_only_shell_options() {
        let rows: Vec<crate::menu_family::ShellOption> = [(1u8, "开屏画面", false), (2, "按键震动", true), (3, "按键音", true)]
            .into_iter()
            .map(|(id, l, checked)| crate::menu_family::ShellOption { id, label: l.encode_utf16().collect(), checked, value_text: None })
            .collect();
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18));
        sc.build_minimap();
        sc.shell_options = rows.clone();
        sc.menu_return_mode = GameMode::InGameMenu;
        sc.enter_menu16();
        sc.game_mode = GameMode::About;
        let hl = |sc: &super::GameScene| sc.options.as_ref().unwrap().highlight_index();
        assert_eq!(sc.options.as_ref().unwrap().len(), 5, "原两行 + 三行附加行");
        for want in [1, 2, 3, 4, 0] {
            sc.key_value = -2;
            sc.tick_menu16();
            assert_eq!(hl(&sc), want, "下移逐行走完一圈（第 5 行后回第 1 行）");
        }
        for want in [4, 3, 2, 1, 0] {
            sc.key_value = -1;
            sc.tick_menu16();
            assert_eq!(hl(&sc), want, "上移逐行走完一圈（第 1 行后到第 5 行）");
        }
        // 不变量逐键核对（复核 T2-F1：偶数次翻转会抵消布尔旁写，只在末尾断言抓不到）
        let inv = |sc: &super::GameScene| {
            let o = sc.options.as_ref().unwrap();
            (
                sc.bgm.clone(),
                sc.minimap_enabled,
                sc.minimap.is_some(),
                sc.global_snapshot(),
                sc.global_dirty,
                o.labels.clone(),
                o.checked.clone(),
            )
        };
        let before = inv(&sc);
        assert!(before.2, "前置：小地图已建");
        sc.options.as_mut().unwrap().set_highlight(3); // 按键震动
        for (k, expect) in [(-5, false), (-4, true), (-3, false)] {
            sc.key_value = k;
            sc.tick_menu16();
            assert_eq!(sc.shell_options[1].checked, expect, "键 {k}：按键震动行翻转");
            assert_eq!(sc.options.as_ref().unwrap().shell_rows()[1].checked, expect, "键 {k}：面板同步");
            assert_eq!(inv(&sc), before, "键 {k}：声音 / 小地图 / SKY_WAR / 脏位 / 原两行不变");
        }
        let (ry, rh) = sc.options.as_ref().unwrap().row_band(3, &sc.layout);
        assert_eq!(sc.pointer_hit(sc.layout.screen_w / 2, ry + rh / 2), PointerHit::Row { index: 3, confirm_key: -5 });
        sc.select_row(3);
        sc.key_value = -5;
        sc.tick_menu16();
        assert!(sc.shell_options[1].checked, "点行确认：按键震动行翻回开");
        assert_eq!((&sc.shell_options[0], &sc.shell_options[2]), (&rows[0], &rows[2]), "其余附加行不动");
        assert_eq!(inv(&sc), before, "点行确认：声音 / 小地图 / SKY_WAR / 脏位 / 原两行不变");
        sc.select_row(1);
        sc.key_value = -5;
        sc.tick_menu16();
        assert!(!sc.options.as_ref().unwrap().checked[1] && sc.minimap.is_none(), "原行为：小地图行照旧关小地图");
    }

    /// 存档槽列表（SlotSelect 面，2026-10-18 用户反馈）：行带单源 + 命中 ±1px +
    /// confirm=-5（run case 7：-6/-5 同义）+ set_cursor 只动光标+滚窗
    #[test]
    fn slot_list_pointer_hit_and_select() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::SaveSlot;
        let font = crate::paint::paint_font();
        let lh = font.height() + 4;
        // > 【2026-10-26 更正】行首=盒顶+16 的盒顶取 **layout 画布**高（同上 options 测更正）
        let (cw, ch) = (sc.layout.screen_w, sc.layout.screen_h);
        assert_eq!((cw, ch), (18 * crate::layout::CELL_PX, 18 * crate::layout::CELL_PX), "Modern 18×18 格");
        let y0 = (ch - sc.slot.content_h) / 2 + 16; // 字面锚：行首=盒顶+16
        let x = cw / 2;
        assert_eq!(
            sc.pointer_hit(x, y0 + 1),
            PointerHit::Row { index: 0, confirm_key: -5 },
            "槽0 顶像素归本行"
        );
        assert_eq!(sc.pointer_hit(x, y0 - 1), PointerHit::None, "槽0 顶-1 不命中");
        assert_eq!(
            sc.pointer_hit(x, y0 + lh),
            PointerHit::Row { index: 1, confirm_key: -5 },
            "行界像素归下槽"
        );
        let hp = sc.player.hp;
        sc.select_row(1);
        assert_eq!(sc.slot.cursor, 1);
        assert_eq!(sc.player.hp, hp, "他态零变化");
    }

    /// STAGE-B-001（2026-10-18 用户裁定）**modern 预设面**：boot 不再进 SoundAsk
    /// （mode 21），直入 MainMenu；默认开声；标题 BGM 请求=track2（防无声标题屏）。
    /// 2026-10-19 门控化（lead 裁定）：001 行为只在 modern 生效，classic 走原版时间轴
    /// （对测见 stage_b001_classic_keeps_sound_ask）。
    #[test]
    fn stage_b001_boot_skips_sound_ask() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        // 001 门控面 = modern 预设（classic 预设保留开机询问）
        m.layout = crate::layout::Layout::modern(13, 13);
        // logo 相位推进（paint 侧计数——直接驱动 logo 尾转出）
        for _ in 0..400 {
            m.tick(0);
            if m.mode != GameMode::Boot {
                break;
            }
            m.paints = m.paints.wrapping_add(1); // paint 计数推进（boot-replay 同构）
        }
        assert_eq!(
            m.mode,
            GameMode::MainMenu,
            "STAGE-B-001（modern）：boot 直入 MainMenu（不得停留 SoundAsk）"
        );
        // 默认开声（BgmState::new=true）
        assert!(m.bgm.sound_enabled, "STAGE-B-001（modern）：默认开声");
        // 标题 BGM 请求（硬要求 3：防无声标题屏）
        assert_eq!(m.bgm_request(), Some(2), "STAGE-B-001（modern）：标题 BGM=track2");
    }

    /// STAGE-B-001 门控化对测（2026-10-19）：classic 预设保留开机声音询问 = 原版
    /// 时间轴（a.java:3371-3373 → SoundAsk + case21 入场 soundEnabled=false a.java:4484）
    /// ——64 枚帧夹具 Java 参照面零偏离的命门回归。
    #[test]
    fn stage_b001_classic_keeps_sound_ask() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        // 默认布局 = classic（BootMachine::new 内建）——不再覆写
        for _ in 0..400 {
            m.tick(0);
            if m.mode != GameMode::Boot {
                break;
            }
            m.paints = m.paints.wrapping_add(1);
        }
        assert_eq!(
            m.mode,
            GameMode::SoundAsk,
            "classic 预设：boot 进 SoundAsk（原版时间轴，a.java:3371-3373）"
        );
        assert_eq!(m.softkeys, (1, 2), "case21 入场软键 (1,2)（a.java:4482-4478）");
        assert!(!m.bgm.sound_enabled, "case21 入场 soundEnabled=false（a.java:4484）");
    }

    /// 游戏内菜单"退出"（kind7）→ 回标题（Java gameMode=1 跨机语义）——
    /// 2026-10-18 用户实报修复：原 other 臂空转＝退出无反应。
    #[test]
    fn ingame_menu_exit_returns_to_title() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::InGameMenu;
        sc.menu_kinds = vec![7]; // 退出项
        sc.menu_item_count = 1;
        sc.menu_cursor = 0;
        sc.key_value = -6; // 确认（菜单族激活键）
        sc.tick(0, &[]);
        assert_eq!(
            sc.game_mode,
            GameMode::MainMenu,
            "kind7 退出须置 MainMenu（跨机信号→shell 换回 BootMachine）"
        );
        // 信号持久性：后续拍不得洗掉换机信号（2026-10-18 用户实报追查）
        sc.tick(1, &[]);
        assert_eq!(sc.game_mode, GameMode::MainMenu, "换机信号须持久");
    }

    /// 退出链无头复现（BUG-020 复发追查 2026-10-18）：标题→退出项→确认→
    /// LogoAnim 退场动画→counter>70→finished。断在哪段一测即知。
    #[test]
    fn boot_exit_chain_reaches_finished() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        // 开机相位推进在 paint 侧（boot-replay 口径）——探针直置 MainMenu 态验退出链本体
        m.mode = GameMode::MainMenu;
        m.title = Some(crate::title::TitleMachine::new());
        // 退出项 = cursor 末位（menu_kinds 尾 kind5）；下移键=-2
        for _ in 0..10 {
            m.tick(-2);
        }
        // 光标推到末位（kind5=LogoAnim=退出）
        let n = if let Some(t2) = m.title.as_ref() { t2.menu_kinds.len() } else { 0 };
        if let Some(t2) = m.title.as_mut() {
            t2.cursor = n as i32 - 1;
        }
        m.tick(-5);
        println!("确认后 mode={:?}", m.mode);
        for i in 0..80 {
            m.tick(0);
            if m.finished {
                println!("第 {i} 拍 finished=true");
                break;
            }
        }
        assert!(m.finished, "退出链断裂：80 拍后 finished 仍为 false（mode={:?}）", m.mode);
    }

    /// 标题侧设置面板（BootMachine::About，C-2 补面）：命中 + confirm=-5 + 只动高亮
    #[test]
    fn boot_options_pointer_hit_and_select() {
        use crate::paint::BootMachine;
        let mut m = BootMachine::new(
            crate::render::ArgbImage::create(1, 1),
            crate::render::ArgbImage::create(1, 1),
            (0..8).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..25).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            (0..2).map(|_| crate::render::ArgbImage::create(1, 1)).collect(),
            crate::render::ArgbImage::create(1, 1),
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
            vec![crate::render::ArgbImage::create(1, 1)],
        );
        m.mode = GameMode::About;
        let list = crate::menu_family::OptionList::new(true, false);
        m.options = Some(list);
        let font = crate::paint::paint_font();
        let row_h = font.height() + 8;
        let y0 = (SCREEN_H - (32 + 2 * row_h)) / 2 + 16; // 字面锚（禁自证）
        assert_eq!(
            m.pointer_hit(SCREEN_W / 2, y0 + 2),
            PointerHit::Row { index: 0, confirm_key: -5 }
        );
        assert_eq!(
            m.pointer_hit(SCREEN_W / 2, y0 + row_h + 2),
            PointerHit::Row { index: 1, confirm_key: -5 }
        );
        assert_eq!(m.pointer_hit(SCREEN_W / 2, y0 - 1), PointerHit::None);
        m.select_row(1);
        if let Some(l) = m.options.as_ref() {
            assert_eq!(l.highlight_index(), 1, "标题侧选行=只动高亮");
        } else {
            panic!();
        }
    }

    /// 【2026-10-26 竖屏实机】Modern 背包：格阵须在盒框内（盒=layout 居中，格原点=盒左+20、
    /// 盒顶+31+fh——paint 累积式）。实机 12×16 格竖屏：格阵画在经典 240×320 左上区、
    /// 盒框却居中 ⇒ 格阵与盒框脱节（截图实证）。翻页箭头 x 同源（盒右-22）。
    #[test]
    fn modern_item_grid_sits_inside_box_frame() {
        for layout in [crate::layout::Layout::modern(12, 16), crate::layout::Layout::modern(18, 18)] {
            let mut sc = scene((2, 2), &[]);
            sc.apply_layout(layout.clone());
            let bx = (layout.screen_w - sc.item_box_w) >> 1;
            let by = (layout.screen_h - sc.item_box_h) >> 1;
            let fh = crate::paint::paint_font().height;
            assert_eq!(
                sc.item_cell_origin(),
                (bx + 20, by + 31 + fh),
                "{}×{}：格原点=盒左+20 / 盒顶+31+fh（与 paint 盒框同源）",
                layout.screen_w,
                layout.screen_h
            );
        }
        // classic：盒=(240-w)/2 ⇒ 原值零偏离
        let sc = scene((2, 2), &[]);
        assert_eq!(
            sc.item_cell_origin(),
            ((SCREEN_W - sc.item_box_w) / 2 + 20, (SCREEN_H - sc.item_box_h) / 2 + 31 + crate::paint::paint_font().height),
            "classic 原值"
        );
    }

    /// 背包栅格：几何单源 + 命中 + (a) confirm_key=-6 + select_row
    #[test]
    fn item_grid_pointer_hit_and_select() {
        let mut sc = scene((2, 2), &[]);
        sc.apply_layout(crate::layout::Layout::modern(18, 18)); // Modern 面（清屏门控域）
        sc.game_mode = GameMode::ItemPanel;
        sc.item_cols = 4;
        sc.item_rows_total = 3;
        sc.item_scroll_top = 0;
        sc.item_scroll_bottom = 3;
        sc.item_cursor = 0;
        // 期望取字面常量（F-1：禁被测自身作期望）——锚 paint_item_panel 累积式
        //（盒顶+16+fh+4+1+10 = 盒顶+31+fh；(320-盒高)/2=…+31+14）
        // > 【2026-10-26 更正】盒顶/盒左取 **layout 画布**（本测 Modern 18×18；paint 盒框早已
        // > layout 居中）——旧期望字面 SCREEN_W/H 把"格阵与盒框脱节"的缺陷钉成了正确
        // >（竖屏实机截图实证，modern_item_grid_sits_inside_box_frame）。
        let (lw, lh) = (sc.layout.screen_w, sc.layout.screen_h);
        assert_eq!((lw, lh), (18 * crate::layout::CELL_PX, 18 * crate::layout::CELL_PX), "Modern 18×18 格");
        let ox = (lw - sc.item_box_w) / 2 + 20;
        let oy = (lh - sc.item_box_h) / 2 + 31 + crate::paint::paint_font().height;
        assert_eq!(sc.item_cell_origin(), (ox, oy), "格原点与 paint 累积式同式");
        // 顶像素 ±1px 归属锚（栅格版，照菜单侧规格）
        assert_eq!(sc.pointer_hit(ox + 16, oy), PointerHit::Row { index: 0, confirm_key: -6 }, "格顶像素归本格");
        assert_eq!(sc.pointer_hit(ox + 16, oy - 1), PointerHit::None, "格顶-1 不命中");
        assert_eq!(sc.pointer_hit(ox + 16, oy + 16), PointerHit::Row { index: 0, confirm_key: -6 });
        assert_eq!(
            sc.pointer_hit(ox + 40 + 16, oy + 40 + 16),
            PointerHit::Row { index: 5, confirm_key: -6 }
        );
        assert_eq!(sc.pointer_hit(ox - 5, oy + 16), PointerHit::None);
        sc.select_row(5);
        assert_eq!(sc.item_cursor, 5);
        assert_eq!(sc.item_scroll_top, 0);
        // 越窗行 → 滚窗同步
        sc.select_row(11);
        assert_eq!(sc.item_cursor, 11);
        assert_eq!(sc.item_scroll_bottom, 3, "末行仍全窗");
    }
