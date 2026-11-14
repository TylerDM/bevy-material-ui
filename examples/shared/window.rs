//! The examples' window, shared by `playtest.rs` and `guild/main.rs`, which
//! each take this file in with `#[path]`.

use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};

/// The size of the window `--screenshots` saves, in pixels.
pub(crate) const SHOT_SIZE: (u32, u32) = (1600, 1260);

/// An example's window titled `title`: borderless full screen at the primary
/// monitor's own resolution - or, for `--screenshots`, a window of
/// [`SHOT_SIZE`] at a scale of one, so the pictures are the same on every
/// machine.
pub(crate) fn window(title: &str, screenshots: bool) -> Window {
    if screenshots {
        Window {
            title: title.into(),
            resolution: WindowResolution::new(SHOT_SIZE.0, SHOT_SIZE.1)
                .with_scale_factor_override(1.0),
            ..default()
        }
    } else {
        Window {
            title: title.into(),
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            ..default()
        }
    }
}

/// The folder named after `--screenshots` on the command line, if any.
pub(crate) fn screenshots_folder() -> Option<std::path::PathBuf> {
    let arguments: Vec<String> = std::env::args().collect();
    arguments
        .iter()
        .position(|argument| argument == "--screenshots")
        .and_then(|at| arguments.get(at + 1))
        .map(std::path::PathBuf::from)
}
