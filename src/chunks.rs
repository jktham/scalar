use bevy::prelude::*;
use fxhash::{FxHashMap, FxHashSet};

use crate::world::{CHUNK_SIZE_X, CHUNK_SIZE_Z};

#[derive(Default, Resource)]
/// look up entity ids by chunk
pub struct ChunkIndex {
    entities: FxHashMap<IVec2, FxHashSet<Entity>>,
}

impl ChunkIndex {
    pub fn add(&mut self, pos: &Vec3, entity: &Entity) {
        let chunk_pos = get_chunk_pos(pos);
        if !self.entities.contains_key(&chunk_pos) {
            self.entities.insert(chunk_pos, FxHashSet::default());
        }

        let set = self.entities.get_mut(&chunk_pos).unwrap();
        set.insert(entity.clone());
    }

    pub fn get(&self, pos: &Vec3) -> Option<&FxHashSet<Entity>> {
        let chunk_pos = get_chunk_pos(pos);
        if !self.entities.contains_key(&chunk_pos) {
            return None;
        }

        let set = self.entities.get(&chunk_pos).unwrap();
        Some(set)
    }

    /// square radius of chunks around pos
    pub fn get_radius(&self, pos: &Vec3, radius: i32) -> FxHashSet<Entity> {
        let mut set = FxHashSet::default();
        for x in -radius..=radius {
            for z in -radius..=radius {
                let s = self
                    .get(&(pos + Vec3::new(x as f32 * CHUNK_SIZE_X, 0.0, z as f32 * CHUNK_SIZE_Z)));
                if s.is_some() {
                    set.extend(s.unwrap());
                }
            }
        }
        set
    }
}

pub fn get_chunk_pos(pos: &Vec3) -> IVec2 {
    IVec2::new(
        (pos.x / CHUNK_SIZE_X).floor() as i32,
        (pos.z / CHUNK_SIZE_Z).floor() as i32,
    )
}
