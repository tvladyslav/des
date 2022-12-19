

pub struct MenuEntry<'u> {
    pub id: u32,
    pub entry_text: &'u str,
    pub processes: Vec<(&'u str, Option<std::process::Child>)>,
}

/*
impl Default for MenuEntry<'_> {
    fn default() -> MenuEntry<'static> {
        MenuEntry {
            id: u32::MAX,
            entry_text: "",
            process_name: "",
            process_child: None,
        }
    }
}
*/
