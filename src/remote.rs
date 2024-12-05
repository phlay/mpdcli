use futures_channel::mpsc;
use iced::{
    widget::image,
    Task,
};
use mpd_client::{
    commands::Command,
    responses::{
        Status,
        SongInQueue,
    },
    client::{ConnectionEvents, Subsystem},
    Client,
};

use crate::error::Error;

#[derive(Debug, Clone, Default)]
pub struct SongInfo {
    pub album: String,
    pub artist: String,
    pub title: String,
    pub album_art: Option<image::Handle>,
}

#[derive(Debug, Clone)]
pub enum RemoteMsg {
    Connected(Client),
    Song(Option<SongInfo>),
}

pub struct Remote {
    client: Client,
    events: ConnectionEvents,
    queue: Vec<SongInQueue>,
}

impl Remote {
    const TARGET: &str = "localhost:6600";

    pub fn start() -> Task<Result<RemoteMsg, Error>> {
        Task::stream(iced::stream::try_channel(1, |tx| async {
            Self::open()
                .await?
                .run(tx)
                .await
        }))
    }

    async fn open() -> Result<Self, Error> {
        use mpd_client::commands;

        let stream = tokio::net::TcpStream::connect(Self::TARGET).await?;
        let (client, events) = Client::connect(stream).await?;
        let queue = client.command(commands::Queue::all()).await?;

        Ok(Remote { client, events, queue })
    }

    async fn run(mut self, mut tx: mpsc::Sender<RemoteMsg>) -> Result<(), Error> {
        use iced::futures::SinkExt;
        use mpd_client::client::ConnectionEvent;

        // inform user, that we are connected and hand out a client structure
        tx.send(RemoteMsg::Connected(self.client.clone())).await?;

        // load initial information bevor waiting for change
        let info = self.get_song_info().await?;
        tx.send(RemoteMsg::Song(info)).await?;

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
        tx: &mut mpsc::Sender<RemoteMsg>,
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
                tx.send(RemoteMsg::Song(info)).await?;
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

pub async fn mpd_command(
    client: Client,
    cmd: impl Command,
) {
    if let Err(err) = client.command(cmd).await {
        tracing::error!("error running mpd command: {err}");
    }
}
