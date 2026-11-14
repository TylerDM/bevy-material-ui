//! The core, driven as a player would: parts painted from the theme, a
//! button hovered and clicked with the mouse and used from the keyboard, a
//! disabled one refusing, and the theme turned dark under it all.

use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{click, focus, headless_app, hover, press, spawn, words};

/// How many times each button was activated.
#[derive(Resource, Default)]
struct Activations(Vec<Entity>);

fn app() -> App {
    let mut app = headless_app(800, 600, 1.0);
    app.init_resource::<Activations>().add_observer(
        |activate: On<Activate>, mut activations: ResMut<Activations>| {
            activations.0.push(activate.entity);
        },
    );
    app
}

fn background(app: &App, entity: Entity) -> Color {
    app.world()
        .get::<BackgroundColor>(entity)
        .expect("a background")
        .0
}

fn theme(app: &App) -> MaterialTheme {
    app.world().resource::<MaterialTheme>().clone()
}

#[test]
fn a_filled_button_is_drawn_in_primary_with_its_label_on_primary_in_label_large() {
    let mut app = app();
    let button = spawn(&mut app, button("Play").bundle());

    let theme = theme(&app);
    assert_eq!(background(&app, button), theme.color(ColorRole::Primary));
    assert_eq!(words(&app, button), "Play");
    let label = app.world().get::<Children>(button).expect("a label")[0];
    assert_eq!(
        app.world().get::<TextColor>(label).expect("a colour").0,
        theme.color(ColorRole::OnPrimary)
    );
    assert_eq!(
        app.world()
            .get::<TextFont>(label)
            .expect("a font")
            .font_size,
        FontSize::Px(TypeRole::LabelLarge.size())
    );
}

#[test]
fn a_hovered_button_shows_its_state_layer_and_a_click_activates_it() {
    let mut app = app();
    let button = spawn(&mut app, button("Play").bundle());
    let rest = background(&app, button);

    hover(&mut app, button);
    assert_ne!(
        background(&app, button),
        rest,
        "hovered shows a state layer"
    );

    click(&mut app, button);
    assert_eq!(app.world().resource::<Activations>().0, [button]);
}

#[test]
fn enter_or_space_activates_a_focused_button() {
    let mut app = app();
    let button = spawn(&mut app, button("Play").tonal().bundle());

    focus(&mut app, button);
    press(&mut app, KeyCode::Enter, Key::Enter);
    press(&mut app, KeyCode::Space, Key::Space);

    assert_eq!(app.world().resource::<Activations>().0, [button, button]);
    let ring = app.world().get::<Outline>(button).expect("a focus ring");
    assert_ne!(ring.color, Color::NONE, "the focus ring shows");
}

#[test]
fn a_disabled_button_is_faded_and_a_click_does_nothing_until_it_is_enabled() {
    let mut app = app();
    let button = spawn(&mut app, button("Play").enabled(false).bundle());

    let faded = background(&app, button);
    assert!(faded.alpha() < 0.2, "faded: {faded:?}");
    click(&mut app, button);
    assert!(app.world().resource::<Activations>().0.is_empty());

    set_enabled(&mut app.world_mut().commands().entity(button), true);
    app.update();
    let hovered = ButtonVariant::Filled
        .look()
        .container_color(&theme(&app), true, false, false);
    assert_eq!(
        background(&app, button),
        hovered,
        "enabled, under the pointer"
    );
    click(&mut app, button);
    assert_eq!(app.world().resource::<Activations>().0, [button]);
}

#[test]
fn turning_the_theme_dark_repaints_every_part() {
    let mut app = app();
    let button = spawn(&mut app, button("Play").bundle());
    let sheet = spawn(&mut app, paper().bundle());
    let words = spawn(
        &mut app,
        text("Hello", TypeRole::BodyLarge, ColorRole::OnSurface),
    );

    app.insert_resource(MaterialTheme::dark());
    app.update();

    let dark = MaterialTheme::dark();
    assert_eq!(background(&app, button), dark.color(ColorRole::Primary));
    assert_eq!(
        background(&app, sheet),
        dark.color(ColorRole::SurfaceContainerLow)
    );
    assert_eq!(
        app.world().get::<TextColor>(words).expect("a colour").0,
        dark.color(ColorRole::OnSurface)
    );
    let label = app.world().get::<Children>(button).expect("a label")[0];
    assert_eq!(
        app.world().get::<TextColor>(label).expect("a colour").0,
        dark.color(ColorRole::OnPrimary)
    );
}

#[test]
fn a_raised_paper_casts_a_shadow_and_an_outlined_one_none() {
    let mut app = app();
    let raised = spawn(&mut app, paper().elevation(Elevation::Level3).bundle());
    let outlined = spawn(&mut app, card(CardKind::Outlined).bundle());

    assert!(
        !app.world()
            .get::<BoxShadow>(raised)
            .expect("a shadow")
            .0
            .is_empty()
    );
    assert!(
        app.world()
            .get::<BoxShadow>(outlined)
            .expect("a shadow")
            .0
            .is_empty()
    );
    assert_eq!(
        app.world()
            .get::<BorderColor>(outlined)
            .expect("a border")
            .top,
        theme(&app).color(ColorRole::OutlineVariant)
    );
}
