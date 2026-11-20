pub mod app;
pub mod caps_detect;
pub mod config;
pub mod desktop_entry;
pub mod settings;

pub const APP_ID: &str = "com.example.CosmicAppletCapsLock";

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<app::CapsLockApplet>(())
}
