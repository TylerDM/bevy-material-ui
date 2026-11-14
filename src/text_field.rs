//! Text fields: Material 3's filled and outlined fields over Bevy's headless
//! [`TextInput`].
//!
//! A field's label sits in it while it is empty and unfocused, and floats
//! small above the text once it is focused or filled. Below it go its
//! supporting text - or its error, in the error colours - and a counter
//! when it has a most it may hold. A clear button empties it.
//!
//! The game spawns the field and observes [`ValueChange<String>`] on the
//! entity it spawned: one as each edit changes the text (`is_final` false),
//! and a final one when Enter is pressed or the field loses the keyboard. It
//! reads the text from the field's [`InputText`], and changes it, its error
//! or its supporting text through [`TextField`] and [`set_input_text`].

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{AcquireFocus, FocusCause, FocusLost, FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle, TextEditChange};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Activate, TextInput, ValueChange};

use crate::button::{IconButtonVariant, icon_button};
use crate::icon::{Glyph, ICON, Icon, icon};
use crate::paint::{ContentColor, Fill, Ink, Paint, Stroke, Typeset};
use crate::theme::{ColorRole, Corner, MaterialTheme, TypeRole, state};

/// A field's height, in logical pixels.
pub const FIELD_HEIGHT: f32 = 56.0;

/// What a text input holds, kept on the entity the game spawned - a text
/// field or a search bar - beside the input inside it that edits it.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct InputText {
    text: String,
    input: Entity,
}

impl Default for InputText {
    fn default() -> Self {
        Self::new(Entity::PLACEHOLDER)
    }
}

impl InputText {
    /// Empty, edited by `input`.
    pub(crate) fn new(input: Entity) -> Self {
        Self {
            text: String::new(),
            input,
        }
    }

    /// The text it holds now.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The input inside the part that edits it: Bevy's [`TextInput`], which
    /// the keyboard's focus goes to.
    pub fn input(&self) -> Entity {
        self.input
    }
}

/// On a [`TextInput`]: the part - field or search bar - whose text it edits.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InputOf(pub(crate) Entity);

/// On a clear button: the input it empties.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Clears(pub(crate) Entity);

/// Replaces the text of a text field or search bar, `part` - the entity the
/// game spawned - as if it had been typed: its change is sent as a
/// [`ValueChange<String>`].
pub fn set_input_text(commands: &mut Commands, part: Entity, text: impl Into<String>) {
    let text = text.into();
    commands.queue(move |world: &mut World| {
        let Some(input) = world.get::<InputText>(part).map(InputText::input) else {
            return;
        };
        if let Some(mut editable) = world.get_mut::<EditableText>(input) {
            editable.editor.set_text(&text);
            editable.queue_edit(bevy::text::TextEdit::TextEnd(false));
        }
    });
}

/// Material's two kinds of text field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum TextFieldVariant {
    /// On the highest container, with a line under it: the more prominent.
    #[default]
    Filled,
    /// Transparent, with an outline: for a form of many fields.
    Outlined,
}

/// A text field, as the game sees and changes it once it is spawned.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct TextField {
    /// Filled or outlined.
    pub variant: TextFieldVariant,
    /// Supporting text under the field: a hint of what it takes.
    pub supporting: Option<String>,
    /// An error: the field is drawn in the error colours, and this is shown
    /// in place of its supporting text.
    pub error: Option<String>,
    /// The most characters it may hold; a counter shows under it.
    pub max_length: Option<usize>,
    /// Whether it can be used.
    pub enabled: bool,
    /// The colour role of the surface an outlined field sits on, drawn behind
    /// its floated label where the label crosses the outline.
    pub surface: ColorRole,
    drawn: Option<FieldState>,
}

impl TextField {
    /// Whether it is in error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// The parts of a text field the drawing changes.
#[derive(Component, Clone, Copy, Debug)]
struct FieldParts {
    frame: Entity,
    label: Option<Entity>,
    placeholder: Option<Entity>,
    below: Entity,
    supporting: Entity,
    counter: Entity,
    clear: Option<Entity>,
    leading: Option<Entity>,
}

/// What a field is drawn as: everything its look depends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FieldState {
    /// It holds the keyboard's focus.
    pub(crate) focused: bool,
    /// The pointer is over it.
    pub(crate) hovered: bool,
    /// It holds no text.
    pub(crate) empty: bool,
    /// It is in error.
    pub(crate) error: bool,
    /// It can't be used.
    pub(crate) disabled: bool,
}

impl FieldState {
    /// Whether its label floats above the text: focused or filled.
    pub(crate) fn floated(self) -> bool {
        self.focused || !self.empty
    }

    /// The colour of the line under a filled field, or an outlined field's
    /// outline, and its width.
    pub(crate) fn indicator(self, variant: TextFieldVariant) -> (Paint, f32) {
        let width = if self.focused && !self.disabled {
            2.0
        } else {
            1.0
        };
        let role = if self.disabled {
            return (
                Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT),
                width,
            );
        } else if self.error {
            ColorRole::Error
        } else if self.focused {
            ColorRole::Primary
        } else if self.hovered {
            ColorRole::OnSurface
        } else if variant == TextFieldVariant::Filled {
            ColorRole::OnSurfaceVariant
        } else {
            ColorRole::Outline
        };
        (Paint::new(role), width)
    }

    /// The colour of its label.
    pub(crate) fn label(self) -> Paint {
        if self.disabled {
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)
        } else if self.error {
            Paint::new(ColorRole::Error)
        } else if self.focused {
            Paint::new(ColorRole::Primary)
        } else {
            Paint::new(ColorRole::OnSurfaceVariant)
        }
    }

    /// The colour of the words under it: supporting text, error, counter.
    pub(crate) fn below(self) -> Paint {
        if self.disabled {
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)
        } else if self.error {
            Paint::new(ColorRole::Error)
        } else {
            Paint::new(ColorRole::OnSurfaceVariant)
        }
    }
}

/// A counter's words: how many characters of how many.
pub fn counter_text(count: usize, max: usize) -> String {
    format!("{count}/{max}")
}

/// A text field, as it is spawned.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialTextField {
    variant: TextFieldVariant,
    label: Option<String>,
    text: String,
    placeholder: Option<String>,
    supporting: Option<String>,
    error: Option<String>,
    max_length: Option<usize>,
    leading: Option<Icon>,
    clearable: bool,
    enabled: bool,
    width: Val,
    surface: ColorRole,
}

/// A filled text field labelled `label`; change it with the builder.
pub fn text_field(label: impl Into<String>) -> MaterialTextField {
    let label = label.into();
    MaterialTextField {
        variant: TextFieldVariant::Filled,
        label: (!label.is_empty()).then_some(label),
        text: String::new(),
        placeholder: None,
        supporting: None,
        error: None,
        max_length: None,
        leading: None,
        clearable: false,
        enabled: true,
        width: Val::Px(280.0),
        surface: ColorRole::Surface,
    }
}

impl MaterialTextField {
    /// Of `variant`.
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Outlined.
    pub fn outlined(self) -> Self {
        self.variant(TextFieldVariant::Outlined)
    }

    /// Holding `text` to begin with.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Showing `placeholder` while empty and focused, or always while empty if
    /// it has no label.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// With `supporting` text under it.
    pub fn supporting(mut self, supporting: impl Into<String>) -> Self {
        self.supporting = Some(supporting.into());
        self
    }

    /// In error, saying `error` under it.
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Holding at most `max` characters, with a counter under it.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// With `icon` before its text.
    pub fn leading_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.leading = Some(icon.into());
        self
    }

    /// With a clear button after its text while it holds any.
    pub fn clearable(mut self) -> Self {
        self.clearable = true;
        self
    }

    /// Enabled, or drawn faded and unusable.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// `width` wide; 280 px unless set.
    pub fn width(mut self, width: Val) -> Self {
        self.width = width;
        self
    }

    /// For an outlined field: on a surface of `surface`, which shows behind
    /// its floated label.
    pub fn on_surface(mut self, surface: ColorRole) -> Self {
        self.surface = surface;
        self
    }

    /// Its components: spawn it, with the game's own components beside it.
    pub fn bundle(self) -> impl Bundle {
        let field = TextField {
            variant: self.variant,
            supporting: self.supporting.clone(),
            error: self.error.clone(),
            max_length: self.max_length,
            enabled: self.enabled,
            surface: self.surface,
            drawn: None,
        };
        (
            field,
            InputText {
                text: self.text.clone(),
                input: Entity::PLACEHOLDER,
            },
            Node {
                width: self.width,
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                ..default()
            },
            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                spawn_parts(parent, self);
            })),
        )
    }
}

/// Spawns a field's parts under it and records them on it.
fn spawn_parts(parent: &mut ChildSpawner, spec: MaterialTextField) {
    let root = parent.target_entity();
    let outlined = spec.variant == TextFieldVariant::Outlined;
    let has_label = spec.label.is_some();
    let start = if spec.leading.is_some() { 12.0 } else { 16.0 };
    let end = if spec.clearable { 4.0 } else { 16.0 };
    let text_top = if outlined || !has_label { 16.0 } else { 24.0 };

    let mut frame_parts = (None, None, None, None, Entity::PLACEHOLDER);
    let frame = parent
        .spawn((
            Node {
                height: Val::Px(FIELD_HEIGHT),
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(start), Val::Px(end), Val::ZERO, Val::ZERO),
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                border: if outlined {
                    UiRect::all(Val::Px(1.0))
                } else {
                    UiRect::bottom(Val::Px(1.0))
                },
                border_radius: if outlined {
                    Corner::ExtraSmall.all()
                } else {
                    Corner::ExtraSmall.top()
                },
                ..default()
            },
            if outlined {
                Fill(Paint::faded(ColorRole::Surface, 0.0))
            } else {
                Fill::new(ColorRole::SurfaceContainerHighest)
            },
            Stroke::new(ColorRole::OnSurfaceVariant),
            ContentColor(Paint::new(ColorRole::OnSurfaceVariant)),
            Hovered::default(),
        ))
        .with_children(|frame| {
            if let Some(leading) = spec.leading.clone() {
                frame_parts.0 = Some(frame.spawn(icon(leading, ICON)).id());
            }
            frame
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|middle| {
                    let mut editable = EditableText::new(&spec.text);
                    editable.max_characters = spec.max_length;
                    frame_parts.4 = middle
                        .spawn((
                            TextInput,
                            editable,
                            InputOf(root),
                            TabIndex(0),
                            Typeset(TypeRole::BodyLarge),
                            Ink::new(ColorRole::OnSurface),
                            TextLayout::no_wrap(),
                            TextCursorStyle::default(),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::ZERO,
                                right: Val::ZERO,
                                top: Val::Px(text_top),
                                height: Val::Px(TypeRole::BodyLarge.line_height()),
                                ..default()
                            },
                        ))
                        .id();
                    if let Some(placeholder) = spec.placeholder.clone() {
                        frame_parts.1 = Some(
                            middle
                                .spawn((
                                    Text::new(placeholder),
                                    Typeset(TypeRole::BodyLarge),
                                    Ink::new(ColorRole::OnSurfaceVariant),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::ZERO,
                                        top: Val::Px(text_top),
                                        ..default()
                                    },
                                    Visibility::Hidden,
                                    Pickable::IGNORE,
                                ))
                                .id(),
                        );
                    }
                    if let Some(label) = spec.label.clone() {
                        frame_parts.2 = Some(
                            middle
                                .spawn((
                                    Text::new(label),
                                    Typeset(TypeRole::BodyLarge),
                                    Ink::new(ColorRole::OnSurfaceVariant),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::ZERO,
                                        top: Val::Px(16.0),
                                        padding: UiRect::horizontal(Val::Px(if outlined {
                                            4.0
                                        } else {
                                            0.0
                                        })),
                                        margin: UiRect::left(Val::Px(if outlined {
                                            -4.0
                                        } else {
                                            0.0
                                        })),
                                        ..default()
                                    },
                                    Fill(Paint::faded(spec.surface, 0.0)),
                                    Pickable::IGNORE,
                                ))
                                .id(),
                        );
                    }
                });
            if spec.clearable {
                frame_parts.3 = Some(
                    frame
                        .spawn((
                            icon_button(Glyph::Close, IconButtonVariant::Standard),
                            Clears(frame_parts.4),
                        ))
                        .id(),
                );
            }
        })
        .id();
    let (leading, placeholder, label, clear, input) = frame_parts;

    let mut below_parts = (Entity::PLACEHOLDER, Entity::PLACEHOLDER);
    let below = parent
        .spawn((
            Node {
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(4.0), Val::ZERO),
                column_gap: Val::Px(16.0),
                width: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|below| {
            below_parts.0 = below
                .spawn((
                    Text::default(),
                    Typeset(TypeRole::BodySmall),
                    Ink::new(ColorRole::OnSurfaceVariant),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .id();
            below_parts.1 = below
                .spawn((
                    Text::default(),
                    Typeset(TypeRole::BodySmall),
                    Ink::new(ColorRole::OnSurfaceVariant),
                    Pickable::IGNORE,
                ))
                .id();
        })
        .id();
    let (supporting, counter) = below_parts;

    let world = parent.world_mut();
    if let Some(mut text) = world.get_mut::<InputText>(root) {
        text.input = input;
    }
    world.entity_mut(root).insert(FieldParts {
        frame,
        label,
        placeholder,
        below,
        supporting,
        counter,
        clear,
        leading,
    });
    world.entity_mut(root).observe(focus_input_on_click);
}

/// A click anywhere on a field - its label, its padding - gives its input
/// the keyboard, unless it is disabled.
fn focus_input_on_click(
    mut acquire: On<AcquireFocus>,
    fields: Query<(&InputText, Option<&TextField>)>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok((text, field)) = fields.get(acquire.event_target()) else {
        return;
    };
    acquire.propagate(false);
    if field.is_none_or(|field| field.enabled) && focus.get() != Some(text.input) {
        focus.set(text.input, FocusCause::Pressed);
    }
}

/// Keeps each part's [`InputText`] to what its input holds, and tells the
/// game of each change.
fn on_text_edit(
    edit: On<TextEditChange>,
    inputs: Query<(&EditableText, &InputOf)>,
    mut parts: Query<&mut InputText>,
    mut commands: Commands,
) {
    let Ok((editable, InputOf(part))) = inputs.get(edit.event_target()) else {
        return;
    };
    let value = editable.value().to_string();
    let Ok(mut text) = parts.get_mut(*part) else {
        return;
    };
    if text.text != value {
        text.text.clone_from(&value);
        commands.trigger(ValueChange {
            source: *part,
            value,
            is_final: false,
        });
    }
}

/// Enter in an input ends its edit: a final change.
fn submit_on_enter(
    keyboard: On<FocusedInput<KeyboardInput>>,
    inputs: Query<&InputOf>,
    parts: Query<&InputText>,
    mut commands: Commands,
) {
    let input = &keyboard.input;
    if !input.state.is_pressed() || input.repeat || input.logical_key != Key::Enter {
        return;
    }
    if let Ok(InputOf(part)) = inputs.get(keyboard.focused_entity)
        && let Ok(text) = parts.get(*part)
    {
        commands.trigger(ValueChange {
            source: *part,
            value: text.text.clone(),
            is_final: true,
        });
    }
}

/// An input losing the keyboard ends its edit: a final change.
fn submit_on_blur(
    lost: On<FocusLost>,
    inputs: Query<&InputOf>,
    parts: Query<&InputText>,
    mut commands: Commands,
) {
    if let Ok(InputOf(part)) = inputs.get(lost.entity)
        && let Ok(text) = parts.get(*part)
    {
        commands.trigger(ValueChange {
            source: *part,
            value: text.text.clone(),
            is_final: true,
        });
    }
}

/// A clear button empties its input and gives it the keyboard.
fn clear_on_activate(
    activate: On<Activate>,
    clears: Query<&Clears>,
    mut inputs: Query<&mut EditableText>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok(Clears(input)) = clears.get(activate.entity) else {
        return;
    };
    if let Ok(mut editable) = inputs.get_mut(*input) {
        editable.clear();
    }
    focus.set(*input, FocusCause::Pressed);
}

/// Draws each field in its state: its label floated or not, its indicator,
/// its placeholder, its supporting text or error, its counter, its clear
/// button, and its input turned off while it is disabled.
#[expect(clippy::too_many_arguments, reason = "a system's queries")]
fn draw_fields(
    theme: Res<MaterialTheme>,
    focus: Res<InputFocus>,
    mut fields: Query<(&mut TextField, Ref<InputText>, &FieldParts)>,
    hovered: Query<&Hovered>,
    mut nodes: Query<&mut Node>,
    mut texts: Query<&mut Text>,
    mut visibilities: Query<&mut Visibility>,
    mut cursors: Query<&mut TextCursorStyle>,
    mut commands: Commands,
) {
    for (mut field, text, parts) in &mut fields {
        let now = FieldState {
            focused: focus.get() == Some(text.input),
            hovered: hovered.get(parts.frame).is_ok_and(|hovered| hovered.0),
            empty: text.text.is_empty(),
            error: field.is_error(),
            disabled: !field.enabled,
        };
        let theme_changed = theme.is_changed();
        if field.drawn == Some(now) && !field.is_changed() && !text.is_changed() && !theme_changed {
            continue;
        }
        let first = field.drawn.is_none();
        let was = field.drawn;
        field.bypass_change_detection().drawn = Some(now);
        let outlined = field.variant == TextFieldVariant::Outlined;

        let (indicator, width) = now.indicator(field.variant);
        commands.entity(parts.frame).try_insert(Stroke(indicator));
        if let Ok(mut node) = nodes.get_mut(parts.frame) {
            let border = if outlined {
                UiRect::all(Val::Px(width))
            } else {
                UiRect::bottom(Val::Px(width))
            };
            if node.border != border {
                node.border = border;
            }
        }
        if !outlined {
            commands.entity(parts.frame).try_insert(if now.disabled {
                Fill(Paint::faded(ColorRole::OnSurface, 0.04))
            } else {
                Fill::new(ColorRole::SurfaceContainerHighest)
            });
        }
        let content = if now.disabled {
            Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)
        } else {
            Paint::new(ColorRole::OnSurfaceVariant)
        };
        commands
            .entity(parts.frame)
            .try_insert(ContentColor(content));

        if let Some(label) = parts.label {
            let floated = now.floated();
            let role = if floated {
                TypeRole::BodySmall
            } else {
                TypeRole::BodyLarge
            };
            if first || was.map(FieldState::floated) != Some(floated) {
                commands.entity(label).try_insert(Typeset(role));
            }
            commands.entity(label).try_insert(Ink(now.label()));
            if let Ok(mut node) = nodes.get_mut(label) {
                let top = match (floated, outlined) {
                    (false, _) => 16.0,
                    (true, false) => 8.0,
                    (true, true) => -8.0 - width,
                };
                node.top = Val::Px(top);
            }
            let behind = if floated && outlined {
                Paint::new(field.surface)
            } else {
                Paint::faded(field.surface, 0.0)
            };
            commands.entity(label).try_insert(Fill(behind));
        }
        if let Some(placeholder) = parts.placeholder
            && let Ok(mut visibility) = visibilities.get_mut(placeholder)
        {
            let shown = now.empty && (now.focused || parts.label.is_none());
            visibility.set_if_neq(if shown {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }

        let below_words = field.error.clone().or_else(|| field.supporting.clone());
        let count = field
            .max_length
            .map(|max| counter_text(text.text.chars().count(), max));
        if let Ok(mut words) = texts.get_mut(parts.supporting) {
            let wanted = below_words.clone().unwrap_or_default();
            if words.0 != wanted {
                words.0 = wanted;
            }
        }
        if let Ok(mut words) = texts.get_mut(parts.counter) {
            let wanted = count.clone().unwrap_or_default();
            if words.0 != wanted {
                words.0 = wanted;
            }
        }
        commands
            .entity(parts.supporting)
            .try_insert(Ink(now.below()));
        commands.entity(parts.counter).try_insert(Ink(now.below()));
        if let Ok(mut node) = nodes.get_mut(parts.below) {
            let display = if below_words.is_some() || count.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
        }

        if let Some(clear) = parts.clear
            && let Ok(mut node) = nodes.get_mut(clear)
        {
            let display = if now.empty || now.disabled {
                Display::None
            } else {
                Display::Flex
            };
            if node.display != display {
                node.display = display;
            }
        }
        let _ = parts.leading;

        let mut input = commands.entity(text.input);
        if now.disabled {
            input
                .try_insert((
                    InteractionDisabled,
                    Ink(Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT)),
                ))
                .try_remove::<TabIndex>();
        } else {
            input
                .try_remove::<InteractionDisabled>()
                .try_insert((TabIndex(0), Ink::new(ColorRole::OnSurface)));
        }
        if let Ok(mut cursor) = cursors.get_mut(text.input) {
            let caret = if now.error {
                ColorRole::Error
            } else {
                ColorRole::Primary
            };
            cursor.color = theme.color(caret);
            cursor.selection_color = theme.color(ColorRole::Primary).with_alpha(0.3);
        }
    }
}

/// The text fields' observers and drawing.
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_text_edit)
        .add_observer(submit_on_enter)
        .add_observer(submit_on_blur)
        .add_observer(clear_on_activate)
        .add_systems(PostUpdate, draw_fields.before(crate::paint::paint_controls));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> FieldState {
        FieldState {
            focused: false,
            hovered: false,
            empty: true,
            error: false,
            disabled: false,
        }
    }

    #[test]
    fn a_label_floats_once_its_field_is_focused_or_filled() {
        assert!(!state().floated());
        assert!(
            FieldState {
                focused: true,
                ..state()
            }
            .floated()
        );
        assert!(
            FieldState {
                empty: false,
                ..state()
            }
            .floated()
        );
    }

    #[test]
    fn a_focused_field_draws_a_thicker_primary_indicator_and_an_error_draws_error() {
        let filled = TextFieldVariant::Filled;
        assert_eq!(
            state().indicator(filled),
            (Paint::new(ColorRole::OnSurfaceVariant), 1.0)
        );
        assert_eq!(
            state().indicator(TextFieldVariant::Outlined),
            (Paint::new(ColorRole::Outline), 1.0)
        );
        let focused = FieldState {
            focused: true,
            ..state()
        };
        assert_eq!(
            focused.indicator(filled),
            (Paint::new(ColorRole::Primary), 2.0)
        );
        assert_eq!(focused.label(), Paint::new(ColorRole::Primary));
        let error = FieldState {
            error: true,
            ..focused
        };
        assert_eq!(error.indicator(filled), (Paint::new(ColorRole::Error), 2.0));
        assert_eq!(error.label(), Paint::new(ColorRole::Error));
        assert_eq!(error.below(), Paint::new(ColorRole::Error));
    }

    #[test]
    fn a_disabled_field_is_drawn_faded_whatever_else_it_is() {
        let disabled = FieldState {
            disabled: true,
            focused: true,
            error: true,
            ..state()
        };
        let faded = Paint::faded(ColorRole::OnSurface, state::DISABLED_CONTENT);
        assert_eq!(disabled.indicator(TextFieldVariant::Filled).0, faded);
        assert_eq!(disabled.label(), faded);
        assert_eq!(disabled.below(), faded);
    }

    #[test]
    fn a_counter_says_how_many_of_how_many() {
        assert_eq!(counter_text(3, 20), "3/20");
    }
}
