use super::SplitterBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::LaserOpticsBehavior;

impl BlockBehavior for SplitterBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn laser_optics(&self) -> Option<LaserOpticsBehavior> {
        Some(LaserOpticsBehavior::Splitter)
    }
}
