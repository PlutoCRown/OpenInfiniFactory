use super::StamperBodyBlock;

use crate::blocks::traits::BlockBehavior;

impl BlockBehavior for StamperBodyBlock {
    fn is_directional(&self) -> bool {
        true
    }
}
