---
name: review
description: Use before pushing work on bevy_material_ui, or when asked to review a change to it - reads the whole change against AGENTS.md (Material 3, built on Bevy's own UI, the rules for a component, testing, writing code, cross-platform, "Done means") and docs/decisions.md, looks at what it draws, and reports what breaks them.
---

# Review a change

CI refuses what fmt, clippy, the docs and the tests can see. Most of this
crate's rules none can: a literal colour that won't follow the theme, a
control without a `Look`, a behaviour Bevy's widget already has written
again, a component missing from the play test, a decision made without
asking. This review reads the change for those. It reports; it fixes only
what the user asks it to.

## 1. Find the change

From the repository's root:

```bash
git fetch origin
git status --porcelain
git merge-base HEAD origin/main
git diff <merge-base>
```

The change is everything since the branch left `main`, committed or not. On
`main` itself, it is what isn't pushed yet (`git diff origin/main`) and the
working tree. Read the whole diff before judging any of it, then read
`AGENTS.md` and `docs/decisions.md` whole.

## 2. Check every changed file

Against, in this order:

1. **Material 3.** A component drawn as Material draws it - its anatomy,
   sizes, colour roles, type, shape, elevation, states and motion. Check a
   size or a role against the spec at <https://m3.material.io/> rather than
   from memory. Something drawn off-spec for convenience is a finding.
2. **Built on Bevy's own UI.** A control is Bevy's headless widget with
   Material drawn over it: keyboard, focus or dragging written again is a
   finding, as is a new dependency on another UI crate or a Bevy feature
   collection (`ui`, `2d`). A new Bevy feature in `Cargo.toml` names its
   reason.
3. **The rules for a component**:
   - every colour a `ColorRole`, every text a `TypeRole`, every height an
     `Elevation`, every corner a `Corner`, every space a `Space` - search the
     diff for `Color::`, `srgb`, `font_size`, `BoxShadow`, and numbers in
     `Val::Px` that should be a token;
   - a control carries a `Look`, its label and icon `Content`; its own
     states change its `Look` or its parts' paint in its module's systems;
   - a component is a bundle the game spawns: it never spawns itself, holds
     a game's state or assumes where it is in the tree;
   - systems and observers registered from the module's `plugin(app)`;
   - a command on a part the system didn't just spawn is `try_insert` or
     `try_remove`, since a game may despawn it the same frame;
   - every focusable control has a `TabIndex` and a `FocusRing`, and the
     keys Material and Bevy's widget name work;
   - time from Bevy's `Time`, never the clock.
4. **The decisions** in `docs/decisions.md`. A change that contradicts one,
   or makes a choice none records (a dependency, a departure from the spec,
   a second way to draw something), is a question for the user, not a
   finding to fix.
5. **Writing code** and **Cross-platform rules** in `AGENTS.md`: a helper
   that duplicates one in `paint.rs`, `theme` or `testing.rs` (search for
   its noun); a name that isn't a word; an item `pub` beyond its callers; an
   option nobody sets; `unwrap()`; `allow` instead of `expect` with a reason;
   a path joined as a string; a `cfg` with an arm missing.
6. **What goes with the change** - "Done means": its tests in the same
   commit, named as the rule each holds and driven through
   `bevy_material_ui::testing`; a new `On*` role or scheme covered by the
   contrast tests; the component in the play test in every state and driven
   by `tests/playtest.rs`; a `///` on every public item; a public item
   renamed or removed said for the next CHANGELOG's "Moving to it"; README,
   AGENTS.md and the decisions where the change made them wrong; a file from
   outside the repository with its source and licence beside it.

A rule is broken only where it says so - report what a rule forbids, not
what you would have written differently. Where the code says why it departs
from a rule (an `expect` with its reason), it isn't a finding unless the
reason is wrong.

## 3. Look at what it draws

When the change draws anything, save the play test's screenfuls, light and
dark, into a folder outside the repository, and read the pages that show
it:

```bash
cargo run --example playtest -- --screenshots <folder>
cargo run --example playtest -- --screenshots <folder> --dark
```

Each needs a window and a GPU; if this machine has none, say so in the
report rather than skipping it silently. A part clipped, overflowing,
misaligned, unreadable or coloured off its role is a finding.

## 4. Report

Most severe first, each one:

- `path:line` - the rule broken, in a few words, and where it is written
  (`see "A part names a role, never a value" in AGENTS.md`);
- what to do instead, in one line.

Then a line for each of the checks above that found nothing, so the reader
knows it was checked, and anything the review couldn't judge. If nothing is
broken, say so plainly. A problem found that the change didn't cause isn't a
finding: it is a cheap fix in its own commit, or an issue (AGENTS.md, "A
problem found on the way").
