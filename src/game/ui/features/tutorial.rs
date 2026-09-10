//! 数据驱动教程：所有教程共用一个面板壳，只替换步骤 ViewModel。

use std::collections::HashMap;

use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;
use crate::game::ui::components::{
    STATUS_TEXT, auto_width_button, flex_row_auto, panel_bundle, panel_content, panel_title_bar,
    panel_title_label, text,
};
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHostMountRoot};
use crate::game::ui::core::{UiMountState, UiNavigation};
use crate::shared::i18n::I18n;

/// 教程步骤在屏幕或世界中的指向目标。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum TutorialAnchor {
    #[default]
    Center,
    Ui(String),
    World(IVec3),
}

/// 教程步骤的展示数据；后续可直接由资源文件反序列化构造。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialStep {
    pub title: String,
    pub body: String,
    pub anchor: TutorialAnchor,
}

/// 一组按顺序播放的教程步骤。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialDefinition {
    pub id: String,
    pub steps: Vec<TutorialStep>,
}

/// 教程定义注册表；功能模块可在启动时注册自己的教程数据。
#[derive(Resource, Default)]
pub struct TutorialCatalog {
    definitions: HashMap<String, TutorialDefinition>,
}

impl TutorialCatalog {
    pub fn register(&mut self, definition: TutorialDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    pub fn get(&self, id: &str) -> Option<&TutorialDefinition> {
        self.definitions.get(id)
    }
}

/// 教程导航意图；业务代码只发送意图，不直接操作 UI 实体。
#[derive(Clone, Debug, Message)]
pub enum TutorialIntent {
    Open(String),
    Next,
    Previous,
    Close,
}

/// 教程面板按钮动作。
#[derive(Component, Clone, Copy, Eq, PartialEq)]
enum TutorialButton {
    Previous,
    Next,
    Close,
}

/// 教程标题文本。
#[derive(Component)]
struct TutorialTitle;

/// 教程正文文本。
#[derive(Component)]
struct TutorialBody;

/// 教程步骤进度文本。
#[derive(Component)]
struct TutorialProgress;

/// 教程面板根实体。
#[derive(Component)]
struct TutorialPanel;

/// 教程功能插件。
pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialCatalog>()
            .add_message::<TutorialIntent>()
            .add_observer(
                |mut click: On<Pointer<Click>>,
                 buttons: Query<&TutorialButton>,
                 mut intents: MessageWriter<TutorialIntent>| {
                    if click.event.button != bevy::picking::pointer::PointerButton::Primary {
                        return;
                    }
                    let Ok(button) = buttons.get(click.entity) else {
                        return;
                    };
                    click.propagate(false);
                    intents.write(match button {
                        TutorialButton::Previous => TutorialIntent::Previous,
                        TutorialButton::Next => TutorialIntent::Next,
                        TutorialButton::Close => TutorialIntent::Close,
                    });
                },
            )
            .add_systems(
                Update,
                (reduce_tutorial_intents, reconcile_tutorial_panel)
                    .chain()
                    .in_set(UiAccessScope)
                    .after(PerfScope::Placement)
                    .before(PerfScope::Menus),
            );
    }
}

/// 把教程意图归约到唯一导航状态。
fn reduce_tutorial_intents(
    mut intents: MessageReader<TutorialIntent>,
    catalog: Res<TutorialCatalog>,
    mut navigation: ResMut<UiNavigation>,
) {
    for intent in intents.read() {
        match intent {
            TutorialIntent::Open(id)
                if catalog.get(id).is_some_and(|item| !item.steps.is_empty()) =>
            {
                navigation.open_tutorial(id.clone());
            }
            TutorialIntent::Next => {
                let Some(session) = navigation.tutorial().cloned() else {
                    continue;
                };
                let Some(definition) = catalog.get(&session.tutorial) else {
                    navigation.close_tutorial();
                    continue;
                };
                if session.step + 1 < definition.steps.len() {
                    navigation.set_tutorial_step(session.step + 1);
                } else {
                    navigation.close_tutorial();
                }
            }
            TutorialIntent::Previous => {
                let Some(session) = navigation.tutorial().cloned() else {
                    continue;
                };
                navigation.set_tutorial_step(session.step.saturating_sub(1));
            }
            TutorialIntent::Close => navigation.close_tutorial(),
            TutorialIntent::Open(_) => {}
        }
    }
}

/// 将教程导航状态增量同步到共享面板壳。
fn reconcile_tutorial_panel(
    navigation: Res<UiNavigation>,
    catalog: Res<TutorialCatalog>,
    i18n: Res<I18n>,
    root: Option<Res<PlayingUiRootEntity>>,
    mut mounts: ResMut<UiMountState>,
    mut commands: Commands,
    mut titles: Query<&mut Text, With<TutorialTitle>>,
    mut bodies: Query<&mut Text, (With<TutorialBody>, Without<TutorialTitle>)>,
    mut progress: Query<
        &mut Text,
        (
            With<TutorialProgress>,
            Without<TutorialTitle>,
            Without<TutorialBody>,
        ),
    >,
    mut buttons: Query<(&TutorialButton, &Children, &mut Node)>,
    mut button_texts: Query<
        &mut Text,
        (
            Without<TutorialTitle>,
            Without<TutorialBody>,
            Without<TutorialProgress>,
        ),
    >,
) {
    let Some(session) = navigation.tutorial() else {
        if let Some(entity) = mounts.tutorial.take() {
            commands.entity(entity).despawn();
        }
        return;
    };
    let Some(definition) = catalog.get(&session.tutorial) else {
        return;
    };
    let Some(step) = definition.steps.get(session.step) else {
        return;
    };

    if mounts.tutorial.is_none() {
        let Some(root) = root.map(|root| root.0) else {
            return;
        };
        let mut mounted = None;
        commands.entity(root).with_children(|root| {
            mounted = Some(
                root.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    UiHostMountRoot,
                    GlobalZIndex(20_000),
                ))
                .with_children(|container| {
                    container
                        .spawn((panel_bundle(560.0), TutorialPanel))
                        .with_children(|panel| {
                            panel.spawn(panel_title_bar()).with_children(|bar| {
                                bar.spawn((panel_title_label(step.title.clone()), TutorialTitle));
                            });
                            panel.spawn(panel_content()).with_children(|content| {
                                content.spawn((
                                    text(step.body.clone(), 16.0, STATUS_TEXT),
                                    TextLayout::justify(Justify::Left),
                                    TutorialBody,
                                ));
                                content.spawn((
                                    text(
                                        format!(
                                            "{} / {}",
                                            session.step + 1,
                                            definition.steps.len()
                                        ),
                                        13.0,
                                        STATUS_TEXT,
                                    ),
                                    TextLayout::justify(Justify::Center),
                                    TutorialProgress,
                                ));
                                content
                                    .spawn(flex_row_auto(36.0, 8.0))
                                    .with_children(|row| {
                                        for (button, label) in [
                                            (TutorialButton::Previous, i18n.t("tutorial.previous")),
                                            (TutorialButton::Next, i18n.t("tutorial.next")),
                                            (TutorialButton::Close, i18n.t("button.close")),
                                        ] {
                                            row.spawn((auto_width_button(36.0), button))
                                                .with_children(|button| {
                                                    button.spawn(text(label, 15.0, Color::WHITE));
                                                });
                                        }
                                    });
                            });
                        });
                })
                .id(),
            );
        });
        mounts.tutorial = mounted;
    }

    for mut title in &mut titles {
        title.0.clone_from(&step.title);
    }
    for mut body in &mut bodies {
        body.0.clone_from(&step.body);
    }
    for mut label in &mut progress {
        label.0 = format!("{} / {}", session.step + 1, definition.steps.len());
    }
    for (button, children, mut node) in &mut buttons {
        node.display = if *button == TutorialButton::Previous && session.step == 0 {
            Display::None
        } else {
            Display::Flex
        };
        let label = match button {
            TutorialButton::Previous => i18n.t("tutorial.previous"),
            TutorialButton::Next if session.step + 1 == definition.steps.len() => {
                i18n.t("tutorial.finish")
            }
            TutorialButton::Next => i18n.t("tutorial.next"),
            TutorialButton::Close => i18n.t("button.close"),
        };
        for child in children.iter() {
            if let Ok(mut text) = button_texts.get_mut(child) {
                text.0 = label.to_string();
            }
        }
    }
}
