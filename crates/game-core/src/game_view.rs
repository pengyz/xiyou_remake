//! mode 3 游戏画面的数据层与瓦片层（loadFloorData 装载 a.java:9375-9417、
//! setCameraClamped 相机 a.java:7014-7062、paintTileLayer 瓦片层 a.java:6966-7022）。
//!
//! # 地图 grid 语义（loadFloorData + rebuildWalkability/paintTileLayer 的 stride-2 一致性）
//!
//! - maplv 头：`u16 wide<<1`、`u16 high<<1`（`>>1` 得格数，a.java:9382-9383）
//! - terrain/transform 各为 `byte[wide*high*4]` 连续区（a.java:9387-9391）
//! - **取值下标 stride-2**：格 (r,c) 的值 = `grid[(r*wide + c) * 2]`
//!   （paintTileLayer a.java:6996-7001 与 rebuildWalkability a.java:7108-7112 同一算式——
//!   每格 4 字节块中有效数据在偶位）
//! - 瓦片值 v：`sx = (v&7)<<4`、`sy = (v>>3)<<4`（tileset 128x208 的
//!   8×13 格 16px 索引）；变换取 mapTransformGrid 同下标
//!
//! # 相机（setCameraClamped，walk 批 camera.rs 的同源 setter——此处为 grid 窗口派生）
//!
//! `cameraPixelX/57` 像素偏移 + `tileColStart..63` 列/行窗口；钳制域
//! `[64, -((n+2)<<5 - view)]`；整图容纳时居中（mapFitsWidth/11）。
//!
//! # 对拍锚（A-gameplay-floor1 T544）
//!
//! preset floor1（cell (3,10)、加载步 11 居中相机）→ 首瓦片
//! `setClip(24,18,16,16) + drawImage(128x208,-8,18)`：cam_x=8/col_start=1、
//! cam_y+20+row_start*16=18、terrain 首格 v=2（sx=32/sy=0）。

use crate::render::{ArgbImage, SoftGraphics};

/// 游戏视图数据层（loadFloorData 装载产物 + 相机窗口）。
pub struct GameView {
    pub wide: i32,
    pub high: i32,
    /// terrain 网格（原始字节；stride-2 索引）。
    pub terrain: Vec<u8>,
    /// transform 网格（同上）。
    pub transform: Vec<u8>,
    /// viewWidthPx/viewHeightPx：视口像素宽高（240/252）。
    pub view_w: i32,
    pub view_h: i32,
    /// mapPixelWidth/mapPixelHeight：地图像素宽高（wide<<5 / high<<5）。
    map_w_px: i32,
    map_h_px: i32,
    /// mapFitsWidth/mapFitsHeight：整图容纳。
    fits_w: bool,
    fits_h: bool,
    /// cameraPixelX/cameraPixelY：相机像素偏移。
    pub cam_x: i32,
    pub cam_y: i32,
    /// tileColStart..63：列/行可见窗口。
    col_start: i32,
    col_end: i32,
    row_start: i32,
    row_end: i32,
    /// 玩家像素位（加载步 11 的 paintPlayerSprite 输入）。
    pub player_px: i32,
    pub player_py: i32,
}

impl GameView {
    /// `loadFloorData(floor)`（a.java:9375-9407）：maplv 头 + 双 grid 读取 + 相机 0。
    /// `m_122/sortEntitiesByY`（实体装载）在 game_view 之外（Floor1Host 侧）。
    pub fn from_maplv(data: &[u8], player_px: i32, player_py: i32) -> Result<GameView, String> {
        if data.len() < 4 {
            return Err("maplv too short".into());
        }
        // 头为 u16 **LE**（resource-formats.md：maplv0 头 1a00 1a00 = 26×26，
        // readU16BE 读后 >>1 得格数 13×13——a.java:9382-9383）
        let wide = u16::from_le_bytes([data[0], data[1]]) as i32 >> 1;
        let high = u16::from_le_bytes([data[2], data[3]]) as i32 >> 1;
        let grid_len = (wide * high << 2) as usize;
        if data.len() < 4 + grid_len * 2 {
            return Err(format!("maplv grid truncated: {} < {}", data.len(), 4 + grid_len * 2));
        }
        let terrain = data[4..4 + grid_len].to_vec();
        let transform = data[4 + grid_len..4 + grid_len * 2].to_vec();
        let mut v = GameView {
            wide,
            high,
            terrain,
            transform,
            view_w: 240,
            view_h: 252,
            map_w_px: wide << 5,
            map_h_px: high << 5,
            fits_w: false,
            fits_h: false,
            cam_x: 0,
            cam_y: 0,
            col_start: 0,
            col_end: 0,
            row_start: 0,
            row_end: 0,
            player_px,
            player_py,
        };
        v.fits_w = v.view_w >= v.map_w_px;
        v.fits_h = v.view_h >= v.map_h_px;
        v.set_camera(0, 0); // loadFloorData 尾的 setCameraClamped(0,0)
        Ok(v)
    }

    /// `setCameraClamped(x, y)`（a.java:7014-7062）：钳制 + 窗口派生。
    pub fn set_camera(&mut self, mut x: i32, mut y: i32) {
        if !self.fits_w {
            let min = -((self.wide + 2 << 5) - self.view_w);
            if x > 64 {
                x = 64;
            } else if x < min {
                x = min;
            }
            self.cam_x = x;
            if x < 0 {
                self.col_start = -x >> 4;
                self.col_end = self.col_start + (self.view_w >> 4) + 1;
            } else {
                self.col_start = 0;
                self.col_end = (self.view_w - x >> 4) + 1;
            }
            if self.col_end > self.wide << 1 {
                self.col_end = self.wide << 1;
            }
        } else {
            self.cam_x = self.view_w - self.map_w_px >> 1;
            self.col_start = 0;
            self.col_end = self.wide << 1;
        }
        if !self.fits_h {
            let min = -((self.high + 2 << 5) - self.view_h);
            if y > 64 {
                y = 64;
            } else if y < min {
                y = min;
            }
            self.cam_y = y;
            if y < 0 {
                self.row_start = -y >> 4;
                self.row_end = self.row_start + (self.view_h >> 4) + 2;
            } else {
                self.row_start = 0;
                self.row_end = (self.view_h - y >> 4) + 1;
            }
            if self.row_end > self.high << 1 {
                self.row_end = self.high << 1;
            }
        } else {
            self.cam_y = self.view_h - self.map_h_px >> 1;
            self.row_start = 0;
            self.row_end = self.high << 1;
        }
    }

    /// 格 (r,c) 的 terrain 值：`idx = r*wide*2 + c`（**行步进 2·wide、
    /// 列步进 1**——paintTileLayer a.java:6996/6998 与 rebuildWalkability a.java:7108 同式）。
    pub fn terrain_at(&self, r: i32, c: i32) -> u8 {
        self.terrain[(r * self.wide * 2 + c) as usize]
    }


    /// 格 (r,c) 的 transform 值（同上布局）。
    pub fn transform_at(&self, r: i32, c: i32) -> u8 {
        self.transform[(r * self.wide * 2 + c) as usize]
    }

    /// `paintTileLayer(0, offset_y)`（a.java:6996-7022）：瓦片层绘制。
    /// `walkPhase==4` 的楼层切换特例未端口（mode 3 稳态不用）。
    pub fn paint_tiles(&self, g: &mut SoftGraphics<'_>, tileset: &ArgbImage, offset_y: i32) {
        let x0 = self.cam_x + (self.col_start << 4);
        let y0 = self.cam_y + offset_y + (self.row_start << 4);
        // 下标布局（a.java:6996/7001-7004）：行 base = col_start + row*wide*2，
        // 列步进 1；**行尾 base += wide*2**（跳过整行距——不是列循环自然累加）
        let mut base = self.col_start + self.row_start * self.wide * 2;
        let mut py = y0;
        let mut row = self.row_start;
        while row < self.row_end {
            let mut idx = base;
            let mut px = x0;
            let mut col = self.col_start;
            while col < self.col_end {
                let v = self.terrain[idx as usize];
                if v > 0 {
                    let sx = ((v & 7) << 4) as i32;
                    let sy = ((v >> 3) << 4) as i32;
                    let t = self.transform[idx as usize] as i32;
                    crate::menu_family::draw_edge_patch(g, tileset, px, py, sx, sy, 16, 16, t);
                }
                col += 1;
                idx += 1;
                px += 16;
            }
            base += self.wide * 2;
            row += 1;
            py += 16;
        }
        g.set_clip(0, 0, 240, 320);
    }

    /// 加载步 11 的居中相机（a.java:3502：`setCameraClamped((view_w-32>>1)-px, (view_h-32>>1)-py)`）。
    pub fn center_on_player(&mut self) {
        self.set_camera((self.view_w - 32 >> 1) - self.player_px, (self.view_h - 32 >> 1) - self.player_py);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn maplv_bytes() -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/raw/maplv1");
        std::fs::read(path).unwrap()
    }

    /// T544 **全瓦片层 ops 对拍**：Rust paint_tiles 的操作流与 Java
    /// A-gameplay-floor1 TICK 0544 的瓦片段（250 条，含 256 截断窗内的
    /// setClip/draw/drawImageT 全集）逐条一致。
    #[test]
    fn t544_tile_layer_ops_match() {
        let mut v = GameView::from_maplv(&maplv_bytes(), 96, 320).unwrap();
        v.center_on_player();
        // tileset = map 容器 [2][0]（128x208）
        let packed = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/raw/map"),
        )
        .unwrap();
        let tiles = game_data::PackedPng::parse(&packed, 12)
            .unwrap()
            .images
            .iter()
            .map(|sub| ArgbImage::from_decoded_png(game_data::decode_png(sub.bytes).unwrap()))
            .collect::<Vec<_>>();
        assert_eq!((tiles[0].width, tiles[0].height), (128, 208));

        let mut screen = ArgbImage::create(240, 320);
        let mut g = SoftGraphics::new(&mut screen);
        v.paint_tiles(&mut g, &tiles[0], 20);

        // 期望：trace 瓦片段（跳过 ops 截断尾差——Java 250 条是截断值，
        // Rust 全量更长；取 Java 前 N 条作前缀比对）
        let expect: Vec<&str> = include_str!("../../../data/golden/t544-tiles.ops")
            .lines()
            .filter(|l| !l.is_empty())
            .collect();
        for (i, want) in expect.iter().enumerate() {
            let got = g.ops.get(i).map(|s| s.as_str()).unwrap_or("<END>");
            // op 文本格式差（", " vs ","）归一
            let norm = |s: &str| s.replace(", ", ",");
            assert_eq!(norm(got), norm(want), "瓦片 op #{i}");
        }
    }

    /// T544 瓦片锚（A-gameplay-floor1 首 3 瓦片 ops 复算）：
    /// player cell (3,10) → 居中相机 (8,-210)，列窗 [0,15) 行窗 [13,30)；
    /// 格(13,0) v=0 跳过；格(13,1) v=2 → `clip(24,18)+draw(−8,18)`；
    /// 格(13,2) v=64 → `clip(40,18)+draw(40,−110)`（trace 第 2 瓦片）。
    #[test]
    fn t544_first_tiles_anchor() {
        let mut v = GameView::from_maplv(&maplv_bytes(), 96, 320).unwrap();
        v.center_on_player();
        assert_eq!((v.cam_x, v.cam_y), (8, -210), "居中相机钳制值");
        assert_eq!(v.terrain_at(v.row_start, v.col_start), 0, "首格 v=0（跳过）");
        let v1 = v.terrain_at(v.row_start, v.col_start + 1);
        assert_eq!(v1, 2, "第二格 v=2");
        assert_eq!(((v1 & 7) << 4, (v1 >> 3) << 4), (32, 0), "sx/sy");
        let clip_x = v.cam_x + ((v.col_start + 1) << 4);
        assert_eq!(clip_x, 24, "clip(24,18)——T544 首瓦片");
        let v2 = v.terrain_at(v.row_start, v.col_start + 2);
        assert_eq!(v2, 64);
        assert_eq!((v2 >> 3) << 4, 128, "sy=128 ⇒ draw y=18-128=-110（trace 实证）");
    }
}
