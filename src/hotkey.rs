//! Win + 左 Ctrl の押下・解放を低レベルキーボードフックで検出する。
//!
//! フックのコールバックは状態を更新してチャンネルに送るだけにする。
//! ここで重い処理をすると、Windows にフックを外されてしまうため。

use std::sync::mpsc::Sender;
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_LCONTROL, VK_LWIN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, SetWindowsHookExW, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    Pressed,
    Released,
}

#[derive(Default)]
struct State {
    lwin: bool,
    lctrl: bool,
    active: bool,
    /// この Win の押下中にホットキーとして使われたか（離したときにスタートメニューを抑えるため）
    win_used: bool,
}

static SENDER: OnceLock<Sender<HotkeyEvent>> = OnceLock::new();
static STATE: Mutex<State> = Mutex::new(State {
    lwin: false,
    lctrl: false,
    active: false,
    win_used: false,
});

/// フックを登録する。呼んだスレッドでメッセージループを回す必要がある。
pub fn install(sender: Sender<HotkeyEvent>) -> anyhow::Result<()> {
    SENDER
        .set(sender)
        .map_err(|_| anyhow::anyhow!("hotkey::install は一度だけ呼ぶ"))?;
    let hook =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), std::ptr::null_mut(), 0) };
    anyhow::ensure!(!hook.is_null(), "キーボードフックを登録できない");
    Ok(())
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kb = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        if kb.flags & LLKHF_INJECTED == 0 {
            let down = matches!(wparam as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
            let up = matches!(wparam as u32, WM_KEYUP | WM_SYSKEYUP);
            if down || up {
                on_key(kb.vkCode as u16, down);
            }
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

fn on_key(vk: u16, down: bool) {
    let mut s = STATE.lock().unwrap();
    match vk {
        VK_LWIN => {
            if !down && s.win_used {
                // Win を離す直前に無害なキーを挟み、スタートメニューが開かないようにする
                send_mask_key();
                s.win_used = false;
            }
            s.lwin = down;
        }
        VK_LCONTROL => s.lctrl = down,
        _ => return,
    }

    let send = |e| {
        if let Some(tx) = SENDER.get() {
            let _ = tx.send(e);
        }
    };
    if !s.active && s.lwin && s.lctrl {
        s.active = true;
        s.win_used = true;
        send(HotkeyEvent::Pressed);
    } else if s.active && !(s.lwin && s.lctrl) {
        s.active = false;
        send(HotkeyEvent::Released);
    }
}

/// 割り当てのない仮想キー 0xE8 を押して離す。
fn send_mask_key() {
    let key = |flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0xE8,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let inputs = [key(0), key(KEYEVENTF_KEYUP)];
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
}
