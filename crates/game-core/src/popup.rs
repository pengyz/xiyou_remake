//! popup 环形队列（spawnPopup a.java:9678-9715 + drawPopupLayer a.java:9719-9779）。
//!
//! 30 槽环形缓冲：write 追加（回绕覆盖）、retire 在 draw 中推进（绘制完
//! 最后一帧后跳过已 retired 槽）。种类：
//! - 1：物品图标条（map[2][6] 37×19 帧 ×value），y 上升 4px/拍
//! - 2/3/4：数字条（map[2][3/4/5] 11 分格数字带，红/绿/金），y 上升 4px/拍；
//!   kind4 定位固定 (240,30)（金币）
//! - 5/6/7：战斗伤害数字（ui[8][9] + 六张散射偏移表，无 y 上升）
//! 计数器 0..=8，>7 置 retired（第 9 拍仍绘制后退役）。

use crate::menu_family::paint_number;
use crate::paint::draw_image_clipped;
use crate::render::{ArgbImage, SoftGraphics};

/// 伤害数字散射偏移表 ×6（Java f_byte_arr_32..37，a.java:705-747 构造常量）；
/// 索引 = 上升计数器 0..=8。
const RISE_X5: [i32; 9] = [10, 6, -14, -26, -36, -38, -39, -41, -43];
const RISE_Y5: [i32; 9] = [16, 4, -10, 0, 14, 12, 14, 11, 13];
const RISE_X6: [i32; 9] = [9, 22, 34, 41, 51, 52, 54, 55, 56];
const RISE_Y6: [i32; 9] = [15, -2, -19, -1, 21, 17, 21, 19, 21];
const RISE_X7: [i32; 9] = [9, -1, -10, -22, -26, -27, -29, -30, -32];
const RISE_Y7: [i32; 9] = [20, 1, -15, 0, 24, 20, 24, 20, 24];

/// popup 层绘制所需的 map/ui 图集（scene 装配）。
pub struct PopupImages {
    /// map[2][3]/[2][4]/[2][5]：kind 2/3/4 的 11 分格数字带
    pub digit_strips: [ArgbImage; 3],
    /// map[2][6]：kind 1 物品图标条（37×19 帧）
    pub icon_strip: ArgbImage,
    /// ui[8][9]：kind 5/6/7 战斗伤害数字
    pub battle_digits: ArgbImage,
}

/// 30 槽环形 popup 队列（Java popupKind/Value/X/Y + popupRetired + popupRiseCounter）。
#[derive(Default)]
pub struct PopupRing {
    kind: [i32; 30],
    value: [i32; 30],
    x: [i32; 30],
    y: [i32; 30],
    counter: [i32; 30],
    retired: [bool; 30],
    write: usize,
    retire: usize,
}

impl PopupRing {
    /// spawnPopup（a.java:9678-9715）。`wx/wy` 为世界坐标（+cam 平移在此发生）；
    /// `digit_w` = map[2][3] 的数字格宽（宽/11）——kind 2/3/4 的 x 居中计算用。
    pub fn spawn(&mut self, kind: i32, mut value: i32, wx: i32, wy: i32, cam_x: i32, cam_y: i32, digit_w: i32) {
        let sx = wx + cam_x;
        let sy = wy + cam_y;
        let w = self.write;
        self.kind[w] = kind;
        self.value[w] = value;
        self.retired[w] = false;
        self.counter[w] = 0;
        self.y[w] = sy;
        if kind == 4 {
            self.x[w] = 240;
            self.y[w] = 30;
        } else if (5..=7).contains(&kind) {
            self.x[w] = sx + 16;
        } else if kind == 1 {
            self.x[w] = sx - 2;
        } else {
            // kind 2/3/4 之外的数字类：数字串总宽 + 半格 居中（Java acc 循环）
            let mut v = value;
            if v < 0 {
                v = -v;
            }
            let mut acc = digit_w;
            loop {
                acc += digit_w;
                v /= 10;
                if v <= 0 {
                    break;
                }
            }
            self.x[w] = sx + ((32 + acc) >> 1);
        }
        self.write = (self.write + 1) % 30;
    }

    /// drawPopupLayer（a.java:9719-9779）：retire→write 遍历绘制并推进计数。
    pub fn draw(&mut self, g: &mut SoftGraphics<'_>, imgs: &PopupImages) {
        let mut i = self.retire;
        loop {
            if i >= 30 {
                i = 0;
            }
            if i == self.write {
                // 末尾：推进 retire（已退役槽出队）
                if self.retire != self.write && self.retired[self.retire] {
                    self.retire = (self.retire + 1) % 30;
                }
                return;
            }
            if !self.retired[i] {
                let kind = self.kind[i];
                let value = self.value[i];
                let x = self.x[i];
                let c = self.counter[i];
                let base_y = 16 + self.y[i];
                match kind {
                    1 => {
                        let y = base_y - (self.counter[i] << 2);
                        draw_image_clipped(g, &imgs.icon_strip, x, y, 0, 19 * value, 37, 19);
                    }
                    2 | 3 | 4 => {
                        let y = base_y - (self.counter[i] << 2);
                        draw_digit_strip(g, &imgs.digit_strips[(kind - 2) as usize], value, x, y);
                    }
                    5 => {
                        paint_number(g, &imgs.battle_digits, value, x + RISE_X5[c as usize], 16 + self.y[i] + RISE_Y5[c as usize]);
                    }
                    6 => {
                        paint_number(g, &imgs.battle_digits, value, x + RISE_X6[c as usize], 16 + self.y[i] + RISE_Y6[c as usize]);
                    }
                    7 => {
                        paint_number(g, &imgs.battle_digits, value, x + RISE_X7[c as usize], 16 + self.y[i] + RISE_Y7[c as usize]);
                    }
                    _ => {}
                }
                if self.counter[i] > 7 {
                    self.retired[i] = true;
                } else {
                    self.counter[i] += 1;
                }
            }
            i += 1;
        }
    }

    /// popup 是否全部退役（测试锚）。
    pub fn is_empty(&self) -> bool {
        self.retire == self.write
    }
}

/// drawDigitStrip（a.java:9780-9807）：11 分格数字带右对齐逐位绘制 +
/// **第 11 格装饰字形无条件绘制**（a.java:9800-9803：负值标志 var5 是死变量，
/// 取反只影响逐位数字，尾部装饰格恒画——T1506 全量 ops 实证）。
/// 返回绘制位数+1（宽度推进用）。
pub fn draw_digit_strip(g: &mut SoftGraphics<'_>, strip: &ArgbImage, mut value: i32, x: i32, y: i32) -> i32 {
    if value < 0 {
        value = -value;
    }
    let digit_w = strip.width / 11;
    let h = strip.height;
    let mut cx = x;
    let mut count = 0;
    loop {
        cx -= digit_w + 1;
        let d = value % 10;
        g.set_clip(cx, y, digit_w, h);
        g.draw_image(strip, cx - d * digit_w, y, 0);
        value /= 10;
        count += 1;
        if value <= 0 {
            break;
        }
    }
    // 尾部装饰格（带宽第 11 格）恒画
    cx -= digit_w + 1;
    g.set_clip(cx, y, digit_w, h);
    g.draw_image(strip, cx - (strip.width - digit_w), y, 0);
    g.set_clip(0, 0, crate::layout::SCREEN_W, crate::layout::SCREEN_H);
    count + 1
}
