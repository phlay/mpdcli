mod error;
mod remote;
mod player;
mod app;

pub fn main() -> iced::Result {
    use crate::app::App;
    tracing_subscriber::fmt::init();
    iced::application(App::title, App::update, App::view)
        .run_with(App::new)
}
