use super::*;
use iced_test::simulator;
use std::time::Duration;

mod close;
mod files;
mod layout;
mod navigation;
mod performance;

fn shortcut_event(key: Key, modifiers: Modifiers) -> iced::Event {
    let mut press = iced_test::simulator::press_key(key, None);
    if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        modifiers: pressed, ..
    }) = &mut press
    {
        *pressed = modifiers;
    }
    press
}
