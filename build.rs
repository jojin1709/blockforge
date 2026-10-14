fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg=/pdbpagesize:8192");
        // Embed the Windows application icon into the .exe.
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon/blockforge.ico");
        res.compile().expect("failed to embed exe icon");
    }
}
