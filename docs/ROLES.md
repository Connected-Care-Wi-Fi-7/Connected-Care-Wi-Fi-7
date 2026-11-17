# Roles

Roles are loose. The team has five members and five areas of the system. Each area has a **lead**, not an owner. Everyone works across areas and can pick up any issue.

Being the lead of an area means you:

- are the point of contact for questions about it,
- keep its issues, labels and milestone in order,
- are the default first reviewer for pull requests that touch it,
- look after one cross-cutting duty that helps the whole team.

Being the lead does not mean you do all the work in the area, or only work in that area. Issues are not assigned by area. Anyone takes any open issue by assigning themselves, and people are expected to work outside their own area regularly. If the lead is busy or away, any other team member can review.

The Team Lead role from `SPECIFICATION.md` is held alongside one of these. It is not a sixth role.

Names are not assigned yet. The "Lead" column stays `TBD` until the team decides.

| Area | Lead | Area label | Covers |
| --- | --- | --- | --- |
| 1. Platform & Integration | TBD | `area:platform` | `cc-core`, `cc-pipeline`, CI, classifier seam |
| 2. Sensing & Simulation | TBD | `area:sensing` | `cc-sim`, driver stub, auxiliary sensor hooks |
| 3. Signal Processing | TBD | `area:dsp` | `cc-dsp` |
| 4. Spatial Mapping & Tracking | TBD | `area:mapping` | `cc-mapping` |
| 5. Dashboard & Alerts | TBD | `area:dashboard`, `area:alerts` | `apps/viewer`, `cc-alerts` |

Issues with `area:project` (spec corrections, licence, ethics, evaluation report) are routed by the Team Lead.

The crates listed here are the planned layout. See `docs/ARCHITECTURE.md`. The "Team goal for M1" lists below are goals for the whole team in that area, not a personal task list for the lead.

## Fall-detection model

The fall-detection model is built by an external collaborator, not by the five team members. The team provides only the integration seam: the `FallClassifier` trait in `cc-core`, a stub threshold classifier to use until the real model exists, and an interface document for the collaborator. The Platform & Integration lead is the contact point for the collaborator.

## 1. Platform & Integration

- **Covers:** `cc-core` (data contracts, traits, config), `cc-pipeline` (stage wiring), CI, and the fall-classifier seam.
- **Area label:** `area:platform`.
- **Requirements:** FR-05 (local edge processing), and FR-03 (integration only, because the model itself comes from the external collaborator).
- **Lead looks after:** keep releases and CI healthy, and act as contact point for the external ML developer.
- **Team goal for M1:**
  - `cc-core` contracts and the four plug-in traits are merged and documented.
  - `cc-pipeline` runs the stages over channels in a headless runner and reports per-stage latency.
  - The stub fall classifier and the interface document for the external collaborator exist.
  - CI is green on `main`, including tests.

## 2. Sensing & Simulation

- **Covers:** `cc-sim` (scripted scenes to synthetic CSI, recording and replay), the hardware driver stub, and the auxiliary sensor hooks.
- **Area label:** `area:sensing`.
- **Requirements:** FR-01 (CSI capture pipeline), FR-06 (synthetic and controlled testing with zero PII), FR-10 (sensor fusion hooks).
- **Lead looks after:** keep the dataset provenance register and make sure no PII enters the repository.
- **Team goal for M1:**
  - The simulator produces a stream of `CsiFrame` values for a scripted room scene (walking, sitting, a fall, two occupants).
  - A recording can be saved and replayed deterministically through the same `CsiSource` trait.
  - The spike on what the Intel BE200 CSI tooling actually emits is written up, so `CsiFrame` can be checked against it.

## 3. Signal Processing

- **Covers:** `cc-dsp` (phase sanitisation and unwrapping, Hampel filter, DWT denoising, motion features).
- **Area label:** `area:dsp`.
- **Requirements:** FR-01 (the processing side), and FR-13 later if time permits.
- **Lead looks after:** performance benchmarks, producing the numbers needed for FR-07.
- **Team goal for M1:**
  - Phase sanitisation and unwrapping are implemented and tested.
  - The Hampel outlier filter is implemented and tested.
  - Both run on simulator output and produce `ProcessedFrame` values that the next stage can consume.

## 4. Spatial Mapping & Tracking

- **Covers:** `cc-mapping` (room model, voxel grid, centroids, multi-occupant tracking).
- **Area label:** `area:mapping`.
- **Requirements:** FR-02 (3D spatial coordinate mapping), FR-08 (multi-occupant tracking).
- **Lead looks after:** accuracy evaluation against ground truth.
- **Team goal for M1:**
  - The room model and voxel grid are built from the config file.
  - A ground-truth occupancy generator turns the simulator scene into occupancy output, so the dashboard has something to show before real mapping exists.
  - The spike and ADR on which localisation method is feasible are written.

## 5. Dashboard & Alerts

- **Covers:** `apps/viewer` (the Bevy desktop app: 3D view, zone editor, alert panel) and `cc-alerts` (zones, rules, debounce, sinks).
- **Area labels:** `area:dashboard` and `area:alerts`.
- **Requirements:** FR-04 (spatial triggers), FR-09 (false-alarm mitigation), FR-11 (caregiver dashboard).
- **Lead looks after:** demo readiness and partner-facing user experience.
- **Team goal for M1:**
  - The Bevy app has a structure with plugins and a bridge to the pipeline.
  - The 3D room view shows voxels, centroids and tracks from simulated data.
  - `cc-alerts` raises an alert for a zone entry and for a fall event, and writes it to a local log through an `AlertSink`.
  - A scripted fall in the simulator raises an alert that is visible in the app.

## A note on FR-11

`SPECIFICATION.md` describes FR-11 as a "desktop web dashboard". The team has decided the dashboard is part of the native Bevy desktop app. Nothing is hosted beyond localhost. The spec wording will be corrected in a separate issue.
