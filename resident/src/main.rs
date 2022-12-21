#![windows_subsystem = "windows"]

use utf16_lit::utf16_null;
use windows::{
    // core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    },
    Win32::UI::WindowsAndMessaging::*,
};

mod menu_entry; //TODO: remove?

mod menu_state;
use menu_state::*;

mod menu_ids;
use menu_ids::*;

#[macro_use]
mod macros;

const TRAY_ICON_ID: u32 = 5;
const TRAY_MESSAGE: u32 = WM_APP + 1;

// Selected by default
const SELECTED_VALUES: [u32; 4] = [MENU_GUEST_VIRTUALBOX, MENU_DEBUGGER_IDA, MENU_FIREWALL_ZONEALARM, MENU_ANTIVIRUS_MCAFEE];

// Main menu
static mut MENU_TRAY: HMENU = 0;
static mut MENU_STATE: MenuState = MenuState::new();

fn to_utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().chain(std::iter::once(0)).collect();
}

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    unsafe { MENU_STATE.init_menu_entries() };

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

    //let menu_name = utf16_null!("some menu name");
    let mut class_name = utf16_null!("notify_icon_class");
    let cursor: HCURSOR = execute!(LoadCursorW(None, IDC_ARROW))?;

    let win_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: module_handle,
        hIcon: icon,
        hCursor: cursor,
        //    lpszMenuName: PWSTR(menu_name.as_ptr() as _),
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

    unsafe {
        create_menu(&mut MENU_TRAY);
    }

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

unsafe fn flip_menu_state(context_menu: HMENU, menu_item: u32) {
    let is_active = MENU_STATE.is_process_active(&menu_item);

    if is_active{
        CheckMenuItem(context_menu, menu_item, MF_UNCHECKED);
        MENU_STATE.stop_process(&menu_item);
    } else {
        CheckMenuItem(context_menu, menu_item, MF_CHECKED);
        MENU_STATE.start_process(&menu_item);
    }
}

unsafe extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        TRAY_MESSAGE => match LOWORD!(lparam) {
            WM_LBUTTONUP => {
                ShowWindow(window, SW_RESTORE);
                0
            }
            WM_RBUTTONUP => {
                let mut point: POINT = Default::default();
                GetCursorPos(&mut point);
                handle_popup_menu(window, point, MENU_TRAY);

                0
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        },
        WM_COMMAND => match LOWORD!(wparam) {
            MENU_PAUSE=> {
                // TODO: Modify menu UI
                MENU_STATE.pause();
                0
            }
            MENU_RESUME => {
                // TODO: Modify menu UI
                MENU_STATE.resume();
                0
            }
            MENU_GUEST | MENU_DEBUGGER | MENU_ANTIVIRUS | MENU_FIREWALL | MENU_TOOLS => {
                // This should never happen. Assert?
                MessageBoxW(
                    0,
                    PWSTR(utf16_null!("Selected non-active menu items.").as_mut_ptr()),
                    PWSTR(utf16_null!("Error").as_mut_ptr()),
                    MB_OK | MB_ICONERROR,
                );
                0
            }
            MENU_DEBUGGER_OLLY
            | MENU_DEBUGGER_WINDBG
            | MENU_DEBUGGER_X64DBG
            | MENU_DEBUGGER_IDA
            | MENU_DEBUGGER_IMMUNITY
            | MENU_GUEST_VIRTUALBOX
            | MENU_GUEST_VMWARE
            | MENU_GUEST_PARALLELS
            | MENU_GUEST_HYPERV
            | MENU_GUEST_VIRTUAL_PC
            | MENU_FIREWALL_COMODO
            | MENU_FIREWALL_GLASSWIRE
            | MENU_FIREWALL_TINYWALL
            | MENU_FIREWALL_ZONEALARM
            | MENU_ANTIVIRUS_AVAST
            | MENU_ANTIVIRUS_AVIRA
            | MENU_ANTIVIRUS_BITDEFENDER
            | MENU_ANTIVIRUS_DRWEB
            | MENU_ANTIVIRUS_ESCAN
            | MENU_ANTIVIRUS_ESET_NOD32
            | MENU_ANTIVIRUS_FSECURE
            | MENU_ANTIVIRUS_GDATA
            | MENU_ANTIVIRUS_KASPERSKY
            | MENU_ANTIVIRUS_MALWAREBYTES
            | MENU_ANTIVIRUS_MCAFEE
            | MENU_ANTIVIRUS_NORTON
            | MENU_ANTIVIRUS_PANDA
            | MENU_ANTIVIRUS_SOPHOS
            | MENU_ANTIVIRUS_TREND_MICRO
            | MENU_ANTIVIRUS_WEBROOT => {
                // TODO: Is there a nice way to bind this variable?
                let m = LOWORD!(wparam);
                flip_menu_state(MENU_TRAY, m);
                0
            }
            MENU_ABOUT => {
                MessageBoxA(window, "About", "Caption", MB_OK);
                0
            }
            MENU_EXIT => {
                SendMessageW(window, WM_CLOSE, 0, 0);
                0
            }
            _ => {
                MessageBoxA(window, wparam.to_string(), "Unknown command", MB_OK);
                0
            }
        },
        WM_PAINT => {
            // println!("WM_PAINT");
            ValidateRect(window, std::ptr::null());
            0
        }
        WM_CREATE => 0,
        WM_DESTROY => {
            // println!("WM_DESTROY");
            exit_routine()
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

unsafe fn exit_routine() -> LRESULT {
    DestroyMenu(MENU_TRAY);
    MENU_STATE.destroy();
    PostQuitMessage(0);
    0
}

fn append_menu(menu: HMENU, entry_ids: &[u32]) {
    for e in entry_ids {
        let bird = if SELECTED_VALUES.contains(e) {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        unsafe {
            AppendMenuW(
                menu,
                bird | MF_STRING,
                *e as usize,
                PWSTR(to_utf16(MENU_STATE.get_name(e)).as_mut_ptr()),
            )
        };
    }
}

unsafe fn create_menu(context_menu: &mut HMENU) {
    let guest_entries: &[u32] = &[
        MENU_GUEST_VIRTUALBOX,
        MENU_GUEST_VMWARE,
        MENU_GUEST_PARALLELS,
        MENU_GUEST_HYPERV,
        MENU_GUEST_VIRTUAL_PC,
    ];

    let debugger_entries: &[u32] = &[
        MENU_DEBUGGER_OLLY,
        MENU_DEBUGGER_WINDBG,
        MENU_DEBUGGER_X64DBG,
        MENU_DEBUGGER_IDA,
        MENU_DEBUGGER_IMMUNITY,
    ];

    let antivirus_entries: &[u32] = &[MENU_ANTIVIRUS_AVAST, MENU_ANTIVIRUS_AVIRA, MENU_ANTIVIRUS_ESCAN];

    let firewall_entries: &[u32] = &[
        MENU_FIREWALL_ZONEALARM, MENU_FIREWALL_GLASSWIRE, MENU_FIREWALL_COMODO, MENU_FIREWALL_TINYWALL
    ];

    let mut pause = utf16_null!("Pause");
    let _resume = utf16_null!("Resume");
    let mut vm_guest = utf16_null!("VM guest process");
    let mut debugger = utf16_null!("Debugger");
    let mut antivirus = utf16_null!("Antivirus");
    let mut firewall = utf16_null!("Firewall");
    let mut about = utf16_null!("About");
    let mut exit = utf16_null!("Exit");

    let guest_submenu: HMENU = CreatePopupMenu();
    append_menu(guest_submenu, guest_entries);

    let debugger_submenu: HMENU = CreatePopupMenu();
    append_menu(debugger_submenu, debugger_entries);

    let antivirus_submenu: HMENU = CreatePopupMenu();
    append_menu(antivirus_submenu, antivirus_entries);

    let firewall_submenu: HMENU = CreatePopupMenu();
    append_menu(firewall_submenu, firewall_entries);

    let menu: HMENU = CreatePopupMenu();
    AppendMenuW(
        menu,
        MF_STRING,
        MENU_PAUSE as usize,
        PWSTR(pause.as_mut_ptr()),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, None);
    AppendMenuW(
        menu,
        MF_STRING | MF_POPUP,
        guest_submenu as usize,
        PWSTR(vm_guest.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        MF_STRING | MF_POPUP,
        debugger_submenu as usize,
        PWSTR(debugger.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        MF_STRING | MF_POPUP,
        antivirus_submenu as usize,
        PWSTR(antivirus.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        MF_STRING | MF_POPUP,
        firewall_submenu as usize,
        PWSTR(firewall.as_mut_ptr()),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, None);
    AppendMenuW(
        menu,
        MF_STRING,
        MENU_ABOUT as usize,
        PWSTR(about.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        MENU_EXIT as usize,
        PWSTR(exit.as_mut_ptr()),
    );

    *context_menu = menu;
}

unsafe extern "system" fn handle_popup_menu(window: HWND, point: POINT, menu: HMENU) {
    SetForegroundWindow(window);
    TrackPopupMenu(
        menu,
        TPM_BOTTOMALIGN | TPM_RIGHTALIGN,
        point.x,
        point.y,
        0,
        window,
        std::ptr::null::<RECT>(),
    );
    PostMessageW(window, WM_NULL, 0, 0);
}
