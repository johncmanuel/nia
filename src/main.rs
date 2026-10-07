mod app;
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
        metadata.target().starts_with("nia") && metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("nia {}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

fn main() -> iced::Result {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
    iced::daemon(desktop::boot, desktop::update, desktop::view)
        .default_font(iced::Font::MONOSPACE)
        .theme(desktop::theme)
        .style(desktop::style)
        .title(desktop::title)
        .subscription(desktop::subscription)
        .run()
}
