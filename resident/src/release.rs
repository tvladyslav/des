// These are the set of variables that needs to be changed when
// preparing release version

use windows::{w, core::PCWSTR};

// Debug
pub const TRAY_ICON_PATH: PCWSTR = w!("resources/find_bug_icon_32px_by_Chenyu_Wang.ico");
pub const HOME_FOLDER: &str = "./target/debug/";

// Release
// pub const TRAY_ICON_PATH: PCWSTR = w!("find_bug_icon_32px_by_Chenyu_Wang.ico");
// pub const HOME_FOLDER: &str = "./";