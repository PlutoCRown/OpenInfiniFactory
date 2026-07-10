use super::SplitterBlock;

use crate::game::blocks::traits::BlockBehavior;

impl BlockBehavior for SplitterBlock {
    fn is_directional(&self) -> bool {
        true
    }
}
