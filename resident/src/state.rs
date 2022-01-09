pub struct MenuEntry<'u> {
    pub id: u32,
    pub entry_text: &'u str,
    pub process_name: &'u str,
    pub process_child: Option<std::process::Child>,
}
