// TODO: pretify?
pub fn to_utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
}

pub fn to_pcwstr(text: &str) -> (Vec<u16>, windows::core::PCWSTR) {
    let v0 = to_utf16(text);
    let v1 = windows::core::PCWSTR(v0.as_ptr());
    (v0, v1)
}