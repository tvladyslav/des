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
        unsafe { SetLastError(ERROR_SUCCESS) };
        let result = unsafe { $func };
        let err: windows::core::Error = windows::core::Error::from_win32();
        match err.info() {
            Option::Some(_) => Err(err),
            Option::None => Ok(result),
        }
    }};
}
