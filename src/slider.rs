//! Sliders: Bevy's headless [`Slider`] drawn as Material's - a track
//! filled with primary up to a round handle - continuous, discrete with a
//! stop at each step, or notched, stopping only at labelled notches.
//!
//! A slider is dragged, clicked on its track (it jumps there), or moved with
//! the arrow keys, Home and End while focused, sending
//! [`ValueChange<f32>`]: many while it is dragged, and a final one
//! (`is_final`) when it is let go. A self-updating slider (the default)
//! moves itself to each value, on a step; keep only a final one as what the
//! game keeps, since keeping each moves the game under the pointer. A
//! controlled slider waits for the game to insert [`SliderValue`].
//!
//! A notched slider's value is a notch's index; [`set_notch`] shows a notch
//! the game chose by other means, or none, its handle hidden.

use std::ops::RangeInclusive;

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::events::PointerRelease;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::{
    Slider, SliderDragState, SliderOrientation, SliderRange, SliderStep, SliderThumb, SliderValue,
    TrackClick, ValueChange,
};

use crate::button::enabled;
use crate::checkbox::{SelfUpdating, Starting, halo};
use crate::focus::FocusRing;
use crate::paint::{Fill, Ink, Paint};
use crate::theme::{ColorRole, Corner, MaterialTheme, TypeRole, state};

/// The height of a slider's row, in logical pixels: its target.
pub const SLIDER_HEIGHT: f32 = 44.0;
/// The height of a slider's track, in logical pixels.
pub const SLIDER_TRACK: f32 = 4.0;
/// The side of a slider's handle, in logical pixels.
pub const HANDLE: f32 = 20.0;
/// The side of a slider's halo: the circle its state layer is drawn in.
const HALO: f32 = 40.0;
/// The side of a stop on a discrete slider's track, in logical pixels.
const STOP: f32 = 2.0;
/// The height of a notched slider's labels' line, in logical pixels.
const LABELS: f32 = 16.0;
/// The most stops a discrete slider draws: past this they would touch.
const MOST_STOPS: usize = 101;

/// A Material slider, and what it was last drawn as.
#[derive(Component, Clone, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct MaterialSliderStyle {
    /// Whether it shows a stop at each step.
    pub stops: bool,
    #[reflect(ignore)]
    drawn: Option<(u32, Option<usize>, bool, bool, bool)>,
}

/// A notched slider: which of its notches is chosen, and how many it has.
#[derive(Component, Clone, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Notches {
    /// The notch chosen; none while what it sets matches none of them.
    chosen: Option<usize>,
    /// How many notches it has.
    count: usize,
}

impl Notches {
    /// The notch chosen, if one is.
    pub fn chosen(&self) -> Option<usize> {
        self.chosen
    }

    /// How many notches it has.
    pub fn count(&self) -> usize {
        self.count
    }
}

/// The part of a slider's track it is filled to.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SliderFill;

/// A slider's track.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SliderTrack;

/// The circle around a slider's handle its state layer is drawn in.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SliderHalo;

/// A stop on a discrete or notched slider's track, at a share of its length
/// from 0 to 1.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SliderStop(pub f32);

/// A notch's label under a notched slider, with the notch's index.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct NotchLabel(pub usize);

/// A slider: its value, range and step, its stops or notches, enabled or
/// not.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialSlider {
    /// Where it starts.
    pub value: f32,
    /// What it runs between.
    pub range: RangeInclusive<f32>,
    /// What it moves by, by key and when it is dragged; 0 for no steps, when
    /// a key moves it a hundredth of its range ([`KEY_SHARE`]).
    pub step: f32,
    /// Whether it shows a stop at each step.
    pub stops: bool,
    /// A notched slider's labels, one to each notch.
    pub notches: Option<Vec<String>>,
    /// A notched slider's notch chosen to start with.
    pub chosen: Option<usize>,
    /// Whether it can be used.
    pub enabled: bool,
    /// Whether it moves itself to each value.
    pub self_updating: bool,
}

/// A continuous slider at `value` within `range`.
pub fn slider(value: f32, range: RangeInclusive<f32>) -> MaterialSlider {
    MaterialSlider {
        value,
        range,
        step: 0.0,
        stops: false,
        notches: None,
        chosen: None,
        enabled: true,
        self_updating: true,
    }
}

/// A slider that stops only at its notches, one to each of `labels`, each
/// label under its notch: `chosen` is the one it is at, or none, its handle
/// hidden, while what it sets matches none of them. Its value is a notch's
/// index.
pub fn notched_slider<S: Into<String>>(
    labels: impl IntoIterator<Item = S>,
    chosen: Option<usize>,
) -> MaterialSlider {
    let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
    let last = labels.len().saturating_sub(1).max(1);
    let chosen = chosen.filter(|&notch| notch < labels.len());
    MaterialSlider {
        value: chosen.unwrap_or(0) as f32,
        range: 0.0..=last as f32,
        step: 1.0,
        stops: true,
        notches: Some(labels),
        chosen,
        enabled: true,
        self_updating: true,
    }
}

impl MaterialSlider {
    /// Moving by `step`, by key and when dragged.
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Discrete: moving by `step`, with a stop drawn at each.
    pub fn discrete(mut self, step: f32) -> Self {
        self.step = step;
        self.stops = true;
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Waiting for the game to insert [`SliderValue`].
    pub fn controlled(mut self) -> Self {
        self.self_updating = false;
        self
    }

    /// Its components: spawn it where it should run across its parent's width.
    pub fn bundle(self) -> impl Bundle {
        let range = SliderRange::from_range(self.range.clone());
        let value = range.clamp(self.value);
        let fraction = range.thumb_position(value);
        let stops = stop_shares(&self);
        let labels = self.notches.clone();
        let notched = labels.is_some();
        let chosen = self.chosen;
        let height = SLIDER_HEIGHT + if notched { LABELS } else { 0.0 };
        let notches = labels.as_ref().map(|labels| Notches {
            chosen,
            count: labels.len(),
        });
        let hidden = notched && chosen.is_none();
        (
            (
                Slider {
                    track_click: TrackClick::Snap,
                    orientation: SliderOrientation::Horizontal,
                },
                SliderValue(value),
                range,
                SliderStep(key_step(self.step, &self.range)),
                MaterialSliderStyle {
                    stops: self.stops,
                    drawn: None,
                },
                Hovered::default(),
                FocusRing,
                TabIndex(0),
                enabled(self.enabled),
                Starting {
                    checked: false,
                    self_updating: self.self_updating,
                },
            ),
            Node {
                width: Val::Percent(100.0),
                min_width: Val::Px(80.0),
                height: Val::Px(height),
                flex_shrink: 1.0,
                border_radius: Corner::ExtraSmall.all(),
                ..default()
            },
            NotchesIf(notches),
            Children::spawn(SpawnWith(move |slider: &mut ChildSpawner| {
                let top = (SLIDER_HEIGHT - SLIDER_TRACK) / 2.0;
                slider
                    .spawn((
                        SliderTrack,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(HANDLE / 2.0),
                            right: Val::Px(HANDLE / 2.0),
                            top: Val::Px(top),
                            height: Val::Px(SLIDER_TRACK),
                            border_radius: Corner::Full.all(),
                            ..default()
                        },
                        Fill::new(ColorRole::SecondaryContainer),
                        Pickable::IGNORE,
                    ))
                    .with_children(|track| {
                        track.spawn((
                            SliderFill,
                            Node {
                                width: Val::Percent(fraction * 100.0),
                                height: Val::Percent(100.0),
                                border_radius: Corner::Full.all(),
                                ..default()
                            },
                            Fill::new(ColorRole::Primary),
                            Pickable::IGNORE,
                        ));
                        for share in stops {
                            track.spawn((
                                SliderStop(share),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Percent(share * 100.0),
                                    margin: UiRect::left(Val::Px(-STOP / 2.0)),
                                    top: Val::Px((SLIDER_TRACK - STOP) / 2.0),
                                    width: Val::Px(STOP),
                                    height: Val::Px(STOP),
                                    border_radius: Corner::Full.all(),
                                    ..default()
                                },
                                Fill::new(ColorRole::OnSecondaryContainer),
                                Pickable::IGNORE,
                            ));
                        }
                    });
                // The handle's travel: the row less the handle's width, so its
                // left edge goes from one end to the other.
                slider
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(HANDLE),
                            top: Val::Px((SLIDER_HEIGHT - HANDLE) / 2.0),
                            height: Val::Px(HANDLE),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|travel| {
                        travel.spawn((
                            SliderHalo,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(fraction * 100.0),
                                margin: UiRect::left(Val::Px((HANDLE - HALO) / 2.0)),
                                top: Val::Px((HANDLE - HALO) / 2.0),
                                width: Val::Px(HALO),
                                height: Val::Px(HALO),
                                border_radius: Corner::Full.all(),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            if hidden {
                                Visibility::Hidden
                            } else {
                                Visibility::Inherited
                            },
                            Pickable::IGNORE,
                        ));
                        travel.spawn((
                            SliderThumb,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(fraction * 100.0),
                                width: Val::Px(HANDLE),
                                height: Val::Px(HANDLE),
                                border_radius: Corner::Full.all(),
                                ..default()
                            },
                            Fill::new(ColorRole::Primary),
                            if hidden {
                                Visibility::Hidden
                            } else {
                                Visibility::Inherited
                            },
                            Pickable::IGNORE,
                        ));
                    });
                let Some(labels) = labels else {
                    return;
                };
                // The labels, each under its notch along the handle's travel:
                // the first from its notch on, the last up to its, so neither
                // leaves the slider's room, and the rest centred on theirs.
                let count = labels.len();
                let share = |notch: usize| notch as f32 / count.saturating_sub(1).max(1) as f32;
                let gap = share(1) * 100.0;
                slider
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(HANDLE / 2.0),
                            right: Val::Px(HANDLE / 2.0),
                            top: Val::Px(SLIDER_HEIGHT - 6.0),
                            height: Val::Px(LABELS),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|axis| {
                        for (notch, words) in labels.into_iter().enumerate() {
                            let (left, width, justify) = if notch == 0 {
                                (0.0, gap / 2.0, JustifyContent::FlexStart)
                            } else if notch == count - 1 {
                                (100.0 - gap / 2.0, gap / 2.0, JustifyContent::FlexEnd)
                            } else {
                                (
                                    share(notch) * 100.0 - gap / 2.0,
                                    gap,
                                    JustifyContent::Center,
                                )
                            };
                            let role = label_role(Some(notch) == chosen, false);
                            axis.spawn((
                                NotchLabel(notch),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Percent(left),
                                    width: Val::Percent(width),
                                    height: Val::Percent(100.0),
                                    justify_content: justify,
                                    align_items: AlignItems::Center,
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                Pickable::IGNORE,
                                children![(
                                    Text::new(words),
                                    crate::paint::Typeset(TypeRole::LabelSmall),
                                    Ink(role),
                                    TextLayout::no_wrap(),
                                    Pickable::IGNORE,
                                )],
                            ));
                        }
                    });
            })),
        )
    }
}

/// A [`Notches`] on a notched slider, and nothing on another: inserted as it
/// is spawned.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
struct NotchesIf(Option<Notches>);

fn on_notches_if(add: On<Add<NotchesIf>>, found: Query<&NotchesIf>, mut commands: Commands) {
    let Ok(NotchesIf(notches)) = found.get(add.entity) else {
        return;
    };
    let notches = notches.clone();
    let mut slider = commands.entity(add.entity);
    slider.remove::<NotchesIf>();
    if let Some(notches) = notches {
        slider.insert(notches);
    }
}

/// Where a slider's stops are along its track, as shares of its length.
fn stop_shares(slider: &MaterialSlider) -> Vec<f32> {
    if let Some(labels) = &slider.notches {
        let last = labels.len().saturating_sub(1).max(1) as f32;
        return (0..labels.len()).map(|notch| notch as f32 / last).collect();
    }
    let (start, end) = (*slider.range.start(), *slider.range.end());
    let span = end - start;
    if !slider.stops || slider.step <= 0.0 || span <= 0.0 {
        return Vec::new();
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a count of steps, capped"
    )]
    let steps = (span / slider.step).round().max(1.0) as usize;
    if steps + 1 > MOST_STOPS {
        return Vec::new();
    }
    (0..=steps)
        .map(|index| (index as f32 * slider.step / span).min(1.0))
        .collect()
}

/// How much of its range a key moves a slider with no steps of its own.
pub const KEY_SHARE: f32 = 0.01;

/// What a key moves a slider by: its `step`, or for one with none (a
/// continuous slider) [`KEY_SHARE`] of its range, so the arrow keys always
/// move it.
pub fn key_step(step: f32, range: &RangeInclusive<f32>) -> f32 {
    if step > 0.0 {
        step
    } else {
        (range.end() - range.start()).abs() * KEY_SHARE
    }
}

/// `value` at the nearest step of `step` from the start of `range`.
pub fn on_a_step(value: f32, range: SliderRange, step: f32) -> f32 {
    if step <= 0.0 {
        return range.clamp(value);
    }
    let steps = ((value - range.start()) / step).round();
    range.clamp(range.start() + steps * step)
}

/// The notch nearest `value`, within `range`.
pub fn notch_at(value: f32, range: SliderRange) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a notch's index, from a handful"
    )]
    let notch = (range.clamp(value) - range.start()).round().max(0.0) as usize;
    notch
}

/// The role a notch's label is drawn in.
fn label_role(chosen: bool, disabled: bool) -> Paint {
    if disabled {
        Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)
    } else if chosen {
        Paint::new(ColorRole::Primary)
    } else {
        Paint::new(ColorRole::OnSurfaceVariant)
    }
}

/// What a slider following its value reads and moves.
type Following<'a> = (&'a SliderRange, &'a SliderStep, Option<&'a mut Notches>);

/// A self-updating slider's handle goes to where the pointer or a key has
/// it, to the nearest step; a notched one is at the notch it went to.
pub(crate) fn follow_value(
    change: On<ValueChange<f32>>,
    mut sliders: Query<Following, (With<MaterialSliderStyle>, With<SelfUpdating>)>,
    mut commands: Commands,
) {
    let Ok((range, step, notches)) = sliders.get_mut(change.source) else {
        return;
    };
    commands
        .entity(change.source)
        .insert(SliderValue(on_a_step(change.value, *range, step.0)));
    if let Some(mut notches) = notches {
        let notch = Some(notch_at(change.value, *range));
        if notches.chosen != notch {
            notches.chosen = notch;
        }
    }
}

/// A click on a track moves the slider with no drag to end: letting go of it
/// is as final as letting go of a drag, whose own end says so.
pub(crate) fn let_go(
    release: On<PointerRelease>,
    sliders: Query<
        (&SliderValue, &SliderDragState, Has<InteractionDisabled>),
        With<MaterialSliderStyle>,
    >,
    mut commands: Commands,
) {
    if let Ok((value, drag, disabled)) = sliders.get(release.entity)
        && !drag.dragging
        && !disabled
    {
        commands.trigger(ValueChange {
            source: release.entity,
            value: value.0,
            is_final: true,
        });
    }
}

/// Shows a notched slider at `chosen`, or at none, its handle hidden, where
/// it stands: what the game keeps changed by other means than the slider.
pub fn set_notch(mut slider: EntityCommands, chosen: Option<usize>) {
    slider.queue(move |mut slider: EntityWorldMut| {
        let count = slider.get::<Notches>().map_or(0, Notches::count);
        let chosen = chosen.filter(|&notch| notch < count);
        if let Some(mut notches) = slider.get_mut::<Notches>()
            && notches.chosen != chosen
        {
            notches.chosen = chosen;
        }
        if let Some(notch) = chosen {
            slider.insert(SliderValue(notch as f32));
        }
    });
}

/// A slider, its state and what it is drawn as.
type SliderQuery<'a> = (
    Entity,
    &'a mut MaterialSliderStyle,
    &'a SliderValue,
    &'a SliderRange,
    Option<&'a Notches>,
    &'a Hovered,
    Has<Pressed>,
    Has<InteractionDisabled>,
);

/// The parts of sliders that move with their value.
type MovingParts<'a> = (
    &'a mut Node,
    Has<SliderThumb>,
    Has<SliderHalo>,
    Has<SliderFill>,
    Has<SliderTrack>,
    Option<&'a mut BackgroundColor>,
);

/// Moves each slider's handle and fill to its value, colours its stops and
/// labels as they fall either side of it, and draws its state.
pub(crate) fn draw_sliders(
    theme: Res<MaterialTheme>,
    mut sliders: Query<SliderQuery>,
    children: Query<&Children>,
    mut parts: Query<MovingParts, Without<MaterialSliderStyle>>,
    stops: Query<&SliderStop>,
    labels: Query<(&NotchLabel, &Children)>,
    mut commands: Commands,
) {
    for (entity, mut style, value, range, notches, hovered, pressed, disabled) in &mut sliders {
        let chosen = notches.and_then(Notches::chosen);
        let hidden = notches.is_some() && chosen.is_none();
        let now = (value.0.to_bits(), chosen, hovered.0, pressed, disabled);
        if style.drawn == Some(now) && !theme.is_changed() {
            continue;
        }
        style.drawn = Some(now);
        let fraction = range.thumb_position(range.clamp(value.0));
        let percent = Val::Percent(fraction * 100.0);
        let (active, inactive, handle) = if disabled {
            (
                Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
                Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTAINER),
                Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
            )
        } else {
            (
                Paint::new(ColorRole::Primary),
                Paint::new(ColorRole::SecondaryContainer),
                Paint::new(ColorRole::Primary),
            )
        };
        for part in children.iter_descendants(entity) {
            if let Ok((mut node, thumb, halo_part, fill, track, background)) = parts.get_mut(part) {
                if thumb || halo_part {
                    node.left = percent;
                    commands.entity(part).insert(if hidden {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    });
                }
                if thumb {
                    commands.entity(part).insert(Fill(handle));
                } else if halo_part {
                    if let Some(mut background) = background {
                        background.0 = if disabled {
                            Color::NONE
                        } else {
                            halo(&theme, ColorRole::Primary, hovered.0, pressed)
                        };
                    }
                } else if fill {
                    node.width = percent;
                    commands.entity(part).insert(Fill(active));
                } else if track {
                    commands.entity(part).insert(Fill(inactive));
                }
            }
            if let Ok(SliderStop(share)) = stops.get(part) {
                let behind = *share <= fraction + 1e-4 && !hidden;
                let role = match (behind, disabled) {
                    (_, true) => Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
                    (true, false) => Paint::new(ColorRole::OnPrimary),
                    (false, false) => Paint::new(ColorRole::OnSecondaryContainer),
                };
                commands.entity(part).insert(Fill(role));
            }
            if let Ok((NotchLabel(notch), words)) = labels.get(part) {
                let paint = label_role(Some(*notch) == chosen, disabled);
                for text in words.iter() {
                    commands.entity(text).insert(Ink(paint));
                }
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_notches_if)
        .add_observer(follow_value)
        .add_observer(let_go)
        .add_systems(
            PostUpdate,
            draw_sliders.before(crate::paint::paint_controls),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_moves_a_slider_by_its_step_or_a_hundredth_of_its_range_without_one() {
        assert_eq!(key_step(5.0, &(0.0..=100.0)), 5.0);
        assert_eq!(key_step(0.0, &(0.0..=100.0)), 1.0);
        assert!((key_step(0.0, &(2.0..=4.0)) - 0.02).abs() < 1e-6);
    }

    #[test]
    fn a_value_goes_to_the_nearest_step_from_the_start_and_stays_in_range() {
        let range = SliderRange::new(60.0, 110.0);
        assert_eq!(on_a_step(72.4, range, 5.0), 70.0);
        assert_eq!(on_a_step(72.6, range, 5.0), 75.0);
        assert_eq!(on_a_step(200.0, range, 5.0), 110.0);
        assert_eq!(on_a_step(0.3, SliderRange::new(0.0, 1.0), 0.0), 0.3);
    }

    #[test]
    fn a_value_between_notches_is_at_the_nearest() {
        let range = SliderRange::new(0.0, 4.0);
        assert_eq!(notch_at(2.6, range), 3);
        assert_eq!(notch_at(-1.0, range), 0);
        assert_eq!(notch_at(9.0, range), 4);
    }

    #[test]
    fn a_discrete_slider_has_a_stop_at_each_step_and_a_notched_one_at_each_notch() {
        let stops = stop_shares(&slider(0.0, 0.0..=10.0).discrete(2.5));
        assert_eq!(stops, [0.0, 0.25, 0.5, 0.75, 1.0]);
        assert!(stop_shares(&slider(0.0, 0.0..=10.0).step(2.5)).is_empty());
        let notched = stop_shares(&notched_slider(["Low", "Medium", "High"], None));
        assert_eq!(notched, [0.0, 0.5, 1.0]);
        assert!(stop_shares(&slider(0.0, 0.0..=1000.0).discrete(1.0)).is_empty());
    }

    #[test]
    fn a_notched_slider_runs_over_its_notches_indices() {
        let notched = notched_slider(["A", "B", "C", "D", "E"], Some(9));
        assert_eq!(notched.range, 0.0..=4.0);
        assert_eq!(notched.chosen, None, "a notch past the last is none");
        assert_eq!(notched.step, 1.0);
    }
}
