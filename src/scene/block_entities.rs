use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::world::block_instance::BlockInstanceId;
use crate::game::world::rendering::BlockEntity;

struct TrackedBlockEntity {
    entity: Entity,
    id: BlockInstanceId,
    pos: IVec3,
}

/// Tracks block render entities within one incremental refresh pass.
///
/// Bevy applies `Commands` at the end of the schedule, so queries stay stale until
/// the next frame. This tracker mirrors spawn/despawn intent for the current pass.
pub struct BlockEntityTracker {
    entries: Vec<TrackedBlockEntity>,
    present_ids: HashSet<BlockInstanceId>,
}

impl BlockEntityTracker {
    pub fn capture(blocks: &Query<(Entity, &BlockEntity)>) -> Self {
        let mut present_ids = HashSet::new();
        let entries = blocks
            .iter()
            .map(|(entity, block)| {
                present_ids.insert(block.id);
                TrackedBlockEntity {
                    entity,
                    id: block.id,
                    pos: block.pos,
                }
            })
            .collect();
        Self {
            entries,
            present_ids,
        }
    }

    pub fn has_id(&self, id: BlockInstanceId) -> bool {
        self.present_ids.contains(&id)
    }

    pub fn mark_spawned(&mut self, id: BlockInstanceId) {
        self.present_ids.insert(id);
    }

    pub fn despawn_all(&mut self, commands: &mut Commands) {
        let victims: Vec<Entity> = self.entries.iter().map(|entry| entry.entity).collect();
        for entity in victims {
            self.remove_entity(commands, entity);
        }
        self.present_ids.clear();
    }

    pub fn despawn_id(&mut self, commands: &mut Commands, id: BlockInstanceId) {
        let victims: Vec<Entity> = self
            .entries
            .iter()
            .filter(|entry| entry.id == id)
            .map(|entry| entry.entity)
            .collect();
        for entity in victims {
            self.remove_entity(commands, entity);
        }
        self.present_ids.remove(&id);
    }

    pub fn despawn_at_pos(&mut self, commands: &mut Commands, pos: IVec3) {
        let victims: Vec<Entity> = self
            .entries
            .iter()
            .filter(|entry| entry.pos == pos)
            .map(|entry| entry.entity)
            .collect();
        for entity in victims {
            self.remove_entity(commands, entity);
        }
        self.rebuild_present_ids();
    }

    pub fn dedupe_duplicate_ids(&mut self, commands: &mut Commands) {
        let mut seen = HashSet::new();
        let victims: Vec<Entity> = self
            .entries
            .iter()
            .filter(|entry| !seen.insert(entry.id))
            .map(|entry| entry.entity)
            .collect();
        for entity in victims {
            self.remove_entity(commands, entity);
        }
        self.rebuild_present_ids();
    }

    fn remove_entity(&mut self, commands: &mut Commands, entity: Entity) {
        let Some(index) = self.entries.iter().position(|entry| entry.entity == entity) else {
            return;
        };
        let entry = self.entries.swap_remove(index);
        commands.entity(entry.entity).despawn();
    }

    fn rebuild_present_ids(&mut self) {
        self.present_ids = self.entries.iter().map(|entry| entry.id).collect();
    }
}
