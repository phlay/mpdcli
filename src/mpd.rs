mod mpd_ctrl;
mod mpd_listen;

use iced::{widget::image, Task};

use crate::error::Error;
use mpd_listen::MpdListen;
pub use mpd_listen::MpdMsg;
pub use mpd_ctrl::{MpdCtrl, Cmd, CmdResult};

#[derive(Debug, Clone, Default)]
pub struct SongInfo {
    pub album: String,
    pub artist: String,
    pub title: String,
    pub album_art: Option<image::Handle>,
}

pub fn connect() -> Task<Result<MpdMsg, Error>> {
    Task::stream(iced::stream::try_channel(1, |tx| async {
        MpdListen::open()
            .await?
            .run(tx)
            .await
    }))
}
