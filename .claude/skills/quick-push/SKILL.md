---
name: quick-push
description: Put this conversation's changes - uncommitted edits and any commits it made - onto the remote main, whatever branch or worktree the session is on, leaving everything else untouched. Use whenever the user says "quick push" (or /quick-push).
---

# Quick push

Get exactly the changes made in THIS conversation onto `origin/main` - not a
branch, not a pull request. The working tree may hold other work too - the
user's own edits, another session's commits - and none of it goes to `main`.

The user asking for a quick push is the authorization to commit and to push
to `main`. Do not ask again unless a stop condition below is hit.

## 1. List what this conversation changed

**Paths**: every file you created, edited or deleted, by any tool, and
every file a command you ran changed (`cargo fmt`, `Cargo.lock` after a
dependency change). **Commits**: every commit you made, oldest first. Files
you only read do not count. If you cannot tell what you changed (the
conversation was summarized), say so and ask.

## 2. Compare it with git

```bash
git fetch origin
git branch --show-current
git status --porcelain=v1 -uall
git diff --cached --name-only
```

| Case | Action |
| --- | --- |
| On your list, changed in git | commit it |
| On your list, clean in git | already committed or reverted - skip it |
| Changed in git, not on your list | leave it alone; name it in the report |

If something not on your list is already staged, or a file on your list
holds somebody else's edits too, build the commit in a temporary worktree
instead (`git worktree add --detach .claude/worktrees/push-<name> origin/main`),
copying in only your files or applying only your hunks, so this checkout's
index stays as it was. Never `git add -p` or `-i`: they wait for input.

## 3. Stage, check and commit

Stage by explicit path only - never `git add -A`, `.` or `commit -a`:

```bash
git add -- <path> <path> ...
git diff --cached --stat
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Never stage build output (`target/`), `.claude/worktrees/` or anything
holding a secret. A check failing: fix it if it is yours, otherwise stop and
say so. Commit with a message in the repository's style
(`git log -8 --format=%s`): a sentence saying what is now true.

## 4. Push to main

On `main`: `git pull --rebase origin main`, then `git push origin main`.

On another branch or in a worktree: put your commits on top of
`origin/main` without disturbing the branch -

```bash
git worktree add --detach .claude/worktrees/push-<name> origin/main
git -C .claude/worktrees/push-<name> cherry-pick <your commits, oldest first>
git -C .claude/worktrees/push-<name> push origin HEAD:main
git worktree remove .claude/worktrees/push-<name>
```

A conflict in the cherry-pick: resolve it if it is plainly yours, otherwise
abort and ask. Never force-push `main`. The `pre-push` hook runs fmt and
clippy; never skip it with `--no-verify` unless the user says so.

## 5. Report

The commit(s) pushed, by hash and subject; anything changed in the tree
that you left out, and why.
