// These are the set of variables that needs to be changed when
// preparing release version

use windows::{w, core::PCWSTR};

// Common
// SHA2-512, can be obtained by command
// Get-FileHash -Algorithm SHA512 -LiteralPath target\release\des-stub.exe | Select-Object -ExpandProperty Hash
pub const STUB_HASH: &str = "C47E23101074B4B52ED6C8C2EBE97AE4A6812C62D4AF62282AC8703360CF6EC8B37B8E5C7C004C66FD07CD3E9A36420C64CB7862E31B8B950E0E1C10EB1DEE82";

// Please edit version in main.rs!

#[cfg(debug_assertions)]
pub const TRAY_ICON_PATH: PCWSTR = w!("resources/find_bug_icon_32px_by_Chenyu_Wang.ico");
#[cfg(debug_assertions)]
pub const HOME_FOLDER: &str = "./target/debug/";
#[cfg(debug_assertions)]
pub const STUB_CONTENT: &[u8; 165888] = std::include_bytes!("..\\..\\target\\debug\\des-stub.exe");

#[cfg(not(debug_assertions))]
pub const TRAY_ICON_PATH: PCWSTR = w!("find_bug_icon_32px_by_Chenyu_Wang.ico");
#[cfg(not(debug_assertions))]
pub const HOME_FOLDER: &str = "./";
#[cfg(not(debug_assertions))]
pub const STUB_CONTENT: &[u8; 165888] = std::include_bytes!("..\\..\\target\\release\\des-stub.exe");