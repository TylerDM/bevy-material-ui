//! The selection controls, used as a player would: checkboxes, radio
//! buttons, switches, sliders and segmented buttons clicked, dragged and
//! moved with the keyboard, each checked for what it sends, what it keeps
//! and what colour it is drawn.

use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy::ui::{Checked, InteractionDisabled};
use bevy::ui_widgets::{Slider, SliderValue};
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{
    centre, click, drag, find, find_all, focus, headless_app, press, rect, spawn, within,
};

/// Every `ValueChange<T>` sent, in order: its source, value and finality.
#[derive(Resource)]
struct Sent<T: Send + Sync + 'static>(Vec<(Entity, T, bool)>);

impl<T: Send + Sync + 'static> Default for Sent<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

fn record<T: Copy + Send + Sync + 'static>(app: &mut App) {
    app.init_resource::<Sent<T>>().add_observer(
        |change: On<ValueChange<T>>, mut sent: ResMut<Sent<T>>| {
            sent.0.push((change.source, change.value, change.is_final));
        },
    );
}

fn app() -> App {
    let mut app = headless_app(800, 600, 1.0);
    record::<bool>(&mut app);
    record::<f32>(&mut app);
    record::<Entity>(&mut app);
    record::<usize>(&mut app);
    app
}

fn sent<T: Copy + Send + Sync + 'static>(app: &App) -> Vec<(Entity, T, bool)> {
    app.world().resource::<Sent<T>>().0.clone()
}

fn color(app: &App, role: ColorRole) -> Color {
    app.world().resource::<MaterialTheme>().color(role)
}

fn background(app: &App, entity: Entity) -> Color {
    app.world()
        .get::<BackgroundColor>(entity)
        .expect("a background")
        .0
}

fn checked(app: &App, entity: Entity) -> bool {
    app.world().get::<Checked>(entity).is_some()
}

fn shown(app: &App, entity: Entity) -> bool {
    app.world().get::<Visibility>(entity) != Some(&Visibility::Hidden)
}

/// The first part at or below `root` with `T` and matching `pick`.
fn find_where<T: Component>(app: &App, root: Entity, pick: impl Fn(&T) -> bool) -> Entity {
    find_all::<T>(app, root)
        .into_iter()
        .find(|entity| pick(app.world().get::<T>(*entity).expect("found")))
        .expect("a matching part")
}

// Checkboxes.

#[test]
fn a_clicked_checkbox_checks_itself_says_so_and_fills_its_box_with_primary() {
    let mut app = app();
    let checkbox = spawn(&mut app, checkbox().bundle());
    let tick = find_where::<CheckboxMark>(&app, checkbox, |mark| mark.0 == Glyph::Check);
    let square = find::<CheckboxBox>(&app, checkbox).expect("a box");
    assert!(!shown(&app, tick), "no tick while unchecked");

    click(&mut app, checkbox);

    assert!(checked(&app, checkbox));
    assert_eq!(sent::<bool>(&app), [(checkbox, true, true)]);
    assert_eq!(background(&app, square), color(&app, ColorRole::Primary));
    assert!(shown(&app, tick), "ticked");

    click(&mut app, checkbox);
    assert!(!checked(&app, checkbox), "a second click unchecks it");
    assert!(!shown(&app, tick));
}

#[test]
fn space_unchecks_a_focused_checked_checkbox() {
    let mut app = app();
    let checkbox = spawn(&mut app, checkbox().checked(true).bundle());
    assert!(checked(&app, checkbox), "it starts checked");

    focus(&mut app, checkbox);
    press(&mut app, KeyCode::Space, Key::Space);

    assert!(!checked(&app, checkbox));
    assert_eq!(sent::<bool>(&app), [(checkbox, false, true)]);
}

#[test]
fn a_controlled_checkbox_says_it_was_clicked_and_waits_for_the_game() {
    let mut app = app();
    let checkbox = spawn(&mut app, checkbox().controlled().bundle());

    click(&mut app, checkbox);

    assert_eq!(sent::<bool>(&app), [(checkbox, true, true)]);
    assert!(!checked(&app, checkbox), "not until the game checks it");
    app.world_mut().entity_mut(checkbox).insert(Checked);
    app.update();
    let square = find::<CheckboxBox>(&app, checkbox).expect("a box");
    assert_eq!(background(&app, square), color(&app, ColorRole::Primary));
}

#[test]
fn a_disabled_checkbox_is_faded_and_ignores_clicks() {
    let mut app = app();
    let checkbox = spawn(&mut app, checkbox().enabled(false).bundle());
    let square = find::<CheckboxBox>(&app, checkbox).expect("a box");

    click(&mut app, checkbox);

    assert!(!checked(&app, checkbox));
    assert!(sent::<bool>(&app).is_empty());
    let outline = app
        .world()
        .get::<BorderColor>(square)
        .expect("a border")
        .top;
    assert!((outline.alpha() - 0.38).abs() < 0.01, "faded: {outline:?}");
}

#[test]
fn clicking_a_checkboxs_label_flips_it() {
    let mut app = app();
    let row = spawn(&mut app, checkbox().labelled("Show the map"));
    let checkbox = find::<bevy::ui_widgets::Checkbox>(&app, row).expect("a checkbox");
    let words = find::<Text>(&app, row).expect("a label");

    click(&mut app, words);

    assert!(checked(&app, checkbox));
    assert_eq!(sent::<bool>(&app), [(checkbox, true, true)]);
}

#[test]
fn an_indeterminate_checkbox_shows_a_dash_and_an_error_one_is_outlined_in_error() {
    let mut app = app();
    let mixed = spawn(&mut app, checkbox().indeterminate(true).bundle());
    let dash = find_where::<CheckboxMark>(&app, mixed, |mark| mark.0 == Glyph::Dash);
    let tick = find_where::<CheckboxMark>(&app, mixed, |mark| mark.0 == Glyph::Check);
    assert!(shown(&app, dash) && !shown(&app, tick));
    let square = find::<CheckboxBox>(&app, mixed).expect("a box");
    assert_eq!(background(&app, square), color(&app, ColorRole::Primary));

    let wrong = spawn(&mut app, checkbox().error(true).bundle());
    let square = find::<CheckboxBox>(&app, wrong).expect("a box");
    assert_eq!(
        app.world()
            .get::<BorderColor>(square)
            .expect("a border")
            .top,
        color(&app, ColorRole::Error)
    );
}

#[test]
fn a_checked_checkbox_is_repainted_when_the_theme_turns_dark() {
    let mut app = app();
    let checkbox = spawn(&mut app, checkbox().checked(true).bundle());

    app.insert_resource(MaterialTheme::dark());
    app.update();

    let square = find::<CheckboxBox>(&app, checkbox).expect("a box");
    assert_eq!(
        background(&app, square),
        MaterialTheme::dark().color(ColorRole::Primary)
    );
}

// Radio buttons.

fn option(app: &App, group: Entity, index: usize) -> Entity {
    find_where::<RadioOption>(app, group, |option| option.0 == index)
}

#[test]
fn clicking_an_option_chooses_it_unchecks_the_rest_and_the_group_names_it() {
    let mut app = app();
    let group = spawn(
        &mut app,
        radio_group(["Easy", "Normal", "Hard"], Some(0)).bundle(),
    );
    let (easy, hard) = (option(&app, group, 0), option(&app, group, 2));
    assert!(checked(&app, easy));

    click(&mut app, hard);

    assert!(checked(&app, hard) && !checked(&app, easy));
    assert_eq!(sent::<Entity>(&app), [(group, hard, true)]);
    assert_eq!(sent::<bool>(&app), [(hard, true, true)]);
    let dot = find::<RadioDot>(&app, hard).expect("a dot");
    assert!(shown(&app, dot));
    assert_eq!(background(&app, dot), color(&app, ColorRole::Primary));
    let ring = find::<RadioRing>(&app, easy).expect("a ring");
    assert_eq!(
        app.world().get::<BorderColor>(ring).expect("a border").top,
        color(&app, ColorRole::OnSurfaceVariant)
    );
}

#[test]
fn the_arrow_keys_home_and_end_move_a_focused_groups_choice() {
    let mut app = app();
    let group = spawn(
        &mut app,
        radio_group(["Easy", "Normal", "Hard"], Some(0)).bundle(),
    );
    let chosen = |app: &App| (0..3).find(|&index| checked(app, option(app, group, index)));

    focus(&mut app, group);
    press(&mut app, KeyCode::ArrowDown, Key::ArrowDown);
    assert_eq!(chosen(&app), Some(1));
    press(&mut app, KeyCode::End, Key::End);
    assert_eq!(chosen(&app), Some(2));
    press(&mut app, KeyCode::Home, Key::Home);
    assert_eq!(chosen(&app), Some(0));
}

#[test]
fn clicking_an_options_label_chooses_it() {
    let mut app = app();
    let group = spawn(&mut app, radio_group(["Easy", "Normal"], None).bundle());
    let normal = option(&app, group, 1);
    let row = app.world().get::<ChildOf>(normal).expect("a row").parent();
    let words = find::<Text>(&app, row).expect("a label");

    click(&mut app, words);

    assert!(checked(&app, normal));
    assert_eq!(sent::<Entity>(&app), [(group, normal, true)]);
}

#[test]
fn a_disabled_option_cannot_be_chosen() {
    let mut app = app();
    let group = spawn(
        &mut app,
        radio_group(["Easy", "Normal"], Some(0)).disable(1).bundle(),
    );
    let normal = option(&app, group, 1);
    assert!(app.world().get::<InteractionDisabled>(normal).is_some());

    click(&mut app, normal);

    assert!(!checked(&app, normal));
    assert!(checked(&app, option(&app, group, 0)));
}

#[test]
fn a_radio_button_on_its_own_checks_itself_when_clicked() {
    let mut app = app();
    let alone = spawn(&mut app, radio().bundle());

    click(&mut app, alone);

    assert!(checked(&app, alone));
}

// Switches.

#[test]
fn a_clicked_switch_turns_on_fills_its_track_and_its_handle_grows_and_slides_across() {
    let mut app = app();
    let toggle = spawn(&mut app, switch().icons(true).bundle());
    let handle = find::<SwitchHandle>(&app, toggle).expect("a handle");
    let off = rect(&app, handle);
    assert_eq!(
        background(&app, toggle),
        color(&app, ColorRole::SurfaceContainerHighest)
    );

    click(&mut app, toggle);
    // Let the pointer go, so the handle is drawn at rest.
    app.update();

    assert!(checked(&app, toggle));
    assert_eq!(sent::<bool>(&app), [(toggle, true, true)]);
    assert_eq!(background(&app, toggle), color(&app, ColorRole::Primary));
    let on = rect(&app, handle);
    assert!(on.center().x > off.center().x + 15.0, "{off:?} to {on:?}");
    assert!(on.width() > off.width(), "it grows");
    let tick = find::<IconRoot>(&app, handle).expect("a tick");
    assert!(shown(&app, tick), "a tick in the handle while on");
    assert!(
        within(on, rect(&app, toggle).inflate(1.0)),
        "inside the track"
    );
}

#[test]
fn enter_turns_a_focused_switch_off() {
    let mut app = app();
    let toggle = spawn(&mut app, switch().on(true).bundle());

    focus(&mut app, toggle);
    press(&mut app, KeyCode::Enter, Key::Enter);

    assert!(!checked(&app, toggle));
    assert_eq!(sent::<bool>(&app), [(toggle, false, true)]);
}

#[test]
fn a_disabled_switch_ignores_clicks() {
    let mut app = app();
    let toggle = spawn(&mut app, switch().enabled(false).bundle());

    click(&mut app, toggle);

    assert!(!checked(&app, toggle));
    assert!(sent::<bool>(&app).is_empty());
}

#[test]
fn clicking_a_switchs_label_turns_it_over() {
    let mut app = app();
    let row = spawn(&mut app, switch().labelled("Music"));
    let toggle = find::<bevy::ui_widgets::Checkbox>(&app, row).expect("a switch");
    let words = find::<Text>(&app, row).expect("a label");

    click(&mut app, words);

    assert!(checked(&app, toggle));
}

// Sliders.

/// A row 420 px wide holding `slider`, and the slider in it.
fn in_row(app: &mut App, slider: MaterialSlider) -> Entity {
    let row = spawn(
        app,
        (
            Node {
                width: Val::Px(420.0),
                ..default()
            },
            children![slider.bundle()],
        ),
    );
    find::<Slider>(app, row).expect("a slider")
}

fn value(app: &App, slider: Entity) -> f32 {
    app.world().get::<SliderValue>(slider).expect("a value").0
}

/// The point at `share` of a slider's handle's travel, in logical pixels.
fn along(app: &mut App, slider: Entity, share: f32) -> Vec2 {
    let whole = rect(app, slider);
    let middle = centre(app, slider);
    let travel = whole.width() - HANDLE;
    Vec2::new(whole.min.x + HANDLE / 2.0 + share * travel, middle.y)
}

#[test]
fn a_click_on_the_track_moves_the_slider_there_on_a_step_and_lets_go_with_a_final_value() {
    let mut app = app();
    let slider = in_row(&mut app, slider(0.0, 0.0..=100.0).step(10.0));

    let at = along(&mut app, slider, 0.72);
    bevy_material_ui::testing::click_at(&mut app, at);

    assert_eq!(value(&app, slider), 70.0);
    let finals: Vec<f32> = sent::<f32>(&app)
        .into_iter()
        .filter(|(_, _, last)| *last)
        .map(|(_, value, _)| value)
        .collect();
    assert_eq!(finals, [70.0], "one final value, on letting go");
    let fill = find::<SliderFill>(&app, slider).expect("a fill");
    assert_eq!(
        app.world().get::<Node>(fill).expect("a node").width,
        Val::Percent(70.0)
    );
    assert_eq!(background(&app, fill), color(&app, ColorRole::Primary));
}

#[test]
fn dragging_a_sliders_handle_takes_it_along_and_ends_with_a_final_value() {
    let mut app = app();
    let slider = in_row(&mut app, slider(20.0, 0.0..=100.0).step(1.0));

    let from = along(&mut app, slider, 0.2);
    let to = along(&mut app, slider, 0.6);
    drag(&mut app, from, to);

    assert!(
        (value(&app, slider) - 60.0).abs() <= 1.0,
        "{}",
        value(&app, slider)
    );
    assert!(
        sent::<f32>(&app).iter().any(|(_, _, last)| *last),
        "a final value when it is let go"
    );
}

#[test]
fn the_arrow_keys_home_and_end_move_a_focused_slider_by_its_step() {
    let mut app = app();
    let slider = in_row(&mut app, slider(50.0, 0.0..=100.0).discrete(25.0));

    focus(&mut app, slider);
    press(&mut app, KeyCode::ArrowRight, Key::ArrowRight);
    assert_eq!(value(&app, slider), 75.0);
    press(&mut app, KeyCode::ArrowLeft, Key::ArrowLeft);
    press(&mut app, KeyCode::ArrowLeft, Key::ArrowLeft);
    assert_eq!(value(&app, slider), 25.0);
    press(&mut app, KeyCode::End, Key::End);
    assert_eq!(value(&app, slider), 100.0);
    press(&mut app, KeyCode::Home, Key::Home);
    assert_eq!(value(&app, slider), 0.0);
}

#[test]
fn a_discrete_sliders_stops_behind_its_handle_are_on_primary_and_those_ahead_on_its_track() {
    let mut app = app();
    let slider = in_row(&mut app, slider(50.0, 0.0..=100.0).discrete(25.0));

    let stops = find_all::<SliderStop>(&app, slider);
    assert_eq!(stops.len(), 5);
    let colours: Vec<Color> = stops.iter().map(|stop| background(&app, *stop)).collect();
    let on_primary = color(&app, ColorRole::OnPrimary);
    let ahead = color(&app, ColorRole::OnSecondaryContainer);
    assert_eq!(colours, [on_primary, on_primary, on_primary, ahead, ahead]);
}

#[test]
fn a_disabled_slider_ignores_the_keys_and_is_faded() {
    let mut app = app();
    let slider = in_row(
        &mut app,
        slider(50.0, 0.0..=100.0).step(10.0).enabled(false),
    );

    focus(&mut app, slider);
    press(&mut app, KeyCode::ArrowRight, Key::ArrowRight);

    assert_eq!(value(&app, slider), 50.0);
    let fill = find::<SliderFill>(&app, slider).expect("a fill");
    assert!((background(&app, fill).alpha() - 0.38).abs() < 0.01);
}

#[test]
fn a_controlled_slider_says_where_it_was_moved_and_waits_for_the_game() {
    let mut app = app();
    let slider = in_row(&mut app, slider(50.0, 0.0..=100.0).step(10.0).controlled());

    focus(&mut app, slider);
    press(&mut app, KeyCode::ArrowRight, Key::ArrowRight);

    assert_eq!(value(&app, slider), 50.0);
    assert_eq!(sent::<f32>(&app), [(slider, 60.0, true)]);
}

/// The notches the notched slider tests use: graphics presets'.
const NOTCHES: [&str; 5] = ["Low", "Medium", "High", "Ultra", "Max"];

fn notched(app: &mut App, chosen: Option<usize>) -> Entity {
    in_row(app, notched_slider(NOTCHES, chosen))
}

/// The notch a notched slider is at, and where its value is.
fn at(app: &App, slider: Entity) -> (Option<usize>, f32) {
    (
        app.world()
            .get::<Notches>(slider)
            .expect("notches")
            .chosen(),
        value(app, slider),
    )
}

/// Each of a notched slider's labels' colours, in order.
fn label_colours(app: &App, slider: Entity) -> Vec<Color> {
    let mut labels: Vec<(usize, Entity)> = find_all::<NotchLabel>(app, slider)
        .into_iter()
        .map(|label| {
            (
                app.world().get::<NotchLabel>(label).expect("a label").0,
                label,
            )
        })
        .collect();
    labels.sort_by_key(|(notch, _)| *notch);
    labels
        .into_iter()
        .map(|(_, label)| {
            let words = find::<Text>(app, label).expect("words");
            app.world().get::<TextColor>(words).expect("a colour").0
        })
        .collect()
}

fn handle_shown(app: &App, slider: Entity) -> bool {
    let thumb = find::<bevy::ui_widgets::SliderThumb>(app, slider).expect("a handle");
    shown(app, thumb)
}

#[test]
fn a_notched_slider_moved_between_notches_snaps_to_the_nearest_and_marks_its_label() {
    let mut app = app();
    let slider = notched(&mut app, Some(2));

    app.world_mut().trigger(ValueChange {
        source: slider,
        value: 2.6_f32,
        is_final: false,
    });
    app.update();

    assert_eq!(at(&app, slider), (Some(3), 3.0));
    let colours = label_colours(&app, slider);
    let primary = color(&app, ColorRole::Primary);
    let quiet = color(&app, ColorRole::OnSurfaceVariant);
    assert_eq!(colours[3], primary, "the notch's label marked");
    assert!(
        colours
            .iter()
            .enumerate()
            .all(|(notch, &tint)| notch == 3 || tint == quiet),
        "and no other: {colours:?}"
    );
}

#[test]
fn a_notched_slider_shows_each_notchs_label_under_its_notch_in_order() {
    let mut app = app();
    let slider = notched(&mut app, Some(0));

    let mut labels: Vec<(usize, Entity)> = find_all::<NotchLabel>(&app, slider)
        .into_iter()
        .map(|label| {
            (
                app.world().get::<NotchLabel>(label).expect("a label").0,
                label,
            )
        })
        .collect();
    labels.sort_by_key(|(notch, _)| *notch);
    let words: Vec<String> = labels
        .iter()
        .map(|(_, label)| {
            let text = find::<Text>(&app, *label).expect("words");
            app.world().get::<Text>(text).expect("words").0.clone()
        })
        .collect();
    assert_eq!(words, NOTCHES);
    let whole = rect(&app, slider);
    let track = rect(&app, find::<SliderTrack>(&app, slider).expect("a track"));
    let rooms: Vec<Rect> = labels.iter().map(|(_, label)| rect(&app, *label)).collect();
    for pair in rooms.windows(2) {
        assert!(pair[0].center().x < pair[1].center().x, "{rooms:?}");
    }
    for room in &rooms {
        assert!(room.min.y >= track.max.y - 1.0, "under the track: {room:?}");
        assert!(
            within(*room, whole),
            "within the slider: {room:?} in {whole:?}"
        );
    }
    let middle = rooms[2].center().x;
    assert!(
        (middle - track.center().x).abs() < 2.0,
        "the middle label under the middle notch: {middle} of {track:?}"
    );
}

#[test]
fn the_arrow_keys_home_and_end_move_a_focused_notched_slider_a_notch_at_a_time() {
    let mut app = app();
    let slider = notched(&mut app, Some(2));
    focus(&mut app, slider);
    let step = |app: &mut App, key_code: KeyCode, key: Key| {
        press(app, key_code, key);
        at(app, slider)
    };

    assert_eq!(
        step(&mut app, KeyCode::ArrowRight, Key::ArrowRight),
        (Some(3), 3.0)
    );
    assert_eq!(
        step(&mut app, KeyCode::ArrowLeft, Key::ArrowLeft),
        (Some(2), 2.0)
    );
    assert_eq!(step(&mut app, KeyCode::End, Key::End), (Some(4), 4.0));
    assert_eq!(
        step(&mut app, KeyCode::ArrowRight, Key::ArrowRight),
        (Some(4), 4.0),
        "never past the last"
    );
    assert_eq!(step(&mut app, KeyCode::Home, Key::Home), (Some(0), 0.0));
}

#[test]
fn a_notched_slider_at_no_notch_hides_its_handle_until_one_is_set_or_chosen() {
    let mut app = app();
    let slider = notched(&mut app, None);

    assert_eq!(at(&app, slider).0, None);
    assert!(!handle_shown(&app, slider), "no notch, no handle");
    let quiet = color(&app, ColorRole::OnSurfaceVariant);
    assert!(
        label_colours(&app, slider)
            .iter()
            .all(|&tint| tint == quiet),
        "no label marked"
    );

    set_notch(app.world_mut().commands().entity(slider), Some(1));
    app.update();
    assert_eq!(at(&app, slider), (Some(1), 1.0));
    assert!(handle_shown(&app, slider));

    set_notch(app.world_mut().commands().entity(slider), None);
    app.update();
    assert!(
        !handle_shown(&app, slider),
        "hidden again, as what it sets is custom"
    );

    app.world_mut().trigger(ValueChange {
        source: slider,
        value: 4.0_f32,
        is_final: true,
    });
    app.update();
    assert_eq!(at(&app, slider), (Some(4), 4.0));
    assert!(handle_shown(&app, slider), "chosen by the pointer or a key");
}

// Segmented buttons.

fn segment(app: &App, row: Entity, index: usize) -> Entity {
    find_where::<Segment>(app, row, |segment| segment.0 == index)
}

#[test]
fn clicking_a_segment_moves_a_single_select_rows_choice_and_both_say_so() {
    let mut app = app();
    let row = spawn(
        &mut app,
        segmented_buttons(["Day", "Week", "Month"], Some(0)).bundle(),
    );
    let (day, month) = (segment(&app, row, 0), segment(&app, row, 2));

    click(&mut app, month);

    assert!(checked(&app, month) && !checked(&app, day));
    assert_eq!(sent::<usize>(&app), [(row, 2, true)]);
    assert_eq!(sent::<bool>(&app), [(month, true, true)]);
    let look = *app.world().get::<Look>(month).expect("a look");
    assert_eq!(look.container, Some(ColorRole::SecondaryContainer));
    let tick = find::<SegmentCheck>(&app, month).expect("a tick");
    assert_eq!(
        app.world().get::<Node>(tick).expect("a node").display,
        Display::Flex
    );
    let gone = find::<SegmentCheck>(&app, day).expect("a tick");
    assert_eq!(
        app.world().get::<Node>(gone).expect("a node").display,
        Display::None
    );
}

#[test]
fn activating_the_chosen_segment_of_a_single_select_row_changes_nothing() {
    let mut app = app();
    let row = spawn(
        &mut app,
        segmented_buttons(["Day", "Week"], Some(1)).bundle(),
    );
    let week = segment(&app, row, 1);

    click(&mut app, week);

    assert!(checked(&app, week));
    assert!(sent::<usize>(&app).is_empty());
}

#[test]
fn a_multi_select_row_flips_each_segment_it_activates() {
    let mut app = app();
    let row = spawn(
        &mut app,
        segmented_buttons(["Bold", "Italic", "Underline"], None)
            .multi([0])
            .bundle(),
    );
    let (bold, italic) = (segment(&app, row, 0), segment(&app, row, 1));

    focus(&mut app, italic);
    press(&mut app, KeyCode::Enter, Key::Enter);
    assert!(checked(&app, bold) && checked(&app, italic));

    click(&mut app, bold);
    assert!(!checked(&app, bold) && checked(&app, italic));
    assert_eq!(
        sent::<bool>(&app),
        [(italic, true, true), (bold, false, true)]
    );
}

#[test]
fn a_disabled_segment_cannot_be_chosen() {
    let mut app = app();
    let row = spawn(
        &mut app,
        segmented_buttons(["Day", "Week"], Some(0))
            .disable(1)
            .bundle(),
    );
    let week = segment(&app, row, 1);

    click(&mut app, week);

    assert!(!checked(&app, week));
    assert!(sent::<usize>(&app).is_empty());
}
