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
