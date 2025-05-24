use std::collections::HashMap;

use iced::{
    Background, Border, Color, Element,
    widget::{Button, button::Style, column, row, text},
};
use itertools::iproduct;
use rand::seq::IteratorRandom;

#[derive(Clone, Copy, Debug)]
enum CellType {
    Mine,
    NonMine { neighbours: usize },
}

#[derive(Clone, Copy, Debug)]
struct CellState {
    is_revealed: bool,
    marking: Marking,
    cell_type: CellType,
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
enum Marking {
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
    fn next(self) -> Self {
        match self {
            Marking::None => Marking::Flag,
            Marking::Flag => Marking::QuestionMark,
            Marking::QuestionMark => Marking::None,
        }
    }
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
            self.cells.insert(
                p,
                CellState {
                    cell_type: CellType::Mine,
                    ..Default::default()
                },
            );

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
        let rows = 0..=self.height;
        let columns = 0..=self.width;

        column(rows.map(move |r| {
            row(columns.clone().map(move |c| {
                let cell = self.cells.get(&Position::new(r, c));
                let content = match cell {
                    Some(&CellState {
                        is_revealed: false, ..
                    })
                    | None => text("").center(),
                    Some(&CellState {
                        cell_type: CellType::Mine,
                        ..
                    }) => text("•").center(),
                    Some(&CellState {
                        cell_type: CellType::NonMine { neighbours },
                        ..
                    }) if neighbours > 0 => text(neighbours).center(),
                    _ => text("").center(),
                };

                Button::new(content)
                    .width(32)
                    .height(32)
                    .on_press(Message::Reveal(Position::new(r, c)))
                    .style(move |_, _| match cell {
                        Some(&CellState {
                            is_revealed: false, ..
                        })
                        | None => {
                            let mut style = Style::default()
                                .with_background(Background::Color(Color::from_rgb8(60, 60, 60)));
                            style.border = Border::default()
                                .width(1)
                                .color(Color::from_rgb8(100, 100, 100));
                            style
                        }
                        Some(&CellState {
                            cell_type: CellType::Mine,
                            ..
                        }) => {
                            let mut style = Style::default()
                                .with_background(Background::Color(Color::from_rgb8(255, 0, 0)));
                            style.border = Border::default()
                                .width(1)
                                .color(Color::from_rgb8(100, 100, 100));
                            style
                        }
                        Some(&CellState {
                            cell_type: CellType::NonMine { .. },
                            ..
                        }) => {
                            let mut style = Style::default().with_background(Background::Color(
                                Color::from_rgb8(240, 240, 240),
                            ));
                            style.border = Border::default()
                                .width(1)
                                .color(Color::from_rgb8(100, 100, 100));
                            style
                        }
                    })
                    .into()
            }))
            .into()
        }))
        .into()
    }
}
