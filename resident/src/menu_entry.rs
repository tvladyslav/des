use sha2::{Sha512, Digest};
use std::io::ErrorKind;
use std::{fs, io, process::Command, path::Path};

use crate::release::{STUB_HASH, STUB_CONTENT};
use crate::config::KEEP_STUB_COPIES;

const PROC_FOLDER: &str = "proc/";

fn verify_file_hash(path: &str) -> Result<(), io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha512::new();
    let _n = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    let hash_string: String = hash.iter().map(|v| format!("{:02X}", v)).collect::<String>();
    if !hash_string.eq_ignore_ascii_case(STUB_HASH) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Stub hash mismatch."));
    }
    Ok(())
}

pub struct MenuEntry<'u> {
    entry_text: &'u str,
    processes: Vec<(&'u str, Option<std::process::Child>)>,
    is_active: bool,
    proc_folder: String,
}

impl <'u> MenuEntry<'u> {
    pub fn new(text: &'u str, process_list: Vec<(&'u str, Option<std::process::Child>)>) -> MenuEntry<'u> {
        let home_folder: String = unsafe { crate::HOME_FOLDER.clone() };
        MenuEntry { entry_text: text, processes: process_list, is_active: false, proc_folder: home_folder + PROC_FOLDER }
    }

    pub fn start_process(&mut self) -> std::io::Result<()> {
        for (process_name, process_child) in &mut self.processes {
            if process_child.is_none() {
                let process_path: String = self.proc_folder.clone() + process_name;
                let try_exist = Path::new(&process_path).try_exists();
                if let Ok(true) = try_exist {
                    verify_file_hash(&process_path)?;
                } else {
                    let res = fs::create_dir_all(&self.proc_folder);
                    if let Err(e) = &res {
                        if e.kind() != ErrorKind::AlreadyExists {
                            return res;
                        }
                    }
                    fs::write(&process_path, STUB_CONTENT)?;
                }
                let c = Command::new(&process_path).arg("arg1").spawn()?;
                *process_child = Some(c);
            }
        }
        self.is_active = true;
        Ok(())
    }

    pub fn stop_process(&mut self) -> std::io::Result<()> {
        for (process_name, process_child) in &mut self.processes {
            if let Some(proc) = process_child {
                proc.kill()?;
                if !KEEP_STUB_COPIES {
                    proc.wait()?;
                    let process_path: String = self.proc_folder.clone() + process_name;
                    fs::remove_file(process_path)?;
                }
                *process_child = None
            }
        }
        self.is_active = false;
        Ok(())
    }

    #[must_use]
    pub fn is_process_active(&self) -> bool {
        self.is_active
    }

    pub fn get_name(&self) -> &'u str {
        self.entry_text
    }

    // pub fn refresh(&mut self) {
    //     panic!("Not implemented!");
    // }
}
