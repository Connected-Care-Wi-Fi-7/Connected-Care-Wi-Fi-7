# Viewer structure

The viewer (`apps/viewer`, package `cc-viewer`) is the Bevy desktop app. It is split into plugins so that the 3D view, the panels and the data feed can be worked on separately.

## Plugins

| Module | Plugin | Responsibility | Extended by |
| --- | --- | --- | --- |
| `bridge.rs` | `BridgePlugin` | Receives pipeline output and exposes it to Bevy | `cc-pipeline` integration (#5) |
| `fake_source.rs` | `FakeSourcePlugin` | Sends a synthetic walking scene until `cc-pipeline` exists | Removed once #5 lands |
| `scene.rs` | `ScenePlugin` | Camera and lighting for the 3D view | 3D room view (#27) |
| `ui.rs` | `UiPlugin` | 2D panels drawn with egui ([ADR 0004](adr/0004-viewer-ui-toolkit.md)) | Alert panel (#30), zone editor (#29), heatmap (#31) |

`main.rs` only creates the window and adds these plugins.

## How data reaches the app

```
pipeline thread                                Bevy app (one pass per rendered frame)
PipelineSender --[state: bounded channel]--+--> drain_pipeline --> LatestOccupancy, LatestTracks  (resources)
               --[alerts: own channel]-----+                   --> AlertReceived                  (message per alert)
                                                               --> BridgeStats                    (received, dropped, connected)
```

- The producer runs on its own thread and calls `PipelineSender::send`, which never blocks. It sends on two lanes:
  - **State** (occupancy and tracks) uses a bounded channel. If it is full, the update is counted in `BridgeStats::dropped` and handed back in `SendError::Full`, so the producer can retry or let it go.
  - **Alerts** use their own unbounded channel, so a busy state lane can never push an alert out.
- `drain_pipeline` runs in `PreUpdate`, takes everything waiting without blocking, and updates the resources and messages above before other systems run.
- Occupancy and tracks keep only the latest value. Every alert is delivered as an `AlertReceived` message: alerts are never dropped by the bridge, and if more arrive than one frame handles, the rest wait for the next frame.
- Other plugins read only these resources and messages. They do not know about threads or channels.

## Adding a feature

- New visuals: add a plugin (or systems in `ScenePlugin`) that reads `LatestOccupancy` or `LatestTracks`.
- New panels: add a system to `UiPlugin` in the `EguiPrimaryContextPass` schedule.
- Reacting to alerts: read `MessageReader<AlertReceived>`.

## Coordinates

The project frame (ADR 0002) is z-up; Bevy is y-up. Converting positions into Bevy space is part of the 3D room view (#27).

## Swapping in the real pipeline

When `cc-pipeline` exists, replace `FakeSourcePlugin` with a plugin that starts the pipeline and gives it a `PipelineSender` from `bridge::channel()`. Nothing else in the viewer needs to change.
