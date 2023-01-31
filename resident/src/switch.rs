use crate::menu_ids::MenuId;

pub trait Switch {
    type ErrorType;

    fn enable(&mut self, menu_item: &MenuId) -> std::result::Result<(), Self::ErrorType>;
    fn disable(&mut self, menu_item: &MenuId) -> std::result::Result<(), Self::ErrorType>;

    #[must_use]
    fn is_enabled(&self, id: &MenuId) -> bool;
}