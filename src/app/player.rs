use iced::{widget::image, widget::svg, Element};
use mpd_client::responses::{
    Status,
    PlayState,
};

use crate::mpd::Cmd;
use super::queue::SongInfo;

const ICON_DATA_PLAY: &'static [u8] = include_bytes!("icons/play.svg");
const ICON_DATA_PAUSE: &'static [u8] = include_bytes!("icons/pause.svg");
const ICON_DATA_PREV: &'static [u8] = include_bytes!("icons/prev.svg");
const ICON_DATA_NEXT: &'static [u8] = include_bytes!("icons/next.svg");

#[derive(Debug, Clone)]
pub struct Player {
    album: String,
    artist: String,
    title: String,
    coverart: Option<image::Handle>,

    volume: u8,
    state: PlayState,
    repeat: bool,
    random: bool,
    consume: bool,

    icon_play: svg::Handle,
    icon_pause: svg::Handle,
    icon_next: svg::Handle,
    icon_prev: svg::Handle,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            album: String::new(),
            artist: String::new(),
            title: String::new(),
            coverart: None,

            volume: 0,
            state: PlayState::Stopped,
            repeat: false,
            random: false,
            consume: false,

            icon_play: svg::Handle::from_memory(ICON_DATA_PLAY),
            icon_pause: svg::Handle::from_memory(ICON_DATA_PAUSE),
            icon_next: svg::Handle::from_memory(ICON_DATA_NEXT),
            icon_prev: svg::Handle::from_memory(ICON_DATA_PREV),
        }
    }
}

impl Player {
    pub fn set_song_info(
        &mut self,
        info: SongInfo,
    ) {
        self.album = info.album;
        self.artist = info.artist;
        self.title = info.title;
        self.coverart = info.coverart;
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
    }

    pub fn set_mixer(&mut self, status: &Status) {
        self.volume = status.volume;
        self.state = status.state;
        self.repeat = status.repeat;
        self.random = status.random;
        self.consume = status.consume;
    }

    pub fn clear(&mut self) {
        self.album = String::new();
        self.artist = String::new();
        self.title = String::new();
        self.coverart = None;
    }

    pub fn view(&self) -> Element<Cmd> {
        use iced::{
            font,
            widget,
            Font,
            Center,
        };

        let artwork: Element<_> = self.coverart
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
                .size(25)
                .font(Font { weight: font::Weight::Bold, ..Font::default() });
            let artist = widget::text(&self.artist)
                .size(18);
            let album = widget::text(&self.album)
                .size(18);

            widget::column![
                title,
                artist,
                album,
            ].spacing(8).align_x(Center).into()
        };


        let icon_play = svg(self.icon_play.clone())
            .width(35);
        let icon_pause = svg(self.icon_pause.clone())
            .width(35);
        let icon_prev = svg(self.icon_prev.clone())
            .width(20);
        let icon_next = svg(self.icon_next.clone())
            .width(20);

        let buttons = widget::Row::new()
            .spacing(35)
            .align_y(Center)
            .push(widget::button(icon_prev).on_press(Cmd::Prev))
            .push(match self.state {
                PlayState::Stopped | PlayState::Paused
                    => widget::button(icon_play)
                        .on_press(Cmd::Play),
                PlayState::Playing
                    => widget::button(icon_pause)
                        .on_press(Cmd::Pause),
            })
            .push(widget::button(icon_next).on_press(Cmd::Next));

        let volume_slider = widget::slider(0..=100, self.volume, Cmd::SetVolume)
            .width(200);

        widget::Column::new()
            .spacing(50)
            .align_x(Center)
            .push(artwork)
            .push(description)
            .push(buttons)
            .push(volume_slider)
            .into()
    }
}
