//! 録音の開始・終了の効果音。
//!
//! 音声ファイルは持たず、起動後の初回に短い 2 音の WAV をメモリ上で作って、
//! Windows 標準の PlaySound で非同期に鳴らす。
//!
//! ワイヤレスヘッドセットなどは、無音が続くと音声の通り道を止め、音が来てから再開する。
//! 再開までの間に鳴らした音は聞こえないので、効果音の前に耳では聞こえない小さな音を流して起こしておく。
//! 完全な無音だと「音が来ていない」とみなして起きない機器があるため、無音ではなく小さな音にしている。
//! 直前に鳴らしたばかりなら機器はまだ起きているので、前置きを省いてすぐ鳴らす。

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};

const SAMPLE_RATE: u32 = 44_100;

/// 機器を起こすための前置きの長さ（秒）。
/// 復帰に時間のかかる機器（USB 受信機で接続するワイヤレスヘッドセットなど）では 0.25 秒だと足りないことがあったため、余裕を見て 0.4 秒にしている
const WAKE_SECONDS: f32 = 0.4;
/// 最後に鳴らしてからこの時間内なら、機器はまだ起きているとみなして前置きを省く
const STILL_AWAKE: Duration = Duration::from_secs(20);

/// 前置きあり・なしの 2 種類の WAV
struct Cue {
    with_wake: Vec<u8>,
    without_wake: Vec<u8>,
}

impl Cue {
    fn new(first_hz: f32, second_hz: f32) -> Self {
        Self {
            with_wake: two_tone_wav(first_hz, second_hz, true),
            without_wake: two_tone_wav(first_hz, second_hz, false),
        }
    }
}

/// 録音開始（低→高）
pub fn play_start() {
    static CUE: OnceLock<Cue> = OnceLock::new();
    play_cue(CUE.get_or_init(|| Cue::new(660.0, 990.0)));
}

/// 録音終了（高→低）
pub fn play_stop() {
    static CUE: OnceLock<Cue> = OnceLock::new();
    play_cue(CUE.get_or_init(|| Cue::new(990.0, 660.0)));
}

fn play_cue(cue: &'static Cue) {
    static LAST_PLAYED: Mutex<Option<Instant>> = Mutex::new(None);
    let mut last = LAST_PLAYED.lock().unwrap();
    let awake = last.is_some_and(|t| t.elapsed() < STILL_AWAKE);
    *last = Some(Instant::now());
    play(if awake {
        &cue.without_wake
    } else {
        &cue.with_wake
    });
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

/// 2 つの短い音を続けて鳴らす 16bit モノラルの WAV を作る。`wake` なら機器を起こす前置きを付ける。
fn two_tone_wav(first_hz: f32, second_hz: f32, wake: bool) -> Vec<u8> {
    let mut samples: Vec<i16> = if wake {
        wake_up().collect()
    } else {
        Vec::new()
    };
    for hz in [first_hz, second_hz] {
        samples.extend(tone(hz, 0.05));
        samples.extend(std::iter::repeat_n(
            0i16,
            (SAMPLE_RATE as f32 * 0.01) as usize,
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

/// 耳では聞こえない小さな低音（30Hz、約 -66dB）。
fn wake_up() -> impl Iterator<Item = i16> {
    let n = (SAMPLE_RATE as f32 * WAKE_SECONDS) as usize;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        ((2.0 * std::f32::consts::PI * 30.0 * t).sin() * 16.0) as i16
    })
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
