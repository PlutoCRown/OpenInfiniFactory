# Goal: Bevy 0.19 + BSN UI Migration Plan

Status: complete for the current `0.19.0-rc.2` migration target. Bevy 0.19 RC upgrade, BSN-compatible UI helpers, registry/lazy-spawn panels, reactive dirty-message refresh, migration examples, and verification gates have been implemented.

## Request

- Upgrade this project from Bevy `0.18.1` to Bevy `0.19`.
- Enable the BSN / B-Scene feature once its crate feature name and public API are confirmed.
- Replace the current mostly pre-spawned, update-polled UI structure with a more extensible panel system.
- Support reactive opening from click and keyboard events.
- Support dynamic panel registration so feature modules and blocks can contribute panels without wiring every panel into the startup UI tree.
- Provide a migration template / example.
- Scan the project and record all UI panels that need migration.

## Current Bevy State

Current dependency:

```toml
bevy = { version = "0.19.0-rc.2", features = ["bevy_ui_widgets"] }
```

As of 2026-06-01, `bevy` latest stable on docs.rs is `0.18.1`; this project now targets `0.19.0-rc.2`, which exists as a pre-release. Bevy's own release process describes release candidates as test windows before final release, and Bevy migration docs explicitly warn that Bevy is still experimental and each release has breaking changes.

Implication: the migration currently targets `0.19.0-rc.2`. BSN API names, crate features, and migration guide details must be rechecked before doing a real BSN syntax conversion.

Useful references checked:

- https://docs.rs/crate/bevy/latest
- https://bevy.org/learn/contribute/project-information/release-process/
- https://bevy.org/learn/migration-guides/introduction/
- https://bevy.org/news/bevys-fifth-birthday/
- https://github.com/bevyengine/bevy/pull/20158

## Current UI Architecture Findings

The project already uses Bevy observers for many interactions:

- `GameUiPlugin` registers observers for block edits, menu actions, panel drag/close, hover, button press/release, settings actions, inventory clicks, and confirm/text prompt actions.
- Pointer events are already handled with `On<Pointer<Click>>`, `On<Pointer<Drag>>`, `On<Pointer<Over>>`, etc.
- Keyboard input is still handled by systems reading `ButtonInput<KeyCode>` or `MessageReader<KeyboardInput>`.

The main structural issue is not lack of observers; it is that most UI is spawned eagerly at startup and then hidden/refreshed by global `update_*_ui` systems:

- `setup_ui` spawns almost every screen and panel immediately.
- `UiPanelId` is a fixed enum in `src/game/state.rs`.
- `UiRuntime` stores a stack of fixed `UiPanelSession` values.
- `PanelVisibility` and `UiPanelBinding` drive display state by polling resources.
- Block panels are hand-wired through `spawn_block_panels`, `spawn_block_dropdown_layers`, `update_*_ui`, and `BlockKind::ui_panel()`.

This makes UI extension difficult because new panels require enum changes, startup spawn wiring, update-system wiring, visibility logic, action enum variants, and often dropdown/update glue.

## Panels / UI Surfaces To Migrate

Core screens and overlays:

- Main menu: `src/game/ui/screens/main_menu/mod.rs`, `actions.rs`, `widgets.rs`
- Save list: `src/game/ui/screens/save_list/mod.rs`, `actions.rs`, `systems.rs`
- Pause menu: `src/game/ui/screens/pause_menu/mod.rs`, `actions.rs`, `widgets.rs`
- Settings panel: `src/game/ui/screens/settings/mod.rs`, `actions.rs`, `systems.rs`, `widgets.rs`
- Inventory panel / hotbar / tooltip / carried item: `src/game/ui/screens/inventory/mod.rs`, `actions.rs`, `render.rs`, `widgets.rs`, `carried_item.rs`
- Confirm dialog: `src/game/ui/screens/confirm_dialog/mod.rs`, `actions.rs`
- Text prompt: implemented under save list modules, especially `src/game/ui/screens/save_list/mod.rs`, `actions.rs`, `systems.rs`
- Status overlays / crosshair / HUD: `src/game/ui/layout.rs`, `src/game/ui/systems/status.rs`, `src/game/ui/systems/hud.rs`
- Virtual controls overlay: `src/game/systems/virtual_controls.rs`
- Debug performance UI: `src/game/systems/debug.rs`

Block edit panels:

- Generator panel: `src/game/world/blocks/generator/ui.rs`, `src/game/world/blocks/panel_systems.rs`
- Goal panel: `src/game/world/blocks/goal/ui.rs`, `src/game/world/blocks/panel_systems.rs`
- Labeler panel: `src/game/world/blocks/labeler/ui.rs`, `src/game/world/blocks/panel_systems.rs`
- Stamper panel: currently reuses `UiPanelId::Labeler` via `src/game/world/blocks/stamper/ui.rs`
- Roller panel: currently reuses `UiPanelId::Labeler` via `src/game/world/blocks/roller/ui.rs`
- Converter panel: `src/game/world/blocks/converter/ui.rs`, `src/game/world/blocks/panel_systems.rs`
- Teleport panel: `src/game/world/blocks/teleport_entrance/ui.rs`, `actions.rs`, plus `src/game/world/blocks/teleport_exit/ui.rs`

Shared UI infrastructure:

- UI setup root: `src/game/ui/layout.rs`
- UI plugin wiring: `src/game/ui/mod.rs`
- Runtime state and marker components: `src/game/ui/types.rs`
- Panel visibility, dragging, close, z layers: `src/game/ui/systems/panels.rs`
- Localization updater: `src/game/ui/systems/localized.rs`
- Component builders: `src/game/ui/components/*.rs`
- Block panel helpers: `src/game/world/blocks/panel_layout.rs`, `ui_components.rs`, `panel_systems.rs`
- Block registry and `BlockKind` UI hooks: `src/game/world/blocks/registry.rs`, `src/game/world/blocks.rs`, block `mod.rs` files

Approximate migration count:

- 7 main/modal screens
- 3 HUD/overlay surfaces
- 7 block-edit panel variants
- 6 shared UI infrastructure modules

## Target Architecture

Introduce a dynamic panel registry resource instead of a fixed `UiPanelId` enum as the central extension point.

Suggested types:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiPanelKey(pub &'static str);

pub const PANEL_SETTINGS: UiPanelKey = UiPanelKey("core.settings");
pub const PANEL_GENERATOR: UiPanelKey = UiPanelKey("block.generator");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPanelContext {
    None,
    ReturnTo(GameMode),
    Block { pos: IVec3 },
}

pub struct UiPanelDescriptor {
    pub key: UiPanelKey,
    pub title_key: &'static str,
    pub blocks_gameplay: bool,
    pub spawn: fn(&mut ChildSpawnerCommands, &I18n, UiPanelContext),
    pub refresh: Option<fn(&mut World, Entity, UiPanelContext)>,
}

#[derive(Resource, Default)]
pub struct UiPanelRegistry {
    panels: HashMap<UiPanelKey, UiPanelDescriptor>,
}
```

Opening panels should be event-driven:

```rust
#[derive(Event)]
pub struct OpenUiPanel {
    pub key: UiPanelKey,
    pub context: UiPanelContext,
}

#[derive(Event)]
pub struct CloseUiPanel {
    pub key: Option<UiPanelKey>,
}
```

Expected flow:

1. A click observer or keyboard system emits `OpenUiPanel`.
2. A central panel host receives the event.
3. The host looks up the descriptor in `UiPanelRegistry`.
4. If the panel entity does not exist, it is spawned from the descriptor.
5. If it exists, context is updated and the panel is shown / focused.
6. Refresh is triggered only on open, context change, i18n change, or relevant domain events, not every frame.

## BSN Template Direction

The Bevy 0.19 RC exposes BSN through `bevy_scene::{bsn, bsn_list}` and scene spawning through `EntityCommandsSceneExt::queue_spawn_related_scenes`. The project now depends on `bevy_scene = "0.19.0-rc.2"` explicitly so UI code can use those APIs directly.

Current project style for BSN panel content:

```rust
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

fn spawn_demo_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(320.0, "demo.title").closable(),
        UiPanelBinding(PANEL_DEMO),
        |panel| {
            panel
                .spawn_empty()
                .queue_spawn_related_scenes::<Children>(demo_panel_scene());
        },
    )
}

fn demo_panel_scene() -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text("Hello panel")
            TextFont {
                font_size: FontSize::Px(24.0)
            }
            TextColor(Color::srgb(0.90, 0.84, 0.76))
        )
    }
}
```

`src/game/ui/demo_panel.rs` is the first compiled BSN migration example. Larger panels should move content sections to `fn ..._scene() -> impl bevy_scene::SceneList` incrementally, while keeping the existing `spawn_panel` wrapper for panel window behavior, title bars, dragging, close buttons, visibility, and localization until those helpers are also migrated.

Keyboard / click opening template:

```rust
pub fn settings_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: EventWriter<OpenUiPanel>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        open.write(OpenUiPanel {
            key: PANEL_SETTINGS,
            context: UiPanelContext::ReturnTo(GameMode::Playing),
        });
    }
}

pub fn open_block_panel_on_selection(
    mut selected: EventReader<BlockSelected>,
    world: Res<WorldBlocks>,
    mut open: EventWriter<OpenUiPanel>,
) {
    for event in selected.read() {
        if let Some(key) = world.system_blocks
            .get(&event.pos)
            .and_then(|block| block.kind.ui_panel_key())
        {
            open.write(OpenUiPanel {
                key,
                context: UiPanelContext::Block { pos: event.pos },
            });
        }
    }
}
```

## Target Style Examples

These examples are the canonical caller-facing style after the registry migration. New panels should be registered once, opened by emitting `OpenUiPanel`, or opened from buttons by attaching `OpensPanel`; callers should not manually spawn the panel or edit central visibility logic.

Minimal keyboard shortcut example. Pressing `T` opens the panel:

```rust
fn open_test_panel_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: MessageWriter<OpenUiPanel>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        open.write(OpenUiPanel::new(PANEL_TEST, UiPanelContext::None));
    }
}
```

Minimal mouse button example. Any spawned button can opt into panel opening by attaching `OpensPanel`:

```rust
fn spawn_open_test_button(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Button,
        OpensPanel {
            key: PANEL_TEST,
            context: UiPanelContext::None,
        },
        Node {
            width: Val::Px(120.0),
            height: Val::Px(36.0),
            ..default()
        },
    ))
    .queue_spawn_related_scenes::<Children>(bsn! {
        (
            Text("Open Test")
            TextFont { font_size: FontSize::Px(14.0) }
            TextColor(Color::WHITE)
        )
    });
}
```

The click handler is global UI infrastructure and only needs to be registered once:

```rust
app.add_observer(open_panel_button_clicked);
```

Full registration example:

```rust
pub const PANEL_TEST: UiPanelKey = UiPanelKey("plugin.test");

pub fn register_test_panel(mut registry: ResMut<UiPanelRegistry>) {
    registry.register(UiPanelDescriptor::new(
        PANEL_TEST,
        "test.title",
        true,
        spawn_test_panel,
    ));
}

fn open_test_panel_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: MessageWriter<OpenUiPanel>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        open.write(OpenUiPanel::new(PANEL_TEST, UiPanelContext::None));
    }
}

fn spawn_test_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(320.0, "test.title").closable(),
        UiPanelBinding(PANEL_TEST),
        |panel| {
            panel
                .spawn_empty()
                .queue_spawn_related_scenes::<Children>(bsn! {
                    (
                        Text("Hello panel")
                        TextFont { font_size: FontSize::Px(16.0) }
                        TextColor(Color::WHITE)
                    )
                });
        },
    )
}
```

Wiring:

```rust
app.add_systems(Startup, register_test_panel);
app.add_systems(Update, open_test_panel_shortcut);
```

## Migration Phases

### Phase 0: Confirm Bevy 0.19 Target

- Done: selected `bevy = "0.19.0-rc.2"` because final `0.19` is not stable yet.
- Done: renamed Bevy feature from `experimental_bevy_ui_widgets` to `bevy_ui_widgets`.
- Deferred until final release: recheck BSN feature name and imports when Bevy 0.19 final / BSN docs are available. This is not blocking the current `0.19.0-rc.2` migration target.
- Done: `cargo check` passes on the selected Bevy 0.19 target.

### Phase 1: Compatibility Upgrade

- Done: upgraded Bevy and fixed compile errors.
- Updated API points:
  - `experimental_bevy_ui_widgets`
  - `CoreSliderDragState` -> `SliderDragState`
  - `Slider` now includes `orientation`
  - `TextLayout::new_with_justify` -> `TextLayout::justify`
  - `TextFont.font` now uses `FontSource::Handle`
  - `TextFont.font_size` now uses `FontSize`
  - light `shadows_enabled` -> `shadow_maps_enabled`
- Done: native `cargo check`.
- Done: wasm target check passes with `cargo check --target wasm32-unknown-unknown`.
- Done: full Trunk build verification. `trunk v0.21.14` is installed at `/Users/plutocrown/.cargo/bin/trunk`, and `trunk build` passes when run with the clean environment command documented below.
- Note: `cargo check-web` initially failed while downloading `wgpu-core-deps-wasm` because the default proxy at `127.0.0.1:7897` was unavailable. Retrying with the project proxy wrapper succeeded for the equivalent wasm target check.
- Note: a plain Trunk build can fail if Cargo inherits unavailable local proxy variables. Verified command:

```bash
env -u http_proxy -u https_proxy -u all_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u NO_COLOR -u CLAP_STYLE -u CLICOLOR -u CLICOLOR_FORCE /Users/plutocrown/.cargo/bin/trunk build
```

### Phase 2: Introduce Registry Without Rewriting Panels

- Done: added `UiPanelKey`, `UiPanelRegistry`, `OpenUiPanel`, `CloseUiPanel`, `OpensPanel`, `UiPanelHost`, and `UiRoot`.
- Done: panel descriptors carry spawn functions.
- Done: `UiRuntime` stack now stores `UiPanelKey`.
- Done: `EditableBlock` and `BlockKind` now expose `ui_panel_key()` directly. Block selection/opening no longer converts from the legacy `UiPanelId` enum.
- Done: `UiPanelId` remains only as a compatibility shim for existing panel bindings and shared block-panel layout helpers.
- Done: settings and block edit panels are registered descriptors.

### Phase 3: Dynamic Spawn / Lazy Panels

- Done for registered panels: main menu, save list, pause menu, inventory, settings, demo, block edit panels, and confirm dialog are no longer eagerly spawned by `setup_ui`.
- Keep root, HUD, modal scrim, and always-on overlays.
- Done: panel entities are spawned lazily on first `OpenUiPanel`.
- Done: spawned entities are tracked in `UiPanelHost`.
- Current policy: cache spawned panels; no automatic despawn yet.
- Done: `sync_mode_panels` bridges existing `GameMode` transitions to `OpenUiPanel` for main menu, save list, pause menu, and inventory.
- Always-on infrastructure surfaces: HUD, virtual controls, debug UI, modal scrim, hotbar, carried item label, inventory tooltip, and dropdown layers remain eager because they are shared overlay/high-frequency infrastructure rather than independently registered panels.
- Current always-on infrastructure policy: HUD/hotbar/tooltip/carried item/modal scrim/dropdown layers remain eager because they are shared overlay infrastructure or high-frequency cursor/layout surfaces rather than independently registered panels. Confirm dialog is no longer eagerly spawned. Text prompt is spawned with the lazily spawned save-list host rather than root startup.
- Still legacy visibility logic: main menu, save list, pause menu, and inventory are registered/lazy spawned but still use `PanelVisibility::GameMode` as their visible condition while `GameMode` remains the business state.
- Done: gameplay blocking is now copied from `UiPanelDescriptor.blocks_gameplay` into each `UiPanelSession` when `OpenUiPanel` is handled, so gameplay systems keep reading `UiRuntime` without adding broad registry parameters.
- Done: added `OpenConfirmDialog`, `OpenTextPrompt`, and `CloseUiModal` messages plus centralized modal handling in the UI systems.
- Done: pause menu, save list, confirm dialog actions, and text prompt actions now open/close modal UI by writing modal messages. Direct modal mutation remains isolated inside the central `modal_messages` system and `UiRuntime` methods.
- Done: confirm dialog root is lazily spawned by the central modal handler on first `OpenConfirmDialog`; it is no longer created by `setup_ui`.

### Phase 4: Reactive Refresh

- Done: broad update polling has been reduced to dirty messages, narrow change gates, or documented high-frequency/layout/render cases.
- Done: `update_settings_text_ui` returns early unless config, pending keybind, i18n, or newly added keybind text changed.
- Done: `update_settings_sliders_ui` returns early unless game settings, active slider state, active dragging, or newly added slider parts changed.
- Done: `update_settings_dropdowns_ui` separates label/value/layout refresh and returns early unless config, settings, i18n, dropdown state, UI scale, open dropdown layout, or newly added dropdown entities require work.
- Done: `update_settings_tabs_ui` returns early unless settings tab state, hover state, or newly added tab buttons changed.
- Done: `update_save_list_ui` now separates structural refresh from button visual refresh. Columns, widths, title, and prompt only refresh when mode, save data, solution entry, language, render state, or newly spawned lazy save-list entities change. Button labels/states refresh on those structural changes, hover changes, or newly added save-list actions.
- Done: simple block panel text refresh is now change-gated. `update_generator_ui`, `update_labeler_ui`, and `update_teleport_ui` return early unless the active UI context, world data, language/rename state, or newly spawned panel text entities changed. `update_converter_ui` now runs only when converter input rows are newly spawned.
- Done: `update_block_panel_dropdowns_ui` is split into label, material-icon, dropdown-layout, and teleport-pair refresh branches. It returns early when active context, world data, language, icon assets, open dropdown state, UI scale, and newly spawned dropdown entities have not changed. Dropdown positioning still runs while a dropdown is open.
- Done: `update_inventory_slots` now separates slot rendering from tooltip positioning. Slot contents, icons, display, selection border, and hover background refresh only when placement, inventory, language, hover, icon assets, or newly spawned slots change. Tooltip positioning still runs while an item is hovered.
- Done: `update_status_ui` now returns early unless placement, inventory, builder mode, simulation state, save state, config, language, or newly spawned status/panel text entities changed.
- Done: `update_hud_visibility` now returns early unless mode, builder mode, simulation state, save state, or newly spawned HUD marker entities changed.
- Done: confirm dialog and text prompt refresh now include newly spawned lazy modal entities and later moved to explicit modal/language dirty messages.
- Done: added panel lifecycle dirty messages: `UiPanelOpened`, `UiPanelClosed`, and `UiPanelContextChanged`. The central `OpenUiPanel` / `CloseUiPanel` handlers now emit these messages when a panel is opened, closed, or reopened with a different context.
- Done: regular user-facing close paths now write `CloseUiPanel` instead of mutating `UiRuntime` directly: Escape closing a modal panel, panel title close buttons, and Settings Back.
- Done: the defensive block-edit stale-context path now also writes `CloseUiPanel`. Direct `UiRuntime::close_*` calls remain only inside the central close-message handler.
- Done: `update_save_list_ui` and `update_inventory_slots` now consume `UiPanelOpened` / `UiPanelContextChanged` dirty messages for their own panel keys, so lazy open and context changes can trigger refresh without relying only on `Added` components or resource `is_changed()`.
- Done: settings refresh systems now consume `UiPanelOpened` / `UiPanelContextChanged` for `UiPanelKey::SETTINGS`.
- Done: block panel refresh systems now consume `UiPanelOpened` / `UiPanelContextChanged` for generator, labeler, converter, teleport, and shared block dropdown refresh.
- Done: added first business dirty messages: `SettingsChanged`, `InventoryChanged`, and `LanguageChanged`.
- Done: settings input changes, slider commits/live updates, selection mode changes, language changes, and reset-defaults now emit `SettingsChanged`; language changes also emit `LanguageChanged`.
- Done: inventory slot clicks now emit `InventoryChanged`, and `update_inventory_slots` consumes it. Settings refresh systems consume `SettingsChanged`.
- Done: added `SaveListChanged` and `BlockSettingsChanged`.
- Done: save-list puzzle selection, prompt-confirm create/rename/save-as flows, direct world loads, main-menu save-list entry, pause-menu save/clear paths, and confirm-dialog save/delete/load/switch paths now emit `SaveListChanged`; `update_save_list_ui` consumes it.
- Done: world-flow UI entry points that replace inventory state now emit `InventoryChanged`: save-list direct loads, prompt-created worlds/solutions, pause-menu builder-mode toggles, and confirm-dialog reset/load/switch paths.
- Done: `update_localized_ui` and `update_status_ui` now consume `LanguageChanged` outside settings-specific systems, and `update_localized_ui` also refreshes newly added `LocalizedText` entities.
- Done: block edit actions that mutate block settings now emit `BlockSettingsChanged`, and block panel refresh systems consume it.
- Done: `update_panel_visibility` and `update_ui_layers` now consume panel lifecycle dirty messages (`UiPanelOpened`, `UiPanelClosed`, `UiPanelContextChanged`) instead of relying only on broad `UiRuntime::is_changed()` gates.
- Done: `UiPanelClosed` now drives close-time cleanup for shared panel state: block dropdowns, teleport rename state, panel drag state, and settings dropdown/keybind/slider state. Close button handling no longer needs to perform those cleanups directly.
- Done: added modal lifecycle dirty messages (`UiModalOpened`, `UiModalClosed`) for confirm dialogs and text prompts. Central modal message handling now emits them, and `update_confirm_dialog_ui` / `update_text_prompt_ui` consume them instead of using broad `UiRuntime::is_changed()` as their primary refresh trigger.
- Done: text prompt keyboard input now emits text-prompt modal dirty messages when the prompt buffer changes, so the input value refresh is event-driven.
- Done: remaining block-panel refresh systems no longer use `UiRuntime::is_changed()` gates. Generator, labeler, converter, teleport, and shared block dropdown refresh now rely on panel lifecycle/context dirty messages, `BlockSettingsChanged`, resource-specific changes, and added UI entities.
- Done: teleport pair/name edits now emit `BlockSettingsChanged`, so teleport panel text/dropdown refresh is triggered by explicit block dirty messages.
- Done: `update_save_list_ui` no longer uses broad `GameMode`, `SaveState`, `SolutionState`, or `I18n` change gates. Save-list structure refresh now relies on `SaveListChanged`, `LanguageChanged`, panel lifecycle/context dirty messages, first render state, and newly added UI entities. Hover remains a direct visual-state trigger for button styling.
- Done: settings text/slider/dropdown refresh no longer uses broad `GameConfig`, `GameSettings`, or `I18n` change gates where `SettingsChanged` / `LanguageChanged` and settings panel lifecycle messages already cover the source mutation paths. Direct UI interaction state gates remain for pending keybind, active slider drag, open dropdown layout, UI scale layout, tab selection, and hover styling.
- Done: added `GameplayUiChanged` for gameplay UI state changes such as mode toggles, hotbar selection, pick-block updates, builder-mode transitions, world save/load/reset flows, and menu-driven simulation state changes.
- Done: `update_status_ui` and `update_hud_visibility` now consume `GameplayUiChanged`, `InventoryChanged`, `SaveListChanged`, `SettingsChanged`, and `LanguageChanged` instead of broad placement/inventory/builder/save/config/i18n gates. They keep `SimulationState::is_changed()` because simulation turn/speed/running text and gameplay HUD visibility can change continuously while simulation runs.
- Done: global localized text refresh now consumes `LanguageChanged` / `SaveListChanged` instead of broad `I18n` / `SaveState` gates.
- Done: inventory slot rendering now consumes `GameplayUiChanged`, `InventoryChanged`, and `LanguageChanged` instead of broad placement/inventory/i18n gates. Hover, icon asset changes, panel lifecycle, tooltip cursor following, and newly added slots remain direct UI/render triggers.
- Done: confirm dialog and text prompt language refresh now consume `LanguageChanged` instead of broad `I18n::is_changed()`.
- Done: remaining `Res::is_changed()` gates were audited. Remaining direct gates are intentionally limited to high-frequency or UI/layout/render asset cases: simulation state text/HUD, hover visuals, slider value drag, pending keybind UI state, open dropdown positioning, UI scale layout, block icon assets, world-backed block panel content, panel layer fallback, and mode-to-panel bridge.
- Done: remaining eager surfaces are documented exceptions for always-on overlay/high-frequency UI infrastructure rather than registered panels.
- Keep high-frequency systems only where truly needed:
  - drag
  - slider dragging
  - open dropdown positioning while a dropdown is visible
  - carried item following cursor
  - inventory tooltip following cursor
  - hover / pressed visual states
  - status text if it depends on continuously changing simulation time
  - virtual touch controls while touch input is active / visible
  - debug performance panel while enabled

### Phase 5: BSN Conversion

- Done: confirmed BSN macro and scene spawning APIs in local Bevy `0.19.0-rc.2` sources: `bevy_scene::{bsn, bsn_list}`, `Scene`, `SceneList`, and `EntityCommandsSceneExt::queue_spawn_related_scenes`.
- Done: added explicit `bevy_scene = "0.19.0-rc.2"` dependency.
- Done: converted `src/game/ui/demo_panel.rs` content to a compiled BSN `demo_panel_scene()` while preserving registry/lazy-panel opening via `UiPanelDescriptor`, `OpenUiPanel`, and `OpensPanel`.
- Done: started production-panel BSN migration by converting the Generator panel's dynamic period text entity to `generator_period_text_scene()` using `bsn!`.
- Done: converted Generator period down/up button visuals and localized labels to BSN-compatible `queue_apply_scene` / `queue_spawn_related_scenes` helpers. The action marker is still inserted directly on the button root so observer behavior stays explicit and stable.
- Done: converted the Generator material icon slot visual frame and icon child to BSN-compatible scene helpers while keeping `BlockMaterialIconSlot` and toggle action directly on the slot root for refresh/observer stability.
- Done: converted Generator row labels to BSN-compatible `generator_label_scene()` helpers.
- Done: Generator panel content is now fully on BSN-compatible helpers; the shared panel window wrapper and shared dropdown overlay list were converted later in this phase.
- Done: removed the now-unused generic `spawn_block_edit_button` helper.
- Done: converted Confirm Dialog title text, message text, action row, button visuals, and button labels to BSN-compatible scene helpers while keeping `ConfirmDialogAction` on the button root for observer stability.
- Done: converted Goal panel row label and material icon slot to BSN-compatible scene helpers while keeping `BlockMaterialIconSlot` and block edit action on the slot root for refresh/observer stability.
- Done: converted Labeler panel row label and color dropdown toggle to BSN-compatible scene helpers while keeping `BlockPanelDropdownLabel` and block edit action on the queried entities for refresh/observer stability.
- Done: converted Converter panel row labels and material icon slots to BSN-compatible scene helpers while keeping `ConverterInputRow`, `BlockMaterialIconSlot`, and block edit actions on their queried roots.
- Done: removed the now-unused shared `spawn_material_icon_slot` helper after Generator, Goal, and Converter moved to BSN slot helpers.
- Done: converted Teleport panel row labels, rename button, editable name text, and pair dropdown toggle to BSN-compatible scene helpers while keeping `TeleportAction`, `BlockPanelText`, and `BlockPanelDropdownLabel` on their queried entities.
- Done: removed now-unused shared row/dropdown helper functions (`spawn_panel_row`, `spawn_panel_label`, `panel_text`, `spawn_block_panel_dropdown`) after all block panels stopped using them.
- Done: block edit panel content is now on BSN-compatible helpers for Generator, Goal, Labeler/Stamper/Roller, Converter, and Teleport. Shared dropdown overlay lists and the shared panel window wrapper have also been converted to BSN-compatible scene helpers.
- Done: converted Main Menu and Pause Menu button widgets to BSN-compatible visual/label helpers while keeping `MainMenuAction` / `PauseMenuAction` on button roots for observer stability.
- Done: converted Save List reusable slot/select/management buttons and Text Prompt input/buttons to BSN-compatible helpers while keeping `SaveListAction`, `TextPromptAction`, and `TextPromptText` on their queried entities.
- Done: converted Settings localized buttons, tabs, slider value text, dropdown toggles, and dropdown list option buttons to BSN-compatible helpers while keeping `SettingsAction`, `KeyBindingButton`, `SettingsText`, `SettingsValueText`, `SettingsDropdownLabel`, and `SettingsDropdownList` on their queried entities.
- Done: removed the now-unused shared `full_width_button` helper after all callers moved to BSN-specific button scenes.
- Done: converted Inventory slot visuals, icon child, and count/label child to BSN-compatible helpers while keeping `InventorySlot` on the button root and preserving the child `ImageNode` / `Text` structure used by the render system.
- Done: converted Inventory carried-item preview and inventory tooltip to BSN-compatible helpers while keeping `CarriedItemPreview`, `InventoryTooltip`, and their child `ImageNode` / `Text` structures used by update systems.
- Done: converted HUD status overlay text entities to BSN-compatible helpers while keeping `Crosshair`, `StatusText`, `InGameHudVisibility`, and `GameplayHudVisibility` markers on queried roots.
- Done: removed the now-unused shared `absolute_text_bundle` helper after HUD status overlays moved to BSN helpers.
- Done: converted shared block dropdown overlay lists and options to BSN-compatible helpers while keeping `BlockPanelDropdownList`, `BlockMaterialIcon`, and option action markers on their queried entities.
- Done: removed stale public exports for legacy shared button helpers that are no longer used outside their implementation modules.
- Done: converted dynamically rebuilt Teleport pair dropdown options to BSN-compatible helpers while keeping `TeleportAction::SetPair` on option button roots.
- Done: removed now-unused legacy `menu_button` / `text_button` helpers after all callers moved to BSN-specific button scenes.
- Done: converted shared panel/window/title/content helpers to BSN scene fragments. `spawn_panel`, Save List, Text Prompt, and Confirm Dialog now use `panel_window_scene`, `panel_title_bar_scene`, `panel_title_label_scene`, `panel_title_button_scene`, `panel_close_label_scene`, and `panel_content_scene` while keeping panel lifecycle markers on queried roots.
- Done: removed now-unused legacy panel bundle helpers (`panel_bundle`, `panel_title_bar`, `panel_title_label`, `panel_title_button`, `panel_content`).
- Done: converted local Save List column layout, Inventory grid layout, Settings row/label/key-binding column layout, and shared block panel row layout to BSN scene helpers.
- Done: audited remaining `impl Bundle` helpers. Remaining direct Bundle helpers are infrastructure primitives: text construction, slider internals, scroll container/content, and root/transparent/flex layout wrappers.
- Done: final completion audit against this document passed through current-state scans and native/wasm/trunk verification.
- Done: convert remaining block panels.
- Done: convert main screens and always-on overlays to BSN-compatible helpers except for documented slider/scroll/layout primitives and small row/grid layout wrappers.
- Done: shared panel/window/dropdown/button helpers have been converted into BSN fragments/templates. Remaining direct Bundle helpers are intentionally limited to infrastructure primitives: text construction, slider internals, scroll container/content, root/transparent/flex layout wrappers.

## Acceptance Criteria

- Done: project compiles on the selected Bevy 0.19 target with native `cargo check`.
- Done: UI panels can be registered by module without editing a central enum through `UiPanelRegistry` / `UiPanelDescriptor`.
- Done: at least one panel opens from keyboard event and one panel opens from click/selection event through `OpenUiPanel`; `src/game/ui/demo_panel.rs` demonstrates both `KeyCode::KeyT` and `OpensPanel`.
- Done: block modules register panel descriptors through the registry path in `register_legacy_panels`, including generator, goal, labeler, converter, and teleport.
- Done: block modules expose panel ownership through `UiPanelKey` (`EditableBlock::ui_panel_key`) instead of requiring new entries in the legacy `UiPanelId` enum for the opening path.
- Current functional coverage to preserve:
  - main menu
  - save list
  - pause menu
  - settings
  - inventory
  - confirm dialog
  - text prompt
  - generator / goal / labeler / stamper / roller / converter / teleport panels
- Done: polling-style `update_*_ui` systems are reduced or gated by events/change detection / dirty messages. Remaining direct gates are documented as high-frequency or UI/layout/render asset cases.
- Done: no registered UI panel is required to be eagerly spawned at startup except always-on HUD/root infrastructure and shared overlay surfaces documented above.
- Done: wasm target check passes with `cargo check --target wasm32-unknown-unknown`.
- Done: full Trunk build verification passes with `/Users/plutocrown/.cargo/bin/trunk build` under a clean proxy/style environment.
- Done: BSN API availability is confirmed for the current Bevy `0.19.0-rc.2` target, and the demo panel now uses a compiled `bsn!` scene as the migration sample.
- Done: production panels have been incrementally converted to BSN-compatible helpers for the migrated scope. Remaining direct Bundle helpers are documented infrastructure primitives rather than panel content builders.

## Final Verification

- Done: `cargo fmt --check`
- Done: native `cargo check` on the selected Bevy `0.19.0-rc.2` target
- Done: `cargo check --target wasm32-unknown-unknown`
- Done: clean-environment `/Users/plutocrown/.cargo/bin/trunk build`
- Done: final scans found no stale open-status markers in this plan.
- Done: final scans found no remaining function calls to removed pre-BSN helper functions such as `label_text`, `localized_text`, `full_width_button`, `text_button`, `absolute_text_bundle`, `panel_bundle`, `spawn_panel_row`, `spawn_panel_label`, `spawn_block_panel_dropdown`, or `spawn_material_icon_slot`.

## Risks

- Bevy `0.19` may still be pre-release; BSN API may shift.
- Existing `experimental_bevy_ui_widgets` slider code may require migration.
- The project already has many observers; duplicating observer/event layers without removing old paths could create double actions.
- Dynamic spawn/despawn can break entity queries that assume panel entities always exist.
- Localization currently updates existing `LocalizedText` entities globally; lazy panels need initial localized spawn plus language-change refresh.
- Block dropdown positioning relies on existing entity queries and computed layout; moving to lazy BSN scenes may require a new dropdown host.

## Initial Execution Recommendation

When approved, start with Phase 0 and Phase 1 only. Do not start the registry rewrite until the Bevy 0.19 compile errors are resolved, because UI API breakage will otherwise mix with architecture changes.
