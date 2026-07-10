use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::blocks::BlockId;
use crate::game::world::rendering::{BlockEntity, BlockEntityLayer};

/// 场景实体索引：工厂/材料与系统层分槽，避免重叠格抓错模型
#[derive(Default)]
pub struct BlockEntityIndex {
    by_id: HashMap<BlockId, Entity>,
    animatable_pos: HashMap<IVec3, Entity>,
    system_pos: HashMap<IVec3, Entity>,
    scene_pos: HashMap<IVec3, Entity>,
}

impl BlockEntityIndex {
    pub fn get_by_id(&self, id: BlockId) -> Option<Entity> {
        if id.is_none() {
            return None;
        }
        self.by_id.get(&id).copied()
    }

    pub fn get_animatable(&self, pos: IVec3) -> Option<Entity> {
        self.animatable_pos.get(&pos).copied()
    }

    pub fn get_system(&self, pos: IVec3) -> Option<Entity> {
        self.system_pos.get(&pos).copied()
    }

    pub fn get_scene(&self, pos: IVec3) -> Option<Entity> {
        self.scene_pos.get(&pos).copied()
    }

    pub fn insert(&mut self, pos: IVec3, id: BlockId, layer: BlockEntityLayer, entity: Entity) {
        match layer {
            BlockEntityLayer::Animatable => {
                self.animatable_pos.insert(pos, entity);
                if !id.is_none() {
                    self.by_id.insert(id, entity);
                }
            }
            BlockEntityLayer::System => {
                self.system_pos.insert(pos, entity);
            }
            BlockEntityLayer::Scene => {
                self.scene_pos.insert(pos, entity);
            }
        }
    }

    // 只从位置槽解绑，保留 by_id（供同回合其它动画仍能按 ID 找到实体）
    pub fn unbind_animatable_pos(&mut self, pos: IVec3) -> Option<Entity> {
        self.animatable_pos.remove(&pos)
    }

    pub fn remove_animatable(&mut self, pos: IVec3) -> Option<Entity> {
        let entity = self.animatable_pos.remove(&pos)?;
        self.by_id.retain(|_, existing| *existing != entity);
        Some(entity)
    }

    pub fn remove_system(&mut self, pos: IVec3) -> Option<Entity> {
        self.system_pos.remove(&pos)
    }

    pub fn remove_scene(&mut self, pos: IVec3) -> Option<Entity> {
        self.scene_pos.remove(&pos)
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.animatable_pos
            .retain(|_, existing| *existing != entity);
        self.system_pos.retain(|_, existing| *existing != entity);
        self.scene_pos.retain(|_, existing| *existing != entity);
        self.by_id.retain(|_, existing| *existing != entity);
    }

    pub fn clear(&mut self) {
        self.by_id.clear();
        self.animatable_pos.clear();
        self.system_pos.clear();
        self.scene_pos.clear();
    }

    pub fn rebuild_from_world(&mut self, blocks: &Query<(Entity, &BlockEntity)>) {
        self.clear();
        for (entity, block) in blocks {
            self.insert(block.pos, block.id, block.layer, entity);
        }
    }
}
