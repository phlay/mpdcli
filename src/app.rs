use iced::{widget, Task, Element};

use crate::error::Error;
use crate::player::Player;
use crate::mpd::{self, MpdMsg, MpdCtrl};

#[derive(Debug, Clone)]
pub enum AppMsg {
    Reconnect,
    Mpd(Result<MpdMsg, Error>),
    Player(mpd::Cmd),
    CmdResult(mpd::CmdResult),
}


pub enum App {
    Unconnected,

    Connected {
        player: Player,
        mpd_ctrl: MpdCtrl,
    },

    Error(Error),
}

impl App {
    pub fn new() -> (Self, Task<AppMsg>) {
        (Self::Unconnected, Self::connect())
    }

    fn connect() -> Task<AppMsg> {
        mpd::connect()
            .map(AppMsg::Mpd)
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

            AppMsg::Mpd(Ok(msg)) => match msg {
                MpdMsg::Connected(mpd_ctrl) => {
                    *self = Self::Connected { mpd_ctrl, player: Player::default() };
                    Task::none()
                }

                MpdMsg::Song(info) => {
                    match self {
                        Self::Connected { player, .. } => player.update_song(info),
                        _ => (),
                    }
                    Task::none()
                }
            }

            AppMsg::Mpd(Err(error)) => {
                *self = Self::Error(error);
                Task::none()
            }


            AppMsg::Player(cmd) => {
                let Self::Connected { mpd_ctrl, .. } = self else {
                    return Task::none();
                };

                let cc = mpd_ctrl.clone();
                Task::perform(async move { cc.command(cmd).await }, AppMsg::CmdResult)
            }

            // Ignore successful command results
            AppMsg::CmdResult(mpd::CmdResult { error: None, .. }) => Task::none(),

            AppMsg::CmdResult(mpd::CmdResult { cmd, error: Some(msg) }) => {
                tracing::error!("command {cmd:?} returned error: {msg}");
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
