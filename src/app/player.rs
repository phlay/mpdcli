use lazy_static::lazy_static;
use iced::{widget::svg, Element};
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


#[derive(Default)]
pub struct Player {
    song_info: Option<SongInfo>,
    progress: Option<Progress>,
    status: Option<Status>,
}


impl Player {
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
            .width(36)
            .style(icon_style_button);

        let icon_pause = svg(ICON_PAUSE.clone())
            .width(36)
            .style(icon_style_button);

        let icon_prev = svg(ICON_PREV.clone())
            .width(20)
            .style(icon_style_button);

        let icon_next = svg(ICON_NEXT.clone())
            .width(20)
            .style(icon_style_button);

        let media_buttons = widget::Row::new()
            .spacing(35)
            .align_y(Center)
            .push(widget::button(icon_prev).on_press(Cmd::Prev))
            .push(if self.is_playing() {
                widget::button(icon_pause).on_press(Cmd::Pause)
            } else {
                widget::button(icon_play).on_press(Cmd::Play)
            })
            .push(widget::button(icon_next).on_press(Cmd::Next));


        let volume_slider = {
            let icon_vol_min = svg(ICON_VOL_MIN.clone())
                .width(20)
                .style(icon_style_volume);

            let icon_vol_max = svg(ICON_VOL_MAX.clone())
                .width(20)
                .style(icon_style_volume);

            let slider = widget::slider(0..=100, self.get_volume(), Cmd::SetVolume)
                .width(200);

            widget::Row::new()
                .spacing(22)
                .align_y(Center)
                .push(icon_vol_min)
                .push(slider)
                .push(icon_vol_max)
        };

        let main_display = widget::Column::new()
            .spacing(40)
            .align_x(Center)
            .push_maybe(self.song_info.as_ref().map(|x| x.view()))
            .push_maybe(self.progress.as_ref().map(|x| x.view()))
            .push(media_buttons)
            .push(volume_slider);


        let option_togglers = self.status.as_ref().map(|s| {
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
            .spacing(30)
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
}

fn icon_style_volume(theme: &iced::Theme, _status: svg::Status) -> svg::Style {
    let pal = theme.extended_palette();
    let color = pal.primary.strong.color;
    svg::Style { color: Some(color) }
}

fn icon_style_button(theme: &iced::Theme, _status: svg::Status) -> svg::Style {
    let pal = theme.extended_palette();
    let color = pal.primary.strong.text;
    svg::Style { color: Some(color) }
}
