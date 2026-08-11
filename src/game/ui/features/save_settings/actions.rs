use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::player::controller::{FlyCamera, capture_player_save};
use crate::game::state::SolutionState;
use crate::game::ui::access::{UiMainThread, i18n, ui};
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::shared::save::{PuzzleLighting, read_save_settings, write_save_settings};

use super::types::{SaveSettingsAction, SaveSettingsUiState};

pub fn emit_save_settings_actions(
    mut click: On<Pointer<Click>>,
    ui_host: Res<UiHost>,
    runtime: Res<UiRuntime>,
    mut writer: MessageWriter<UiAction>,
    actions: Query<&SaveSettingsAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) || !runtime.is_save_settings_open() {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::SAVE_SETTINGS,
        kind: UiActionKind::SaveSettings(action),
    });
}

fn persist_settings(state: &mut SaveSettingsUiState, lighting: &mut PuzzleLighting) {
    let Some(slot) = state.slot.clone() else {
        return;
    };
    if !write_save_settings(&slot, &state.data) {
        return;
    }
    lighting.position = state.data.light_position;
    lighting.direction = state.data.light_direction;
    lighting.illuminance = state.data.light_intensity;
}

fn parse_values(value: &str, count: usize) -> Option<Vec<f32>> {
    let values: Vec<f32> = value
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    (values.len() == count && values.iter().all(|value| value.is_finite())).then_some(values)
}

fn format_vec3(value: Option<Vec3>) -> String {
    value
        .map(|value| format!("{:.3}, {:.3}, {:.3}", value.x, value.y, value.z))
        .unwrap_or_default()
}

fn open_value_prompt(action: SaveSettingsAction, state: &SaveSettingsUiState) {
    let (title, default_value, count) = match action {
        SaveSettingsAction::EditLightPosition => (
            "save_settings.light_position",
            format_vec3(state.data.light_position),
            3,
        ),
        SaveSettingsAction::EditLightDirection => (
            "save_settings.light_direction",
            format_vec3(state.data.light_direction),
            3,
        ),
        SaveSettingsAction::EditLightIntensity => (
            "save_settings.light_intensity",
            state.data.light_intensity.to_string(),
            1,
        ),
        SaveSettingsAction::EditSolutionSpawn => (
            "save_settings.solution_spawn",
            state
                .data
                .solution_spawn
                .as_ref()
                .map(|save| {
                    format!(
                        "{:.3}, {:.3}, {:.3}, {:.3}, {:.3}",
                        save.x, save.y, save.z, save.yaw, save.pitch
                    )
                })
                .unwrap_or_default(),
            5,
        ),
        _ => return,
    };
    let Some(slot) = state.slot.clone() else {
        return;
    };
    ui.open_text_prompt_then(
        crate::game::ui::core::text_prompt::TextPromptProps {
            title: i18n.t(title),
            default_value,
            save_text: i18n.t("button.confirm"),
            cancel_text: i18n.t("button.cancel"),
            max_characters: None,
        },
        move |result, world| {
            let crate::game::ui::core::text_prompt::TextPromptResult::Saved(value) = result else {
                return;
            };
            let Some(values) = parse_values(&value, count) else {
                return;
            };
            let mut current = read_save_settings(&slot).unwrap_or_default();
            match action {
                SaveSettingsAction::EditLightPosition => {
                    current.light_position = Some(Vec3::new(values[0], values[1], values[2]));
                }
                SaveSettingsAction::EditLightDirection => {
                    let direction = Vec3::new(values[0], values[1], values[2]).normalize_or_zero();
                    if direction == Vec3::ZERO {
                        return;
                    }
                    current.light_direction = Some(direction);
                }
                SaveSettingsAction::EditLightIntensity => {
                    current.light_intensity = values[0].max(0.0);
                }
                SaveSettingsAction::EditSolutionSpawn => {
                    current.solution_spawn = Some(crate::shared::save::PlayerSave {
                        x: values[0],
                        y: values[1],
                        z: values[2],
                        yaw: values[3],
                        pitch: values[4],
                        flying: false,
                    });
                }
                _ => return,
            }
            if write_save_settings(&slot, &current) {
                let mut state = world.resource_mut::<SaveSettingsUiState>();
                state.data = current.clone();
                let mut solution_state = world.resource_mut::<SolutionState>();
                solution_state.solution_spawn = current.solution_spawn.clone();
                solution_state.factory_block_filter = Some(current.factory_block_filter.clone());
                let mut lighting = world.resource_mut::<PuzzleLighting>();
                lighting.position = current.light_position;
                lighting.direction = current.light_direction;
                lighting.illuminance = current.light_intensity;
            }
        },
    );
}

pub fn dispatch_save_settings_actions(
    _ui_thread: UiMainThread,
    mut actions: MessageReader<UiAction>,
    mut state: ResMut<SaveSettingsUiState>,
    mut solution_state: ResMut<SolutionState>,
    mut lighting: ResMut<PuzzleLighting>,
    player: Query<(&FlyCamera, &Transform)>,
) {
    for message in actions.read() {
        if message.instance != UiInstanceId::SAVE_SETTINGS {
            continue;
        }
        let UiActionKind::SaveSettings(action) = message.kind.clone() else {
            continue;
        };
        match action {
            SaveSettingsAction::EditLightPosition
            | SaveSettingsAction::EditLightDirection
            | SaveSettingsAction::EditLightIntensity
            | SaveSettingsAction::EditSolutionSpawn => {
                open_value_prompt(action, &state);
            }
            SaveSettingsAction::CaptureLightPose => {
                let Ok((_, transform)) = player.single() else {
                    continue;
                };
                state.data.light_position = Some(transform.translation);
                state.data.light_direction =
                    Some((transform.rotation * Vec3::Z).normalize_or_zero());
                persist_settings(&mut state, &mut lighting);
            }
            SaveSettingsAction::CaptureSolutionSpawn => {
                let Ok((camera, transform)) = player.single() else {
                    continue;
                };
                state.data.solution_spawn = Some(capture_player_save(camera, transform));
                solution_state.solution_spawn = state.data.solution_spawn.clone();
                persist_settings(&mut state, &mut lighting);
            }
            SaveSettingsAction::SetFactoryFilterMode(mode) => {
                state.data.factory_block_filter.mode = mode;
                solution_state.factory_block_filter = Some(state.data.factory_block_filter.clone());
                persist_settings(&mut state, &mut lighting);
            }
            SaveSettingsAction::OpenFactoryPicker => {
                state.picker_open = true;
            }
            SaveSettingsAction::CloseFactoryPicker => {
                state.picker_open = false;
            }
            SaveSettingsAction::ToggleFactoryBlock(kind) => {
                if let Some(index) = state
                    .data
                    .factory_block_filter
                    .kinds
                    .iter()
                    .position(|candidate| *candidate == kind)
                {
                    state.data.factory_block_filter.kinds.remove(index);
                } else {
                    state.data.factory_block_filter.kinds.push(kind);
                }
                solution_state.factory_block_filter = Some(state.data.factory_block_filter.clone());
                persist_settings(&mut state, &mut lighting);
            }
            SaveSettingsAction::UploadSkybox => {
                let Some(slot) = state.slot.clone() else {
                    continue;
                };
                ui.open_text_prompt_then(
                    crate::game::ui::core::text_prompt::TextPromptProps {
                        title: i18n.t("save_settings.skybox_upload"),
                        default_value: String::new(),
                        save_text: i18n.t("button.confirm"),
                        cancel_text: i18n.t("button.cancel"),
                        max_characters: None,
                    },
                    move |result, world| {
                        let crate::game::ui::core::text_prompt::TextPromptResult::Saved(path) =
                            result
                        else {
                            return;
                        };
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Ok(bytes) = std::fs::read(path.trim()) {
                            if crate::shared::save::write_save_skybox(&slot, &bytes) {
                                world.resource_mut::<SaveSettingsUiState>().skybox_bytes =
                                    Some(bytes);
                            }
                        }
                    },
                );
            }
        }
    }
}
