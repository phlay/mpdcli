use std::collections::HashMap;
use bytes::BytesMut;
use mpd_client::{
    responses::SongInQueue,
    commands::SongId,
};

use super::song_info::SongInfo;

#[derive(Default)]
pub struct Queue {
    infos: HashMap<SongId, SongInfo>,
}

impl Queue {
    pub fn update(&mut self, queue: Vec<SongInQueue>) {
        self.infos = queue.into_iter()
            .map(|v| (v.id, v.into()))
            .collect();
    }

    pub fn update_coverart(&mut self, id: SongId, data: Option<BytesMut>) {
        if let Some(info) = self.infos.get_mut(&id) {
            info.update_coverart(data);
        }
    }

    pub fn get(&self, id: &SongId) -> Option<&SongInfo> {
        self.infos.get(id)
    }
}
