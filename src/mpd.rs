mod mpd_ctrl;
mod mpd_events;

use iced::Task;

use crate::error::Error;
pub use mpd_events::MpdEvent;
pub use mpd_ctrl::{MpdCtrl, Cmd, CmdResult};

pub fn connect() -> Task<Result<MpdEvent, Error>> {
    Task::stream(iced::stream::try_channel(1, |tx| async {
        mpd_events::MpdEvents::open()
            .await?
            .run(tx)
            .await
    }))
}
