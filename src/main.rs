//! azarasi-voice 本体。Win + 左 Ctrl を押している間だけ録音し、離したら文字にして貼り付ける。
//!
//! azarasi-voice.exe [--console] [--model <dir>]
//! azarasi-voice.exe --settings   … 設定画面（トレイから起動される別プロセス）
//!
//! 通常はコンソールを出さずにタスクトレイに常駐する。`--console` でログ表示用のコンソールを開く。
//! 設定は %APPDATA%\azarasi-voice\config.toml。

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod asr;
#[cfg(windows)]
mod audio;
#[cfg(windows)]
mod autostart;
#[cfg(windows)]
mod config;
#[cfg(windows)]
mod hotkey;
#[cfg(windows)]
mod icon_draw;
#[cfg(windows)]
mod paste;
#[cfg(windows)]
mod settings;
#[cfg(windows)]
mod tray;

#[cfg(not(windows))]
fn main() {
    eprintln!("azarasi-voice は Windows 専用。`mise run build` で exe をビルドする。");
    std::process::exit(1);
}

/// 名前付きミューテックスで二重起動を防ぐ。既に起動していれば false。
#[cfg(windows)]
fn acquire_single_instance(name: &str) -> bool {
    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    // ハンドルはプロセス終了まで保持する（閉じない）
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
    !handle.is_null() && unsafe { GetLastError() } != ERROR_ALREADY_EXISTS
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use config::Config;
    use hotkey::HotkeyEvent;
    use std::path::PathBuf;
    use std::sync::mpsc;

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("--settings") {
        if !acquire_single_instance("Local\\azarasi-voice-settings") {
            return Ok(()); // 設定画面は既に開いている
        }
        return settings::run();
    }

    let exe_dir = std::env::current_exe()?.parent().map(PathBuf::from).unwrap_or_default();
    let mut model_dir = exe_dir.join("model");
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--console" => unsafe {
                windows_sys::Win32::System::Console::AllocConsole();
            },
            "--model" => model_dir = it.next().context("--model に値がない")?.into(),
            other => anyhow::bail!("不明な引数: {other}"),
        }
    }

    if !acquire_single_instance("Local\\azarasi-voice") {
        return Ok(()); // 既に常駐している
    }
    anyhow::ensure!(
        model_dir.join("model.int8.onnx").exists(),
        "モデルが見つからない: {}（--model で指定できる）",
        model_dir.display()
    );

    // 音声認識ワーカー
    let mut config = Config::load();
    let mut config_mtime = config::modified();
    let (job_tx, job_rx) = mpsc::channel();
    let asr_config = asr::AsrConfig {
        model_dir,
        threads: config.threads as i32,
        unload_after: config.unload_after(),
    };
    std::thread::spawn(move || {
        asr::run(asr_config, job_rx, |text| {
            if !text.is_empty()
                && let Err(e) = paste::paste(&text)
            {
                eprintln!("[paste] 貼り付けに失敗: {e}");
            }
            tray::notify(tray::Status::Idle);
        })
    });

    // ホットキーの押下・解放に応じて録音を開始・終了する
    let (key_tx, key_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut recording: Option<audio::Recording> = None;
        for event in key_rx {
            match event {
                HotkeyEvent::Pressed => {
                    // 設定画面で変更されていたら読み直す
                    let mtime = config::modified();
                    if mtime != config_mtime {
                        config_mtime = mtime;
                        let new = Config::load();
                        if new != config {
                            println!("[config] 設定を読み直した: threads={}, unload_after_minutes={}", new.threads, new.unload_after_minutes);
                            config = new;
                            let _ = job_tx.send(asr::Job::Reconfigure {
                                threads: config.threads as i32,
                                unload_after: config.unload_after(),
                            });
                        }
                    }

                    let _ = job_tx.send(asr::Job::Prepare);
                    match audio::Recording::start() {
                        Ok(r) => {
                            println!("[rec] 録音開始");
                            tray::notify(tray::Status::Recording);
                            recording = Some(r);
                        }
                        Err(e) => eprintln!("[rec] 録音を開始できない: {e}"),
                    }
                }
                HotkeyEvent::Released => {
                    let Some(r) = recording.take() else { continue };
                    let samples = r.finish();
                    let sample_rate = audio::SAMPLE_RATE;
                    let sec = samples.len() as f32 / sample_rate as f32;
                    println!("[rec] 録音終了 ({sec:.1} 秒)");
                    if sec < 0.3 {
                        tray::notify(tray::Status::Idle);
                        continue; // 押し間違いとみなす
                    }
                    tray::notify(tray::Status::Processing);
                    let _ = job_tx.send(asr::Job::Transcribe { samples, sample_rate });
                }
            }
        }
    });

    hotkey::install(key_tx)?;
    println!("azarasi-voice 起動。Win + 左 Ctrl を押している間だけ録音する。終了はトレイのメニューから。");
    tray::run()
}
