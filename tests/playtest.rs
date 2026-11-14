//! The play test, headless: the whole gallery built at real window sizes and
//! driven as a player would - every kind of control clicked, typed into,
//! dragged or keyed, every overlay opened and closed, the theme turned dark
//! and reseeded - checking what each did, by the gallery's own log.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::keyboard::Key;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui::{Checked, InteractionDisabled};
use bevy::ui_widgets::Button as Pressable;
use bevy_material_ui::gallery::{
    BrightnessToggle, Gallery, GalleryLog, GalleryPlugin, Sample, SeedToggle, spawn_gallery,
};
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{
    click, descendants, find, find_all, focus, headless_app, hover, press, settle, type_text, wait,
    words,
};

/// A headless app showing the gallery in a window `width` x `height`.
fn gallery_at(width: u32, height: u32) -> (App, Entity) {
    let mut app = headless_app(width, height, 1.0);
    app.add_plugins(GalleryPlugin);
    let root = app
        .world_mut()
        .run_system_once(|mut commands: Commands| spawn_gallery(&mut commands))
        .expect("the gallery is spawned");
    settle(&mut app);
    (app, root)
}

fn gallery() -> (App, Entity) {
    gallery_at(1440, 960)
}

fn log(app: &App) -> GalleryLog {
    app.world().resource::<GalleryLog>().clone()
}

fn last(app: &App) -> String {
    log(app).last().unwrap_or_default().to_owned()
}

/// Every part carrying the sample `name`.
fn samples(app: &App, root: Entity, name: &str) -> Vec<Entity> {
    find_all::<Sample>(app, root)
        .into_iter()
        .filter(|entity| {
            app.world()
                .get::<Sample>(*entity)
                .is_some_and(|sample| sample.0 == name)
        })
        .collect()
}

/// The part carrying the sample `name`.
fn sample(app: &App, root: Entity, name: &str) -> Entity {
    samples(app, root, name)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("a sample named {name:?}"))
}

/// The first part at or below `root` whose words are `label` and that is, or
/// is inside, something pressable.
fn saying(app: &App, root: Entity, label: &str) -> Entity {
    std::iter::once(root)
        .chain(descendants(app, root))
        .filter(|part| {
            app.world()
                .get::<Text>(*part)
                .is_some_and(|text| text.0 == label)
        })
        .find_map(|part| {
            std::iter::once(part)
                .chain(app.world().get::<ChildOf>(part).map(ChildOf::parent))
                .chain(
                    app.world()
                        .get::<ChildOf>(part)
                        .and_then(|parent| app.world().get::<ChildOf>(parent.parent()))
                        .map(ChildOf::parent),
                )
                .find(|at| app.world().get::<Pressable>(*at).is_some())
                .or(Some(part))
        })
        .unwrap_or_else(|| panic!("something saying {label:?}"))
}

/// The pressable parts below `root`, not `root` itself.
fn buttons_inside(app: &App, root: Entity) -> Vec<Entity> {
    find_all::<Pressable>(app, root)
        .into_iter()
        .filter(|entity| *entity != root)
        .collect()
}

fn checked(app: &App, entity: Entity) -> bool {
    app.world().get::<Checked>(entity).is_some()
}

/// The first part of `T` anywhere in the world.
fn anywhere<T: Component>(app: &mut App) -> Option<Entity> {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .next()
}

#[test]
fn the_gallery_builds_and_lays_out_at_every_window_size() {
    for &(width, height, _) in bevy_material_ui::testing::WINDOWS {
        let (app, root) = gallery_at(width, height);
        assert!(app.world().get::<Gallery>(root).is_some());
        let parts = descendants(&app, root);
        assert!(parts.len() > 600, "{width}x{height}: {} parts", parts.len());
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
fn the_gallery_shows_every_component_family() {
    let (app, root) = gallery();
    for name in [
        "filled button",
        "tonal button",
        "elevated button",
        "outlined button",
        "text button",
        "disabled button",
        "icon button",
        "fab",
        "extended fab",
        "view",
        "sizes",
        "checkbox",
        "indeterminate checkbox",
        "error checkbox",
        "labelled checkbox",
        "difficulty",
        "switch",
        "icon switch",
        "labelled switch",
        "volume",
        "view distance",
        "quality",
        "name",
        "seed",
        "motto",
        "port",
        "search",
        "result",
        "assist chip",
        "filter chip",
        "input chip",
        "suggestion chip",
        "players",
        "primary tabs",
        "secondary tabs",
        "edit menu",
        "game mode",
        "navigation bar",
        "navigation rail",
        "navigation drawer",
        "open dialog",
        "open sheet",
        "show snackbar",
        "plain tooltip",
        "rich tooltip",
    ] {
        assert!(
            !samples(&app, root, name).is_empty(),
            "no {name:?} in the gallery"
        );
    }
    for family in [
        find::<LinearProgress>(&app, root),
        find::<CircularProgress>(&app, root),
        find::<Badge>(&app, root),
        find::<Avatar>(&app, root),
        find::<Skeleton>(&app, root),
        find::<MaterialScrollbar>(&app, root),
        find::<AppBar>(&app, root),
    ] {
        assert!(family.is_some(), "a display family is missing");
    }
}

#[test]
fn every_kind_of_button_in_the_gallery_activates_when_clicked() {
    let (mut app, root) = gallery();
    for name in [
        "filled button",
        "tonal button",
        "elevated button",
        "outlined button",
        "text button",
        "icon button",
        "fab",
        "extended fab",
    ] {
        let button = sample(&app, root, name);
        click(&mut app, button);
        assert_eq!(last(&app), name);
    }
}

#[test]
fn a_disabled_button_in_the_gallery_does_nothing_when_clicked() {
    let (mut app, root) = gallery();
    let disabled = sample(&app, root, "disabled button");
    assert!(app.world().get::<InteractionDisabled>(disabled).is_some());
    let before = log(&app);
    click(&mut app, disabled);
    assert_eq!(log(&app), before);
}

#[test]
fn the_app_bar_turns_the_whole_gallery_dark_and_reseeds_it() {
    let (mut app, root) = gallery();
    let surface = |app: &App| app.world().get::<BackgroundColor>(root).expect("a fill").0;
    assert_eq!(
        surface(&app),
        MaterialTheme::light().color(ColorRole::Surface)
    );

    let toggle = find::<BrightnessToggle>(&app, root).expect("the brightness toggle");
    click(&mut app, toggle);
    assert_eq!(last(&app), "theme Dark");
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

    // Every part painted with a plain fill follows the new theme.
    for part in find_all::<Fill>(&app, root) {
        if app.world().get::<Look>(part).is_some() {
            continue;
        }
        let fill = app.world().get::<Fill>(part).expect("a fill").0;
        let drawn = app
            .world()
            .get::<BackgroundColor>(part)
            .expect("a background")
            .0;
        assert_eq!(drawn, fill.color(&theme), "{fill:?}");
    }
}

#[test]
fn the_d_and_s_keys_change_the_theme_but_not_while_typing_in_a_field() {
    let (mut app, root) = gallery();
    press(&mut app, KeyCode::KeyD, Key::Character("d".into()));
    assert_eq!(
        app.world().resource::<MaterialTheme>().brightness(),
        Brightness::Dark
    );
    press(&mut app, KeyCode::KeyS, Key::Character("s".into()));
    assert!(app.world().resource::<MaterialTheme>().seed.is_some());

    let name = sample(&app, root, "name");
    click(&mut app, name);
    let before = app.world().resource::<MaterialTheme>().clone();
    press(&mut app, KeyCode::KeyD, Key::Character("d".into()));
    assert_eq!(*app.world().resource::<MaterialTheme>(), before);
}

#[test]
fn checkboxes_toggle_on_a_click_or_on_their_label_and_a_disabled_one_refuses() {
    let (mut app, root) = gallery();
    let plain = sample(&app, root, "checkbox");
    click(&mut app, plain);
    assert_eq!(last(&app), "checkbox: true");
    assert!(checked(&app, plain));
    click(&mut app, plain);
    assert_eq!(last(&app), "checkbox: false");
    assert!(!checked(&app, plain));

    let labelled = sample(&app, root, "labelled checkbox");
    let label = saying(&app, labelled, "Show coordinates");
    click(&mut app, label);
    assert_eq!(last(&app), "labelled checkbox: true");

    let disabled = sample(&app, root, "disabled checkbox");
    let before = log(&app);
    click(&mut app, disabled);
    assert_eq!(log(&app), before);
    assert!(checked(&app, disabled), "it stays as it was");
}

#[test]
fn a_radio_group_chooses_the_option_clicked_and_skips_a_disabled_one() {
    let (mut app, root) = gallery();
    let group = sample(&app, root, "difficulty");
    let options = find_all::<RadioOption>(&app, group);
    let option = |app: &App, index: usize| {
        options
            .iter()
            .copied()
            .find(|entity| app.world().get::<RadioOption>(*entity) == Some(&RadioOption(index)))
            .expect("the option")
    };
    let easy = option(&app, 1);
    click(&mut app, easy);
    assert_eq!(last(&app), "difficulty: option 1");
    assert!(checked(&app, easy));
    assert!(!checked(&app, option(&app, 2)), "only one is chosen");

    let peaceful = option(&app, 0);
    let before = log(&app);
    click(&mut app, peaceful);
    assert_eq!(log(&app), before, "a disabled option can't be chosen");
}

#[test]
fn switches_turn_over_when_clicked() {
    let (mut app, root) = gallery();
    let off = sample(&app, root, "switch");
    click(&mut app, off);
    assert_eq!(last(&app), "switch: true");
    let on = sample(&app, root, "on switch");
    click(&mut app, on);
    assert_eq!(last(&app), "on switch: false");
    let labelled = sample(&app, root, "labelled switch");
    let music = saying(&app, labelled, "Music");
    click(&mut app, music);
    assert_eq!(last(&app), "labelled switch: true");
}

#[test]
fn sliders_move_with_the_arrow_keys_and_a_notched_one_goes_notch_to_notch() {
    let (mut app, root) = gallery();
    let volume = sample(&app, root, "volume");
    focus(&mut app, volume);
    press(&mut app, KeyCode::ArrowRight, Key::ArrowRight);
    let moved = last(&app);
    assert!(moved.starts_with("volume: "), "{moved}");
    let value: f32 = moved["volume: ".len()..].parse().expect("a number");
    assert!(value > 60.0, "{value}");

    let quality = sample(&app, root, "quality");
    focus(&mut app, quality);
    press(&mut app, KeyCode::End, Key::End);
    assert_eq!(last(&app), "quality: 4.0");
    press(&mut app, KeyCode::Home, Key::Home);
    assert_eq!(last(&app), "quality: 0.0");
}

#[test]
fn a_segmented_button_chooses_the_segment_clicked() {
    let (mut app, root) = gallery();
    let view = sample(&app, root, "view");
    let month = saying(&app, view, "Month");
    click(&mut app, month);
    assert_eq!(last(&app), "view: 2");
}

#[test]
fn a_text_field_takes_what_is_typed_and_its_clear_button_empties_it() {
    let (mut app, root) = gallery();
    let name = sample(&app, root, "name");
    click(&mut app, name);
    type_text(&mut app, "Ada");
    let typed = app
        .world()
        .get::<InputText>(name)
        .expect("its text")
        .text()
        .to_owned();
    assert_eq!(typed, "Ada");
    assert_eq!(last(&app), "name: \"Ada\"");

    let seed = sample(&app, root, "seed");
    let clear = buttons_inside(&app, seed)
        .into_iter()
        .next()
        .expect("a clear button");
    click(&mut app, clear);
    let cleared = app
        .world()
        .get::<InputText>(seed)
        .expect("its text")
        .text()
        .to_owned();
    assert_eq!(cleared, "");
    assert_eq!(last(&app), "seed: \"\"");
}

#[test]
fn chips_filter_when_clicked_and_an_input_chip_goes_when_removed() {
    let (mut app, root) = gallery();
    let filter = sample(&app, root, "filter chip");
    click(&mut app, filter);
    assert_eq!(last(&app), "filter chip: true");

    let assist = sample(&app, root, "assist chip");
    click(&mut app, assist);
    assert_eq!(last(&app), "assist chip");

    let input = samples(&app, root, "input chip");
    assert_eq!(input.len(), 2);
    let remove = buttons_inside(&app, input[0])
        .into_iter()
        .next()
        .expect("a remove button");
    click(&mut app, remove);
    assert_eq!(last(&app), "chip removed");
    assert_eq!(
        samples(&app, root, "input chip").len(),
        1,
        "the chip is gone"
    );
}

#[test]
fn a_list_row_is_chosen_when_clicked() {
    let (mut app, root) = gallery();
    let players = sample(&app, root, "players");
    let rows = find_all::<ListRow>(&app, players);
    assert_eq!(rows.len(), 4);
    click(&mut app, rows[1]);
    assert_eq!(last(&app), "players: chose");
    assert!(words(&app, rows[1]).contains("Grace"));
}

#[test]
fn a_tab_is_chosen_when_clicked_and_shows_its_panel() {
    let (mut app, root) = gallery();
    let primary = sample(&app, root, "primary tabs");
    let items = find_all::<MaterialTab>(&app, primary)[1];
    click(&mut app, items);
    assert_eq!(last(&app), "primary tabs: tab");
    let shown: Vec<String> = find_all::<TabPanel>(&app, root)
        .into_iter()
        .filter(|panel| {
            app.world()
                .get::<Node>(*panel)
                .is_some_and(|node| node.display != Display::None)
        })
        .map(|panel| words(&app, panel))
        .collect();
    assert_eq!(shown, ["Every item"]);
}

#[test]
fn a_menu_opens_and_choosing_an_item_sends_its_index() {
    let (mut app, root) = gallery();
    let menu = sample(&app, root, "edit menu");
    let opener = find::<MenuOpener>(&app, menu).expect("the menu's button");
    click(&mut app, opener);
    let surface = find::<MenuSurface>(&app, menu).expect("the menu");
    assert_eq!(
        app.world().get::<Visibility>(surface),
        Some(&Visibility::Visible)
    );
    let copy = find_all::<MenuChoice>(&app, menu)
        .into_iter()
        .find(|item| app.world().get::<MenuChoice>(*item) == Some(&MenuChoice(1)))
        .expect("Copy");
    click(&mut app, copy);
    assert_eq!(last(&app), "edit menu: 1");
    assert_eq!(
        app.world().get::<Visibility>(surface),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn a_select_shows_the_option_chosen_from_its_menu() {
    let (mut app, root) = gallery();
    let select = sample(&app, root, "game mode");
    let opener = find::<MenuOpener>(&app, select).expect("the field");
    click(&mut app, opener);
    let creative = find_all::<MenuChoice>(&app, select)
        .into_iter()
        .find(|item| app.world().get::<MenuChoice>(*item) == Some(&MenuChoice(1)))
        .expect("Creative");
    click(&mut app, creative);
    assert_eq!(last(&app), "game mode: 1");
    let field = app.world().get::<SelectField>(select).expect("the field");
    assert_eq!(field.chosen(), Some("Creative"));
}

#[test]
fn navigation_bars_rails_and_drawers_go_to_the_destination_clicked() {
    let (mut app, root) = gallery();
    for (name, index) in [
        ("navigation bar", 2),
        ("navigation rail", 3),
        ("navigation drawer", 2),
    ] {
        let navigation = sample(&app, root, name);
        let destination = find_all::<NavigationDestination>(&app, navigation)
            .into_iter()
            .find(|entity| {
                app.world().get::<NavigationDestination>(*entity)
                    == Some(&NavigationDestination(index))
            })
            .expect("the destination");
        click(&mut app, destination);
        assert_eq!(last(&app), format!("{name}: {index}"));
    }
    let rail = sample(&app, root, "navigation rail");
    let fab = find::<RailFab>(&app, rail).expect("the rail's FAB");
    click(&mut app, fab);
    assert_eq!(last(&app), "rail fab");
}

#[test]
fn an_app_bars_buttons_say_which_was_pressed() {
    let (mut app, root) = gallery();
    let bar = find::<AppBar>(&app, root).expect("an app bar");
    let buttons = find_all::<AppBarButton>(&app, bar);
    assert_eq!(buttons.len(), 2, "navigation and one action");
    click(&mut app, buttons[1]);
    assert_eq!(last(&app), "app bar: Action(0)");
}

#[test]
fn a_dialog_opens_on_a_scrim_and_answers_with_the_action_chosen_or_escape() {
    let (mut app, root) = gallery();
    let open = sample(&app, root, "open dialog");
    click(&mut app, open);
    let surface = anywhere::<DialogSurface>(&mut app).expect("the dialog");
    assert!(anywhere::<DialogScrim>(&mut app).is_some(), "on a scrim");
    let delete = saying(&app, surface, "Delete");
    click(&mut app, delete);
    assert_eq!(last(&app), format!("dialog: {CONFIRM}"));
    assert!(anywhere::<DialogSurface>(&mut app).is_none(), "and it goes");

    let open = sample(&app, root, "open rich dialog");
    click(&mut app, open);
    assert!(anywhere::<DialogSurface>(&mut app).is_some());
    wait(&mut app, 0.2);
    press(&mut app, KeyCode::Escape, Key::Escape);
    assert_eq!(last(&app), "dialog: dismissed");
    assert!(anywhere::<DialogSurface>(&mut app).is_none());
}

#[test]
fn a_bottom_sheet_rises_and_escape_closes_it() {
    let (mut app, root) = gallery();
    let open = sample(&app, root, "open sheet");
    click(&mut app, open);
    assert!(anywhere::<SheetSurface>(&mut app).is_some());
    wait(&mut app, 0.2);
    press(&mut app, KeyCode::Escape, Key::Escape);
    assert_eq!(last(&app), "sheet closed");
    assert!(anywhere::<SheetSurface>(&mut app).is_none());
}

#[test]
fn a_snackbar_shows_its_action_and_goes_by_itself() {
    let (mut app, root) = gallery();
    let undo = sample(&app, root, "show undo snackbar");
    click(&mut app, undo);
    let action = anywhere::<SnackbarActionButton>(&mut app).expect("the snackbar's action");
    click(&mut app, action);
    assert_eq!(last(&app), "snackbar: undo");

    let show = sample(&app, root, "show snackbar");
    click(&mut app, show);
    assert!(anywhere::<SnackbarView>(&mut app).is_some(), "it shows");
    wait(&mut app, SNACKBAR_SECONDS + 1.0);
    assert!(
        anywhere::<SnackbarView>(&mut app).is_none(),
        "and goes by itself"
    );
}

#[test]
fn a_tooltip_appears_after_hovering_a_while() {
    let (mut app, root) = gallery();
    let tipped = sample(&app, root, "plain tooltip");
    hover(&mut app, tipped);
    assert!(anywhere::<TooltipView>(&mut app).is_none(), "not at once");
    wait(&mut app, TOOLTIP_DELAY + 0.3);
    let tip = anywhere::<TooltipView>(&mut app).expect("the tooltip");
    assert!(words(&app, tip).contains("Add a world"));
}

#[test]
fn tab_moves_the_keyboards_focus_through_the_gallery_with_a_ring_on_it() {
    let (mut app, _) = gallery();
    let mut seen = Vec::new();
    for _ in 0..6 {
        press(&mut app, KeyCode::Tab, Key::Tab);
        let focused = app.world().resource::<InputFocus>().get();
        seen.push(focused);
    }
    let distinct: std::collections::HashSet<_> = seen.iter().flatten().collect();
    assert!(distinct.len() >= 5, "focus moved: {seen:?}");
    let now = seen.last().copied().flatten().expect("something focused");
    let ring = app.world().get::<Outline>(now).map(|outline| outline.color);
    assert!(
        ring.is_some_and(|color| color != Color::NONE),
        "the focused control shows its ring: {ring:?}"
    );
}

#[test]
fn every_part_of_the_gallery_hangs_from_a_ui_node() {
    let (mut app, root) = gallery();
    // Open every overlay too, so what they spawn is held to it.
    for name in ["open dialog", "open sheet", "show undo snackbar"] {
        let opener = sample(&app, root, name);
        click(&mut app, opener);
    }
    let tipped = sample(&app, root, "rich tooltip");
    hover(&mut app, tipped);
    wait(&mut app, TOOLTIP_DELAY + 0.3);
    let mut orphans = Vec::new();
    let mut nodes = app
        .world_mut()
        .query_filtered::<(Entity, &ChildOf), With<Node>>();
    for (entity, parent) in nodes.iter(app.world()) {
        if app.world().get::<Node>(parent.parent()).is_none() {
            orphans.push((entity, parent.parent()));
        }
    }
    let described: Vec<String> = orphans
        .iter()
        .map(|(entity, parent)| format!("{entity} under {parent}: {}", words(&app, *entity)))
        .collect();
    assert!(
        described.is_empty(),
        "UI nodes under non-UI parents: {described:#?}"
    );
}
