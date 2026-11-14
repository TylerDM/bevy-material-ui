//! The Guild Clerk, played headlessly: `examples/guild` built at real window
//! sizes and played as a clerk would - contracts typed and chosen, a party
//! picked, filed or returned, taken back, the day closed and the next
//! opened - checking the guild's books and what the screen shows.

#[path = "../examples/guild/game.rs"]
mod game;

use bevy::ecs::system::RunSystemOnce;
use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::Button as Pressable;
use bevy_material_ui::prelude::*;
use bevy_material_ui::testing::{
    WINDOWS, click, descendants, find, find_all, focus, headless_app, hover, press, settle,
    type_text, wait, words,
};
use game::{
    Action, COLOURS, Class, DANGER_PAY_FROM, DayOverDialog, Draft, Field, Guild, GuildPlugin,
    GuildRoot, HealerHint, Member, REQUESTS, ROSTER, Recruit, spawn_guild,
};

/// A headless app showing the guild in a window `width` x `height`, its
/// first client already in.
fn guild_at(width: u32, height: u32) -> (App, Entity) {
    let mut app = headless_app(width, height, 1.0);
    app.add_plugins(GuildPlugin);
    let root = app
        .world_mut()
        .run_system_once(|mut commands: Commands| spawn_guild(&mut commands))
        .expect("the guild is spawned");
    settle(&mut app);
    wait(&mut app, 1.5);
    (app, root)
}

fn guild() -> (App, Entity) {
    guild_at(1440, 960)
}

fn books(app: &App) -> Guild {
    app.world().resource::<Guild>().clone()
}

/// The part carrying `field`.
fn field(app: &App, root: Entity, field: Field) -> Entity {
    find_all::<Field>(app, root)
        .into_iter()
        .find(|part| app.world().get::<Field>(*part) == Some(&field))
        .unwrap_or_else(|| panic!("a part for {field:?}"))
}

/// The part doing `action`.
fn action(app: &App, root: Entity, action: Action) -> Entity {
    find_all::<Action>(app, root)
        .into_iter()
        .find(|part| app.world().get::<Action>(*part) == Some(&action))
        .unwrap_or_else(|| panic!("a part doing {action:?}"))
}

/// The first part at or below `root` whose words are `label`, or the
/// pressable part it is inside.
fn saying(app: &App, root: Entity, label: &str) -> Entity {
    let text = std::iter::once(root)
        .chain(descendants(app, root))
        .find(|part| {
            app.world()
                .get::<Text>(*part)
                .is_some_and(|text| text.0 == label)
        })
        .unwrap_or_else(|| panic!("something saying {label:?}"));
    std::iter::once(text)
        .chain(
            std::iter::successors(app.world().get::<ChildOf>(text), |parent| {
                app.world().get::<ChildOf>(parent.parent())
            })
            .map(ChildOf::parent),
        )
        .take(4)
        .find(|part| app.world().get::<Pressable>(*part).is_some())
        .unwrap_or(text)
}

/// The first part of `T` anywhere in the world.
fn anywhere<T: Component>(app: &mut App) -> Option<Entity> {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .next()
}

/// Whether `part` and every part above it are laid out.
fn shown(app: &App, part: Entity) -> bool {
    std::iter::once(part)
        .chain(
            std::iter::successors(app.world().get::<ChildOf>(part), |parent| {
                app.world().get::<ChildOf>(parent.parent())
            })
            .map(ChildOf::parent),
        )
        .all(|at| {
            app.world()
                .get::<Node>(at)
                .is_none_or(|node| node.display != Display::None)
        })
}

/// The adventurers the roster shows.
fn roster_shown(app: &App, root: Entity) -> Vec<&'static str> {
    find_all::<Recruit>(app, root)
        .into_iter()
        .filter(|row| shown(app, *row))
        .map(|row| ROSTER[app.world().get::<Recruit>(row).expect("a recruit").0].name)
        .collect()
}

/// The part below `within` carrying `T` equal to `value`.
fn with<T: Component + PartialEq>(app: &App, within: Entity, value: &T) -> Entity {
    find_all::<T>(app, within)
        .into_iter()
        .find(|part| app.world().get::<T>(*part) == Some(value))
        .expect("the part")
}

fn type_into(app: &mut App, root: Entity, into: Field, words: &str) {
    let part = field(app, root, into);
    click(app, part);
    type_text(app, words);
}

/// Opens the select for `into` and chooses option `index`.
fn choose(app: &mut App, root: Entity, into: Field, index: usize) {
    let select = field(app, root, into);
    let opener = find::<MenuOpener>(app, select).expect("the select's field");
    click(app, opener);
    let option = with(app, select, &MenuChoice(index));
    click(app, option);
}

/// Clicks the words `label` of the labelled control for `of`.
fn click_label(app: &mut App, root: Entity, of: Field, label: &str) {
    let control = field(app, root, of);
    let words = saying(app, control, label);
    click(app, words);
}

/// Fills in the contract for request `index` rightly, with `party` (indices
/// of the roster) chosen from the roster.
fn fill(app: &mut App, root: Entity, index: usize, party: &[usize]) {
    let request = REQUESTS[index];
    type_into(app, root, Field::Title, "The job");
    type_into(app, root, Field::Reward, &request.budget.to_string());
    let kind = field(app, root, Field::Kind);
    let segment = with(app, kind, &Segment(request.kind));
    click(app, segment);
    choose(app, root, Field::Region, request.region);
    let danger = field(app, root, Field::Danger);
    let option = with(app, danger, &RadioOption(request.danger));
    click(app, option);
    let size = field(app, root, Field::Size);
    focus(app, size);
    press(app, KeyCode::Home, Key::Home);
    for _ in 1..request.party {
        press(app, KeyCode::ArrowRight, Key::ArrowRight);
    }
    if request.rush {
        click_label(app, root, Field::Rush, "Rush job");
    }
    click_label(
        app,
        root,
        Field::Terms,
        "The client accepts the guild's terms",
    );
    if request.danger >= DANGER_PAY_FROM {
        click_label(app, root, Field::DangerPay, "Danger pay agreed");
    }
    for member in party {
        let row = with(app, root, &Recruit(*member));
        click(app, row);
    }
}

/// Approves the contract and answers its dialog "File it".
fn file(app: &mut App, root: Entity) {
    let approve = action(app, root, Action::Approve);
    click(app, approve);
    let surface = anywhere::<DialogSurface>(app).expect("the dialog asking to file it");
    let file_it = saying(app, surface, "File it");
    click(app, file_it);
}

/// Returns the client at the desk, and waits out the snackbar saying so,
/// which covers the foot of a short window.
fn return_client(app: &mut App, root: Entity) {
    let back = action(app, root, Action::Return);
    click(app, back);
    wait(app, SNACKBAR_SECONDS + 1.0);
}

/// Goes to the place at `index` of the rail.
fn go(app: &mut App, root: Entity, index: usize) {
    let rail = field(app, root, Field::Places);
    let destination = with(app, rail, &NavigationDestination(index));
    click(app, destination);
}

#[test]
fn a_right_contract_is_filed_for_three_reputation_and_its_party_goes_out() {
    let (mut app, root) = guild();
    fill(&mut app, root, 0, &[0, 1]);
    assert_eq!(app.world().resource::<Draft>().party, [0, 1]);
    file(&mut app, root);

    let books = books(&app);
    assert_eq!(books.at, 1);
    assert_eq!(books.reputation, 3);
    assert_eq!(books.gold, 6, "a tenth of the reward");
    assert!(
        anywhere::<DialogSurface>(&mut app).is_none(),
        "the dialog goes"
    );
    let note = anywhere::<SnackbarView>(&mut app).expect("a snackbar saying so");
    assert!(
        words(&app, note).contains("well done"),
        "{}",
        words(&app, note)
    );

    wait(&mut app, 1.5);
    let roster = roster_shown(&app, root);
    assert!(
        !roster.contains(&ROSTER[0].name),
        "Aldric is out: {roster:?}"
    );
    assert!(!roster.contains(&ROSTER[1].name), "Bryn is out: {roster:?}");
    assert_eq!(roster.len(), ROSTER.len() - 2);
}

#[test]
fn a_contract_with_mistakes_costs_reputation_and_the_ledger_names_them() {
    let (mut app, root) = guild();
    type_into(&mut app, root, Field::Title, "Sheep");
    type_into(&mut app, root, Field::Reward, "60");
    click_label(
        &mut app,
        root,
        Field::Terms,
        "The client accepts the guild's terms",
    );
    file(&mut app, root);
    assert_eq!(
        books(&app).reputation,
        -3,
        "many mistakes cost three at most"
    );

    go(&mut app, root, 1);
    let ledger = words(&app, root);
    assert!(
        ledger.contains("Filed, wrong on the kind of quest"),
        "{ledger}"
    );
    assert!(ledger.contains("Farmer Bram"), "{ledger}");
}

#[test]
fn approving_without_a_title_or_a_reward_shows_errors_and_files_nothing() {
    let (mut app, root) = guild();
    let approve = action(&app, root, Action::Approve);
    click(&mut app, approve);
    assert!(
        anywhere::<DialogSurface>(&mut app).is_none(),
        "nothing to file"
    );
    let error = |app: &App, of: Field| {
        app.world()
            .get::<TextField>(field(app, root, of))
            .expect("a text field")
            .error
            .clone()
    };
    assert_eq!(
        error(&app, Field::Title).as_deref(),
        Some("A contract needs a title")
    );
    assert_eq!(
        error(&app, Field::Reward).as_deref(),
        Some("Gold, in whole coins")
    );
    assert_eq!(books(&app).at, 0);

    type_into(&mut app, root, Field::Title, "Wolves");
    assert_eq!(error(&app, Field::Title), None, "typing fixes it");
    assert!(
        error(&app, Field::Reward).is_some(),
        "the other is still wrong"
    );
}

#[test]
fn returning_a_good_client_costs_reputation_returning_a_crime_earns_it_and_the_last_ends_the_day() {
    let (mut app, root) = guild();
    for _ in 0..3 {
        return_client(&mut app, root);
    }
    assert_eq!(books(&app).reputation, -6);
    assert!(anywhere::<DayOverDialog>(&mut app).is_none());

    let stranger = words(&app, root);
    assert!(stranger.contains("A hooded stranger"), "{stranger}");
    return_client(&mut app, root);
    assert_eq!(books(&app).reputation, -4);
    let day_over = anywhere::<DayOverDialog>(&mut app).expect("the day is over");
    assert!(words(&app, day_over).contains("Day 1 is over"));

    let tomorrow = saying(&app, day_over, "Open tomorrow");
    click(&mut app, tomorrow);
    let books = books(&app);
    assert_eq!(books.day, 2);
    assert_eq!(books.today, [4, 5, 6, 7]);
    assert_eq!(books.at, 0);
}

#[test]
fn undo_takes_the_contract_just_filed_back_onto_the_desk() {
    let (mut app, root) = guild();
    fill(&mut app, root, 0, &[0, 1]);
    file(&mut app, root);
    assert_eq!(books(&app).at, 1);

    let undo = anywhere::<SnackbarActionButton>(&mut app).expect("the snackbar's undo");
    click(&mut app, undo);
    let books = books(&app);
    assert_eq!(books.at, 0);
    assert_eq!(books.reputation, 0);
    assert_eq!(books.gold, 0);
    let draft = app.world().resource::<Draft>().clone();
    assert_eq!(draft.party, [0, 1]);
    let title = field(&app, root, Field::Title);
    assert_eq!(
        app.world()
            .get::<InputText>(title)
            .expect("its text")
            .text(),
        "The job",
        "the form is as it was filed"
    );
    assert_eq!(find_all::<Member>(&app, root).len(), 2, "with its party");
}

#[test]
fn the_rail_fab_opens_the_next_day_only_once_every_client_is_dealt_with() {
    let (mut app, root) = guild();
    let fab = find::<RailFab>(&app, root).expect("the rail's FAB");
    click(&mut app, fab);
    assert_eq!(books(&app).day, 1, "clients are still waiting");
    let note = anywhere::<SnackbarView>(&mut app).expect("a snackbar saying so");
    assert!(words(&app, note).contains("still waiting"));
    wait(&mut app, SNACKBAR_SECONDS + 1.0);

    for _ in 0..4 {
        return_client(&mut app, root);
    }
    // The day's dialog is dismissed; the closed doors stay.
    press(&mut app, KeyCode::Escape, Key::Escape);
    assert!(anywhere::<DayOverDialog>(&mut app).is_none());
    assert!(words(&app, root).contains("The doors are closed"));

    let fab = find::<RailFab>(&app, root).expect("the rail's FAB");
    click(&mut app, fab);
    assert_eq!(books(&app).day, 2);
}

#[test]
fn the_roster_narrows_to_a_search_and_to_the_classes_chosen() {
    let (mut app, root) = guild();
    assert_eq!(roster_shown(&app, root).len(), ROSTER.len());

    type_into(&mut app, root, Field::Search, "dara");
    assert_eq!(roster_shown(&app, root), ["Dara Quill"]);
    let search = field(&app, root, Field::Search);
    let clear = find::<SearchBar>(&app, root)
        .and_then(|bar| app.world().get::<SearchBar>(bar))
        .map(SearchBar::clear)
        .expect("the search's clear button");
    assert!(find_all::<Pressable>(&app, search).contains(&clear));
    click(&mut app, clear);
    assert_eq!(roster_shown(&app, root).len(), ROSTER.len());

    let healers = field(&app, root, Field::Class(Class::Healer));
    click(&mut app, healers);
    assert_eq!(
        roster_shown(&app, root),
        ["Dara Quill", "Hale Brook", "Juno Fair"]
    );
}

#[test]
fn a_dangerous_party_without_a_healer_is_offered_one_and_taking_the_offer_shows_the_healers() {
    let (mut app, root) = guild();
    let hint = find::<HealerHint>(&app, root).expect("the suggestion");
    assert!(!shown(&app, hint), "no danger chosen yet");

    let danger = field(&app, root, Field::Danger);
    let high = with(&app, danger, &RadioOption(2));
    click(&mut app, high);
    assert!(shown(&app, hint), "a high-danger party lacks a healer");

    click(&mut app, hint);
    assert_eq!(
        roster_shown(&app, root),
        ["Dara Quill", "Hale Brook", "Juno Fair"]
    );
    let healers = field(&app, root, Field::Class(Class::Healer));
    assert!(
        app.world().get::<Checked>(healers).is_some(),
        "its chip is on"
    );

    let dara = with(&app, root, &Recruit(3));
    click(&mut app, dara);
    assert!(!shown(&app, hint), "the party has its healer");
    let chip = find::<Member>(&app, root).expect("Dara's chip");
    let remove = find_all::<Pressable>(&app, chip)
        .into_iter()
        .find(|part| *part != chip)
        .expect("the chip's remove button");
    click(&mut app, remove);
    assert!(app.world().resource::<Draft>().party.is_empty());
    assert!(shown(&app, hint), "and lacks it again");
}

#[test]
fn the_settings_turn_the_guild_dark_and_recolour_it() {
    let (mut app, root) = guild();
    go(&mut app, root, 2);
    click_label(&mut app, root, Field::Dark, "Dark theme");
    assert_eq!(
        app.world().resource::<MaterialTheme>().brightness(),
        Brightness::Dark
    );
    choose(&mut app, root, Field::Colours, 2);
    let theme = app.world().resource::<MaterialTheme>().clone();
    assert_eq!(theme.seed, Some(hex(COLOURS[2].1)));
    assert_eq!(theme.brightness(), Brightness::Dark, "it stays dark");
    assert_eq!(
        app.world().get::<BackgroundColor>(root).expect("a fill").0,
        theme.color(ColorRole::Surface)
    );
}

#[test]
fn the_courier_shows_a_skeleton_until_the_client_comes_in() {
    let mut app = headless_app(1440, 960, 1.0);
    app.add_plugins(GuildPlugin);
    let root = app
        .world_mut()
        .run_system_once(|mut commands: Commands| spawn_guild(&mut commands))
        .expect("the guild is spawned");
    settle(&mut app);
    let skeleton = find::<Skeleton>(&app, root).expect("a skeleton");
    let said = saying(&app, root, &format!("\"{}\"", REQUESTS[0].words));
    assert!(shown(&app, skeleton));
    assert!(!shown(&app, said), "the client isn't in yet");
    wait(&mut app, 1.5);
    assert!(!shown(&app, skeleton), "the courier is gone");
    assert!(shown(&app, said), "and the client is in");
}

#[test]
fn the_handbook_opens_from_the_app_bar_and_from_the_rewards_tooltip() {
    let (mut app, root) = guild();
    let bar = find::<AppBar>(&app, root).expect("the app bar");
    let more = find::<AppBarButton>(&app, bar).expect("its action");
    click(&mut app, more);
    let sheet = anywhere::<SheetSurface>(&mut app).expect("the handbook");
    assert!(words(&app, sheet).contains("The clerk's handbook"));
    wait(&mut app, 0.2);
    press(&mut app, KeyCode::Escape, Key::Escape);
    assert!(anywhere::<SheetSurface>(&mut app).is_none());

    let reward = field(&app, root, Field::Reward);
    hover(&mut app, reward);
    wait(&mut app, TOOLTIP_DELAY + 0.3);
    let tip = anywhere::<TooltipView>(&mut app).expect("the tooltip");
    assert!(words(&app, tip).contains("never more"));
    let open = saying(&app, tip, "The handbook");
    click(&mut app, open);
    assert!(anywhere::<SheetSurface>(&mut app).is_some());
}

#[test]
fn every_place_lays_out_at_every_window_size_with_every_part_on_a_ui_parent() {
    for &(width, height, _) in WINDOWS {
        let (mut app, root) = guild_at(width, height);
        for place in 0..3 {
            go(&mut app, root, place);
            for part in descendants(&app, root) {
                if let Some(node) = app.world().get::<ComputedNode>(part) {
                    assert!(
                        node.size.is_finite(),
                        "{width}x{height}, place {place}: a part laid out to {:?}",
                        node.size
                    );
                }
            }
        }
        let mut orphans = Vec::new();
        let mut nodes = app
            .world_mut()
            .query_filtered::<(Entity, &ChildOf), With<Node>>();
        for (entity, parent) in nodes.iter(app.world()) {
            if app.world().get::<Node>(parent.parent()).is_none() {
                orphans.push(entity);
            }
        }
        assert!(orphans.is_empty(), "{width}x{height}: {orphans:?}");
    }
}

#[test]
fn the_desk_ledger_and_settings_use_every_family_of_component() {
    let (mut app, root) = guild();
    assert!(app.world().get::<GuildRoot>(root).is_some());
    for (family, found) in [
        ("app bar", find::<AppBar>(&app, root)),
        ("navigation rail", find::<RailFab>(&app, root)),
        ("avatar", find::<Avatar>(&app, root)),
        ("badge", find::<Badge>(&app, root)),
        ("skeleton", find::<Skeleton>(&app, root)),
        ("circular progress", find::<CircularProgress>(&app, root)),
        ("linear progress", find::<LinearProgress>(&app, root)),
        ("text field", find::<TextField>(&app, root)),
        ("segmented buttons", find::<Segment>(&app, root)),
        ("select", find::<SelectField>(&app, root)),
        ("radio buttons", find::<RadioOption>(&app, root)),
        ("slider", find::<SliderTrack>(&app, root)),
        ("switch", find::<SwitchHandle>(&app, root)),
        ("checkbox", find::<CheckboxBox>(&app, root)),
        ("search bar", find::<SearchBar>(&app, root)),
        ("list", find::<ListRow>(&app, root)),
        ("scrollbar", find::<MaterialScrollbar>(&app, root)),
        ("tooltip", find::<Tooltip>(&app, root)),
        ("card", find::<Raised>(&app, root)),
    ] {
        assert!(found.is_some(), "the desk has no {family}");
    }
    for (kind, name) in [
        (ChipKind::Assist, "assist"),
        (ChipKind::Filter, "filter"),
        (ChipKind::Suggestion, "suggestion"),
    ] {
        assert!(
            find_all::<ChipKind>(&app, root)
                .into_iter()
                .any(|chip| app.world().get::<ChipKind>(chip) == Some(&kind)),
            "the desk has no {name} chip"
        );
    }
    go(&mut app, root, 1);
    assert!(
        find::<MaterialTab>(&app, root).is_some(),
        "the ledger has tabs"
    );
    go(&mut app, root, 2);
    assert!(find::<SwitchHandle>(&app, root).is_some());
    assert!(find::<SliderTrack>(&app, root).is_some());
}
