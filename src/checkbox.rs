//! Checkboxes: Bevy's headless [`Checkbox`] drawn as Material's - an 18 px
//! box in a 40 px target, filled with primary and ticked when checked.
//!
//! A click, or Enter or Space while it has the keyboard's focus, sends
//! [`ValueChange<bool>`] on the checkbox. A checkbox made
//! [`MaterialCheckbox::self_updating`] (the default) checks or unchecks
//! itself; one made [`MaterialCheckbox::controlled`] waits for the game to
//! insert or remove [`Checked`].

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::events::PointerClick;
use bevy::prelude::*;
use bevy::ui::{Checked, InteractionDisabled};
use bevy::ui_widgets::{Checkbox, RadioButton, RadioGroup, ToggleChecked, ValueChange};

use crate::button::enabled;
use crate::focus::FocusRing;
use crate::icon::{Glyph, icon};
use crate::paint::{ContentColor, Fill, Look, Paint, Stroke, label};
use crate::theme::{ColorRole, Corner, MaterialTheme, TypeRole, layered, state};

/// A control that checks or unchecks itself when used, rather than waiting
/// for the game to insert or remove [`Checked`]: on a checkbox, a switch, a
/// segment, or a radio group (which checks the radio button chosen).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SelfUpdating;

/// A row holding a control and its label: a click anywhere on it uses the
/// control, as a click on a label does.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct LabelRow;

/// The side of a selection control's target, in logical pixels: the circle
/// its state layer is drawn in.
pub const TARGET: f32 = 40.0;
/// The side of a checkbox's box, in logical pixels.
pub const BOX: f32 = 18.0;

/// A checkbox's own state: whether it is drawn as an error, or as neither
/// on nor off.
#[derive(Component, Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct CheckboxStyle {
    /// Drawn in the error colours, as a required box left unchecked.
    pub error: bool,
    /// Drawn with a dash, neither on nor off: a box over others some of
    /// which are checked. Its value still flips with [`Checked`].
    pub indeterminate: bool,
    /// What it was last drawn as, so it is drawn again only when that
    /// changes.
    #[reflect(ignore)]
    drawn: Option<(bool, bool, bool, bool)>,
}

/// The box inside a checkbox.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct CheckboxBox;

/// The mark inside a checkbox's box: a tick, or a dash.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct CheckboxMark(pub Glyph);

/// A checkbox: checked or not, an error or not, indeterminate or not,
/// enabled or not.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialCheckbox {
    /// Whether it starts checked.
    pub checked: bool,
    /// Drawn in the error colours.
    pub error: bool,
    /// Drawn with a dash.
    pub indeterminate: bool,
    /// Whether it can be used.
    pub enabled: bool,
    /// Whether it checks itself when used.
    pub self_updating: bool,
}

/// An unchecked checkbox that checks itself when used.
pub fn checkbox() -> MaterialCheckbox {
    MaterialCheckbox {
        checked: false,
        error: false,
        indeterminate: false,
        enabled: true,
        self_updating: true,
    }
}

impl MaterialCheckbox {
    /// Starting checked, or not.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Drawn in the error colours.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Drawn with a dash, neither on nor off.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Checking itself when used - the default.
    pub fn self_updating(mut self) -> Self {
        self.self_updating = true;
        self
    }

    /// Waiting for the game to insert or remove [`Checked`] in answer to its
    /// [`ValueChange<bool>`].
    pub fn controlled(mut self) -> Self {
        self.self_updating = false;
        self
    }

    /// Its components: spawn it, with the game's own beside it.
    pub fn bundle(self) -> impl Bundle {
        (
            (
                Checkbox,
                CheckboxStyle {
                    error: self.error,
                    indeterminate: self.indeterminate,
                    drawn: None,
                },
                Look::bare(ColorRole::OnSurface),
                FocusRing,
                TabIndex(0),
                enabled(self.enabled),
                Starting {
                    checked: self.checked,
                    self_updating: self.self_updating,
                },
            ),
            target_node(),
            children![(
                CheckboxBox,
                Node {
                    width: Val::Px(BOX),
                    height: Val::Px(BOX),
                    border: UiRect::all(Val::Px(BORDER)),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Fill(Paint::faded(ColorRole::Primary, 0.0)),
                Stroke::new(ColorRole::OnSurfaceVariant),
                ContentColor(Paint::new(ColorRole::OnPrimary)),
                Pickable::IGNORE,
                children![mark(Glyph::Check), mark(Glyph::Dash)],
            )],
        )
    }

    /// The checkbox with `words` after it, in a row a click anywhere on
    /// which uses it.
    pub fn labelled(self, words: impl Into<String>) -> impl Bundle {
        labelled(self.bundle(), words.into())
    }
}

/// A checkbox's mark, hidden until drawn: laid over the box's inside, so a
/// tick and a dash take the same place, and sized to it, inside its border.
fn mark(glyph: Glyph) -> impl Bundle {
    (
        CheckboxMark(glyph),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        Pickable::IGNORE,
        children![icon(glyph, BOX - 2.0 * BORDER)],
    )
}

/// The width of a checkbox's box's border, in logical pixels.
const BORDER: f32 = 2.0;

/// A control with `words` after it, in a [`LabelRow`].
pub(crate) fn labelled(control: impl Bundle, words: String) -> impl Bundle {
    (
        LabelRow,
        Node {
            align_items: AlignItems::Center,
            padding: UiRect::right(Val::Px(12.0)),
            column_gap: Val::Px(4.0),
            min_height: Val::Px(TARGET),
            ..default()
        },
        Children::spawn((Spawn(control), Spawn(label(words, TypeRole::BodyLarge)))),
    )
}

/// A 40 px round target, centring what is inside it.
pub(crate) fn target_node() -> Node {
    Node {
        width: Val::Px(TARGET),
        height: Val::Px(TARGET),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        border_radius: Corner::Full.all(),
        ..default()
    }
}

/// How a control is to start: checked or not, updating itself or not. Taken
/// off as it is spawned.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Starting {
    pub(crate) checked: bool,
    pub(crate) self_updating: bool,
}

pub(crate) fn on_starting(
    add: On<Add<Starting>>,
    starts: Query<&Starting>,
    mut commands: Commands,
) {
    let Ok(start) = starts.get(add.entity) else {
        return;
    };
    let mut control = commands.entity(add.entity);
    control.remove::<Starting>();
    if start.checked {
        control.insert(Checked);
    }
    if start.self_updating {
        control.insert(SelfUpdating);
    }
}

/// A self-updating checkbox, switch or segment checks or unchecks itself.
pub(crate) fn update_self(
    change: On<ValueChange<bool>>,
    controls: Query<(), (With<SelfUpdating>, Without<RadioButton>)>,
    mut commands: Commands,
) {
    if !controls.contains(change.source) {
        return;
    }
    if change.value {
        commands.entity(change.source).insert(Checked);
    } else {
        commands.entity(change.source).remove::<Checked>();
    }
}

/// A click on a [`LabelRow`] off its control uses the control: a checkbox
/// or switch flips, a radio button is chosen.
#[expect(clippy::too_many_arguments, reason = "a system's queries")]
pub(crate) fn click_label(
    click: On<PointerClick>,
    rows: Query<(), With<LabelRow>>,
    children: Query<&Children>,
    checkboxes: Query<(), With<Checkbox>>,
    radios: Query<(Has<Checked>, Has<InteractionDisabled>), With<RadioButton>>,
    groups: Query<(), With<RadioGroup>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    if !rows.contains(click.entity) {
        return;
    }
    for part in children.iter_descendants(click.entity) {
        if checkboxes.contains(part) {
            commands.trigger(ToggleChecked { entity: part });
            return;
        }
        if let Ok((checked, disabled)) = radios.get(part) {
            if !checked && !disabled {
                crate::radio::choose(part, &groups, &parents, &mut commands);
            }
            return;
        }
    }
}

/// The colour of a state layer of `role` over nothing, in a state: what a
/// control's halo is drawn in.
pub(crate) fn halo(theme: &MaterialTheme, role: ColorRole, hovered: bool, pressed: bool) -> Color {
    let opacity = if pressed {
        state::PRESSED
    } else if hovered {
        state::HOVERED
    } else {
        return Color::NONE;
    };
    layered(Color::NONE, theme.color(role), opacity)
}

/// What a checkbox's box, mark and state layer are drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxPaints {
    /// The box's fill.
    pub fill: Paint,
    /// The box's outline.
    pub outline: Paint,
    /// The mark's colour.
    pub mark: Paint,
    /// The role of the state layer around it.
    pub layer: ColorRole,
}

/// What a checkbox in a state is drawn in.
pub fn checkbox_paints(on: bool, error: bool, disabled: bool) -> CheckboxPaints {
    let (accent, on_accent) = if error {
        (ColorRole::Error, ColorRole::OnError)
    } else {
        (ColorRole::Primary, ColorRole::OnPrimary)
    };
    match (on, disabled) {
        (true, false) => CheckboxPaints {
            fill: Paint::new(accent),
            outline: Paint::new(accent),
            mark: Paint::new(on_accent),
            layer: accent,
        },
        (false, false) => CheckboxPaints {
            fill: Paint::faded(accent, 0.0),
            outline: Paint::new(if error {
                ColorRole::Error
            } else {
                ColorRole::OnSurfaceVariant
            }),
            mark: Paint::new(on_accent),
            layer: if error {
                ColorRole::Error
            } else {
                ColorRole::OnSurface
            },
        },
        (true, true) => CheckboxPaints {
            fill: Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
            outline: Paint::faded(ColorRole::OnSurface, 0.0),
            mark: Paint::new(ColorRole::Surface),
            layer: ColorRole::OnSurface,
        },
        (false, true) => CheckboxPaints {
            fill: Paint::faded(ColorRole::OnSurface, 0.0),
            outline: Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
            mark: Paint::new(ColorRole::Surface),
            layer: ColorRole::OnSurface,
        },
    }
}

/// A control that is checked or not: its entity, what it was last drawn
/// as, its look, whether it is checked and whether it is disabled.
pub(crate) type CheckedControl<'a, S> = (
    Entity,
    &'a mut S,
    &'a mut Look,
    Has<Checked>,
    Has<InteractionDisabled>,
);

/// Draws each checkbox as it is checked, indeterminate, an error or
/// disabled.
pub(crate) fn draw_checkboxes(
    theme: Res<MaterialTheme>,
    mut checkboxes: Query<CheckedControl<CheckboxStyle>>,
    children: Query<&Children>,
    boxes: Query<(), With<CheckboxBox>>,
    marks: Query<&CheckboxMark>,
    mut commands: Commands,
) {
    for (entity, mut style, mut look, checked, disabled) in &mut checkboxes {
        let now = (checked, style.indeterminate, style.error, disabled);
        if style.drawn == Some(now) && !theme.is_changed() {
            continue;
        }
        style.drawn = Some(now);
        let on = checked || style.indeterminate;
        let paints = checkbox_paints(on, style.error, disabled);
        if look.content != paints.layer {
            look.content = paints.layer;
        }
        for part in children.iter_descendants(entity) {
            if boxes.contains(part) {
                commands.entity(part).insert((
                    Fill(paints.fill),
                    Stroke(paints.outline),
                    ContentColor(paints.mark),
                ));
            } else if let Ok(CheckboxMark(glyph)) = marks.get(part) {
                let shown = match glyph {
                    Glyph::Dash => style.indeterminate,
                    _ => checked && !style.indeterminate,
                };
                commands.entity(part).insert(if shown {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_starting)
        .add_observer(update_self)
        .add_observer(click_label)
        .add_systems(
            PostUpdate,
            draw_checkboxes.before(crate::paint::paint_controls),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_checked_checkbox_is_filled_with_primary_and_ticked_on_primary() {
        let paints = checkbox_paints(true, false, false);
        assert_eq!(paints.fill, Paint::new(ColorRole::Primary));
        assert_eq!(paints.mark, Paint::new(ColorRole::OnPrimary));
        assert_eq!(paints.layer, ColorRole::Primary);
    }

    #[test]
    fn an_unchecked_checkbox_is_an_outline_in_on_surface_variant_and_an_error_one_in_error() {
        let paints = checkbox_paints(false, false, false);
        assert_eq!(paints.outline, Paint::new(ColorRole::OnSurfaceVariant));
        assert_eq!(paints.fill.opacity, 0.0);
        assert_eq!(
            checkbox_paints(false, true, false).outline,
            Paint::new(ColorRole::Error)
        );
        assert_eq!(
            checkbox_paints(true, true, false).fill,
            Paint::new(ColorRole::Error)
        );
    }

    #[test]
    fn a_disabled_checkbox_is_drawn_in_faded_on_surface() {
        let on = checkbox_paints(true, false, true);
        assert_eq!(
            on.fill,
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)
        );
        let off = checkbox_paints(false, false, true);
        assert_eq!(
            off.outline,
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)
        );
    }

    #[test]
    fn a_halo_shows_only_while_hovered_or_pressed_and_presses_stronger() {
        let theme = MaterialTheme::light();
        assert_eq!(halo(&theme, ColorRole::Primary, false, false), Color::NONE);
        let hovered = halo(&theme, ColorRole::Primary, true, false).alpha();
        let pressed = halo(&theme, ColorRole::Primary, true, true).alpha();
        assert!(pressed > hovered && hovered > 0.0);
    }
}
