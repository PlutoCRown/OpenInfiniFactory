use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::blocks::BlockData;

use super::grid::WorldBlocks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FactoryBlockId(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactoryWorldLayer {
    Turn,
    Solution,
}

impl FactoryBlockId {
    pub fn from_u32(value: u32) -> Self {
        Self(value)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Default, Resource)]
pub struct FactoryBlockRegistry {
    next_id: u32,
    solution: HashMap<FactoryBlockId, IVec3>,
    turn: HashMap<FactoryBlockId, IVec3>,
    turn_by_pos: HashMap<IVec3, FactoryBlockId>,
}

fn manhattan_distance(a: IVec3, b: IVec3) -> i32 {
    (a - b).abs().element_sum()
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

    pub fn id_at(&self, pos: IVec3, layer: FactoryWorldLayer) -> Option<FactoryBlockId> {
        match layer {
            FactoryWorldLayer::Turn => self.turn_id_at(pos),
            FactoryWorldLayer::Solution => self.solution_id_at(pos),
        }
    }

    pub fn pos_at(&self, id: FactoryBlockId, layer: FactoryWorldLayer) -> Option<IVec3> {
        match layer {
            FactoryWorldLayer::Turn => self.turn_pos(id),
            FactoryWorldLayer::Solution => self.solution_pos(id),
        }
    }

    pub fn turn_id_at(&self, pos: IVec3) -> Option<FactoryBlockId> {
        self.turn_by_pos.get(&pos).copied()
    }

    pub fn solution_id_at(&self, pos: IVec3) -> Option<FactoryBlockId> {
        self.solution
            .iter()
            .find_map(|(id, solution_pos)| (*solution_pos == pos).then_some(*id))
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

    pub fn reconcile_turn_positions(&mut self, solution: &WorldBlocks, turn: &WorldBlocks) {
        if self.solution.is_empty() {
            return;
        }

        let mut candidates: Vec<(IVec3, BlockData)> = turn
            .blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.is_factory().then_some((*pos, *block)))
            .collect();
        candidates.sort_by_key(|(pos, _)| (pos.x, pos.y, pos.z));

        let mut ids: Vec<_> = self.solution.keys().copied().collect();
        ids.sort();

        let mut next_turn = HashMap::new();
        let mut next_by_pos = HashMap::new();
        for id in ids {
            let Some(solution_pos) = self.solution.get(&id).copied() else {
                continue;
            };
            let Some(expected) = solution.blocks.get(&solution_pos) else {
                continue;
            };
            let hint = self.turn.get(&id).copied().unwrap_or(solution_pos);
            let (index, (pos, _)) = candidates
                .iter()
                .enumerate()
                .filter(|(_, (_, block))| {
                    block.kind == expected.kind && block.facing == expected.facing
                })
                .min_by_key(|(_, (pos, _))| manhattan_distance(*pos, hint))
                .unwrap_or_else(|| {
                    candidates
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, block))| block.kind == expected.kind)
                        .min_by_key(|(_, (pos, _))| manhattan_distance(*pos, hint))
                        .expect("missing turn factory block for reconciled id")
                });
            let (pos, _) = candidates.remove(index);
            next_turn.insert(id, pos);
            next_by_pos.insert(pos, id);
        }

        self.turn = next_turn;
        self.turn_by_pos = next_by_pos;
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
