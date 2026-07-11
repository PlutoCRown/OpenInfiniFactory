use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

use bevy::prelude::Resource;

use crate::game::simulation::core::simulate_turn;

use super::snapshot::{CachedTurn, SimSnapshot};

/// 后台预取线程：按运行配置提前算未来回合
#[derive(Resource)]
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

    pub fn configure(&self, display_turn: u64, running: bool, step_requested: bool, active: bool) {
        let _ = self.command_tx.send(WorkerCommand::Configure {
            display_turn,
            running,
            step_requested,
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
    let mut snapshot = SimSnapshot::from_world(
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    let mut simulated_through = 0_u64;

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
                    let output = simulate_turn(
                        &mut snapshot.world,
                        &mut snapshot.pending_generated,
                        &mut snapshot.signal_cache,
                        next_turn,
                        &mut snapshot.structure_state,
                        &mut snapshot.movement_influence,
                        &mut snapshot.pusher_state,
                        None,
                        None,
                    );
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
