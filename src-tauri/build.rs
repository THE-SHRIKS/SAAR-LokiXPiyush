fn main() {
    tauri_build::build();

    // The end-to-end test links the Tauri mock runtime, which needs the common-controls v6
    // manifest. Only that build gets it, so the other test binaries are unaffected.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let e2e = std::env::var("CARGO_FEATURE_E2E").is_ok();
    if target_os == "windows" && e2e {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-manifest.xml");
        println!("cargo:rerun-if-changed=test-manifest.xml");
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}", manifest.display());
    }
}
