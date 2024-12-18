mod connected;
mod song_info;
mod queue;
mod progress;
mod player;

use std::time::Duration;
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
    const PERIODIC_REDRAW: Duration = Duration::from_millis(250);

    pub fn new() -> (Self, Task<AppMsg>) {
        (Self::Unconnected, Self::connect())
    }

    fn connect() -> Task<AppMsg> {
        mpd_connect().map(AppMsg::from)
    }

    pub fn title(&self) -> String {
        let title = match self {
            Self::Unconnected => "Unconnected",
            Self::Connected(con) => con.title(),
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
                let request_queue = con.request_queue()
                    .map(AppMsg::from);

                *self = Self::Connected(con);
                request_queue
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
            Self::Unconnected
                => widget::text("Connecting to MPD").size(20).into(),

            Self::Connected(con) => con.view().map(AppMsg::Operate),

            Self::Error(error) => widget::Column::new()
                .spacing(20)
                .align_x(iced::Center)
                .push(widget::text("Error").size(40))
                .push(widget::text(error.to_string()).size(20))
                .push(widget::button("Reconnect").on_press(AppMsg::Reconnect))
                .into(),
        };

        widget::center(content).into()
    }

    pub fn subscriptions(&self) -> Subscription<AppMsg> {
        Subscription::batch([
            self.subscribe_redraw_timer(),
            self.subscribe_keyboard(),
        ])
    }

    fn subscribe_keyboard(&self) -> Subscription<AppMsg> {
        use iced::keyboard::{Key, Modifiers, key::Named};
        use crate::mpd::Cmd;
        use connected::Toggle;

        iced::keyboard::on_key_press(|k, m| {
            match (k, m) {
                (Key::Named(Named::Escape), _)
                    => Some(AppMsg::Quit),

                (Key::Named(Named::Space), _)
                    => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::Play))),

                (Key::Character(v), Modifiers::CTRL) if v == "q"
                    => Some(AppMsg::Quit),

                (Key::Character(key), _) => match key.as_str() {
                    "f" | "n" => Some(AppMsg::Operate(ConMsg::Player(Cmd::Next))),
                    "b" => Some(AppMsg::Operate(ConMsg::Player(Cmd::Prev))),
                    "o" => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::ShowOptions))),
                    "i" => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::ShowSongInfo))),
                    "a" => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::ShowCoverArt))),
                    "p" => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::ShowProgress))),
                    "r" => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::Random))),
                    "l" => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::Loop))),
                    "c" => Some(AppMsg::Operate(ConMsg::Toggle(Toggle::Consume))),
                    _ => None,
                },

                _ => None,
            }
        })
    }

    fn subscribe_redraw_timer(&self) -> Subscription<AppMsg> {
        match self {
            App::Connected(con) if con.is_playing() => {
                iced::time::every(Self::PERIODIC_REDRAW)
                    .map(|_| AppMsg::Operate(ConMsg::Redraw))
            }

            _ => Subscription::none(),
        }
    }
}
