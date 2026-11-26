//! 設定ファイル（%APPDATA%\azarasi-voice\config.toml）の読み書き。
//!
//! 設定画面は別プロセスなので、本体とはこのファイルを介してやりとりする。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// 音声認識に使う CPU スレッド数
    pub threads: u32,
    /// 最後に使ってからモデルを外すまでの分数。0 なら外さない。
    pub unload_after_minutes: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threads: 4,
            unload_after_minutes: 5,
        }
    }
}

impl Config {
    pub fn unload_after(&self) -> Option<Duration> {
        (self.unload_after_minutes > 0)
            .then(|| Duration::from_secs(self.unload_after_minutes as u64 * 60))
    }

    /// 設定ファイルを読む。無ければ既定値。
    pub fn load() -> Self {
        let Ok(path) = path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
                eprintln!("[config] 読めないので既定値を使う: {e}");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn dir() -> Result<PathBuf> {
    let appdata = std::env::var_os("APPDATA").context("APPDATA が無い")?;
    Ok(PathBuf::from(appdata).join("azarasi-voice"))
}

pub fn path() -> Result<PathBuf> {
    Ok(dir()?.join("config.toml"))
}

/// 設定ファイルの更新時刻。変更検知に使う。
pub fn modified() -> Option<SystemTime> {
    std::fs::metadata(path().ok()?).ok()?.modified().ok()
}
