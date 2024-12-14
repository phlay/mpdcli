use std::time::{Instant, Duration};
use crate::mpd::Cmd;

pub struct Progress {
    elapsed: Duration,
    duration: Duration,
    timestamp: Option<Instant>,
}

impl Progress {
    pub fn new(elapsed: Duration, duration: Duration, playing: bool) -> Self {
        let timestamp = if playing {
            Some(Instant::now())
        } else {
            None
        };

        Self {
            elapsed,
            duration,
            timestamp,
        }
    }

    pub fn view(&self) -> iced::Element<'_, Cmd> {
        use iced::widget::{progress_bar, text, row, column};
        use iced::Fill;

        let duration = self.duration.as_secs_f32();

        let elapsed = if let Some(time) = self.timestamp {
            let delta = Instant::now().duration_since(time);
            (self.elapsed + delta).as_secs_f32()
        } else {
            self.elapsed.as_secs_f32()
        };

        let remaining = if elapsed < duration {
            duration - elapsed
        } else {
            0.0
        };

        let progress = if duration > 0.1 {
            elapsed  / duration
        } else {
            0.0
        };

        let bar = progress_bar(0.0..=1.0, progress)
            .style(|theme| {
                progress_bar::Style {
                    border: iced::border::rounded(5),
                    ..progress_bar::primary(theme)
                }
            })
            .height(50)
            .width(320);

        let timing = row![
            text(show_min_secs(elapsed)).size(12).width(Fill),
            text(show_min_secs(-remaining)).size(12),
        ].width(320);

        column![bar, timing].spacing(3).into()
    }
}

fn show_min_secs(secs: f32) -> String {
    let sgn = secs.signum() as i32;
    let n = secs.abs().round() as i32;
    format!("{}:{:02}", sgn * n / 60, n % 60)
}
