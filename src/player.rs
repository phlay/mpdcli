use iced::Element;

use crate::mpd::SongInfo;

#[derive(Debug, Clone)]
pub enum PlayerMsg {
    Play,
    Prev,
    Next,
}

#[derive(Debug, Clone, Default)]
pub struct Player {
    song: Option<SongInfo>,
}

impl Player {
    pub fn update_song(&mut self, song: Option<SongInfo>) {
        tracing::info!("song updated: {song:?}");
        self.song = song;
    }

    pub fn view(&self) -> Element<PlayerMsg> {
        use iced::{widget, font, widget::image, Font, Center};

        let artwork = self.song.as_ref()
            .and_then(|nfo| nfo.album_art.clone())
            .map(|h| image::viewer(h).width(250));

        let song_description: Element<_> = match self.song.as_ref() {
            Some(info) => {
                let title = widget::text(&info.title)
                    .size(25)
                    .font(Font { weight: font::Weight::Bold, ..Font::default() });
                let artist = widget::text(&info.artist)
                    .size(20);
                let album = widget::text(&info.album)
                    .size(20);

                widget::column![
                    title,
                    artist,
                    album,
                ].spacing(8).align_x(Center).into()
            }

            None => {
                widget::text("Queue Finished")
                    .size(40)
                    .align_x(Center)
                    .into()
            }
        };

        let buttons = widget::row![
            widget::button("prev").on_press(PlayerMsg::Prev),
            widget::button("play").on_press(PlayerMsg::Play),
            widget::button("next").on_press(PlayerMsg::Next),
        ].spacing(30);

        widget::Column::new()
            .spacing(50)
            .align_x(Center)
            .push_maybe(artwork)
            .push(song_description)
            .push(buttons)
            .into()
    }
}
