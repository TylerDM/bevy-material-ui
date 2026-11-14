//! Badges and avatars.
//!
//! A [`badge`] marks an icon: a small dot for "something new", or a count.
//! Spawn it as a child of the icon, or of the node around it, and it sits on
//! its top-right corner. An [`avatar`] is a person or a thing as a circle:
//! their initials or their picture.

use bevy::prelude::*;

use crate::paint::{Content, ContentColor, Fill, Paint, Typeset};
use crate::theme::{ColorRole, Corner, TypeRole};

/// The most a count badge shows; more is shown as this and a plus.
pub const BADGE_MAX: u32 = 999;

/// A small badge's side, and a large badge's height, in logical pixels.
pub const SMALL_BADGE: f32 = 6.0;
/// A large badge's height, in logical pixels.
pub const LARGE_BADGE: f32 = 16.0;

/// A badge: a dot, or a count.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum Badge {
    /// A small dot: something new, no number.
    Dot,
    /// A count, shown up to [`BADGE_MAX`].
    Count(u32),
}

/// What a count badge says for `count`: the number, or "999+" past
/// [`BADGE_MAX`].
pub fn badge_label(count: u32) -> String {
    if count > BADGE_MAX {
        format!("{BADGE_MAX}+")
    } else {
        count.to_string()
    }
}

/// The words of a count badge.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct BadgeLabel;

/// A badge for the top-right corner of the icon it is spawned in. Change
/// the [`Badge`] on it to change what it shows.
pub fn badge(badge: Badge) -> impl Bundle {
    (
        badge,
        badge_node(badge),
        Fill::new(ColorRole::Error),
        ContentColor(Paint::new(ColorRole::OnError)),
        Pickable::IGNORE,
        children![(
            BadgeLabel,
            Text::new(match badge {
                Badge::Dot => String::new(),
                Badge::Count(count) => badge_label(count),
            }),
            Typeset(TypeRole::LabelSmall),
            Content,
            Pickable::IGNORE,
        )],
    )
}

/// A badge's node: a dot in the corner, or a pill starting at the icon's
/// middle and rising above its top.
fn badge_node(badge: Badge) -> Node {
    match badge {
        Badge::Dot => Node {
            position_type: PositionType::Absolute,
            right: Val::ZERO,
            top: Val::ZERO,
            width: Val::Px(SMALL_BADGE),
            height: Val::Px(SMALL_BADGE),
            border_radius: Corner::Full.all(),
            ..default()
        },
        Badge::Count(_) => Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Px(-4.0),
            min_width: Val::Px(LARGE_BADGE),
            height: Val::Px(LARGE_BADGE),
            padding: UiRect::horizontal(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(LARGE_BADGE / 2.0)),
            ..default()
        },
    }
}

/// Redraws a badge whose count changed.
fn draw_badges(
    mut badges: Query<(&Badge, &mut Node, &Children), Changed<Badge>>,
    mut labels: Query<&mut Text, With<BadgeLabel>>,
) {
    for (badge, mut node, children) in &mut badges {
        *node = badge_node(*badge);
        let words = match badge {
            Badge::Dot => String::new(),
            Badge::Count(count) => badge_label(*count),
        };
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child)
                && text.0 != words
            {
                text.0.clone_from(&words);
            }
        }
    }
}

/// What an avatar shows.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum AvatarFace {
    /// Up to two letters.
    Initials(String),
    /// A picture, cropped to the circle.
    Image(Handle<Image>),
}

/// An avatar's side, in logical pixels: Material's usual.
pub const AVATAR: f32 = 40.0;

/// Up to two initials of `name`: the first letters of its first and last
/// words, upper case - "Ada Lovelace" is "AL", "Plato" is "P".
pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let first = |word: &&str| word.chars().next();
    let letters = match words.as_slice() {
        [] => Vec::new(),
        [only] => first(only).into_iter().collect(),
        [head, .., tail] => first(head).into_iter().chain(first(tail)).collect(),
    };
    letters.into_iter().flat_map(char::to_uppercase).collect()
}

/// Marks an avatar.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Avatar;

/// An avatar `side` logical pixels across showing `face`: initials on the
/// primary container, or a picture.
pub fn avatar(face: AvatarFace, side: f32) -> impl Bundle {
    (
        Avatar,
        Node {
            width: Val::Px(side),
            height: Val::Px(side),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            border_radius: Corner::Full.all(),
            flex_shrink: 0.0,
            ..default()
        },
        Fill::new(ColorRole::PrimaryContainer),
        ContentColor(Paint::new(ColorRole::OnPrimaryContainer)),
        Children::spawn(SpawnWith(move |inside: &mut ChildSpawner| match face {
            AvatarFace::Initials(letters) => {
                inside.spawn((
                    Text::new(letters),
                    Typeset(TypeRole::TitleMedium),
                    Content,
                    Pickable::IGNORE,
                ));
            }
            AvatarFace::Image(image) => {
                inside.spawn((
                    ImageNode::new(image),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
        })),
    )
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_badges);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_badge_shows_its_number_up_to_999_and_then_999_plus() {
        assert_eq!(badge_label(0), "0");
        assert_eq!(badge_label(7), "7");
        assert_eq!(badge_label(999), "999");
        assert_eq!(badge_label(1000), "999+");
        assert_eq!(badge_label(u32::MAX), "999+");
    }

    #[test]
    fn initials_are_the_first_letters_of_the_first_and_last_words() {
        assert_eq!(initials("Ada Lovelace"), "AL");
        assert_eq!(initials("grace brewster murray hopper"), "GH");
        assert_eq!(initials("Plato"), "P");
        assert_eq!(initials("  "), "");
        assert_eq!(initials("élodie durand"), "ÉD");
    }
}
