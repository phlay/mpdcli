use futures_channel::mpsc;
use mpd_client::{
    commands::SongId,
    Client,
};

use crate::error::Error;

#[derive(Debug, Clone, Default)]
pub struct SongInfo {
    pub album: String,
    pub artist: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub enum RemoteMsg {
    Connected(Client),
    Disconnect,
    Song(SongInfo),
    NoSong,
}


pub async fn task_handle_mpd(mut tx: mpsc::Sender<RemoteMsg>) -> Result<(), Error> {
    use mpd_client::commands;
    use iced::futures::SinkExt;

    let stream = tokio::net::TcpStream::connect("localhost:6600").await?;
    let (client, mut events) = Client::connect(stream).await?;

    tx.send(RemoteMsg::Connected(client.clone())).await?;

    while let Some(ev) = events.next().await {
        tracing::info!("mpd event: {ev:?}");

        let status = client.command(commands::Status).await?;

        match status.current_song.map(|s| s.1) {
            Some(id) => {
                let info = find_song_in_queue(&client, id).await?;
                tx.send(RemoteMsg::Song(info)).await?;
            }

            None => {
                tx.send(RemoteMsg::NoSong).await?;
            }
        }
    }

    tx.send(RemoteMsg::Disconnect).await?;

    Ok(())
}

async fn find_song_in_queue(client: &Client, id: SongId) -> Result<SongInfo, Error> {
    use mpd_client::commands;
    let queue = client.command(commands::Queue::all()).await?;

    match queue.iter().find(|&s| s.id == id) {
        Some(entry) => Ok(SongInfo {
            album: String::from(entry.song.album().unwrap_or("")),
            artist: entry.song.artists().join(", "),
            title: String::from(entry.song.title().unwrap_or("")),
        }),

        None => Err(Error::InvalidQueue),
    }
}
