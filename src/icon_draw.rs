// アイコンの描画（角丸の四角に、正面を向いた白いあざらしの顔）。
//
// 外部の画像ファイルやライブラリを使わず、距離関数で描いてアンチエイリアスする。
// トレイ・設定画面（実行時）と exe のアイコン（build.rs から include!）の両方で使う。

/// 待機中（紺）
pub const COLOR_IDLE: [u8; 3] = [0x2B, 0x4C, 0x7E];
/// 録音中（赤）
pub const COLOR_RECORDING: [u8; 3] = [0xD9, 0x3B, 0x3B];

const WHITE: [f32; 3] = [255.0, 255.0, 255.0];
const DARK: [f32; 3] = [0x22 as f32, 0x26 as f32, 0x30 as f32];
const BLUSH: [f32; 3] = [0xF4 as f32, 0xA6 as f32, 0xB4 as f32];
const SHADE: [f32; 3] = [0xDD as f32, 0xE4 as f32, 0xEC as f32];

type P = (f32, f32);

fn length(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

fn sd_circle(p: P, c: P, r: f32) -> f32 {
    length(p.0 - c.0, p.1 - c.1) - r
}

/// 楕円までのおおよその符号付き距離（アイコンの大きさなら十分）。
fn sd_ellipse(p: P, c: P, rx: f32, ry: f32) -> f32 {
    let (x, y) = ((p.0 - c.0) / rx, (p.1 - c.1) / ry);
    (length(x, y) - 1.0) * rx.min(ry)
}

/// 点 p から線分 a-b までの距離。
fn dist_segment(p: P, a: P, b: P) -> f32 {
    let (pax, pay) = (p.0 - a.0, p.1 - a.1);
    let (bax, bay) = (b.0 - a.0, b.1 - a.1);
    let h = ((pax * bax + pay * bay) / (bax * bax + bay * bay)).clamp(0.0, 1.0);
    length(pax - bax * h, pay - bay * h)
}

/// 角丸の四角（中心 0.5, 0.5、半径 r の角）までの符号付き距離。内側が負。
fn sd_round_rect(p: P, half: f32, r: f32) -> f32 {
    let qx = (p.0 - 0.5).abs() - (half - r);
    let qy = (p.1 - 0.5).abs() - (half - r);
    length(qx.max(0.0), qy.max(0.0)) + qx.max(qy).min(0.0) - r
}

/// 下半分だけの円弧（「ω」の口に使う）。
fn sd_lower_arc(p: P, c: P, r: f32, stroke: f32) -> f32 {
    if p.1 >= c.1 {
        (length(p.0 - c.0, p.1 - c.1) - r).abs() - stroke / 2.0
    } else {
        sd_circle(p, (c.0 - r, c.1), stroke / 2.0).min(sd_circle(p, (c.0 + r, c.1), stroke / 2.0))
    }
}

/// size×size の RGBA（ストレートアルファ）を描く。
pub fn render_icon(size: u32, bg: [u8; 3]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let px = 1.0 / size as f32;
    // 小さいサイズでは細部が 1px を切って消えるので、目と鼻を大きめにする
    let small = size <= 24;
    let detailed = size >= 48;
    let eye_r = if small { 0.075 } else { 0.058 };
    let eye_x = if small { 0.18 } else { 0.15 };
    let nose_c = if small { (0.5, 0.67) } else { (0.5, 0.635) };
    let nose = if small { (0.07, 0.05) } else { (0.05, 0.036) };
    let stroke = 0.022f32.max(0.9 * px);

    for y in 0..size {
        for x in 0..size {
            let p = ((x as f32 + 0.5) * px, (y as f32 + 0.5) * px);
            let cover = |d: f32| (0.5 - d / px).clamp(0.0, 1.0);

            let bg_a = cover(sd_round_rect(p, 0.5, 0.22));
            let mut c = [bg[0] as f32, bg[1] as f32, bg[2] as f32];
            let mut paint = |color: [f32; 3], a: f32| {
                for k in 0..3 {
                    c[k] = c[k] * (1.0 - a) + color[k] * a;
                }
            };

            // 顔（下がはみ出して、四角の縁で切れる）
            let head = sd_ellipse(p, (0.5, 0.60), 0.40, 0.36);
            paint(WHITE, cover(head));
            // 顔の下側にうっすら影をつけて丸みを出す（大きいサイズだけ）
            if !small {
                let shade = head.max(-sd_ellipse(p, (0.5, 0.55), 0.40, 0.34));
                paint(SHADE, cover(shade));
            }
            // ほっぺ
            if !small {
                paint(BLUSH, cover(sd_ellipse(p, (0.26, 0.72), 0.065, 0.04)) * 0.8);
                paint(BLUSH, cover(sd_ellipse(p, (0.74, 0.72), 0.065, 0.04)) * 0.8);
            }
            // 目
            for ex in [0.5 - eye_x, 0.5 + eye_x] {
                paint(DARK, cover(sd_circle(p, (ex, 0.53), eye_r)));
                if !small {
                    paint(WHITE, cover(sd_circle(p, (ex + 0.018, 0.51), eye_r * 0.32)));
                }
            }
            // 鼻
            paint(DARK, cover(sd_ellipse(p, nose_c, nose.0, nose.1)));
            // 口（ω）
            if !small {
                let mouth = sd_lower_arc(p, (0.465, 0.675), 0.035, stroke)
                    .min(sd_lower_arc(p, (0.535, 0.675), 0.035, stroke));
                paint(DARK, cover(mouth));
            }
            // ひげ（小さいサイズではノイズになるので描かない）
            if detailed {
                let mut whisker = f32::MAX;
                for (y0, y1) in [(0.635, 0.605), (0.665, 0.67)] {
                    whisker = whisker
                        .min(dist_segment(p, (0.34, y0), (0.19, y1)))
                        .min(dist_segment(p, (0.66, y0), (0.81, y1)));
                }
                paint(DARK, cover(whisker - stroke * 0.35) * 0.45);
            }

            let to_u8 = |v: f32| v.round().clamp(0.0, 255.0) as u8;
            rgba.extend_from_slice(&[to_u8(c[0]), to_u8(c[1]), to_u8(c[2]), to_u8(bg_a * 255.0)]);
        }
    }
    rgba
}
