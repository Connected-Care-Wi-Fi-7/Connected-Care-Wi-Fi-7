# Contributing

This guide explains how the team works on Connected Care Wi-Fi 7. Read it before your first pull request. For what the system is, see `SPECIFICATION.md`. For how the code is meant to be organised, see `docs/ARCHITECTURE.md`. For who owns what, see `docs/ROLES.md`.

## Workflow

1. **Issue first.** Every change starts with a GitHub issue created from the issue form. If the work is not described in an issue, create one before you write code. Apply the `type:`, `area:` and `priority:` labels and a milestone.
2. **Branch.** Create a branch from an up-to-date `main`, named `type/short-name`. The type is one of `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `ci`. Use lower-case words separated by hyphens. Examples:
   - `feat/hampel-filter`
   - `fix/voxel-grid-off-by-one`
   - `docs/update-requirements`
   - `chore/project-standards`
3. **Commit with Conventional Commits.** The format is `type: short description` or `type(scope): short description`, in the imperative mood. Examples:
   - `feat(cc-dsp): add Hampel outlier filter`
   - `fix(cc-sim): keep frame timestamps monotonic`
   - `docs: describe the replay file format`
   - `ci: pin the Rust toolchain`
4. **Open a pull request** against `main`. Fill in the template and put `Closes #<issue number>` under "Linked issue" so the issue closes when the PR merges.
5. **Review.** One approving review from another team member is required. Reviewers check the Definition of Done below.
6. **CI must be green** before merging.
7. **Squash merge.** The squashed commit message must also follow Conventional Commits.

Keep pull requests small, under roughly 400 changed lines. If a change is larger, split it into several issues and several PRs. Small PRs are reviewed faster and are easier to revert.

Never push directly to `main`.

## Local checks

Run these three commands before you push. CI runs the first two today; `cargo test` joins CI once the Cargo workspace is on `main`.

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

To fix formatting automatically, run `cargo fmt --all`.

## Definition of Done

An issue is done when all of the following are true:

- Every acceptance criterion in the issue is ticked.
- Tests were added or updated for the change.
- `cargo fmt` and `cargo clippy -D warnings` are clean.
- Public items have documentation comments.
- The feature runs against the simulator, so anyone can try it without hardware.
- The PR was reviewed, CI is green and the PR is merged.

If an acceptance criterion can only be checked on real hardware, say so in the issue. The Hardware dependency field in the issue form exists for this. Results measured on the simulator must be described as "simulator-verified", never as "hardware-verified".

## Architecture rules

- Only `apps/viewer` depends on Bevy. Every crate under `crates/` is headless and unit-testable.
- Crates communicate only through types defined in `cc-core`. A crate must not reach into another crate's internals.
- Hardware sits behind a trait defined in `cc-core` and behind a cargo feature. The rest of the code never mentions a specific device.
- A change to a `cc-core` contract (data types or traits) needs an ADR and a review from the Platform & Integration role. See "Decisions" below.

## Rust rules

- The Rust toolchain version is pinned in the repository. Do not change it without an issue.
- Use `#![forbid(unsafe_code)]` in every crate except hardware driver crates. If a driver crate needs `unsafe`, keep it in the smallest possible block and explain why in a comment.
- Use `thiserror` for error types in library crates and `anyhow` in applications.
- Do not use `unwrap` or `expect` outside tests. Return errors instead.
- Document public items with `///` comments.

## Zero-cloud rules

The project requires all processing to stay on the local machine (FR-05).

- Do not write outbound network code. No HTTP clients, no telemetry, no update checks.
- Any listener you add must bind to `127.0.0.1` only, never `0.0.0.0`.
- Check every new dependency before adding it. It must not pull in network clients without a reason agreed in the issue. Dependencies will be checked by `cargo-deny` once it is set up in CI.
- Hardware and sensors are chosen by a local config file, not by a remote service.

## Privacy and data rules

The project processes zero personally identifiable information (PII) (FR-06).

- Do not commit real captures, recordings of real people, or anything that could identify a person. This includes names, email addresses, room numbers and photographs.
- Do not commit large data files. Datasets stay outside the repository, and the repository records where they come from.
- Use simulated data by default. Use public datasets only after recording them in the provenance register.
- **Dataset provenance register.** Once the first dataset is used, create `docs/DATA_PROVENANCE.md`. Add one entry per dataset giving its name, where it came from (link), its licence, the date it was obtained, what the team uses it for, and confirmation that it contains no PII. A dataset with no entry must not be used.

## Labels

Every issue carries one label from each of these groups. The form cannot set them for you, so apply them by hand.

| Prefix | Meaning | Values |
| --- | --- | --- |
| `type:` | What kind of work it is | `bug`, `feature`, `task`, `spike`, `docs` |
| `area:` | Which part of the system, matching a role in `docs/ROLES.md` | `platform`, `sensing`, `dsp`, `mapping`, `dashboard`, `alerts`, `project` |
| `priority:` | How important it is | `must`, `should`, `could` |

Two further labels have no prefix:

- `hw:needed` marks an issue that cannot be finished without real hardware.
- `blocked` marks an issue waiting on something else. Say what in a comment.

A spike is a time-boxed investigation. Its output is an answer or a decision, often an ADR, not production code.

## Milestones

Dates are proposed and may change by team agreement.

| Milestone | Due | Exit condition |
| --- | --- | --- |
| M0 Foundations | 27 Nov 2026 | Workspace merged, contracts and traits agreed, CI green with tests |
| M1 Walking skeleton | 18 Dec 2026 | Simulated scene flows end to end into the Bevy app using fakes; a scripted fall raises an alert |
| M2 Real algorithms on simulated/open data | 5 Feb 2027 | Fakes replaced by real DSP, mapping and tracking; acceptance numbers measured on the simulator |
| M3 Hardware integration & evaluation | 26 Feb 2027 | Intel BE200 source plugged in if hardware arrives; evaluation report drafted |
| M4 Delivery | 12 Mar 2027 | Feature freeze, Hard Deadline 1 |

## Decisions (ADRs)

We record significant technical decisions as Architecture Decision Records in `docs/adr/`.

1. Copy `docs/adr/0000-template.md` to the next free number, for example `docs/adr/0002-short-title.md`.
2. Fill it in. Set the status to `Proposed`.
3. Open a PR. Discuss in the review.
4. When the team agrees, set the status to `Accepted` and merge.

Write an ADR when you change a `cc-core` contract, choose between libraries or methods, or change the architecture. Do not edit an accepted ADR to change its decision. Write a new ADR that supersedes it and update the old one's status to say so.

## Releases

- Tag the `main` branch at the end of each milestone, for example `m1-walking-skeleton`.
- Keep a `CHANGELOG.md` at the repository root. Add the notable changes for each milestone when you tag it.
