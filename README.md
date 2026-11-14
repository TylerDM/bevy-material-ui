# bevy-material-ui

**[Material Design 3](https://m3.material.io/) for [Bevy](https://bevyengine.org/)'s UI.**
A theme of colour roles, type, shape, elevation and state layers, and the
components drawn from it - papers and cards, buttons, fields, chips,
selection controls, lists, tabs, menus, navigation, dialogs, sheets,
snackbars, tooltips and progress - for a game's menus, HUD and tools.

It is built on Bevy's own first-party UI and nothing else: every component is
laid out by `bevy_ui` and behaves as the headless widgets of
`bevy_ui_widgets` do (a button activates on a click, or on Enter or Space
while focused; a slider drags, snaps and moves with the arrow keys; a menu
opens, navigates and closes with the keyboard). This crate draws them as
Material's. Bevy's other widget set, `bevy_feathers`, is styled for Bevy's
editor and says itself it is not for a game's UI; this is.

> **Working in this repo?** [`AGENTS.md`](AGENTS.md) is the reference for how
> it is put together and the rules that hold - the theme, painting, the
> components, testing, releasing. This file covers what is in the box,
> getting it running and using it from a game. The two do not repeat each
> other.

## What is in the box

| Area | What it gives a game |
| ---- | -------------------- |
| **The theme** | `MaterialTheme`: Material 3's baseline light and dark schemes, or a whole scheme from one brand colour (`MaterialTheme::from_seed`); its fifteen kinds of text, shapes, six levels of elevation, state layers, spacing and motion. Replace or change the resource and every part repaints - light to dark at a keypress |
| **Painting** | `Fill`, `Stroke`, `Ink`, `Typeset`, `Raised`: a part names a colour role, a kind of text or a level, never a colour or a size of its own. Controls carry a `Look` and are drawn hovered, pressed, focused and disabled as Material's state layers |
| **Papers and cards** | `paper()` - a surface at a height with a shape and an outline - and elevated, filled and outlined cards, dividers |
| **Buttons** | Filled, tonal, elevated, outlined and text buttons with icons; icon buttons; small, regular, large and extended FABs; segmented buttons |
| **Selection** | Checkboxes (with indeterminate), radio groups, switches, continuous, discrete and notched sliders |
| **Text and chips** | Filled and outlined text fields with floating labels, supporting and error text, counters and clear buttons; search bars; assist, filter, input and suggestion chips |
| **Navigation** | Lists, primary and secondary tabs, top app bars, navigation bars, rails and drawers, menus and select fields |
| **Feedback** | Dialogs on a scrim, bottom sheets, a snackbar queue, plain and rich tooltips, linear and circular progress, badges and avatars, scroll areas |
| **Icons** | A small set of glyphs the components need (check, close, chevrons, arrows, menu...), drawn from strokes - no icon font - and any picture of a game's own, tinted |
| **Testing** | `bevy_material_ui::testing`: a real Bevy UI with no window or GPU, and a pointer and keyboard to drive it - for this crate's tests and a game's |

Roboto, Material's typeface, is built in (the default `roboto` feature, SIL
OFL - [`assets/fonts/Roboto-OFL.txt`](assets/fonts/Roboto-OFL.txt)). A game
with faces of its own turns default features off and sets
`MaterialTheme::fonts`.

## Using it from a game

The crate is released as tagged GitHub releases, not on crates.io. Take it
as a git dependency pinned to a release tag, with the Bevy version it is
built for (0.20):

```toml
[dependencies]
bevy = "0.20.1"
bevy_material_ui = { git = "https://github.com/TylerDM/bevy-material-ui", tag = "v0.1.0" }
```

Or, with your own fonts and without Roboto's 480 KB:

```toml
bevy_material_ui = { git = "https://github.com/TylerDM/bevy-material-ui", tag = "v0.1.0", default-features = false }
```

The crate asks Bevy only for the features its components need (`bevy_ui`,
`bevy_ui_widgets`, `bevy_input_focus`, `ui_picking`...); rendering - `bevy_ui_render`
and a window - is the game's to turn on, as a game on Bevy already does.

```rust,no_run
use bevy::prelude::*;
use bevy_material_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialUiPlugin))
        .insert_resource(MaterialTheme::from_seed(hex(0x545CB0), Brightness::Dark))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(paper().bundle()).with_children(|menu| {
        menu.spawn(text("Paused", TypeRole::HeadlineSmall, ColorRole::OnSurface));
        menu.spawn(button("Resume").bundle())
            .observe(|_: On<Activate>| info!("resume"));
        menu.spawn(button("Quit").outlined().bundle())
            .observe(|_: On<Activate>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            });
    });
}
```

Every component is a bundle - a function, or a builder's `bundle()` - that
the game spawns where it likes, with its own components beside it. What a
control does is the game's: it observes the control's events (`Activate`,
`ValueChange<T>` and the few the components add) and keeps the state.

`MaterialUiPlugin` adds Bevy's headless widgets and keyboard focus too,
unless the game added them first. Size the whole interface with Bevy's own
`UiScale`.

## The play test

```bash
cargo run --example playtest
```

Every component in every state on one screen, all of it live: click, type,
drag, tab through it with the keyboard, open the menus and dialogs, queue
snackbars, and switch light and dark (`D`) or the seed colour (`S`) to see
the whole interface repaint. It fills the primary monitor, borderless, at
its native resolution; close it with Alt+F4 (Cmd+Q on macOS). Bevy builds
slowly the first time (a few minutes); after that it starts in seconds.

To check its look without sitting at it, `--screenshots` scrolls the whole
gallery, saves each screenful - and an open dialog, sheet and snackbar - as
PNGs in a folder, and exits (add `--dark` for the dark theme):

```bash
cargo run --example playtest -- --screenshots target/screenshots
```

The same play test runs headless in the test suite (`tests/playtest.rs`):
it builds the whole screen and drives it as a player would - every control
clicked, typed into or keyed, every overlay opened and closed - and checks
what each does.

## The Guild Clerk

```bash
cargo run --example guild
```

The components in a game, where they make sense: you are the clerk of an
adventurers' guild. Each client says what they want; write the contract
(text fields, segmented buttons, a select, radio buttons, a notched slider,
a switch, checkboxes), choose its party from a roster you search and filter
(a search bar, filter chips, a list, input chips), and file it through a
dialog - or return a request the guild shouldn't take. A snackbar offers
Undo; the ledger's tabs show the day and the quests out, their progress
bars filling as parties travel; the settings turn the guild dark or
recolour it from a seed. It fills the primary monitor like the play test,
and `tests/guild.rs` plays a day of it headlessly. The game's own code is
in `examples/guild/game.rs`: how a game keeps its state and reads the
controls' events.

## Getting it running

Prerequisites: [Rust](https://rustup.rs/) - `rust-toolchain.toml` installs
the toolchain on the first `cargo` command - and on Linux, Bevy's
libraries:

```bash
sudo apt-get install libwayland-dev libxkbcommon-dev
```

Then:

```bash
git config --local include.path ../.gitconfig
```

```bash
cargo test
```

The first line applies the per-clone git settings (line endings, the
`pre-push` hook) - see "Cross-platform git".

## Layout

```
bevy-material-ui/
├── .cargo/config.toml  links with rust-lld on Windows
├── .github/            CI on three platforms, the release workflow, Dependabot
├── .githooks/pre-push  fmt and clippy before a push
├── .claude/skills/     the agents' skills: review, release, quick-push
├── AGENTS.md           how the crate is put together and the rules that hold
├── CHANGELOG.md        what each release changed, and what a game must do
├── docs/decisions.md   what was chosen, why, and whose choice it was
├── assets/fonts/       Roboto, and its licence
├── examples/playtest.rs  every component, live
├── examples/guild/     the Guild Clerk: a game of forms using every component
├── src/
│   ├── lib.rs          MaterialUiPlugin, the prelude
│   ├── theme/          the scheme, seeded schemes, type, tokens
│   ├── paint.rs        Fill, Stroke, Ink, Typeset, Raised, Look, Content
│   ├── testing.rs      the headless harness
│   └── <component>.rs  one file per component family
└── tests/              each family driven headlessly, and the play test
```

The crate is organised by component, as a design system has no features.

## Releasing

An agent does all of this with the `release` skill (`.claude/skills/release/SKILL.md`,
`/release` in Claude Code). By hand:

1. Write the release's section at the top of `CHANGELOG.md` - what changed,
   and under "Moving to it" what a game has to change - and set `version` in
   `Cargo.toml` to it, on `main`.
2. Tag that commit and push the tag:

   ```bash
   git tag v0.2.0
   ```

   ```bash
   git push origin v0.2.0
   ```

`.github/workflows/release.yml` refuses a tag that disagrees with
`Cargo.toml` or has no CHANGELOG section, runs every CI check on the tagged
commit, then creates the GitHub release with the CHANGELOG section as its
notes and the packaged crate attached. A game moves to it by changing its
`tag`.

- **The version is Semantic Versioning's**, read the Cargo way while it is
  0.x: a change a game must make is a minor (0.1 -> 0.2), anything else a
  patch. A Bevy upgrade is always a minor.
- **A version is released once.** Never move a tag a game may pin; fix
  forward with the next version.

## Licence

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your
option - as Bevy is. Roboto, built in by the `roboto` feature, is under the SIL
Open Font License ([`assets/fonts/Roboto-OFL.txt`](assets/fonts/Roboto-OFL.txt)).

## Cross-platform git

A file has the same bytes on Windows, macOS and Linux. **`.gitattributes`**
sets `* text=auto eol=lf`, which overrides `core.autocrlf` on every clone.
**`.gitconfig`** carries what `.gitattributes` cannot (it lives in
`.git/config`, which is not cloned), included once per clone by the command
above: `core.autocrlf=false`, `core.eol=lf`, `core.longpaths=true` (Cargo's
`target/` passes Windows' 260-character limit), the `pre-push` hook,
`merge.conflictstyle=zdiff3`. `.editorconfig` covers what your editor
writes; `rustfmt.toml` pins rustfmt to LF.

If the whole repository ever shows as modified, it is a normalisation
mismatch:

```bash
git add --renormalize .
```

### Case on disk

Windows and macOS directories are case-insensitive by default; Linux's are
not, so `Text_field.rs` and `text_field.rs` are two files there and one
elsewhere. Git probes `core.ignorecase` per clone, so `.gitconfig`
deliberately leaves it alone - forcing it causes phantom renames. Rename a
file's case in two steps, or Git on Windows and macOS won't see it:

```bash
git mv src/Chip.rs src/chip-tmp.rs
```

```bash
git mv src/chip-tmp.rs src/chip.rs
```

### Executable files

NTFS has no executable bit, so Windows clones run with `core.filemode=false`
and never set one. A file that must run on macOS and Linux - the `pre-push`
hook, which Git there skips otherwise - is stored executable in the index:

```bash
git add --chmod=+x .githooks/pre-push
```

### Linking on Windows

`.cargo/config.toml` links with `rust-lld`, which ships with rustup's
toolchain, instead of MSVC's `link.exe`: several times faster for a Bevy
binary, and every test file is one. It applies only to building this
repository; a game links with its own settings.
