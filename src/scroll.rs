//! A scroll area: a game's parts in a column that scrolls with the wheel,
//! the trackpad or a drag of its thin scrollbar - Bevy's own
//! [`ScrollArea`] and [`Scrollbar`] - its thumb hidden while everything fits.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{
    ControlOrientation, ScrollArea, Scrollbar, ScrollbarDragState, ScrollbarThumb,
};

use crate::paint::{Fill, Paint};
use crate::theme::{ColorRole, Corner};

/// The scrollbar's gutter and its thumb's width, in logical pixels.
pub const SCROLLBAR_GUTTER: f32 = 8.0;
/// The thumb's opacity at rest; under the pointer or dragged it is opaque.
pub const THUMB_REST_OPACITY: f32 = 0.38;

/// Marks a scroll area's scrollbar.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MaterialScrollbar;

/// Marks a scroll area's thumb.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct MaterialScrollThumb;

/// Marks the column inside a scroll area that scrolls.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct ScrollViewport;

/// A scroll area holding the parts `content` spawns, in a column; size it
/// with a `Node` of your own around it, or give it a height.
pub fn scroll_area(
    height: Val,
    content: impl FnOnce(&mut ChildSpawner) + Send + Sync + 'static,
) -> impl Bundle {
    (
        Node {
            height,
            min_height: Val::ZERO,
            width: Val::Percent(100.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |area: &mut ChildSpawner| {
            let viewport = area
                .spawn((
                    ScrollViewport,
                    ScrollArea,
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        min_height: Val::ZERO,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .with_children(content)
                .id();
            area.spawn((
                MaterialScrollbar,
                Scrollbar::new(viewport, ControlOrientation::Vertical, 24.0),
                Node {
                    width: Val::Px(SCROLLBAR_GUTTER),
                    align_self: AlignSelf::Stretch,
                    flex_shrink: 0.0,
                    ..default()
                },
                children![(
                    MaterialScrollThumb,
                    ScrollbarThumb {
                        border_radius: Corner::Full.all(),
                        ..default()
                    },
                    Hovered::default(),
                    Fill(Paint::faded(
                        ColorRole::OnSurfaceVariant,
                        THUMB_REST_OPACITY
                    )),
                )],
            ));
        })),
    )
}

/// A thumb that may have been hovered, left, grabbed or let go this frame.
type ThumbChanged = (
    With<MaterialScrollThumb>,
    Or<(Changed<Hovered>, Changed<ScrollbarDragState>)>,
);

/// Draws each thumb opaque while it is under the pointer or dragged.
fn draw_thumbs(
    thumbs: Query<(Entity, &Hovered, &ScrollbarDragState, &Fill), ThumbChanged>,
    mut commands: Commands,
) {
    for (thumb, hovered, drag, fill) in &thumbs {
        let opacity = if hovered.0 || drag.dragging {
            1.0
        } else {
            THUMB_REST_OPACITY
        };
        if fill.0.opacity != opacity {
            commands
                .entity(thumb)
                .try_insert(Fill(Paint::faded(ColorRole::OnSurfaceVariant, opacity)));
        }
    }
}

/// Hides each scrollbar while everything in its area fits.
fn hide_needless_scrollbars(
    mut scrollbars: Query<(&Scrollbar, &mut Visibility), With<MaterialScrollbar>>,
    areas: Query<&ComputedNode>,
) {
    for (scrollbar, mut visibility) in &mut scrollbars {
        let Ok(area) = areas.get(scrollbar.target) else {
            continue;
        };
        let wanted = if overflows(area.content_size().y, area.size().y) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(wanted);
    }
}

/// Whether content `content` tall overflows a view `view` tall, give or take
/// a pixel's rounding.
pub fn overflows(content: f32, view: f32) -> bool {
    content > view + 0.5
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_thumbs).add_systems(
        PostUpdate,
        hide_needless_scrollbars.after(bevy::ui::UiSystems::Layout),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scrollbar_is_needed_only_when_its_content_is_taller_than_its_view() {
        assert!(overflows(400.0, 200.0));
        assert!(!overflows(200.0, 200.0));
        assert!(
            !overflows(200.3, 200.0),
            "a rounding pixel is not an overflow"
        );
        assert!(!overflows(100.0, 200.0));
    }
}
