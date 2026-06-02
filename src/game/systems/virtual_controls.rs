use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::{GameMode, SimulationState};
use crate::game::ui::UiRuntime;
use crate::game::world::rendering::GameplayRuntimeEntity;

const ENABLED_ON_THIS_PLATFORM: bool = cfg!(target_arch = "wasm32");
const JOYSTICK_RADIUS: f32 = 58.0;
const JOYSTICK_DEAD_ZONE: f32 = 8.0;
const LEFT_STICK_CENTER: Vec2 = Vec2::new(92.0, 120.0);
const LOOK_ZONE_MIN_WIDTH: f32 = 240.0;
const LOOK_SENSITIVITY: f32 = 1.25;
const ACTION_BUTTON_SIZE: f32 = 54.0;
const ACTION_BUTTON_GAP: f32 = 12.0;
const ACTION_BUTTON_RIGHT: f32 = 22.0;
const ACTION_BUTTON_BOTTOM: f32 = 34.0;

#[derive(Resource, Default)]
pub struct VirtualControls {
    pub movement: Vec2,
    pub look_delta: Vec2,
    pub place: VirtualButton,
    pub delete: VirtualButton,
    pub jump_or_fly_up: VirtualButton,
    pub fly_down: VirtualButton,
    pub rotate_or_rollback: VirtualButton,
    pub inventory: VirtualButton,
    pub alternate: VirtualButton,
}

impl VirtualControls {
    fn clear_frame_edges(&mut self) {
        self.look_delta = Vec2::ZERO;
        self.place.clear_edges();
        self.delete.clear_edges();
        self.jump_or_fly_up.clear_edges();
        self.fly_down.clear_edges();
        self.rotate_or_rollback.clear_edges();
        self.inventory.clear_edges();
        self.alternate.clear_edges();
    }

    fn release_all(&mut self) {
        self.movement = Vec2::ZERO;
        self.look_delta = Vec2::ZERO;
        self.place.set_pressed(false);
        self.delete.set_pressed(false);
        self.jump_or_fly_up.set_pressed(false);
        self.fly_down.set_pressed(false);
        self.rotate_or_rollback.set_pressed(false);
        self.inventory.set_pressed(false);
        self.alternate.set_pressed(false);
    }
}

#[derive(Default, Clone, Copy)]
pub struct VirtualButton {
    pub pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

impl VirtualButton {
    fn clear_edges(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
    }

    fn set_pressed(&mut self, pressed: bool) {
        if pressed == self.pressed {
            return;
        }
        self.pressed = pressed;
        if pressed {
            self.just_pressed = true;
        } else {
            self.just_released = true;
        }
    }
}

#[derive(Resource, Default)]
pub struct VirtualTouchState {
    movement_touch: Option<u64>,
    look_touch: Option<u64>,
}

#[derive(Component)]
pub struct VirtualControlsOverlay;

#[derive(Component, Clone, Copy)]
pub struct VirtualControlVisual(VirtualControlVisualKind);

#[derive(Clone, Copy)]
enum VirtualControlVisualKind {
    MovementBase,
    MovementKnob,
    Button(VirtualControlButton),
}

#[derive(Clone, Copy)]
enum VirtualControlButton {
    Place,
    Delete,
    Jump,
    Down,
    Rotate,
    Inventory,
    Alternate,
}

pub fn spawn_virtual_controls_ui(commands: &mut Commands) {
    if !ENABLED_ON_THIS_PLATFORM {
        return;
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
            GlobalZIndex(80),
            VirtualControlsOverlay,
            GameplayRuntimeEntity,
        ))
        .with_children(|root| {
            root.spawn((
                control_disc_node(LEFT_STICK_CENTER, JOYSTICK_RADIUS),
                BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.28)),
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                VirtualControlVisual(VirtualControlVisualKind::MovementBase),
            ));
            root.spawn((
                control_disc_node(LEFT_STICK_CENTER, 22.0),
                BackgroundColor(Color::srgba(0.92, 0.96, 1.0, 0.46)),
                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.22)),
                VirtualControlVisual(VirtualControlVisualKind::MovementKnob),
            ));

            spawn_action_button(root, VirtualControlButton::Place, "P", 0, 0);
            spawn_action_button(root, VirtualControlButton::Delete, "D", 1, 0);
            spawn_action_button(root, VirtualControlButton::Jump, "^", 0, 1);
            spawn_action_button(root, VirtualControlButton::Down, "v", 1, 1);
            spawn_action_button(root, VirtualControlButton::Rotate, "R", 0, 2);
            spawn_action_button(root, VirtualControlButton::Inventory, "I", 1, 2);
            spawn_action_button(root, VirtualControlButton::Alternate, "C", 2, 2);
        });
}

pub fn update_virtual_controls(
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mode: Res<GameMode>,
    ui_runtime: Res<UiRuntime>,
    mut state: ResMut<VirtualTouchState>,
    mut controls: ResMut<VirtualControls>,
) {
    controls.clear_frame_edges();

    if !ENABLED_ON_THIS_PLATFORM || *mode != GameMode::Playing || ui_runtime.blocks_gameplay() {
        state.movement_touch = None;
        state.look_touch = None;
        controls.release_all();
        return;
    }

    let Ok(window) = windows.single() else {
        controls.release_all();
        return;
    };
    let screen_size = Vec2::new(window.width(), window.height());
    if screen_size.x <= 0.0 || screen_size.y <= 0.0 {
        controls.release_all();
        return;
    }

    let left_center = Vec2::new(LEFT_STICK_CENTER.x, screen_size.y - LEFT_STICK_CENTER.y);
    let look_zone_x = (screen_size.x - LOOK_ZONE_MIN_WIDTH).max(screen_size.x * 0.5);

    for touch in touches.iter_just_pressed() {
        let pos = touch.position();
        if state.movement_touch.is_none() && pos.distance(left_center) <= JOYSTICK_RADIUS * 1.9 {
            state.movement_touch = Some(touch.id());
        } else if state.look_touch.is_none()
            && pos.x >= look_zone_x
            && !action_button_at(pos, screen_size).is_some()
        {
            state.look_touch = Some(touch.id());
        }
    }

    if state
        .movement_touch
        .is_some_and(|id| touches.get_pressed(id).is_none())
    {
        state.movement_touch = None;
    }
    if state
        .look_touch
        .is_some_and(|id| touches.get_pressed(id).is_none())
    {
        state.look_touch = None;
    }

    controls.movement = state
        .movement_touch
        .and_then(|id| touches.get_pressed(id))
        .map(|touch| normalized_stick(touch.position() - left_center))
        .unwrap_or(Vec2::ZERO);

    if let Some(touch) = state.look_touch.and_then(|id| touches.get_pressed(id)) {
        controls.look_delta += touch.delta() * LOOK_SENSITIVITY;
    }

    let mut place = false;
    let mut delete = false;
    let mut jump = false;
    let mut down = false;
    let mut rotate = false;
    let mut inventory = false;
    let mut alternate = false;
    for touch in touches.iter() {
        if Some(touch.id()) == state.movement_touch || Some(touch.id()) == state.look_touch {
            continue;
        }
        match action_button_at(touch.position(), screen_size) {
            Some(VirtualControlButton::Place) => place = true,
            Some(VirtualControlButton::Delete) => delete = true,
            Some(VirtualControlButton::Jump) => jump = true,
            Some(VirtualControlButton::Down) => down = true,
            Some(VirtualControlButton::Rotate) => rotate = true,
            Some(VirtualControlButton::Inventory) => inventory = true,
            Some(VirtualControlButton::Alternate) => alternate = true,
            None => {}
        }
    }

    controls.place.set_pressed(place);
    controls.delete.set_pressed(delete);
    controls.jump_or_fly_up.set_pressed(jump);
    controls.fly_down.set_pressed(down);
    controls.rotate_or_rollback.set_pressed(rotate);
    controls.inventory.set_pressed(inventory);
    controls.alternate.set_pressed(alternate);
}

pub fn update_virtual_controls_ui(
    controls: Res<VirtualControls>,
    mode: Res<GameMode>,
    simulation: Res<SimulationState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut overlays: Query<&mut Node, (With<VirtualControlsOverlay>, Without<VirtualControlVisual>)>,
    mut visuals: Query<(
        &mut Node,
        &mut BackgroundColor,
        &VirtualControlVisual,
        Option<&Children>,
    )>,
    mut text_query: Query<&mut Text, Without<VirtualControlVisual>>,
) {
    if !ENABLED_ON_THIS_PLATFORM {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let screen_size = Vec2::new(window.width(), window.height());
    let visible = *mode == GameMode::Playing;
    for mut node in &mut overlays {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !visible {
        return;
    }

    let left_center = Vec2::new(LEFT_STICK_CENTER.x, screen_size.y - LEFT_STICK_CENTER.y);
    let knob_center = left_center + controls.movement * JOYSTICK_RADIUS;
    for (mut node, mut background, visual, children) in &mut visuals {
        match visual.0 {
            VirtualControlVisualKind::MovementBase => {
                position_disc(&mut node, LEFT_STICK_CENTER, JOYSTICK_RADIUS);
            }
            VirtualControlVisualKind::MovementKnob => {
                position_disc_from_top(&mut node, knob_center, 22.0);
            }
            VirtualControlVisualKind::Button(button) => {
                let pressed = virtual_button_pressed(&controls, button);
                background.0 = if pressed {
                    Color::srgba(0.18, 0.22, 0.25, 0.72)
                } else {
                    Color::srgba(0.05, 0.06, 0.07, 0.46)
                };
                if let Some(children) = children {
                    for child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(child) {
                            text.0 = button_label(button, simulation.running).to_string();
                        }
                    }
                }
            }
        }
    }
}

fn normalized_stick(delta: Vec2) -> Vec2 {
    if delta.length() <= JOYSTICK_DEAD_ZONE {
        return Vec2::ZERO;
    }
    let scaled = delta / JOYSTICK_RADIUS;
    Vec2::new(scaled.x, -scaled.y).clamp_length_max(1.0)
}

fn control_disc_node(bottom_left_center: Vec2, radius: f32) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(radius * 2.0),
        height: Val::Px(radius * 2.0),
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::MAX,
        ..default()
    };
    position_disc(&mut node, bottom_left_center, radius);
    node
}

fn position_disc(node: &mut Node, bottom_left_center: Vec2, radius: f32) {
    node.left = Val::Px(bottom_left_center.x - radius);
    node.bottom = Val::Px(bottom_left_center.y - radius);
    node.top = Val::Auto;
    node.right = Val::Auto;
}

fn position_disc_from_top(node: &mut Node, top_left_center: Vec2, radius: f32) {
    node.left = Val::Px(top_left_center.x - radius);
    node.top = Val::Px(top_left_center.y - radius);
    node.bottom = Val::Auto;
    node.right = Val::Auto;
}

fn spawn_action_button(
    root: &mut ChildSpawnerCommands,
    button: VirtualControlButton,
    label: &'static str,
    column: u32,
    row: u32,
) {
    let step = ACTION_BUTTON_SIZE + ACTION_BUTTON_GAP;
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(ACTION_BUTTON_SIZE),
            height: Val::Px(ACTION_BUTTON_SIZE),
            right: Val::Px(ACTION_BUTTON_RIGHT + column as f32 * step),
            bottom: Val::Px(ACTION_BUTTON_BOTTOM + row as f32 * step),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::MAX,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.46)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.24)),
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
        VirtualControlVisual(VirtualControlVisualKind::Button(button)),
    ))
    .with_children(|button| {
        button.spawn((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.86)),
        ));
    });
}

fn action_button_at(pos: Vec2, screen_size: Vec2) -> Option<VirtualControlButton> {
    [
        (VirtualControlButton::Place, 0, 0),
        (VirtualControlButton::Delete, 1, 0),
        (VirtualControlButton::Jump, 0, 1),
        (VirtualControlButton::Down, 1, 1),
        (VirtualControlButton::Rotate, 0, 2),
        (VirtualControlButton::Inventory, 1, 2),
        (VirtualControlButton::Alternate, 2, 2),
    ]
    .into_iter()
    .find_map(|(button, column, row)| {
        let step = ACTION_BUTTON_SIZE + ACTION_BUTTON_GAP;
        let left = screen_size.x - ACTION_BUTTON_RIGHT - ACTION_BUTTON_SIZE - column as f32 * step;
        let top = screen_size.y - ACTION_BUTTON_BOTTOM - ACTION_BUTTON_SIZE - row as f32 * step;
        (pos.x >= left
            && pos.x <= left + ACTION_BUTTON_SIZE
            && pos.y >= top
            && pos.y <= top + ACTION_BUTTON_SIZE)
            .then_some(button)
    })
}

fn virtual_button_pressed(controls: &VirtualControls, button: VirtualControlButton) -> bool {
    match button {
        VirtualControlButton::Place => controls.place.pressed,
        VirtualControlButton::Delete => controls.delete.pressed,
        VirtualControlButton::Jump => controls.jump_or_fly_up.pressed,
        VirtualControlButton::Down => controls.fly_down.pressed,
        VirtualControlButton::Rotate => controls.rotate_or_rollback.pressed,
        VirtualControlButton::Inventory => controls.inventory.pressed,
        VirtualControlButton::Alternate => controls.alternate.pressed,
    }
}

fn button_label(button: VirtualControlButton, simulation_running: bool) -> &'static str {
    match button {
        VirtualControlButton::Place => "P",
        VirtualControlButton::Delete => "D",
        VirtualControlButton::Jump => "^",
        VirtualControlButton::Down => "v",
        VirtualControlButton::Rotate if simulation_running => "Undo",
        VirtualControlButton::Rotate => "R",
        VirtualControlButton::Inventory => "I",
        VirtualControlButton::Alternate => "C",
    }
}
