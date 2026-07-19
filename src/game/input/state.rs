//! 每帧游玩输入快照（键盘 ∪ 虚拟遥感）

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use crate::game::player::controller::MouseLookBaseline;
use crate::shared::config::{ActionKeyName, GameConfig};
use crate::shared::touch_profile::TouchProfile;

/// 离散动作的按下边沿
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionPulse {
    pub just_pressed: bool,
    pub pressed: bool,
    pub just_released: bool,
}

impl ActionPulse {
    pub fn from_key(keys: &ButtonInput<KeyCode>, key: KeyCode) -> Self {
        Self {
            just_pressed: keys.just_pressed(key),
            pressed: keys.pressed(key),
            just_released: keys.just_released(key),
        }
    }

    pub fn from_mouse(buttons: &ButtonInput<MouseButton>, button: MouseButton) -> Self {
        Self {
            just_pressed: buttons.just_pressed(button),
            pressed: buttons.pressed(button),
            just_released: buttons.just_released(button),
        }
    }

    pub fn or_with(&mut self, other: Self) {
        self.just_pressed |= other.just_pressed;
        self.pressed |= other.pressed;
        self.just_released |= other.just_released;
    }
}

/// 本帧游玩输入（由键盘与虚拟遥感共同写入）
#[derive(Resource, Clone, Debug, Default)]
pub struct GameplayInputState {
    pub move_axis: Vec2,
    pub look_delta: Vec2,
    pub jump: ActionPulse,
    pub fly_up: bool,
    pub fly_down: bool,
    pub place: ActionPulse,
    pub delete: ActionPulse,
    pub pick: ActionPulse,
    pub cancel_edit_gesture: bool,
    pub open_block_config: bool,
    pub pause: bool,
    pub inventory: bool,
    pub rotate: bool,
    pub alternate: bool,
    pub simulate: bool,
    pub sim_step: bool,
    pub sim_fast: bool,
    pub rollback: bool,
    /// 虚拟遥感本帧写入的额外移动/视角（gather 末尾合并）
    pub virtual_move_axis: Vec2,
    pub virtual_look_delta: Vec2,
    pub virtual_jump: ActionPulse,
    pub virtual_fly_up: bool,
    pub virtual_fly_down: bool,
    pub virtual_place: ActionPulse,
    pub virtual_delete: ActionPulse,
    pub virtual_cancel_edit: bool,
    pub virtual_open_block_config: bool,
    pub virtual_pause: bool,
    pub virtual_inventory: bool,
    pub virtual_rotate: bool,
    pub virtual_alternate: bool,
    pub virtual_simulate: bool,
    pub virtual_sim_step: bool,
    pub virtual_sim_fast: bool,
    pub virtual_rollback: bool,
}

impl GameplayInputState {
    /// 清空连续轴类虚拟贡献（放置/删除边沿由 virtual remote 系统写入，此处不清）
    pub fn clear_virtual_axes(&mut self) {
        self.virtual_move_axis = Vec2::ZERO;
        self.virtual_look_delta = Vec2::ZERO;
        self.virtual_jump = ActionPulse::default();
        self.virtual_fly_up = false;
        self.virtual_fly_down = false;
    }
}

/// 从键盘鼠标采集并与虚拟遥感合并
pub fn gather_gameplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut look_baseline: ResMut<MouseLookBaseline>,
    config: Res<GameConfig>,
    touch: Res<TouchProfile>,
    mut input: ResMut<GameplayInputState>,
) {
    let bindings = &config.key_bindings;
    let mut move_axis = Vec2::ZERO;
    if keys.pressed(bindings.forward.key_code()) {
        move_axis.y += 1.0;
    }
    if keys.pressed(bindings.backward.key_code()) {
        move_axis.y -= 1.0;
    }
    if keys.pressed(bindings.right.key_code()) {
        move_axis.x += 1.0;
    }
    if keys.pressed(bindings.left.key_code()) {
        move_axis.x -= 1.0;
    }

    let mut look_delta = Vec2::ZERO;
    if !touch.enabled {
        for event in mouse_motion.read() {
            if look_baseline.resync_on_next_motion {
                if event.delta != Vec2::ZERO {
                    // 丢掉回中并进的那次位移，基准视为已在中心
                    look_baseline.resync_on_next_motion = false;
                }
                continue;
            }
            look_delta += event.delta;
        }
    } else {
        mouse_motion.clear();
    }

    let place_button = config
        .input(ActionKeyName::Place)
        .mouse_button()
        .unwrap_or(MouseButton::Left);
    let delete_button = config
        .input(ActionKeyName::Delete)
        .mouse_button()
        .unwrap_or(MouseButton::Right);
    let pick_button = config
        .input(ActionKeyName::Pick)
        .mouse_button()
        .unwrap_or(MouseButton::Middle);

    let jump = ActionPulse::from_key(&keys, bindings.jump_or_fly_up.key_code());
    let fly_up = keys.pressed(bindings.jump_or_fly_up.key_code());
    let fly_down = keys.pressed(bindings.fly_down.key_code());

    // 合并虚拟贡献（虚拟系统在本系统之前写入 virtual_*）
    let virtual_move = input.virtual_move_axis;
    let virtual_look = input.virtual_look_delta;
    let virtual_jump = input.virtual_jump;
    let virtual_fly_up = input.virtual_fly_up;
    let virtual_fly_down = input.virtual_fly_down;
    let virtual_place = input.virtual_place;
    let virtual_delete = input.virtual_delete;
    let virtual_cancel = input.virtual_cancel_edit;
    let virtual_open = input.virtual_open_block_config;
    let virtual_pause = input.virtual_pause;
    let virtual_inventory = input.virtual_inventory;
    let virtual_rotate = input.virtual_rotate;
    let virtual_alternate = input.virtual_alternate;
    let virtual_simulate = input.virtual_simulate;
    let virtual_sim_step = input.virtual_sim_step;
    let virtual_sim_fast = input.virtual_sim_fast;
    let virtual_rollback = input.virtual_rollback;

    let mut place = if touch.enabled {
        ActionPulse::default()
    } else {
        ActionPulse::from_mouse(&mouse_buttons, place_button)
    };
    let mut delete = if touch.enabled {
        ActionPulse::default()
    } else {
        ActionPulse::from_mouse(&mouse_buttons, delete_button)
    };
    place.or_with(virtual_place);
    delete.or_with(virtual_delete);

    let mut merged_jump = jump;
    merged_jump.or_with(virtual_jump);

    *input = GameplayInputState {
        move_axis: {
            let mut axis = move_axis + virtual_move;
            if axis.length_squared() > 1.0 {
                axis = axis.normalize();
            }
            axis
        },
        look_delta: look_delta + virtual_look,
        jump: merged_jump,
        fly_up: fly_up || virtual_fly_up,
        fly_down: fly_down || virtual_fly_down,
        place,
        delete,
        pick: ActionPulse::from_mouse(&mouse_buttons, pick_button),
        cancel_edit_gesture: virtual_cancel,
        open_block_config: virtual_open,
        pause: keys.just_pressed(bindings.pause.key_code()) || virtual_pause,
        inventory: keys.just_pressed(bindings.inventory.key_code()) || virtual_inventory,
        rotate: keys.just_pressed(bindings.rotate_or_rollback.key_code()) || virtual_rotate,
        alternate: keys.just_pressed(bindings.alternate.key_code()) || virtual_alternate,
        simulate: keys.just_pressed(bindings.simulate.key_code()) || virtual_simulate,
        sim_step: keys.just_pressed(bindings.simulation_step.key_code()) || virtual_sim_step,
        sim_fast: keys.pressed(bindings.simulation_fast.key_code()) || virtual_sim_fast,
        rollback: keys.just_pressed(bindings.simulation_rollback.key_code()) || virtual_rollback,
        ..default()
    };
}
