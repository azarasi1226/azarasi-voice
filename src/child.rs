//! 子プロセス（設定画面・ダウンロード画面）の起動。
//!
//! 子プロセスはジョブオブジェクトにまとめておき、本体が終了したら（異常終了や強制終了でも）
//! Windows に子プロセスも終了させる。子プロセスが起動したプロセス（curl.exe など）も同じジョブに入る。

use std::os::windows::io::AsRawHandle;
use std::process::{Child, Command};
use std::sync::OnceLock;
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};

/// ジョブオブジェクトのハンドル。閉じずに持ち続け、本体の終了時に Windows が閉じる（そのとき子が終了する）。
struct Job(usize);

fn job() -> Option<&'static Job> {
    static JOB: OnceLock<Option<Job>> = OnceLock::new();
    JOB.get_or_init(|| unsafe {
        let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if handle.is_null() {
            return None;
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let ok = SetInformationJobObject(
            handle,
            JobObjectExtendedLimitInformation,
            (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        (ok != 0).then_some(Job(handle as usize))
    })
    .as_ref()
}

/// 自分自身の exe を `arg` 付きで起動し、本体と一緒に終了するようにする。
pub fn spawn(arg: &str) -> std::io::Result<Child> {
    let child = Command::new(std::env::current_exe()?).arg(arg).spawn()?;
    if let Some(job) = job() {
        unsafe { AssignProcessToJobObject(job.0 as _, child.as_raw_handle()) };
    }
    Ok(child)
}
