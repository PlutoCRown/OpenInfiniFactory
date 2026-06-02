use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::state::{GameMode, SolutionState, WorldEntryMode};
use crate::game::ui::components::{
    hover_border, inset_border, pressed_border, raised_border, BUTTON_BG, BUTTON_HOVER_BG,
};
use crate::game::ui::types::{
    LanguageChanged, PanelText, PanelTextKind, SaveListAction, SaveListButton, SaveListChanged,
    SaveListCloseButton, SaveListPanel, SaveListPrompt, SaveListPuzzleColumn, SaveListRenderState,
    SaveListSolutionColumn, TextPromptAction, TextPromptKind, TextPromptRoot, TextPromptText,
    UiHoverState, UiModalClosed, UiModalKind, UiModalOpened, UiPanelContextChanged, UiPanelKey,
    UiPanelOpened, UiRuntime,
};
use crate::shared::i18n::I18n;
use crate::shared::save::SaveState;

use super::{spawn_save_management_row, spawn_save_select_row, spawn_save_slot_button};

#[derive(SystemParam)]
pub struct SaveListRefreshTriggers<'w, 's> {
    added_panel_texts: Query<'w, 's, (), Added<PanelText>>,
    added_prompts: Query<'w, 's, (), Added<SaveListPrompt>>,
    added_panels: Query<'w, 's, (), Added<SaveListPanel>>,
    added_puzzle_columns: Query<'w, 's, (), Added<SaveListPuzzleColumn>>,
    added_solution_columns: Query<'w, 's, (), Added<SaveListSolutionColumn>>,
    added_buttons: Query<'w, 's, (), Added<SaveListAction>>,
    opened: MessageReader<'w, 's, UiPanelOpened>,
    context_changed: MessageReader<'w, 's, UiPanelContextChanged>,
    save_list_changed: MessageReader<'w, 's, SaveListChanged>,
    language_changed: MessageReader<'w, 's, LanguageChanged>,
}

impl SaveListRefreshTriggers<'_, '_> {
    fn layout_added(&self) -> bool {
        !self.added_panels.is_empty()
            || !self.added_puzzle_columns.is_empty()
            || !self.added_solution_columns.is_empty()
    }

    fn structure_dirty(&mut self) -> bool {
        !self.added_panel_texts.is_empty()
            || !self.added_prompts.is_empty()
            || !self.added_panels.is_empty()
            || !self.added_puzzle_columns.is_empty()
            || !self.added_solution_columns.is_empty()
            || self
                .opened
                .read()
                .any(|message| save_list_entry_for_key(message.key).is_some())
            || self
                .context_changed
                .read()
                .any(|message| save_list_entry_for_key(message.key).is_some())
            || self.save_list_changed.read().next().is_some()
            || self.language_changed.read().next().is_some()
    }

    fn buttons_added(&self) -> bool {
        !self.added_buttons.is_empty()
    }
}

pub fn update_save_list_ui(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    i18n: Res<I18n>,
    hover: Res<UiHoverState>,
    mut render_state: ResMut<SaveListRenderState>,
    mut triggers: SaveListRefreshTriggers,
    mut commands: Commands,
    mut texts: ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (Without<PanelText>, Without<SaveListPrompt>)>,
        Query<(&SaveListPrompt, &mut Text)>,
    )>,
    mut puzzle_columns: Query<
        (&SaveListPuzzleColumn, Entity, &mut Node, &Children),
        (With<SaveListPuzzleColumn>, Without<SaveListSolutionColumn>),
    >,
    mut solution_columns: Query<
        (&SaveListSolutionColumn, Entity, &mut Node, &Children),
        (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>),
    >,
    mut panels: Query<
        (&SaveListPanel, &mut Node),
        (
            With<SaveListPanel>,
            Without<SaveListPuzzleColumn>,
            Without<SaveListSolutionColumn>,
        ),
    >,
    mut buttons: Query<
        (
            Entity,
            &SaveListAction,
            &SaveListButton,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<SaveListCloseButton>),
    >,
) {
    let force_rebuild = triggers.layout_added();
    let entry = solution_state.save_list_entry;
    let current_render_state = match entry {
        WorldEntryMode::EditPuzzle => &render_state.edit,
        WorldEntryMode::PlaySolution => &render_state.play,
    };
    let refresh_structure =
        current_render_state.puzzle_keys.is_empty() || force_rebuild || triggers.structure_dirty();
    let refresh_buttons = refresh_structure || hover.is_changed() || triggers.buttons_added();

    if !refresh_structure && !refresh_buttons {
        return;
    }

    let play_flow = entry == WorldEntryMode::PlaySolution;
    let edit_flow = entry == WorldEntryMode::EditPuzzle;
    let puzzle_rows = save_list_puzzle_rows(&save_state, edit_flow);
    let solution_rows = save_state
        .selected_puzzle_solutions()
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    let show_solutions = play_flow && save_state.selected_puzzle.is_some();

    if refresh_structure {
        update_save_list_title(&mode, entry, &i18n, &mut texts);
        update_save_list_columns(
            &mut commands,
            &mut render_state,
            &mut panels,
            &mut puzzle_columns,
            &mut solution_columns,
            &puzzle_rows,
            &solution_rows,
            &i18n,
            entry,
            edit_flow,
            show_solutions,
            force_rebuild,
        );
        update_save_list_prompt(play_flow, &save_state, &i18n, &mut texts);
    }

    if refresh_buttons {
        update_save_list_buttons(&save_state, &i18n, &hover, &mut texts, &mut buttons, entry);
    }
}

fn save_list_entry_for_key(key: UiPanelKey) -> Option<WorldEntryMode> {
    match key {
        UiPanelKey::SAVE_LIST_EDIT => Some(WorldEntryMode::EditPuzzle),
        UiPanelKey::SAVE_LIST_PLAY => Some(WorldEntryMode::PlaySolution),
        _ => None,
    }
}

fn save_list_entry_for_flow(play_flow: bool) -> WorldEntryMode {
    if play_flow {
        WorldEntryMode::PlaySolution
    } else {
        WorldEntryMode::EditPuzzle
    }
}

fn save_list_puzzle_rows(save_state: &SaveState, edit_flow: bool) -> Vec<String> {
    if edit_flow {
        save_state
            .puzzles()
            .into_iter()
            .map(|entry| entry.name.clone())
            .collect()
    } else {
        save_state
            .puzzle_choices()
            .into_iter()
            .map(|choice| choice.name)
            .collect()
    }
}

fn update_save_list_title(
    mode: &GameMode,
    entry: WorldEntryMode,
    i18n: &I18n,
    texts: &mut ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (Without<PanelText>, Without<SaveListPrompt>)>,
        Query<(&SaveListPrompt, &mut Text)>,
    )>,
) {
    for (panel_text, mut text) in &mut texts.p0() {
        if panel_text.0 == PanelTextKind::SaveListTitle {
            text.0 = match *mode {
                GameMode::SaveListMain => match entry {
                    WorldEntryMode::EditPuzzle => i18n.text("save.title.edit_puzzle"),
                    WorldEntryMode::PlaySolution => i18n.text("save.title.play_solution"),
                },
                _ => i18n.text("save.title.default"),
            };
        }
    }
}

fn update_save_list_columns(
    commands: &mut Commands,
    render_states: &mut SaveListRenderState,
    panels: &mut Query<
        (&SaveListPanel, &mut Node),
        (
            With<SaveListPanel>,
            Without<SaveListPuzzleColumn>,
            Without<SaveListSolutionColumn>,
        ),
    >,
    puzzle_columns: &mut Query<
        (&SaveListPuzzleColumn, Entity, &mut Node, &Children),
        (With<SaveListPuzzleColumn>, Without<SaveListSolutionColumn>),
    >,
    solution_columns: &mut Query<
        (&SaveListSolutionColumn, Entity, &mut Node, &Children),
        (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>),
    >,
    puzzle_rows: &[String],
    solution_rows: &[String],
    i18n: &I18n,
    entry: WorldEntryMode,
    edit_flow: bool,
    show_solutions: bool,
    force_rebuild: bool,
) {
    let render_state = match entry {
        WorldEntryMode::EditPuzzle => &mut render_states.edit,
        WorldEntryMode::PlaySolution => &mut render_states.play,
    };
    let puzzle_width = save_list_column_width(
        puzzle_rows,
        if edit_flow {
            SaveListColumnKind::ManagementPuzzle
        } else {
            SaveListColumnKind::PuzzleSelect
        },
        i18n,
    );
    let solution_width =
        save_list_column_width(solution_rows, SaveListColumnKind::ManagementSolution, i18n);
    let panel_width = if show_solutions {
        puzzle_width + solution_width + 44.0
    } else {
        puzzle_width + 32.0
    };
    for (panel_entry, mut panel) in panels {
        if panel_entry.0 != entry {
            continue;
        }
        panel.width = Val::Px(panel_width);
    }

    for (column_entry, entity, mut node, children) in puzzle_columns {
        if column_entry.0 != entry {
            continue;
        }
        node.display = Display::Flex;
        node.width = Val::Px(puzzle_width);
        if force_rebuild || render_state.puzzle_keys != puzzle_rows {
            if edit_flow {
                rebuild_management_column(
                    commands,
                    entity,
                    children,
                    entry,
                    Some(SaveListAction::NewPuzzle),
                    puzzle_rows,
                    SaveListAction::LoadPuzzle,
                    SaveListAction::RenamePuzzle,
                    SaveListAction::DeletePuzzle,
                    puzzle_width,
                );
            } else {
                rebuild_selection_column(
                    commands,
                    entity,
                    children,
                    entry,
                    puzzle_rows,
                    SaveListAction::LoadPuzzle,
                );
            }
        }
    }
    if render_state.puzzle_keys != puzzle_rows {
        render_state.puzzle_keys = puzzle_rows.to_vec();
    }

    for (column_entry, entity, mut node, children) in solution_columns {
        if column_entry.0 != entry {
            continue;
        }
        node.display = if show_solutions {
            Display::Flex
        } else {
            Display::None
        };
        node.width = Val::Px(solution_width);
        if force_rebuild || render_state.solution_keys != solution_rows {
            rebuild_management_column(
                commands,
                entity,
                children,
                entry,
                Some(SaveListAction::NewSolution),
                solution_rows,
                SaveListAction::LoadSolution,
                SaveListAction::RenameSolution,
                SaveListAction::DeleteSolution,
                solution_width,
            );
        }
    }
    if render_state.solution_keys != solution_rows {
        render_state.solution_keys = solution_rows.to_vec();
    }
}

fn rebuild_management_column(
    commands: &mut Commands,
    column_entity: Entity,
    children: &Children,
    entry: WorldEntryMode,
    create_action: Option<SaveListAction>,
    names: &[String],
    load: fn(String) -> SaveListAction,
    rename: fn(String) -> SaveListAction,
    delete: fn(String) -> SaveListAction,
    width: f32,
) {
    for child in children.iter() {
        commands.entity(child).despawn();
    }
    commands.entity(column_entity).with_children(|column| {
        for name in names {
            spawn_save_management_row(
                column,
                entry,
                load(name.clone()),
                rename(name.clone()),
                delete(name.clone()),
                width,
            );
        }
        if let Some(create_action) = create_action {
            spawn_save_slot_button(column, entry, create_action);
        }
    });
}

#[derive(Clone, Copy)]
enum SaveListColumnKind {
    PuzzleSelect,
    ManagementPuzzle,
    ManagementSolution,
}

fn save_list_column_width(names: &[String], kind: SaveListColumnKind, i18n: &I18n) -> f32 {
    let longest_name = names
        .iter()
        .map(|name| name.chars().count())
        .max()
        .unwrap_or(0) as f32;
    let action_chars = match kind {
        SaveListColumnKind::PuzzleSelect => [
            i18n.text("save.select_puzzle").chars().count(),
            i18n.text("save.selected_puzzle").chars().count(),
        ]
        .into_iter()
        .max()
        .unwrap_or(0) as f32,
        SaveListColumnKind::ManagementPuzzle => {
            i18n.text("save.load_puzzle").chars().count() as f32
        }
        SaveListColumnKind::ManagementSolution => {
            i18n.text("save.load_solution").chars().count() as f32
        }
    };
    let estimated = (longest_name + action_chars) * 9.5 + 56.0;
    match kind {
        SaveListColumnKind::PuzzleSelect => estimated.clamp(320.0, 520.0),
        SaveListColumnKind::ManagementPuzzle | SaveListColumnKind::ManagementSolution => {
            estimated.clamp(390.0, 560.0)
        }
    }
}

fn rebuild_selection_column(
    commands: &mut Commands,
    column_entity: Entity,
    children: &Children,
    entry: WorldEntryMode,
    names: &[String],
    load: fn(String) -> SaveListAction,
) {
    for child in children.iter() {
        commands.entity(child).despawn();
    }
    commands.entity(column_entity).with_children(|column| {
        for name in names {
            spawn_save_select_row(column, entry, load(name.clone()));
        }
    });
}

fn update_save_list_prompt(
    play_flow: bool,
    save_state: &SaveState,
    i18n: &I18n,
    texts: &mut ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (Without<PanelText>, Without<SaveListPrompt>)>,
        Query<(&SaveListPrompt, &mut Text)>,
    )>,
) {
    for (prompt, mut text) in &mut texts.p2() {
        if prompt.0 != save_list_entry_for_flow(play_flow) {
            continue;
        }
        text.0 = if play_flow && save_state.selected_puzzle.is_none() {
            i18n.text("save.choose_puzzle_prompt")
        } else {
            String::new()
        };
    }
}

fn update_save_list_buttons(
    save_state: &SaveState,
    i18n: &I18n,
    hover: &UiHoverState,
    texts: &mut ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (Without<PanelText>, Without<SaveListPrompt>)>,
        Query<(&SaveListPrompt, &mut Text)>,
    )>,
    buttons: &mut Query<
        (
            Entity,
            &SaveListAction,
            &SaveListButton,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<SaveListCloseButton>),
    >,
    entry: WorldEntryMode,
) {
    let play_flow = entry == WorldEntryMode::PlaySolution;
    let edit_flow = entry == WorldEntryMode::EditPuzzle;

    for (entity, action, button, children, mut background, mut border) in buttons {
        if button.0 != entry {
            continue;
        }
        let label = save_list_button_label(action, save_state, play_flow, i18n);
        let enabled = save_list_button_enabled(action, save_state, edit_flow, play_flow);
        let selected = save_list_button_selected(action, save_state, play_flow);
        let hovered = enabled && hover.entity == Some(entity);

        *background = if hovered {
            BUTTON_HOVER_BG.into()
        } else if enabled && selected {
            Color::srgba(0.22, 0.35, 0.32, 0.96).into()
        } else if enabled {
            BUTTON_BG.into()
        } else {
            Color::srgba(0.12, 0.12, 0.13, 0.82).into()
        };
        *border = if hovered {
            hover_border()
        } else if selected {
            pressed_border()
        } else if enabled {
            raised_border()
        } else {
            inset_border()
        };

        for child in children.iter() {
            if let Ok(mut text) = texts.p1().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

fn save_list_button_label(
    action: &SaveListAction,
    save_state: &SaveState,
    play_flow: bool,
    i18n: &I18n,
) -> String {
    match action {
        SaveListAction::LoadPuzzle(name) => {
            if play_flow {
                if save_state.selected_puzzle.as_deref() == Some(name.as_str()) {
                    i18n.fmt("save.selected_puzzle", &[("name", name.clone())])
                } else {
                    i18n.fmt("save.select_puzzle", &[("name", name.clone())])
                }
            } else {
                i18n.fmt("save.load_puzzle", &[("name", name.clone())])
            }
        }
        SaveListAction::LoadSolution(name) => {
            i18n.fmt("save.load_solution", &[("name", name.clone())])
        }
        SaveListAction::RenamePuzzle(_) | SaveListAction::RenameSolution(_) => {
            i18n.text("button.rename")
        }
        SaveListAction::DeletePuzzle(_) | SaveListAction::DeleteSolution(_) => {
            i18n.text("button.delete")
        }
        SaveListAction::NewPuzzle => i18n.text("button.new_puzzle"),
        SaveListAction::NewSolution => i18n.text("button.new_solution"),
        SaveListAction::Back => i18n.text("button.back"),
    }
}

fn save_list_button_enabled(
    action: &SaveListAction,
    save_state: &SaveState,
    edit_flow: bool,
    play_flow: bool,
) -> bool {
    match action {
        SaveListAction::LoadPuzzle(name) => {
            save_state
                .puzzle_choices()
                .iter()
                .any(|choice| &choice.name == name)
                || save_state.puzzles().iter().any(|entry| &entry.name == name)
        }
        SaveListAction::LoadSolution(name)
        | SaveListAction::RenameSolution(name)
        | SaveListAction::DeleteSolution(name) => {
            play_flow
                && save_state
                    .selected_puzzle_solutions()
                    .iter()
                    .any(|entry| &entry.name == name)
        }
        SaveListAction::RenamePuzzle(name) | SaveListAction::DeletePuzzle(name) => {
            edit_flow && save_state.puzzles().iter().any(|entry| &entry.name == name)
        }
        SaveListAction::NewPuzzle => edit_flow,
        SaveListAction::NewSolution => play_flow && save_state.selected_puzzle.is_some(),
        SaveListAction::Back => true,
    }
}

fn save_list_button_selected(
    action: &SaveListAction,
    save_state: &SaveState,
    play_flow: bool,
) -> bool {
    matches!(action, SaveListAction::LoadPuzzle(name) if play_flow
        && save_state.selected_puzzle.as_deref() == Some(name.as_str()))
}

pub fn update_text_prompt_ui(
    ui_runtime: Res<UiRuntime>,
    i18n: Res<I18n>,
    mut roots: Query<(&mut Node, &mut Visibility), With<TextPromptRoot>>,
    mut texts: ParamSet<(
        Query<(&TextPromptText, &mut Text)>,
        Query<(&TextPromptAction, &Children)>,
        Query<&mut Text, Without<TextPromptText>>,
    )>,
    added_roots: Query<(), Added<TextPromptRoot>>,
    added_texts: Query<(), Added<TextPromptText>>,
    added_actions: Query<(), Added<TextPromptAction>>,
    mut modal_opened: MessageReader<UiModalOpened>,
    mut modal_closed: MessageReader<UiModalClosed>,
    mut language_changed: MessageReader<LanguageChanged>,
) {
    let modal_dirty = modal_opened
        .read()
        .any(|message| message.kind == UiModalKind::TextPrompt)
        || modal_closed
            .read()
            .any(|message| message.kind == UiModalKind::TextPrompt);
    let language_dirty = language_changed.read().next().is_some();

    if !modal_dirty
        && !language_dirty
        && added_roots.is_empty()
        && added_texts.is_empty()
        && added_actions.is_empty()
    {
        return;
    }

    let visible = ui_runtime.text_prompt().is_some();
    for (mut node, mut visibility) in &mut roots {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let Some(prompt) = ui_runtime.text_prompt() else {
        return;
    };

    for (kind_marker, mut text) in &mut texts.p0() {
        text.0 = match kind_marker {
            TextPromptText::Title => text_prompt_title(&prompt.kind, &i18n),
            TextPromptText::Value => format!("{}_", prompt.value),
        };
    }

    let mut button_labels = Vec::new();
    for (action, children) in &mut texts.p1() {
        button_labels.push((*action, children.iter().collect::<Vec<_>>()));
    }
    for (action, children) in button_labels {
        let label = match action {
            TextPromptAction::Confirm => i18n.text("button.confirm"),
            TextPromptAction::Cancel => i18n.text("button.cancel"),
        };
        for child in children {
            if let Ok(mut text) = texts.p2().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

fn text_prompt_title(kind: &TextPromptKind, i18n: &I18n) -> String {
    match kind {
        TextPromptKind::NewPuzzle => i18n.text("save.prompt.new_puzzle"),
        TextPromptKind::NewSolution { .. } => i18n.text("save.prompt.new_solution"),
        TextPromptKind::RenamePuzzle { .. } => i18n.text("save.prompt.rename_puzzle"),
        TextPromptKind::RenameSolution { .. } => i18n.text("save.prompt.rename_solution"),
        TextPromptKind::SaveAsNewPuzzle => i18n.text("save.prompt.save_as_new_puzzle"),
    }
}
