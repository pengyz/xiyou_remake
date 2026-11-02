    use crate::render::FontMetrics;
    use super::*;
    use crate::menu_family::slot_equip_test_support::*;

    fn solid(w: i32, h: i32) -> ArgbImage {
        ArgbImage::from_rgb(&vec![0xFF202020; (w * h) as usize], w, h, true)
    }

    fn many(n: usize) -> Vec<ArgbImage> {
        (0..n).map(|_| solid(24, 24)).collect()
    }

    /// maplv1 夹具 + 合成图集（sptarm = synthetic_sptarm，经 m_122 映射进 entity）。
    fn fixtures() -> (Vec<i32>, Vec<i32>) {
        let width_table: Vec<i32> = include_str!("../../tests/fixtures/type_width_px.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        let walk: Vec<i32> = include_str!("../../tests/fixtures/tile_walkability.txt")
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect();
        (width_table, walk)
    }

    fn synth_images(icons: &[ArgbImage]) -> SceneImages {
        SceneImages {
            backdrop: solid(77, SCREEN_H),
            shadow: solid(24, 24),
            actor: solid(24, 24),
            actor_face: solid(24, 24),
            ui: many(25),
            stair_icons: (solid(24, 24), solid(24, 24)),
            open_anim: solid(24, 24),
            opened_door: solid(24, 24),
            entity: SceneImages::map_entity_images(&many(13), &many(23), icons, &many(20), &many(20)),
            popup: crate::popup::PopupImages {
                digit_strips: [solid(24, 24), solid(24, 24), solid(24, 24)],
                icon_strip: solid(24, 24),
                battle_digits: solid(24, 24),
            },
            spark_sheet: solid(24, 24),
            corner_sprites: solid(24, 24),
            map211: None,
            map210: solid(24, 24),
            menu_strip: solid(24, 24),
            battle_icon: solid(24, 24),
            battle_strip: solid(24, 24),
            faces: many(12),
            ending: solid(24, 24),
            xtq: many(6),
        }
    }

    fn scene_with(icons: &[ArgbImage]) -> GameScene {
        let (width_table, walk) = fixtures();
        GameScene::load_floor(
            include_bytes!("../../tests/fixtures/maplv1.bin"),
            3 << 5,
            10 << 5,
            &[],
            &width_table,
            &walk,
            synth_images(icons),
            vec![None; 56],
        )
        .unwrap()
    }

    /// 【BUG-030】金币弹字（spawnPopup kind 4，a.java:9706 `popupX = 240`）右缘锚**跟随金币列**：
    /// classic 240 = 金币数值右锚 237 + 3；modern 金币列贴右（`screen_w − 3`）⇒ 弹字须同步贴右。
    /// 判据 = 弹字首位裁剪 x − 金币数值右锚，两预设相等（旧字面量 240 ⇒ modern 偏左 screen_w − 240）。
    #[test]
    fn gold_popup_tracks_gold_column() {
        let icons = synthetic_sptarm();
        let offset = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            let dw = sc.images.popup.digit_strips[0].width / 11;
            sc.popups.spawn(4, 1, 0, 0, sc.view.cam_x, sc.view.cam_y, dw);
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let mut g = SoftGraphics::new(&mut img);
            sc.popups.draw(&mut g, &sc.images.popup);
            let first = g.ops.iter().find(|o| o.starts_with("setClip(")).expect("金币弹字裁剪");
            op_xy(first).unwrap().0 - sc.layout.status_gold_num_x
        };
        let classic = offset(crate::layout::Layout::classic());
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(
                offset(crate::layout::Layout::modern(gw, gh)),
                classic,
                "modern {gw}×{gh}：金币弹字须对齐金币列（与 classic 同相对位）"
            );
        }
    }

    /// `setClip(x,y,w,h)` 四元解析（reset 臂 `setClip(0,0,画布)` 同格式）。
    fn clip4(o: &str) -> Option<(i32, i32, i32, i32)> {
        let a = o.strip_prefix("setClip(")?.strip_suffix(')')?;
        let v: Vec<i32> = a.split(',').filter_map(|t| t.trim().parse().ok()).collect();
        (v.len() == 4).then(|| (v[0], v[1], v[2], v[3]))
    }

    /// 【BUG-031①】walkPhase==2 观察形态四角箭头（a.java:5964-5967：`(5−bob,148)` `(109,25−bob)`
    /// `(109,viewBottom−29+bob)` `(215+bob,148)`）：148 = 160−12、109 = 120−11、215 = 240−25 是屏心/屏宽
    /// 折叠字面量，同块下臂早随 view_bottom（BUG-029 同型）⇒ modern 须随 center_y / center_x / screen_w。
    #[test]
    fn soul_view_corner_arrows_frame_the_screen() {
        let icons = synthetic_sptarm();
        for layout in [
            crate::layout::Layout::classic(),
            crate::layout::Layout::modern(12, 16),
            crate::layout::Layout::modern(13, 13),
            crate::layout::Layout::modern(18, 18),
        ] {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.walk_phase = 2;
            sc.view_locked = false;
            let bob = sc.bob_offset;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint_player(&mut g, sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
                g.ops.clone()
            };
            let arrows: Vec<(i32, i32)> =
                ops.iter().filter_map(|o| clip4(o)).filter(|c| (c.2, c.3) == (22, 24)).map(|c| (c.0, c.1)).collect();
            let (cx, cy, vb, w) = (layout.center_x, layout.center_y, sc.view_bottom, layout.screen_w);
            assert_eq!(
                arrows,
                vec![(5 - bob, cy - 12), (cx - 11, 25 - bob), (cx - 11, vb - 29 + bob), (w - 25 + bob, cy - 12)],
                "{}×{}：四角箭头须框住画面（左/右屏心 y、上/下屏心 x、右贴右缘）",
                layout.screen_w,
                layout.screen_h
            );
        }
    }

    /// 【BUG-031②】炼丹（mode 9，a.java:2696-2722）：框 / 花费行 / 高亮行已随屏居中，三行属性图标 x 97、
    /// 加值 x 142（= 高亮行 x 62 + 35 / + 80）仍是 classic 字面量 ⇒ modern 落到框外。判据：高亮行之后的
    /// 全部绘制 op 相对高亮行左上的坐标序列，modern 与 classic 逐项相等（纯平移）。
    #[test]
    fn alchemy_rows_translate_with_highlight_row() {
        let icons = synthetic_sptarm();
        let rel = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::Alchemy;
            sc.alchemy_cursor = 0;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let hi = ops
                .iter()
                .position(|o| o.starts_with("fillRect(") && o.contains(",115,16,"))
                .expect("高亮行 fillRect(x,y,115,16)");
            let (rx, ry) = op_xy(&ops[hi]).unwrap();
            ops[hi + 1..]
                .iter()
                .filter(|o| !o.starts_with("setClip(0,0,")) // 整画布 reset 臂不属行内容
                .filter_map(|o| {
                    let (x, y) = op_xy(o)?;
                    Some((o.split('(').next().unwrap_or("").to_string(), x - rx, y - ry))
                })
                .collect::<Vec<_>>()
        };
        let classic = rel(crate::layout::Layout::classic());
        assert!(classic.len() >= 6, "前置：三行图标+加值均有绘制 op：{classic:?}");
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(rel(crate::layout::Layout::modern(gw, gh)), classic, "modern {gw}×{gh}：炼丹行内容须随高亮行平移");
        }
    }

    /// 【BUG-031③】敌人图鉴（mode 5，a.java:2497-2595）：面板已随屏居中（modern 面板宽 = 屏宽 − 32），
    /// 翻页箭头 x 112（= 120 − 8 屏心折叠）与预测伤害列 `(240 + 面板宽 >> 1) − 75`（SCREEN_W 常量）未随动。
    /// 判据：箭头相对面板中线、伤害图标相对面板右缘的 x，modern 与 classic 相等。
    #[test]
    fn encyclopedia_arrows_and_damage_column_follow_panel() {
        let icons = synthetic_sptarm();
        let probe = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::EnemyInfo;
            sc.images.ui[16] = solid(17, 9); // 预测伤害图标：独有尺寸便于定位
            sc.menu.encyc_panel_w = sc.layout.encyc_panel_w;
            sc.menu.encyc_types = vec![41, 42, 43];
            sc.menu.encyc_damage = vec![10, 20, -1];
            sc.menu.encyc_count = 3;
            sc.menu.encyc_rows_visible = 1;
            sc.menu.encyc_panel_h = 72;
            sc.menu.encyc_scroll_top = 1; // 中间页：上下翻页箭头都画
            sc.menu.encyc_scroll_bottom = 2;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let bx = (layout.screen_w - sc.menu.encyc_panel_w) >> 1;
            let (mid, right) = (bx + (sc.menu.encyc_panel_w >> 1), bx + sc.menu.encyc_panel_w);
            let arrows: Vec<i32> =
                ops.iter().filter_map(|o| clip4(o)).filter(|c| (c.2, c.3) == (15, 17)).map(|c| c.0 - mid).collect();
            let dmg: Vec<i32> =
                ops.iter().filter(|o| o.starts_with("drawImage(17x9,")).filter_map(|o| op_xy(o)).map(|p| p.0 - right).collect();
            (arrows, dmg)
        };
        let classic = probe(crate::layout::Layout::classic());
        assert_eq!(classic, (vec![-8, -8], vec![-87]), "classic：箭头 112 = 中线 120 − 8；伤害图标 149 − 12 = 右缘 224 − 87");
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(probe(crate::layout::Layout::modern(gw, gh)), classic, "modern {gw}×{gh}：箭头随面板中线、伤害列随面板右缘");
        }
    }

    /// 【BUG-031④】记录浏览（mode 12，a.java:2905-2914）：页码 x 120、双箭头 x `60 − bob` / `160 + bob` 为屏心
    /// 派生（Layout 早有 center_x / arrow_pair_left_x / arrow_pair_right_x，标题页已用）⇒ modern 须随屏心。
    /// 判据：页码主色串与两箭头的 x 相对 center_x，modern 与 classic 相等（y 见 BUG-032）。
    #[test]
    fn record_browser_indicator_and_arrows_follow_center() {
        let icons = synthetic_sptarm();
        let probe = |layout: crate::layout::Layout| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::RecordBrowser;
            sc.images.ui[14] = solid(20, 13); // 箭头图：独有尺寸便于定位
            sc.persist.journal = vec![(0, 0), (0, 1), (0, 2)];
            sc.menu.record_page = 1; // 中间页：左右箭头都画
            sc.menu.page_indicator = "2 / 3".into();
            sc.dialog.overlay_active = false;
            let bob = sc.bob_offset;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let cx = layout.center_x;
            let text = ops.iter().filter(|o| o.starts_with("drawString(")).filter_map(|o| op_xy_after_text(o)).last().map(|p| p.0 - cx);
            let arrows: Vec<i32> = ops
                .iter()
                .filter(|o| o.starts_with("drawImage(20x13,") || o.starts_with("drawImageT(20x13,"))
                .filter_map(|o| op_xy(o))
                .map(|p| p.0 - cx)
                .collect();
            (text, arrows, bob)
        };
        let classic = probe(crate::layout::Layout::classic());
        let bob = classic.2;
        assert_eq!((classic.0, classic.1.clone()), (Some(0), vec![-60 - bob, 40 + bob]), "classic：页码 120、箭头 60−bob / 160+bob");
        for (gw, gh) in [(12, 16), (13, 13), (18, 18)] {
            assert_eq!(probe(crate::layout::Layout::modern(gw, gh)), classic, "modern {gw}×{gh}：页码与箭头须随屏心");
        }
    }

    /// 【BUG-032】记录浏览双箭头 y（a.java:2908 `var46 += fontHeight + 4 - 13 >> 1`；javap 5242-5255
    /// `iload_2 … iconst_1 ishr iadd`）= 页码行底 + ((fh + 4 − 13) >> 1)，非 (行底 + …) >> 1。两预设同式。
    #[test]
    fn record_browser_arrow_y_sits_beside_indicator() {
        let icons = synthetic_sptarm();
        for layout in [crate::layout::Layout::classic(), crate::layout::Layout::modern(12, 16)] {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout.clone());
            sc.game_mode = GameMode::RecordBrowser;
            sc.images.ui[14] = solid(20, 13);
            sc.persist.journal = vec![(0, 0), (0, 1), (0, 2)];
            sc.menu.record_page = 1;
            sc.dialog.overlay_active = false;
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            let fh = crate::paint::paint_font().height;
            let text_bottom = layout.screen_h - fh - 4;
            let ys: Vec<i32> = ops
                .iter()
                .filter(|o| o.starts_with("drawImage(20x13,") || o.starts_with("drawImageT(20x13,"))
                .filter_map(|o| op_xy(o))
                .map(|p| p.1)
                .collect();
            let y = text_bottom + ((fh + 4 - 13) >> 1);
            assert_eq!(ys, vec![y, y], "{}×{}：箭头 y = 页码行底 + ((fh+4−13)>>1)", layout.screen_w, layout.screen_h);
        }
    }

    /// `drawString(text,x,y,anchor)` 的 (x,y)（跳过首个文本参数）。
    fn op_xy_after_text(o: &str) -> Option<(i32, i32)> {
        let args = o.strip_prefix("drawString(")?.strip_suffix(')')?;
        let nums: Vec<i32> = args.rsplitn(4, ',').filter_map(|t| t.trim().parse().ok()).collect();
        // rsplitn 逆序：anchor, y, x
        (nums.len() == 3).then(|| (nums[2], nums[1]))
    }

    /// STAGE-B-002 ② 消费面：paint_status_bar 锚位随 `self.layout`（modern 贴右/
    /// 黑填充界），classic 零偏离（既有像素/ops 测全绿护航）。
    #[test]
    fn status_bar_anchors_follow_layout() {
        let icons = synthetic_sptarm();
        let norm = |o: &String| o.replace(", ", ",");
        // classic：填充 (65,0,175,18)——240-65=175，原值
        let mut sc = scene_with(&icons);
        let mut img = ArgbImage::create(crate::layout::SCREEN_W, crate::layout::SCREEN_H);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_status_bar(&mut g);
            let ops: Vec<String> = g.ops.iter().map(norm).collect();
            assert!(
                ops.iter().any(|o| o.contains("fillRect(65,0,175,18,")),
                "classic 原值填充：{ops:?}"
            );
        }
        // modern(13,13)=416：填充 (65,0,351,18)——416-65=351；钥匙首槽 clip x=241（贴右）
        let mut sc = scene_with(&icons);
        sc.layout = crate::layout::Layout::modern(13, 13);
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_status_bar(&mut g);
            let ops: Vec<String> = g.ops.iter().map(norm).collect();
            assert!(
                ops.iter().any(|o| o.contains("fillRect(65,0,351,18,")),
                "modern 贴右填充宽随屏：{ops:?}"
            );
            assert!(
                ops.iter().any(|o| o.contains("setClip(241,0,18,16)")),
                "modern 钥匙首槽贴右 x=241：{ops:?}"
            );
        }
    }

    /// 返修判据（用户实机三破绽 2026-10-18）：modern 渲染**分布**——瓦片绘制占
    /// [view_top, view_bottom) 全宽、HUD/软键落底带。钉"绘制坐标系整体换轨"
    /// （002 首版只连了 Layout 字段值、draw 落点仍是旧坐标系 ⇒ 地图只画左上角块、
    /// 属性浮屏中、钥/对话框各自为政）。
    #[test]
    fn modern_render_distribution_spans_full_canvas() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        // op 坐标抽取（"name(x,y,…)" → (x,y)）
        let coords: Vec<(i32, i32)> = ops
            .iter()
            .filter_map(|o| {
                let args = o.split('(').nth(1)?.split(')').next()?;
                let mut it = args.split(',');
                let x: i32 = it.next()?.trim().parse().ok()?;
                let y: i32 = it.next()?.trim().parse().ok()?;
                Some((x, y))
            })
            .collect();
        // ① 地图瓦片（clip 16×16 专属模式）横跨全宽且填满可视窗：
        //    破绽现状=旧视口 15 列×~16 行 ≈240 片、max_x≈224；换轨后 26 列 ≥380 片、max_x≥400
        let tiles: Vec<(i32, i32)> = ops
            .iter()
            .filter(|o| o.starts_with("setClip(") && o.ends_with(",16,16)"))
            .filter_map(|o| {
                let args = o.split('(').nth(1)?.split(')').next()?;
                let mut it = args.split(',');
                let x: i32 = it.next()?.trim().parse().ok()?;
                let y: i32 = it.next()?.trim().parse().ok()?;
                Some((x, y))
            })
            .collect();
        let tile_max_x = tiles.iter().map(|t| t.0).max().unwrap_or(0);
        assert!(tile_max_x >= 380, "modern 瓦片须横跨画布全宽（现 tile_max_x={tile_max_x}）");
        assert!(
            tiles.len() >= 380,
            "modern 瓦片须填满可视窗（现 {} 片，旧视口窗≈240）",
            tiles.len()
        );
        // ② HUD 落底带（≥366）：现破绽=属性浮屏中（旧 hud_base=270）
        let bottom_max_y = coords.iter().map(|c| c.1).max().unwrap_or(0);
        assert!(bottom_max_y >= 366, "modern HUD/软键须落底带（现 bottom_max_y={bottom_max_y}）");
    }

    /// 返修判据补：modern 画布绘制零大幅越界（坐标系整体换轨后无旧系残留出框）。
    #[test]
    fn modern_render_no_draw_outside_canvas() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            for o in &g.ops {
                let Some(args) = o.split('(').nth(1).and_then(|s| s.split(')').next()) else {
                    continue;
                };
                let nums: Vec<i32> = args
                    .split(',')
                    .filter_map(|t| t.trim().trim_start_matches('#').parse().ok())
                    .collect();
                if let (Some(&x), Some(&y)) = (nums.first(), nums.get(1)) {
                    assert!(x < 416 + 64 && y < 416 + 64, "越界绘制 {o}（坐标系残留）");
                }
            }
        }
    }

    /// 残项清账判据（lead 派单 2026-10-18，探针三选一结论入册）：
    /// - mapbg 77×320 **全不透明**（探针实证）⇒ 排除"透明透出画布黑"；
    /// - 黑区像素 **alpha=0（0x00000000 未绘制面）**（自验图解码实证）⇒ 排除"黑瓦片/画出来的黑"；
    /// - 故来源=**未绘制面**（mode 3 链在地图下缘悬垂带没垫任何层）。
    ///   本判据钉"无未解释黑区"：modern 全链后视口带内未绘制像素 = 0。
    #[test]
    fn modern_render_no_unexplained_black() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let packed = std::fs::read(repo.join("assets/raw/mapbg")).unwrap();
        let bg = game_data::PackedPng::parse(&packed, 1)
            .unwrap()
            .images
            .iter()
            .map(|s| ArgbImage::from_decoded_png(game_data::decode_png(s.bytes).unwrap()))
            .next()
            .unwrap();
        // mapbg 无透明（来源分类断言①：排除透出候选）
        assert!(
            bg.argb.iter().all(|p| p >> 24 == 0xFF),
            "mapbg 须全不透明（若有透明则黑区另有透出路径）"
        );
        let mut sc = scene_with(&synthetic_sptarm());
        sc.images.backdrop = bg;
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
        }
        // 无未解释黑区（来源分类断言②：alpha=0 未绘制面须为 0）——视口带 [20,366)
        let mut undrawn = 0usize;
        for y in 20..366 {
            for x in 0..416 {
                if img.argb[y * 416 + x] == 0 {
                    undrawn += 1;
                }
            }
        }
        assert_eq!(undrawn, 0, "modern 视口带内未绘制像素数（黑区根因面）");
    }

    /// v6 判据（用户"背景颜色很奇怪/过渡不自然"）：真背景图**边缘钳制延展**——
    /// 顶带像素=图顶行色、底带=图底行色；镜像反转（沙地翻上屏顶/天空翻下底带）即红。
    #[test]
    fn modern_backdrop_edge_clamp_no_mirror_flip() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let packed = std::fs::read(repo.join("assets/raw/mapbg")).unwrap();
        let bg = game_data::PackedPng::parse(&packed, 1)
            .unwrap()
            .images
            .iter()
            .map(|s| ArgbImage::from_decoded_png(game_data::decode_png(s.bytes).unwrap()))
            .next()
            .unwrap();
        let bw = bg.width as usize;
        let bh = bg.height as usize;
        let (top_row, bottom_row) = (bg.argb[0], bg.argb[(bh - 1) * bw]);
        let mut sc = scene_with(&synthetic_sptarm());
        sc.images.backdrop = bg;
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.backdrop_scroll = -1;
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_parallax(&mut g, true);
        }
        let top_px = img.argb[4 * 416 + 40]; // 顶带采样
        let bottom_px = img.argb[411 * 416 + 40]; // 底带采样
        assert_eq!(top_px, top_row & 0x00FF_FFFF | 0xFF00_0000, "顶带=图顶行色（钳制延展）");
        assert_eq!(bottom_px, bottom_row & 0x00FF_FFFF | 0xFF00_0000, "底带=图底行色（钳制延展）");
    }

    /// modern 双缺陷判据（lead classic 对照定案 2026-10-18）：
    /// ① 顶栏金币图标必须绘出（像素级：图标位≠状态栏底色——裁剪复位常量化会把
    ///    绘制裁掉但 ops 仍在，故必须像素判）；② 对话文本必须绘出（drawString 在场）。
    #[test]
    fn modern_topbar_gold_icon_and_dialog_text_drawn() {
        let icons = synthetic_sptarm();
        // ① 金币图标像素判（modern 状态栏：图标位 (gold_icon_x+8, 8) ≠ STATUS_FILL）
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
        }
        let gx = sc.layout.status_gold_icon_x as usize;
        let icon_px = img.argb[8 * 416 + gx + 8];
        assert_ne!(
            icon_px, STATUS_FILL,
            "金币图标须绘出（现像素={icon_px:#x} = 底色 ⇒ 被裁剪复位裁掉）"
        );
        // ② 对话文本在场（对话态绘制含 drawString）
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.game_mode = GameMode::Dialog;
        sc.open_dialog_page(0);
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        assert!(
            ops.iter().any(|o| o.starts_with("drawString") || o.starts_with("drawSubstring")),
            "对话文本须绘出（drawString/drawSubstring 在场）：{:?}",
            ops.iter().filter(|o| o.contains("tring")).collect::<Vec<_>>()
        );
        // 【v7 判据】对话框体底色=HUD_FILL（用户实报"框底色被背景覆盖"——
        // 裁剪复位常量漏网把 body 填充裁掉、露背景）
        let body = img.argb[404 * 416 + 260];
        assert_eq!(body, HUD_FILL | 0xFF00_0000, "对话框体底色须=HUD_FILL（现={body:#010x}=背景色透出）");
    }

    /// P0 补修判据（2026-10-20 存档入口漏网）：loadFloorSave 直入 26 格楼层——
    /// tile 专色全绘（半绘=视图重建残留 240-era 范围）。真修点=load_floor_from_save
    /// 路径（loadSlotTable→loadFloorSave→视图重建）。
    #[test]
    fn modern_save_load_tiles_cover_full_view() {
        let (wide, high) = (26i32, 26i32);
        let mut maplv = Vec::new();
        maplv.extend_from_slice(&((wide * 2) as u16).to_le_bytes());
        maplv.extend_from_slice(&((high * 2) as u16).to_le_bytes());
        let gl = (wide * high * 4) as usize;
        maplv.extend_from_slice(&vec![1u8; gl]);
        maplv.extend_from_slice(&vec![0u8; gl]);
        // FloorSave：preset-floor2-mov 的 MOT_L0 真 blob（save 形状=Java loadFloorSave 布局）
        let stores = crate::save::parse_preset(include_str!("../../tests/fixtures/preset-floor2-mov.preset"));
        let blob = stores.iter().find(|(k, _)| k.starts_with("MOT_L0@")).unwrap().1.clone();
        let save = crate::save::parse_floor_save(&blob, 0).unwrap();
        let icons = synthetic_sptarm();
        let (wt, walk) = fixtures();
        let mut sc = load_floor_from_save(
            &maplv,
            &[],
            &wt,
            &walk,
            synth_images(&icons),
            vec![None; 56],
            &save,
        )
        .unwrap();
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut tileset = ArgbImage::create(64, 64);
        tileset.argb.fill(0xFF00_FF00);
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &tileset);
        }
        let w = 416usize;
        let mut right_tiles = 0;
        for x in 300..410 {
            if img.argb[200 * w + x] == 0xFF00_FF00 {
                right_tiles += 1;
            }
        }
        assert!(
            right_tiles > 20,
            "存档直入后右半须绘出 tile（现 {right_tiles}/110 专色）——半绘 artifact"
        );
    }

    /// P0 回归判据（2026-10-20 modern 地形半绘·真修点路径）：modern 换层加载
    /// 26 格宽楼层后 tile 层须绘满视口（右半空白=半绘 artifact）——走
    /// load_floor_data 重建视图路径（from_maplv classic 初始+布局换轨修复点）。
    #[test]
    fn modern_floor_switch_tiles_cover_full_view() {
        // 26 格宽合成 maplv（头 u16LE=52；tile 全 1=可见地形）
        let (wide, high) = (26i32, 26i32);
        let mut maplv = Vec::new();
        maplv.extend_from_slice(&((wide * 2) as u16).to_le_bytes());
        maplv.extend_from_slice(&((high * 2) as u16).to_le_bytes());
        let gl = (wide * high * 4) as usize;
        // 地形段全置 tile 1（绘制索引=行距 wide·2、列距 1——全段覆盖免索引形错）
        maplv.extend_from_slice(&vec![1u8; gl]);
        maplv.extend_from_slice(&vec![0u8; gl]);
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.load_floor_data(&maplv, 10 << 5, 10 << 5, &[], 3).unwrap();
        // tileset=专色绿（tile 专色判据——背板/其他层误计防）
        let mut tileset = ArgbImage::create(64, 64);
        tileset.argb.fill(0xFF00_FF00);
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &tileset);
        }
        // 右半视口（x=300..410）中带行（y=200）须有 tile 专色像素
        let w = 416usize;
        let mut right_tiles = 0;
        for x in 300..410 {
            if img.argb[200 * w + x] == 0xFF00_FF00 {
                right_tiles += 1;
            }
        }
        assert!(
            right_tiles > 20,
            "modern 换层后右半须绘出 tile（现 {right_tiles}/110 专色）——半绘 artifact"
        );
    }

    /// BUG-022 判据（用户实机 2026-10-20"扩大分辨率后，切屏时，切换动画是按
    /// 传统分辨率绘制的，没有覆盖全屏"）：换层遮幅**全黑帧**（wipe=4，格 8px
    /// 无缝）须覆盖整个画布——旧硬编码 40 行×30 列（=240×320 传统分辨率）只盖
    /// 左上角。双臂：modern 416×416=主判据；classic 240×320=零漂移锚（同判据
    /// 恒过=网格仍为 30×40 原值）。
    #[test]
    fn wipe_full_black_covers_whole_canvas() {
        for (tag, layout, w, h) in [
            ("classic", crate::layout::Layout::classic(), 240i32, 320i32),
            ("modern", crate::layout::Layout::modern(13, 13), 416, 416),
        ] {
            let icons = synthetic_sptarm();
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            // 全黑帧：closing 相位推进 3→4（size=8/off=0 无缝格）；
            // wipe=4 不触 swap_floor（仅 wipe>4 才换层）。
            sc.transitioning = true;
            sc.wipe_closing = true;
            sc.wipe = 3;
            let mut img = ArgbImage::create(w, h);
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
            }
            for (i, p) in img.argb.iter().enumerate() {
                assert_eq!(
                    *p, 0xFF00_0000,
                    "[{tag}] 遮幅全黑帧像素 {i} 未覆盖（旧网格止步 240×320）"
                );
            }
        }
    }

    /// BUG-022 连带判据：遮幅点阵**中段相位**（wipe=2，格 4px/off=2）同样须铺满
    /// 画布右下角——旧网格点阵末格止步 (234,314)，右下角格永不落点。
    #[test]
    fn modern_wipe_dither_spans_whole_canvas() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        sc.transitioning = true;
        sc.wipe_closing = true;
        sc.wipe = 1; // closing 相位 1→2（size=4/off=2）
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint_wipe_tail(&mut g);
        }
        // 末格 (51,51) 起点 off+51*8=410，覆盖 410..414（中段相位右下角格）
        assert_eq!(
            img.argb[412 * 416 + 412], 0xFF00_0000,
            "modern 中段点阵右下角格未落点（旧网格止步 240×320）"
        );
    }

    /// STAGE-B-004a ①c 判据（对话框底边距=游戏视图底黑边）：modern 无对话态下
    /// 底行面板两侧不得留黑——背景带须补铺到底（黑边=未补铺 artifact）。
    #[test]
    fn modern_bottom_band_sides_never_black() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let mut img = ArgbImage::create(416, 416);
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
        }
        let w = 416usize;
        for (x, y) in [(5, 415), (410, 415), (5, 400), (410, 400)] {
            let p = img.argb[y * w + x];
            assert_ne!(
                p & 0x00FF_FFFF,
                0,
                "({x},{y}) 不得黑（游戏视图底黑边 artifact，现={p:#010x}）"
            );
        }
    }

    /// 底部区口径修正判据（lead 设计口径级修正 2026-10-18：**经典尺寸固定+整体居中**，
    /// 非随屏拉伸）：面板宽 == 经典常量、居中偏移 = (screen_w−经典面板宽)/2、
    /// classic 下偏移回代原值（0）。
    #[test]
    fn bottom_panel_classic_size_centered() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let dx = sc.bottom_panel_dx();
        assert_eq!(dx, (416 - 240) / 2, "modern 面板居中偏移 =（屏宽−经典面板宽 240）/2");
        sc.game_mode = GameMode::Dialog;
        sc.open_dialog_page(0);
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        // 对话框填充宽 == 经典常量 149、起点 == 80+dx（面板内相对布局=classic 同构）
        assert!(
            ops.iter()
                .any(|o| o.contains(&format!("fillRect({},", 80 + dx)) && o.contains(",149,")),
            "对话框填充宽须为经典常量 149（拉伸=违口径）：{:?}",
            ops.iter().filter(|o| o.contains("149") || o.contains("fillRect(80")).collect::<Vec<_>>()
        );
        // classic 回代：偏移 0（240 宽面板在 240 屏=原值）
        let mut sc2 = scene_with(&icons);
        assert_eq!(sc2.bottom_panel_dx(), 0, "classic 面板偏移回代原值 0（零偏离）");
        let _ = &mut sc2;
    }

    /// 底部区像素体检（用户复验 2026-10-18"渲染还是不对"）：量对话文本包围盒的
    /// **实测数**，classic vs modern 对照——文本字号/位置/框体关系拿数说话。
    #[test]
    // 【2026-10-19】依赖 analysis/b002-*-v7.png（可再生工作区不入库）——
    // clean 树/CI 必红（STAGE-B-002 v7 实验探针污染正式测试）。标 ignore
    // 待自包含化（生成 PNG 再探）或移实验区。lead 处置。
    #[ignore = "实验探针：依赖 analysis/ 探针 PNG（可再生工作区），自包含化后转正"]
    fn probe_bottom_region_pixel_metrics() {
        let icons = synthetic_sptarm();
        for (label, layout) in [
            ("classic", crate::layout::Layout::classic()),
            ("modern", crate::layout::Layout::modern(13, 13)),
        ] {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            sc.game_mode = GameMode::Dialog;
            sc.open_dialog_page(0);
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
            }
            let dx = sc.bottom_panel_dx();
            // 文本行带（block_h=40 ⇒ y0=h-40+12 起）内非底色像素包围盒
            let band_y0 = h as i32 - 40 + 12;
            let (mut x_min, mut x_max, mut y_min, mut y_max) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
            for y in band_y0..(h as i32 - 2) {
                for x in 0..w as i32 {
                    let p = img.argb[(y as usize) * w + x as usize];
                    if p != 0 && p != HUD_FILL && p != STATUS_FILL {
                        x_min = x_min.min(x);
                        x_max = x_max.max(x);
                        y_min = y_min.min(y);
                        y_max = y_max.max(y);
                    }
                }
            }
            eprintln!(
                "PROBE {label} dx={dx} 文本带包围盒 x[{x_min},{x_max}] y[{y_min},{y_max}] 宽={} 高={}",
                x_max - x_min + 1,
                y_max - y_min + 1
            );
        }
        // 真图实测：解码两预设自验图，量底部带逐行内容列段（非底色像素游程）
        // +【v7】对话框体打点：body/顶条/左面板三点采样——框内应=框底色，
        // 若=沙色即"对话框底色被背景覆盖/透印"实证（用户 2026-10-18 质询）
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for (label, file) in [
            ("classic", "analysis/b002-classic-v7.png"),
            ("modern", "analysis/b002-modern-v7.png"),
        ] {
            let bytes = std::fs::read(repo.join(file)).unwrap();
            let dec = game_data::decode_png(&bytes).unwrap();
            let (w, h) = (dec.width as usize, dec.height as usize);
            let px = |x: usize, y: usize| dec.argb[y * w + x];
            eprintln!(
                "PROBE shot-{label} {w}x{h} 框体打点 body({},{})={:#010x} topstrip({},{})={:#010x} leftpanel({},{})={:#010x}（HUD_FILL={:#010x}）",
                w * 5 / 8, h - 12, px(w * 5 / 8, h - 12),
                w / 2, h - 34, px(w / 2, h - 34),
                w / 6, h - 12, px(w / 6, h - 12),
                HUD_FILL | 0xFF00_0000
            );
            for y in (h - 50)..h {
                let mut runs: Vec<(usize, usize)> = Vec::new();
                let mut start = usize::MAX;
                for x in 0..w {
                    let p = dec.argb[y * w + x];
                    let bg = p == 0 || p == HUD_FILL || p == STATUS_FILL;
                    if !bg && start == usize::MAX {
                        start = x;
                    }
                    if bg && start != usize::MAX {
                        runs.push((start, x - 1));
                        start = usize::MAX;
                    }
                }
                if start != usize::MAX {
                    runs.push((start, w - 1));
                }
                if !runs.is_empty() && y % 2 == 0 {
                    eprintln!("PROBE shot-{label} y={y} runs={runs:?}");
                }
            }
        }
        // 标题"大闹天宫"屏对齐实测（用户 2026-10-19"还是没有对齐"）：
        // classic vs modern 内容块边界对照
        for (label, file) in [
            ("title-classic", "analysis/loading-280.png"),
            ("title-modern", "analysis/title-modern.png"),
        ] {
            let bytes = std::fs::read(repo.join(file)).unwrap();
            let dec = game_data::decode_png(&bytes).unwrap();
            let (w, h) = (dec.width as usize, dec.height as usize);
            for y in (0..h).step_by(16) {
                let mut runs: Vec<(usize, usize)> = Vec::new();
                let mut start = usize::MAX;
                for x in 0..w {
                    let p = dec.argb[y * w + x];
                    let (r, g, b) = ((p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF);
                    let bg = p == 0 || (b > 150 && r < 120 && g < 130); // 蓝底/透明=背景
                    if !bg && start == usize::MAX {
                        start = x;
                    }
                    if bg && start != usize::MAX {
                        runs.push((start, x - 1));
                        start = usize::MAX;
                    }
                }
                if start != usize::MAX {
                    runs.push((start, w - 1));
                }
                if !runs.is_empty() {
                    eprintln!("PROBE {label} y={y} runs={runs:?}");
                }
            }
        }
    }

    /// 底部区组合关系判据（lead"组合关系整体钉死"，v5 扩展：**层序也是组合关系**）：
    /// 同状态 classic vs modern 的底部带绘制**序列**（op 名+去平移坐标，保序）
    /// 逐项相等——坐标多重集之外再钉绘制顺序（谁盖谁）。
    #[test]
    fn bottom_composition_is_translate_only() {
        let icons = synthetic_sptarm();
        let coords = |layout: crate::layout::Layout, speaker: i32| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            sc.game_mode = GameMode::Dialog;
            sc.open_dialog_page(0);
            sc.dialog.dialog_speaker = speaker; // open_dialog_page 会置 speaker——须在其后覆盖
            if speaker > 0 {
                sc.images.entity[speaker as usize] = Some(solid(36, 40)); // 立绘播种（分支必画）
            }
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                ops = g.ops.clone();
            }
            let dx = sc.bottom_panel_dx();
            let dy = h as i32 - 320;
            // 保序序列：(op 名, x−dx, y−dy)——层序也入判据（v5）；
            // 背景层（drawImageT 镜像行 / 列宽 77 边缘填充 / 画布自适应列）不计
            // ——组合关系指面板构图
            ops
                .iter()
                .filter(|o| !o.starts_with("drawImageT"))
                .filter(|o| !(o.starts_with("fillRect(") && o.contains(",77,")))
                .filter_map(|o| {
                    let (x, y) = op_xy(o)?;
                    (y >= h as i32 - 45).then(|| {
                        let name = o.split('(').next().unwrap_or("").to_string();
                        (name, x - dx, y - dy)
                    })
                })
                .collect::<Vec<_>>()
        };
        // speaker>0（立绘分支）、speaker==72（牛魔王定尺剪裁臂 a.java:2852）与 speaker==0
        // （面孔分支）三路都钉（坐标+层序）。72 臂漏钉 ⇒ 字面量 y=275 漏网（BUG-029）。
        for speaker in [0, 1, 72] {
            let classic = coords(crate::layout::Layout::classic(), speaker);
            let modern = coords(crate::layout::Layout::modern(13, 13), speaker);
            assert_eq!(
                modern, classic,
                "底部区组合关系须纯平移且层序不变（speaker={speaker}）"
            );
        }
    }

    /// 观察形态判据（lead 裁决 (a) 2026-10-19 用户批准，STAGE-B-002 偏离点）：
    /// 空格观察形态（walkPhase==2 未锁定）modern **角色恒居中**（视口中心成像
    /// 出现角色=与基帧不同）；classic 保持原机制（角色留 cam+player 位、
    /// 视口中心与基帧同）——classic 零偏离。
    #[test]
    fn soul_view_character_pinned_center_modern_only() {
        let icons = synthetic_sptarm();
        let shot = |layout: crate::layout::Layout, walk: i32| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            sc.view.player_px = 32;
            sc.view.player_py = 32;
            for i in 0..4 {
                sc.afterimage_x[i] = 32;
                sc.afterimage_y[i] = 32;
            }
            sc.walk_phase = walk;
            sc.view_locked = false;
            let (cx, cy) = (
                ((sc.layout.game_view_w - 41) >> 1) as usize,
                (sc.layout.view_top_y + sc.layout.game_view_h / 2) as usize,
            );
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            let ops;
            {
                // 直驱 paint_player（cam 传参=真实调用方口径：view.cam+view_top）
                let mut g = SoftGraphics::new(&mut img);
                sc.paint_player(&mut g, sc.view.cam_x, sc.view.cam_y + sc.layout.view_top_y);
                ops = g.ops.clone();
            }
            if std::env::var("PROBE_SOUL").is_ok() {
                eprintln!("PROBE walk={walk} ops: {:?}", ops);
            }
            (img, cx, cy)
        };
        for (label, layout, expect_pin) in [
            ("classic", crate::layout::Layout::classic(), false),
            ("modern", crate::layout::Layout::modern(13, 13), true),
        ] {
            let (base, cx, cy) = shot(layout.clone(), 0);
            let (soul, _, _) = shot(layout, 2);
            let bw = base.width as usize;
            let mut diff = 0usize;
            for y in cy..cy + 30 {
                for x in cx..cx + 30 {
                    if base.argb[y * bw + x] != soul.argb[y * bw + x] {
                        diff += 1;
                    }
                }
            }
            if expect_pin {
                assert!(diff > 40, "{label}：观察态角色须钉视口中心（中心窗 diff={diff}）");
            } else {
                assert_eq!(diff, 0, "{label}：观察态原机制（角色留 cam+player 位，中心窗须与基帧同）");
            }
        }
    }

    /// 【task-14 场景级·布局纪律】菜单臂二属性分类判据：逐臂须满足其一——
    /// (A) 纯平移（内容 ops classic/modern 逐项恒等=origin 隐式平移）或
    /// (B) 相对布局保持（屏心偏移双预设同值，公式锚语义）。
    /// 双破=错位元素（违布局纪律）——37b53a4 双移回归即此判据揪出。
    /// 【分类表 2026-10-20】InGameMenu=公式锚(B)；EnemyInfo/ItemPanel 随测落定。
    #[test]
    fn menu_arms_layout_discipline() {
        let icons = synthetic_sptarm();
        /// 【分类表 2026-10-20】InGameMenu=公式锚(B) 已定；
        /// **EnemyInfo(paint_menu5)=混装**（c=94 m=-8）、**ItemPanel=混装**
        /// （c=-41 m=-2）——modern 侧近居中（-8/-2）但相对布局漂移=双属性皆破，
        /// 分类未定前不接线，列入收口余项（逐元素锚对账后定 A/B）。
        type Setup = fn(&mut GameScene);
        let arms: &[(&str, Setup)] = &[("InGameMenu", |sc| {
            sc.game_mode = GameMode::InGameMenu;
            sc.enter_menu4();
        })];
        let ops_of = |layout: crate::layout::Layout, setup: Setup| {
            let mut sc = scene_with(&icons);
            sc.apply_layout(layout);
            setup(&mut sc);
            let (w, h) = (sc.layout.screen_w as usize, sc.layout.screen_h as usize);
            let mut img = ArgbImage::create(w as i32, h as i32);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                ops = g.ops.clone();
            }
            (ops, w)
        };
        let content = |ops: Vec<String>| -> Vec<String> {
            ops.into_iter()
                .filter(|o| {
                    if o.starts_with("drawImageT") || o.contains("77x320") {
                        return false;
                    }
                    if o.starts_with("fillRect(") || o.starts_with("setClip(") {
                        let nums: Vec<i32> = o
                            .split('(')
                            .nth(1)
                            .and_then(|s| s.split(')').next())
                            .map(|a| {
                                a.split(',')
                                    .filter_map(|t| t.trim().trim_start_matches('#').parse().ok())
                                    .collect()
                            })
                            .unwrap_or_default();
                        if nums.len() >= 4 && (nums[2] >= 240 || nums[3] >= 320) {
                            return false;
                        }
                    }
                    true
                })
                .collect()
        };
        let center_of = |ops: &Vec<String>, w: i32| -> i32 {
            let (mut lo, mut hi) = (i32::MAX, i32::MIN);
            for o in ops {
                if let Some((x, _)) = op_xy(o) {
                    lo = lo.min(x);
                    hi = hi.max(x);
                }
            }
            (lo + hi) / 2 - w / 2
        };
        for (name, setup) in arms {
            let (c_ops, c_w) = ops_of(crate::layout::Layout::classic(), *setup);
            let (m_ops, m_w) = ops_of(crate::layout::Layout::modern(13, 13), *setup);
            let (c, m) = (content(c_ops), content(m_ops));
            let pure_translate = m == c;
            let (co, mo) = (center_of(&c, c_w as i32), center_of(&m, m_w as i32));
            assert!(
                pure_translate || co == mo,
                "{name} 须过布局纪律：纯平移（ops 恒等）或相对布局保持（屏心偏移同值 c={co} m={mo}）"
            );
        }
    }

    /// 【lead 数据vs渲染二面 2026-10-20·③】tile 层**逐层全扫**（55 maplv）：
    /// 画布内的数据格（v>0）必有对应绘制（setClip(x,y,16,16)）——数据有格未画
    /// =渲染面缺格（同族漏网）；数据真无格=该层设计如此（判据自动静默=合法）。
    /// 破缺层报 (层号, 缺格数) ⇒ ①数据面/②渲染面二选一定位。
    #[test]
    fn tiles_draw_all_data_cells_in_canvas_all_layers() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut bad: Vec<(u32, usize)> = Vec::new();
        for n in 0..55u32 {
            let Ok(bytes) = std::fs::read(repo.join(format!("assets/raw/maplv{n}"))) else {
                continue;
            };
            let Ok(mut v) = crate::game_view::GameView::from_maplv(&bytes, 0, 0, crate::game_view::ViewDims::classic()) else {
                continue;
            };
            v.set_layout_dims(416, 348, 416, 416);
            let mut img = ArgbImage::create(416, 416);
            let ops;
            {
                let mut g = SoftGraphics::new(&mut img);
                v.paint_tiles(&mut g, &solid(64, 64), 20);
                ops = g.ops.clone();
            }
            let drawn: std::collections::HashSet<(i32, i32)> = ops
                .iter()
                .filter(|o| o.starts_with("setClip(") && o.ends_with(",16,16)"))
                .filter_map(|o| op_xy(o))
                .collect();
            let mut miss = 0usize;
            for r in 0..(v.high as i32) * 2 {
                for c in 0..(v.wide as i32) * 2 {
                    if v.terrain_at(r, c) > 0 {
                        let (x, y) = (v.cam_x + c * 16, 20 + v.cam_y + r * 16);
                        if (0..416).contains(&x) && (20..366).contains(&y) && !drawn.contains(&(x, y)) {
                            miss += 1;
                        }
                    }
                }
            }
            if miss > 0 {
                bad.push((n, miss));
            }
            if std::env::var("PROBE_DATA_RIGHT").is_ok() {
                let (mut right, mut map_w) = (0i32, (v.wide as i32) * 32);
                for r in 0..(v.high as i32) * 2 {
                    for c in 0..(v.wide as i32) * 2 {
                        if v.terrain_at(r, c) > 0 {
                            right = right.max(c * 16 + 16);
                        }
                    }
                }
                eprintln!("PROBE maplv{n} 数据右缘={right}px 地图宽={map_w}px（右缘<地图宽=①数据边界）");
            }
        }
        assert!(
            bad.is_empty(),
            "②渲染面缺格层（数据有格未画=同族漏网）: {bad:?}；空表=①数据边界合法"
        );
    }

    /// 跳过 "77x320" 尺寸记号与 '#rrggbb' 色参（drawImage/drawImageTransformed 可用）。
    fn op_xy(o: &str) -> Option<(i32, i32)> {
        let args = o.split('(').nth(1)?.split(')').next()?;
        let mut nums = args
            .split(',')
            .filter_map(|t| t.trim().trim_start_matches('#').parse::<i32>().ok());
        Some((nums.next()?, nums.next()?))
    }

    /// 【2026-10-26 壳层呈现开关】mode 3 游戏画面：`softkey_bar_visible=false` 时绘制序列 =
    /// 默认序列**恰去掉一段 paintSoftkeyBar**（HUD 左右下角的软键底图 + 图标），其余 op
    /// （瓦片/实体/HUD/状态栏…）逐条不变；软键种类状态照常（壳据此画外置软键）。
    #[test]
    fn softkey_bar_switch_removes_exactly_the_softkey_block() {
        let icons = synthetic_sptarm();
        let layout = crate::layout::Layout::modern(12, 16);
        let render = |visible: bool| {
            let mut sc = scene_with(&icons);
            let mut l = layout.clone();
            l.softkey_bar_visible = visible;
            sc.apply_layout(l);
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let ops = {
                let mut g = SoftGraphics::new(&mut img);
                sc.paint(&mut g, &solid(64, 64));
                g.ops.clone()
            };
            (ops, sc.menu.softkeys)
        };
        let (shown, keys) = render(true);
        let (hidden, keys_hidden) = render(false);
        assert_eq!(keys, (1, 3), "mode 3 软键种类");
        assert_eq!(keys_hidden, keys, "开关不动软键状态");
        // 期望删去的块 = 同尺寸画布上单独调 paintSoftkeyBar 的 op 串
        let block = {
            let sc = scene_with(&icons);
            let mut img = ArgbImage::create(layout.screen_w, layout.screen_h);
            let mut g = SoftGraphics::new(&mut img);
            crate::paint::m_034_softkeys(&mut g, &sc.images.ui[10], &sc.images.ui[11], 1, 3, &layout);
            g.ops.clone()
        };
        assert!(!block.is_empty());
        let at: Vec<usize> = (0..=shown.len().saturating_sub(block.len()))
            .filter(|&i| shown[i..i + block.len()] == block[..])
            .collect();
        assert_eq!(at.len(), 1, "默认帧含且仅含一段软键栏");
        let mut expect = shown.clone();
        expect.drain(at[0]..at[0] + block.len());
        assert_eq!(hidden, expect, "关闭 = 恰去掉软键栏，其余 op 逐条不变");
    }

    /// 第二轮返修判据（用户复验三破绽 2026-10-18）：
    /// ②背景铺满：视差列/行须盖满画布（现状=只盖左上固定区，右侧/下方大片黑）；
    /// ③HUD 内容组水平居中偏移 ≤ 12px；
    /// ④瓦片"右缘止"判定：绘制数 == 窗口内 v>0 格数（=数据边界则合法、
    ///   相机窗口缺格则断言破——区分二选一）。
    #[test]
    fn modern_render_background_fill_and_hud_center() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.apply_layout(crate::layout::Layout::modern(13, 13));
        let dx = sc.bottom_panel_dx();
        let mut img = ArgbImage::create(416, 416);
        let ops;
        {
            let mut g = SoftGraphics::new(&mut img);
            sc.paint(&mut g, &solid(64, 64));
            ops = g.ops.clone();
        }
        if std::env::var("PROBE_FILLS").is_ok() {
            for o in &ops {
                if o.starts_with("fillRect") || o.contains("Transformed") {
                    eprintln!("PROBE {o}");
                }
            }
        }
        // ② 背景覆盖：视差绘制（backdrop 77×320）x 跨度盖满 [0,416)、上下两行堆叠
        let bg: Vec<(i32, i32)> = ops
            .iter()
            .filter(|o| o.contains("77x320") || o.contains("drawImageTransformed"))
            .filter_map(|o| op_xy(o))
            .collect();
        // 【v5 判据扩展①】全画布未绘制像素=0（含底部带——面板两侧露背景非黑边）
        let mut undrawn_all = 0usize;
        for y in 0..416 {
            for x in 0..416 {
                if img.argb[y * 416 + x] == 0 {
                    undrawn_all += 1;
                }
            }
        }
        assert_eq!(undrawn_all, 0, "modern 全画布未绘制像素（底部带黑边=视差未铺满高）");
        let bg_max_x = bg.iter().map(|c| c.0).max().unwrap_or(-999);
        let bg_min_x = bg.iter().map(|c| c.0).min().unwrap_or(999);
        assert!(
            bg_max_x + 77 >= 416 && bg_min_x <= 0,
            "背景须铺满画布宽（现列起点 [{bg_min_x},{bg_max_x}]，末列+77 须 ≥416）"
        );
        assert!(bg.len() >= 5, "背景列须覆盖画布宽（每列一条基行；现 {} 条）", bg.len());
        // ③ HUD 内容组居中（底带**内容**x 跨度中心 vs 屏心，偏移 ≤12；
        //    框体=全宽设计（fill 宽≥200 / 边条 x<20 或 ≥400）不计入内容组）
        let hud: Vec<(i32, i32)> = ops
            .iter()
            .filter(|o| !o.starts_with("drawImageT")) // 背景层（画布自适应）不计
            .filter(|o| !(o.starts_with("fillRect(") && o.contains(",77,"))) // 背景边缘填充不计
            .filter(|o| op_xy(o).map(|c| c.1 > 368 && c.1 < 412).unwrap_or(false))
            .filter(|o| {
                let w = o
                    .split('(')
                    .nth(1)
                    .and_then(|s| s.split(')').next())
                    .map(|a| {
                        a.split(',')
                            .filter_map(|t| t.trim().trim_start_matches('#').parse::<i32>().ok())
                            .nth(2)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                w < 200
            })
            .filter_map(|o| op_xy(o))
            // 内容组域：排除框体四柱列（dx-42/dx/dx+176/dx+229 = 框角/框缘/右缘/右条
            // ——框体=面板构件非内容）与域外 x
            .filter(|c| {
                (26..330).contains(&c.0) && ![dx - 42, dx, dx + 176, dx + 229].contains(&c.0)
            })
            .collect();
        let (hmin, hmax) = (
            hud.iter().map(|c| c.0).min().unwrap_or(0),
            hud.iter().map(|c| c.0).max().unwrap_or(0),
        );
        let center = (hmin + hmax) / 2;
        assert!(
            (center - 208).abs() <= 12,
            "HUD 内容组水平居中（现 center={center} vs 屏心 208，跨度 [{hmin},{hmax}]，明细 {hud:?}）"
        );
        // ④ 瓦片右缘判定（二选一定位）：绘制数 ≤ 数据 v>0 格数 且
        //    tile_max_x == 数据右缘（=数据边界则两者相等=合法；窗口缺格则绘制右缘<数据右缘=bug）
        let (v0_total, data_right) = {
            let v = &sc.view;
            let (mut n, mut right) = (0usize, 0i32);
            for r in 0..(v.high as usize) * 2 {
                for c in 0..(v.wide as usize) * 2 {
                    if v.terrain_at(r as i32, c as i32) > 0 {
                        n += 1;
                        right = right.max((c as i32) * 16);
                    }
                }
            }
            (n, right)
        };
        let tiles_x: Vec<i32> = ops
            .iter()
            .filter(|o| o.starts_with("setClip(") && o.ends_with(",16,16)"))
            .filter_map(|o| op_xy(o))
            .map(|c| c.0)
            .collect();
        let tile_max_x = tiles_x.iter().copied().max().unwrap_or(0);
        assert!(
            tiles_x.len() <= v0_total,
            "绘制瓦片数 {} 不得超数据 v>0 格数 {v0_total}（右缘止=数据边界即合法）",
            tiles_x.len()
        );
        assert_eq!(
            tile_max_x, data_right,
            "瓦片右缘判定：tile_max_x={tile_max_x} vs 数据右缘={data_right}（相等=数据边界合法；小于=相机窗口缺格 bug）"
        );
    }

    fn loaded_with(icons: &[ArgbImage], save: &crate::save::FloorSave) -> GameScene {
        let (width_table, walk) = fixtures();
        load_floor_from_save(
            include_bytes!("../../tests/fixtures/maplv1.bin"),
            &[],
            &width_table,
            &walk,
            synth_images(icons),
            vec![None; 56],
            save,
        )
        .unwrap()
    }

    /// 换层水位（a.java:7163-7164）：50 抬升+脏；51 结局层不计；单调不回退。
    #[test]
    fn scene_watermark_bump_marks_dirty() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.note_max_floor(50);
        assert!(sc.take_global_dirty(), "水位抬升 ⇒ dirty");
        assert_eq!(sc.global_snapshot().max_floor_watermark, 50);
        sc.note_max_floor(51);
        assert!(!sc.take_global_dirty(), "51 不计入");
        assert_eq!(sc.global_snapshot().max_floor_watermark, 50);
        sc.note_max_floor(30);
        assert_eq!(sc.global_snapshot().max_floor_watermark, 50, "单调不回退");
    }

    /// probeCrossFloorSnapshot 同层路径（a.java:8640-8655）：存活目标型 ⇒ true；removed 跳过。
    #[test]
    fn probe_finds_live_entities_same_floor() {
        let mut sc = scene_with(&synthetic_sptarm());
        let f = sc.persist.floor;
        entity::spawn(&mut sc.entities, &mut sc.grid, 12, 3 << 5, 3 << 5, 0, &sc.width_table, None);
        assert!(sc.probe_cross_floor_snapshot(f, 12), "存活型12 ⇒ true");
        assert!(!sc.probe_cross_floor_snapshot(f, 99), "无此型 ⇒ false");
        let e = sc.entities.count - 1;
        sc.entities.items[e].removed = true;
        assert!(!sc.probe_cross_floor_snapshot(f, 12), "removed 不计");
    }

    /// tryRunScene 共臂门控（a.java:8193-8201）：flags[7] 挡；否则需 floor23 有存活型12。
    #[test]
    fn try_run_scene_shared_probe_gate() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.persist.floor = 23; // 同层化：probe(23,12) 免跨层资产
        assert!(!sc.try_run_scene(32, false), "无型12 ⇒ 挡");
        entity::spawn(&mut sc.entities, &mut sc.grid, 12, 3 << 5, 3 << 5, 0, &sc.width_table, None);
        assert!(sc.try_run_scene(32, false), "有型12 ⇒ 过");
        assert!(!sc.try_run_scene(32, true), "from_walk ⇒ 挡（!script_cursor0）");
        sc.script_line_flags[7] = true;
        assert!(!sc.try_run_scene(32, false), "flags[7] ⇒ 挡");
    }

    /// tryRunScene case 7（a.java:8123-8127）：from_walk ∧ 无型12 ⇒ 过 + 置 flags[32]。
    #[test]
    fn try_run_scene_case7_sets_flag32() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.persist.floor = 23;
        assert!(!sc.try_run_scene(7, false), "非 from_walk ⇒ 挡");
        assert!(sc.try_run_scene(7, true), "from_walk ∧ 无型12 ⇒ 过");
        assert!(sc.script_line_flags[32], "过门置 flags[32]");
    }

    /// changeFloor 四提示浮层（a.java:7135/7142/7153/7156）：strict 下/上界 + 负层/超 55。
    #[test]
    fn change_floor_prompt_texts() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.persist.min_floor = 2;
        sc.persist.max_floor = 10;
        assert!(!sc.change_floor(1, false, true), "strict 低于 min ⇒ false");
        assert_eq!(sc.dialog.overlay_text.as_deref(), Some(MISC_TEXTS[0]));
        assert!(!sc.change_floor(11, false, true), "strict 高于 max ⇒ false");
        assert_eq!(sc.dialog.overlay_text.as_deref(), Some(MISC_TEXTS[1]));
        assert!(!sc.change_floor(-1, false, false), "负层 ⇒ false");
        assert_eq!(sc.dialog.overlay_text.as_deref(), Some(MISC_TEXTS[2]));
        assert!(!sc.change_floor(56, false, false), "超 55 ⇒ false");
        assert_eq!(sc.dialog.overlay_text.as_deref(), Some(MISC_TEXTS[3]));
    }

    /// LAY_x（a.java:8355-8362）：脚本层跳转——m_119 存态 + 目标层 + 遮幅（dialogPhase=5）。
    #[test]
    fn script_lay_jumps_floor_with_wipe() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.dialog.overlay_text = Some("x".into());
        sc.script_line = "LAY_7 ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.persist.pending_floor, 7);
        assert!(sc.transitioning);
        assert_eq!(sc.dialog.dialog_phase, 5);
        assert!(sc.dialog.overlay_text.is_none());
    }

    /// GIN_a_b（a.java:8424-8437）：1=得金 b；尾递归执行下一条（END→行尾收尾）。
    #[test]
    fn script_gin_gives_gold_and_recurses() {
        let mut sc = scene_with(&synthetic_sptarm());
        let g0 = sc.player.gold;
        sc.script_line = "GIN_1_1000 ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.player.gold, g0 + 1000, "gainGold(1000)");
        // 尾递归自证：GIN 后递归执行到行尾收尾分支（dialogPhase=4 + 置 flag）
        assert_eq!(sc.dialog.dialog_phase, 4, "尾递归走到行尾收尾（a.java:8435-8436）");
        assert!(sc.script_line_flags[sc.current_script_index]);
    }

    /// RES_x（a.java:8470-8490）：easy（difficulty==0）重置 run 统计；非 easy 空转。
    #[test]
    fn script_res_resets_run_stats_on_easy() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.menu.difficulty_index = 0;
        sc.player.hp = 999;
        sc.player.gold = 777;
        sc.player.yellow_keys = 5;
        sc.player.atk = 50;
        sc.persist.min_floor = 9;
        sc.persist.max_floor = 40;
        sc.persist.journal.push((3, 1));
        sc.script_line = "RES_0 ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!((sc.player.hp, sc.player.atk, sc.player.def), (400, 10, 10));
        assert_eq!(sc.player.gold, 4);
        assert_eq!((sc.persist.min_floor, sc.persist.max_floor), (1, 3));
        assert_eq!((sc.player.yellow_keys, sc.player.red_keys, sc.player.blue_keys), (0, 0, 0));
        assert_eq!(sc.alchemy_upgrade_count, 0);
        assert!(sc.persist.journal.is_empty(), "inventoryEntryCount 清零（inventoryEntryCount+跟随对）");
        // 非 easy 不动
        let mut sc2 = scene_with(&synthetic_sptarm());
        sc2.menu.difficulty_index = 1;
        sc2.player.hp = 999;
        sc2.script_line = "RES_0 ".to_string();
        sc2.script_cursor = 0;
        sc2.execute_script_instruction();
        assert_eq!(sc2.player.hp, 999, "difficulty!=0 空转");
    }

    /// ADD_a_b_c_d（a.java:8438-8468）：同层生成路径（a==currentFloor 时免存/载切换）。
    #[test]
    fn script_add_spawns_entity_same_floor() {
        let mut sc = scene_with(&synthetic_sptarm());
        let f = sc.persist.floor;
        let n0 = sc.entities.count;
        sc.script_line = format!("ADD_{}_72_11_10 ", f);
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.entities.count, n0 + 1);
        assert_eq!(sc.entities.items[n0].type_, 72);
        assert_eq!((sc.entities.items[n0].px, sc.entities.items[n0].py), (11 << 5, 10 << 5));
    }
    /// ①②装备往返：serialize 写真实值（a.java:9192-9193）、load 恢复（a.java:9260-9261）。
    #[test]
    fn save_roundtrip_restores_equipment() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.equipped_weapon = 79;
        sc.equipped_armor = 80;
        let blob = sc.serialize_slot_blob();
        assert_eq!(blob[5], 79, "blob[5]=equippedWeaponType（a.java:9192）");
        assert_eq!(blob[6], 80, "blob[6]=equippedArmorType（a.java:9193）");
        let save = crate::save::parse_floor_save(&blob, 128).expect("blob 可解析");
        assert_eq!((save.equipped_weapon_type, save.equipped_armor_type), (79, 80));
        let sc2 = loaded_with(&synthetic_sptarm(), &save);
        assert_eq!((sc2.equipped_weapon, sc2.equipped_armor), (79, 80), "load 恢复装备（a.java:9260-9261）");
    }

    /// 写侧 blob 布局 = Java saveFloorState（a.java:9190-9236）：playerCellX/Y 为 short
    /// （a.java:9197-9198）、alchemyUpgradeCount 实值（9206）、inventoryEntryCount（原 inventoryEntryCount）+
    /// f_byte_arr_14/15 日志对（9207-9212，读侧 9278-9283 → journal）——parse → load 全字段往返。
    #[test]
    fn save_blob_layout_matches_java_and_roundtrips() {
        let mut sc = scene_with(&synthetic_sptarm()); // floor1 (3,10)
        sc.player.hp = 777;
        sc.player.atk = 41;
        sc.player.def = 42;
        sc.player.yellow_keys = 3;
        sc.player.blue_keys = 2;
        sc.player.red_keys = 1;
        sc.player.gold = 1234;
        sc.alchemy_upgrade_count = 5;
        sc.persist.journal = vec![(0, 3), (1, 7)];
        sc.script_line_flags[0] = true;
        let blob = sc.serialize_slot_blob();
        assert_eq!(&blob[7..11], &[0, 3, 0, 10], "writeShort(playerCellX/Y)");
        assert_eq!(&blob[11..15], &777i32.to_be_bytes(), "hp 紧随两 short");
        assert_eq!(&blob[29..33], &1234i32.to_be_bytes(), "gold");
        assert_eq!(&blob[33..35], &5i16.to_be_bytes(), "writeShort(alchemyUpgradeCount)");
        assert_eq!(&blob[35..41], &[0, 2, 0, 3, 1, 7], "inventoryEntryCount=2 + 日志对");
        assert_eq!(blob[41], 1, "旗标段紧随日志对（flags[0]）");
        assert_eq!(blob.len(), 37 + 2 * 2 + 128 + 1 + 56 * 2, "道具栈 0 + 56 层全 null(-1)");
        let save = crate::save::parse_floor_save(&blob, 0).expect("blob 可解析");
        assert_eq!((save.player_cell_x, save.player_cell_y), (3, 10));
        assert_eq!((save.player_hp, save.player_atk, save.player_def), (777, 41, 42));
        assert_eq!((save.yellow_keys, save.blue_keys, save.red_keys), (3, 2, 1));
        assert_eq!((save.gold, save.alchemy_upgrades), (1234, 5));
        assert_eq!(save.journal, vec![(0, 3), (1, 7)]);
        assert!(save.floor_states.iter().all(|s| s.is_none()));
        let sc2 = loaded_with(&synthetic_sptarm(), &save);
        assert_eq!((sc2.player_cell_x, sc2.player_cell_y), (3, 10));
        assert_eq!((sc2.player.hp, sc2.player.atk, sc2.player.def), (777, 41, 42));
        assert_eq!((sc2.player.gold, sc2.alchemy_upgrade_count), (1234, 5));
        assert_eq!(sc2.persist.journal, vec![(0, 3), (1, 7)]);
        assert!(sc2.script_line_flags[0]);
    }

    /// MOT_IF 6 槽表（loadSlotTable 读序 a.java:8974-8988）：`(槽, 武器, 甲)`。
    fn mot_if(slots: &[(usize, i8, i8)]) -> Vec<u8> {
        let mut o = Vec::new();
        for k in 0..6 {
            match slots.iter().find(|s| s.0 == k) {
                Some(&(_, w, a)) => {
                    o.extend_from_slice(&[1, 1, w as u8, a as u8]);
                    for v in [500i32, 30, 30, 1, 0, 0, 100] {
                        o.extend_from_slice(&v.to_be_bytes());
                    }
                }
                None => o.push(0),
            }
        }
        o
    }

    fn paint7(sc: &mut GameScene) -> ArgbImage {
        let mut screen = ArgbImage::create(SCREEN_W, SCREEN_H);
        {
            let mut g = SoftGraphics::new(&mut screen);
            g.set_font(Some(crate::paint::paint_font()));
            sc.paint_slot_list(&mut g);
        }
        screen
    }

    fn enter7(sc: &mut GameScene) {
        sc.menu.menu_return_mode = GameMode::InGameMenu;
        sc.game_mode = GameMode::SaveSlot;
        sc.enter_menu7();
    }

    fn key7(sc: &mut GameScene, k: i32) {
        sc.key_value = k;
        sc.tick_menu7();
        sc.key_value = 0;
    }

    /// 读入槽表的装备 ≠0 → 图标（经 entityTypeImage 反查同一位图）；=0 → “无”。
    #[test]
    fn scene_slot_list_draws_icon_or_material_name() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.persist.preset_mot_if = Some(mot_if(&[(0, 34, 80), (1, 0, 0), (2, 79, 41)]));
        enter7(&mut sc);
        let s = paint7(&mut sc);
        assert_frame_icon(&s, WEAPON_X, FRAME_Y, &icons, 1, "槽0 武器=34");
        assert_frame_icon(&s, ARMOR_X, FRAME_Y, &icons, 9, "槽0 甲=80");
        key7(&mut sc, -2);
        let s = paint7(&mut sc);
        assert_frame_material_name(&s, WEAPON_X, FRAME_Y, "槽1 武器=0");
        assert_frame_material_name(&s, ARMOR_X, FRAME_Y, "槽1 甲=0");
        key7(&mut sc, -2);
        let s = paint7(&mut sc);
        assert_frame_icon(&s, WEAPON_X, FRAME_Y, &icons, 8, "槽2 武器=79（>7 钳 8）");
        assert_frame_icon(&s, ARMOR_X, FRAME_Y, &icons, 9, "槽2 甲=41（>7 钳 9）");
    }

    /// markSlotEntry（a.java:9013-9014）：存档写回当前 equippedWeaponType/ArmorType，
    /// 详情面板随即按新值画（“保存成功！”浮层下的槽表帧）。
    #[test]
    fn scene_save_copies_equipment_into_slot_table() {
        let icons = synthetic_sptarm();
        let mut sc = scene_with(&icons);
        sc.persist.preset_mot_if = Some(mot_if(&[(0, 0, 0)]));
        sc.equipped_weapon = 33;
        sc.equipped_armor = 0;
        enter7(&mut sc);
        for _ in 0..4 {
            key7(&mut sc, -2);
        }
        assert_eq!(sc.menu.slot.cursor, 4);
        assert!(!sc.menu.slot.valid[4]);
        key7(&mut sc, -5);
        assert!(sc.menu.slot.valid[4]);
        assert_eq!((sc.menu.slot.weapon[4], sc.menu.slot.armor[4]), (33, 0));
        let s = paint7(&mut sc);
        assert_frame_icon(&s, WEAPON_X, FRAME_Y, &icons, 0, "存档后 武器=33");
        assert_frame_material_name(&s, ARMOR_X, FRAME_Y, "存档后 甲=0");
        // 再存一次（同槽覆写）：拾甲 80 后的装备态
        sc.dialog.overlay_active = false;
        sc.equipped_weapon = 0;
        sc.equipped_armor = 80;
        key7(&mut sc, -5);
        assert_eq!((sc.menu.slot.weapon[4], sc.menu.slot.armor[4]), (0, 80));
        let s = paint7(&mut sc);
        assert_frame_material_name(&s, WEAPON_X, FRAME_Y, "覆写后 武器=0");
        assert_frame_icon(&s, ARMOR_X, FRAME_Y, &icons, 9, "覆写后 甲=80");
    }

    /// 型 6 隐形实体 bump 显形（a.java:5410-5417）：`cur_y++` **恒执行**于显形判定前；
    /// 显形三元组写序钉死 visible→entityState(solid)=2→entityAnimRow(anim_idx)=9
    /// （a.java:5413/5414/5415 逐字，本测试以行号注释锚定写序契约）；已显形再 bump
    /// 仅 cur_y++、三字段**不重写**（a.java:5412 条件）；`continue` 续扫同格后继槽
    /// （同格双型 6 一次 bump 全显形）；cur_y>0 豁免地形终审（a.java:5533-5534）。
    #[test]
    fn type6_bump_reveal_triple_write_order() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 6, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let e1 = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 6, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let e2 = sc.entities.count - 1;
        // m_122 载入语义：型 6 生成后覆盖 visible=false（a.java:9514-9517）
        sc.entities.items[e1].visible = false;
        sc.entities.items[e2].visible = false;
        // 不可走地形：bump 放行只能靠 cur_y++ 豁免（a.java:5533）
        sc.walkable[2][2] = false;

        assert!(
            sc.interact_with_cell(2, 2),
            "型 6 bump 放行（cur_y++ 豁免地形终审，a.java:5533）"
        );
        for e in [e1, e2] {
            assert!(sc.entities.items[e].visible, "显形 1/3 visible=true（a.java:5413）");
            assert_eq!(sc.entities.items[e].state, 2, "显形 2/3 entityState=2（a.java:5414）");
            assert_eq!(sc.entities.items[e].anim_row, 9, "显形 3/3 entityAnimRow=9（a.java:5415）");
        }

        // 二次 bump：已显形仅 cur_y++，三元组不再重写（a.java:5412 `if (!entityVisible)`）
        sc.entities.items[e1].anim_row = 42;
        assert!(sc.interact_with_cell(2, 2), "已显形再 bump 仍放行（cur_y++ 恒执行）");
        assert_eq!(sc.entities.items[e1].anim_row, 42, "一次性显形：anim 不重写");
        assert!(sc.entities.items[e1].visible, "一次性显形：visible 不重写");
        assert_eq!(sc.entities.items[e1].state, 2, "一次性显形：solid 不重写");
    }

    /// MVS 型 82 配对传送（a.java:8519-8592）三断言之【配对+互传+消耗】：
    /// 倒序扫 type==82 按 `param&255` 两两配对（本例 3 与 0x0103=259 低字节
    /// 同键 ⇒ 配对成立，钉 &255 掩码）；cat8 怪物经 carry_monsters_to（a.java:8872-8878）
    /// 搬到对端格（同型同 param spawn + 旧格全同型摘槽）；两 82 点位被
    /// removeGroupAtCellByType(…, 82)（a.java:6434-6449）消耗（removed=true）；
    /// 无参指令游标 = 标签后 1 字符位（a.java:8591 `scriptCursor = script_cursor0 + 1`）。
    #[test]
    fn mvs_pairs_by_param_low_byte_transfers_and_consumes() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 0x0103, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let ema = sc.entities.count - 1;
        // m_122 载入语义：型 82 生成后覆盖 visible=false（a.java:9601-9605）
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        assert!(sc.entities.items[e82a].removed && sc.entities.items[e82b].removed, "消耗：两 82 点位摘槽");
        assert!(sc.entities.items[ema].removed, "原地怪物被 carry_monsters_to 旧格全同型摘除");
        assert_eq!(sc.entities.count, ema + 2, "carry_monsters_to 复制-删除：目标格新增 1 只");
        let moved = ema + 1;
        assert_eq!(sc.entities.items[moved].type_, 41, "互传：同型复制");
        assert!(!sc.entities.items[moved].removed, "互传：副本存活");
        assert_eq!(
            (sc.entities.items[moved].px, sc.entities.items[moved].py),
            (5 << 5, 5 << 5),
            "互传：落到对端格（82B 格）"
        );
        assert_eq!(sc.script_cursor, 4, "无参 op 游标=标签后 1 位（a.java:8591）");
    }

    /// MVS 配对负例（a.java:8532-8542）：`param&255` 不同键 ⇒ 半对（buf 只存
    /// 首元素）⇒ 配对循环 `cur_idx > 0` 不成立 ⇒ 全程无事发生——82 不消耗、
    /// 怪物不搬、实体数不变。
    #[test]
    fn mvs_mismatched_pair_key_leaves_world_untouched() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 4, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let ema = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;
        let count_before = sc.entities.count;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        assert!(!sc.entities.items[e82a].removed && !sc.entities.items[e82b].removed, "错配 ⇒ 82 不消耗");
        assert!(!sc.entities.items[ema].removed, "错配 ⇒ 怪物不搬");
        assert_eq!(sc.entities.count, count_before, "错配 ⇒ 无 spawn");
    }

    /// MVS 双向互传（a.java:8561-8590 两向 + 8571-8573 交换）：B 格怪物→A 格、
    /// A 格怪物→B 格；方向 2 会重搬方向 1 落进扫描格的**到达怪物**（含槽压缩后
    /// stale 尾重触发 quirk，a.java:6440-6443 就地压缩不清尾）⇒ 合成双怪场景
    /// 终态 = 双怪堆聚于 buf[0]（逆序首扫 82）实体格。Stage A 保真钉住。
    #[test]
    fn mvs_two_way_transfer_piles_monsters_at_first_slot_cell() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let ema = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 42, 5 << 5, 5 << 5, 0, &sc.width_table, None);
        let emb = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        // 原始 4 只全部摘除（41/42 旧格全同型删除 quirk 一并带走过路副本）
        for e in [e82a, e82b, ema, emb] {
            assert!(sc.entities.items[e].removed, "原地实体 {e} 摘除");
        }
        // 存活终态：双怪副本堆聚 (5,5)（buf[0]=82B 逆序首扫实体格）
        let mut alive: Vec<i32> = (0..sc.entities.count)
            .filter(|&i| !sc.entities.items[i].removed && sc.entities.items[i].type_ >= 41)
            .map(|i| sc.entities.items[i].type_)
            .collect();
        alive.sort_unstable();
        assert_eq!(alive, vec![41, 42], "双怪副本均存活且各型一只");
        for i in 0..sc.entities.count {
            if !sc.entities.items[i].removed && sc.entities.items[i].type_ >= 41 {
                assert_eq!(
                    (sc.entities.items[i].px, sc.entities.items[i].py),
                    (5 << 5, 5 << 5),
                    "副本堆聚 (5,5)"
                );
            }
        }
    }

    /// 真未知 op（16 标签外）忠实行为 = 静默停摆（a.java:8594-8595 原地
    /// return：无臂写 scriptCursor ⇒ 下一拍重解析同一 token）；不推游标、
    /// 不改 dialogPhase。禁止 3+N 跳字节续行（Java 无此行为）。
    /// 语义见 docs/spec/evidence-todos.md T-1 R-TODO-1b。
    #[test]
    fn unknown_op_silently_stalls_cursor() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.script_line = "XYZ ".to_string();
        sc.script_cursor = 0;
        sc.dialog.dialog_phase = 3;

        sc.execute_script_instruction();
        assert_eq!(sc.script_cursor, 0, "未知 op 不推游标（原地 return）");
        assert_eq!(sc.dialog.dialog_phase, 3, "未知 op 不改 dialogPhase");

        sc.execute_script_instruction(); // 下一拍重解析同一 token
        assert_eq!(sc.script_cursor, 0, "下一拍原地重解析（停摆持续）");
        assert_eq!(sc.dialog.dialog_phase, 3, "停摆期间 dialogPhase 不变");
    }

    /// 型 67 多段拼装元数据（a.java:682-697 f_byte_arr2_00/01 逐值）：9 片件
    /// (sheet, srcX, srcY, w, h) + 3 帧 × 5 段 (pieceId, dx, dy, 第4字节)。
    /// 第 4 字节三行恒 0 且绘制循环不读（保真占位，语义未知，C 级不入结论）。
    #[test]
    fn seg67_tables_match_java_metadata() {
        let seg = crate::resource_cfg::global().seg67();
        assert_eq!(
            seg.pieces.as_slice(),
            [
                (0, 0, 0, 33, 56),
                (0, 33, 0, 33, 56),
                (0, 66, 0, 33, 56),
                (2, 0, 0, 18, 26),
                (3, 0, 0, 18, 34),
                (1, 0, 0, 55, 61),
                (4, 0, 0, 14, 26),
                (4, 14, 0, 14, 26),
                (4, 28, 0, 24, 26),
            ],
            "片件表（a.java:682-692）"
        );
        assert_eq!(
            seg.frames,
            [
                [(6, -1, -79, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -53, 0), (0, -16, -27, 0)],
                [(7, -1, -80, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -54, 0), (2, -16, -26, 0)],
                [(8, -1, -81, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -55, 0), (1, -16, -25, 0)],
            ]
            .map(|row: [(i32, i32, i32, i32); 5]| row.to_vec())
            .to_vec(),
            "帧拼装表（a.java:693-697）"
        );
    }

    /// 型 67 多段拼装渲染（a.java:6580-6603）：影子 (x+8, y+22)（6581）→
    /// 基点 (x+16, y+16)（6582-6586）→ SEG67_FRAMES[裸帧偏移] 5 段，每段
    /// 片件 SEG67_PIECES[seg[0]] 画 xtq[seg.sheet] @ (基点+dx, 基点+dy)，
    /// 源 (srcX, srcY) 尺寸 w×h（6591-6601）。**裸帧偏移不乘精灵宽**
    /// （a.java:6578，对照通用臂 6528 乘宽）。逐像素对拍独立转写的参考帧；
    /// 帧行 3 偏移序列 {0,1,2,1} 来自 animOffsetTable[3]（a.java:485-497）。
    #[test]
    fn paint_type67_assembles_five_segments_like_java() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 67, 64, 96, 0, &sc.width_table, None);
        let e = sc.entities.count - 1;
        sc.entities.items[e].anim_row = 3; // animRowByType[67]=3（a.java:498 数组）
        const PIECES: [(i32, i32, i32, i32, i32); 9] = [
            (0, 0, 0, 33, 56),
            (0, 33, 0, 33, 56),
            (0, 66, 0, 33, 56),
            (2, 0, 0, 18, 26),
            (3, 0, 0, 18, 34),
            (1, 0, 0, 55, 61),
            (4, 0, 0, 14, 26),
            (4, 14, 0, 14, 26),
            (4, 28, 0, 24, 26),
        ];
        const FRAMES: [[(i32, i32, i32, i32); 5]; 3] = [
            [(6, -1, -79, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -53, 0), (0, -16, -27, 0)],
            [(7, -1, -80, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -54, 0), (2, -16, -26, 0)],
            [(8, -1, -81, 0), (3, -26, -5, 0), (4, 11, 6, 0), (5, -25, -55, 0), (1, -16, -25, 0)],
        ];
        const ROW3: [usize; 4] = [0, 1, 2, 1]; // animOffsetTable[3]（a.java:485-497）

        for f in 0..4i32 {
            sc.entities.items[e].anim_frame = f;
            let mut actual = ArgbImage::create(SCREEN_W, SCREEN_H);
            {
                let mut g = SoftGraphics::new(&mut actual);
                sc.paint_entities(&mut g, 0, 0);
            }
            let mut expected = ArgbImage::create(SCREEN_W, SCREEN_H);
            {
                let mut g2 = SoftGraphics::new(&mut expected);
                g2.draw_image(&sc.images.shadow, 64 + 8, 96 + 22, 0); // 影子（6581）
                for &(piece, dx, dy, _pad) in &FRAMES[ROW3[f as usize]] {
                    let (sheet, src_x, src_y, w, h) = PIECES[piece as usize];
                    draw_image_clipped(
                        &mut g2,
                        &sc.images.xtq[sheet as usize],
                        64 + 16 + dx,
                        96 + 16 + dy,
                        src_x,
                        src_y,
                        w,
                        h,
                    );
                }
                // 玩家尾绘（paint_entities 尾段同源，非本测试标的）
                sc.paint_player(&mut g2, 0, 0);
            }
            assert_eq!(
                actual.argb, expected.argb,
                "帧 {f}（裸帧偏移 {}）拼装逐像素一致",
                ROW3[f as usize]
            );
        }
    }

    /// MVS 多对配对扫描序（a.java:8528-8542 + 8546-8558）：逆序首扫的 82 进
    /// buf[偶]，同键后扫的进 buf[奇]；配对循环按 buf 序处理。本例两对
    /// （键 3 / 键 7）各自互传——键 3 对的怪物落 buf[0]=82p1b 格 (5,5)、
    /// 键 7 对的落 buf[0]=82p2b 格 (11,11)。错配/串对都会改写目的地。
    #[test]
    fn mvs_multi_pair_pinning_scans_and_targets_each_pair_cell() {
        let mut sc = scene_with(&synthetic_sptarm());
        // 对 1（键 3）：82p1a (2,2) / 82p1b (5,5)；对 2（键 7）：82p2a (8,8) / 82p2b (11,11)
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let ep1a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let ep1b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 8 << 5, 8 << 5, 7, &sc.width_table, None);
        let ep2a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 11 << 5, 11 << 5, 7, &sc.width_table, None);
        let ep2b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let m1 = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 42, 8 << 5, 8 << 5, 0, &sc.width_table, None);
        let m2 = sc.entities.count - 1;
        for e in [ep1a, ep1b, ep2a, ep2b] {
            sc.entities.items[e].visible = false;
        }

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        // 四点位全消耗、两原地怪物摘除
        for e in [ep1a, ep1b, ep2a, ep2b, m1, m2] {
            assert!(sc.entities.items[e].removed, "原地实体 {e} 摘除");
        }
        // 互传目的地按各对 buf[0] 实体格：键 3 对 → (5,5)，键 7 对 → (11,11)
        let mut dest: Vec<(i32, i32, i32)> = (0..sc.entities.count)
            .filter(|&i| !sc.entities.items[i].removed && sc.entities.items[i].type_ >= 41)
            .map(|i| (sc.entities.items[i].type_, sc.entities.items[i].px, sc.entities.items[i].py))
            .collect();
        dest.sort_unstable();
        assert_eq!(
            dest,
            vec![(41, 5 << 5, 5 << 5), (42, 11 << 5, 11 << 5)],
            "两对各归各的 buf[0] 82 格（扫描序/配对键钉死）"
        );
    }

    /// MVS 无怪纯配对（a.java:8564-8568）：82 消耗（removeGroupAtCellByType(…,82)）
    /// 是 cat8 搬运分支**内部的副作用**——扫描格无怪物时该分支不触发 ⇒ 配对成立
    /// 也**不消耗**点位、无 spawn。补遗 #4"一次性消耗两点位"的精确条件由此钉死。
    #[test]
    fn mvs_pair_without_monsters_leaves_82_points() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;
        let count_before = sc.entities.count;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        assert!(!sc.entities.items[e82a].removed && !sc.entities.items[e82b].removed, "无怪 ⇒ 82 不消耗");
        assert_eq!(sc.entities.count, count_before, "无怪 ⇒ 无 spawn");
        assert_eq!(sc.script_cursor, 4, "无参 op 游标照常 = 标签后 1 位（a.java:8591）");
    }

    /// carry_monsters_to 旧格同型一并消失 + stale 尾复活（a.java:8872-8878 + 6434-6449 +
    /// 8561-8568 循环体）：同格双 41 时，M1 的 carry_monsters_to 清旧格把 M2 **一并带走**
    /// （removeGroupAtCellByType 按型全清，非搬移）；随后槽组循环的容量快照
    /// 不变、压缩不清尾 ⇒ M2 经 stale 尾重触发再被复制到目标格。净终态 =
    /// 双 41 副本堆聚 (5,5)、原地全灭、实体数 +2（两次 carry_monsters_to spawn）。
    #[test]
    fn m_107_old_cell_same_type_sweep_with_stale_revival() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82a = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 5 << 5, 5 << 5, 3, &sc.width_table, None);
        let e82b = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let m1 = sc.entities.count - 1;
        entity::spawn(&mut sc.entities, &mut sc.grid, 41, 2 << 5, 2 << 5, 0, &sc.width_table, None);
        let m2 = sc.entities.count - 1;
        sc.entities.items[e82a].visible = false;
        sc.entities.items[e82b].visible = false;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        for e in [e82a, e82b, m1, m2] {
            assert!(sc.entities.items[e].removed, "原地实体 {e} 摘除（含同型一并带走）");
        }
        assert_eq!(sc.entities.count, m2 + 3, "两次 carry_monsters_to spawn（stale 尾复活）");
        let alive: Vec<usize> = (0..sc.entities.count).filter(|&i| !sc.entities.items[i].removed).collect();
        assert_eq!(alive.len(), 2, "双 41 副本存活");
        for i in alive {
            assert_eq!(sc.entities.items[i].type_, 41);
            assert_eq!(
                (sc.entities.items[i].px, sc.entities.items[i].py),
                (5 << 5, 5 << 5),
                "副本堆聚目标格 (5,5)"
            );
        }
    }

    /// MVS 不成对单只 82 空转（a.java:8528-8542 配对槽判空 buf[v]-1<=0 存首元素、
    /// 8546-8558 配对循环 `cur_idx > 0` 才进搬运分支）：孤只 82 只占半对（buf[偶]）
    /// ⇒ 搬运分支不触发 ⇒ 全表无变化（不消耗、无 spawn）；无参指令游标仍照常
    /// 推进 = 标签后 1 字符位（a.java:8591）。fixture-plan §2 FX-MVS-A 断言点补口。
    #[test]
    fn mvs_unpaired_single_82_is_noop_but_advances_cursor() {
        let mut sc = scene_with(&synthetic_sptarm());
        entity::spawn(&mut sc.entities, &mut sc.grid, 82, 2 << 5, 2 << 5, 3, &sc.width_table, None);
        let e82 = sc.entities.count - 1;
        sc.entities.items[e82].visible = false;
        let count_before = sc.entities.count;

        sc.script_line = "MVS ".to_string();
        sc.script_cursor = 0;
        sc.execute_script_instruction();

        // 全表无变化：不消耗、无 spawn、字段不动
        assert!(!sc.entities.items[e82].removed, "孤只 82 不消耗");
        assert_eq!(sc.entities.count, count_before, "孤只 82 无 spawn");
        assert_eq!(sc.entities.items[e82].type_, 82);
        assert_eq!(sc.entities.items[e82].param, 3);
        assert!(!sc.entities.items[e82].visible, "hidden 状态不被改写");
        assert_eq!(sc.entities.items[e82].state, 0);
        // 游标照常推进（空转 ≠ 停摆；停摆语义属未知 op，见 unknown_op 测试）
        assert_eq!(sc.script_cursor, 4, "无参 op 游标 = 标签后 1 位（a.java:8591）");

        // 幂等：游标拨回重解析同一 token，MVS 臂本身仍空转
        // （注意不拨游标的第二次调用会走行尾收尾 a.java:8597-8604，非重解析）
        sc.script_cursor = 0;
        sc.execute_script_instruction();
        assert_eq!(sc.entities.count, count_before);
        assert!(!sc.entities.items[e82].removed);
    }

    /// 暗道换乘键 49/55 **分立臂**（a.java:5295-5306）：49=上行 →
    /// findFloorGateEntity(true)（型 7 门）+ changeFloor(param&255, false, true)；
    /// 55=下行 → findFloorGateEntity(false)（型 8 门）+ changeFloor(…, true, true)。
    /// 回归钉（Rust 移植 bug 修复，非原版 bug、不属 D4 保真）：①曾先清零
    /// key_value 再判 up ⇒ up 恒 false、49 臂被并入 55；②Java case break 后
    /// 续走 keyHeldCode switch（a.java:5308 起），臂末 return 曾吞同拍方向步进。
    #[test]
    fn field_input_keys_49_55_split_up_down_gates() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.menu.stair_hint = true; // f_bool_13（a.java:5297/5303 门控）
        sc.persist.min_floor = 1;
        sc.persist.max_floor = 55;
        // 双门并存：型 7（上行）目标层 10、型 8（下行）目标层 20（param>>9==0）
        entity::spawn(&mut sc.entities, &mut sc.grid, 7, 2 << 5, 2 << 5, 10, &sc.width_table, None);
        entity::spawn(&mut sc.entities, &mut sc.grid, 8, 5 << 5, 5 << 5, 20, &sc.width_table, None);

        // 49 = 上行：只认型 7 门；changeFloor 第二参 false
        sc.key_value = 49;
        sc.handle_field_input();
        assert_eq!(sc.key_value, 0, "键值清零（a.java:5296）");
        assert_eq!(
            sc.persist.pending_floor, 10,
            "49 走型 7 门 findFloorGateEntity(true)（a.java:5297）"
        );
        assert!(!sc.persist.went_down, "49 → changeFloor(…, false, true)（a.java:5297）");

        // 55 = 下行：只认型 8 门；changeFloor 第二参 true
        sc.key_value = 55;
        sc.handle_field_input();
        assert_eq!(sc.key_value, 0, "键值清零（a.java:5302）");
        assert_eq!(
            sc.persist.pending_floor, 20,
            "55 走型 8 门 findFloorGateEntity(false)（a.java:5303）"
        );
        assert!(sc.persist.went_down, "55 → changeFloor(…, true, true)（a.java:5303）");

        // break 语义：臂末不 return，同拍 keyHeldCode switch 照跑（a.java:5308 起）
        sc.key_value = 49;
        sc.key_held = -1;
        sc.handle_field_input();
        assert_eq!(sc.facing, 1, "49 臂后 keyHeld switch 照跑（-1=上 → facing=1）");
    }

    /// 49/55 门控：f_bool_13（stair_hint）未亮则双键均不动（a.java:5297/5303）。
    #[test]
    fn field_input_keys_49_55_gated_by_stair_hint() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.menu.stair_hint = false;
        entity::spawn(&mut sc.entities, &mut sc.grid, 7, 2 << 5, 2 << 5, 10, &sc.width_table, None);
        entity::spawn(&mut sc.entities, &mut sc.grid, 8, 5 << 5, 5 << 5, 20, &sc.width_table, None);
        sc.persist.pending_floor = -1;
        for k in [49, 55] {
            sc.key_value = k;
            sc.handle_field_input();
        }
        assert_eq!(sc.persist.pending_floor, -1, "stair_hint 未亮 ⇒ 不换层");
    }

    /// -5/53 战况屏臂 break 语义（a.java:5271-5294 case -5/53 共臂尾 break）：
    /// Java break 出 keyValue switch 后**续走** keyHeldCode switch（5308 起）
    /// ——与方向键同拍时同拍 facing+tryStep。原实现臂末 return 吞掉（与型 15
    /// 同型移植缺陷；2026-10-16 lead 裁定修，非 D4 保真对象）。
    #[test]
    fn field_input_key_minus5_falls_through_to_key_held() {
        let mut sc = scene_with(&synthetic_sptarm());
        // 臂效（无方向键，keyHeld switch 落 default）：战况屏态 + 相机解锁
        sc.key_value = -5;
        sc.handle_field_input();
        assert_eq!(sc.walk_phase, 2, "战况屏态照设（a.java:5292）");
        assert!(!sc.view_locked, "cameraLockedOnPlayer=false（a.java:5280）");
        // break 语义：同拍方向键照跑 keyHeldCode switch（a.java:5308 起）
        sc.key_value = -5;
        sc.key_held = -1;
        sc.handle_field_input();
        assert_eq!(sc.facing, 1, "-5 臂后 keyHeld switch 照跑（-1=上 → facing=1）");
    }

    /// 53 同臂（a.java:5272 case 53 与 -5 共体）：break 语义同上。
    #[test]
    fn field_input_key_53_falls_through_to_key_held() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.key_value = 53;
        sc.handle_field_input();
        assert_eq!(sc.walk_phase, 2, "战况屏态照设（a.java:5292）");
        sc.key_value = 53;
        sc.key_held = -3;
        sc.handle_field_input();
        assert_eq!(sc.facing, 3, "53 臂后 keyHeld switch 照跑（-3=左 → facing=3）");
    }

    /// overlayKind 六臂分派（run() tableswitch 0-5：javap renamed dump 95-663 +
    /// deobf 3252-3345）BUG-002 活码核查补移植钉①：kind4/3/兜底的 -2/-1 翻页
    /// = handleOverlayKey 的 top∓**1**（a.java:4925-4934）——原实现误用
    /// ±lines_per_page（kind0(15/17) 的 ±lines 与 kind5 的 ±1 是对的，勿动）。
    #[test]
    fn overlay_paging_keys_are_single_line_for_kind4_and_default() {
        let mut sc = scene_with(&synthetic_sptarm());
        let long = "一二三四五六七八九十甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌亥"
            .repeat(12);
        sc.show_overlay_message(4, &long, 0, 0);
        assert!(
            sc.dialog.overlay_total_lines > sc.dialog.overlay_lines_per_page + 1,
            "总行数须超出单页才可翻页 total={} lpp={}",
            sc.dialog.overlay_total_lines,
            sc.dialog.overlay_lines_per_page
        );
        sc.dialog.overlay_page_top = 0;
        sc.key_value = -2;
        sc.tick(0, &[]);
        assert_eq!(sc.dialog.overlay_page_top, 1, "kind4 -2 = top+1（handleOverlayKey ±1）");

        sc.show_overlay_message(3, &long, 0, 3);
        sc.dialog.overlay_page_top = 0;
        sc.key_value = -2;
        sc.tick(0, &[]);
        assert_eq!(sc.dialog.overlay_page_top, 1, "kind3 default → handleOverlayKey ±1");
    }

    /// 钉②：kind2 全吞键（Java `case 2: default: break;`，deobf 3290-3291）——
    /// 调用面 0 命中（Java 亦无生产者），死码臂为忠实而保（同 MVS 先例）。
    #[test]
    fn overlay_kind2_swallows_all_keys() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.show_overlay_message(2, "占位", 0, 0);
        for k in [-7, -6, -1, 1] {
            sc.key_value = k;
            sc.tick(0, &[]);
            assert!(sc.dialog.overlay_active, "kind2 全吞键（key={k}）");
        }
    }

    /// 钉③：kind1 图标支路（javap run() 336-441）：right∉{2,3} 全吞键；
    /// right∈{2,3}：0 吞、-2/-1 走 handleOverlayKey、其余关。
    #[test]
    fn overlay_kind1_icon_dispatch() {
        let mut sc = scene_with(&synthetic_sptarm());
        sc.show_overlay_message(1, "占位", 0, 1); // right=1（钥匙图标类）
        sc.key_value = -7;
        sc.tick(0, &[]);
        assert!(sc.dialog.overlay_active, "right 非 2/3：全吞键");

        sc.show_overlay_message(1, "占位", 0, 3); // right=3（拾取浮层实况）
        sc.key_value = 0;
        sc.tick(0, &[]);
        assert!(sc.dialog.overlay_active, "right 为 2/3 + key0 吞");
        sc.key_value = -7;
        sc.tick(0, &[]);
        assert!(!sc.dialog.overlay_active, "right 为 2/3 + 非翻页键关闭");
    }
