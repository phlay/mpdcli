use std::collections::HashMap;
use iced::widget::image;
use mpd_client::{
    responses::SongInQueue,
    commands::SongId,
};

#[derive(Default)]
pub struct Queue {
    queue: Vec<SongInQueue>,
    coverart: HashMap<SongId, Option<image::Handle>>,
}

impl Queue {
    pub fn update(&mut self, queue: Vec<SongInQueue>) {
        self.queue = queue;
    }

    pub fn update_coverart(&mut self, id: SongId, art: Option<image::Handle>) {
        self.coverart.insert(id, art);
    }

    pub fn get(&self, id: SongId) -> Option<SongInfo> {
        self.queue
            .iter()
            .find(|&song| song.id == id)
            .map(|entry| {
                SongInfo {
                    album: entry.song.album().unwrap_or("").to_owned(),
                    artist: entry.song.artists().join(", "),
                    title: entry.song.title().unwrap_or("").to_owned(),
                    coverart: self.coverart.get(&id).cloned().flatten(),
                }
            })
    }

    pub fn get_missing_art(&self) -> Option<(SongId, String)> {
        self.queue
            .iter()
            .find(|&e| self.coverart.get(&e.id).is_none())
            .map(|e| (e.id, e.song.url.clone()))
    }
}


pub struct SongInfo {
    pub album: String,
    pub artist: String,
    pub title: String,
    pub coverart: Option<image::Handle>,
}
