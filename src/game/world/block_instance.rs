use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::factory_registry::FactoryBlockId;
use super::grid::WorldBlocks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MaterialBlockId(u32);

impl MaterialBlockId {
    pub fn from_u32(value: u32) -> Self {
        Self(value)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BlockInstanceId {
    Factory(FactoryBlockId),
    Material(MaterialBlockId),
}

impl BlockInstanceId {
    pub fn resolve(
        world: &WorldBlocks,
        factory_registry: &super::factory_registry::FactoryBlockRegistry,
        material_registry: &MaterialBlockRegistry,
        pos: IVec3,
    ) -> Option<Self> {
        if world.blocks.get(&pos).is_some_and(|block| block.kind.is_factory()) {
            return factory_registry
                .turn_id_at(pos)
                .map(Self::Factory);
        }
        if world.is_material_at(pos) {
            return material_registry.turn_id_at(pos).map(Self::Material);
        }
        None
    }
}

#[derive(Clone, Default, Resource)]
pub struct MaterialBlockRegistry {
    next_id: u32,
    turn: HashMap<MaterialBlockId, IVec3>,
    turn_by_pos: HashMap<IVec3, MaterialBlockId>,
}

impl MaterialBlockRegistry {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn rebuild_from_world(world: &WorldBlocks) -> Self {
        let mut registry = Self::default();
        let mut positions: Vec<_> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
            .collect();
        positions.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        for pos in positions {
            registry.assign_turn(pos);
        }
        registry
    }

    pub fn turn_id_at(&self, pos: IVec3) -> Option<MaterialBlockId> {
        self.turn_by_pos.get(&pos).copied()
    }

    pub fn turn_pos(&self, id: MaterialBlockId) -> Option<IVec3> {
        self.turn.get(&id).copied()
    }

    pub fn assign_turn(&mut self, pos: IVec3) -> MaterialBlockId {
        let id = MaterialBlockId(self.next_id);
        self.next_id += 1;
        self.turn.insert(id, pos);
        self.turn_by_pos.insert(pos, id);
        id
    }

    pub fn on_material_inserted(&mut self, pos: IVec3) {
        if !self.turn_by_pos.contains_key(&pos) {
            self.assign_turn(pos);
        }
    }

    pub fn on_material_removed(&mut self, pos: IVec3) {
        let Some(id) = self.turn_by_pos.remove(&pos) else {
            return;
        };
        self.turn.remove(&id);
    }

    pub fn translate_turn(&mut self, positions: &HashSet<IVec3>, offset: IVec3) {
        for pos in positions {
            let Some(id) = self.turn_by_pos.remove(pos) else {
                continue;
            };
            let new_pos = *pos + offset;
            self.turn.insert(id, new_pos);
            self.turn_by_pos.insert(new_pos, id);
        }
    }

    pub fn rotate_turn(&mut self, positions: &HashSet<IVec3>, pivot: IVec3, clockwise: bool) {
        for pos in positions {
            let Some(id) = self.turn_by_pos.remove(pos) else {
                continue;
            };
            let new_pos = rotate_pos_y(*pos, pivot, clockwise);
            self.turn.insert(id, new_pos);
            self.turn_by_pos.insert(new_pos, id);
        }
    }
}

fn rotate_pos_y(pos: IVec3, pivot: IVec3, clockwise: bool) -> IVec3 {
    let rel = pos - pivot;
    pivot
        + if clockwise {
            IVec3::new(-rel.z, rel.y, rel.x)
        } else {
            IVec3::new(rel.z, rel.y, -rel.x)
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind};
    use crate::game::world::direction::Facing;
    use crate::game::world::factory_registry::FactoryBlockRegistry;

    #[test]
    fn block_instance_id_follows_factory_registry_after_translate() {
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::ZERO,
            BlockData {
                kind: BlockKind::Platform,
                facing: Facing::North,
            },
        );
        let mut factory = FactoryBlockRegistry::rebuild_from_world(&world);
        let id = factory.turn_id_at(IVec3::ZERO).unwrap();
        factory.translate_turn(&HashSet::from([IVec3::ZERO]), IVec3::X);
        world.blocks.clear();
        world.insert(
            IVec3::X,
            BlockData {
                kind: BlockKind::Platform,
                facing: Facing::North,
            },
        );
        assert_eq!(
            BlockInstanceId::resolve(&world, &factory, &MaterialBlockRegistry::default(), IVec3::X),
            Some(BlockInstanceId::Factory(id))
        );
    }
}
