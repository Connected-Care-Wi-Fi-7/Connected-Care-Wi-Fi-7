# AGENTS.md

Instructions for AI coding agents (Claude Code, Codex, Cursor, Copilot and others) working in this repository. Humans should read `CONTRIBUTING.md`; this file summarises it and adds what an agent needs to know. If the two disagree, `CONTRIBUTING.md` wins and this file should be fixed.

Everything an agent needs is in this file and the documents it links. Do not rely on local, uncommitted tool folders (such as `.claude/`) for project rules.

## The project

Connected Care Wi-Fi 7 is a UCL COMP0016 team project with Intel. It is a Rust application that runs entirely on a local machine and turns Wi-Fi 7 Channel State Information (CSI) into occupancy tracking, zone triggers and fall alerts. The user interface is a native Bevy desktop app.

**Current state:** early scaffold. The workspace contains only:

- `apps/viewer` (package `cc-viewer`): a Bevy app that opens a window with a 3D camera.
- `crates/cc-core`: an empty stub for the shared data contracts.

The rest of the crates in `docs/ARCHITECTURE.md` (`cc-sim`, `cc-dsp`, `cc-mapping`, `cc-alerts`, `cc-pipeline`) are planned, not built. Do not assume their types or APIs exist. The backlog lives in GitHub issues and the project board "Connected Care Wi-Fi 7".

## Read before changing anything

| Document | What it tells you |
| --- | --- |
| `SPECIFICATION.md` | Requirements (FR-01 to FR-15) and acceptance targets. When you cite a requirement, give its number and name, for example "FR-03 (spatial triggers)", so a renumbering cannot silently change its meaning |
| `CONTRIBUTING.md` | Workflow, Definition of Done, coding, zero-cloud and privacy rules |
| `docs/ARCHITECTURE.md` | Target crate layout, data flow, plug-in traits, dependency rule |
| `docs/ROLES.md` | Areas, area leads and how reviews are routed |
| `docs/adr/` | Accepted decisions. Do not contradict an accepted ADR |

## Commands

The Rust toolchain is pinned in `rust-toolchain.toml`; `rustup` installs it automatically on first use. Do not change the pinned version without an issue.

```sh
cargo run -p cc-viewer                                  # run the desktop app
cargo fmt --all                                         # format
cargo fmt --all -- --check                              # CI format check
cargo clippy --workspace --all-targets -- -D warnings   # CI lint, warnings are errors
cargo test --workspace                                  # CI tests
```

On Linux, Bevy needs these system packages (the same list CI installs):

```sh
sudo apt-get install -y --no-install-recommends \
  pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

The `[profile.dev]` settings in the root `Cargo.toml` optimise dependencies in debug builds because Bevy is unusably slow without them. Keep them.

The first build compiles Bevy and takes several minutes. That is normal.

## Architecture rules

- Only `apps/viewer` may depend on Bevy. Every crate under `crates/` is headless and unit-testable.
- Crates communicate only through types and traits defined in `cc-core`. A crate must not depend on another stage crate (for example, `cc-mapping` never imports `cc-dsp`); `cc-pipeline` wires them together.
- **`cc-core` contracts are sensitive.** Changing its data types or traits needs an ADR and a review from the Platform & Integration lead. If your task needs such a change, stop and say so instead of making it silently.
- Hardware sits behind one of the four traits (`CsiSource`, `FallClassifier`, `AuxSensor`, `AlertSink`) and a cargo feature that is off by default. Implementations are chosen by a local config file. No dynamic library loading.
- The fall-detection model (FR-02, fall detection classifier) comes from an external collaborator. Only build the `FallClassifier` seam and the stub classifier; do not write ML training code or create model-training tasks.
- New crates go in `crates/`, are named `cc-<name>`, and take `version`, `edition`, `rust-version` and `publish` from the workspace (`version.workspace = true` and so on). Shared dependency versions go in `[workspace.dependencies]`.

## Rust rules

- Put `#![forbid(unsafe_code)]` in every crate except hardware driver crates. In a driver crate, keep `unsafe` blocks minimal and explain each one in a comment.
- Errors: `thiserror` in library crates, `anyhow` in applications.
- No `unwrap()` or `expect()` outside tests. Return errors instead.
- Document every public item with `///`.
- Add or update tests with every change. Tests must run without hardware or network.

## Zero-cloud and privacy rules

These are hard requirements (FR-04 local edge processing, FR-05 privacy-first testing), not preferences.

- No outbound network code: no HTTP clients, telemetry, crash reporting or update checks.
- Any listener binds to `127.0.0.1` only, never `0.0.0.0`.
- Before adding a dependency, check that it does not pull in network clients. Mention every new dependency in the PR description.
- Never commit real CSI captures (`*.pcap`, `*.pcapng`, anything under `/data/`), recordings of real people, names, room numbers or anything else that could identify a person.
- Never commit large data files, `.env` files, secrets, or local tool folders such as `.claude/`.
- Use simulated data by default. A public dataset may only be used after it has an entry in `docs/DATA_PROVENANCE.md`.
- Results measured on the simulator must be described as "simulator-verified", never as "hardware-verified".

## Git and pull request workflow

- **Issue first.** Every change has a GitHub issue. If there is none, ask the user before creating one.
- **Never commit or push to `main`.** Work on a branch named `type/short-name` (types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `ci`), created from an up-to-date `main`.
- **Conventional Commits:** `type(scope): imperative description`, for example `feat(cc-dsp): add Hampel outlier filter`.
- **Pull requests** target `main`, use `.github/pull_request_template.md`, and include `Closes #<issue>`. The link is required: it is how the area lead gets requested as reviewer.
- Keep PRs under about 400 changed lines. Split bigger work into several issues and PRs.
- CI (`.github/workflows/ci.yml`) must be green. PRs are squash-merged.
- Do not commit, push, open PRs, create issues or edit the project board unless the user asked you to.

## AI-assisted work

The full rule is in "AI-assisted work" in `CONTRIBUTING.md`. In short: the human who submits the work is its author. Never act under an AI tool's own GitHub account, do not add `Co-Authored-By` trailers for AI tools to commits, do not add "Generated with ..." footers to pull requests, and never add links to AI tool sessions or chat threads, or hidden attribution comments. A short, plain one-line disclosure in an issue or pull request description is allowed, for example "The fix was handed off to Claude and reviewed by me."

## Reviews

Each issue's area lead is its default reviewer. Labels, the project board's "Lead reviewer" field and PR review requests are set automatically from the `area:` label; see "How review routing works" in `docs/ROLES.md`. The mapping lives in `.github/area-leads.json`. Do not edit that file or `.github/CODEOWNERS` unless the task is about changing leads.

## Before you say a task is done

1. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and report what they actually printed. Do not claim they pass without running them.
2. Check every acceptance criterion in the issue and say which ones are met.
3. If something could only be checked on hardware, say it was not verified.
