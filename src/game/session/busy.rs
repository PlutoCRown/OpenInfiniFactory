//! 会话忙碌态：读档/存档退出时的中间提示

use bevy::prelude::*;

/// 会话级忙碌提示（加载中 / 保存中）
#[derive(Resource, Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBusy {
    #[default]
    None,
    Loading,
    Saving,
}

impl SessionBusy {
    pub fn is_busy(self) -> bool {
        self != Self::None
    }

    pub fn label_key(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Loading => Some("status.session_loading"),
            Self::Saving => Some("status.session_saving"),
        }
    }
}

/// 加载遮罩用的封面（进档前从存档列表截取）
#[derive(Resource, Default)]
pub struct SessionBusyCover {
    pub image: Option<Handle<Image>>,
}

impl SessionBusyCover {
    pub fn clear(&mut self) {
        self.image = None;
    }
}
