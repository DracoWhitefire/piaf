# Scope

This document describes the intended scope of PIAF and, just as importantly, what is out of scope for early versions.

## Current scope

The first phase of PIAF focuses on reading and interpreting EDID data as an input to capability discovery.

Specifically, early work should cover:

- raw byte input,
- validation of basic structure,
- checksum verification,
- parsing of the EDID base block,
- conservative handling of extension blocks,
- conversion into a typed capability model.

## Out of scope for early versions

The following are intentionally not part of the first milestone:

- a full HDMI implementation,
- packet generation or serialization,
- driver development,
- electrical or PHY-level signaling,
- vendor-specific behavior beyond safe parsing,
- broad platform integration.

## Why start with EDID

EDID is a good foundational module because it is:

- self-contained,
- binary and testable,
- directly useful to other layers,
- a natural source for a display capability model.

A successful EDID parser creates a stable base for later work on metadata handling, packet construction, and related modules.

## Evolution strategy

PIAF should evolve in small, independent steps:

1. parse and validate,
2. expose a structured intermediate model,
3. normalize into `DisplayCapabilities`,
4. improve robustness with fixture-based testing,
5. extend support conservatively.

All development must maintain `no_std` compatibility for core modules. Optional features such as `serde` support and diagnostic pretty-printing can be enabled via crate features.
