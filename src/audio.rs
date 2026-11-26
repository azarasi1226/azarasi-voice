//! マイク録音。録音中だけマイクを開き、待機中は閉じておく。

use anyhow::{Context, Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use sherpa_onnx::LinearResampler;
use std::sync::{Arc, Mutex};

/// 音声認識モデルの入力サンプリングレート。
pub const SAMPLE_RATE: u32 = 16_000;

pub struct Recording {
    _stream: Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl Recording {
    /// 既定のマイクで録音を始める。
    pub fn start() -> Result<Self> {
        let device = cpal::default_host()
            .default_input_device()
            .context("マイクが見つからない")?;
        let supported = device.default_input_config()?;
        let sample_rate = supported.sample_rate();
        let channels = supported.channels() as usize;
        let config = supported.config();
        let buffer = Arc::new(Mutex::new(Vec::with_capacity(sample_rate as usize * 30)));

        // 開始直後に WASAPI が「データの途切れ」を報告してくるが、録音には影響しないので無視する
        let on_err = |e: cpal::Error| {
            if e.kind() != cpal::ErrorKind::Xrun {
                eprintln!("[audio] エラー: {e}");
            }
        };
        let stream = match supported.sample_format() {
            SampleFormat::F32 => {
                let buf = buffer.clone();
                device.build_input_stream(
                    config,
                    move |data: &[f32], _: &_| push_mono(&buf, data, channels, |s| s),
                    on_err,
                    None,
                )?
            }
            SampleFormat::I16 => {
                let buf = buffer.clone();
                device.build_input_stream(
                    config,
                    move |data: &[i16], _: &_| {
                        push_mono(&buf, data, channels, |s| s as f32 / i16::MAX as f32)
                    },
                    on_err,
                    None,
                )?
            }
            other => return Err(anyhow!("未対応のサンプル形式: {other:?}")),
        };
        stream.play()?;

        Ok(Self { _stream: stream, buffer, sample_rate })
    }

    /// 録音を止めて、16kHz モノラルの音声を返す。
    pub fn finish(self) -> Vec<f32> {
        let sample_rate = self.sample_rate;
        let buffer = self.buffer.clone();
        drop(self); // ストリームを閉じる
        let samples = std::mem::take(&mut *buffer.lock().unwrap());
        if sample_rate == SAMPLE_RATE {
            return samples;
        }
        match LinearResampler::create(sample_rate as i32, SAMPLE_RATE as i32) {
            Some(r) => r.resample(&samples, true),
            None => {
                eprintln!("[audio] {sample_rate}Hz から変換できない");
                Vec::new()
            }
        }
    }
}

/// 複数チャンネルを平均してモノラルにして貯める。
fn push_mono<T: Copy>(buf: &Mutex<Vec<f32>>, data: &[T], channels: usize, to_f32: impl Fn(T) -> f32) {
    let mut buf = buf.lock().unwrap();
    for frame in data.chunks(channels) {
        let sum: f32 = frame.iter().map(|&s| to_f32(s)).sum();
        buf.push(sum / channels as f32);
    }
}
