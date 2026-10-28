//! T3 shell 触屏映射（touch-native-undo-plan §1「映射在壳、逻辑在核」）：
//! 触点→格坐标反变换（视口/相机/缩放逆变换终态）、撤销按钮命中、滑动=方向
//! 手动踏步（一格/次、撞墙即停）、双模并存（键/触不互斥）、打断全通道。
//!
//! **纯逻辑零 IO**：android/win 双壳同构消费本模块（win 鼠标=触点等价）。
//! 红线：game-core 零改动——本模块只做映射与动作流组装，执行走 core 既有入口
//! （walk_to/probe_step/interrupt_route/undo_journal）。
//!
//! 方向编码 = try_step facing 域（scene.rs:4985）：0=Down 1=Up 2=Right 3=Left。


// ---------------- 实现 ----------------

/// 逻辑画布→格反变换几何（视口/相机/缩放逆变换的壳侧终态：各壳 Viewport 已把
/// 设备像素反映射为逻辑 px；本层做相机/tile_off 反变换 + 图内判定）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TouchGeom {
    /// cameraPixelX（view.cam_x）
    pub cam_x: i32,
    /// cameraPixelY（view.cam_y）
    pub cam_y: i32,
    /// 瓦片层 y 偏移（layout::view_top_y，classic=20）
    pub tile_off: i32,
    /// 地图格列数（view.wide）
    pub cols: i32,
    /// 地图格行数（view.high）
    pub rows: i32,
}

impl TouchGeom {
    /// 格像素 = 游戏格网 32 px（`player_cell_x = player_px >> 5`、实体 `cx << 5`、`map_w_px = wide << 5`；
    /// walkable / walk_to / probe_step 同一格网）。
    /// > 【2026-10-28 更正 · BUG-042】原取 16（"paint_tiles 16px 步进"）：那是瓦片层的**子瓦片**（`wide*2` 列、每格 2×2 块），
    /// > 不是游戏格。用 16 换算后格号翻倍 ⇒ 点地图大部分区域越界被忽略、其余落到错误格（多为墙 ⇒ 探步撞墙），点地寻路基本不走。
    pub const TILE: i32 = 32;

    /// 屏幕逻辑 px → 格（半开 TILE = 32 px 格网；界外 None）。
    pub fn cell_at(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let cx = (x - self.cam_x).div_euclid(Self::TILE);
        let cy = (y - self.cam_y - self.tile_off).div_euclid(Self::TILE);
        (cx >= 0 && cy >= 0 && cx < self.cols && cy < self.rows).then_some((cx, cy))
    }

    /// 格 → 屏幕逻辑原点（`cell_at` 正变换，四角互逆锚）。
    pub fn cell_screen_xy(&self, cx: i32, cy: i32) -> (i32, i32) {
        (self.cam_x + cx * Self::TILE, self.cam_y + self.tile_off + cy * Self::TILE)
    }
}

/// 撤销按钮区宽（逻辑，屏幕右上角）。
pub const UNDO_ZONE_W: i32 = 56;
/// 撤销按钮区高（逻辑）。
pub const UNDO_ZONE_H: i32 = 20;

/// 撤销按钮命中（常驻区=右上角 56×20；plan §0 边界 3）。
pub fn undo_zone_hit(x: i32, y: i32, screen_w: i32, screen_h: i32) -> bool {
    screen_h > 0 && x >= screen_w - UNDO_ZONE_W && x < screen_w && y >= 0 && y < UNDO_ZONE_H
}

/// 滑动 → 方向（try_step facing 域 0=Down 1=Up 2=Right 3=Left）；主轴主导、
/// 长度 < min_len ⇒ None（tap）。
pub fn swipe_dir(dx: i32, dy: i32, min_len: i32) -> Option<u8> {
    if dx.abs() < min_len && dy.abs() < min_len {
        return None;
    }
    Some(if dx.abs() >= dy.abs() {
        if dx >= 0 { 2 } else { 3 }
    } else if dy >= 0 {
        0
    } else {
        1
    })
}

/// 手势判定（tap / 滑动踏步）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesture {
    Tap(i32, i32),
    Swipe(u8),
    None,
}

/// 手势追踪器：**一格/次**——一个手势至多一次滑动踏步（越阈即发、发后停手，
/// 撞墙即停语义=无后续自动步）。
#[derive(Debug)]
pub struct GestureTracker {
    min_len: i32,
    start: Option<(i32, i32)>,
    fired: bool,
}

impl GestureTracker {
    pub fn new(min_len: i32) -> GestureTracker {
        GestureTracker { min_len, start: None, fired: false }
    }

    pub fn down(&mut self, x: i32, y: i32) {
        self.start = Some((x, y));
        self.fired = false;
    }

    /// 位移中：越阈 → 一次性 Some(dir)（同手势后续 None）。
    pub fn move_to(&mut self, x: i32, y: i32) -> Option<u8> {
        if self.fired {
            return None;
        }
        let (sx, sy) = self.start?;
        let dir = swipe_dir(x - sx, y - sy, self.min_len)?;
        self.fired = true;
        Some(dir)
    }

    /// 抬起：已消费手势=None；短按=Tap(down 点)。
    pub fn up(&mut self, _x: i32, _y: i32) -> Gesture {
        let start = self.start.take();
        if self.fired {
            self.fired = false;
            return Gesture::None;
        }
        match start {
            Some((sx, sy)) => Gesture::Tap(sx, sy),
            None => Gesture::None,
        }
    }
}

/// 点按分类（撤销区 > 点墙=探步入 > 点空地=walk_to）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapAction {
    Undo,
    WalkTo((i32, i32)),
    /// 探步入（朝点中的墙格；facing 域 0=Down 1=Up 2=Right 3=Left）
    ProbeStep(u8),
    Ignore,
}

/// 点按分类：触点→格→动作（T3 §1：点墙格=探步入请求，点空地=WalkTo(tile)）。
pub fn classify_tap(
    x: i32,
    y: i32,
    geom: &TouchGeom,
    screen_w: i32,
    screen_h: i32,
    player: (i32, i32),
    walkable: impl Fn(i32, i32) -> bool,
) -> TapAction {
    if undo_zone_hit(x, y, screen_w, screen_h) {
        return TapAction::Undo;
    }
    let Some((cx, cy)) = geom.cell_at(x, y) else {
        return TapAction::Ignore;
    };
    if (cx, cy) == player {
        return TapAction::Ignore;
    }
    if walkable(cx, cy) {
        TapAction::WalkTo((cx, cy))
    } else {
        // 点墙格=探步入（主轴朝向；探墙双通道与按键踏步等权走 try_step）
        let (dx, dy) = (cx - player.0, cy - player.1);
        TapAction::ProbeStep(if dx.abs() >= dy.abs() {
            if dx >= 0 { 2 } else { 3 }
        } else if dy >= 0 {
            0
        } else {
            1
        })
    }
}

/// 壳侧动作流（win/android 同构消费；执行=core 既有入口）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellAction {
    /// 打断寻路（任意手动输入首动作；core interrupt_route 幂等，route 未开=无操作）
    Interrupt,
    WalkTo((i32, i32)),
    ProbeStep(u8),
    Undo,
    Key(i32),
    KeyRelease,
}

/// 全通道分发器：**打断信号全通道发出**（触点/滑动/方向键/软键/撤销皆首动作
/// Interrupt）；**双模并存**（键/触动作流无互斥态、无压制）。
pub struct Dispatcher;

impl Dispatcher {
    pub fn tap(a: TapAction) -> Vec<ShellAction> {
        let act = match a {
            TapAction::Undo => ShellAction::Undo,
            TapAction::WalkTo(t) => ShellAction::WalkTo(t),
            TapAction::ProbeStep(d) => ShellAction::ProbeStep(d),
            TapAction::Ignore => return Vec::new(),
        };
        vec![ShellAction::Interrupt, act]
    }

    /// 滑动=方向手动踏步（一格/次；探步返回 false=撞墙即停，无后续步）。
    pub fn swipe(dir: u8) -> Vec<ShellAction> {
        vec![ShellAction::Interrupt, ShellAction::ProbeStep(dir)]
    }

    pub fn key(code: i32) -> Vec<ShellAction> {
        vec![ShellAction::Interrupt, ShellAction::Key(code)]
    }

    pub fn release() -> Vec<ShellAction> {
        vec![ShellAction::KeyRelease]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// maplv5 对照锚同款几何（cam(-144,-16)、tile_off=20）。maplv5 头 26 = 子瓦片数 ⇒ **13×13 游戏格**（32 px）。
    /// > 【2026-10-28 更正 · BUG-042】原写"26×26 格图"并按 16 px 求期望——把子瓦片当成了格；下列期望改由游戏绘制公式
    /// > 推出：格 (c,r) 屏幕原点 = (cam_x + 32c, cam_y + tile_off + 32r)（scene.rs paint_entities / paint_player 同式）。
    fn geom() -> TouchGeom {
        TouchGeom { cam_x: -144, cam_y: -16, tile_off: 20, cols: 13, rows: 13 }
    }

    // —— ① 坐标反变换四角（视口/相机反变换：逻辑 px → 格） ——

    #[test]
    fn cell_at_four_corners_of_tile_window() {
        let g = geom();
        // 格 (5,1) 起绘于屏幕 (16,36)（cam_x+32*5=16、cam_y+20+32*1=36）
        assert_eq!(g.cell_at(16, 36), Some((5, 1)), "格左上角");
        assert_eq!(g.cell_at(47, 67), Some((5, 1)), "格右下内沿（32px 半开）");
        assert_eq!(g.cell_at(48, 68), Some((6, 2)), "右下邻格起点");
        assert_eq!(g.cell_at(239, 319), Some((11, 9)), "画面右下角（x=(239+144)/32、y=(319-4)/32）");
        // 反变换四角互逆：cell_screen_xy(cell_at(p)) = 格原点
        for (x, y) in [(16, 36), (48, 68), (112, 164), (239, 319)] {
            let (cx, cy) = g.cell_at(x, y).unwrap();
            let (ox, oy) = g.cell_screen_xy(cx, cy);
            assert!(x >= ox && x < ox + 32 && y >= oy && y < oy + 32, "({x},{y}) 落回格原点 ({ox},{oy})");
        }
    }

    #[test]
    fn cell_at_out_of_map_is_none() {
        let g = geom();
        assert_eq!(g.cell_at(-145, 20), None, "左越界（col<0；cam_x=-144 左沿）");
        assert_eq!(g.cell_at(0, 3), None, "上越界（row<0；地图顶沿 y=4）");
        assert_eq!(g.cell_at(272, 20), None, "右越界（col>=13：-144+13*32=272）");
        assert_eq!(g.cell_at(271, 20), Some((12, 0)), "右沿内");
        assert_eq!(g.cell_at(0, 420), None, "下越界（row>=13：4+13*32=420）");
        assert_eq!(g.cell_at(0, 419), Some((4, 12)), "下沿内");
    }

    // —— ② 撤销命中（常驻按钮区，右上 56×20） ——

    #[test]
    fn undo_zone_hit_corners() {
        let (w, h) = (240, 320);
        assert!(undo_zone_hit(w - 56, 0, w, h), "区内左上角");
        assert!(undo_zone_hit(w - 1, 19, w, h), "区内右下角");
        assert!(!undo_zone_hit(w - 57, 0, w, h), "区外左沿");
        assert!(!undo_zone_hit(w - 1, 20, w, h), "区外下沿");
        assert!(!undo_zone_hit(w - 56, 0, 240, 320 - 320), "screen_h=0 防御");
    }

    // —— ③ 滑动=方向手动踏步（一格/次、撞墙即停） ——

    #[test]
    fn swipe_maps_to_facing_and_one_step_per_gesture() {
        assert_eq!(swipe_dir(0, 40, 24), Some(0), "下滑=Down(facing 0)");
        assert_eq!(swipe_dir(0, -40, 24), Some(1), "上滑=Up(facing 1)");
        assert_eq!(swipe_dir(40, 0, 24), Some(2), "右滑=Right(facing 2)");
        assert_eq!(swipe_dir(-40, 0, 24), Some(3), "左滑=Left(facing 3)");
        assert_eq!(swipe_dir(4, 4, 24), None, "阈值内=非滑动（tap）");
        // 一格/次：一个手势至多一次滑动踏步
        let mut gt = GestureTracker::new(24);
        gt.down(100, 100);
        assert_eq!(gt.move_to(140, 100), Some(2), "越阈值发一次");
        assert_eq!(gt.move_to(300, 100), None, "同手势续滑不再发（一格/次）");
        assert_eq!(gt.up(300, 100), Gesture::None, "手势已消费");
    }

    #[test]
    fn swipe_wall_stop_no_repeat() {
        // 撞墙即停：一次手势一步；探步返回 false（撞墙）后同手势不再产生后续步
        let acts = Dispatcher::swipe(2);
        assert_eq!(acts, vec![ShellAction::Interrupt, ShellAction::ProbeStep(2)], "滑动=打断+一步探步");
        let mut gt = GestureTracker::new(24);
        gt.down(0, 0);
        let first = gt.move_to(80, 0);
        assert_eq!(first, Some(2));
        assert_eq!(gt.move_to(200, 0), None, "撞墙（步失败）也无后续步——手势已停");
    }

    #[test]
    fn tap_classifies_walk_wall_probe_and_undo() {
        let g = geom();
        let player = (5, 1);
        let walkable = |x: i32, y: i32| !(x == 6 && y == 1); // (6,1)=墙，其余可走
        // 点空地 → WalkTo(tile)：格 (7,2) 原点 (80,68)
        assert_eq!(
            classify_tap(90, 80, &g, 240, 320, player, &walkable),
            TapAction::WalkTo((7, 2)),
            "点空地=walk_to(格)"
        );
        // 点墙格（相邻）→ 探步入（朝墙方向）：格 (6,1) 原点 (48,36)
        assert_eq!(
            classify_tap(50, 40, &g, 240, 320, player, &walkable),
            TapAction::ProbeStep(2),
            "点墙格=探步入 Right"
        );
        // 撤销区 → Undo
        assert_eq!(classify_tap(200, 5, &g, 240, 320, player, &walkable), TapAction::Undo, "撤销区=Undo");
        // 玩家自身格 / 画外 → Ignore
        assert_eq!(classify_tap(17, 37, &g, 240, 320, player, &walkable), TapAction::Ignore, "点自身格=忽略");
        assert_eq!(classify_tap(5000, 0, &g, 240, 320, player, &walkable), TapAction::Ignore, "画外=忽略");
    }

    // —— ④ 双模无互斥（键/触并存，不是替代） ——

    #[test]
    fn dual_mode_keys_and_taps_coexist() {
        // 触点通道动作后，键通道照常发键（无互斥开关/无压制）
        let g = geom();
        let tap = Dispatcher::tap(classify_tap(90, 80, &g, 240, 320, (5, 1), &|_, _| true));
        assert_eq!(tap[1], ShellAction::WalkTo((7, 2)));
        let key = Dispatcher::key(-4);
        assert_eq!(key, vec![ShellAction::Interrupt, ShellAction::Key(-4)], "tap 后按键不被压制");
        let tap2 = Dispatcher::tap(TapAction::WalkTo((12, 2)));
        assert_eq!(tap2[1], ShellAction::WalkTo((12, 2)), "按键后 tap 不被压制");
        assert_eq!(Dispatcher::release(), vec![ShellAction::KeyRelease]);
    }

    // —— ⑤ 打断信号全通道发出（触点/滑动/方向键/软键） ——

    #[test]
    fn interrupt_signal_on_every_channel() {
        let g = geom();
        let tap = Dispatcher::tap(classify_tap(90, 80, &g, 240, 320, (5, 1), &|_, _| false));
        assert_eq!(tap[0], ShellAction::Interrupt, "触点通道发打断");
        assert_eq!(Dispatcher::swipe(0)[0], ShellAction::Interrupt, "滑动通道发打断");
        assert_eq!(Dispatcher::key(-1)[0], ShellAction::Interrupt, "方向键通道发打断");
        assert_eq!(Dispatcher::key(-7)[0], ShellAction::Interrupt, "软键通道发打断");
        assert_eq!(Dispatcher::tap(TapAction::Undo)[0], ShellAction::Interrupt, "撤销通道发打断");
    }
}
