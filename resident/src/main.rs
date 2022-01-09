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

// use std::process::Command;

mod state;
use state::*;

#[macro_use]
mod macros;

const TRAY_ICON_ID: u32 = 5;
const TRAY_MESSAGE: u32 = WM_APP + 1;

const MENU_GUEST: u32 = 1;
const MENU_GUEST_VIRTUALBOX: u32 = 11;
const MENU_GUEST_VMWARE: u32 = 12;
const MENU_DISASSEMBLER: u32 = 2;
const MENU_DISASSEMBLER_IDA: u32 = 21;
const MENU_DISASSEMBLER_X64DBG: u32 = 22;
const MENU_DEBUGGER: u32 = 3;
const MENU_ANTIVIRUS: u32 = 4;
const MENU_FIREWALL: u32 = 5;
const MENU_ABOUT: u32 = 10;
const MENU_EXIT: u32 = 77;
const MENU_PAUSE: u32 = 78;
const MENU_RESUME: u32 = 79;

// Selected by default
const SELECTED_VALUES: [u32; 2] = [
    MENU_GUEST_VIRTUALBOX,
    MENU_DISASSEMBLER_IDA,
];
    
// Main menu
static mut MENU: HMENU = 0;

fn to_utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().chain(std::iter::once(0)).collect();
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

    unsafe {create_menu(&mut MENU);}

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
    let state: u32 = GetMenuState(context_menu, menu_item, MF_BYCOMMAND);

    if state == MF_CHECKED {
        CheckMenuItem(context_menu, menu_item, MF_UNCHECKED);
    } else {
        CheckMenuItem(context_menu, menu_item, MF_CHECKED);
    }
}

unsafe extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message as u32 {
        TRAY_MESSAGE => {
            match LOWORD!(lparam) {
                WM_LBUTTONUP => {
                    ShowWindow(window, SW_RESTORE);
                    0
                }
                WM_RBUTTONUP => {
                    let mut point: POINT = Default::default();
                    GetCursorPos(&mut point);
                    handle_popup_menu(window, point, MENU);

                    0
                }
                _ => DefWindowProcW(window, message, wparam, lparam),
            }
        }
        WM_COMMAND => match LOWORD!(wparam) {
            MENU_PAUSE | MENU_RESUME => {
                // TODO: Pause all or resume all
                0
            }
            MENU_GUEST => {
                //MessageBoxA(window, "Guest", "Caption", MB_OK);
                flip_menu_state(MENU, MENU_GUEST);
                0
            }
            MENU_DISASSEMBLER => {
                //MessageBoxA(window, "Disasm", "Caption", MB_OK);
                flip_menu_state(MENU, MENU_DISASSEMBLER);
                0
            }
            MENU_DEBUGGER => {
                //MessageBoxA(window, "Debugger", "Caption", MB_OK);
                flip_menu_state(MENU, MENU_DEBUGGER);
                0
            }
            MENU_ANTIVIRUS => {
                //MessageBoxA(window, "Antivirus", "Caption", MB_OK);
                flip_menu_state(MENU, MENU_ANTIVIRUS);
                0
            }
            MENU_FIREWALL => {
                //MessageBoxA(window, "Firewall", "Caption", MB_OK);
                flip_menu_state(MENU, MENU_FIREWALL);
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
        WM_CREATE => {
            0
        }
        WM_DESTROY => {
            // println!("WM_DESTROY");
            exit_routine()
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

unsafe fn exit_routine() -> LRESULT {
    DestroyMenu(MENU);
    PostQuitMessage(0);
    0
}

unsafe fn append_menu(menu: HMENU, entries: &Vec<MenuEntry>) {
    for e in entries {
        let bird = if SELECTED_VALUES.contains(&e.id) {MF_CHECKED} else {MF_UNCHECKED};
        AppendMenuW(
            menu,
            bird | MF_STRING,
            e.id as usize,
            PWSTR(to_utf16(&e.entry_text).as_mut_ptr()),
        );
    }
}

unsafe fn create_menu(context_menu: &mut HMENU) {
    let guest_entries: Vec<MenuEntry> = vec![
        MenuEntry {id: MENU_GUEST_VIRTUALBOX, entry_text: "VirtualBox", process_name: "TODO", process_child: None},
        MenuEntry {id: MENU_GUEST_VMWARE, entry_text: "VMware", process_name: "TODO", process_child: None},
    ];

    let disassembler_entries: Vec<MenuEntry> = vec![
        MenuEntry {id: MENU_DISASSEMBLER_IDA, entry_text: "IDA Pro", process_name: "TODO", process_child: None},
        MenuEntry {id: MENU_DISASSEMBLER_X64DBG, entry_text: "x64dbg", process_name: "x64dbg.exe", process_child: None},
    ];

    let mut pause = utf16_null!("Pause");
    let mut resume = utf16_null!("Resume");
    let mut vm_guest = utf16_null!("VM guest process");
    let mut disasm = utf16_null!("Disassembler");
    let mut debugger = utf16_null!("Debugger");
    let mut antivirus = utf16_null!("Antivirus");
    let mut firewall = utf16_null!("Firewall");
    let mut about = utf16_null!("About");
    let mut exit = utf16_null!("Exit");

    let guest_submenu: HMENU = CreatePopupMenu();
    append_menu(guest_submenu, &guest_entries);

    let disassembler_submenu: HMENU = CreatePopupMenu();
    append_menu(disassembler_submenu, &disassembler_entries);
    
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
        disassembler_submenu as usize,
        PWSTR(disasm.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        // if DEFAULT_MENU_STATE.debugger {MF_CHECKED} else {MF_UNCHECKED} | MF_STRING,
        MF_UNCHECKED | MF_STRING,
        MENU_DEBUGGER as usize,
        PWSTR(debugger.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        // if DEFAULT_MENU_STATE.antivirus {MF_CHECKED} else {MF_UNCHECKED} | MF_STRING,
        MF_UNCHECKED | MF_STRING,
        MENU_ANTIVIRUS as usize,
        PWSTR(antivirus.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        // if DEFAULT_MENU_STATE.firewall {MF_CHECKED} else {MF_UNCHECKED} | MF_STRING,
        MF_UNCHECKED | MF_STRING,
        MENU_FIREWALL as usize,
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
        0 as *const RECT,
    );
    PostMessageW(window, WM_NULL, 0, 0);
}
