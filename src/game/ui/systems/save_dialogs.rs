pub fn update_save_list_ui(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    i18n: Res<I18n>,
    mut texts: ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, Without<PanelText>>,
    )>,
    mut slots: Query<
        (
            &SaveListAction,
            &Interaction,
            &Children,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
) {
    for (panel_text, mut text) in &mut texts.p0() {
        if panel_text.0 == PanelTextKind::SaveListTitle {
            text.0 = match *mode {
                GameMode::SaveListMain => i18n.text("save.title.main"),
                _ => i18n.text("save.title.default"),
            };
        }
    }

    let puzzles = save_state.puzzles();
    let solutions = save_state
        .selected_puzzle
        .as_deref()
        .map(|puzzle| save_state.solutions_for_puzzle(puzzle))
        .unwrap_or_default();
    let play_flow = solution_state.save_list_entry == WorldEntryMode::PlaySolution;
    let edit_flow = solution_state.save_list_entry == WorldEntryMode::EditPuzzle;

    for (action, interaction, children, mut background) in &mut slots {
        let label = match *action {
            SaveListAction::LoadPuzzle(index) => puzzles
                .get(index)
                .map(|entry| {
                    if save_state.selected_puzzle.as_deref() == Some(entry.name.as_str()) {
                        i18n.fmt("save.selected_puzzle", &[("name", entry.name.clone())])
                    } else if play_flow {
                        i18n.fmt("save.select_puzzle", &[("name", entry.name.clone())])
                    } else {
                        i18n.fmt("save.load_puzzle", &[("name", entry.name.clone())])
                    }
                })
                .unwrap_or_else(|| i18n.text("empty_slot")),
            SaveListAction::LoadSolution(index) => solutions
                .get(index)
                .map(|entry| i18n.fmt("save.load_solution", &[("name", entry.name.clone())]))
                .unwrap_or_else(|| i18n.text("empty_slot")),
            SaveListAction::DeletePuzzle(_) | SaveListAction::DeleteSolution(_) => {
                i18n.text("button.delete")
            }
            SaveListAction::NewPuzzle => i18n.text("button.new_puzzle"),
            SaveListAction::NewSolution => i18n.text("button.new_solution"),
            SaveListAction::Back => i18n.text("button.back"),
        };

        let enabled_load = match *action {
            SaveListAction::LoadPuzzle(index) => puzzles.get(index).is_some(),
            SaveListAction::LoadSolution(index) => play_flow && solutions.get(index).is_some(),
            SaveListAction::DeletePuzzle(index) => puzzles.get(index).is_some(),
            SaveListAction::DeleteSolution(index) => play_flow && solutions.get(index).is_some(),
            SaveListAction::NewPuzzle => edit_flow,
            SaveListAction::NewSolution => play_flow && save_state.selected_puzzle.is_some(),
            SaveListAction::Back => true,
        };
        let selected_puzzle_button = matches!(*action, SaveListAction::LoadPuzzle(_))
            && match *action {
                SaveListAction::LoadPuzzle(index) => puzzles.get(index).is_some_and(|entry| {
                    save_state.selected_puzzle.as_deref() == Some(entry.name.as_str())
                }),
                _ => false,
            };

        *background = if enabled_load && *interaction == Interaction::Hovered {
            BUTTON_HOVER_BG.into()
        } else if enabled_load && selected_puzzle_button {
            Color::srgba(0.22, 0.35, 0.32, 0.96).into()
        } else if enabled_load {
            BUTTON_BG.into()
        } else {
            Color::srgba(0.12, 0.12, 0.13, 0.82).into()
        };

        for child in children.iter() {
            if let Ok(mut text) = texts.p1().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

pub fn update_confirm_dialog_ui(
    dialog: Res<ConfirmDialogState>,
    i18n: Res<I18n>,
    mut texts: ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, Without<PanelText>>,
    )>,
    mut action_buttons: Query<(&ConfirmDialogAction, &mut Node, &Children), With<Button>>,
) {
    if !dialog.is_changed() && !i18n.is_changed() {
        return;
    }

    let Some(kind) = dialog.kind.as_ref() else {
        return;
    };
    for (panel_text, mut text) in &mut texts.p0() {
        text.0 = match panel_text.0 {
            PanelTextKind::ConfirmTitle => i18n.text("confirm.title"),
            PanelTextKind::ConfirmMessage => match kind {
                ConfirmDialogKind::DeleteSave { name } => {
                    i18n.fmt("save.confirm_delete", &[("name", name.clone())])
                }
                ConfirmDialogKind::ResetSolution => i18n.text("confirm.reset_solution"),
                ConfirmDialogKind::ReturnToMain => i18n.text("confirm.return_to_main"),
                ConfirmDialogKind::SaveSolutionBeforeEdit => {
                    i18n.text("confirm.save_solution_before_edit")
                }
            },
            _ => continue,
        };
    }
    let secondary_visible = matches!(
        kind,
        ConfirmDialogKind::ReturnToMain | ConfirmDialogKind::SaveSolutionBeforeEdit
    );
    for (action, mut node, children) in &mut action_buttons {
        if matches!(*action, ConfirmDialogAction::Secondary) {
            node.display = if secondary_visible {
                Display::Flex
            } else {
                Display::None
            };
        } else {
            node.display = Display::Flex;
        }
        let label = confirm_dialog_button_label(kind, *action, &i18n);
        for child in children.iter() {
            if let Ok(mut text) = texts.p1().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

fn confirm_dialog_button_label(
    kind: &ConfirmDialogKind,
    action: ConfirmDialogAction,
    i18n: &I18n,
) -> String {
    match action {
        ConfirmDialogAction::Primary => match kind {
            ConfirmDialogKind::DeleteSave { .. } => i18n.text("button.delete"),
            ConfirmDialogKind::ResetSolution => i18n.text("button.confirm_reset_solution"),
            ConfirmDialogKind::ReturnToMain => i18n.text("button.save_and_back"),
            ConfirmDialogKind::SaveSolutionBeforeEdit => i18n.text("button.save_solution_and_edit"),
        },
        ConfirmDialogAction::Secondary => match kind {
            ConfirmDialogKind::ReturnToMain => i18n.text("button.discard_and_back"),
            ConfirmDialogKind::SaveSolutionBeforeEdit => {
                i18n.text("button.discard_solution_and_edit")
            }
            _ => String::new(),
        },
        ConfirmDialogAction::Cancel => i18n.text("button.cancel"),
    }
}
