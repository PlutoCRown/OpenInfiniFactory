use super::*;
use crate::game::world::blocks::*;

pub(super) fn weld_behavior(_block: &WeldPointBlock) -> Option<WeldBehavior> {
    Some(WeldBehavior::Node)
}
