use crate::game::ui::screens::{
    spawn_save_management_row, spawn_save_select_row,
};

pub fn update_save_list_ui(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    i18n: Res<I18n>,
    hover: Res<UiHoverState>,
    mut render_state: ResMut<SaveListRenderState>,
    mut commands: Commands,
    mut texts: ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (Without<PanelText>, Without<SaveListPrompt>)>,
        Query<&mut Text, With<SaveListPrompt>>,
    )>,
    mut puzzle_columns: Query<
        (Entity, &mut Node, &Children),
        (With<SaveListPuzzleColumn>, Without<SaveListSolutionColumn>),
    >,
    mut solution_columns: Query<
        (Entity, &mut Node, &Children),
        (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>),
    >,
    mut panels: Query<
        &mut Node,
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
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<SaveListCloseButton>),
    >,
) {
    let play_flow = solution_state.save_list_entry == WorldEntryMode::PlaySolution;
    let edit_flow = solution_state.save_list_entry == WorldEntryMode::EditPuzzle;
    let puzzle_rows = save_list_puzzle_rows(&save_state, edit_flow);
    let solution_rows = save_state
        .selected_puzzle_solutions()
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    let show_solutions = play_flow && save_state.selected_puzzle.is_some();

    update_save_list_title(&mode, &solution_state, &i18n, &mut texts);
    update_save_list_columns(
        &mut commands,
        &mut render_state,
        &mut panels,
        &mut puzzle_columns,
        &mut solution_columns,
        &puzzle_rows,
        &solution_rows,
        &i18n,
        solution_state.save_list_entry,
        edit_flow,
        show_solutions,
    );
    update_save_list_prompt(play_flow, &save_state, &i18n, &mut texts);
    update_save_list_buttons(
        &save_state,
        &solution_state,
        &i18n,
        &hover,
        &mut texts,
        &mut buttons,
    );
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
    solution_state: &SolutionState,
    i18n: &I18n,
    texts: &mut ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (Without<PanelText>, Without<SaveListPrompt>)>,
        Query<&mut Text, With<SaveListPrompt>>,
    )>,
) {
    for (panel_text, mut text) in &mut texts.p0() {
        if panel_text.0 == PanelTextKind::SaveListTitle {
            text.0 = match *mode {
                GameMode::SaveListMain => match solution_state.save_list_entry {
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
    render_state: &mut SaveListRenderState,
    panels: &mut Query<
        &mut Node,
        (
            With<SaveListPanel>,
            Without<SaveListPuzzleColumn>,
            Without<SaveListSolutionColumn>,
        ),
    >,
    puzzle_columns: &mut Query<
        (Entity, &mut Node, &Children),
        (With<SaveListPuzzleColumn>, Without<SaveListSolutionColumn>),
    >,
    solution_columns: &mut Query<
        (Entity, &mut Node, &Children),
        (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>),
    >,
    puzzle_rows: &[String],
    solution_rows: &[String],
    i18n: &I18n,
    entry: WorldEntryMode,
    edit_flow: bool,
    show_solutions: bool,
) {
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
    for mut panel in panels {
        panel.width = Val::Px(panel_width);
    }

    for (entity, mut node, children) in puzzle_columns {
        node.display = Display::Flex;
        node.width = Val::Px(puzzle_width);
        if render_state.entry != Some(entry) || render_state.puzzle_keys != puzzle_rows {
            if edit_flow {
                rebuild_management_column(
                    commands,
                    entity,
                    children,
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
                    puzzle_rows,
                    SaveListAction::LoadPuzzle,
                );
            }
        }
    }
    if render_state.entry != Some(entry) || render_state.puzzle_keys != puzzle_rows {
        render_state.puzzle_keys = puzzle_rows.to_vec();
    }

    for (entity, mut node, children) in solution_columns {
        node.display = if show_solutions {
            Display::Flex
        } else {
            Display::None
        };
        node.width = Val::Px(solution_width);
        if render_state.entry != Some(entry) || render_state.solution_keys != solution_rows {
            rebuild_management_column(
                commands,
                entity,
                children,
                Some(SaveListAction::NewSolution),
                solution_rows,
                SaveListAction::LoadSolution,
                SaveListAction::RenameSolution,
                SaveListAction::DeleteSolution,
                solution_width,
            );
        }
    }
    if render_state.entry != Some(entry) || render_state.solution_keys != solution_rows {
        render_state.solution_keys = solution_rows.to_vec();
    }
    render_state.entry = Some(entry);
}

fn rebuild_management_column(
    commands: &mut Commands,
    column_entity: Entity,
    children: &Children,
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
                load(name.clone()),
                rename(name.clone()),
                delete(name.clone()),
                width,
            );
        }
        if let Some(create_action) = create_action {
            column
                .spawn((full_width_button(34.0), create_action))
                .with_children(|button| {
                    button.spawn(text("", 15.0, Color::WHITE));
                });
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
    let longest_name = names.iter().map(|name| name.chars().count()).max().unwrap_or(0) as f32;
    let action_chars = match kind {
        SaveListColumnKind::PuzzleSelect => [
            i18n.text("save.select_puzzle").chars().count(),
            i18n.text("save.selected_puzzle").chars().count(),
        ]
        .into_iter()
        .max()
        .unwrap_or(0) as f32,
        SaveListColumnKind::ManagementPuzzle => i18n.text("save.load_puzzle").chars().count() as f32,
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
    names: &[String],
    load: fn(String) -> SaveListAction,
) {
    for child in children.iter() {
        commands.entity(child).despawn();
    }
    commands.entity(column_entity).with_children(|column| {
        for name in names {
            spawn_save_select_row(column, load(name.clone()));
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
        Query<&mut Text, With<SaveListPrompt>>,
    )>,
) {
    for mut text in &mut texts.p2() {
        text.0 = if play_flow && save_state.selected_puzzle.is_none() {
            i18n.text("save.choose_puzzle_prompt")
        } else {
            String::new()
        };
    }
}

fn update_save_list_buttons(
    save_state: &SaveState,
    solution_state: &SolutionState,
    i18n: &I18n,
    hover: &UiHoverState,
    texts: &mut ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (Without<PanelText>, Without<SaveListPrompt>)>,
        Query<&mut Text, With<SaveListPrompt>>,
    )>,
    buttons: &mut Query<
        (
            Entity,
            &SaveListAction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<SaveListCloseButton>),
    >,
) {
    let play_flow = solution_state.save_list_entry == WorldEntryMode::PlaySolution;
    let edit_flow = solution_state.save_list_entry == WorldEntryMode::EditPuzzle;

    for (entity, action, children, mut background, mut border) in buttons {
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

pub fn update_confirm_dialog_ui(
    ui_runtime: Res<UiRuntime>,
    i18n: Res<I18n>,
    mut texts: ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, Without<PanelText>>,
    )>,
    mut action_buttons: Query<(&ConfirmDialogAction, &mut Node, &Children), With<Button>>,
) {
    if !ui_runtime.is_changed() && !i18n.is_changed() {
        return;
    }

    let Some(dialog) = ui_runtime.confirm_dialog() else {
        return;
    };
    for (panel_text, mut text) in &mut texts.p0() {
        text.0 = match panel_text.0 {
            PanelTextKind::ConfirmTitle => i18n.text(dialog.title_key),
            PanelTextKind::ConfirmMessage => confirm_dialog_message(&dialog.message, &i18n),
            _ => continue,
        };
    }
    for (action, mut node, children) in &mut action_buttons {
        if matches!(*action, ConfirmDialogAction::Secondary) {
            node.display = if dialog.secondary_key.is_some() {
                Display::Flex
            } else {
                Display::None
            };
        } else {
            node.display = Display::Flex;
        }
        let label = confirm_dialog_button_label(dialog, *action, &i18n);
        let width = confirm_dialog_button_width(&label);
        node.width = Val::Px(width);
        node.min_width = Val::Px(width);
        node.flex_grow = 0.0;
        for child in children.iter() {
            if let Ok(mut text) = texts.p1().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
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
) {
    if !ui_runtime.is_changed() && !i18n.is_changed() {
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

fn confirm_dialog_button_label(
    dialog: &ConfirmDialogState,
    action: ConfirmDialogAction,
    i18n: &I18n,
) -> String {
    match action {
        ConfirmDialogAction::Primary => i18n.text(dialog.primary_key),
        ConfirmDialogAction::Secondary => dialog
            .secondary_key
            .map(|key| i18n.text(key))
            .unwrap_or_default(),
        ConfirmDialogAction::Cancel => i18n.text(dialog.cancel_key),
    }
}

fn confirm_dialog_message(message: &ConfirmDialogMessage, i18n: &I18n) -> String {
    match message {
        ConfirmDialogMessage::TextKey(key) => i18n.text(key),
        ConfirmDialogMessage::Named { key, name } => {
            i18n.fmt(key, &[("name", name.clone())])
        }
    }
}

fn confirm_dialog_button_width(label: &str) -> f32 {
    let char_count = label.chars().count() as f32;
    let wide_count = label.chars().filter(|ch| !ch.is_ascii()).count() as f32;
    let estimated_text_width = char_count * 10.0 + wide_count * 8.0;
    estimated_text_width.clamp(118.0, 230.0)
}
