use std::collections::HashMap;

use iced::{
    Color, Element,
    Length::Fill,
    Point, Renderer, Size, Theme,
    advanced::{graphics::core::event, mouse},
    mouse::Button,
    widget::{
        Canvas,
        canvas::{self, Event, Frame, Text},
    },
};
use itertools::iproduct;
use rand::seq::IteratorRandom;

#[derive(Clone, Copy, Debug)]
pub enum CellType {
    Mine,
    NonMine { neighbours: usize },
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub is_revealed: bool,
    pub marking: Marking,
    pub cell_type: CellType,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            is_revealed: false,
            marking: Marking::None,
            cell_type: CellType::NonMine { neighbours: 0 },
        }
    }
}

impl Cell {
    pub fn mine() -> Self {
        Cell {
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

#[derive(Clone, Debug)]
pub struct GameState {
    cells: HashMap<Position, Cell>,
    width: usize,
    height: usize,
    mines: usize,
    has_revealed_any: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Reveal(Position),
    ToggleMark(Position),
    RevealSurrounding(Position),
}

impl GameState {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let cells = HashMap::from_iter(
            (0..=width)
                .flat_map(|c| (0..=height).map(move |r| (Position::new(r, c), Cell::default()))),
        );

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
            self.cells.insert(p, Cell::mine());

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
            if let Cell {
                is_revealed: false,
                marking: Marking::None,
                ..
            } = cell
            {
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

    fn reveal_surrounding(&mut self, position: &Position) {
        let cell = self.cells.get(&position);
        if let Some(&Cell {
            is_revealed: true,
            cell_type: CellType::NonMine { neighbours },
            ..
        }) = cell
        {
            let (marked, unmarked): (Vec<_>, Vec<_>) =
                position.neighbours().partition(|position| {
                    matches!(
                        self.cells.get(position),
                        Some(&Cell {
                            is_revealed: false,
                            marking: Marking::Flag,
                            ..
                        })
                    )
                });

            if marked.len() == neighbours {
                for n in unmarked {
                    self.reveal(&n);
                }
            }
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
            Message::RevealSurrounding(position) => self.reveal_surrounding(&position),
        }
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
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
        let cells = {
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
                        Cell {
                            is_revealed: true,
                            cell_type: CellType::Mine,
                            ..
                        } => (Color::from_rgb8(0xff, 0, 0), Some("•".to_owned())),
                        Cell {
                            is_revealed: true,
                            cell_type: CellType::NonMine { neighbours },
                            ..
                        } if *neighbours > 0 => (
                            Color::from_rgb8(0xff, 0xff, 0xff),
                            Some(format!("{neighbours}")),
                        ),
                        Cell {
                            is_revealed: true,
                            cell_type: CellType::NonMine { neighbours: 0 },
                            ..
                        } => (Color::from_rgb8(0xff, 0xff, 0xff), None),
                        Cell {
                            is_revealed: false,
                            marking: Marking::Flag,
                            ..
                        } => (Color::from_rgb8(0xff, 0x30, 0x10), Some("!".to_owned())),
                        Cell {
                            is_revealed: false,
                            marking: Marking::QuestionMark,
                            ..
                        } => (Color::from_rgb8(0x20, 0x80, 0x40), Some("?".to_owned())),
                        _ => (Color::from_rgb8(0x40, 0x40, 0x40), None),
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

            frame.into_geometry()
        };

        let overlay = {
            let mut frame = Frame::new(renderer, bounds.size());
            frame.scale(32.0);

            if let InteractionState::Pressed(button, position) = *state {
                match button {
                    Button::Middle => {
                        let neighbours = position
                            .neighbours()
                            .flat_map(|n| self.cells.get_key_value(&n))
                            .filter_map(|(position, cell)| {
                                if matches!(
                                    cell,
                                    Cell {
                                        is_revealed: false,
                                        marking: Marking::None,
                                        ..
                                    }
                                ) {
                                    Some(position)
                                } else {
                                    None
                                }
                            });

                        for n in neighbours {
                            let position = Point::new(n.column as f32, n.row as f32);
                            frame.fill_rectangle(
                                position,
                                Size::UNIT,
                                Color::from_rgb8(0x10, 0x10, 0x10),
                            );
                        }
                    }
                    _ => {
                        if let Some(&Cell {
                            is_revealed: false, ..
                        }) = self.cells.get(&position)
                        {
                            let position = Point::new(position.column as f32, position.row as f32);
                            frame.fill_rectangle(
                                position,
                                Size::UNIT,
                                Color::from_rgb8(0x10, 0x10, 0x10),
                            );
                        }
                    }
                }
            } else {
                let hovered_cell = cursor
                    .position_in(bounds)
                    .map(|position| Position::at(position))
                    .and_then(|position| self.cells.get_key_value(&position));

                if let Some((
                    &position,
                    &Cell {
                        is_revealed: false, ..
                    },
                )) = hovered_cell
                {
                    let position = Point::new(position.column as f32, position.row as f32);
                    frame.fill_rectangle(
                        position,
                        Size::UNIT,
                        Color::from_rgba8(0xff, 0xff, 0xff, 0.5),
                    );
                }
            }

            frame.into_geometry()
        };

        vec![cells, overlay]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        let Some(cursor_position) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };

        let position = Position::at(cursor_position);
        let cell = self.cells.get(&position);

        if let Some(&Cell {
            is_revealed: false, ..
        }) = cell
        {
            mouse::Interaction::Pointer
        } else {
            if let InteractionState::Pressed(_, pressed_position) = *state {
                if let Some(&Cell {
                    is_revealed: false, ..
                }) = self.cells.get(&pressed_position)
                {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::Idle
                }
            } else {
                mouse::Interaction::Idle
            }
        }
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
        let current_state = *state;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                *state = InteractionState::Pressed(button, position);

                (event::Status::Captured, None)
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                *state = InteractionState::None;

                if matches!(current_state, InteractionState::Pressed(b, p) if b == button && p == position)
                {
                    let message = match button {
                        Button::Left => Some(Message::Reveal(position)),
                        Button::Right => Some(Message::ToggleMark(position)),
                        Button::Middle => Some(Message::RevealSurrounding(position)),
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
