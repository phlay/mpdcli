use iced::{widget, Task, Element};

use crate::error::Error;
use crate::player::Player;
use crate::remote::RemoteMsg;

#[derive(Debug, Clone)]
pub enum AppMsg {
    Reconnect,
    Remote(Result<RemoteMsg, Error>),
}


pub enum App {
    Unconnected,
    Connected(Player),
    Error(Error),
}

impl App {
    pub fn new() -> (Self, Task<AppMsg>) {
        (Self::Unconnected, Self::connect())
    }

    fn connect() -> Task<AppMsg> {
        use iced::stream::try_channel;
        use crate::remote::task_handle_mpd;

        Task::stream(try_channel(1, task_handle_mpd))
            .map(AppMsg::Remote)
    }

    pub fn title(&self) -> String {
        let title = match self {
            Self::Unconnected => "Unconnected",
            Self::Connected(_) => "Connected",
            Self::Error(_) => "Error",
        };

        format!("Player - {title}")
    }

    pub fn update(&mut self, message: AppMsg) -> Task<AppMsg> {
        match message {
            AppMsg::Reconnect => {
                *self = Self::Unconnected;
                Self::connect()
            }

            AppMsg::Remote(Ok(msg)) => match msg {
                RemoteMsg::Connected(_client) => {
                    *self = Self::Connected(Player::default());
                    Task::none()
                }

                RemoteMsg::Disconnect => {
                    // Maybe a reconnect screen would be better
                    *self = Self::Unconnected;
                    Self::connect()
                }

                RemoteMsg::Song(info) => {
                    match self {
                        Self::Connected(player) => player.update_song(info),
                        _ => (),
                    }
                    Task::none()
                }

                RemoteMsg::NoSong => {
                    match self {
                        Self::Connected(player) => player.clear_song(),
                        _ => (),
                    }
                    Task::none()
                }
            }

            AppMsg::Remote(Err(error)) => {
                *self = Self::Error(error);
                Task::none()
            }
        }
    }


    pub fn view(&self) -> Element<AppMsg> {
        tracing::info!("view called");
        let content: Element<_> = match self {
            Self::Unconnected => {
                widget::text("Connecting").size(20).into()
            }

            Self::Connected(player) => player.view(),

            Self::Error(error) => widget::column![
                widget::text("Can't connect to MPD").size(20),
                widget::text(error.to_string()).size(20),
                widget::button("Reconnect").on_press(AppMsg::Reconnect)
            ].spacing(20).align_x(iced::Center).into(),
        };

        widget::center(content).into()
    }
}
