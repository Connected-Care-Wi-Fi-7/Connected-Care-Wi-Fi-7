# ADR 0002: Shared data contracts and conventions

## Status

Proposed

## Date

2026-11-19

## Context

Components need shared data types and consistent units, coordinates,
timestamps and CSI indexing. ADR 0001 places boundary types in `cc-core`.
Hardware output and the classifier's inputs are not yet confirmed.

## Decision

We will define the six shared types in `cc-core`, independent of Bevy and
hardware. The following conventions are proposed for review:

- **Coordinates:** metres in a right-handed frame, with x and y along
  perpendicular floor directions and z upwards. The origin is a fixed,
  documented floor corner of the mapped area. Components using other frames
  must transform their positions into this frame.
- **Bounds and cells:** the lower boundary is included and the far boundary
  excluded on each axis. Grid cells follow the same lower-inclusive,
  upper-exclusive rule. Where an area dimension is not divisible by the voxel
  size, include a partial cell at the far edge; positions outside the mapped
  area remain outside even if the nominal cell extends beyond it.
- **Height:** an unknown centroid height is represented separately from zero;
  the draft uses `Option<f64>`.
- **Occupancy:** the frame carries grid origin, dimensions and voxel size. The
  draft stores cells x-fastest, then y, then z, using unknown, empty and
  occupied states.
- **Time:** timestamps distinguish capture, estimate-valid, fall-event and
  alert-creation times. The draft uses `u64` fields pending a unit and epoch.
- **CSI:** each draft complex measurement carries transmit-antenna,
  receive-antenna and subcarrier indices. Numbering is pending review.
- **Tracks and events:** a track ID is temporary, not a person's identity.
  Tracks carry their latest estimate; a fall or alert may have no track ID.

## Open questions

- Can simulation, mapping and the viewer use the proposed origin, axis
  directions, boundary rules and partial cells? How will each deployment
  choose the physical origin and share a frame across rooms?
- Can the mapping method produce the proposed three occupancy states, or do
  its consumers need continuous values or uncertainty information?
- What timestamp unit, epoch and clock conversions will simulation, replay and
  hardware use? Which instant or interval represents a processing window?
- Does the BE200 tooling investigated in #8 provide complex measurements and
  transmit/receive antenna and subcarrier indices as drafted? What indexing,
  units and additional metadata does it actually supply?
- Can Signal Processing produce the drafted `ProcessedFrame`, and does it
  retain the information needed by the chosen localisation method?
- Can the classifier supply a track association and event time, and do its
  output and the alert fields meet the collaborator's and dashboard's needs?

## Consequences

- Shared types give components a common vocabulary and can be tested without
  hardware. The draft does not establish real-world sensing accuracy.
- Shared-contract changes require Platform & Integration review under ADR
  0001. All five area leads review issue #3.
- Resolve the open questions and update code and tests before acceptance.

## Alternatives considered

- **Separate types per component:** requires conversions and risks inconsistent
  interpretations at boundaries.
- **Zero for unknown height:** ambiguous because zero is a valid height.
- **Continuous occupancy or wall-clock timestamps:** both remain candidates;
  neither choice has enough evidence to settle yet.
