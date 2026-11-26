//! 設定画面（`azarasi-voice.exe --settings` で起動する別プロセス）。
//!
//! 閉じたらプロセスごと終了するので、常駐中の本体のメモリは増えない。
//! GPU を使わないよう、Slint のソフトウェアレンダラー（CPU 描画）だけを有効にしている。

use crate::autostart;
use crate::config::{Config, Formatting};
use crate::icon_draw;
use anyhow::Result;
use slint::ComponentHandle;

slint::slint! {
    #[edition = "1.19"]
    import { Button, CheckBox, ComboBox, LineEdit, SpinBox, GroupBox, TabWidget, AboutSlint } from "std-widgets.slint";

    export component SettingsWindow inherits Window {
        title: "azarasi-voice 設定";
        preferred-width: 460px;
        default-font-family: "Yu Gothic UI";
        default-font-size: 14px;

        in-out property <int> threads;
        in-out property <int> unload-minutes;
        in-out property <int> formatting-index;
        in-out property <string> api-key;
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
                            GridLayout {
                                spacing: 8px;
                                Row {
                                    Text { text: "CPU スレッド数"; vertical-alignment: center; }
                                    SpinBox { minimum: 1; maximum: 16; value <=> root.threads; }
                                }
                                Row {
                                    Text { text: "モデルを外すまで（分、0 で外さない）"; vertical-alignment: center; }
                                    SpinBox { minimum: 0; maximum: 120; value <=> root.unload-minutes; }
                                }
                            }
                        }

                        GroupBox {
                            title: "起動";
                            CheckBox { text: "Windows の起動時に自動で起動する"; checked <=> root.autostart; }
                        }

                        GroupBox {
                            title: "整形（Claude）";
                            GridLayout {
                                spacing: 8px;
                                Row {
                                    Text { text: "整形"; vertical-alignment: center; }
                                    ComboBox {
                                        model: ["オフ", "軽め（未実装）", "しっかり（未実装）"];
                                        current-index <=> root.formatting-index;
                                    }
                                }
                                Row {
                                    Text { text: "API キー"; vertical-alignment: center; }
                                    LineEdit {
                                        input-type: password;
                                        placeholder-text: "sk-ant-...";
                                        text <=> root.api-key;
                                    }
                                }
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

fn formatting_to_index(f: Formatting) -> i32 {
    match f {
        Formatting::Off => 0,
        Formatting::Light => 1,
        Formatting::Full => 2,
    }
}

fn index_to_formatting(i: i32) -> Formatting {
    match i {
        1 => Formatting::Light,
        2 => Formatting::Full,
        _ => Formatting::Off,
    }
}

pub fn run() -> Result<()> {
    let config = Config::load();
    let ui = SettingsWindow::new()?;
    ui.set_threads(config.threads as i32);
    ui.set_unload_minutes(config.unload_after_minutes as i32);
    ui.set_formatting_index(formatting_to_index(config.formatting));
    ui.set_api_key(config.anthropic_api_key.clone().into());
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
            threads: ui.get_threads().max(1) as u32,
            unload_after_minutes: ui.get_unload_minutes().max(0) as u32,
            formatting: index_to_formatting(ui.get_formatting_index()),
            anthropic_api_key: ui.get_api_key().trim().to_string(),
        };
        let result = new.save().and_then(|()| autostart::set_enabled(ui.get_autostart()));
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
