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
    pub threads: i32,
    /// None ならモデルを外さない。
    pub unload_after: Option<Duration>,
}

pub enum Job {
    /// 録音開始。まだならモデルを読み込んでおく。
    Prepare,
    /// 録音した音声を文字にして、結果を `on_text` に渡す。
    Transcribe { samples: Vec<f32>, sample_rate: u32 },
}

pub fn run(config: AsrConfig, jobs: Receiver<Job>, mut on_text: impl FnMut(String)) {
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

        if recognizer.is_none() {
            recognizer = load(&config);
        }
        let Some(rec) = recognizer.as_ref() else { continue };

        if let Job::Transcribe { samples, sample_rate } = job {
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
            let text = text.trim().to_string();
            if !text.is_empty() {
                on_text(text);
            }
        }
    }
}

fn load(config: &AsrConfig) -> Option<OfflineRecognizer> {
    let dir = &config.model_dir;
    let mut rc = OfflineRecognizerConfig::default();
    rc.model_config.nemo_ctc = OfflineNemoEncDecCtcModelConfig {
        model: Some(dir.join("model.int8.onnx").to_string_lossy().into_owned()),
    };
    rc.model_config.tokens = Some(dir.join("tokens.txt").to_string_lossy().into_owned());
    rc.model_config.num_threads = config.threads;

    let t = Instant::now();
    let rec = OfflineRecognizer::create(&rc);
    match &rec {
        Some(_) => println!("[asr] モデル読み込み: {:.2} 秒", t.elapsed().as_secs_f64()),
        None => eprintln!("[asr] モデルを読み込めない: {}", dir.display()),
    }
    rec
}
