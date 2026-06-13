//! Registers Bevy ECS `Resource` for types shared by the game client and headless sim App.
//!
//! Two Bevy runtimes coexist:
//! - **Game client** (`main`): window, UI, 3D scene + full simulation resources
//! - **Headless sim** (`oif-debug-http`): `SimCorePlugin` only — ECS resources, no window/render

use bevy::prelude::*;

use crate::shared::launch::LaunchOptions;
use crate::sim_core::{SimulationWorker, TurnCache};

impl Resource for TurnCache {}
impl Resource for LaunchOptions {}
impl Resource for SimulationWorker {}
impl Resource for crate::game::simulation::runtime::SimulationPresentationState {}
