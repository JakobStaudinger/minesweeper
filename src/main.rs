use game_state::GameState;
use iced::{
    Element,
    Length::Fill,
    Size, Task,
    widget::{button, column, text},
    window,
};

mod cell;
mod game_state;

fn main() -> iced::Result {
    iced::application("Minesweeper", Application::update, Application::view)
        .window_size(Size::new(300.0, 300.0))
        .run()
}

enum ApplicationState {
    Menu,
    Game(GameState),
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self::Menu
    }
}

#[derive(Default)]
struct Application {
    state: ApplicationState,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    SelectDifficulty(Difficulty),
    GameMessage(game_state::Message),
}

#[derive(Clone, Copy, Debug)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Application {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectDifficulty(difficulty) => {
                let (width, height, mines) = match difficulty {
                    Difficulty::Easy => (10, 8, 10),
                    Difficulty::Medium => (18, 14, 40),
                    Difficulty::Hard => (24, 20, 99),
                };
                self.state = ApplicationState::Game(GameState::new(width, height, mines));

                window::get_oldest().and_then(move |id| {
                    window::resize(id, Size::new((width * 32) as f32, (height * 32) as f32))
                })
            }
            Message::GameMessage(message) => {
                if let ApplicationState::Game(state) = &mut self.state {
                    state.update(message)
                }

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            ApplicationState::Menu => column![
                button(text("Easy").center().width(Fill))
                    .on_press(Message::SelectDifficulty(Difficulty::Easy))
                    .width(Fill),
                button(text("Medium").center().width(Fill))
                    .on_press(Message::SelectDifficulty(Difficulty::Medium))
                    .width(Fill),
                button(text("Hard").center().width(Fill))
                    .on_press(Message::SelectDifficulty(Difficulty::Hard))
                    .width(Fill),
            ]
            .padding(24)
            .spacing(12)
            .width(Fill)
            .into(),
            ApplicationState::Game(game_state) => game_state
                .view()
                .map(|message| Message::GameMessage(message)),
        }
    }
}
