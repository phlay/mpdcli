use iced::Element;

use crate::app::AppMsg;
use crate::remote::SongInfo;


#[derive(Debug, Clone, Default)]
pub struct Player {
    song: Option<SongInfo>,
}

impl Player {
    pub fn clear_song(&mut self) {
        tracing::info!("no song registered");
        self.song = None;
    }

    pub fn update_song(&mut self, song: SongInfo) {
        tracing::info!("song updated: {song:?}");
        self.song = Some(song);
    }

    pub fn view(&self) -> Element<AppMsg> {
        use iced::{widget, font, Font, Center};
        let song_description: Element<_> = match self.song.as_ref() {
            Some(info) => {
                let title = widget::text(&info.title)
                    .size(40)
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
                widget::text("No song loaded")
                    .size(40)
                    .align_x(Center)
                    .into()
            }
        };

        let buttons = widget::row![
            widget::button("prev"),
            widget::button("play"),
            widget::button("next"),
        ].spacing(30);


        widget::column![
            song_description,
            buttons,
        ].spacing(50).align_x(Center).into()
    }
}
