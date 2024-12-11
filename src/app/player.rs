use std::time::{Instant, Duration};
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

    elapsed: Option<Duration>,
    duration: Option<Duration>,

    pub volume: u8,
    state: PlayState,
    repeat: bool,
    random: bool,
    consume: bool,

    last_updated: Instant,

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
            elapsed: None,
            duration: None,
            last_updated: Instant::now(),

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

    pub fn clear(&mut self) {
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
        self.elapsed = status.elapsed;
        self.duration = status.duration;
        self.last_updated = Instant::now();
    }

    pub fn update_progress(&mut self) {
        if self.state == PlayState::Stopped || self.state == PlayState::Paused {
            return;
        }

        let Some(elapsed) = self.elapsed else {
            return
        };

        let now = Instant::now();
        self.elapsed = Some(elapsed + now.duration_since(self.last_updated));
        self.last_updated = now;
    }

    pub fn view(&self) -> Element<Cmd> {
        use iced::{
            font,
            widget,
            Font,
            Center,
            Fill,
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


        let progress_bar = {
            let value = match (self.elapsed, self.duration) {
                (Some(e), Some(d)) if !e.is_zero() => e.div_duration_f32(d),
                _ => 0.0,
            };

            let bar = widget::progress_bar(0.0..=1.0, value)
                .height(45)
                .width(300);


            let elapsed = self.elapsed
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            let duration = self.duration
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            let remaining = duration - elapsed;

            let timing = widget::row![
                widget::text(format!("{}:{:02}", elapsed / 60, elapsed % 60))
                    .width(Fill),
                widget::text(format!("-{}:{:02}", remaining / 60, remaining % 60)),
            ].width(300);

            widget::column![bar, timing]
        };


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
            ].spacing(5).align_x(Center).into()
        };

        let icon_play = svg(self.icon_play.clone())
            .width(36);
        let icon_pause = svg(self.icon_pause.clone())
            .width(36);
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

        let main_display = widget::Column::new()
            .spacing(45)
            .align_x(Center)
            .push(artwork)
            .push(progress_bar)
            .push(description)
            .push(buttons)
            .push(volume_slider);


        let togglers = widget::Row::new()
            .push(widget::toggler(self.random)
                .label("random")
                .on_toggle(Cmd::SetRandom)
            )
            .push(widget::toggler(self.repeat)
                .label("repeat")
                .on_toggle(Cmd::SetRepeat)
            )
            .push(widget::toggler(self.consume)
                .label("consume")
                .on_toggle(Cmd::SetConsume)
            )
            .spacing(30)
            .align_y(Center);


        widget::Column::new()
            .align_x(Center)
            .padding(10)
            .push(widget::center(main_display))
            .push(togglers)
            .into()
    }
}
