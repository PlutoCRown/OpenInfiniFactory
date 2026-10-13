//! 应用调度契约：业务插件声明所属阶段，性能观测不决定业务顺序。
use bevy::prelude::*;

/// 启动阶段的依赖：配置先就绪，再加载内容，最后挂载界面和音频。
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStartupSet {
    Configuration,
    Content,
    Presentation,
}

/// 每帧的业务阶段；会话替换在模拟推进前完成，表现只消费当前会话结果。
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    PreUpdateRest,
    VirtualRemote,
    InputGather,
    PlayerMove,
    Hover,
    Placement,
    Menus,
    Session,
    SimulationControls,
    Simulation,
    Presentation,
    View,
    Animation,
    UiInventory,
    UiStatus,
    UiChrome,
    UiFeat,
    Debug,
}

/// 安装应用阶段顺序；可在没有性能插件的无窗口 App 中使用。
pub struct GameSchedulePlugin;

impl Plugin for GameSchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Startup,
            (
                GameStartupSet::Configuration,
                GameStartupSet::Content,
                GameStartupSet::Presentation,
            )
                .chain(),
        );
        app.configure_sets(
            Update,
            (
                GameSet::PreUpdateRest,
                GameSet::VirtualRemote,
                GameSet::InputGather,
                GameSet::PlayerMove,
                GameSet::Hover,
                GameSet::Placement,
                GameSet::Menus,
                GameSet::Session,
                GameSet::SimulationControls,
                GameSet::Simulation,
                GameSet::Presentation,
                GameSet::View,
                GameSet::Animation,
                GameSet::UiInventory,
                GameSet::UiStatus,
                GameSet::UiChrome,
                GameSet::UiFeat,
                GameSet::Debug,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 记录业务阶段的实际顺序，测试不依赖性能插件或窗口。
    #[derive(Resource, Default)]
    struct Trace(Vec<&'static str>);

    /// 菜单提交的延迟请求必须先落地，后续会话阶段才能读取。
    #[derive(Resource)]
    struct MenuRequest;

    /// 反向注册启动工作，配置与资源包仍须先于界面和音乐初始化。
    #[test]
    fn startup_configuration_and_content_precede_presentation() {
        let mut app = App::new();
        app.add_plugins(GameSchedulePlugin).init_resource::<Trace>();
        app.add_systems(
            Startup,
            (
                (|mut trace: ResMut<Trace>| trace.0.push("ui/audio"))
                    .in_set(GameStartupSet::Presentation),
                (|mut trace: ResMut<Trace>| trace.0.push("packs")).in_set(GameStartupSet::Content),
                (|mut trace: ResMut<Trace>| trace.0.push("config"))
                    .in_set(GameStartupSet::Configuration),
            ),
        );
        app.update();
        assert_eq!(
            app.world().resource::<Trace>().0,
            ["config", "packs", "ui/audio"]
        );
    }

    /// 按逆序注册系统，仍应遵循应用契约并应用菜单命令。
    #[test]
    fn session_commit_precedes_simulation_without_perf_plugin() {
        let mut app = App::new();
        app.add_plugins(GameSchedulePlugin).init_resource::<Trace>();
        app.add_systems(
            Update,
            (
                (|mut trace: ResMut<Trace>| trace.0.push("present")).in_set(GameSet::Presentation),
                (|mut trace: ResMut<Trace>| trace.0.push("turn")).in_set(GameSet::Simulation),
                (|mut trace: ResMut<Trace>| trace.0.push("controls"))
                    .in_set(GameSet::SimulationControls),
                (|_: Res<MenuRequest>, mut trace: ResMut<Trace>| trace.0.push("session"))
                    .in_set(GameSet::Session),
                (|mut commands: Commands, mut trace: ResMut<Trace>| {
                    commands.insert_resource(MenuRequest);
                    trace.0.push("menu");
                })
                .in_set(GameSet::Menus),
            ),
        );
        app.update();
        assert_eq!(
            app.world().resource::<Trace>().0,
            ["menu", "session", "controls", "turn", "present"]
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 初始化真实业务插件的调度图，检查跨插件循环和查询冲突，不启动窗口或渲染。
    #[test]
    fn gameplay_and_ui_schedule_initializes_without_rendering() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<crate::game::state::GameMode>()
            .add_plugins((
                GameSchedulePlugin,
                crate::game::input::GameplayInputPlugin,
                crate::game::systems::gameplay::GameplayPlugin,
                crate::game::session::SessionPlugin,
                crate::sim_bridge::SimulationBridgePlugin,
                crate::game::ui::GameUiPlugin,
                crate::game::systems::perf::PerfPlugin,
            ));
        app.world_mut().schedule_scope(Update, |world, schedule| {
            schedule
                .initialize(world)
                .expect("business schedule must have no cycles or query conflicts");
        });
    }

    /// 组合完整游戏插件，检查资源归属调整后的启动及更新调度图。
    #[test]
    fn complete_game_plugin_graph_initializes_without_gpu_or_window() {
        use bevy::asset::{AssetApp, AssetPlugin};
        use bevy::audio::AudioSource;
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::state::app::StatesPlugin,
        ))
        .init_asset::<bevy::shader::Shader>()
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<Font>()
        .init_asset::<StandardMaterial>()
        .init_asset::<AudioSource>()
        .init_resource::<crate::shared::launch::LaunchOptions>()
        .add_plugins(crate::game::GamePlugin);
        app.finish();
        app.cleanup();
        assert!(
            app.world()
                .contains_resource::<crate::shared::save::PuzzleLighting>()
        );
        assert!(
            app.world()
                .contains_resource::<crate::shared::config::GameConfig>()
        );
        assert!(
            app.world()
                .contains_resource::<crate::game::ui::InventoryItems>()
        );
        assert!(
            app.world()
                .contains_resource::<crate::scene::BlockEntityIndex>()
        );
        app.world_mut().schedule_scope(Startup, |world, schedule| {
            schedule
                .initialize(world)
                .expect("startup graph must be acyclic");
        });
        app.world_mut().schedule_scope(Update, |world, schedule| {
            schedule
                .initialize(world)
                .expect("complete game graph must be acyclic and queries valid");
        });
        app.world_mut()
            .schedule_scope(PostUpdate, |world, schedule| {
                schedule
                    .initialize(world)
                    .expect("post-update graph must have valid queries");
            });
    }
}
