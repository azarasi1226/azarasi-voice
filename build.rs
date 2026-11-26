//! Windows 向けビルドの前処理。
//!
//! 1. exe のアイコン：src/icon_draw.rs で描いた画像から .ico を作り、llvm-rc でリソースにして埋め込む。
//! 2. WSL から cargo-xwin でビルドするときの回避策：
//!    sherpa-onnx に同梱の ONNX Runtime は `PathCch.lib` をリンクしようとするが、
//!    xwin が展開する Windows SDK には `pathcch.lib` / `PATHCCH.lib` しかない。
//!    Windows ならファイル名の大文字小文字は区別されないが Linux では区別されるため、
//!    OUT_DIR に `PathCch.lib` という名前のシンボリックリンクを作ってリンク先に追加する。

#[allow(dead_code)]
mod icon_draw {
    include!("src/icon_draw.rs");
}

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/icon_draw.rs");
    println!("cargo:rerun-if-env-changed=XWIN_CACHE_DIR");

    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    embed_icon(&out_dir);
    if !cfg!(windows) {
        fix_pathcch_case(&out_dir);
    }
}

/// .ico を作ってリソースとして voice-input.exe に埋め込む。
fn embed_icon(out_dir: &Path) {
    let ico = out_dir.join("voice-input.ico");
    std::fs::write(&ico, build_ico(&[16, 24, 32, 48, 64, 256])).unwrap();

    let rc = out_dir.join("voice-input.rc");
    let ico_str = ico.display().to_string().replace('\\', "\\\\");
    std::fs::write(&rc, format!("1 ICON \"{ico_str}\"\n")).unwrap();

    let res = out_dir.join("voice-input.res");
    let rc_tool = if cfg!(windows) { "rc.exe" } else { "llvm-rc" };
    let status = Command::new(rc_tool)
        .arg("/FO")
        .arg(&res)
        .arg(&rc)
        .status()
        .unwrap_or_else(|e| panic!("{rc_tool} を実行できない（sudo apt install llvm）: {e}"));
    assert!(status.success(), "{rc_tool} が失敗した");

    // lld-link / link.exe はどちらも .res をそのまま入力に取れる
    println!("cargo:rustc-link-arg-bin=voice-input={}", res.display());
}

/// 32bpp の BMP 形式で複数サイズを入れた .ico を作る。
fn build_ico(sizes: &[u32]) -> Vec<u8> {
    let images: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&size| {
            let rgba = icon_draw::render_icon(size, icon_draw::COLOR_IDLE);
            let mut dib = Vec::new();
            // BITMAPINFOHEADER（高さは XOR + AND マスクの分で 2 倍）
            for v in [40u32, size, size * 2] {
                dib.extend_from_slice(&v.to_le_bytes());
            }
            dib.extend_from_slice(&1u16.to_le_bytes()); // planes
            dib.extend_from_slice(&32u16.to_le_bytes()); // bpp
            dib.extend_from_slice(&[0u8; 24]); // 圧縮なし、その他 0
            // ピクセル（下から上、BGRA）
            for y in (0..size).rev() {
                for x in 0..size {
                    let i = ((y * size + x) * 4) as usize;
                    dib.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
                }
            }
            // AND マスク（アルファを使うので全部 0。各行 4 バイト境界）
            let mask_row = (size.div_ceil(32) * 4) as usize;
            dib.extend(std::iter::repeat_n(0u8, mask_row * size as usize));
            dib
        })
        .collect();

    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0]); // reserved, type = icon
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (&size, img) in sizes.iter().zip(&images) {
        let dim = if size >= 256 { 0 } else { size as u8 }; // 256 は 0 で表す
        ico.extend_from_slice(&[dim, dim, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(img.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += img.len() as u32;
    }
    for img in images {
        ico.extend_from_slice(&img);
    }
    ico
}

fn fix_pathcch_case(out_dir: &Path) {
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

    let out = out_dir.join("xwin-case-fix");
    std::fs::create_dir_all(&out).unwrap();
    let link = out.join("PathCch.lib");
    let _ = std::fs::remove_file(&link);
    #[cfg(unix)]
    std::os::unix::fs::symlink(&src, &link).unwrap();
    println!("cargo:rustc-link-search=native={}", out.display());
}
