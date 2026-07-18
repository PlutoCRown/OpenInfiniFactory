use crate::game::blocks::BlockData;
use crate::game::ui::types::InventoryItem;
use crate::game::world::direction::Facing;
use crate::shared::config::{ActionKeyName, GameConfig};
use crate::shared::save::SaveKind;

pub fn update_status_ui(
    _ui_thread: UiMainThread,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    save_state: Res<SaveState>,
    config: Res<GameConfig>,
    mut texts: Query<(&StatusText, &mut Text)>,
) {
    for (status, mut text) in &mut texts {
        let next = match status.0 {
            StatusTextKind::Gameplay => {
                gameplay_status_text(&placement, &world, &inventory, &save_state)
            }
            StatusTextKind::SimulationOverlay => {
                if *builder_mode != BuilderMode::Play {
                    String::new()
                } else {
                    let start = config.input(ActionKeyName::Simulate).name().to_string();
                    let fast = config
                        .input(ActionKeyName::SimulationFast)
                        .name()
                        .to_string();
                    let step = config
                        .input(ActionKeyName::SimulationStep)
                        .name()
                        .to_string();
                    let rollback = config
                        .input(ActionKeyName::SimulationRollback)
                        .name()
                        .to_string();
                    let (state_key, controls_key, controls_args): (
                        &str,
                        &str,
                        Vec<(&str, String)>,
                    ) = if !simulation.is_active() {
                        (
                            "simulation_state.ready",
                            "simulation_controls.ready",
                            vec![("start", start)],
                        )
                    } else if simulation.running && simulation.speed > 1.0 {
                        (
                            "simulation_state.fast",
                            "simulation_controls.fast",
                            vec![("fast", fast), ("step", step), ("rollback", rollback)],
                        )
                    } else if simulation.running {
                        (
                            "simulation_state.playing",
                            "simulation_controls.playing",
                            vec![("step", step), ("fast", fast), ("rollback", rollback)],
                        )
                    } else {
                        (
                            "simulation_state.paused",
                            "simulation_controls.paused",
                            vec![("step", step), ("start", start), ("rollback", rollback)],
                        )
                    };
                    i18n.fmt(
                        "status.simulation_overlay",
                        &[
                            ("state", i18n.t(state_key)),
                            ("turns", simulation.turn.to_string()),
                            ("controls", i18n.fmt(controls_key, &controls_args)),
                        ],
                    )
                }
            }
        };
        if text.0 != next {
            text.0 = next;
        }
    }
}

fn gameplay_status_text(
    placement: &PlacementState,
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &SaveState,
) -> String {
    let mut lines = Vec::new();
    lines.push(world_status_line(save_state));
    lines.push(held_status_line(inventory, placement));
    lines.extend(target_status_lines(placement, world));
    lines.join("\n")
}

fn world_status_line(save_state: &SaveState) -> String {
    let Some(slot) = save_state.current.as_ref() else {
        return i18n.t("status.gameplay.no_world");
    };
    let kind = save_state
        .current_kind
        .unwrap_or_else(|| slot.kind());
    let kind_label = i18n.t(match kind {
        SaveKind::Puzzle => "save.kind.puzzle",
        SaveKind::Solution => "save.kind.solution",
    });
    i18n.fmt(
        "status.gameplay.world",
        &[("name", slot.display_name()), ("kind", kind_label)],
    )
}

fn held_status_line(inventory: &InventoryItems, placement: &PlacementState) -> String {
    let item_label = inventory.hotbar[placement.selected]
        .map(inventory_item_label)
        .unwrap_or_else(|| i18n.t("empty"));
    i18n.fmt("status.gameplay.held", &[("item", item_label)])
}

fn inventory_item_label(item: InventoryItem) -> String {
    i18n.t(item.name_key())
}

fn target_status_lines(placement: &PlacementState, world: &WorldBlocks) -> Vec<String> {
    let Some(hit) = placement.target.as_ref() else {
        return vec![i18n.t("status.gameplay.aim_none")];
    };
    let place_at = hit.pos + hit.normal;
    let block = world
        .blocks
        .get(&hit.pos)
        .or_else(|| world.system_blocks.get(&hit.pos));
    let block_label = block
        .map(block_label)
        .unwrap_or_else(|| i18n.t("status.gameplay.scene"));
    let facing = block
        .map(|block| facing_label(block.facing))
        .unwrap_or("-");
    vec![
        i18n.fmt(
            "status.gameplay.aim",
            &[
                ("x", hit.pos.x.to_string()),
                ("y", hit.pos.y.to_string()),
                ("z", hit.pos.z.to_string()),
                ("block", block_label),
                ("facing", facing.to_string()),
            ],
        ),
        i18n.fmt(
            "status.gameplay.place",
            &[
                ("x", place_at.x.to_string()),
                ("y", place_at.y.to_string()),
                ("z", place_at.z.to_string()),
            ],
        ),
    ]
}

fn block_label(block: &BlockData) -> String {
    i18n.t(block.kind.name_key())
}

fn facing_label(facing: Facing) -> &'static str {
    match facing {
        Facing::North => "N",
        Facing::East => "E",
        Facing::South => "S",
        Facing::West => "W",
    }
}
