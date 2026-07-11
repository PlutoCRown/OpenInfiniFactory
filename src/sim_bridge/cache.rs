use std::collections::BTreeMap;

use bevy::prelude::Resource;

use super::snapshot::CachedTurn;

/// 预取深度上限（worker 侧参考）
pub const TURN_PREFETCH_DEPTH: u64 = 4;

/// 已预计算回合缓存，供表现层按显示回合取出
#[derive(Resource, Default)]
pub struct TurnCache {
    pub simulated_through: u64,
    pending: BTreeMap<u64, CachedTurn>,
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

    pub fn take_pending(&mut self, expected_turn: u64) -> Option<CachedTurn> {
        self.pending.remove(&expected_turn)
    }

    pub fn store_prefetch(&mut self, cached: CachedTurn) {
        self.simulated_through = cached.output.turn;
        self.pending.insert(cached.output.turn, cached);
    }

    pub fn ingest_worker_results(&mut self, results: impl IntoIterator<Item = CachedTurn>) {
        for cached in results {
            if self.pending.contains_key(&cached.output.turn) {
                continue;
            }
            self.simulated_through = self.simulated_through.max(cached.output.turn);
            self.pending.insert(cached.output.turn, cached);
        }
    }
}

fn game_prefetch_depth() -> u64 {
    2
}
