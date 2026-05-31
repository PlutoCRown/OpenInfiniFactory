pub fn update_inventory_slots(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    i18n: Res<I18n>,
    block_icons: Option<Res<BlockIconAssets>>,
    mut slot_query: Query<
        (
            &InventorySlot,
            &Interaction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut texts: ParamSet<(
        Query<&mut Text, Without<CarriedItemPreview>>,
        Query<&mut Text, Without<CarriedItemPreview>>,
    )>,
    mut icons: Query<&mut ImageNode>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tooltip: Query<(&mut Node, &Children), With<InventoryTooltip>>,
) {
    let mut hovered_item = None;
    for (slot, interaction, children, mut background, mut border) in &mut slot_query {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };
        let icon_handle = item
            .and_then(|item| item.block())
            .and_then(|kind| block_icons.as_deref().and_then(|icons| icons.get(kind)));
        let has_icon = icon_handle.is_some();
        if *interaction == Interaction::Hovered {
            hovered_item = item;
        }

        let selected_hotbar = slot.area == SlotArea::Hotbar && slot.index == placement.selected;
        let base_color = item
            .map(slot_color)
            .unwrap_or(Color::srgb(0.255, 0.251, 0.251));
        *background = if has_icon && *interaction == Interaction::Hovered {
            Color::srgb(0.32, 0.31, 0.31).into()
        } else if has_icon {
            Color::srgb(0.255, 0.251, 0.251).into()
        } else if *interaction == Interaction::Hovered && item.is_none() {
            Color::srgb(0.32, 0.31, 0.31).into()
        } else if *interaction == Interaction::Hovered {
            base_color.with_alpha(1.0).into()
        } else {
            base_color.into()
        };
        *border = if selected_hotbar {
            BorderColor {
                top: Color::srgb(1.0, 0.94, 0.80),
                left: Color::srgb(1.0, 0.94, 0.80),
                right: Color::srgb(0.36, 0.25, 0.12),
                bottom: Color::srgb(0.36, 0.25, 0.12),
            }
        } else if *interaction == Interaction::Hovered {
            hover_border()
        } else {
            inset_border()
        };

        for child in children.iter() {
            if let Ok(mut text) = texts.p0().get_mut(child) {
                text.0 = if has_icon {
                    String::new()
                } else {
                    item.map(|kind| i18n.text(short_item_name(kind)))
                        .unwrap_or_default()
                };
            }
            if let Ok(mut image) = icons.get_mut(child) {
                *image = icon_handle.clone().map(ImageNode::new).unwrap_or_default();
            }
        }
    }

    let Ok((mut tooltip_node, tooltip_children)) = tooltip.single_mut() else {
        return;
    };
    let Some(item) = hovered_item else {
        tooltip_node.display = Display::None;
        return;
    };
    let Ok(window) = windows.single() else {
        tooltip_node.display = Display::None;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        tooltip_node.display = Display::None;
        return;
    };

    tooltip_node.display = Display::Flex;
    tooltip_node.left = Val::Px(cursor.x + 16.0);
    tooltip_node.top = Val::Px(cursor.y + 16.0);
    for child in tooltip_children.iter() {
        if let Ok(mut text) = texts.p1().get_mut(child) {
            text.0 = i18n.text(item.name_key());
        }
    }
}
