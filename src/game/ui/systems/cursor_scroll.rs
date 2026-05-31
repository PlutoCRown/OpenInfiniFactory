pub fn update_carried_item_ui(
    carried: Res<CarriedItem>,
    i18n: Res<I18n>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut preview: Query<(&mut Node, &mut BackgroundColor, &Children), With<CarriedItemPreview>>,
    mut preview_images: Query<&mut ImageNode>,
    mut preview_text: Query<&mut Text>,
) {
    let Ok((mut style, mut background, children)) = preview.single_mut() else {
        return;
    };

    let Some(item) = carried.item() else {
        style.display = Display::None;
        for child in children.iter() {
            if let Ok(mut image) = preview_images.get_mut(child) {
                *image = ImageNode::default();
            }
            if let Ok(mut text) = preview_text.get_mut(child) {
                text.0.clear();
            }
        }
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        style.display = Display::None;
        return;
    };

    style.display = Display::Flex;
    style.left = Val::Px(cursor.x + 4.0);
    style.top = Val::Px(cursor.y + 4.0);
    *background = slot_color(item).with_alpha(0.9).into();

    let icon_handle = item
        .block()
        .and_then(|kind| block_icons.as_deref().and_then(|icons| icons.get(kind)));
    for child in children.iter() {
        if let Ok(mut image) = preview_images.get_mut(child) {
            *image = icon_handle
                .as_ref()
                .map(|handle| ImageNode::new(handle.clone()))
                .unwrap_or_default();
        }
        if let Ok(mut text) = preview_text.get_mut(child) {
            text.0 = if icon_handle.is_some() {
                String::new()
            } else {
                i18n.text(short_item_name(item))
            };
        }
    }
}

pub fn update_scroll_containers(
    ui_runtime: Res<UiRuntime>,
    _settings_tab: Res<SettingsTab>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut containers: Query<(&mut ScrollContainer, &Children, &ComputedNode)>,
    mut contents: Query<(&mut Node, &ComputedNode), With<ScrollContent>>,
) {
    if !ui_runtime.is_settings_open() {
        return;
    }

    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();

    for (mut container, children, node) in &mut containers {
        let Some(child) = children.iter().find(|child| contents.get(*child).is_ok()) else {
            continue;
        };
        let Ok((mut style, content_node)) = contents.get_mut(child) else {
            continue;
        };

        container.max_offset = (content_node.size().y - node.size().y).max(0.0);
        if wheel_delta.abs() > f32::EPSILON {
            container.offset =
                (container.offset - wheel_delta * 32.0).clamp(0.0, container.max_offset);
        } else {
            container.offset = container.offset.clamp(0.0, container.max_offset);
        }
        style.top = Val::Px(-container.offset);
    }
}
