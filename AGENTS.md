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
```

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
- **Keyboard and focus work for every control**: each focusable control
  has a `TabIndex` and a `FocusRing`; Tab, Enter, Space, the arrows and
  Escape do what Material and Bevy's widget say.
- **Time is Bevy's `Time`**, never `Instant::now` (clippy refuses it), so the
  harness's sixtieth-of-a-second frames make timed parts - snackbars,
  tooltips, animations - certain in tests.
- **Every component is in the play test**, in every state it has, and the
  headless play test drives it.

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
