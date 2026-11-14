//! How parts are coloured and set from the theme: each names a role - a
//! colour, a kind of text, a level - and is repainted from
//! [`MaterialTheme`] when it is added and whenever the theme changes.
//!
//! - [`Fill`], [`Stroke`] and [`Ink`] give a part's background, border and
//!   text a colour role; [`Typeset`] sets its text as a kind of text;
//!   [`Raised`] lifts it to a level of elevation.
//! - A control's colours depend on its state, so a control carries a
//!   [`Look`] instead: its container and content roles, drawn with
//!   Material's state layers as it is hovered, pressed or disabled.
//! - What is inside a control - its label, its icon - carries [`Content`],
//!   and is drawn in the content colour of the nearest [`Look`] or
//!   [`ContentColor`] above it, faded with it when the control is disabled.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};

use crate::theme::{ColorRole, Elevation, MaterialTheme, TypeRole, layered, state};

/// A colour role at an opacity.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Paint {
    /// The role.
    pub role: ColorRole,
    /// How opaque it is drawn, 0 to 1.
    pub opacity: f32,
}

impl Paint {
    /// `role`, opaque.
    pub const fn new(role: ColorRole) -> Self {
        Self { role, opacity: 1.0 }
    }

    /// `role` at `opacity`.
    pub const fn faded(role: ColorRole, opacity: f32) -> Self {
        Self { role, opacity }
    }

    /// Its colour in `theme`.
    pub fn color(self, theme: &MaterialTheme) -> Color {
        let color = theme.color(self.role);
        color.with_alpha(color.alpha() * self.opacity)
    }
}

impl From<ColorRole> for Paint {
    fn from(role: ColorRole) -> Self {
        Self::new(role)
    }
}

/// A part's background, in a colour role.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[component(immutable)]
#[require(BackgroundColor)]
#[reflect(Component)]
pub struct Fill(pub Paint);

impl Fill {
    /// Filled with `role`.
    pub fn new(role: ColorRole) -> Self {
        Self(Paint::new(role))
    }
}

/// A part's border, in a colour role. Its width is its `Node`'s.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[component(immutable)]
#[require(BorderColor)]
#[reflect(Component)]
pub struct Stroke(pub Paint);

impl Stroke {
    /// Bordered with `role`.
    pub fn new(role: ColorRole) -> Self {
        Self(Paint::new(role))
    }
}

/// A text's colour, as a colour role.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[component(immutable)]
#[require(TextColor)]
#[reflect(Component)]
pub struct Ink(pub Paint);

impl Ink {
    /// Text in `role`.
    pub fn new(role: ColorRole) -> Self {
        Self(Paint::new(role))
    }
}

/// A text's face, size, weight and line height, as a kind of text.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[component(immutable)]
#[require(TextFont)]
#[reflect(Component)]
pub struct Typeset(pub TypeRole);

/// A part lifted to a level of elevation: a shadow under it.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[component(immutable)]
#[require(BoxShadow)]
#[reflect(Component)]
pub struct Raised(pub Elevation);

/// A control's colours at rest, from which its states are drawn: hovered
/// and pressed as Material's state layers - its content laid over its
/// container - and disabled faded to `OnSurface`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[require(BackgroundColor, Hovered)]
#[reflect(Component)]
pub struct Look {
    /// What it is filled with; none for a control with no container, whose
    /// states show as a wash of its content.
    pub container: Option<ColorRole>,
    /// Its label's and icon's colour.
    pub content: ColorRole,
    /// How high it stands at rest, if it casts a shadow; it rises a level
    /// while hovered.
    pub elevation: Option<Elevation>,
}

impl Look {
    /// A control filled with `container`, its content in `content`.
    pub const fn filled(container: ColorRole, content: ColorRole) -> Self {
        Self {
            container: Some(container),
            content,
            elevation: None,
        }
    }

    /// A control with no container, its content in `content`.
    pub const fn bare(content: ColorRole) -> Self {
        Self {
            container: None,
            content,
            elevation: None,
        }
    }

    /// This look standing at `elevation`.
    pub const fn raised(mut self, elevation: Elevation) -> Self {
        self.elevation = Some(elevation);
        self
    }

    /// Its container's colour in `theme`, in a state.
    pub fn container_color(
        self,
        theme: &MaterialTheme,
        hovered: bool,
        pressed: bool,
        disabled: bool,
    ) -> Color {
        if disabled {
            return match self.container {
                Some(_) => theme
                    .color(ColorRole::OnSurface)
                    .with_alpha(state::DISABLED_CONTAINER),
                None => Color::NONE,
            };
        }
        let under = self.container.map_or(Color::NONE, |role| theme.color(role));
        let opacity = if pressed {
            state::PRESSED
        } else if hovered {
            state::HOVERED
        } else {
            return under;
        };
        layered(under, theme.color(self.content), opacity)
    }

    /// Its content's colour in `theme`.
    pub fn content_color(self, theme: &MaterialTheme, disabled: bool) -> Color {
        if disabled {
            theme
                .color(ColorRole::OnSurface)
                .with_alpha(state::DISABLED_CONTENT)
        } else {
            theme.color(self.content)
        }
    }

    /// Its shadow in a state.
    pub fn shadow(self, theme: &MaterialTheme, hovered: bool, disabled: bool) -> Option<BoxShadow> {
        let level = self.elevation?;
        let level = if disabled {
            Elevation::Level0
        } else if hovered {
            level.raised()
        } else {
            level
        };
        Some(level.shadow(theme.color(ColorRole::Shadow)))
    }
}

/// The content colour of what is inside a part that is not a control: an
/// icon on a card, the words of a snackbar.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[component(immutable)]
#[reflect(Component)]
pub struct ContentColor(pub Paint);

/// A label, an icon or a part of one, drawn in the content colour of the
/// nearest [`Look`] or [`ContentColor`] at or above it: its text if it has
/// text, its image's tint if it is an image, otherwise its background.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Content;

/// Words of a kind of text, in a colour role.
pub fn text(words: impl Into<String>, role: TypeRole, color: ColorRole) -> impl Bundle {
    (Text::new(words), Typeset(role), Ink::new(color))
}

/// Words of a kind of text in the content colour of what they are in: a
/// control's label.
pub fn label(words: impl Into<String>, role: TypeRole) -> impl Bundle {
    (Text::new(words), Typeset(role), Content, Pickable::IGNORE)
}

/// Turns a control on or off: one turned off is drawn faded and nothing can
/// use it.
pub fn set_enabled(control: &mut EntityCommands, enabled: bool) {
    if enabled {
        control.remove::<InteractionDisabled>();
    } else {
        control.insert(InteractionDisabled);
    }
}

pub(crate) fn on_fill(
    insert: On<Insert<Fill>>,
    theme: Res<MaterialTheme>,
    mut parts: Query<(&Fill, &mut BackgroundColor)>,
) {
    if let Ok((fill, mut background)) = parts.get_mut(insert.entity) {
        background.0 = fill.0.color(&theme);
    }
}

pub(crate) fn on_stroke(
    insert: On<Insert<Stroke>>,
    theme: Res<MaterialTheme>,
    mut parts: Query<(&Stroke, &mut BorderColor)>,
) {
    if let Ok((stroke, mut border)) = parts.get_mut(insert.entity) {
        *border = BorderColor::all(stroke.0.color(&theme));
    }
}

pub(crate) fn on_ink(
    insert: On<Insert<Ink>>,
    theme: Res<MaterialTheme>,
    mut parts: Query<(&Ink, &mut TextColor)>,
) {
    if let Ok((ink, mut color)) = parts.get_mut(insert.entity) {
        color.0 = ink.0.color(&theme);
    }
}

pub(crate) fn on_typeset(
    insert: On<Insert<Typeset>>,
    theme: Res<MaterialTheme>,
    mut parts: Query<(&Typeset, &mut TextFont)>,
    mut commands: Commands,
) {
    if let Ok((typeset, mut font)) = parts.get_mut(insert.entity) {
        *font = theme.font(typeset.0);
        commands
            .entity(insert.entity)
            .insert(MaterialTheme::line_height(typeset.0));
    }
}

pub(crate) fn on_raised(
    insert: On<Insert<Raised>>,
    theme: Res<MaterialTheme>,
    mut parts: Query<(&Raised, &mut BoxShadow)>,
) {
    if let Ok((raised, mut shadow)) = parts.get_mut(insert.entity) {
        *shadow = raised.0.shadow(theme.color(ColorRole::Shadow));
    }
}

/// Repaints every themed part when the theme changes.
pub(crate) fn repaint_on_theme_change(
    theme: Res<MaterialTheme>,
    mut fills: Query<(&Fill, &mut BackgroundColor), Without<Look>>,
    mut strokes: Query<(&Stroke, &mut BorderColor)>,
    mut inks: Query<(&Ink, &mut TextColor), Without<Content>>,
    mut typesets: Query<(&Typeset, &mut TextFont)>,
    mut raised: Query<(&Raised, &mut BoxShadow), Without<Look>>,
) {
    if !theme.is_changed() || theme.is_added() {
        return;
    }
    for (fill, mut background) in &mut fills {
        background.0 = fill.0.color(&theme);
    }
    for (stroke, mut border) in &mut strokes {
        *border = BorderColor::all(stroke.0.color(&theme));
    }
    for (ink, mut color) in &mut inks {
        color.0 = ink.0.color(&theme);
    }
    for (typeset, mut font) in &mut typesets {
        *font = theme.font(typeset.0);
    }
    for (level, mut shadow) in &mut raised {
        *shadow = level.0.shadow(theme.color(ColorRole::Shadow));
    }
}

/// A control whose drawn state may have changed this frame.
type ControlChanged = Or<(
    Changed<Hovered>,
    Changed<Look>,
    Added<Pressed>,
    Added<InteractionDisabled>,
)>;

/// A control's state, and what it is drawn with.
type ControlState<'a> = (
    &'a Look,
    &'a Hovered,
    Has<Pressed>,
    Has<InteractionDisabled>,
    &'a mut BackgroundColor,
    Option<&'a mut BoxShadow>,
);

/// Draws each control's container in its state: hovered and pressed as
/// state layers, disabled faded, and its shadow risen while hovered.
pub(crate) fn paint_controls(
    theme: Res<MaterialTheme>,
    mut released: RemovedComponents<Pressed>,
    mut enabled: RemovedComponents<InteractionDisabled>,
    mut controls: ParamSet<(Query<ControlState, ControlChanged>, Query<ControlState>)>,
) {
    let paint = |(look, hovered, pressed, disabled, mut background, shadow): (
        &Look,
        &Hovered,
        bool,
        bool,
        Mut<BackgroundColor>,
        Option<Mut<BoxShadow>>,
    )| {
        background.set_if_neq(BackgroundColor(
            look.container_color(&theme, hovered.0, pressed, disabled),
        ));
        if let (Some(mut drawn), Some(wanted)) = (shadow, look.shadow(&theme, hovered.0, disabled))
            && *drawn != wanted
        {
            *drawn = wanted;
        }
    };
    if theme.is_changed() {
        for control in &mut controls.p1() {
            paint(control);
        }
        return;
    }
    for control in &mut controls.p0() {
        paint(control);
    }
    let left: Vec<Entity> = released.read().chain(enabled.read()).collect();
    let mut all = controls.p1();
    for entity in left {
        if let Ok(control) = all.get_mut(entity) {
            paint(control);
        }
    }
}

/// A part whose content's colour may have changed this frame.
type ProviderChanged = Or<(
    Changed<Look>,
    Changed<ContentColor>,
    Added<InteractionDisabled>,
)>;

/// What a [`Content`] may be coloured through: its text, its image, its background.
type LeafColors<'a> = (
    Option<&'a mut TextColor>,
    Option<&'a mut ImageNode>,
    Option<&'a mut BackgroundColor>,
);

/// Draws each [`Content`] in the content colour of the nearest [`Look`] or
/// [`ContentColor`] at or above it.
#[expect(clippy::too_many_arguments, reason = "a system's queries")]
pub(crate) fn paint_content(
    theme: Res<MaterialTheme>,
    added: Query<Entity, Added<Content>>,
    changed: Query<Entity, ProviderChanged>,
    mut enabled: RemovedComponents<InteractionDisabled>,
    all: Query<Entity, With<Content>>,
    providers: Query<(
        Option<&Look>,
        Option<&ContentColor>,
        Has<InteractionDisabled>,
    )>,
    parents: Query<&ChildOf>,
    mut leaves: Query<LeafColors, With<Content>>,
) {
    let provider_changed = !changed.is_empty() || enabled.read().count() > 0;
    let targets: Vec<Entity> = if theme.is_changed() || provider_changed {
        all.iter().collect()
    } else {
        added.iter().collect()
    };
    for leaf in targets {
        let Some(color) = content_color(leaf, &theme, &providers, &parents) else {
            continue;
        };
        let Ok((text, image, background)) = leaves.get_mut(leaf) else {
            continue;
        };
        if let Some(mut text) = text {
            text.set_if_neq(TextColor(color));
        } else if let Some(mut image) = image {
            if image.color != color {
                image.color = color;
            }
        } else if let Some(mut background) = background {
            background.set_if_neq(BackgroundColor(color));
        }
    }
}

/// The content colour `leaf` takes: that of the nearest provider at or
/// above it, faded if a control on the way is disabled.
fn content_color(
    leaf: Entity,
    theme: &MaterialTheme,
    providers: &Query<(
        Option<&Look>,
        Option<&ContentColor>,
        Has<InteractionDisabled>,
    )>,
    parents: &Query<&ChildOf>,
) -> Option<Color> {
    let mut at = leaf;
    loop {
        if let Ok((look, content, disabled)) = providers.get(at) {
            if let Some(look) = look {
                return Some(look.content_color(theme, disabled));
            }
            if let Some(content) = content {
                return Some(content.0.color(theme));
            }
        }
        at = parents.get(at).ok()?.parent();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ColorRole, MaterialTheme};

    #[test]
    fn a_hovered_or_pressed_control_is_tinted_by_its_content_and_a_disabled_one_faded() {
        let theme = MaterialTheme::light();
        let look = Look::filled(ColorRole::Primary, ColorRole::OnPrimary);
        let rest = look.container_color(&theme, false, false, false);
        let hovered = look.container_color(&theme, true, false, false);
        let pressed = look.container_color(&theme, true, true, false);
        assert_eq!(rest, theme.color(ColorRole::Primary));
        assert_ne!(hovered, rest);
        assert!(
            pressed.to_srgba().red > hovered.to_srgba().red,
            "pressed is tinted further toward white than hovered"
        );
        let disabled = look.container_color(&theme, false, false, true);
        assert!((disabled.alpha() - state::DISABLED_CONTAINER).abs() < 1e-4);
        let content = look.content_color(&theme, true);
        assert!((content.alpha() - state::DISABLED_CONTENT).abs() < 1e-4);
    }

    #[test]
    fn a_control_with_no_container_shows_its_state_as_a_wash_of_its_content() {
        let theme = MaterialTheme::light();
        let look = Look::bare(ColorRole::Primary);
        assert_eq!(
            look.container_color(&theme, false, false, false),
            Color::NONE
        );
        let hovered = look.container_color(&theme, true, false, false);
        assert!((hovered.alpha() - state::HOVERED).abs() < 1e-4);
        assert_eq!(look.container_color(&theme, true, false, true), Color::NONE);
    }

    #[test]
    fn a_raised_control_rises_a_level_when_hovered_and_lies_flat_when_disabled() {
        let theme = MaterialTheme::light();
        let look = Look::filled(ColorRole::SurfaceContainerLow, ColorRole::Primary)
            .raised(Elevation::Level1);
        let shadow = |level: Elevation| level.shadow(theme.color(ColorRole::Shadow));
        assert_eq!(
            look.shadow(&theme, false, false),
            Some(shadow(Elevation::Level1))
        );
        assert_eq!(
            look.shadow(&theme, true, false),
            Some(shadow(Elevation::Level2))
        );
        assert_eq!(
            look.shadow(&theme, true, true),
            Some(shadow(Elevation::Level0))
        );
        assert_eq!(
            Look::bare(ColorRole::Primary).shadow(&theme, true, false),
            None
        );
    }
}
