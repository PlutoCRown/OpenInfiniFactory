use std::fmt::Write;

use crate::game::blocks::BlockKind;
use crate::game::ui::types::InventoryItem;
use crate::game::world::direction::Facing;
use crate::shared::config::{ActionKeyName, GameConfig};
use crate::shared::save::SaveKind;

/// 游戏状态栏缓存：世界/手持很少变，瞄准行才跟视角每帧变
pub(crate) struct GameplayStatusCache {
    world_line: String,
    held_line: String,
    aim_tpl: String,
    place_tpl: String,
    aim_none: String,
    scene_label: String,
    last_block_kind: Option<Option<BlockKind>>,
    block_label: String,
    aim_line: String,
    place_line: String,
    composed: String,
    num_x: String,
    num_y: String,
    num_z: String,
}

impl Default for GameplayStatusCache {
    fn default() -> Self {
        Self {
            world_line: String::new(),
            held_line: String::new(),
            aim_tpl: String::new(),
            place_tpl: String::new(),
            aim_none: String::new(),
            scene_label: String::new(),
            last_block_kind: None,
            block_label: String::new(),
            aim_line: String::new(),
            place_line: String::new(),
            composed: String::new(),
            num_x: String::new(),
            num_y: String::new(),
            num_z: String::new(),
        }
    }
}

pub fn update_status_ui(
    _ui_thread: UiMainThread,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    save_state: Res<SaveState>,
    config: Res<GameConfig>,
    i18n_revision: Res<crate::game::ui::access::I18nRevision>,
    mut primed: Local<bool>,
    mut last_gameplay_sig: Local<(usize, Option<(IVec3, IVec3)>)>,
    mut gameplay_cache: Local<GameplayStatusCache>,
    mut texts: Query<(&StatusText, &mut Text)>,
) {
    let gameplay_sig = (
        placement.selected,
        placement.target.as_ref().map(|hit| (hit.pos, hit.normal)),
    );
    let i18n_dirty = i18n_revision.is_changed();
    let headers_dirty = !*primed
        || inventory.is_changed()
        || save_state.is_changed()
        || i18n_dirty
        || placement.selected != last_gameplay_sig.0;
    let target_dirty =
        !*primed || *last_gameplay_sig != gameplay_sig || world.is_changed() || i18n_dirty;
    let gameplay_dirty = headers_dirty || target_dirty;
    let simulation_dirty = !*primed
        || builder_mode.is_changed()
        || simulation.is_changed()
        || config.is_changed()
        || i18n_dirty;
    let force = !*primed;
    *primed = true;
    *last_gameplay_sig = gameplay_sig;
    if !force && !gameplay_dirty && !simulation_dirty {
        return;
    }

    if gameplay_dirty {
        if headers_dirty {
            refresh_gameplay_headers(
                &mut gameplay_cache,
                &placement,
                &inventory,
                &save_state,
                i18n_dirty || force,
            );
        }
        if target_dirty {
            refresh_gameplay_target(&mut gameplay_cache, &placement, &world);
        }
        compose_gameplay_status(&mut gameplay_cache);
    }

    for (status, mut text) in &mut texts {
        match status.0 {
            StatusTextKind::Gameplay => {
                if force || gameplay_dirty {
                    if text.0 != gameplay_cache.composed {
                        text.0.clone_from(&gameplay_cache.composed);
                    }
                }
            }
            StatusTextKind::SimulationOverlay => {
                if !force && !simulation_dirty {
                    continue;
                }
                let next = if *builder_mode != BuilderMode::Play {
                    String::new()
                } else {
                    simulation_overlay_text(&simulation, &config)
                };
                if text.0 != next {
                    text.0 = next;
                }
            }
        }
    }
}

fn refresh_gameplay_headers(
    cache: &mut GameplayStatusCache,
    placement: &PlacementState,
    inventory: &InventoryItems,
    save_state: &SaveState,
    refresh_templates: bool,
) {
    if refresh_templates {
        cache.aim_tpl = i18n.t("status.gameplay.aim");
        cache.place_tpl = i18n.t("status.gameplay.place");
        cache.aim_none = i18n.t("status.gameplay.aim_none");
        cache.scene_label = i18n.t("status.gameplay.scene");
        cache.last_block_kind = None;
    }
    cache.world_line = world_status_line(save_state);
    cache.held_line = held_status_line(inventory, placement);
}

fn refresh_gameplay_target(
    cache: &mut GameplayStatusCache,
    placement: &PlacementState,
    world: &WorldBlocks,
) {
    let Some(hit) = placement.target.as_ref() else {
        cache.aim_line.clone_from(&cache.aim_none);
        cache.place_line.clear();
        cache.last_block_kind = Some(None);
        return;
    };
    let place_at = hit.pos + hit.normal;
    let block = world
        .blocks
        .get(&hit.pos)
        .or_else(|| world.system_blocks.get(&hit.pos));
    let kind = block.map(|block| block.kind);
    if cache.last_block_kind != Some(kind) {
        cache.last_block_kind = Some(kind);
        match block {
            Some(block) => {
                cache.block_label = i18n.t(block.kind.name_key());
            }
            None => cache.block_label.clone_from(&cache.scene_label),
        }
    }
    let facing = block.map(|block| facing_label(block.facing)).unwrap_or("-");
    write_i32(&mut cache.num_x, hit.pos.x);
    write_i32(&mut cache.num_y, hit.pos.y);
    write_i32(&mut cache.num_z, hit.pos.z);
    subst_template(
        &cache.aim_tpl,
        &[
            ("x", cache.num_x.as_str()),
            ("y", cache.num_y.as_str()),
            ("z", cache.num_z.as_str()),
            ("block", cache.block_label.as_str()),
            ("facing", facing),
        ],
        &mut cache.aim_line,
    );
    write_i32(&mut cache.num_x, place_at.x);
    write_i32(&mut cache.num_y, place_at.y);
    write_i32(&mut cache.num_z, place_at.z);
    subst_template(
        &cache.place_tpl,
        &[
            ("x", cache.num_x.as_str()),
            ("y", cache.num_y.as_str()),
            ("z", cache.num_z.as_str()),
        ],
        &mut cache.place_line,
    );
}

fn compose_gameplay_status(cache: &mut GameplayStatusCache) {
    cache.composed.clear();
    cache.composed.push_str(&cache.world_line);
    cache.composed.push('\n');
    cache.composed.push_str(&cache.held_line);
    cache.composed.push('\n');
    cache.composed.push_str(&cache.aim_line);
    if !cache.place_line.is_empty() {
        cache.composed.push('\n');
        cache.composed.push_str(&cache.place_line);
    }
}

fn simulation_overlay_text(simulation: &SimulationState, config: &GameConfig) -> String {
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
    let (state_key, controls_key, controls_args): (&str, &str, Vec<(&str, String)>) =
        if !simulation.is_active() {
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

fn world_status_line(save_state: &SaveState) -> String {
    let Some(slot) = save_state.current.as_ref() else {
        return i18n.t("status.gameplay.no_world");
    };
    let kind = save_state.current_kind.unwrap_or_else(|| slot.kind());
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

fn facing_label(facing: Facing) -> &'static str {
    match facing {
        Facing::North => "N",
        Facing::East => "E",
        Facing::South => "S",
        Facing::West => "W",
    }
}

fn write_i32(buf: &mut String, value: i32) {
    buf.clear();
    let _ = write!(buf, "{value}");
}

/// 把 `{name}` 模板填进 out，避免 i18n.fmt 每帧多次 String::replace
fn subst_template(template: &str, values: &[(&str, &str)], out: &mut String) {
    out.clear();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            return;
        };
        let key = &after[..end];
        if let Some((_, value)) = values.iter().find(|(name, _)| *name == key) {
            out.push_str(value);
        } else {
            out.push('{');
            out.push_str(key);
            out.push('}');
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
}
