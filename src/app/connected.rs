use iced::{Task, Element};
use bytes::BytesMut;
use mpd_client::{
    responses::{
        Status,
        SongInQueue,
    },
    client::Subsystem,
    commands::SongId,
};

use crate::mpd::{self, MpdCtrl};
use crate::error::Error;
use super::player::Player;
use super::queue::Queue;

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

pub struct Connected {
    ctrl: MpdCtrl,
    player: Player,
    queue: Queue,
}

impl Connected {
    pub fn new(ctrl: MpdCtrl) -> Self {
        Self {
            ctrl,
            player: Player::default(),
            queue: Queue::default(),
        }
    }

    pub fn update(&mut self, msg: ConMsg) -> Task<Result<ConMsg, Error>> {
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
                        return Task::done(Err(Error::InvalidQueue));
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
                    async move { cc.command(cmd).await },
                    |result| Ok(ConMsg::CmdResult(result)),
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

    pub fn view(&self) -> Element<ConMsg> {
        self.player
            .view()
            .map(ConMsg::Player)
    }

    pub fn update_queue(&self) -> Task<Result<ConMsg, Error>> {
        let cc = self.ctrl.clone();
        Task::perform(
            async move { cc.get_queue().await },
            |result| match result {
                Ok(queue) => Ok(ConMsg::UpdateQueue(queue)),
                Err(error) => Err(error),
            }
        )
    }

    fn update_song_info(&self) -> Task<Result<ConMsg, Error>> {
        let cc = self.ctrl.clone();
        Task::perform(
            async move {
                cc.get_status().await
            },

            |result| match result {
                Ok(status) => Ok(ConMsg::UpdateSongInfo(status)),
                Err(error) => Err(error),
            }
        )
    }

    fn update_mixer(&self) -> Task<Result<ConMsg, Error>> {
        tracing::info!("updating mixer");

        let cc = self.ctrl.clone();
        Task::perform(
            async move {
                cc.get_status().await
            },
            |result| match result {
                Ok(status) => Ok(ConMsg::UpdateMixer(status)),
                Err(error) => Err(error),
            }
        )
    }

    fn retrieve_cover_art(&self) -> Task<Result<ConMsg, Error>> {
        let Some((id, uri)) = self.queue.get_missing_art() else {
            return self.update_song_info();
        };

        tracing::info!("retrieving cover art for {uri}");

        let cc = self.ctrl.clone();
        Task::perform(
            async move { cc.get_cover_art(&uri).await },

            move |result| match result {
                Ok(art) => Ok(ConMsg::UpdateCoverArt(id, art)),

                // Handle "File Not Found" (code 50) response
                Err(Error::MpdErrorResponse(50))
                    => Ok(ConMsg::UpdateCoverArt(id, None)),

                // Escalate other errors
                Err(error) => Err(error),
            }
        )
    }
}
