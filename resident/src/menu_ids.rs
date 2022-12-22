#[allow(non_camel_case_types)]
#[derive(PartialOrd, PartialEq, Ord, Eq, Clone, Copy, FromPrimitive)]
pub enum MenuId {
    GUEST = 1,
    GUEST_VIRTUALBOX,
    GUEST_VMWARE,
    GUEST_PARALLELS,
    GUEST_HYPERV,
    GUEST_VIRTUAL_PC,
    // DEBUGGER,
    // DEBUGGER_OLLY,
    // DEBUGGER_WINDBG,
    // DEBUGGER_X64DBG,
    // DEBUGGER_IDA,
    // DEBUGGER_IMMUNITY,
    // DEBUGGER_RADARE2,
    // DEBUGGER_BINARY_NINJA,
    // ANTIVIRUS,
    // ANTIVIRUS_BITDEFENDER,
    // ANTIVIRUS_NORTON,
    // ANTIVIRUS_TREND_MICRO,
    // ANTIVIRUS_KASPERSKY,
    // ANTIVIRUS_AVIRA,
    // ANTIVIRUS_AVAST,
    // ANTIVIRUS_MCAFEE,
    // ANTIVIRUS_DRWEB,
    // ANTIVIRUS_ESET_NOD32,
    // ANTIVIRUS_SOPHOS,
    // ANTIVIRUS_PANDA,
    // ANTIVIRUS_WEBROOT,
    // ANTIVIRUS_MALWAREBYTES,
    // ANTIVIRUS_FSECURE,
    // ANTIVIRUS_GDATA,
    // ANTIVIRUS_ESCAN,
    // FIREWALL,
    // FIREWALL_ZONEALARM,
    // FIREWALL_GLASSWIRE,
    // FIREWALL_COMODO,
    // FIREWALL_TINYWALL,

    // TOOLS,
    // TOOLS_PEID,
    // TOOLS_RESOURCE_HACKER,
    // TOOLS_DIE,
    // TOOLS_BYTECODE_VIEWER,
    // TOOLS_PROCESS_MONITOR,
    // TOOLS_PROCESS_EXPLORER,
    // TOOLS_TCPVIEW,
    // TOOLS_WIRESHARK,
    // TOOLS_PE_TOOLS,
    // TOOLS_SPYXX,

    ABOUT,
    EXIT,
    PAUSE,
    RESUME,
}