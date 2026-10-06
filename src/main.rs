mod app;
mod document;
mod ui;

fn main() -> iced::Result {
    iced::application(app::boot, app::update, ui::view)
        .default_font(iced::Font::MONOSPACE)
        .title(|state: &app::App| state.title())
        .subscription(app::subscription)
        .window_size((1000.0, 760.0))
        .exit_on_close_request(false)
        .run()
}
