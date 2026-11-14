//! The Guild Clerk's game: its clients, its roster, the clerk's draft
//! contract, the ledger, and the screens drawn from them.
//!
//! The game keeps every value - the draft, the guild's books, the settings -
//! in its own resources; the controls only report what the clerk did, as
//! the crate's components do. A screen is rebuilt from those resources when
//! the clerk goes somewhere else or a request is dealt with, and what
//! changes in between (the roster's filter, a hint, a quest coming home) is
//! shown by systems without rebuilding.

use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::ui::Checked;
use bevy::ui_widgets::ScrollArea;
use bevy_material_ui::prelude::*;

/// Everything the game needs; spawn its screen with [`spawn_guild`].
pub(crate) struct GuildPlugin;

impl Plugin for GuildPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(guild_theme(0, Brightness::Light))
            .init_resource::<Guild>()
            .init_resource::<Draft>()
            .init_resource::<Settings>()
            .init_resource::<RosterFilter>()
            .init_resource::<Screen>()
            // The first client is brought in as every other is.
            .insert_resource(Courier(Settings::default().courier))
            .init_resource::<Undoable>()
            .add_systems(
                Update,
                (rebuild, arrive, show_roster, show_hint, track_quests).chain(),
            )
            .add_observer(on_text)
            .add_observer(on_toggle)
            .add_observer(on_choice)
            .add_observer(on_slide)
            .add_observer(on_pick)
            .add_observer(on_activate)
            .add_observer(file_when_confirmed)
            .add_observer(day_over_answered)
            .add_observer(on_member_removed)
            .add_observer(on_undo)
            .add_observer(on_rail_fab)
            .add_observer(on_app_bar)
            .add_observer(on_tooltip_action);
    }
}

/// Spawns the game's screen, filling the window; it is drawn on the next
/// frame from the game's resources.
pub(crate) fn spawn_guild(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            GuildRoot,
            TabGroup::new(0),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Fill::new(ColorRole::Surface),
        ))
        .id()
}

// The world of the game.

/// The kinds of quest the guild takes.
pub(crate) const KINDS: [&str; 3] = ["Hunt", "Escort", "Delivery"];
/// Where its quests go.
pub(crate) const REGIONS: [&str; 4] = ["Mossy Vale", "Ironpeak", "Saltmarsh", "Old Road"];
/// How dangerous a quest is, least first.
pub(crate) const DANGERS: [&str; 4] = ["Low", "Moderate", "High", "Deadly"];
/// From this danger on, a contract needs danger pay agreed.
pub(crate) const DANGER_PAY_FROM: usize = 2;
/// The colours the guild can be painted in: a name and a seed.
pub(crate) const COLOURS: [(&str, u32); 4] = [
    ("Guild brown", 0x7A5230),
    ("Forest green", 0x2E7D32),
    ("Royal blue", 0x3F51B5),
    ("Ember red", 0xC62828),
];

/// What an adventurer does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Class {
    /// Holds the line.
    Fighter,
    /// Scouts and shoots.
    Ranger,
    /// Casts.
    Mage,
    /// Mends.
    Healer,
}

impl Class {
    /// Every class, in the order the roster's chips show them.
    pub(crate) const ALL: [Class; 4] = [Class::Fighter, Class::Ranger, Class::Mage, Class::Healer];

    /// Its name.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Class::Fighter => "Fighter",
            Class::Ranger => "Ranger",
            Class::Mage => "Mage",
            Class::Healer => "Healer",
        }
    }

    /// The mistake a contract makes when a party needing one has none.
    fn missing(self) -> &'static str {
        match self {
            Class::Fighter => "a fighter in the party",
            Class::Ranger => "a ranger in the party",
            Class::Mage => "a mage in the party",
            Class::Healer => "a healer in the party",
        }
    }
}

/// An adventurer of the guild.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Adventurer {
    /// Their name.
    pub(crate) name: &'static str,
    /// What they do.
    pub(crate) class: Class,
    /// Their rank.
    pub(crate) rank: &'static str,
}

const fn adventurer(name: &'static str, class: Class, rank: &'static str) -> Adventurer {
    Adventurer { name, class, rank }
}

/// The guild's adventurers.
pub(crate) const ROSTER: [Adventurer; 10] = [
    adventurer("Aldric Stone", Class::Fighter, "Gold"),
    adventurer("Bryn Ashby", Class::Ranger, "Silver"),
    adventurer("Cass Merrow", Class::Mage, "Silver"),
    adventurer("Dara Quill", Class::Healer, "Gold"),
    adventurer("Edda Thorn", Class::Fighter, "Bronze"),
    adventurer("Finn Vale", Class::Ranger, "Bronze"),
    adventurer("Gale Orrin", Class::Mage, "Gold"),
    adventurer("Hale Brook", Class::Healer, "Bronze"),
    adventurer("Ivo Marsh", Class::Fighter, "Silver"),
    adventurer("Juno Fair", Class::Healer, "Silver"),
];

/// What a client asks for, and the contract that would answer it rightly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Request {
    /// Who asks.
    pub(crate) client: &'static str,
    /// What they say at the desk: everything the contract needs is in it.
    pub(crate) words: &'static str,
    /// The kind of quest, an index of [`KINDS`].
    pub(crate) kind: usize,
    /// Where, an index of [`REGIONS`].
    pub(crate) region: usize,
    /// How dangerous, an index of [`DANGERS`].
    pub(crate) danger: usize,
    /// How many go.
    pub(crate) party: usize,
    /// A class the party must have.
    pub(crate) needs: Option<Class>,
    /// Whether it is a rush job.
    pub(crate) rush: bool,
    /// The most the client will pay.
    pub(crate) budget: u32,
    /// The guild must not take it: the clerk returns it.
    pub(crate) refuse: bool,
}

const fn request(
    client: &'static str,
    words: &'static str,
    (kind, region, danger, party): (usize, usize, usize, usize),
    needs: Option<Class>,
    rush: bool,
    budget: u32,
) -> Request {
    Request {
        client,
        words,
        kind,
        region,
        danger,
        party,
        needs,
        rush,
        budget,
        refuse: false,
    }
}

const fn refused(client: &'static str, words: &'static str) -> Request {
    Request {
        client,
        words,
        kind: 0,
        region: 0,
        danger: 0,
        party: 0,
        needs: None,
        rush: false,
        budget: 0,
        refuse: true,
    }
}

/// Every client, in the order they come to the desk; the days go round them.
pub(crate) const REQUESTS: [Request; 10] = [
    request(
        "Farmer Bram",
        "Wolves are taking my sheep in the Mossy Vale. Hungry, not fierce - low danger. \
         Two hunters should do it, and there's no rush. I can pay 60 gold.",
        (0, 0, 0, 2),
        None,
        false,
        60,
    ),
    request(
        "Lady Corvina",
        "Escort my carriage to the Saltmarsh before tomorrow's tide - a rush. Bandits are \
         bold there: high danger. Three of you, one a healer. 300 gold.",
        (1, 2, 2, 3),
        Some(Class::Healer),
        true,
        300,
    ),
    request(
        "Postmaster Odo",
        "A sealed letter for the smith at Ironpeak. The road is safe - low danger. One rider, \
         no hurry. 25 gold.",
        (2, 1, 0, 1),
        None,
        false,
        25,
    ),
    refused(
        "A hooded stranger",
        "Take this box to the Old Road crossing at midnight. Don't open it, and don't write \
         anything down. 500 gold - after.",
    ),
    request(
        "Brother Ansel",
        "Something stirs in the crypts under Ironpeak. Moderate danger, says the abbot. Hunt \
         it with four, and bring a mage for the wards. 180 gold, in good time.",
        (0, 1, 1, 4),
        Some(Class::Mage),
        false,
        180,
    ),
    request(
        "Merchant Tamsin",
        "Escort my wagons along the Old Road to market - moderate danger. Two guards, one a \
         ranger to scout. 120 gold, and quickly: market opens in two days.",
        (1, 3, 1, 2),
        Some(Class::Ranger),
        true,
        120,
    ),
    request(
        "Hollin the witch",
        "The drake of the Saltmarsh has woken. Deadly, I won't lie. Hunt it with five of your \
         best, a healer among them. 900 gold; it sleeps a while yet.",
        (0, 2, 3, 5),
        Some(Class::Healer),
        false,
        900,
    ),
    request(
        "Miller Gwen",
        "Flour for the orphanage in the Mossy Vale - a delivery, safe roads, low danger. Two \
         carriers. 40 gold. They're hungry now, so hurry.",
        (2, 0, 0, 2),
        None,
        true,
        40,
    ),
    refused(
        "Sergeant Rook",
        "Fifty gold to give the new captain a beating. Nobody need know it was the guild.",
    ),
    request(
        "Scholar Iva",
        "Escort me to the ruins of Ironpeak's high pass. High danger; the cold kills. Three \
         companions, one a mage who knows fire. 260 gold, no hurry.",
        (1, 1, 2, 3),
        Some(Class::Mage),
        false,
        260,
    ),
];

/// How long a quest is out, by its danger: harder ones take longer.
pub(crate) fn quest_seconds(danger: usize) -> f32 {
    20.0 + 10.0 * danger as f32
}

// The game's state.

/// The contract the clerk is writing, as the form's controls left it.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Draft {
    /// The quest's title.
    pub(crate) title: String,
    /// The reward, as typed.
    pub(crate) reward: String,
    /// The kind of quest chosen, an index of [`KINDS`].
    pub(crate) kind: Option<usize>,
    /// The region chosen, an index of [`REGIONS`].
    pub(crate) region: Option<usize>,
    /// The danger chosen, an index of [`DANGERS`].
    pub(crate) danger: Option<usize>,
    /// The party's size, less one: the notch the slider is at.
    pub(crate) size: usize,
    /// A rush job.
    pub(crate) rush: bool,
    /// The client accepts the guild's terms.
    pub(crate) terms: bool,
    /// Danger pay is agreed.
    pub(crate) danger_pay: bool,
    /// The party, as indices of [`ROSTER`].
    pub(crate) party: Vec<usize>,
}

/// What became of a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Filed, with the mistakes its contract made.
    Filed(Vec<&'static str>),
    /// Returned to the client.
    Returned,
}

/// A request the clerk dealt with.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Handled {
    /// The day it came.
    pub(crate) day: u32,
    /// Which request, an index of [`REQUESTS`].
    pub(crate) request: usize,
    /// What became of it.
    pub(crate) verdict: Verdict,
    /// The contract as filed or returned, so it can be taken back.
    pub(crate) draft: Draft,
    /// What it did to the guild's reputation.
    pub(crate) reputation: i32,
    /// What it earned the guild.
    pub(crate) gold: i32,
    /// When its party comes home, in seconds of the game's time; none for a
    /// request returned.
    pub(crate) back_at: Option<f32>,
}

/// The guild's books: the day, today's clients, and everything dealt with.
#[derive(Resource, Clone, Debug, PartialEq)]
pub(crate) struct Guild {
    /// The day, from one.
    pub(crate) day: u32,
    /// Today's clients, as indices of [`REQUESTS`].
    pub(crate) today: Vec<usize>,
    /// How many of today's have been dealt with.
    pub(crate) at: usize,
    /// The next client to come, an index of [`REQUESTS`].
    pub(crate) next: usize,
    /// The guild's reputation.
    pub(crate) reputation: i32,
    /// The guild's gold.
    pub(crate) gold: i32,
    /// Everything dealt with, oldest first.
    pub(crate) handled: Vec<Handled>,
}

impl Default for Guild {
    fn default() -> Self {
        let mut guild = Guild {
            day: 0,
            today: Vec::new(),
            at: 0,
            next: 0,
            reputation: 0,
            gold: 0,
            handled: Vec::new(),
        };
        guild.open(Settings::default().per_day);
        guild
    }
}

impl Guild {
    /// Opens the next day, with `clients` coming.
    pub(crate) fn open(&mut self, clients: usize) {
        self.day += 1;
        self.today = (0..clients)
            .map(|offset| (self.next + offset) % REQUESTS.len())
            .collect();
        self.next = (self.next + clients) % REQUESTS.len();
        self.at = 0;
    }

    /// The client at the desk now, if the day has one left.
    pub(crate) fn current(&self) -> Option<usize> {
        self.today.get(self.at).copied()
    }

    /// Whether every one of today's clients has been dealt with.
    pub(crate) fn closed(&self) -> bool {
        self.at >= self.today.len()
    }

    /// Whether adventurer `index` is out on a quest at `now`.
    pub(crate) fn away(&self, index: usize, now: f32) -> bool {
        self.handled.iter().any(|handled| {
            handled.back_at.is_some_and(|back| now < back) && handled.draft.party.contains(&index)
        })
    }

    /// Deals with the client at the desk: `verdict` on the contract `draft`,
    /// at `now`. Returns what it did to the reputation.
    pub(crate) fn deal(&mut self, verdict: Verdict, draft: Draft, now: f32) -> i32 {
        let index = self.current().expect("a client is at the desk");
        let request = REQUESTS[index];
        let (reputation, gold) = score(&request, &verdict);
        let back_at = match verdict {
            Verdict::Filed(_) => Some(now + quest_seconds(draft.danger.unwrap_or(0))),
            Verdict::Returned => None,
        };
        self.handled.push(Handled {
            day: self.day,
            request: index,
            verdict,
            draft,
            reputation,
            gold,
            back_at,
        });
        self.reputation += reputation;
        self.gold += gold;
        self.at += 1;
        reputation
    }

    /// Takes back the last request dealt with today, returning its contract.
    pub(crate) fn take_back(&mut self) -> Option<Draft> {
        let last = self.handled.last()?;
        if last.day != self.day || self.at == 0 {
            return None;
        }
        let last = self.handled.pop().expect("there is a last");
        self.reputation -= last.reputation;
        self.gold -= last.gold;
        self.at -= 1;
        Some(last.draft)
    }
}

/// The mistakes `draft` makes as the contract for `request`, none when it
/// is right.
pub(crate) fn judge(request: &Request, draft: &Draft) -> Vec<&'static str> {
    if request.refuse {
        return vec!["a contract the guild must not take"];
    }
    let mut mistakes = Vec::new();
    if draft.kind != Some(request.kind) {
        mistakes.push("the kind of quest");
    }
    if draft.region != Some(request.region) {
        mistakes.push("the region");
    }
    if draft.danger != Some(request.danger) {
        mistakes.push("the danger");
    }
    if draft.size + 1 != request.party {
        mistakes.push("the party's size");
    }
    if draft.party.len() != request.party {
        mistakes.push("how many go");
    }
    if let Some(class) = request.needs
        && !draft
            .party
            .iter()
            .any(|member| ROSTER[*member].class == class)
    {
        mistakes.push(class.missing());
    }
    if draft.rush != request.rush {
        mistakes.push("the rush");
    }
    if !draft
        .reward
        .trim()
        .parse::<u32>()
        .is_ok_and(|reward| reward <= request.budget)
    {
        mistakes.push("the reward");
    }
    if !draft.terms {
        mistakes.push("the client's agreement");
    }
    if draft.danger_pay != (request.danger >= DANGER_PAY_FROM) {
        mistakes.push("danger pay");
    }
    mistakes
}

/// What `verdict` on `request` earns: reputation, and gold.
pub(crate) fn score(request: &Request, verdict: &Verdict) -> (i32, i32) {
    match (request.refuse, verdict) {
        (true, Verdict::Returned) => (2, 0),
        (true, Verdict::Filed(_)) => (-3, 0),
        (false, Verdict::Returned) => (-2, 0),
        (false, Verdict::Filed(mistakes)) if mistakes.is_empty() => {
            (3, i32::try_from(request.budget / 10).expect("a fee fits"))
        }
        (false, Verdict::Filed(mistakes)) => (
            -i32::try_from(mistakes.len().min(3)).expect("at most three"),
            0,
        ),
    }
}

/// How the clerk likes the game.
#[derive(Resource, Clone, Debug, PartialEq)]
pub(crate) struct Settings {
    /// Seconds the courier takes to bring each client in.
    pub(crate) courier: f32,
    /// Clients a day, from the next day on.
    pub(crate) per_day: usize,
    /// Suggest fixes to a party.
    pub(crate) suggestions: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            courier: 1.0,
            per_day: 4,
            suggestions: true,
        }
    }
}

/// What the roster shows: the search typed and the classes chosen.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RosterFilter {
    /// The search, as typed.
    pub(crate) query: String,
    /// The classes chosen; none chosen shows every class.
    pub(crate) classes: Vec<Class>,
}

impl RosterFilter {
    /// Whether `adventurer` is shown.
    pub(crate) fn shows(&self, adventurer: &Adventurer) -> bool {
        let query = self.query.trim().to_lowercase();
        (query.is_empty() || adventurer.name.to_lowercase().contains(&query))
            && (self.classes.is_empty() || self.classes.contains(&adventurer.class))
    }
}

/// The places of the game, in the rail's order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Place {
    /// The desk: the client, the contract, the party.
    #[default]
    Desk,
    /// The books.
    Ledger,
    /// How the game looks and plays.
    Settings,
}

impl Place {
    const ALL: [Place; 3] = [Place::Desk, Place::Ledger, Place::Settings];
}

/// Where the clerk is; the screen is rebuilt whenever this changes.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Screen {
    /// The place shown.
    pub(crate) place: Place,
}

/// Seconds until the courier brings the client in.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Courier(pub(crate) f32);

/// The snackbar whose Undo takes the last contract back.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Undoable(pub(crate) Option<SnackbarId>);

/// The guild's theme: colour `index` of [`COLOURS`], light or dark.
pub(crate) fn guild_theme(index: usize, brightness: Brightness) -> MaterialTheme {
    MaterialTheme::from_seed(hex(COLOURS[index].1), brightness)
}

// What the screen's parts are.

/// The screen's root.
#[derive(Component, Debug)]
pub(crate) struct GuildRoot;

/// What a control sets: found at or above the part a change came from.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Field {
    /// The quest's title.
    Title,
    /// The reward.
    Reward,
    /// The kind of quest.
    Kind,
    /// The region.
    Region,
    /// The danger.
    Danger,
    /// The party's size.
    Size,
    /// A rush job.
    Rush,
    /// The guild's terms.
    Terms,
    /// Danger pay.
    DangerPay,
    /// The roster's search.
    Search,
    /// The roster's chip for a class.
    Class(Class),
    /// The roster.
    Roster,
    /// The places.
    Places,
    /// Light or dark.
    Dark,
    /// The guild's colours.
    Colours,
    /// The courier's speed.
    Courier,
    /// Clients a day.
    PerDay,
    /// Suggesting fixes.
    Suggestions,
}

/// What a button or chip does when used.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// Asks to file the contract.
    Approve,
    /// Returns the request to the client.
    Return,
    /// Empties the form.
    Clear,
    /// Opens the handbook.
    Handbook,
    /// Shows only the healers.
    SuggestHealer,
    /// Opens the next day.
    OpenTomorrow,
}

/// A roster row: the adventurer it is, an index of [`ROSTER`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Recruit(pub(crate) usize);

/// A chip of the party: the adventurer it is, an index of [`ROSTER`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Member(pub(crate) usize);

/// The set holding the party's chips.
#[derive(Component, Debug)]
pub(crate) struct PartyChips;

/// The suggestion to add a healer, shown while a dangerous party lacks one.
#[derive(Component, Debug)]
pub(crate) struct HealerHint;

/// Shown while the courier brings the client in.
#[derive(Component, Debug)]
pub(crate) struct Arriving;

/// Shown once the client is in.
#[derive(Component, Debug)]
pub(crate) struct Arrived;

/// A quest's progress bar, for the [`Handled`] at this index.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct QuestBar(pub(crate) usize);

/// A quest's status line, for the [`Handled`] at this index.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct QuestStatus(pub(crate) usize);

/// The dialog asking whether to file the contract.
#[derive(Component, Debug)]
pub(crate) struct FileDialog;

/// The dialog closing the day.
#[derive(Component, Debug)]
pub(crate) struct DayOverDialog;

// What the controls report.

/// The [`Field`] at or above `entity`, and the part carrying it.
fn field_of(
    entity: Entity,
    fields: &Query<&Field>,
    parents: &Query<&ChildOf>,
) -> Option<(Entity, Field)> {
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find_map(|at| fields.get(at).ok().map(|field| (at, *field)))
}

fn on_text(
    change: On<ValueChange<String>>,
    fields: Query<&Field>,
    parents: Query<&ChildOf>,
    mut text_fields: Query<&mut TextField>,
    mut draft: ResMut<Draft>,
    mut filter: ResMut<RosterFilter>,
) {
    let Some((part, field)) = field_of(change.source, &fields, &parents) else {
        return;
    };
    match field {
        Field::Title => draft.title.clone_from(&change.value),
        Field::Reward => draft.reward.clone_from(&change.value),
        Field::Search => filter.query.clone_from(&change.value),
        _ => return,
    }
    // Typing into a field in error is fixing it.
    if let Ok(mut text_field) = text_fields.get_mut(part)
        && text_field.error.is_some()
    {
        text_field.error = None;
    }
}

fn on_toggle(
    change: On<ValueChange<bool>>,
    fields: Query<&Field>,
    parents: Query<&ChildOf>,
    mut draft: ResMut<Draft>,
    mut filter: ResMut<RosterFilter>,
    mut settings: ResMut<Settings>,
    mut theme: ResMut<MaterialTheme>,
) {
    let Some((_, field)) = field_of(change.source, &fields, &parents) else {
        return;
    };
    let on = change.value;
    match field {
        Field::Rush => draft.rush = on,
        Field::Terms => draft.terms = on,
        Field::DangerPay => draft.danger_pay = on,
        Field::Class(class) => {
            filter.classes.retain(|chosen| *chosen != class);
            if on {
                filter.classes.push(class);
            }
        }
        Field::Dark => {
            *theme = theme.at(if on {
                Brightness::Dark
            } else {
                Brightness::Light
            });
        }
        Field::Suggestions => settings.suggestions = on,
        _ => {}
    }
}

fn on_choice(
    change: On<ValueChange<usize>>,
    fields: Query<&Field>,
    parents: Query<&ChildOf>,
    mut draft: ResMut<Draft>,
    mut screen: ResMut<Screen>,
    mut theme: ResMut<MaterialTheme>,
) {
    let Some((_, field)) = field_of(change.source, &fields, &parents) else {
        return;
    };
    match field {
        Field::Kind => draft.kind = Some(change.value),
        Field::Region => draft.region = Some(change.value),
        Field::Places => {
            if let Some(place) = Place::ALL.get(change.value) {
                screen.place = *place;
            }
        }
        Field::Colours => {
            *theme = guild_theme(change.value, theme.brightness()).with_fonts(theme.fonts.clone());
        }
        _ => {}
    }
}

fn on_slide(
    change: On<ValueChange<f32>>,
    fields: Query<&Field>,
    parents: Query<&ChildOf>,
    mut draft: ResMut<Draft>,
    mut settings: ResMut<Settings>,
) {
    let Some((_, field)) = field_of(change.source, &fields, &parents) else {
        return;
    };
    match field {
        Field::Size => draft.size = notch(change.value),
        Field::Courier => settings.courier = change.value,
        Field::PerDay => settings.per_day = notch(change.value),
        _ => {}
    }
}

/// A slider's value as the whole number it stands for.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a slider's values here are small, rounded, and clamped to zero first"
)]
fn notch(value: f32) -> usize {
    value.max(0.0).round() as usize
}

fn on_pick(
    change: On<ValueChange<Entity>>,
    fields: Query<&Field>,
    parents: Query<&ChildOf>,
    picked: Query<(Option<&RadioOption>, Option<&Recruit>)>,
    chips: Query<Entity, With<PartyChips>>,
    mut draft: ResMut<Draft>,
    mut commands: Commands,
) {
    let Some((_, field)) = field_of(change.source, &fields, &parents) else {
        return;
    };
    let Ok((option, recruit)) = picked.get(change.value) else {
        return;
    };
    match field {
        Field::Danger => {
            if let Some(option) = option {
                draft.danger = Some(option.0);
            }
        }
        Field::Roster => {
            let Some(recruit) = recruit else {
                return;
            };
            if draft.party.contains(&recruit.0) {
                return;
            }
            draft.party.push(recruit.0);
            if let Ok(set) = chips.single() {
                commands.entity(set).with_child(member_chip(recruit.0));
            }
        }
        _ => {}
    }
}

fn on_member_removed(
    removed: On<ChipRemoved>,
    members: Query<&Member>,
    mut draft: ResMut<Draft>,
    mut commands: Commands,
) {
    let Ok(member) = members.get(removed.entity) else {
        return;
    };
    draft.party.retain(|index| *index != member.0);
    commands.entity(removed.entity).despawn();
}

/// The [`Action`] at or above `entity`.
fn action_of(
    entity: Entity,
    actions: &Query<&Action>,
    parents: &Query<&ChildOf>,
) -> Option<Action> {
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find_map(|at| actions.get(at).ok().copied())
}

/// The guild's books and the desk, which most of what the clerk does
/// changes together.
#[derive(SystemParam)]
struct Books<'w> {
    guild: ResMut<'w, Guild>,
    draft: ResMut<'w, Draft>,
    filter: ResMut<'w, RosterFilter>,
    screen: ResMut<'w, Screen>,
    courier: ResMut<'w, Courier>,
    settings: Res<'w, Settings>,
}

impl Books<'_> {
    /// Clears the desk for the next client, whom the courier brings in.
    fn next_client(&mut self) {
        *self.draft = Draft::default();
        self.courier.0 = self.settings.courier;
        self.screen.place = Place::Desk;
    }

    /// Opens the next day, once today's clients are all dealt with; says
    /// whether it did.
    fn open_tomorrow(&mut self) -> bool {
        if !self.guild.closed() {
            return false;
        }
        let clients = self.settings.per_day;
        self.guild.open(clients);
        self.next_client();
        true
    }
}

fn on_activate(
    activate: On<Activate>,
    actions: Query<&Action>,
    parents: Query<&ChildOf>,
    mut controls: Query<(Entity, &Field, Option<&mut TextField>)>,
    mut books: Books,
    mut snackbars: ResMut<Snackbars>,
    mut commands: Commands,
) {
    let Some(action) = action_of(activate.entity, &actions, &parents) else {
        return;
    };
    match action {
        Action::Approve => {
            let mut ready = true;
            for (_, field, text_field) in &mut controls {
                let error = match field {
                    Field::Title if books.draft.title.trim().is_empty() => {
                        Some("A contract needs a title")
                    }
                    Field::Reward if books.draft.reward.trim().parse::<u32>().is_err() => {
                        Some("Gold, in whole coins")
                    }
                    _ => None,
                };
                if let (Some(error), Some(mut text_field)) = (error, text_field) {
                    text_field.error = Some(error.to_owned());
                    ready = false;
                }
            }
            if ready {
                commands.spawn((
                    FileDialog,
                    dialog("File this contract?")
                        .text(summary(&books.draft))
                        .action(CANCEL, "Not yet")
                        .action(CONFIRM, "File it")
                        .bundle(),
                ));
            }
        }
        Action::Return => {
            let Some(index) = books.guild.current() else {
                return;
            };
            let draft = books.draft.clone();
            let reputation = books.guild.deal(Verdict::Returned, draft, 0.0);
            snackbars.show(format!(
                "Returned to {} ({reputation:+} reputation)",
                REQUESTS[index].client
            ));
            books.next_client();
            if books.guild.closed() {
                commands.spawn(day_over(&books.guild));
            }
        }
        Action::Clear => {
            *books.draft = Draft::default();
            books.screen.set_changed();
        }
        Action::Handbook => {
            commands.spawn(handbook());
        }
        Action::SuggestHealer => {
            books.filter.classes = vec![Class::Healer];
            for (chip, field, _) in &controls {
                match field {
                    Field::Class(Class::Healer) => {
                        commands.entity(chip).insert(Checked);
                    }
                    Field::Class(_) => {
                        commands.entity(chip).remove::<Checked>();
                    }
                    _ => {}
                }
            }
        }
        Action::OpenTomorrow => {
            books.open_tomorrow();
        }
    }
}

/// Files the contract when its dialog is answered "File it".
fn file_when_confirmed(
    answered: On<DialogResult>,
    dialogs: Query<(), With<FileDialog>>,
    time: Res<Time>,
    mut books: Books,
    mut snackbars: ResMut<Snackbars>,
    mut undoable: ResMut<Undoable>,
    mut commands: Commands,
) {
    if !dialogs.contains(answered.dialog) || !answered.is_confirmed() {
        return;
    }
    let Some(index) = books.guild.current() else {
        return;
    };
    let request = REQUESTS[index];
    let draft = books.draft.clone();
    let mistakes = judge(&request, &draft);
    let words = if mistakes.is_empty() {
        format!("Filed for {}: well done", request.client)
    } else {
        format!(
            "Filed for {}: wrong on {}",
            request.client,
            mistakes.join(", ")
        )
    };
    let reputation = books
        .guild
        .deal(Verdict::Filed(mistakes), draft, time.elapsed_secs());
    undoable.0 = Some(
        snackbars.show_snackbar(
            Snackbar::new(format!("{words} ({reputation:+} reputation)"))
                .action("Undo")
                .seconds(SNACKBAR_ACTION_SECONDS),
        ),
    );
    books.next_client();
    if books.guild.closed() {
        commands.spawn(day_over(&books.guild));
    }
}

fn on_undo(
    action: On<SnackbarAction>,
    mut undoable: ResMut<Undoable>,
    mut books: Books,
    day_over: Query<Entity, With<DayOverDialog>>,
    mut commands: Commands,
) {
    if undoable.0 != Some(action.id) {
        return;
    }
    undoable.0 = None;
    if let Some(taken) = books.guild.take_back() {
        *books.draft = taken;
        books.courier.0 = 0.0;
        books.screen.place = Place::Desk;
        // The day isn't over after all.
        for dialog in &day_over {
            commands.entity(dialog).despawn();
        }
    }
}

fn on_rail_fab(_: On<RailFabActivated>, mut books: Books, mut snackbars: ResMut<Snackbars>) {
    if !books.open_tomorrow() {
        snackbars.show("Clients are still waiting");
    }
}

fn on_app_bar(_: On<AppBarAction>, mut commands: Commands) {
    commands.spawn(handbook());
}

fn on_tooltip_action(_: On<TooltipAction>, mut commands: Commands) {
    commands.spawn(handbook());
}

/// Answers the day's last dialog: tomorrow, or a look at the ledger.
fn day_over_answered(
    answered: On<DialogResult>,
    dialogs: Query<(), With<DayOverDialog>>,
    mut books: Books,
) {
    if !dialogs.contains(answered.dialog) {
        return;
    }
    match &answered.answer {
        DialogAnswer::Chose(key) if key == TOMORROW => {
            books.open_tomorrow();
        }
        DialogAnswer::Chose(key) if key == LEDGER => books.screen.place = Place::Ledger,
        _ => {}
    }
}

/// The day's last dialog's key for opening the next day.
pub(crate) const TOMORROW: &str = "tomorrow";
/// The day's last dialog's key for going to the ledger.
pub(crate) const LEDGER: &str = "ledger";

/// The contract `draft` in a sentence, for the dialog filing it.
pub(crate) fn summary(draft: &Draft) -> String {
    fn named(names: &[&'static str], chosen: Option<usize>) -> &'static str {
        chosen
            .and_then(|index| names.get(index).copied())
            .unwrap_or("unchosen")
    }
    let party: Vec<&str> = draft
        .party
        .iter()
        .map(|member| ROSTER[*member].name)
        .collect();
    format!(
        "{} - {} in {}, {} danger, a party of {}{}. Reward {} gold{}.",
        draft.title.trim(),
        named(&KINDS, draft.kind),
        named(&REGIONS, draft.region),
        named(&DANGERS, draft.danger),
        draft.size + 1,
        if party.is_empty() {
            String::new()
        } else {
            format!(": {}", party.join(", "))
        },
        draft.reward.trim(),
        if draft.rush { ", rushed" } else { "" },
    )
}

// What changes on screen between rebuilds.

fn arrive(
    time: Res<Time>,
    mut courier: ResMut<Courier>,
    mut arriving: Query<&mut Node, (With<Arriving>, Without<Arrived>)>,
    mut arrived: Query<&mut Node, (With<Arrived>, Without<Arriving>)>,
) {
    if courier.0 > 0.0 {
        courier.0 = (courier.0 - time.delta_secs()).max(0.0);
    }
    let here = courier.0 <= 0.0;
    for mut node in &mut arriving {
        show(&mut node, !here);
    }
    for mut node in &mut arrived {
        show(&mut node, here);
    }
}

/// Shows or hides a part, changing it only when it differs.
fn show(node: &mut Mut<Node>, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}

fn show_roster(
    time: Res<Time>,
    guild: Res<Guild>,
    draft: Res<Draft>,
    filter: Res<RosterFilter>,
    mut rows: Query<(&Recruit, &mut Node)>,
) {
    let now = time.elapsed_secs();
    for (recruit, mut node) in &mut rows {
        let shown = filter.shows(&ROSTER[recruit.0])
            && !draft.party.contains(&recruit.0)
            && !guild.away(recruit.0, now);
        show(&mut node, shown);
    }
}

fn show_hint(
    settings: Res<Settings>,
    draft: Res<Draft>,
    mut hints: Query<&mut Node, With<HealerHint>>,
) {
    let lacking = draft.danger.is_some_and(|danger| danger >= DANGER_PAY_FROM)
        && !draft
            .party
            .iter()
            .any(|member| ROSTER[*member].class == Class::Healer);
    for mut node in &mut hints {
        show(&mut node, settings.suggestions && lacking);
    }
}

fn track_quests(
    time: Res<Time>,
    guild: Res<Guild>,
    mut bars: Query<(&QuestBar, &mut Progress)>,
    mut statuses: Query<(&QuestStatus, &mut Text)>,
) {
    let now = time.elapsed_secs();
    for (bar, mut progress) in &mut bars {
        if let Some(share) = guild
            .handled
            .get(bar.0)
            .and_then(|handled| quest_share(handled, now))
        {
            progress.0 = Some(share);
        }
    }
    for (status, mut text) in &mut statuses {
        if let Some(handled) = guild.handled.get(status.0) {
            let words = quest_status(handled, now);
            if text.0 != words {
                text.0 = words;
            }
        }
    }
}

/// How far a quest has gone at `now`, from nought to one; none for a request
/// returned.
pub(crate) fn quest_share(handled: &Handled, now: f32) -> Option<f32> {
    let back = handled.back_at?;
    let length = quest_seconds(handled.draft.danger.unwrap_or(0));
    Some((1.0 - (back - now) / length).clamp(0.0, 1.0))
}

/// What a quest's status line says at `now`.
fn quest_status(handled: &Handled, now: f32) -> String {
    match (&handled.verdict, handled.back_at) {
        (Verdict::Filed(_), Some(back)) if now < back => {
            format!("Out - back in {:.0} s", (back - now).ceil())
        }
        (Verdict::Filed(mistakes), _) if mistakes.is_empty() => "Home, the job done".to_owned(),
        (Verdict::Filed(_), _) => "Home, the client unhappy".to_owned(),
        (Verdict::Returned, _) => "Returned".to_owned(),
    }
}

// Building the screen.

/// Everything a screen is drawn from.
#[derive(SystemParam)]
struct View<'w> {
    screen: Res<'w, Screen>,
    guild: Res<'w, Guild>,
    draft: Res<'w, Draft>,
    filter: Res<'w, RosterFilter>,
    settings: Res<'w, Settings>,
    theme: Res<'w, MaterialTheme>,
}

fn rebuild(view: View, roots: Query<Entity, With<GuildRoot>>, mut commands: Commands) {
    let View {
        screen,
        guild,
        draft,
        filter,
        settings,
        theme,
    } = view;
    if !screen.is_changed() {
        return;
    }
    for root in &roots {
        commands
            .entity(root)
            .despawn_children()
            .with_children(|root| {
                // The rail runs down the whole left side; beside it, the
                // app bar over a page that scrolls.
                root.spawn((
                    Field::Places,
                    navigation_rail(
                        vec![
                            destination(Glyph::Dot, "Desk"),
                            destination(Glyph::Check, "Ledger"),
                            destination(Glyph::Menu, "Settings"),
                        ],
                        Place::ALL
                            .iter()
                            .position(|place| *place == screen.place)
                            .expect("every place is in the rail"),
                        Some(Glyph::ArrowForward.into()),
                    ),
                ));
                root.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    flex_basis: Val::ZERO,
                    min_width: Val::ZERO,
                    height: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|main| {
                    top(main, &guild);
                    main.spawn((
                        ScrollArea,
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            flex_basis: Val::ZERO,
                            min_height: Val::ZERO,
                            overflow: Overflow::scroll_y(),
                            padding: UiRect::all(Space::ExtraLarge.val()),
                            row_gap: Space::Large.val(),
                            ..default()
                        },
                    ))
                    .with_children(|page| match screen.place {
                        Place::Desk => desk(page, &guild, &draft, &filter),
                        Place::Ledger => ledger(page, &guild),
                        Place::Settings => settings_page(page, &settings, &theme),
                    });
                });
            });
    }
}

/// The app bar, and how the day is going under it.
fn top(root: &mut ChildSpawnerCommands, guild: &Guild) {
    root.spawn(
        top_app_bar(format!("The Guild Clerk - day {}", guild.day))
            .action(Glyph::MoreVert)
            .bundle(),
    );
    root.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Space::Small.val(),
        padding: UiRect::axes(Space::ExtraLarge.val(), Space::Small.val()),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|status| {
        status.spawn(text(
            format!(
                "Clients dealt with: {} of {}   Reputation: {}   Gold: {}",
                guild.at,
                guild.today.len(),
                guild.reputation,
                guild.gold
            ),
            TypeRole::LabelLarge,
            ColorRole::OnSurfaceVariant,
        ));
        let share = if guild.today.is_empty() {
            1.0
        } else {
            guild.at as f32 / guild.today.len() as f32
        };
        status.spawn(linear_progress(Some(share)));
    });
}

/// A column `width` wide, its parts `gap` apart.
fn column(width: Val) -> Node {
    Node {
        width,
        flex_direction: FlexDirection::Column,
        row_gap: Space::Large.val(),
        flex_shrink: 0.0,
        ..default()
    }
}

/// A small heading over a control.
fn heading(parent: &mut ChildSpawnerCommands, words: &str) {
    parent.spawn(text(
        words,
        TypeRole::TitleSmall,
        ColorRole::OnSurfaceVariant,
    ));
}

/// The desk: the client, the contract and the party - or, once the day's
/// clients are dealt with, the closed doors.
fn desk(page: &mut ChildSpawnerCommands, guild: &Guild, draft: &Draft, filter: &RosterFilter) {
    let Some(index) = guild.current() else {
        closed_doors(page, guild);
        return;
    };
    page.spawn(Node {
        column_gap: Space::ExtraLarge.val(),
        row_gap: Space::ExtraLarge.val(),
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::FlexStart,
        ..default()
    })
    .with_children(|desk| {
        client_card(desk, &REQUESTS[index], guild.today.len() - guild.at - 1);
        contract(desk, draft);
        party(desk, draft, filter);
    });
}

fn client_card(desk: &mut ChildSpawnerCommands, request: &Request, waiting: usize) {
    desk.spawn(
        card(CardKind::Elevated)
            .width(Val::Px(340.0))
            .gap(Space::Large.val())
            .bundle(),
    )
    .with_children(|card| {
        card.spawn(Node {
            align_items: AlignItems::Center,
            column_gap: Space::Medium.val(),
            ..default()
        })
        .with_children(|header| {
            header.spawn(avatar(
                AvatarFace::Initials(initials(request.client)),
                AVATAR,
            ));
            header.spawn((
                text(request.client, TypeRole::TitleMedium, ColorRole::OnSurface),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            // The clients still waiting behind this one.
            header
                .spawn((
                    Node {
                        width: Val::Px(ICON),
                        height: Val::Px(ICON),
                        ..default()
                    },
                    ContentColor(Paint::new(ColorRole::OnSurfaceVariant)),
                    Tooltip::plain(format!("{waiting} more waiting")),
                ))
                .with_children(|waiting_icon| {
                    waiting_icon.spawn(icon(Glyph::Menu, ICON));
                    if waiting > 0 {
                        waiting_icon.spawn(badge(Badge::Count(
                            u32::try_from(waiting).expect("a day's clients fit"),
                        )));
                    }
                });
        });
        card.spawn((
            Arriving,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Space::Small.val(),
                ..default()
            },
        ))
        .with_children(|arriving| {
            arriving
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: Space::Medium.val(),
                    ..default()
                })
                .with_children(|line| {
                    line.spawn(circular_progress(None));
                    line.spawn(text(
                        "The courier is bringing them in...",
                        TypeRole::BodyMedium,
                        ColorRole::OnSurfaceVariant,
                    ));
                });
            for width in [280.0, 300.0, 220.0] {
                arriving.spawn(skeleton(Val::Px(width), Val::Px(14.0), Corner::ExtraSmall));
            }
        });
        card.spawn((
            Arrived,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Space::Medium.val(),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|arrived| {
            arrived.spawn(text(
                format!("\"{}\"", request.words),
                TypeRole::BodyLarge,
                ColorRole::OnSurface,
            ));
            arrived.spawn(chip_set()).with_child((
                Action::Handbook,
                chip("The clerk's handbook")
                    .icon(Glyph::ArrowForward)
                    .bundle(),
            ));
        });
    });
}

fn contract(desk: &mut ChildSpawnerCommands, draft: &Draft) {
    desk.spawn(
        card(CardKind::Outlined)
            .width(Val::Px(540.0))
            .gap(Space::Large.val())
            .bundle(),
    )
    .with_children(|form| {
        form.spawn(text("Contract", TypeRole::TitleLarge, ColorRole::OnSurface));
        form.spawn((
            Field::Title,
            text_field("Quest title")
                .outlined()
                .max_length(40)
                .text(draft.title.clone())
                .width(Val::Percent(100.0))
                .bundle(),
        ));
        form.spawn((
            Field::Reward,
            text_field("Reward")
                .outlined()
                .supporting("In gold")
                .text(draft.reward.clone())
                .width(Val::Px(200.0))
                .bundle(),
            Tooltip::rich("Write what the client offers, never more: the guild pays the rest.")
                .title("Rewards")
                .action("The handbook"),
        ));
        heading(form, "Quest");
        form.spawn((Field::Kind, segmented_buttons(KINDS, draft.kind).bundle()));
        let mut region = select("Region", REGIONS);
        if let Some(chosen) = draft.region {
            region = region.selected(chosen);
        }
        form.spawn((Field::Region, region.bundle()));
        heading(form, "Danger");
        form.spawn((
            Field::Danger,
            radio_group(DANGERS, draft.danger).horizontal().bundle(),
        ));
        heading(form, "Party size");
        form.spawn((
            Field::Size,
            notched_slider(["1", "2", "3", "4", "5"], Some(draft.size)).bundle(),
        ));
        form.spawn((Field::Rush, switch().on(draft.rush).labelled("Rush job")));
        form.spawn((
            Field::Terms,
            checkbox()
                .checked(draft.terms)
                .labelled("The client accepts the guild's terms"),
        ));
        form.spawn((
            Field::DangerPay,
            checkbox()
                .checked(draft.danger_pay)
                .labelled("Danger pay agreed"),
        ));
        form.spawn(divider(false));
        form.spawn(Node {
            column_gap: Space::Medium.val(),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|buttons| {
            buttons.spawn((
                Action::Approve,
                button("Approve").icon(Glyph::Check).bundle(),
                Tooltip::plain("File the contract with the guild"),
            ));
            buttons.spawn((
                Action::Return,
                button("Return to client").outlined().bundle(),
            ));
            buttons.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            buttons.spawn((
                Action::Clear,
                icon_button(Glyph::Close, IconButtonVariant::Standard),
                Tooltip::plain("Clear the form"),
            ));
        });
    });
}

/// A chip of the party, for adventurer `index`.
fn member_chip(index: usize) -> impl Bundle {
    (Member(index), chip(ROSTER[index].name).input().bundle())
}

fn party(desk: &mut ChildSpawnerCommands, draft: &Draft, filter: &RosterFilter) {
    desk.spawn(
        card(CardKind::Outlined)
            .width(Val::Px(420.0))
            .gap(Space::Large.val())
            .bundle(),
    )
    .with_children(|party| {
        party.spawn(text("Party", TypeRole::TitleLarge, ColorRole::OnSurface));
        party
            .spawn((PartyChips, chip_set()))
            .with_children(|chips| {
                for member in &draft.party {
                    chips.spawn(member_chip(*member));
                }
            });
        party.spawn((
            HealerHint,
            Action::SuggestHealer,
            chip("Add a healer?").suggestion().bundle(),
        ));
        party.spawn(divider(false));
        party.spawn((Field::Search, search_bar("Search the roster", Glyph::Menu)));
        party.spawn(chip_set()).with_children(|chips| {
            for class in Class::ALL {
                chips.spawn((
                    Field::Class(class),
                    chip(class.name())
                        .filter(filter.classes.contains(&class))
                        .bundle(),
                ));
            }
        });
        party
            .spawn(column(Val::Percent(100.0)))
            .with_children(|holder| {
                holder.spawn(scroll_area(Val::Px(320.0), |rows| {
                    rows.spawn((Field::Roster, list())).with_children(|roster| {
                        for (index, adventurer) in ROSTER.iter().enumerate() {
                            roster.spawn((
                                Recruit(index),
                                list_item(adventurer.name)
                                    .avatar(initials(adventurer.name))
                                    .supporting(format!(
                                        "{} - {}",
                                        adventurer.class.name(),
                                        adventurer.rank
                                    ))
                                    .trailing_icon(Glyph::Plus)
                                    .bundle(),
                            ));
                        }
                    });
                }));
            });
    });
}

fn closed_doors(page: &mut ChildSpawnerCommands, guild: &Guild) {
    page.spawn(
        card(CardKind::Filled)
            .width(Val::Px(520.0))
            .gap(Space::Large.val())
            .bundle(),
    )
    .with_children(|card| {
        card.spawn(text(
            "The doors are closed",
            TypeRole::HeadlineSmall,
            ColorRole::OnSurface,
        ));
        card.spawn(text(
            day_summary(guild),
            TypeRole::BodyLarge,
            ColorRole::OnSurfaceVariant,
        ));
        card.spawn((
            Action::OpenTomorrow,
            button("Open tomorrow").icon(Glyph::ArrowForward).bundle(),
        ));
    });
}

/// How today went, in a sentence.
pub(crate) fn day_summary(guild: &Guild) -> String {
    let today: Vec<&Handled> = guild
        .handled
        .iter()
        .filter(|handled| handled.day == guild.day)
        .collect();
    let right = today
        .iter()
        .filter(|handled| handled.reputation > 0)
        .count();
    let reputation: i32 = today.iter().map(|handled| handled.reputation).sum();
    format!(
        "Day {}: {right} of {} dealt with rightly, {reputation:+} reputation. The guild stands at {} reputation and {} gold.",
        guild.day,
        today.len(),
        guild.reputation,
        guild.gold
    )
}

fn day_over(guild: &Guild) -> impl Bundle {
    (
        DayOverDialog,
        dialog(format!("Day {} is over", guild.day))
            .icon(Glyph::Check)
            .text(day_summary(guild))
            .action(LEDGER, "See the ledger")
            .action(TOMORROW, "Open tomorrow")
            .bundle(),
    )
}

fn handbook() -> impl Bundle {
    bottom_sheet(|sheet| {
        sheet.spawn(text(
            "The clerk's handbook",
            TypeRole::TitleLarge,
            ColorRole::OnSurface,
        ));
        for rule in [
            "Read what the client says: the kind of quest, the region, the danger, how many go, and anyone the party must have.",
            "Write the reward the client offers, never more.",
            "Every contract needs the client to accept the guild's terms. High and deadly quests need danger pay agreed too; no others do.",
            "Mark a rush job only when the client is in a hurry.",
            "Return what the guild doesn't do: secrets, crimes, anything no one would sign.",
            "A right contract earns 3 reputation and a tenth of the reward; each mistake costs one, up to 3. Returning a good client costs 2; returning a bad one earns 2.",
        ] {
            sheet.spawn(text(
                rule,
                TypeRole::BodyMedium,
                ColorRole::OnSurfaceVariant,
            ));
        }
    })
}

fn ledger(page: &mut ChildSpawnerCommands, guild: &Guild) {
    page.spawn(text(
        day_summary(guild),
        TypeRole::TitleMedium,
        ColorRole::OnSurface,
    ));
    page.spawn(column(Val::Px(640.0))).with_children(|books| {
        let row = books
            .spawn(tabs(TabsKind::Primary).bundle())
            .with_children(|tabs| {
                tabs.spawn(tab(TabsKind::Primary, "Today").icon(Glyph::Check).bundle());
                tabs.spawn(
                    tab(TabsKind::Primary, "Quests out")
                        .icon(Glyph::ArrowForward)
                        .bundle(),
                );
            })
            .id();
        books.spawn(tab_panels(row)).with_children(|panels| {
            panels.spawn(tab_panel(0)).with_children(|today| {
                today_list(today, guild);
            });
            panels.spawn(tab_panel(1)).with_children(|out| {
                quests_out(out, guild);
            });
        });
    });
}

fn today_list(panel: &mut ChildSpawnerCommands, guild: &Guild) {
    let today: Vec<&Handled> = guild
        .handled
        .iter()
        .filter(|handled| handled.day == guild.day)
        .collect();
    if today.is_empty() {
        panel.spawn(text(
            "Nobody has been dealt with yet today.",
            TypeRole::BodyMedium,
            ColorRole::OnSurfaceVariant,
        ));
        return;
    }
    panel.spawn(list()).with_children(|rows| {
        for handled in today {
            let request = REQUESTS[handled.request];
            let verdict = match &handled.verdict {
                Verdict::Returned => "Returned to the client".to_owned(),
                Verdict::Filed(mistakes) if mistakes.is_empty() => {
                    "Filed, all of it right".to_owned()
                }
                Verdict::Filed(mistakes) => format!("Filed, wrong on {}", mistakes.join(", ")),
            };
            rows.spawn(
                list_item(request.client)
                    .avatar(initials(request.client))
                    .three_line(verdict)
                    .trailing_text(format!("{:+}", handled.reputation))
                    .bundle(),
            );
        }
    });
}

fn quests_out(panel: &mut ChildSpawnerCommands, guild: &Guild) {
    let mut any = false;
    for (index, handled) in guild.handled.iter().enumerate() {
        if handled.back_at.is_none() {
            continue;
        }
        any = true;
        let request = REQUESTS[handled.request];
        let party: Vec<&str> = handled
            .draft
            .party
            .iter()
            .map(|member| ROSTER[*member].name)
            .collect();
        panel
            .spawn(card(CardKind::Filled).gap(Space::Small.val()).bundle())
            .with_children(|card| {
                card.spawn(text(
                    format!("{} - {}", handled.draft.title.trim(), request.client),
                    TypeRole::TitleMedium,
                    ColorRole::OnSurface,
                ));
                card.spawn(text(
                    if party.is_empty() {
                        "Nobody went".to_owned()
                    } else {
                        party.join(", ")
                    },
                    TypeRole::BodyMedium,
                    ColorRole::OnSurfaceVariant,
                ));
                card.spawn((QuestBar(index), linear_progress(Some(0.0))));
                card.spawn((
                    QuestStatus(index),
                    text("", TypeRole::LabelMedium, ColorRole::OnSurfaceVariant),
                ));
            });
    }
    if !any {
        panel.spawn(text(
            "No quests are out.",
            TypeRole::BodyMedium,
            ColorRole::OnSurfaceVariant,
        ));
    }
}

fn settings_page(page: &mut ChildSpawnerCommands, settings: &Settings, theme: &MaterialTheme) {
    page.spawn(
        card(CardKind::Outlined)
            .width(Val::Px(480.0))
            .gap(Space::Large.val())
            .bundle(),
    )
    .with_children(|card| {
        card.spawn(text("Settings", TypeRole::TitleLarge, ColorRole::OnSurface));
        card.spawn((
            Field::Dark,
            switch()
                .on(theme.brightness() == Brightness::Dark)
                .icons(true)
                .labelled("Dark theme"),
        ));
        let colour = COLOURS
            .iter()
            .position(|(_, seed)| theme.seed == Some(hex(*seed)))
            .unwrap_or(0);
        card.spawn((
            Field::Colours,
            select("Guild colours", COLOURS.map(|(name, _)| name))
                .selected(colour)
                .bundle(),
        ));
        heading(card, "Courier's time: none to 3 seconds");
        card.spawn((Field::Courier, slider(settings.courier, 0.0..=3.0).bundle()));
        heading(card, "Clients a day, from tomorrow: 3 to 6");
        card.spawn((
            Field::PerDay,
            slider(settings.per_day as f32, 3.0..=6.0)
                .discrete(1.0)
                .bundle(),
        ));
        card.spawn((
            Field::Suggestions,
            checkbox()
                .checked(settings.suggestions)
                .labelled("Suggest fixes to a party"),
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The contract that answers request `index` rightly.
    fn right(index: usize) -> Draft {
        let request = REQUESTS[index];
        let mut party: Vec<usize> = request
            .needs
            .and_then(|class| ROSTER.iter().position(|member| member.class == class))
            .into_iter()
            .collect();
        let others: Vec<usize> = (0..ROSTER.len())
            .filter(|member| !party.contains(member))
            .take(request.party - party.len())
            .collect();
        party.extend(others);
        Draft {
            title: "A quest".to_owned(),
            reward: request.budget.to_string(),
            kind: Some(request.kind),
            region: Some(request.region),
            danger: Some(request.danger),
            size: request.party - 1,
            rush: request.rush,
            terms: true,
            danger_pay: request.danger >= DANGER_PAY_FROM,
            party,
        }
    }

    #[test]
    fn every_good_request_has_a_right_contract_and_it_earns_three_reputation() {
        for (index, request) in REQUESTS.iter().enumerate() {
            if request.refuse {
                continue;
            }
            assert_eq!(judge(request, &right(index)), Vec::<&str>::new(), "{index}");
            assert_eq!(score(request, &Verdict::Filed(Vec::new())).0, 3);
        }
    }

    #[test]
    fn each_mistake_in_a_contract_is_named_and_costs_one_reputation_up_to_three() {
        let request = REQUESTS[1];
        let mut draft = right(1);
        draft.reward = "301".to_owned();
        draft
            .party
            .retain(|member| ROSTER[*member].class != Class::Healer);
        let mistakes = judge(&request, &draft);
        assert_eq!(
            mistakes,
            ["how many go", "a healer in the party", "the reward"]
        );
        assert_eq!(score(&request, &Verdict::Filed(mistakes)).0, -3);
    }

    #[test]
    fn returning_a_bad_client_earns_reputation_and_filing_one_costs_it() {
        let stranger = REQUESTS[3];
        assert!(stranger.refuse);
        assert_eq!(score(&stranger, &Verdict::Returned), (2, 0));
        assert_eq!(score(&stranger, &Verdict::Filed(Vec::new())).0, -3);
        assert_eq!(score(&REQUESTS[0], &Verdict::Returned), (-2, 0));
    }

    #[test]
    fn a_day_brings_the_next_clients_and_wraps_round_the_list() {
        let mut guild = Guild::default();
        assert_eq!(guild.today, [0, 1, 2, 3]);
        guild.at = 4;
        guild.open(4);
        guild.open(4);
        assert_eq!(guild.day, 3);
        assert_eq!(guild.today, [8, 9, 0, 1]);
    }

    #[test]
    fn a_party_is_away_until_its_quest_comes_home() {
        let mut guild = Guild::default();
        let reputation = guild.deal(Verdict::Filed(Vec::new()), right(0), 10.0);
        assert_eq!(reputation, 3);
        let member = guild.handled[0].draft.party[0];
        assert!(guild.away(member, 10.0 + quest_seconds(0) - 0.1));
        assert!(!guild.away(member, 10.0 + quest_seconds(0)));
        assert_eq!(guild.take_back(), Some(right(0)));
        assert_eq!(guild.reputation, 0);
        assert_eq!(guild.at, 0);
    }
}
