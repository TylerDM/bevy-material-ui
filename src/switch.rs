//! Switches: Material 3's switch, a 52 x 32 track whose handle grows and
//! slides across when it is turned on, drawn over Bevy's headless
//! [`Checkbox`] - so it is used and observed exactly as a checkbox is: a
//! click, or Enter or Space while focused, sends [`ValueChange<bool>`], and
//! a self-updating switch (the default) turns itself over.
//!
//! [`ValueChange<bool>`]: bevy::ui_widgets::ValueChange

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Checked, InteractionDisabled, Pressed};
use bevy::ui_widgets::Checkbox;

use crate::button::enabled;
use crate::checkbox::{Starting, halo, labelled};
use crate::focus::FocusRing;
use crate::icon::{Glyph, icon};
use crate::paint::{ContentColor, Fill, Paint, Stroke};
use crate::theme::{ColorRole, Corner, MaterialTheme, state};

/// A switch's track, in logical pixels.
pub const SWITCH_TRACK: Vec2 = Vec2::new(52.0, 32.0);
/// The side of a switch's halo: the circle its state layer is drawn in.
const HALO: f32 = 40.0;
/// The track's border, in logical pixels.
const TRACK_BORDER: f32 = 2.0;

/// A switch's own state.
#[derive(Component, Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SwitchStyle {
    /// Whether its handle shows a tick while it is on.
    pub icons: bool,
    #[reflect(ignore)]
    drawn: Option<SwitchState>,
}

/// What a switch is drawn as: on, hovered, pressed, disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SwitchState {
    /// Turned on.
    pub on: bool,
    /// Under the pointer.
    pub hovered: bool,
    /// Pressed.
    pub pressed: bool,
    /// Unusable.
    pub disabled: bool,
}

/// The circle around a switch's handle its state layer is drawn in.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SwitchHalo;

/// A switch's handle.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SwitchHandle;

/// A switch: on or off, with a tick in its handle or not, enabled or not.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialSwitch {
    /// Whether it starts on.
    pub on: bool,
    /// Whether its handle shows a tick while on.
    pub icons: bool,
    /// Whether it can be used.
    pub enabled: bool,
    /// Whether it turns itself over when used.
    pub self_updating: bool,
}

/// A switch, off, that turns itself over when used.
pub fn switch() -> MaterialSwitch {
    MaterialSwitch {
        on: false,
        icons: false,
        enabled: true,
        self_updating: true,
    }
}

impl MaterialSwitch {
    /// Starting on, or off.
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// Showing a tick in its handle while on.
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Waiting for the game to insert or remove [`Checked`].
    pub fn controlled(mut self) -> Self {
        self.self_updating = false;
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let state = SwitchState {
            on: self.on,
            hovered: false,
            pressed: false,
            disabled: !self.enabled,
        };
        let paints = switch_paints(state);
        let handle = handle_side(state);
        (
            (
                Checkbox,
                SwitchStyle {
                    icons: self.icons,
                    drawn: None,
                },
                Hovered::default(),
                FocusRing,
                TabIndex(0),
                enabled(self.enabled),
                Starting {
                    checked: self.on,
                    self_updating: self.self_updating,
                },
            ),
            Node {
                width: Val::Px(SWITCH_TRACK.x),
                height: Val::Px(SWITCH_TRACK.y),
                border: UiRect::all(Val::Px(TRACK_BORDER)),
                border_radius: Corner::Full.all(),
                flex_shrink: 0.0,
                ..default()
            },
            Fill(paints.track),
            Stroke(paints.border),
            children![(
                SwitchHalo,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(halo_left(self.on)),
                    top: Val::Px((SWITCH_TRACK.y - HALO) / 2.0 - TRACK_BORDER),
                    width: Val::Px(HALO),
                    height: Val::Px(HALO),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: Corner::Full.all(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
                children![(
                    SwitchHandle,
                    Node {
                        width: Val::Px(handle),
                        height: Val::Px(handle),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: Corner::Full.all(),
                        ..default()
                    },
                    Fill(paints.handle),
                    ContentColor(paints.icon),
                    Pickable::IGNORE,
                    children![(icon(Glyph::Check, 16.0), Visibility::Hidden)],
                )],
            )],
        )
    }

    /// The switch with `words` after it, in a row a click anywhere on which
    /// turns it over.
    pub fn labelled(self, words: impl Into<String>) -> impl Bundle {
        labelled(self.bundle(), words.into())
    }
}

/// Where a switch's halo stands from the track's inner left edge: the
/// handle's centre 16 px in from the end it is at.
pub fn halo_left(on: bool) -> f32 {
    let centre = if on {
        SWITCH_TRACK.y / 2.0 + SWITCH_TRACK.x - SWITCH_TRACK.y
    } else {
        SWITCH_TRACK.y / 2.0
    };
    centre - HALO / 2.0 - TRACK_BORDER
}

/// The side of a switch's handle in a state: 16 off, 24 on, 28 pressed.
pub fn handle_side(state: SwitchState) -> f32 {
    if state.pressed && !state.disabled {
        28.0
    } else if state.on {
        24.0
    } else {
        16.0
    }
}

/// What a switch's parts are drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchPaints {
    /// The track.
    pub track: Paint,
    /// The track's border.
    pub border: Paint,
    /// The handle.
    pub handle: Paint,
    /// The tick in the handle.
    pub icon: Paint,
    /// The role of the state layer around the handle.
    pub layer: ColorRole,
}

/// What a switch in `state` is drawn in, as Material 3's switch.
pub fn switch_paints(state: SwitchState) -> SwitchPaints {
    let SwitchState {
        on,
        hovered,
        pressed,
        disabled,
    } = state;
    match (on, disabled) {
        (true, false) => SwitchPaints {
            track: Paint::new(ColorRole::Primary),
            border: Paint::new(ColorRole::Primary),
            handle: Paint::new(if hovered || pressed {
                ColorRole::PrimaryContainer
            } else {
                ColorRole::OnPrimary
            }),
            icon: Paint::new(ColorRole::OnPrimaryContainer),
            layer: ColorRole::Primary,
        },
        (false, false) => SwitchPaints {
            track: Paint::new(ColorRole::SurfaceContainerHighest),
            border: Paint::new(ColorRole::Outline),
            handle: Paint::new(if hovered || pressed {
                ColorRole::OnSurfaceVariant
            } else {
                ColorRole::Outline
            }),
            icon: Paint::new(ColorRole::SurfaceContainerHighest),
            layer: ColorRole::OnSurface,
        },
        (true, true) => SwitchPaints {
            track: Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTAINER),
            border: Paint::faded(ColorRole::OnSurface, 0.0),
            handle: Paint::new(ColorRole::Surface),
            icon: Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
            layer: ColorRole::OnSurface,
        },
        (false, true) => SwitchPaints {
            track: Paint::faded(
                ColorRole::SurfaceContainerHighest,
                state::DISABLED_CONTAINER,
            ),
            border: Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTAINER),
            handle: Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
            icon: Paint::faded(ColorRole::SurfaceContainerHighest, state::DISABLED_CONTENT),
            layer: ColorRole::OnSurface,
        },
    }
}

/// A switch, its state and the parts it is drawn with.
type SwitchQuery<'a> = (
    Entity,
    &'a mut SwitchStyle,
    &'a Hovered,
    Has<Checked>,
    Has<Pressed>,
    Has<InteractionDisabled>,
);

/// What a switch's halo is moved and coloured through.
type HaloParts<'a> = (&'a mut Node, &'a mut BackgroundColor);

/// Draws each switch as it is turned, hovered, pressed or disabled.
pub(crate) fn draw_switches(
    theme: Res<MaterialTheme>,
    mut switches: Query<SwitchQuery>,
    children: Query<&Children>,
    mut halos: Query<HaloParts, (With<SwitchHalo>, Without<SwitchHandle>)>,
    mut handles: Query<&mut Node, (With<SwitchHandle>, Without<SwitchHalo>)>,
    marks: Query<(), With<crate::icon::IconRoot>>,
    mut commands: Commands,
) {
    for (entity, mut style, hovered, on, pressed, disabled) in &mut switches {
        let now = SwitchState {
            on,
            hovered: hovered.0,
            pressed,
            disabled,
        };
        if style.drawn == Some(now) && !theme.is_changed() {
            continue;
        }
        style.drawn = Some(now);
        let paints = switch_paints(now);
        commands
            .entity(entity)
            .try_insert((Fill(paints.track), Stroke(paints.border)));
        let icons = style.icons;
        for part in children.iter_descendants(entity) {
            if let Ok((mut node, mut background)) = halos.get_mut(part) {
                node.left = Val::Px(halo_left(on));
                let shown = !disabled;
                background.0 = if shown {
                    halo(&theme, paints.layer, hovered.0, pressed)
                } else {
                    Color::NONE
                };
            } else if let Ok(mut node) = handles.get_mut(part) {
                let side = Val::Px(handle_side(now));
                node.width = side;
                node.height = side;
                commands
                    .entity(part)
                    .try_insert((Fill(paints.handle), ContentColor(paints.icon)));
            } else if marks.contains(part) {
                commands.entity(part).try_insert(if icons && on {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        draw_switches.before(crate::paint::paint_controls),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(on: bool) -> SwitchState {
        SwitchState {
            on,
            hovered: false,
            pressed: false,
            disabled: false,
        }
    }

    #[test]
    fn a_switch_turned_on_fills_its_track_with_primary_and_its_handle_grows() {
        assert_eq!(
            switch_paints(at(true)).track,
            Paint::new(ColorRole::Primary)
        );
        assert_eq!(
            switch_paints(at(false)).track,
            Paint::new(ColorRole::SurfaceContainerHighest)
        );
        assert_eq!(
            switch_paints(at(false)).border,
            Paint::new(ColorRole::Outline)
        );
        assert_eq!(handle_side(at(false)), 16.0);
        assert_eq!(handle_side(at(true)), 24.0);
        let pressed = SwitchState {
            pressed: true,
            ..at(false)
        };
        assert_eq!(handle_side(pressed), 28.0);
    }

    #[test]
    fn a_switchs_handle_sits_sixteen_pixels_in_from_the_end_it_is_at() {
        let centre = |on: bool| halo_left(on) + HALO / 2.0 + TRACK_BORDER;
        assert_eq!(centre(false), 16.0);
        assert_eq!(centre(true), SWITCH_TRACK.x - 16.0);
    }

    #[test]
    fn a_disabled_switch_is_faded() {
        let off = switch_paints(SwitchState {
            disabled: true,
            ..at(false)
        });
        assert_eq!(off.handle.opacity, state::DISABLED_CONTENT);
        let on = switch_paints(SwitchState {
            disabled: true,
            ..at(true)
        });
        assert_eq!(on.track.opacity, state::DISABLED_CONTAINER);
    }
}
