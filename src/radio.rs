//! Radio buttons: Bevy's headless [`RadioButton`] and [`RadioGroup`] drawn
//! as Material's - a 20 px ring in a 40 px target, primary with a dot when
//! chosen.
//!
//! The group holds the keyboard's focus: the arrow keys, Home and End move
//! the choice. A click on an option, or on its label, chooses it. The group
//! sends [`ValueChange<Entity>`] naming the radio button chosen, whose
//! [`RadioOption`] says which option it is; the radio button sends
//! [`ValueChange<bool>`]. A [`radio_group`] made self-updating (the default)
//! checks the option chosen and unchecks the rest; a controlled one waits for
//! the game to move [`Checked`].

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::{RadioButton, RadioGroup, ValueChange};

use crate::button::enabled;
use crate::checkbox::{CheckedControl, SelfUpdating, Starting, labelled, target_node};
use crate::focus::FocusRing;
use crate::paint::{Fill, Look, Paint, Stroke};
use crate::theme::{ColorRole, Corner, MaterialTheme, state};

/// The side of a radio button's ring, and of its dot, in logical pixels.
pub const RING: f32 = 20.0;
/// The side of a chosen radio button's dot, in logical pixels.
pub const DOT: f32 = 10.0;

/// Which option of its group a radio button is, from 0.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct RadioOption(pub usize);

/// What a radio button was last drawn as.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct RadioStyle {
    #[reflect(ignore)]
    drawn: Option<(bool, bool)>,
}

/// A radio button's ring.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct RadioRing;

/// A radio button's dot, shown while it is chosen.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct RadioDot;

/// A radio button: chosen or not, enabled or not.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialRadio {
    /// Whether it starts chosen.
    pub checked: bool,
    /// Whether it can be used.
    pub enabled: bool,
    /// Whether it checks itself when chosen - for a radio button alone; one
    /// in a [`radio_group`] is checked by its group.
    pub self_updating: bool,
}

/// A radio button, not chosen, that checks itself when chosen.
pub fn radio() -> MaterialRadio {
    MaterialRadio {
        checked: false,
        enabled: true,
        self_updating: true,
    }
}

impl MaterialRadio {
    /// Starting chosen, or not.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Waiting for the game, or its group, to insert [`Checked`].
    pub fn controlled(mut self) -> Self {
        self.self_updating = false;
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        (
            (
                RadioButton,
                RadioStyle::default(),
                Look::bare(ColorRole::OnSurface),
                enabled(self.enabled),
                Starting {
                    checked: self.checked,
                    self_updating: self.self_updating,
                },
            ),
            target_node(),
            children![(
                RadioRing,
                Node {
                    width: Val::Px(RING),
                    height: Val::Px(RING),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: Corner::Full.all(),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Stroke::new(ColorRole::OnSurfaceVariant),
                Pickable::IGNORE,
                children![(
                    RadioDot,
                    Node {
                        width: Val::Px(DOT),
                        height: Val::Px(DOT),
                        border_radius: Corner::Full.all(),
                        ..default()
                    },
                    Fill::new(ColorRole::Primary),
                    Visibility::Hidden,
                    Pickable::IGNORE,
                )],
            )],
        )
    }

    /// The radio button with `words` after it, in a row a click anywhere on
    /// which chooses it.
    pub fn labelled(self, words: impl Into<String>) -> impl Bundle {
        labelled(self.bundle(), words.into())
    }
}

/// A group of labelled options, one of which may be chosen.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialRadioGroup {
    /// Each option's label, in order.
    pub options: Vec<String>,
    /// The option chosen to start with.
    pub selected: Option<usize>,
    /// Options laid out across rather than down.
    pub horizontal: bool,
    /// Whether it checks the option chosen itself.
    pub self_updating: bool,
    /// Options that can't be chosen.
    pub disabled: Vec<usize>,
}

/// A group of radio buttons, one to each of `options`, with `selected`
/// chosen.
pub fn radio_group<S: Into<String>>(
    options: impl IntoIterator<Item = S>,
    selected: Option<usize>,
) -> MaterialRadioGroup {
    MaterialRadioGroup {
        options: options.into_iter().map(Into::into).collect(),
        selected,
        horizontal: false,
        self_updating: true,
        disabled: Vec::new(),
    }
}

impl MaterialRadioGroup {
    /// Laying its options out across.
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Waiting for the game to move [`Checked`] in answer to its
    /// [`ValueChange<Entity>`].
    pub fn controlled(mut self) -> Self {
        self.self_updating = false;
        self
    }

    /// With the option at `index` unusable.
    pub fn disable(mut self, index: usize) -> Self {
        self.disabled.push(index);
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let Self {
            options,
            selected,
            horizontal,
            self_updating,
            disabled,
        } = self;
        (
            RadioGroup,
            TabIndex(0),
            FocusRing,
            Starting {
                checked: false,
                self_updating,
            },
            Node {
                flex_direction: if horizontal {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                },
                column_gap: Val::Px(16.0),
                border_radius: Corner::ExtraSmall.all(),
                ..default()
            },
            Children::spawn(SpawnWith(move |group: &mut ChildSpawner| {
                for (index, words) in options.into_iter().enumerate() {
                    let option = radio()
                        .checked(selected == Some(index))
                        .enabled(!disabled.contains(&index))
                        .controlled();
                    group.spawn(labelled((option.bundle(), RadioOption(index)), words));
                }
            })),
        )
    }
}

/// Chooses `radio`, as a click does: it sends [`ValueChange<bool>`], and its
/// group, if it is in one, [`ValueChange<Entity>`].
pub(crate) fn choose(
    radio: Entity,
    groups: &Query<(), With<RadioGroup>>,
    parents: &Query<&ChildOf>,
    commands: &mut Commands,
) {
    commands.trigger(ValueChange::<bool> {
        source: radio,
        value: true,
        is_final: true,
    });
    if let Some(group) = parents
        .iter_ancestors(radio)
        .find(|ancestor| groups.contains(*ancestor))
    {
        commands.trigger(ValueChange::<Entity> {
            source: group,
            value: radio,
            is_final: true,
        });
    }
}

/// A self-updating group checks the radio button chosen, at any depth, and
/// unchecks the rest.
pub(crate) fn update_group(
    change: On<ValueChange<Entity>>,
    groups: Query<(), (With<RadioGroup>, With<SelfUpdating>)>,
    children: Query<&Children>,
    radios: Query<(), With<RadioButton>>,
    mut commands: Commands,
) {
    if !groups.contains(change.source) {
        return;
    }
    for part in children.iter_descendants(change.source) {
        if !radios.contains(part) {
            continue;
        }
        if part == change.value {
            commands.entity(part).try_insert(Checked);
        } else {
            commands.entity(part).try_remove::<Checked>();
        }
    }
}

/// A self-updating radio button on its own checks itself when chosen.
pub(crate) fn update_radio(
    change: On<ValueChange<bool>>,
    radios: Query<(), (With<RadioButton>, With<SelfUpdating>)>,
    mut commands: Commands,
) {
    if change.value && radios.contains(change.source) {
        commands.entity(change.source).try_insert(Checked);
    }
}

/// The colours of a radio button's ring and its state layer, in a state.
pub fn radio_paints(checked: bool, disabled: bool) -> (Paint, ColorRole) {
    match (checked, disabled) {
        (_, true) => (
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
            ColorRole::OnSurface,
        ),
        (true, false) => (Paint::new(ColorRole::Primary), ColorRole::Primary),
        (false, false) => (
            Paint::new(ColorRole::OnSurfaceVariant),
            ColorRole::OnSurface,
        ),
    }
}

/// Draws each radio button as it is chosen or disabled.
pub(crate) fn draw_radios(
    theme: Res<MaterialTheme>,
    mut radios: Query<CheckedControl<RadioStyle>>,
    children: Query<&Children>,
    rings: Query<(), With<RadioRing>>,
    dots: Query<(), With<RadioDot>>,
    mut commands: Commands,
) {
    for (entity, mut style, mut look, checked, disabled) in &mut radios {
        let now = (checked, disabled);
        if style.drawn == Some(now) && !theme.is_changed() {
            continue;
        }
        style.drawn = Some(now);
        let (ring, layer) = radio_paints(checked, disabled);
        if look.content != layer {
            look.content = layer;
        }
        for part in children.iter_descendants(entity) {
            if rings.contains(part) {
                commands.entity(part).try_insert(Stroke(ring));
            } else if dots.contains(part) {
                commands.entity(part).try_insert((
                    Fill(ring),
                    if checked {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                ));
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(update_group)
        .add_observer(update_radio)
        .add_systems(PostUpdate, draw_radios.before(crate::paint::paint_controls));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chosen_radio_button_is_primary_and_one_not_chosen_on_surface_variant() {
        assert_eq!(radio_paints(true, false).0, Paint::new(ColorRole::Primary));
        assert_eq!(
            radio_paints(false, false).0,
            Paint::new(ColorRole::OnSurfaceVariant)
        );
        assert_eq!(radio_paints(true, true).0.opacity, state::DISABLED_CONTENT);
    }
}
