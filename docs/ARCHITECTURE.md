# Architecture

**Status: target design.** This document describes where the code is going. The crates listed here do not exist yet. Names and boundaries may change through ADRs in `docs/adr/`. Exact types and function signatures are decided in the `cc-core` issues, not here.

## Overview

Connected Care Wi-Fi 7 is a Rust application that runs entirely on a local machine. It turns Wi-Fi 7 Channel State Information (CSI) into occupancy tracking, zone triggers and alerts. The caregiver dashboard is part of a native Bevy desktop app. Nothing is hosted beyond localhost.

The five layers in `SPECIFICATION.md` become crates with typed boundaries. All the boundary types live in one shared crate, `cc-core`.

There is no hardware yet. Everything is built against a simulator, and hardware is added later behind traits.

## Target crate layout

```
crates/cc-core      data contracts, traits, config, units and coordinate conventions
crates/cc-sim       scripted scene to synthetic CSI, recording and replay
crates/cc-dsp       phase unwrapping, Hampel filter, DWT, motion features
crates/cc-mapping   voxel grid, centroids, multi-occupant tracking
crates/cc-alerts    zones, rules, debounce, sinks
crates/cc-pipeline  wires stages with channels, per-stage latency metrics (headless)
apps/viewer         Bevy app: 3D view, zone editor, alert panel
```

Today `crates/cc-core` and `apps/viewer` exist. The viewer's internal structure is described in [VIEWER.md](VIEWER.md).

## Data flow

Data moves through the pipeline in one direction. Each stage consumes one type from `cc-core` and produces the next.

```
 CsiSource          cc-dsp            cc-mapping         FallClassifier       cc-alerts
 (sim, replay,  --> filters and   --> voxels, centroids, --> fall events  -->  zone and fall
  later hardware)   motion features    tracks                                  rules
      |                  |                  |                    |                |
   CsiFrame      ProcessedFrame      OccupancyFrame / Track   FallEvent         Alert
                                            |                                     |
                                            +-------------> apps/viewer <---------+
                                                           (3D view, alert panel)
                                                                                  |
 AuxSensor (simulated PIR, pressure pad) --> feeds verification in cc-alerts   AlertSink
```

`cc-pipeline` connects these stages with channels and measures how long each stage takes. It is headless, so the whole pipeline can run in tests without a window.

The data types named above (`CsiFrame`, `ProcessedFrame`, `OccupancyFrame`, `Track`, `FallEvent`, `Alert`) are defined in `cc-core`. Their fields, units, timestamps and coordinate frame are decided in the `cc-core` issues and recorded in an ADR.

## Plug-in traits

Four traits in `cc-core` define where the pipeline can swap implementations. See [ADR 0003](adr/0003-plugin-trait-contracts.md) for their contracts.

| Trait | Responsibility | Planned implementations | Later integrations |
| --- | --- | --- | --- |
| `CsiSource` | Supplies a stream of CSI frames to the pipeline | Simulator, file replay | A real driver crate such as `cc-driver-be200` |
| `FallClassifier` | Decides whether a fall has occurred from tracking and motion data | A stub threshold detector | The model from the external collaborator |
| `AuxSensor` | Supplies readings from complementary sensors to cross-check Wi-Fi detections (FR-12, sensor fusion hooks) | Simulated PIR and pressure pad | Real sensors |
| `AlertSink` | Delivers an alert somewhere | Local log and in-app alert panel | MQTT, nurse-call or other integrations |

Implementations are compiled into the caller, with real hardware drivers behind Cargo features that are off by default. The caller selects among available implementations at startup using a local config file.

### Local plug-in configuration

[The example TOML](../config/plugins.example.toml) has four required, case-sensitive keys. Each currently accepts `"fake"`, which selects the corresponding in-memory test implementation.

| Key | Trait |
| --- | --- |
| `csi_source` | `CsiSource` |
| `fall_classifier` | `FallClassifier` |
| `aux_sensor` | `AuxSensor` |
| `alert_sink` | `AlertSink` |

Pass the file path explicitly to `cc_core::PluginConfig::load`. The loader validates the names and returns typed selections. An unknown name produces an error identifying the key and rejected value. The caller matches each selection to a constructor. The integration test performs that step with the in-memory fakes, and `cc-pipeline` will own the production constructors.

Run the example through the integration test from the repository root:

```sh
cargo test -p cc-core --test plugin_config
```

## The dependency rule

- Only `apps/viewer` depends on Bevy. Everything under `crates/` is headless and unit-testable.
- Crates communicate only through types and traits in `cc-core`. For example, `cc-mapping` never imports `cc-dsp`. Both depend on `cc-core`, and `cc-pipeline` connects them.
- Hardware appears only behind a trait and a cargo feature.

`cc-core` is therefore the most sensitive crate. A change to its contracts needs an ADR and a review from the Platform & Integration lead.

## Fake at every seam (walking skeleton)

Each boundary gets a fake producer first:

- Simulated CSI stands in for real capture.
- A ground-truth occupancy generator, taken from the simulator scene, stands in for real mapping.
- A stub threshold classifier stands in for the fall model.

With these, every team member can work in parallel from the start, and the Bevy app can show a complete scene before any real algorithm exists. Real implementations then replace the fakes one stage at a time, and each replacement can be checked against the simulator's ground truth. The reasoning is recorded in `docs/adr/0001-walking-skeleton-and-plugin-traits.md`.

## Adding a real hardware driver later

1. Create a new crate, for example `crates/cc-driver-be200`.
2. Implement the `CsiSource` trait in it. Driver crates are the only place `unsafe` code may be allowed.
3. Add a cargo feature that enables the crate, off by default.
4. Add the driver's name to `cc-core`'s CSI source selection and map that selection to its constructor in `cc-pipeline` under the same Cargo feature.
5. Select the driver by name in the local config file.

Changes to a `cc-core` contract, including a change to `CsiFrame`, require an ADR and review from the Platform & Integration lead.

The same steps apply to a real `AuxSensor` or `AlertSink`.

## Constraints from the specification

- Everything runs locally (FR-04, local edge processing). No outbound network code. Any listener binds to `127.0.0.1` only.
- No personally identifiable information in the repository or in test data (FR-05, privacy-first testing).
- Acceptance targets in the specification (250 ms alert latency, 35 cm centroid error) can only be measured on simulated data until hardware is available. Reports must say which results are simulator-verified.
