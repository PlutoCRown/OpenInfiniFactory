use super::WeldPointBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{WeldBehavior};

impl BlockBehavior for WeldPointBlock {
    fn weld_behavior(&self) -> Option<WeldBehavior> {
        Some(WeldBehavior::Node)
    }
}
