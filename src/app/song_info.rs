use bytes::BytesMut;
use mpd_client::responses::SongInQueue;
use iced::{
    widget::image,
    Element,
};

use crate::mpd::Cmd;

#[derive(Clone)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    url: String,
    coverart: Option<image::Handle>,
    missing_cover: bool,
}

impl SongInfo {
    pub fn view(&self, show_info: bool, show_art: bool) -> Element<Cmd> {
        use iced::{font, widget, border, Font, Center};

        let coverart: Option<Element<_>> = if show_art {
            Some(self.coverart
                .as_ref()
                .map(|handle| image(handle.clone())
                    .height(220)
                    .into()
                )
                .unwrap_or(widget::container("")
                    .center(220)
                    .style(|theme| widget::container::Style {
                        border: border::rounded(5),
                        ..widget::container::rounded_box(theme)
                    })
                    .into()
                )
            )
        } else {
            None
        };

        let description: Option<Element<_>> = if show_info {
            let title = widget::text(&self.title)
                .size(26)
                .font(Font { weight: font::Weight::Bold, ..Font::default() });
            let artist = widget::text(&self.artist)
                .size(16);
            let album = widget::text(&self.album)
                .size(16);

            Some(widget::Column::new()
                .spacing(5)
                .align_x(Center)
                .push(title)
                .push(artist)
                .push(album)
                .into())
        } else {
            None
        };

        widget::Column::new()
            .align_x(Center)
            .spacing(40)
            .push_maybe(coverart)
            .push_maybe(description)
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
        let title = if let Some(title) = nfo.song.title() {
            title.to_owned()
        } else {
            use std::path::Path;
            let path = Path::new(&nfo.song.url);
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(String::from("<unknown file>"))
        };

        Self {
            title,
            artist: nfo.song.artists().join(", "),
            album: nfo.song.album().unwrap_or("").to_owned(),
            url: nfo.song.url.clone(),
            coverart: None,
            missing_cover: true,
        }
    }
}
