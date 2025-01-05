use std::time::Duration;
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

    static ref ICONS_VOLUME: Vec<svg::Handle> = vec![
        svg::Handle::from_memory(include_bytes!("icons/vol0.svg")),
        svg::Handle::from_memory(include_bytes!("icons/vol1.svg")),
        svg::Handle::from_memory(include_bytes!("icons/vol2.svg")),
        svg::Handle::from_memory(include_bytes!("icons/vol3.svg")),
        svg::Handle::from_memory(include_bytes!("icons/vol4.svg")),
    ];
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
        use iced::{widget, Center, Fill};

        let song_info = self.song_info
            .as_ref()
            .map(|x| x.view(self.show_song_info, self.show_coverart))
            .unwrap_or(widget::text("").into());

        let progress_bar = self.progress
            .as_ref()
            .filter(|_| self.show_progress)
            .map(|x| x.view());


        let media_buttons = {
            let icon_play = svg(ICON_PLAY.clone())
                .style(icon_style_button);

            let icon_pause = svg(ICON_PAUSE.clone())
                .style(icon_style_button);

            let icon_prev = svg(ICON_PREV.clone())
                .style(icon_style_button);

            let icon_next = svg(ICON_NEXT.clone())
                .style(icon_style_button);

            widget::Row::new()
                .align_y(Center)
                .push(widget::button(icon_prev)
                    .style(button_style)
                    .width(38)
                    .on_press(Cmd::Prev)
                )
                .push(if self.is_playing() {
                    widget::button(icon_pause)
                        .style(button_style)
                        .width(50)
                        .on_press(Cmd::Pause)
                } else {
                    widget::button(icon_play)
                        .style(button_style)
                        .width(50)
                        .on_press(Cmd::Play)
                })
                .push(widget::button(icon_next)
                    .style(button_style)
                    .width(38)
                    .on_press(Cmd::Next)
                )
        };

        let volume_slider = {
            let volume = self.get_volume();
            let index = ((volume + 24) / 25) as usize;
            let icon_volume = svg(ICONS_VOLUME[index].clone())
                .width(20)
                .style(icon_style_volume);

            let slider = widget::slider(0..=100, self.get_volume(), Cmd::SetVolume)
                .width(100);

            widget::Row::new()
                .spacing(18)
                .align_y(Center)
                .push(slider)
                .push(icon_volume)
        };

        let control_bar = {
            use iced::widget::Container;

            let timing = self.progress
                .as_ref()
                .map(|p| p.timing())
                .unwrap_or(String::new());

            let volume_container = Container::new(volume_slider)
                .height(40)
                .align_y(Center)
                .align_right(Fill);

            widget::Row::new()
                .push(Container::new(widget::text(timing)).align_left(Fill))
                .push(media_buttons)
                .push(volume_container)
        };


        let progress_and_control_bar = widget::Column::new()
            .spacing(3)
            .push_maybe(progress_bar)
            .push(control_bar);

        let option_togglers = self.status
            .as_ref()
            .filter(|_| self.show_options)
            .map(|status| widget::Row::new()
                .push(widget::toggler(status.random)
                    .label("random")
                    .text_size(12)
                    .on_toggle(Cmd::SetRandom)
                )
                .push(widget::toggler(status.repeat)
                    .label("loop")
                    .text_size(12)
                    .on_toggle(Cmd::SetRepeat)
                )
                .push(widget::toggler(status.consume)
                    .label("consume")
                    .text_size(12)
                    .on_toggle(Cmd::SetConsume)
                )
                .spacing(32)
                .align_y(Center)
            );

        widget::Column::new()
            .align_x(Center)
            .spacing(25)
            .padding(20)
            .push_maybe(option_togglers)
            .push(widget::center(song_info))
            .push(progress_and_control_bar)
            .into()
    }

    pub fn set_elapsed(&mut self, duration: Duration) {
        let Some(mut status) = self.status.take() else {
            return
        };

        if status.elapsed.is_none() {
            return;
        }

        status.elapsed = Some(duration);
        let playing = status.state == PlayState::Playing;
        self.progress = match (status.elapsed, status.duration) {
            (Some(e), Some(d)) => Some(Progress::new(e, d, playing)),
            _ => None,
        };
        self.status = Some(status);
    }

    pub fn is_playing(&self) -> bool {
        self.status
            .as_ref()
            .map(|status| status.state == PlayState::Playing)
            .unwrap_or(false)
    }

    fn get_volume(&self) -> u8 {
        self.status
            .as_ref()
            .map(|status| std::cmp::min(status.volume, 100))
            .unwrap_or(0)
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

    pub fn get_random(&self) -> Option<bool> {
        self.status
            .as_ref()
            .map(|status| status.random)
    }

    pub fn get_loop(&self) -> Option<bool> {
        self.status
            .as_ref()
            .map(|status| status.repeat)
    }

    pub fn get_consume(&self) -> Option<bool> {
        self.status
            .as_ref()
            .map(|status| status.consume)
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
    use iced::{
        widget::button::{Status, Style},
        Background,
    };

    let pal = theme.extended_palette();
    let base = button::secondary(theme, status);

    match status {
        Status::Active => Style {
            background: None,
            ..base
        },
        Status::Hovered => Style {
            background: Some(Background::Color(pal.primary.base.color)),
            ..base
        },

        _ => base,
    }
}
