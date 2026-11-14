//! A whole scheme from one colour: Material's tonal-spot scheme, as the
//! Material Theme Builder makes from a seed.
//!
//! Material builds its palettes in HCT, whose tone is CIE L*. This builds
//! them in CIE `LCh` - the same tone, so every pair of roles keeps Material's
//! contrast, with `LCh`'s hue and chroma standing in for `CAM16`'s. A seed gives
//! colours close to the Theme Builder's, not identical; a game that wants
//! exact ones writes them into a [`Scheme`] with [`Scheme::from_fn`].

use bevy::color::{Lcha, Srgba};
use bevy::prelude::*;

use super::scheme::{Brightness, ColorRole, Scheme};

/// One hue at one chroma, at every tone from black (0) to white (100).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalPalette {
    /// The hue, in degrees.
    pub hue: f32,
    /// The chroma wanted, in CIE units (0 to about 130); each tone is drawn
    /// at as much of it as sRGB can show.
    pub chroma: f32,
}

impl TonalPalette {
    /// The palette of `color`'s own hue and chroma.
    pub fn of(color: Color) -> Self {
        let lch = Lcha::from(color);
        Self {
            hue: lch.hue,
            chroma: lch.chroma * 100.0,
        }
    }

    /// This palette at `tone`, 0 black to 100 white: the most chroma up to
    /// the palette's that sRGB holds at that tone and hue.
    pub fn tone(self, tone: f32) -> Color {
        let lightness = (tone / 100.0).clamp(0.0, 1.0);
        let at = |chroma: f32| Lcha::new(lightness, chroma / 100.0, self.hue, 1.0);
        if in_gamut(at(self.chroma)) {
            return Srgba::from(at(self.chroma)).into();
        }
        // Halve the interval: the chroma sRGB holds is a single edge.
        let (mut low, mut high) = (0.0, self.chroma);
        for _ in 0..24 {
            let middle = f32::midpoint(low, high);
            if in_gamut(at(middle)) {
                low = middle;
            } else {
                high = middle;
            }
        }
        let srgb = Srgba::from(at(low));
        Srgba::new(
            srgb.red.clamp(0.0, 1.0),
            srgb.green.clamp(0.0, 1.0),
            srgb.blue.clamp(0.0, 1.0),
            1.0,
        )
        .into()
    }
}

/// Whether sRGB can show `color`.
fn in_gamut(color: Lcha) -> bool {
    const EPSILON: f32 = 1e-4;
    let srgb = Srgba::from(color);
    [srgb.red, srgb.green, srgb.blue]
        .iter()
        .all(|channel| (-EPSILON..=1.0 + EPSILON).contains(channel))
}

/// The six palettes of a tonal-spot scheme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palettes {
    /// Primary roles: the seed's hue, at least moderately colourful.
    pub primary: TonalPalette,
    /// Secondary roles: the seed's hue, muted.
    pub secondary: TonalPalette,
    /// Tertiary roles: a neighbouring hue.
    pub tertiary: TonalPalette,
    /// Surfaces: the seed's hue, nearly grey.
    pub neutral: TonalPalette,
    /// Outlines and quieter text: the seed's hue, a little less grey.
    pub neutral_variant: TonalPalette,
    /// Error roles: always red.
    pub error: TonalPalette,
}

impl Palettes {
    /// The palettes Material's tonal spot makes from `seed`.
    pub fn tonal_spot(seed: Color) -> Self {
        let seed = TonalPalette::of(seed);
        let hue = |turn: f32| (seed.hue + turn).rem_euclid(360.0);
        Self {
            primary: TonalPalette {
                hue: seed.hue,
                chroma: seed.chroma.max(48.0),
            },
            secondary: TonalPalette {
                hue: seed.hue,
                chroma: 16.0,
            },
            tertiary: TonalPalette {
                hue: hue(60.0),
                chroma: 24.0,
            },
            neutral: TonalPalette {
                hue: seed.hue,
                chroma: 4.0,
            },
            neutral_variant: TonalPalette {
                hue: seed.hue,
                chroma: 8.0,
            },
            error: TonalPalette {
                hue: 25.0,
                chroma: 84.0,
            },
        }
    }

    /// The scheme of these palettes at `brightness`: each role the tone
    /// Material gives it.
    pub fn scheme(&self, brightness: Brightness) -> Scheme {
        use ColorRole::*;
        let light = brightness == Brightness::Light;
        let pick = |light_tone: f32, dark_tone: f32| if light { light_tone } else { dark_tone };
        Scheme::from_fn(brightness, |role| {
            let (palette, tone) = match role {
                Primary => (self.primary, pick(40.0, 80.0)),
                OnPrimary => (self.primary, pick(100.0, 20.0)),
                PrimaryContainer => (self.primary, pick(90.0, 30.0)),
                OnPrimaryContainer => (self.primary, pick(10.0, 90.0)),
                InversePrimary => (self.primary, pick(80.0, 40.0)),
                Secondary => (self.secondary, pick(40.0, 80.0)),
                OnSecondary => (self.secondary, pick(100.0, 20.0)),
                SecondaryContainer => (self.secondary, pick(90.0, 30.0)),
                OnSecondaryContainer => (self.secondary, pick(10.0, 90.0)),
                Tertiary => (self.tertiary, pick(40.0, 80.0)),
                OnTertiary => (self.tertiary, pick(100.0, 20.0)),
                TertiaryContainer => (self.tertiary, pick(90.0, 30.0)),
                OnTertiaryContainer => (self.tertiary, pick(10.0, 90.0)),
                Error => (self.error, pick(40.0, 80.0)),
                OnError => (self.error, pick(100.0, 20.0)),
                ErrorContainer => (self.error, pick(90.0, 30.0)),
                OnErrorContainer => (self.error, pick(10.0, 90.0)),
                Surface => (self.neutral, pick(98.0, 6.0)),
                OnSurface => (self.neutral, pick(10.0, 90.0)),
                SurfaceDim => (self.neutral, pick(87.0, 6.0)),
                SurfaceBright => (self.neutral, pick(98.0, 24.0)),
                SurfaceContainerLowest => (self.neutral, pick(100.0, 4.0)),
                SurfaceContainerLow => (self.neutral, pick(96.0, 10.0)),
                SurfaceContainer => (self.neutral, pick(94.0, 12.0)),
                SurfaceContainerHigh => (self.neutral, pick(92.0, 17.0)),
                SurfaceContainerHighest => (self.neutral, pick(90.0, 22.0)),
                InverseSurface => (self.neutral, pick(20.0, 90.0)),
                InverseOnSurface => (self.neutral, pick(95.0, 20.0)),
                SurfaceVariant => (self.neutral_variant, pick(90.0, 30.0)),
                OnSurfaceVariant => (self.neutral_variant, pick(30.0, 80.0)),
                Outline => (self.neutral_variant, pick(50.0, 60.0)),
                OutlineVariant => (self.neutral_variant, pick(80.0, 30.0)),
                Shadow | Scrim => (self.neutral, 0.0),
            };
            palette.tone(tone)
        })
    }
}

impl Scheme {
    /// The tonal-spot scheme of `seed` at `brightness`, as Material's
    /// Theme Builder makes one: a game's brand colour in, every role out.
    pub fn from_seed(seed: Color, brightness: Brightness) -> Self {
        Palettes::tonal_spot(seed).scheme(brightness)
    }
}

#[cfg(test)]
mod tests {
    use super::super::scheme::{READABLE_PAIRS, contrast};
    use super::*;

    #[test]
    fn a_tone_is_its_lightness_so_white_and_black_are_the_ends() {
        let palette = TonalPalette {
            hue: 280.0,
            chroma: 40.0,
        };
        assert!(contrast(palette.tone(100.0), Color::WHITE) < 1.01);
        assert!(contrast(palette.tone(0.0), Color::BLACK) < 1.01);
        let darker = contrast(palette.tone(40.0), Color::WHITE);
        let lighter = contrast(palette.tone(80.0), Color::WHITE);
        assert!(darker > lighter, "tone 40 is darker than tone 80");
    }

    #[test]
    fn a_tone_beyond_srgb_is_drawn_at_the_chroma_srgb_holds() {
        let palette = TonalPalette {
            hue: 140.0,
            chroma: 130.0,
        };
        for tone in [10.0, 50.0, 90.0] {
            let srgb = palette.tone(tone).to_srgba();
            for channel in [srgb.red, srgb.green, srgb.blue] {
                assert!((0.0..=1.0).contains(&channel), "{tone}: {srgb:?}");
            }
        }
    }

    #[test]
    fn every_text_role_of_a_seeded_scheme_reads_on_its_surface_for_many_seeds() {
        for seed in [
            0x6750A4, 0x545CB0, 0xDD4C1D, 0x1B8A3C, 0xF5C400, 0x00A0B0, 0x803D1B, 0x000000,
            0xFFFFFF, 0x7F7F7F,
        ] {
            for brightness in [Brightness::Light, Brightness::Dark] {
                let scheme = Scheme::from_seed(super::super::scheme::hex(seed), brightness);
                for &(text, surface) in READABLE_PAIRS {
                    let ratio = contrast(scheme.color(text), scheme.color(surface));
                    assert!(
                        ratio >= 4.5,
                        "{seed:06X} {brightness:?}: {text:?} on {surface:?} is {ratio:.2}:1"
                    );
                }
            }
        }
    }

    #[test]
    fn a_seeded_scheme_keeps_the_seeds_hue_in_its_primary() {
        let seed = super::super::scheme::hex(0x1B8A3C);
        let primary = Scheme::from_seed(seed, Brightness::Light).color(ColorRole::Primary);
        let hue = |color: Color| Lcha::from(color).hue;
        let turn = (hue(primary) - hue(seed)).abs();
        assert!(
            turn.min(360.0 - turn) < 3.0,
            "{} against {}",
            hue(primary),
            hue(seed)
        );
    }
}
