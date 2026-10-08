use super::*;
use iced::keyboard::{Key, Modifiers, key::Named};

#[test]
fn handbook_opens_offline_and_returns_without_changing_the_document() {
    let mut app = blank();
    app.current_mut().editor = text_editor::Content::with_text("# My draft\nUnfinished work");
    app.current_mut().document.mark_edited();
    let source = app.current().editor.text();
    let cursor = app.current().editor.cursor();
    let _ = update(&mut app, Message::ToggleReading);
    assert!(app.current().reading.is_some());

    let mut ui = simulator(crate::ui::view(&app));
    ui.click("Handbook").expect("handbook button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    let handbook = app.handbook.as_ref().expect("embedded handbook loaded");
    assert!(format!("{:?}", handbook.items()).contains("Start a document"));
    assert_eq!(app.current().editor.text(), source);

    let mut ui = simulator(crate::ui::view(&app));
    let _ = ui.snapshot(&app.theme).expect("render embedded handbook");
    ui.click("Back to writing")
        .expect("handbook back button visible");
    for message in ui.into_messages() {
        let _ = update(&mut app, message);
    }
    assert!(app.handbook.is_none());
    assert_eq!(app.current().editor.text(), source);
    assert_eq!(app.current().editor.cursor(), cursor);
    assert!(app.current().document.dirty);
    assert!(app.current().reading.is_some());
}

#[test]
fn escape_closes_handbook_in_one_press() {
    let mut app = blank();
    let _ = update(&mut app, Message::ShowHandbook);
    assert!(app.handbook.is_some());
    let _ = update(
        &mut app,
        Message::KeyPressed(Key::Named(Named::Escape), Modifiers::NONE),
    );
    assert!(app.handbook.is_none());
}

#[test]
fn handbook_text_stays_clear_of_the_scrollbar() {
    let mut app = blank();
    app.theme = iced::Theme::CatppuccinMocha;
    app.editor_font = iced::Font::MONOSPACE;
    let source = format!("# Handbook\n\n{}", "a ".repeat(2500));
    app.handbook = Some(iced::widget::markdown::Content::parse(&source));
    let mut ui = iced_test::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(1024.0, 768.0),
        crate::ui::view(&app),
    );
    let directory =
        std::env::temp_dir().join(format!("nia-handbook-gutter-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    ui.snapshot(&app.theme)
        .expect("render handbook with long text")
        .matches_image(directory.join("preview.png"))
        .expect("write preview");
    let path = std::fs::read_dir(&directory)
        .expect("read preview directory")
        .next()
        .expect("preview exists")
        .expect("read preview entry")
        .path();
    let mut reader = png::Decoder::new(std::io::BufReader::new(
        std::fs::File::open(path).expect("open preview"),
    ))
    .read_info()
    .expect("decode preview");
    let mut pixels = vec![0; reader.output_buffer_size().expect("preview size")];
    let info = reader.next_frame(&mut pixels).expect("read preview pixels");
    assert_eq!(info.width, 2048);
    let background = [pixels[0], pixels[1], pixels[2]];
    let text_in_gutter = (330..1100).any(|y| {
        (1740..1756).any(|x| {
            let i = (y * 2048 + x) * 4;
            pixels[i..i + 3] != background
        })
    });
    std::fs::remove_dir_all(directory).expect("remove preview");
    assert!(
        !text_in_gutter,
        "handbook text overlaps the scrollbar gutter"
    );
}
