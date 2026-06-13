use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

use crate::game::simulation::core::simulate_turn;
use crate::game::simulation::SimulationWorlds;
use crate::sim_core::SimulationDebugLog;

use super::snapshot::{CachedTurn, SimSnapshot};

pub struct SimulationWorker {
    command_tx: Sender<WorkerCommand>,
    result_rx: Mutex<Receiver<CachedTurn>>,
    _thread: JoinHandle<()>,
}

enum WorkerCommand {
    Shutdown,
    Reset {
        snapshot: SimSnapshot,
        display_turn: u64,
    },
    Configure {
        display_turn: u64,
        running: bool,
        step_requested: bool,
        speed: f32,
        active: bool,
    },
}

impl SimulationWorker {
    pub fn spawn() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let thread = thread::spawn(move || worker_main(command_rx, result_tx));
        Self {
            command_tx,
            result_rx: Mutex::new(result_rx),
            _thread: thread,
        }
    }

    pub fn reset(&self, snapshot: SimSnapshot, display_turn: u64) {
        let _ = self.command_tx.send(WorkerCommand::Reset {
            snapshot,
            display_turn,
        });
    }

    pub fn configure(
        &self,
        display_turn: u64,
        running: bool,
        step_requested: bool,
        speed: f32,
        active: bool,
    ) {
        let _ = self.command_tx.send(WorkerCommand::Configure {
            display_turn,
            running,
            step_requested,
            speed,
            active,
        });
    }

    pub fn drain_results(&self) -> Vec<CachedTurn> {
        self.result_rx
            .lock()
            .expect("simulation worker results lock")
            .try_iter()
            .collect()
    }
}

impl Drop for SimulationWorker {
    fn drop(&mut self) {
        let _ = self.command_tx.send(WorkerCommand::Shutdown);
    }
}

fn worker_main(command_rx: Receiver<WorkerCommand>, result_tx: Sender<CachedTurn>) {
    let mut snapshot = SimSnapshot::at_simulation_start(
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    let mut simulated_through = 0_u64;
    let mut sim_log = SimulationDebugLog::default();
    sim_log.set_enabled(false);

    while let Ok(command) = command_rx.recv() {
        match command {
            WorkerCommand::Shutdown => break,
            WorkerCommand::Reset {
                snapshot: state,
                display_turn,
            } => {
                snapshot = state;
                simulated_through = display_turn;
            }
            WorkerCommand::Configure {
                display_turn,
                running,
                step_requested,
                speed: _,
                active,
            } => {
                if !active {
                    continue;
                }
                if !running && !step_requested {
                    continue;
                }
                let depth = if running { game_prefetch_depth() } else { 1 };
                let target = display_turn + depth;
                while simulated_through < target {
                    let next_turn = simulated_through + 1;
                    let mut worlds = SimulationWorlds::from_snapshot_parts(
                        snapshot.solution.clone(),
                        snapshot.solution_structures.clone(),
                        snapshot.world.clone(),
                        snapshot.structure_state.clone(),
                        snapshot.factory_registry.clone(),
                        snapshot.material_registry.clone(),
                    );
                    let output = simulate_turn(
                        &mut worlds,
                        &mut snapshot.pending_generated,
                        &mut snapshot.signal_cache,
                        next_turn,
                        &mut snapshot.pusher_state,
                        &mut snapshot.movement_influence,
                        None,
                        None,
                    );
                    snapshot.world = worlds.turn;
                    snapshot.structure_state = worlds.turn_structures;
                    snapshot.factory_registry = worlds.factory_registry;
                    snapshot.material_registry = worlds.material_registry;
                    simulated_through = next_turn;
                    let after = snapshot.clone();
                    if result_tx.send(CachedTurn { output, after }).is_err() {
                        return;
                    }
                    if !running && !step_requested {
                        break;
                    }
                }
            }
        }
    }
}

fn game_prefetch_depth() -> u64 {
    2
}
