//! 設定画面（`azarasi-voice.exe --settings` で起動する別プロセス）。
//!
//! 閉じたらプロセスごと終了するので、常駐中の本体のメモリは増えない。
//! GPU を使わないよう、Slint のソフトウェアレンダラー（CPU 描画）だけを有効にしている。

use crate::autostart;
use crate::config::Config;
use crate::icon_draw;
use anyhow::Result;
use slint::ComponentHandle;

slint::slint! {
    #[edition = "1.19"]
    import { Button, CheckBox, ComboBox, GroupBox, TabWidget, AboutSlint } from "std-widgets.slint";

    export component SettingsWindow inherits Window {
        title: "azarasi-voice 設定";
        preferred-width: 460px;
        default-font-family: "Yu Gothic UI";
        default-font-size: 14px;

        in-out property <bool> sound;
        in-out property <int> unload-index;
        in-out property <bool> autostart;
        in property <image> app-icon;
        icon: app-icon;
        in-out property <string> status;
        callback save();
        callback close-clicked();

        VerticalLayout {
            padding: 12px;
            spacing: 12px;

            TabWidget {
                Tab {
                    title: "設定";
                    VerticalLayout {
                        padding: 8px;
                        spacing: 12px;
                        alignment: start;

                        GroupBox {
                            title: "音声認識";
                            VerticalLayout {
                                spacing: 6px;
                                HorizontalLayout {
                                    spacing: 8px;
                                    Text { text: "メモリを解放するまで（分）"; vertical-alignment: center; horizontal-stretch: 1; }
                                    ComboBox {
                                        model: ["1 分", "2 分", "3 分", "5 分", "8 分", "13 分", "21 分", "34 分", "55 分", "解放しない"];
                                        current-index <=> root.unload-index;
                                    }
                                }
                                Text {
                                    text: "最後に使ってからこの時間が経つと、音声認識モデル（約 700 MB）をメモリから解放します。次に使うときは約 1 秒で読み込み直します。";
                                    wrap: word-wrap;
                                    color: #999;
                                    font-size: 12px;
                                }
                            }
                        }

                        GroupBox {
                            title: "その他";
                            VerticalLayout {
                                spacing: 4px;
                                CheckBox { text: "Windows の起動時に自動で起動する"; checked <=> root.autostart; }
                                CheckBox { text: "録音の開始・終了で音を鳴らす"; checked <=> root.sound; }
                            }
                        }

                        Text {
                            text: "ホットキー：Win + 左 Ctrl を押している間だけ録音";
                            color: #888;
                        }
                    }
                }
                Tab {
                    title: "このアプリについて";
                    VerticalLayout {
                        padding: 8px;
                        alignment: start;
                        Text { text: "azarasi-voice — ローカル音声認識（Parakeet 日本語版）による音声入力"; wrap: word-wrap; }
                        AboutSlint { }
                    }
                }
            }

            HorizontalLayout {
                spacing: 8px;
                Text { text: root.status; vertical-alignment: center; horizontal-stretch: 1; }
                Button { text: "閉じる"; clicked => { root.close-clicked(); } }
                Button { text: "保存"; primary: true; clicked => { root.save(); } }
            }
        }
    }
}

/// 「メモリを解放するまで」の選択肢（分）。画面の ComboBox の並びと揃える。0 は「解放しない」。
const UNLOAD_MINUTES: [u32; 10] = [1, 2, 3, 5, 8, 13, 21, 34, 55, 0];

fn unload_minutes_to_index(minutes: u32) -> i32 {
    let i = UNLOAD_MINUTES.iter().position(|&m| m == minutes);
    // 一覧にない値（手で書き換えた設定ファイルなど）は既定の 5 分として表示する
    i.or_else(|| UNLOAD_MINUTES.iter().position(|&m| m == 5))
        .unwrap_or(0) as i32
}

fn index_to_unload_minutes(index: i32) -> u32 {
    UNLOAD_MINUTES.get(index as usize).copied().unwrap_or(5)
}

pub fn run() -> Result<()> {
    let config = Config::load();
    let ui = SettingsWindow::new()?;
    ui.set_sound(config.sound);
    ui.set_unload_index(unload_minutes_to_index(config.unload_after_minutes));
    ui.set_autostart(autostart::is_enabled());
    const ICON_SIZE: u32 = 64;
    let pixels = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
        &icon_draw::render_icon(ICON_SIZE, icon_draw::COLOR_IDLE),
        ICON_SIZE,
        ICON_SIZE,
    );
    ui.set_app_icon(slint::Image::from_rgba8(pixels));

    let weak = ui.as_weak();
    ui.on_save(move || {
        let Some(ui) = weak.upgrade() else { return };
        let new = Config {
            sound: ui.get_sound(),
            unload_after_minutes: index_to_unload_minutes(ui.get_unload_index()),
        };
        let result = new
            .save()
            .and_then(|()| autostart::set_enabled(ui.get_autostart()));
        match result {
            Ok(()) => ui.set_status("保存した".into()),
            Err(e) => ui.set_status(format!("保存に失敗: {e}").into()),
        }
    });

    let weak = ui.as_weak();
    ui.on_close_clicked(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = ui.hide();
        }
    });

    ui.run()?;
    Ok(())
}
