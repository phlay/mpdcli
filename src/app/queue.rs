use std::collections::HashMap;
use iced::widget::image;
use mpd_client::{
    responses::SongInQueue,
    commands::SongId,
};

#[derive(Clone)]
pub struct SongInfo {
    pub id: SongId,
    pub album: String,
    pub artist: String,
    pub title: String,
    pub url: String,
    pub coverart: Option<image::Handle>,
    pub missing_cover: bool,
}

#[derive(Default)]
pub struct Queue {
    infos: HashMap<SongId, SongInfo>,
}

impl Queue {
    pub fn update(&mut self, queue: Vec<SongInQueue>) {
        self.infos = queue.into_iter()
            .map(|v| (v.id, SongInfo {
                id: v.id,
                title: v.song.title().unwrap_or("").to_owned(),
                artist: v.song.artists().join(", "),
                album: v.song.album().unwrap_or("").to_owned(),
                url: v.song.url.clone(),
                coverart: None,
                missing_cover: true,
            }))
            .collect();
    }

    pub fn update_coverart(&mut self, id: SongId, art: Option<image::Handle>) {
        if let Some(entry) = self.infos.get_mut(&id) {
            entry.coverart = art;
            entry.missing_cover = false;
        }
    }

    pub fn get(&self, id: &SongId) -> Option<&SongInfo> {
        self.infos.get(id)
    }

    pub fn get_missing(&self) -> Option<(SongId, String)> {
        self.infos
            .iter()
            .find(|(_, nfo)| nfo.missing_cover)
            .map(|(&id, nfo)| (id, nfo.url.clone()))
    }
}
