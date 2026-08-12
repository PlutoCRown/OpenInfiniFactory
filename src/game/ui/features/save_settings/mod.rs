mod actions;
pub mod types;

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageFormat, ImageSampler, ImageType};
use bevy::prelude::*;

pub use actions::{dispatch_save_settings_actions, emit_save_settings_actions};
pub use types::SaveSettingsUiState;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::{UiAccessScope, UiMainThread, i18n};
use crate::game::ui::core::runtime::UiNavigation;
use crate::game::ui::screens::{
    SaveSettingsBlockIcon, SaveSettingsCheckMark, SaveSettingsCrossMark, SaveSettingsFactoryPicker,
    SaveSettingsFilterMark, SaveSettingsSkyboxPreview, SaveSettingsValueText,
};
use crate::game::world::rendering::BlockIconAssets;

/// 存档设置页功能插件。
pub struct SaveSettingsPlugin;

impl Plugin for SaveSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveSettingsUiState>()
            .add_observer(emit_save_settings_actions)
            .add_systems(
                Update,
                (dispatch_save_settings_actions, update_save_settings_ui)
                    .chain()
                    .in_set(UiAccessScope)
                    .after(PerfScope::Placement)
                    .before(PerfScope::Menus),
            );
    }
}

pub fn update_save_settings_ui(
    _ui_thread: UiMainThread,
    runtime: Res<UiNavigation>,
    state: Res<SaveSettingsUiState>,
    block_icons: Option<Res<BlockIconAssets>>,
    mut preview: Query<&mut ImageNode, With<SaveSettingsSkyboxPreview>>,
    mut images: ResMut<Assets<Image>>,
    mut values: Query<(&SaveSettingsValueText, &mut Text)>,
    mut block_images: Query<
        (&SaveSettingsBlockIcon, &mut ImageNode),
        Without<SaveSettingsSkyboxPreview>,
    >,
    mut picker: Query<&mut Node, With<SaveSettingsFactoryPicker>>,
    mut marks: Query<
        (&SaveSettingsFilterMark, &Children, &mut Visibility),
        (
            Without<SaveSettingsCheckMark>,
            Without<SaveSettingsCrossMark>,
        ),
    >,
    mut check_marks: Query<
        &mut Visibility,
        (
            With<SaveSettingsCheckMark>,
            Without<SaveSettingsFilterMark>,
            Without<SaveSettingsCrossMark>,
        ),
    >,
    mut cross_marks: Query<
        &mut Visibility,
        (
            With<SaveSettingsCrossMark>,
            Without<SaveSettingsFilterMark>,
            Without<SaveSettingsCheckMark>,
        ),
    >,
    added_preview: Query<(), Added<SaveSettingsSkyboxPreview>>,
) {
    if !runtime.is_save_settings_open() {
        return;
    }
    if state.is_changed() || !added_preview.is_empty() {
        for mut image_node in &mut preview {
            *image_node = state
                .skybox_bytes
                .as_deref()
                .and_then(|bytes| {
                    Image::from_buffer(
                        bytes,
                        ImageType::Format(ImageFormat::Png),
                        CompressedImageFormats::NONE,
                        true,
                        ImageSampler::Default,
                        RenderAssetUsages::default(),
                    )
                    .ok()
                    .map(|image| ImageNode::new(images.add(image)))
                })
                .unwrap_or_default();
        }
    }
    for mut node in &mut picker {
        node.display = if state.picker_open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if state.is_changed() || block_icons.as_ref().is_some_and(|icons| icons.is_changed()) {
        for (marker, mut image_node) in &mut block_images {
            *image_node = block_icons
                .as_deref()
                .and_then(|icons| icons.get(marker.0))
                .map(ImageNode::new)
                .unwrap_or_default();
        }
        for (marker, children, mut visibility) in &mut marks {
            let selected = state.data.factory_block_filter.kinds.contains(&marker.0);
            *visibility = if selected {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            let show_check = selected
                && state.data.factory_block_filter.mode
                    == crate::shared::save::FactoryBlockFilterMode::Whitelist;
            for child in children.iter() {
                if let Ok(mut mark_visibility) = check_marks.get_mut(child) {
                    *mark_visibility = if show_check {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
                if let Ok(mut mark_visibility) = cross_marks.get_mut(child) {
                    *mark_visibility = if show_check {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    };
                }
            }
        }
        for (marker, mut text) in &mut values {
            text.0 = match marker {
                SaveSettingsValueText::LightPosition => format_vec3(state.data.light_position),
                SaveSettingsValueText::LightDirection => format_vec3(state.data.light_direction),
                SaveSettingsValueText::LightIntensity => {
                    format!("{:.2}", state.data.light_intensity)
                }
                SaveSettingsValueText::SolutionSpawn => state
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
                SaveSettingsValueText::FilterMode => match state.data.factory_block_filter.mode {
                    crate::shared::save::FactoryBlockFilterMode::Blacklist => {
                        i18n.t("save_settings.blacklist")
                    }
                    crate::shared::save::FactoryBlockFilterMode::Whitelist => {
                        i18n.t("save_settings.whitelist")
                    }
                },
            };
        }
    }
}

fn format_vec3(value: Option<Vec3>) -> String {
    value
        .map(|value| format!("{:.3}, {:.3}, {:.3}", value.x, value.y, value.z))
        .unwrap_or_default()
}
