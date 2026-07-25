//! 物品栏上方居中提示：瞬间出现，停留 1s 后 1s 淡出；新文案重置为不透明

use bevy::prelude::*;

use crate::game::state::{GameMode, PlacementState};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;
use crate::game::ui::components::text;
use crate::game::ui::types::{InventoryItem, InventoryItems};
use crate::shared::i18n::I18n;

const HOLD_SECS: f32 = 1.0;
const FADE_SECS: f32 = 1.0;

/// 游玩提示条状态（物品名 / 表面不可放置 / 后续报错）
#[derive(Resource, Default)]
pub struct GameplayToast {
    message: String,
    /// 距上次 show 的秒数；None 表示隐藏
    age: Option<f32>,
}

impl GameplayToast {
    /// 显示文案并重置为完全不透明（中途换文案也走这里）
    pub fn show(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.age = Some(0.0);
    }

    /// 「表面不可放置 {item}」
    pub fn show_cannot_place_on_surface(&mut self, locale: &I18n, item_name: &str) {
        self.show(locale.fmt(
            "toast.cannot_place_on_surface",
            &[("item", item_name)],
        ));
    }
}

/// 提示条文案实体标记
#[derive(Component)]
pub struct GameplayToastText;

/// 在快捷栏上方生成居中白字提示
pub fn spawn_gameplay_toast(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn((
                text("", 18.0, Color::srgba(1.0, 1.0, 1.0, 0.0)),
                TextLayout::justify(Justify::Center),
                Visibility::Hidden,
                GameplayToastText,
                Pickable::IGNORE,
            ));
        });
}

/// 推进计时并同步 Text / 透明度
pub fn update_gameplay_toast(
    time: Res<Time>,
    mut toast: ResMut<GameplayToast>,
    mut texts: Query<(&mut Text, &mut TextColor, &mut Visibility), With<GameplayToastText>>,
) {
    let Some(age) = toast.age.as_mut() else {
        return;
    };
    *age += time.delta_secs();
    let age = *age;
    if age >= HOLD_SECS + FADE_SECS {
        toast.message.clear();
        toast.age = None;
        for (_, _, mut visibility) in &mut texts {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let alpha = if age <= HOLD_SECS {
        1.0
    } else {
        1.0 - (age - HOLD_SECS) / FADE_SECS
    };

    for (mut text, mut color, mut visibility) in &mut texts {
        if text.0 != toast.message {
            text.0.clone_from(&toast.message);
        }
        *color = TextColor(Color::srgba(1.0, 1.0, 1.0, alpha));
        *visibility = Visibility::Visible;
    }
}

/// 切换快捷栏选中项时显示物品名
pub fn toast_on_hotbar_select(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    locale: Res<I18n>,
    mut toast: ResMut<GameplayToast>,
    mut last: Local<Option<(usize, Option<InventoryItem>)>>,
) {
    let current = (
        placement.selected,
        inventory.hotbar.get(placement.selected).copied().flatten(),
    );
    let Some(prev) = *last else {
        *last = Some(current);
        return;
    };
    if prev == current {
        return;
    }
    *last = Some(current);
    if let Some(item) = current.1 {
        toast.show(locale.t(item.name_key()).to_string());
    }
}

pub struct GameplayToastPlugin;

impl Plugin for GameplayToastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplayToast>().add_systems(
            Update,
            (
                toast_on_hotbar_select,
                update_gameplay_toast,
            )
                .chain()
                .run_if(|mode: Res<State<GameMode>>| *mode.get() == GameMode::Playing)
                .in_set(UiAccessScope)
                .after(PerfScope::Placement)
                .before(PerfScope::Ui),
        );
    }
}
