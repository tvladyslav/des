use std::ops::Deref;

use windows::Win32::UI::WindowsAndMessaging::HICON;

use crate::menu_tray::MenuTray;
use crate::menu_state::MenuState;

// This struct keeps 2 tray menu versions and decides which one to show now
pub struct TrayMenuState {
    is_paused: bool,
    menu_tray_active: MenuTray,
    menu_tray_paused: MenuTray,
    icon_active: HICON,
    icon_paused: HICON,
}

impl Deref for TrayMenuState {
    type Target = MenuTray;
    fn deref(&self) -> &Self::Target {
        if self.is_paused {
            &self.menu_tray_paused
        } else {
            &self.menu_tray_active
        }
    }
}

impl TrayMenuState {
    pub const fn new() -> TrayMenuState {
        TrayMenuState {is_paused: false, menu_tray_active: MenuTray::new(), menu_tray_paused: MenuTray::new(),
        icon_active: HICON(0), icon_paused: HICON(0)}
    }

    pub unsafe fn init(&mut self, menu_state: &MenuState, autostart: bool, icon_active: HICON, icon_paused: HICON)
    -> windows::core::Result<()> {
        self.icon_active = icon_active;
        self.icon_paused = icon_paused;
        self.menu_tray_active.create_menu_active(menu_state, autostart)?;
        self.menu_tray_paused.create_menu_paused(autostart)?;
        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        self.menu_tray_active.destroy();
        self.menu_tray_paused.destroy();
    }

    pub fn get_icon(&self) -> HICON {
        if self.is_paused {
            self.icon_paused
        } else {
            self.icon_active
        }
    }

    pub unsafe fn pause(&mut self, is_autorun_enabled: bool) {
        self.is_paused = true;
        self.menu_tray_paused.update_autorun_item(is_autorun_enabled);
    }

    pub unsafe fn resume(&mut self, is_autorun_enabled: bool) {
        self.is_paused = false;
        self.menu_tray_active.update_autorun_item(is_autorun_enabled);
    }
}
