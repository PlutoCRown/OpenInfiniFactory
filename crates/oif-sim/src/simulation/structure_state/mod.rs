//! 世界结构表：工厂/材料连通、可变形子集、回合 held

use glam::IVec3;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{AcceptorId, BlockId, BlockKind, MovementRule};
use crate::world::grid::WorldBlocks;

use super::signal_offsets;

include!("types.rs");
include!("rebuild.rs");
include!("deform.rs");
include!("query.rs");
include!("connectivity.rs");

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
