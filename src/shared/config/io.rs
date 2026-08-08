pub fn load_config() -> GameConfig {
    let key = persistent_storage::config_key();
    let Some(contents) = persistent_storage::read(key) else {
        let config = GameConfig::default();
        // Web hydrate 前不要落盘，避免被 replace_all 冲掉
        if persistent_storage::is_ready() {
            save_config(&config);
        }
        return config;
    };
    ron::from_str::<GameConfig>(&contents).unwrap_or_else(|error| {
        warn!("Failed to load config: {error}");
        GameConfig::default()
    })
}

pub fn save_config(config: &GameConfig) {
    let key = persistent_storage::config_key();
    match ron::ser::to_string_pretty(config, PrettyConfig::default()) {
        Ok(serialized) => {
            if !persistent_storage::write(key, &serialized) {
                warn!("Failed to write config");
            }
        }
        Err(error) => warn!("Failed to serialize config: {error}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn open_config_folder() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_config_folder() {
    use std::fs;
    use std::process::Command;

    let dir = crate::shared::platform::saves_directory();
    if let Err(error) = fs::create_dir_all(dir) {
        warn!("Failed to create config directory: {error}");
        return;
    }

    if let Err(error) = Command::new("open").arg(dir).spawn() {
        warn!("Failed to open config folder: {error}");
    }
}

const CONFIG_KEYS: [ConfigKey; 27] = [
    ConfigKey::Escape,
    ConfigKey::Space,
    ConfigKey::ShiftLeft,
    ConfigKey::ShiftRight,
    ConfigKey::Slash,
    ConfigKey::KeyA,
    ConfigKey::KeyC,
    ConfigKey::KeyD,
    ConfigKey::KeyE,
    ConfigKey::KeyF,
    ConfigKey::KeyI,
    ConfigKey::KeyP,
    ConfigKey::KeyR,
    ConfigKey::KeyS,
    ConfigKey::KeyV,
    ConfigKey::KeyW,
    ConfigKey::KeyX,
    ConfigKey::KeyZ,
    ConfigKey::Digit1,
    ConfigKey::Digit2,
    ConfigKey::Digit3,
    ConfigKey::Digit4,
    ConfigKey::Digit5,
    ConfigKey::Digit6,
    ConfigKey::Digit7,
    ConfigKey::Digit8,
    ConfigKey::Digit9,
];

pub fn chord_from_input(keys: &ButtonInput<KeyCode>) -> Option<ConfigChord> {
    let key = CONFIG_KEYS
        .iter()
        .copied()
        .filter(|key| !key.is_modifier())
        .find(|key| keys.just_pressed(key.key_code()))?;
    Some(ConfigChord {
        key,
        primary_modifier: primary_modifier_pressed(keys),
        shift: shift_modifier_pressed(keys),
        alt: alt_modifier_pressed(keys),
    })
}

pub fn key_from_input(keys: &ButtonInput<KeyCode>) -> Option<ConfigKey> {
    CONFIG_KEYS
        .iter()
        .copied()
        .find(|key| keys.just_pressed(key.key_code()))
}

pub fn input_from_buttons(
    keys: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
) -> Option<ConfigInput> {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        return Some(ConfigInput::MouseLeft);
    }
    if mouse_buttons.just_pressed(MouseButton::Right) {
        return Some(ConfigInput::MouseRight);
    }
    if mouse_buttons.just_pressed(MouseButton::Middle) {
        return Some(ConfigInput::MouseMiddle);
    }
    key_from_input(keys).map(ConfigInput::Key)
}

fn key_from_input_code(key_code: KeyCode) -> Option<ConfigKey> {
    CONFIG_KEYS
        .iter()
        .copied()
        .find(|key| key.key_code() == key_code)
}
