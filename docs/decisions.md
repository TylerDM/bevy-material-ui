# Decisions

What was chosen for this crate, why, and whose choice it was, so each can be
revisited on its merits. A decision binds as a rule in `AGENTS.md` does. One
that isn't here is the user's to make: ask, then add it here in the same
change. Values and mechanisms are written in the code, never restated here.

## The crate

- **Licensed MIT OR Apache-2.0**, as Bevy is, so a game can take it under
  either (the user, 2026-11-13).
- **Released as git tags, not on crates.io** (`publish = false`): a game pins
  `tag = "v<version>"`. Publishing to crates.io is outward-facing beyond what
  was asked; it can be added later without changing how a release is cut.
  (Claude, 2026-11-13, while the user was away.)
- **A version is released once; a failed release is fixed forward** as the
  next patch, never by moving or reusing a tag, since a game may already pin
  it (the user, 2026-11-13).
- **Roboto is built in and on by default** (the `roboto` feature): a game
  with its own faces turns default features off (the user, 2026-11-13).

## Components

- **One avatar.** Lists draw their leading avatar with the badge module's
  `avatar()` rather than one of their own: two ways to draw one thing would
  drift apart. (Claude, 2026-11-13.)
- **A continuous slider moves 1% of its range per arrow key.** Bevy's slider
  moves by its step, and one with no step didn't move at all; 1% is what a
  browser's range input does. Alternatives were 5%, or requiring a step.
  (Claude, 2026-11-13.)
- **A dialog or sheet shows the focus ring when it opens**, because Bevy's
  modal dialog moves focus into itself for keyboard users. Hiding the ring
  when it was opened by the mouse would mean fighting Bevy's widget.
  (Claude, 2026-11-13.)
- **The play test lives in the library**, as `bevy_material_ui::gallery`, so
  a game can show it in its own theme; `examples/playtest.rs` is a window
  around it. (Claude, 2026-11-13.)
