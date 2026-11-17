# ADR 0001: Walking skeleton and plug-in traits

## Status

Accepted

## Date

2026-11-17

## Context

The project turns Wi-Fi 7 CSI into occupancy tracking, zone triggers and alerts, in Rust, with no cloud dependency. The specification describes five layers. The team has five members, each owning one area.

There is no hardware yet. The planned hardware is an Intel BE200 adapter, and we do not know when it will arrive. The fall-detection model is built by an external collaborator, so the team provides only the integration point for it. The main delivery deadline is 12 March 2027.

We need a structure that lets all five members start work immediately, lets the Bevy app show something end to end early, and lets real hardware and the real model be added later without rewriting the rest.

## Decision

We will build a walking skeleton with a fake at every seam.

- The five layers become crates with typed boundaries. All boundary types live in a shared `cc-core` crate.
- Every boundary gets a fake producer first: simulated CSI, ground-truth occupancy from the simulator scene, and a stub fall classifier. Real implementations replace the fakes one stage at a time.
- Hardware and swappable behaviour sit behind four compile-time traits in `cc-core`: `CsiSource`, `FallClassifier`, `AuxSensor` and `AlertSink`.
- The implementation of each trait is chosen by a local config file. Real drivers live in their own crates behind cargo features.
- We will not load dynamic libraries. A new driver is a new crate that implements one trait.
- Only `apps/viewer` depends on Bevy. All crates under `crates/` are headless.

The target layout is described in `docs/ARCHITECTURE.md`.

## Consequences

- All members can work in parallel from the first week, because each depends only on `cc-core` types and a fake producer.
- The Bevy app can demonstrate a scripted fall end to end before any real algorithm exists, at the end of milestone M1.
- Integration risk is found early, because the stages are connected from the start.
- Adding the Intel BE200 source later means writing one new crate and one cargo feature. If the BE200 tooling emits data that does not fit `CsiFrame`, the `cc-core` contract must change. A spike early in the project checks this.
- `cc-core` becomes a shared dependency of everything, so changes to it need an ADR and review from the Platform & Integration role. Until it is stable, other roles may be blocked on it.
- Results measured on the simulator must be labelled as simulator-verified until hardware is available.

## Alternatives considered

- **Build layer by layer in pipeline order.** Rejected. The dashboard and alerts owners would have nothing to build on for weeks, and integration problems would appear late, close to the deadline.
- **Wait for hardware before building.** Rejected. Nothing would be delivered if the hardware is late or never arrives.
- **Load drivers as dynamic libraries at run time.** Rejected. It adds complexity and unsafe boundaries that the project does not need. Compile-time traits with cargo features are simpler and are checked by the compiler.
