//! 传送门配对与命名

use glam::IVec3;
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::blocks::BlockKind;

use super::{BlockSettings, TeleportSettings, WorldBlocks};

/// Culture 文明舰名（Iain M. Banks），用作传送口默认名
static CULTURE_SHIP_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../culture_ship_names.json"))
        .expect("culture_ship_names.json must be a JSON string array")
});

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

    pub(super) fn next_teleport_name(&self) -> String {
        let names = CULTURE_SHIP_NAMES.as_slice();
        // 入口/出口共用一名单，已占用名全局唯一
        let used: HashSet<&str> = self
            .block_settings
            .iter()
            .filter_map(|(pos, settings)| {
                if !self.system_blocks.get(pos).is_some_and(|block| {
                    block
                        .kind
                        .material_processor()
                        .is_some_and(|processor| processor.is_teleport())
                }) {
                    return None;
                }
                match settings {
                    BlockSettings::Teleport(settings) => Some(settings.name.as_str()),
                    _ => None,
                }
            })
            .collect();

        for name in names {
            if !used.contains(name.as_str()) {
                return name.clone();
            }
        }

        for index in 2.. {
            for name in names {
                let candidate = format!("{name} {index}");
                if !used.contains(candidate.as_str()) {
                    return candidate;
                }
            }
        }
        unreachable!()
    }
}
