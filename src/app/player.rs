use iced::{widget::image, Element};

use crate::mpd::Cmd;
use super::queue::SongInfo;

#[derive(Debug, Clone, Default)]
pub struct Player {
    pub album: String,
    pub artist: String,
    pub title: String,
    pub coverart: Option<image::Handle>,
}

impl Player {
    pub fn update(
        &mut self,
        info: SongInfo,
    ) {
        self.album = info.album;
        self.artist = info.artist;
        self.title = info.title;
        self.coverart = info.coverart;
    }

    pub fn clear(&mut self) {
        self.album = String::new();
        self.artist = String::new();
        self.title = String::new();
        self.coverart = None;
    }

    pub fn view(&self) -> Element<Cmd> {
        use iced::{widget, font, widget::image, Font, Center};

        let artwork = self.coverart.as_ref()
            .map(|handle| image::viewer(handle.clone()).width(300));

        let description: Element<_> = {
            let title = widget::text(&self.title)
                .size(25)
                .font(Font { weight: font::Weight::Bold, ..Font::default() });
            let artist = widget::text(&self.artist)
                .size(20);
            let album = widget::text(&self.album)
                .size(20);

            widget::column![
                title,
                artist,
                album,
            ].spacing(8).align_x(Center).into()
        };

        let buttons = widget::row![
            widget::button("prev").on_press(Cmd::Prev),
            widget::button("play").on_press(Cmd::Play),
            widget::button("next").on_press(Cmd::Next),
        ].spacing(30);

        widget::Column::new()
            .spacing(50)
            .align_x(Center)
            .push_maybe(artwork)
            .push(description)
            .push(buttons)
            .into()
    }
}
