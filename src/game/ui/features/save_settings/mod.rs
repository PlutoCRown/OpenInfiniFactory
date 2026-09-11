mod actions;
pub mod types;

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageFormat, ImageSampler, ImageType};
use bevy::prelude::*;

pub use actions::{dispatch_save_settings_actions, emit_save_settings_actions};
pub use types::{SaveSettingsSkyboxPreviewCache, SaveSettingsUiState};

use crate::game::ui::access::{UiContext, i18n};
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
        app.add_message::<crate::game::ui::core::host::UiAction<types::SaveSettingsAction>>();
        app.init_resource::<SaveSettingsUiState>()
            .init_resource::<SaveSettingsSkyboxPreviewCache>()
            .add_observer(emit_save_settings_actions)
            .add_systems(
                Update,
                (dispatch_save_settings_actions, update_save_settings_ui)
                    .chain()
                    .in_set(crate::game::schedule::GameSet::Menus),
            );
    }
}

pub fn update_save_settings_ui(
    ui_context: UiContext,
    runtime: Res<UiNavigation>,
    state: Res<SaveSettingsUiState>,
    mut preview_cache: ResMut<SaveSettingsSkyboxPreviewCache>,
    mut last_filter: Local<Option<crate::shared::save::FactoryBlockFilter>>,
    block_icons: Option<Res<BlockIconAssets>>,
    mut preview: Query<&mut ImageNode, With<SaveSettingsSkyboxPreview>>,
    mut images: ResMut<Assets<Image>>,
    mut values: Query<(&SaveSettingsValueText, &mut Text)>,
    mut block_images: Query<
        (Ref<SaveSettingsBlockIcon>, &mut ImageNode),
        Without<SaveSettingsSkyboxPreview>,
    >,
    mut picker: Query<&mut Node, With<SaveSettingsFactoryPicker>>,
    mut marks: Query<
        (Ref<SaveSettingsFilterMark>, &Children, &mut Visibility),
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
    let locale_changed = ui_context.locale_changed();
    let _ui_scope = ui_context.enter();
    if !runtime.is_save_settings_open() {
        return;
    }
    let skybox_changed = match (&preview_cache.source, &state.skybox_bytes) {
        (Some(cached), Some(current)) => !std::sync::Arc::ptr_eq(cached, current),
        (None, None) => false,
        _ => true,
    };
    if skybox_changed {
        preview_cache.source.clone_from(&state.skybox_bytes);
        preview_cache.handle = state.skybox_bytes.as_deref().and_then(|bytes| {
            Image::from_buffer(
                bytes,
                ImageType::Format(ImageFormat::Png),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::Default,
                RenderAssetUsages::default(),
            )
            .ok()
            .map(|image| images.add(image))
        });
    }
    if preview_cache.is_changed() || !added_preview.is_empty() {
        for mut image_node in &mut preview {
            let next = preview_cache
                .handle
                .clone()
                .map(ImageNode::new)
                .unwrap_or_default();
            *image_node = next;
        }
    }
    for mut node in &mut picker {
        let display = if state.picker_open {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let icons_changed = block_icons.as_ref().is_some_and(|icons| icons.is_changed());
    for (marker, mut image_node) in &mut block_images {
        if icons_changed || marker.is_added() {
            let next = block_icons
                .as_deref()
                .and_then(|icons| icons.get(marker.0))
                .map(ImageNode::new)
                .unwrap_or_default();
            *image_node = next;
        }
    }
    let filter_changed = last_filter.as_ref() != Some(&state.data.factory_block_filter);
    if filter_changed {
        *last_filter = Some(state.data.factory_block_filter.clone());
    }
    for (marker, children, mut visibility) in &mut marks {
        if filter_changed || marker.is_added() {
            let selected = state.data.factory_block_filter.kinds.contains(&marker.0);
            visibility.set_if_neq(if selected {
                Visibility::Visible
            } else {
                Visibility::Hidden
            });
            let show_check = selected
                && state.data.factory_block_filter.mode
                    == crate::shared::save::FactoryBlockFilterMode::Whitelist;
            for child in children.iter() {
                if let Ok(mut mark_visibility) = check_marks.get_mut(child) {
                    mark_visibility.set_if_neq(if show_check {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    });
                }
                if let Ok(mut mark_visibility) = cross_marks.get_mut(child) {
                    mark_visibility.set_if_neq(if show_check {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    });
                }
            }
        }
    }
    let state_changed = state.is_changed();
    if state_changed || locale_changed {
        for (marker, mut text) in &mut values {
            let next = match marker {
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
            if text.0 != next {
                text.0 = next;
            }
        }
    }
}

fn format_vec3(value: Option<Vec3>) -> String {
    value
        .map(|value| format!("{:.3}, {:.3}, {:.3}", value.x, value.y, value.z))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::UiPanelId;
    use crate::game::ui::core::runtime::UiPanelContext;

    /// 打开选择器只改变 UI 状态，不会再次解码同一天空盒。
    #[test]
    fn picker_change_reuses_skybox_handle() {
        let mut navigation = UiNavigation::default();
        navigation.open(UiPanelId::Settings, UiPanelContext::SaveSettingsFromPause);
        let png: &[u8] = &[
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 4, 0, 0, 0, 181, 28, 12, 2, 0, 0, 0, 11, 73, 68, 65, 84, 120, 218, 99, 100, 248, 15,
            0, 1, 5, 1, 1, 39, 24, 227, 102, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ];
        let mut app = App::new();
        app.insert_resource(navigation)
            .insert_resource(crate::shared::i18n::I18n::new(
                crate::shared::i18n::DEFAULT_LANGUAGE,
            ))
            .insert_resource(SaveSettingsUiState {
                skybox_bytes: Some(png.into()),
                ..default()
            })
            .init_resource::<SaveSettingsSkyboxPreviewCache>()
            .init_resource::<Assets<Image>>()
            .add_systems(Update, update_save_settings_ui);
        app.world_mut()
            .spawn((SaveSettingsSkyboxPreview, ImageNode::default()));
        app.update();
        let first = app
            .world()
            .resource::<SaveSettingsSkyboxPreviewCache>()
            .handle
            .clone();
        app.world_mut()
            .resource_mut::<SaveSettingsUiState>()
            .picker_open = true;
        app.update();
        let second = app
            .world()
            .resource::<SaveSettingsSkyboxPreviewCache>()
            .handle
            .clone();
        assert_eq!(first, second);
        assert!(first.is_some());
    }
}
