#![windows_subsystem = "windows"]

use utf16_lit::utf16_null;
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    },
    Win32::UI::WindowsAndMessaging::*,
};

const TRAY_ICON_ID: u32 = 5;
const TRAY_MESSAGE: u32 = WM_APP + 1;

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

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    let module_handle: HINSTANCE = execute!(GetModuleHandleW(None))?;

    let icon_handle = execute!(LoadImageW(
        module_handle,
        "resources/find_bug_icon_32px_by_Chenyu_Wang.ico",
        IMAGE_ICON,
        32,
        32,
        LR_DEFAULTSIZE | LR_LOADFROMFILE | LR_SHARED,
    ))?;
    let icon: HICON = icon_handle.0;

    let menu_name = utf16_null!("some menu name");
    let mut class_name = utf16_null!("notify_icon_class");
    let cursor: HCURSOR = execute!(LoadCursorW(None, IDC_ARROW))?;

    // let mut vm_guest = utf16_null!("VM guest process");
    // let mut disasm = utf16_null!("Disassembler");
    // let mut about = utf16_null!("About");

    // let menu: HMENU = execute!(CreatePopupMenu())?;
    // let mut menu_added: bool = true;
    // menu_added &= execute!(AppendMenuW(menu, MF_UNCHECKED | MF_STRING, 0, PWSTR(vm_guest.as_mut_ptr())))?.as_bool();
    // menu_added &= execute!(AppendMenuW(menu, MF_CHECKED | MF_STRING, 0, PWSTR(disasm.as_mut_ptr())))?.as_bool();
    // menu_added &= execute!(AppendMenuW(menu, MF_CHECKED | MF_STRING, 0, PWSTR(about.as_mut_ptr())))?.as_bool();
    // assert!(menu_added == true);

    let win_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: module_handle,
        hIcon: icon,
        hCursor: cursor,
        lpszMenuName: PWSTR(menu_name.as_ptr() as _),
        lpszClassName: PWSTR(class_name.as_mut_ptr() as _),
        hIconSm: icon,

        ..Default::default()
    };

    let atom: u16 = execute!(RegisterClassExW(&win_class))?;
    assert!(atom != 0);

    let mut window_name = utf16_null!("The window");

    let win_handle: HWND = execute!(CreateWindowExW(
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
    ))?;

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

    let is_added: BOOL = execute!(Shell_NotifyIconW(NIM_ADD, &tray_data))?;
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

    let is_deleted: BOOL = execute!(Shell_NotifyIconW(NIM_DELETE, &tray_data))?;
    assert!(is_deleted.as_bool());

    Ok(())
}

unsafe extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message as u32 {
        TRAY_MESSAGE => {
            match (lparam as u32) & 0x0000FFFF {
                WM_LBUTTONUP => {
                    ShowWindow(window, SW_RESTORE);
                    0
                }
                WM_RBUTTONUP => {
                    // println!("Params are {0} {1}", wparam, lparam);
                    let mut point: POINT = Default::default();
                    GetCursorPos(&mut point);
                    HandlePopupMenu(window, point);

                    0
                }
                _ => DefWindowProcW(window, message, wparam, lparam),
            }
        }
        WM_PAINT => {
            // println!("WM_PAINT");
            ValidateRect(window, std::ptr::null());
            0
        }
        WM_DESTROY => {
            // println!("WM_DESTROY");
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

unsafe extern "system" fn HandlePopupMenu(window: HWND, point: POINT) {
    let mut vm_guest = utf16_null!("VM guest process");
    let mut disasm = utf16_null!("Disassembler");
    let mut about = utf16_null!("About");
    //let mut lp: Vec<u16> = lparam.to_string().encode_utf16().chain(Some(0)).collect();
    //let mut wp: Vec<u16> = wparam.to_string().encode_utf16().chain(Some(0)).collect();

    let menu: HMENU = execute!(CreatePopupMenu()).unwrap();
    let mut menu_added: bool = true;
    menu_added &= execute!(AppendMenuW(
        menu,
        MF_UNCHECKED | MF_STRING,
        0,
        PWSTR(vm_guest.as_mut_ptr())
    ))
    .unwrap()
    .as_bool();
    menu_added &= execute!(AppendMenuW(
        menu,
        MF_CHECKED | MF_STRING,
        0,
        PWSTR(disasm.as_mut_ptr())
    ))
    .unwrap()
    .as_bool();
    menu_added &= execute!(AppendMenuW(menu, MF_SEPARATOR, 0, None))
        .unwrap()
        .as_bool();
    menu_added &= execute!(AppendMenuW(
        menu,
        MF_CHECKED | MF_STRING,
        0,
        PWSTR(about.as_mut_ptr())
    ))
    .unwrap()
    .as_bool();
    //menu_added &= execute!(AppendMenuW(menu, MF_CHECKED | MF_STRING, 0, PWSTR(wp.as_mut_ptr()))).unwrap().as_bool();
    //menu_added &= execute!(AppendMenuW(menu, MF_CHECKED | MF_STRING, 0, PWSTR(lp.as_mut_ptr()))).unwrap().as_bool();
    assert!(menu_added == true);

    SetForegroundWindow(window);
    let selection = TrackPopupMenu(
        menu,
        TPM_BOTTOMALIGN | TPM_RIGHTALIGN,
        point.x,
        point.y,
        0,
        window,
        0 as *const RECT,
    );
    PostMessageW(window, WM_NULL, 0, 0);
    //MessageBoxW(window, selection.to_string(), "None", MB_OK);
    DestroyMenu(menu);
}
