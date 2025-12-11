fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let icon_path = std::path::Path::new(&manifest_dir).join("assets").join("icon.ico");
        res.set_icon(icon_path.to_str().unwrap());
        
        // Keep console window open for debug builds
        #[cfg(debug_assertions)]
        {
            // Console subsystem is already the default for debug builds
        }
        
        res.compile().unwrap();
    }
}
