# ADR 0005: Commit, pull request and release conventions

## Status

Proposed

## Date

2026-11-24

## Context

Commit format and size were enforced only by review. History on `main` mixed issue numbers and PR numbers in commit subjects (`(#4)` from rebase-merged commits, `(#48)` from GitHub's squash merge). The 400-line limit on pull requests forced related work, such as a parent issue and its sub-issues, into several PRs. The project will publish versioned releases, so it needs a changelog that does not depend on someone remembering to write it.

## Decision

We will follow Conventional Commits 1.0.0 with two project rules: the scope is required, and a branch commit's subject ends in its issue number, `type(scope): description (#<issue>)`. The allowed types are `feat`, `fix`, `perf`, `docs`, `chore`, `refactor`, `test` and `ci`. A commit has one scope. We do not use breaking-change markers.

We will limit each commit, not each PR, to 200 changed lines, excluding `Cargo.lock` and binary files, following Google's "Small CLs" guidance. A `Large-Commit: <reason>` trailer overrides the limit for scaffolding and generated files only. A PR covers one parent issue and may close its sub-issues. A PR title has no number; GitHub's squash merge appends `(#<PR>)`, so every commit on `main` ends in its PR number.

We will enforce this with versioned git hooks in `.githooks/` (`commit-msg` for format and size, `pre-push` for fmt, clippy and test), written in plain `sh`, and with a CI workflow that checks the PR title and runs the hook tests.

We will version releases with Semantic Versioning 2.0.0 and tag them `v<version>`. `CHANGELOG.md` follows Keep a Changelog 1.1.0 and is regenerated from `main` by git-cliff at each release; it is not edited by hand.

## Consequences

- Review can go commit by commit, and a large PR no longer needs splitting into several PRs.
- Every contributor must run `git config core.hooksPath .githooks` once per clone. Until they do, only CI catches mistakes, and CI checks the PR title, not each branch commit.
- `--no-verify` skips the hooks. CI repeats the Rust checks and the title check, so what reaches `main` is still checked.
- The changelog is only as good as the PR titles, because the squash commit subject becomes the changelog entry.
- Contributors need git-cliff installed to cut a release; nobody needs it day to day.
- The `pre-push` hook makes pushing slower, because clippy and the tests run first.
- The CI title check is not yet a required status check. Making it required is a branch protection change for the repository admins.

## Alternatives considered

- **The `pre-commit` framework, lefthook or cargo-husky.** Each adds a tool or dependency to install. Plain `sh` hooks need nothing beyond git.
- **An environment variable to override the size limit.** It leaves no record. A trailer stays in history and is visible in review.
- **Issue numbers in PR titles.** The squash merge would then produce `(#<issue>) (#<PR>)`. The PR already links its issues.
- **A hand-written changelog.** It goes stale and duplicates information the commits already hold.
- **Keeping the 400-line PR limit.** It splits a parent issue's work across PRs, and review happens commit by commit anyway.
