use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, SpatialAudioSink,
    Volume,
};
use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::blocks::BlockKind;
use crate::game::player::controller::FlyCamera;
use crate::game::state::{GameSettings, SimulationState};
use crate::game::world::grid::{WorldBlocks, grid_to_world};

const MAX_SOUND_DISTANCE: f32 = 24.0;

/// 游戏音频事件：只描述声音类型和世界位置，不直接操作音频实体。
#[derive(Message, Clone, Copy)]
pub struct PlaySound {
    pub sound: SoundId,
    pub position: Option<Vec3>,
    pub gain: f32,
}

/// 游戏内可播放的基础音效分类。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SoundId {
    UiClick,
    BlockPlace,
    BlockBreak,
    MachineWork,
    MachineMotor,
    Weld,
    DrillBreak,
    SciFi,
    Acceptance,
}

/// 启动时加载的音频资源集合，具体声音以后可以继续替换为正式素材。
#[derive(Resource)]
struct SoundAssets {
    music: Handle<AudioSource>,
    ui_click: Handle<AudioSource>,
    block_place: Handle<AudioSource>,
    block_break: Handle<AudioSource>,
    machine_work: Handle<AudioSource>,
    machine_motor: Handle<AudioSource>,
    weld: Handle<AudioSource>,
    drill_break: Handle<AudioSource>,
    sci_fi: Handle<AudioSource>,
    acceptance: Handle<AudioSource>,
}

impl FromWorld for SoundAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            music: asset_server.load("audio/music.wav"),
            ui_click: asset_server.load("audio/ui_click.wav"),
            block_place: asset_server.load("audio/block_place.wav"),
            block_break: asset_server.load("audio/block_break.wav"),
            machine_work: asset_server.load("audio/machine_work.wav"),
            machine_motor: asset_server.load("audio/machine_motor.wav"),
            weld: asset_server.load("audio/weld.wav"),
            drill_break: asset_server.load("audio/drill_break.wav"),
            sci_fi: asset_server.load("audio/sci_fi.wav"),
            acceptance: asset_server.load("audio/acceptance.wav"),
        }
    }
}

impl SoundAssets {
    fn source(&self, sound: SoundId) -> Handle<AudioSource> {
        match sound {
            SoundId::UiClick => self.ui_click.clone(),
            SoundId::BlockPlace => self.block_place.clone(),
            SoundId::BlockBreak => self.block_break.clone(),
            SoundId::MachineWork => self.machine_work.clone(),
            SoundId::MachineMotor => self.machine_motor.clone(),
            SoundId::Weld => self.weld.clone(),
            SoundId::DrillBreak => self.drill_break.clone(),
            SoundId::SciFi => self.sci_fi.clone(),
            SoundId::Acceptance => self.acceptance.clone(),
        }
    }
}

/// 音乐播放实体，音量变化时直接更新 sink。
#[derive(Component)]
struct MusicAudio;

/// 模拟期间持续播放的机器底噪实体。
#[derive(Component)]
struct MachineAudioLoop {
    pos: IVec3,
}

/// 注册音频消息、资源和空间音频系统。
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .init_resource::<SoundAssets>()
            .add_systems(Startup, start_music)
            .add_systems(
                Update,
                (
                    play_sound_events,
                    sync_machine_audio_loops,
                    update_music_volume,
                    update_machine_audio_volumes,
                )
                    .chain(),
            );
    }
}

/// 启动全局循环音乐。
fn start_music(mut commands: Commands, assets: Res<SoundAssets>, settings: Res<GameSettings>) {
    commands.spawn((
        MusicAudio,
        AudioPlayer::new(assets.music.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(
            settings.master_volume * settings.music_volume,
        )),
    ));
}

/// 把音频事件转换成一次性空间音频实体。
fn play_sound_events(
    mut commands: Commands,
    mut events: MessageReader<PlaySound>,
    assets: Res<SoundAssets>,
    settings: Res<GameSettings>,
    listener: Query<&GlobalTransform, With<FlyCamera>>,
) {
    let listener_position = listener
        .single()
        .ok()
        .map(|transform| transform.translation());
    for event in events.read() {
        let Some(position) = event.position else {
            commands.spawn((
                AudioPlayer::new(assets.source(event.sound)),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(
                    settings.master_volume * settings.sfx_volume * event.gain,
                )),
            ));
            continue;
        };

        let distance_gain = listener_position
            .map(|listener| distance_gain(listener.distance(position)))
            .unwrap_or(1.0);
        commands.spawn((
            AudioPlayer::new(assets.source(event.sound)),
            PlaybackSettings::DESPAWN
                .with_spatial(true)
                .with_volume(Volume::Linear(
                    settings.master_volume * settings.sfx_volume * event.gain * distance_gain,
                )),
            Transform::from_translation(position),
        ));
    }
}

/// 让传送带和钻头只在模拟运行且通电时拥有持续底噪。
fn sync_machine_audio_loops(
    mut commands: Commands,
    assets: Res<SoundAssets>,
    settings: Res<GameSettings>,
    simulation: Res<SimulationState>,
    world: Res<WorldBlocks>,
    loops: Query<(Entity, &MachineAudioLoop)>,
) {
    let active_positions: HashSet<IVec3> = if simulation.running {
        world
            .blocks
            .iter()
            .filter_map(|(&pos, block)| {
                let motor = matches!(
                    block.kind,
                    BlockKind::Conveyor | BlockKind::ReverseConveyor | BlockKind::Drill
                );
                (motor && simulation.last_powered_devices.contains(&pos)).then_some(pos)
            })
            .collect()
    } else {
        HashSet::new()
    };

    for (entity, audio_loop) in &loops {
        if !active_positions.contains(&audio_loop.pos) {
            commands.entity(entity).despawn();
        }
    }

    let existing_positions: HashSet<IVec3> =
        loops.iter().map(|(_, audio_loop)| audio_loop.pos).collect();
    for pos in active_positions.difference(&existing_positions) {
        commands.spawn((
            MachineAudioLoop { pos: *pos },
            AudioPlayer::new(assets.machine_motor.clone()),
            PlaybackSettings::LOOP
                .with_spatial(true)
                .with_volume(Volume::Linear(settings.master_volume * settings.sfx_volume)),
            Transform::from_translation(grid_to_world(*pos)),
        ));
    }
}

/// 实时更新音乐音量，覆盖总音量和音乐分音量设置。
fn update_music_volume(
    settings: Res<GameSettings>,
    mut sinks: Query<&mut AudioSink, With<MusicAudio>>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(
            settings.master_volume * settings.music_volume,
        ));
    }
}

/// 实时更新机器底噪的距离衰减与音量设置。
fn update_machine_audio_volumes(
    settings: Res<GameSettings>,
    listener: Query<&GlobalTransform, With<FlyCamera>>,
    mut loops: Query<(&Transform, &mut SpatialAudioSink), With<MachineAudioLoop>>,
) {
    let Some(listener) = listener
        .single()
        .ok()
        .map(|transform| transform.translation())
    else {
        return;
    };
    for (transform, mut sink) in &mut loops {
        sink.set_volume(Volume::Linear(
            settings.master_volume
                * settings.sfx_volume
                * distance_gain(listener.distance(transform.translation)),
        ));
    }
}

/// 把世界距离映射为线性音量，超过最大范围后静音。
fn distance_gain(distance: f32) -> f32 {
    (1.0 - (distance / MAX_SOUND_DISTANCE).clamp(0.0, 1.0)).powi(2)
}
