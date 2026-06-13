use bevy::prelude::*;
use std::collections::HashMap;

use super::grid::WorldBlocks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FactoryBlockId(u32);

#[derive(Clone, Default, Resource)]
pub struct FactoryBlockRegistry {
    next_id: u32,
    solution: HashMap<FactoryBlockId, IVec3>,
    turn: HashMap<FactoryBlockId, IVec3>,
    turn_by_pos: HashMap<IVec3, FactoryBlockId>,
}

impl FactoryBlockRegistry {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn rebuild_from_world(world: &WorldBlocks) -> Self {
        let mut registry = Self::default();
        let mut positions: Vec<_> = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_factory().then_some(*pos))
            .collect();
        positions.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        for pos in positions {
            registry.assign_turn(pos);
        }
        registry
    }

    pub fn freeze_solution(&mut self) {
        self.solution = self.turn.clone();
    }

    pub fn is_solution_frozen(&self) -> bool {
        !self.solution.is_empty()
    }

    pub fn turn_to_solution_pos(&self, turn_pos: IVec3) -> Option<IVec3> {
        let id = self.turn_by_pos.get(&turn_pos)?;
        self.solution.get(id).copied()
    }

    pub fn solution_pos(&self, id: FactoryBlockId) -> Option<IVec3> {
        self.solution.get(&id).copied()
    }

    pub fn turn_pos(&self, id: FactoryBlockId) -> Option<IVec3> {
        self.turn.get(&id).copied()
    }

    pub fn has_turn_factory(&self, pos: IVec3) -> bool {
        self.turn_by_pos.contains_key(&pos)
    }

    pub fn assign_turn(&mut self, pos: IVec3) -> FactoryBlockId {
        let id = FactoryBlockId(self.next_id);
        self.next_id += 1;
        self.turn.insert(id, pos);
        self.turn_by_pos.insert(pos, id);
        id
    }

    pub fn on_factory_inserted(&mut self, pos: IVec3) {
        if !self.turn_by_pos.contains_key(&pos) {
            self.assign_turn(pos);
        }
    }

    pub fn on_factory_removed(&mut self, pos: IVec3) {
        let Some(id) = self.turn_by_pos.remove(&pos) else {
            return;
        };
        self.turn.remove(&id);
    }

    pub fn translate_turn(&mut self, positions: &std::collections::HashSet<IVec3>, offset: IVec3) {
        for pos in positions {
            let Some(id) = self.turn_by_pos.remove(pos) else {
                continue;
            };
            let new_pos = *pos + offset;
            self.turn.insert(id, new_pos);
            self.turn_by_pos.insert(new_pos, id);
        }
    }

    pub fn rotate_turn(
        &mut self,
        positions: &std::collections::HashSet<IVec3>,
        pivot: IVec3,
        clockwise: bool,
    ) {
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
