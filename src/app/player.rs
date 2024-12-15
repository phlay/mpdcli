use lazy_static::lazy_static;
use iced::{
    widget::{svg, button},
    Element,
    Theme,
};
use mpd_client::{
    commands::SongId,
    responses::{Status, PlayState},
};

use crate::mpd::Cmd;
use super::song_info::SongInfo;
use super::progress::Progress;

lazy_static! {
    static ref ICON_PLAY: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/play.svg"));

    static ref ICON_PAUSE: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/pause.svg"));

    static ref ICON_PREV: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/prev.svg"));

    static ref ICON_NEXT: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/next.svg"));

    static ref ICON_VOL_MIN: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/volmin.svg"));

    static ref ICON_VOL_MAX: svg::Handle =
        svg::Handle::from_memory(include_bytes!("icons/volmax.svg"));
}


pub struct Player {
    song_info: Option<SongInfo>,
    progress: Option<Progress>,
    status: Option<Status>,
    show_song_info: bool,
    show_coverart: bool,
    show_progress: bool,
    show_options: bool,
}


impl Player {
    pub fn new() -> Self {
        Self {
            song_info: None,
            progress: None,
            status: None,

            show_song_info: true,
            show_coverart: true,
            show_progress: true,
            show_options: true,
        }
    }

    pub fn set_song_info(&mut self, info: SongInfo) {
        self.song_info = Some(info);
    }

    pub fn clear_song_info(&mut self) {
        self.song_info = None;
    }

    pub fn update_status(&mut self, status: Status) {
        let playing = status.state == PlayState::Playing;
        self.progress = match (status.elapsed, status.duration) {
            (Some(e), Some(d)) => Some(Progress::new(e, d, playing)),
            _ => None,
        };
        self.status = Some(status);
    }

    pub fn view(&self) -> Element<Cmd> {
        use iced::{widget, Center};

        let icon_play = svg(ICON_PLAY.clone())
            .width(40)
            .style(icon_style_button);

        let icon_pause = svg(ICON_PAUSE.clone())
            .width(40)
            .style(icon_style_button);

        let icon_prev = svg(ICON_PREV.clone())
            .width(20)
            .style(icon_style_button);

        let icon_next = svg(ICON_NEXT.clone())
            .width(20)
            .style(icon_style_button);

        let media_buttons = widget::Row::new()
            .spacing(40)
            .align_y(Center)
            .push(widget::button(icon_prev)
                .style(button_style)
                .on_press(Cmd::Prev)
            )
            .push(if self.is_playing() {
                widget::button(icon_pause)
                    .style(button_style)
                    .on_press(Cmd::Pause)
            } else {
                widget::button(icon_play)
                    .style(button_style)
                    .on_press(Cmd::Play)
            })
            .push(widget::button(icon_next)
                .style(button_style)
                .on_press(Cmd::Next)
            );


        let volume_slider = {
            let icon_vol_min = svg(ICON_VOL_MIN.clone())
                .width(20)
                .style(icon_style_volume);

            let icon_vol_max = svg(ICON_VOL_MAX.clone())
                .width(20)
                .style(icon_style_volume);

            let slider = widget::slider(0..=100, self.get_volume(), Cmd::SetVolume)
                .width(220);

            widget::Row::new()
                .spacing(18)
                .align_y(Center)
                .push(icon_vol_min)
                .push(slider)
                .push(icon_vol_max)
        };

        let song_info = self.song_info
            .as_ref()
            .map(|x| x.view(self.show_song_info, self.show_coverart));

        let progress = self.progress
            .as_ref()
            .filter(|_| self.show_progress)
            .map(|x| x.view());

        let main_display = widget::Column::new()
            .spacing(40)
            .align_x(Center)
            .push_maybe(song_info)
            .push_maybe(progress)
            .push(media_buttons)
            .push(volume_slider);


        let option_togglers = self.status
            .as_ref()
            .filter(|_| self.show_options)
            .map(|s| {
            widget::Row::new()
            .push(widget::toggler(s.random)
                .label("random")
                .text_size(12)
                .on_toggle(Cmd::SetRandom)
            )
            .push(widget::toggler(s.repeat)
                .label("repeat")
                .text_size(12)
                .on_toggle(Cmd::SetRepeat)
            )
            .push(widget::toggler(s.consume)
                .label("consume")
                .text_size(12)
                .on_toggle(Cmd::SetConsume)
            )
            .spacing(32)
            .align_y(Center)
        });


        widget::Column::new()
            .align_x(Center)
            .padding(10)
            .push(widget::center(main_display))
            .push_maybe(option_togglers)
            .into()
    }

    pub fn is_playing(&self) -> bool {
        if let Some(status) = self.status.as_ref() {
            status.state == PlayState::Playing
        } else {
            false
        }
    }

    fn get_volume(&self) -> u8 {
        if let Some(status) = self.status.as_ref() {
            status.volume
        } else {
            0
        }
    }

    pub fn set_volume(&mut self, volume: u8) {
        if let Some(status) = self.status.as_mut() {
            status.volume = volume;
        }
    }

    pub fn get_current_id(&self) -> Option<SongId> {
        self.status
            .as_ref()
            .and_then(|status| status.current_song.map(|x| x.1))
    }

    pub fn get_next_id(&self) -> Option<SongId> {
        self.status
            .as_ref()
            .and_then(|status| status.next_song.map(|x| x.1))
    }

    pub fn toggle_show_song_info(&mut self) {
        self.show_song_info = !self.show_song_info;
    }

    pub fn toggle_show_coverart(&mut self) {
        self.show_coverart = !self.show_coverart;
    }

    pub fn toggle_show_progress(&mut self) {
        self.show_progress = !self.show_progress;
    }

    pub fn toggle_show_options(&mut self) {
        self.show_options = !self.show_options;
    }

    pub fn get_song_title(&self) -> Option<&str> {
        self.song_info
            .as_ref()
            .map(|nfo| nfo.title.as_str())
    }
}

fn icon_style_volume(theme: &Theme, _status: svg::Status) -> svg::Style {
    let pal = theme.extended_palette();
    let color = pal.secondary.base.color;
    svg::Style { color: Some(color) }
}

fn icon_style_button(theme: &Theme, _status: svg::Status) -> svg::Style {
    let pal = theme.extended_palette();
    // actually a label would be primary.strong.text, but this
    // looks to intense for our svgs
    let color = pal.primary.base.text;
    svg::Style { color: Some(color) }
}

fn button_style(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        border: iced::border::rounded(5),
        ..button::primary(theme, status)
    }
}
