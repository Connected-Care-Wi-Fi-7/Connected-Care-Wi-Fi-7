# ADR 0002: Shared data contracts and conventions

## Status

Proposed

## Date

2026-11-19

## Context

Components need shared data types and consistent units, coordinates, timestamps and CSI indexing. ADR 0001 places boundary types in `cc-core`. Hardware output and the classifier's inputs are not yet confirmed.

## Decision

We will define the six shared types in `cc-core`, independent of Bevy and hardware. The following conventions are proposed for review:

- **Coordinates:** metres in a right-handed frame, with x and y along perpendicular floor directions and z upwards. The origin is a fixed, documented floor corner of the mapped area. Components using other frames must transform their positions into this frame.
- **Bounds and cells:** the lower boundary is included and the far boundary excluded on each axis. Grid cells follow the same lower-inclusive, upper-exclusive rule. Where an area dimension is not divisible by the voxel size, include a partial cell at the far edge; positions outside the mapped area remain outside even if the nominal cell extends beyond it.
- **Height:** an unknown centroid height is represented separately from zero; the draft uses `Option<f64>`.
- **Occupancy:** the frame carries grid origin, dimensions and voxel size. The draft stores cells x-fastest, then y, then z, using unknown, empty and occupied states.
- **Time:** CSI and processed frames carry capture time in nanoseconds since the session time origin, preserved in replay. Occupancy, track, fall-event and alert timestamps use `u64` pending an agreed unit and epoch.
- **CSI:** one program run uses one session, initialised by Sensing, with a fixed layout and radio configuration. Tensors are ordered `[tx_stream, rx_chain, subcarrier]`, with subcarrier varying fastest and actual tone indices recorded in increasing frequency order. Unavailable values are represented separately from zero.
- **Replay:** a recording must retain the session configuration alongside its frames. A layout change requires a new program session; the recording and replay implementations are outside this ADR.
- **Processing:** processed samples retain denoised amplitude, unwrapped phase and background-subtracted amplitude, plus optional variance and motion energy features. Their exact feature definitions remain open.
- **Tracks and events:** a track ID is temporary, not a person's identity. Tracks carry their latest estimate; a fall or alert may have no track ID.

## Open questions

- Can simulation, mapping and the viewer use the proposed origin, axis directions, boundary rules and partial cells? How will each deployment choose the physical origin and share a frame across rooms?
- Can the mapping method produce the proposed three occupancy states, or do its consumers need continuous values or uncertainty information?
- What establishes the session time origin, and how are session-relative timestamps related to other timestamps and clocks? Which instant or interval represents a processing window?
- Does the BE200 tooling investigated in #8 provide the proposed complex tensor, stable stream/chain indexing, tone indices and radio metadata?
- Can Signal Processing produce the drafted `ProcessedFrame`, and does it retain the information needed by the chosen localisation method?
- What input signal, window, formula and units define variance and motion energy for Mapping?
- How will the pipeline verify tensor lengths against the active session, including after deserialization, and reject mismatches?
- Can the classifier supply a track association and event time, and do its output and the alert fields meet the collaborator's and dashboard's needs?

## Consequences

- Shared types give components a common vocabulary and can be tested without hardware. The draft does not establish real-world sensing accuracy.
- Shared-contract changes require Platform & Integration review under ADR 0001. All five area leads review issue #3.
- Resolve the open questions and update code and tests before acceptance.

## Alternatives considered

- **Separate types per component:** requires conversions and risks inconsistent interpretations at boundaries.
- **Zero for unknown height:** ambiguous because zero is a valid height.
- **Continuous occupancy or wall-clock timestamps:** both remain candidates; neither choice has enough evidence to settle yet.
