//! Text fields, search and chips, used as a player would: clicked, typed
//! into, cleared, toggled and removed with the mouse and the keyboard.

use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui::Checked;
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{
    click, descendants, find, headless_app, press, rect, spawn, type_text,
};

/// Everything the components told the game, in order.
#[derive(Resource, Default)]
struct Heard {
    texts: Vec<(Entity, String, bool)>,
    toggles: Vec<(Entity, bool)>,
    activations: Vec<Entity>,
    removed: Vec<Entity>,
    navigated: Vec<Entity>,
}

fn app() -> App {
    let mut app = headless_app(1000, 700, 1.0);
    app.init_resource::<Heard>()
        .add_observer(
            |change: On<ValueChange<String>>, mut heard: ResMut<Heard>| {
                heard
                    .texts
                    .push((change.source, change.value.clone(), change.is_final));
            },
        )
        .add_observer(|change: On<ValueChange<bool>>, mut heard: ResMut<Heard>| {
            heard.toggles.push((change.source, change.value));
        })
        .add_observer(|activate: On<Activate>, mut heard: ResMut<Heard>| {
            heard.activations.push(activate.entity);
        })
        .add_observer(|removed: On<ChipRemoved>, mut heard: ResMut<Heard>| {
            heard.removed.push(removed.entity);
        })
        .add_observer(|navigate: On<SearchNavigate>, mut heard: ResMut<Heard>| {
            heard.navigated.push(navigate.entity);
        });
    app
}

/// A column to lay several parts out in, one under another, so none covers
/// another.
fn column(app: &mut App) -> Entity {
    spawn(
        app,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        },
    )
}

/// Spawns `bundle` in `column` and lays it out.
fn add(app: &mut App, column: Entity, bundle: impl Bundle) -> Entity {
    let entity = app.world_mut().spawn((bundle, ChildOf(column))).id();
    bevy_material_ui::testing::settle(app);
    entity
}

fn theme(app: &App) -> MaterialTheme {
    app.world().resource::<MaterialTheme>().clone()
}

fn heard(app: &App) -> &Heard {
    app.world().resource::<Heard>()
}

/// The text entity at or below `root` saying exactly `words`.
fn saying(app: &App, root: Entity, words: &str) -> Entity {
    std::iter::once(root)
        .chain(descendants(app, root))
        .find(|part| {
            app.world()
                .get::<Text>(*part)
                .is_some_and(|text| text.0 == words)
        })
        .unwrap_or_else(|| panic!("nothing says {words:?}"))
}

fn text_of(app: &App, field: Entity) -> String {
    app.world()
        .get::<InputText>(field)
        .expect("a text input")
        .text()
        .to_owned()
}

fn input_of(app: &App, field: Entity) -> Entity {
    app.world()
        .get::<InputText>(field)
        .expect("a text input")
        .input()
}

/// A field's frame: the box the text is typed in.
fn frame(app: &App, field: Entity) -> Entity {
    app.world().get::<Children>(field).expect("parts")[0]
}

fn font_size(app: &App, text: Entity) -> f32 {
    match app.world().get::<TextFont>(text).expect("a font").font_size {
        FontSize::Px(size) => size,
        other => panic!("{other:?}"),
    }
}

fn color(app: &App, text: Entity) -> Color {
    app.world().get::<TextColor>(text).expect("a colour").0
}

fn border(app: &App, entity: Entity) -> (Color, UiRect) {
    let world = app.world();
    (
        world.get::<BorderColor>(entity).expect("a border").bottom,
        world.get::<Node>(entity).expect("a node").border,
    )
}

fn focused(app: &App) -> Option<Entity> {
    app.world()
        .resource::<bevy::input_focus::InputFocus>()
        .get()
}

fn displayed(app: &App, entity: Entity) -> bool {
    app.world().get::<Node>(entity).expect("a node").display != Display::None
}

fn visible(app: &App, entity: Entity) -> bool {
    app.world().get::<Visibility>(entity) != Some(&Visibility::Hidden)
}

#[test]
fn a_label_sits_in_an_empty_field_and_floats_small_and_primary_once_it_is_focused() {
    let mut app = app();
    let field = spawn(&mut app, text_field("Name").bundle());
    let label = saying(&app, field, "Name");
    let theme = theme(&app);

    assert_eq!(font_size(&app, label), TypeRole::BodyLarge.size());
    assert_eq!(color(&app, label), theme.color(ColorRole::OnSurfaceVariant));
    assert_eq!(
        border(&app, frame(&app, field)),
        (
            theme.color(ColorRole::OnSurfaceVariant),
            UiRect::bottom(Val::Px(1.0))
        )
    );

    let target = frame(&app, field);
    click(&mut app, target);

    assert_eq!(focused(&app), Some(input_of(&app, field)));
    assert_eq!(font_size(&app, label), TypeRole::BodySmall.size());
    assert_eq!(color(&app, label), theme.color(ColorRole::Primary));
    assert_eq!(
        border(&app, frame(&app, field)),
        (
            theme.color(ColorRole::Primary),
            UiRect::bottom(Val::Px(2.0))
        )
    );
}

#[test]
fn typing_into_a_focused_field_changes_its_text_and_tells_the_game_and_enter_ends_the_edit() {
    let mut app = app();
    let field = spawn(&mut app, text_field("Name").bundle());
    let target = frame(&app, field);
    click(&mut app, target);

    type_text(&mut app, "Ada");
    assert_eq!(text_of(&app, field), "Ada");
    let texts: Vec<_> = heard(&app)
        .texts
        .iter()
        .map(|(_, text, _)| text.clone())
        .collect();
    assert_eq!(texts, ["A", "Ad", "Ada"]);
    assert!(
        heard(&app)
            .texts
            .iter()
            .all(|(source, _, last)| *source == field && !last)
    );

    press(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(
        heard(&app).texts.last(),
        Some(&(field, "Ada".to_owned(), true))
    );
}

#[test]
fn a_filled_field_keeps_its_label_floated_when_it_loses_the_keyboard() {
    let mut app = app();
    let list = column(&mut app);
    let field = add(&mut app, list, text_field("Name").bundle());
    let other = add(&mut app, list, text_field("Other").bundle());
    let label = saying(&app, field, "Name");
    let target = frame(&app, field);
    click(&mut app, target);
    type_text(&mut app, "x");

    let target = frame(&app, other);
    click(&mut app, target);

    assert_eq!(focused(&app), Some(input_of(&app, other)));
    assert_eq!(
        font_size(&app, label),
        TypeRole::BodySmall.size(),
        "still floated"
    );
    assert_eq!(
        color(&app, label),
        theme(&app).color(ColorRole::OnSurfaceVariant)
    );
    assert_eq!(
        heard(&app).texts.last(),
        Some(&(field, "x".to_owned(), true)),
        "blur ends it"
    );
}

#[test]
fn a_clear_button_shows_only_while_a_field_holds_text_and_empties_it() {
    let mut app = app();
    let field = spawn(
        &mut app,
        text_field("Search").text("abc").clearable().bundle(),
    );
    let clear = find::<bevy::ui_widgets::Button>(&app, frame(&app, field)).expect("a clear button");
    assert!(displayed(&app, clear));

    click(&mut app, clear);

    assert_eq!(text_of(&app, field), "");
    assert!(!displayed(&app, clear));
    assert_eq!(
        focused(&app),
        Some(input_of(&app, field)),
        "the field keeps the keyboard"
    );
    assert_eq!(
        heard(&app).texts.last(),
        Some(&(field, String::new(), false))
    );
}

#[test]
fn a_field_in_error_is_drawn_in_the_error_colours_and_says_its_error_instead_of_its_hint() {
    let mut app = app();
    let field = spawn(
        &mut app,
        text_field("Email")
            .supporting("Your sign-in")
            .error("Not an address")
            .bundle(),
    );
    let theme = theme(&app);
    let message = saying(&app, field, "Not an address");
    assert_eq!(color(&app, message), theme.color(ColorRole::Error));
    assert_eq!(
        color(&app, saying(&app, field, "Email")),
        theme.color(ColorRole::Error)
    );
    assert_eq!(
        border(&app, frame(&app, field)).0,
        theme.color(ColorRole::Error)
    );

    app.world_mut()
        .get_mut::<TextField>(field)
        .expect("a field")
        .error = None;
    app.update();

    let hint = saying(&app, field, "Your sign-in");
    assert_eq!(color(&app, hint), theme.color(ColorRole::OnSurfaceVariant));
    assert_eq!(
        border(&app, frame(&app, field)).0,
        theme.color(ColorRole::OnSurfaceVariant)
    );
}

#[test]
fn a_field_with_a_most_refuses_more_and_counts_what_it_holds() {
    let mut app = app();
    let field = spawn(&mut app, text_field("Code").max_length(4).bundle());
    assert!(
        std::iter::once(field)
            .chain(descendants(&app, field))
            .any(|part| app
                .world()
                .get::<Text>(part)
                .is_some_and(|text| text.0 == "0/4"))
    );

    let target = frame(&app, field);
    click(&mut app, target);
    type_text(&mut app, "abcdef");

    assert_eq!(text_of(&app, field), "abcd");
    saying(&app, field, "4/4");
}

#[test]
fn a_disabled_field_takes_neither_the_keyboard_nor_typing_and_is_faded() {
    let mut app = app();
    let field = spawn(&mut app, text_field("Name").enabled(false).bundle());

    let target = frame(&app, field);
    click(&mut app, target);
    type_text(&mut app, "no");

    assert_ne!(focused(&app), Some(input_of(&app, field)));
    assert_eq!(text_of(&app, field), "");
    assert!(color(&app, saying(&app, field, "Name")).alpha() < 0.5);
}

#[test]
fn an_outlined_field_is_bordered_all_round_and_focus_thickens_it_in_primary() {
    let mut app = app();
    let field = spawn(&mut app, text_field("City").outlined().bundle());
    let theme = theme(&app);
    assert_eq!(
        border(&app, frame(&app, field)),
        (theme.color(ColorRole::Outline), UiRect::all(Val::Px(1.0)))
    );

    let target = frame(&app, field);
    click(&mut app, target);

    assert_eq!(
        border(&app, frame(&app, field)),
        (theme.color(ColorRole::Primary), UiRect::all(Val::Px(2.0)))
    );
    let label = saying(&app, field, "City");
    assert!(
        rect(&app, label).center().y < rect(&app, frame(&app, field)).min.y + 2.0,
        "the floated label sits on the outline"
    );
}

#[test]
fn a_placeholder_shows_in_a_focused_empty_field_and_goes_once_it_is_typed_over() {
    let mut app = app();
    let field = spawn(
        &mut app,
        text_field("Name").placeholder("Ada Lovelace").bundle(),
    );
    let placeholder = saying(&app, field, "Ada Lovelace");
    assert!(
        !visible(&app, placeholder),
        "the label stands in for it unfocused"
    );

    let target = frame(&app, field);
    click(&mut app, target);
    assert!(visible(&app, placeholder));

    type_text(&mut app, "G");
    assert!(!visible(&app, placeholder));
}

#[test]
fn text_set_by_the_game_shows_in_the_field_and_is_told_as_a_change() {
    let mut app = app();
    let field = spawn(&mut app, text_field("Name").bundle());

    set_input_text(&mut app.world_mut().commands(), field, "Grace");
    app.update();
    app.update();

    assert_eq!(text_of(&app, field), "Grace");
    assert_eq!(
        heard(&app).texts.last(),
        Some(&(field, "Grace".to_owned(), false))
    );
    let label = saying(&app, field, "Name");
    assert_eq!(
        font_size(&app, label),
        TypeRole::BodySmall.size(),
        "filled, so floated"
    );
}

#[test]
fn tab_moves_the_keyboard_from_one_field_to_the_next() {
    let mut app = app();
    let list = column(&mut app);
    let first = add(&mut app, list, text_field("First").bundle());
    let second = add(&mut app, list, text_field("Second").bundle());
    let target = frame(&app, first);
    click(&mut app, target);

    press(&mut app, KeyCode::Tab, Key::Tab);

    assert_eq!(focused(&app), Some(input_of(&app, second)));
}

#[test]
fn a_search_bar_takes_a_query_shows_its_clear_button_and_hides_its_placeholder() {
    let mut app = app();
    let root = spawn(
        &mut app,
        Node {
            width: Val::Px(480.0),
            ..default()
        },
    );
    let bar = app
        .world_mut()
        .spawn((search_bar("Search blocks", Glyph::Menu), ChildOf(root)))
        .id();
    bevy_material_ui::testing::settle(&mut app);
    let found = *app.world().get::<SearchBar>(bar).expect("a search bar");
    let placeholder = saying(&app, bar, "Search blocks");
    assert!(visible(&app, placeholder));
    assert!(!displayed(&app, found.clear()));
    assert_eq!(
        app.world().get::<BackgroundColor>(bar).expect("a fill").0,
        theme(&app).color(ColorRole::SurfaceContainerHigh)
    );

    let target = input_of(&app, bar);
    click(&mut app, target);
    type_text(&mut app, "oak");

    assert_eq!(text_of(&app, bar), "oak");
    assert_eq!(
        heard(&app).texts.last(),
        Some(&(bar, "oak".to_owned(), false))
    );
    assert!(!visible(&app, placeholder));
    assert!(displayed(&app, found.clear()));

    click(&mut app, found.clear());
    assert_eq!(text_of(&app, bar), "");

    click(&mut app, found.leading());
    assert_eq!(heard(&app).navigated, [bar]);
}

#[test]
fn a_search_result_is_a_row_that_activates_when_chosen() {
    let mut app = app();
    let results = spawn(&mut app, search_results());
    let row = app
        .world_mut()
        .spawn((search_result("Oak log", Some("Wood")), ChildOf(results)))
        .id();
    bevy_material_ui::testing::settle(&mut app);
    saying(&app, row, "Wood");

    click(&mut app, row);

    assert_eq!(heard(&app).activations, [row]);
}

#[test]
fn a_filter_chip_turns_on_and_off_when_clicked_and_shows_its_check_while_on() {
    let mut app = app();
    let chip = spawn(&mut app, chip("Wood").filter(false).bundle());
    let theme = theme(&app);
    let check = find::<IconRoot>(&app, chip).expect("a check");
    assert!(!displayed(&app, check));
    assert_eq!(
        app.world().get::<Node>(chip).expect("a node").border,
        UiRect::all(Val::Px(1.0))
    );

    click(&mut app, chip);
    app.update();

    assert!(app.world().get::<Checked>(chip).is_some());
    assert_eq!(heard(&app).toggles, [(chip, true)]);
    assert!(displayed(&app, check));
    let hovered =
        chip_look(ChipKind::Filter, false, true).container_color(&theme, true, false, false);
    assert_eq!(
        app.world().get::<BackgroundColor>(chip).expect("a fill").0,
        hovered
    );
    assert_eq!(
        app.world().get::<Node>(chip).expect("a node").border,
        UiRect::ZERO
    );

    click(&mut app, chip);
    app.update();
    assert!(app.world().get::<Checked>(chip).is_none());
    assert_eq!(heard(&app).toggles, [(chip, true), (chip, false)]);
    assert!(!displayed(&app, check));
}

#[test]
fn space_toggles_a_focused_filter_chip() {
    let mut app = app();
    let chip = spawn(&mut app, chip("Stone").filter(true).bundle());
    bevy_material_ui::testing::focus(&mut app, chip);

    press(&mut app, KeyCode::Space, Key::Space);
    app.update();

    assert_eq!(heard(&app).toggles, [(chip, false)]);
    assert!(app.world().get::<Checked>(chip).is_none());
}

#[test]
fn an_input_chips_remove_button_tells_the_game_it_is_removed() {
    let mut app = app();
    let chip = spawn(&mut app, chip("Ada").input().bundle());
    let remove = descendants(&app, chip)
        .into_iter()
        .find(|part| app.world().get::<bevy::ui_widgets::Button>(*part).is_some())
        .expect("a remove button");

    click(&mut app, remove);

    assert_eq!(heard(&app).removed, [chip]);
    assert!(
        !heard(&app).activations.contains(&chip),
        "the chip itself is not used"
    );
}

#[test]
fn an_assist_chip_activates_when_clicked_and_a_disabled_one_does_not() {
    let mut app = app();
    let list = column(&mut app);
    let assist = add(
        &mut app,
        list,
        chip("Add to map").icon(Glyph::Plus).bundle(),
    );
    let disabled = add(
        &mut app,
        list,
        chip("Share").suggestion().enabled(false).bundle(),
    );

    click(&mut app, assist);
    click(&mut app, disabled);

    assert_eq!(heard(&app).activations, [assist]);
    assert_eq!(
        app.world().get::<Node>(assist).expect("a node").height,
        Val::Px(CHIP_HEIGHT)
    );
}

#[test]
fn an_elevated_chip_casts_a_shadow_instead_of_an_outline() {
    let mut app = app();
    let chip = spawn(&mut app, chip("Lift").elevated().bundle());

    assert!(
        !app.world()
            .get::<BoxShadow>(chip)
            .expect("a shadow")
            .0
            .is_empty()
    );
    assert_eq!(
        app.world().get::<Node>(chip).expect("a node").border,
        UiRect::ZERO
    );
}

#[test]
fn a_chip_set_wraps_its_chips_onto_another_row_when_they_fill_it() {
    let mut app = app();
    let set = spawn(&mut app, chip_set());
    app.world_mut().entity_mut(set).insert(Node {
        width: Val::Px(240.0),
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(8.0),
        row_gap: Val::Px(8.0),
        ..default()
    });
    let chips: Vec<Entity> = ["Granite", "Diorite", "Andesite", "Basalt"]
        .iter()
        .map(|words| {
            app.world_mut()
                .spawn((chip(*words).bundle(), ChildOf(set)))
                .id()
        })
        .collect();
    bevy_material_ui::testing::settle(&mut app);

    let first = rect(&app, chips[0]);
    let last = rect(&app, chips[3]);
    assert!(last.min.y >= first.max.y, "{first:?} then {last:?}");
}
