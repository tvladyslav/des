use std::ops::Deref;

use windows::w;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::menu_ids::MenuId;
use crate::menu_state::MenuState;
use crate::switch::Switch;
use crate::utf16::to_pcwstr;

pub struct MenuTray {
    menu: HMENU
}

impl Deref for MenuTray {
    type Target = HMENU;
    fn deref(&self) -> &Self::Target {
        &self.menu
    }
}

impl MenuTray {
    pub const fn new() -> MenuTray {
        // TODO: try is_invalid
        MenuTray { menu: HMENU(-1) }
    }

    pub fn is_initialized(&self) -> bool {
        self.menu.0 != -1
    }

    pub unsafe fn update_autorun_item(&self, is_enabled: bool) {
        let autostart_tick = if is_enabled {
            MF_CHECKED.0
        } else {
            MF_UNCHECKED.0
        };
        CheckMenuItem(self.menu, MenuId::AUTOSTART as u32, autostart_tick);
    }

    unsafe fn append_last_entries(&mut self, autostart: bool) {
        let autostart_tick = if autostart {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        assert!(self.is_initialized());
        AppendMenuW(self.menu, MF_SEPARATOR, 0, None);
        AppendMenuW(
            self.menu,
            MF_STRING | autostart_tick,
            MenuId::AUTOSTART as usize,
            w!("Autostart"),
        );
        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::ABOUT as usize,
            w!("About"),
        );
        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::EXIT as usize,
            w!("Exit"),
        );
    }

    pub unsafe fn create_menu_paused(&mut self, autostart: bool) -> windows::core::Result<()> {
        assert!(!self.is_initialized());

        self.menu = CreatePopupMenu()?;

        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::RESUME as usize,
            w!("Resume"),
        );
        self.append_last_entries(autostart);
        Ok(())
    }

    pub unsafe fn create_menu_active(&mut self, menu_state: &MenuState, autostart: bool) -> windows::core::Result<()> {
        assert!(!self.is_initialized());
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

        let firewall_entries: &[MenuId] = &[
                MenuId::FIREWALL_COMODO,
                MenuId::FIREWALL_GLASSWIRE,
                MenuId::FIREWALL_TINYWALL,
                MenuId::FIREWALL_ZONEALARM,
        ];

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

        let guest_submenu: HMENU = CreatePopupMenu()?;
        append_menu(guest_submenu, menu_state, guest_entries);

        let debugger_submenu: HMENU = CreatePopupMenu()?;
        append_menu(debugger_submenu, menu_state, debugger_entries);

        let antivirus_submenu: HMENU = CreatePopupMenu()?;
        append_menu(antivirus_submenu, menu_state, antivirus_entries);

        let firewall_submenu: HMENU = CreatePopupMenu()?;
        append_menu(firewall_submenu, menu_state, firewall_entries);

        let tools_submenu: HMENU = CreatePopupMenu()?;
        append_menu(tools_submenu, menu_state, tools_entries);

        self.menu = CreatePopupMenu()?;
        AppendMenuW(
            self.menu,
            MF_STRING,
            MenuId::PAUSE as usize,
            w!("Pause"),
        );
        AppendMenuW(self.menu, MF_SEPARATOR, 0, None);
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            guest_submenu.0 as usize,
            w!("VM guest process"),
        );
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            debugger_submenu.0 as usize,
            w!("Debugger"),
        );
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            antivirus_submenu.0 as usize,
            w!("Antivirus"),
        );
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            firewall_submenu.0 as usize,
            w!("Firewall"),
        );
        AppendMenuW(
            self.menu,
            MF_STRING | MF_POPUP,
            tools_submenu.0 as usize,
            w!("Tools"),
        );
        self.append_last_entries(autostart);
        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        DestroyMenu(self.menu);
    }
}

fn append_menu(menu: HMENU, menu_state: &MenuState, entry_ids: &[MenuId]) {
    for e in entry_ids {
        let bird = if menu_state.is_enabled(e) {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        unsafe {
            AppendMenuW(
                menu,
                bird | MF_STRING,
                *e as usize,
                to_pcwstr(menu_state.get_name(e)).1,
            )
        };
    }
}