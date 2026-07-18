mod input;
mod patch;

pub use input::edit_history_input;
pub use patch::{
    build_cell_patch, build_relocate_patch, build_rotation_patch, build_settings_patch,
    capture_cell, capture_welds_for_ids, weld_diff, FacePanelDelta, WorldPatch,
};

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::world::direction::Facing;
use crate::game::world::grid::{BlockSettings, WorldBlocks};
use crate::game::state::{SelectionSnapshot, SelectionState};

use patch::CellSnapshot;

use patch::SettingsDelta;

const MAX_UNDO: usize = 128;

/// 单条可撤销编辑命令
#[derive(Clone, Debug)]
pub struct EditCommand {
    pub patch: WorldPatch,
    /// 选区状态前后快照（创建/取消/搬移/复制选区时写入）
    pub selection: Option<(SelectionSnapshot, SelectionSnapshot)>,
}

/// 连续旋转合并缓冲
#[derive(Clone, Debug)]
struct PendingRotation {
    pos: IVec3,
    original: CellSnapshot,
    current_facing: Facing,
}

/// 运行时编辑历史栈，不写入存档
#[derive(Resource, Default)]
pub struct EditHistory {
    undo: Vec<EditCommand>,
    redo: Vec<EditCommand>,
    pending_rotation: Option<PendingRotation>,
}

impl EditHistory {
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.pending_rotation = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.pending_rotation.is_some()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// 提交一条补丁；会先冲刷未提交的旋转合并
    pub fn record(&mut self, patch: WorldPatch) {
        self.record_full(patch, None);
    }

    /// 提交补丁并附带选区状态变化
    pub fn record_with_selection(
        &mut self,
        patch: WorldPatch,
        before: SelectionSnapshot,
        after: SelectionSnapshot,
    ) {
        self.record_full(patch, Some((before, after)));
    }

    fn record_full(
        &mut self,
        patch: WorldPatch,
        selection: Option<(SelectionSnapshot, SelectionSnapshot)>,
    ) {
        self.flush_pending_rotation();
        if patch.is_empty() {
            match &selection {
                Some((before, after)) if before != after => {}
                _ => return,
            }
        }
        self.undo.push(EditCommand { patch, selection });
        if self.undo.len() > MAX_UNDO {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// 仅记录设置变更
    pub fn record_settings(
        &mut self,
        pos: IVec3,
        before: Option<BlockSettings>,
        after: Option<BlockSettings>,
    ) {
        if before == after {
            return;
        }
        self.record(build_settings_patch(pos, before, after));
    }

    /// 旋转前准备：必要时冲刷其它格的待合并旋转
    pub fn prepare_rotation(&mut self, world: &WorldBlocks, pos: IVec3) {
        if self
            .pending_rotation
            .as_ref()
            .is_some_and(|pending| pending.pos != pos)
        {
            self.flush_pending_rotation();
        }
        if self.pending_rotation.is_none() {
            if let Some(original) = capture_cell(world, pos) {
                let current_facing = original.block.facing;
                self.pending_rotation = Some(PendingRotation {
                    pos,
                    original,
                    current_facing,
                });
            }
        }
    }

    /// 旋转完成后更新合并缓冲
    pub fn finish_rotation(&mut self, pos: IVec3, facing: Facing) {
        if let Some(pending) = &mut self.pending_rotation {
            if pending.pos == pos {
                pending.current_facing = facing;
            }
        }
    }

    /// 将连续旋转合并为一条命令
    pub fn flush_pending_rotation(&mut self) {
        let Some(pending) = self.pending_rotation.take() else {
            return;
        };
        if let Some(patch) =
            build_rotation_patch(pending.pos, pending.original, pending.current_facing)
        {
            self.push_without_flush(patch);
        }
    }

    pub fn undo(
        &mut self,
        world: &mut WorldBlocks,
        selection: &mut SelectionState,
    ) -> Option<WorldPatch> {
        self.flush_pending_rotation();
        let command = self.undo.pop()?;
        let patch = command.patch.clone();
        patch.apply_inverse(world);
        if let Some((before, _)) = &command.selection {
            before.apply_to(selection);
        }
        self.redo.push(command);
        Some(patch)
    }

    pub fn redo(
        &mut self,
        world: &mut WorldBlocks,
        selection: &mut SelectionState,
    ) -> Option<WorldPatch> {
        self.flush_pending_rotation();
        let command = self.redo.pop()?;
        let patch = command.patch.clone();
        patch.apply_forward(world);
        if let Some((_, after)) = &command.selection {
            after.apply_to(selection);
        }
        self.undo.push(command);
        Some(patch)
    }

    fn push_without_flush(&mut self, patch: WorldPatch) {
        if patch.is_empty() {
            return;
        }
        self.undo.push(EditCommand {
            patch,
            selection: None,
        });
        if self.undo.len() > MAX_UNDO {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
}

/// 应用单格设置变更并记入历史
pub fn apply_block_settings_with_history(
    history: &mut EditHistory,
    world: &mut WorldBlocks,
    pos: IVec3,
    apply: impl FnOnce(&mut WorldBlocks),
) {
    let before = world.block_settings.get(&pos).cloned();
    apply(world);
    let after = world.block_settings.get(&pos).cloned();
    history.record_settings(pos, before, after);
}

/// 传送门配对可能改动多个格子的设置
pub fn apply_teleport_pair_with_history(
    history: &mut EditHistory,
    world: &mut WorldBlocks,
    pos: IVec3,
    partner: Option<IVec3>,
) {
    let old_pair = world.teleport_settings(pos).pair;
    let mut affected = HashSet::from([pos]);
    if let Some(old) = old_pair {
        affected.insert(old);
    }
    if let Some(new) = partner {
        affected.insert(new);
        if let Some(previous) = world.teleport_settings(new).pair {
            affected.insert(previous);
        }
    }
    let before: HashMap<IVec3, Option<BlockSettings>> = affected
        .iter()
        .map(|p| (*p, world.block_settings.get(p).cloned()))
        .collect();
    world.set_teleport_pair(pos, partner);
    let settings: Vec<SettingsDelta> = before
        .into_iter()
        .map(|(p, b)| SettingsDelta {
            pos: p,
            before: b,
            after: world.block_settings.get(&p).cloned(),
        })
        .filter(|delta| delta.before != delta.after)
        .collect();
    if settings.is_empty() {
        return;
    }
    history.record(WorldPatch {
        settings,
        ..Default::default()
    });
}
