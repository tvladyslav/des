use std::ops::Deref;

use utf16_lit::utf16_null;
use windows::Win32::Foundation::PWSTR;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::menu_ids::MenuId;
use crate::menu_state::MenuState;

pub struct MenuTray {
    pub menu: HMENU
}

impl Deref for MenuTray {
    type Target = HMENU;
    fn deref(&self) -> &Self::Target {
        &self.menu
    }
}

impl MenuTray {
    pub const fn new() -> MenuTray {
        MenuTray { menu: -1 }
    }

    unsafe fn append_last_entries(&mut self) {
        assert!(self.menu != -1);
        let mut about = utf16_null!("About");
        let mut exit = utf16_null!("Exit");
        AppendMenuW(self.menu, MF_SEPARATOR, 0, None);
        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::ABOUT as usize,
            PWSTR(about.as_mut_ptr()),
        );
        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::EXIT as usize,
            PWSTR(exit.as_mut_ptr()),
        );
    }

    pub unsafe fn create_menu_paused(&mut self) {
        assert!(self.menu == -1);
        let mut resume = utf16_null!("Resume");

        self.menu = CreatePopupMenu();

        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::RESUME as usize,
            PWSTR(resume.as_mut_ptr()),
        );
        self.append_last_entries();
    }

    pub unsafe fn create_menu_active(&mut self, menu_state: &MenuState) {
        assert!(self.menu == -1);
        let guest_entries: &[MenuId] = &[
            MenuId::GUEST_VIRTUALBOX,
            MenuId::GUEST_VMWARE,
            MenuId::GUEST_PARALLELS,
            MenuId::GUEST_HYPERV,
            MenuId::GUEST_VIRTUAL_PC,
        ];

        let debugger_entries: &[MenuId] = &[
            MenuId::DEBUGGER_OLLY,
            MenuId::DEBUGGER_WINDBG,
            MenuId::DEBUGGER_X64DBG,
            MenuId::DEBUGGER_IDA,
            MenuId::DEBUGGER_IMMUNITY,
            MenuId::DEBUGGER_RADARE2,
            MenuId::DEBUGGER_BINARY_NINJA,
        ];

        let antivirus_entries: &[MenuId] = &[
            MenuId::ANTIVIRUS_AVIRA,
            MenuId::ANTIVIRUS_ESCAN,
            MenuId::ANTIVIRUS_FORTINET,
            MenuId::ANTIVIRUS_GDATA,
            MenuId::ANTIVIRUS_K7,
            MenuId::ANTIVIRUS_MCAFEE,
        ];

        // let firewall_entries: &[MenuId] = &[
        //     MenuId::FIREWALL_ZONEALARM,
        //     MenuId::FIREWALL_GLASSWIRE,
        //     MenuId::FIREWALL_COMODO,
        //     MenuId::FIREWALL_TINYWALL,
        // ];

        let tools_entries: &[MenuId] = &[
            MenuId::TOOLS_PEID,
            MenuId::TOOLS_RESOURCE_HACKER,
            MenuId::TOOLS_DIE,
            MenuId::TOOLS_DEBUG_VIEW,
            MenuId::TOOLS_PROCESS_MONITOR,
            MenuId::TOOLS_PROCESS_EXPLORER,
            MenuId::TOOLS_TCPVIEW,
            MenuId::TOOLS_WIRESHARK,
            MenuId::TOOLS_PE_TOOLS,
            MenuId::TOOLS_SPYXX,
            MenuId::TOOLS_CTK_RES_EDIT,
            MenuId::TOOLS_XN_RES_EDITOR,
        ];

        let mut pause = utf16_null!("Pause");
        let mut vm_guest = utf16_null!("VM guest process");
        let mut debugger = utf16_null!("Debugger");
        let mut antivirus = utf16_null!("Antivirus");
        // let mut firewall = utf16_null!("Firewall");
        let mut tools = utf16_null!("Tools");

        let guest_submenu: HMENU = CreatePopupMenu();
        append_menu(guest_submenu, menu_state, guest_entries);

        let debugger_submenu: HMENU = CreatePopupMenu();
        append_menu(debugger_submenu, menu_state, debugger_entries);

        let antivirus_submenu: HMENU = CreatePopupMenu();
        append_menu(antivirus_submenu, menu_state, antivirus_entries);

        // let firewall_submenu: HMENU = CreatePopupMenu();
        // append_menu(firewall_submenu, firewall_entries);

        let tools_submenu: HMENU = CreatePopupMenu();
        append_menu(tools_submenu, menu_state, tools_entries);

        self.menu = CreatePopupMenu();
        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::PAUSE as usize,
            PWSTR(pause.as_mut_ptr()),
        );
        AppendMenuW(self.menu, MF_SEPARATOR, 0, None);
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            guest_submenu as usize,
            PWSTR(vm_guest.as_mut_ptr()),
        );
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            debugger_submenu as usize,
            PWSTR(debugger.as_mut_ptr()),
        );
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            antivirus_submenu as usize,
            PWSTR(antivirus.as_mut_ptr()),
        );
        // AppendMenuW(
        //     self.menu,
        //     MF_STRING | MF_POPUP,
        //     firewall_submenu as usize,
        //     PWSTR(firewall.as_mut_ptr()),
        // );
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            tools_submenu as usize,
            PWSTR(tools.as_mut_ptr()),
        );
        self.append_last_entries();
    }

    pub unsafe fn destroy(&mut self) {
        DestroyMenu(self.menu);
    }
}

fn to_utf16(text: &str) -> Vec<u16> {
    return text.encode_utf16().chain(std::iter::once(0)).collect();
}

fn append_menu(menu: HMENU, menu_state: &MenuState, entry_ids: &[MenuId]) {
    for e in entry_ids {
        let bird = if menu_state.is_process_active(e) {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        unsafe {
            AppendMenuW(
                menu,
                bird | MF_STRING,
                *e as usize,
                PWSTR(to_utf16(menu_state.get_name(e)).as_mut_ptr()),
            )
        };
    }
}