use iced::{widget, Task, Element};

use crate::error::Error;
use crate::player::{Player, PlayerMsg};
use crate::remote::{Remote, RemoteMsg, mpd_command};

#[derive(Debug, Clone)]
pub enum AppMsg {
    Reconnect,
    Remote(Result<RemoteMsg, Error>),
    Player(PlayerMsg),
}


pub enum App {
    Unconnected,

    Connected {
        player: Player,
        client: mpd_client::Client,
    },

    Error(Error),
}

impl App {
    pub fn new() -> (Self, Task<AppMsg>) {
        (Self::Unconnected, Self::connect())
    }

    fn connect() -> Task<AppMsg> {
        Remote::start()
            .map(AppMsg::Remote)
    }

    pub fn title(&self) -> String {
        let title = match self {
            Self::Unconnected => "Unconnected",
            Self::Connected { .. } => "Connected",
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
                RemoteMsg::Connected(client) => {
                    *self = Self::Connected { client, player: Player::default() };
                    Task::none()
                }

                RemoteMsg::Song(info) => {
                    match self {
                        Self::Connected { player, .. } => player.update_song(info),
                        _ => (),
                    }
                    Task::none()
                }
            }

            AppMsg::Remote(Err(error)) => {
                *self = Self::Error(error);
                Task::none()
            }


            AppMsg::Player(msg) => {
                use mpd_client::commands;

                let Self::Connected { client, .. } = self else {
                    return Task::none();
                };

                let cc = client.clone();

                match msg {
                    PlayerMsg::Play => {
                        let _ = tokio::spawn(mpd_command(cc, commands::SetPause(true)));
                    }
                    PlayerMsg::Prev => {
                        let _ = tokio::spawn(mpd_command(cc, commands::Previous));
                    }
                    PlayerMsg::Next => {
                        let _ = tokio::spawn(mpd_command(cc, commands::Next));
                    }
                }

                Task::none()
            }
        }
    }


    pub fn view(&self) -> Element<AppMsg> {
        let content: Element<_> = match self {
            Self::Unconnected => {
                widget::text("Connecting").size(20).into()
            }

            Self::Connected { player, .. } => player.view().map(AppMsg::Player),

            Self::Error(error) => widget::column![
                widget::text("Can't connect to MPD").size(20),
                widget::text(error.to_string()).size(20),
                widget::button("Reconnect").on_press(AppMsg::Reconnect)
            ].spacing(20).align_x(iced::Center).into(),
        };

        widget::center(content).into()
    }
}
