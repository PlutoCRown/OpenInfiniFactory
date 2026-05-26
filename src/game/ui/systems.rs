use bevy::prelude::*;

use crate::game::state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use crate::shared::config::{ConfigAction, GameConfig};
use crate::shared::i18n::I18n;
use crate::shared::save::SaveState;

use super::types::{
    BackpackPanel, CarriedItem, CarriedLabel, Crosshair, CurrentSaveText, DeleteSelectionModeText,
    FovText, HotbarText, InventoryItems, InventorySlot, InventoryTitle, KeyBindingButton,
    KeyBindingLabel, LanguageText, LocalizedText, MainMenuPanel, PausePanel, PendingKeyBind,
    PlaceSelectionModeText, SaveListAction, SaveListLabel, SaveListPanel, SaveListTitle,
    SettingsGameplayGroup, SettingsKeyBindingsGroup, SettingsPanel, SettingsStatusText,
    SettingsTab, SimulationText, SlotArea, SlotLabel, UiScaleText,
};
use super::widgets::{short_item_name, slot_color};

#[derive(Resource, Clone)]
pub struct UiFont(pub Handle<Font>);

pub fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFont(asset_server.load("fonts/PingFangSC-Regular.ttf")));
}

pub fn apply_ui_font(ui_font: Option<Res<UiFont>>, mut text_query: Query<&mut Text, Added<Text>>) {
    let Some(ui_font) = ui_font else {
        return;
    };

    for mut text in &mut text_query {
        for section in &mut text.sections {
            section.style.font = ui_font.0.clone();
        }
    }
}

pub fn inventory_slot_clicks(
    mut interaction_query: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>),
    >,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mode: Res<GameMode>,
) {
    if *mode != GameMode::Inventory {
        return;
    }

    for (interaction, slot) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let slot_item = match slot.area {
            SlotArea::Hotbar => &mut inventory.hotbar[slot.index],
            SlotArea::Backpack => &mut inventory.backpack[slot.index],
        };
        std::mem::swap(slot_item, carried.item_mut());
        placement.selection.clear();
        placement.edit_gesture = None;
    }
}

pub fn update_status_ui(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    settings: Res<GameSettings>,
    save_state: Res<SaveState>,
    carried: Res<CarriedItem>,
    i18n: Res<I18n>,
    mut hotbar: Query<&mut Text, (With<HotbarText>, Without<SlotLabel>, Without<CarriedLabel>)>,
    mut inventory_title: Query<
        &mut Text,
        (
            With<InventoryTitle>,
            Without<HotbarText>,
            Without<SlotLabel>,
            Without<CarriedLabel>,
            Without<FovText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut carried_label: Query<
        &mut Text,
        (
            With<CarriedLabel>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut fov_text: Query<
        &mut Text,
        (
            With<FovText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<UiScaleText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut ui_scale_text: Query<
        &mut Text,
        (
            With<UiScaleText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut simulation_text: Query<
        &mut Text,
        (
            With<SimulationText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<UiScaleText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut current_save_text: Query<
        &mut Text,
        (
            With<CurrentSaveText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<SimulationText>,
        ),
    >,
) {
    if let Ok(mut text) = hotbar.get_single_mut() {
        let selected = inventory.hotbar[placement.selected]
            .map(|kind| i18n.text(kind.name_key()))
            .unwrap_or_else(|| i18n.text("empty"));
        text.sections[0].value = i18n.fmt(
            "status.hotbar",
            &[
                ("mode", builder_mode_name(*builder_mode, &i18n)),
                ("selected", selected),
                ("facing", i18n.text(placement.facing.name_key())),
            ],
        );
    }

    if let Ok(mut text) = inventory_title.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "inventory.title",
            &[("mode", builder_mode_name(*builder_mode, &i18n))],
        );
    }

    if let Ok(mut text) = carried_label.get_single_mut() {
        text.sections[0].value = carried
            .item()
            .map(|kind| i18n.fmt("inventory.holding", &[("item", i18n.text(kind.name_key()))]))
            .unwrap_or_default();
    }

    if let Ok(mut text) = fov_text.get_single_mut() {
        text.sections[0].value = format!("FOV {:.0}", settings.fov_degrees);
    }

    if let Ok(mut text) = ui_scale_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "settings.ui_scale",
            &[("scale", format!("{:.1}", settings.ui_scale))],
        );
    }

    if let Ok(mut text) = simulation_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "status.simulation",
            &[
                ("mode", builder_mode_name(*builder_mode, &i18n)),
                ("turns", simulation.turn.to_string()),
                (
                    "state",
                    i18n.text(if simulation.running {
                        "state.playing"
                    } else {
                        "state.paused"
                    }),
                ),
                ("speed", format!("{:.0}", simulation.speed)),
            ],
        );
    }

    if let Ok(mut text) = current_save_text.get_single_mut() {
        text.sections[0].value = save_state
            .current
            .as_ref()
            .map(|name| i18n.fmt("save.world", &[("name", name.clone())]))
            .unwrap_or_else(|| i18n.text("save.no_world_loaded"));
    }
}

fn builder_mode_name(mode: BuilderMode, i18n: &I18n) -> String {
    match mode {
        BuilderMode::Edit => i18n.text("mode.edit"),
        BuilderMode::Play => i18n.text("mode.play"),
    }
}

pub fn update_settings_status_ui(
    config: Res<GameConfig>,
    settings_tab: Res<SettingsTab>,
    pending_key_bind: Res<PendingKeyBind>,
    i18n: Res<I18n>,
    mut settings_status: Query<
        &mut Text,
        (
            With<SettingsStatusText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<FovText>,
            Without<PlaceSelectionModeText>,
            Without<DeleteSelectionModeText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
            Without<KeyBindingLabel>,
        ),
    >,
    mut key_labels: Query<
        (&Parent, &mut Text),
        (With<KeyBindingLabel>, Without<SettingsStatusText>),
    >,
    mut place_mode_text: Query<
        &mut Text,
        (
            With<PlaceSelectionModeText>,
            Without<SettingsStatusText>,
            Without<KeyBindingLabel>,
            Without<DeleteSelectionModeText>,
        ),
    >,
    mut delete_mode_text: Query<
        &mut Text,
        (
            With<DeleteSelectionModeText>,
            Without<SettingsStatusText>,
            Without<KeyBindingLabel>,
            Without<PlaceSelectionModeText>,
        ),
    >,
    key_buttons: Query<&KeyBindingButton>,
) {
    if let Ok(mut text) = settings_status.get_single_mut() {
        let tab_name = match *settings_tab {
            SettingsTab::Gameplay => i18n.text("tab.gameplay"),
            SettingsTab::KeyBindings => i18n.text("tab.key_bindings"),
        };
        let pending = pending_key_bind
            .0
            .map(|action| {
                i18n.fmt(
                    "settings.pending_key",
                    &[("action", i18n.text(action.label_key()))],
                )
            })
            .unwrap_or_else(|| i18n.text("settings.rebind_hint"));
        text.sections[0].value = i18n.fmt(
            "settings.status",
            &[
                ("tab", tab_name),
                ("pending", pending),
                (
                    "simulate",
                    config.key(ConfigAction::Simulate).name().to_string(),
                ),
                (
                    "rollback",
                    config
                        .key(ConfigAction::RotateOrRollback)
                        .name()
                        .to_string(),
                ),
                (
                    "inventory",
                    config.key(ConfigAction::Inventory).name().to_string(),
                ),
                ("pause", config.key(ConfigAction::Pause).name().to_string()),
            ],
        );
    }

    for (parent, mut text) in &mut key_labels {
        let Ok(button) = key_buttons.get(parent.get()) else {
            continue;
        };
        let suffix = pending_key_bind
            .0
            .filter(|pending| *pending == button.0)
            .map(|_| "...")
            .unwrap_or(config.key(button.0).name());
        text.sections[0].value = format!("{}: {suffix}", i18n.text(button.0.label_key()));
    }

    if let Ok(mut text) = place_mode_text.get_single_mut() {
        text.sections[0].value = i18n.text(config.place_selection_mode.label_key());
    }

    if let Ok(mut text) = delete_mode_text.get_single_mut() {
        text.sections[0].value = i18n.text(config.delete_selection_mode.label_key());
    }
}

pub fn update_localized_ui(
    i18n: Res<I18n>,
    mut localized_text: Query<(&LocalizedText, &mut Text), Without<LanguageText>>,
    mut language_text: Query<&mut Text, With<LanguageText>>,
) {
    if !i18n.is_changed() {
        return;
    }

    for (localized, mut text) in &mut localized_text {
        text.sections[0].value = i18n.text(localized.key);
    }

    for mut text in &mut language_text {
        text.sections[0].value = i18n.fmt(
            "button.language",
            &[("language", i18n.language().native_name().to_string())],
        );
    }
}

pub fn update_panel_visibility(
    mode: Res<GameMode>,
    settings_tab: Res<SettingsTab>,
    mut style_sets: ParamSet<(
        Query<&mut Style, With<MainMenuPanel>>,
        Query<&mut Style, With<SaveListPanel>>,
        Query<&mut Style, With<SettingsPanel>>,
        Query<&mut Style, With<SettingsGameplayGroup>>,
        Query<&mut Style, With<SettingsKeyBindingsGroup>>,
        Query<&mut Style, With<BackpackPanel>>,
        Query<&mut Style, With<PausePanel>>,
    )>,
    mut crosshair: Query<&mut Visibility, With<Crosshair>>,
) {
    for mut style in &mut style_sets.p0() {
        style.display = if *mode == GameMode::MainMenu {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p1() {
        style.display = if matches!(*mode, GameMode::SaveListMain | GameMode::SaveListPause) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p2() {
        style.display = if *mode == GameMode::Settings {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p3() {
        style.display = if *mode == GameMode::Settings && *settings_tab == SettingsTab::Gameplay {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p4() {
        style.display = if *mode == GameMode::Settings && *settings_tab == SettingsTab::KeyBindings
        {
            Display::Grid
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p5() {
        style.display = if *mode == GameMode::Inventory {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p6() {
        style.display = if *mode == GameMode::Paused {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut visibility in &mut crosshair {
        *visibility = if *mode == GameMode::Playing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn update_inventory_slots(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    i18n: Res<I18n>,
    mut slot_query: Query<
        (
            &InventorySlot,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut labels: Query<&mut Text, (With<SlotLabel>, Without<HotbarText>, Without<CarriedLabel>)>,
) {
    for (slot, children, mut background, mut border) in &mut slot_query {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };

        let selected_hotbar = slot.area == SlotArea::Hotbar && slot.index == placement.selected;
        *background = item
            .map(slot_color)
            .unwrap_or(Color::srgba(0.16, 0.16, 0.17, 0.92))
            .into();
        *border = if selected_hotbar {
            Color::srgb(1.0, 1.0, 1.0).into()
        } else {
            Color::srgb(0.22, 0.22, 0.22).into()
        };

        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(*child) {
                text.sections[0].value = item
                    .map(|kind| i18n.text(short_item_name(kind)))
                    .unwrap_or_default();
            }
        }
    }
}

pub fn update_save_list_ui(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    i18n: Res<I18n>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<SaveListTitle>>,
        Query<&mut Text, With<SaveListLabel>>,
    )>,
    mut slots: Query<(&SaveListAction, &Children, &mut BackgroundColor), With<Button>>,
) {
    if let Ok(mut title) = text_sets.p0().get_single_mut() {
        title.sections[0].value = match *mode {
            GameMode::SaveListMain => i18n.text("save.title.main"),
            GameMode::SaveListPause => i18n.text("save.title.pause"),
            _ => i18n.text("save.title.default"),
        };
    }

    for (action, children, mut background) in &mut slots {
        let label = match *action {
            SaveListAction::Load(index) => save_state
                .slots
                .get(index)
                .map(|name| i18n.fmt("save.load", &[("name", name.clone())]))
                .unwrap_or_else(|| i18n.text("empty_slot")),
            SaveListAction::Back => i18n.text("button.back"),
        };

        let enabled_load = match *action {
            SaveListAction::Load(index) => save_state.slots.get(index).is_some(),
            SaveListAction::Back => true,
        };
        *background = if enabled_load {
            Color::srgba(0.22, 0.24, 0.26, 0.96).into()
        } else {
            Color::srgba(0.12, 0.12, 0.13, 0.82).into()
        };

        for child in children.iter() {
            if let Ok(mut text) = text_sets.p1().get_mut(*child) {
                text.sections[0].value = label.clone();
            }
        }
    }
}
