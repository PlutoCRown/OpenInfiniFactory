use bevy::picking::prelude::{Out, Over, Pointer};
use bevy::prelude::*;

use crate::game::ui::types::UiHoverState;

pub fn ui_hovered(event: On<Pointer<Over>>, mut hover: ResMut<UiHoverState>) {
    hover.entity = Some(event.entity);
}

pub fn ui_unhovered(event: On<Pointer<Out>>, mut hover: ResMut<UiHoverState>) {
    if hover.entity == Some(event.entity) {
        hover.entity = None;
    }
}
