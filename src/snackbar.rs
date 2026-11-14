//! Snackbars: brief messages at the bottom of the window, one at a time.
//!
//! A game asks for one through the [`Snackbars`] resource; they are shown in
//! the order asked, each for its time, then the next. One with an action
//! sends [`SnackbarAction`] when it is used; every one sends
//! [`SnackbarClosed`] as it goes, saying why.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::button::{IconButtonVariant, button, icon_button};
use crate::icon::Glyph;
use crate::paint::{Fill, Look, Raised, text};
use crate::theme::{ColorRole, Corner, Elevation, Space, TypeRole};

/// How long a snackbar is shown, in seconds, without an action and with one.
pub const SNACKBAR_SECONDS: f32 = 4.0;
/// How long a snackbar with an action is shown: long enough to reach it.
pub const SNACKBAR_ACTION_SECONDS: f32 = 8.0;

/// The layer snackbars are drawn in: above the game, below a modal.
const SNACKBAR_LAYER: i32 = 98;

/// Which snackbar: handed back when one is asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct SnackbarId(pub u64);

/// A snackbar to show.
#[derive(Clone, Debug, PartialEq)]
pub struct Snackbar {
    /// What it says.
    pub message: String,
    /// Its action's label, if it has one.
    pub action: Option<String>,
    /// Whether it has a close button.
    pub closable: bool,
    /// How long it is shown, in seconds.
    pub seconds: f32,
}

impl Snackbar {
    /// A snackbar saying `message`, shown for [`SNACKBAR_SECONDS`].
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action: None,
            closable: false,
            seconds: SNACKBAR_SECONDS,
        }
    }

    /// With an action saying `label`, and shown for
    /// [`SNACKBAR_ACTION_SECONDS`] unless told otherwise.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        if self.seconds == SNACKBAR_SECONDS {
            self.seconds = SNACKBAR_ACTION_SECONDS;
        }
        self
    }

    /// With a close button.
    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }

    /// Shown for `seconds`.
    pub fn seconds(mut self, seconds: f32) -> Self {
        self.seconds = seconds;
        self
    }
}

/// The snackbar showing now.
#[derive(Clone, Debug, PartialEq)]
struct Showing {
    id: SnackbarId,
    /// The row holding it, taken away whole.
    host: Entity,
    left: f32,
}

/// The snackbars waiting to be shown, and the one showing.
#[derive(Resource, Debug, Default)]
pub struct Snackbars {
    waiting: VecDeque<(SnackbarId, Snackbar)>,
    showing: Option<Showing>,
    next: u64,
    /// Closed by its action or its close button, waiting for a frame.
    closing: Option<(SnackbarId, SnackbarCloseReason)>,
}

impl Snackbars {
    /// Shows a snackbar saying `message` once those before it have gone.
    pub fn show(&mut self, message: impl Into<String>) -> SnackbarId {
        self.show_snackbar(Snackbar::new(message))
    }

    /// Shows `snackbar` once those before it have gone.
    pub fn show_snackbar(&mut self, snackbar: Snackbar) -> SnackbarId {
        let id = SnackbarId(self.next);
        self.next += 1;
        self.waiting.push_back((id, snackbar));
        id
    }

    /// The snackbar showing now, if one is.
    pub fn showing(&self) -> Option<SnackbarId> {
        self.showing.as_ref().map(|showing| showing.id)
    }

    /// How many are waiting behind the one showing.
    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }

    /// Closes the snackbar showing now, if it is `id`.
    pub fn dismiss(&mut self, id: SnackbarId) {
        if self.showing() == Some(id) {
            self.closing = Some((id, SnackbarCloseReason::Dismissed));
        }
        self.waiting.retain(|(waiting, _)| *waiting != id);
    }
}

/// Why a snackbar went.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum SnackbarCloseReason {
    /// Its time ran out.
    Timeout,
    /// Its action was used.
    Action,
    /// Its close button was used, or the game dismissed it.
    Dismissed,
}

/// Sent when a snackbar's action is used.
#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnackbarAction {
    /// The snackbar.
    pub id: SnackbarId,
}

/// Sent as a snackbar goes.
#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnackbarClosed {
    /// The snackbar.
    pub id: SnackbarId,
    /// Why it went.
    pub reason: SnackbarCloseReason,
}

/// Marks a snackbar on screen, with which it is.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SnackbarView(pub SnackbarId);

/// Marks a snackbar's action button.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SnackbarActionButton(pub SnackbarId);

/// Marks a snackbar's close button.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SnackbarCloseButton(pub SnackbarId);

/// A snackbar's parts: the row along the bottom of the window holding it,
/// and the snackbar itself.
fn snackbar_bundle(id: SnackbarId, snackbar: Snackbar) -> impl Bundle {
    let Snackbar {
        message,
        action,
        closable,
        ..
    } = snackbar;
    let end = if action.is_some() || closable {
        Space::Small.val()
    } else {
        Space::Large.val()
    };
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            right: Val::ZERO,
            bottom: Space::Large.val(),
            justify_content: JustifyContent::Center,
            padding: UiRect::horizontal(Space::Large.val()),
            ..default()
        },
        GlobalZIndex(SNACKBAR_LAYER),
        Pickable::IGNORE,
        children![(
            SnackbarView(id),
            Node {
                min_height: Val::Px(48.0),
                min_width: Val::Px(288.0),
                max_width: Val::Px(600.0),
                padding: UiRect::new(Space::Large.val(), end, Val::Px(6.0), Val::Px(6.0)),
                column_gap: Space::Small.val(),
                align_items: AlignItems::Center,
                border_radius: Corner::ExtraSmall.all(),
                ..default()
            },
            Fill::new(ColorRole::InverseSurface),
            Raised(Elevation::Level3),
            Children::spawn(SpawnWith(move |bar: &mut ChildSpawner| {
                bar.spawn((
                    text(message, TypeRole::BodyMedium, ColorRole::InverseOnSurface),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ));
                if let Some(label) = action {
                    bar.spawn((button(label).text().bundle(), SnackbarActionButton(id)))
                        .insert(Look::bare(ColorRole::InversePrimary));
                }
                if closable {
                    bar.spawn((
                        icon_button(Glyph::Close, IconButtonVariant::Standard),
                        SnackbarCloseButton(id),
                    ))
                    .insert(Look::bare(ColorRole::InverseOnSurface));
                }
            })),
        )],
    )
}

/// Shows the next snackbar when none is showing, counts down the one
/// showing, and takes it away when its time is up or it was closed.
fn run_snackbars(time: Res<Time>, mut snackbars: ResMut<Snackbars>, mut commands: Commands) {
    let closing = snackbars.closing.take();
    if let Some(showing) = snackbars.showing.as_mut() {
        showing.left -= time.delta_secs();
        let reason = match closing {
            Some((id, reason)) if id == showing.id => Some(reason),
            _ if showing.left <= 0.0 => Some(SnackbarCloseReason::Timeout),
            _ => None,
        };
        if let Some(reason) = reason {
            let Showing { id, host, .. } = *showing;
            commands.entity(host).try_despawn();
            commands.trigger(SnackbarClosed { id, reason });
            snackbars.showing = None;
        }
        return;
    }
    let Some((id, snackbar)) = snackbars.waiting.pop_front() else {
        return;
    };
    let left = snackbar.seconds;
    let host = commands.spawn(snackbar_bundle(id, snackbar)).id();
    snackbars.showing = Some(Showing { id, host, left });
}

/// Uses a snackbar's action or close button.
fn on_activate(
    activate: On<Activate>,
    actions: Query<&SnackbarActionButton>,
    closes: Query<&SnackbarCloseButton>,
    mut snackbars: ResMut<Snackbars>,
    mut commands: Commands,
) {
    if let Ok(SnackbarActionButton(id)) = actions.get(activate.entity) {
        commands.trigger(SnackbarAction { id: *id });
        if snackbars.showing() == Some(*id) {
            snackbars.closing = Some((*id, SnackbarCloseReason::Action));
        }
    } else if let Ok(SnackbarCloseButton(id)) = closes.get(activate.entity) {
        snackbars.dismiss(*id);
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Snackbars>()
        .add_observer(on_activate)
        .add_systems(Update, run_snackbars);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snackbars_wait_in_the_order_asked_and_each_has_its_own_id() {
        let mut snackbars = Snackbars::default();
        let first = snackbars.show("Saved");
        let second = snackbars.show("Sent");
        assert_ne!(first, second);
        assert_eq!(snackbars.waiting(), 2);
        assert_eq!(snackbars.waiting[0].0, first);
        snackbars.dismiss(first);
        assert_eq!(
            snackbars.waiting(),
            1,
            "one dismissed before it shows never shows"
        );
    }

    #[test]
    fn a_snackbar_with_an_action_stays_longer_unless_told_otherwise() {
        assert_eq!(Snackbar::new("Saved").seconds, SNACKBAR_SECONDS);
        assert_eq!(
            Snackbar::new("Deleted").action("Undo").seconds,
            SNACKBAR_ACTION_SECONDS
        );
        assert_eq!(
            Snackbar::new("Deleted").seconds(2.0).action("Undo").seconds,
            2.0
        );
    }
}
