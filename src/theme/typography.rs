//! Material's type scale: fifteen kinds of text, each a size, a line height,
//! a weight and a tracking.

use bevy::prelude::*;
use bevy::text::FontWeight;

/// A kind of text, as Material 3 names them, largest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum TypeRole {
    /// The largest text on the screen: short, important numbers or words.
    DisplayLarge,
    /// A smaller display.
    DisplayMedium,
    /// The smallest display.
    DisplaySmall,
    /// High-emphasis text on smaller screens: a page's title.
    HeadlineLarge,
    /// A medium headline.
    HeadlineMedium,
    /// A small headline: a dialog's title.
    HeadlineSmall,
    /// A top app bar's title.
    TitleLarge,
    /// A card's or a list's title.
    TitleMedium,
    /// A small title: a tab's, a section's.
    TitleSmall,
    /// Running text, at length.
    BodyLarge,
    /// The default body text.
    BodyMedium,
    /// Small body text: supporting text, captions.
    BodySmall,
    /// Inside controls: buttons, tabs, chips.
    LabelLarge,
    /// A navigation item's label, a badge's.
    LabelMedium,
    /// The smallest label: timestamps, a field's counter.
    LabelSmall,
}

impl TypeRole {
    /// Every kind, largest first.
    pub const ALL: [TypeRole; 15] = [
        TypeRole::DisplayLarge,
        TypeRole::DisplayMedium,
        TypeRole::DisplaySmall,
        TypeRole::HeadlineLarge,
        TypeRole::HeadlineMedium,
        TypeRole::HeadlineSmall,
        TypeRole::TitleLarge,
        TypeRole::TitleMedium,
        TypeRole::TitleSmall,
        TypeRole::BodyLarge,
        TypeRole::BodyMedium,
        TypeRole::BodySmall,
        TypeRole::LabelLarge,
        TypeRole::LabelMedium,
        TypeRole::LabelSmall,
    ];

    /// Its size and its line's height, in logical pixels.
    pub fn metrics(self) -> (f32, f32) {
        match self {
            TypeRole::DisplayLarge => (57.0, 64.0),
            TypeRole::DisplayMedium => (45.0, 52.0),
            TypeRole::DisplaySmall => (36.0, 44.0),
            TypeRole::HeadlineLarge => (32.0, 40.0),
            TypeRole::HeadlineMedium => (28.0, 36.0),
            TypeRole::HeadlineSmall => (24.0, 32.0),
            TypeRole::TitleLarge => (22.0, 28.0),
            TypeRole::TitleMedium | TypeRole::BodyLarge => (16.0, 24.0),
            TypeRole::TitleSmall | TypeRole::BodyMedium | TypeRole::LabelLarge => (14.0, 20.0),
            TypeRole::BodySmall | TypeRole::LabelMedium => (12.0, 16.0),
            TypeRole::LabelSmall => (11.0, 16.0),
        }
    }

    /// Its size, in logical pixels.
    pub fn size(self) -> f32 {
        self.metrics().0
    }

    /// Its line's height, in logical pixels.
    pub fn line_height(self) -> f32 {
        self.metrics().1
    }

    /// Whether it is set in the medium weight rather than the regular.
    pub fn is_medium(self) -> bool {
        matches!(
            self,
            TypeRole::TitleMedium
                | TypeRole::TitleSmall
                | TypeRole::LabelLarge
                | TypeRole::LabelMedium
                | TypeRole::LabelSmall
        )
    }

    /// Whether it is set in the brand face (display, headline, title large)
    /// rather than the plain one.
    pub fn is_brand(self) -> bool {
        matches!(
            self,
            TypeRole::DisplayLarge
                | TypeRole::DisplayMedium
                | TypeRole::DisplaySmall
                | TypeRole::HeadlineLarge
                | TypeRole::HeadlineMedium
                | TypeRole::HeadlineSmall
                | TypeRole::TitleLarge
        )
    }

    /// Its weight.
    pub fn weight(self) -> FontWeight {
        if self.is_medium() {
            FontWeight::MEDIUM
        } else {
            FontWeight::NORMAL
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_of_text_shrinks_from_large_to_small_and_display_is_largest() {
        assert!(
            TypeRole::ALL
                .iter()
                .all(|role| role.size() <= TypeRole::DisplayLarge.size())
        );
        for pair in TypeRole::ALL.chunks(3).flat_map(|group| group.windows(2)) {
            assert!(
                pair[0].size() >= pair[1].size(),
                "{:?} is smaller than {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn every_line_is_taller_than_its_text_and_labels_are_medium() {
        for role in TypeRole::ALL {
            assert!(role.line_height() > role.size(), "{role:?}");
        }
        assert_eq!(TypeRole::LabelLarge.weight(), FontWeight::MEDIUM);
        assert_eq!(TypeRole::BodyLarge.weight(), FontWeight::NORMAL);
        assert!(TypeRole::HeadlineSmall.is_brand() && !TypeRole::BodyLarge.is_brand());
    }
}
