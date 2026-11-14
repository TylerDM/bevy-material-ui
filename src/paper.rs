//! Papers and cards: Material's surfaces, the sheets everything else is laid
//! on. A [`paper`] is a surface at a height - a colour role, a shadow and a
//! shape - and a [`card`] is a paper of one of Material's three kinds of card.

use bevy::prelude::*;

use crate::paint::{ContentColor, Fill, Paint, Raised, Stroke};
use crate::theme::{ColorRole, Corner, Elevation, Space};

/// A surface: a colour, a height, a shape and an outline.
#[derive(Clone, Debug, PartialEq)]
pub struct Paper {
    /// What it is filled with.
    pub surface: ColorRole,
    /// What is drawn on it.
    pub content: ColorRole,
    /// How high it stands.
    pub elevation: Elevation,
    /// Its corners.
    pub corner: Corner,
    /// Its border, if it has one.
    pub outline: Option<ColorRole>,
    /// The space inside its edge.
    pub padding: UiRect,
    /// Space between what is laid on it.
    pub gap: Val,
    /// Whether what is laid on it goes down (a column) or across (a row).
    pub direction: FlexDirection,
    /// Its width; Val::Auto fits what is on it.
    pub width: Val,
    /// Its height; Val::Auto fits what is on it.
    pub height: Val,
}

impl Default for Paper {
    fn default() -> Self {
        Self {
            surface: ColorRole::SurfaceContainerLow,
            content: ColorRole::OnSurface,
            elevation: Elevation::Level1,
            corner: Corner::Medium,
            outline: None,
            padding: UiRect::all(Space::Large.val()),
            gap: Space::Small.val(),
            direction: FlexDirection::Column,
            width: Val::Auto,
            height: Val::Auto,
        }
    }
}

/// A paper standing one level up: the usual surface for a panel.
pub fn paper() -> Paper {
    Paper::default()
}

impl Paper {
    /// Filled with `surface`, with what is on it in `content`.
    pub fn surface(mut self, surface: ColorRole, content: ColorRole) -> Self {
        self.surface = surface;
        self.content = content;
        self
    }

    /// Standing at `elevation`.
    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// With `corner`s.
    pub fn corner(mut self, corner: Corner) -> Self {
        self.corner = corner;
        self
    }

    /// Square, as a paper filling an edge of the screen.
    pub fn square(self) -> Self {
        self.corner(Corner::None)
    }

    /// Flat, with an outline in [`ColorRole::OutlineVariant`].
    pub fn outlined(mut self) -> Self {
        self.elevation = Elevation::Level0;
        self.outline = Some(ColorRole::OutlineVariant);
        self
    }

    /// With `padding` inside its edge.
    pub fn padding(mut self, padding: UiRect) -> Self {
        self.padding = padding;
        self
    }

    /// With `gap` between what is laid on it.
    pub fn gap(mut self, gap: Val) -> Self {
        self.gap = gap;
        self
    }

    /// Laying what is on it across, as a row.
    pub fn row(mut self) -> Self {
        self.direction = FlexDirection::Row;
        self
    }

    /// `width` wide.
    pub fn width(mut self, width: Val) -> Self {
        self.width = width;
        self
    }

    /// `height` tall.
    pub fn height(mut self, height: Val) -> Self {
        self.height = height;
        self
    }

    /// Its components: spawn it, then its children. It carries its own
    /// `Node`; size it with [`Paper::width`] and [`Paper::height`].
    pub fn bundle(self) -> impl Bundle {
        let border = if self.outline.is_some() {
            UiRect::all(Val::Px(1.0))
        } else {
            UiRect::ZERO
        };
        (
            Node {
                flex_direction: self.direction,
                width: self.width,
                height: self.height,
                padding: self.padding,
                row_gap: self.gap,
                column_gap: self.gap,
                border,
                border_radius: self.corner.all(),
                ..default()
            },
            Fill::new(self.surface),
            ContentColor(Paint::new(self.content)),
            Raised(self.elevation),
            Stroke::new(self.outline.unwrap_or(self.surface)),
        )
    }
}

/// Material's three kinds of card.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum CardKind {
    /// On a low container, standing a level up.
    #[default]
    Elevated,
    /// On the highest container, flat.
    Filled,
    /// On the surface, flat, with an outline.
    Outlined,
}

/// A card of `kind`: a paper with a card's corners and padding.
pub fn card(kind: CardKind) -> Paper {
    let paper = Paper::default().corner(Corner::Medium);
    match kind {
        CardKind::Elevated => paper,
        CardKind::Filled => paper
            .surface(ColorRole::SurfaceContainerHighest, ColorRole::OnSurface)
            .elevation(Elevation::Level0),
        CardKind::Outlined => paper
            .surface(ColorRole::Surface, ColorRole::OnSurface)
            .outlined(),
    }
}

/// A rule between groups: a hairline in [`ColorRole::OutlineVariant`],
/// across its parent's width (or down its height, `vertical`).
pub fn divider(vertical: bool) -> impl Bundle {
    let node = if vertical {
        Node {
            width: Val::Px(1.0),
            align_self: AlignSelf::Stretch,
            flex_shrink: 0.0,
            ..default()
        }
    } else {
        Node {
            height: Val::Px(1.0),
            width: Val::Percent(100.0),
            flex_shrink: 0.0,
            ..default()
        }
    };
    (node, Fill::new(ColorRole::OutlineVariant), Pickable::IGNORE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_cards_are_raised_filled_and_outlined() {
        let elevated = card(CardKind::Elevated);
        assert_eq!(elevated.elevation, Elevation::Level1);
        assert_eq!(elevated.surface, ColorRole::SurfaceContainerLow);
        let filled = card(CardKind::Filled);
        assert_eq!(filled.elevation, Elevation::Level0);
        assert_eq!(filled.surface, ColorRole::SurfaceContainerHighest);
        let outlined = card(CardKind::Outlined);
        assert_eq!(outlined.outline, Some(ColorRole::OutlineVariant));
        assert_eq!(outlined.elevation, Elevation::Level0);
    }
}
