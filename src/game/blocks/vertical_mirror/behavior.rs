use super::VerticalMirrorBlock;

use crate::game::blocks::traits::BlockBehavior;

impl BlockBehavior for VerticalMirrorBlock {
    fn is_directional(&self) -> bool {
        true
    }
}
