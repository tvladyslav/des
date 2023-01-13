#![windows_subsystem = "windows"]

use windows::{
    w,
    // core::*,
    Win32::Foundation::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

#[macro_use]
mod macros;

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    // TODO: some argument key
    if std::env::args().count() == 1 {
        execute!(MessageBoxW(
            HWND(0),
            w!("Don't run this application manually."),
            w!("Error"),
            MB_OK | MB_ICONERROR
        ))?;
        return Ok(());
    }

    let module_handle: HINSTANCE = unsafe {GetModuleHandleW(None) }?;
    let cursor: HCURSOR = unsafe {LoadCursorW(None, IDC_ARROW) }?;
    let class_name = w!("stub_class");

    let win_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: module_handle,
        hCursor: cursor,
        lpszClassName: class_name,

        ..Default::default()
    };

    let atom: u16 = execute!(RegisterClassExW(&win_class))?;
    assert!(atom != 0);

    let _win_handle: HWND = execute!(CreateWindowExW(
        Default::default(),
        class_name,
        w!("The window"),
        WS_DISABLED,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        None,
        None,
        module_handle,
        None,
    ))?;

    let mut message = MSG::default();

    unsafe {
        while GetMessageW(&mut message, HWND::default(), 0, 0).into() {
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

unsafe extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(window, message, wparam, lparam)
}
