use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct GameplayCamera;

/// 游玩 UI 叠加相机：只向窗口画 2D UI，不画 3D
#[derive(Component)]
pub struct PlayingUiCamera;

/// 全屏显示 3D 离屏纹理的 UI 底图
#[derive(Component)]
pub struct GameplayViewBackdrop;

/// 3D 视角离屏渲染目标，封面截图直接读此 Image
#[derive(Resource, Clone)]
pub struct GameplayViewImage(pub Handle<Image>);

pub const MENU_CLEAR: Color = Color::srgb(0.58, 0.68, 0.76);

pub fn gameplay_view_size(window: &Window) -> (u32, u32) {
    (
        window.physical_width().max(1),
        window.physical_height().max(1),
    )
}

pub fn new_gameplay_view_image(width: u32, height: u32) -> Image {
    Image::new_target_texture(
        width,
        height,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    )
}

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

pub fn sync_gameplay_view_image_size(
    window: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut images: ResMut<Assets<Image>>,
    view: Option<Res<GameplayViewImage>>,
) {
    let Some(view) = view else {
        return;
    };
    let Ok(window) = window.single() else {
        return;
    };
    let (width, height) = gameplay_view_size(window);
    let Some(mut image) = images.get_mut(&view.0) else {
        return;
    };
    if image.width() == width && image.height() == height {
        return;
    }
    image.resize(Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    });
}
