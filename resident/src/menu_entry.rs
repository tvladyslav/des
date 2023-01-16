use sha2::{Sha512, Digest};
use std::{fs, io, process::Command};
use crate::release::{HOME_FOLDER, STUB_HASH};

fn get_file_hash(path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha512::new();
    let _n = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(hash.iter().map(|v| format!("{:02X}", v)).collect())
}

pub struct MenuEntry<'u> {
    entry_text: &'u str,
    processes: Vec<(&'u str, Option<std::process::Child>)>,
    is_active: bool,
}

impl <'u> MenuEntry<'u> {
    pub fn new(text: &'u str, process_list: Vec<(&'u str, Option<std::process::Child>)>) -> MenuEntry<'u> {
        MenuEntry { entry_text: text, processes: process_list, is_active: false }
    }

    pub fn start_process(&mut self) -> std::io::Result<()> {
        let stub_path = String::from(HOME_FOLDER) + "des-stub.exe";
        let stub_hash = get_file_hash(&stub_path)?;
        if !stub_hash.eq_ignore_ascii_case(STUB_HASH) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Stub hash mismatch."));
        }

        self.start_process_unsafe()
    }

    pub fn start_process_unsafe(&mut self) -> std::io::Result<()> {
        let stub_path = String::from(HOME_FOLDER) + "des-stub.exe";
        for (process_name, process_child) in &mut self.processes {
            if process_child.is_none() {
                let process_path: String = String::from(HOME_FOLDER) + process_name;
                fs::copy(&stub_path, &process_path)?;
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
                proc.wait()?;
                let process_path: String = String::from(HOME_FOLDER) + process_name;
                fs::remove_file(process_path)?;
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
