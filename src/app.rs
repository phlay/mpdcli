mod connected;
mod song_info;
mod queue;
mod progress;
mod player;

use iced::{widget, Task, Element, Subscription};
use crate::error::Error;
use crate::mpd::{MpdEvent, MpdCtrl, mpd_connect};

use connected::{Connected, ConMsg};

#[derive(Debug, Clone)]
pub enum AppMsg {
    Reconnect,
    Connect(MpdCtrl),
    Operate(ConMsg),
    Error(Error),
    Quit,
}

impl From<Result<ConMsg, Error>> for AppMsg {
    fn from(result: Result<ConMsg, Error>) -> Self {
        match result {
            Ok(msg) => AppMsg::Operate(msg),
            Err(err) => AppMsg::Error(err),
        }
    }
}

impl From<Result<MpdEvent, Error>> for AppMsg {
    fn from(result: Result<MpdEvent, Error>) -> Self {
        match result {
            Ok(MpdEvent::Connected(ctrl)) => AppMsg::Connect(ctrl),
            Ok(MpdEvent::Change(sub)) => AppMsg::Operate(ConMsg::Change(sub)),
            Err(error) => AppMsg::Error(error),
        }
    }
}


pub enum App {
    Unconnected,
    Connected(Connected),
    Error(Error),
}

impl App {
    const APP_NAME: &str = env!("CARGO_PKG_NAME");
    const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
    const TICK_MS: u64 = 500;

    pub fn new() -> (Self, Task<AppMsg>) {
        (Self::Unconnected, Self::connect())
    }

    fn connect() -> Task<AppMsg> {
        mpd_connect().map(AppMsg::from)
    }

    pub fn title(&self) -> String {
        let title = match self {
            Self::Unconnected => "Unconnected",
            Self::Connected(_) => "Connected",
            Self::Error(_) => "Error",
        };

        format!("{} {} - {}", Self::APP_NAME, Self::APP_VERSION, title)
    }

    pub fn update(&mut self, message: AppMsg) -> Task<AppMsg> {
        match message {
            AppMsg::Reconnect => {
                *self = Self::Unconnected;
                Self::connect()
            }

            AppMsg::Connect(ctrl) => {
                let con = Connected::new(ctrl);
                let update_queue = con.update_queue()
                    .map(AppMsg::from);

                *self = Self::Connected(con);
                update_queue
            }

            AppMsg::Operate(msg) => match self {
                Self::Connected(c) => c.update(msg).map(AppMsg::from),
                _ => Task::none(),
            }

            AppMsg::Error(error) => {
                *self = Self::Error(error);
                Task::none()
            }

            AppMsg::Quit => {
                std::process::exit(0);
            }
        }
    }

    pub fn view(&self) -> Element<AppMsg> {
        let content: Element<_> = match self {
            Self::Unconnected => {
                widget::text("Connecting").size(20).into()
            }

            Self::Connected(con) => con.view().map(AppMsg::Operate),

            Self::Error(error) => widget::column![
                widget::text("Error").size(40),
                widget::text(error.to_string()).size(20),
                widget::button("Reconnect").on_press(AppMsg::Reconnect)
            ].spacing(20).align_x(iced::Center).into(),
        };

        widget::center(content).into()
    }

    pub fn subscriptions(&self) -> Subscription<AppMsg> {
        let subs = vec![
            self.subscribe_tick(),
            self.subscribe_keyboard(),
        ];

        Subscription::batch(subs.into_iter())
    }

    fn subscribe_keyboard(&self) -> Subscription<AppMsg> {
        use iced::keyboard::{Key, Modifiers, key::Named};
        use crate::mpd::Cmd;

        iced::keyboard::on_key_press(|k, m| {
            match (k, m) {
                (Key::Character(v), Modifiers::CTRL) if v == "q"
                    => Some(AppMsg::Quit),
                (Key::Named(Named::Escape), _)
                    => Some(AppMsg::Quit),
                (Key::Character(v), _) if v == "f" || v == "n"
                    => Some(AppMsg::Operate(ConMsg::Player(Cmd::Next))),
                (Key::Character(v), _) if v == "b"
                    => Some(AppMsg::Operate(ConMsg::Player(Cmd::Prev))),
                (Key::Named(Named::Enter), _)
                    => Some(AppMsg::Operate(ConMsg::Player(Cmd::Play))),
                (Key::Named(Named::Space), _)
                    => Some(AppMsg::Operate(ConMsg::Player(Cmd::Pause))),

                _ => None,
            }
        })
    }

    fn subscribe_tick(&self) -> Subscription<AppMsg> {
        iced::time::every(std::time::Duration::from_millis(Self::TICK_MS))
            .map(|_| AppMsg::Operate(ConMsg::Tick))
    }
}
