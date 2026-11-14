//! Material's focus indicator: a ring around the control that holds the
//! keyboard's focus, shown only while the keyboard is being used to move it.

use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::prelude::*;

use crate::theme::{ColorRole, MaterialTheme};

/// Draws Material's focus ring around this part while it, or a part inside
/// it, holds the keyboard's focus and the focus is being shown.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct FocusRing;

/// The focus ring's width and its gap from the part, in logical pixels.
const RING: f32 = 3.0;
const GAP: f32 = 2.0;

/// Shows the focus ring on the focused part, or the nearest part with a
/// [`FocusRing`] that holds it, and hides every other.
pub(crate) fn draw_focus_rings(
    focus: Option<Res<InputFocus>>,
    visible: Option<Res<InputFocusVisible>>,
    theme: Res<MaterialTheme>,
    rings: Query<Entity, With<FocusRing>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let changed = focus.as_ref().is_some_and(DetectChanges::is_changed)
        || visible.as_ref().is_some_and(DetectChanges::is_changed)
        || theme.is_changed();
    if !changed {
        return;
    }
    let shown = visible.is_some_and(|visible| visible.0);
    let ringed = focus
        .and_then(|focus| focus.get())
        .filter(|_| shown)
        .and_then(|focused| {
            std::iter::once(focused)
                .chain(parents.iter_ancestors(focused))
                .find(|entity| rings.contains(*entity))
        });
    for ring in &rings {
        let color = if Some(ring) == ringed {
            theme.color(ColorRole::Secondary)
        } else {
            Color::NONE
        };
        commands
            .entity(ring)
            .try_insert(Outline::new(Val::Px(RING), Val::Px(GAP), color));
    }
}
