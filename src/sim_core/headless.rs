use bevy::prelude::*;

use super::plugin::SimCorePlugin;

/// Headless Bevy App: ECS resources only, no window or rendering.
pub fn build_headless_sim_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(SimCorePlugin);
    app.finish();
    app
}
