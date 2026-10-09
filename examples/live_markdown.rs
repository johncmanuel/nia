// Throwaway prototype. Run with: cargo run --example live_markdown
#[path = "live_markdown/projection.rs"]
mod projection;

use iced::keyboard::{self, Key, key::Named};
use iced::mouse;
use iced::widget::{canvas, column, container, row, scrollable, text};
use iced::{Element, Font, Length, Point, Rectangle, Renderer, Size, Theme, font};
use projection::{Glyph, LineKind, Style, project};

const INITIAL: &str = r#"# Live Markdown

Try **bold**, *italics*, ~~strike~~, ==highlight==, and `code`.
See [the guide](https://www.markdownguide.org/) and ![a photo](photo.png).
Water is H~2~O. Math is X^2^. A footnote[^1] and :joy:.

> A blockquote can contain **strong** text.
- An unordered item
1. An ordered item
- [x] A finished task
- [ ] A task to do

---

| Name | Status |
| --- | --- |
| nia | prototype |

```rust
fn main() { println!("Markdown stays as source"); }
```

Term
: A definition for the term

[^1]: The footnote text.

Click a line to reveal its source markers. Click away to hide them."#;
const LEFT: f32 = 28.0;
const TOP: f32 = 32.0;
const BODY_SIZE: f32 = 21.0;
const HEADING_SIZE: f32 = 32.0;

#[derive(Default)]
struct Demo {
    source: String,
    cursor: usize,
    anchor: usize,
}

#[derive(Debug, Clone)]
enum Message {
    Place {
        offset: usize,
        extend: bool,
    },
    Key {
        key: Key,
        text: Option<String>,
        modifiers: keyboard::Modifiers,
    },
}

impl Demo {
    fn new() -> Self {
        let source = INITIAL.to_owned();
        Self {
            cursor: source.len(),
            anchor: source.len(),
            source,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Place { offset, extend } => {
                self.cursor = offset.min(self.source.len());
                if !extend {
                    self.anchor = self.cursor;
                }
            }
            Message::Key {
                key,
                text,
                modifiers,
            } => self.key_pressed(key, text, modifiers),
        }
    }

    fn key_pressed(&mut self, key: Key, text: Option<String>, modifiers: keyboard::Modifiers) {
        if modifiers.command() {
            if matches!(key, Key::Character(ref value) if value.eq_ignore_ascii_case("a")) {
                self.anchor = 0;
                self.cursor = self.source.len();
            }
            return;
        }
        match key {
            Key::Named(Named::ArrowLeft) => {
                self.move_to(
                    previous_boundary(&self.source, self.cursor),
                    modifiers.shift(),
                );
            }
            Key::Named(Named::ArrowRight) => {
                self.move_to(next_boundary(&self.source, self.cursor), modifiers.shift());
            }
            Key::Named(Named::ArrowUp) => {
                self.move_to(
                    vertical_boundary(&self.source, self.cursor, -1),
                    modifiers.shift(),
                );
            }
            Key::Named(Named::ArrowDown) => {
                self.move_to(
                    vertical_boundary(&self.source, self.cursor, 1),
                    modifiers.shift(),
                );
            }
            Key::Named(Named::Home) => {
                let start = self.source[..self.cursor]
                    .rfind('\n')
                    .map_or(0, |at| at + 1);
                self.move_to(start, modifiers.shift());
            }
            Key::Named(Named::End) => {
                let end = self.source[self.cursor..]
                    .find('\n')
                    .map_or(self.source.len(), |at| self.cursor + at);
                self.move_to(end, modifiers.shift());
            }
            Key::Named(Named::Backspace) => {
                if self.cursor == self.anchor {
                    self.anchor = previous_boundary(&self.source, self.cursor);
                }
                self.replace_selection("");
            }
            Key::Named(Named::Delete) => {
                if self.cursor == self.anchor {
                    self.anchor = next_boundary(&self.source, self.cursor);
                }
                self.replace_selection("");
            }
            Key::Named(Named::Enter) => self.replace_selection("\n"),
            _ => {
                if !modifiers.control()
                    && !modifiers.alt()
                    && let Some(text) = text.filter(|value| !value.is_empty())
                {
                    self.replace_selection(&text);
                }
            }
        }
    }

    fn move_to(&mut self, offset: usize, extend: bool) {
        self.cursor = offset;
        if !extend {
            self.anchor = offset;
        }
    }

    fn replace_selection(&mut self, replacement: &str) {
        let start = self.cursor.min(self.anchor);
        let end = self.cursor.max(self.anchor);
        self.source.replace_range(start..end, replacement);
        self.cursor = start + replacement.len();
        self.anchor = self.cursor;
    }

    fn view(&self) -> Element<'_, Message> {
        let live = scrollable(
            canvas(Live {
                source: &self.source,
                cursor: self.cursor,
                anchor: self.anchor,
            })
            .width(Length::Fill)
            .height(Length::Fixed(
                (self.source.lines().count() as f32 * 75.0).max(700.0),
            )),
        )
        .width(Length::FillPortion(2))
        .height(Length::Fill);
        let source = column![
            text("Markdown source (read only)").size(15),
            scrollable(text(&self.source).font(Font::MONOSPACE).size(14))
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(12)
        .width(Length::FillPortion(1));
        column![
            text("Live Markdown canvas prototype").size(22),
            text("Click the left pane to edit. Markers appear on the active line; click away to hide them. Drag to select. This is disposable prototype code.").size(14),
            row![
                container(live).width(Length::FillPortion(2)),
                container(source).width(Length::FillPortion(1)).padding(16),
            ]
            .spacing(12)
            .height(Length::Fill),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}

fn previous_boundary(source: &str, offset: usize) -> usize {
    source[..offset]
        .char_indices()
        .last()
        .map_or(0, |(at, _)| at)
}

fn next_boundary(source: &str, offset: usize) -> usize {
    source[offset..]
        .chars()
        .next()
        .map_or(offset, |ch| offset + ch.len_utf8())
}

fn vertical_boundary(source: &str, offset: usize, direction: i32) -> usize {
    let line_start = source[..offset].rfind('\n').map_or(0, |at| at + 1);
    let column = source[line_start..offset].chars().count();
    let target_start = if direction < 0 {
        if line_start == 0 {
            return offset;
        }
        source[..line_start - 1].rfind('\n').map_or(0, |at| at + 1)
    } else {
        let Some(end) = source[offset..].find('\n') else {
            return offset;
        };
        offset + end + 1
    };
    let target_end = source[target_start..]
        .find('\n')
        .map_or(source.len(), |at| target_start + at);
    source[target_start..target_end]
        .char_indices()
        .nth(column)
        .map_or(target_end, |(at, _)| target_start + at)
}

struct Live<'a> {
    source: &'a str,
    cursor: usize,
    anchor: usize,
}

#[derive(Default)]
struct LiveState {
    focused: bool,
    dragging: bool,
}

impl canvas::Program<Message> for Live<'_> {
    type State = LiveState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.focused = cursor.is_over(bounds);
                state.dragging = state.focused;
                let point = cursor.position_in(bounds)?;
                Some(
                    canvas::Action::publish(Message::Place {
                        offset: layout(self.source, self.cursor, bounds.width).hit_test(point),
                        extend: false,
                    })
                    .and_capture(),
                )
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                let point = cursor.position_in(bounds)?;
                Some(
                    canvas::Action::publish(Message::Place {
                        offset: layout(self.source, self.cursor, bounds.width).hit_test(point),
                        extend: true,
                    })
                    .and_capture(),
                )
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
                None
            }
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                text,
                modifiers,
                ..
            }) if state.focused => Some(
                canvas::Action::publish(Message::Key {
                    key: key.clone(),
                    text: text.as_ref().map(ToString::to_string),
                    modifiers: *modifiers,
                })
                .and_capture(),
            ),
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let palette = theme.palette();
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), palette.background);
        let layout = layout(self.source, self.cursor, bounds.width);
        let selection = self.cursor.min(self.anchor)..self.cursor.max(self.anchor);
        for decoration in &layout.decorations {
            match decoration.kind {
                LineKind::Quote => frame.fill_rectangle(
                    Point::new(LEFT - 12.0, decoration.y),
                    Size::new(3.0, decoration.height),
                    palette.primary.scale_alpha(0.7),
                ),
                LineKind::Code | LineKind::Fence => frame.fill_rectangle(
                    Point::new(LEFT - 8.0, decoration.y),
                    Size::new((bounds.width - LEFT * 1.5).max(0.0), decoration.height),
                    palette.success.scale_alpha(0.08),
                ),
                LineKind::Table => frame.fill_rectangle(
                    Point::new(LEFT - 8.0, decoration.y),
                    Size::new((bounds.width - LEFT * 1.5).max(0.0), decoration.height),
                    palette.primary.scale_alpha(0.05),
                ),
                LineKind::Rule | LineKind::TableRule if decoration.empty => frame.fill_rectangle(
                    Point::new(LEFT, decoration.y + decoration.height / 2.0),
                    Size::new((bounds.width - LEFT * 2.0).max(0.0), 2.0),
                    palette.primary.scale_alpha(0.55),
                ),
                _ => {}
            }
        }
        for glyph in &layout.glyphs {
            if glyph.glyph.style == Style::Highlight {
                frame.fill_rectangle(
                    glyph.position,
                    Size::new(glyph.width, glyph.height),
                    palette.warning.scale_alpha(0.35),
                );
            }
            if selection.start < glyph.source_end && glyph.source_start < selection.end {
                frame.fill_rectangle(
                    glyph.position,
                    Size::new(glyph.width, glyph.height),
                    palette.primary.scale_alpha(0.35),
                );
            }
            let size = glyph_size(glyph.glyph.style, glyph.heading);
            let font = Font {
                style: if glyph.glyph.style == Style::Emphasis {
                    font::Style::Italic
                } else {
                    font::Style::Normal
                },
                weight: if matches!(glyph.glyph.style, Style::Strong | Style::Heading) {
                    font::Weight::Bold
                } else {
                    font::Weight::Normal
                },
                ..Font::MONOSPACE
            };
            frame.fill_text(canvas::Text {
                content: glyph.glyph.ch.to_string(),
                position: Point::new(
                    glyph.position.x,
                    glyph.position.y
                        + match glyph.glyph.style {
                            Style::Subscript => 9.0,
                            Style::Superscript | Style::Footnote => -3.0,
                            _ => 0.0,
                        },
                ),
                color: match glyph.glyph.style {
                    Style::Marker => palette.text.scale_alpha(0.45),
                    Style::Code => palette.success,
                    Style::Link | Style::Footnote | Style::Border => palette.primary,
                    Style::Quote => palette.text.scale_alpha(0.75),
                    _ => palette.text,
                },
                size: size.into(),
                font,
                ..canvas::Text::default()
            });
            if matches!(glyph.glyph.style, Style::Strike | Style::Link) {
                let line_y = glyph.position.y
                    + if glyph.glyph.style == Style::Strike {
                        glyph.height * 0.43
                    } else {
                        glyph.height * 0.85
                    };
                frame.fill_rectangle(
                    Point::new(glyph.position.x, line_y),
                    Size::new(glyph.width, 1.0),
                    palette.primary.scale_alpha(0.8),
                );
            }
        }
        let caret = layout.caret(self.cursor);
        frame.fill_rectangle(
            Point::new(caret.position.x, caret.position.y),
            Size::new(2.0, caret.height),
            palette.primary,
        );
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::default()
        }
    }
}

#[derive(Clone)]
struct PlacedGlyph {
    glyph: Glyph,
    source_start: usize,
    source_end: usize,
    position: Point,
    width: f32,
    height: f32,
    heading: bool,
}

#[derive(Clone, Copy)]
struct Caret {
    offset: usize,
    position: Point,
    height: f32,
}

struct Layout {
    glyphs: Vec<PlacedGlyph>,
    carets: Vec<Caret>,
    decorations: Vec<Decoration>,
}

struct Decoration {
    kind: LineKind,
    y: f32,
    height: f32,
    empty: bool,
}

impl Layout {
    fn hit_test(&self, point: Point) -> usize {
        self.carets
            .iter()
            .min_by(|a, b| {
                let distance = |caret: &Caret| {
                    let y = (point.y - caret.position.y).abs();
                    y * 1000.0 + (point.x - caret.position.x).abs()
                };
                distance(a).total_cmp(&distance(b))
            })
            .map_or(0, |caret| caret.offset)
    }

    fn caret(&self, offset: usize) -> Caret {
        self.carets
            .iter()
            .rev()
            .min_by_key(|caret| caret.offset.abs_diff(offset))
            .copied()
            .unwrap_or(Caret {
                offset: 0,
                position: Point::new(LEFT, TOP),
                height: BODY_SIZE * 1.45,
            })
    }
}

fn layout(source: &str, cursor: usize, width: f32) -> Layout {
    let active_line = source[..cursor]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let mut result = Layout {
        glyphs: Vec::new(),
        carets: Vec::new(),
        decorations: Vec::new(),
    };
    let mut y = TOP;
    for line in project(source, active_line) {
        let line_y = y;
        let heading = line.kind == LineKind::Heading;
        let height = if heading {
            HEADING_SIZE * 1.4
        } else {
            BODY_SIZE * 1.45
        };
        result.decorations.push(Decoration {
            kind: line.kind,
            y,
            height,
            empty: line.glyphs.is_empty(),
        });
        let decoration_index = result.decorations.len() - 1;
        let mut x = LEFT;
        result.carets.push(Caret {
            offset: line.glyphs.first().map_or(line.start, |glyph| glyph.start),
            position: Point::new(x, y),
            height,
        });
        let visible = line.glyphs;
        for (index, glyph) in visible.iter().enumerate() {
            // The crate uses character-width estimates too; exact shaping is a production concern.
            let glyph_width = estimated_width(glyph, heading);
            if !glyph.ch.is_whitespace() && (index == 0 || visible[index - 1].ch.is_whitespace()) {
                let word_width: f32 = visible[index..]
                    .iter()
                    .take_while(|next| !next.ch.is_whitespace())
                    .map(|next| estimated_width(next, heading))
                    .sum();
                if x > LEFT && x + word_width > width - LEFT && word_width < width - LEFT * 2.0 {
                    y += height;
                    x = LEFT;
                    result.carets.push(Caret {
                        offset: glyph.start,
                        position: Point::new(x, y),
                        height,
                    });
                }
            }
            if x + glyph_width > width - LEFT && x > LEFT {
                y += height;
                x = LEFT;
                result.carets.push(Caret {
                    offset: glyph.start,
                    position: Point::new(x, y),
                    height,
                });
            }
            let start = glyph.start;
            let end = glyph.end;
            result.glyphs.push(PlacedGlyph {
                glyph: glyph.clone(),
                source_start: start,
                source_end: end,
                position: Point::new(x, y),
                width: glyph_width,
                height,
                heading,
            });
            x += glyph_width;
            result.carets.push(Caret {
                offset: end,
                position: Point::new(x, y),
                height,
            });
        }
        result.decorations[decoration_index].height = y - line_y + height;
        y += height + 8.0;
    }
    result
}

fn glyph_size(style: Style, heading: bool) -> f32 {
    let base = if heading { HEADING_SIZE } else { BODY_SIZE };
    if matches!(
        style,
        Style::Subscript | Style::Superscript | Style::Footnote
    ) {
        base * 0.75
    } else {
        base
    }
}

fn estimated_width(glyph: &Glyph, heading: bool) -> f32 {
    let size = glyph_size(glyph.style, heading);
    if glyph.ch.is_ascii() {
        size * 0.62
    } else {
        size
    }
}

fn main() -> iced::Result {
    iced::application(Demo::new, Demo::update, Demo::view)
        .theme(|_: &Demo| Theme::TokyoNightStorm)
        .run()
}

#[cfg(test)]
#[path = "live_markdown/tests.rs"]
mod tests;
