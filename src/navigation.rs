//! Navigation between a game's top-level places: the navigation bar along
//! the bottom of a small screen, the rail down the side of a larger one, and
//! the drawer of destinations in sections.
//!
//! Each holds its selection in [`NavigationSelection`] on itself. Choosing a
//! destination - a click, or Enter or Space while it has the keyboard's
//! focus - selects it and sends [`ValueChange<usize>`], the destination's
//! index, on the bar, rail or drawer. A game that moves elsewhere by other
//! means inserts a new [`NavigationSelection`].

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button, ValueChange};

use crate::focus::FocusRing;
use crate::icon::Icon;
use crate::paint::{ContentColor, Fill, Ink, Look, Paint, Typeset, label};
use crate::theme::{ColorRole, Corner, Elevation, TypeRole};

/// A place to go: its icon and its name.
#[derive(Clone, Debug, PartialEq)]
pub struct Destination {
    /// Its icon.
    pub icon: Icon,
    /// Its name.
    pub label: String,
}

/// A destination showing `icon`, named `label`.
pub fn destination(icon: impl Into<Icon>, label: impl Into<String>) -> Destination {
    Destination {
        icon: icon.into(),
        label: label.into(),
    }
}

/// Material's three ways to navigate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum NavigationKind {
    /// Along the bottom: 80 px tall.
    #[default]
    Bar,
    /// Down the side: 80 px wide.
    Rail,
    /// A sheet of destinations in sections: 360 px wide.
    Drawer,
}

/// Marks a navigation bar, rail or drawer.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Navigation(pub NavigationKind);

/// The index of the selected destination; insert a new one to select
/// another.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct NavigationSelection(pub usize);

/// A destination's index among its bar's, rail's or drawer's.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[component(immutable)]
#[reflect(Component)]
pub struct NavigationDestination(pub usize);

/// The pill behind a destination's icon, drawn when it is active.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ActiveIndicator;

/// A destination's name under its icon.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
struct DestinationLabel;

/// The look of a destination's indicator, active or not.
pub fn indicator_look(active: bool) -> Look {
    if active {
        Look::filled(
            ColorRole::SecondaryContainer,
            ColorRole::OnSecondaryContainer,
        )
    } else {
        Look::bare(ColorRole::OnSurfaceVariant)
    }
}

/// The colour of a destination's name under its icon, active or not.
pub fn label_role(active: bool) -> ColorRole {
    if active {
        ColorRole::OnSurface
    } else {
        ColorRole::OnSurfaceVariant
    }
}

/// A destination of a bar or a rail: an indicator around its icon, its
/// name under it.
fn icon_destination(
    index: usize,
    destination: Destination,
    pill: Vec2,
    active: bool,
    grow: bool,
) -> impl Bundle {
    let Destination { icon, label: words } = destination;
    (
        Button,
        NavigationDestination(index),
        TabIndex(0),
        FocusRing,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(4.0),
            min_width: Val::Px(pill.x),
            flex_grow: if grow { 1.0 } else { 0.0 },
            flex_basis: if grow { Val::ZERO } else { Val::Auto },
            ..default()
        },
        children![
            (
                ActiveIndicator,
                indicator_look(active),
                Node {
                    width: Val::Px(pill.x),
                    height: Val::Px(pill.y),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: Corner::Full.all(),
                    ..default()
                },
                children![crate::icon::icon(icon, crate::icon::ICON)],
            ),
            (
                DestinationLabel,
                Text::new(words),
                Typeset(TypeRole::LabelMedium),
                Ink::new(label_role(active)),
                Pickable::IGNORE,
            ),
        ],
    )
}

/// A navigation bar of 3 to 5 `destinations`, the one at `selected` active.
pub fn navigation_bar(destinations: Vec<Destination>, selected: usize) -> impl Bundle {
    (
        Navigation(NavigationKind::Bar),
        NavigationSelection(selected),
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
            padding: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(12.0), Val::Px(16.0)),
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: Val::Px(8.0),
            flex_shrink: 0.0,
            ..default()
        },
        Fill::new(ColorRole::SurfaceContainer),
        crate::paint::Raised(Elevation::Level2),
        Children::spawn(SpawnWith(move |bar: &mut ChildSpawner| {
            for (index, destination) in destinations.into_iter().enumerate() {
                bar.spawn(icon_destination(
                    index,
                    destination,
                    Vec2::new(64.0, 32.0),
                    index == selected,
                    true,
                ));
            }
        })),
    )
}

/// Marks a rail's floating action button.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct RailFab;

/// Sent on a rail when its floating action button is used.
#[derive(EntityEvent, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RailFabActivated {
    /// The rail.
    #[event_target]
    pub rail: Entity,
}

/// A navigation rail of `destinations`, the one at `selected` active, with a
/// floating action button showing `fab` above them if it has one.
pub fn navigation_rail(
    destinations: Vec<Destination>,
    selected: usize,
    fab: Option<Icon>,
) -> impl Bundle {
    (
        Navigation(NavigationKind::Rail),
        NavigationSelection(selected),
        Node {
            width: Val::Px(80.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(44.0)),
            row_gap: Val::Px(12.0),
            flex_shrink: 0.0,
            ..default()
        },
        Fill::new(ColorRole::Surface),
        Children::spawn(SpawnWith(move |rail: &mut ChildSpawner| {
            if let Some(icon) = fab {
                rail.spawn((
                    Node {
                        margin: UiRect::bottom(Val::Px(28.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_child((crate::fab(icon, crate::FabSize::Regular, None), RailFab));
            }
            for (index, destination) in destinations.into_iter().enumerate() {
                rail.spawn(icon_destination(
                    index,
                    destination,
                    Vec2::new(56.0, 32.0),
                    index == selected,
                    false,
                ));
            }
        })),
    )
}

/// A navigation drawer: its sections of destinations, and which is active.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NavigationDrawer {
    /// Its sections, each with an optional headline, in order.
    pub sections: Vec<(Option<String>, Vec<Destination>)>,
    /// The index, counting across sections, of the active destination.
    pub selected: usize,
}

/// A navigation drawer with no destinations yet: add sections and
/// destinations with the builder.
pub fn navigation_drawer() -> NavigationDrawer {
    NavigationDrawer::default()
}

impl NavigationDrawer {
    /// Starts a new section headed `headline`.
    pub fn headline(mut self, headline: impl Into<String>) -> Self {
        self.sections.push((Some(headline.into()), Vec::new()));
        self
    }

    /// Adds `destination` to the last section.
    pub fn destination(mut self, destination: Destination) -> Self {
        match self.sections.last_mut() {
            Some((_, destinations)) => destinations.push(destination),
            None => self.sections.push((None, vec![destination])),
        }
        self
    }

    /// With the destination at `index`, counting across sections, active.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let NavigationDrawer { sections, selected } = self;
        let end = Corner::Large.radius();
        (
            Navigation(NavigationKind::Drawer),
            NavigationSelection(selected),
            Node {
                width: Val::Px(360.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                flex_shrink: 0.0,
                border_radius: BorderRadius::new(Val::ZERO, end, end, Val::ZERO),
                ..default()
            },
            Fill::new(ColorRole::SurfaceContainerLow),
            ContentColor(Paint::new(ColorRole::OnSurfaceVariant)),
            Children::spawn(SpawnWith(move |drawer: &mut ChildSpawner| {
                let mut index = 0;
                for (number, (headline, destinations)) in sections.into_iter().enumerate() {
                    if number > 0 {
                        drawer
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_child(crate::divider(false));
                    }
                    if let Some(headline) = headline {
                        drawer.spawn((
                            Text::new(headline),
                            Typeset(TypeRole::TitleSmall),
                            Ink::new(ColorRole::OnSurfaceVariant),
                            Node {
                                padding: UiRect::new(
                                    Val::Px(16.0),
                                    Val::Px(16.0),
                                    Val::Px(18.0),
                                    Val::Px(18.0),
                                ),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                    }
                    for destination in destinations {
                        drawer.spawn(drawer_destination(index, destination, index == selected));
                        index += 1;
                    }
                }
            })),
        )
    }
}

/// A destination of a drawer: a pill holding its icon and name.
fn drawer_destination(index: usize, destination: Destination, active: bool) -> impl Bundle {
    let Destination { icon, label: words } = destination;
    (
        Button,
        NavigationDestination(index),
        ActiveIndicator,
        TabIndex(0),
        FocusRing,
        indicator_look(active),
        Node {
            height: Val::Px(56.0),
            padding: UiRect::new(Val::Px(16.0), Val::Px(24.0), Val::ZERO, Val::ZERO),
            column_gap: Val::Px(12.0),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            border_radius: Corner::Full.all(),
            ..default()
        },
        children![
            crate::icon::icon(icon, crate::icon::ICON),
            label(words, TypeRole::LabelLarge),
        ],
    )
}

/// Selects a destination when it is used, and sends its index on its
/// navigation.
fn on_destination(
    activate: On<Activate>,
    destinations: Query<&NavigationDestination>,
    mut navigations: Query<&mut NavigationSelection, With<Navigation>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let Ok(NavigationDestination(index)) = destinations.get(activate.entity) else {
        return;
    };
    let Some(navigation) = parents
        .iter_ancestors(activate.entity)
        .find(|ancestor| navigations.contains(*ancestor))
    else {
        return;
    };
    let Ok(mut selection) = navigations.get_mut(navigation) else {
        return;
    };
    if selection.0 == *index {
        return;
    }
    selection.0 = *index;
    commands.trigger(ValueChange {
        source: navigation,
        value: *index,
        is_final: true,
    });
}

/// Sends [`RailFabActivated`] on a rail when its floating action button is
/// used.
fn on_rail_fab(
    activate: On<Activate>,
    fabs: Query<(), With<RailFab>>,
    rails: Query<(), With<Navigation>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    if !fabs.contains(activate.entity) {
        return;
    }
    if let Some(rail) = parents
        .iter_ancestors(activate.entity)
        .find(|ancestor| rails.contains(*ancestor))
    {
        commands.trigger(RailFabActivated { rail });
    }
}

/// Draws each destination active or not, as its navigation's selection
/// changes.
fn draw_destinations(
    navigations: Query<(Entity, &NavigationSelection), Changed<NavigationSelection>>,
    children: Query<&Children>,
    destinations: Query<&NavigationDestination>,
    mut indicators: Query<&mut Look, With<ActiveIndicator>>,
    labels: Query<(), With<DestinationLabel>>,
    mut commands: Commands,
) {
    for (navigation, NavigationSelection(selected)) in &navigations {
        for part in children.iter_descendants(navigation) {
            let Ok(NavigationDestination(index)) = destinations.get(part) else {
                continue;
            };
            let active = index == selected;
            for inner in std::iter::once(part).chain(children.iter_descendants(part)) {
                if let Ok(mut look) = indicators.get_mut(inner) {
                    look.set_if_neq(indicator_look(active));
                }
                if labels.contains(inner) {
                    commands.entity(inner).insert(Ink::new(label_role(active)));
                }
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_destination)
        .add_observer(on_rail_fab)
        .add_systems(PostUpdate, draw_destinations.before(crate::PaintSystems));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_active_destination_sits_on_the_secondary_container() {
        assert_eq!(
            indicator_look(true).container,
            Some(ColorRole::SecondaryContainer)
        );
        assert_eq!(
            indicator_look(true).content,
            ColorRole::OnSecondaryContainer
        );
        assert_eq!(indicator_look(false).container, None);
        assert_eq!(label_role(true), ColorRole::OnSurface);
        assert_eq!(label_role(false), ColorRole::OnSurfaceVariant);
    }

    #[test]
    fn a_drawers_destinations_go_into_its_last_section() {
        let drawer = navigation_drawer()
            .destination(destination(crate::Glyph::Menu, "All"))
            .headline("Labels")
            .destination(destination(crate::Glyph::Dot, "Family"))
            .destination(destination(crate::Glyph::Dot, "Work"));
        assert_eq!(drawer.sections.len(), 2);
        assert_eq!(drawer.sections[0].0, None);
        assert_eq!(drawer.sections[1].0.as_deref(), Some("Labels"));
        assert_eq!(drawer.sections[1].1.len(), 2);
    }
}
