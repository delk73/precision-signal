fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_arch == "arm" && target_os == "none" {
        println!(
            "cargo:rustc-link-search={}",
            std::env::var("CARGO_MANIFEST_DIR").unwrap()
        );
        println!("cargo:rustc-link-arg=-Tlink.x");
    }
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}
