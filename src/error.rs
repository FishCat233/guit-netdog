use windows::core::{Error, HRESULT};

/// 把 Win32 的返回码转成 `windows` crate 的 `Error`。
pub fn win32_result_to_error(result: u32) -> Error {
    Error::from_hresult(HRESULT::from_win32(result))
}
