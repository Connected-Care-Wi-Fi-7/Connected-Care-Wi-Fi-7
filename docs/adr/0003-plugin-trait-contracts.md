# ADR 0003: Plug-in trait contracts

## Status

Proposed

## Date

2026-11-22

## Context

ADR 0001 defines the plug-in boundaries. Issue #4 specifies their APIs. A CSI
source needs to expose the fixed session metadata and capture timestamps
introduced in ADR 0002 to the pipeline.

## Decision

We will use a synchronous, `dyn`-compatible `CsiSource` trait with a `Send`
bound so the pipeline can move a source to another thread. Implementations
expose the fixed `CsiSession` from construction. Polling returns the next frame,
reports that a frame is temporarily pending, or reports a terminal end of
stream. Polling does not wait for a future frame. Read failures use a concrete
`thiserror` error type that preserves the underlying cause. Replay preserves
recorded capture timestamps.

We will use a synchronous, `dyn`-compatible `FallClassifier` trait with a
`Send` bound. Each update borrows a `ProcessedFrame` and the current slice of
`Track` estimates. The classifier can retain its own rolling window. A
successful update returns a vector of `FallEvent`s, which may be empty or
contain multiple events. Event timestamps are session-relative and track
associations are optional. Failures use a concrete `thiserror` error type.

We will use a synchronous, `dyn`-compatible `AuxSensor` trait with a `Send`
bound. Polling returns a new `AuxReading`, no new reading, or a concrete
`thiserror` error. The provisional reading carries a binary activation state
and an observation time in nanoseconds since the first CSI capture. The fake
uses that timeline directly. Future adapters must align any separate sensor
clock before supplying readings. The reading format can be revisited when
real sensor requirements are known.

## Consequences

The simulator, replay and future hardware drivers can expose the same interface.
The caller decides when to poll again after `Pending`. Future changes to the
plug-in trait contracts require an ADR and Platform & Integration lead review.

## Alternatives considered

A single optional frame would give the caller no way to tell a temporary gap
from the end of a recording.

A single optional fall event would limit each classifier update to one fall.
