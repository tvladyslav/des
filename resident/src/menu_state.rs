use crate::menu_entry::*;
use crate::menu_ids::MenuId;

use std::collections::BTreeMap;

pub struct MenuState<'a> {
    m: BTreeMap<MenuId, MenuEntry<'a>>,
    is_paused: bool,
    paused_process_list: Vec<MenuId>,
}

impl <'m> MenuState<'m> {
    pub const fn new() -> MenuState<'m> {
        MenuState { m: BTreeMap::new(), is_paused: false, paused_process_list: Vec::new() }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    fn stop_all_running_processes(&mut self) -> std::io::Result<()> {
        let active_process_list = self.get_active_process_list();
        self.stop_running_processes(&active_process_list)
    }

    fn stop_running_processes(&mut self, process_list: &Vec<MenuId>) -> std::io::Result<()> {
        for id in process_list {
            self.stop_process(id)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn get_name(&self, key: &MenuId) -> &'m str {
        // TODO: remove unwrap
        self.m.get(key).unwrap().get_name()
    }

    #[must_use]
    pub fn is_process_active(&self, id: &MenuId) -> bool {
        // TODO: remove unwrap
        self.m.get(id).unwrap().is_process_active()
    }

    #[must_use]
    fn get_active_process_list(&self) -> Vec<MenuId> {
        let mut active_process_list: Vec<MenuId> = Vec::new();
        for (id, me) in &self.m {
            if me.is_process_active() {
                active_process_list.push(*id);
            }
        }
        active_process_list
    }

    pub fn pause(&mut self) -> std::io::Result<()> {
        if !self.is_paused {
            let active_processes = self.get_active_process_list();
            self.paused_process_list.clone_from(&active_processes);
            self.stop_running_processes(&active_processes)?;
            self.is_paused = true;
        }
        Ok(())
    }

    pub fn resume(&mut self) -> std::io::Result<()> {
        if self.is_paused {
            let process_to_resume = self.paused_process_list.clone();
            for id in &process_to_resume {
                self.start_process(id)?;
            }
            self.paused_process_list.clear();
            self.is_paused = false;
        }
        Ok(())
    }

    pub fn start_process(&mut self, id: &MenuId) -> std::io::Result<()> {
        let menu_entry = self.m.get_mut(id).ok_or(std::io::ErrorKind::NotFound)?;
        menu_entry.start_process()
    }

    pub fn stop_process(&mut self, id: &MenuId) -> std::io::Result<()> {
        let menu_entry = self.m.get_mut(id).ok_or(std::io::ErrorKind::NotFound)?;
        menu_entry.stop_process()
    }

    pub fn destroy(&mut self) {
        let _ignored = self.stop_all_running_processes();
        self.m.clear();
        self.paused_process_list.clear();
    }

    pub fn init_menu_entries(&mut self) {
        self.m.insert(MenuId::GUEST_VIRTUALBOX, MenuEntry::new(
            "VirtualBox",
            vec![
                ("VBoxTray.exe", None),    // VirtualBox Guest Additions Tray Application
                ("VBoxService.exe", None), // VirtualBox Guest Additions Service
            ])
        );
        self.m.insert(MenuId::GUEST_VMWARE, MenuEntry::new(
            "VMware",
            vec![
                ("vmacthlp.exe", None),    // VMware Activation Helper
                ("vmtoolsd.exe", None),    // VMware Tools Core Service
                ("vmwaretray.exe", None),  // VMware Tools tray application
                ("vmware-tray.exe", None), // VMware Tray Process
                ("VMwareUser.exe", None),  // VMware Tools Service
            ])
        );
        self.m.insert(MenuId::GUEST_PARALLELS, MenuEntry::new(
            "Parallels",
            vec![
                ("prl_cc.exe", None),        // Parallels Control Center
                ("prl_tools.exe", None),     // Parallels Tools
                ("SharedIntApp.exe", None),  // Parallels Server/Desktop (runtime switch)
            ])
        );
        self.m.insert(MenuId::GUEST_HYPERV, MenuEntry::new(
            "Hyper-V",
            vec![
                ("VmComputeAgent.exe", None), // Hyper-V Guest Compute Service
            ])
        );
        self.m.insert(MenuId::GUEST_VIRTUAL_PC, MenuEntry::new(
            "Windows Virtual PC",
            vec![
                ("vmusrvc.exe", None),  // Virtual Machine User Services
                ("vmsrvc.exe", None),   // Virtual Machine Services
            ])
        );
        // self.m.insert(MenuId::DEBUGGER_OLLY, MenuEntry::new(
        //     "OllyDBG",
        //     vec![("ollydbg.exe", None)]
        // ));
        // self.m.insert(MenuId::DEBUGGER_WINDBG, MenuEntry::new(
        //     "WinDBG",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::DEBUGGER_X64DBG, MenuEntry::new(
        //     "x64dbg",
        //     // process_name: "x64dbg.exe",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::DEBUGGER_IDA, MenuEntry::new(
        //     "IDA Pro",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::DEBUGGER_IMMUNITY, MenuEntry::new(
        //     "Immunity",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::ANTIVIRUS_AVAST, MenuEntry::new(
        //     "Avast",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::ANTIVIRUS_AVIRA, MenuEntry::new(
        //     "Avira",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::ANTIVIRUS_ESCAN, MenuEntry::new(
        //     "eScan",
        //     vec![
        //         ("avpmapp.exe", None),  // eScan File Monitoring System
        //         ("econceal.exe", None), // eConceal Service
        //         ("escanmon.exe", None), // eScan Monitoring Tray
        //         ("escanpro.exe", None), // eScan Protection Center
        //     ])
        // );
        // self.m.insert(MenuId::FIREWALL_ZONEALARM, MenuEntry::new(
        //     "ZoneAlarm",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::FIREWALL_GLASSWIRE, MenuEntry::new(
        //     "GlassWire",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::FIREWALL_COMODO, MenuEntry::new(
        //     "Comodo",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::FIREWALL_TINYWALL, MenuEntry::new(
        //     "TinyWall",
        //     Vec::new()
        // )
        // );
        // self.m.insert(MenuId::TOOLS_PEID, MenuEntry::new(
        //     "PEiD",
        //     vec![("PEiD.exe", None)]
        // ));
        // self.m.insert(MenuId::TOOLS_RESOURCE_HACKER, MenuEntry::new(
        //     "Resource hacker",
        //     vec![("ResourceHacker.exe", None)]
        // ));
        // self.m.insert(MenuId::TOOLS_DIE, MenuEntry::new(
        //     "Detect It Easy",
        //     vec![
        //         ("die.exe", None),
        //         ("diec.exe", None),
        //         ("diel.exe", None)
        //     ]
        // ));
/*


        MenuId::TOOLS_BYTECODE_VIEWER,
        MenuId::TOOLS_PROCESS_MONITOR,
        MenuId::TOOLS_PROCESS_EXPLORER, procexp.exe procexp64.exe procexp64a.exe
        MenuId::TOOLS_TCPVIEW,
        MenuId::TOOLS_WIRESHARK,
        MenuId::TOOLS_PE_TOOLS,
        MenuId::TOOLS_SPYXX,
        CTKResEdit.exe
        binaryninja.exe
*/
    }

}