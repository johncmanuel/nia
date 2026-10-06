use super::*;
use iced::keyboard::key;
use iced_test::simulator;

#[test]
fn one_captured_escape_closes_focused_search() {
    let mut app = app::blank();
    app.show_search = true;
    let mut ui = simulator(ui::view(&app));
    ui.click("Find in document").expect("search input visible");
    let press = iced_test::simulator::press_key(Key::Named(key::Named::Escape), None);
    let status = ui.simulate([press.clone()])[0];
    drop(ui);
    assert_eq!(status, event::Status::Captured);
    let id = window::Id::unique();
    let routed = keyboard_message(press, status, id).expect("captured escape routed");
    if let Message::Window(_, message) = routed {
        let _ = app::update(&mut app, message);
    }
    assert!(!app.show_search);
}
