/// 活塞/拦截器伸出状态，按方块运行时 ID 索引（随实体移动，不跟格子走）
#[derive(Default, Clone)]
pub struct PusherState {
    entries: HashMap<BlockId, PusherStateEntry>,
}

#[derive(Clone, Copy)]
struct PusherStateEntry {
    extended: bool,
    /// 开局快照时头前是否已有工厂方块；运行时掉到面前的不粘
    bound_front: bool,
}

impl PusherState {
    pub fn rebuild_from_world(world: &WorldBlocks) -> Self {
        let entries = world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                matches!(
                    block.kind.movement_rule(block.facing),
                    Some(MovementRule::PoweredTranslate { .. })
                )
                .then_some({
                    let head = *pos + block.facing.forward_ivec3();
                    (
                        block.id,
                        PusherStateEntry {
                            extended: world
                                .blocks
                                .get(&head)
                                .is_some_and(|b| b.kind == BlockKind::PusherHead),
                            bound_front: world.is_factory_at(head),
                        },
                    )
                })
            })
            .collect();
        Self { entries }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn sustained_animations(&self, world: &WorldBlocks) -> HashMap<IVec3, PusherMotion> {
        world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                self.entries
                    .get(&block.id)
                    .filter(|entry| entry.extended)
                    .map(|_| {
                        (
                            *pos,
                            PusherMotion {
                                from_extension: 1.0,
                                to_extension: 1.0,
                            },
                        )
                    })
            })
            .collect()
    }

    pub(super) fn actuating_devices(
        &self,
        world: &WorldBlocks,
        powered_devices: &HashSet<IVec3>,
    ) -> HashSet<IVec3> {
        world
            .blocks
            .iter()
            .filter_map(|(pos, block)| {
                let Some(MovementRule::PoweredTranslate {
                    extend_when_powered,
                    ..
                }) = block.kind.movement_rule(block.facing)
                else {
                    return None;
                };
                let powered = powered_devices.contains(pos);
                let desired_extended = if extend_when_powered {
                    powered
                } else {
                    !powered
                };
                let current_extended = self
                    .entries
                    .get(&block.id)
                    .map(|entry| entry.extended)
                    .unwrap_or(false);
                (desired_extended != current_extended).then_some(*pos)
            })
            .collect()
    }

    /// 已伸出头占格：世界里的真实 PusherHead 方块
    pub(super) fn hard_head_occupancy(world: &WorldBlocks) -> HashSet<IVec3> {
        world
            .blocks
            .iter()
            .filter(|(_, block)| block.kind == BlockKind::PusherHead)
            .map(|(pos, _)| *pos)
            .collect()
    }

    /// 该格若为已伸出推杆头，返回其本体坐标
    pub(super) fn body_at_extended_head(world: &WorldBlocks, head: IVec3) -> Option<IVec3> {
        let head_block = world.blocks.get(&head)?;
        if head_block.kind != BlockKind::PusherHead {
            return None;
        }
        let body_pos = head - head_block.facing.forward_ivec3();
        let body = world.blocks.get(&body_pos)?;
        matches!(
            body.kind.movement_rule(body.facing),
            Some(MovementRule::PoweredTranslate { .. })
        )
        .then_some(body_pos)
    }

    /// 推动/收回执行成功后提交伸出状态，并同步真实头方块
    pub(super) fn set_extended(&mut self, world: &mut WorldBlocks, id: BlockId, extended: bool) {
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        if entry.extended == extended {
            return;
        }
        entry.extended = extended;
        let Some((pos, facing)) = world
            .blocks
            .iter()
            .find(|(_, block)| block.id == id)
            .map(|(pos, block)| (*pos, block.facing))
        else {
            return;
        };
        let head = pos + facing.forward_ivec3();
        if extended {
            if !world.blocks.contains_key(&head) {
                let _ = world.insert(head, BlockData::new(BlockKind::PusherHead, facing));
            }
        } else if world
            .blocks
            .get(&head)
            .is_some_and(|block| block.kind == BlockKind::PusherHead)
        {
            let _ = world.remove(&head);
        }
    }
}
