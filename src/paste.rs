//! クリップボード経由での貼り付け。元のクリップボードの中身は貼り付け後に戻す。

use anyhow::Result;
use std::thread::sleep;
use std::time::{Duration, Instant};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
    VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LWIN, VK_RCONTROL, VK_RWIN,
};

const VK_V: VIRTUAL_KEY = 0x56;

pub fn paste(text: &str) -> Result<()> {
    // Win などが押されたままだと Ctrl+V が別のショートカットになるので、離されるのを待つ
    wait_modifiers_released(Duration::from_secs(3));

    let mut cb = arboard::Clipboard::new()?;
    let previous_text = cb.get_text().ok();
    let previous_image = if previous_text.is_none() { cb.get_image().ok() } else { None };

    cb.set_text(text)?;
    send_ctrl_v();

    // 貼り付け先のアプリがクリップボードを読み終わるのを待ってから戻す
    sleep(Duration::from_millis(300));
    if let Some(t) = previous_text {
        cb.set_text(t)?;
    } else if let Some(img) = previous_image {
        cb.set_image(img)?;
    }
    Ok(())
}

fn wait_modifiers_released(timeout: Duration) {
    let start = Instant::now();
    let held = |vk: VIRTUAL_KEY| unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000 != 0;
    while start.elapsed() < timeout
        && [VK_LWIN, VK_RWIN, VK_LCONTROL, VK_RCONTROL].into_iter().any(held)
    {
        sleep(Duration::from_millis(10));
    }
}

fn send_ctrl_v() {
    let key = |vk, flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 },
        },
    };
    let inputs = [
        key(VK_CONTROL, 0),
        key(VK_V, 0),
        key(VK_V, KEYEVENTF_KEYUP),
        key(VK_CONTROL, KEYEVENTF_KEYUP),
    ];
    unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), size_of::<INPUT>() as i32) };
}
