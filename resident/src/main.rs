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

use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

mod state;
use state::*;

#[macro_use]
mod macros;

const TRAY_ICON_ID: u32 = 5;
const TRAY_MESSAGE: u32 = WM_APP + 1;

const MENU_GUEST: u32 = 1;
const MENU_GUEST_VIRTUALBOX: u32 = 11;
const MENU_GUEST_VMWARE: u32 = 12;
const MENU_GUEST_PARALLELS: u32 = 13;
const MENU_GUEST_HYPERV: u32 = 14;
const MENU_GUEST_VIRTUAL_PC: u32 = 15;
// const MENU_DISASSEMBLER: u32 = 2;
const MENU_DEBUGGER: u32 = 3;
const MENU_DEBUGGER_OLLY: u32 = 31;
const MENU_DEBUGGER_WINDBG: u32 = 32;
const MENU_DEBUGGER_X64DBG: u32 = 33;
const MENU_DEBUGGER_IDA: u32 = 34;
const MENU_DEBUGGER_IMMUNITY: u32 = 35;
const MENU_ANTIVIRUS: u32 = 4;
const MENU_ANTIVIRUS_BITDEFENDER: u32 = 41;
const MENU_ANTIVIRUS_NORTON: u32 = 42;
const MENU_ANTIVIRUS_TREND_MICRO: u32 = 43;
const MENU_ANTIVIRUS_KASPERSKY: u32 = 44;
const MENU_ANTIVIRUS_AVIRA: u32 = 45;
const MENU_ANTIVIRUS_AVAST: u32 = 46;
const MENU_ANTIVIRUS_MCAFEE: u32 = 47;
const MENU_ANTIVIRUS_DRWEB: u32 = 48;
const MENU_ANTIVIRUS_ESET_NOD32: u32 = 49;
const MENU_ANTIVIRUS_SOPHOS: u32 = 50;
const MENU_ANTIVIRUS_PANDA: u32 = 51;
const MENU_ANTIVIRUS_WEBROOT: u32 = 52;
const MENU_ANTIVIRUS_MALWAREBYTES: u32 = 53;
const MENU_ANTIVIRUS_FSECURE: u32 = 54;
const MENU_ANTIVIRUS_GDATA: u32 = 55;
const MENU_ANTIVIRUS_ESCAN: u32 = 56;
const MENU_FIREWALL: u32 = 8;
const MENU_FIREWALL_ZONEALARM: u32 = 81;
const MENU_FIREWALL_GLASSWIRE: u32 = 82;
const MENU_FIREWALL_COMODO: u32 = 83;
const MENU_FIREWALL_TINYWALL: u32 = 84;
const MENU_TOOLS: u32 = 90;
/*
const MENU_TOOLS_PEID: u32 = 91;
const MENU_TOOLS_RESOURCE_HACKER: u32 = 92;
const MENU_TOOLS_DIE: u32 = 93;
const MENU_TOOLS_BYTECODE_VIEWER: u32 = 94;
const MENU_TOOLS_PROCESS_MONITOR: u32 = 95;
const MENU_TOOLS_PROCESS_EXPLORER: u32 = 96;
const MENU_TOOLS_TCPVIEW: u32 = 97;
const MENU_TOOLS_WIRESHARK: u32 = 98;
const MENU_TOOLS_PE_TOOLS: u32 = 99;
*/
const MENU_ABOUT: u32 = 10;
const MENU_EXIT: u32 = 77;
const MENU_PAUSE: u32 = 78;
const MENU_RESUME: u32 = 79;

const HOME_FOLDER: &str = "./target/debug/";

// Selected by default
const SELECTED_VALUES: [u32; 4] = [MENU_GUEST_VIRTUALBOX, MENU_DEBUGGER_IDA, MENU_FIREWALL_ZONEALARM, MENU_ANTIVIRUS_MCAFEE];

// Main menu
static mut MENU: HMENU = 0;

static mut IS_PAUSED: bool = false;
static mut PAUSED_PROCESSES: Vec<u32> = Vec::new();

// const BTreeMap::new is available since 1.66
// It is not possible to initalize map in place without external crates
// which enables lazy evaluations.
// I prefer less dependencies to lazy initialization.
static mut MENU_ENTRIES: BTreeMap<u32, MenuEntry> = BTreeMap::new();

// Must be unsafe because modifies mutable static variable
unsafe fn init_menu_entries() {
    MENU_ENTRIES.insert(MENU_GUEST_VIRTUALBOX, MenuEntry {
        id: MENU_GUEST_VIRTUALBOX,
        entry_text: "VirtualBox",
        processes: vec![
            ("VBoxTray.exe", None),    // VirtualBox Guest Additions Tray Application
            ("VBoxService.exe", None), // VirtualBox Guest Additions Service
        ],
    });
    MENU_ENTRIES.insert(MENU_GUEST_VMWARE, MenuEntry {
        id: MENU_GUEST_VMWARE,
        entry_text: "VMware",
        processes: vec![
            ("vmacthlp.exe", None), // VMware Activation Helper
            ("vmtoolsd.exe", None), // VMware Tools Core Service
        ],
    });
    MENU_ENTRIES.insert(MENU_GUEST_PARALLELS, MenuEntry {
        id: MENU_GUEST_PARALLELS,
        entry_text: "Parallels",
        processes: vec![
            ("prl_cc.exe", None),        // Parallels Control Center
            ("prl_tools.exe", None),     // Parallels Tools
            ("SharedIntApp.exe", None),  // Parallels Server/Desktop (runtime switch)
        ],
    });
    MENU_ENTRIES.insert(MENU_GUEST_HYPERV, MenuEntry {
        id: MENU_GUEST_HYPERV,
        entry_text: "Hyper-V",
        processes: vec![
            ("VmComputeAgent.exe", None), // Hyper-V Guest Compute Service
        ],
    });
    MENU_ENTRIES.insert(MENU_GUEST_VIRTUAL_PC, MenuEntry {
        id: MENU_GUEST_VIRTUAL_PC,
        entry_text: "Windows Virtual PC",
        processes: vec![
            ("vmusrvc.exe", None),  // Virtual Machine User Services
            ("vmsrvc.exe", None),   // Virtual Machine Services
        ],
    });
    MENU_ENTRIES.insert(MENU_DEBUGGER_OLLY, MenuEntry {
        id: MENU_DEBUGGER_OLLY,
        entry_text: "OllyDBG",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_DEBUGGER_WINDBG, MenuEntry {
        id: MENU_DEBUGGER_WINDBG,
        entry_text: "WinDBG",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_DEBUGGER_X64DBG, MenuEntry {
        id: MENU_DEBUGGER_X64DBG,
        entry_text: "x64dbg",
        // process_name: "x64dbg.exe",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_DEBUGGER_IDA, MenuEntry {
        id: MENU_DEBUGGER_IDA,
        entry_text: "IDA Pro",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_DEBUGGER_IMMUNITY, MenuEntry {
        id: MENU_DEBUGGER_IMMUNITY,
        entry_text: "Immunity",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_ANTIVIRUS_AVAST, MenuEntry {
        id: MENU_ANTIVIRUS_AVAST,
        entry_text: "Avast",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_ANTIVIRUS_AVIRA, MenuEntry {
        id: MENU_ANTIVIRUS_AVIRA,
        entry_text: "Avira",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_ANTIVIRUS_ESCAN, MenuEntry {
        id: MENU_ANTIVIRUS_ESCAN,
        entry_text: "eScan",
        processes: vec![
            ("avpmapp.exe", None),  // eScan File Monitoring System
            ("econceal.exe", None), // eConceal Service
            ("escanmon.exe", None), // eScan Monitoring Tray
            ("escanpro.exe", None), // eScan Protection Center
    ],
    });
    MENU_ENTRIES.insert(MENU_FIREWALL_ZONEALARM, MenuEntry {
        id: MENU_FIREWALL_ZONEALARM,
        entry_text: "ZoneAlarm",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_FIREWALL_GLASSWIRE, MenuEntry {
        id: MENU_FIREWALL_GLASSWIRE,
        entry_text: "GlassWire",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_FIREWALL_COMODO, MenuEntry {
        id: MENU_FIREWALL_COMODO,
        entry_text: "Comodo",
        processes: Vec::new(),
    });
    MENU_ENTRIES.insert(MENU_FIREWALL_TINYWALL, MenuEntry {
        id: MENU_FIREWALL_TINYWALL,
        entry_text: "TinyWall",
        processes: Vec::new(),
    });
}

#[must_use]
unsafe fn stop_running_processes() -> Vec<u32> {
    let mut stopped_processes: Vec<u32> = Vec::new();
    for (id, me) in &mut MENU_ENTRIES {
        for (_name, child) in &mut me.processes {
            if let Some(c) = child {
                stopped_processes.push(*id);
                let res = c.kill();
                debug_assert!(res.is_ok());
            }
        }
    }

    stopped_processes
}

fn to_utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().chain(std::iter::once(0)).collect();
}

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    unsafe { init_menu_entries() };

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
        create_menu(&mut MENU);
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
    // let state: u32 = GetMenuState(context_menu, menu_item, MF_BYCOMMAND);

    let menu_entry: &mut MenuEntry = MENU_ENTRIES.get_mut(&menu_item).expect("This menu entry doesn't exist");
    debug_assert!(menu_item == menu_entry.id, "Id mismatch, revisit the MENU_ENTRIES map please.");

    for (process_name, process_child) in &mut menu_entry.processes {
        let process_path: String = String::from(HOME_FOLDER) + process_name;

        *process_child = match process_child {
            Some(proc) => {
                CheckMenuItem(context_menu, menu_item, MF_UNCHECKED);
                let res1 = proc.kill();
                debug_assert!(res1.is_ok());
                let res2 = fs::remove_file(process_path);
                debug_assert!(res2.is_ok());
                None
            },
            None => {
                CheckMenuItem(context_menu, menu_item, MF_CHECKED);
                // TODO: Check stub's SHA3

                let res = fs::copy(String::from(HOME_FOLDER) + "des-stub.exe", &process_path);
                debug_assert!(res.is_ok());
                Some(Command::new(&process_path).arg("arg1").spawn().expect("Failed to start command!"))
                // menu_entry.process_child = Some(Command::new("ping").args(["/n", "10", "bash.im"]).spawn().expect("Failed to start command!"));
            }
        }
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
                handle_popup_menu(window, point, MENU);

                0
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        },
        WM_COMMAND => match LOWORD!(wparam) {
            MENU_PAUSE | MENU_RESUME => {
                // TODO: Pause all or resume all
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
                flip_menu_state(MENU, m);
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
    DestroyMenu(MENU);
    let _ignored = stop_running_processes();
    MENU_ENTRIES.clear();
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
                PWSTR(to_utf16(MENU_ENTRIES.get(e).unwrap().entry_text).as_mut_ptr()),
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
