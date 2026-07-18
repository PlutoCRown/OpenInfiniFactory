use std::time::Duration;

use bevy::camera::visibility::VisibilitySystems;
use bevy::light::SimulationLightSystems;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::transform::TransformSystems;
use bevy::ui::UiSystems;

/// 声明所有性能观测段。增删 scope 只需改 `perf_scopes! { ... }` 这一处。
macro_rules! perf_scopes {
    ($($name:ident => $label:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, SystemSet)]
        pub enum PerfScope {
            $($name),+
        }

        impl PerfScope {
            pub const ORDER: &'static [Self] = &[$(Self::$name,)+];

            pub fn label(self) -> &'static str {
                match self {
                    $(Self::$name => $label),+
                }
            }

            fn idx(self) -> usize {
                Self::ORDER
                    .iter()
                    .position(|scope| *scope == self)
                    .expect("PerfScope must appear in ORDER")
            }
        }

        $(
            paste::paste! {
                pub fn [<perf_mark_ $name:snake>](mut perf: ResMut<PerfStats>) {
                    perf.advance(PerfScope::$name);
                }
            }
        )+
    };
}

perf_scopes! {
    PreUpdate => "PreUpdate",
    Input => "Input",
    Menus => "Menus",
    Simulation => "Simulation",
    View => "View",
    Animation => "Animation",
    Ui => "UI",
    Debug => "Debug",
    PostUpdateStart => "Update tail",
    PostUpdateUi => "Post/UI layout",
    PostUpdateTransform => "Post/Transform",
    PostVisPrep => "Post/Vis prep",
    PostVisCheck => "Post/Vis check",
    PostVisLights => "Post/Vis lights",
    PostVisDone => "Post/Vis done",
    Last => "Last",
}

#[derive(Resource)]
pub struct PerfStats {
    frame_started: Instant,
    last_main_finished: Option<Instant>,
    mark: Instant,
    frame_ms: SmoothedMs,
    main_ms: SmoothedMs,
    scopes: [SmoothedMs; PerfScope::ORDER.len()],
    main_other_ms: SmoothedMs,
    render_other_ms: SmoothedMs,
    render_gap_ms: SmoothedMs,
}

impl Default for PerfStats {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            frame_started: now,
            last_main_finished: None,
            mark: now,
            frame_ms: SmoothedMs::default(),
            main_ms: SmoothedMs::default(),
            scopes: [(); PerfScope::ORDER.len()].map(|_| SmoothedMs::default()),
            main_other_ms: SmoothedMs::default(),
            render_other_ms: SmoothedMs::default(),
            render_gap_ms: SmoothedMs::default(),
        }
    }
}

#[derive(Default)]
struct SmoothedMs {
    value: f64,
    initialized: bool,
}

impl SmoothedMs {
    fn sample(&mut self, duration: Duration) {
        self.sample_ms(duration.as_secs_f64() * 1000.0);
    }

    fn sample_ms(&mut self, ms: f64) {
        if self.initialized {
            self.value = self.value * 0.86 + ms * 0.14;
        } else {
            self.value = ms;
            self.initialized = true;
        }
    }
}

impl PerfStats {
    fn advance(&mut self, scope: PerfScope) {
        let elapsed = self.mark_elapsed();
        self.scopes[scope.idx()].sample(elapsed);
    }

    fn mark_elapsed(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.mark);
        self.mark = now;
        elapsed
    }

    pub fn scope_ms(&self, scope: PerfScope) -> f64 {
        self.scopes[scope.idx()].value
    }

    pub fn frame_ms(&self) -> f64 {
        self.frame_ms.value
    }

    pub fn main_ms(&self) -> f64 {
        self.main_ms.value
    }

    pub fn main_other_ms(&self) -> f64 {
        self.main_other_ms.value
    }

    pub fn render_other_ms(&self) -> f64 {
        self.render_other_ms.value
    }

    pub fn render_gap_ms(&self) -> f64 {
        self.render_gap_ms.value
    }

    pub fn format_scope_section(&self) -> String {
        PerfScope::ORDER
            .iter()
            .map(|scope| {
                format!(
                    "  {:>22}: {:>8.2} us",
                    scope.label(),
                    micros(self.scope_ms(*scope))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn micros(ms: f64) -> f64 {
    ms * 1000.0
}

pub fn begin_perf_frame(mut perf: ResMut<PerfStats>) {
    let now = Instant::now();
    let frame_started = perf.frame_started;
    perf.frame_ms
        .sample(now.saturating_duration_since(frame_started));
    if let Some(last_main_finished) = perf.last_main_finished {
        perf.render_gap_ms
            .sample(now.saturating_duration_since(last_main_finished));
    }
    perf.frame_started = now;
    perf.mark = now;
}

pub fn finish_perf_frame(mut perf: ResMut<PerfStats>) {
    let main_ms = Instant::now()
        .saturating_duration_since(perf.frame_started)
        .as_secs_f64()
        * 1000.0;
    let measured_main_ms: f64 = PerfScope::ORDER
        .iter()
        .map(|scope| perf.scope_ms(*scope))
        .sum();
    perf.main_ms.sample_ms(main_ms);
    perf.main_other_ms
        .sample_ms((main_ms - measured_main_ms).max(0.0));
    let frame_ms = perf.frame_ms.value;
    perf.render_other_ms
        .sample_ms((frame_ms - main_ms).max(0.0));
    perf.last_main_finished = Some(Instant::now());
}

pub struct PerfPlugin;

impl Plugin for PerfPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerfStats>()
            // 只锁住曾漂进 UI 计时的后半段；Input→View 已有业务 after/before，不必整段串死
            .configure_sets(
                Update,
                (
                    PerfScope::View,
                    PerfScope::Animation,
                    PerfScope::Ui,
                    PerfScope::Debug,
                )
                    .chain(),
            )
            .add_systems(First, begin_perf_frame)
            .add_systems(
                PreUpdate,
                perf_mark_pre_update
                    .in_set(PerfScope::PreUpdate)
                    .after(UiSystems::Focus),
            )
            .add_systems(Update, perf_mark_input.in_set(PerfScope::Input))
            .add_systems(Update, perf_mark_menus.in_set(PerfScope::Menus))
            .add_systems(Update, perf_mark_simulation.in_set(PerfScope::Simulation))
            .add_systems(Update, perf_mark_view.in_set(PerfScope::View))
            .add_systems(Update, perf_mark_animation.in_set(PerfScope::Animation))
            .add_systems(Update, perf_mark_ui.in_set(PerfScope::Ui))
            .add_systems(Update, perf_mark_debug.in_set(PerfScope::Debug))
            .add_systems(
                PostUpdate,
                perf_mark_post_update_start
                    .in_set(PerfScope::PostUpdateStart)
                    .before(UiSystems::Prepare),
            )
            .add_systems(
                PostUpdate,
                perf_mark_post_update_ui
                    .in_set(PerfScope::PostUpdateUi)
                    .after(UiSystems::Layout)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                perf_mark_post_update_transform
                    .in_set(PerfScope::PostUpdateTransform)
                    .after(TransformSystems::Propagate)
                    .before(VisibilitySystems::UpdateFrusta),
            )
            .add_systems(
                PostUpdate,
                perf_mark_post_vis_prep
                    .in_set(PerfScope::PostVisPrep)
                    .after(VisibilitySystems::UpdateFrusta)
                    .after(VisibilitySystems::CalculateBounds)
                    .after(VisibilitySystems::VisibilityPropagate)
                    .before(VisibilitySystems::CheckVisibility),
            )
            .add_systems(
                PostUpdate,
                perf_mark_post_vis_check
                    .in_set(PerfScope::PostVisCheck)
                    .after(VisibilitySystems::CheckVisibility)
                    .before(SimulationLightSystems::AssignLightsToClusters)
                    .before(SimulationLightSystems::UpdateLightFrusta)
                    .before(SimulationLightSystems::CheckLightVisibility),
            )
            .add_systems(
                PostUpdate,
                perf_mark_post_vis_lights
                    .in_set(PerfScope::PostVisLights)
                    .after(SimulationLightSystems::CheckLightVisibility)
                    .before(VisibilitySystems::MarkNewlyHiddenEntitiesInvisible),
            )
            .add_systems(
                PostUpdate,
                perf_mark_post_vis_done
                    .in_set(PerfScope::PostVisDone)
                    .after(VisibilitySystems::MarkNewlyHiddenEntitiesInvisible),
            )
            .add_systems(
                Last,
                (
                    perf_mark_last
                        .in_set(PerfScope::Last)
                        .before(finish_perf_frame),
                    finish_perf_frame,
                )
                    .chain(),
            );
    }
}
