//! Feedback, overlays and display, driven as a player would: dialogs
//! answered and dismissed, snackbars coming and going, tooltips appearing on
//! a resting pointer, progress drawn, badges counted, a sheet risen and
//! closed, and a scroll area's bar.

use std::f32::consts::TAU;

use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy::ui::{BorderGradient, Gradient, ScrollPosition, UiTransform, Val2};
use bevy::ui_widgets::Button;
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{
    click, click_at, find, find_all, focus, headless_app, hover, press, rect, scroll_into_view,
    settle, spawn, wait, within, words,
};

/// Every event of type `E` seen, in order.
#[derive(Resource)]
struct Seen<E: Clone + Send + Sync + 'static>(Vec<E>);

fn record<E: Event + Clone>(app: &mut App) {
    app.insert_resource(Seen::<E>(Vec::new())).add_observer(
        |event: On<E>, mut seen: ResMut<Seen<E>>| {
            seen.0.push(event.event().clone());
        },
    );
}

fn seen<E: Event + Clone>(app: &App) -> Vec<E> {
    app.world().resource::<Seen<E>>().0.clone()
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

fn exists(app: &App, entity: Entity) -> bool {
    app.world().get_entity(entity).is_ok()
}

fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut().query::<&T>().iter(app.world()).count()
}

fn only<T: Component>(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .single(app.world())
        .expect("exactly one")
}

/// Spawns undle as a child of parent, lays it out and returns it - to keep
/// parts in one layout, where each spawn stacks a full-window root of its own.
fn add(app: &mut App, parent: Entity, bundle: impl Bundle) -> Entity {
    let entity = app.world_mut().spawn((bundle, ChildOf(parent))).id();
    settle(app);
    entity
}

/// A column well below the window's top, with room above for a tooltip.
fn column(app: &mut App) -> Entity {
    spawn(
        app,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(24.0),
            padding: UiRect::top(Val::Px(80.0)),
            ..default()
        },
    )
}

fn app() -> App {
    let mut app = headless_app(800, 600, 1.0);
    record::<DialogResult>(&mut app);
    record::<SnackbarAction>(&mut app);
    record::<SnackbarClosed>(&mut app);
    record::<TooltipAction>(&mut app);
    record::<SheetClosed>(&mut app);
    record::<Activate>(&mut app);
    app
}

/// The action of a dialog answering `key`.
fn action(app: &App, dialog: Entity, key: &str) -> Entity {
    find_all::<DialogActionKey>(app, dialog)
        .into_iter()
        .find(|entity| {
            app.world()
                .get::<DialogActionKey>(*entity)
                .expect("a key")
                .0
                == key
        })
        .expect("the action")
}

#[test]
fn a_dialog_is_drawn_on_its_container_over_a_scrim_with_its_title_text_and_actions() {
    let mut app = app();
    let dialog = spawn(
        &mut app,
        confirm_dialog("Delete the world?", "It can't be undone.", "Delete", "Keep").bundle(),
    );

    let theme = theme(&app);
    let scrim = background(&app, dialog);
    assert!((scrim.alpha() - SCRIM_OPACITY).abs() < 1e-4, "{scrim:?}");
    let surface = find::<DialogSurface>(&app, dialog).expect("a surface");
    assert_eq!(
        background(&app, surface),
        theme.color(ColorRole::SurfaceContainerHigh)
    );
    assert_eq!(
        words(&app, dialog),
        "Delete the world? It can't be undone. Keep Delete"
    );
    let window = rect(&app, dialog);
    assert_eq!(
        window.size(),
        Vec2::new(800.0, 600.0),
        "the scrim covers the window"
    );
    let drawn = rect(&app, surface);
    assert!(
        drawn.width() >= 280.0 && drawn.width() <= 560.0,
        "{drawn:?}"
    );
    assert!(
        (drawn.center() - window.center()).length() < 2.0,
        "centred: {drawn:?}"
    );
}

#[test]
fn a_dialogs_confirm_action_answers_confirmed_and_closes_it() {
    let mut app = app();
    let dialog = spawn(
        &mut app,
        confirm_dialog("Delete?", "Gone for good.", "Delete", "Keep").bundle(),
    );

    let confirm = action(&app, dialog, CONFIRM);
    click(&mut app, confirm);

    let results = seen::<DialogResult>(&app);
    assert_eq!(results.len(), 1, "{results:?}");
    assert!(results[0].is_confirmed());
    assert_eq!(results[0].dialog, dialog);
    assert!(!exists(&app, dialog), "closed");
}

#[test]
fn a_dialogs_cancel_action_answers_with_its_key() {
    let mut app = app();
    let dialog = spawn(
        &mut app,
        confirm_dialog("Delete?", "Gone for good.", "Delete", "Keep").bundle(),
    );

    let cancel = action(&app, dialog, CANCEL);
    click(&mut app, cancel);

    let results = seen::<DialogResult>(&app);
    assert_eq!(results[0].answer, DialogAnswer::Chose(CANCEL.to_owned()));
    assert!(!results[0].is_confirmed());
}

#[test]
fn escape_dismisses_a_dialog() {
    let mut app = app();
    let dialog = spawn(
        &mut app,
        confirm_dialog("Leave?", "", "Leave", "Stay").bundle(),
    );
    // The dialog takes the keyboard's focus a moment after it opens.
    wait(&mut app, 0.2);

    press(&mut app, KeyCode::Escape, Key::Escape);

    let results = seen::<DialogResult>(&app);
    assert_eq!(results.len(), 1, "{results:?}");
    assert_eq!(results[0].answer, DialogAnswer::Dismissed);
    assert!(!exists(&app, dialog));
}

#[test]
fn the_scrim_takes_a_click_meant_for_what_is_behind_it_and_dismisses_the_dialog() {
    let mut app = app();
    let behind = spawn(&mut app, button("Behind").bundle());
    let at = {
        let drawn = rect(&app, behind);
        drawn.center()
    };
    // On no root of its own, so that once it goes nothing covers the button.
    let dialog = app
        .world_mut()
        .spawn(dialog("Hello").action("ok", "OK").bundle())
        .id();
    settle(&mut app);

    click_at(&mut app, at);

    assert!(
        !seen::<Activate>(&app)
            .iter()
            .any(|activate| activate.entity == behind),
        "the button behind is never reached"
    );
    assert_eq!(
        seen::<DialogResult>(&app)[0].answer,
        DialogAnswer::Dismissed
    );
    assert!(!exists(&app, dialog));

    click_at(&mut app, at);
    assert!(
        seen::<Activate>(&app)
            .iter()
            .any(|activate| activate.entity == behind),
        "with the dialog gone, it is"
    );
}

#[test]
fn a_dialog_holds_the_games_own_parts_and_an_icon_in_secondary() {
    let mut app = app();
    let dialog = spawn(
        &mut app,
        dialog("Pick a seed")
            .icon(Glyph::Check)
            .body(|body| {
                body.spawn(text("Seed 42", TypeRole::BodyLarge, ColorRole::OnSurface));
            })
            .action("ok", "OK")
            .bundle(),
    );

    assert!(words(&app, dialog).contains("Seed 42"));
    let glyph = find::<IconRoot>(&app, dialog).expect("an icon");
    let stroke = app.world().get::<Children>(glyph).expect("strokes")[0];
    assert_eq!(
        background(&app, stroke),
        theme(&app).color(ColorRole::Secondary)
    );
}

fn show(app: &mut App, snackbar: Snackbar) -> SnackbarId {
    let id = app
        .world_mut()
        .resource_mut::<Snackbars>()
        .show_snackbar(snackbar);
    settle(app);
    id
}

#[test]
fn a_snackbar_shows_on_the_inverse_surface_at_the_bottom_centre() {
    let mut app = app();
    show(&mut app, Snackbar::new("World saved"));

    let view = only::<SnackbarView>(&mut app);
    let theme = theme(&app);
    assert_eq!(
        background(&app, view),
        theme.color(ColorRole::InverseSurface)
    );
    assert_eq!(words(&app, view), "World saved");
    let message = find::<Text>(&app, view).expect("words");
    assert_eq!(
        app.world().get::<TextColor>(message).expect("a colour").0,
        theme.color(ColorRole::InverseOnSurface)
    );
    let drawn = rect(&app, view);
    assert!((drawn.center().x - 400.0).abs() < 2.0, "centred: {drawn:?}");
    assert!(
        drawn.max.y > 500.0 && drawn.max.y <= 600.0,
        "at the bottom: {drawn:?}"
    );
}

#[test]
fn snackbars_show_one_at_a_time_each_for_its_time_and_then_the_next() {
    let mut app = app();
    let first = show(&mut app, Snackbar::new("First"));
    let second = show(&mut app, Snackbar::new("Second"));

    assert_eq!(count::<SnackbarView>(&mut app), 1, "one at a time");
    assert_eq!(app.world().resource::<Snackbars>().showing(), Some(first));

    wait(&mut app, SNACKBAR_SECONDS - 0.5);
    assert_eq!(
        app.world().resource::<Snackbars>().showing(),
        Some(first),
        "not yet"
    );

    wait(&mut app, 1.0);
    assert_eq!(
        seen::<SnackbarClosed>(&app),
        [SnackbarClosed {
            id: first,
            reason: SnackbarCloseReason::Timeout
        }]
    );
    settle(&mut app);
    assert_eq!(app.world().resource::<Snackbars>().showing(), Some(second));
    let view = only::<SnackbarView>(&mut app);
    assert_eq!(words(&app, view), "Second");

    wait(&mut app, SNACKBAR_SECONDS + 0.5);
    assert_eq!(count::<SnackbarView>(&mut app), 0, "all gone");
}

#[test]
fn a_snackbars_action_sends_its_event_and_closes_it() {
    let mut app = app();
    let id = show(&mut app, Snackbar::new("Block broken").action("Undo"));

    let undo = only::<SnackbarActionButton>(&mut app);
    assert_eq!(words(&app, undo), "Undo");
    let label = find::<Text>(&app, undo).expect("a label");
    assert_eq!(
        app.world().get::<TextColor>(label).expect("a colour").0,
        theme(&app).color(ColorRole::InversePrimary)
    );
    click(&mut app, undo);

    assert_eq!(seen::<SnackbarAction>(&app), [SnackbarAction { id }]);
    assert_eq!(
        seen::<SnackbarClosed>(&app),
        [SnackbarClosed {
            id,
            reason: SnackbarCloseReason::Action
        }]
    );
    assert_eq!(count::<SnackbarView>(&mut app), 0);
}

#[test]
fn a_snackbars_close_button_dismisses_it() {
    let mut app = app();
    let id = show(&mut app, Snackbar::new("Connected").closable());

    let close = only::<SnackbarCloseButton>(&mut app);
    click(&mut app, close);

    assert_eq!(
        seen::<SnackbarClosed>(&app),
        [SnackbarClosed {
            id,
            reason: SnackbarCloseReason::Dismissed
        }]
    );
    assert_eq!(count::<SnackbarView>(&mut app), 0);
}

#[test]
fn a_tooltip_appears_after_the_pointer_rests_and_goes_when_it_leaves() {
    let mut app = app();
    let column = column(&mut app);
    let save = add(
        &mut app,
        column,
        (button("Save").bundle(), Tooltip::plain("Saves the world")),
    );
    let elsewhere = add(&mut app, column, button("Elsewhere").bundle());

    hover(&mut app, save);
    wait(&mut app, TOOLTIP_DELAY - 0.2);
    assert_eq!(count::<TooltipView>(&mut app), 0, "not before the delay");

    wait(&mut app, 0.3);
    settle(&mut app);
    let view = only::<TooltipView>(&mut app);
    assert_eq!(words(&app, view), "Saves the world");
    assert_eq!(
        background(&app, view),
        theme(&app).color(ColorRole::InverseSurface)
    );
    let (tip, anchor) = (rect(&app, view), rect(&app, save));
    assert!(
        tip.max.y <= anchor.min.y + 1.0,
        "above its part: {tip:?} {anchor:?}"
    );
    assert!(
        (tip.center().x - anchor.center().x).abs() < 2.0,
        "centred on it"
    );

    hover(&mut app, elsewhere);
    app.update();
    assert_eq!(
        count::<TooltipView>(&mut app),
        0,
        "gone when the pointer leaves"
    );
}

#[test]
fn a_tooltip_with_no_room_above_goes_below() {
    let mut app = app();
    // The harness's column starts 16 px from the top: no room above.
    let save = spawn(&mut app, (button("Save").bundle(), Tooltip::plain("Saves")));

    hover(&mut app, save);
    wait(&mut app, TOOLTIP_DELAY + 0.1);
    settle(&mut app);

    let view = only::<TooltipView>(&mut app);
    assert!(
        rect(&app, view).min.y >= rect(&app, save).max.y - 1.0,
        "below its part"
    );
}

#[test]
fn a_rich_tooltips_action_is_sent_to_its_part() {
    let mut app = app();
    let elsewhere = spawn(
        &mut app,
        Node {
            height: Val::Px(120.0),
            ..default()
        },
    );
    let info = spawn(
        &mut app,
        (
            button("Info").bundle(),
            Tooltip::rich("Seeds make the same world every time.")
                .title("Seeds")
                .action("Learn more"),
        ),
    );
    let _ = elsewhere;

    hover(&mut app, info);
    wait(&mut app, TOOLTIP_DELAY + 0.1);
    settle(&mut app);
    let view = only::<TooltipView>(&mut app);
    assert_eq!(
        background(&app, view),
        theme(&app).color(ColorRole::SurfaceContainer)
    );
    assert!(words(&app, view).starts_with("Seeds Seeds make"));
    let learn = find::<Button>(&app, view).expect("its action");

    click(&mut app, learn);

    assert_eq!(
        seen::<TooltipAction>(&app),
        [TooltipAction { anchor: info }]
    );
}

fn indicator(app: &App, track: Entity) -> Entity {
    find::<ProgressIndicator>(app, track).expect("an indicator")
}

#[test]
fn a_linear_progress_indicator_fills_its_track_to_its_share() {
    let mut app = app();
    let track = spawn(
        &mut app,
        (Node {
            width: Val::Px(400.0),
            ..default()
        },),
    );
    let progress = app
        .world_mut()
        .spawn((linear_progress(Some(0.4)), ChildOf(track)))
        .id();
    settle(&mut app);

    let theme = theme(&app);
    assert_eq!(
        background(&app, progress),
        theme.color(ColorRole::SecondaryContainer)
    );
    let bar = indicator(&app, progress);
    assert_eq!(background(&app, bar), theme.color(ColorRole::Primary));
    assert!(
        (rect(&app, bar).width() - 160.0).abs() < 1.5,
        "{:?}",
        rect(&app, bar)
    );
    assert_eq!(rect(&app, progress).height(), LINEAR_THICKNESS);

    app.world_mut()
        .entity_mut(progress)
        .insert(Progress(Some(0.9)));
    settle(&mut app);
    assert!((rect(&app, bar).width() - 360.0).abs() < 1.5);
}

#[test]
fn an_indeterminate_linear_indicator_sweeps_its_track() {
    let mut app = app();
    let track = spawn(
        &mut app,
        Node {
            width: Val::Px(400.0),
            ..default()
        },
    );
    let progress = app
        .world_mut()
        .spawn((linear_progress(None), ChildOf(track)))
        .id();
    settle(&mut app);
    let bar = indicator(&app, progress);

    let mut spans = Vec::new();
    for _ in 0..12 {
        wait(&mut app, 0.1);
        let node = app.world().get::<Node>(bar).expect("a node");
        spans.push((node.left, node.width));
    }
    spans.dedup();
    assert!(spans.len() > 6, "it moves: {spans:?}");
}

#[test]
fn a_circular_indicator_draws_its_share_as_an_arc_on_its_track() {
    let mut app = app();
    let circle = spawn(&mut app, circular_progress(Some(0.5)));

    let theme = theme(&app);
    let gradient = app
        .world()
        .get::<BorderGradient>(circle)
        .expect("a gradient");
    let Gradient::Conic(conic) = &gradient.0[0] else {
        panic!("a conic gradient: {gradient:?}");
    };
    assert_eq!(conic.stops[1].angle, Some(TAU / 2.0));
    assert_eq!(conic.stops[1].color, theme.color(ColorRole::Primary));
    assert_eq!(
        conic.stops[2].color,
        theme.color(ColorRole::SecondaryContainer)
    );
    assert_eq!(rect(&app, circle).size(), Vec2::splat(CIRCULAR_SIDE));

    app.world_mut().entity_mut(circle).insert(Progress(None));
    let start = |app: &App| match &app
        .world()
        .get::<BorderGradient>(circle)
        .expect("a gradient")
        .0[0]
    {
        Gradient::Conic(conic) => conic.start,
        other => panic!("{other:?}"),
    };
    wait(&mut app, 0.2);
    let first = start(&app);
    wait(&mut app, 0.2);
    assert_ne!(start(&app), first, "an indeterminate spinner turns");
}

#[test]
fn a_count_badge_caps_at_999_plus_and_follows_its_count() {
    let mut app = app();
    let holder = spawn(
        &mut app,
        Node {
            width: Val::Px(24.0),
            height: Val::Px(24.0),
            ..default()
        },
    );
    let count_badge = app
        .world_mut()
        .spawn((badge(Badge::Count(1500)), ChildOf(holder)))
        .id();
    settle(&mut app);

    let theme = theme(&app);
    assert_eq!(words(&app, count_badge), "999+");
    assert_eq!(background(&app, count_badge), theme.color(ColorRole::Error));
    let label = find::<BadgeLabel>(&app, count_badge).expect("a label");
    assert_eq!(
        app.world().get::<TextColor>(label).expect("a colour").0,
        theme.color(ColorRole::OnError)
    );
    assert_eq!(rect(&app, count_badge).height(), LARGE_BADGE);

    app.world_mut()
        .entity_mut(count_badge)
        .insert(Badge::Count(3));
    settle(&mut app);
    assert_eq!(words(&app, count_badge), "3");

    app.world_mut().entity_mut(count_badge).insert(Badge::Dot);
    settle(&mut app);
    assert_eq!(rect(&app, count_badge).size(), Vec2::splat(SMALL_BADGE));
    let (dot, icon) = (rect(&app, count_badge), rect(&app, holder));
    assert!((dot.max.x - icon.max.x).abs() < 1.0 && (dot.min.y - icon.min.y).abs() < 1.0);
}

#[test]
fn an_avatar_shows_initials_on_the_primary_container() {
    let mut app = app();
    let face = spawn(
        &mut app,
        avatar(AvatarFace::Initials(initials("Ada Lovelace")), AVATAR),
    );

    let theme = theme(&app);
    assert_eq!(words(&app, face), "AL");
    assert_eq!(
        background(&app, face),
        theme.color(ColorRole::PrimaryContainer)
    );
    let letters = find::<Text>(&app, face).expect("letters");
    assert_eq!(
        app.world().get::<TextColor>(letters).expect("a colour").0,
        theme.color(ColorRole::OnPrimaryContainer)
    );
    assert_eq!(rect(&app, face).size(), Vec2::splat(AVATAR));
}

fn sheet(app: &mut App) -> Entity {
    spawn(
        app,
        bottom_sheet(|content| {
            content.spawn(text(
                "Share the world",
                TypeRole::TitleMedium,
                ColorRole::OnSurface,
            ));
        }),
    )
}

#[test]
fn a_sheet_rises_from_the_bottom_edge_onto_its_container() {
    let mut app = app();
    let scrim = sheet(&mut app);
    let surface = find::<SheetSurface>(&app, scrim).expect("a surface");

    let offset = |app: &App| {
        app.world()
            .get::<UiTransform>(surface)
            .expect("a transform")
            .translation
    };
    assert_ne!(offset(&app), Val2::ZERO, "still rising");
    wait(&mut app, 0.5);
    assert_eq!(offset(&app), Val2::percent(0.0, 0.0), "risen");
    settle(&mut app);

    assert_eq!(
        background(&app, surface),
        theme(&app).color(ColorRole::SurfaceContainerLow)
    );
    assert!(words(&app, scrim).contains("Share the world"));
    let drawn = rect(&app, surface);
    assert!(
        (drawn.max.y - 600.0).abs() < 1.0,
        "on the bottom edge: {drawn:?}"
    );
    assert!(drawn.width() <= SHEET_MAX_WIDTH + 1.0);
}

#[test]
fn a_click_on_the_scrim_above_a_sheet_closes_it() {
    let mut app = app();
    let scrim = sheet(&mut app);
    wait(&mut app, 0.5);

    click_at(&mut app, Vec2::new(400.0, 40.0));

    assert_eq!(seen::<SheetClosed>(&app), [SheetClosed { sheet: scrim }]);
    assert!(!exists(&app, scrim));
}

#[test]
fn escape_or_the_drag_handle_closes_a_sheet() {
    let mut app = app();
    let scrim = sheet(&mut app);
    wait(&mut app, 0.5);
    press(&mut app, KeyCode::Escape, Key::Escape);
    assert_eq!(seen::<SheetClosed>(&app).len(), 1, "Escape");
    assert!(!exists(&app, scrim));

    let scrim = sheet(&mut app);
    wait(&mut app, 0.5);
    let handle = find::<SheetHandle>(&app, scrim).expect("a handle");
    focus(&mut app, handle);
    press(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(seen::<SheetClosed>(&app).len(), 2, "the handle");
    assert!(!exists(&app, scrim));
}

fn rows(count: usize) -> impl Bundle {
    scroll_area(Val::Px(200.0), move |column| {
        for row in 0..count {
            column.spawn((
                Node {
                    height: Val::Px(40.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                children![text(
                    format!("Row {row}"),
                    TypeRole::BodyLarge,
                    ColorRole::OnSurface
                )],
            ));
        }
    })
}

#[test]
fn the_harness_scrolls_a_part_into_view_through_a_scroll_area_inside_another() {
    let mut app = app();
    let outer = spawn(
        &mut app,
        scroll_area(Val::Px(300.0), |page| {
            for _ in 0..20 {
                page.spawn(Node {
                    height: Val::Px(40.0),
                    flex_shrink: 0.0,
                    ..default()
                });
            }
            page.spawn(rows(30));
        }),
    );
    let last = find_all::<Text>(&app, outer)
        .into_iter()
        .find(|part| {
            app.world()
                .get::<Text>(*part)
                .is_some_and(|text| text.0 == "Row 29")
        })
        .expect("the inner area's last row");
    let inner = find_all::<bevy::ui_widgets::ScrollArea>(&app, outer)
        .into_iter()
        .find(|area| *area != outer)
        .expect("the inner area");
    assert!(
        !within(rect(&app, last), rect(&app, outer)),
        "it starts out of view"
    );

    scroll_into_view(&mut app, last);
    assert!(
        within(rect(&app, last), rect(&app, inner)),
        "the inner area scrolled to it"
    );
    assert!(
        within(rect(&app, inner), rect(&app, outer)),
        "and the outer area to the inner"
    );
}

#[test]
fn a_scroll_areas_bar_shows_only_when_its_content_overflows_and_follows_the_scroll() {
    let mut app = app();
    let long = spawn(&mut app, rows(30));
    let short = spawn(&mut app, rows(2));

    let bar = |app: &App, area| find::<MaterialScrollbar>(app, area).expect("a scrollbar");
    let shown = |app: &App, area| {
        app.world().get::<Visibility>(bar(app, area)) != Some(&Visibility::Hidden)
    };
    assert!(shown(&app, long), "30 rows overflow 200 px");
    assert!(!shown(&app, short), "2 rows fit");

    let thumb = find::<MaterialScrollThumb>(&app, long).expect("a thumb");
    let rest = background(&app, thumb);
    assert!((rest.alpha() - THUMB_REST_OPACITY).abs() < 1e-3, "{rest:?}");
    let top = rect(&app, thumb).min.y;

    let viewport = find::<ScrollViewport>(&app, long).expect("a viewport");
    app.world_mut()
        .entity_mut(viewport)
        .insert(ScrollPosition(Vec2::new(0.0, 500.0)));
    settle(&mut app);
    assert!(
        rect(&app, thumb).min.y > top + 10.0,
        "the thumb follows the scroll"
    );
}

#[test]
fn a_skeleton_pulses_on_the_highest_container() {
    let mut app = app();
    let placeholder = spawn(
        &mut app,
        skeleton(Val::Px(120.0), Val::Px(16.0), Corner::ExtraSmall),
    );

    let full = theme(&app).color(ColorRole::SurfaceContainerHighest);
    let mut opacities = Vec::new();
    for _ in 0..10 {
        wait(&mut app, 0.1);
        let drawn = background(&app, placeholder);
        assert_eq!(drawn.with_alpha(1.0), full.with_alpha(1.0));
        opacities.push(drawn.alpha());
    }
    let (least, most) = opacities
        .iter()
        .fold((1.0_f32, 0.0_f32), |(least, most), &alpha| {
            (least.min(alpha), most.max(alpha))
        });
    assert!(most - least > 0.2, "it pulses: {opacities:?}");
    assert_eq!(rect(&app, placeholder).size(), Vec2::new(120.0, 16.0));
}
