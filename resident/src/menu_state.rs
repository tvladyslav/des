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
        self.m.insert(MenuId::DEBUGGER_OLLY, MenuEntry::new(
            "OllyDBG",
            vec![("ollydbg.exe", None)]
        ));
        self.m.insert(MenuId::DEBUGGER_WINDBG, MenuEntry::new(
            "WinDBG",
            vec![
                ("windbg.exe", None),
                // ("dbgsrv.exe", None),
                ("usbview.exe", None),
                ("logviewer.exe", None),
            ])
        );
        self.m.insert(MenuId::DEBUGGER_X64DBG, MenuEntry::new(
            "x64dbg",
            vec![("x64dbg.exe", None)]
        ));
        self.m.insert(MenuId::DEBUGGER_IDA, MenuEntry::new(
            "IDA Pro",
            vec![("ida64.exe", None)]
        ));
        self.m.insert(MenuId::DEBUGGER_IMMUNITY, MenuEntry::new(
            "Immunity",
            vec![("ImmunityDebugger.exe", None)]
        ));
        self.m.insert(MenuId::DEBUGGER_RADARE2, MenuEntry::new(
            "Radare 2",
            vec![("iaito.exe", None)]
        ));
        self.m.insert(MenuId::DEBUGGER_BINARY_NINJA, MenuEntry::new(
            "Binary ninja",
            vec![("binaryninja.exe", None)]
        ));
        self.m.insert(MenuId::ANTIVIRUS_AVIRA, MenuEntry::new(
            "Avira",
            vec![
                ("Avira.OptimizerHost.exe", None), // Avira Optimizer Host
                ("Avira Safe Shopping.exe", None), // Avira Safe Shopping add-on for browsers
                ("Avira.ServiceHost.exe", None),   // Avira Service Host
                ("Avira.SoftwareUpdater.ServiceHost.exe", None), // Avira Updater Service Host
                ("Avira.Spotlight.Service.exe", None),
                ("Avira.Systray.exe", None),       // Avira Launcher
                ("Avira.SystrayStartTrigger.exe", None), // Avira System Tray Service Start Trigger
                ("Avira.VpnService.exe", None),    // Avira Phantom VPN
                ("Avira.WebAppHost.exe", None),    // Avira Phantom VPN or WebAppHost
                ("ProtectedService.exe", None),    // Avira Protected Antimalware Service
                ("avscan.exe", None),              // Avira OnDemand File Scanner
                ("toastnotifier.exe", None),       // AVToastNotifier
                ("avupdate.exe", None),            // Updater for Avira products
                ("ipmgui.exe", None),              // In Product Messaging Application
                ("avgnt.exe", None),               // Avira AntiVir Guard Notification Tray
            ])
        );
        self.m.insert(MenuId::ANTIVIRUS_ESCAN, MenuEntry::new(
            "eScan",
            vec![
                ("avpmapp.exe", None),  // eScan File Monitoring System
                ("econceal.exe", None), // eConceal Service
                ("escanmon.exe", None), // eScan Monitoring Tray
                ("escanpro.exe", None), // eScan Protection Center
                ("avpMWrap.exe", None), // eScan Antivirus Suite
                ("eScanRAD.exe", None), // eScan Remote Administration
                ("MAILDISP.EXE", None), // eScan Mail Scanner Component
                ("traycser.exe", None), // eScan Client Updater
                ("trayeser.exe", None), // eScan Management Console
                ("TRAYICOC.EXE", None), // eScan Client updater
                ("TRAYICOS.EXE", None), // eScan Server updater
                ("traysser.exe", None), // Service Module for eScan Server updater
                ("consctl.exe", None),  // eScan Application Blocker
                ("mwagent.exe", None),  // eScan Agent Application or MicroWorld Agent
            ])
        );
        self.m.insert(MenuId::ANTIVIRUS_FORTINET, MenuEntry::new(
            "Fortinet",
            vec![
                // https://docs.fortinet.com/document/forticlient/7.0.7/administration-guide/209271/forticlient-windows-processes
                ("FCVbltScan.exe", None),  // FortiClient Vulnerability Scan Daemon
                ("FortiAvatar.exe", None), // FortiClient User Avatar Agent
                ("FortiClient.exe", None), // FortiClient Console
                ("fcappdb.exe", None),     // FortiClient Application Database Service
                ("fcaptmon.exe", None),    // FortiClient Sandbox Agent
                ("FCDBLog.exe", None),     // FortiClient Logging Daemon
                ("FCHelper64.exe", None),  // FortiClient System Helper
                ("fmon.exe", None),        // FortiClient Realtime AntiVirus Protection
                ("fortiae.exe", None),     // FortiClient Anti-Exploit
                ("FortiESNAC.exe", None),  // FortiClient Network Access Control
                ("fortifws.exe", None),    // FortiClient Firewall Service
                ("FortiProxy.exe", None),  // FortiClient Proxy Service
                ("FortiScand.exe", None),  // FortiClient Scan Server
                ("FortiSettings.exe", None), // FortiClient Settings Service
                ("FortiSSLVPNdaemon.exe", None), // FortiClient SSLVPN daemon
                ("FortiTray.exe", None),   // FortiClient System Tray Controller
                ("FortiUSBmon.exe", None), // FortiClient USB monitor protection
                ("FortiWF.exe", None),     // FortiClient Web Filter Service
            ])
        );
        self.m.insert(MenuId::ANTIVIRUS_GDATA, MenuEntry::new(
            "G Data",
            vec![
                ("AVK.exe", None),        // G Data AntiVirus UI
                ("AVKWCtlx64.exe", None), // G Data Filesystem Monitor Service
                ("GdBgInx64.exe", None),  // G Data AntiVirus Bankguard
                ("AVKProxy.exe", None),   // G Data AntiVirus Proxy Service
                ("GDScan.exe", None),     // G Data AntiVirus Scan Server
                ("AVKService.exe", None), // G Data InternetSecurity Scheduler Service
                ("AVKTray.exe", None),    // G DATA InternetSecurity Tray Application
                ("GDSC.exe", None),       // G DATA SecurityCenter
                ("GDKBFltExe32.exe", None),
            ])
        );
        self.m.insert(MenuId::ANTIVIRUS_K7, MenuEntry::new(
            "K7",
            vec![
                ("K7RTScan.exe", None),    // K7 RealTime AntiVirus Services
                ("K7FWSrvc.exe", None),    // K7 Firewall Services
                ("K7PSSrvc.exe", None),    // K7 Privacy Manager
                ("K7EmlPxy.exe", None),    // K7 EMail Proxy Server
                ("K7TSecurity.exe", None), // K7 User Agent
                ("K7AVScan.exe", None),    // K7 AntiVirus Scanner Loader
                ("K7CrvSvc.exe", None),    // K7 Carnivore Service
                ("K7SysMon.exe", None),    // K7 System Monitor
                ("K7TSMain.exe", None),    // K7 Total Security
                ("K7TSMngr.exe", None),    // K7 TotalSecurity Service Manager
            ])
        );
        self.m.insert(MenuId::ANTIVIRUS_MCAFEE, MenuEntry::new(
            "McAfee",
            vec![
                ("mcapexe.exe", None),  // McAfee Access Protection
                ("mcshield.exe", None), // Part of McAfee real-time protection
                ("McUICnt.exe", None),  // McAfee HTML User Interface (UI) Container
                ("MfeAVSvc.exe", None), // McAfee Cloud AV
                ("mfemms.exe", None),   // McAfee Management Service
                ("mfevtps.exe", None),  // McAfee Process Validation Service
                ("MMSSHOST.exe", None), // McAfee Management Service Host
                ("QcShm.exe", None),    // McAfee QuickClean
                ("PEFService.exe", None),          // Intel Security PEF Service
                ("ModuleCoreService.exe", None),   // McAfee Module Core Service
                ("ProtectedModuleHost.exe", None), // McAfee Protected Module Host
            ])
        );
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
        self.m.insert(MenuId::TOOLS_PEID, MenuEntry::new(
            "PEiD",
            vec![("PEiD.exe", None)]
        ));
        self.m.insert(MenuId::TOOLS_RESOURCE_HACKER, MenuEntry::new(
            "Resource hacker",
            vec![("ResourceHacker.exe", None)]
        ));
        self.m.insert(MenuId::TOOLS_DIE, MenuEntry::new(
            "Detect It Easy",
            vec![
                ("die.exe", None),
                ("diec.exe", None),
                ("diel.exe", None)
            ]
        ));
        self.m.insert(MenuId::TOOLS_DEBUG_VIEW, MenuEntry::new(
            "Debug View",
            vec![
                ("Dbgview.exe", None),
                ("dbgview64.exe", None)
            ]
        ));
        self.m.insert(MenuId::TOOLS_PROCESS_MONITOR, MenuEntry::new(
            "Process Monitor",
            vec![
                ("Procmon.exe", None),
                ("Procmon64.exe", None)
            ]
        ));
        self.m.insert(MenuId::TOOLS_PROCESS_EXPLORER, MenuEntry::new(
            "Process Explorer",
            vec![
                ("procexp.exe", None),
                ("procexp64.exe", None)
            ]
        ));
        self.m.insert(MenuId::TOOLS_TCPVIEW, MenuEntry::new(
            "TCP View",
            vec![
                ("tcpvcon.exe", None),
                ("tcpvcon64.exe", None),
                ("tcpview.exe", None),
                ("tcpview64.exe", None)
            ]
        ));
        self.m.insert(MenuId::TOOLS_WIRESHARK, MenuEntry::new(
            "Wireshark",
            vec![
                ("dumpcap.exe", None),
                ("Wireshark.exe", None)
            ]
        ));
        self.m.insert(MenuId::TOOLS_PE_TOOLS, MenuEntry::new(
            "PE Tools",
            vec![("PETools.exe", None)]
        ));
        self.m.insert(MenuId::TOOLS_SPYXX, MenuEntry::new(
            "Spy++",
            vec![("spyxx.exe", None)]
        ));
        self.m.insert(MenuId::TOOLS_CTK_RES_EDIT, MenuEntry::new(
            "CTK Res Edit",
            vec![("CTKResEdit.exe", None)]
        ));
        self.m.insert(MenuId::TOOLS_XN_RES_EDITOR, MenuEntry::new(
            "XN Resource Editor",
            vec![("XNResourceEditor.exe", None)]
        ));
    }

}