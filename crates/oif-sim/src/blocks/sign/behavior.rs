use super::SignBlock;

use crate::blocks::traits::BlockBehavior;

impl BlockBehavior for SignBlock {
    fn is_directional(&self) -> bool {
        true
    }
}
