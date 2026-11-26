//! 音声認識ワーカー。
//!
//! モデルは「録音が始まった瞬間」に読み込み始め（話している間に終わる）、
//! `unload_after` の間使われなければメモリから外す。

use sherpa_onnx::{OfflineNemoEncDecCtcModelConfig, OfflineRecognizer, OfflineRecognizerConfig};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

pub struct AsrConfig {
    pub model_dir: PathBuf,
    /// None ならモデルを外さない。
    pub unload_after: Option<Duration>,
}

pub enum Job {
    /// 設定が変わった。
    Reconfigure { unload_after: Option<Duration> },
    /// 録音開始。まだならモデルを読み込んでおく。
    Prepare,
    /// モデルを外す（モデルのファイルを入れ替える前に使う）。
    Unload,
    /// 録音した音声を文字にして、結果を `on_text` に渡す（認識できなければ空文字）。
    Transcribe { samples: Vec<f32>, sample_rate: u32 },
}

pub fn run(mut config: AsrConfig, jobs: Receiver<Job>, mut on_text: impl FnMut(String)) {
    let mut recognizer: Option<OfflineRecognizer> = None;

    loop {
        let job = match (recognizer.is_some(), config.unload_after) {
            (true, Some(idle)) => match jobs.recv_timeout(idle) {
                Ok(job) => job,
                Err(RecvTimeoutError::Timeout) => {
                    recognizer = None;
                    println!("[asr] {} 秒使われなかったのでモデルを解放", idle.as_secs());
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => return,
            },
            _ => match jobs.recv() {
                Ok(job) => job,
                Err(_) => return,
            },
        };

        if let Job::Reconfigure { unload_after } = job {
            config.unload_after = unload_after;
            continue;
        }
        if let Job::Unload = job {
            recognizer = None;
            continue;
        }

        if recognizer.is_none() {
            recognizer = load(&config);
        }
        let Some(rec) = recognizer.as_ref() else {
            if matches!(job, Job::Transcribe { .. }) {
                on_text(String::new());
            }
            continue;
        };

        if let Job::Transcribe {
            samples,
            sample_rate,
        } = job
        {
            let t = Instant::now();
            let stream = rec.create_stream();
            stream.accept_waveform(sample_rate as i32, &samples);
            rec.decode(&stream);
            let text = stream.get_result().map(|r| r.text).unwrap_or_default();
            let audio_sec = samples.len() as f64 / sample_rate as f64;
            println!(
                "[asr] {audio_sec:.1} 秒の音声を {:.2} 秒で認識: {text}",
                t.elapsed().as_secs_f64()
            );
            on_text(text.trim().to_string());
        }
    }
}

/// 音声認識に使うスレッド数。論理コア数の 1/4 を 1〜4 に収める。
/// ゲームなどの CPU を奪いにくくするため控えめにし、4 より増やしても速くならないので上限を 4 にしている。
fn auto_threads() -> i32 {
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    (cores / 4).clamp(1, 4) as i32
}

fn load(config: &AsrConfig) -> Option<OfflineRecognizer> {
    let dir = &config.model_dir;
    let mut rc = OfflineRecognizerConfig::default();
    rc.model_config.nemo_ctc = OfflineNemoEncDecCtcModelConfig {
        model: Some(dir.join("model.int8.onnx").to_string_lossy().into_owned()),
    };
    rc.model_config.tokens = Some(dir.join("tokens.txt").to_string_lossy().into_owned());
    let threads = auto_threads();
    rc.model_config.num_threads = threads;

    let t = Instant::now();
    let rec = OfflineRecognizer::create(&rc);
    match &rec {
        Some(_) => println!(
            "[asr] モデル読み込み: {:.2} 秒（{threads} スレッド）",
            t.elapsed().as_secs_f64()
        ),
        None => eprintln!("[asr] モデルを読み込めない: {}", dir.display()),
    }
    rec
}
