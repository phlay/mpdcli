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

use crate::mpd::{MpdCtrl, Cmd, CmdResult};
use crate::error::Error;
use super::player::Player;
use super::queue::Queue;

#[derive(Debug, Clone)]
pub enum ConMsg {
    Change(Subsystem),
    Player(Cmd),
    Tick,
    CmdResult(CmdResult),
    UpdateSongInfo(Status),
    UpdateQueue(Vec<SongInQueue>),
    UpdateStatus(Status),
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
                tracing::debug!("change of subsystem: {sub:?}");

                use mpd_client::client::Subsystem;
                match sub {
                    Subsystem::Player => self.request_song_info(),
                    Subsystem::Queue => self.request_queue(),
                    Subsystem::Mixer => self.request_status(),
                    Subsystem::Options => self.request_status(),

                    _ => Task::none(),
                }
            }

            ConMsg::Player(cmd) => {
                // to make the volume slider react faster we inject the
                // user requested volume back before the server supplies
                // us with the real value (which should be identical).
                if let Cmd::SetVolume(vol) = cmd {
                    self.player.set_volume(vol);
                }

                let cc = self.ctrl.clone();
                Task::perform(
                    async move { cc.command(cmd).await },
                    |result| Ok(ConMsg::CmdResult(result)),
                )
            }

            ConMsg::Tick => {
                tracing::trace!("tick");
                self.player.update_progress();
                Task::none()
            }

            ConMsg::UpdateSongInfo(status) => {
                tracing::debug!("update song information");

                self.player.update_status(status);

                if let Some(id) = self.player.get_current_id() {
                    if let Some(info) = self.queue.get(&id) {
                        self.player.set_song_info(info.clone());
                        if info.is_cover_missing() {
                            self.request_cover_art(id)
                        } else if let Some(next) = self.player.get_next_id() {
                            self.request_cover_art(next)
                        } else {
                            Task::none()
                        }
                    } else {
                        tracing::error!("current song {} not in queue", id.0);
                        Task::done(Err(Error::InvalidQueue))
                    }

                } else {
                    self.player.clear_song_info();
                    Task::none()
                }
            }

            ConMsg::UpdateQueue(queue) => {
                tracing::debug!("update queue");
                self.queue.update(queue);
                self.request_song_info()
            }

            ConMsg::UpdateStatus(status) => {
                tracing::debug!("update player status");
                self.player.update_status(status);
                Task::none()
            }

            ConMsg::UpdateCoverArt(id, data) => {
                tracing::debug!("update cover art for id {}", id.0);
                self.queue.update_coverart(id, data.map(|x| x.0));

                if self.player.get_current_id() == Some(id) {
                    // we got the current cover, update player
                    if let Some(info) = self.queue.get(&id) {
                        self.player.set_song_info(info.clone());
                    } else {
                        tracing::error!("current song {} not in queue", id.0);
                    }

                    // now also try to retrieve cover for next song
                    if let Some(next_id) = self.player.get_next_id() {
                        self.request_cover_art(next_id)
                    } else {
                        Task::none()
                    }

                } else {
                    // we got the cover for something else
                    if let Some(current_id) = self.player.get_current_id() {
                        self.request_cover_art(current_id)
                    } else {
                        Task::none()
                    }
                }
            }

            ConMsg::CmdResult(CmdResult { cmd, error }) => {
                if let Some(msg) = error {
                    tracing::warn!("command {cmd:?} returned error: {msg}");
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<ConMsg> {
        self.player.view().map(ConMsg::Player)
    }

    pub fn request_queue(&self) -> Task<Result<ConMsg, Error>> {
        let cc = self.ctrl.clone();
        Task::perform(
            async move { cc.get_queue().await },
            |result| match result {
                Ok(queue) => Ok(ConMsg::UpdateQueue(queue)),
                Err(error) => Err(error),
            }
        )
    }

    fn request_song_info(&self) -> Task<Result<ConMsg, Error>> {
        let cc = self.ctrl.clone();
        Task::perform(
            async move { cc.get_status().await },
            |result| match result {
                Ok(status) => Ok(ConMsg::UpdateSongInfo(status)),
                Err(error) => Err(error),
            }
        )
    }

    fn request_status(&self) -> Task<Result<ConMsg, Error>> {
        let cc = self.ctrl.clone();
        Task::perform(
            async move { cc.get_status().await },
            |result| match result {
                Ok(status) => Ok(ConMsg::UpdateStatus(status)),
                Err(error) => Err(error),
            }
        )
    }

    fn request_cover_art(&self, id: SongId) -> Task<Result<ConMsg, Error>> {
        let Some(info) = self.queue.get(&id) else {
            tracing::warn!("requested cover artwork for unqueued song {}", id.0);
            return Task::none();
        };

        if !info.is_cover_missing() {
            return Task::none();
        }

        let url = info.get_url().to_owned();
        tracing::debug!("requesting cover art for {}: {url}", id.0);

        let cc = self.ctrl.clone();
        Task::perform(
            async move { cc.get_cover_art(&url).await },

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
