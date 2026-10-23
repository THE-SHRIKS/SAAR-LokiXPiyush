fn main() {
    tauri_build::build();

    // Test binaries link the window-system stack through the mock runtime. Without the
    // common-controls v6 manifest they load comctl32 v5 and fail with STATUS_ENTRYPOINT_NOT_FOUND.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-manifest.xml");
        println!("cargo:rerun-if-changed=test-manifest.xml");
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}", manifest.display());
    }
}
