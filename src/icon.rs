//! Icons: a game's own pictures, or a few glyphs the components need -
//! a check, a close, chevrons, arrows - drawn from rounded strokes, so the
//! crate needs no icon font and they are crisp at any size.
//!
//! Every icon is [`Content`]: inside a control it takes the control's content
//! colour; elsewhere, give its parent a [`crate::paint::ContentColor`].

use bevy::prelude::*;
use bevy::ui::UiTransform;

use crate::paint::Content;

/// A glyph drawn from strokes on Material's 24-unit icon grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum Glyph {
    /// A tick: chosen, done.
    Check,
    /// A short bar: a checkbox neither on nor off.
    Dash,
    /// A cross: close, clear, remove.
    Close,
    /// A plus: add.
    Plus,
    /// A chevron pointing left.
    ChevronLeft,
    /// A chevron pointing right: more inside.
    ChevronRight,
    /// A chevron pointing up.
    ChevronUp,
    /// A chevron pointing down: a menu opens.
    ChevronDown,
    /// An arrow pointing left: back.
    ArrowBack,
    /// An arrow pointing right: forward.
    ArrowForward,
    /// Three lines: a menu, a navigation drawer.
    Menu,
    /// Three dots, one above the other: more actions.
    MoreVert,
    /// A round dot.
    Dot,
}

impl Glyph {
    /// Every glyph.
    pub const ALL: [Glyph; 13] = [
        Glyph::Check,
        Glyph::Dash,
        Glyph::Close,
        Glyph::Plus,
        Glyph::ChevronLeft,
        Glyph::ChevronRight,
        Glyph::ChevronUp,
        Glyph::ChevronDown,
        Glyph::ArrowBack,
        Glyph::ArrowForward,
        Glyph::Menu,
        Glyph::MoreVert,
        Glyph::Dot,
    ];

    /// Its strokes, each from one point to another on the 24-unit grid, and
    /// the strokes' width.
    pub fn strokes(self) -> (Vec<(Vec2, Vec2)>, f32) {
        const fn p(x: f32, y: f32) -> Vec2 {
            Vec2::new(x, y)
        }
        match self {
            Glyph::Check => (vec![(p(5.0, 12.5), p(9.5, 17.0)), (p(9.5, 17.0), p(19.0, 7.0))], 2.4),
            Glyph::Dash => (vec![(p(6.0, 12.0), p(18.0, 12.0))], 2.4),
            Glyph::Close => (vec![(p(6.0, 6.0), p(18.0, 18.0)), (p(18.0, 6.0), p(6.0, 18.0))], 2.0),
            Glyph::Plus => (vec![(p(12.0, 5.0), p(12.0, 19.0)), (p(5.0, 12.0), p(19.0, 12.0))], 2.0),
            Glyph::ChevronLeft => (vec![(p(15.0, 6.0), p(9.0, 12.0)), (p(9.0, 12.0), p(15.0, 18.0))], 2.0),
            Glyph::ChevronRight => {
                (vec![(p(9.0, 6.0), p(15.0, 12.0)), (p(15.0, 12.0), p(9.0, 18.0))], 2.0)
            }
            Glyph::ChevronUp => (vec![(p(6.0, 15.0), p(12.0, 9.0)), (p(12.0, 9.0), p(18.0, 15.0))], 2.0),
            Glyph::ChevronDown => (vec![(p(6.0, 9.0), p(12.0, 15.0)), (p(12.0, 15.0), p(18.0, 9.0))], 2.0),
            Glyph::ArrowBack => (
                vec![
                    (p(5.0, 12.0), p(19.0, 12.0)),
                    (p(5.0, 12.0), p(11.0, 6.0)),
                    (p(5.0, 12.0), p(11.0, 18.0)),
                ],
                2.0,
            ),
            Glyph::ArrowForward => (
                vec![
                    (p(5.0, 12.0), p(19.0, 12.0)),
                    (p(19.0, 12.0), p(13.0, 6.0)),
                    (p(19.0, 12.0), p(13.0, 18.0)),
                ],
                2.0,
            ),
            Glyph::Menu => (
                vec![
                    (p(4.0, 7.0), p(20.0, 7.0)),
                    (p(4.0, 12.0), p(20.0, 12.0)),
                    (p(4.0, 17.0), p(20.0, 17.0)),
                ],
                2.0,
            ),
            Glyph::MoreVert => (
                vec![
                    (p(12.0, 6.0), p(12.0, 6.0)),
                    (p(12.0, 12.0), p(12.0, 12.0)),
                    (p(12.0, 18.0), p(12.0, 18.0)),
                ],
                3.0,
            ),
            Glyph::Dot => (vec![(p(12.0, 12.0), p(12.0, 12.0))], 10.0),
        }
    }
}

/// An icon: a glyph, or a game's own picture, tinted with its content colour.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum Icon {
    /// A built-in glyph.
    Glyph(Glyph),
    /// A picture: white where it should take the content colour.
    Image(Handle<Image>),
}

impl From<Glyph> for Icon {
    fn from(glyph: Glyph) -> Self {
        Icon::Glyph(glyph)
    }
}

impl From<Handle<Image>> for Icon {
    fn from(image: Handle<Image>) -> Self {
        Icon::Image(image)
    }
}

/// Material's usual icon size, in logical pixels.
pub const ICON: f32 = 24.0;

/// One stroke of a glyph: a rounded bar from `from` to `to`, `width` wide, on
/// a grid scaled by `scale`.
fn stroke(from: Vec2, to: Vec2, width: f32, scale: f32) -> impl Bundle {
    let middle = (from + to) / 2.0 * scale;
    let length = from.distance(to) * scale + width * scale;
    let thickness = width * scale;
    let angle = (to - from).to_angle();
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(middle.x - length / 2.0),
            top: Val::Px(middle.y - thickness / 2.0),
            width: Val::Px(length),
            height: Val::Px(thickness),
            border_radius: BorderRadius::all(Val::Px(thickness / 2.0)),
            ..default()
        },
        UiTransform::from_rotation(Rot2::radians(angle)),
        BackgroundColor::default(),
        Content,
        Pickable::IGNORE,
    )
}

/// The marker on an icon's root.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct IconRoot;

/// `icon`, `size` logical pixels square, in the content colour of what it is
/// in.
pub fn icon(icon: impl Into<Icon>, size: f32) -> impl Bundle {
    let icon = icon.into();
    let node = Node {
        width: Val::Px(size),
        height: Val::Px(size),
        flex_shrink: 0.0,
        ..default()
    };
    (
        IconRoot,
        node,
        Pickable::IGNORE,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| match icon {
            Icon::Glyph(glyph) => {
                let (strokes, width) = glyph.strokes();
                let scale = size / ICON;
                for (from, to) in strokes {
                    parent.spawn(stroke(from, to, width, scale));
                }
            }
            Icon::Image(image) => {
                parent.spawn((
                    ImageNode {
                        image,
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    Content,
                    Pickable::IGNORE,
                ));
            }
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_stays_inside_its_grid() {
        for glyph in Glyph::ALL {
            let (strokes, width) = glyph.strokes();
            assert!(!strokes.is_empty(), "{glyph:?}");
            for (from, to) in strokes {
                for point in [from, to] {
                    let reach = width / 2.0;
                    assert!(
                        point.x - reach >= 0.0
                            && point.y - reach >= 0.0
                            && point.x + reach <= ICON
                            && point.y + reach <= ICON,
                        "{glyph:?} leaves its grid at {point}"
                    );
                }
            }
        }
    }
}
