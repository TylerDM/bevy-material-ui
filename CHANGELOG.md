# Changelog

What each release changed, newest first, and under "Moving to it" what a
game has to change to take it. The release workflow publishes a version's
section as its GitHub release notes, and refuses a tag without one.

The versions are [Semantic Versioning](https://semver.org)'s, read the Cargo
way while the crate is 0.x: a change a game must make is a minor, anything
else a patch.

## [Unreleased]

- **Fixed**: a game that despawns its screen in its own systems - to move
  to another screen, say - no longer risks a panic from the crate's systems
  commanding a part that is gone (a skeleton's pulse did it every frame).
- **The Guild Clerk**: `cargo run --example guild`, a short game of forms
  that uses every component where a game would, and shows how a game keeps
  its state and reads the controls' events.
- **The play test** fills the primary monitor, borderless, at its native
  resolution.

### Moving to it

Nothing.

## [0.1.0]

The first release, on Bevy 0.20.1.

- **The theme**: `MaterialTheme` with Material 3's baseline light and dark
  schemes and schemes seeded from one colour (`MaterialTheme::from_seed`),
  the fifteen-role type scale, shapes, six levels of elevation, state
  layers, spacing and motion; Roboto built in (the default `roboto` feature).
- **Painting**: `Fill`, `Stroke`, `Ink`, `Typeset`, `Raised`, `Look`,
  `Content` and `ContentColor`, repainted whenever the theme changes.
- **Components**: papers, cards and dividers; buttons, icon buttons, FABs
  and segmented buttons; checkboxes, radio groups, switches and sliders;
  text fields, search bars and chips; lists, tabs, app bars, navigation
  bars, rails and drawers, menus and select fields; dialogs, bottom sheets,
  snackbars, tooltips, progress indicators, badges, avatars, loading
  skeletons and scroll areas; icons drawn from strokes.
- **Testing**: `bevy_material_ui::testing`, a headless Bevy UI with a
  pointer, a keyboard and time, for a game's tests too.
- **The play test**: `cargo run --example playtest` (`--screenshots` saves
  every screenful), and its headless twin in the test suite.

### Moving to it

Add the dependency (see README's "Using it from a game") and
`MaterialUiPlugin`.
