use std::fs;
use std::process::Command;

// TODO: should differ for cargo and non-cargo run
const HOME_FOLDER: &str = "./target/debug/";

pub struct MenuEntry<'u> {
    entry_text: &'u str,
    processes: Vec<(&'u str, Option<std::process::Child>)>,
    is_active: bool,
}

impl <'u> MenuEntry<'u> {
    pub fn new(text: &'u str, process_list: Vec<(&'u str, Option<std::process::Child>)>) -> MenuEntry<'u> {
        MenuEntry { entry_text: text, processes: process_list, is_active: false }
    }

    pub fn start_process(&mut self) {
        // TODO: Check stub's SHA3
        for (process_name, process_child) in &mut self.processes {
            if process_child.is_none() {
                let process_path: String = String::from(HOME_FOLDER) + process_name;
                let res = fs::copy(String::from(HOME_FOLDER) + "des-stub.exe", &process_path);
                debug_assert!(res.is_ok());
                *process_child = Some(Command::new(&process_path).arg("arg1").spawn().expect("Failed to start command!"))
            }
        }
        self.is_active = true;
    }

    pub fn stop_process(&mut self) {
        for (process_name, process_child) in &mut self.processes {
            let process_path: String = String::from(HOME_FOLDER) + process_name;
            if let Some(proc) = process_child {
                let res1 = proc.kill();
                debug_assert!(res1.is_ok());
                // TODO: make this removal optional
                let res2 = fs::remove_file(process_path);
                debug_assert!(res2.is_ok());
                *process_child = None
            }
        }
        self.is_active = false;
    }

    #[must_use]
    pub fn is_process_active(&self) -> bool {
        self.is_active
    }

    pub fn get_name(&self) -> &'u str {
        self.entry_text
    }

    pub fn refresh(&mut self) {
        panic!("Not implemented!");
    }
}
