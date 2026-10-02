fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/favicon.ico");
        res.set("FileDescription", "KeyMouse Fluent Design");
        res.set("ProductName", "KeyMouse");
        res.set("OriginalFilename", "keymouse.exe");
        res.set("LegalCopyright", "Copyright (c) 2026 PhatMT");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile windows resource: {}", e);
        }
    }
}
