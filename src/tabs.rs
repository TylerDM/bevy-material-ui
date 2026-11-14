//! Tabs: primary tabs, at the top of a page, and secondary tabs, within one.
//!
//! A [`tabs`] row is Bevy's headless [`TabList`]: a click on a tab selects
//! it, and while a tab has the keyboard's focus the arrow keys, Home and End
//! move between tabs and Enter or Space selects one. It sends
//! [`bevy::ui_widgets::ValueChange<Option<Entity>>`] on the row and keeps its
//! own selection. [`tab_panels`] shows only the selected tab's panel.

use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::{SelectedTab, Tab, TabList, observe, tablist_self_update};

use crate::focus::FocusRing;
use crate::icon::Icon;
use crate::paint::{Fill, Look, Stroke, label};
use crate::theme::{ColorRole, TypeRole};

/// Material's two kinds of tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum TabsKind {
    /// At the top of a page: the selected tab's words in primary, a short
    /// indicator under them.
    #[default]
    Primary,
    /// Within a page: the indicator across the whole tab.
    Secondary,
}

impl TabsKind {
    /// The selected tab's content colour, and the indicator's thickness.
    pub fn selected(self) -> (ColorRole, f32) {
        match self {
            TabsKind::Primary => (ColorRole::Primary, 3.0),
            TabsKind::Secondary => (ColorRole::OnSurface, 2.0),
        }
    }
}

/// Marks a row of tabs, of its kind.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MaterialTabs(pub TabsKind);

/// The tab a row starts on, as an index among its tabs.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct InitialTab(usize);

/// A row of tabs: its kind and which it starts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialTabsRow {
    /// Its kind.
    pub kind: TabsKind,
    /// The index of the tab it starts on.
    pub selected: usize,
}

/// A row of tabs of `kind`, starting on the first: spawn [`tab`]s of the same
/// kind inside it.
pub fn tabs(kind: TabsKind) -> MaterialTabsRow {
    MaterialTabsRow { kind, selected: 0 }
}

impl MaterialTabsRow {
    /// Starting on the tab at `index`.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        (
            MaterialTabs(self.kind),
            TabList::default(),
            InitialTab(self.selected),
            observe(tablist_self_update),
            Node {
                width: Val::Percent(100.0),
                border: UiRect::bottom(Val::Px(1.0)),
                flex_shrink: 0.0,
                ..default()
            },
            Fill::new(ColorRole::Surface),
            Stroke::new(ColorRole::OutlineVariant),
        )
    }
}

/// Marks a tab, of its kind.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MaterialTab(pub TabsKind);

/// The bar under the selected tab.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct TabIndicator;

/// A tab: its kind, words, icon and whether it can be chosen.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialTabHeader {
    /// Its kind, which its row's must match.
    pub kind: TabsKind,
    /// What it says.
    pub label: String,
    /// An icon above its words (primary) or before them (secondary).
    pub icon: Option<Icon>,
    /// Whether it can be chosen.
    pub enabled: bool,
}

/// A tab of `kind` saying `label`.
pub fn tab(kind: TabsKind, label: impl Into<String>) -> MaterialTabHeader {
    MaterialTabHeader {
        kind,
        label: label.into(),
        icon: None,
        enabled: true,
    }
}

impl MaterialTabHeader {
    /// With `icon`.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Enabled, or drawn faded and unchoosable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Its height in logical pixels: 64 for a primary tab with an icon,
    /// otherwise 48.
    pub fn height(&self) -> f32 {
        if self.kind == TabsKind::Primary && self.icon.is_some() {
            64.0
        } else {
            48.0
        }
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let height = self.height();
        let MaterialTabHeader {
            kind,
            label: words,
            icon,
            enabled,
        } = self;
        let (_, thickness) = kind.selected();
        let indicator = move || {
            (
                TabIndicator,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    right: Val::ZERO,
                    bottom: Val::ZERO,
                    height: Val::Px(thickness),
                    border_radius: BorderRadius::top(Val::Px(thickness)),
                    ..default()
                },
                Fill::new(ColorRole::Primary),
                Visibility::Hidden,
                Pickable::IGNORE,
            )
        };
        (
            (
                Tab,
                MaterialTab(kind),
                Look::bare(ColorRole::OnSurfaceVariant),
                FocusRing,
                crate::button::enabled(enabled),
            ),
            Node {
                flex_grow: 1.0,
                flex_basis: Val::ZERO,
                height: Val::Px(height),
                padding: UiRect::horizontal(Val::Px(16.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Children::spawn(SpawnWith(move |tab: &mut ChildSpawner| {
                let primary = kind == TabsKind::Primary;
                tab.spawn((
                    Node {
                        height: Val::Percent(100.0),
                        flex_direction: if primary {
                            FlexDirection::Column
                        } else {
                            FlexDirection::Row
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(2.0),
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|inside| {
                    if let Some(glyph) = icon {
                        inside.spawn(crate::icon::icon(glyph, crate::icon::ICON));
                    }
                    inside.spawn(label(words, TypeRole::TitleSmall));
                    if primary {
                        inside.spawn(indicator());
                    }
                });
                if !primary {
                    tab.spawn(indicator());
                }
            })),
        )
    }
}

/// Selects each new row's starting tab, once its tabs are spawned.
fn select_initial_tabs(
    rows: Query<(Entity, &InitialTab, &Children), With<TabList>>,
    tabs: Query<(), With<Tab>>,
    mut commands: Commands,
) {
    for (row, InitialTab(index), children) in &rows {
        let chosen = children
            .iter()
            .filter(|child| tabs.contains(*child))
            .nth(*index);
        let mut row = commands.entity(row);
        row.try_remove::<InitialTab>();
        if let Some(chosen) = chosen {
            row.try_insert(SelectedTab(Some(chosen)));
        }
    }
}

/// A tab new or newly selected this frame.
type TabChanged = (With<MaterialTab>, Or<(Added<Selected>, Added<MaterialTab>)>);

/// Draws each tab as selected or not: its content colour and its indicator.
fn draw_tabs(
    changed: Query<Entity, TabChanged>,
    mut unselected: RemovedComponents<Selected>,
    mut tabs: Query<(&MaterialTab, &mut Look, Has<Selected>)>,
    children: Query<&Children>,
    mut indicators: Query<&mut Visibility, With<TabIndicator>>,
) {
    let mut redraw: Vec<Entity> = changed.iter().collect();
    redraw.extend(unselected.read());
    for entity in redraw {
        let Ok((MaterialTab(kind), mut look, selected)) = tabs.get_mut(entity) else {
            continue;
        };
        let content = if selected {
            kind.selected().0
        } else {
            ColorRole::OnSurfaceVariant
        };
        if look.content != content {
            look.content = content;
        }
        for part in children.iter_descendants(entity) {
            if let Ok(mut visibility) = indicators.get_mut(part) {
                visibility.set_if_neq(if selected {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
        }
    }
}

/// A container of panels, one for each tab of the row `tabs`, showing only
/// the selected tab's: spawn [`tab_panel`]s inside it.
pub fn tab_panels(tabs: Entity) -> impl Bundle {
    (
        TabPanels(tabs),
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            ..default()
        },
    )
}

/// Panels shown by the selection of the row of tabs it names.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct TabPanels(pub Entity);

/// The panel of the tab at an index among its row's tabs.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct TabPanel(pub usize);

/// The panel of the tab at `index`, shown only while that tab is selected.
pub fn tab_panel(index: usize) -> impl Bundle {
    (
        TabPanel(index),
        Node {
            flex_direction: FlexDirection::Column,
            display: Display::None,
            ..default()
        },
    )
}

/// The index of `selected` among the tabs of a row with `children`.
pub(crate) fn tab_index(
    selected: Option<Entity>,
    children: &Children,
    tabs: &Query<(), With<Tab>>,
) -> Option<usize> {
    let selected = selected?;
    children
        .iter()
        .filter(|child| tabs.contains(*child))
        .position(|child| child == selected)
}

/// A panel or container of panels new this frame.
type PanelAdded = Or<(Added<TabPanels>, Added<TabPanel>)>;

/// Shows each container's panel for its row's selected tab.
fn show_panels(
    rows: Query<(&SelectedTab, &Children), With<TabList>>,
    changed_rows: Query<(), (With<TabList>, Changed<SelectedTab>)>,
    added: Query<(), PanelAdded>,
    containers: Query<(&TabPanels, &Children)>,
    tabs: Query<(), With<Tab>>,
    mut panels: Query<(&TabPanel, &mut Node)>,
) {
    if changed_rows.is_empty() && added.is_empty() {
        return;
    }
    for (TabPanels(row), inside) in &containers {
        let Ok((selected, row_children)) = rows.get(*row) else {
            continue;
        };
        let shown = tab_index(selected.0, row_children, &tabs);
        for child in inside.iter() {
            if let Ok((TabPanel(index), mut node)) = panels.get_mut(child) {
                let display = if Some(*index) == shown {
                    Display::Flex
                } else {
                    Display::None
                };
                if node.display != display {
                    node.display = display;
                }
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, select_initial_tabs).add_systems(
        PostUpdate,
        (draw_tabs, show_panels).before(crate::PaintSystems),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_primary_tab_with_an_icon_is_taller_and_its_indicator_thicker() {
        assert_eq!(tab(TabsKind::Primary, "Music").height(), 48.0);
        assert_eq!(
            tab(TabsKind::Primary, "Music")
                .icon(crate::Glyph::Menu)
                .height(),
            64.0
        );
        assert_eq!(
            tab(TabsKind::Secondary, "Music")
                .icon(crate::Glyph::Menu)
                .height(),
            48.0
        );
        assert_eq!(TabsKind::Primary.selected(), (ColorRole::Primary, 3.0));
        assert_eq!(TabsKind::Secondary.selected(), (ColorRole::OnSurface, 2.0));
    }
}
