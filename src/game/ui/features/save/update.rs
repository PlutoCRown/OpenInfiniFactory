use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageFormat, ImageType};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, block_on, futures_lite::future};

use crate::game::state::GameMode;
use crate::game::ui::access::UiContext;
use crate::game::ui::components::{
    BUTTON_BG, BUTTON_HOVER_BG, DisabledButton, disabled_border, hover_border, pressed_border,
    raised_border,
};
use crate::game::ui::core::{StartMenuPage, UiNavigation};
use crate::game::ui::screens::{spawn_save_puzzle_row, spawn_save_solution_card};
use crate::game::ui::types::{
    SaveListAction, SaveListCloseButton, SaveListCoverHost, SaveListCoverImage,
    SaveListCoverLoading, SaveListFavoriteStar, SaveListFreeHint, SaveListPuzzleRows,
    SaveListPuzzleScroll, SaveListRenderState, SaveListRowKind, SaveListRowMeta, SaveListRowName,
    SaveListSaveRow, SaveListSolutionRows, SaveListSolutionScroll, SaveListSolutionSection,
    SaveListTitleText, UiHoverState,
};
use crate::shared::save::{SaveKind, SaveSlot, SaveState, read_cover_png};

use super::view::{
    SaveListViewCtx, save_list_puzzle_rows, save_list_title, selected_top_level_kind,
};

fn save_list_visible(mode: &State<GameMode>, navigation: &UiNavigation) -> bool {
    *mode.get() == GameMode::StartMenu && navigation.start_menu() == StartMenuPage::SaveList
}

/// 重建谜题/方案行，并刷新标题
pub fn update_save_list_rows(
    ui_context: UiContext,
    mode: Res<State<GameMode>>,
    navigation: Res<UiNavigation>,
    save_state: Res<SaveState>,
    mut render_state: ResMut<SaveListRenderState>,
    mut commands: Commands,
    mut titles: Query<&mut Text, With<SaveListTitleText>>,
    puzzle_rows_query: Query<Entity, With<SaveListPuzzleRows>>,
    solution_rows_query: Query<Entity, With<SaveListSolutionRows>>,
    children_query: Query<&Children>,
) {
    let locale_changed = ui_context.locale_changed();
    let _ui_scope = ui_context.enter();
    if !save_list_visible(&mode, &navigation) {
        return;
    }

    let structure_changed = mode.is_changed() || navigation.is_changed() || save_state.is_changed();
    if !structure_changed
        && !locale_changed
        && !render_state.paint_buttons
        && !row_hosts_stale(
            puzzle_rows_query.iter(),
            &children_query,
            render_state.puzzle_keys.len(),
        )
        && !row_hosts_stale(
            solution_rows_query.iter(),
            &children_query,
            render_state.solution_keys.len()
                + usize::from(
                    save_state.selected_puzzle.is_some()
                        && selected_top_level_kind(&save_state) != Some(SaveKind::Free),
                ),
        )
    {
        render_state.rows_rebuilt = false;
        return;
    }

    let puzzle_rows = save_list_puzzle_rows(&save_state);
    let free_selected = selected_top_level_kind(&save_state) == Some(SaveKind::Free);
    let solution_rows = if free_selected {
        Vec::new()
    } else {
        save_state
            .selected_puzzle_solutions()
            .iter()
            .filter_map(|entry| entry.slot.solution.clone())
            .collect::<Vec<_>>()
    };

    let puzzle_rows_stale =
        row_hosts_stale(puzzle_rows_query.iter(), &children_query, puzzle_rows.len())
            || render_state.puzzle_keys != puzzle_rows;
    let solution_expected = if free_selected {
        0
    } else if save_state.selected_puzzle.is_some() {
        solution_rows.len() + 1
    } else {
        0
    };
    let solution_rows_stale = row_hosts_stale(
        solution_rows_query.iter(),
        &children_query,
        solution_expected,
    ) || render_state.solution_keys != solution_rows;

    if structure_changed || locale_changed {
        let title = save_list_title();
        for mut text in &mut titles {
            if text.0 != title {
                text.0 = title.clone();
            }
        }
    }

    let mut rebuilt = false;
    if puzzle_rows_stale {
        if puzzle_rows_query.is_empty() {
            render_state.paint_buttons = true;
        } else {
            for entity in &puzzle_rows_query {
                commands.entity(entity).despawn_related::<Children>();
                commands.entity(entity).with_children(|parent| {
                    for name in &puzzle_rows {
                        spawn_save_puzzle_row(parent, name.clone());
                    }
                });
            }
            render_state.puzzle_keys = puzzle_rows;
            rebuilt = true;
        }
    }

    if solution_rows_stale {
        if solution_rows_query.is_empty() {
            render_state.paint_buttons = true;
        } else {
            for entity in &solution_rows_query {
                commands.entity(entity).despawn_related::<Children>();
                commands.entity(entity).with_children(|parent| {
                    for name in &solution_rows {
                        spawn_save_solution_card(parent, Some(name.clone()));
                    }
                    if !free_selected && save_state.selected_puzzle.is_some() {
                        spawn_save_solution_card(parent, None);
                    }
                });
            }
            render_state.solution_keys = solution_rows;
            rebuilt = true;
        }
    }

    if rebuilt {
        render_state.paint_buttons = true;
        render_state.rows_rebuilt = true;
    } else {
        render_state.rows_rebuilt = false;
    }
}

/// 刷新封面图（后台读盘解码，不阻塞点选；含 object-fit: cover 尺寸）
pub fn update_save_list_cover(
    mode: Res<State<GameMode>>,
    navigation: Res<UiNavigation>,
    save_state: Res<SaveState>,
    mut render_state: ResMut<SaveListRenderState>,
    mut images: ResMut<Assets<Image>>,
    mut cover_images: Query<(&mut ImageNode, &mut Node), With<SaveListCoverImage>>,
    mut cover_loading: Query<&mut Visibility, With<SaveListCoverLoading>>,
    cover_hosts: Query<&ComputedNode, With<SaveListCoverHost>>,
) {
    if !save_list_visible(&mode, &navigation) {
        render_state.cover_task = None;
        return;
    }

    let cover_slot = match selected_top_level_kind(&save_state) {
        Some(SaveKind::Free) => save_state
            .selected_puzzle
            .as_ref()
            .map(|name| SaveSlot::free(name.clone())),
        Some(SaveKind::Puzzle) => {
            if let Some(solution) = save_state.selected_solution.as_ref() {
                save_state
                    .selected_puzzle
                    .as_ref()
                    .map(|puzzle| SaveSlot::solution(puzzle.clone(), solution.clone()))
            } else {
                save_state
                    .selected_puzzle
                    .as_ref()
                    .map(|puzzle| SaveSlot::puzzle(puzzle.clone()))
            }
        }
        _ => None,
    };
    let next_key = cover_slot
        .as_ref()
        .map(|slot| slot.storage_path())
        .unwrap_or_default();
    let key_changed = render_state.cover_key.as_deref() != Some(next_key.as_str());

    if key_changed {
        render_state.cover_key = Some(next_key.clone());
        render_state.cover_task = None;
        for (mut image_node, mut node) in cover_images.iter_mut() {
            *image_node = ImageNode::default();
            node.display = Display::None;
        }
        let show_loading = !next_key.is_empty();
        for mut visibility in cover_loading.iter_mut() {
            *visibility = if show_loading {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if let Some(slot) = cover_slot {
            let key = next_key.clone();
            render_state.cover_task = Some(AsyncComputeTaskPool::get().spawn(async move {
                let image = read_cover_png(&slot).and_then(|bytes| {
                    Image::from_buffer(
                        &bytes,
                        ImageType::Format(ImageFormat::Png),
                        CompressedImageFormats::NONE,
                        true,
                        bevy::image::ImageSampler::Default,
                        RenderAssetUsages::default(),
                    )
                    .ok()
                });
                (key, image)
            }));
        }
    }

    if let Some(mut task) = render_state.cover_task.take() {
        match block_on(future::poll_once(&mut task)) {
            Some((done_key, image_opt)) => {
                if render_state.cover_key.as_deref() == Some(done_key.as_str()) {
                    for mut visibility in cover_loading.iter_mut() {
                        *visibility = Visibility::Hidden;
                    }
                    match image_opt {
                        Some(image) => {
                            let handle = images.add(image);
                            for (mut image_node, mut node) in cover_images.iter_mut() {
                                *image_node = ImageNode {
                                    image: handle.clone(),
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                };
                                node.display = Display::Flex;
                            }
                        }
                        None => {
                            for (mut image_node, mut node) in cover_images.iter_mut() {
                                *image_node = ImageNode::default();
                                node.display = Display::None;
                            }
                        }
                    }
                }
            }
            None => {
                render_state.cover_task = Some(task);
            }
        }
    }

    for (image_node, mut node) in cover_images.iter_mut() {
        if node.display == Display::None {
            continue;
        }
        let Some(host) = cover_hosts.iter().next() else {
            continue;
        };
        if host.is_empty() {
            continue;
        }
        let Some(image) = images.get(&image_node.image) else {
            continue;
        };
        let size = image.size();
        if size.x == 0 || size.y == 0 {
            continue;
        }
        let inv = host.inverse_scale_factor();
        let host_w = host.size().x * inv;
        let host_h = host.size().y * inv;
        if host_w <= 1.0 || host_h <= 1.0 {
            continue;
        }
        let img_aspect = size.x as f32 / size.y as f32;
        let host_aspect = host_w / host_h;
        let (w, h) = if host_aspect > img_aspect {
            (host_w, host_w / img_aspect)
        } else {
            (host_h * img_aspect, host_h)
        };
        node.width = Val::Px(w);
        node.height = Val::Px(h);
        node.left = Val::Px((host_w - w) * 0.5);
        node.top = Val::Px((host_h - h) * 0.5);
        node.position_type = PositionType::Absolute;
    }
}

/// 谜题纵滑 + 方案横滑
pub fn update_save_list_scroll(
    mode: Res<State<GameMode>>,
    navigation: Res<UiNavigation>,
    hover: Res<UiHoverState>,
    children_query: Query<&Children>,
    parents: Query<&ChildOf>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut puzzle_scroll: Query<(Entity, &mut SaveListPuzzleScroll, &ComputedNode, &Children)>,
    mut puzzle_content: Query<&mut Node, (With<SaveListPuzzleRows>, Without<SaveListSolutionRows>)>,
    mut solution_scroll: Query<(
        Entity,
        &mut SaveListSolutionScroll,
        &ComputedNode,
        &Children,
    )>,
    mut solution_content: Query<
        &mut Node,
        (With<SaveListSolutionRows>, Without<SaveListPuzzleRows>),
    >,
) {
    if !save_list_visible(&mode, &navigation) {
        mouse_wheel.clear();
        return;
    }

    let wheel: f32 = mouse_wheel.read().map(|e| e.y).sum();
    update_puzzle_vscroll(
        wheel,
        &mut puzzle_scroll,
        &mut puzzle_content,
        hover.entity,
        &children_query,
        &parents,
    );
    update_solution_hscroll(
        wheel,
        &mut solution_scroll,
        &mut solution_content,
        hover.entity,
        &children_query,
        &parents,
    );
}

/// 刷新按钮样式与文案，并同步 Free/Puzzle 区域显隐
pub fn update_save_list_styles(
    ui_context: UiContext,
    mode: Res<State<GameMode>>,
    navigation: Res<UiNavigation>,
    save_state: Res<SaveState>,
    hover: Res<UiHoverState>,
    mut render_state: ResMut<SaveListRenderState>,
    mut commands: Commands,
    mut texts: Query<&mut Text, (Without<SaveListTitleText>, Without<SaveListFavoriteStar>)>,
    kind_texts: Query<(), With<SaveListRowKind>>,
    name_texts: Query<(), With<SaveListRowName>>,
    meta_texts: Query<(), With<SaveListRowMeta>>,
    mut star_texts: Query<(&mut Text, &mut TextColor), With<SaveListFavoriteStar>>,
    mut meta_nodes: Query<&mut Node, (With<SaveListRowMeta>, Without<Button>)>,
    mut buttons: Query<
        (
            Entity,
            &SaveListAction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Node,
            Option<&SaveListSaveRow>,
            Option<&DisabledButton>,
        ),
        (With<Button>, Without<SaveListCloseButton>),
    >,
    mut solution_sections: Query<
        &mut Node,
        (
            With<SaveListSolutionSection>,
            Without<Button>,
            Without<SaveListRowMeta>,
        ),
    >,
    mut free_hints: Query<
        (&mut Node, &mut Visibility),
        (
            With<SaveListFreeHint>,
            Without<Button>,
            Without<SaveListSolutionSection>,
            Without<SaveListRowMeta>,
        ),
    >,
) {
    let locale_changed = ui_context.locale_changed();
    let _ui_scope = ui_context.enter();
    if !save_list_visible(&mode, &navigation) {
        return;
    }

    let structure_changed = mode.is_changed() || navigation.is_changed() || save_state.is_changed();
    let paint_labels = structure_changed || locale_changed || render_state.paint_buttons;
    let style_changed =
        structure_changed || locale_changed || hover.is_changed() || render_state.paint_buttons;
    if !render_state.rows_rebuilt {
        render_state.paint_buttons = false;
    }

    let free_selected = selected_top_level_kind(&save_state) == Some(SaveKind::Free);
    if structure_changed {
        for mut node in &mut solution_sections {
            node.display = if free_selected {
                Display::None
            } else {
                Display::Flex
            };
        }
        for (mut node, mut visibility) in &mut free_hints {
            if free_selected {
                node.display = Display::Flex;
                *visibility = Visibility::Visible;
            } else {
                node.display = Display::None;
                *visibility = Visibility::Hidden;
            }
        }
    }

    if !style_changed {
        return;
    }

    let ctx = SaveListViewCtx {
        save_state: &save_state,
    };
    let previous_hover = render_state.last_hover;
    render_state.last_hover = hover.entity;
    for (entity, action, children, mut background, mut border, mut node, save_row, disabled) in
        &mut buttons
    {
        if !paint_labels && Some(entity) != hover.entity && Some(entity) != previous_hover {
            continue;
        }
        let view = action.button_view(&ctx);
        if let Some(display) = view.display {
            if node.display != display {
                node.display = display;
            }
        } else if node.display == Display::None {
            node.display = Display::Flex;
        }
        let hovered = view.enabled && hover.entity == Some(entity);

        if save_row.is_some() {
            *background = if view.selected {
                Color::srgba(0.38, 0.28, 0.08, 0.98).into()
            } else if hovered {
                Color::srgba(0.20, 0.20, 0.21, 0.98).into()
            } else {
                Color::srgba(0.12, 0.12, 0.14, 0.96).into()
            };
            *border = BorderColor::all(Color::NONE);
        } else {
            *background = if view.enabled && view.selected {
                Color::srgba(0.22, 0.35, 0.32, 0.96).into()
            } else if hovered {
                BUTTON_HOVER_BG.into()
            } else if view.enabled {
                BUTTON_BG.into()
            } else {
                Color::srgba(0.12, 0.12, 0.13, 0.82).into()
            };
            *border = if view.selected {
                pressed_border()
            } else if hovered {
                hover_border()
            } else if view.enabled {
                raised_border()
            } else {
                disabled_border()
            };
        }

        // 同步禁用标记，挡住全局 HoverButton 的按下/悬停反馈
        match (view.enabled, disabled.is_some()) {
            (false, false) => {
                commands.entity(entity).insert(DisabledButton);
            }
            (true, true) => {
                commands.entity(entity).remove::<DisabledButton>();
            }
            _ => {}
        }

        if paint_labels {
            for child in children.iter() {
                if name_texts.contains(child) {
                    if let Ok(mut text) = texts.get_mut(child) {
                        if text.0 != view.label {
                            text.0 = view.label.clone();
                        }
                    }
                } else if kind_texts.contains(child) {
                    if let Ok(mut text) = texts.get_mut(child) {
                        let kind = view.kind.clone().unwrap_or_default();
                        if text.0 != kind {
                            text.0 = kind;
                        }
                    }
                } else if meta_texts.contains(child) {
                    let meta = view.meta.clone().unwrap_or_default();
                    if let Ok(mut text) = texts.get_mut(child) {
                        if text.0 != meta {
                            text.0 = meta;
                        }
                    }
                    if let Ok(mut node) = meta_nodes.get_mut(child) {
                        let display = if view.meta.is_some() {
                            Display::Flex
                        } else {
                            Display::None
                        };
                        if node.display != display {
                            node.display = display;
                        }
                    }
                } else if let Ok((mut text, mut color)) = star_texts.get_mut(child) {
                    let next_color = if view.favorite {
                        Color::srgb(1.0, 0.78, 0.12)
                    } else {
                        Color::srgb(0.45, 0.45, 0.47)
                    };
                    if text.0 != "★" {
                        text.0 = "★".to_string();
                    }
                    if color.0 != next_color {
                        color.0 = next_color;
                    }
                } else if let Ok(mut text) = texts.get_mut(child) {
                    // 页脚等无分栏按钮：整段写 label
                    if text.0 != view.label {
                        text.0 = view.label.clone();
                    }
                }
            }
        }
    }
}

fn update_puzzle_vscroll(
    wheel: f32,
    puzzle_scroll: &mut Query<(Entity, &mut SaveListPuzzleScroll, &ComputedNode, &Children)>,
    puzzle_content: &mut Query<
        &mut Node,
        (With<SaveListPuzzleRows>, Without<SaveListSolutionRows>),
    >,
    hover_entity: Option<Entity>,
    children_query: &Query<&Children>,
    parents: &Query<&ChildOf>,
) {
    for (scroll_entity, mut scroll, host, children) in puzzle_scroll.iter_mut() {
        let Some(content_entity) = children
            .iter()
            .find(|child| puzzle_content.get(*child).is_ok())
        else {
            continue;
        };
        let Ok(mut content) = puzzle_content.get_mut(content_entity) else {
            continue;
        };
        let row_count = children_query
            .get(content_entity)
            .map(|c| c.len())
            .unwrap_or(0) as f32;
        let content_h = row_count * 47.0 + 8.0;
        let host_h = if host.is_empty() {
            0.0
        } else {
            host.size().y * host.inverse_scale_factor()
        };
        scroll.max_offset = (content_h - host_h).max(0.0);
        let over = hover_entity.is_some_and(|entity| is_descendant(entity, scroll_entity, parents));
        if wheel.abs() > f32::EPSILON && over {
            scroll.offset = (scroll.offset - wheel * 32.0).clamp(0.0, scroll.max_offset);
        } else {
            scroll.offset = scroll.offset.clamp(0.0, scroll.max_offset);
        }
        let next = Val::Px(-scroll.offset);
        if content.top != next {
            content.top = next;
        }
    }
}

fn update_solution_hscroll(
    wheel: f32,
    solution_scroll: &mut Query<(
        Entity,
        &mut SaveListSolutionScroll,
        &ComputedNode,
        &Children,
    )>,
    solution_content: &mut Query<
        &mut Node,
        (With<SaveListSolutionRows>, Without<SaveListPuzzleRows>),
    >,
    hover_entity: Option<Entity>,
    children_query: &Query<&Children>,
    parents: &Query<&ChildOf>,
) {
    for (scroll_entity, mut scroll, host, children) in solution_scroll.iter_mut() {
        let Some(content_entity) = children
            .iter()
            .find(|child| solution_content.get(*child).is_ok())
        else {
            continue;
        };
        let Ok(mut content) = solution_content.get_mut(content_entity) else {
            continue;
        };
        let card_count = children_query
            .get(content_entity)
            .map(|c| c.len())
            .unwrap_or(0) as f32;
        let content_w = card_count * 148.0 + 12.0;
        let host_w = if host.is_empty() {
            0.0
        } else {
            host.size().x * host.inverse_scale_factor()
        };
        scroll.max_offset = (content_w - host_w).max(0.0);
        let over = hover_entity.is_some_and(|entity| is_descendant(entity, scroll_entity, parents));
        if wheel.abs() > f32::EPSILON && over {
            scroll.offset = (scroll.offset - wheel * 40.0).clamp(0.0, scroll.max_offset);
        } else {
            scroll.offset = scroll.offset.clamp(0.0, scroll.max_offset);
        }
        let next = Val::Px(-scroll.offset);
        if content.left != next {
            content.left = next;
        }
    }
}

fn is_descendant(entity: Entity, ancestor: Entity, parents: &Query<&ChildOf>) -> bool {
    let mut current = entity;
    loop {
        if current == ancestor {
            return true;
        }
        let Ok(parent) = parents.get(current) else {
            return false;
        };
        current = parent.parent();
    }
}

fn row_hosts_stale(
    hosts: impl IntoIterator<Item = Entity>,
    children: &Query<&Children>,
    expected_len: usize,
) -> bool {
    let mut any = false;
    for entity in hosts {
        any = true;
        let count = children.get(entity).map(|c| c.len()).unwrap_or(0);
        if count != expected_len {
            return true;
        }
    }
    !any && expected_len > 0
}
