use glam::IVec3;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{BlockData, BlockId, MovementRule};
use crate::world::direction::Facing;
use crate::world::grid::{MaterialFace, WorldBlocks};

use super::motion::{BlockMotion, BlockMotionKind, PusherMotion};
use super::structure_state::{StructureId, StructureState};
use super::suction::SuctionLinks;

pub(crate) use super::structure_state::material_structure;

pub(super) fn gravity_moves(
    world: &WorldBlocks,
    structures: &mut StructureState,
    skip_factory_positions: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    let ids = structures.gravity_structure_ids();
    let mut moves = Vec::new();
    let mut handled = HashSet::new();

    for id in ids {
        let Some(positions) = structures.structure_positions(id) else {
            continue;
        };
        let Some(&sample) = positions.iter().next() else {
            continue;
        };
        if handled.contains(&sample) {
            continue;
        }

        // 经吸盘扩展；粘到固定结构则整体不可落（种子为完整结构时并集等价）
        let Some(structure) =
            structures.linked_expand_pusher_subset(suction, positions, IVec3::NEG_Y)
        else {
            handled.extend(positions.iter().copied());
            continue;
        };
        handled.extend(structure.iter().copied());

        if structure
            .iter()
            .any(|pos| skip_factory_positions.contains(pos))
        {
            continue;
        }
        if structure
            .iter()
            .any(|pos| world.anchors_material_at_teleport_entrance(*pos))
        {
            continue;
        }
        if structure_supported_by_lifter(world, &structure) {
            for pos in &structure {
                if let Some(sid) = structures.id_at(*pos) {
                    structures.clear_gravity_support(sid);
                }
            }
            continue;
        }
        if structures.gravity_support_valid(id, world, hard_pusher_head_occupancy) {
            continue;
        }
        // 整块结构一起下落；不可拆开，否则焊接材料会被撕开并丢掉焊缝
        if can_move_gravity_structure(
            world,
            &structure,
            structures,
            hard_pusher_head_occupancy,
            suction,
        ) {
            for pos in &structure {
                if let Some(sid) = structures.id_at(*pos) {
                    structures.clear_gravity_support(sid);
                }
            }
            moves.push(StructureMove::translate_marked(
                id,
                structure,
                IVec3::NEG_Y,
                MovementMark::Vertical,
            ));
        } else {
            structures.record_gravity_support(id, world, hard_pusher_head_occupancy);
        }
    }
    moves
}

pub(super) enum StructureMove {
    Translate {
        structure_id: StructureId,
        structure: HashSet<IVec3>,
        offset: IVec3,
        actor: Option<PusherActor>,
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
    pub(super) pos: IVec3,
    pub(super) animation: PusherAnimationKind,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PusherAnimationKind {
    Extend,
    Retract,
}

#[derive(Default, Clone)]
pub struct MovementInfluenceCache {
    /// 按结构 ID + 推动源方块 ID 累计；结构 ID 稳定时跨回合保留，优先未作用过的源
    counts: HashMap<StructureId, HashMap<BlockId, u32>>,
}

impl MovementInfluenceCache {
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
            actor: None,
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
            actor: Some(actor),
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

/// 合并重力与设备运动标签：保留全部重叠标签，按优先级排序，执行时再 fallback
pub(super) fn merge_structure_movement_plan(
    mut planned_moves: Vec<StructureMove>,
    device_moves: Vec<StructureMove>,
    influence_cache: &mut MovementInfluenceCache,
    structures: &StructureState,
    world: &WorldBlocks,
) -> Vec<StructureMove> {
    let living_structures: HashSet<StructureId> = structures.structure_ids().collect();
    let living_blocks: HashSet<BlockId> = world.blocks.values().map(|block| block.id).collect();
    influence_cache.prune_missing(&living_structures, &living_blocks);
    planned_moves.extend(device_moves);
    planned_moves.sort_by(|a, b| compare_movement_priority(a, b, influence_cache));
    planned_moves
}

fn compare_movement_priority(
    a: &StructureMove,
    b: &StructureMove,
    influence_cache: &MovementInfluenceCache,
) -> Ordering {
    movement_priority_key(a, influence_cache).cmp(&movement_priority_key(b, influence_cache))
}

fn movement_priority_key(
    movement: &StructureMove,
    influence_cache: &MovementInfluenceCache,
) -> (u8, u32, ConveyorSourcePriority) {
    // 种类优先：活塞 > 抬升 > 下落 > 旋转 > 传送带
    (
        movement_kind_priority(movement),
        movement
            .source()
            .map_or(0, |_| influence_cache.count(movement)),
        conveyor_source_priority(movement),
    )
}

fn movement_kind_priority(movement: &StructureMove) -> u8 {
    match movement {
        StructureMove::Translate {
            mark: MovementMark::Push,
            ..
        } => 0,
        // 抬升器：有 source 的竖直移动
        StructureMove::Translate {
            mark: MovementMark::Vertical,
            source: Some(_),
            ..
        } => 1,
        StructureMove::Translate {
            mark: MovementMark::Vertical,
            source: None,
            ..
        } => 2,
        StructureMove::Rotate { .. } => 3,
        StructureMove::Translate {
            mark: MovementMark::Conveyor,
            ..
        } => 4,
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct ConveyorSourcePriority {
    positive_x: i32,
    negative_x: i32,
    positive_y: i32,
    negative_y: i32,
    positive_z: i32,
    negative_z: i32,
}

fn conveyor_source_priority(movement: &StructureMove) -> ConveyorSourcePriority {
    let Some(source) = movement.source_pos() else {
        return ConveyorSourcePriority::neutral();
    };
    if !matches!(
        movement,
        StructureMove::Translate {
            mark: MovementMark::Conveyor,
            ..
        }
    ) {
        return ConveyorSourcePriority::neutral();
    }
    ConveyorSourcePriority {
        positive_x: -source.x,
        negative_x: source.x,
        positive_y: -source.y,
        negative_y: source.y,
        positive_z: -source.z,
        negative_z: source.z,
    }
}

impl ConveyorSourcePriority {
    fn neutral() -> Self {
        Self {
            positive_x: 0,
            negative_x: 0,
            positive_y: 0,
            negative_y: 0,
            positive_z: 0,
            negative_z: 0,
        }
    }
}

fn structure_supported_by_lifter(world: &WorldBlocks, structure: &HashSet<IVec3>) -> bool {
    world.blocks.iter().any(|(pos, block)| {
        matches!(
            block.kind.movement_rule(block.facing),
            Some(MovementRule::Lift { range }) if structure.contains(&(*pos + IVec3::Y * (range + 1)))
        )
    })
}

/// 运动执行前：按计划压碎/让出冲突的脆弱材料（与钻头/激光销毁分离）
pub(super) fn apply_fragile_shatter_before_execute(
    world: &mut WorldBlocks,
    moves: &mut [StructureMove],
    structures: &mut StructureState,
) {
    let mut shatter = HashSet::new();
    for movement in moves.iter() {
        match movement {
            StructureMove::Translate {
                structure,
                offset,
                actor,
                ..
            } => {
                if let Some(actor) = actor {
                    if matches!(actor.animation, PusherAnimationKind::Extend) {
                        if let Some(block) = world.blocks.get(&actor.pos) {
                            let head = actor.pos + block.facing.forward_ivec3();
                            if world.is_fragile_material_at(head) {
                                shatter.insert(head);
                            }
                        }
                    }
                }
                if *offset == IVec3::ZERO {
                    continue;
                }
                for pos in structure {
                    let target = *pos + *offset;
                    if !structure.contains(&target) && world.is_fragile_material_at(target) {
                        shatter.insert(target);
                    }
                    if world.is_fragile_material_at(*pos)
                        && target.y >= 0
                        && !structure.contains(&target)
                        && !world.cell_accepts_move_from(*pos, target)
                        && !world.is_fragile_material_at(target)
                    {
                        shatter.insert(*pos);
                    }
                }
            }
            StructureMove::Rotate { .. } => {}
        }
    }
    if shatter.is_empty() {
        return;
    }

    let mut affected: HashMap<StructureId, HashSet<IVec3>> = HashMap::new();
    for pos in &shatter {
        if let Some(id) = structures.id_at(*pos) {
            affected.entry(id).or_default().insert(*pos);
        }
        world.remove(pos);
    }
    for (id, removed) in affected {
        let Some(old) = structures.structure_positions(id).cloned() else {
            continue;
        };
        let new_positions: HashSet<IVec3> = old.difference(&removed).copied().collect();
        structures.replace_structure_positions(&old, new_positions);
    }
    for movement in moves.iter_mut() {
        match movement {
            StructureMove::Translate { structure, .. } | StructureMove::Rotate { structure, .. } => {
                for pos in &shatter {
                    structure.remove(pos);
                }
            }
        }
    }
}

/// 按序执行运动标签：失败则试下一个；种子判占用，成功后标记展开后的格子。
/// `hard_pusher_head_occupancy` 为本回合开始时已伸出的头；执行中随 Push 伸出/收回更新。
pub(super) fn execute_structure_moves_with_pushers(
    world: &mut WorldBlocks,
    moves: Vec<StructureMove>,
    structures: &mut StructureState,
    influence_cache: &mut MovementInfluenceCache,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    suction: &SuctionLinks,
) -> (
    HashMap<IVec3, BlockMotion>,
    HashMap<IVec3, PusherMotion>,
) {
    let mut moved = HashSet::new();
    let mut animations = HashMap::new();
    let mut pusher_animations = HashMap::new();
    let mut executed = Vec::new();
    let mut heads = hard_pusher_head_occupancy.clone();
    for movement in moves {
        match movement {
            StructureMove::Translate {
                structure_id,
                structure,
                offset,
                actor,
                mark,
                source,
                source_pos: _,
            } => {
                // 仅用种子结构判占用；展开在当前世界上做，避免预展开导致误跳过
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                // 收回标签检查时忽略自己的头，否则粘头拉回会撞上尚未释放的头占位
                let mut heads_for_check = heads.clone();
                if let Some(actor) = &actor {
                    if matches!(actor.animation, PusherAnimationKind::Retract) {
                        if let Some(block) = world.blocks.get(&actor.pos) {
                            heads_for_check.remove(&(actor.pos + block.facing.forward_ivec3()));
                        }
                    }
                }
                let seed = structure.clone();
                let Some(structure) = expanded_move_structure(
                    world,
                    &structure,
                    offset,
                    structures,
                    movement_expansion_mode(mark, source),
                    suction,
                ) else {
                    continue;
                };
                // 展开卷入的格子若本回合已动过，本标签失败（可 fallback）
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                // 活塞头是实体：本回合已提交的头会挡住后续更低优先级移动
                if offset != IVec3::ZERO
                    && hard_pusher_head_blocks_move(&structure, offset, &heads_for_check)
                {
                    continue;
                }
                if offset == IVec3::NEG_Y
                    && hard_pusher_head_blocked_below(world, &seed, &heads_for_check)
                {
                    continue;
                }
                if offset != IVec3::ZERO {
                    for pos in &structure {
                        if let Some(block) = world.blocks.get(pos) {
                            animations.insert(
                                *pos + offset,
                                BlockMotion {
                                    block_id: block.id,
                                    from_pos: *pos,
                                    to_pos: *pos + offset,
                                    from_facing: block.facing,
                                    to_facing: block.facing,
                                    kind: BlockMotionKind::Move,
                                },
                            );
                        }
                    }
                    moved.extend(structure.iter().copied());
                    move_structure(world, &structure, offset);
                    structures.move_positions(&structure, offset);
                    let target_structure: HashSet<IVec3> =
                        structure.iter().map(|pos| *pos + offset).collect();
                    moved.extend(target_structure);
                } else {
                    // 空头伸出/收回：零位移 Push，只占位并抑制本结构重力 fallback
                    moved.extend(structure.iter().copied());
                }
                if let Some(actor) = actor {
                    let (from_extension, to_extension) = match actor.animation {
                        PusherAnimationKind::Extend => (0.0, 1.0),
                        PusherAnimationKind::Retract => (1.0, 0.0),
                    };
                    pusher_animations.insert(
                        actor.pos,
                        PusherMotion {
                            from_extension,
                            to_extension,
                        },
                    );
                    if let Some(block) = world.blocks.get(&actor.pos) {
                        let head = actor.pos + block.facing.forward_ivec3();
                        match actor.animation {
                            PusherAnimationKind::Extend => {
                                heads.insert(head);
                            }
                            PusherAnimationKind::Retract => {
                                heads.remove(&head);
                            }
                        }
                    }
                    // 粘头推动的是前方结构：活塞本体也算本回合已动作，抑制自身重力
                    if let Some(actor_id) = structures.id_at(actor.pos) {
                        if let Some(actor_structure) = structures.structure_positions(actor_id) {
                            moved.extend(actor_structure.iter().copied());
                        }
                    }
                }
                if let Some(source) = source {
                    executed.push(ExecutedMovement {
                        structure_id,
                        source,
                    });
                }
            }
            StructureMove::Rotate {
                structure_id,
                structure,
                pivot,
                clockwise,
                source,
                source_pos: _,
            } => {
                let structure = with_factory_attachment_children(world, &structure);
                if structure.iter().any(|pos| moved.contains(pos)) {
                    continue;
                }
                if can_rotate_structure(world, &structure, pivot, clockwise) {
                    let targets: Vec<IVec3> = structure
                        .iter()
                        .map(|pos| rotate_pos_y(*pos, pivot, clockwise))
                        .collect();
                    for pos in &structure {
                        if let Some(block) = world.blocks.get(pos) {
                            let target = rotate_pos_y(*pos, pivot, clockwise);
                            animations.insert(
                                target,
                                BlockMotion {
                                    block_id: block.id,
                                    from_pos: *pos,
                                    to_pos: target,
                                    from_facing: block.facing,
                                    to_facing: rotate_facing(block.facing, clockwise),
                                    kind: BlockMotionKind::Rotate { pivot, clockwise },
                                },
                            );
                        }
                    }
                    moved.extend(structure.iter().copied());
                    rotate_structure(world, &structure, pivot, clockwise);
                    let target_structure: HashSet<IVec3> = targets.iter().copied().collect();
                    structures.replace_structure_positions(&structure, target_structure.clone());
                    if let Some(source) = source {
                        executed.push(ExecutedMovement {
                            structure_id,
                            source,
                        });
                    }
                    moved.extend(target_structure);
                }
            }
        }
    }
    influence_cache.record_executed(executed);
    (animations, pusher_animations)
}

fn can_move_gravity_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    structures: &StructureState,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    suction: &SuctionLinks,
) -> bool {
    if hard_pusher_head_blocked_below(world, structure, hard_pusher_head_occupancy) {
        return false;
    }
    let Some(expanded) = expanded_move_structure(
        world,
        structure,
        IVec3::NEG_Y,
        structures,
        MovementExpansionMode::Gravity,
        suction,
    ) else {
        return false;
    };
    !hard_pusher_head_blocks_move(&expanded, IVec3::NEG_Y, hard_pusher_head_occupancy)
}

fn hard_pusher_head_blocked_below(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    structure.iter().any(|pos| {
        let Some(block) = world.blocks.get(pos) else {
            return false;
        };
        if !matches!(
            block.kind.movement_rule(block.facing),
            Some(crate::blocks::MovementRule::PoweredTranslate { .. })
        ) {
            return false;
        }
        let head = *pos + block.facing.forward_ivec3();
        if !hard_pusher_head_occupancy.contains(&head) {
            return false;
        }
        let target = head + IVec3::NEG_Y;
        // 活塞头是实体：下方有方块或其它活塞头都算挡住
        target.y < 0
            || (!structure.contains(&target)
                && (!world.can_move_into_yielding_fragile(target)
                    || hard_pusher_head_occupancy.contains(&target)))
    })
}

fn hard_pusher_head_blocks_move(
    structure: &HashSet<IVec3>,
    offset: IVec3,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
) -> bool {
    structure.iter().any(|pos| {
        let target = *pos + offset;
        !structure.contains(&target) && hard_pusher_head_occupancy.contains(&target)
    })
}

fn expanded_move_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    structures: &StructureState,
    mode: MovementExpansionMode,
    suction: &SuctionLinks,
) -> Option<HashSet<IVec3>> {
    // 先经吸盘并集；种子内已有结构 id 不膨胀，只并入其它粘连结构
    let structure = structures.linked_expand_pusher_subset(suction, structure, offset)?;
    let structure = with_factory_attachment_children(world, &structure);

    if offset.abs().element_sum() != 1 {
        return can_move_structure_without_push(world, &structure, offset)
            .then_some(structure);
    }

    let mut expanded = structure.clone();
    let mut queue: VecDeque<IVec3> = structure.iter().copied().collect();
    while let Some(pos) = queue.pop_front() {
        let target = pos + offset;
        if target.y < 0 || expanded.contains(&target) {
            continue;
        }
        if world.cell_accepts_move_from(pos, target) {
            continue;
        }

        let pushed = pushable_structure_at(world, structures, target, offset, suction)?;
        let pushed = with_factory_attachment_children(world, &pushed);
        if mode == MovementExpansionMode::Gravity && structure_supported_by_lifter(world, &pushed) {
            return None;
        }
        for pushed_pos in pushed {
            if expanded.insert(pushed_pos) {
                queue.push_back(pushed_pos);
            }
        }
    }

    can_move_structure_without_push(world, &expanded, offset)
        .then_some(expanded)
        .filter(|expanded| {
            !expanded
                .iter()
                .any(|pos| world.anchors_material_at_teleport_entrance(*pos))
        })
}

/// 把工厂附着子格（告示等）并入待移动集合
fn with_factory_attachment_children(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
) -> HashSet<IVec3> {
    if world.factory_attachments.is_empty() {
        return structure.clone();
    }
    let id_to_pos: HashMap<BlockId, IVec3> = world
        .blocks
        .iter()
        .filter(|(_, block)| !block.id.is_none())
        .map(|(pos, block)| (block.id, *pos))
        .collect();
    let parent_ids: HashSet<BlockId> = structure
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .filter(|id| !id.is_none())
        .collect();
    let mut expanded = structure.clone();
    for (child_id, att) in &world.factory_attachments {
        if !parent_ids.contains(&att.parent) {
            continue;
        }
        if let Some(&child_pos) = id_to_pos.get(child_id) {
            expanded.insert(child_pos);
        }
    }
    expanded
}

pub(super) fn can_translate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
    structures: &StructureState,
    suction: &SuctionLinks,
) -> bool {
    expanded_move_structure(
        world,
        structure,
        offset,
        structures,
        MovementExpansionMode::Normal,
        suction,
    )
    .is_some()
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MovementExpansionMode {
    Normal,
    Gravity,
}

fn movement_expansion_mode(mark: MovementMark, source: Option<BlockId>) -> MovementExpansionMode {
    if mark == MovementMark::Vertical && source.is_none() {
        MovementExpansionMode::Gravity
    } else {
        MovementExpansionMode::Normal
    }
}

fn pushable_structure_at(
    world: &WorldBlocks,
    structures: &StructureState,
    pos: IVec3,
    offset: IVec3,
    suction: &SuctionLinks,
) -> Option<HashSet<IVec3>> {
    let block = world.blocks.get(&pos)?;
    if block.kind.is_material() || block.kind.is_factory() {
        return structures.linked_pushable_at(suction, pos, offset);
    }
    None
}

fn can_move_structure_without_push(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    offset: IVec3,
) -> bool {
    structure.iter().all(|pos| {
        let target = *pos + offset;
        if target.y < 0 {
            return false;
        }
        if structure.contains(&target) || world.cell_accepts_move_from(*pos, target) {
            return true;
        }
        // 结构内脆弱撞实心：碎裂后放行
        world.is_fragile_material_at(*pos)
    })
}

pub(super) fn move_structure(world: &mut WorldBlocks, structure: &HashSet<IVec3>, offset: IVec3) {
    let moves: Vec<(IVec3, IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| {
            world
                .blocks
                .get(pos)
                .copied()
                .map(|block| (*pos, *pos + offset, block))
        })
        .collect();
    world.relocate_blocks(moves);
}

pub(super) fn can_rotate_structure(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) -> bool {
    structure.iter().all(|pos| {
        let target = rotate_pos_y(*pos, pivot, clockwise);
        target.y >= 0 && (structure.contains(&target) || world.can_place_platform_at(target))
    })
}

pub(super) fn rotate_structure(
    world: &mut WorldBlocks,
    structure: &HashSet<IVec3>,
    pivot: IVec3,
    clockwise: bool,
) {
    let structure_ids: HashSet<BlockId> = structure
        .iter()
        .filter_map(|pos| world.blocks.get(pos).map(|block| block.id))
        .collect();
    let moves: Vec<(IVec3, IVec3, BlockData)> = structure
        .iter()
        .filter_map(|pos| {
            world.blocks.get(pos).copied().map(|mut block| {
                block.facing = rotate_facing(block.facing, clockwise);
                (*pos, rotate_pos_y(*pos, pivot, clockwise), block)
            })
        })
        .collect();
    world.relocate_blocks(moves);

    // 焊接按 BlockId 无需改写；旋转只更新面附着法线
    let updated_paints: HashMap<_, _> = world
        .material_paints
        .iter()
        .map(|(face, color)| {
            if structure_ids.contains(&face.block) {
                (
                    MaterialFace {
                        block: face.block,
                        normal: rotate_offset_y(face.normal, clockwise),
                    },
                    *color,
                )
            } else {
                (*face, *color)
            }
        })
        .collect();
    world.material_paints = updated_paints;

    // 附着法线随结构绕 Y 旋转
    for att in world.material_attachments.values_mut() {
        if structure_ids.contains(&att.parent) {
            att.parent_face_normal = rotate_offset_y(att.parent_face_normal, clockwise);
        }
    }
    for att in world.factory_attachments.values_mut() {
        if structure_ids.contains(&att.parent) {
            att.parent_face_normal = rotate_offset_y(att.parent_face_normal, clockwise);
        }
    }

    let updated_panels: HashSet<_> = world
        .wire_face_panels
        .iter()
        .map(|face| {
            if structure_ids.contains(&face.block) {
                MaterialFace {
                    block: face.block,
                    normal: rotate_offset_y(face.normal, clockwise),
                }
            } else {
                *face
            }
        })
        .collect();
    if updated_panels != world.wire_face_panels {
        world.wire_face_panels = updated_panels;
        world.topology_revision = world.topology_revision.wrapping_add(1);
    }
}

pub(super) fn rotate_pos_y(pos: IVec3, pivot: IVec3, clockwise: bool) -> IVec3 {
    let rel = pos - pivot;
    pivot + rotate_offset_y(rel, clockwise)
}

fn rotate_offset_y(offset: IVec3, clockwise: bool) -> IVec3 {
    if clockwise {
        IVec3::new(-offset.z, offset.y, offset.x)
    } else {
        IVec3::new(offset.z, offset.y, -offset.x)
    }
}

fn rotate_facing(facing: Facing, clockwise: bool) -> Facing {
    if clockwise {
        facing.rotate()
    } else {
        facing.rotate_counter()
    }
}

