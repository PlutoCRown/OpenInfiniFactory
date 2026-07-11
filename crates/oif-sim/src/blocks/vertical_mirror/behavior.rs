use super::VerticalMirrorBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::LaserOpticsBehavior;

impl BlockBehavior for VerticalMirrorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn laser_optics(&self) -> Option<LaserOpticsBehavior> {
        Some(LaserOpticsBehavior::VerticalMirror)
    }
}
