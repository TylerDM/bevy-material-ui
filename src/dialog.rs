//! Dialogs: Material's basic dialog, over Bevy's headless
//! [`ModalDialog`], on a scrim that covers the window and takes every click
//! meant for what is behind it.
//!
//! A dialog closes itself when one of its actions is used, when the scrim is
//! clicked or when Escape is pressed, and says how in a [`DialogResult`] on
//! the entity the game spawned - observe it there, or anywhere.

use bevy::ecs::entity::EntityHashSet;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::{FixedNode, OverrideClip};
use bevy::ui_widgets::{Activate, ModalDialog, ModalDialogBarrier, RequestClose};

use crate::button::button;
use crate::icon::{Icon, icon};
use crate::paint::{ContentColor, Fill, Paint, Raised, text};
use crate::theme::{ColorRole, Corner, Elevation, Space, TypeRole};

/// The scrim's opacity behind a dialog or a modal sheet: Material's 32 %.
pub const SCRIM_OPACITY: f32 = 0.32;

/// The layer modal parts are drawn in, above everything a game draws.
pub(crate) const MODAL_LAYER: i32 = 99;

/// Marks the entity a dialog is spawned as: the scrim, holding the dialog.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct DialogScrim;

/// The dialog's own surface, inside its scrim.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct DialogSurface;

/// One of a dialog's actions: the key it answers with.
#[derive(Component, Clone, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct DialogActionKey(pub String);

/// How a dialog was answered.
#[derive(Clone, Debug, PartialEq, Eq, Reflect)]
pub enum DialogAnswer {
    /// One of its actions was used: its key.
    Chose(String),
    /// It was dismissed: the scrim clicked, or Escape pressed.
    Dismissed,
}

/// The key a confirmation dialog's confirm action answers with.
pub const CONFIRM: &str = "confirm";
/// The key a confirmation dialog's cancel action answers with.
pub const CANCEL: &str = "cancel";

/// Sent to a dialog's scrim - the entity the game spawned - as it closes:
/// how it was answered. The dialog is despawned after it.
#[derive(EntityEvent, Clone, Debug, PartialEq, Eq, Reflect)]
pub struct DialogResult {
    /// The dialog, as spawned.
    #[event_target]
    pub dialog: Entity,
    /// How it was answered.
    pub answer: DialogAnswer,
}

impl DialogResult {
    /// Whether its confirm action was used.
    pub fn is_confirmed(&self) -> bool {
        self.answer == DialogAnswer::Chose(CONFIRM.to_owned())
    }
}

/// An action of a dialog: its key and its label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogAction {
    /// What the dialog answers with when it is used.
    pub key: String,
    /// What its button says.
    pub label: String,
}

/// What a dialog shows inside it, besides its title and text: a game's own
/// parts.
type Body = Box<dyn FnOnce(&mut ChildSpawner) + Send + Sync>;

/// A basic dialog: an icon, a title, supporting text, the game's own parts
/// and actions.
pub struct MaterialDialog {
    /// Its headline.
    pub title: String,
    /// Its supporting text.
    pub text: Option<String>,
    /// An icon above the title, which centres it.
    pub icon: Option<Icon>,
    /// Its actions, laid out at its end, the last the most important.
    pub actions: Vec<DialogAction>,
    body: Option<Body>,
}

/// A dialog titled `title`, with no actions yet.
pub fn dialog(title: impl Into<String>) -> MaterialDialog {
    MaterialDialog {
        title: title.into(),
        text: None,
        icon: None,
        actions: Vec::new(),
        body: None,
    }
}

/// A dialog asking to confirm something: `title`, `text`, and a cancel and
/// a confirm action answering [`CANCEL`] and [`CONFIRM`].
pub fn confirm_dialog(
    title: impl Into<String>,
    text: impl Into<String>,
    confirm: impl Into<String>,
    cancel: impl Into<String>,
) -> MaterialDialog {
    dialog(title)
        .text(text)
        .action(CANCEL, cancel)
        .action(CONFIRM, confirm)
}

impl MaterialDialog {
    /// With supporting text under its title.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// With `icon` above its title.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// With an action saying `label`, answering `key`.
    pub fn action(mut self, key: impl Into<String>, label: impl Into<String>) -> Self {
        self.actions.push(DialogAction {
            key: key.into(),
            label: label.into(),
        });
        self
    }

    /// With the game's own parts under its text, spawned by `body`.
    pub fn body(mut self, body: impl FnOnce(&mut ChildSpawner) + Send + Sync + 'static) -> Self {
        self.body = Some(Box::new(body));
        self
    }

    /// Its components: spawn it anywhere, and it covers the window.
    pub fn bundle(self) -> impl Bundle {
        let Self {
            title,
            text: words,
            icon: glyph,
            actions,
            body,
        } = self;
        let centred = glyph.is_some();
        (
            scrim(),
            DialogScrim,
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..full_window()
            },
            children![(
                DialogSurface,
                ModalDialog,
                Node {
                    min_width: Val::Px(280.0),
                    max_width: Val::Px(560.0),
                    flex_direction: FlexDirection::Column,
                    align_items: if centred {
                        AlignItems::Center
                    } else {
                        AlignItems::Stretch
                    },
                    padding: UiRect::all(Space::ExtraLarge.val()),
                    row_gap: Space::Large.val(),
                    border_radius: Corner::ExtraLarge.all(),
                    ..default()
                },
                Fill::new(ColorRole::SurfaceContainerHigh),
                Raised(Elevation::Level3),
                ContentColor(Paint::new(ColorRole::OnSurface)),
                Children::spawn(SpawnWith(move |surface: &mut ChildSpawner| {
                    if let Some(glyph) = glyph {
                        surface
                            .spawn((
                                Node::default(),
                                ContentColor(Paint::new(ColorRole::Secondary)),
                            ))
                            .with_child(icon(glyph, 24.0));
                    }
                    surface.spawn(text(title, TypeRole::HeadlineSmall, ColorRole::OnSurface));
                    if let Some(words) = words {
                        surface.spawn(text(
                            words,
                            TypeRole::BodyMedium,
                            ColorRole::OnSurfaceVariant,
                        ));
                    }
                    if let Some(body) = body {
                        surface
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Space::Small.val(),
                                align_self: AlignSelf::Stretch,
                                ..default()
                            })
                            .with_children(body);
                    }
                    if !actions.is_empty() {
                        surface
                            .spawn(Node {
                                align_self: AlignSelf::Stretch,
                                justify_content: JustifyContent::FlexEnd,
                                column_gap: Space::Small.val(),
                                padding: UiRect::top(Space::Small.val()),
                                ..default()
                            })
                            .with_children(|row| {
                                for action in actions {
                                    row.spawn((
                                        button(action.label).text().bundle(),
                                        DialogActionKey(action.key),
                                    ));
                                }
                            });
                    }
                })),
            )],
        )
    }
}

/// A node covering the whole window, above everything else.
pub(crate) fn full_window() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::ZERO,
        right: Val::ZERO,
        top: Val::ZERO,
        bottom: Val::ZERO,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// A modal scrim: Bevy's barrier, which closes what it holds when clicked
/// or when Escape is pressed inside it, drawn at the scrim's opacity above
/// everything a game draws.
pub(crate) fn scrim() -> impl Bundle {
    (
        ModalDialogBarrier,
        FixedNode,
        OverrideClip,
        GlobalZIndex(MODAL_LAYER),
        Fill(Paint::faded(ColorRole::Scrim, SCRIM_OPACITY)),
    )
}

/// The scrim at or above `entity`, if there is one.
fn scrim_of(
    entity: Entity,
    scrims: &Query<(), (With<DialogScrim>, Without<Closing>)>,
    parents: &Query<&ChildOf>,
) -> Option<Entity> {
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find(|candidate| scrims.contains(*candidate))
}

/// Answers a dialog with the key of the action that was used, and closes it.
fn on_action(
    activate: On<Activate>,
    keys: Query<&DialogActionKey>,
    scrims: Query<(), (With<DialogScrim>, Without<Closing>)>,
    parents: Query<&ChildOf>,
    mut closed: Local<EntityHashSet>,
    mut commands: Commands,
) {
    let Ok(DialogActionKey(key)) = keys.get(activate.entity) else {
        return;
    };
    if let Some(dialog) = scrim_of(activate.entity, &scrims, &parents)
        && closed.insert(dialog)
    {
        close(&mut commands, dialog, DialogAnswer::Chose(key.clone()));
    }
}

/// Dismisses a dialog whose scrim was clicked, or in which Escape was pressed.
fn on_request_close(
    close_request: On<RequestClose>,
    scrims: Query<(), (With<DialogScrim>, Without<Closing>)>,
    parents: Query<&ChildOf>,
    mut closed: Local<EntityHashSet>,
    mut commands: Commands,
) {
    // The request travels up from where it began; answer it once, there.
    if close_request.event_target() != close_request.source {
        return;
    }
    if let Some(dialog) = scrim_of(close_request.source, &scrims, &parents)
        && closed.insert(dialog)
    {
        close(&mut commands, dialog, DialogAnswer::Dismissed);
    }
}

/// Marks a modal part that has been answered and is going, so a request to
/// close it that arrives twice answers it once.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Closing;

fn close(commands: &mut Commands, dialog: Entity, answer: DialogAnswer) {
    commands.entity(dialog).try_insert(Closing);
    commands.trigger(DialogResult { dialog, answer });
    commands.entity(dialog).try_despawn();
}

/// Makes a dialog with no action of its own focusable, so Escape reaches it.
fn on_surface(add: On<Add<DialogSurface>>, mut commands: Commands) {
    commands.entity(add.entity).insert_if_new(TabIndex(0));
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_action)
        .add_observer(on_request_close)
        .add_observer(on_surface);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_confirmation_dialog_cancels_first_and_confirms_last() {
        let asked = confirm_dialog("Delete?", "It can't be undone.", "Delete", "Keep");
        let keys: Vec<&str> = asked
            .actions
            .iter()
            .map(|action| action.key.as_str())
            .collect();
        assert_eq!(keys, [CANCEL, CONFIRM]);
        assert_eq!(asked.actions[1].label, "Delete");
        assert_eq!(asked.text.as_deref(), Some("It can't be undone."));
    }

    #[test]
    fn only_the_confirm_action_confirms() {
        let result = |answer| DialogResult {
            dialog: Entity::PLACEHOLDER,
            answer,
        };
        assert!(result(DialogAnswer::Chose(CONFIRM.to_owned())).is_confirmed());
        assert!(!result(DialogAnswer::Chose(CANCEL.to_owned())).is_confirmed());
        assert!(!result(DialogAnswer::Dismissed).is_confirmed());
    }
}
