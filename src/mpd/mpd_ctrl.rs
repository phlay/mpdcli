use mpd_client::{
    Client,
    client::CommandError,
    responses::Status,
};

#[derive(Debug, Clone)]
pub enum Cmd {
    Play,
    Prev,
    Next,
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
        };

        CmdResult { cmd, error: error.map(|e| e.to_string()) }
    }
}
