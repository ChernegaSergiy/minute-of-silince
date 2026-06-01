fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let status = std::process::Command::new("swiftc")
            .args([
                "-parse-as-library",
                "-g",
                "-O",
                "-emit-library",
                "-static",
                "-o",
                &format!("{}/libMediaVolumeHelper.a", out_dir),
                "src/platform/macos/MediaVolumeHelper.swift",
            ])
            .status()
            .unwrap();

        if !status.success() {
            panic!("Swift compilation failed");
        }

        // Link the compiled static library
        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=MediaVolumeHelper");

        // Link Swift runtime libraries
        println!("cargo:rustc-link-search=native=/usr/lib/swift");
        println!(
            "cargo:rustc-link-search=native=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx"
        );
        println!("cargo:rustc-link-lib=dylib=swiftCore");
        println!("cargo:rustc-link-lib=dylib=swiftAppKit");
    }

    tauri_build::build();
}
