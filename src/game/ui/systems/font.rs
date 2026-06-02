#[derive(Resource, Clone)]
pub struct UiFont(pub Handle<Font>);

pub fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFont(asset_server.load("fonts/PingFangSC-Regular.ttf")));
}

pub fn apply_ui_font(
    ui_font: Option<Res<UiFont>>,
    mut text_query: Query<&mut TextFont, Added<Text>>,
) {
    let Some(ui_font) = ui_font else {
        return;
    };

    for mut font in &mut text_query {
        font.font = FontSource::Handle(ui_font.0.clone());
    }
}
