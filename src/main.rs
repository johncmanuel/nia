mod app;
mod config;
mod desktop;
mod document;
mod focus;
mod font;
mod keybindings;
mod markdown;
mod recent;
mod recovery;
mod search;
mod theme;
mod ui;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target().starts_with("nia") && metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!(
                "nia {} [{}]: {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

fn main() -> iced::Result {
    if log::set_logger(&LOGGER).is_ok() {
        let level = if std::env::var("NIA_LOG").as_deref() == Ok("debug") {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        };
        log::set_max_level(level);
    }
    iced::daemon(desktop::boot, desktop::update, desktop::view)
        .default_font(iced::Font::MONOSPACE)
        .theme(desktop::theme)
        .style(desktop::style)
        .title(desktop::title)
        .subscription(desktop::subscription)
        .run()
        .inspect_err(|_| log::error!("application runtime failed"))
}
