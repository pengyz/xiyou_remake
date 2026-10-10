//! 相机三件套 —— m_064（设置+夹取）/ m_105（趋近步进）/ m_104 锚点语义。
//!
//! 证据：reference/src/deobf/a.java
//! - m_064 :7015-7066（钳制：x∈[min,64]；地图窄于屏幕时居中）
//! - m_105 :8669-8700（每 tick (Δ>>2)+2 趋近 target，越界钳制）
//! - m_104 头部（:8246 区域）：cameraX/Y = 屏幕中心像素 - 相机偏移 = 玩家世界位
//! trace 实证：m_105 每步 6px（Δ=16 时 (16>>2)+2），tick20002-20010 十步收敛。

fn self_view_w() -> i32 {
    240
}

fn self_view_h() -> i32 {
    252 // f_int_59 唯一赋值（a.java:9384；复核 R2/R13：无 320）
}

pub struct Camera {
    /// f_int_56/57：相机偏移（左上角相对地图原点）
    pub offset_x: i32,
    pub offset_y: i32,
    /// f_int_123/124：相机锚（趋近当前位置）
    pub anchor_x: i32,
    pub anchor_y: i32,
    /// f_int_125/126：锚点目标
    pub target_x: i32,
    pub target_y: i32,
    /// f_int_58/59：视口尺寸（f_int_58=240、f_int_59=252，a.java:9384-9385——
    /// 无 320；此前文档的 320 为幽灵值，复核 R13/R2 指认）
    pub view_w: i32,
    pub view_h: i32,
    /// 地图像素尺寸（f_int_52/53 = cells<<5）
    pub map_px_w: i32,
    pub map_px_h: i32,
    /// f_bool_10/11：地图窄/矮于视口（居中模式标志）
    pub map_fits_w: bool,
    pub map_fits_h: bool,
}

impl Camera {
    /// m_121 尾部语义：视口 240 宽；f_bool_10 = 240 >= map_px_w。
    pub fn new(map_px_w: i32, map_px_h: i32) -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            anchor_x: 0,
            anchor_y: 0,
            target_x: 0,
            target_y: 0,
            view_w: 240,
            view_h: 252, // m_121: f_int_59 = 252
            map_px_w,
            map_px_h,
            map_fits_w: self_view_w() >= map_px_w,
            map_fits_h: self_view_h() >= map_px_h,
        }
    }

    /// m_064 忠实移植（a.java:7015-7066）。
    /// x 夹取：>64 → 64；< -((cellsW+2)<<5 - 240) → 该下限；y 对称（视口 252）。
    pub fn set_offset(&mut self, x: i32, y: i32) {
        if !self.map_fits_w {
            let min_x = -(((self.map_px_w >> 5) + 2) << 5) + self.view_w;
            self.offset_x = if x > 64 {
                64
            } else if x < min_x {
                min_x
            } else {
                x
            };
        } else {
            self.offset_x = (self.view_w - self.map_px_w) >> 1;
        }
        // m_064 的 y 钳制嵌在 !f_bool_10 分支内（a.java:7047-7059 花括号结构，
        // 复核 R8）：map 宽度适配时 y 落盘不钳制——原版怪癖保留。
        if !self.map_fits_w {
            let min_y = -(((self.map_px_h >> 5) + 2) << 5) + self.view_h;
            self.offset_y = if y > 64 {
                64
            } else if y < min_y {
                min_y
            } else {
                y
            };
        } else {
            self.offset_y = (self.view_h - self.map_px_h) >> 1;
        }
    }

    /// m_104(0) 头部语义：锚点 = 屏幕中心像素 - 相机偏移（= 玩家世界位）。
    pub fn snap_anchor_to_player(&mut self, player_px: i32, player_py: i32) {
        self.anchor_x = ((self.view_w - 32) >> 1) - self.offset_x;
        self.anchor_y = ((self.view_h - 32) >> 1) - self.offset_y;
        let _ = (player_px, player_py);
    }

    /// m_105 忠实移植：每轴 (Δ>>2)+2 趋近 target，跨过后钳制，尾行由锚点重导出
    /// offset（a.java:8669-8700，复核 R7 补 :8700 尾行）。
    pub fn step_toward_target(&mut self) {
        if self.anchor_x < self.target_x {
            self.anchor_x += ((self.target_x - self.anchor_x) >> 2) + 2;
            if self.anchor_x > self.target_x {
                self.anchor_x = self.target_x;
            }
        } else if self.anchor_x > self.target_x {
            self.anchor_x += ((self.target_x - self.anchor_x) >> 2) - 2;
            if self.anchor_x < self.target_x {
                self.anchor_x = self.target_x;
            }
        }
        if self.anchor_y < self.target_y {
            self.anchor_y += ((self.target_y - self.anchor_y) >> 2) + 2;
            if self.anchor_y > self.target_y {
                self.anchor_y = self.target_y;
            }
        } else if self.anchor_y > self.target_y {
            self.anchor_y += ((self.target_y - self.anchor_y) >> 2) - 2;
            if self.anchor_y < self.target_y {
                self.anchor_y = self.target_y;
            }
        }
    }

    /// m_097：锚点到位判定。
    pub fn at_target(&self) -> bool {
        self.anchor_x == self.target_x && self.anchor_y == self.target_y
    }
}
