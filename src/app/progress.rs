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
        use iced::widget::slider;

        let duration = self.duration.as_secs_f32();
        let elapsed = self.elapsed();

        slider(0.0..=duration, elapsed, |s| Cmd::Seek(Duration::from_secs_f32(s)))
            .into()
    }

    pub fn timing(&self) -> String {
        let (ela_min, ela_sec) = split_min_secs(self.elapsed());
        let (dur_min, dur_sec) = split_min_secs(self.duration.as_secs_f32());
        format!("{ela_min}:{ela_sec:02} / {dur_min}:{dur_sec:02}")
    }

    fn elapsed(&self) -> f32 {
        self.timestamp
            .map(|time| {
                let delta = Instant::now().duration_since(time);
                (self.elapsed + delta).as_secs_f32()
            })
            .unwrap_or(self.elapsed.as_secs_f32())
    }
}

fn split_min_secs(secs: f32) -> (i32, i32) {
    let sgn = secs.signum() as i32;
    let n = secs.abs().round() as i32;
    (sgn * n / 60, n % 60)
}
