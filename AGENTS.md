# AGENTS.md

Guidance for AI coding agents working in this repository - the cross-tool
convention (Claude Code, Codex, Copilot, Cursor and others). There is
deliberately no separate CLAUDE.md to keep in sync.

## What this repository is

**bevy_material_ui is Material Design 3 for Bevy's UI**: one Rust crate, a
library, released as tagged GitHub releases that games - the-world first -
take as a git dependency. It runs wherever Bevy does, and Windows, macOS and
Linux are all first-class: CI tests on all three.

```
src/theme/      MaterialTheme: the scheme (colour roles), seeded schemes, the type scale, tokens
src/paint.rs    how a part is coloured and set from the theme - read it before any component
src/focus.rs    the focus ring
src/icon.rs     glyphs drawn from strokes, and pictures as icons
src/testing.rs  the headless harness: a real Bevy UI, a pointer and a keyboard, no window
src/<family>.rs one file per component family: paper, button, checkbox, text_field, menu, dialog...
tests/          each family driven headlessly; tests/playtest.rs drives the whole play test
examples/playtest.rs  every component, live, in a window
examples/guild/       the Guild Clerk: a game of forms using every component; tests/guild.rs plays it
docs/decisions.md     what was chosen, why, and whose choice it was
```

## Done means

Before calling a change finished:

- [ ] `docs/decisions.md` read for what it touches, and no decision made
      silently: asked, then recorded there ("Decisions")
- [ ] its tests in the same commit, each named as the rule it holds
      ("Testing")
- [ ] every component it adds or changes in the play test, in every state,
      and anything it draws looked at in `--screenshots`, light and dark
- [ ] `cargo test`, `cargo clippy --all-targets --all-features -- -D
      warnings`, `cargo fmt --all` and the docs clean
- [ ] the diff reread for anything that duplicates code already here and
      anything added but not used ("Writing code")
- [ ] what a game sees changed: a line for the next `CHANGELOG.md` section,
      and under "Moving to it" what a game must change
- [ ] AGENTS.md, README.md, `docs/decisions.md` and the skills updated if any
      is now wrong
- [ ] work done by hand twice, or costly in tokens, made a script ("Scripts,
      not tokens")
- [ ] every problem found on the way, and not fixed, an issue ("A problem
      found on the way")
- [ ] the `review` skill read the change before it is pushed

## The skills

Each topic's own steps are a skill in `.claude/skills/<name>/SKILL.md`,
found by its description; a rule in one binds as a rule here does.

| Skill | Use it |
| --- | --- |
| [`review`](.claude/skills/review/SKILL.md) | before pushing: the change read against this file and the decisions |
| [`release`](.claude/skills/release/SKILL.md) | to cut a release: the version, the CHANGELOG, the tag, the GitHub release |
| [`quick-push`](.claude/skills/quick-push/SKILL.md) | to put a session's changes on the remote `main` |

## Decisions

**[`docs/decisions.md`](docs/decisions.md) records what was chosen, why and
by whom; a decision binds as a rule here does.** Read it before work it
touches. A choice it doesn't make - a new dependency, a departure from the
spec, a second way to draw something, how a release is published - is the
user's: ask, then record the answer there in the same change.

## A problem found on the way

**A cheap fix found on the way is made; anything bigger is an issue, never
a detour and never left unsaid.** Something wrong that isn't what you were
asked to do - a bug, a missing test, a part drawn off-spec, a doc that has
drifted - is fixed if it takes a few lines and decides nothing unrecorded,
in its own commit, named in the report. Otherwise search the issues on
[TylerDM/bevy-material-ui](https://github.com/TylerDM/bevy-material-ui/issues)
(a comment on the one already asking beats a second), then file it with
`gh issue create`: what is wrong, where, how it was seen, and the fix if
known. The report names each issue's link.

## Scripts, not tokens

**A task an agent repeats, or would spend many tokens doing by hand, is
code**: a test where it checks, an example or a `testing` helper where it
drives, as `--screenshots` saves every screenful of the play test rather
than an agent opening the window and capturing it. Look for one before
doing such a task by hand, and name a new one where agents look - here
under "Commands", or in its skill.

## Assets that aren't ours

**A file from outside the repository records where it came from and its
licence, beside it, before it is committed** - as
`assets/fonts/Roboto-OFL.txt` does for Roboto - or it isn't used. Its licence
must let a game ship it under MIT OR Apache-2.0 terms; README's "What is in
the box" names any a game inherits.

## The design system is Material Design 3

**Every component follows [Material Design 3](https://m3.material.io/)** - its
anatomy, sizes, colour roles, type, shapes, elevation, states and motion.
That is the crate's purpose, not a starting style. A component Material has
is drawn as Material draws it; one it lacks is composed from its tokens.
When the spec and convenience disagree, the spec wins; when a game's needs
and the spec disagree, the game themes it (a scheme, fonts) - it never
needs a component drawn off-spec.

## Built on Bevy's own UI, and nothing else

- **Layout is `bevy_ui`'s, behaviour is `bevy_ui_widgets`'.** A control is
  Bevy's headless widget - `Button`, `Checkbox`, `RadioGroup`, `Slider`,
  `TextInput`, `ListBox`, `TabList`, `MenuButton`, `ModalDialog`,
  `Scrollbar` - with Material drawn over it. Never reimplement a behaviour
  Bevy's widget has (keyboard, focus, dragging); never take a dependency on
  another UI crate. `bevy_feathers` is Bevy's editor style, not a base.
- **External state, as `bevy_ui_widgets` has it.** A control emits
  `Activate` or `ValueChange<T>`; the game keeps the value and sets the
  control's state component (`Checked`, `SliderValue`...). A component that
  offers a self-updating mode says so.
- **The crate asks Bevy only for the features it uses**, each by name in
  `Cargo.toml` with its reason; rendering is the game's. Never a feature
  collection (`ui`, `2d`). A new one needs a reason a game would accept.
- **Bevy is pre-1.0 and pinned.** An upgrade is a release of its own (a
  minor), with every component and test brought along in one change.

## The rules for a component

- **A part names a role, never a value.** Colours are `ColorRole`s through
  `Fill`, `Stroke`, `Ink`, `Look`, `Content` or `ContentColor`; text is a
  `TypeRole` through `Typeset`; height is an `Elevation` through `Raised`;
  corners are `Corner`s; space is `Space`. A literal colour, font size or
  shadow in a component is a defect: it will not follow the theme.
- **A control carries a `Look`**, and its label and icon carry `Content`, so
  hover, press, focus and disabled are drawn by `paint.rs` for every control
  alike. A control's own states beyond those (checked, selected, open) are
  drawn by its module's systems, by changing its `Look` or its parts' paint.
- **A component is a bundle the game spawns**: a function returning
  `impl Bundle`, or a builder whose `bundle()` does. The game puts its own
  components beside it. A component never spawns itself, never holds a
  game's state, and never assumes where it is in the tree.
- **A module registers its own systems and observers** from
  `pub(crate) fn plugin(app: &mut App)`, called by `MaterialUiPlugin`.
- **A part may be despawned in any frame.** A game replaces its screens in
  its own systems, in no order with the crate's, and Bevy panics on a
  command to an entity that is gone. So a system or observer commanding a
  part it didn't spawn in that same command uses `try_insert` and
  `try_remove`, never `insert` and `remove`.
- **Keyboard and focus work for every control**: each focusable control
  has a `TabIndex` and a `FocusRing`; Tab, Enter, Space, the arrows and
  Escape do what Material and Bevy's widget say.
- **Time is Bevy's `Time`**, never `Instant::now` (clippy refuses it), so the
  harness's sixtieth-of-a-second frames make timed parts - snackbars,
  tooltips, animations - certain in tests.
- **Every component is in the play test**, in every state it has, and the
  headless play test drives it. The Guild Clerk (`examples/guild/`) shows
  them in a game; a component a game would use belongs in it too.

## Testing

- **Every change comes with its tests**, in the same commit: a unit test for
  pure logic beside it, and the component driven headlessly in its family's
  file under `tests/` - clicked, typed into, keyed, hovered - asserting what
  it emitted and what it painted from the theme.
- **A test's name is the rule it holds**, as a sentence:
  `a_disabled_button_is_faded_and_a_click_does_nothing_until_it_is_enabled`.
- **Tests need no window or GPU.** `bevy_material_ui::testing` builds a real
  Bevy UI laid out at a window size, with a pointer (`click`, `hover`), a
  keyboard (`focus`, `press`) and time (`wait`). Add a helper there rather
  than driving Bevy's messages by hand in a test.
- **Contrast is tested**: every `On*` role reads on its surface at 4.5:1 in
  the baseline schemes and in seeded schemes across many seeds.

## Writing code

- **Names are words**: `|button|`, not `|b|`.
- **Each item says what it is in a sentence** - a `///` on every public item
  (`missing_docs` warns, and CI makes warnings errors). Comments say why,
  never what the next line does.
- **Build what is asked for.** No option nobody sets, nothing `pub` beyond
  its callers, no code kept for later.
- **A panic says why it can't happen**: `expect("...")`, not `unwrap()`.
- **An exception is visible**: `#[expect(lint, reason = "...")]`, never
  `allow`. Turning a lint off is a line in `Cargo.toml` with its reason.
- **No `unsafe`.** The crate forbids it.
- **Search before you write**: a second copy of a helper is a refactor into
  the module both callers reach (`paint.rs`, `theme`, `testing.rs`).

## Cross-platform rules

README's "Cross-platform git" covers line endings, case on disk and
executable bits. The rules for the code:

- **Case matters.** `mod text_field;` finds `text_field.rs` and never
  `Text_Field.rs` on Linux. Match a file's case exactly in every path -
  `mod`, `include_bytes!`, asset paths - though Windows and macOS forgive it.
- **Asset paths are Bevy's**: relative to `assets/`, with `/` on every
  platform. Other paths are built with `Path`/`PathBuf`, never by joining
  strings, and never relative to the current directory.
- **A script that must run on all three platforms is Rust** - an example, a
  test or `build.rs` - not `.sh` or `.ps1`. The git hook is the exception:
  Git runs it with its own `sh` everywhere, and it is stored executable
  (`git add --chmod=+x`).
- **Platform-specific code goes behind `#[cfg(target_os = ...)]`**, every
  arm written in the same change. `#[cfg(unix)]` is true on macOS too, so it
  never means Linux. Needing one is a reason to look for the Bevy way first.

## Commands

```bash
cargo test                                         # every test, headless
cargo run --example playtest                       # every component, live
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
cargo doc --no-deps --all-features --open
```

`.githooks/pre-push` runs fmt and clippy; CI (`.github/workflows/ci.yml`)
runs those, the docs and the tests on Linux, Windows and macOS. The
formatter's output is the only formatting.

## Releasing

A release is a `v<version>` tag on `main` whose `Cargo.toml` version and
`CHANGELOG.md` section agree; `.github/workflows/release.yml` checks it and
publishes the GitHub release. The `release` skill (`.claude/skills/release/`)
does all of it; README's "Releasing" says it by hand. A version is
released once - fix forward.

## Upgrading Bevy

Bump `bevy` in both `[dependencies]` and `[dev-dependencies]`, and
`rust-version` to the new Bevy's minimum; read Bevy's migration guide for
`bevy_ui`, `bevy_ui_widgets`, `bevy_input_focus`, `bevy_picking` and
`bevy_text`; bring every component and test along; and release it as a minor,
naming the Bevy version in the CHANGELOG. A game moves to it with its own
Bevy upgrade.

## The split with README.md

This file holds the architecture and the rules. `README.md` covers only
what it does not: what is in the box, using it from a game, the play test,
getting it running, the layout, releasing by hand and the git setup.
Explain a mechanism in one and link to it from the other; update both in
the change that makes either wrong.
