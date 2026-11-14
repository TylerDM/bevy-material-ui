//! Search: Material 3's search bar - a pill holding a text input, a leading
//! icon and a clear button - and a list of results under it, whose rows the
//! game supplies.
//!
//! The game observes, on the bar it spawned, [`ValueChange<String>`] as the
//! query is typed (final on Enter or when it loses the keyboard) and
//! [`SearchNavigate`] when its leading icon is pressed; it reads the query
//! from the bar's [`InputText`]. Each result row is a button sending
//! [`Activate`].

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::{Activate, Button, TextInput};

use crate::button::{IconButtonVariant, icon_button};
use crate::focus::FocusRing;
use crate::icon::{Glyph, Icon};
use crate::paint::{ContentColor, Fill, Ink, Look, Paint, Raised, Typeset, label};
use crate::text_field::{Clears, InputOf, InputText};
use crate::theme::{ColorRole, Corner, Elevation, MaterialTheme, TypeRole};

/// A search bar's height, in logical pixels.
pub const SEARCH_HEIGHT: f32 = 56.0;

/// Sent to a search bar when its leading icon is pressed: open the menu, or
/// go back.
#[derive(EntityEvent, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchNavigate {
    /// The search bar.
    pub entity: Entity,
}

/// A search bar, as the game finds it once spawned.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchBar {
    leading: Entity,
    placeholder: Entity,
    clear: Entity,
}

impl SearchBar {
    /// Its leading icon button.
    pub fn leading(&self) -> Entity {
        self.leading
    }

    /// Its clear button.
    pub fn clear(&self) -> Entity {
        self.clear
    }
}

/// On a search bar's leading button: the bar it navigates for.
#[derive(Component, Clone, Copy, Debug)]
struct NavigatesFor(Entity);

/// A search bar showing `placeholder` while empty, with `leading` before it,
/// such as [`Glyph::Menu`] or [`Glyph::ArrowBack`], and a clear button after
/// it while it holds a query. It fills its parent's width.
pub fn search_bar(placeholder: impl Into<String>, leading: impl Into<Icon>) -> impl Bundle {
    let placeholder = placeholder.into();
    let leading = leading.into();
    (
        InputText::default(),
        Node {
            height: Val::Px(SEARCH_HEIGHT),
            width: Val::Percent(100.0),
            min_width: Val::Px(360.0),
            padding: UiRect::horizontal(Val::Px(4.0)),
            column_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
            border_radius: Corner::Full.all(),
            flex_shrink: 0.0,
            ..default()
        },
        Fill::new(ColorRole::SurfaceContainerHigh),
        ContentColor(Paint::new(ColorRole::OnSurfaceVariant)),
        Raised(Elevation::Level0),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            let root = parent.target_entity();
            let leading = parent
                .spawn((
                    icon_button(leading, IconButtonVariant::Standard),
                    NavigatesFor(root),
                ))
                .id();
            let mut parts = (Entity::PLACEHOLDER, Entity::PLACEHOLDER);
            parent
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(TypeRole::BodyLarge.line_height()),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|middle| {
                    parts.0 = middle
                        .spawn((
                            TextInput,
                            EditableText::default(),
                            InputOf(root),
                            TabIndex(0),
                            Typeset(TypeRole::BodyLarge),
                            Ink::new(ColorRole::OnSurface),
                            TextLayout::no_wrap(),
                            TextCursorStyle::default(),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::ZERO,
                                right: Val::ZERO,
                                top: Val::ZERO,
                                bottom: Val::ZERO,
                                ..default()
                            },
                        ))
                        .id();
                    parts.1 = middle
                        .spawn((
                            Text::new(placeholder),
                            Typeset(TypeRole::BodyLarge),
                            Ink::new(ColorRole::OnSurfaceVariant),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::ZERO,
                                top: Val::ZERO,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .id();
                });
            let (input, placeholder) = parts;
            let clear = parent
                .spawn((
                    icon_button(Glyph::Close, IconButtonVariant::Standard),
                    Clears(input),
                ))
                .id();
            let world = parent.world_mut();
            if let Some(mut text) = world.get_mut::<InputText>(root) {
                *text = InputText::new(input);
            }
            world.entity_mut(root).insert(SearchBar {
                leading,
                placeholder,
                clear,
            });
        })),
    )
}

/// A list of results under a search bar: a raised surface the game puts
/// [`search_result`] rows in.
pub fn search_results() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::vertical(Val::Px(8.0)),
            border_radius: Corner::Large.all(),
            ..default()
        },
        Fill::new(ColorRole::SurfaceContainerHigh),
        ContentColor(Paint::new(ColorRole::OnSurface)),
        Raised(Elevation::Level2),
    )
}

/// Marks a search result row.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SearchResult;

/// A result row saying `headline`, with `supporting` text under it if any:
/// a button sending [`Activate`] when chosen.
pub fn search_result(headline: impl Into<String>, supporting: Option<&str>) -> impl Bundle {
    let headline = headline.into();
    let supporting = supporting.map(str::to_owned);
    (
        Button,
        SearchResult,
        Look::bare(ColorRole::OnSurface),
        FocusRing,
        TabIndex(0),
        Node {
            min_height: Val::Px(56.0),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(label(headline, TypeRole::BodyLarge));
            if let Some(supporting) = supporting {
                parent.spawn((
                    Text::new(supporting),
                    Typeset(TypeRole::BodyMedium),
                    Ink::new(ColorRole::OnSurfaceVariant),
                    Pickable::IGNORE,
                ));
            }
        })),
    )
}

/// A leading button tells its bar to navigate.
fn navigate_on_activate(
    activate: On<Activate>,
    leading: Query<&NavigatesFor>,
    mut commands: Commands,
) {
    if let Ok(NavigatesFor(bar)) = leading.get(activate.entity) {
        commands.trigger(SearchNavigate { entity: *bar });
    }
}

/// A search bar spawned or typed into this frame.
type BarChanged = Or<(Changed<InputText>, Added<SearchBar>)>;

/// Shows each bar's placeholder only while it is empty, and its clear
/// button only while it holds a query.
fn draw_search_bars(
    theme: Res<MaterialTheme>,
    bars: Query<(&SearchBar, &InputText), BarChanged>,
    all: Query<&InputText, With<SearchBar>>,
    mut visibilities: Query<&mut Visibility>,
    mut nodes: Query<&mut Node>,
    mut cursors: Query<&mut TextCursorStyle>,
) {
    for (bar, text) in &bars {
        let empty = text.text().is_empty();
        if let Ok(mut visibility) = visibilities.get_mut(bar.placeholder) {
            visibility.set_if_neq(if empty {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
        if let Ok(mut node) = nodes.get_mut(bar.clear) {
            let display = if empty { Display::None } else { Display::Flex };
            if node.display != display {
                node.display = display;
            }
        }
    }
    let changed: Vec<Entity> = if theme.is_changed() {
        all.iter().map(InputText::input).collect()
    } else {
        bars.iter().map(|(_, text)| text.input()).collect()
    };
    for input in changed {
        if let Ok(mut cursor) = cursors.get_mut(input) {
            cursor.color = theme.color(ColorRole::Primary);
            cursor.selection_color = theme.color(ColorRole::Primary).with_alpha(0.3);
        }
    }
}

/// The search bars' observers and drawing.
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(navigate_on_activate).add_systems(
        PostUpdate,
        draw_search_bars.before(crate::paint::paint_controls),
    );
}
