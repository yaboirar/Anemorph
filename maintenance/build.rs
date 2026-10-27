fn main() {
    println!("cargo:rerun-if-env-changed=ANEMORPH_INTEGRATION_CERT");
    let bytes = match std::env::var_os("ANEMORPH_INTEGRATION_CERT") {
        Some(path) => {
            println!(
                "cargo:rerun-if-changed={}",
                std::path::Path::new(&path).display()
            );
            std::fs::read(path).expect("Cannot read public integration certificate")
        }
        None => Vec::new(),
    };
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("integration.cer"),
        bytes,
    )
    .unwrap();
}
