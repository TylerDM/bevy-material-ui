//! The play test: every component in every state, on one screen, all of it
//! live - the window `cargo run --example playtest` opens, and what
//! `tests/playtest.rs` drives headlessly as a player would. A game can show
//! it too (the-world's `--ui-gallery`), in its own theme.
//!
//! Every control in it carries a [`Sample`] naming it; whatever it does - an
//! activation, a new value, a dialog answered - is noted in the
//! [`GalleryLog`] by that name and shown at the foot of the screen. `D` turns
//! the theme light or dark and `S` moves to the next seed colour, as the app
//! bar's buttons do.

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use crate::prelude::*;

/// What the gallery needs to run: its log, its keys, and the observers that
/// note what its controls do. Spawn it with [`spawn_gallery`], under a camera
/// of the game's.
pub struct GalleryPlugin;

impl Plugin for GalleryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GalleryLog>()
            .add_systems(Update, (theme_keys, show_log, sweep_progress))
            .add_observer(note_activate)
            .add_observer(note_value::<bool>)
            .add_observer(note_value::<f32>)
            .add_observer(note_value::<usize>)
            .add_observer(note_value::<String>)
            .add_observer(note_entity)
            .add_observer(note_tab)
            .add_observer(note_dialog)
            .add_observer(note_sheet)
            .add_observer(note_snackbar_action)
            .add_observer(remove_chip)
            .add_observer(note_app_bar)
            .add_observer(note_rail_fab);
    }
}

/// The seeds `S` and the seed button move through: Material's baseline
/// first, then colours far apart on the wheel.
pub const SEEDS: [u32; 6] = [0x6750A4, 0x545CB0, 0x1B8A3C, 0xDD4C1D, 0x00838F, 0xB0005E];

/// What the gallery's controls did, oldest first: shown on screen and read
/// by the headless play test.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct GalleryLog(pub Vec<String>);

impl GalleryLog {
    /// Notes that `what` happened.
    pub fn note(&mut self, what: impl Into<String>) {
        self.0.push(what.into());
    }

    /// The last thing that happened.
    pub fn last(&self) -> Option<&str> {
        self.0.last().map(String::as_str)
    }
}

/// The name a gallery control is noted by in the [`GalleryLog`], and found by
/// in the play test.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample(pub &'static str);

/// The gallery's root.
#[derive(Component, Debug)]
pub struct Gallery;

/// The app bar's button that turns the theme light or dark.
#[derive(Component, Debug)]
pub struct BrightnessToggle;

/// The app bar's button that moves to the next seed colour.
#[derive(Component, Debug)]
pub struct SeedToggle;

/// The line at the foot of the gallery saying what was last done.
#[derive(Component, Debug)]
struct LogLine;

/// A determinate progress indicator the gallery sweeps from empty to full, so
/// it is seen moving.
#[derive(Component, Debug)]
struct Sweeping;

/// The theme at the seed after its own: the next in [`SEEDS`], at the same
/// brightness and with the same faces.
pub fn next_seed(theme: &MaterialTheme) -> MaterialTheme {
    let now = theme.seed.unwrap_or(hex(SEEDS[0]));
    let at = SEEDS
        .iter()
        .position(|seed| hex(*seed) == now)
        .map_or(1, |index| (index + 1) % SEEDS.len());
    MaterialTheme::from_seed(hex(SEEDS[at]), theme.brightness()).with_fonts(theme.fonts.clone())
}

/// The theme turned light or dark.
pub fn other_brightness(theme: &MaterialTheme) -> MaterialTheme {
    theme.at(match theme.brightness() {
        Brightness::Light => Brightness::Dark,
        Brightness::Dark => Brightness::Light,
    })
}

fn theme_keys(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<bevy::input_focus::InputFocus>,
    typing: Query<(), With<bevy::ui_widgets::TextInput>>,
    mut theme: ResMut<MaterialTheme>,
) {
    // A letter typed into a field is the field's, not a shortcut.
    if focus.get().is_some_and(|focused| typing.contains(focused)) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyD) {
        *theme = other_brightness(&theme);
    }
    if keys.just_pressed(KeyCode::KeyS) {
        *theme = next_seed(&theme);
    }
}

/// The name of the [`Sample`] at or above `entity`.
fn sample_of(
    entity: Entity,
    samples: &Query<&Sample>,
    parents: &Query<&ChildOf>,
) -> Option<&'static str> {
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find_map(|at| samples.get(at).ok().map(|sample| sample.0))
}

fn note_activate(
    activate: On<Activate>,
    samples: Query<&Sample>,
    parents: Query<&ChildOf>,
    mut log: ResMut<GalleryLog>,
) {
    if let Some(name) = sample_of(activate.entity, &samples, &parents) {
        log.note(name);
    }
}

fn note_value<T: std::fmt::Debug + Send + Sync + 'static>(
    change: On<ValueChange<T>>,
    samples: Query<&Sample>,
    parents: Query<&ChildOf>,
    mut log: ResMut<GalleryLog>,
) {
    if let Some(name) = sample_of(change.source, &samples, &parents) {
        log.note(format!("{name}: {:?}", change.value));
    }
}

fn note_entity(
    change: On<ValueChange<Entity>>,
    samples: Query<&Sample>,
    parents: Query<&ChildOf>,
    radios: Query<&RadioOption>,
    mut log: ResMut<GalleryLog>,
) {
    if let Some(name) = sample_of(change.source, &samples, &parents) {
        match radios.get(change.value) {
            Ok(option) => log.note(format!("{name}: option {}", option.0)),
            Err(_) => log.note(format!("{name}: chose")),
        }
    }
}

fn note_tab(
    change: On<ValueChange<Option<Entity>>>,
    samples: Query<&Sample>,
    parents: Query<&ChildOf>,
    mut log: ResMut<GalleryLog>,
) {
    if let Some(name) = sample_of(change.source, &samples, &parents) {
        log.note(format!("{name}: tab"));
    }
}

fn note_dialog(answered: On<DialogResult>, mut log: ResMut<GalleryLog>) {
    log.note(match &answered.answer {
        DialogAnswer::Chose(key) => format!("dialog: {key}"),
        DialogAnswer::Dismissed => "dialog: dismissed".to_owned(),
    });
}

fn note_sheet(_: On<SheetClosed>, mut log: ResMut<GalleryLog>) {
    log.note("sheet closed");
}

fn note_snackbar_action(_: On<SnackbarAction>, mut log: ResMut<GalleryLog>) {
    log.note("snackbar: undo");
}

fn remove_chip(removed: On<ChipRemoved>, mut commands: Commands, mut log: ResMut<GalleryLog>) {
    commands.entity(removed.entity).try_despawn();
    log.note("chip removed");
}

fn note_app_bar(action: On<AppBarAction>, mut log: ResMut<GalleryLog>) {
    log.note(format!("app bar: {:?}", action.button));
}

fn note_rail_fab(_: On<RailFabActivated>, mut log: ResMut<GalleryLog>) {
    log.note("rail fab");
}

/// Keeps the log line saying what was last done.
fn show_log(log: Res<GalleryLog>, mut lines: Query<&mut Text, With<LogLine>>) {
    if !log.is_changed() {
        return;
    }
    if let Some(last) = log.last() {
        for mut line in &mut lines {
            line.0 = format!("Last: {last}");
        }
    }
}

/// Sweeps each [`Sweeping`] indicator from empty to full every four seconds.
fn sweep_progress(time: Res<Time>, mut indicators: Query<&mut Progress, With<Sweeping>>) {
    let share = (time.elapsed_secs() / 4.0).fract();
    for mut progress in &mut indicators {
        progress.0 = Some(share);
    }
}

/// Spawns the whole gallery, filling the window, and returns its root.
pub fn spawn_gallery(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Gallery,
            TabGroup::new(0),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Fill::new(ColorRole::Surface),
        ))
        .with_children(|root| {
            app_bar(root);
            root.spawn((
                ScrollArea,
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    flex_basis: Val::ZERO,
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Space::ExtraLarge.val()),
                    row_gap: Space::ExtraLarge.val(),
                    ..default()
                },
            ))
            .with_children(|page| {
                theme_section(page);
                papers_section(page);
                buttons_section(page);
                selection_section(page);
                input_section(page);
                lists_and_menus_section(page);
                navigation_section(page);
                feedback_section(page);
            });
            root.spawn((
                LogLine,
                text(
                    "Nothing done yet",
                    TypeRole::BodySmall,
                    ColorRole::OnSurfaceVariant,
                ),
                Node {
                    margin: UiRect::all(Space::Small.val()),
                    ..default()
                },
            ));
        })
        .id()
}

fn app_bar(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            height: Val::Px(64.0),
            padding: UiRect::horizontal(Space::Large.val()),
            column_gap: Space::Small.val(),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        },
        Fill::new(ColorRole::SurfaceContainer),
    ))
    .with_children(|bar| {
        bar.spawn((
            text(
                "Material for Bevy",
                TypeRole::TitleLarge,
                ColorRole::OnSurface,
            ),
            Node {
                flex_grow: 1.0,
                ..default()
            },
        ));
        bar.spawn((
            BrightnessToggle,
            button("Light / dark (D)").tonal().bundle(),
        ))
        .observe(
            |_: On<Activate>, mut theme: ResMut<MaterialTheme>, mut log: ResMut<GalleryLog>| {
                *theme = other_brightness(&theme);
                log.note(format!("theme {:?}", theme.brightness()));
            },
        );
        bar.spawn((SeedToggle, button("Next colour (S)").outlined().bundle()))
            .observe(
                |_: On<Activate>, mut theme: ResMut<MaterialTheme>, mut log: ResMut<GalleryLog>| {
                    *theme = next_seed(&theme);
                    log.note("seed changed");
                },
            );
    });
}

/// A titled section of the gallery: a card holding rows of parts.
fn section(
    page: &mut ChildSpawnerCommands,
    title: &str,
    build: impl FnOnce(&mut ChildSpawnerCommands),
) {
    page.spawn(card(CardKind::Outlined).gap(Space::Large.val()).bundle())
        .with_children(|card| {
            card.spawn(text(title, TypeRole::TitleLarge, ColorRole::OnSurface));
            build(card);
        });
}

/// A small heading inside a section.
fn heading(parent: &mut ChildSpawnerCommands, words: &str) {
    parent.spawn(text(
        words,
        TypeRole::TitleSmall,
        ColorRole::OnSurfaceVariant,
    ));
}

/// A row of parts that wraps.
fn row(parent: &mut ChildSpawnerCommands, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    parent
        .spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: Space::Medium.val(),
            row_gap: Space::Medium.val(),
            ..default()
        })
        .with_children(build);
}

/// A column of parts, `width` wide.
fn column(
    parent: &mut ChildSpawnerCommands,
    width: Val,
    build: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            width,
            flex_direction: FlexDirection::Column,
            row_gap: Space::Small.val(),
            ..default()
        })
        .with_children(build);
}

fn theme_section(page: &mut ChildSpawnerCommands) {
    section(page, "Theme", |card| {
        heading(card, "Every colour role of the scheme");
        row(card, |swatches| {
            for (surface, on) in [
                (ColorRole::Primary, ColorRole::OnPrimary),
                (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer),
                (ColorRole::Secondary, ColorRole::OnSecondary),
                (
                    ColorRole::SecondaryContainer,
                    ColorRole::OnSecondaryContainer,
                ),
                (ColorRole::Tertiary, ColorRole::OnTertiary),
                (ColorRole::TertiaryContainer, ColorRole::OnTertiaryContainer),
                (ColorRole::Error, ColorRole::OnError),
                (ColorRole::ErrorContainer, ColorRole::OnErrorContainer),
                (ColorRole::SurfaceContainerHighest, ColorRole::OnSurface),
                (ColorRole::InverseSurface, ColorRole::InverseOnSurface),
            ] {
                swatches
                    .spawn((
                        Node {
                            width: Val::Px(112.0),
                            height: Val::Px(56.0),
                            padding: UiRect::all(Space::Small.val()),
                            align_items: AlignItems::End,
                            border_radius: Corner::Small.all(),
                            ..default()
                        },
                        Fill::new(surface),
                    ))
                    .with_child(text(format!("{surface:?}"), TypeRole::LabelSmall, on));
            }
        });
        heading(card, "The type scale");
        for role in TypeRole::ALL {
            card.spawn(text(format!("{role:?}"), role, ColorRole::OnSurface));
        }
        heading(card, "Icons");
        row(card, |icons| {
            icons
                .spawn((
                    Node {
                        column_gap: Space::Medium.val(),
                        ..default()
                    },
                    ContentColor(Paint::new(ColorRole::OnSurfaceVariant)),
                ))
                .with_children(|glyphs| {
                    for glyph in Glyph::ALL {
                        glyphs.spawn(icon(glyph, ICON));
                    }
                });
        });
    });
}

fn papers_section(page: &mut ChildSpawnerCommands) {
    section(page, "Papers and cards", |sheet| {
        row(sheet, |papers| {
            for level in Elevation::ALL {
                papers
                    .spawn(
                        paper()
                            .elevation(level)
                            .width(Val::Px(120.0))
                            .height(Val::Px(80.0))
                            .bundle(),
                    )
                    .with_child(text(
                        format!("{level:?}"),
                        TypeRole::LabelLarge,
                        ColorRole::OnSurface,
                    ));
            }
        });
        row(sheet, |cards| {
            for kind in [CardKind::Elevated, CardKind::Filled, CardKind::Outlined] {
                cards
                    .spawn(card(kind).width(Val::Px(200.0)).bundle())
                    .with_children(|inside| {
                        inside.spawn(text(
                            format!("{kind:?} card"),
                            TypeRole::TitleMedium,
                            ColorRole::OnSurface,
                        ));
                        inside.spawn(text(
                            "Supporting text on a card.",
                            TypeRole::BodyMedium,
                            ColorRole::OnSurfaceVariant,
                        ));
                    });
            }
        });
        sheet.spawn(divider(false));
        row(sheet, |skeletons| {
            skeletons.spawn(skeleton(Val::Px(40.0), Val::Px(40.0), Corner::Full));
            skeletons.spawn(skeleton(Val::Px(200.0), Val::Px(16.0), Corner::ExtraSmall));
            skeletons.spawn(skeleton(Val::Px(120.0), Val::Px(16.0), Corner::ExtraSmall));
        });
    });
}

fn buttons_section(page: &mut ChildSpawnerCommands) {
    section(page, "Buttons", |card| {
        row(card, |buttons| {
            for (variant, name) in ButtonVariant::ALL.into_iter().zip([
                "filled button",
                "tonal button",
                "elevated button",
                "outlined button",
                "text button",
            ]) {
                buttons.spawn((
                    Sample(name),
                    button(format!("{variant:?}")).variant(variant).bundle(),
                ));
            }
        });
        row(card, |buttons| {
            for variant in ButtonVariant::ALL {
                buttons.spawn((
                    Sample("disabled button"),
                    button(format!("{variant:?}"))
                        .variant(variant)
                        .icon(Glyph::Plus)
                        .enabled(false)
                        .bundle(),
                ));
            }
        });
        row(card, |buttons| {
            for (variant, glyph) in IconButtonVariant::ALL.into_iter().zip([
                Glyph::Menu,
                Glyph::Check,
                Glyph::Close,
                Glyph::MoreVert,
            ]) {
                buttons.spawn((Sample("icon button"), icon_button(glyph, variant)));
            }
            for size in [FabSize::Small, FabSize::Regular, FabSize::Large] {
                buttons.spawn((Sample("fab"), fab(Glyph::Plus, size, None)));
            }
            buttons.spawn((
                Sample("extended fab"),
                fab(Glyph::Plus, FabSize::Regular, Some("Create")),
            ));
        });
        heading(card, "Segmented buttons");
        row(card, |segments| {
            segments.spawn((
                Sample("view"),
                segmented_buttons(["Day", "Week", "Month"], Some(1)).bundle(),
            ));
            segments.spawn((
                Sample("sizes"),
                segmented_buttons(["S", "M", "L", "XL"], None)
                    .multi([0, 2])
                    .disable(3)
                    .bundle(),
            ));
        });
    });
}

fn selection_section(page: &mut ChildSpawnerCommands) {
    section(page, "Selection", |card| {
        heading(card, "Checkboxes");
        row(card, |boxes| {
            boxes.spawn((Sample("checkbox"), checkbox().bundle()));
            boxes.spawn((
                Sample("checked checkbox"),
                checkbox().checked(true).bundle(),
            ));
            boxes.spawn((
                Sample("indeterminate checkbox"),
                checkbox().indeterminate(true).bundle(),
            ));
            boxes.spawn((
                Sample("error checkbox"),
                checkbox().error(true).checked(true).bundle(),
            ));
            boxes.spawn((
                Sample("disabled checkbox"),
                checkbox().checked(true).enabled(false).bundle(),
            ));
            boxes.spawn((
                Sample("labelled checkbox"),
                checkbox().labelled("Show coordinates"),
            ));
        });
        heading(card, "Radio buttons");
        card.spawn((
            Sample("difficulty"),
            radio_group(["Peaceful", "Easy", "Normal", "Hard"], Some(2))
                .horizontal()
                .disable(0)
                .bundle(),
        ));
        heading(card, "Switches");
        row(card, |switches| {
            switches.spawn((Sample("switch"), switch().bundle()));
            switches.spawn((Sample("on switch"), switch().on(true).bundle()));
            switches.spawn((
                Sample("icon switch"),
                switch().on(true).icons(true).bundle(),
            ));
            switches.spawn((
                Sample("disabled switch"),
                switch().on(true).enabled(false).bundle(),
            ));
            switches.spawn((Sample("labelled switch"), switch().labelled("Music")));
        });
        heading(card, "Sliders");
        column(card, Val::Px(420.0), |sliders| {
            sliders.spawn((Sample("volume"), slider(60.0, 0.0..=100.0).bundle()));
            sliders.spawn((
                Sample("view distance"),
                slider(8.0, 2.0..=16.0).discrete(2.0).bundle(),
            ));
            sliders.spawn((
                Sample("quality"),
                notched_slider(["Low", "Medium", "High", "Ultra", "Max"], Some(2)).bundle(),
            ));
            sliders.spawn((
                Sample("disabled slider"),
                slider(30.0, 0.0..=100.0).enabled(false).bundle(),
            ));
        });
    });
}

fn input_section(page: &mut ChildSpawnerCommands) {
    section(page, "Text and chips", |card| {
        row(card, |fields| {
            fields.spawn((
                Sample("name"),
                text_field("World name")
                    .supporting("Shown in the server list")
                    .bundle(),
            ));
            fields.spawn((
                Sample("seed"),
                text_field("Seed")
                    .outlined()
                    .leading_icon(Glyph::Dot)
                    .clearable()
                    .text("8675309")
                    .bundle(),
            ));
            fields.spawn((Sample("motto"), text_field("Motto").max_length(24).bundle()));
            fields.spawn((
                Sample("port"),
                text_field("Port")
                    .outlined()
                    .text("99999")
                    .error("Ports run to 65535")
                    .bundle(),
            ));
            fields.spawn((
                Sample("disabled field"),
                text_field("Server").text("Offline").enabled(false).bundle(),
            ));
        });
        heading(card, "Search");
        column(card, Val::Px(480.0), |search| {
            search.spawn((Sample("search"), search_bar("Search blocks", Glyph::Menu)));
            search.spawn(search_results()).with_children(|results| {
                for (headline, supporting) in [
                    ("Oak log", Some("Wood")),
                    ("Oak planks", Some("Wood, crafted")),
                    ("Oak sapling", None),
                ] {
                    results.spawn((Sample("result"), search_result(headline, supporting)));
                }
            });
        });
        heading(card, "Chips");
        card.spawn(chip_set()).with_children(|chips| {
            chips.spawn((
                Sample("assist chip"),
                chip("Directions").icon(Glyph::ArrowForward).bundle(),
            ));
            chips.spawn((Sample("elevated chip"), chip("Share").elevated().bundle()));
            chips.spawn((Sample("filter chip"), chip("Ores").filter(false).bundle()));
            chips.spawn((
                Sample("chosen filter chip"),
                chip("Trees").filter(true).bundle(),
            ));
            chips.spawn((Sample("input chip"), chip("Ada").input().bundle()));
            chips.spawn((Sample("input chip"), chip("Grace").input().bundle()));
            chips.spawn((
                Sample("suggestion chip"),
                chip("Try a bigger world").suggestion().bundle(),
            ));
            chips.spawn((
                Sample("disabled chip"),
                chip("Locked").enabled(false).bundle(),
            ));
        });
    });
}

fn lists_and_menus_section(page: &mut ChildSpawnerCommands) {
    section(page, "Lists, tabs and menus", |card| {
        row(card, |sides| {
            column(sides, Val::Px(360.0), |left| {
                heading(left, "List");
                left.spawn((Sample("players"), list()))
                    .with_children(|rows| {
                        rows.spawn(list_item("Ada").avatar("AL").supporting("Online").bundle());
                        rows.spawn(
                            list_item("Grace")
                                .avatar("GH")
                                .supporting("In the nether")
                                .trailing_text("2h")
                                .bundle(),
                        );
                        rows.spawn(
                            list_item("Settings")
                                .leading_icon(Glyph::Menu)
                                .three_line("Sound, graphics, controls and the interface's scale")
                                .trailing_icon(Glyph::ChevronRight)
                                .bundle(),
                        );
                        rows.spawn(list_item("Banned").enabled(false).bundle());
                    });
            });
            column(sides, Val::Px(420.0), |right| {
                heading(right, "Tabs");
                let row = right
                    .spawn((Sample("primary tabs"), tabs(TabsKind::Primary).bundle()))
                    .with_children(|tabs| {
                        tabs.spawn(tab(TabsKind::Primary, "Blocks").icon(Glyph::Dot).bundle());
                        tabs.spawn(tab(TabsKind::Primary, "Items").icon(Glyph::Plus).bundle());
                        tabs.spawn(tab(TabsKind::Primary, "Mobs").icon(Glyph::Check).bundle());
                    })
                    .id();
                right.spawn(tab_panels(row)).with_children(|panels| {
                    for (index, words) in ["Every block", "Every item", "Every creature"]
                        .into_iter()
                        .enumerate()
                    {
                        panels.spawn(tab_panel(index)).with_child(text(
                            words,
                            TypeRole::BodyMedium,
                            ColorRole::OnSurfaceVariant,
                        ));
                    }
                });
                right
                    .spawn((Sample("secondary tabs"), tabs(TabsKind::Secondary).bundle()))
                    .with_children(|tabs| {
                        tabs.spawn(tab(TabsKind::Secondary, "Overview").bundle());
                        tabs.spawn(tab(TabsKind::Secondary, "Details").bundle());
                        tabs.spawn(tab(TabsKind::Secondary, "Locked").enabled(false).bundle());
                    });
                heading(right, "Menu and select");
                row_of(right, |menus| {
                    menus.spawn((
                        Sample("edit menu"),
                        menu(button("Edit").outlined().icon(Glyph::ChevronDown))
                            .item(menu_item("Cut").leading(Glyph::Close).shortcut("Ctrl+X"))
                            .item(menu_item("Copy").shortcut("Ctrl+C"))
                            .item(menu_item("Paste").shortcut("Ctrl+V").enabled(false))
                            .divider()
                            .item(menu_item("Select all").selected(true))
                            .bundle(),
                    ));
                    menus.spawn((
                        Sample("game mode"),
                        select("Game mode", ["Survival", "Creative", "Adventure"])
                            .selected(0)
                            .bundle(),
                    ));
                });
            });
        });
    });
}

/// [`row`], under a name that a local `row` entity doesn't shadow.
fn row_of(parent: &mut ChildSpawnerCommands, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    row(parent, build);
}

fn navigation_section(page: &mut ChildSpawnerCommands) {
    section(page, "Navigation", |card| {
        heading(card, "Top app bars");
        column(card, Val::Percent(100.0), |bars| {
            bars.spawn(
                top_app_bar("Small")
                    .navigation(Glyph::ArrowBack)
                    .action(Glyph::MoreVert)
                    .bundle(),
            );
            bars.spawn(
                top_app_bar("Centre aligned")
                    .kind(AppBarKind::CenterAligned)
                    .navigation(Glyph::Menu)
                    .bundle(),
            );
            bars.spawn(
                top_app_bar("Medium")
                    .kind(AppBarKind::Medium)
                    .navigation(Glyph::ArrowBack)
                    .bundle(),
            );
            bars.spawn(
                top_app_bar("Large")
                    .kind(AppBarKind::Large)
                    .navigation(Glyph::ArrowBack)
                    .action(Glyph::Plus)
                    .bundle(),
            );
        });
        heading(card, "Bar, rail and drawer");
        row(card, |places| {
            let destinations = || {
                vec![
                    destination(Glyph::Dot, "Play"),
                    destination(Glyph::Plus, "Build"),
                    destination(Glyph::Check, "Quests"),
                    destination(Glyph::Menu, "Settings"),
                ]
            };
            places
                .spawn(Node {
                    width: Val::Px(400.0),
                    ..default()
                })
                .with_child((Sample("navigation bar"), navigation_bar(destinations(), 0)));
            places
                .spawn(Node {
                    height: Val::Px(400.0),
                    ..default()
                })
                .with_child((
                    Sample("navigation rail"),
                    navigation_rail(destinations(), 1, Some(Glyph::Plus.into())),
                ));
            places.spawn((
                Sample("navigation drawer"),
                navigation_drawer()
                    .headline("Worlds")
                    .destination(destination(Glyph::Dot, "Island"))
                    .destination(destination(Glyph::Dot, "Desert"))
                    .headline("Servers")
                    .destination(destination(Glyph::Check, "Friends"))
                    .selected(0)
                    .bundle(),
            ));
        });
    });
}

fn feedback_section(page: &mut ChildSpawnerCommands) {
    section(page, "Feedback and overlays", |card| {
        heading(card, "Dialogs, sheets and snackbars");
        row(card, |buttons| {
            buttons
                .spawn((
                    Sample("open dialog"),
                    button("Open a dialog").tonal().bundle(),
                ))
                .observe(|_: On<Activate>, mut commands: Commands| {
                    commands.spawn(
                        confirm_dialog(
                            "Delete this world?",
                            "Its blocks, its chests and its maps are gone for good.",
                            "Delete",
                            "Keep",
                        )
                        .bundle(),
                    );
                });
            buttons
                .spawn((
                    Sample("open rich dialog"),
                    button("A dialog with a form").tonal().bundle(),
                ))
                .observe(|_: On<Activate>, mut commands: Commands| {
                    commands.spawn(
                        dialog("New world")
                            .icon(Glyph::Plus)
                            .text("Pick a name; the seed is chosen for you.")
                            .body(|body| {
                                body.spawn(text_field("World name").bundle());
                            })
                            .action("cancel", "Cancel")
                            .action("create", "Create")
                            .bundle(),
                    );
                });
            buttons
                .spawn((
                    Sample("open sheet"),
                    button("Open a sheet").tonal().bundle(),
                ))
                .observe(|_: On<Activate>, mut commands: Commands| {
                    commands.spawn(bottom_sheet(|sheet| {
                        sheet.spawn(text("Share", TypeRole::TitleLarge, ColorRole::OnSurface));
                        sheet.spawn(text(
                            "Send this world's seed to a friend.",
                            TypeRole::BodyMedium,
                            ColorRole::OnSurfaceVariant,
                        ));
                    }));
                });
            buttons
                .spawn((Sample("show snackbar"), button("Show a snackbar").bundle()))
                .observe(|_: On<Activate>, mut snackbars: ResMut<Snackbars>| {
                    snackbars.show("World saved");
                });
            buttons
                .spawn((
                    Sample("show undo snackbar"),
                    button("Snackbar with undo").outlined().bundle(),
                ))
                .observe(|_: On<Activate>, mut snackbars: ResMut<Snackbars>| {
                    snackbars
                        .show_snackbar(Snackbar::new("Block broken").action("Undo").closable());
                });
        });
        heading(card, "Tooltips");
        row(card, |tips| {
            tips.spawn((
                Sample("plain tooltip"),
                icon_button(Glyph::Plus, IconButtonVariant::Standard),
                Tooltip::plain("Add a world"),
            ));
            tips.spawn((
                Sample("rich tooltip"),
                button("Seeds").text().bundle(),
                Tooltip::rich("A seed makes the same world every time.")
                    .title("Seeds")
                    .action("Learn more"),
            ));
        });
        heading(card, "Progress");
        row(card, |indicators| {
            indicators
                .spawn(Node {
                    width: Val::Px(240.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Space::Large.val(),
                    ..default()
                })
                .with_children(|lines| {
                    lines.spawn(linear_progress(Some(0.6)));
                    lines.spawn((Sweeping, linear_progress(Some(0.0))));
                    lines.spawn(linear_progress(None));
                });
            indicators.spawn(circular_progress(Some(0.75)));
            indicators.spawn((Sweeping, circular_progress(Some(0.0))));
            indicators.spawn(circular_progress(None));
        });
        heading(card, "Badges and avatars");
        row(card, |badges| {
            for count in [None, Some(3), Some(1500)] {
                badges
                    .spawn((
                        Node {
                            width: Val::Px(ICON),
                            height: Val::Px(ICON),
                            ..default()
                        },
                        ContentColor(Paint::new(ColorRole::OnSurfaceVariant)),
                    ))
                    .with_children(|holder| {
                        holder.spawn(icon(Glyph::Menu, ICON));
                        holder.spawn(badge(count.map_or(Badge::Dot, Badge::Count)));
                    });
            }
            badges.spawn(avatar(
                AvatarFace::Initials(initials("Ada Lovelace")),
                AVATAR,
            ));
            badges.spawn(avatar(AvatarFace::Initials(initials("Plato")), AVATAR));
        });
        heading(card, "Scroll area");
        card.spawn(scroll_area(Val::Px(160.0), |rows| {
            for index in 1..=30 {
                rows.spawn(text(
                    format!("Chunk {index}"),
                    TypeRole::BodyMedium,
                    ColorRole::OnSurface,
                ));
            }
        }));
    });
}
