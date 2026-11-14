//! The play test: every component in every state, on one screen, all live.
//!
//! ```bash
//! cargo run --example playtest
//! ```
//!
//! It fills the primary monitor, borderless, at its native resolution; Alt+F4
//! (Cmd+Q on macOS) closes it.
//! Click, type, drag and tab through it; `D` turns the theme light or dark
//! and `S` moves to the next seed colour. `tests/playtest.rs` drives the same
//! gallery headlessly.
//!
//! `cargo run --example playtest -- --screenshots <folder> [--dark]` instead
//! scrolls through the whole gallery, saves each screenful as a PNG in the
//! folder, and exits: how the gallery's look is checked without sitting at it.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::ui_widgets::ScrollArea;
use bevy::window::WindowMode;
use bevy_material_ui::gallery::{GalleryPlugin, spawn_gallery};
use bevy_material_ui::prelude::*;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let screenshots = arguments
        .iter()
        .position(|argument| argument == "--screenshots")
        .and_then(|at| arguments.get(at + 1))
        .map(PathBuf::from);
    let dark = arguments.iter().any(|argument| argument == "--dark");

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy_material_ui play test".into(),
                // Borderless full screen takes the monitor's own resolution.
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }),
        MaterialUiPlugin,
        GalleryPlugin,
    ))
    .add_systems(Startup, setup);
    if dark {
        app.insert_resource(MaterialTheme::dark());
    }
    if let Some(folder) = screenshots {
        std::fs::create_dir_all(&folder).expect("the screenshots' folder can be made");
        app.insert_resource(Tour {
            folder,
            frame: 0,
            page: 0,
            waiting: false,
            scroll_next: false,
            overlay: None,
        })
        .add_systems(Update, tour);
    }
    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    spawn_gallery(&mut commands);
}

/// A tour of the gallery for `--screenshots`: a screenful saved, then the
/// page scrolled on, until its end.
#[derive(Resource)]
struct Tour {
    folder: PathBuf,
    frame: u32,
    page: u32,
    /// A screenshot is being saved.
    waiting: bool,
    /// The screenful is saved; scroll on next.
    scroll_next: bool,
    /// Past the last page: the overlay being shown, and what it spawned.
    overlay: Option<(usize, Option<Entity>)>,
}

/// The overlays the tour shows after the pages, each saved on its own.
const OVERLAYS: [&str; 3] = ["dialog", "sheet", "snackbar"];

/// Frames to let a screenful settle - fonts, layout, animations - before it
/// is saved.
const SETTLE_FRAMES: u32 = 40;

fn tour(
    mut tour: ResMut<Tour>,
    mut pages: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollArea>>,
    mut snackbars: ResMut<Snackbars>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if tour.waiting {
        return;
    }
    if let Some((index, shown)) = tour.overlay {
        overlay(
            &mut tour,
            index,
            shown,
            &mut snackbars,
            &mut commands,
            &mut exit,
        );
        return;
    }
    // The gallery's own page is the largest scroll area.
    let Some((mut scroll, node)) = pages
        .iter_mut()
        .max_by(|a, b| a.1.size().y.total_cmp(&b.1.size().y))
    else {
        return;
    };
    if tour.scroll_next {
        tour.scroll_next = false;
        let visible = node.size().y * node.inverse_scale_factor();
        let content = node.content_size().y * node.inverse_scale_factor();
        if scroll.y + visible >= content - 1.0 || tour.page > 40 {
            tour.overlay = Some((0, None));
            tour.frame = 0;
            return;
        }
        scroll.y = (scroll.y + visible * 0.9).min(content - visible);
        tour.page += 1;
        tour.frame = 0;
        return;
    }
    tour.frame += 1;
    if tour.frame < SETTLE_FRAMES {
        return;
    }
    let path = tour.folder.join(format!("page-{:02}.png", tour.page));
    tour.waiting = true;
    tour.scroll_next = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(|_: On<ScreenshotCaptured>, mut tour: ResMut<Tour>| {
            tour.waiting = false;
        });
}

/// Shows overlay `index` of [`OVERLAYS`] over the gallery, saves it, takes
/// it away, and moves on to the next; exits after the last.
fn overlay(
    tour: &mut Tour,
    index: usize,
    shown: Option<Entity>,
    snackbars: &mut Snackbars,
    commands: &mut Commands,
    exit: &mut MessageWriter<AppExit>,
) {
    if tour.scroll_next {
        // Saved: take it away and go on to the next.
        tour.scroll_next = false;
        if let Some(entity) = shown {
            commands.entity(entity).try_despawn();
        }
        if index + 1 >= OVERLAYS.len() {
            exit.write(AppExit::Success);
        } else {
            tour.overlay = Some((index + 1, None));
            tour.frame = 0;
        }
        return;
    }
    if tour.frame == 0 {
        let spawned = match OVERLAYS[index] {
            "dialog" => Some(
                commands
                    .spawn(
                        confirm_dialog(
                            "Delete this world?",
                            "Its blocks, its chests and its maps are gone for good.",
                            "Delete",
                            "Keep",
                        )
                        .bundle(),
                    )
                    .id(),
            ),
            "sheet" => Some(
                commands
                    .spawn(bottom_sheet(|sheet| {
                        sheet.spawn(text("Share", TypeRole::TitleLarge, ColorRole::OnSurface));
                        sheet.spawn(text(
                            "Send this world's seed to a friend.",
                            TypeRole::BodyMedium,
                            ColorRole::OnSurfaceVariant,
                        ));
                    }))
                    .id(),
            ),
            _ => {
                // A minute, so the frames spent saving screenshots never time it out.
                snackbars.show_snackbar(
                    Snackbar::new("Block broken")
                        .action("Undo")
                        .closable()
                        .seconds(60.0),
                );
                None
            }
        };
        tour.overlay = Some((index, spawned));
    }
    tour.frame += 1;
    if tour.frame < SETTLE_FRAMES {
        return;
    }
    let path = tour.folder.join(format!("overlay-{}.png", OVERLAYS[index]));
    tour.waiting = true;
    tour.scroll_next = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(|_: On<ScreenshotCaptured>, mut tour: ResMut<Tour>| {
            tour.waiting = false;
        });
}
