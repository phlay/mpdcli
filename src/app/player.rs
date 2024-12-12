use std::time::{Instant, Duration};
use lazy_static::lazy_static;
use iced::{widget::image, widget::svg, Element};
use mpd_client::{
    commands::SongId,
    responses::{
        Status,
        PlayState,
    },
};

use crate::mpd::Cmd;
use super::queue::SongInfo;

lazy_static! {
    static ref ICON_PLAY: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/play.svg"));

    static ref ICON_PAUSE: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/pause.svg"));

    static ref ICON_PREV: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/prev.svg"));

    static ref ICON_NEXT: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/next.svg"));
}



pub struct Player {
    pub current: Option<SongId>,
    album: String,
    artist: String,
    title: String,
    coverart: Option<image::Handle>,

    progress: Option<Progress>,

    pub volume: u8,
    state: PlayState,
    repeat: bool,
    random: bool,
    consume: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            current: None,
            album: String::new(),
            artist: String::new(),
            title: String::new(),
            coverart: None,

            progress: None,

            volume: 0,
            state: PlayState::Stopped,
            repeat: false,
            random: false,
            consume: false,
        }
    }
}

impl Player {
    pub fn set_song_info(&mut self, info: &SongInfo) {
        self.current = Some(info.id);
        self.album = info.album.clone();
        self.artist = info.artist.clone();
        self.title = info.title.clone();
        self.coverart = info.coverart.clone();
    }

    pub fn clear(&mut self) {
        self.current = None;
        self.album = String::new();
        self.artist = String::new();
        self.title = String::new();
        self.coverart = None;
    }

    pub fn update_status(&mut self, status: &Status) {
        // mixer
        self.volume = status.volume;
        self.state = status.state;
        // options
        self.random = status.random;
        self.repeat = status.repeat;
        self.consume = status.consume;
        // progress
        self.progress = match (status.elapsed, status.duration) {
            (Some(e), Some(d)) => Some(Progress::new(e, d)),
            _ => None,
        };
    }

    pub fn update_progress(&mut self) {
        if self.state == PlayState::Playing {
            if let Some(progress) = self.progress.as_mut() {
                progress.update();
            }
        }
    }

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


        let song_description: Element<_> = {
            let title = widget::text(&self.title)
                .size(26)
                .font(Font { weight: font::Weight::Bold, ..Font::default() });
            let artist = widget::text(&self.artist)
                .size(16);
            let album = widget::text(&self.album)
                .size(16);

            widget::column![
                title,
                artist,
                album,
            ].spacing(5).align_x(Center).into()
        };

        let icon_play = svg(ICON_PLAY.clone())
            .width(36);
        let icon_pause = svg(ICON_PAUSE.clone())
            .width(36);
        let icon_prev = svg(ICON_PREV.clone())
            .width(20);
        let icon_next = svg(ICON_NEXT.clone())
            .width(20);

        let media_buttons = widget::Row::new()
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


        let main_display = widget::Column::new()
            .spacing(40)
            .align_x(Center)
            .push(coverart)
            .push(song_description)
            .push_maybe(self.progress.as_ref().map(|t| t.view()))
            .push(media_buttons)
            .push(volume_slider);


        let option_togglers = widget::Row::new()
            .push(widget::toggler(self.random)
                .label("random")
                .text_size(12)
                .on_toggle(Cmd::SetRandom)
            )
            .push(widget::toggler(self.repeat)
                .label("repeat")
                .text_size(12)
                .on_toggle(Cmd::SetRepeat)
            )
            .push(widget::toggler(self.consume)
                .label("consume")
                .text_size(12)
                .on_toggle(Cmd::SetConsume)
            )
            .spacing(30)
            .align_y(Center);


        widget::Column::new()
            .align_x(Center)
            .padding(10)
            .push(widget::center(main_display))
            .push(option_togglers)
            .into()
    }
}

struct Progress {
    elapsed: Duration,
    duration: Duration,
    last_update: Instant,
}

impl Progress {
    pub fn new(elapsed: Duration, duration: Duration) -> Self {
        Self {
            elapsed,
            duration,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        if self.elapsed < self.duration {
            let now = Instant::now();
            self.elapsed += now.duration_since(self.last_update);
            self.last_update = now;
        }
    }

    pub fn view(&self) -> iced::Element<'_, Cmd> {
        use iced::widget::{progress_bar, text, row, column};
        use iced::Fill;

        let bar = progress_bar(0.0..=1.0, self.progress())
                .height(45)
                .width(300);

        let elapsed = self.elapsed.as_secs();
        let remaining = if self.elapsed < self.duration {
            self.duration.as_secs() - elapsed
        } else {
            0
        };

        let timing = row![
            text(format!("{}:{:02}", elapsed / 60, elapsed % 60))
                .size(12)
                .width(Fill),
            text(format!("-{}:{:02}", remaining / 60, remaining % 60))
                .size(12),
        ].width(300);

        column![bar, timing].spacing(2).into()
    }

    fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 0.0;
        }

        self.elapsed.div_duration_f32(self.duration)
    }
}
