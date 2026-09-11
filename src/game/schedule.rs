//! 应用调度契约：业务插件声明所属阶段，性能观测不决定业务顺序。
use bevy::prelude::*;

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
}
