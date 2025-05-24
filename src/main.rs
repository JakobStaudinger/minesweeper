use game_state::GameState;
use iced::{
    Element,
    widget::{button, column},
};

mod game_state;

fn main() -> iced::Result {
    iced::run("Minesweeper", Application::update, Application::view)
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
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectDifficulty(difficulty) => {
                let (width, height, mines) = match difficulty {
                    Difficulty::Easy => (10, 6, 20),
                    Difficulty::Medium => (20, 12, 120),
                    Difficulty::Hard => (60, 35, 1400),
                };
                self.state = ApplicationState::Game(GameState::new(width, height, mines))
            }
            Message::GameMessage(message) => {
                if let ApplicationState::Game(state) = &mut self.state {
                    state.update(message)
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            ApplicationState::Menu => column![
                button("Easy").on_press(Message::SelectDifficulty(Difficulty::Easy)),
                button("Medium").on_press(Message::SelectDifficulty(Difficulty::Medium)),
                button("Hard").on_press(Message::SelectDifficulty(Difficulty::Hard)),
            ]
            .into(),
            ApplicationState::Game(game_state) => game_state
                .view()
                .map(|message| Message::GameMessage(message)),
        }
    }
}
