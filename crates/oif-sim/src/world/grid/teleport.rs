//! 传送门配对与命名

use glam::IVec3;
use std::collections::HashSet;

use crate::blocks::BlockKind;

use super::{BlockSettings, TeleportSettings, WorldBlocks};

const TELEPORT_ENTRANCE_NAMES: &[&str] = &["Alpha In", "Beta In", "Gamma In", "Delta In"];
const TELEPORT_EXIT_NAMES: &[&str] = &["Alpha Out", "Beta Out", "Gamma Out", "Delta Out"];

impl WorldBlocks {
    pub fn teleport_settings(&self, pos: IVec3) -> TeleportSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Teleport(settings)) => settings.clone(),
            _ => TeleportSettings::unnamed(pos),
        }
    }

    pub fn teleport_partner(&self, pos: IVec3) -> Option<IVec3> {
        if let Some(pair) = self.teleport_settings(pos).pair {
            if self
                .system_blocks
                .get(&pair)
                .is_some_and(|block| self.teleport_roles_match(pos, pair, block.kind))
            {
                return Some(pair);
            }
        }
        for (other_pos, settings) in &self.block_settings {
            if *other_pos == pos {
                continue;
            }
            let BlockSettings::Teleport(settings) = settings else {
                continue;
            };
            if settings.pair != Some(pos) {
                continue;
            }
            let Some(block) = self.system_blocks.get(other_pos) else {
                continue;
            };
            if self.teleport_roles_match(pos, *other_pos, block.kind) {
                return Some(*other_pos);
            }
        }
        None
    }

    pub fn set_teleport_pair(&mut self, pos: IVec3, partner: Option<IVec3>) {
        let Some(block) = self.system_blocks.get(&pos).copied() else {
            return;
        };
        if !block
            .kind
            .material_processor()
            .is_some_and(|processor| processor.is_teleport())
        {
            return;
        }

        if let Some(old) = self.teleport_settings(pos).pair {
            if partner != Some(old) {
                let mut old_settings = self.teleport_settings(old);
                if old_settings.pair == Some(pos) {
                    old_settings.pair = None;
                    self.set_teleport_settings(old, old_settings);
                }
            }
        }

        if let Some(partner_pos) = partner {
            let Some(partner_block) = self.system_blocks.get(&partner_pos).copied() else {
                return;
            };
            if !self.teleport_roles_match(pos, partner_pos, partner_block.kind) {
                return;
            }

            if let Some(previous) = self.teleport_settings(partner_pos).pair {
                if previous != pos {
                    let mut previous_settings = self.teleport_settings(previous);
                    previous_settings.pair = None;
                    self.set_teleport_settings(previous, previous_settings);
                }
            }

            let mut partner_settings = self.teleport_settings(partner_pos);
            partner_settings.pair = Some(pos);
            self.set_teleport_settings(partner_pos, partner_settings);
        }

        let mut settings = self.teleport_settings(pos);
        settings.pair = partner;
        self.set_teleport_settings(pos, settings);
    }

    pub fn set_teleport_settings(&mut self, pos: IVec3, settings: TeleportSettings) {
        self.set_block_settings(pos, BlockSettings::Teleport(settings));
    }

    fn teleport_roles_match(&self, pos: IVec3, other: IVec3, other_kind: BlockKind) -> bool {
        let Some(block) = self.system_blocks.get(&pos) else {
            return false;
        };
        let Some(role) = block.kind.material_processor() else {
            return false;
        };
        role.teleport_partner_role() == other_kind.material_processor() && pos != other
    }

    pub(super) fn next_teleport_name(&self, kind: BlockKind) -> String {
        let base_names = match kind.material_processor() {
            Some(crate::blocks::MaterialProcessor::TeleportEntrance) => TELEPORT_ENTRANCE_NAMES,
            Some(crate::blocks::MaterialProcessor::TeleportExit) => TELEPORT_EXIT_NAMES,
            _ => &[],
        };
        let used: HashSet<String> = self
            .block_settings
            .iter()
            .filter_map(|(pos, settings)| {
                if !self
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| block.kind == kind)
                {
                    return None;
                }
                match settings {
                    BlockSettings::Teleport(settings) => Some(settings.name.clone()),
                    _ => None,
                }
            })
            .collect();

        for name in base_names {
            if !used.contains(*name) {
                return (*name).to_owned();
            }
        }

        for index in 2.. {
            for name in base_names {
                let candidate = format!("{name} {index}");
                if !used.contains(&candidate) {
                    return candidate;
                }
            }
        }
        unreachable!()
    }
}
