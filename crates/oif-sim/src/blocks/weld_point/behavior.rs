use super::WeldPointBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{WeldBehavior};

impl BlockBehavior for WeldPointBlock {
    fn weld_behavior(&self) -> Option<WeldBehavior> {
        Some(WeldBehavior::Node)
    }
}
