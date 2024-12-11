use futures_channel::mpsc;
use mpd_client::{
    client::ConnectionEvents,
    Client,
};

pub use mpd_client::client::Subsystem;

use crate::error::Error;
use super::MpdCtrl;

#[derive(Debug, Clone)]
pub enum MpdEvent {
    Connected(MpdCtrl),
    Change(Subsystem),
}

pub struct MpdEvents {
    client: Client,
    events: ConnectionEvents,
}

impl MpdEvents {
    const TARGET: &str = "localhost:6600";
    const BINARY_LIMIT: usize = 655360;

    pub async fn open() -> Result<Self, Error> {
        let stream = tokio::net::TcpStream::connect(Self::TARGET).await?;
        let (client, events) = Client::connect(stream).await?;

        Ok(MpdEvents { client, events })
    }

    pub async fn run(mut self, mut tx: mpsc::Sender<MpdEvent>) -> Result<(), Error> {
        use iced::futures::SinkExt;
        use mpd_client::{
            commands,
            client::ConnectionEvent,
        };

        // Set large binary limit for faster cover-art download
        self.client.command(commands::SetBinaryLimit(Self::BINARY_LIMIT)).await?;

        // inform user, that we are connected and hand out a remote control
        tx.send(MpdEvent::Connected(MpdCtrl::new(self.client.clone()))).await?;

        // listen for further events from mpd
        while let Some(ev) = self.events.next().await {
            match ev {
                ConnectionEvent::SubsystemChange(sub)
                    => tx.send(MpdEvent::Change(sub)).await?,

                ConnectionEvent::ConnectionClosed(error)
                    => return Err(error.into()),
            }
        }

        Err(Error::Disconnect)
    }
}
