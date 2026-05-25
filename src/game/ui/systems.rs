use bevy::prelude::*;

use crate::game::state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use crate::game::world::blocks::BlockKind;
use crate::shared::config::{ConfigAction, GameConfig};
use crate::shared::save::SaveState;

use super::types::{
    BackpackPanel, CarriedItem, CarriedLabel, Crosshair, CurrentSaveText, FovText, HotbarText,
    InventoryItems, InventorySlot, InventoryTitle, KeyBindingButton, KeyBindingLabel,
    MainMenuPanel, PausePanel, PendingKeyBind, SaveListAction, SaveListLabel, SaveListPanel,
    SaveListTitle, SettingsGameplayGroup, SettingsKeyBindingsGroup, SettingsPanel,
    SettingsStatusText, SettingsTab, SimulationText, SlotArea, SlotLabel,
};
use super::widgets::{short_item_name, slot_color};

pub fn inventory_slot_clicks(
    mut interaction_query: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>),
    >,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
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
            .map(BlockKind::name)
            .unwrap_or("Empty");
        text.sections[0].value = format!(
            "Mode: {}   Selected: {}   Facing: {}   Inventory   Pause",
            builder_mode_name(*builder_mode),
            selected,
            placement.facing.name()
        );
    }

    if let Ok(mut text) = inventory_title.get_single_mut() {
        text.sections[0].value = format!("Inventory - {} Mode", builder_mode_name(*builder_mode));
    }

    if let Ok(mut text) = carried_label.get_single_mut() {
        text.sections[0].value = carried
            .item()
            .map(|kind| format!("Holding: {}", kind.name()))
            .unwrap_or_default();
    }

    if let Ok(mut text) = fov_text.get_single_mut() {
        text.sections[0].value = format!("FOV {:.0}", settings.fov_degrees);
    }

    if let Ok(mut text) = simulation_text.get_single_mut() {
        text.sections[0].value = format!(
            "Mode: {}\nTurns: {}\nSim: {} x{:.0}",
            builder_mode_name(*builder_mode),
            simulation.turn,
            if simulation.running {
                "Playing"
            } else {
                "Paused"
            },
            simulation.speed
        );
    }

    if let Ok(mut text) = current_save_text.get_single_mut() {
        text.sections[0].value = save_state
            .current
            .as_ref()
            .map(|name| format!("World: {name}"))
            .unwrap_or_else(|| "No world loaded".to_string());
    }
}

fn builder_mode_name(mode: BuilderMode) -> &'static str {
    match mode {
        BuilderMode::Edit => "Edit",
        BuilderMode::Play => "Play",
    }
}

pub fn update_settings_status_ui(
    config: Res<GameConfig>,
    settings_tab: Res<SettingsTab>,
    pending_key_bind: Res<PendingKeyBind>,
    mut settings_status: Query<
        &mut Text,
        (
            With<SettingsStatusText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<FovText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
            Without<KeyBindingLabel>,
        ),
    >,
    mut key_labels: Query<
        (&Parent, &mut Text),
        (With<KeyBindingLabel>, Without<SettingsStatusText>),
    >,
    key_buttons: Query<&KeyBindingButton>,
) {
    if let Ok(mut text) = settings_status.get_single_mut() {
        let tab_name = match *settings_tab {
            SettingsTab::Gameplay => "Gameplay",
            SettingsTab::KeyBindings => "Key Bindings",
        };
        let pending = pending_key_bind
            .0
            .map(|action| format!("Press a key for {}", action.label()))
            .unwrap_or_else(|| "Click a key binding to rebind it.".to_string());
        text.sections[0].value = format!(
            "{tab_name} | Config: saves/config.ron\n{pending}\nSim: {}  Rollback: {}  Inventory: {}  Pause: {}",
            config.key(ConfigAction::Simulate).name(),
            config.key(ConfigAction::RotateOrRollback).name(),
            config.key(ConfigAction::Inventory).name(),
            config.key(ConfigAction::Pause).name()
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
        text.sections[0].value = format!("{}: {suffix}", button.0.label());
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
                    .map(|kind| short_item_name(kind).to_string())
                    .unwrap_or_default();
            }
        }
    }
}

pub fn update_save_list_ui(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<SaveListTitle>>,
        Query<&mut Text, With<SaveListLabel>>,
    )>,
    mut slots: Query<(&SaveListAction, &Children, &mut BackgroundColor), With<Button>>,
) {
    if let Ok(mut title) = text_sets.p0().get_single_mut() {
        title.sections[0].value = match *mode {
            GameMode::SaveListMain => "Load Save".to_string(),
            GameMode::SaveListPause => "Switch Save".to_string(),
            _ => "Saves".to_string(),
        };
    }

    for (action, children, mut background) in &mut slots {
        let label = match *action {
            SaveListAction::Load(index) => save_state
                .slots
                .get(index)
                .map(|name| format!("Load {name}"))
                .unwrap_or_else(|| "Empty Slot".to_string()),
            SaveListAction::Back => "Back".to_string(),
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
