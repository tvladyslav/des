use std::io::ErrorKind;

use windows::Win32::Foundation::*;

// TODO: pretify?
pub fn to_utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
}

pub fn to_pcwstr(text: &str) -> (Vec<u16>, windows::core::PCWSTR) {
    let v0 = to_utf16(text);
    let v1 = windows::core::PCWSTR(v0.as_ptr());
    (v0, v1)
}

// One should avoid this function and use opposite direction.
// windows::core::Error => std::io::Error is profided by windows crate.
pub fn to_win_error(error: std::io::Error) -> windows::core::Error {
    let r1 = match error.kind() {
        ErrorKind::NotFound => ERROR_FILE_NOT_FOUND,
        ErrorKind::PermissionDenied => ERROR_ACCESS_DENIED,
        _ => ERROR_SUCCESS,
    };
    windows::core::Error::from(r1)
}