mod error;
mod mpd;
mod app;

use crate::app::App;

pub fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(error) = iced::application(App::title, App::update, App::view)
        .subscription(App::subscriptions)
        .theme(|_| iced::Theme::KanagawaDragon)
        .run_with(App::new)
    {
        tracing::error!("error running iced runtime: {error}");
        std::process::exit(1);
    }
}
