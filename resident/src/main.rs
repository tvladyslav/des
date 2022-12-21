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

#[macro_use]
extern crate num_derive;
use num_traits::FromPrimitive;

mod menu_entry; //TODO: remove?

mod menu_state;
use menu_state::*;

mod menu_ids;
use menu_ids::MenuId;

#[macro_use]
mod macros;

const TRAY_ICON_ID: u32 = 5;
const TRAY_MESSAGE: u32 = WM_APP + 1;

// Selected by default
const SELECTED_VALUES: [MenuId; 1] = [
    MenuId::GUEST_VIRTUALBOX,
    // MenuId::DEBUGGER_IDA,
    // MenuId::FIREWALL_ZONEALARM,
    // MenuId::ANTIVIRUS_MCAFEE,
];

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

unsafe fn flip_menu_state(context_menu: HMENU, menu_item: MenuId) -> std::io::Result<()> {
    let is_active = MENU_STATE.is_process_active(&menu_item);

    if is_active {
        CheckMenuItem(context_menu, menu_item as u32, MF_UNCHECKED);
        MENU_STATE.stop_process(&menu_item)
    } else {
        CheckMenuItem(context_menu, menu_item as u32, MF_CHECKED);
        MENU_STATE.start_process(&menu_item)
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
        WM_COMMAND => {
            let lo_wparam: MenuId = FromPrimitive::from_u32(LOWORD!(wparam)).unwrap();
            match lo_wparam {
                MenuId::PAUSE => {
                    // TODO: Modify menu UI
                    MENU_STATE.pause();
                    0
                }
                MenuId::RESUME => {
                    // TODO: Modify menu UI
                    MENU_STATE.resume();
                    0
                }
                MenuId::GUEST
                // | MenuId::DEBUGGER
                // | MenuId::ANTIVIRUS
                // | MenuId::FIREWALL
                // | MenuId::TOOLS
                => {
                    // This should never happen. Assert?
                    MessageBoxW(
                        0,
                        PWSTR(utf16_null!("Selected non-active menu items.").as_mut_ptr()),
                        PWSTR(utf16_null!("Error").as_mut_ptr()),
                        MB_OK | MB_ICONERROR,
                    );
                    0
                }
                // MenuId::DEBUGGER_OLLY
                // | MenuId::DEBUGGER_WINDBG
                // | MenuId::DEBUGGER_X64DBG
                // | MenuId::DEBUGGER_IDA
                // | MenuId::DEBUGGER_IMMUNITY
                MenuId::GUEST_VIRTUALBOX
                | MenuId::GUEST_VMWARE
                | MenuId::GUEST_PARALLELS
                | MenuId::GUEST_HYPERV
                | MenuId::GUEST_VIRTUAL_PC
                // | MenuId::FIREWALL_COMODO
                // | MenuId::FIREWALL_GLASSWIRE
                // | MenuId::FIREWALL_TINYWALL
                // | MenuId::FIREWALL_ZONEALARM
                // | MenuId::ANTIVIRUS_AVAST
                // | MenuId::ANTIVIRUS_AVIRA
                // | MenuId::ANTIVIRUS_BITDEFENDER
                // | MenuId::ANTIVIRUS_DRWEB
                // | MenuId::ANTIVIRUS_ESCAN
                // | MenuId::ANTIVIRUS_ESET_NOD32
                // | MenuId::ANTIVIRUS_FSECURE
                // | MenuId::ANTIVIRUS_GDATA
                // | MenuId::ANTIVIRUS_KASPERSKY
                // | MenuId::ANTIVIRUS_MALWAREBYTES
                // | MenuId::ANTIVIRUS_MCAFEE
                // | MenuId::ANTIVIRUS_NORTON
                // | MenuId::ANTIVIRUS_PANDA
                // | MenuId::ANTIVIRUS_SOPHOS
                // | MenuId::ANTIVIRUS_TREND_MICRO
                // | MenuId::ANTIVIRUS_WEBROOT
                => {
                    // TODO: Is there a nice way to bind this variable?
                    flip_menu_state(MENU_TRAY, lo_wparam);
                    0
                }
                MenuId::ABOUT => {
                    MessageBoxA(window, "About", "Caption", MB_OK);
                    0
                }
                MenuId::EXIT => {
                    SendMessageW(window, WM_CLOSE, 0, 0);
                    0
                }
                _ => {
                    MessageBoxA(window, wparam.to_string(), "Unknown command", MB_OK);
                    0
                }
            }
        }
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

fn append_menu(menu: HMENU, entry_ids: &[MenuId]) {
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
    let guest_entries: &[MenuId] = &[
        MenuId::GUEST_VIRTUALBOX,
        MenuId::GUEST_VMWARE,
        MenuId::GUEST_PARALLELS,
        MenuId::GUEST_HYPERV,
        MenuId::GUEST_VIRTUAL_PC,
    ];

    // let debugger_entries: &[MenuId] = &[
    //     MenuId::DEBUGGER_OLLY,
    //     MenuId::DEBUGGER_WINDBG,
    //     MenuId::DEBUGGER_X64DBG,
    //     MenuId::DEBUGGER_IDA,
    //     MenuId::DEBUGGER_IMMUNITY,
    // ];

    // let antivirus_entries: &[MenuId] = &[
    //     MenuId::ANTIVIRUS_AVAST,
    //     MenuId::ANTIVIRUS_AVIRA,
    //     MenuId::ANTIVIRUS_ESCAN,
    // ];

    // let firewall_entries: &[MenuId] = &[
    //     MenuId::FIREWALL_ZONEALARM,
    //     MenuId::FIREWALL_GLASSWIRE,
    //     MenuId::FIREWALL_COMODO,
    //     MenuId::FIREWALL_TINYWALL,
    // ];

    // let tools_entries: &[MenuId] = &[
    //     MenuId::TOOLS_PEID,
    //     MenuId::TOOLS_RESOURCE_HACKER,
    //     MenuId::TOOLS_DIE,
    //     MenuId::TOOLS_BYTECODE_VIEWER,
    //     MenuId::TOOLS_PROCESS_MONITOR,
    //     MenuId::TOOLS_PROCESS_EXPLORER,
    //     MenuId::TOOLS_TCPVIEW,
    //     MenuId::TOOLS_WIRESHARK,
    //     MenuId::TOOLS_PE_TOOLS,
    //     MenuId::TOOLS_SPYXX,
    // ];

    let mut pause = utf16_null!("Pause");
    let _resume = utf16_null!("Resume");
    let mut vm_guest = utf16_null!("VM guest process");
    // let mut debugger = utf16_null!("Debugger");
    // let mut antivirus = utf16_null!("Antivirus");
    // let mut firewall = utf16_null!("Firewall");
    let mut about = utf16_null!("About");
    let mut exit = utf16_null!("Exit");

    let guest_submenu: HMENU = CreatePopupMenu();
    append_menu(guest_submenu, guest_entries);

    // let debugger_submenu: HMENU = CreatePopupMenu();
    // append_menu(debugger_submenu, debugger_entries);

    // let antivirus_submenu: HMENU = CreatePopupMenu();
    // append_menu(antivirus_submenu, antivirus_entries);

    // let firewall_submenu: HMENU = CreatePopupMenu();
    // append_menu(firewall_submenu, firewall_entries);

    // let tools_submenu: HMENU = CreatePopupMenu();
    // append_menu(tools_submenu, tools_entries);

    let menu: HMENU = CreatePopupMenu();
    AppendMenuW(
        menu,
        MF_STRING,
        MenuId::PAUSE as usize,
        PWSTR(pause.as_mut_ptr()),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, None);
    AppendMenuW(
        menu,
        MF_STRING | MF_POPUP,
        guest_submenu as usize,
        PWSTR(vm_guest.as_mut_ptr()),
    );
    // AppendMenuW(
    //     menu,
    //     MF_STRING | MF_POPUP,
    //     debugger_submenu as usize,
    //     PWSTR(debugger.as_mut_ptr()),
    // );
    // AppendMenuW(
    //     menu,
    //     MF_STRING | MF_POPUP,
    //     antivirus_submenu as usize,
    //     PWSTR(antivirus.as_mut_ptr()),
    // );
    // AppendMenuW(
    //     menu,
    //     MF_STRING | MF_POPUP,
    //     firewall_submenu as usize,
    //     PWSTR(firewall.as_mut_ptr()),
    // );
    AppendMenuW(menu, MF_SEPARATOR, 0, None);
    AppendMenuW(
        menu,
        MF_STRING,
        MenuId::ABOUT as usize,
        PWSTR(about.as_mut_ptr()),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        MenuId::EXIT as usize,
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
