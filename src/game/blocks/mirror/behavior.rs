use super::MirrorBlock;

use crate::game::blocks::traits::BlockBehavior;

impl BlockBehavior for MirrorBlock {
    fn is_directional(&self) -> bool {
        true
    }
}
