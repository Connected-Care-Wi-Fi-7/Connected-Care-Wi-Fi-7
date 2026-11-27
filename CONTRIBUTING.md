# Contributing

This guide explains how the team works on Connected Care Wi-Fi 7. Read it before your first pull request. For what the system is, see `SPECIFICATION.md`. For how the code is meant to be organised, see `docs/ARCHITECTURE.md`. For who leads which area, see `docs/ROLES.md`.

## Workflow

1. **Issue first.** Every change starts with a GitHub issue created from the issue form. Roles are loose (see `docs/ROLES.md`): take any open issue in any area by assigning yourself. If the work is not described in an issue, create one before you write code. The `type:`, `area:` and `priority:` labels are applied from the form answers automatically; check them and set a milestone.
2. **Branch.** Create a branch from an up-to-date `main`, named `type/short-name`. The type is one of `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `ci`. Use lower-case words separated by hyphens. Examples:
   - `feat/hampel-filter`
   - `fix/voxel-grid-off-by-one`
   - `docs/update-requirements`
   - `chore/project-standards`
3. **Commit** in small steps, following "Commits" below. The hooks in `.githooks/` check every commit (see "Local checks").
4. **Open a pull request** against `main`. Fill in the template. The title follows "Commits" below but without a number, for example `feat(cc-dsp): add Hampel outlier filter`. No hook can check the title, so the reviewer does. A PR covers one parent issue and may also close that issue's sub-issues. Put one `Closes #<number>` line per issue under "Linked issue", because closing a parent issue does not close its sub-issues. Unrelated changes go in separate PRs. There is no line limit on a PR.
5. **Review.** The PR's lead reviewer must approve it: the lead of the linked issue's area, or the Team Lead if you are that lead, or the backup reviewer if you are also the Team Lead. They are requested automatically (see "How review routing works" in `docs/ROLES.md`). Anyone else may review too. If the PR changes anything under `.github/`, a code owner (the Platform & Integration lead or the Team Lead, but not the author) must also approve it, because a workflow there could fake a required check. Every new push needs a fresh approval, every "Request changes" review must be resolved and every conversation resolved before merging. These rules apply to everyone, admins included. Reviewers check the Definition of Done below.
6. **CI must be green** before merging. The required checks are `rust`, `deny`, `localhost-bind` and `Lead approval`.
7. **Merge commit.** Merge with "Create a merge commit", not squash or rebase, so the reviewed branch commits land on `main` as they are, each reading `type(scope): description (#<issue>)`. Before merging, fold any `fixup!` or `squash!` commits into the commits they fix with `git rebase -i --autosquash main`.

Never push directly to `main`.

## Commits

We follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) with two project rules: the scope is required, and the subject ends in the issue number. The format is:

```
type(scope): description (#<issue>)

optional body

optional footer
```

- **type** is one of `feat`, `fix`, `perf`, `docs`, `chore`, `refactor`, `test`, `ci`.
- **scope** is one lower-case area, usually a crate or folder, for example `cc-dsp`, `viewer`, `agents` or `github`. One scope per commit: work in two areas is two commits.
- **description** is in the imperative mood: "add", not "added".
- Leave a blank line before the body. The body explains why, not what.
- Examples:
  - `feat(cc-dsp): add Hampel outlier filter (#12)`
  - `fix(cc-sim): keep frame timestamps monotonic (#20)`
  - `docs(adr): record the replay file format (#31)`

Messages git writes itself (`Merge ...`, `Revert "..."`, `fixup!`, `squash!`) are accepted as they are. `fixup!` and `squash!` commits must be folded in before the PR merges (see step 7 above).

**Size.** Keep each commit under 200 changed lines (added plus deleted), so a large PR can be reviewed one commit at a time. `Cargo.lock` and binary files are not counted. This follows Google's [Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html) guidance. Only for scaffolding or generated files that cannot be split, add a trailer as the last line of the message saying why:

```
chore(viewer): scaffold the Bevy app (#7)

Large-Commit: generated project template
```

Reviewers should question any `Large-Commit` that is not scaffolding or generated output.

## Local checks

Turn on the project's git hooks once per clone:

```
git config core.hooksPath .githooks
```

- `commit-msg` checks the commit format and size rules in "Commits".
- `pre-push` runs the three CI checks below and stops the push if one fails.

The hooks are plain `sh` scripts with no extra dependencies, and work on macOS, Linux and Git Bash on Windows. `--no-verify` skips them, but CI runs the same checks, so only use it when you know CI will catch the problem anyway. After changing a hook, run `.githooks/test-hooks`.

The pre-push checks, which CI also runs:

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
- A change to a `cc-core` contract (data types or traits) needs an ADR and a review from the Platform & Integration lead. See "Decisions" below.

## Rust rules

- The Rust toolchain version is pinned in the repository. Do not change it without an issue.
- Use `#![forbid(unsafe_code)]` in every crate except hardware driver crates. If a driver crate needs `unsafe`, keep it in the smallest possible block and explain why in a comment.
- Use `thiserror` for error types in library crates and `anyhow` in applications.
- Do not use `unwrap` or `expect` outside tests. Return errors instead.
- Document public items with `///` comments.

## Zero-cloud rules

The project requires all processing to stay on the local machine (FR-05, local edge processing).

- Do not write outbound network code. No HTTP clients, no telemetry, no update checks.
- Any listener you add must bind to `127.0.0.1` only, never `0.0.0.0`.
- Check every new dependency before adding it. It must not pull in network clients without a reason agreed in the issue.
- Hardware and sensors are chosen by a local config file, not by a remote service.

CI enforces these rules with two jobs. Both are required checks on `main`, so a PR cannot merge while either fails:

- **`deny`** runs [`cargo-deny`](https://embarkstudios.github.io/cargo-deny/) with `deny.toml` over all features, so crates behind off-by-default hardware features are checked too. It fails on:
  - a banned crate: HTTP clients, HTTP/QUIC stacks, WebSockets, TLS libraries and telemetry or crash reporting. Each ban in `deny.toml` gives its reason. MQTT clients are not banned yet; that needs a team decision.
  - a licence outside the permissive list (MIT, MIT-0, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-3.0, Zlib). No copyleft. If one crate needs another licence, add a per-crate `exceptions` entry with a reason instead of widening the list.
  - a RustSec advisory (vulnerable, unmaintained or yanked crate). Ignore one only with an `ignore` entry and a reason.
  - a dependency from anywhere other than crates.io.
  Duplicate crate versions are only a warning, because Bevy pulls in several.
- **`localhost-bind`** runs `scripts/check-localhost-bind` over tracked `.rs` and `.toml` files. It fails on any wildcard address (`0.0.0.0`, `[::]`, `Ipv4Addr::UNSPECIFIED` and similar) and on any IPv4 address other than `127.0.0.1` in a bind call or a `bind`/`listen` setting. A line that has to mention such an address, such as a test that checks it is rejected, can end with `// bind-check: allow`; reviewers should question every use. It is a text check, so addresses built at runtime are not seen and review still applies.

To run both locally (install cargo-deny once with `cargo install --locked cargo-deny`):

```
cargo deny --all-features check
scripts/check-localhost-bind
```

## Privacy and data rules

The project processes zero personally identifiable information (PII) (FR-06, privacy-first synthetic and controlled testing).

- Do not commit real captures, recordings of real people, or anything that could identify a person. This includes names, email addresses, room numbers and photographs.
- Do not commit large data files. Datasets stay outside the repository, and the repository records where they come from.
- Use simulated data by default. Use public datasets only after recording them in the provenance register.
- **Dataset provenance register.** Once the first dataset is used, create `docs/DATA_PROVENANCE.md`. Add one entry per dataset giving its name, where it came from (link), its licence, the date it was obtained, what the team uses it for, and confirmation that it contains no PII. A dataset with no entry must not be used.

## AI-assisted work

AI tools may be used. The person who submits the work is its author and is responsible for it: they must understand it and be able to explain it.

- AI tools never contribute under their own GitHub account. Every commit, issue, pull request and comment comes from a team member's account.
- Do not add `Co-Authored-By` trailers for AI tools to commits, and do not add promotional "Generated with ..." footers to pull requests.
- Do not add links to AI tool sessions or chat threads, or hidden attribution comments, to commits, issues, pull requests or comments.
- Every issue and pull request that an AI tool drafted, in whole or in part, must end with a short, plain one-line disclosure, for example "This was drafted by Claude and reviewed by me before being submitted."
- This does not replace any declaration the module requires for assessed work. Follow the module's own rules on declaring AI use.

## Labels

Every issue carries one label from each of these groups. A workflow (`.github/workflows/issue-triage.yml`) applies them from the issue form answers when the issue is opened. Labels you set by hand are never overwritten.

| Prefix | Meaning | Values |
| --- | --- | --- |
| `type:` | What kind of work it is | `bug`, `feature`, `task`, `spike`, `docs` |
| `area:` | Which part of the system, matching an area in `docs/ROLES.md` | `platform`, `sensing`, `dsp`, `mapping`, `dashboard`, `alerts`, `project` |
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

Versions follow [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html). While the version is `0.x`, each milestone release bumps the minor version (`0.1.0`, `0.2.0`) and a fix between milestones bumps the patch version (`0.2.1`). Tags are `v<version>`, for example `v0.2.0`.

`CHANGELOG.md` follows [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) and is generated from the commits on `main` by [git-cliff](https://git-cliff.org) using `cliff.toml`. `feat` commits go under "Added", `perf` under "Changed" and `fix` under "Fixed"; other types are left out. Do not edit `CHANGELOG.md` by hand: the next release regenerates it, so fix a bad entry by writing better commit messages. Install git-cliff with `brew install git-cliff` or `cargo install git-cliff`.

To release:

1. Create a release issue and a branch `chore/release-v<version>` from an up-to-date `main`.
2. Set `version` under `[workspace.package]` in `Cargo.toml`.
3. Run `git cliff --tag v<version> -o CHANGELOG.md`.
4. Open a PR titled `chore(release): v<version>` and merge it.
5. Tag the merge commit and push the tag:

   ```
   git checkout main && git pull
   git tag -a v<version> -m "v<version>"
   git push origin v<version>
   ```
