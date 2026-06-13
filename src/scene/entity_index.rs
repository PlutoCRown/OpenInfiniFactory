use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::world::rendering::BlockEntity;

#[derive(Default)]
pub struct BlockEntityIndex {
    by_pos: HashMap<IVec3, Entity>,
}

impl BlockEntityIndex {
    pub fn get(&self, pos: IVec3) -> Option<Entity> {
        self.by_pos.get(&pos).copied()
    }

    pub fn insert(&mut self, pos: IVec3, entity: Entity) {
        self.by_pos.insert(pos, entity);
    }

    pub fn remove(&mut self, pos: IVec3) -> Option<Entity> {
        self.by_pos.remove(&pos)
    }

    pub fn clear(&mut self) {
        self.by_pos.clear();
    }

    pub fn rebuild_from_world(&mut self, blocks: &Query<(Entity, &BlockEntity)>) {
        self.by_pos.clear();
        for (entity, block) in blocks {
            self.by_pos.insert(block.pos, entity);
        }
    }
}
