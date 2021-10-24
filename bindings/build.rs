fn main() {
    windows::build!(
        Windows::Win32::Foundation::*,
        Windows::Win32::Graphics::Gdi::ValidateRect,
        Windows::Win32::UI::Controls::IMAGE_FLAGS,
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::UI::Shell::{ Shell_NotifyIconW, NOTIFYICONDATAW },
        Windows::Win32::System::Diagnostics::Debug::SetLastError,
        Windows::Win32::System::LibraryLoader::GetModuleHandleW,
        Windows::Win32::System::Mmc::*,
    );
}