

pub struct MenuEntry<'u> {
    pub id: u32,
    pub entry_text: &'u str,
    pub process_name: &'u str,
    pub process_child: Option<std::process::Child>,
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
