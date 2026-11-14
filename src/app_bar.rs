//! Top app bars: a page's title, a navigation button before it and action
//! buttons after it - small, centre-aligned, medium and large.
//!
//! A bar's buttons are icon buttons; when one is used the bar sends
//! [`AppBarAction`] on itself, saying which, so the game observes the bar
//! rather than each button.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::button::{IconButtonVariant, icon_button};
use crate::icon::Icon;
use crate::paint::{ContentColor, Fill, Paint, Typeset};
use crate::theme::{ColorRole, TypeRole};

/// Material's four top app bars.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum AppBarKind {
    /// 64 px, its title beside its buttons.
    #[default]
    Small,
    /// 64 px, its title centred.
    CenterAligned,
    /// 112 px, its title in a headline under its buttons.
    Medium,
    /// 152 px, its title in a larger headline under its buttons.
    Large,
}

impl AppBarKind {
    /// Its height in logical pixels, and its title's kind of text.
    pub fn metrics(self) -> (f32, TypeRole) {
        match self {
            AppBarKind::Small | AppBarKind::CenterAligned => (64.0, TypeRole::TitleLarge),
            AppBarKind::Medium => (112.0, TypeRole::HeadlineSmall),
            AppBarKind::Large => (152.0, TypeRole::HeadlineMedium),
        }
    }

    /// Whether its title is on a row of its own, under the buttons.
    pub fn title_below(self) -> bool {
        matches!(self, AppBarKind::Medium | AppBarKind::Large)
    }
}

/// Marks a top app bar.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct AppBar;

/// Which of a bar's buttons was used.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
#[component(immutable)]
#[reflect(Component)]
pub enum AppBarButton {
    /// The navigation button before the title: back, or a menu.
    Navigation,
    /// The action button at this index, counting from the first after the
    /// title.
    Action(usize),
}

/// Sent on a bar when one of its buttons is used.
#[derive(EntityEvent, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppBarAction {
    /// The bar.
    #[event_target]
    pub bar: Entity,
    /// Which button.
    pub button: AppBarButton,
}

/// The bar's title, so a game can change its words.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct AppBarTitle;

/// A top app bar: its kind, title and buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct TopAppBar {
    /// Its kind.
    pub kind: AppBarKind,
    /// Its title.
    pub title: String,
    /// The navigation button's icon, if it has one.
    pub navigation: Option<Icon>,
    /// The action buttons' icons, in order.
    pub actions: Vec<Icon>,
}

/// A small top app bar titled `title`.
pub fn top_app_bar(title: impl Into<String>) -> TopAppBar {
    TopAppBar {
        kind: AppBarKind::Small,
        title: title.into(),
        navigation: None,
        actions: Vec::new(),
    }
}

impl TopAppBar {
    /// Of `kind`.
    pub fn kind(mut self, kind: AppBarKind) -> Self {
        self.kind = kind;
        self
    }

    /// With a navigation button showing `icon` before the title.
    pub fn navigation(mut self, icon: impl Into<Icon>) -> Self {
        self.navigation = Some(icon.into());
        self
    }

    /// With an action button showing `icon`, after those already added.
    pub fn action(mut self, icon: impl Into<Icon>) -> Self {
        self.actions.push(icon.into());
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let TopAppBar {
            kind,
            title,
            navigation,
            actions,
        } = self;
        let (height, role) = kind.metrics();
        let title_bundle = move |node: Node, centred: bool| {
            (
                AppBarTitle,
                Text::new(title.clone()),
                Typeset(role),
                crate::paint::Content,
                TextLayout::justify(if centred {
                    Justify::Center
                } else {
                    Justify::Left
                }),
                node,
                Pickable::IGNORE,
            )
        };
        (
            AppBar,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(height),
                flex_direction: FlexDirection::Column,
                padding: UiRect::horizontal(Val::Px(4.0)),
                flex_shrink: 0.0,
                ..default()
            },
            Fill::new(ColorRole::Surface),
            ContentColor(Paint::new(ColorRole::OnSurface)),
            Children::spawn(SpawnWith(move |bar: &mut ChildSpawner| {
                let mut row = bar.spawn((
                    Node {
                        height: Val::Px(64.0),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
                row.with_children(|row| {
                    if let Some(icon) = navigation {
                        row.spawn((
                            icon_button(icon, IconButtonVariant::Standard),
                            AppBarButton::Navigation,
                        ));
                    } else if kind == AppBarKind::CenterAligned {
                        row.spawn(Node {
                            width: Val::Px(crate::ICON_BUTTON),
                            ..default()
                        });
                    }
                    if kind.title_below() {
                        row.spawn(Node {
                            flex_grow: 1.0,
                            ..default()
                        });
                    } else {
                        row.spawn(title_bundle(
                            Node {
                                flex_grow: 1.0,
                                flex_basis: Val::ZERO,
                                margin: UiRect::horizontal(Val::Px(12.0)),
                                ..default()
                            },
                            kind == AppBarKind::CenterAligned,
                        ));
                    }
                    for (index, icon) in actions.into_iter().enumerate() {
                        row.spawn((
                            icon_button(icon, IconButtonVariant::Standard),
                            AppBarButton::Action(index),
                        ));
                    }
                });
                if kind.title_below() {
                    bar.spawn((
                        Node {
                            flex_grow: 1.0,
                            align_items: AlignItems::FlexEnd,
                            padding: UiRect::new(
                                Val::Px(12.0),
                                Val::Px(12.0),
                                Val::ZERO,
                                Val::Px(if kind == AppBarKind::Large {
                                    28.0
                                } else {
                                    24.0
                                }),
                            ),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_child(title_bundle(Node::default(), false));
                }
            })),
        )
    }
}

/// Sends [`AppBarAction`] on a bar when one of its buttons is used.
fn on_bar_button(
    activate: On<Activate>,
    buttons: Query<&AppBarButton>,
    bars: Query<(), With<AppBar>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    if let Some(bar) = parents
        .iter_ancestors(activate.entity)
        .find(|ancestor| bars.contains(*ancestor))
    {
        commands.trigger(AppBarAction {
            bar,
            button: *button,
        });
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_bar_button);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taller_bars_set_their_title_larger_under_their_buttons() {
        assert_eq!(AppBarKind::Small.metrics(), (64.0, TypeRole::TitleLarge));
        assert_eq!(
            AppBarKind::CenterAligned.metrics(),
            (64.0, TypeRole::TitleLarge)
        );
        assert_eq!(
            AppBarKind::Medium.metrics(),
            (112.0, TypeRole::HeadlineSmall)
        );
        assert_eq!(
            AppBarKind::Large.metrics(),
            (152.0, TypeRole::HeadlineMedium)
        );
        assert!(!AppBarKind::Small.title_below() && AppBarKind::Large.title_below());
    }
}
