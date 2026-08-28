fn main() {
    let mut attributes = tauri_build::Attributes::new();
    #[cfg(windows)]
    {
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        embed_manifest_for_tests();
    }
    tauri_build::try_build(attributes).unwrap();
}

// Workaround for https://github.com/tauri-apps/tauri/issues/13419
// Embed the Windows app manifest into every artifact (including test binaries),
// otherwise `cargo test` fails with STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139).
#[cfg(windows)]
fn embed_manifest_for_tests() {
    static WINDOWS_MANIFEST_FILE: &str = "windows-app-manifest.xml";

    let manifest = std::env::current_dir().unwrap().join(WINDOWS_MANIFEST_FILE);

    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
        manifest.to_str().unwrap()
    );
}
