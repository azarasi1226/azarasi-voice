// アイコンの描画（角丸の四角に白いマイク）。
//
// 外部の画像ファイルやライブラリを使わず、距離関数で描いてアンチエイリアスする。
// トレイ・設定画面（実行時）と exe のアイコン（build.rs から include!）の両方で使う。

/// 待機中（紺）
pub const COLOR_IDLE: [u8; 3] = [0x2B, 0x4C, 0x7E];
/// 録音中（赤）
pub const COLOR_RECORDING: [u8; 3] = [0xD9, 0x3B, 0x3B];
/// 認識中（オレンジ）
pub const COLOR_PROCESSING: [u8; 3] = [0xE0, 0x8A, 0x1E];

/// 点 p から線分 a-b までの距離。
fn dist_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (pax, pay) = (p.0 - a.0, p.1 - a.1);
    let (bax, bay) = (b.0 - a.0, b.1 - a.1);
    let h = ((pax * bax + pay * bay) / (bax * bax + bay * bay)).clamp(0.0, 1.0);
    ((pax - bax * h).powi(2) + (pay - bay * h).powi(2)).sqrt()
}

/// 角丸の四角（中心 0.5, 0.5、半径 r の角）までの符号付き距離。内側が負。
fn sd_round_rect(p: (f32, f32), half: f32, r: f32) -> f32 {
    let qx = (p.0 - 0.5).abs() - (half - r);
    let qy = (p.1 - 0.5).abs() - (half - r);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - r
}

/// マイクの形までの符号付き距離（座標は 0..1）。stroke は線の太さ。
fn sd_mic(p: (f32, f32), stroke: f32) -> f32 {
    // 本体（カプセル）
    let body = dist_segment(p, (0.5, 0.30), (0.5, 0.44)) - 0.12;
    // 本体を受ける U 字のアーム（中心 (0.5, 0.44)、半径 0.22 の円の下半分）
    let (dx, dy) = (p.0 - 0.5, p.1 - 0.44);
    let ring = ((dx * dx + dy * dy).sqrt() - 0.22).abs() - stroke / 2.0;
    let arm = if dy >= 0.0 {
        ring
    } else {
        // 上半分は U 字の両端だけ（丸く止める）
        let end_l = ((p.0 - 0.28).powi(2) + dy * dy).sqrt() - stroke / 2.0;
        let end_r = ((p.0 - 0.72).powi(2) + dy * dy).sqrt() - stroke / 2.0;
        end_l.min(end_r)
    };
    // 支柱と台座
    let stem = dist_segment(p, (0.5, 0.66), (0.5, 0.79)) - stroke / 2.0;
    let base = dist_segment(p, (0.37, 0.80), (0.63, 0.80)) - stroke / 2.0;
    body.min(arm).min(stem).min(base)
}

/// size×size の RGBA（ストレートアルファ）を描く。
pub fn render_icon(size: u32, bg: [u8; 3]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let px = 1.0 / size as f32;
    // 小さいサイズでは線が 1px を切ってぼやけるので太くする
    let stroke = 0.065f32.max(1.3 * px);
    for y in 0..size {
        for x in 0..size {
            let p = ((x as f32 + 0.5) * px, (y as f32 + 0.5) * px);
            let cover = |d: f32| (0.5 - d / px).clamp(0.0, 1.0);
            let bg_a = cover(sd_round_rect(p, 0.5, 0.22));
            let mic_a = cover(sd_mic(p, stroke)) * bg_a;
            let mix = |b: u8| (b as f32 * (1.0 - mic_a) + 255.0 * mic_a).round() as u8;
            rgba.extend_from_slice(&[mix(bg[0]), mix(bg[1]), mix(bg[2]), (bg_a * 255.0).round() as u8]);
        }
    }
    rgba
}
