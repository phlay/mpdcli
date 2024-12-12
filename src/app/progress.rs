use std::time::{Instant, Duration};
use crate::mpd::Cmd;

pub struct Progress {
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
        if self.elapsed >= self.duration {
            return
        }

        let now = Instant::now();
        self.elapsed += now.duration_since(self.last_update);
        if self.elapsed > self.duration {
            self.elapsed = self.duration;
        }

        self.last_update = now;
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

        column![bar, timing].spacing(3).into()
    }

    fn progress(&self) -> f32 {
        if self.duration.is_zero() {
           0.0
        } else {
            self.elapsed.div_duration_f32(self.duration)
        }
    }
}
