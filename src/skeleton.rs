//! Skeletons: placeholders the shape of what is loading, pulsing gently
//! until the game replaces them.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::paint::{Fill, Paint};
use crate::theme::{ColorRole, Corner};

/// How long one pulse of a skeleton takes, in seconds.
pub const PULSE_SECONDS: f32 = 1.6;

/// A placeholder, and how long it has been pulsing.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Skeleton {
    /// Seconds it has been pulsing.
    pub pulsed: f32,
}

/// How opaque a skeleton is `seconds` into pulsing: between 0.5 and 1.
pub fn pulse(seconds: f32) -> f32 {
    0.75 + 0.25 * (seconds / PULSE_SECONDS * TAU).cos()
}

/// A placeholder `width` by `height` with `corner`s, on the highest
/// container.
pub fn skeleton(width: Val, height: Val, corner: Corner) -> impl Bundle {
    (
        Skeleton::default(),
        Node {
            width,
            height,
            border_radius: corner.all(),
            flex_shrink: 0.0,
            ..default()
        },
        Fill::new(ColorRole::SurfaceContainerHighest),
        Pickable::IGNORE,
    )
}

/// Pulses every skeleton.
fn pulse_skeletons(
    time: Res<Time>,
    mut skeletons: Query<(Entity, &mut Skeleton)>,
    mut commands: Commands,
) {
    for (entity, mut skeleton) in &mut skeletons {
        skeleton.pulsed += time.delta_secs();
        commands.entity(entity).try_insert(Fill(Paint::faded(
            ColorRole::SurfaceContainerHighest,
            pulse(skeleton.pulsed),
        )));
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, pulse_skeletons);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skeleton_pulses_between_half_and_fully_opaque() {
        assert!((pulse(0.0) - 1.0).abs() < 1e-5);
        assert!((pulse(PULSE_SECONDS / 2.0) - 0.5).abs() < 1e-5);
        for step in 0..40 {
            let opacity = pulse(step as f32 * 0.1);
            assert!((0.5 - 1e-5..=1.0 + 1e-5).contains(&opacity), "{opacity}");
        }
    }

    #[test]
    fn a_skeleton_a_game_despawns_the_frame_it_pulses_is_never_commanded_after() {
        // A game replacing its screen in `Update` may queue the despawn just
        // before the pulse queues its paint; Bevy panics on a command to an
        // entity that is gone. The order is pinned here; in a game it is
        // whatever the scheduler picks.
        let mut app = crate::testing::headless_app(200, 200, 1.0);
        let placeholder = crate::testing::spawn(
            &mut app,
            skeleton(Val::Px(80.0), Val::Px(16.0), crate::theme::Corner::Small),
        );
        app.add_systems(
            Update,
            (move |mut commands: Commands| {
                commands.entity(placeholder).despawn();
            })
            .before_ignore_deferred(pulse_skeletons),
        );
        app.update();
        assert!(app.world().get_entity(placeholder).is_err());
    }
}
