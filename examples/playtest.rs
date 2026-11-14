//! The play test: every component in every state, on one screen, all live.
//!
//! ```bash
//! cargo run --example playtest
//! ```
//!
//! Click, type, drag and tab through it; `D` turns the theme light or dark
//! and `S` moves to the next seed colour. `tests/playtest.rs` drives the same
//! gallery headlessly.

use bevy::prelude::*;
use bevy_material_ui::MaterialUiPlugin;
use bevy_material_ui::gallery::{GalleryPlugin, spawn_gallery};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "bevy_material_ui play test".into(),
                    resolution: (1440, 960).into(),
                    ..default()
                }),
                ..default()
            }),
            MaterialUiPlugin,
            GalleryPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    spawn_gallery(&mut commands);
}
