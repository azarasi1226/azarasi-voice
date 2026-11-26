//! 音声認識モデルの置き場所と、初回起動時のダウンロード。
//!
//! モデルはこのリポジトリの GitHub Release（model-parakeet-ja-int8-v1）に置いてあり、
//! ファイルごとに大きさと SHA-256 をここに固定している。知らないうちにモデルが変わることはない。
//! ダウンロードには Windows 標準の curl.exe を使う（通信ライブラリを exe に入れずに済む）。

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const BASE_URL: &str = "https://github.com/azarasi1226/azarasi-voice/releases/download/model-parakeet-ja-int8-v1";

pub struct ModelFile {
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

pub const FILES: &[ModelFile] = &[
    ModelFile {
        name: "tokens.txt",
        size: 28_557,
        sha256: "732f64c53909f2620c713f4106b487d92e6f54a6915b3cd3d1dbd32f9f4f392a",
    },
    ModelFile {
        name: "model.int8.onnx",
        size: 655_542_604,
        sha256: "3addd00ef5bd1742078389e540b77394e4a508bdf2f4c9ad1b4a76d93e76598e",
    },
];

pub fn total_size() -> u64 {
    FILES.iter().map(|f| f.size).sum()
}

/// 既定のモデルの置き場所（%LOCALAPPDATA%\azarasi-voice\model）。exe の置き場所には依存しない。
/// 環境変数 AZARASI_VOICE_MODEL_DIR で差し替えられる（初回セットアップの動作確認用）。
pub fn default_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("AZARASI_VOICE_MODEL_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let local = std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA が無い")?;
    Ok(PathBuf::from(local).join("azarasi-voice").join("model"))
}

/// モデルが揃っているか。起動のたびに 600MB のハッシュを計算すると遅いので、大きさだけ見る。
/// ハッシュはダウンロード直後に確認している。
pub fn is_ready(dir: &Path) -> bool {
    FILES
        .iter()
        .all(|f| std::fs::metadata(dir.join(f.name)).is_ok_and(|m| m.len() == f.size))
}

/// モデルをダウンロードする。`on_progress(済んだバイト数, 合計)` で進み具合を伝える。
/// `cancel` が立ったら途中でやめる。
pub fn download(dir: &Path, cancel: &AtomicBool, mut on_progress: impl FnMut(u64, u64)) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let total = total_size();
    let mut done_before = 0;

    for file in FILES {
        let dest = dir.join(file.name);
        if std::fs::metadata(&dest).is_ok_and(|m| m.len() == file.size) {
            done_before += file.size;
            on_progress(done_before, total);
            continue;
        }

        let part = dir.join(format!("{}.part", file.name));
        let _ = std::fs::remove_file(&part);
        let curl = Path::new(&std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()))
            .join("System32")
            .join("curl.exe");
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut child = Command::new(&curl)
            .args(["--location", "--fail", "--silent", "--show-error", "--retry", "3", "--output"])
            .arg(&part)
            .arg(format!("{BASE_URL}/{}", file.name))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .with_context(|| format!("{} を起動できない", curl.display()))?;

        // curl が書き込んでいるファイルの大きさを見て進み具合を出す
        let status = loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                let _ = std::fs::remove_file(&part);
                bail!("キャンセルした");
            }
            if let Some(status) = child.try_wait()? {
                break status;
            }
            let got = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
            on_progress(done_before + got.min(file.size), total);
            std::thread::sleep(Duration::from_millis(200));
        };
        if !status.success() {
            let mut err = String::new();
            if let Some(mut e) = child.stderr.take() {
                let _ = e.read_to_string(&mut err);
            }
            let _ = std::fs::remove_file(&part);
            bail!("ダウンロードに失敗した: {}", err.trim());
        }

        let hash = sha256_file(&part)?;
        if hash != file.sha256 {
            let _ = std::fs::remove_file(&part);
            bail!("{} の中身が一致しない（壊れているか、別のファイル）", file.name);
        }
        std::fs::rename(&part, &dest)?;
        done_before += file.size;
        on_progress(done_before, total);
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}
