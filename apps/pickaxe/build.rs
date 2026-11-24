//! Windows only: embed the app icon and version info (VERSIONINFO) into `pickaxe.exe`.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `PICKAXE_REQUIRE_WINRES=1` (set by
//! the release workflow) turns it into an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/pickaxe.ico");
    println!("cargo:rerun-if-env-changed=PICKAXE_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/pickaxe.ico")
        .set("ProductName", "Pickaxe")
        .set("FileDescription", "Pickaxe: find out how any software works")
        .set("LegalCopyright", "Pickaxe. MIT OR Apache-2.0 (see LICENSE-MIT, LICENSE-APACHE, NOTICE).")
        .set("OriginalFilename", "pickaxe.exe")
        .set("InternalName", "pickaxe");
    if let Err(e) = res.compile() {
        if std::env::var_os("PICKAXE_REQUIRE_WINRES").is_some() {
            println!("cargo::error=embedding Windows resources failed: {e}");
            return;
        }
        println!("cargo:warning=pickaxe.exe built without icon/version resources: {e}");
    }
}
