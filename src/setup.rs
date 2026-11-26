//! 初回セットアップ画面（`azarasi-voice.exe --setup`）。モデルのダウンロードの進み具合を表示する。
//!
//! 設定画面と同じく別プロセスにして、終わったらプロセスごと消えるようにしている。

use crate::{icon_draw, model};
use anyhow::Result;
use slint::ComponentHandle;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

slint::slint! {
    #[edition = "1.19"]
    import { Button, ProgressIndicator } from "std-widgets.slint";

    export component SetupWindow inherits Window {
        title: "azarasi-voice 初回セットアップ";
        preferred-width: 440px;
        default-font-family: "Yu Gothic UI";
        default-font-size: 14px;

        in property <image> app-icon;
        icon: app-icon;
        in property <float> progress;
        in property <string> detail;
        in property <bool> failed;
        callback retry();
        callback cancel();

        VerticalLayout {
            padding: 20px;
            spacing: 12px;

            Text {
                text: root.failed ? "ダウンロードできませんでした" : "音声認識モデルをダウンロードしています";
                font-size: 16px;
                font-weight: 600;
            }
            Text {
                text: "初回だけ必要です。終わると自動でタスクトレイに常駐します。";
                color: #999;
                wrap: word-wrap;
            }
            ProgressIndicator { progress: root.progress; }
            Text { text: root.detail; wrap: word-wrap; }

            HorizontalLayout {
                alignment: end;
                spacing: 8px;
                Button { text: "キャンセル"; clicked => { root.cancel(); } }
                if root.failed: Button { text: "再試行"; primary: true; clicked => { root.retry(); } }
            }
        }
    }
}

/// ダウンロード画面を出す。モデルが揃ったら画面を閉じて戻る。
pub fn run() -> Result<()> {
    let dir = model::default_dir()?;
    let ui = SetupWindow::new()?;
    const ICON_SIZE: u32 = 64;
    ui.set_app_icon(slint::Image::from_rgba8(
        slint::SharedPixelBuffer::clone_from_slice(
            &icon_draw::render_icon(ICON_SIZE, icon_draw::COLOR_IDLE),
            ICON_SIZE,
            ICON_SIZE,
        ),
    ));

    let cancel = Arc::new(AtomicBool::new(false));
    let start = {
        let weak = ui.as_weak();
        let cancel = cancel.clone();
        let dir = dir.clone();
        move || {
            let weak = weak.clone();
            let cancel = cancel.clone();
            let dir = dir.clone();
            if let Some(ui) = weak.upgrade() {
                ui.set_failed(false);
                ui.set_detail("準備中…".into());
            }
            std::thread::spawn(move || {
                let progress_ui = weak.clone();
                let result = model::download(&dir, &cancel, |done, total| {
                    let mb = |b: u64| b as f64 / 1_000_000.0;
                    let detail = format!("{:.0} MB / {:.0} MB", mb(done), mb(total));
                    let ratio = done as f32 / total as f32;
                    let _ = progress_ui.upgrade_in_event_loop(move |ui| {
                        ui.set_progress(ratio);
                        ui.set_detail(detail.into());
                    });
                });
                let _ = weak.upgrade_in_event_loop(move |ui| match result {
                    Ok(()) => {
                        let _ = ui.hide();
                    }
                    Err(e) => {
                        ui.set_failed(true);
                        ui.set_detail(
                            format!("{e}\nネットワークを確認して「再試行」を押してください。")
                                .into(),
                        );
                    }
                });
            });
        }
    };

    ui.on_retry({
        let start = start.clone();
        move || start()
    });
    ui.on_cancel({
        let weak = ui.as_weak();
        let cancel = cancel.clone();
        move || {
            cancel.store(true, Ordering::Relaxed);
            if let Some(ui) = weak.upgrade() {
                let _ = ui.hide();
            }
        }
    });

    start();
    ui.run()?;
    // キャンセルで閉じた場合、裏の curl を止めて部分ファイルを消すまで少し待つ
    cancel.store(true, Ordering::Relaxed);
    std::thread::sleep(std::time::Duration::from_millis(300));
    Ok(())
}
