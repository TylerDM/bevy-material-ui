---
name: release
description: Use when the user says "release", "cut a release", "publish the crate", "ship vX.Y.Z" (or /release) - choose the version, write the CHANGELOG section, bump Cargo.toml, tag, watch the release workflow and check the GitHub release arrived.
---

# Release

A release of bevy_material_ui is a `v<version>` tag on `main`. Pushing it
runs `.github/workflows/release.yml`, which refuses a tag whose version
disagrees with `Cargo.toml` or has no `CHANGELOG.md` section, runs every CI
check on the tagged commit, and publishes the GitHub release - the
CHANGELOG section as its notes, the packaged crate attached. Games take the
crate as a git dependency pinned to the tag; there is no crates.io publish.

README's "Releasing" is the rule this follows; it wins if this disagrees.

**Ask nothing.** The user asking for a release is the approval of all of it:
the version this skill chooses, the notes it writes, the commit to `main`
and the tag's push. Carry on to the end unless a stop condition below is
hit. **A released version can never be replaced** - a game may already pin
the tag - so get the version and the notes right before anything leaves the
machine. They are shown in the report, not put to the user first.

## 1. Check it is safe to start

```bash
git fetch origin --tags
git status --porcelain=v1
git branch --show-current
git rev-parse HEAD origin/main
gh run list --workflow ci.yml --branch main --limit 1
```

Stop and say why, unless the tree is clean, the branch is `main` at the
same commit as `origin/main`, and CI on that commit finished green (still
running: wait for it with `gh run watch <id> --exit-status`, in the
background; failed: stop). If HEAD is already tagged, there is nothing new
to release.

## 2. Choose the version

The last release is the newest tag: `git tag --sort=-v:refname`. Read what
landed since:

```bash
git log --format="%h %s" <last tag>..HEAD
git diff <last tag>..HEAD --stat
git diff <last tag>..HEAD -- src/lib.rs Cargo.toml
```

While the crate is 0.x, read Semantic Versioning the Cargo way:

| Change since the last tag | Version |
| --- | --- |
| A game must change its code: a public item renamed, removed or changed in signature; a component's events or state components changed; a Bevy upgrade; a new required feature | minor (0.1.3 -> 0.2.0) |
| Anything else: a new component, a fix, a look closer to the spec | patch (0.1.3 -> 0.1.4) |

A version the user names wins. The version must be new
(`git ls-remote --tags origin v<version>` prints nothing).

## 3. Write the notes

A section at the top of `CHANGELOG.md`, under the intro:

```markdown
## [<version>]

<one line: what this release is about; the Bevy version if it moved>

- **<area>**: what is new or fixed, in a game's words.

### Moving to it

What a game has to change, each with the old name and the new; "Nothing."
when nothing.
```

Read the diffs from step 2 for every public item renamed or removed - in
`src/lib.rs`'s `pub use` lines and each module's `pub` items - and say each
under "Moving to it".

## 4. Bump, check and commit

Set `version` in `Cargo.toml` to it, then from the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --allow-dirty --no-verify
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Release v<version>"
git push origin main
```

Any check failing: stop, and say what failed. Do not release around it.

## 5. Tag and watch

```bash
git tag v<version>
git push origin v<version>
gh run list --workflow release.yml --limit 1
gh run watch <id> --exit-status
```

Run the watch in the background. If the workflow fails, read its log
(`gh run view <id> --log-failed`), fix forward on `main`, and release the
next version: never move or reuse the tag.

## 6. Check it arrived

```bash
gh release view v<version>
```

It must exist, carry the notes and the `.crate` asset.

## 7. Report

The version and why (the rule from step 2), the notes as published, the
release's link, and the line a game puts in its `Cargo.toml`:

```toml
bevy_material_ui = { git = "https://github.com/TylerDM/bevy-material-ui", tag = "v<version>" }
```
