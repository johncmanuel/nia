use super::{BODY_SIZE, Demo, LEFT, Point, TOP, layout};
use iced_test::{Simulator, simulator};

#[test]
fn clicking_visible_heading_maps_to_source_after_marker() {
    let source = "# Heading\ntext";
    let position = layout(source, source.len(), 700.0).hit_test(Point::new(28.0, 33.0));
    assert_eq!(position, 2);
}

#[test]
fn typing_at_hidden_heading_start_preserves_markdown() {
    let mut demo = Demo::new();
    demo.source = "# Heading\nother".to_owned();
    demo.cursor = demo.source.len();
    demo.anchor = demo.cursor;
    let offset = layout(&demo.source, demo.cursor, 700.0).hit_test(Point::new(28.0, 33.0));
    demo.update(super::Message::Place {
        offset,
        extend: false,
    });
    demo.replace_selection("New ");
    assert_eq!(demo.source, "# New Heading\nother");
}

#[test]
fn selection_inside_hidden_emphasis_keeps_markers() {
    let mut demo = Demo::new();
    demo.source = "**bold**\nother".to_owned();
    demo.cursor = demo.source.len();
    let line = layout(&demo.source, demo.cursor, 700.0);
    demo.anchor = line.hit_test(Point::new(LEFT, TOP + 1.0));
    demo.cursor = line.hit_test(Point::new(LEFT + 4.0 * BODY_SIZE * 0.62, TOP + 1.0));
    demo.replace_selection("new");
    assert_eq!(demo.source, "**new**\nother");
}

#[test]
fn edits_replace_selected_utf8_source() {
    let mut demo = Demo::new();
    demo.source = "Café **bold**".to_owned();
    demo.anchor = 0;
    demo.cursor = 5;
    demo.replace_selection("Tea");
    assert_eq!(demo.source, "Tea **bold**");
    assert_eq!(demo.cursor, 3);
}

#[test]
fn wrapped_click_stays_on_source_boundary() {
    let source = "# Long heading text";
    let layout = layout(source, source.len(), 130.0);
    let offset = layout.hit_test(Point::new(28.0, 79.0));
    assert!(source.is_char_boundary(offset));
}

#[test]
fn iced_canvas_accepts_click_and_typing() {
    let mut demo = Demo::new();
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(1000.0, 650.0),
        demo.view(),
    );
    ui.point_at(Point::new(75.0, 160.0));
    ui.simulate(simulator::click());
    ui.typewrite("Z");
    let messages: Vec<_> = ui.into_messages().collect();
    assert!(
        messages.len() >= 2,
        "canvas should emit a click and a key event"
    );
    for message in messages {
        demo.update(message);
    }
    assert!(demo.source.contains('Z'));
    assert!(demo.source.is_char_boundary(demo.cursor));
}

#[test]
#[ignore = "manual visual inspection"]
fn render_preview_for_inspection() {
    let demo = Demo::new();
    let mut ui = Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(1000.0, 650.0),
        demo.view(),
    );
    ui.snapshot(&iced::Theme::TokyoNightStorm)
        .expect("render prototype")
        .matches_image("/tmp/nia-live-markdown-prototype.png")
        .expect("write prototype image");
}
