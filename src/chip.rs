//! Chips: Material 3's four kinds - assist, filter, input and suggestion -
//! 32 px tall, with small corners, in a set that wraps.
//!
//! What a game observes on the chip it spawned:
//! - an assist, input or suggestion chip sends [`Activate`] when used;
//! - a filter chip is Bevy's headless [`Checkbox`]: it sends
//!   [`ValueChange<bool>`] and turns itself on or off ([`Checked`]);
//! - an input chip's remove button sends [`ChipRemoved`] to the chip, and the
//!   game despawns it.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::{Activate, Button, Checkbox, checkbox_self_update, observe};

use crate::button::enabled;
use crate::focus::FocusRing;
use crate::icon::{Glyph, Icon, icon};
use crate::paint::{ContentColor, Look, Paint, Stroke, label};
use crate::theme::{ColorRole, Corner, Elevation, TypeRole};

/// A chip's height, in logical pixels.
pub const CHIP_HEIGHT: f32 = 32.0;
/// A chip's icon's side, in logical pixels.
const CHIP_ICON: f32 = 18.0;

/// Material's four kinds of chip.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum ChipKind {
    /// A smart action that appears in context: "Add to calendar".
    #[default]
    Assist,
    /// A choice among others, on or off, to narrow what is shown.
    Filter,
    /// A piece of information the player entered: a name, a tag.
    Input,
    /// A suggested reply or query, generated for the player.
    Suggestion,
}

/// Sent to an input chip when its remove button is pressed: the game takes
/// the item away and despawns the chip.
#[derive(EntityEvent, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipRemoved {
    /// The chip.
    pub entity: Entity,
}

/// A chip, as it is spawned.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialChip {
    kind: ChipKind,
    label: String,
    icon: Option<Icon>,
    elevated: bool,
    selected: bool,
    enabled: bool,
}

/// An assist chip saying `label`; change its kind with the builder.
pub fn chip(label: impl Into<String>) -> MaterialChip {
    MaterialChip {
        kind: ChipKind::Assist,
        label: label.into(),
        icon: None,
        elevated: false,
        selected: false,
        enabled: true,
    }
}

impl MaterialChip {
    /// Of `kind`.
    pub fn kind(mut self, kind: ChipKind) -> Self {
        self.kind = kind;
        self
    }

    /// A filter chip, on (`selected`) or off.
    pub fn filter(mut self, selected: bool) -> Self {
        self.kind = ChipKind::Filter;
        self.selected = selected;
        self
    }

    /// An input chip, with a remove button.
    pub fn input(self) -> Self {
        self.kind(ChipKind::Input)
    }

    /// A suggestion chip.
    pub fn suggestion(self) -> Self {
        self.kind(ChipKind::Suggestion)
    }

    /// With `icon` before its label; an input chip's may be an avatar, a
    /// picture of who or what it names, drawn round.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Raised on a low container instead of outlined: for a chip over a
    /// busy background.
    pub fn elevated(mut self) -> Self {
        self.elevated = true;
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Its components: spawn it, with the game's own components beside it.
    pub fn bundle(self) -> impl Bundle {
        let filter = self.kind == ChipKind::Filter;
        let input = self.kind == ChipKind::Input;
        let has_icon = self.icon.is_some() || (filter && self.selected);
        let start = if has_icon { 8.0 } else { 16.0 };
        let end = if input { 8.0 } else { 16.0 };
        let look = chip_look(self.kind, self.elevated, self.selected);
        let border = chip_border(self.kind, self.elevated, self.selected);
        let icon = self.icon;
        let words = self.label;
        let kind = self.kind;
        let selected = self.selected;
        (
            (
                self.kind,
                look,
                FocusRing,
                TabIndex(0),
                Stroke::new(ColorRole::OutlineVariant),
                BoxShadow::default(),
                enabled(self.enabled),
                ChipBehaviour::of(kind, selected),
                ChipElevated(self.elevated),
            ),
            Node {
                height: Val::Px(CHIP_HEIGHT),
                padding: UiRect::new(Val::Px(start), Val::Px(end), Val::ZERO, Val::ZERO),
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                border: UiRect::all(Val::Px(border)),
                border_radius: Corner::Small.all(),
                ..default()
            },
            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                let root = parent.target_entity();
                if filter {
                    parent
                        .spawn((crate::icon::icon(Glyph::Check, CHIP_ICON), FilterCheck))
                        .insert(Node {
                            width: Val::Px(CHIP_ICON),
                            height: Val::Px(CHIP_ICON),
                            flex_shrink: 0.0,
                            display: if selected {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            ..default()
                        });
                }
                if let Some(leading) = icon {
                    let avatar = input && matches!(leading, Icon::Image(_));
                    let side = if avatar { 24.0 } else { CHIP_ICON };
                    let mut spawned = parent.spawn(crate::icon::icon(leading, side));
                    if avatar {
                        spawned.insert(Node {
                            width: Val::Px(side),
                            height: Val::Px(side),
                            flex_shrink: 0.0,
                            overflow: Overflow::clip(),
                            border_radius: Corner::Full.all(),
                            ..default()
                        });
                    } else if kind == ChipKind::Assist {
                        spawned.insert(ContentColor(Paint::new(ColorRole::Primary)));
                    }
                }
                parent.spawn(label(words, TypeRole::LabelLarge));
                if input {
                    parent.spawn(remove_button(root));
                }
            })),
        )
    }
}

/// The behaviour beside a chip's look: a filter chip is a checkbox, every
/// other kind a button.
#[derive(Component, Clone, Copy, Debug)]
#[component(on_add = chip_behaviour_added)]
enum ChipBehaviour {
    Button,
    Filter { selected: bool },
}

impl ChipBehaviour {
    fn of(kind: ChipKind, selected: bool) -> Self {
        if kind == ChipKind::Filter {
            Self::Filter { selected }
        } else {
            Self::Button
        }
    }
}

/// Gives a chip Bevy's widget for its kind as it is spawned.
fn chip_behaviour_added(
    mut world: bevy::ecs::world::DeferredWorld,
    context: bevy::ecs::lifecycle::HookContext,
) {
    let Some(behaviour) = world.get::<ChipBehaviour>(context.entity).copied() else {
        return;
    };
    let mut commands = world.commands();
    let mut chip = commands.entity(context.entity);
    chip.try_remove::<ChipBehaviour>();
    match behaviour {
        ChipBehaviour::Button => {
            chip.try_insert(Button);
        }
        ChipBehaviour::Filter { selected } => {
            chip.try_insert((Checkbox, observe(checkbox_self_update)));
            if selected {
                chip.try_insert(Checked);
            }
        }
    }
}

/// Whether a chip stands raised rather than outlined.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct ChipElevated(bool);

/// The check a filter chip shows while on.
#[derive(Component, Clone, Copy, Debug, Default)]
struct FilterCheck;

/// On an input chip's remove button: the chip it removes.
#[derive(Component, Clone, Copy, Debug)]
struct Removes(Entity);

/// An input chip's remove button: a small round button with a cross.
fn remove_button(chip: Entity) -> impl Bundle {
    (
        Button,
        Removes(chip),
        Look::bare(ColorRole::OnSurfaceVariant),
        Node {
            width: Val::Px(24.0),
            height: Val::Px(24.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            border_radius: Corner::Full.all(),
            ..default()
        },
        children![icon(Glyph::Close, CHIP_ICON)],
    )
}

/// A chip's look at rest.
pub fn chip_look(kind: ChipKind, elevated: bool, selected: bool) -> Look {
    let look = match (kind, selected) {
        (ChipKind::Filter | ChipKind::Input, false) => Look::bare(ColorRole::OnSurfaceVariant),
        (ChipKind::Filter | ChipKind::Input, true) => Look::filled(
            ColorRole::SecondaryContainer,
            ColorRole::OnSecondaryContainer,
        ),
        (ChipKind::Assist | ChipKind::Suggestion, _) => Look::bare(ColorRole::OnSurface),
    };
    if elevated {
        Look {
            container: Some(look.container.unwrap_or(ColorRole::SurfaceContainerLow)),
            ..look
        }
        .raised(Elevation::Level1)
    } else {
        look
    }
}

/// A chip's border's width: none when raised or selected, 1 px otherwise.
pub fn chip_border(kind: ChipKind, elevated: bool, selected: bool) -> f32 {
    if elevated || (kind == ChipKind::Filter && selected) {
        0.0
    } else {
        1.0
    }
}

/// A row of chips that wraps onto more rows as it fills.
pub fn chip_set() -> impl Bundle {
    Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(8.0),
        row_gap: Val::Px(8.0),
        align_items: AlignItems::Center,
        ..default()
    }
}

/// A remove button tells its chip it is removed.
fn remove_on_activate(activate: On<Activate>, removes: Query<&Removes>, mut commands: Commands) {
    if let Ok(Removes(chip)) = removes.get(activate.entity) {
        commands.trigger(ChipRemoved { entity: *chip });
    }
}

/// What a filter chip's drawing reads and changes.
type FilterChipParts<'a> = (
    &'a ChipKind,
    &'a ChipElevated,
    Has<Checked>,
    &'a mut Look,
    &'a mut Node,
    &'a Children,
);

/// Draws each filter chip on or off: its colours, its border and its check.
fn draw_filter_chips(
    mut turned_off: RemovedComponents<Checked>,
    turned_on: Query<Entity, (With<ChipKind>, Added<Checked>)>,
    mut chips: Query<FilterChipParts>,
    mut checks: Query<&mut Node, (With<FilterCheck>, Without<ChipKind>)>,
) {
    let changed: Vec<Entity> = turned_on.iter().chain(turned_off.read()).collect();
    for entity in changed {
        let Ok((kind, elevated, selected, mut look, mut node, children)) = chips.get_mut(entity)
        else {
            continue;
        };
        if *kind != ChipKind::Filter {
            continue;
        }
        let wanted = chip_look(*kind, elevated.0, selected);
        if *look != wanted {
            *look = wanted;
        }
        let border = UiRect::all(Val::Px(chip_border(*kind, elevated.0, selected)));
        if node.border != border {
            node.border = border;
        }
        node.padding.left = Val::Px(if selected { 8.0 } else { 16.0 });
        for child in children.iter() {
            if let Ok(mut check) = checks.get_mut(child) {
                check.display = if selected {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

/// The chips' observers and drawing.
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(remove_on_activate).add_systems(
        PostUpdate,
        draw_filter_chips.before(crate::paint::paint_controls),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_selected_filter_chip_is_filled_with_the_secondary_container_and_has_no_border() {
        let on = chip_look(ChipKind::Filter, false, true);
        assert_eq!(on.container, Some(ColorRole::SecondaryContainer));
        assert_eq!(on.content, ColorRole::OnSecondaryContainer);
        assert_eq!(chip_border(ChipKind::Filter, false, true), 0.0);
        let off = chip_look(ChipKind::Filter, false, false);
        assert_eq!(off.container, None);
        assert_eq!(chip_border(ChipKind::Filter, false, false), 1.0);
    }

    #[test]
    fn an_elevated_chip_stands_on_a_low_container_with_no_border() {
        let raised = chip_look(ChipKind::Assist, true, false);
        assert_eq!(raised.container, Some(ColorRole::SurfaceContainerLow));
        assert_eq!(raised.elevation, Some(Elevation::Level1));
        assert_eq!(chip_border(ChipKind::Assist, true, false), 0.0);
        assert_eq!(chip_border(ChipKind::Assist, false, false), 1.0);
    }
}
