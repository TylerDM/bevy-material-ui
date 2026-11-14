//! Progress indicators: linear and circular, determinate - how far along -
//! or indeterminate - busy, for how long unknown.
//!
//! The game sets how far along with the [`Progress`] component; the
//! indicator draws it, and animates itself while it is indeterminate.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::ui::{AngularColorStop, BorderGradient, ConicGradient, UiPosition};

use crate::paint::Fill;
use crate::theme::{ColorRole, Corner, MaterialTheme, emphasized};

/// How far along a task is: a share from 0 to 1, or none while unknown.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Progress(pub Option<f32>);

impl Progress {
    /// The share done, clamped to 0 to 1, or none while unknown.
    pub fn share(self) -> Option<f32> {
        self.0.map(|share| share.clamp(0.0, 1.0))
    }
}

/// How long an indeterminate indicator has been animating, in seconds.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ProgressClock(pub f32);

/// A linear indicator's track.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[require(Progress, ProgressClock)]
#[reflect(Component)]
pub struct LinearProgress;

/// The part of a linear indicator filled to how far along.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ProgressIndicator;

/// A circular indicator.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[require(Progress, ProgressClock)]
#[reflect(Component)]
pub struct CircularProgress;

/// A linear indicator's thickness, in logical pixels.
pub const LINEAR_THICKNESS: f32 = 4.0;
/// A circular indicator's side and its stroke, in logical pixels.
pub const CIRCULAR_SIDE: f32 = 48.0;
/// A circular indicator's stroke, in logical pixels.
pub const CIRCULAR_STROKE: f32 = 4.0;
/// How long one sweep of an indeterminate indicator takes, in seconds.
pub const SWEEP_SECONDS: f32 = 1.5;

/// A linear progress indicator across its parent, `share` along (0 to 1),
/// or indeterminate with none.
pub fn linear_progress(share: Option<f32>) -> impl Bundle {
    (
        LinearProgress,
        Progress(share),
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(LINEAR_THICKNESS),
            overflow: Overflow::clip(),
            border_radius: Corner::Full.all(),
            flex_shrink: 0.0,
            ..default()
        },
        Fill::new(ColorRole::SecondaryContainer),
        children![(
            ProgressIndicator,
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                bottom: Val::ZERO,
                left: Val::ZERO,
                width: Val::Percent(share.map_or(0.0, |share| share.clamp(0.0, 1.0) * 100.0)),
                border_radius: Corner::Full.all(),
                ..default()
            },
            Fill::new(ColorRole::Primary),
            Pickable::IGNORE,
        )],
    )
}

/// A circular progress indicator, `share` around (0 to 1), or an
/// indeterminate spinner with none.
pub fn circular_progress(share: Option<f32>) -> impl Bundle {
    (
        CircularProgress,
        Progress(share),
        Node {
            width: Val::Px(CIRCULAR_SIDE),
            height: Val::Px(CIRCULAR_SIDE),
            border: UiRect::all(Val::Px(CIRCULAR_STROKE)),
            border_radius: Corner::Full.all(),
            flex_shrink: 0.0,
            ..default()
        },
        BorderColor::all(Color::NONE),
        BorderGradient::default(),
    )
}

/// Where a linear indicator's bar lies along its track, as percentages of
/// it: its left edge and its width.
pub fn linear_span(progress: Progress, clock: f32) -> (f32, f32) {
    if let Some(share) = progress.share() {
        return (0.0, share * 100.0);
    }
    // A bar sweeping from off the left to off the right, quickening and
    // easing as Material's does.
    let phase = (clock / SWEEP_SECONDS).fract();
    let head = emphasized(phase) * 140.0 - 20.0;
    let tail = emphasized((phase - 0.25).max(0.0) / 0.75) * 140.0 - 40.0;
    let left = tail.max(0.0);
    (left, (head.min(100.0) - left).max(0.0))
}

/// Where a circular indicator's arc lies, in radians clockwise from the
/// top: its start and its length.
pub fn circular_arc(progress: Progress, clock: f32) -> (f32, f32) {
    if let Some(share) = progress.share() {
        return (0.0, share * TAU);
    }
    // An arc that grows and shrinks as the whole turns.
    let phase = (clock / SWEEP_SECONDS).fract();
    let length = TAU * (0.1 + 0.65 * (phase * TAU).sin().abs());
    let start = (clock / SWEEP_SECONDS * TAU * 1.25).rem_euclid(TAU);
    (start, length)
}

/// A conic gradient drawing an arc of `length` from `start` in `arc`, the
/// rest of the circle in `track`.
pub fn arc_gradient(start: f32, length: f32, arc: Color, track: Color) -> ConicGradient {
    let length = length.clamp(0.0, TAU);
    ConicGradient::new(
        UiPosition::CENTER,
        vec![
            AngularColorStop::new(arc, 0.0),
            AngularColorStop::new(arc, length),
            AngularColorStop::new(track, length),
            AngularColorStop::new(track, TAU),
        ],
    )
    .with_start(start)
}

/// Counts every indeterminate indicator's time.
fn tick(time: Res<Time>, mut clocks: Query<(&Progress, &mut ProgressClock)>) {
    for (progress, mut clock) in &mut clocks {
        if progress.0.is_none() {
            clock.0 += time.delta_secs();
        }
    }
}

/// Draws each linear indicator's bar where it lies.
fn draw_linear(
    tracks: Query<(&Progress, &ProgressClock, &Children), With<LinearProgress>>,
    mut bars: Query<&mut Node, With<ProgressIndicator>>,
) {
    for (progress, clock, children) in &tracks {
        let (left, width) = linear_span(*progress, clock.0);
        for child in children.iter() {
            if let Ok(mut bar) = bars.get_mut(child) {
                let left = Val::Percent(left);
                let width = Val::Percent(width);
                if bar.left != left || bar.width != width {
                    bar.left = left;
                    bar.width = width;
                }
            }
        }
    }
}

/// Draws each circular indicator's arc where it lies: on the track while
/// determinate, alone while spinning.
fn draw_circular(
    theme: Res<MaterialTheme>,
    mut circles: Query<(&Progress, &ProgressClock, &mut BorderGradient), With<CircularProgress>>,
) {
    let arc = theme.color(ColorRole::Primary);
    for (progress, clock, mut gradient) in &mut circles {
        let track = if progress.0.is_some() {
            theme.color(ColorRole::SecondaryContainer)
        } else {
            Color::NONE
        };
        let (start, length) = circular_arc(*progress, clock.0);
        let wanted = BorderGradient(vec![arc_gradient(start, length, arc, track).into()]);
        if *gradient != wanted {
            *gradient = wanted;
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (tick, draw_linear, draw_circular).chain());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_determinate_indicator_fills_to_its_share_and_never_past_its_ends() {
        assert_eq!(linear_span(Progress(Some(0.4)), 3.0), (0.0, 40.0));
        assert_eq!(linear_span(Progress(Some(1.7)), 0.0), (0.0, 100.0));
        assert_eq!(linear_span(Progress(Some(-1.0)), 0.0), (0.0, 0.0));
        assert_eq!(circular_arc(Progress(Some(0.25)), 0.0), (0.0, TAU / 4.0));
    }

    #[test]
    fn an_indeterminate_bar_moves_and_stays_on_its_track() {
        let spans: Vec<(f32, f32)> = (0..30)
            .map(|frame| linear_span(Progress(None), frame as f32 * 0.05))
            .collect();
        for &(left, width) in &spans {
            assert!(
                left >= 0.0 && left + width <= 100.0 + 1e-3,
                "{left} {width}"
            );
        }
        assert!(spans.windows(2).any(|pair| pair[0] != pair[1]), "it moves");
    }

    #[test]
    fn an_indeterminate_spinner_turns_and_its_arc_changes_length() {
        let (start_a, length_a) = circular_arc(Progress(None), 0.1);
        let (start_b, length_b) = circular_arc(Progress(None), 0.5);
        assert_ne!(start_a, start_b);
        assert_ne!(length_a, length_b);
        assert!(length_a > 0.0 && length_a < TAU);
    }

    #[test]
    fn an_arc_is_its_colour_to_its_length_and_the_track_after() {
        let gradient = arc_gradient(0.0, 1.0, Color::WHITE, Color::BLACK);
        let angles: Vec<Option<f32>> = gradient.stops.iter().map(|stop| stop.angle).collect();
        assert_eq!(angles, [Some(0.0), Some(1.0), Some(1.0), Some(TAU)]);
        assert_eq!(gradient.stops[1].color, Color::WHITE);
        assert_eq!(gradient.stops[2].color, Color::BLACK);
    }
}
