use bevy::prelude::*;

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct GameplayCamera;

pub const MENU_CLEAR: Color = Color::srgb(0.58, 0.68, 0.76);

pub fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(MENU_CLEAR),
            ..default()
        },
        Msaa::Off,
        IsDefaultUiCamera,
        UiCamera,
    ));
}

pub fn configure_ui_camera_for_playing(
    mut ui_cameras: Query<(Entity, &mut Camera), With<UiCamera>>,
    mut commands: Commands,
) {
    if let Ok((entity, mut camera)) = ui_cameras.single_mut() {
        camera.is_active = false;
        commands.entity(entity).remove::<IsDefaultUiCamera>();
    }
}

pub fn configure_ui_camera_for_start_menu(
    mut ui_cameras: Query<(Entity, &mut Camera), With<UiCamera>>,
    mut commands: Commands,
) {
    if let Ok((entity, mut camera)) = ui_cameras.single_mut() {
        camera.is_active = true;
        camera.order = 0;
        camera.clear_color = ClearColorConfig::Custom(MENU_CLEAR);
        commands.entity(entity).insert(IsDefaultUiCamera);
    }
}
