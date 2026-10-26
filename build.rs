fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .set("ProductName", "Anemorph")
            .set("FileDescription", "Anemorph episode renamer")
            .compile()
            .expect("Could not embed the application icon");
    }
}
