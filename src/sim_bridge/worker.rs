//! 模拟预取 worker：桌面后台线程；Web 无线程，在 poll 时主线程推进。

use bevy::prelude::Resource;

use crate::game::simulation::core::simulate_turn;

use super::snapshot::{CachedTurn, SimSnapshot};

/// 预取入口：平台分流，对外 API 一致
#[derive(Resource)]
pub struct SimulationWorker {
    #[cfg(not(target_arch = "wasm32"))]
    backend: threaded::Backend,
    #[cfg(target_arch = "wasm32")]
    backend: inline::Backend,
}

impl SimulationWorker {
    pub fn spawn() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            backend: threaded::Backend::spawn(),
            #[cfg(target_arch = "wasm32")]
            backend: inline::Backend::new(),
        }
    }

    pub fn reset(&self, snapshot: SimSnapshot, display_turn: u64) {
        self.backend.reset(snapshot, display_turn);
    }

    pub fn configure(&self, display_turn: u64, running: bool, step_requested: bool, active: bool) {
        self.backend
            .configure(display_turn, running, step_requested, active);
    }

    pub fn drain_results(&self) -> Vec<CachedTurn> {
        self.backend.drain_results()
    }
}

fn game_prefetch_depth() -> u64 {
    2
}

/// 按配置从当前快照向前预取若干回合
fn prefetch_turns(
    snapshot: &mut SimSnapshot,
    simulated_through: &mut u64,
    display_turn: u64,
    running: bool,
    step_requested: bool,
) -> Vec<CachedTurn> {
    let mut out = Vec::new();
    if !running && !step_requested {
        return out;
    }
    let depth = if running { game_prefetch_depth() } else { 1 };
    let target = display_turn + depth;
    while *simulated_through < target {
        let next_turn = *simulated_through + 1;
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
        *simulated_through = next_turn;
        out.push(CachedTurn {
            output,
            after: snapshot.clone(),
        });
        if !running && !step_requested {
            break;
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
mod threaded {
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::Mutex;
    use std::thread::{self, JoinHandle};

    use super::{prefetch_turns, CachedTurn, SimSnapshot};

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

    pub struct Backend {
        command_tx: Sender<WorkerCommand>,
        result_rx: Mutex<Receiver<CachedTurn>>,
        _thread: JoinHandle<()>,
    }

    impl Backend {
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
            active: bool,
        ) {
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

    impl Drop for Backend {
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
                    let batch = prefetch_turns(
                        &mut snapshot,
                        &mut simulated_through,
                        display_turn,
                        running,
                        step_requested,
                    );
                    for turn in batch {
                        if result_tx.send(turn).is_err() {
                            return;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod inline {
    use std::sync::Mutex;

    use super::{prefetch_turns, CachedTurn, SimSnapshot};

    struct State {
        snapshot: SimSnapshot,
        simulated_through: u64,
        pending: Vec<CachedTurn>,
    }

    pub struct Backend {
        state: Mutex<State>,
    }

    impl Backend {
        pub fn new() -> Self {
            Self {
                state: Mutex::new(State {
                    snapshot: SimSnapshot::from_world(
                        &Default::default(),
                        &Default::default(),
                        &Default::default(),
                        &Default::default(),
                        &Default::default(),
                        &Default::default(),
                    ),
                    simulated_through: 0,
                    pending: Vec::new(),
                }),
            }
        }

        pub fn reset(&self, snapshot: SimSnapshot, display_turn: u64) {
            let mut state = self.state.lock().expect("inline sim worker lock");
            state.snapshot = snapshot;
            state.simulated_through = display_turn;
            state.pending.clear();
        }

        pub fn configure(
            &self,
            display_turn: u64,
            running: bool,
            step_requested: bool,
            active: bool,
        ) {
            if !active {
                return;
            }
            let mut guard = self.state.lock().expect("inline sim worker lock");
            let state = &mut *guard;
            let batch = prefetch_turns(
                &mut state.snapshot,
                &mut state.simulated_through,
                display_turn,
                running,
                step_requested,
            );
            state.pending.extend(batch);
        }

        pub fn drain_results(&self) -> Vec<CachedTurn> {
            std::mem::take(&mut self.state.lock().expect("inline sim worker lock").pending)
        }
    }
}
