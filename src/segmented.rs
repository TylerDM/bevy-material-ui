//! Segmented buttons: a row of outlined segments, one of which (or, for a
//! multi-select row, any of which) is chosen - filled with the secondary
//! container and ticked.
//!
//! Each segment is Bevy's headless [`Button`], activated by a click or by
//! Enter or Space while focused. When one is, it sends [`ValueChange<bool>`]
//! with whether it is now chosen, and the row sends [`ValueChange<usize>`]
//! with the segment's index. A self-updating row (the default) moves
//! [`Checked`] itself; a controlled one waits for the game to. A single-select
//! row keeps one segment chosen: activating the chosen one changes nothing.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::{Activate, Button, ValueChange};

use crate::button::enabled;
use crate::checkbox::{CheckedControl, SelfUpdating, Starting};
use crate::focus::FocusRing;
use crate::icon::{Glyph, icon};
use crate::paint::{Look, Paint, Stroke, label};
use crate::theme::{ColorRole, TypeRole, state};

/// A row of segmented buttons: whether more than one may be chosen.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SegmentedButtons {
    /// Any number of segments may be chosen, rather than exactly one.
    pub multi: bool,
}

/// A segment of a [`SegmentedButtons`] row, with its index from 0.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct Segment(pub usize);

/// The tick before a chosen segment's label.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SegmentCheck;

/// What a segment was last drawn as.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SegmentStyle {
    #[reflect(ignore)]
    drawn: Option<(bool, bool)>,
}

/// The height of a segmented button, in logical pixels.
pub const SEGMENT_HEIGHT: f32 = 40.0;

/// A row of segmented buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialSegmentedButtons {
    /// Each segment's label, in order.
    pub labels: Vec<String>,
    /// The segments chosen to start with.
    pub selected: Vec<usize>,
    /// Whether any number may be chosen.
    pub multi: bool,
    /// Whether it moves [`Checked`] itself.
    pub self_updating: bool,
    /// Segments that can't be used.
    pub disabled: Vec<usize>,
}

/// A single-select row of segments, one to each of `labels`, with
/// `selected` chosen.
pub fn segmented_buttons<S: Into<String>>(
    labels: impl IntoIterator<Item = S>,
    selected: Option<usize>,
) -> MaterialSegmentedButtons {
    MaterialSegmentedButtons {
        labels: labels.into_iter().map(Into::into).collect(),
        selected: selected.into_iter().collect(),
        multi: false,
        self_updating: true,
        disabled: Vec::new(),
    }
}

impl MaterialSegmentedButtons {
    /// Any number of segments may be chosen; `selected` start chosen.
    pub fn multi(mut self, selected: impl IntoIterator<Item = usize>) -> Self {
        self.multi = true;
        self.selected = selected.into_iter().collect();
        self
    }

    /// Waiting for the game to move [`Checked`].
    pub fn controlled(mut self) -> Self {
        self.self_updating = false;
        self
    }

    /// With the segment at `index` unusable.
    pub fn disable(mut self, index: usize) -> Self {
        self.disabled.push(index);
        self
    }

    /// Its components.
    pub fn bundle(self) -> impl Bundle {
        let Self {
            labels,
            selected,
            multi,
            self_updating,
            disabled,
        } = self;
        (
            SegmentedButtons { multi },
            Starting {
                checked: false,
                self_updating,
            },
            Node {
                height: Val::Px(SEGMENT_HEIGHT),
                flex_shrink: 0.0,
                ..default()
            },
            Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
                let count = labels.len();
                for (index, words) in labels.into_iter().enumerate() {
                    row.spawn(segment(
                        index,
                        count,
                        words,
                        selected.contains(&index),
                        !disabled.contains(&index),
                    ));
                }
            })),
        )
    }
}

/// The segment at `index` of `count`.
fn segment(index: usize, count: usize, words: String, chosen: bool, usable: bool) -> impl Bundle {
    let end = Val::Px(SEGMENT_HEIGHT / 2.0);
    let mut corners = BorderRadius::ZERO;
    if index == 0 {
        corners.top_left = end.into();
        corners.bottom_left = end.into();
    }
    if index + 1 == count {
        corners.top_right = end.into();
        corners.bottom_right = end.into();
    }
    (
        (
            Button,
            Segment(index),
            SegmentStyle::default(),
            segment_look(chosen),
            FocusRing,
            TabIndex(0),
            Stroke::new(ColorRole::Outline),
            enabled(usable),
            Starting {
                checked: chosen,
                self_updating: false,
            },
        ),
        Node {
            min_width: Val::Px(48.0),
            height: Val::Percent(100.0),
            padding: UiRect::horizontal(Val::Px(12.0)),
            column_gap: Val::Px(8.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 1.0,
            border: UiRect::all(Val::Px(1.0)),
            // Each border drawn once: a segment overlaps the one before it.
            margin: UiRect::left(Val::Px(if index == 0 { 0.0 } else { -1.0 })),
            border_radius: corners,
            ..default()
        },
        children![
            (
                SegmentCheck,
                Node {
                    display: if chosen { Display::Flex } else { Display::None },
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
                children![icon(Glyph::Check, 18.0)],
            ),
            label(words, TypeRole::LabelLarge),
        ],
    )
}

/// A segment's look, chosen or not.
pub fn segment_look(chosen: bool) -> Look {
    if chosen {
        Look::filled(
            ColorRole::SecondaryContainer,
            ColorRole::OnSecondaryContainer,
        )
    } else {
        Look::bare(ColorRole::OnSurface)
    }
}

/// The segments a row has chosen once the segment at `index` is
/// activated: a multi-select row flips it, a single-select row moves to it.
pub fn chosen_after(chosen: &[bool], index: usize, multi: bool) -> Vec<bool> {
    chosen
        .iter()
        .enumerate()
        .map(|(at, &was)| match (multi, at == index) {
            (true, true) => !was,
            (true, false) => was,
            (false, here) => here,
        })
        .collect()
}

/// An activated segment changes what its row has chosen, and both say so.
pub(crate) fn activate_segment(
    activate: On<Activate>,
    segments: Query<(&Segment, &ChildOf, Has<Checked>)>,
    rows: Query<(&SegmentedButtons, &Children, Has<SelfUpdating>)>,
    chosen: Query<(Entity, &Segment, Has<Checked>)>,
    mut commands: Commands,
) {
    let Ok((Segment(index), parent, was)) = segments.get(activate.entity) else {
        return;
    };
    let Ok((row, children, self_updating)) = rows.get(parent.parent()) else {
        return;
    };
    if !row.multi && was {
        return;
    }
    let mut parts: Vec<(Entity, usize, bool)> = children
        .iter()
        .filter_map(|child| chosen.get(child).ok())
        .map(|(entity, Segment(at), checked)| (entity, *at, checked))
        .collect();
    parts.sort_by_key(|(_, at, _)| *at);
    let before: Vec<bool> = parts.iter().map(|(_, _, checked)| *checked).collect();
    let position = parts.iter().position(|(_, at, _)| at == index).unwrap_or(0);
    let after = chosen_after(&before, position, row.multi);
    commands.trigger(ValueChange {
        source: activate.entity,
        value: after[position],
        is_final: true,
    });
    commands.trigger(ValueChange {
        source: parent.parent(),
        value: *index,
        is_final: true,
    });
    if self_updating {
        for ((entity, _, _), now) in parts.iter().zip(after) {
            if now {
                commands.entity(*entity).insert(Checked);
            } else {
                commands.entity(*entity).remove::<Checked>();
            }
        }
    }
}

/// Draws each segment as it is chosen or disabled.
pub(crate) fn draw_segments(
    mut segments: Query<CheckedControl<SegmentStyle>>,
    children: Query<&Children>,
    mut icons: Query<&mut Node, With<SegmentCheck>>,
    mut commands: Commands,
) {
    for (entity, mut style, mut look, checked, disabled) in &mut segments {
        let now = (checked, disabled);
        if style.drawn == Some(now) {
            continue;
        }
        style.drawn = Some(now);
        let wanted = segment_look(checked);
        if *look != wanted {
            *look = wanted;
        }
        commands.entity(entity).insert(Stroke(if disabled {
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTAINER)
        } else {
            Paint::new(ColorRole::Outline)
        }));
        for part in children.iter_descendants(entity) {
            if let Ok(mut node) = icons.get_mut(part) {
                node.display = if checked {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(activate_segment).add_systems(
        PostUpdate,
        draw_segments.before(crate::paint::paint_controls),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_select_row_moves_its_choice_and_a_multi_select_row_flips_one() {
        let chosen = [true, false, false];
        assert_eq!(chosen_after(&chosen, 2, false), [false, false, true]);
        assert_eq!(chosen_after(&chosen, 2, true), [true, false, true]);
        assert_eq!(chosen_after(&chosen, 0, true), [false, false, false]);
    }

    #[test]
    fn a_chosen_segment_is_filled_with_the_secondary_container() {
        assert_eq!(
            segment_look(true).container,
            Some(ColorRole::SecondaryContainer)
        );
        assert_eq!(segment_look(false).container, None);
    }
}
