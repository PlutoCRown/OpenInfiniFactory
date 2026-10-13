use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::shared::i18n::Language;
use crate::shared::persistent_storage;

include!("game_config.rs");
include!("virtual_controls.rs");
include!("keys.rs");
include!("game_config_impl.rs");
include!("io.rs");

/// 设置页与配置读取共用的缩放范围。
pub const UI_SCALE_MIN: f32 = 0.5;
/// UI 缩放的上限。
pub const UI_SCALE_MAX: f32 = 3.0;
/// 重力倍率的下限。
pub const GRAVITY_SCALE_MIN: f32 = 1.0;
/// 重力倍率的上限。
pub const GRAVITY_SCALE_MAX: f32 = 2.0;
/// 鼠标灵敏度的下限。
pub const MOUSE_SENSITIVITY_MIN: f32 = 0.5;
/// 鼠标灵敏度的上限。
pub const MOUSE_SENSITIVITY_MAX: f32 = 2.0;

#[cfg(test)]
mod normalization_tests {
    use super::*;

    /// 读取配置时同时处理越界和非有限值，运行系统无需重复校验。
    #[test]
    fn invalid_numeric_settings_are_normalized_together() {
        let config = GameConfig {
            fov_degrees: 190.0,
            ui_scale: f32::NAN,
            gravity_scale: -1.0,
            mouse_sensitivity_x: f32::INFINITY,
            mouse_sensitivity_y: 30.0,
            virtual_controls_opacity: -0.5,
            master_volume: 1.8,
            music_volume: f32::NEG_INFINITY,
            sfx_volume: 0.4,
            ..default()
        }
        .normalized();
        assert_eq!(config.fov_degrees, 110.0);
        assert_eq!(config.ui_scale, DEFAULT_CONFIG.ui_scale);
        assert_eq!(config.gravity_scale, GRAVITY_SCALE_MIN);
        assert_eq!(
            config.mouse_sensitivity_x,
            DEFAULT_CONFIG.mouse_sensitivity_x
        );
        assert_eq!(config.mouse_sensitivity_y, MOUSE_SENSITIVITY_MAX);
        assert_eq!(config.virtual_controls_opacity, 0.0);
        assert_eq!(config.master_volume, 1.0);
        assert_eq!(config.music_volume, DEFAULT_CONFIG.music_volume);
        assert_eq!(config.sfx_volume, 0.4);
    }
}
