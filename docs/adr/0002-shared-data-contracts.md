# ADR 0002: Shared data contracts and conventions

## Status

Proposed

## Date

2026-11-19

## Context

Components need shared data types and consistent units, coordinates, timestamps and CSI indexing. ADR 0001 places boundary types in `cc-core`. Hardware output and the classifier's inputs are not yet confirmed.

## Decision

We will define the six shared types in `cc-core`, independent of Bevy and hardware. The following conventions are proposed for review:

- **Coordinates:** metres in a right-handed Cartesian frame. The configured origin is a fixed floor corner of the mapped area; positive x follows one chosen floor edge, positive y follows the perpendicular floor edge with x cross y pointing upwards, and positive z is up. The deployment records the physical corner and orientation. Multiple rooms use this common frame or are transformed into it.
- **Bounds and cells:** the lower boundary is included and the far boundary excluded on each axis. Grid cells follow the same lower-inclusive, upper-exclusive rule. Where an area dimension is not divisible by the voxel size, include a partial cell at the far edge; positions outside the mapped area remain outside even if the nominal cell extends beyond it.
- **Height:** an unknown centroid height is represented separately from zero; the draft uses `Option<f64>`.
- **Occupancy:** the frame carries grid origin, exact physical extent, dimensions and voxel size. Cells are stored x-fastest, then y, then z, using unknown, empty and occupied states. Values in a partial edge cell represent only its intersection with the mapped area. Grid construction and validation are handled in #20.
- **Time:** all six types use unsigned 64-bit nanoseconds relative to the first CSI capture of the active session, which defines zero. CSI and processed frames use capture time; occupancy and tracks use estimate-valid time; fall events use estimated event time; alerts use creation time. Live event and alert times use the same monotonic session clock; replay preserves capture offsets and uses a replay timeline, not the host wall clock. Values are not UTC and cannot align separate sessions or other sensor clocks without an explicit conversion. Pre-origin data cannot be represented.
- **CSI:** one program run uses one session, initialised by Sensing, with a fixed layout and radio configuration. Tensors are ordered `[tx_stream, rx_chain, subcarrier]`, with subcarrier varying fastest. Stream and chain indices are zero-based, stable for the session and assigned by the source adapter; actual signed OFDM tone indices are strictly increasing and may have gaps. The centre frequency and tone spacing give the frequency corresponding to each index. Missing values are distinct from zero; tensor length is checked against the session layout, including after deserialization.
- **Replay:** a recording must retain the session configuration alongside its frames. A layout change requires a new program session; the recording and replay implementations are outside this ADR.
- **Processing:** processed samples retain denoised amplitude, unwrapped phase and background-subtracted amplitude, plus optional variance and motion energy features. Their exact feature definitions remain open.
- **Tracks and events:** a track ID is temporary, not a person's identity. Tracks carry their latest estimate; a fall or alert may have no track ID.

## Open questions

- Does BE200 tooling provide the proposed data and metadata, or must Sensing adapt them (#8)? Hardware compatibility is not established by this ADR.
- Does the localisation method (#22) have enough information in `ProcessedFrame`, and are three occupancy states sufficient for its outputs and consumers?
- What signal, window, formula and units define variance and motion energy (#18)? For a window, the processed timestamp denotes the latest capture represented; whether a start time is also needed remains for DSP and Mapping to establish.
- Can the classifier provide the proposed fall event time and optional track association? Confirm with the collaborator.
- How should clocks from multiple sessions, auxiliary sensors or external systems be aligned if later supported? This single-session contract does not specify that conversion.

## Consequences

- Shared types give components a common vocabulary and can be tested without hardware. The draft does not establish real-world sensing accuracy.
- Shared-contract changes require Platform & Integration review under ADR 0001. All five area leads review issue #3.
- Producers must use the session timeline and check deserialized tensor lengths against the session layout. Grid construction and validation are addressed in #20. Review the simulation-first conventions and track the open hardware, algorithm and integration questions before acceptance.

## Alternatives considered

- **Separate types per component:** requires conversions and risks inconsistent interpretations at boundaries.
- **Zero for unknown height:** ambiguous because zero is a valid height.
- **Continuous occupancy or wall-clock timestamps:** both remain candidates; neither choice has enough evidence to settle yet.
