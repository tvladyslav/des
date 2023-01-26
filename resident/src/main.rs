#![windows_subsystem = "windows"]

use windows::{
    w,
    // core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    },
    Win32::UI::WindowsAndMessaging::*,
};

extern crate sha2;

#[macro_use]
extern crate num_derive;
use num_traits::FromPrimitive;

mod menu_entry;
mod release;

mod config;
use crate::config::DEFAULT_PROCESS;

mod utf16;
use utf16::{to_pcwstr, to_utf16};

mod menu_state;
use menu_state::MenuState;

mod menu_ids;
use menu_ids::MenuId;

mod menu_tray;
use menu_tray::MenuTray;

#[macro_use]
mod macros;

const TRAY_ICON_ID: u32 = 5;
const TRAY_MESSAGE: u32 = WM_APP + 1;
const LRESULT_SUCCESS: LRESULT = LRESULT(0);

// Main menu
static mut MENU_TRAY_ACTIVE: MenuTray = MenuTray::new();
static mut MENU_TRAY_PAUSED: MenuTray = MenuTray::new();
static mut MENU_STATE: MenuState = MenuState::new();

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    let module_handle: HINSTANCE;
    let icon: HICON;
    let cursor: HCURSOR;
    let active_icon_res = windows::core::PCWSTR(17 as *const u16);
    unsafe {
        MENU_STATE.init_menu_entries();

        module_handle = GetModuleHandleW(None)?;
        assert!(!module_handle.is_invalid());

        icon = LoadIconW(
            module_handle,
            active_icon_res
        )?;
        assert!(!icon.is_invalid());

        cursor = LoadCursorW(None, IDC_ARROW)?;
        assert!(!cursor.is_invalid());

        for m in DEFAULT_PROCESS {
            let res = MENU_STATE.start_process(m);
            if let Err(e) = res {
                let err: String = "Can't autorun default processes. ".to_string() + &e.to_string();
                MessageBoxW(HWND(0), to_pcwstr(&err).1, w!("Error"), MB_OK | MB_ICONERROR);
                break;
            }
        }

        MENU_TRAY_ACTIVE.create_menu_active(&MENU_STATE)?;
        MENU_TRAY_PAUSED.create_menu_paused()?;
    }

    let class_name = w!("notify_icon_class");

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
        lpszClassName: class_name,
        hIconSm: icon,

        ..Default::default()
    };

    let atom: u16 = execute!(RegisterClassExW(&win_class))?;
    assert!(atom != 0);

    let win_handle: HWND = execute!(CreateWindowExW(
        Default::default(),
        class_name,
        w!("The window"),
        WS_DISABLED,  // WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        None,
        None,
        module_handle,
        None,
    ))?;

    let mut tray_data: NOTIFYICONDATAW = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: win_handle,
        uID: TRAY_ICON_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: TRAY_MESSAGE,
        hIcon: icon,

        ..Default::default()
    };

    {
        // Yes, we have to tiptoe around sz_tip to assure it's length is exactly 128 bytes
        let mut sz_tip: Vec<u16> = Vec::with_capacity(128);
        sz_tip.extend_from_slice(unsafe { w!("Hostile environment imitator").as_wide() } );
        sz_tip.resize(128, 0);
        tray_data.szTip.clone_from_slice(&sz_tip);
    }

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

    let is_tray_icon_deleted: BOOL = execute!(Shell_NotifyIconW(NIM_DELETE, &tray_data))?;
    assert!(is_tray_icon_deleted.as_bool());

    // let is_icon_deleted: BOOL = execute!(DestroyIcon(icon))?;
    // assert!(is_icon_deleted.as_bool());

    Ok(())
}

unsafe fn flip_menu_state(context_menu: HMENU, menu_item: MenuId) -> std::io::Result<()> {
    let is_active = MENU_STATE.is_process_active(&menu_item);

    if is_active {
        MENU_STATE.stop_process(&menu_item)?;
        CheckMenuItem(context_menu, menu_item as u32, MF_UNCHECKED.0);
    } else {
        MENU_STATE.start_process(&menu_item)?;
        CheckMenuItem(context_menu, menu_item as u32, MF_CHECKED.0);
    }
    Ok(())
}

unsafe extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        TRAY_MESSAGE => match LOWORD!(lparam) {
            // WM_LBUTTONUP => {
            //     ShowWindow(window, SW_RESTORE);
            //     0
            // }
            WM_LBUTTONUP | WM_RBUTTONUP => {
                let mut point: POINT = Default::default();
                GetCursorPos(&mut point);
                let menu_handle = if MENU_STATE.is_paused() {
                    *MENU_TRAY_PAUSED
                } else {
                    *MENU_TRAY_ACTIVE
                };
                handle_popup_menu(window, point, menu_handle);
                LRESULT_SUCCESS
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        },
        WM_COMMAND => {
            let lo_wparam: MenuId = FromPrimitive::from_u32(LOWORD!(wparam)).unwrap();
            match lo_wparam {
                MenuId::PAUSE => {
                    // TODO: Modify menu UI
                    let res = MENU_STATE.pause();
                    if let Err(e) = res {
                        let err: String = "Can't pause processes. ".to_string() + &e.to_string();
                        MessageBoxW(window, to_pcwstr(&err).1, w!("Error"), MB_OK | MB_ICONERROR);
                    }
                    LRESULT_SUCCESS
                }
                MenuId::RESUME => {
                    // TODO: Modify menu UI
                    let res = MENU_STATE.resume();
                    if let Err(e) = res {
                        let err: String = "Can't resume processes ".to_string() + &e.to_string();
                        MessageBoxW(window, to_pcwstr(&err).1, w!("Error"), MB_OK | MB_ICONERROR);
                    }
                    LRESULT_SUCCESS
                }
                MenuId::GUEST
                | MenuId::DEBUGGER
                | MenuId::ANTIVIRUS
                | MenuId::FIREWALL
                | MenuId::TOOLS
                => {
                    // This should never happen. Assert?
                    MessageBoxW(window, w!("Selected non-active menu items."), w!("Error"), MB_OK | MB_ICONERROR);
                    LRESULT_SUCCESS
                }
                MenuId::GUEST_VIRTUALBOX
                | MenuId::GUEST_VMWARE
                | MenuId::GUEST_PARALLELS
                | MenuId::GUEST_HYPERV
                | MenuId::GUEST_VIRTUAL_PC
                | MenuId::DEBUGGER_OLLY
                | MenuId::DEBUGGER_WINDBG
                | MenuId::DEBUGGER_X64DBG
                | MenuId::DEBUGGER_IDA
                | MenuId::DEBUGGER_IMMUNITY
                | MenuId::DEBUGGER_RADARE2
                | MenuId::DEBUGGER_BINARY_NINJA
                // | MenuId::ANTIVIRUS_AVAST
                | MenuId::ANTIVIRUS_AVIRA
                // | MenuId::ANTIVIRUS_BITDEFENDER
                // | MenuId::ANTIVIRUS_DRWEB
                // | MenuId::ANTIVIRUS_ESET_NOD32
                | MenuId::ANTIVIRUS_ESCAN
                | MenuId::ANTIVIRUS_FORTINET
                // | MenuId::ANTIVIRUS_FSECURE
                | MenuId::ANTIVIRUS_GDATA
                | MenuId::ANTIVIRUS_K7
                // | MenuId::ANTIVIRUS_KASPERSKY
                // | MenuId::ANTIVIRUS_MALWAREBYTES
                | MenuId::ANTIVIRUS_MCAFEE
                // | MenuId::ANTIVIRUS_NORTON
                // | MenuId::ANTIVIRUS_PANDA
                // | MenuId::ANTIVIRUS_SOPHOS
                // | MenuId::ANTIVIRUS_TREND_MICRO
                // | MenuId::ANTIVIRUS_WEBROOT
                | MenuId::FIREWALL_COMODO
                | MenuId::FIREWALL_GLASSWIRE
                | MenuId::FIREWALL_TINYWALL
                | MenuId::FIREWALL_ZONEALARM
                | MenuId::TOOLS_PEID
                | MenuId::TOOLS_RESOURCE_HACKER
                | MenuId::TOOLS_DIE
                | MenuId::TOOLS_DEBUG_VIEW
                | MenuId::TOOLS_PROCESS_MONITOR
                | MenuId::TOOLS_PROCESS_EXPLORER
                | MenuId::TOOLS_TCPVIEW
                | MenuId::TOOLS_WIRESHARK
                | MenuId::TOOLS_PE_TOOLS
                | MenuId::TOOLS_SPYXX
                | MenuId::TOOLS_CTK_RES_EDIT
                | MenuId::TOOLS_XN_RES_EDITOR
                => {
                    // TODO: Is there a nice way to bind this variable?
                    let res = flip_menu_state(*MENU_TRAY_ACTIVE, lo_wparam);
                    if let Err(e) = res {
                        let err: String = "Can't finish your request. ".to_string() + &e.to_string();
                        MessageBoxW(window, to_pcwstr(&err).1, w!("Error"), MB_OK | MB_ICONERROR);
                    }
                    LRESULT_SUCCESS
                }
                MenuId::ABOUT => {
                    let text = w!(
                        "Version: 1.1.0\n \
                        Author: Vladyslav Tsilytskyi\n \
                        Tray icon: Chenyu Wang\n \
                        License: GPLv3\n \
                        DES application does one simple thing - it\n \
                        spawns many dummy processes that look like\n \
                        hostile for malware and viruses.");
                    MessageBoxW(window, text, w!("About"), MB_OK);
                    LRESULT_SUCCESS
                }
                MenuId::EXIT => {
                    SendMessageW(window, WM_CLOSE, WPARAM(0), LPARAM(0));
                    LRESULT_SUCCESS
                }
            }
        }
        WM_PAINT => {
            ValidateRect(window, None);
            LRESULT_SUCCESS
        }
        WM_CREATE => LRESULT_SUCCESS,
        WM_DESTROY => {
            // We are in a process of destruction
            exit_routine()
        }
        _ => DefWindowProcW(window, message, wparam, lparam), // WM_CLOSE lands here
    }
}

unsafe fn exit_routine() -> LRESULT {
    // https://learn.microsoft.com/en-us/windows/win32/learnwin32/closing-the-window
    MENU_TRAY_ACTIVE.destroy();
    MENU_TRAY_PAUSED.destroy();
    MENU_STATE.destroy();
    PostQuitMessage(0); // This spawns WM_QUIT which terminates main loop
    LRESULT_SUCCESS
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
        None,
    );
    PostMessageW(window, WM_NULL, WPARAM(0), LPARAM(0));
}
