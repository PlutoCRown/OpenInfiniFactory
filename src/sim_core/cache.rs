use std::collections::BTreeMap;

use crate::sim_core::TurnOutput;

pub const TURN_PREFETCH_DEPTH: u64 = 4;

#[derive(Default)]
pub struct TurnCache {
    pub simulated_through: u64,
    pending: BTreeMap<u64, TurnOutput>,
}

impl TurnCache {
    pub fn clear(&mut self) {
        self.simulated_through = 0;
        self.pending.clear();
    }

    pub fn reset_to_turn(&mut self, turn: u64) {
        self.simulated_through = turn;
        self.pending.clear();
    }

    pub fn has_prefetched(&self, display_turn: u64) -> bool {
        self.pending.contains_key(&(display_turn + 1))
    }

    pub fn has_pending_turn(&self, turn: u64) -> bool {
        self.pending.contains_key(&turn)
    }

    pub fn needs_prefetch(&self, display_turn: u64) -> bool {
        self.simulated_through < display_turn + game_prefetch_depth()
    }

    pub fn take_pending(&mut self, expected_turn: u64) -> Option<TurnOutput> {
        self.pending.remove(&expected_turn)
    }

    pub fn store_prefetch(&mut self, output: TurnOutput) {
        self.simulated_through = output.turn;
        self.pending.insert(output.turn, output);
    }
}

fn game_prefetch_depth() -> u64 {
    1
}
