pub(super) enum StructureMove {
    Translate {
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        offset: IVec3,
        /// 同结构同位移可合并多个推杆，同回合同步伸出/收回
        actors: Vec<PusherActor>,
        mark: MovementMark,
        source: Option<BlockId>,
        source_pos: Option<IVec3>,
    },
    Rotate {
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        pivot: IVec3,
        clockwise: bool,
        source: Option<BlockId>,
        source_pos: Option<IVec3>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum MovementMark {
    Conveyor,
    Push,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PusherActor {
    pub(super) id: BlockId,
    pub(super) pos: IVec3,
    pub(super) animation: PusherAnimationKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PusherAnimationKind {
    Extend,
    Retract,
}

/// 模拟裁决历史：累计已执行的推动来源，清空会改变后续运动优先级
#[derive(bevy_ecs::prelude::Resource, Default, Clone)]
pub struct MovementHistory {
    /// 按结构 ID + 推动源方块 ID 累计；结构 ID 稳定时跨回合保留，优先未作用过的源
    counts: HashMap<StructureId, HashMap<BlockId, u32>>,
}

impl MovementHistory {
    pub fn clear(&mut self) {
        self.counts.clear();
    }

    fn count(&self, movement: &StructureMove) -> u32 {
        let Some(source) = movement.source() else {
            return 0;
        };
        self.counts
            .get(&movement.structure_id())
            .and_then(|sources| sources.get(&source).copied())
            .unwrap_or(0)
    }

    /// 只丢掉已不存在的结构/源，不因本回合未作用而清零
    fn prune_missing(
        &mut self,
        living_structures: &HashSet<StructureId>,
        living_blocks: &HashSet<BlockId>,
    ) {
        self.counts.retain(|structure, sources| {
            if !living_structures.contains(structure) {
                return false;
            }
            sources.retain(|source, _| living_blocks.contains(source));
            !sources.is_empty()
        });
    }

    fn record_executed(&mut self, executed: Vec<ExecutedMovement>) {
        for movement in executed {
            *self
                .counts
                .entry(movement.structure_id)
                .or_default()
                .entry(movement.source)
                .or_insert(0) += 1;
        }
    }
}

struct ExecutedMovement {
    structure_id: StructureId,
    source: BlockId,
}

impl StructureMove {
    pub(super) fn translate_marked(
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        offset: IVec3,
        mark: MovementMark,
    ) -> Self {
        Self::Translate {
            structure_id,
            structure,
            offset,
            actors: Vec::new(),
            mark,
            source: None,
            source_pos: None,
        }
    }

    pub(super) fn translate_by_pusher_actor(
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        offset: IVec3,
        actor: PusherActor,
        mark: MovementMark,
    ) -> Self {
        Self::Translate {
            structure_id,
            structure,
            offset,
            actors: vec![actor],
            mark,
            source: None,
            source_pos: None,
        }
    }

    pub(super) fn rotate(
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        pivot: IVec3,
        clockwise: bool,
    ) -> Self {
        Self::Rotate {
            structure_id,
            structure,
            pivot,
            clockwise,
            source: None,
            source_pos: None,
        }
    }

    pub(super) fn with_source(mut self, source: BlockId, source_pos: IVec3) -> Self {
        match &mut self {
            Self::Translate {
                source: slot,
                source_pos: pos_slot,
                ..
            }
            | Self::Rotate {
                source: slot,
                source_pos: pos_slot,
                ..
            } => {
                *slot = Some(source);
                *pos_slot = Some(source_pos);
            }
        }
        self
    }

    fn source(&self) -> Option<BlockId> {
        match self {
            Self::Translate { source, .. } | Self::Rotate { source, .. } => *source,
        }
    }

    fn source_pos(&self) -> Option<IVec3> {
        match self {
            Self::Translate { source_pos, .. } | Self::Rotate { source_pos, .. } => *source_pos,
        }
    }

    pub(super) fn structure_id(&self) -> StructureId {
        match self {
            Self::Translate { structure_id, .. } | Self::Rotate { structure_id, .. } => {
                *structure_id
            }
        }
    }

    pub(super) fn structure(&self) -> &HashSet<IVec3> {
        match self {
            Self::Translate { structure, .. } | Self::Rotate { structure, .. } => structure,
        }
    }
}
