use super::PusherHeadBlock;

use crate::game::blocks::BlockModel;
use crate::game::blocks::traits::BlockRender;

/// 无独立模型：头的外观由本体 Pusher 动画部件绘制，此格只负责碰撞占位
impl BlockRender for PusherHeadBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(&[])
    }
}
