#[allow(unused_imports)]
use windows::{Win32::System::Registry::{HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE}};
use cfg_if;

use crate::MenuId::*;

pub const KEEP_STUB_COPIES: bool = true;
pub const DEFAULT_PROCESS: &[crate::MenuId] = &[
    GUEST_VIRTUALBOX,
    DEBUGGER_IDA,
    ANTIVIRUS_FORTINET,
    FIREWALL_ZONEALARM,
    TOOLS_PEID,
    TOOLS_PROCESS_MONITOR,
    TOOLS_PROCESS_EXPLORER,
    TOOLS_TCPVIEW,
    TOOLS_WIRESHARK,
    TOOLS_PE_TOOLS,
    TOOLS_SPYXX,
];

cfg_if::cfg_if! {
    if #[cfg(feature = "user_current")] {
        pub const ROOT_KEY: HKEY = HKEY_CURRENT_USER;
    } else if #[cfg(feature = "user_all")] {
        pub const ROOT_KEY: HKEY = HKEY_LOCAL_MACHINE;
    } else {
        pub const ROOT_KEY: HKEY = HKEY_CURRENT_USER;
    }
}



