mod player;
mod queue;

use bytes::BytesMut;
use iced::{widget, Task, Element, Subscription};
use mpd_client::{
    responses::{
        Status,
        SongInQueue,
    },
    client::Subsystem,
    commands::SongId,
};

use crate::error::Error;
use crate::mpd::{self, MpdEvent, MpdCtrl};

use player::Player;
use queue::Queue;

#[derive(Debug, Clone)]
pub enum AppMsg {
    Reconnect,
    Connect(MpdCtrl),
    Error(Error),
    Op(ConMsg),
    Quit,
}


#[derive(Debug, Clone)]
pub enum ConMsg {
    Change(Subsystem),
    Player(mpd::Cmd),
    CmdResult(mpd::CmdResult),
    UpdateSongInfo(Status),
    UpdateQueue(Vec<SongInQueue>),
    UpdateMixer(Status),
    UpdateCoverArt(SongId, Option<(BytesMut, Option<String>)>),
}


pub enum App {
    Unconnected,
    Connected(AppConnected),
    Error(Error),
}

impl App {
    pub fn new() -> (Self, Task<AppMsg>) {
        (Self::Unconnected, Self::connect())
    }

    fn connect() -> Task<AppMsg> {
        mpd::connect()
            .map(|result| match result {
                Ok(MpdEvent::Connected(ctrl)) => AppMsg::Connect(ctrl),
                Ok(MpdEvent::Change(sub)) => AppMsg::Op(ConMsg::Change(sub)),
                Err(error) => AppMsg::Error(error),
            })
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

            AppMsg::Connect(ctrl) => {
                let con = AppConnected::new(ctrl.clone());
                *self = Self::Connected(con);

                // First thing after connection is retrieving the queue
                Task::perform(async move {
                        ctrl.get_queue().await
                    },
                    |result| match result {
                        Ok(queue) => AppMsg::Op(ConMsg::UpdateQueue(queue)),
                        Err(error) => AppMsg::Error(error),
                    },
                )
            }

            AppMsg::Op(msg) => match self {
                Self::Connected(con) => con.update(msg),
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

            Self::Connected(con) => con.view().map(AppMsg::Op),

            Self::Error(error) => widget::column![
                widget::text("Error").size(40),
                widget::text(error.to_string()).size(20),
                widget::button("Reconnect").on_press(AppMsg::Reconnect)
            ].spacing(20).align_x(iced::Center).into(),
        };

        widget::center(content).into()
    }

    pub fn subscription(&self) -> Subscription<AppMsg> {
        use iced::keyboard::{Key, Modifiers};

        iced::keyboard::on_key_press(|k, m| {
            match (k, m) {
                (Key::Character(v) , Modifiers::CTRL) if v == "q"
                    => Some(AppMsg::Quit),

                _ => None,
            }
        })
    }
}


pub struct AppConnected {
    ctrl: MpdCtrl,
    player: Player,
    queue: Queue,
}

impl AppConnected {
    fn new(ctrl: MpdCtrl) -> Self {
        Self {
            ctrl,
            player: Player::default(),
            queue: Queue::default(),
        }
    }

    pub fn update(&mut self, msg: ConMsg) -> Task<AppMsg> {
        match msg {
            ConMsg::Change(sub) => {
                tracing::info!("change of subsystem: {sub:?}");

                use mpd_client::client::Subsystem;
                match sub {
                    Subsystem::Player => self.update_song_info(),
                    Subsystem::Queue => self.update_queue(),
                    Subsystem::Mixer => self.update_mixer(),

                    _ => Task::none(),
                }
            }

            ConMsg::UpdateSongInfo(status) => {
                tracing::info!("update song information");

                self.player.set_mixer(&status);
                if let Some(id) = status.current_song.map(|t| t.1) {
                    if let Some(info) = self.queue.get(id) {
                        self.player.set_song_info(info);
                    } else {
                        return Task::done(AppMsg::Error(Error::InvalidQueue));
                    }
                } else {
                    self.player.clear();
                }

                Task::none()
            }

            ConMsg::UpdateQueue(queue) => {
                tracing::info!("update queue");

                self.queue.update(queue);
                self.retrieve_cover_art()
            }

            ConMsg::UpdateMixer(status) => {
                self.player.set_mixer(&status);
                Task::none()
            }

            ConMsg::UpdateCoverArt(id, data) => {
                use iced::widget::image::Handle;

                tracing::info!("update cover art for id {}", id.0);

                let image = data.map(|t| Handle::from_bytes(t.0));

                self.queue
                    .update_coverart(id, image);

                self.retrieve_cover_art()
            }

            ConMsg::Player(cmd) => {
                // to make the volume slider react faster we inject this
                // value back ourself
                if let mpd::Cmd::SetVolume(vol) = cmd {
                    self.player.set_volume(vol);
                }

                let cc = self.ctrl.clone();
                Task::perform(
                    async move {
                        cc.command(cmd).await
                    },
                    |result| AppMsg::Op(ConMsg::CmdResult(result)),
                )
            }

            ConMsg::CmdResult(mpd::CmdResult { cmd, error }) => {
                if let Some(msg) = error {
                    tracing::error!("command {cmd:?} returned error: {msg}");
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<ConMsg> {
        self.player
            .view()
            .map(ConMsg::Player)
    }

    fn update_song_info(&self) -> Task<AppMsg> {
        let cc = self.ctrl.clone();
        Task::perform(
            async move {
                cc.get_status().await
            },

            |result| match result {
                Ok(status) => AppMsg::Op(ConMsg::UpdateSongInfo(status)),
                Err(error) => AppMsg::Error(error),
            }
        )
    }

    fn update_queue(&self) -> Task<AppMsg> {
        let cc = self.ctrl.clone();
        Task::perform(
            async move {
                cc.get_queue().await
            },
            |result| match result {
                Ok(queue) => AppMsg::Op(ConMsg::UpdateQueue(queue)),
                Err(error) => AppMsg::Error(error),
            }
        )
    }

    fn update_mixer(&self) -> Task<AppMsg> {
        tracing::info!("updating mixer");

        let cc = self.ctrl.clone();
        Task::perform(
            async move {
                cc.get_status().await
            },
            |result| match result {
                Ok(status) => AppMsg::Op(ConMsg::UpdateMixer(status)),
                Err(error) => AppMsg::Error(error),
            }
        )
    }

    fn retrieve_cover_art(&self) -> Task<AppMsg> {
        let Some((id, uri)) = self.queue.get_missing_art() else {
            return self.update_song_info();
        };

        tracing::info!("retrieving cover art for {uri}");

        let cc = self.ctrl.clone();
        Task::perform(
            async move {
                cc.get_cover_art(&uri).await
            },
            move |result| match result {
                Ok(art) => AppMsg::Op(ConMsg::UpdateCoverArt(id, art)),

                // Handle "File Not Found" (code 50) response
                Err(Error::MpdErrorResponse(50))
                    => AppMsg::Op(ConMsg::UpdateCoverArt(id, None)),

                Err(error) => AppMsg::Error(error),
            }
        )
    }
}
