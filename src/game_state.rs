use std::collections::HashMap;

use iced::{
    Color, Element,
    Length::Fill,
    Point, Renderer, Size, Theme,
    advanced::{graphics::core::event, mouse},
    mouse::Button,
    widget::{
        Canvas,
        canvas::{self, Cache, Event, Frame, Text},
        column, row,
    },
};
use itertools::iproduct;
use rand::{Rng, seq::IteratorRandom};

use crate::cell::Cell;

#[derive(Clone, Copy, Debug)]
pub enum CellType {
    Mine,
    NonMine { neighbours: usize },
}

#[derive(Clone, Copy, Debug)]
pub struct CellState {
    pub is_revealed: bool,
    pub marking: Marking,
    pub cell_type: CellType,
}

impl Default for CellState {
    fn default() -> Self {
        Self {
            is_revealed: false,
            marking: Marking::None,
            cell_type: CellType::NonMine { neighbours: 0 },
        }
    }
}

impl CellState {
    pub fn mine() -> Self {
        CellState {
            cell_type: CellType::Mine,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Marking {
    None,
    Flag,
    QuestionMark,
}

impl Default for Marking {
    fn default() -> Self {
        Self::None
    }
}

impl Marking {
    pub fn next(self) -> Self {
        match self {
            Marking::None => Marking::Flag,
            Marking::Flag => Marking::QuestionMark,
            Marking::QuestionMark => Marking::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Position {
    row: i32,
    column: i32,
}

impl Position {
    fn new(row: usize, column: usize) -> Self {
        Self {
            row: row as i32,
            column: column as i32,
        }
    }

    fn at(point: Point) -> Self {
        Self {
            row: (point.y / 32.0).floor() as i32,
            column: (point.x / 32.0).floor() as i32,
        }
    }

    fn neighbours(&self) -> impl Iterator<Item = Position> {
        iproduct!(-1..=1, -1..=1)
            .filter(|&(x, y)| x != 0 || y != 0)
            .map(|(x, y)| Position {
                row: self.row + y,
                column: self.column + x,
            })
    }
}

pub struct GameState {
    cells: HashMap<Position, CellState>,
    width: usize,
    height: usize,
    mines: usize,
    has_revealed_any: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Reveal(Position),
    ToggleMark(Position),
}

impl GameState {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let cells =
            HashMap::from_iter((0..=width).flat_map(|c| {
                (0..=height).map(move |r| (Position::new(r, c), CellState::default()))
            }));

        Self {
            width,
            height,
            cells,
            mines,
            has_revealed_any: false,
        }
    }

    fn initialize_state(&mut self, starting_position: Position) {
        let mut rng = rand::rng();
        let start_neighbors: Vec<_> = starting_position.neighbours().collect();
        let mine_positions = self
            .cells
            .keys()
            .filter(|p| **p != starting_position && !start_neighbors.contains(p))
            .map(|p| p.clone())
            .choose_multiple(&mut rng, self.mines);

        for p in mine_positions {
            self.cells.insert(p, CellState::mine());

            for neighbor in p.neighbours() {
                let cell = self.cells.get_mut(&neighbor);
                if let Some(state) = cell {
                    if let CellType::NonMine { neighbours } = &mut state.cell_type {
                        *neighbours += 1;
                    }
                }
            }
        }
    }

    fn reveal(&mut self, position: &Position) {
        let cell = self.cells.get_mut(&position);
        if let Some(cell) = cell {
            if !cell.is_revealed {
                cell.is_revealed = true;

                if let CellType::NonMine { neighbours: 0 } = cell.cell_type {
                    for n in position.neighbours() {
                        self.reveal(&n);
                    }
                }
            }
        }
    }

    fn toggle_mark(&mut self, position: &Position) {
        let cell = self.cells.get_mut(&position);
        if let Some(cell) = cell {
            cell.marking = cell.marking.next();
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Reveal(position) => {
                if !self.has_revealed_any {
                    self.initialize_state(position);
                    self.has_revealed_any = true;
                }

                self.reveal(&position);
            }
            Message::ToggleMark(position) => self.toggle_mark(&position),
        }
    }

    pub fn view(&self) -> Element<Message> {
        // let rows = 0..=self.height;
        // let columns = 0..=self.width;

        Canvas::new(self).width(Fill).height(Fill).into()

        // column(rows.map(move |r| {
        //     row(columns.clone().map(move |c| {
        //         let position = Position::new(r, c);
        //         let Some(cell) = self.cells.get(&position) else {
        //             return "".into();
        //         };

        //         let state = match cell {
        //             CellState {
        //                 is_revealed: true,
        //                 cell_type: CellType::Mine,
        //                 ..
        //             } => crate::cell::State::Revealed(crate::cell::Content::Mine),
        //             CellState {
        //                 is_revealed: true,
        //                 cell_type: CellType::NonMine { neighbours },
        //                 ..
        //             } => crate::cell::State::Revealed(crate::cell::Content::Number(*neighbours)),
        //             _ => crate::cell::State::Normal,
        //         };

        //         Cell::new()
        //             .with_state(state)
        //             .on_click_with(move |button| match button {
        //                 Button::Left => Some(Message::Reveal(position)),
        //                 Button::Right => Some(Message::ToggleMark(position)),
        //                 _ => None,
        //             })
        //             .into()
        //     }))
        //     .into()
        // }))
        // .into()
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum InteractionState {
    #[default]
    None,
    Pressed(Button, Position),
}

impl canvas::Program<Message> for GameState {
    type State = InteractionState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(
            Point::ORIGIN,
            frame.size(),
            Color::from_rgb8(0x20, 0x20, 0x20),
        );

        frame.with_save(|frame| {
            frame.scale(32.0);

            for (position, cell) in &self.cells {
                let (color, text): (Color, Option<String>) = match cell {
                    CellState {
                        is_revealed: true,
                        cell_type: CellType::Mine,
                        ..
                    } => (Color::from_rgb8(0xff, 0, 0), Some("•".to_owned())),
                    CellState {
                        is_revealed: true,
                        cell_type: CellType::NonMine { neighbours },
                        ..
                    } if *neighbours > 0 => (
                        Color::from_rgb8(0xff, 0xff, 0xff),
                        Some(format!("{neighbours}")),
                    ),
                    CellState {
                        is_revealed: true,
                        cell_type: CellType::NonMine { neighbours: 0 },
                        ..
                    } => (Color::from_rgb8(0xff, 0xff, 0xff), None),
                    CellState {
                        is_revealed: false,
                        marking: Marking::Flag,
                        ..
                    } => (Color::from_rgb8(0xff, 0x30, 0x10), Some("!".to_owned())),
                    CellState {
                        is_revealed: false,
                        marking: Marking::QuestionMark,
                        ..
                    } => (Color::from_rgb8(0x20, 0x80, 0x40), Some("?".to_owned())),
                    _ => (Color::from_rgb8(0x20, 0x20, 0x20), None),
                };

                let position = Point::new(position.column as f32, position.row as f32);
                frame.fill_rectangle(position, Size::UNIT, color);
                let position = Point::new(position.x + 0.5, position.y + 0.5);

                if let Some(content) = text {
                    frame.fill_text(Text {
                        content,
                        position,
                        size: 0.7.into(),
                        color: Color::BLACK,
                        horizontal_alignment: iced::alignment::Horizontal::Center,
                        vertical_alignment: iced::alignment::Vertical::Center,
                        ..Default::default()
                    });
                }
            }
        });

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        mouse::Interaction::Pointer
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let Some(cursor_position) = cursor.position_in(bounds) else {
            return (event::Status::Ignored, None);
        };

        let position = Position::at(cursor_position);
        let cell = self.cells.get(&position);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                *state = InteractionState::Pressed(button, position);

                (event::Status::Captured, None)
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                if matches!(state, &mut InteractionState::Pressed(b, p) if b == button && p == position)
                {
                    let message = match button {
                        Button::Left => Some(Message::Reveal(position)),
                        Button::Right => Some(Message::ToggleMark(position)),
                        _ => None,
                    };

                    (event::Status::Captured, message)
                } else {
                    (event::Status::Ignored, None)
                }
            }
            _ => (event::Status::Ignored, None),
        }
    }
}
