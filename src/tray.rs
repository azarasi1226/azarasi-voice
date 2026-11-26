//! タスクトレイのアイコンとメニュー。
//!
//! アイコンはメインスレッド（メッセージループのあるスレッド）でしか触れないので、
//! 他のスレッドからは `notify` でスレッドメッセージを送って状態を伝える。

use crate::icon_draw;
use anyhow::Result;
use std::sync::atomic::{AtomicU32, Ordering};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage, WM_APP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Status {
    Idle = WM_APP + 1,
    Recording = WM_APP + 2,
    Quit = WM_APP + 3,
}

static MAIN_THREAD: AtomicU32 = AtomicU32::new(0);

/// メインスレッドに状態の変化を伝える。どのスレッドからでも呼べる。
pub fn notify(status: Status) {
    let tid = MAIN_THREAD.load(Ordering::Relaxed);
    if tid != 0 {
        unsafe { PostThreadMessageW(tid, status as u32, 0, 0) };
    }
}

/// トレイアイコンを作ってメッセージループを回す。「終了」が選ばれたら戻る。
pub fn run() -> Result<()> {
    MAIN_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);

    let open_item = MenuItem::new("設定を開く", true, None);
    let quit_item = MenuItem::new("終了", true, None);
    let menu = Menu::new();
    menu.append(&open_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let idle_icon = make_icon(icon_draw::COLOR_IDLE)?;
    let recording_icon = make_icon(icon_draw::COLOR_RECORDING)?;
    let tray: TrayIcon = TrayIconBuilder::new()
        .with_icon(idle_icon.clone())
        .with_tooltip("azarasi-voice（Win + 左 Ctrl で録音）")
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()?;

    TrayIconEvent::set_event_handler(Some(|e| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = e
        {
            open_settings();
        }
    }));
    let (open_id, quit_id) = (open_item.id().clone(), quit_item.id().clone());
    MenuEvent::set_event_handler(Some(std::sync::Arc::new(move |e: MenuEvent| {
        if e.id == open_id {
            open_settings();
        } else if e.id == quit_id {
            notify(Status::Quit);
        }
    })));

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            match msg.message {
                m if m == Status::Idle as u32 => {
                    let _ = tray.set_icon(Some(idle_icon.clone()));
                }
                m if m == Status::Recording as u32 => {
                    let _ = tray.set_icon(Some(recording_icon.clone()));
                }
                m if m == Status::Quit as u32 => break,
                _ => {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
    Ok(())
}

/// 設定画面を別プロセスで開く。
fn open_settings() {
    let result = std::env::current_exe()
        .and_then(|exe| std::process::Command::new(exe).arg("--settings").spawn());
    if let Err(e) = result {
        eprintln!("[tray] 設定画面を開けない: {e}");
    }
}

/// トレイ用のアイコン。高 DPI でもぼやけないよう 32px で作り、Windows に縮小させる。
fn make_icon(color: [u8; 3]) -> Result<Icon> {
    const SIZE: u32 = 32;
    Ok(Icon::from_rgba(
        icon_draw::render_icon(SIZE, color),
        SIZE,
        SIZE,
    )?)
}
