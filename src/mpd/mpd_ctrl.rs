use bytes::BytesMut;
use mpd_client::{
    Client,
    responses::{
        Status,
        SongInQueue,
    }
};

use crate::error::Error;

#[derive(Debug, Clone)]
pub enum Cmd {
    Play,
    Pause,
    Prev,
    Next,
    SetVolume(u8),
    SetRandom(bool),
    SetRepeat(bool),
    SetConsume(bool),
}

#[derive(Clone, Debug)]
pub struct CmdResult {
    pub cmd: Cmd,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MpdCtrl {
    client: Client,
}


impl MpdCtrl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn command(&self, cmd: Cmd) -> CmdResult {
        use mpd_client::commands;

        let error = match cmd {
            Cmd::Play => {
                self.client.command(commands::Play::current())
                    .await
                    .err()
            }

            Cmd::Pause => {
                self.client.command(commands::SetPause(true))
                    .await
                    .err()
            }

            Cmd::Prev => {
                self.client.command(commands::Previous)
                    .await
                    .err()
            }

            Cmd::Next => {
                self.client.command(commands::Next)
                    .await
                    .err()
            }

            Cmd::SetVolume(vol) => {
                self.client.command(commands::SetVolume(vol))
                    .await
                    .err()
            }

            Cmd::SetRandom(b) => {
                self.client.command(commands::SetRandom(b))
                    .await
                    .err()
            }

            Cmd::SetRepeat(b) => {
                self.client.command(commands::SetRepeat(b))
                    .await
                    .err()
            }

            Cmd::SetConsume(b) => {
                self.client.command(commands::SetConsume(b))
                    .await
                    .err()
            }
        };

        CmdResult { cmd, error: error.map(|e| e.to_string()) }
    }

    pub async fn get_status(&self) -> Result<Status, Error> {
        self.client
            .command(mpd_client::commands::Status)
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_queue(&self) -> Result<Vec<SongInQueue>, Error> {
        self.client
            .command(mpd_client::commands::Queue::all())
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_cover_art(&self, uri: &str)
        -> Result<Option<(BytesMut, Option<String>)>, Error>
    {
        self.client
            .album_art(uri)
            .await
            .map_err(|e| e.into())
    }
}
