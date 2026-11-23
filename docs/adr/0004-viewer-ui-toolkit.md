# ADR 0004: Viewer UI toolkit

## Status

Proposed

## Date

2026-11-23

## Context

The viewer (`apps/viewer`) needs 2D interface elements on top of the 3D scene: a status panel (#26), an alert feed with acknowledge buttons and live telemetry (#30), a zone editor with numeric inputs (#29) and a heatmap view (#31). The alert feed must stay usable with at least 100 entries. The interface is a caregiver-facing tool, so clarity matters more than custom styling. The delivery deadline is 12 March 2027 and the area has one lead.

Two options integrate with Bevy 0.20, the version pinned in the workspace:

- `bevy_ui`, built into Bevy. Interface elements are ECS entities laid out with flexbox.
- `egui` through the `bevy_egui` crate. Version 0.43.1 targets Bevy 0.20. Interface code runs every frame in immediate mode.

## Decision

We will use `egui` through `bevy_egui` for the viewer's 2D interface.

We will enable only the `render`, `default_fonts` and `picking` features. The default `open_url` feature opens links in a web browser and `manage_clipboard` adds clipboard crates; the viewer needs neither, and leaving `open_url` off keeps the app free of anything that reaches outside the machine (FR-04, local edge processing). `picking` lets egui panels block mouse input from reaching the 3D scene, which the zone editor and orbit camera need.

UI code stays in the viewer's `ui` module and reads only bridge resources and messages, so the toolkit can be replaced without touching the bridge or the scene.

## Consequences

- Tables, scroll areas, text and number inputs, sliders and collapsible sections come ready-made, which covers the alert feed and zone editor with little code.
- Immediate-mode UI code is short and easy to review: a panel is one function that reads resources and draws widgets.
- `bevy_egui` is a third-party crate that releases after each Bevy version. Upgrading Bevy depends on a matching `bevy_egui` release, and egui's own API changes between versions (for example, panels changed in egui 0.36).
- The interface has egui's look. Deep visual customisation is harder than with `bevy_ui`.
- egui widgets are not ECS entities, so they cannot be queried or animated like scene objects.

## Alternatives considered

- **`bevy_ui`:** no extra dependency and always matches the Bevy version, but it lacks ready-made text inputs, tables and scroll lists suitable for the alert feed and zone editor, so each would be built by hand. Rejected for the time it would cost, not for quality. It remains the fallback if `bevy_egui` stops tracking Bevy.
- **Web dashboard:** ruled out by FR-08 (caregiver dashboard), which specifies a native desktop app, and by the localhost-only rule.
