/// Macro to invoke unsafe win32 API and check error code.
/// Uses 3 Win API calls.
///
/// # Arguments
///
/// * `func` - function with all arguments; there is no restriction on return type
///
/// # Return value
///
/// `windows::Result<T> where T` is an arbitrary type
///
/// # Examples
///
/// ```
/// let atom: u16 = execute!(RegisterClassExW(&win_class))?;
/// ```
macro_rules! execute {
    ($func:expr) => {{
        unsafe { SetLastError(0) };
        let result = unsafe { $func };
        let err: windows::core::Error = windows::core::Error::from_win32();
        match err.info() {
            Option::Some(_) => Err(err),
            Option::None => Ok(result),
        }
    }};
}

#[macro_export]
macro_rules! dprint {
    ($str:expr) => {{
        unsafe { windows::Win32::System::Diagnostics::Debug::OutputDebugStringW(
            windows::Win32::Foundation::PWSTR($str.as_mut_ptr())
        ); }
    }};
}

macro_rules! MessageBoxV {
    ($handle:expr,$text:expr,$caption:expr,$icon:expr) => {{
        windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
            $handle,
            windows::Win32::Foundation::PWSTR(to_utf16($text).as_mut_ptr()),
            windows::Win32::Foundation::PWSTR(to_utf16($caption).as_mut_ptr()),
            $icon
        );
    }}
}

macro_rules! LOWORD {
    ($var:expr) => {{
        ($var as u32) & 0x0000FFFF
    }};
}

macro_rules! _HIWORD {
    ($var:expr) => {{
        ($var as u32) & 0xFFFF0000
    }};
}
