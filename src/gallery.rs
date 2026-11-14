//! The play test: every component in every state, on one screen, all of it
//! live - the window `cargo run --example playtest` opens, and what
//! `tests/playtest.rs` drives headlessly as a player would. A game can show
//! it too (the-world's `--ui-gallery`), in its own theme.
//!
//! `D` turns the theme light or dark and `S` moves to the next seed colour,
//! as the app bar's buttons do.

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use crate::prelude::*;
use crate::theme::hex;

/// What the gallery needs to run: its log and its keys. Spawn it with
/// [`spawn_gallery`], under a camera of the game's.
pub struct GalleryPlugin;

impl Plugin for GalleryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GalleryLog>()
            .add_systems(Update, (theme_keys, show_log));
    }
}

/// The seeds `S` and the seed button move through: Material's baseline
/// first, then colours far apart on the wheel.
pub const SEEDS: [u32; 6] = [0x6750A4, 0x545CB0, 0x1B8A3C, 0xDD4C1D, 0x00838F, 0xB0005E];

/// What the gallery's controls did, newest last: shown on screen and read
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

/// The theme `brightness` and the seed after `seed`: the next in [`SEEDS`].
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

fn theme_keys(keys: Res<ButtonInput<KeyCode>>, mut theme: ResMut<MaterialTheme>) {
    if keys.just_pressed(KeyCode::KeyD) {
        *theme = other_brightness(&theme);
    }
    if keys.just_pressed(KeyCode::KeyS) {
        *theme = next_seed(&theme);
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
                bevy::ui_widgets::ScrollArea,
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

/// Notes `what` in the log when the control it is observed on activates.
fn log_on_activate(what: &'static str) -> impl FnMut(On<Activate>, ResMut<GalleryLog>) {
    move |_: On<Activate>, mut log: ResMut<GalleryLog>| log.note(what)
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

fn theme_section(page: &mut ChildSpawnerCommands) {
    section(page, "Theme", |card| {
        card.spawn(text(
            "Every colour role of the scheme",
            TypeRole::TitleSmall,
            ColorRole::OnSurfaceVariant,
        ));
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
        card.spawn(text(
            "The type scale",
            TypeRole::TitleSmall,
            ColorRole::OnSurfaceVariant,
        ));
        for role in TypeRole::ALL {
            card.spawn(text(format!("{role:?}"), role, ColorRole::OnSurface));
        }
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
    });
}

fn buttons_section(page: &mut ChildSpawnerCommands) {
    section(page, "Buttons", |card| {
        row(card, |buttons| {
            for variant in ButtonVariant::ALL {
                buttons
                    .spawn(button(format!("{variant:?}")).variant(variant).bundle())
                    .observe(log_on_activate(match variant {
                        ButtonVariant::Filled => "filled button",
                        ButtonVariant::Tonal => "tonal button",
                        ButtonVariant::Elevated => "elevated button",
                        ButtonVariant::Outlined => "outlined button",
                        ButtonVariant::Text => "text button",
                    }));
            }
        });
        row(card, |buttons| {
            for variant in ButtonVariant::ALL {
                buttons.spawn(
                    button(format!("{variant:?}"))
                        .variant(variant)
                        .icon(Glyph::Plus)
                        .enabled(false)
                        .bundle(),
                );
            }
        });
        row(card, |buttons| {
            for (index, variant) in IconButtonVariant::ALL.into_iter().enumerate() {
                let glyph = [Glyph::Menu, Glyph::Check, Glyph::Close, Glyph::MoreVert][index];
                buttons
                    .spawn(icon_button(glyph, variant))
                    .observe(log_on_activate("icon button"));
            }
            for size in [FabSize::Small, FabSize::Regular, FabSize::Large] {
                buttons
                    .spawn(fab(Glyph::Plus, size, None))
                    .observe(log_on_activate("fab"));
            }
            buttons
                .spawn(fab(Glyph::Plus, FabSize::Regular, Some("Create")))
                .observe(log_on_activate("extended fab"));
        });
    });
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
