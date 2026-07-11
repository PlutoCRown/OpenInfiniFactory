use super::MirrorBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::LaserOpticsBehavior;

impl BlockBehavior for MirrorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn laser_optics(&self) -> Option<LaserOpticsBehavior> {
        Some(LaserOpticsBehavior::Mirror)
    }
}
