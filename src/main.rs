#![windows_subsystem = "windows"]

use bindings::{
    Windows::Win32::Foundation::*,
    Windows::Win32::System::Diagnostics::Debug::SetLastError,
    Windows::Win32::System::LibraryLoader::GetModuleHandleW,
    Windows::Win32::UI::Controls::{LR_DEFAULTSIZE, LR_LOADFROMFILE, LR_SHARED},
    Windows::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    },
    Windows::Win32::UI::WindowsAndMessaging::*,
    Windows::Win32::Graphics::Gdi::ValidateRect,
    Windows::Win32::System::Mmc::*,
};
use windows::{Handle};
use utf16_lit::utf16_null;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

const TRAY_ICON_ID: u32 = 5;
const TRAY_MESSAGE: u32 = WM_APP + 1;

/// Macro to invoke unsafe win32 API and check error code.
/// Uses 1 WinAPI call on success path or 2 (+ `GetLastError`) if needs to check an error code.
///
/// # Arguments
///
/// * `func` - function with all arguments; return value must implement `windows.Handle` trait
///
/// # Return value
///
/// `windows::Result<H> where H: windows.Handle` type
///
/// # Examples
///
/// ```
/// let module_handle: HINSTANCE = execute!(GetModuleHandleW(None))?;
/// ```
macro_rules! execute {
    ($func:expr) => {
        {
            let result = unsafe { $func };
            if result.is_invalid() {
                let err: windows::Error = windows::Error::from_win32();
                Err(err)
            } else {
                Ok(result)
            }
        }
    };
}

/// Macro to invoke unsafe win32 API and check error code.
/// Uses 3 Win API calls (+ `SetLastError` comparing to `execute!`) -> probably slower.
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
/// let atom: u16 = execute_not_handle!(RegisterClassExW(&win_class))?;
/// ```
macro_rules! execute_not_handle {
    ($func:expr) => {
        {
            unsafe {SetLastError(0)};
            let result = unsafe { $func };
            let err: windows::Error = windows::Error::from_win32();
            match err.info() {
                Option::Some(_) => Err(err),
                Option::None    => Ok(result),
            }
        }
    };
}

#[cfg(windows)]
fn win32_string( value : &str ) -> Vec<u16> {
    OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}

#[cfg(windows)]
fn main() -> windows::Result<()> {
    let module_handle: HINSTANCE = execute!(GetModuleHandleW(None))?;

    let icon_handle = execute!(
        LoadImageW(
            module_handle,
            "resources/find_bug_icon_32px_by_Chenyu_Wang.ico",
            IMAGE_ICON,
            32,
            32,
            LR_DEFAULTSIZE | LR_LOADFROMFILE | LR_SHARED,
        )
    )?;
    let icon: HICON = HICON(icon_handle.0);

    let menu_name = utf16_null!("some menu name");
    let mut class_name = utf16_null!("notify_icon_class");
    let cursor: HCURSOR = execute!(LoadCursorW(None, IDC_ARROW))?;

    let mut about = utf16_null!("About");
    let item_about: CONTEXTMENUITEM2 = CONTEXTMENUITEM2 {
        strName: PWSTR(about.as_mut_ptr()),

        ..Default::default()
    };

    // let menu: HMENU = execute!(CreateMenu())?;
    // let mut menu_added: BOOL;
    // menu_added = execute_not_handle!(AppendMenuW(menu, MF_CHECKED | MF_STRING, 0, PWSTR(about.as_mut_ptr())))?;

    // let menu_provider = unsafe {EmptyMenuList()};
    // let add_result: windows::Result<()> = unsafe {menu_provider.AddItem(&item_about as *const CONTEXTMENUITEM2)};

    let win_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: module_handle,
        hIcon: icon,
        hCursor: cursor,
        // lpszMenuName: PWSTR(menu_name.as_ptr() as _),
        lpszClassName: PWSTR(class_name.as_mut_ptr() as _),
        hIconSm: icon,

        ..Default::default()
    };

    let atom: u16 = execute_not_handle!(RegisterClassExW(&win_class))?;
    assert!(atom != 0);

    let mut window_name = utf16_null!("The window");

    let win_handle: HWND = execute!(
        CreateWindowExW(
            Default::default(),
            PWSTR(class_name.as_mut_ptr()),
            PWSTR(window_name.as_mut_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            module_handle,
            std::ptr::null_mut(),
        )
    )?;

    let mut sz_tip: Vec<u16> = Vec::with_capacity(128);
    sz_tip.extend_from_slice(&utf16_null!("Debug environment simulator"));
    sz_tip.resize(128, 0);

    let mut tray_data: NOTIFYICONDATAW = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: win_handle,
        uID: TRAY_ICON_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: TRAY_MESSAGE,
        hIcon: icon,

        ..Default::default()
    };

    tray_data.szTip.clone_from_slice(sz_tip.as_slice());

    let is_added: BOOL = execute_not_handle!(Shell_NotifyIconW(NIM_ADD, &tray_data))?;
    assert!(is_added.as_bool());

    // unsafe {
    //     ShowWindow(win_handle, SW_SHOW);
    // }

    let mut message = MSG::default();

    unsafe {
        while GetMessageW(&mut message, HWND::default(), 0, 0).into() {
            // TranslateMessage(&mut message);
            DispatchMessageW(&message);
        }
    }

    let is_deleted: BOOL = execute_not_handle!(Shell_NotifyIconW(NIM_DELETE, &tray_data))?;
    assert!(is_deleted.as_bool());

    Ok(())
}

unsafe extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message as u32 {
        TRAY_MESSAGE => {
            match (lparam.0 as u32) & 0x0000FFFF {
                WM_LBUTTONDOWN => {
                    ShowWindow(window, SW_RESTORE);
                    LRESULT(0)
                }
                // WM_RBUTTONDOWN | WM_CONTEXTMENU => {
                //     ShowContextMenu(hWnd);
                //     LRESULT(0)
                // }
                _ => DefWindowProcW(window, message, wparam, lparam),
            }
        }
        WM_PAINT => {
            // println!("WM_PAINT");
            ValidateRect(window, std::ptr::null());
            LRESULT(0)
        }
        WM_DESTROY => {
            // println!("WM_DESTROY");
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}
