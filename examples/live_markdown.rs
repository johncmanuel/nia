// Throwaway prototype. Run with: cargo run --example live_markdown
#[path = "live_markdown/projection.rs"]
mod projection;

use iced::keyboard::{self, Key, key::Named};
use iced::mouse;
use iced::widget::{canvas, column, container, row, scrollable, text};
use iced::{Element, Font, Length, Point, Rectangle, Renderer, Size, Theme, font};
use projection::{Glyph, Style, project};

const INITIAL: &str = "# A heading\n\nClick here and type. Try **bold** and *italics*.\n\nClick another line to hide its Markdown markers.";
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
        let live = canvas(Live {
            source: &self.source,
            cursor: self.cursor,
            anchor: self.anchor,
        })
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
        for glyph in &layout.glyphs {
            if selection.start < glyph.source_end && glyph.source_start < selection.end {
                frame.fill_rectangle(
                    glyph.position,
                    Size::new(glyph.width, glyph.height),
                    palette.primary.scale_alpha(0.35),
                );
            }
            let size = if glyph.heading {
                HEADING_SIZE
            } else {
                BODY_SIZE
            };
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
                position: glyph.position,
                color: if glyph.glyph.style == Style::Marker {
                    palette.text.scale_alpha(0.45)
                } else {
                    palette.text
                },
                size: size.into(),
                font,
                ..canvas::Text::default()
            });
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
    };
    let mut y = TOP;
    for line in project(source, active_line) {
        let height = if line.heading {
            HEADING_SIZE * 1.4
        } else {
            BODY_SIZE * 1.45
        };
        let size = if line.heading {
            HEADING_SIZE
        } else {
            BODY_SIZE
        };
        let mut x = LEFT;
        result.carets.push(Caret {
            offset: line.glyphs.first().map_or(line.start, |glyph| glyph.start),
            position: Point::new(x, y),
            height,
        });
        for glyph in line.glyphs {
            // The crate uses character-width estimates too; exact shaping is a production concern.
            let glyph_width = if glyph.ch.is_ascii() {
                size * 0.62
            } else {
                size
            };
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
                glyph,
                source_start: start,
                source_end: end,
                position: Point::new(x, y),
                width: glyph_width,
                height,
                heading: line.heading,
            });
            x += glyph_width;
            result.carets.push(Caret {
                offset: end,
                position: Point::new(x, y),
                height,
            });
        }
        y += height + 8.0;
    }
    result
}

fn main() -> iced::Result {
    iced::application(Demo::new, Demo::update, Demo::view)
        .theme(|_: &Demo| Theme::TokyoNightStorm)
        .run()
}

#[cfg(test)]
#[path = "live_markdown/tests.rs"]
mod tests;
