use windows:: {
    w,
    core:: {
        PCWSTR,
        Result,
    },
    Win32:: {
        System::Registry::*,
        Foundation::{
            ERROR_SUCCESS,
            ERROR_FILE_NOT_FOUND,
            ERROR_BAD_PATHNAME,
        },
    },
};

use core::ffi::c_void;

use crate::{
    convert::{to_utf16, to_win_error},
    menu_ids::MenuId,
    simple_execute,
    switch::Switch,
};

const STARTUP_SUBPATH: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const VALUE_NAME: PCWSTR = w!("des");

pub struct AutoStart {
    is_enabled: bool,
    handle: HKEY,
}

impl Switch for AutoStart {
    type ErrorType = windows::core::Error;
    fn enable(&mut self, _id: &MenuId) -> windows::core::Result<()> {
        let path = std::env::current_exe().map_err(to_win_error)?;
        let path_str: &str = path.to_str().ok_or(windows::core::Error::from(ERROR_BAD_PATHNAME))?;
        let path_vec = to_utf16(path_str);
        simple_execute!(RegSetValueExW(
            self.handle,
            VALUE_NAME,
            0,
            REG_SZ,
            Some(path_vec.align_to::<u8>().1),
        ));
        self.is_enabled = true;
        Ok(())
    }

    fn disable(&mut self, _id: &MenuId) -> windows::core::Result<()> {
        simple_execute!(RegDeleteValueW(
            self.handle,
            VALUE_NAME
        ));
        self.is_enabled = false;
        Ok(())
    }

    #[must_use]
    fn is_enabled(&self, _id: &MenuId) -> bool {
        self.is_enabled
    }
}

impl AutoStart {
    pub const fn new() -> AutoStart {
        AutoStart { is_enabled: false, handle: HKEY(0) }
    }

    pub fn init(&mut self) -> Result<bool> {
        let mut res = unsafe {RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            STARTUP_SUBPATH,
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            None,
            &mut self.handle,
            None    // Not interested in this value
        )};
        if res != ERROR_SUCCESS {
            res = unsafe {RegCreateKeyExW(
                HKEY_CURRENT_USER,
                STARTUP_SUBPATH,
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                None,
                &mut self.handle,
                None    // Not interested in this value
            )};
            if res != ERROR_SUCCESS {
                return Err(res.into());
            }
        }
        self.is_enabled = self.is_enabled_in_registry()?;
        Ok(self.is_enabled)
    }

    pub unsafe fn destroy(&mut self) {
        let _err = RegCloseKey(self.handle);
        // Ignore error, application is closing anyway
    }

    fn is_enabled_in_registry(&self) -> Result<bool> {
        let buf_size: usize = 1024;
        let mut reg_val_type: REG_VALUE_TYPE = REG_NONE;
        let mut pvdata: Vec<u16> = Vec::with_capacity(buf_size);
        let mut pcbdata: u32 = buf_size as u32;
        let result = unsafe { RegGetValueW(
            self.handle,
            None,
            VALUE_NAME,
            RRF_RT_REG_SZ,
            Some(&mut reg_val_type),
            Some(pvdata.as_mut_ptr() as *mut c_void),
            Some(&mut pcbdata)
        ) } ;

        if result == ERROR_FILE_NOT_FOUND {
            return Ok(false);
        }

        if result != ERROR_SUCCESS {
            return Err(result.into())
        }

        pcbdata /= 2;   // Bytes to u16

        let path = std::env::current_exe().map_err(to_win_error)?;
        let path_str: &str = path.to_str().ok_or(windows::core::Error::from(ERROR_BAD_PATHNAME))?;
        let path_vec = to_utf16(path_str);

        unsafe { pvdata.set_len(pcbdata as usize); }
        let ret = reg_val_type == REG_SZ
                        && pcbdata as usize == path_vec.len()
                        && path_vec == pvdata;
        Ok(ret)
    }
}
