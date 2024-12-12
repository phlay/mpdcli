use lazy_static::lazy_static;
use iced::{widget::svg, Element};
use mpd_client::{
    commands::SongId,
    responses::{
        Status,
        PlayState,
    },
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
}



pub struct Player {
    song_info: Option<SongInfo>,
    progress: Option<Progress>,
    status: Option<Status>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            song_info: None,
            progress: None,
            status: None,
        }
    }
}

impl Player {
    pub fn set_song_info(&mut self, info: SongInfo) {
        self.song_info = Some(info);
    }

    pub fn clear(&mut self) {
        self.song_info = None;
    }

    pub fn update_status(&mut self, status: Status) {
        self.progress = match (status.elapsed, status.duration) {
            (Some(e), Some(d)) => Some(Progress::new(e, d)),
            _ => None,
        };

        self.status = Some(status);
    }

    pub fn update_progress(&mut self) {
        if self.is_playing() {
            if let Some(progress) = self.progress.as_mut() {
                progress.update();
            }
        }
    }

    pub fn view(&self) -> Element<Cmd> {
        use iced::{widget, Center};

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
            .push(if self.is_playing() {
                widget::button(icon_pause).on_press(Cmd::Pause)
            } else {
                widget::button(icon_play).on_press(Cmd::Play)
            })
            .push(widget::button(icon_next).on_press(Cmd::Next));

        let volume_slider = widget::slider(0..=100, self.get_volume(), Cmd::SetVolume)
            .width(200);

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

    fn is_playing(&self) -> bool {
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
