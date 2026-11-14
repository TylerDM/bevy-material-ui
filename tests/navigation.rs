//! Navigation, lists and menus, driven as a player would: rows chosen with
//! the mouse and the keys, tabs switched, an app bar's buttons used,
//! destinations picked from a bar, a rail and a drawer, and menus and a
//! select field opened, chosen from and closed.

use bevy::input::keyboard::Key;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui_widgets::SelectedTab;
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{
    click, find, find_all, focus, headless_app, press, rect, settle, spawn, words,
};

/// Every choice sent, by what sent it.
#[derive(Resource, Default)]
struct Sent {
    indices: Vec<(Entity, usize)>,
    rows: Vec<(Entity, Entity)>,
    tabs: Vec<(Entity, Option<Entity>)>,
    bar_buttons: Vec<(Entity, AppBarButton)>,
    fabs: Vec<Entity>,
}

fn app() -> App {
    let mut app = headless_app(1280, 900, 1.0);
    app.init_resource::<Sent>()
        .add_observer(|change: On<ValueChange<usize>>, mut sent: ResMut<Sent>| {
            sent.indices.push((change.source, change.value));
        })
        .add_observer(|change: On<ValueChange<Entity>>, mut sent: ResMut<Sent>| {
            sent.rows.push((change.source, change.value));
        })
        .add_observer(
            |change: On<ValueChange<Option<Entity>>>, mut sent: ResMut<Sent>| {
                sent.tabs.push((change.source, change.value));
            },
        )
        .add_observer(|action: On<AppBarAction>, mut sent: ResMut<Sent>| {
            sent.bar_buttons.push((action.bar, action.button));
        })
        .add_observer(|fab: On<RailFabActivated>, mut sent: ResMut<Sent>| {
            sent.fabs.push(fab.rail);
        });
    app
}

/// Spawns `bundle` inside `parent` and lays it out.
fn child(app: &mut App, parent: Entity, bundle: impl Bundle) -> Entity {
    let entity = app.world_mut().spawn((bundle, ChildOf(parent))).id();
    settle(app);
    entity
}

fn theme(app: &App) -> MaterialTheme {
    app.world().resource::<MaterialTheme>().clone()
}

fn background(app: &App, entity: Entity) -> Color {
    app.world()
        .get::<BackgroundColor>(entity)
        .expect("a background")
        .0
}

fn sent(app: &App) -> &Sent {
    app.world().resource::<Sent>()
}

fn focused(app: &App) -> Option<Entity> {
    app.world().resource::<InputFocus>().get()
}

fn shown(app: &App, entity: Entity) -> bool {
    app.world().get::<Visibility>(entity) != Some(&Visibility::Hidden)
}

// --- Lists -----------------------------------------------------------------

fn a_list(app: &mut App) -> (Entity, [Entity; 3]) {
    let column = spawn(
        app,
        Node {
            width: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    let list = child(app, column, list());
    let rows = [
        child(
            app,
            list,
            list_item("Inbox").leading_icon(Glyph::Menu).bundle(),
        ),
        child(
            app,
            list,
            list_item("Drafts")
                .supporting("Two unsent")
                .avatar("DR")
                .trailing_text("2")
                .bundle(),
        ),
        child(
            app,
            list,
            list_item("Spam")
                .three_line("Kept for thirty days, then deleted for good")
                .enabled(false)
                .bundle(),
        ),
    ];
    (list, rows)
}

#[test]
fn rows_are_as_tall_as_their_lines_and_say_their_words() {
    let mut app = app();
    let (_, [one, two, three]) = a_list(&mut app);

    assert_eq!(rect(&app, one).height(), 56.0);
    assert_eq!(rect(&app, two).height(), 72.0);
    assert!(rect(&app, three).height() >= 88.0);
    let said = words(&app, two);
    for part in ["DR", "Drafts", "Two unsent", "2"] {
        assert!(said.contains(part), "{part} in {said}");
    }
}

#[test]
fn a_clicked_row_is_selected_in_the_secondary_container_and_sent() {
    let mut app = app();
    let (list, [inbox, drafts, _]) = a_list(&mut app);

    click(&mut app, drafts);

    assert!(app.world().get::<Selected>(drafts).is_some());
    assert_eq!(sent(&app).rows, [(list, drafts)]);
    assert_eq!(
        *app.world().get::<Look>(drafts).expect("a look"),
        row_look(true)
    );
    assert_ne!(background(&app, drafts), Color::NONE);
    assert_eq!(background(&app, inbox), Color::NONE, "the others stay bare");

    click(&mut app, inbox);
    assert!(
        app.world().get::<Selected>(drafts).is_none(),
        "one row at a time"
    );
    assert_eq!(
        *app.world().get::<Look>(drafts).expect("a look"),
        row_look(false)
    );
}

#[test]
fn a_disabled_row_is_faded_and_cannot_be_chosen() {
    let mut app = app();
    let (_, [_, _, spam]) = a_list(&mut app);

    assert!(
        background(&app, spam).alpha() < 0.01,
        "a bare row stays bare"
    );
    let headline = find::<Text>(&app, spam).expect("words");
    let color = app.world().get::<TextColor>(headline).expect("a colour").0;
    assert!(color.alpha() < 0.5, "faded: {color:?}");

    click(&mut app, spam);
    assert!(app.world().get::<Selected>(spam).is_none());
    assert!(sent(&app).rows.is_empty());
}

#[test]
fn the_arrow_keys_move_through_a_focused_list_and_enter_chooses_a_row() {
    let mut app = app();
    let (list, [inbox, drafts, _]) = a_list(&mut app);

    focus(&mut app, list);
    press(&mut app, KeyCode::ArrowDown, Key::ArrowDown);
    press(&mut app, KeyCode::Enter, Key::Enter);

    assert_eq!(sent(&app).rows, [(list, drafts)]);
    assert!(app.world().get::<Selected>(drafts).is_some());
    let outline = app
        .world()
        .get::<Outline>(drafts)
        .expect("the active row's ring");
    assert_ne!(
        outline.color,
        Color::NONE,
        "the row the keyboard is on is ringed"
    );
    let other = app.world().get::<Outline>(inbox).expect("a ring");
    assert_eq!(other.color, Color::NONE);
}

// --- Tabs ------------------------------------------------------------------

fn some_tabs(app: &mut App, kind: TabsKind) -> (Entity, [Entity; 3], [Entity; 3]) {
    let page = spawn(
        app,
        Node {
            width: Val::Px(600.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    let row = child(app, page, tabs(kind).bundle());
    let headers = [
        child(app, row, tab(kind, "Video").icon(Glyph::Dot).bundle()),
        child(app, row, tab(kind, "Photos").bundle()),
        child(app, row, tab(kind, "Audio").bundle()),
    ];
    let panels_root = child(app, page, tab_panels(row));
    let panels = [0, 1, 2].map(|index| {
        child(
            app,
            panels_root,
            (
                tab_panel(index),
                children![text(
                    format!("Panel {index}"),
                    TypeRole::BodyLarge,
                    ColorRole::OnSurface
                )],
            ),
        )
    });
    settle(app);
    (row, headers, panels)
}

fn displayed(app: &App, panel: Entity) -> bool {
    app.world().get::<Node>(panel).expect("a node").display != Display::None
}

fn indicator_shown(app: &App, tab: Entity) -> bool {
    let indicator = find::<TabIndicator>(app, tab).expect("an indicator");
    shown(app, indicator)
}

#[test]
fn a_row_of_tabs_starts_on_its_first_tab_showing_only_that_panel() {
    let mut app = app();
    let (row, [video, photos, _], panels) = some_tabs(&mut app, TabsKind::Primary);

    assert_eq!(
        app.world().get::<SelectedTab>(row),
        Some(&SelectedTab(Some(video)))
    );
    assert!(indicator_shown(&app, video) && !indicator_shown(&app, photos));
    assert_eq!(
        panels.map(|panel| displayed(&app, panel)),
        [true, false, false]
    );
    let label = find_all::<Text>(&app, video)[0];
    assert_eq!(
        app.world().get::<TextColor>(label).expect("a colour").0,
        theme(&app).color(ColorRole::Primary),
        "a selected primary tab's words are primary"
    );
    let other = find::<Text>(&app, photos).expect("words");
    assert_eq!(
        app.world().get::<TextColor>(other).expect("a colour").0,
        theme(&app).color(ColorRole::OnSurfaceVariant)
    );
}

#[test]
fn a_clicked_tab_is_selected_sent_and_shows_its_panel() {
    let mut app = app();
    let (row, [video, photos, _], panels) = some_tabs(&mut app, TabsKind::Secondary);

    click(&mut app, photos);

    assert_eq!(sent(&app).tabs, [(row, Some(photos))]);
    assert_eq!(
        app.world().get::<SelectedTab>(row),
        Some(&SelectedTab(Some(photos)))
    );
    assert!(indicator_shown(&app, photos) && !indicator_shown(&app, video));
    assert_eq!(
        panels.map(|panel| displayed(&app, panel)),
        [false, true, false]
    );
    let label = find::<Text>(&app, photos).expect("words");
    assert_eq!(
        app.world().get::<TextColor>(label).expect("a colour").0,
        theme(&app).color(ColorRole::OnSurface),
        "a selected secondary tab's words are on-surface"
    );
}

#[test]
fn the_arrow_keys_move_between_tabs_and_enter_selects_one() {
    let mut app = app();
    let (row, [video, _, audio], panels) = some_tabs(&mut app, TabsKind::Primary);

    focus(&mut app, video);
    press(&mut app, KeyCode::ArrowLeft, Key::ArrowLeft);
    assert_eq!(
        focused(&app),
        Some(audio),
        "left from the first wraps to the last"
    );
    press(&mut app, KeyCode::Enter, Key::Enter);

    assert_eq!(
        app.world().get::<SelectedTab>(row),
        Some(&SelectedTab(Some(audio)))
    );
    assert_eq!(
        panels.map(|panel| displayed(&app, panel)),
        [false, false, true]
    );
}

// --- App bars --------------------------------------------------------------

#[test]
fn an_app_bars_buttons_are_sent_on_the_bar_saying_which() {
    let mut app = app();
    let bar = spawn(
        &mut app,
        top_app_bar("Settings")
            .navigation(Glyph::ArrowBack)
            .action(Glyph::Plus)
            .action(Glyph::MoreVert)
            .bundle(),
    );
    let buttons = find_all::<AppBarButton>(&app, bar);
    assert_eq!(buttons.len(), 3);

    click(&mut app, buttons[0]);
    click(&mut app, buttons[2]);

    assert_eq!(
        sent(&app).bar_buttons,
        [
            (bar, AppBarButton::Navigation),
            (bar, AppBarButton::Action(1))
        ]
    );
    assert_eq!(rect(&app, bar).height(), 64.0);
    assert_eq!(background(&app, bar), theme(&app).color(ColorRole::Surface));
    let title = find::<AppBarTitle>(&app, bar).expect("a title");
    assert_eq!(app.world().get::<Text>(title).expect("words").0, "Settings");
}

#[test]
fn medium_and_large_app_bars_are_taller_with_their_title_under_the_buttons() {
    let mut app = app();
    let medium = spawn(
        &mut app,
        top_app_bar("Library")
            .kind(AppBarKind::Medium)
            .navigation(Glyph::Menu)
            .bundle(),
    );
    let large = spawn(
        &mut app,
        top_app_bar("Library").kind(AppBarKind::Large).bundle(),
    );

    assert_eq!(rect(&app, medium).height(), 112.0);
    assert_eq!(rect(&app, large).height(), 152.0);
    let title = find::<AppBarTitle>(&app, medium).expect("a title");
    let button = find::<AppBarButton>(&app, medium).expect("a button");
    assert!(
        rect(&app, title).min.y >= rect(&app, button).max.y,
        "the title is under the buttons"
    );
}

// --- Navigation ------------------------------------------------------------

fn places() -> Vec<Destination> {
    vec![
        destination(Glyph::Menu, "Home"),
        destination(Glyph::Dot, "Map"),
        destination(Glyph::Plus, "Craft"),
    ]
}

fn destinations(app: &App, root: Entity) -> Vec<Entity> {
    find_all::<NavigationDestination>(app, root)
}

fn pill_background(app: &App, destination: Entity) -> Color {
    let pill = find::<ActiveIndicator>(app, destination).expect("an indicator");
    background(app, pill)
}

#[test]
fn a_navigation_bar_selects_and_sends_the_destination_clicked() {
    let mut app = app();
    let bar = spawn(&mut app, navigation_bar(places(), 0));
    let items = destinations(&app, bar);
    let secondary_container = theme(&app).color(ColorRole::SecondaryContainer);
    assert_eq!(pill_background(&app, items[0]), secondary_container);
    assert_eq!(pill_background(&app, items[2]), Color::NONE);
    assert_eq!(rect(&app, bar).height(), 80.0);

    click(&mut app, items[2]);

    assert_eq!(
        app.world().get::<NavigationSelection>(bar),
        Some(&NavigationSelection(2))
    );
    assert_eq!(sent(&app).indices, [(bar, 2)]);
    assert_eq!(pill_background(&app, items[0]), Color::NONE);
    assert_ne!(pill_background(&app, items[2]), Color::NONE);

    click(&mut app, items[2]);
    assert_eq!(
        sent(&app).indices.len(),
        1,
        "choosing the active one again sends nothing"
    );
}

#[test]
fn a_destination_chosen_from_the_keyboard_is_selected() {
    let mut app = app();
    let bar = spawn(&mut app, navigation_bar(places(), 0));
    let items = destinations(&app, bar);

    focus(&mut app, items[1]);
    press(&mut app, KeyCode::Enter, Key::Enter);

    assert_eq!(
        app.world().get::<NavigationSelection>(bar),
        Some(&NavigationSelection(1))
    );
}

#[test]
fn a_game_moving_elsewhere_redraws_the_active_destination() {
    let mut app = app();
    let rail = spawn(&mut app, navigation_rail(places(), 0, None));
    let items = destinations(&app, rail);

    app.world_mut()
        .entity_mut(rail)
        .insert(NavigationSelection(1));
    app.update();

    assert_eq!(pill_background(&app, items[0]), Color::NONE);
    assert_eq!(
        pill_background(&app, items[1]),
        theme(&app).color(ColorRole::SecondaryContainer)
    );
    assert!(
        sent(&app).indices.is_empty(),
        "the game's own move is not sent back"
    );
}

#[test]
fn a_rails_floating_action_button_is_sent_on_the_rail() {
    let mut app = app();
    let rail = spawn(
        &mut app,
        navigation_rail(places(), 0, Some(Glyph::Plus.into())),
    );
    let fab = find::<RailFab>(&app, rail).expect("a FAB");

    click(&mut app, fab);

    assert_eq!(sent(&app).fabs, [rail]);
    assert_eq!(rect(&app, rail).width(), 80.0);
    let items = destinations(&app, rail);
    click(&mut app, items[1]);
    assert_eq!(sent(&app).indices, [(rail, 1)]);
}

#[test]
fn a_drawers_destinations_are_counted_across_its_sections() {
    let mut app = app();
    let drawer = spawn(
        &mut app,
        navigation_drawer()
            .headline("Mail")
            .destination(destination(Glyph::Menu, "Inbox"))
            .destination(destination(Glyph::ArrowForward, "Sent"))
            .headline("Labels")
            .destination(destination(Glyph::Dot, "Family"))
            .selected(0)
            .bundle(),
    );
    let items = destinations(&app, drawer);
    assert_eq!(items.len(), 3);
    assert_eq!(rect(&app, drawer).width(), 360.0);
    assert_eq!(
        background(&app, drawer),
        theme(&app).color(ColorRole::SurfaceContainerLow)
    );

    click(&mut app, items[2]);

    assert_eq!(sent(&app).indices, [(drawer, 2)]);
    assert_eq!(
        *app.world().get::<Look>(items[2]).expect("a look"),
        indicator_look(true)
    );
    assert_eq!(
        *app.world().get::<Look>(items[0]).expect("a look"),
        indicator_look(false)
    );
    assert!(words(&app, drawer).contains("Labels"));
}

// --- Menus -----------------------------------------------------------------

fn a_menu(app: &mut App) -> (Entity, Entity, Entity, Vec<Entity>) {
    let root = spawn(
        app,
        menu(button("Edit").outlined())
            .item(menu_item("Cut").leading(Glyph::Close).shortcut("Ctrl+X"))
            .divider()
            .item(menu_item("Copy").shortcut("Ctrl+C"))
            .item(menu_item("Paste").enabled(false))
            .bundle(),
    );
    let opener = find::<MenuOpener>(app, root).expect("a button");
    let popup = find::<MenuSurface>(app, root).expect("a popup");
    let items = find_all::<MenuChoice>(app, root);
    (root, opener, popup, items)
}

#[test]
fn a_menu_opens_under_its_button_with_the_first_item_focused() {
    let mut app = app();
    let (_, opener, popup, items) = a_menu(&mut app);
    assert!(!shown(&app, popup), "closed until opened");

    click(&mut app, opener);
    settle(&mut app);

    assert!(shown(&app, popup));
    assert_eq!(focused(&app), Some(items[0]));
    assert!(
        rect(&app, popup).min.y >= rect(&app, opener).max.y,
        "below its button"
    );
    assert_eq!(
        background(&app, popup),
        theme(&app).color(ColorRole::SurfaceContainer)
    );
    assert!(
        !app.world()
            .get::<BoxShadow>(popup)
            .expect("a shadow")
            .0
            .is_empty()
    );
}

#[test]
fn choosing_a_menu_item_sends_its_index_and_closes_the_menu() {
    let mut app = app();
    let (root, opener, popup, items) = a_menu(&mut app);

    click(&mut app, opener);
    settle(&mut app);
    click(&mut app, items[1]);

    assert_eq!(sent(&app).indices, [(root, 1)], "dividers are not counted");
    assert!(!shown(&app, popup));
}

#[test]
fn a_disabled_menu_item_cannot_be_chosen() {
    let mut app = app();
    let (_, opener, popup, items) = a_menu(&mut app);

    click(&mut app, opener);
    settle(&mut app);
    click(&mut app, items[2]);

    assert!(sent(&app).indices.is_empty());
    assert!(shown(&app, popup), "it stays open");
}

#[test]
fn escape_closes_a_menu_and_gives_the_focus_back_to_its_button() {
    let mut app = app();
    let (_, opener, popup, _) = a_menu(&mut app);

    click(&mut app, opener);
    settle(&mut app);
    press(&mut app, KeyCode::Escape, Key::Escape);

    assert!(!shown(&app, popup));
    assert_eq!(focused(&app), Some(opener));
}

#[test]
fn the_arrow_keys_open_a_menu_and_move_through_it_and_enter_chooses() {
    let mut app = app();
    let (root, opener, popup, items) = a_menu(&mut app);

    focus(&mut app, opener);
    press(&mut app, KeyCode::ArrowDown, Key::ArrowDown);
    settle(&mut app);
    assert!(shown(&app, popup));
    assert_eq!(focused(&app), Some(items[0]));

    press(&mut app, KeyCode::ArrowDown, Key::ArrowDown);
    assert_eq!(focused(&app), Some(items[1]));
    press(&mut app, KeyCode::Enter, Key::Enter);

    assert_eq!(sent(&app).indices, [(root, 1)]);
    assert!(!shown(&app, popup));
}

#[test]
fn a_select_field_shows_the_option_chosen_from_its_menu() {
    let mut app = app();
    let field = spawn(
        &mut app,
        select("Quality", ["Low", "Medium", "High"])
            .selected(0)
            .bundle(),
    );
    let opener = find::<MenuOpener>(&app, field).expect("a field");
    let value = find::<SelectValue>(&app, field).expect("its value");
    assert_eq!(app.world().get::<Text>(value).expect("words").0, "Low");
    assert!(words(&app, opener).contains("Quality"));

    click(&mut app, opener);
    settle(&mut app);
    let items = find_all::<MenuChoice>(&app, field);
    assert_eq!(
        *app.world().get::<Look>(items[0]).expect("a look"),
        item_look(true)
    );
    click(&mut app, items[2]);
    settle(&mut app);

    assert_eq!(sent(&app).indices, [(field, 2)]);
    assert_eq!(
        app.world()
            .get::<SelectField>(field)
            .expect("a field")
            .selected,
        Some(2)
    );
    assert_eq!(app.world().get::<Text>(value).expect("words").0, "High");
    assert_eq!(
        *app.world().get::<Look>(items[2]).expect("a look"),
        item_look(true)
    );
    assert_eq!(
        *app.world().get::<Look>(items[0]).expect("a look"),
        item_look(false)
    );
    let popup = find::<MenuSurface>(&app, field).expect("a popup");
    assert!(!shown(&app, popup));
}

#[test]
fn turning_the_theme_dark_repaints_the_navigation() {
    let mut app = app();
    let bar = spawn(&mut app, navigation_bar(places(), 0));
    let items = destinations(&app, bar);

    app.insert_resource(MaterialTheme::dark());
    app.update();

    let dark = MaterialTheme::dark();
    assert_eq!(
        background(&app, bar),
        dark.color(ColorRole::SurfaceContainer)
    );
    assert_eq!(
        pill_background(&app, items[0]),
        dark.color(ColorRole::SecondaryContainer)
    );
}
