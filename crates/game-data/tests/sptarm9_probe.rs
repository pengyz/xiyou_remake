//! 临时探针（迭代 74，用后即删）：sptarm[9]（80 银缕锁甲图标）Rust 解码
//! 输出 PPM，供与 PIL 解码对照定位颜色失真根因。
use game_data::packed_png::PackedPng;
use game_data::decode_png;

fn ppm(path: &str, w: i32, h: i32, argb: &[u32]) {
    let mut out = format!("P6\n{} {}\n255\n", w, h).into_bytes();
    for &p in argb {
        out.push(((p >> 16) & 0xFF) as u8);
        out.push(((p >> 8) & 0xFF) as u8);
        out.push(p as u8);
    }
    std::fs::write(path, out).unwrap();
}

#[test]
fn sptarm9_probe() {
    let data = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/raw/sptarm")).unwrap();
    let pack = PackedPng::parse(&data, 10).unwrap();
    // [8] = 79 乌金棍、[9] = 80 银缕锁甲（entityTypeImage 覆盖表同源）
    for (k, img) in pack.images.iter().enumerate().skip(8).take(2) {
        let d = decode_png(img.bytes).unwrap();
        println!("sub{}: {}x{} argb[0]={:#08x} argb[中心]={:#08x}", k, d.width, d.height, d.argb[0], d.argb[d.argb.len()/2 + d.width as usize/2]);
        ppm(&format!("/tmp/sptarm_rust_{}.ppm", k), d.width, d.height, &d.argb);
    }
}
