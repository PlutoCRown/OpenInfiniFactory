use crate::game::blocks::BlockData;
use crate::game::ui::types::InventoryItem;
use crate::game::world::direction::Facing;
use crate::shared::save::SaveKind;

pub fn update_status_ui(
    _ui_thread: UiMainThread,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    save_state: Res<SaveState>,
    mut texts: Query<(&StatusText, &mut Text)>,
) {
    for (status, mut text) in &mut texts {
        if status.0 != StatusTextKind::Gameplay {
            continue;
        }
        let next = gameplay_status_text(&placement, &world, &inventory, &save_state);
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
