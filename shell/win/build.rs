// build.rs（shell/win）——Windows 可执行资源嵌入（2026-10-18 用户需求：exe 图标）。
// 图标源 = assets/raw/i62x62.png（原版 MIDlet-Icon，manifest: MIDlet-Icon: i62x62.png）
// 经 PIL 多尺寸化为 assets/app.ico（16/24/32/48/62）。
// 附带版本信息（资源管理器属性页）；非 Windows 平台零动作。
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set("ProductName", "囧囧西游-大闹天宫");
        res.set("FileDescription", "囧囧西游-大闹天宫（Rust 重制版）");
        res.set("LegalCopyright", "重制版 2026；原作 SNOWFISH 2010");
        if let Err(e) = res.compile() {
            eprintln!("winresource 编译失败（非致命，继续无图标构建）：{e}");
        }
    }
}
