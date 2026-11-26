//! voice-input 本体。Win + 左 Ctrl を押している間だけ録音し、離したら文字にして貼り付ける。
//!
//! voice-input.exe [--model <dir>] [--threads <n>] [--unload-after <秒, 0 で外さない>]

#[cfg(windows)]
mod asr;
#[cfg(windows)]
mod audio;
#[cfg(windows)]
mod hotkey;
#[cfg(windows)]
mod paste;

#[cfg(not(windows))]
fn main() {
    eprintln!("voice-input は Windows 専用。`mise run build` で exe をビルドする。");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use hotkey::HotkeyEvent;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::time::Duration;

    // 引数
    let exe_dir = std::env::current_exe()?.parent().map(PathBuf::from).unwrap_or_default();
    let mut model_dir = exe_dir.join("model");
    let mut threads = 4;
    let mut unload_after = Some(Duration::from_secs(300));
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = || args.next().with_context(|| format!("{a} に値がない"));
        match a.as_str() {
            "--model" => model_dir = value()?.into(),
            "--threads" => threads = value()?.parse()?,
            "--unload-after" => {
                let secs: u64 = value()?.parse()?;
                unload_after = (secs > 0).then(|| Duration::from_secs(secs));
            }
            other => anyhow::bail!("不明な引数: {other}"),
        }
    }
    anyhow::ensure!(
        model_dir.join("model.int8.onnx").exists(),
        "モデルが見つからない: {}（--model で指定できる）",
        model_dir.display()
    );

    // 音声認識ワーカー
    let (job_tx, job_rx) = mpsc::channel();
    let config = asr::AsrConfig { model_dir, threads, unload_after };
    std::thread::spawn(move || {
        asr::run(config, job_rx, |text| {
            if let Err(e) = paste::paste(&text) {
                eprintln!("[paste] 貼り付けに失敗: {e}");
            }
        })
    });

    // ホットキーの押下・解放に応じて録音を開始・終了する
    let (key_tx, key_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut recording: Option<audio::Recording> = None;
        for event in key_rx {
            match event {
                HotkeyEvent::Pressed => {
                    let _ = job_tx.send(asr::Job::Prepare);
                    match audio::Recording::start() {
                        Ok(r) => {
                            println!("[rec] 録音開始");
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
                        continue; // 押し間違いとみなす
                    }
                    let _ = job_tx.send(asr::Job::Transcribe { samples, sample_rate });
                }
            }
        }
    });

    println!("voice-input 起動。Win + 左 Ctrl を押している間だけ録音する。終了は Ctrl+C。");
    hotkey::run(key_tx)
}
