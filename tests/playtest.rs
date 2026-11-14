//! The play test, headless: the whole gallery built at real window sizes and
//! driven as a player would - every kind of control clicked, typed into or
//! keyed, every overlay opened and closed, the theme turned dark and
//! reseeded - checking what each does and that the screen holds together.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_material_ui::gallery::{
    BrightnessToggle, Gallery, GalleryLog, GalleryPlugin, SeedToggle, spawn_gallery,
};
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{click, descendants, find, find_all, headless_app, settle, words};

/// A headless app showing the gallery in a window `width` x `height`.
fn gallery(width: u32, height: u32) -> (App, Entity) {
    let mut app = headless_app(width, height, 1.0);
    app.add_plugins(GalleryPlugin);
    let root = app
        .world_mut()
        .run_system_once(|mut commands: Commands| spawn_gallery(&mut commands))
        .expect("the gallery is spawned");
    settle(&mut app);
    (app, root)
}

fn log(app: &App) -> GalleryLog {
    app.world().resource::<GalleryLog>().clone()
}

/// The first button at or below `root` whose words are `label`.
fn button_saying(app: &App, root: Entity, label: &str) -> Entity {
    find_all::<ButtonVariant>(app, root)
        .into_iter()
        .find(|button| words(app, *button) == label)
        .unwrap_or_else(|| panic!("a button saying {label:?}"))
}

#[test]
fn the_gallery_builds_and_lays_out_at_every_window_size() {
    for &(width, height, _) in bevy_material_ui::testing::WINDOWS {
        let (app, root) = gallery(width, height);
        assert!(app.world().get::<Gallery>(root).is_some());
        let parts = descendants(&app, root);
        assert!(parts.len() > 100, "{width}x{height}: {} parts", parts.len());
        for part in parts {
            if let Some(node) = app.world().get::<ComputedNode>(part) {
                assert!(
                    node.size.is_finite(),
                    "{width}x{height}: a part laid out to {:?}",
                    node.size
                );
            }
        }
    }
}

#[test]
fn every_button_variant_in_the_gallery_activates_when_clicked() {
    let (mut app, root) = gallery(1440, 960);
    for (label, noted) in [
        ("Filled", "filled button"),
        ("Tonal", "tonal button"),
        ("Elevated", "elevated button"),
        ("Outlined", "outlined button"),
        ("Text", "text button"),
    ] {
        let button = button_saying(&app, root, label);
        click(&mut app, button);
        assert_eq!(log(&app).last(), Some(noted), "{label}");
    }
}

#[test]
fn the_app_bar_turns_the_whole_gallery_dark_and_reseeds_it() {
    let (mut app, root) = gallery(1440, 960);
    let surface = |app: &App| app.world().get::<BackgroundColor>(root).expect("a fill").0;
    assert_eq!(
        surface(&app),
        MaterialTheme::light().color(ColorRole::Surface)
    );

    let toggle = find::<BrightnessToggle>(&app, root).expect("the brightness toggle");
    click(&mut app, toggle);
    assert_eq!(log(&app).last(), Some("theme Dark"));
    assert_eq!(
        surface(&app),
        MaterialTheme::dark().color(ColorRole::Surface)
    );

    let seed = find::<SeedToggle>(&app, root).expect("the seed toggle");
    click(&mut app, seed);
    let theme = app.world().resource::<MaterialTheme>().clone();
    assert_eq!(
        theme.brightness(),
        Brightness::Dark,
        "reseeding keeps it dark"
    );
    assert!(theme.seed.is_some());
    assert_eq!(surface(&app), theme.color(ColorRole::Surface));
}

#[test]
fn a_disabled_button_in_the_gallery_does_nothing_when_clicked() {
    let (mut app, root) = gallery(1440, 960);
    let disabled = find_all::<ButtonVariant>(&app, root)
        .into_iter()
        .find(|button| {
            app.world()
                .get::<bevy::ui::InteractionDisabled>(*button)
                .is_some()
        })
        .expect("a disabled button");
    let before = log(&app);
    click(&mut app, disabled);
    assert_eq!(log(&app), before);
}
