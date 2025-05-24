use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Tree, Widget, tree};
use iced::advanced::{mouse, renderer, text};
use iced::mouse::Button;
use iced::{Color, Element, Event, Length, Rectangle, Size, event, touch};

use crate::Message;

#[derive(Clone, Copy)]
pub enum Content {
    Mine,
    Number(usize),
}

#[derive(Clone, Copy)]
pub enum State {
    Normal,
    Hovered,
    Pressed(Button),
    Revealed(Content),
}

impl Default for State {
    fn default() -> Self {
        Self::Normal
    }
}

impl State {
    pub fn is_interactive(&self) -> bool {
        match self {
            State::Normal => true,
            State::Hovered => true,
            State::Pressed(_) => true,
            _ => false,
        }
    }
}

pub struct Cell<'a, Message> {
    on_click: Option<OnClick<'a, Message>>,
    state: State,
}

enum OnClick<'a, Message> {
    Static(Message),
    Dynamic(Box<dyn Fn(Button) -> Option<Message> + 'a>),
}

impl<'a, Message: Clone> OnClick<'a, Message> {
    pub fn get(&self, button: Button) -> Option<Message> {
        match self {
            Self::Static(message) => Some(message.clone()),
            Self::Dynamic(closure) => closure(button),
        }
    }
}

impl<'a, Message> Cell<'a, Message> {
    pub fn new() -> Self {
        Self {
            on_click: None,
            state: State::Normal,
        }
    }

    pub fn on_click_with(self, on_click: impl Fn(Button) -> Option<Message> + 'a) -> Self {
        Self {
            on_click: Some(OnClick::Dynamic(Box::new(on_click))),
            ..self
        }
    }

    pub fn with_state(self, state: State) -> Self {
        Self { state, ..self }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Cell<'a, Message>
where
    Message: 'a + Clone,
    Renderer: renderer::Renderer + text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, 32.0, 32.0, 0, |_| {
            layout::Node::new(Size::new(32.0, 32.0))
        })
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        if !self.state.is_interactive() {
            return event::Status::Ignored;
        }

        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let State::Pressed(_) = self.state {
                    return event::Status::Ignored;
                }

                let bounds = layout.bounds();

                if bounds.contains(position) {
                    self.state = State::Hovered;
                } else {
                    self.state = State::Normal;
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) {
                    self.state = State::Pressed(button);

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) {
                    let should_fire = matches!(self.state, State::Pressed(b) if b == button);
                    self.state = State::Hovered;

                    if should_fire {
                        if let Some(message) = self
                            .on_click
                            .as_ref()
                            .and_then(|on_click| on_click.get(button))
                        {
                            shell.publish(message);
                        }

                        return event::Status::Captured;
                    }
                } else {
                    self.state = State::Normal;
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        match self.state {
            State::Normal => {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb8(0x20, 0x20, 0x20),
                );
            }
            State::Hovered => {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb8(0x30, 0x30, 0x30),
                );
            }
            State::Pressed(_) => {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb8(0x05, 0x05, 0x05),
                );
            }
            State::Revealed(Content::Mine) => {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb8(0xff, 0, 0),
                );
            }
            State::Revealed(Content::Number(n)) => {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb8(0xe0, 0xe0, 0xe0),
                );
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Cell<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + text::Renderer,
    Message: 'a + Clone,
{
    fn from(cell: Cell<'a, Message>) -> Self {
        Self::new(cell)
    }
}
