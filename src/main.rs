mod error;
mod mpd;
mod app;

use crate::app::App;

pub fn main() {
    tracing_subscriber::fmt::init();

    if let Err(error) = iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| iced::Theme::Nord)
        .run_with(App::new)
    {
        tracing::error!("error running iced runtime: {error}");
        std::process::exit(1);
    }
}
