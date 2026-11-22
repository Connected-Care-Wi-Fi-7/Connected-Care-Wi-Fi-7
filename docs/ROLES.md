# Roles

Roles are loose. The team has five members and five areas of the system. Each area has a **lead**, not an owner. Everyone works across areas and can pick up any issue.

Being the lead of an area means you:

- are the point of contact for questions about it,
- keep its issues, labels and milestone in order,
- are the default first reviewer for pull requests that touch it,
- look after one cross-cutting duty that helps the whole team.

Being the lead does not mean you do all the work in the area, or only work in that area. Issues are not assigned by area. Anyone takes any open issue by assigning themselves, and people are expected to work outside their own area regularly. If the lead is busy or away, any other team member can review, but merging still needs the lead reviewer's approval (see "How review routing works").

The Team Lead role from `SPECIFICATION.md` is held by Daniel Harrington alongside leading Sensing & Simulation. It is not a sixth role.

| Area | Lead | GitHub | Area label | Covers |
| --- | --- | --- | --- | --- |
| 1. Platform & Integration | Matej Pechoucek | `MatejPechoucek` | `area:platform` | `cc-core`, `cc-pipeline`, CI, classifier seam |
| 2. Sensing & Simulation | Daniel Harrington (Team Lead) | `danielharrington06` | `area:sensing` | `cc-sim`, driver stub, auxiliary sensor hooks |
| 3. Signal Processing | Ben Dalton | `bendalton409` | `area:dsp` | `cc-dsp` |
| 4. Spatial Mapping & Tracking | Daniel Wheeler | `Dwheeler4` | `area:mapping` | `cc-mapping` |
| 5. Dashboard & Alerts | Magizh Umashankar | `Magizh-Shankar` | `area:dashboard`, `area:alerts` | `apps/viewer`, `cc-alerts` |

Issues with `area:project` (spec corrections, licence, ethics, evaluation report) are routed by the Team Lead.

### How review routing works

The lead of an issue's area is its default reviewer. This is automated:

- **Issues.** The issue form's Type, Area and Priority answers become `type:`, `area:` and `priority:` labels. The project board's **Lead reviewer** field is then set to the lead of the issue's `area:` label. Sub-issues are skipped; their parent issue carries the reviewer.
- **Pull requests.** The PR gets the `area:` label of the issue it closes (`Closes #N`), and a review is requested from that area's lead. If the author is the lead, the Team Lead is requested instead. If the author is also the Team Lead, the backup reviewer from `.github/area-leads.json` is requested instead, so someone is always asked to review. The request is repeated on every push, so the lead reviewer is asked to approve the new commit. Once the PR is ready for review it is added to the project board, and its **Lead reviewer** field is set to the same person. `.github/CODEOWNERS` also requests the lead when files in their area change.
- **Merging.** That same person, the PR's lead reviewer, must approve it before it can merge. The required "Lead approval" check (`.github/workflows/lead-approval.yml`) passes only when the lead reviewer of every area on the PR has approved its latest commit, so each new push needs a fresh approval. Anyone else may review and approve too, but that does not replace the lead reviewer's approval.

The area to lead mapping lives in `.github/area-leads.json`. When a lead changes, update that file, `.github/CODEOWNERS`, the table above, and the options of the board's "Lead reviewer" field. If a lead reviewer is away, change the lead in `.github/area-leads.json` through a PR; nobody can bypass the merge rules.

The crates listed here are the planned layout. See `docs/ARCHITECTURE.md`. The "Team goal for M1" lists below are goals for the whole team in that area, not a personal task list for the lead.

## Fall-detection model

The fall-detection model is built by an external collaborator, not by the five team members. The team provides only the integration seam: the `FallClassifier` trait in `cc-core`, a stub threshold classifier to use until the real model exists, and an interface document for the collaborator. The Platform & Integration lead is the contact point for the collaborator.

## 1. Platform & Integration

- **Covers:** `cc-core` (data contracts, traits, config), `cc-pipeline` (stage wiring), CI, and the fall-classifier seam.
- **Area label:** `area:platform`.
- **Requirements:** FR-04 (local edge processing), and FR-02 (fall detection classifier, integration only, because the model itself comes from the external collaborator).
- **Lead looks after:** keep releases and CI healthy, and act as contact point for the external ML developer.
- **Team goal for M1:**
  - `cc-core` contracts and the four plug-in traits are merged and documented.
  - `cc-pipeline` runs the stages over channels in a headless runner and reports per-stage latency.
  - The stub fall classifier and the interface document for the external collaborator exist.
  - CI is green on `main`, including tests.

## 2. Sensing & Simulation

- **Covers:** `cc-sim` (scripted scenes to synthetic CSI, recording and replay), the hardware driver stub, and the auxiliary sensor hooks.
- **Area label:** `area:sensing`.
- **Requirements:** FR-01 (CSI capture pipeline), FR-05 (synthetic and controlled testing with zero PII), FR-12 (sensor fusion hooks).
- **Lead looks after:** keep the dataset provenance register and make sure no PII enters the repository.
- **Team goal for M1:**
  - The simulator produces a stream of `CsiFrame` values for a scripted room scene (walking, sitting, a fall, two occupants).
  - A recording can be saved and replayed deterministically through the same `CsiSource` trait.
  - The spike on what the Intel BE200 CSI tooling actually emits is written up, so `CsiFrame` can be checked against it.

## 3. Signal Processing

- **Covers:** `cc-dsp` (phase sanitisation and unwrapping, Hampel filter, DWT denoising, motion features).
- **Area label:** `area:dsp`.
- **Requirements:** FR-01 (the processing side), and FR-14 (respiration sensing) later if time permits.
- **Lead looks after:** performance benchmarks, producing the numbers needed for FR-06 (evaluation report).
- **Team goal for M1:**
  - Phase sanitisation and unwrapping are implemented and tested.
  - The Hampel outlier filter is implemented and tested.
  - Both run on simulator output and produce `ProcessedFrame` values that the next stage can consume.

## 4. Spatial Mapping & Tracking

- **Covers:** `cc-mapping` (room model, voxel grid, centroids, multi-occupant tracking).
- **Area label:** `area:mapping`.
- **Requirements:** FR-09 (2D spatial mapping), FR-10 (3D spatial mapping), FR-07 (multi-occupant tracking).
- **Lead looks after:** accuracy evaluation against ground truth.
- **Team goal for M1:**
  - The room model and voxel grid are built from the config file.
  - A ground-truth occupancy generator turns the simulator scene into occupancy output, so the dashboard has something to show before real mapping exists.
  - The spike and ADR on which localisation method is feasible are written.

## 5. Dashboard & Alerts

- **Covers:** `apps/viewer` (the Bevy desktop app: 3D view, zone editor, alert panel) and `cc-alerts` (zones, rules, debounce, sinks).
- **Area labels:** `area:dashboard` and `area:alerts`.
- **Requirements:** FR-03 (spatial triggers), FR-11 (false-alarm mitigation), FR-08 (caregiver dashboard).
- **Lead looks after:** demo readiness and partner-facing user experience.
- **Team goal for M1:**
  - The Bevy app has a structure with plugins and a bridge to the pipeline.
  - The 3D room view shows voxels, centroids and tracks from simulated data.
  - `cc-alerts` raises an alert for a zone entry and for a fall event, and writes it to a local log through an `AlertSink`.
  - A scripted fall in the simulator raises an alert that is visible in the app.
