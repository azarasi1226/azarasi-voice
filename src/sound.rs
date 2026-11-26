//! 録音の開始・終了の効果音。
//!
//! 音声ファイルは持たず、起動後の初回に短い 2 音の WAV をメモリ上で作って、
//! Windows 標準の PlaySound で非同期に鳴らす。

use std::sync::OnceLock;
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};

const SAMPLE_RATE: u32 = 44_100;

/// 録音開始（低→高）
pub fn play_start() {
    static WAV: OnceLock<Vec<u8>> = OnceLock::new();
    play(WAV.get_or_init(|| two_tone_wav(660.0, 990.0)));
}

/// 録音終了（高→低）
pub fn play_stop() {
    static WAV: OnceLock<Vec<u8>> = OnceLock::new();
    play(WAV.get_or_init(|| two_tone_wav(990.0, 660.0)));
}

/// SND_ASYNC で鳴らすので、再生が終わるまで `wav` が生きている必要がある（static に置いている）。
fn play(wav: &'static [u8]) {
    unsafe {
        PlaySoundW(
            wav.as_ptr().cast(),
            std::ptr::null_mut(),
            SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
        )
    };
}

/// 2 つの短い音を続けて鳴らす 16bit モノラルの WAV を作る。
fn two_tone_wav(first_hz: f32, second_hz: f32) -> Vec<u8> {
    let mut samples = Vec::new();
    for hz in [first_hz, second_hz] {
        samples.extend(tone(hz, 0.06));
        samples.extend(std::iter::repeat_n(
            0i16,
            (SAMPLE_RATE as f32 * 0.015) as usize,
        ));
    }

    let data_len = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // fmt チャンクの大きさ
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // モノラル
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // 1 秒あたりのバイト数
    wav.extend_from_slice(&2u16.to_le_bytes()); // 1 サンプルのバイト数
    wav.extend_from_slice(&16u16.to_le_bytes()); // ビット数
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    wav
}

/// 立ち上がりを数 ms でなめらかにし、指数的に減衰させた正弦波（「ポッ」という柔らかい音）。
fn tone(hz: f32, seconds: f32) -> impl Iterator<Item = i16> {
    let n = (SAMPLE_RATE as f32 * seconds) as usize;
    (0..n).map(move |i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let attack = (t / 0.004).min(1.0);
        let decay = (-t * 40.0).exp();
        let v = (2.0 * std::f32::consts::PI * hz * t).sin() * attack * decay * 0.3;
        (v * i16::MAX as f32) as i16
    })
}
