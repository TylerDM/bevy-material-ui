//! Material's shapes, elevation, states, spacing and motion, as names.

use bevy::prelude::*;

/// How round a part's corners are, as Material 3's shape scale names it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum Corner {
    /// Square.
    None,
    /// 4 px: a menu, a snackbar, a plain tooltip.
    ExtraSmall,
    /// 8 px: a chip, a rich tooltip.
    Small,
    /// 12 px: a card, a small FAB.
    #[default]
    Medium,
    /// 16 px: a FAB, a navigation drawer.
    Large,
    /// 28 px: a dialog, a sheet.
    ExtraLarge,
    /// A half of the shorter side: a button, a badge, a slider's handle.
    Full,
}

impl Corner {
    /// Its radius, in logical pixels; [`Corner::Full`] is a pill at any size.
    pub fn radius(self) -> Val {
        match self {
            Corner::None => Val::ZERO,
            Corner::ExtraSmall => Val::Px(4.0),
            Corner::Small => Val::Px(8.0),
            Corner::Medium => Val::Px(12.0),
            Corner::Large => Val::Px(16.0),
            Corner::ExtraLarge => Val::Px(28.0),
            Corner::Full => Val::Percent(50.0),
        }
    }

    /// Every corner rounded so.
    pub fn all(self) -> BorderRadius {
        BorderRadius::all(self.radius())
    }

    /// The top two corners rounded so, the bottom square: a filled field, a
    /// bottom sheet.
    pub fn top(self) -> BorderRadius {
        BorderRadius::top(self.radius())
    }
}

/// How far a surface stands above the one below it, as Material 3's six
/// levels. Drawn as a shadow; a surface that should show its height as a
/// tint uses a higher surface container instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub enum Elevation {
    /// Flat: no shadow.
    #[default]
    Level0,
    /// 1 dp: an elevated card, a navigation drawer's sheet.
    Level1,
    /// 3 dp: a menu, a navigation bar, a scrolled top app bar.
    Level2,
    /// 6 dp: a FAB, a dialog, a search view.
    Level3,
    /// 8 dp: a hovered FAB.
    Level4,
    /// 12 dp: the highest.
    Level5,
}

impl Elevation {
    /// Every level, lowest first.
    pub const ALL: [Elevation; 6] = [
        Elevation::Level0,
        Elevation::Level1,
        Elevation::Level2,
        Elevation::Level3,
        Elevation::Level4,
        Elevation::Level5,
    ];

    /// Its height in dp.
    pub fn dp(self) -> f32 {
        match self {
            Elevation::Level0 => 0.0,
            Elevation::Level1 => 1.0,
            Elevation::Level2 => 3.0,
            Elevation::Level3 => 6.0,
            Elevation::Level4 => 8.0,
            Elevation::Level5 => 12.0,
        }
    }

    /// The level above this one, as a hovered control rises; the highest
    /// stays.
    pub fn raised(self) -> Self {
        match self {
            Elevation::Level0 => Elevation::Level1,
            Elevation::Level1 => Elevation::Level2,
            Elevation::Level2 => Elevation::Level3,
            Elevation::Level3 => Elevation::Level4,
            Elevation::Level4 | Elevation::Level5 => Elevation::Level5,
        }
    }

    /// Its shadow in `shadow`'s colour: Material's key and ambient lights,
    /// a sharp one below and a soft one around.
    pub fn shadow(self, shadow: Color) -> BoxShadow {
        let dp = self.dp();
        if dp == 0.0 {
            return BoxShadow(Vec::new());
        }
        let key = ShadowStyle {
            color: shadow.with_alpha(0.30),
            x_offset: Val::ZERO,
            y_offset: Val::Px(dp.clamp(1.0, 4.0)),
            spread_radius: Val::ZERO,
            blur_radius: Val::Px(dp.max(2.0)),
        };
        let ambient = ShadowStyle {
            color: shadow.with_alpha(0.15),
            x_offset: Val::ZERO,
            y_offset: Val::Px(dp / 2.0 + 1.0),
            spread_radius: Val::Px(dp / 3.0),
            blur_radius: Val::Px(dp * 2.0 + 2.0),
        };
        BoxShadow(vec![key, ambient])
    }
}

/// How strongly a control's state shows over it: Material's state layers,
/// its content colour laid over its container at these opacities.
pub mod state {
    /// Under the pointer.
    pub const HOVERED: f32 = 0.08;
    /// Holding the keyboard's focus.
    pub const FOCUSED: f32 = 0.10;
    /// Pressed.
    pub const PRESSED: f32 = 0.10;
    /// Being dragged.
    pub const DRAGGED: f32 = 0.16;
    /// A disabled control's content is drawn at this opacity...
    pub const DISABLED_CONTENT: f32 = 0.38;
    /// ...and its container at this one.
    pub const DISABLED_CONTAINER: f32 = 0.12;
}

/// Space between things, in logical pixels: Material's 4 px grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum Space {
    /// 4 px.
    ExtraSmall,
    /// 8 px.
    Small,
    /// 12 px.
    Medium,
    /// 16 px: the usual margin of a part's content.
    Large,
    /// 24 px: a dialog's padding.
    ExtraLarge,
    /// 32 px.
    Huge,
}

impl Space {
    /// Its size in logical pixels.
    pub fn px(self) -> f32 {
        match self {
            Space::ExtraSmall => 4.0,
            Space::Small => 8.0,
            Space::Medium => 12.0,
            Space::Large => 16.0,
            Space::ExtraLarge => 24.0,
            Space::Huge => 32.0,
        }
    }

    /// As a [`Val`].
    pub fn val(self) -> Val {
        Val::Px(self.px())
    }
}

/// How long Material's motion takes, in seconds.
pub mod duration {
    /// A small change: a state layer, a switch's handle.
    pub const SHORT: f32 = 0.1;
    /// A part entering: a menu, a snackbar.
    pub const MEDIUM: f32 = 0.25;
    /// A large change: a dialog, a sheet.
    pub const LONG: f32 = 0.45;
}

/// Material's standard easing, `cubic-bezier(0.2, 0, 0, 1)`, at `t` from 0
/// to 1.
pub fn emphasized(t: f32) -> f32 {
    cubic_bezier(0.2, 0.0, 0.0, 1.0, t.clamp(0.0, 1.0))
}

/// The y of a CSS cubic Bézier from (0, 0) to (1, 1) at x = `x`.
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let curve = |a: f32, b: f32, t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    // x rises with t on these curves, so halve the interval for t.
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..20 {
        let middle = f32::midpoint(low, high);
        if curve(x1, x2, middle) < x {
            low = middle;
        } else {
            high = middle;
        }
    }
    curve(y1, y2, f32::midpoint(low, high))
}

/// `over` laid on `under` at `opacity`, as a state layer is.
pub fn layered(under: Color, over: Color, opacity: f32) -> Color {
    let under_srgb = under.to_srgba();
    let over_srgb = over.to_srgba().with_alpha(1.0);
    if under_srgb.alpha == 0.0 {
        return over_srgb.with_alpha(opacity).into();
    }
    under_srgb
        .mix(&over_srgb, opacity)
        .with_alpha(under_srgb.alpha)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_follow_materials_shape_scale() {
        assert_eq!(Corner::None.radius(), Val::ZERO);
        assert_eq!(Corner::Medium.radius(), Val::Px(12.0));
        assert_eq!(Corner::ExtraLarge.radius(), Val::Px(28.0));
        assert_eq!(Corner::Full.radius(), Val::Percent(50.0));
    }

    #[test]
    fn a_higher_surface_casts_a_longer_shadow_and_a_flat_one_none() {
        assert!(Elevation::Level0.shadow(Color::BLACK).0.is_empty());
        let blur = |level: Elevation| match level.shadow(Color::BLACK).0[1].blur_radius {
            Val::Px(px) => px,
            other => panic!("{other:?}"),
        };
        for pair in Elevation::ALL[1..].windows(2) {
            assert!(blur(pair[1]) > blur(pair[0]), "{pair:?}");
        }
        assert_eq!(Elevation::Level1.raised(), Elevation::Level2);
        assert_eq!(Elevation::Level5.raised(), Elevation::Level5);
    }

    #[test]
    fn emphasized_easing_starts_at_zero_ends_at_one_and_leads() {
        assert!(emphasized(0.0).abs() < 1e-3);
        assert!((emphasized(1.0) - 1.0).abs() < 1e-3);
        assert!(
            emphasized(0.5) > 0.5,
            "Material's easing is ahead at half time"
        );
        assert!(emphasized(0.3) < emphasized(0.6));
    }

    #[test]
    fn a_state_layer_tints_toward_its_colour_by_its_opacity() {
        let tinted = layered(Color::WHITE, Color::BLACK, 0.5).to_srgba();
        assert!((tinted.red - 0.5).abs() < 0.01, "{tinted:?}");
        assert_eq!(tinted.alpha, 1.0);
        let wash = layered(Color::NONE, Color::BLACK, state::HOVERED).to_srgba();
        assert!(
            (wash.alpha - state::HOVERED).abs() < 1e-4,
            "on nothing, a wash"
        );
    }

    #[test]
    fn spacing_is_on_a_four_pixel_grid() {
        for space in [
            Space::ExtraSmall,
            Space::Small,
            Space::Medium,
            Space::Large,
            Space::ExtraLarge,
            Space::Huge,
        ] {
            assert_eq!(space.px() % 4.0, 0.0, "{space:?}");
        }
    }
}
