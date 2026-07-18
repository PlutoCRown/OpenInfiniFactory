use bevy::prelude::*;

use crate::game::state::{GameMode, SolutionState, StartMenuScreen, WorldEntryMode};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiMainThread;
use crate::game::ui::access::{UiAccessScope, ui};
use crate::game::ui::core::host::UiRootEntity;
use crate::game::ui::core::runtime::UiPanelContext;
use crate::game::ui::menu_button::{MenuButtonClick, MenuButtonSet, spawn_menu_button};
use crate::list_ui_config;
use crate::shared::save::SaveState;

pub struct StartMenuPlugin;

struct StartMenuCtx<'w> {
    start_menu_screen: &'w mut StartMenuScreen,
    save_state: &'w mut SaveState,
    solution_state: &'w mut SolutionState,
    ui_root: Option<Entity>,
}

struct StartMenuButton {
    label_key: &'static str,
    on_click: fn(&mut StartMenuCtx<'_>, &mut Commands),
}

const START_MENU_BUTTONS: &[StartMenuButton] = list_ui_config!(
    StartMenuButton,
    ctx: StartMenuCtx<'_>,
    {
        key: "button.edit_puzzle"
        on_click(ctx, _commands) {
            ctx.save_state.refresh();
            ctx.save_state.select_puzzle(None);
            ctx.solution_state.save_list_entry = WorldEntryMode::EditPuzzle;
            *ctx.start_menu_screen = StartMenuScreen::SaveList;
        }
    };
    {
        key: "button.start_playing"
        on_click(ctx, _commands) {
            ctx.save_state.refresh();
            ctx.save_state.select_puzzle(None);
            ctx.solution_state.save_list_entry = WorldEntryMode::PlaySolution;
            *ctx.start_menu_screen = StartMenuScreen::SaveList;
        }
    };
    {
        key: "button.settings"
        on_click(ctx, commands) {
            ui.mount_settings(
                commands,
                ctx.ui_root,
                UiPanelContext::SettingsFromStartMenu,
            );
        }
    }
);

#[cfg(not(target_arch = "wasm32"))]
const START_MENU_QUIT: StartMenuButton = StartMenuButton {
    label_key: "button.quit_game",
    on_click: {
        fn on_click(_ctx: &mut StartMenuCtx<'_>, _commands: &mut Commands) {
            std::process::exit(0);
        }
        on_click
    },
};

fn start_menu_buttons() -> Vec<&'static StartMenuButton> {
    let mut buttons: Vec<&'static StartMenuButton> = START_MENU_BUTTONS.iter().collect();
    #[cfg(not(target_arch = "wasm32"))]
    buttons.push(&START_MENU_QUIT);
    buttons
}

impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            dispatch_start_menu_clicks
                .in_set(UiAccessScope)
                .after(PerfScope::Placement)
                .before(PerfScope::Menus),
        );
    }
}

pub fn spawn_start_menu_buttons(panel: &mut ChildSpawnerCommands) {
    for (index, button) in start_menu_buttons().into_iter().enumerate() {
        spawn_menu_button(
            panel,
            MenuButtonSet::StartMenu,
            index as u8,
            button.label_key,
        );
    }
}

fn dispatch_start_menu_clicks(
    _ui_thread: UiMainThread,
    mut clicks: MessageReader<MenuButtonClick>,
    mode: Res<State<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    ui_root: Option<Res<UiRootEntity>>,
    mut commands: Commands,
) {
    if *mode.get() != GameMode::StartMenu {
        return;
    }
    let ui_root = ui_root.as_deref().map(|root| root.0);
    let buttons = start_menu_buttons();
    for click in clicks.read() {
        if click.set != MenuButtonSet::StartMenu {
            continue;
        }
        let Some(button) = buttons.get(click.index as usize) else {
            continue;
        };
        let mut ctx = StartMenuCtx {
            start_menu_screen: &mut start_menu_screen,
            save_state: &mut save_state,
            solution_state: &mut solution_state,
            ui_root,
        };
        (button.on_click)(&mut ctx, &mut commands);
    }
}
