//! WSL から cargo-xwin で Windows 向けにビルドするときの回避策。
//!
//! sherpa-onnx に同梱の ONNX Runtime は `PathCch.lib` をリンクしようとするが、
//! xwin が展開する Windows SDK には `pathcch.lib` / `PATHCCH.lib` しかない。
//! Windows ならファイル名の大文字小文字は区別されないが Linux では区別されるため、
//! OUT_DIR に `PathCch.lib` という名前のシンボリックリンクを作ってリンク先に追加する。

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=XWIN_CACHE_DIR");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" || cfg!(windows) {
        return;
    }

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "aarch64",
        _ => "x86_64",
    };
    let cache = env::var_os("XWIN_CACHE_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache/cargo-xwin")))
        .map(|c| c.join("xwin"));
    let Some(cache) = cache else { return };

    let src = cache.join("sdk/lib/um").join(arch).join("pathcch.lib");
    if !src.exists() {
        return;
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("xwin-case-fix");
    std::fs::create_dir_all(&out).unwrap();
    let link = out.join("PathCch.lib");
    let _ = std::fs::remove_file(&link);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&src, &link).unwrap();
    println!("cargo:rustc-link-search=native={}", out.display());
}
