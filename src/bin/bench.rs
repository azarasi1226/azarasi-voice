//! 音声認識の速度とメモリを測るための試作。
//!
//! cargo run --release --bin bench -- <モデルのディレクトリ> <スレッド数> <wav>...

use sherpa_onnx::{
    OfflineNemoEncDecCtcModelConfig, OfflineRecognizer, OfflineRecognizerConfig, Wave,
};
use std::time::Instant;

/// 現在の常駐メモリ（Windows はワーキングセット）を MB で返す。
#[cfg(windows)]
fn rss_mb() -> Option<f64> {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut c: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
    c.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    (ok != 0).then(|| c.WorkingSetSize as f64 / 1024.0 / 1024.0)
}

/// 現在の常駐メモリ（RSS）を MB で返す。
#[cfg(not(windows))]
fn rss_mb() -> Option<f64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024.0)
}

fn print_rss(label: &str) {
    if let Some(mb) = rss_mb() {
        println!("[mem] {label}: {mb:.0} MB");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: bench <model_dir> <threads> <wav>...");
        std::process::exit(2);
    }
    let model_dir = &args[1];
    let threads: i32 = args[2].parse().expect("threads");
    let wavs = &args[3..];

    print_rss("起動直後");

    let mut config = OfflineRecognizerConfig::default();
    config.model_config.nemo_ctc = OfflineNemoEncDecCtcModelConfig {
        model: Some(format!("{model_dir}/model.int8.onnx")),
    };
    config.model_config.tokens = Some(format!("{model_dir}/tokens.txt"));
    config.model_config.num_threads = threads;

    let t = Instant::now();
    let recognizer = OfflineRecognizer::create(&config).expect("create recognizer");
    println!("[load] モデル読み込み: {:.2} 秒", t.elapsed().as_secs_f64());
    print_rss("読み込み後");

    for path in wavs {
        let wave = Wave::read(path).expect("read wav");
        let audio_sec = wave.num_samples() as f64 / wave.sample_rate() as f64;
        for run in 1..=3 {
            let t = Instant::now();
            let stream = recognizer.create_stream();
            stream.accept_waveform(wave.sample_rate(), wave.samples());
            recognizer.decode(&stream);
            let text = stream.get_result().map(|r| r.text).unwrap_or_default();
            let sec = t.elapsed().as_secs_f64();
            println!(
                "[asr] {path} ({audio_sec:.1} 秒の音声) {run}回目: {sec:.2} 秒 (音声長の {:.0}%)",
                sec / audio_sec * 100.0
            );
            if run == 1 {
                println!("      → {text}");
            }
        }
    }
    print_rss("認識後");

    drop(recognizer);
    print_rss("モデル解放後");
}
