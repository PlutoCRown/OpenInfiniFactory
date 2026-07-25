#[derive(Resource, Clone)]
pub struct UiFont(pub Handle<Font>);

pub fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    // MiSans 可变字体：常规与标题加粗共用同一文件，靠 FontWeight 区分
    commands.insert_resource(UiFont(asset_server.load("fonts/MiSansVF.ttf")));
}

pub fn apply_ui_font(
    ui_font: Option<Res<UiFont>>,
    mut text_query: Query<
        &mut TextFont,
        (
            Or<(Added<Text>, Added<bevy::text::EditableText>)>,
            Without<crate::game::systems::debug::DebugText>,
            Without<crate::game::ui::types::PanelTitleText>,
        ),
    >,
) {
    let Some(ui_font) = ui_font else {
        return;
    };

    for mut font in &mut text_query {
        font.font = ui_font.0.clone().into();
    }
}
