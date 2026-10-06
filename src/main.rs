mod app;
mod document;
mod ui;

fn main() -> iced::Result {
    iced::application(app::boot, app::update, ui::view)
        .default_font(iced::Font::MONOSPACE)
        .title(|state: &app::App| state.title())
        .subscription(app::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1000.0, 760.0),
            maximized: true,
            exit_on_close_request: false,
            ..Default::default()
        })
        .run()
}
