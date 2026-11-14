//! Material's colour roles, and a scheme that gives each a colour.
//!
//! A part of the interface asks for a role - [`ColorRole::Primary`],
//! [`ColorRole::OnSurfaceVariant`] - never a colour of its own, so the whole
//! interface changes with the scheme: light to dark, or to a game's own
//! colours, in one place.

use bevy::prelude::*;

/// Whether a scheme is drawn light or dark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum Brightness {
    /// Dark content on light surfaces.
    #[default]
    Light,
    /// Light content on dark surfaces.
    Dark,
}

macro_rules! roles {
    ($($(#[$doc:meta])* $role:ident),* $(,)?) => {
        /// A colour by what it is for, as Material 3 names them. Every `On*`
        /// role is drawn on the role it names and reads on it at 4.5:1 or
        /// better.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
        pub enum ColorRole {
            $($(#[$doc])* $role,)*
        }

        impl ColorRole {
            /// Every role, in order.
            pub const ALL: &'static [ColorRole] = &[$(ColorRole::$role,)*];
            /// How many roles there are.
            pub const COUNT: usize = Self::ALL.len();
        }
    };
}

roles! {
    /// The main action, the selected state, focus, a slider's fill.
    Primary,
    /// Text and icons on [`ColorRole::Primary`].
    OnPrimary,
    /// A container that stands out less than primary: a FAB, a selected chip.
    PrimaryContainer,
    /// Text and icons on [`ColorRole::PrimaryContainer`].
    OnPrimaryContainer,
    /// Less prominent parts: filter chips, tonal buttons.
    Secondary,
    /// Text and icons on [`ColorRole::Secondary`].
    OnSecondary,
    /// Tonal buttons, the active indicator of navigation.
    SecondaryContainer,
    /// Text and icons on [`ColorRole::SecondaryContainer`].
    OnSecondaryContainer,
    /// Contrasting accents.
    Tertiary,
    /// Text and icons on [`ColorRole::Tertiary`].
    OnTertiary,
    /// A contrasting container.
    TertiaryContainer,
    /// Text and icons on [`ColorRole::TertiaryContainer`].
    OnTertiaryContainer,
    /// Errors, and what can't be done - never selection.
    Error,
    /// Text and icons on [`ColorRole::Error`].
    OnError,
    /// A container for an error: a badge's, a banner's.
    ErrorContainer,
    /// Text and icons on [`ColorRole::ErrorContainer`].
    OnErrorContainer,
    /// The default background of everything.
    Surface,
    /// Text and icons on any surface.
    OnSurface,
    /// An older role, kept for schemes that name it; prefer the containers.
    SurfaceVariant,
    /// Quieter text and icons on a surface: hints, captions, inactive icons.
    OnSurfaceVariant,
    /// The dimmest surface.
    SurfaceDim,
    /// The brightest surface.
    SurfaceBright,
    /// The lowest container: below a card on a busy surface.
    SurfaceContainerLowest,
    /// A low container: an elevated card, a navigation drawer.
    SurfaceContainerLow,
    /// The default container: a navigation bar, a menu.
    SurfaceContainer,
    /// A high container: a dialog, a filled field.
    SurfaceContainerHigh,
    /// The highest container: a filled card, a switch's track.
    SurfaceContainerHighest,
    /// A surface that stands out against all the others: a snackbar, a tooltip.
    InverseSurface,
    /// Text and icons on [`ColorRole::InverseSurface`].
    InverseOnSurface,
    /// The primary colour on [`ColorRole::InverseSurface`]: a snackbar's action.
    InversePrimary,
    /// A border that must be seen: an outlined field's, an outlined button's.
    Outline,
    /// A decorative border: a divider, an outlined card's.
    OutlineVariant,
    /// Shadows of elevation.
    Shadow,
    /// Behind a modal: a dialog, a modal sheet.
    Scrim,
}

/// A colour for every [`ColorRole`]: what the interface is drawn with.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct Scheme {
    brightness: Brightness,
    colors: [Color; ColorRole::COUNT],
}

impl Scheme {
    /// A scheme of `brightness` giving each role the colour `color` returns.
    pub fn from_fn(brightness: Brightness, mut color: impl FnMut(ColorRole) -> Color) -> Self {
        let mut colors = [Color::NONE; ColorRole::COUNT];
        for (slot, role) in colors.iter_mut().zip(ColorRole::ALL) {
            *slot = color(*role);
        }
        Self { brightness, colors }
    }

    /// Whether this scheme is light or dark.
    pub fn brightness(&self) -> Brightness {
        self.brightness
    }

    /// The colour of `role`.
    pub fn color(&self, role: ColorRole) -> Color {
        self.colors[role as usize]
    }

    /// This scheme with `role` drawn in `color`: a game's own touch on a
    /// generated scheme.
    pub fn with(mut self, role: ColorRole, color: Color) -> Self {
        self.colors[role as usize] = color;
        self
    }

    /// Material 3's baseline light scheme, from its baseline purple.
    pub fn baseline_light() -> Self {
        use ColorRole::*;
        Self::from_fn(Brightness::Light, |role| {
            hex(match role {
                Primary => 0x6750A4,
                OnPrimary | OnSecondary | OnTertiary | OnError | SurfaceContainerLowest => 0xFFFFFF,
                PrimaryContainer => 0xEADDFF,
                OnPrimaryContainer => 0x21005D,
                Secondary => 0x625B71,
                SecondaryContainer => 0xE8DEF8,
                OnSecondaryContainer => 0x1D192B,
                Tertiary => 0x7D5260,
                TertiaryContainer => 0xFFD8E4,
                OnTertiaryContainer => 0x31111D,
                Error => 0xB3261E,
                ErrorContainer => 0xF9DEDC,
                OnErrorContainer => 0x410E0B,
                Surface | SurfaceBright => 0xFEF7FF,
                OnSurface => 0x1D1B20,
                SurfaceVariant => 0xE7E0EC,
                OnSurfaceVariant => 0x49454F,
                SurfaceDim => 0xDED8E1,
                SurfaceContainerLow => 0xF7F2FA,
                SurfaceContainer => 0xF3EDF7,
                SurfaceContainerHigh => 0xECE6F0,
                SurfaceContainerHighest => 0xE6E0E9,
                InverseSurface => 0x322F35,
                InverseOnSurface => 0xF5EFF7,
                InversePrimary => 0xD0BCFF,
                Outline => 0x79747E,
                OutlineVariant => 0xCAC4D0,
                Shadow | Scrim => 0x000000,
            })
        })
    }

    /// Material 3's baseline dark scheme, from its baseline purple.
    pub fn baseline_dark() -> Self {
        use ColorRole::*;
        Self::from_fn(Brightness::Dark, |role| {
            hex(match role {
                Primary => 0xD0BCFF,
                OnPrimary => 0x381E72,
                PrimaryContainer => 0x4F378B,
                OnPrimaryContainer => 0xEADDFF,
                Secondary => 0xCCC2DC,
                OnSecondary => 0x332D41,
                SecondaryContainer => 0x4A4458,
                OnSecondaryContainer => 0xE8DEF8,
                Tertiary => 0xEFB8C8,
                OnTertiary => 0x492532,
                TertiaryContainer => 0x633B48,
                OnTertiaryContainer => 0xFFD8E4,
                Error => 0xF2B8B5,
                OnError => 0x601410,
                ErrorContainer => 0x8C1D18,
                OnErrorContainer => 0xF9DEDC,
                Surface | SurfaceDim => 0x141218,
                OnSurface | InverseSurface => 0xE6E0E9,
                SurfaceVariant | OutlineVariant => 0x49454F,
                OnSurfaceVariant => 0xCAC4D0,
                SurfaceBright => 0x3B383E,
                SurfaceContainerLowest => 0x0F0D13,
                SurfaceContainerLow => 0x1D1B20,
                SurfaceContainer => 0x211F26,
                SurfaceContainerHigh => 0x2B2930,
                SurfaceContainerHighest => 0x36343B,
                InverseOnSurface => 0x322F35,
                InversePrimary => 0x6750A4,
                Outline => 0x938F99,
                Shadow | Scrim => 0x000000,
            })
        })
    }
}

impl Default for Scheme {
    fn default() -> Self {
        Self::baseline_light()
    }
}

/// An opaque colour from `0xRRGGBB`.
pub const fn hex(rgb: u32) -> Color {
    Color::srgb_u8(
        ((rgb >> 16) & 0xFF) as u8,
        ((rgb >> 8) & 0xFF) as u8,
        (rgb & 0xFF) as u8,
    )
}

/// WCAG's contrast ratio between two opaque colours: 1 to 21.
pub fn contrast(a: Color, b: Color) -> f32 {
    let luminance = |color: Color| {
        let linear = color.to_linear();
        0.2126 * linear.red + 0.7152 * linear.green + 0.0722 * linear.blue
    };
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Each `On*` role with the role it is drawn on: the pairs that must read.
pub const READABLE_PAIRS: &[(ColorRole, ColorRole)] = {
    use ColorRole::*;
    &[
        (OnPrimary, Primary),
        (OnPrimaryContainer, PrimaryContainer),
        (OnSecondary, Secondary),
        (OnSecondaryContainer, SecondaryContainer),
        (OnTertiary, Tertiary),
        (OnTertiaryContainer, TertiaryContainer),
        (OnError, Error),
        (OnErrorContainer, ErrorContainer),
        (OnSurface, Surface),
        (OnSurfaceVariant, Surface),
        (OnSurface, SurfaceContainerLowest),
        (OnSurface, SurfaceContainerLow),
        (OnSurface, SurfaceContainer),
        (OnSurface, SurfaceContainerHigh),
        (OnSurface, SurfaceContainerHighest),
        (OnSurfaceVariant, SurfaceContainerHighest),
        (Primary, Surface),
        (InverseOnSurface, InverseSurface),
        (InversePrimary, InverseSurface),
    ]
};

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_readable(scheme: &Scheme, name: &str) {
        for &(text, surface) in READABLE_PAIRS {
            let ratio = contrast(scheme.color(text), scheme.color(surface));
            assert!(
                ratio >= 4.5,
                "{name}: {text:?} on {surface:?} is {ratio:.2}:1"
            );
        }
    }

    #[test]
    fn every_text_role_of_the_baseline_schemes_reads_on_its_surface() {
        assert_readable(&Scheme::baseline_light(), "light");
        assert_readable(&Scheme::baseline_dark(), "dark");
    }

    #[test]
    fn a_scheme_gives_each_role_its_own_colour_and_a_touch_changes_one() {
        let scheme = Scheme::baseline_light();
        assert_eq!(scheme.color(ColorRole::Primary), hex(0x6750A4));
        assert_eq!(scheme.color(ColorRole::OnPrimary), hex(0xFFFFFF));
        let touched = scheme.clone().with(ColorRole::Primary, hex(0x123456));
        assert_eq!(touched.color(ColorRole::Primary), hex(0x123456));
        assert_eq!(
            touched.color(ColorRole::Secondary),
            scheme.color(ColorRole::Secondary)
        );
    }

    #[test]
    fn the_baseline_schemes_are_light_and_dark() {
        assert_eq!(Scheme::baseline_light().brightness(), Brightness::Light);
        assert_eq!(Scheme::baseline_dark().brightness(), Brightness::Dark);
        assert!(
            contrast(
                Scheme::baseline_dark().color(ColorRole::Surface),
                Color::BLACK
            ) < 2.0,
            "a dark surface is dark"
        );
    }

    #[test]
    fn contrast_runs_from_one_to_twenty_one() {
        assert!((contrast(Color::WHITE, Color::WHITE) - 1.0).abs() < 0.001);
        assert!((contrast(Color::BLACK, Color::WHITE) - 21.0).abs() < 0.01);
    }
}
