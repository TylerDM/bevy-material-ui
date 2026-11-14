//! Lists: rows of one, two or three lines, each with an optional leading
//! icon or avatar and trailing text, icon or control.
//!
//! A [`list`] is Bevy's headless [`ListBox`]: a click on a row selects it, and
//! while the list has the keyboard's focus the arrow keys, Home and End move
//! between its rows and Enter or Space selects one. It sends
//! [`ValueChange<Entity>`](bevy::ui_widgets::ValueChange) - the row chosen - on the list, and by default
//! keeps its own selection; the selected row is drawn in
//! [`ColorRole::SecondaryContainer`].

use bevy::input_focus::InputFocusVisible;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Selected};
use bevy::ui_widgets::{ActiveDescendant, ListBox, ListItem, listbox_update_selection, observe};

use crate::focus::FocusRing;
use crate::icon::Icon;
use crate::paint::{Content, ContentColor, Fill, Look, Paint, Typeset};
use crate::theme::{ColorRole, MaterialTheme, TypeRole, state};

/// A part drawn in a quieter content colour than its control's: a row's
/// supporting text, a destination's label. It is faded with its control
/// when the control is disabled.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(ContentColor(Paint::new(ColorRole::OnSurfaceVariant)))]
pub(crate) struct Quiet(pub(crate) ColorRole);

/// Words of a kind of text in a quiet colour, faded with their control.
pub(crate) fn quiet_text(
    words: impl Into<String>,
    role: TypeRole,
    color: ColorRole,
) -> impl Bundle {
    (
        Text::new(words),
        Typeset(role),
        Content,
        Quiet(color),
        ContentColor(Paint::new(color)),
        Pickable::IGNORE,
    )
}

/// A slot whose icons are drawn in a quiet colour, faded with their control.
pub(crate) fn quiet_slot(color: ColorRole) -> impl Bundle {
    (
        Node {
            flex_shrink: 0.0,
            ..default()
        },
        Quiet(color),
        ContentColor(Paint::new(color)),
        Pickable::IGNORE,
    )
}

/// Draws each quiet part in its colour, or faded while its control is
/// disabled.
pub(crate) fn fade_quiet(
    added: Query<(), Added<Quiet>>,
    disabled: Query<(), Added<InteractionDisabled>>,
    mut enabled: RemovedComponents<InteractionDisabled>,
    quiet: Query<(Entity, &Quiet)>,
    controls: Query<Has<InteractionDisabled>, With<Look>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let changed = !added.is_empty() || !disabled.is_empty() || enabled.read().count() > 0;
    if !changed {
        return;
    }
    for (entity, Quiet(role)) in &quiet {
        let faded = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| controls.get(ancestor).ok())
            .unwrap_or(false);
        let paint = if faded {
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)
        } else {
            Paint::new(*role)
        };
        commands.entity(entity).try_insert(ContentColor(paint));
    }
}

/// Marks a list.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MaterialList;

/// A list of rows: spawn [`list_item`]s inside it, and [`crate::divider`]s
/// between groups. It keeps its own selection; a game that keeps it instead
/// observes [`ValueChange<Entity>`](bevy::ui_widgets::ValueChange) and inserts or removes `Selected` on rows
/// itself, spawning the list with [`list_unmanaged`].
pub fn list() -> impl Bundle {
    (list_unmanaged(), observe(listbox_update_selection))
}

/// A list whose selection the game keeps: it sends [`ValueChange<Entity>`](bevy::ui_widgets::ValueChange)
/// and changes nothing itself.
pub fn list_unmanaged() -> impl Bundle {
    (
        MaterialList,
        ListBox,
        TabIndex(0),
        FocusRing,
        Fill::new(ColorRole::Surface),
        ContentColor(Paint::new(ColorRole::OnSurface)),
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::vertical(Val::Px(8.0)),
            ..default()
        },
    )
}

/// What is before a row's words.
#[derive(Clone, Debug, PartialEq)]
pub enum Leading {
    /// An icon, in [`ColorRole::OnSurfaceVariant`].
    Icon(Icon),
    /// A round avatar of initials, in the primary container.
    Avatar(String),
}

/// What is after a row's words.
#[derive(Clone, Debug, PartialEq)]
pub enum Trailing {
    /// Short supporting text: a count, a time.
    Text(String),
    /// An icon.
    Icon(Icon),
    /// An empty slot, marked [`TrailingSlot`], for a control the game
    /// spawns into it: a switch, a checkbox.
    Slot,
}

/// The slot at the end of a row that a game puts a control in.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct TrailingSlot;

/// Marks a row of a list.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ListRow;

/// A row: its headline, supporting text, leading and trailing parts.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialListItem {
    /// Its first line.
    pub headline: String,
    /// The line or two under it.
    pub supporting: Option<String>,
    /// Whether the supporting text may take two lines, making a three-line
    /// row.
    pub three_line: bool,
    /// Before the words.
    pub leading: Option<Leading>,
    /// After the words.
    pub trailing: Option<Trailing>,
    /// Whether it can be chosen.
    pub enabled: bool,
}

/// A one-line row saying `headline`.
pub fn list_item(headline: impl Into<String>) -> MaterialListItem {
    MaterialListItem {
        headline: headline.into(),
        supporting: None,
        three_line: false,
        leading: None,
        trailing: None,
        enabled: true,
    }
}

/// How many lines a row has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum Lines {
    /// A headline alone: 56 px.
    One,
    /// A headline and a line of supporting text: 72 px.
    Two,
    /// A headline and two lines of supporting text: 88 px.
    Three,
}

impl Lines {
    /// A row's height with this many lines, in logical pixels.
    pub fn height(self) -> f32 {
        match self {
            Lines::One => 56.0,
            Lines::Two => 72.0,
            Lines::Three => 88.0,
        }
    }
}

impl MaterialListItem {
    /// With `supporting` text under the headline, on one line.
    pub fn supporting(mut self, supporting: impl Into<String>) -> Self {
        self.supporting = Some(supporting.into());
        self
    }

    /// With `supporting` text under the headline, on up to two lines.
    pub fn three_line(mut self, supporting: impl Into<String>) -> Self {
        self.supporting = Some(supporting.into());
        self.three_line = true;
        self
    }

    /// With `icon` before the words.
    pub fn leading_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.leading = Some(Leading::Icon(icon.into()));
        self
    }

    /// With an avatar of `initials` before the words.
    pub fn avatar(mut self, initials: impl Into<String>) -> Self {
        self.leading = Some(Leading::Avatar(initials.into()));
        self
    }

    /// With `text` after the words.
    pub fn trailing_text(mut self, text: impl Into<String>) -> Self {
        self.trailing = Some(Trailing::Text(text.into()));
        self
    }

    /// With `icon` after the words.
    pub fn trailing_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.trailing = Some(Trailing::Icon(icon.into()));
        self
    }

    /// With an empty [`TrailingSlot`] after the words, for a control.
    pub fn trailing_slot(mut self) -> Self {
        self.trailing = Some(Trailing::Slot);
        self
    }

    /// Enabled, or drawn faded and unchoosable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// How many lines it has.
    pub fn lines(&self) -> Lines {
        match (&self.supporting, self.three_line) {
            (None, _) => Lines::One,
            (Some(_), false) => Lines::Two,
            (Some(_), true) => Lines::Three,
        }
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let lines = self.lines();
        let MaterialListItem {
            headline,
            supporting,
            leading,
            trailing,
            enabled,
            ..
        } = self;
        let align = if lines == Lines::Three {
            AlignItems::FlexStart
        } else {
            AlignItems::Center
        };
        (
            (
                ListRow,
                ListItem,
                Look::bare(ColorRole::OnSurface),
                crate::button::enabled(enabled),
            ),
            Node {
                min_height: Val::Px(lines.height()),
                padding: UiRect::new(
                    Val::Px(16.0),
                    Val::Px(24.0),
                    Val::Px(if lines == Lines::Three { 12.0 } else { 8.0 }),
                    Val::Px(if lines == Lines::Three { 12.0 } else { 8.0 }),
                ),
                column_gap: Val::Px(16.0),
                align_items: align,
                flex_shrink: 0.0,
                ..default()
            },
            Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
                match leading {
                    Some(Leading::Icon(glyph)) => {
                        row.spawn(quiet_slot(ColorRole::OnSurfaceVariant))
                            .with_child(crate::icon::icon(glyph, crate::icon::ICON));
                    }
                    Some(Leading::Avatar(initials)) => {
                        row.spawn(crate::badge::avatar(
                            crate::badge::AvatarFace::Initials(initials),
                            crate::badge::AVATAR,
                        ));
                    }
                    None => {}
                }
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        flex_basis: Val::ZERO,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|words| {
                    words.spawn(crate::paint::label(headline, TypeRole::BodyLarge));
                    if let Some(supporting) = supporting {
                        words.spawn(quiet_text(
                            supporting,
                            TypeRole::BodyMedium,
                            ColorRole::OnSurfaceVariant,
                        ));
                    }
                });
                match trailing {
                    Some(Trailing::Text(words)) => {
                        row.spawn(quiet_text(
                            words,
                            TypeRole::LabelSmall,
                            ColorRole::OnSurfaceVariant,
                        ));
                    }
                    Some(Trailing::Icon(glyph)) => {
                        row.spawn(quiet_slot(ColorRole::OnSurfaceVariant))
                            .with_child(crate::icon::icon(glyph, crate::icon::ICON));
                    }
                    Some(Trailing::Slot) => {
                        row.spawn((TrailingSlot, Node::default()));
                    }
                    None => {}
                }
            })),
        )
    }
}

/// The look of a row, chosen or not.
pub fn row_look(selected: bool) -> Look {
    if selected {
        Look::filled(
            ColorRole::SecondaryContainer,
            ColorRole::OnSecondaryContainer,
        )
    } else {
        Look::bare(ColorRole::OnSurface)
    }
}

/// Draws each row as chosen or not, as its `Selected` comes and goes.
fn draw_rows(
    chosen: Query<Entity, (With<ListRow>, Added<Selected>)>,
    mut unchosen: RemovedComponents<Selected>,
    mut rows: Query<(&mut Look, Has<Selected>), With<ListRow>>,
) {
    let changed: Vec<Entity> = chosen.iter().chain(unchosen.read()).collect();
    for row in changed {
        if let Ok((mut look, selected)) = rows.get_mut(row) {
            look.set_if_neq(row_look(selected));
        }
    }
}

/// A list whose keyboard row moved this frame.
type ActiveChanged = (With<MaterialList>, Changed<ActiveDescendant>);

/// Outlines the row the keyboard is on in a focused list.
fn draw_active_rows(
    theme: Res<MaterialTheme>,
    visible: Option<Res<InputFocusVisible>>,
    lists: Query<(&ActiveDescendant, &Children), ActiveChanged>,
    children: Query<&Children>,
    rows: Query<(), With<ListRow>>,
    mut commands: Commands,
) {
    let shown = visible.is_some_and(|visible| visible.0);
    for (active, kids) in &lists {
        let mut stack: Vec<Entity> = kids.iter().collect();
        while let Some(entity) = stack.pop() {
            if rows.contains(entity) {
                let color = if shown && active.0 == Some(entity) {
                    theme.color(ColorRole::Secondary)
                } else {
                    Color::NONE
                };
                commands.entity(entity).try_insert(Outline::new(
                    Val::Px(3.0),
                    Val::Px(-3.0),
                    color,
                ));
            } else if let Ok(inner) = children.get(entity) {
                stack.extend(inner.iter());
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (draw_rows, draw_active_rows, fade_quiet).before(crate::PaintSystems),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_has_one_two_or_three_lines_by_its_supporting_text() {
        assert_eq!(list_item("Inbox").lines(), Lines::One);
        assert_eq!(
            list_item("Inbox").supporting("Twelve new").lines(),
            Lines::Two
        );
        assert_eq!(
            list_item("Inbox").three_line("Twelve new").lines(),
            Lines::Three
        );
        assert_eq!(Lines::One.height(), 56.0);
        assert_eq!(Lines::Two.height(), 72.0);
        assert_eq!(Lines::Three.height(), 88.0);
    }

    #[test]
    fn a_chosen_row_is_drawn_in_the_secondary_container() {
        assert_eq!(
            row_look(true).container,
            Some(ColorRole::SecondaryContainer)
        );
        assert_eq!(row_look(false).container, None);
    }
}
