impl GameConfig {
    /// 统一校验配置数值，损坏或非有限值回到默认设置。
    pub fn normalized(mut self) -> Self {
        for (value, min, max, default) in [
            (
                &mut self.fov_degrees,
                50.0,
                110.0,
                DEFAULT_CONFIG.fov_degrees,
            ),
            (
                &mut self.ui_scale,
                UI_SCALE_MIN,
                UI_SCALE_MAX,
                DEFAULT_CONFIG.ui_scale,
            ),
            (
                &mut self.gravity_scale,
                GRAVITY_SCALE_MIN,
                GRAVITY_SCALE_MAX,
                DEFAULT_CONFIG.gravity_scale,
            ),
            (
                &mut self.mouse_sensitivity_x,
                MOUSE_SENSITIVITY_MIN,
                MOUSE_SENSITIVITY_MAX,
                DEFAULT_CONFIG.mouse_sensitivity_x,
            ),
            (
                &mut self.mouse_sensitivity_y,
                MOUSE_SENSITIVITY_MIN,
                MOUSE_SENSITIVITY_MAX,
                DEFAULT_CONFIG.mouse_sensitivity_y,
            ),
            (
                &mut self.virtual_controls_opacity,
                0.0,
                1.0,
                DEFAULT_CONFIG.virtual_controls_opacity,
            ),
            (
                &mut self.master_volume,
                0.0,
                1.0,
                DEFAULT_CONFIG.master_volume,
            ),
            (
                &mut self.music_volume,
                0.0,
                1.0,
                DEFAULT_CONFIG.music_volume,
            ),
            (&mut self.sfx_volume, 0.0, 1.0, DEFAULT_CONFIG.sfx_volume),
        ] {
            *value = if value.is_finite() {
                value.clamp(min, max)
            } else {
                default
            };
        }
        self
    }

    pub fn chord(&self, action: ActionKeyName) -> ConfigChord {
        match action {
            ActionKeyName::Undo => self.key_bindings.undo,
            ActionKeyName::Redo => self.key_bindings.redo,
            ActionKeyName::Copy => self.key_bindings.copy,
            ActionKeyName::Paste => self.key_bindings.paste,
            ActionKeyName::ToggleSelectionTool => self.key_bindings.toggle_selection_tool,
            _ => ConfigChord::default_undo(),
        }
    }

    pub fn binding_display(&self, action: ActionKeyName) -> String {
        if action.is_chord() {
            self.chord(action).display_name()
        } else {
            self.input(action).name().to_string()
        }
    }

    pub fn set_chord(&mut self, action: ActionKeyName, chord: ConfigChord) {
        match action {
            ActionKeyName::Undo => self.key_bindings.undo = chord,
            ActionKeyName::Redo => self.key_bindings.redo = chord,
            ActionKeyName::Copy => self.key_bindings.copy = chord,
            ActionKeyName::Paste => self.key_bindings.paste = chord,
            ActionKeyName::ToggleSelectionTool => self.key_bindings.toggle_selection_tool = chord,
            _ => {}
        }
    }

    pub fn key(&self, action: ActionKeyName) -> ConfigKey {
        match action {
            ActionKeyName::Pause => self.key_bindings.pause,
            ActionKeyName::Inventory => self.key_bindings.inventory,
            ActionKeyName::Alternate => self.key_bindings.alternate,
            ActionKeyName::RotateOrRollback => self.key_bindings.rotate_or_rollback,
            ActionKeyName::Simulate => self.key_bindings.simulate,
            ActionKeyName::SimulationStep => self.key_bindings.simulation_step,
            ActionKeyName::SimulationFast => self.key_bindings.simulation_fast,
            ActionKeyName::SimulationRollback => self.key_bindings.simulation_rollback,
            ActionKeyName::Debug => self.key_bindings.debug,
            ActionKeyName::DebugStructure => self.key_bindings.debug_structure,
            ActionKeyName::Forward => self.key_bindings.forward,
            ActionKeyName::Backward => self.key_bindings.backward,
            ActionKeyName::Left => self.key_bindings.left,
            ActionKeyName::Right => self.key_bindings.right,
            ActionKeyName::JumpOrFlyUp => self.key_bindings.jump_or_fly_up,
            ActionKeyName::FlyDown => self.key_bindings.fly_down,
            ActionKeyName::Undo => self.key_bindings.undo.key,
            ActionKeyName::Redo => self.key_bindings.redo.key,
            ActionKeyName::Copy => self.key_bindings.copy.key,
            ActionKeyName::Paste => self.key_bindings.paste.key,
            ActionKeyName::ToggleSelectionTool => self.key_bindings.toggle_selection_tool.key,
            ActionKeyName::Place | ActionKeyName::Delete | ActionKeyName::Pick => {
                return self
                    .input(action)
                    .key_code()
                    .map(key_from_code)
                    .unwrap_or(ConfigKey::KeyI);
            }
        }
    }

    pub fn input(&self, action: ActionKeyName) -> ConfigInput {
        match action {
            ActionKeyName::Pause => ConfigInput::Key(self.key_bindings.pause),
            ActionKeyName::Inventory => ConfigInput::Key(self.key_bindings.inventory),
            ActionKeyName::Alternate => ConfigInput::Key(self.key_bindings.alternate),
            ActionKeyName::RotateOrRollback => {
                ConfigInput::Key(self.key_bindings.rotate_or_rollback)
            }
            ActionKeyName::Simulate => ConfigInput::Key(self.key_bindings.simulate),
            ActionKeyName::SimulationStep => ConfigInput::Key(self.key_bindings.simulation_step),
            ActionKeyName::SimulationFast => ConfigInput::Key(self.key_bindings.simulation_fast),
            ActionKeyName::SimulationRollback => {
                ConfigInput::Key(self.key_bindings.simulation_rollback)
            }
            ActionKeyName::Debug => ConfigInput::Key(self.key_bindings.debug),
            ActionKeyName::DebugStructure => ConfigInput::Key(self.key_bindings.debug_structure),
            ActionKeyName::Forward => ConfigInput::Key(self.key_bindings.forward),
            ActionKeyName::Backward => ConfigInput::Key(self.key_bindings.backward),
            ActionKeyName::Left => ConfigInput::Key(self.key_bindings.left),
            ActionKeyName::Right => ConfigInput::Key(self.key_bindings.right),
            ActionKeyName::JumpOrFlyUp => ConfigInput::Key(self.key_bindings.jump_or_fly_up),
            ActionKeyName::FlyDown => ConfigInput::Key(self.key_bindings.fly_down),
            ActionKeyName::Undo
            | ActionKeyName::Redo
            | ActionKeyName::Copy
            | ActionKeyName::Paste
            | ActionKeyName::ToggleSelectionTool => ConfigInput::Key(self.chord(action).key),
            ActionKeyName::Place => self.key_bindings.place,
            ActionKeyName::Delete => self.key_bindings.delete,
            ActionKeyName::Pick => self.key_bindings.pick,
        }
    }

    pub fn set_key(&mut self, action: ActionKeyName, key: ConfigKey) {
        match action {
            ActionKeyName::Pause => self.key_bindings.pause = key,
            ActionKeyName::Inventory => self.key_bindings.inventory = key,
            ActionKeyName::Alternate => self.key_bindings.alternate = key,
            ActionKeyName::RotateOrRollback => self.key_bindings.rotate_or_rollback = key,
            ActionKeyName::Simulate => self.key_bindings.simulate = key,
            ActionKeyName::SimulationStep => self.key_bindings.simulation_step = key,
            ActionKeyName::SimulationFast => self.key_bindings.simulation_fast = key,
            ActionKeyName::SimulationRollback => self.key_bindings.simulation_rollback = key,
            ActionKeyName::Debug => self.key_bindings.debug = key,
            ActionKeyName::DebugStructure => self.key_bindings.debug_structure = key,
            ActionKeyName::Forward => self.key_bindings.forward = key,
            ActionKeyName::Backward => self.key_bindings.backward = key,
            ActionKeyName::Left => self.key_bindings.left = key,
            ActionKeyName::Right => self.key_bindings.right = key,
            ActionKeyName::JumpOrFlyUp => self.key_bindings.jump_or_fly_up = key,
            ActionKeyName::FlyDown => self.key_bindings.fly_down = key,
            ActionKeyName::Undo
            | ActionKeyName::Redo
            | ActionKeyName::Copy
            | ActionKeyName::Paste
            | ActionKeyName::ToggleSelectionTool => {}
            ActionKeyName::Place | ActionKeyName::Delete | ActionKeyName::Pick => {
                self.set_input(action, ConfigInput::Key(key));
            }
        }
    }

    pub fn set_input(&mut self, action: ActionKeyName, input: ConfigInput) {
        match action {
            ActionKeyName::Place => self.key_bindings.place = input,
            ActionKeyName::Delete => self.key_bindings.delete = input,
            ActionKeyName::Pick => self.key_bindings.pick = input,
            _ => {
                if let ConfigInput::Key(key) = input {
                    self.set_key(action, key);
                }
            }
        }
    }
}

fn key_from_code(key_code: KeyCode) -> ConfigKey {
    key_from_input_code(key_code).unwrap_or(ConfigKey::KeyI)
}
