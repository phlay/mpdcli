use bytes::BytesMut;
use mpd_client::responses::SongInQueue;
use iced::{
    widget::image,
    Element,
};

use crate::mpd::Cmd;

#[derive(Clone)]
pub struct SongInfo {
    album: String,
    artist: String,
    title: String,
    url: String,
    coverart: Option<image::Handle>,
    missing_cover: bool,
}


impl SongInfo {
    pub fn view(&self) -> Element<Cmd> {
        use iced::{
            font,
            widget,
            Font,
            Center,
        };

        let coverart: Element<_> = self.coverart
            .as_ref()
            .map(|handle| image(handle.clone())
                .height(200)
                .into()
            )
            .unwrap_or(widget::container("no artwork")
                .center(200)
                .style(widget::container::rounded_box)
                .into()
            );

        let description: Element<_> = {
            let title = widget::text(&self.title)
                .size(26)
                .font(Font { weight: font::Weight::Bold, ..Font::default() });
            let artist = widget::text(&self.artist)
                .size(16);
            let album = widget::text(&self.album)
                .size(16);

            widget::Column::new()
                .spacing(5)
                .align_x(Center)
                .push(title)
                .push(artist)
                .push(album)
                .into()
        };

        widget::Column::new()
            .align_x(Center)
            .spacing(40)
            .push(coverart)
            .push(description)
            .into()
    }

    pub fn is_cover_missing(&self) -> bool {
        self.missing_cover
    }

    pub fn update_coverart(&mut self, data: Option<BytesMut>) {
        self.coverart = data.map(image::Handle::from_bytes);
        self.missing_cover = false;
    }

    pub fn get_url(&self) -> &str {
        self.url.as_str()
    }
}

impl From<SongInQueue> for SongInfo {
    fn from(nfo: SongInQueue) -> Self {
        Self {
            title: nfo.song.title().unwrap_or("").to_owned(),
            artist: nfo.song.artists().join(", "),
            album: nfo.song.album().unwrap_or("").to_owned(),
            url: nfo.song.url.clone(),
            coverart: None,
            missing_cover: true,
        }
    }
}
