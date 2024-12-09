use futures_channel::mpsc;
use iced::widget::image;
use mpd_client::{
    responses::{
        Status,
        SongInQueue,
    },
    client::{ConnectionEvents, Subsystem},
    Client,
};

use crate::error::Error;
use super::{MpdCtrl, SongInfo};

#[derive(Debug, Clone)]
pub enum MpdMsg {
    Connected(MpdCtrl),
    Song(Option<SongInfo>),
}

pub struct MpdListen {
    client: Client,
    events: ConnectionEvents,
    queue: Vec<SongInQueue>,
}

impl MpdListen {
    const TARGET: &str = "localhost:6600";
    const BINARY_LIMIT: usize = 655360;

    pub async fn open() -> Result<Self, Error> {
        use mpd_client::commands;

        let stream = tokio::net::TcpStream::connect(Self::TARGET).await?;
        let (client, events) = Client::connect(stream).await?;
        let queue = client.command(commands::Queue::all()).await?;

        Ok(MpdListen { client, events, queue })
    }

    pub async fn run(mut self, mut tx: mpsc::Sender<MpdMsg>) -> Result<(), Error> {
        use iced::futures::SinkExt;
        use mpd_client::{
            commands,
            client::ConnectionEvent,
        };

        // inform user, that we are connected and hand out a client structure
        tx.send(MpdMsg::Connected(MpdCtrl::new(self.client.clone()))).await?;

        // load initial information bevor waiting for change
        let info = self.get_song_info().await?;
        tx.send(MpdMsg::Song(info)).await?;

        self.client.command(commands::SetBinaryLimit(Self::BINARY_LIMIT)).await?;

        // listen for further events from mpd
        while let Some(ev) = self.events.next().await {
            match ev {
                ConnectionEvent::SubsystemChange(sub)
                    => self.subsystem_change(sub, &mut tx).await?,

                ConnectionEvent::ConnectionClosed(error)
                    => return Err(error.into()),
            }
        }

        Err(Error::Disconnect)
    }

    async fn subsystem_change(
        &mut self,
        subsystem: Subsystem,
        tx: &mut mpsc::Sender<MpdMsg>,
    ) -> Result<(), Error> {
        use iced::futures::SinkExt;
        use mpd_client::commands;

        match subsystem {
            Subsystem::Queue => {
                tracing::info!("reloading queue");

                // TODO: we should here load albumart for all queue entries

                self.queue = self.client
                    .command(commands::Queue::all())
                    .await?;
            }

            Subsystem::Mixer => {
                tracing::info!("volume change");
            }

            Subsystem::Player => {
                tracing::info!("player change");
                let info = self.get_song_info().await?;
                tx.send(MpdMsg::Song(info)).await?;
            }

            _ => {
                tracing::info!("ignoring subsystem change: {subsystem:?}");
            }
        }
        Ok(())
    }

    async fn get_status(&self) -> Result<Status, Error> {
        self.client
            .command(mpd_client::commands::Status)
            .await
            .map_err(|e| e.into())
    }


    async fn get_song_info(&self) -> Result<Option<SongInfo>, Error> {
        let status = self.get_status().await?;
        match status.current_song.map(|s| s.1) {
            Some(id) => {
                match self.queue.iter().find(|&song| song.id == id) {
                    Some(entry) => {
                        let album_art = self.client
                            .album_art(&entry.song.url)
                            .await
                            .ok()
                            .flatten()
                            .map(|img| image::Handle::from_bytes(img.0));

                        Ok(Some(SongInfo {
                            album: entry.song.album().unwrap_or("").to_owned(),
                            artist: entry.song.artists().join(", "),
                            title: entry.song.title().unwrap_or("").to_owned(),
                            album_art,
                        }))
                    }

                    None => Err(Error::InvalidQueue),
                }
            }
            None => Ok(None),
        }
    }
}
