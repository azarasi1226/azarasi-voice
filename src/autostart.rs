//! Windows 起動時の自動起動。
//!
//! HKCU\Software\Microsoft\Windows\CurrentVersion\Run に exe のパスを登録する。
//! 登録状態はレジストリそのものを正とし、設定ファイルには持たない。

use anyhow::{Result, bail};
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "azarasi-voice";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 自動起動に登録されているか。
pub fn is_enabled() -> bool {
    let (key, name) = (wide(RUN_KEY), wide(VALUE_NAME));
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    status == ERROR_SUCCESS
}

/// 自動起動を登録・解除する。
///
/// 登録するのは今動いている exe と同じフォルダの `azarasi-voice.exe`。
/// デプロイ時に古い exe は `azarasi-voice.old-*.exe` に名前を変えられるので、
/// current_exe() をそのまま使うと消える予定のファイルを登録してしまうことがある。
pub fn set_enabled(enabled: bool) -> Result<()> {
    let (key, name) = (wide(RUN_KEY), wide(VALUE_NAME));
    let status = if enabled {
        let exe = std::env::current_exe()?.with_file_name("azarasi-voice.exe");
        let command = wide(&format!("\"{}\"", exe.display()));
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                REG_SZ,
                command.as_ptr().cast(),
                (command.len() * 2) as u32,
            )
        }
    } else {
        if !is_enabled() {
            return Ok(());
        }
        unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr()) }
    };
    if status != ERROR_SUCCESS {
        bail!("レジストリを書き換えられない（エラー {status}）");
    }
    Ok(())
}
