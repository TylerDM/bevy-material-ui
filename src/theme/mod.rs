//! The theme: Material's colour roles ([`Scheme`]), its type scale
//! ([`TypeRole`]), its shapes, elevation, states, spacing and motion
//! ([`tokens`]), and the faces text is set in ([`MaterialFonts`]) - all held
//! in one resource, [`MaterialTheme`]. Replace it, or change it, and the
//! whole interface repaints.

mod scheme;
mod seed;
pub mod tokens;
mod typography;

use bevy::prelude::*;
use bevy::text::{FontSize, LineHeight};

pub use scheme::{Brightness, ColorRole, READABLE_PAIRS, Scheme, contrast, hex};
pub use seed::{Palettes, TonalPalette};
pub use tokens::{Corner, Elevation, Space, emphasized, layered, state};
pub use typography::TypeRole;

/// Roboto, built into the crate: the faces' default with the `roboto`
/// feature.
#[cfg(feature = "roboto")]
pub const ROBOTO: Handle<Font> = bevy::asset::uuid_handle!("6e7b1c1e-3f43-4f4e-9a1c-5d0b3a8e2f17");

/// The faces text is set in.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct MaterialFonts {
    /// Body text, labels and titles but the largest.
    pub plain: Handle<Font>,
    /// Display, headline and large title text; the plain face if a game has
    /// no brand face of its own.
    pub brand: Handle<Font>,
}

impl Default for MaterialFonts {
    fn default() -> Self {
        #[cfg(feature = "roboto")]
        let face = ROBOTO;
        #[cfg(not(feature = "roboto"))]
        let face = Handle::default();
        Self {
            plain: face.clone(),
            brand: face,
        }
    }
}

impl MaterialFonts {
    /// One face for everything.
    pub fn single(face: Handle<Font>) -> Self {
        Self {
            plain: face.clone(),
            brand: face,
        }
    }

    /// The face `role` is set in.
    pub fn face(&self, role: TypeRole) -> &Handle<Font> {
        if role.is_brand() {
            &self.brand
        } else {
            &self.plain
        }
    }
}

/// The theme the whole interface is drawn in. Every part asks it for a role,
/// a colour or a kind of text, at the moment it is drawn, so changing it
/// repaints everything.
#[derive(Resource, Clone, Debug, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct MaterialTheme {
    /// The colours.
    pub scheme: Scheme,
    /// The faces.
    pub fonts: MaterialFonts,
    /// The colour the scheme was made from, if it was, so it can be made
    /// again at the other brightness.
    pub seed: Option<Color>,
}

impl Default for MaterialTheme {
    fn default() -> Self {
        Self::light()
    }
}

impl MaterialTheme {
    /// Material's baseline light theme.
    pub fn light() -> Self {
        Self {
            scheme: Scheme::baseline_light(),
            fonts: MaterialFonts::default(),
            seed: None,
        }
    }

    /// Material's baseline dark theme.
    pub fn dark() -> Self {
        Self {
            scheme: Scheme::baseline_dark(),
            ..Self::light()
        }
    }

    /// A theme made from `seed`, a game's brand colour, at `brightness`.
    pub fn from_seed(seed: Color, brightness: Brightness) -> Self {
        Self {
            scheme: Scheme::from_seed(seed, brightness),
            fonts: MaterialFonts::default(),
            seed: Some(seed),
        }
    }

    /// This theme with `fonts`.
    pub fn with_fonts(mut self, fonts: MaterialFonts) -> Self {
        self.fonts = fonts;
        self
    }

    /// Whether it is light or dark.
    pub fn brightness(&self) -> Brightness {
        self.scheme.brightness()
    }

    /// This theme made again at `brightness`: from its seed, or from the
    /// baseline. A scheme written by hand, with no seed, becomes the baseline.
    pub fn at(&self, brightness: Brightness) -> Self {
        if brightness == self.brightness() {
            return self.clone();
        }
        let scheme = match (self.seed, brightness) {
            (Some(seed), _) => Scheme::from_seed(seed, brightness),
            (None, Brightness::Light) => Scheme::baseline_light(),
            (None, Brightness::Dark) => Scheme::baseline_dark(),
        };
        Self {
            scheme,
            ..self.clone()
        }
    }

    /// The colour of `role`.
    pub fn color(&self, role: ColorRole) -> Color {
        self.scheme.color(role)
    }

    /// The face, size and weight of `role`.
    pub fn font(&self, role: TypeRole) -> TextFont {
        TextFont {
            font: self.fonts.face(role).clone().into(),
            font_size: FontSize::Px(role.size()),
            weight: role.weight(),
            ..default()
        }
    }

    /// The line height of `role`.
    pub fn line_height(role: TypeRole) -> LineHeight {
        LineHeight::Px(role.line_height())
    }
}

/// Puts Roboto into the app's fonts, under [`ROBOTO`].
#[cfg(feature = "roboto")]
pub(crate) fn add_roboto(fonts: Option<ResMut<Assets<Font>>>) {
    const BYTES: &[u8] = include_bytes!("../../assets/fonts/Roboto.ttf");
    if let Some(mut fonts) = fonts {
        fonts
            .insert(&ROBOTO, Font::from_bytes(BYTES.to_vec()))
            .expect("a fixed handle is never stale");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seeded_theme_turns_dark_from_its_seed_and_a_plain_one_from_the_baseline() {
        let seeded = MaterialTheme::from_seed(hex(0x1B8A3C), Brightness::Light);
        let dark = seeded.at(Brightness::Dark);
        assert_eq!(dark.brightness(), Brightness::Dark);
        assert_eq!(dark.scheme, Scheme::from_seed(hex(0x1B8A3C), Brightness::Dark));
        assert_eq!(dark.seed, seeded.seed);

        assert_eq!(MaterialTheme::light().at(Brightness::Dark), MaterialTheme::dark());
        assert_eq!(MaterialTheme::dark().at(Brightness::Light), MaterialTheme::light());
    }

    #[test]
    fn text_is_set_in_its_roles_face_size_and_weight() {
        let brand = Handle::<Font>::default();
        let fonts = MaterialFonts {
            plain: Handle::default(),
            brand: brand.clone(),
        };
        let theme = MaterialTheme::light().with_fonts(fonts);
        let font = theme.font(TypeRole::LabelLarge);
        assert_eq!(font.font_size, FontSize::Px(14.0));
        assert_eq!(font.weight, bevy::text::FontWeight::MEDIUM);
        assert_eq!(theme.fonts.face(TypeRole::HeadlineSmall), &brand);
    }
}
