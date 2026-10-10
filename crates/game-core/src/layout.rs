//! 屏幕布局常量——**视口扩展的唯一修改点**。
//!
//! # 为什么集中于此（D2 扩展缝 + 视口扩大预案）
//!
//! 原版把 240×320 内联在几百个绘制调用里（A 级证据见各常量的 Java 行号）。
//! Stage A 像素一致要求这些数值**逐处与 Java 一致**，因此这里收编为具名常量、
//! 绘制代码禁止内联数字；Stage B 扩大视口时，把这些 `const` 升级为运行时
//! `LayoutConfig` 结构（或按分支参数化），消费方已全部经由此模块。
//!
//! # 分类纪律
//!
//! - `SCREEN_*`：设备视口（240×320），未来可变；
//! - 语义常量（软键栏 y、菜单柱 x 等）：**相对视口推导**的写成 `fn`（如
//!   `softkey_y()`），保证视口变化时推导关系不破；
//! - 与视口无关的纯数据（字符串、颜色码）保持 `const`。

/// 设备视口宽（原版硬编码，MIDP 机型 240）。
pub const SCREEN_W: i32 = 240;
/// 设备视口高（原版硬编码，MIDP 机型 320）。
pub const SCREEN_H: i32 = 320;

/// mode 21 声音询问的提示文字（a.java:3052）的 UTF-16 码元序列。
/// 「是否开启声音？」= U+662F U+5426 U+5F00 U+542F U+58F0 U+97F3 U+FF1F。
pub const SOUND_PROMPT_TEXT: &[u16] = &[0x662F, 0x5426, 0x5F00, 0x542F, 0x58F0, 0x97F3, 0xFF1F];

/// mode 21 提示文字锚点 x（a.java:3052 `drawString(..., 120, 160, 17)`）。
/// 语义 = 水平居中锚：`SCREEN_W / 2`。
pub fn prompt_cx() -> i32 {
    SCREEN_W / 2
}

/// mode 21 提示文字锚点 y（a.java:3052）。语义 = 垂直居中：`SCREEN_H / 2`。
pub fn prompt_cy() -> i32 {
    SCREEN_H / 2
}

/// m_034 左软键底图 y（a.java:5972 `drawImage(f_Image_arr2_00[8][10], 0, 302, 0)`）。
/// 语义 = `SCREEN_H - 18`（302 = 320-18）。
pub fn softkey_base_y() -> i32 {
    SCREEN_H - 18
}

/// m_034 右软键底图 x（a.java:5977 `m_004(..., 222, 302, 1)`）。
/// 语义 = 右对齐 18px 边距：`SCREEN_W - 18`。
pub fn softkey_right_x() -> i32 {
    SCREEN_W - 18
}

/// m_034 软键图标 y（a.java:5973 `m_002(..., 2, 307, ...)`）。
pub fn softkey_icon_y() -> i32 {
    SCREEN_H - 13
}

/// m_034 左软键图标 x（a.java:5973）。
pub const SOFTKEY_ICON_LEFT_X: i32 = 2;

/// m_034 右软键图标 x（a.java:5978 `m_002(..., 226, 307, ...)`）。
/// 语义 = `SCREEN_W - 14`。
pub fn softkey_icon_right_x() -> i32 {
    SCREEN_W - 14
}

/// m_034 软键精灵条子图标尺寸 12×10（a.java:5973/5978 的 m_002 宽高参数）。
pub const SOFTKEY_ICON_W: i32 = 12;
pub const SOFTKEY_ICON_H: i32 = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_prompt_text_is_utf16_exact() {
        // "是否开启声音？" = U+662F U+5426 U+5F00 U+542F U+58F0 U+97F3 U+FF1F
        let expect: [u16; 7] = [0x662F, 0x5426, 0x5F00, 0x542F, 0x58F0, 0x97F3, 0xFF1F];
        assert_eq!(SOUND_PROMPT_TEXT, &expect);
    }

    #[test]
    fn derived_layout_matches_java_literals() {
        assert_eq!(prompt_cx(), 120);
        assert_eq!(prompt_cy(), 160);
        assert_eq!(softkey_base_y(), 302);
        assert_eq!(softkey_right_x(), 222);
        assert_eq!(softkey_icon_y(), 307);
    }
}
