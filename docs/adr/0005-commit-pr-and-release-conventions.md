# ADR 0005: Commit, pull request and release conventions

## Status

Proposed

## Date

2026-11-24

## Context

Commit format and size were enforced only by review. History on `main` mixed issue numbers and PR numbers in commit subjects (`(#4)` from rebase-merged commits, `(#48)` from GitHub's squash merge). The 400-line limit on pull requests forced related work, such as a parent issue and its sub-issues, into several PRs. The project will publish versioned releases, so it needs a changelog that does not depend on someone remembering to write it.

## Decision

We will follow Conventional Commits 1.0.0 with two project rules: the scope is required, and a branch commit's subject ends in its issue number, `type(scope): description (#<issue>)`. The allowed types are `feat`, `fix`, `perf`, `docs`, `chore`, `refactor`, `test` and `ci`. A commit has one scope. We do not use breaking-change markers.

We will limit each commit, not each PR, to 200 changed lines, excluding `Cargo.lock` and binary files, following Google's "Small CLs" guidance. A `Large-Commit: <reason>` trailer overrides the limit for scaffolding and generated files only. A PR covers one parent issue and may close its sub-issues. A PR title has no number. PRs are merged with a merge commit, not squashed, so the reviewed branch commits land on `main` unchanged and each ends in its issue number.

We will enforce this with versioned git hooks in `.githooks/` (`commit-msg` for format and size, `pre-push` for fmt, clippy and test), written in plain `sh`. CI runs the hook tests. The PR title is checked by the reviewer.

We will version releases with Semantic Versioning 2.0.0 and tag them `v<version>`. `CHANGELOG.md` follows Keep a Changelog 1.1.0 and is regenerated from `main` by git-cliff at each release; it is not edited by hand.

## Consequences

- Review can go commit by commit, and a large PR no longer needs splitting into several PRs.
- Every contributor must run `git config core.hooksPath .githooks` once per clone. Until they do, only CI and review catch mistakes.
- `--no-verify` skips the hooks. CI repeats the Rust checks, so broken code still cannot reach `main`; commit format and size are then left to review.
- The changelog is only as good as the commit messages, because git-cliff reads the branch commits on `main`. `Merge pull request` commits are not conventional, so git-cliff skips them. Entries end in issue numbers, not PR numbers.
- `fixup!` and `squash!` commits would reach `main` unchanged, so authors fold them in before merging.
- Contributors need git-cliff installed to cut a release; nobody needs it day to day.
- The `pre-push` hook makes pushing slower, because clippy and the tests run first.

## Alternatives considered

- **The `pre-commit` framework, lefthook or cargo-husky.** Each adds a tool or dependency to install. Plain `sh` hooks need nothing beyond git.
- **An environment variable to override the size limit.** It leaves no record. A trailer stays in history and is visible in review.
- **Squash merge.** It gives one commit per PR ending in its PR number, but throws away the small, reviewed commits that the 200-line limit produces. This was the first version of this decision and was changed in #59.
- **A hand-written changelog.** It goes stale and duplicates information the commits already hold.
- **A CI check of the PR title.** Rejected as unnecessary: the title is one line a reviewer can check, and GitHub lets it be fixed at merge time.
- **Keeping the 400-line PR limit.** It splits a parent issue's work across PRs, and review happens commit by commit anyway.
