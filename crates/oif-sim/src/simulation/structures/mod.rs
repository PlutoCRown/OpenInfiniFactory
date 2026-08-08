use glam::IVec3;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{BlockData, BlockId, BlockKind, MovementRule};
use crate::world::direction::Facing;
use crate::world::grid::{MaterialFace, WorldBlocks};

use super::motion::{BlockMotion, BlockMotionKind, PusherMotion};
use super::structure_state::{StructureId, StructureState};
use super::suction::SuctionLinks;

pub(crate) use super::structure_state::material_structure;

include!("types.rs");
include!("gravity.rs");
include!("plan.rs");
include!("execute.rs");
include!("collision.rs");
