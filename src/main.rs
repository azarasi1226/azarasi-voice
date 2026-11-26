//! azarasi-voice 本体。Win + 左 Ctrl を押している間だけ録音し、離したら文字にして貼り付ける。
//!
//! azarasi-voice.exe [--console]
//! azarasi-voice.exe --settings   … 設定画面（トレイから起動される別プロセス）
//! azarasi-voice.exe --setup      … 初回のモデルのダウンロード画面（モデルが無いと自動で起動される）
//!
//! 通常はコンソールを出さずにタスクトレイに常駐する。`--console` でログ表示用のコンソールを開く。
//! 設定は %APPDATA%\azarasi-voice\config.toml、モデルは %LOCALAPPDATA%\azarasi-voice\model。

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
mod model;
#[cfg(windows)]
mod paste;
#[cfg(windows)]
mod settings;
#[cfg(windows)]
mod setup;
#[cfg(windows)]
mod sound;
#[cfg(windows)]
mod tray;

#[cfg(not(windows))]
fn main() {
    eprintln!("azarasi-voice は Windows 専用。`mise run build` で exe をビルドする。");
    std::process::exit(1);
}

/// モデルのハッシュを確かめ、壊れていたら消して再ダウンロードする。
#[cfg(windows)]
fn verify_model_in_background(dir: &std::path::Path, job_tx: &std::sync::mpsc::Sender<asr::Job>) {
    let started = std::time::Instant::now();
    let result = {
        let _background = model::background_mode();
        model::find_corrupt(dir)
    };
    let bad = match result {
        Ok(bad) if bad.is_empty() => {
            println!(
                "[model] ハッシュを確認した（{:.1} 秒）",
                started.elapsed().as_secs_f64()
            );
            return;
        }
        Ok(bad) => bad,
        Err(e) => {
            eprintln!("[model] ハッシュを確認できない: {e}");
            return;
        }
    };

    eprintln!(
        "[model] 壊れているファイルがある: {:?}",
        bad.iter().map(|f| f.name).collect::<Vec<_>>()
    );
    // 読み込み中のモデルを外してから消す（読み込み中のファイルは消せない）
    let _ = job_tx.send(asr::Job::Unload);
    for file in bad {
        let path = dir.join(file.name);
        for _ in 0..20 {
            if std::fs::remove_file(&path).is_ok() || !path.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).arg("--setup").status();
    }
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
    use config::Config;
    use hotkey::HotkeyEvent;
    use std::sync::mpsc;

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("--settings") {
        if !acquire_single_instance("Local\\azarasi-voice-settings") {
            return Ok(()); // 設定画面は既に開いている
        }
        return settings::run();
    }
    if args.first().map(String::as_str) == Some("--setup") {
        return setup::run();
    }

    for a in &args {
        match a.as_str() {
            "--console" => unsafe {
                windows_sys::Win32::System::Console::AllocConsole();
            },
            other => anyhow::bail!("不明な引数: {other}"),
        }
    }

    if !acquire_single_instance("Local\\azarasi-voice") {
        return Ok(()); // 既に常駐している
    }

    let model_dir = model::default_dir()?;
    if !model::is_ready(&model_dir) {
        // 初回：ダウンロード画面を別プロセスで出して、終わるのを待つ
        std::process::Command::new(std::env::current_exe()?)
            .arg("--setup")
            .status()?;
        if !model::is_ready(&model_dir) {
            return Ok(()); // キャンセルされた
        }
    }

    // 音声認識ワーカー
    let mut config = Config::load();
    let mut config_mtime = config::modified();
    let (job_tx, job_rx) = mpsc::channel();
    let asr_config = asr::AsrConfig {
        model_dir: model_dir.clone(),
        unload_after: config.unload_after(),
    };
    std::thread::spawn(move || {
        asr::run(asr_config, job_rx, |text| {
            if !text.is_empty()
                && let Err(e) = paste::paste(&text)
            {
                eprintln!("[paste] 貼り付けに失敗: {e}");
            }
        })
    });

    // 起動時に裏でモデルのハッシュを確かめる。壊れていたら消して、ダウンロード画面を出す
    {
        let job_tx = job_tx.clone();
        std::thread::spawn(move || verify_model_in_background(&model_dir, &job_tx));
    }

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
                            println!("[config] 設定を読み直した: {new:?}");
                            config = new;
                            let _ = job_tx.send(asr::Job::Reconfigure {
                                unload_after: config.unload_after(),
                            });
                        }
                    }

                    let _ = job_tx.send(asr::Job::Prepare);
                    match audio::Recording::start() {
                        Ok(r) => {
                            println!("[rec] 録音開始");
                            tray::notify(tray::Status::Recording);
                            if config.sound {
                                sound::play_start();
                            }
                            recording = Some(r);
                        }
                        Err(e) => eprintln!("[rec] 録音を開始できない: {e}"),
                    }
                }
                HotkeyEvent::Released => {
                    let Some(r) = recording.take() else { continue };
                    tray::notify(tray::Status::Idle);
                    if config.sound {
                        sound::play_stop();
                    }
                    let samples = r.finish();
                    let sample_rate = audio::SAMPLE_RATE;
                    let sec = samples.len() as f32 / sample_rate as f32;
                    println!("[rec] 録音終了 ({sec:.1} 秒)");
                    if sec < 0.3 {
                        continue; // 押し間違いとみなす
                    }
                    let _ = job_tx.send(asr::Job::Transcribe {
                        samples,
                        sample_rate,
                    });
                }
            }
        }
    });

    hotkey::install(key_tx)?;
    println!(
        "azarasi-voice 起動。Win + 左 Ctrl を押している間だけ録音する。終了はトレイのメニューから。"
    );
    tray::run()
}
