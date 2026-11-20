fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    cosmic_applet_capslock::run()
}
