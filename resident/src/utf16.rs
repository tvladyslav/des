// TODO: pretify?
pub fn to_utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
}

pub fn to_pcwstr(text: *const u16) -> windows::core::PCWSTR {
    windows::core::PCWSTR(text)
}