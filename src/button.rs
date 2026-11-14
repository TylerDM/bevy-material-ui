//! Buttons, icon buttons and floating action buttons.
//!
//! Each is Bevy's headless [`bevy::ui_widgets::Button`] drawn as Material's:
//! activated by a click, or by Enter or Space while it has the keyboard's
//! focus, sending [`bevy::ui_widgets::Activate`] - which the game observes
//! on the entity, with an observer or with its own component beside the
//! button's bundle.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Button;

use crate::focus::FocusRing;
use crate::icon::Icon;
use crate::paint::{Look, Stroke, label};
use crate::theme::{ColorRole, Corner, Elevation, TypeRole};

/// How prominent a button is - which is its colour: Material's five.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum ButtonVariant {
    /// The action a screen is built around: primary, filled.
    #[default]
    Filled,
    /// A supporting action, between filled and outlined.
    Tonal,
    /// A filled button that must stand off a busy background.
    Elevated,
    /// Important, but not the main action: an outline, no fill.
    Outlined,
    /// The least prominent: no container, as in a dialog's actions.
    Text,
}

impl ButtonVariant {
    /// Every variant, most prominent first.
    pub const ALL: [ButtonVariant; 5] = [
        ButtonVariant::Filled,
        ButtonVariant::Tonal,
        ButtonVariant::Elevated,
        ButtonVariant::Outlined,
        ButtonVariant::Text,
    ];

    /// Its look at rest.
    pub fn look(self) -> Look {
        match self {
            ButtonVariant::Filled => Look::filled(ColorRole::Primary, ColorRole::OnPrimary),
            ButtonVariant::Tonal => Look::filled(
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            ),
            ButtonVariant::Elevated => {
                Look::filled(ColorRole::SurfaceContainerLow, ColorRole::Primary)
                    .raised(Elevation::Level1)
            }
            ButtonVariant::Outlined | ButtonVariant::Text => Look::bare(ColorRole::Primary),
        }
    }
}

/// A button's height, in logical pixels.
pub const BUTTON_HEIGHT: f32 = 40.0;

/// A button: a label, a variant, an icon before the label, enabled or not.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialButton {
    /// What it says.
    pub label: String,
    /// How prominent it is.
    pub variant: ButtonVariant,
    /// An icon before its label.
    pub icon: Option<Icon>,
    /// Whether it can be used.
    pub enabled: bool,
}

/// A filled button saying `label`; change its variant with the builder.
pub fn button(label: impl Into<String>) -> MaterialButton {
    MaterialButton {
        label: label.into(),
        variant: ButtonVariant::Filled,
        icon: None,
        enabled: true,
    }
}

impl MaterialButton {
    /// Of `variant`.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Tonal.
    pub fn tonal(self) -> Self {
        self.variant(ButtonVariant::Tonal)
    }

    /// Elevated.
    pub fn elevated(self) -> Self {
        self.variant(ButtonVariant::Elevated)
    }

    /// Outlined.
    pub fn outlined(self) -> Self {
        self.variant(ButtonVariant::Outlined)
    }

    /// A text button.
    pub fn text(self) -> Self {
        self.variant(ButtonVariant::Text)
    }

    /// With `icon` before its label.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Its components: spawn it, with the game's own action beside it.
    pub fn bundle(self) -> impl Bundle {
        let outlined = self.variant == ButtonVariant::Outlined;
        let text = self.variant == ButtonVariant::Text;
        let (start, end) = match (&self.icon, text) {
            (Some(_), true) => (12.0, 16.0),
            (None, true) => (12.0, 12.0),
            (Some(_), false) => (16.0, 24.0),
            (None, false) => (24.0, 24.0),
        };
        let icon = self.icon;
        let words = self.label;
        (
            (
                Button,
                self.variant,
                self.variant.look(),
                FocusRing,
                TabIndex(0),
                Stroke::new(ColorRole::Outline),
                BoxShadow::default(),
                enabled(self.enabled),
            ),
            Node {
                height: Val::Px(BUTTON_HEIGHT),
                min_width: Val::Px(48.0),
                padding: UiRect::new(Val::Px(start), Val::Px(end), Val::ZERO, Val::ZERO),
                column_gap: Val::Px(8.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                border: UiRect::all(Val::Px(if outlined { 1.0 } else { 0.0 })),
                border_radius: Corner::Full.all(),
                ..default()
            },
            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                if let Some(glyph) = icon {
                    parent.spawn(crate::icon::icon(glyph, 18.0));
                }
                parent.spawn(label(words, TypeRole::LabelLarge));
            })),
        )
    }
}

/// Whether a control starts out usable: one that doesn't is given
/// `InteractionDisabled` as it is spawned. Turn it on and off afterwards
/// with [`crate::paint::set_enabled`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StartsEnabled(pub(crate) bool);

/// A bundle starting a control enabled or not.
pub(crate) fn enabled(enabled: bool) -> impl Bundle {
    StartsEnabled(enabled)
}

pub(crate) fn on_starts_enabled(
    add: On<Add<StartsEnabled>>,
    controls: Query<&StartsEnabled>,
    mut commands: Commands,
) {
    let Ok(StartsEnabled(enabled)) = controls.get(add.entity) else {
        return;
    };
    let mut control = commands.entity(add.entity);
    control.try_remove::<StartsEnabled>();
    if !enabled {
        control.try_insert(InteractionDisabled);
    }
}

/// Material's four kinds of icon button.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum IconButtonVariant {
    /// No container: the least prominent, in a toolbar or an app bar.
    #[default]
    Standard,
    /// Filled with primary.
    Filled,
    /// Filled with the secondary container.
    Tonal,
    /// An outline, no fill.
    Outlined,
}

impl IconButtonVariant {
    /// Every variant.
    pub const ALL: [IconButtonVariant; 4] = [
        IconButtonVariant::Standard,
        IconButtonVariant::Filled,
        IconButtonVariant::Tonal,
        IconButtonVariant::Outlined,
    ];

    /// Its look at rest.
    pub fn look(self) -> Look {
        match self {
            IconButtonVariant::Standard | IconButtonVariant::Outlined => {
                Look::bare(ColorRole::OnSurfaceVariant)
            }
            IconButtonVariant::Filled => Look::filled(ColorRole::Primary, ColorRole::OnPrimary),
            IconButtonVariant::Tonal => Look::filled(
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            ),
        }
    }
}

/// An icon button's side, in logical pixels.
pub const ICON_BUTTON: f32 = 40.0;

/// An icon button showing `icon`, of `variant`.
pub fn icon_button(icon: impl Into<Icon>, variant: IconButtonVariant) -> impl Bundle {
    let icon = icon.into();
    (
        (
            Button,
            variant,
            variant.look(),
            FocusRing,
            TabIndex(0),
            Stroke::new(ColorRole::OutlineVariant),
        ),
        Node {
            width: Val::Px(ICON_BUTTON),
            height: Val::Px(ICON_BUTTON),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            border: UiRect::all(Val::Px(if variant == IconButtonVariant::Outlined {
                1.0
            } else {
                0.0
            })),
            border_radius: Corner::Full.all(),
            ..default()
        },
        children![crate::icon::icon(icon, crate::icon::ICON)],
    )
}

/// The three sizes of floating action button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum FabSize {
    /// 40 px.
    Small,
    /// 56 px.
    #[default]
    Regular,
    /// 96 px.
    Large,
}

impl FabSize {
    /// Its side, its icon's side and its corners.
    pub fn metrics(self) -> (f32, f32, Corner) {
        match self {
            FabSize::Small => (40.0, 24.0, Corner::Medium),
            FabSize::Regular => (56.0, 24.0, Corner::Large),
            FabSize::Large => (96.0, 36.0, Corner::ExtraLarge),
        }
    }
}

/// Marks a floating action button.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Fab;

/// A floating action button: the screen's main action, standing above it.
/// `label` makes it an extended FAB, its words after its icon.
pub fn fab(icon: impl Into<Icon>, size: FabSize, words: Option<&str>) -> impl Bundle {
    let icon = icon.into();
    let (side, icon_side, corner) = size.metrics();
    let words = words.map(str::to_owned);
    let extended = words.is_some();
    (
        (
            Button,
            Fab,
            Look::filled(ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer)
                .raised(Elevation::Level3),
            FocusRing,
            TabIndex(0),
            BoxShadow::default(),
        ),
        Node {
            min_width: Val::Px(side),
            height: Val::Px(if extended { 56.0 } else { side }),
            padding: UiRect::horizontal(Val::Px(if extended { 16.0 } else { 0.0 })),
            column_gap: Val::Px(12.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            border_radius: if extended { Corner::Large } else { corner }.all(),
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn(crate::icon::icon(icon, icon_side));
            if let Some(words) = words {
                parent.spawn(label(words, TypeRole::LabelLarge));
            }
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buttons_variant_is_its_colour() {
        assert_eq!(
            ButtonVariant::Filled.look().container,
            Some(ColorRole::Primary)
        );
        assert_eq!(
            ButtonVariant::Tonal.look().container,
            Some(ColorRole::SecondaryContainer)
        );
        assert_eq!(
            ButtonVariant::Elevated.look().elevation,
            Some(Elevation::Level1)
        );
        assert_eq!(ButtonVariant::Outlined.look().container, None);
        assert_eq!(ButtonVariant::Text.look().content, ColorRole::Primary);
    }

    #[test]
    fn the_three_fabs_grow_and_round_with_their_size() {
        let sides: Vec<f32> = [FabSize::Small, FabSize::Regular, FabSize::Large]
            .iter()
            .map(|size| size.metrics().0)
            .collect();
        assert_eq!(sides, [40.0, 56.0, 96.0]);
    }
}
