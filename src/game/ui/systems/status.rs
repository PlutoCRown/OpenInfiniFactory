use std::fmt::Write;

use crate::game::blocks::BlockKind;
use crate::game::world::direction::Facing;
use crate::shared::config::{ActionKeyName, GameConfig};
use crate::shared::i18n::{I18n, subst_template};
use crate::shared::save::SaveKind;

/// 游戏状态栏缓存：世界/手持很少变，瞄准行跟 AimFocus 变
pub struct GameplayStatusCache {
    world_line: String,
    blocks_line: String,
    blocks_tpl: String,
    held_line: String,
    aim_tpl: String,
    place_tpl: String,
    structure_tpl: String,
    aim_none: String,
    structure_none: String,
    scene_label: String,
    last_block_kind: Option<Option<BlockKind>>,
    block_label: String,
    aim_line: String,
    structure_line: String,
    place_line: String,
    composed: String,
    num_x: String,
    num_y: String,
    num_z: String,
    num_scene: String,
    num_factory: String,
    num_material: String,
    /// 模拟 overlay 嵌套模板缓冲
    overlay_controls: String,
    overlay_out: String,
    turn_buf: String,
}

impl Default for GameplayStatusCache {
    fn default() -> Self {
        Self {
            world_line: String::new(),
            blocks_line: String::new(),
            blocks_tpl: String::new(),
            held_line: String::new(),
            aim_tpl: String::new(),
            place_tpl: String::new(),
            structure_tpl: String::new(),
            aim_none: String::new(),
            structure_none: String::new(),
            scene_label: String::new(),
            last_block_kind: None,
            block_label: String::new(),
            aim_line: String::new(),
            structure_line: String::new(),
            place_line: String::new(),
            composed: String::new(),
            num_x: String::new(),
            num_y: String::new(),
            num_z: String::new(),
            num_scene: String::new(),
            num_factory: String::new(),
            num_material: String::new(),
            overlay_controls: String::new(),
            overlay_out: String::new(),
            turn_buf: String::new(),
        }
    }
}

pub fn update_status_ui(
    _ui_thread: UiMainThread,
    locale: Res<I18n>,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    aim: Res<AimFocus>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    save_state: Res<SaveState>,
    config: Res<GameConfig>,
    mut primed: Local<bool>,
    mut last_selected: Local<usize>,
    mut gameplay_cache: Local<GameplayStatusCache>,
    mut texts: Query<(&StatusText, &mut Text)>,
) {
    let force = !*primed;
    let headers_dirty = force
        || inventory.is_changed()
        || save_state.is_changed()
        || placement.selected != *last_selected;
    let aim_dirty = force || aim.is_changed();
    let blocks_dirty = force || world.is_changed();
    let gameplay_dirty = headers_dirty || aim_dirty || blocks_dirty;
    let simulation_dirty =
        force || builder_mode.is_changed() || simulation.is_changed() || config.is_changed();
    *primed = true;
    *last_selected = placement.selected;
    if !force && !gameplay_dirty && !simulation_dirty {
        return;
    }

    if gameplay_dirty {
        if headers_dirty {
            refresh_gameplay_headers(
                &mut gameplay_cache,
                &locale,
                &placement,
                &inventory,
                &save_state,
                force,
            );
        }
        if aim_dirty {
            refresh_gameplay_target(&mut gameplay_cache, &locale, &aim);
        }
        if blocks_dirty {
            write_blocks_status_line(&mut gameplay_cache, &world);
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
                let next_empty = *builder_mode != BuilderMode::Play;
                if next_empty {
                    if !text.0.is_empty() {
                        text.0.clear();
                    }
                } else {
                    write_simulation_overlay(&mut gameplay_cache, &locale, &simulation, &config);
                    if text.0 != gameplay_cache.overlay_out {
                        text.0.clone_from(&gameplay_cache.overlay_out);
                    }
                }
            }
        }
    }
}

fn refresh_gameplay_headers(
    cache: &mut GameplayStatusCache,
    locale: &I18n,
    placement: &PlacementState,
    inventory: &InventoryItems,
    save_state: &SaveState,
    refresh_templates: bool,
) {
    if refresh_templates {
        cache.aim_tpl.clear();
        cache.aim_tpl.push_str(locale.t("status.gameplay.aim"));
        cache.place_tpl.clear();
        cache.place_tpl.push_str(locale.t("status.gameplay.place"));
        cache.structure_tpl.clear();
        cache
            .structure_tpl
            .push_str(locale.t("status.gameplay.structure"));
        cache.aim_none.clear();
        cache
            .aim_none
            .push_str(locale.t("status.gameplay.aim_none"));
        cache.structure_none.clear();
        cache
            .structure_none
            .push_str(locale.t("status.gameplay.structure_none"));
        cache.scene_label.clear();
        cache
            .scene_label
            .push_str(locale.t("status.gameplay.scene"));
        cache.blocks_tpl.clear();
        cache
            .blocks_tpl
            .push_str(locale.t("status.gameplay.blocks"));
        cache.last_block_kind = None;
    }
    write_world_status_line(&mut cache.world_line, locale, save_state);
    write_held_status_line(&mut cache.held_line, locale, inventory, placement);
}

fn refresh_gameplay_target(cache: &mut GameplayStatusCache, locale: &I18n, aim: &AimFocus) {
    let Some(hit) = aim.hit else {
        cache.aim_line.clone_from(&cache.aim_none);
        cache.structure_line.clear();
        cache.place_line.clear();
        cache.last_block_kind = Some(None);
        return;
    };
    let kind = aim.block.map(|block| block.kind);
    if cache.last_block_kind != Some(kind) {
        cache.last_block_kind = Some(kind);
        match aim.block {
            Some(block) => {
                cache.block_label.clear();
                cache.block_label.push_str(locale.t(block.kind.name_key()));
            }
            None => cache.block_label.clone_from(&cache.scene_label),
        }
    }
    let facing = aim
        .block
        .map(|block| facing_label(block.facing))
        .unwrap_or("-");
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
    if let Some(block) = aim.block {
        if !block.id.is_none() {
            let _ = write!(cache.aim_line, " #{}", block.id.0);
        }
    }
    match aim.structure_id {
        Some(id) => {
            write_u64(&mut cache.num_x, id.0);
            subst_template(
                &cache.structure_tpl,
                &[("id", cache.num_x.as_str())],
                &mut cache.structure_line,
            );
        }
        None => cache.structure_line.clone_from(&cache.structure_none),
    }
    let place_at = aim.place_at.unwrap_or(hit.pos);
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
    cache.composed.push_str(&cache.blocks_line);
    cache.composed.push('\n');
    cache.composed.push_str(&cache.held_line);
    cache.composed.push('\n');
    cache.composed.push_str(&cache.aim_line);
    if !cache.structure_line.is_empty() {
        cache.composed.push('\n');
        cache.composed.push_str(&cache.structure_line);
    }
    if !cache.place_line.is_empty() {
        cache.composed.push('\n');
        cache.composed.push_str(&cache.place_line);
    }
}

/// 统计场景 / 工厂 / 材料三层方块数量（读 WorldBlocks 增量计数，O(1)）
fn write_blocks_status_line(cache: &mut GameplayStatusCache, world: &WorldBlocks) {
    write_usize(&mut cache.num_scene, world.scene_count);
    write_usize(&mut cache.num_factory, world.factory_count);
    write_usize(&mut cache.num_material, world.material_count);
    if cache.blocks_tpl.is_empty() {
        return;
    }
    subst_template(
        &cache.blocks_tpl,
        &[
            ("scene", cache.num_scene.as_str()),
            ("factory", cache.num_factory.as_str()),
            ("material", cache.num_material.as_str()),
        ],
        &mut cache.blocks_line,
    );
}

fn write_simulation_overlay(
    cache: &mut GameplayStatusCache,
    locale: &I18n,
    simulation: &SimulationState,
    config: &GameConfig,
) {
    let start = config.input(ActionKeyName::Simulate).name();
    let fast = config.input(ActionKeyName::SimulationFast).name();
    let step = config.input(ActionKeyName::SimulationStep).name();
    let rollback = config.input(ActionKeyName::SimulationRollback).name();
    let (state_key, controls_key, controls_args): (&str, &str, &[(&str, &str)]) =
        if !simulation.is_active() {
            (
                "simulation_state.ready",
                "simulation_controls.ready",
                &[("start", start)],
            )
        } else if simulation.running && simulation.speed > 1.0 {
            (
                "simulation_state.fast",
                "simulation_controls.fast",
                &[("fast", fast), ("step", step), ("rollback", rollback)],
            )
        } else if simulation.running {
            (
                "simulation_state.playing",
                "simulation_controls.playing",
                &[("step", step), ("fast", fast), ("rollback", rollback)],
            )
        } else {
            (
                "simulation_state.paused",
                "simulation_controls.paused",
                &[("step", step), ("start", start), ("rollback", rollback)],
            )
        };
    subst_template(
        locale.t(controls_key),
        controls_args,
        &mut cache.overlay_controls,
    );
    write_u64(&mut cache.turn_buf, simulation.turn);
    let state = locale.t(state_key);
    subst_template(
        locale.t("status.simulation_overlay"),
        &[
            ("state", state),
            ("turns", cache.turn_buf.as_str()),
            ("controls", cache.overlay_controls.as_str()),
        ],
        &mut cache.overlay_out,
    );
}

fn write_world_status_line(out: &mut String, locale: &I18n, save_state: &SaveState) {
    let Some(slot) = save_state.current.as_ref() else {
        out.clear();
        out.push_str(locale.t("status.gameplay.no_world"));
        return;
    };
    let kind = save_state.current_kind.unwrap_or_else(|| slot.kind());
    let kind_label = locale.t(match kind {
        SaveKind::Puzzle => "save.kind.puzzle",
        SaveKind::Free => "save.kind.free",
        SaveKind::Solution => "save.kind.solution",
    });
    let name = slot.display_name();
    locale.fmt_into(
        out,
        "status.gameplay.world",
        &[("name", name.as_str()), ("kind", kind_label)],
    );
}

fn write_held_status_line(
    out: &mut String,
    locale: &I18n,
    inventory: &InventoryItems,
    placement: &PlacementState,
) {
    let item_label = match inventory.hotbar[placement.selected] {
        Some(item) => locale.t(item.name_key()),
        None => locale.t("empty"),
    };
    locale.fmt_into(out, "status.gameplay.held", &[("item", item_label)]);
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

fn write_u64(buf: &mut String, value: u64) {
    buf.clear();
    let _ = write!(buf, "{value}");
}

fn write_usize(buf: &mut String, value: usize) {
    buf.clear();
    let _ = write!(buf, "{value}");
}
