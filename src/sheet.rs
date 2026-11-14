//! The modal bottom sheet: a surface rising from the bottom edge, on a
//! scrim, holding a game's own parts - a list of actions, a form.
//!
//! It closes when the scrim is clicked, when Escape is pressed, or when its
//! drag handle is used, and sends [`SheetClosed`] to the entity the game
//! spawned as it goes.

use bevy::ecs::entity::EntityHashSet;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::{UiTransform, Val2};
use bevy::ui_widgets::{Activate, Button, ModalDialog, RequestClose};

use crate::dialog::{Closing, full_window, scrim};
use crate::focus::FocusRing;
use crate::paint::{ContentColor, Fill, Look, Paint, Raised};
use crate::theme::tokens::duration;
use crate::theme::{ColorRole, Corner, Elevation, Space, emphasized};

/// Marks the entity a sheet is spawned as: the scrim, holding the sheet.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SheetScrim;

/// The sheet's own surface, rising into place.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SheetSurface {
    /// How long it has been rising, in seconds.
    pub risen: f32,
}

/// A sheet's drag handle: using it closes the sheet.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SheetHandle;

/// Sent to a sheet's scrim - the entity the game spawned - as it closes.
#[derive(EntityEvent, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SheetClosed {
    /// The sheet, as spawned.
    #[event_target]
    pub sheet: Entity,
}

/// A sheet's widest, in logical pixels: on a wide window it stands centred.
pub const SHEET_MAX_WIDTH: f32 = 640.0;

/// A modal bottom sheet holding the parts `content` spawns.
pub fn bottom_sheet(
    content: impl FnOnce(&mut ChildSpawner) + Send + Sync + 'static,
) -> impl Bundle {
    (
        scrim(),
        SheetScrim,
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            ..full_window()
        },
        children![(
            SheetSurface::default(),
            ModalDialog,
            Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(SHEET_MAX_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::bottom(Space::ExtraLarge.val()),
                border_radius: Corner::ExtraLarge.top(),
                ..default()
            },
            UiTransform {
                translation: Val2::percent(0.0, 100.0),
                ..default()
            },
            Fill::new(ColorRole::SurfaceContainerLow),
            Raised(Elevation::Level1),
            ContentColor(Paint::new(ColorRole::OnSurface)),
            Children::spawn(SpawnWith(move |sheet: &mut ChildSpawner| {
                sheet
                    .spawn((
                        SheetHandle,
                        Button,
                        Look::bare(ColorRole::OnSurfaceVariant),
                        FocusRing,
                        TabIndex(0),
                        Node {
                            align_self: AlignSelf::Center,
                            padding: UiRect::axes(Space::Large.val(), Val::Px(22.0)),
                            border_radius: Corner::Full.all(),
                            ..default()
                        },
                    ))
                    .with_child((
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(4.0),
                            border_radius: Corner::Full.all(),
                            ..default()
                        },
                        Fill(Paint::faded(ColorRole::OnSurfaceVariant, 0.4)),
                        Pickable::IGNORE,
                    ));
                sheet
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::horizontal(Space::Large.val()),
                        row_gap: Space::Small.val(),
                        ..default()
                    })
                    .with_children(content);
            })),
        )],
    )
}

/// Raises each sheet into place from below the window's edge.
fn rise(time: Res<Time>, mut sheets: Query<(&mut SheetSurface, &mut UiTransform)>) {
    for (mut sheet, mut transform) in &mut sheets {
        if sheet.risen >= duration::MEDIUM {
            continue;
        }
        sheet.risen = (sheet.risen + time.delta_secs()).min(duration::MEDIUM);
        let shown = emphasized(sheet.risen / duration::MEDIUM);
        transform.translation = Val2::percent(0.0, (1.0 - shown) * 100.0);
    }
}

/// A used drag handle closes its sheet.
fn on_handle(
    activate: On<Activate>,
    handles: Query<(), With<SheetHandle>>,
    mut commands: Commands,
) {
    if handles.contains(activate.entity) {
        commands.trigger(RequestClose {
            source: activate.entity,
        });
    }
}

/// Closes a sheet whose scrim was clicked, in which Escape was pressed, or
/// whose handle was used.
fn on_request_close(
    close_request: On<RequestClose>,
    scrims: Query<(), (With<SheetScrim>, Without<Closing>)>,
    parents: Query<&ChildOf>,
    mut closed: Local<EntityHashSet>,
    mut commands: Commands,
) {
    if close_request.event_target() != close_request.source {
        return;
    }
    let source = close_request.source;
    if let Some(sheet) = std::iter::once(source)
        .chain(parents.iter_ancestors(source))
        .find(|candidate| scrims.contains(*candidate))
        && closed.insert(sheet)
    {
        commands.entity(sheet).try_insert(Closing);
        commands.trigger(SheetClosed { sheet });
        commands.entity(sheet).try_despawn();
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_handle)
        .add_observer(on_request_close)
        .add_systems(Update, rise);
}
