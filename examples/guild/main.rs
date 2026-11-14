//! The Guild Clerk: a short game made of forms, showing every component of
//! the crate where a game would use it.
//!
//! ```bash
//! cargo run --example guild
//! ```
//!
//! You are the clerk of an adventurers' guild. Each day a few clients come
//! to the desk; read what each one asks, write the contract, choose its
//! party from the roster and file it - or return what the guild shouldn't
//! take. The ledger keeps the score. It fills the primary monitor,
//! borderless, at its native resolution; Alt+F4 (Cmd+Q on macOS) closes it.
//!
//! The game itself is in `game.rs`, which `tests/guild.rs` plays headlessly.
//!
//! `cargo run --example guild -- --screenshots <folder>` instead saves the
//! desk, the ledger and the settings as PNGs in the folder, light and dark,
//! and exits: how the game's look is checked without sitting at it.

mod game;
#[path = "../shared/window.rs"]
mod window;

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy_material_ui::prelude::*;

fn main() {
    let screenshots = window::screenshots_folder();
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window::window("The Guild Clerk", screenshots.is_some())),
            ..default()
        }),
        MaterialUiPlugin,
        game::GuildPlugin,
    ))
    .add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
        game::spawn_guild(&mut commands);
    });
    if let Some(folder) = screenshots {
        std::fs::create_dir_all(&folder).expect("the screenshots' folder can be made");
        app.insert_resource(Tour {
            folder,
            shot: 0,
            since: None,
            waiting: false,
        })
        .add_systems(Update, tour);
    }
    app.run();
}

/// A tour of the game for `--screenshots`: each of [`SHOTS`] shown, left to
/// settle, and saved.
#[derive(Resource)]
struct Tour {
    folder: PathBuf,
    shot: usize,
    /// When the shot being taken was shown, in seconds of the game's time.
    since: Option<f32>,
    /// A screenshot is being saved.
    waiting: bool,
}

/// What the tour saves: a name, the place, and whether the theme is dark.
const SHOTS: [(&str, game::Place, bool); 6] = [
    ("desk", game::Place::Desk, false),
    ("ledger", game::Place::Ledger, false),
    ("settings", game::Place::Settings, false),
    ("desk-dark", game::Place::Desk, true),
    ("ledger-dark", game::Place::Ledger, true),
    ("settings-dark", game::Place::Settings, true),
];

/// Seconds to let a place settle - the courier, fonts, layout - before it
/// is saved: longer than the courier takes.
const SETTLE_SECONDS: f32 = 2.0;

fn tour(
    time: Res<Time>,
    mut tour: ResMut<Tour>,
    mut screen: ResMut<game::Screen>,
    mut theme: ResMut<MaterialTheme>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if tour.waiting {
        return;
    }
    let Some(&(name, place, dark)) = SHOTS.get(tour.shot) else {
        exit.write(AppExit::Success);
        return;
    };
    let now = time.elapsed_secs();
    let Some(since) = tour.since else {
        tour.since = Some(now);
        if screen.place != place {
            screen.place = place;
        }
        let brightness = if dark {
            Brightness::Dark
        } else {
            Brightness::Light
        };
        if theme.brightness() != brightness {
            *theme = theme.at(brightness);
        }
        return;
    };
    if now - since < SETTLE_SECONDS {
        return;
    }
    let path = tour.folder.join(format!("{name}.png"));
    tour.waiting = true;
    tour.since = None;
    tour.shot += 1;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(|_: On<ScreenshotCaptured>, mut tour: ResMut<Tour>| {
            tour.waiting = false;
        });
}
