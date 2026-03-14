# Scope

This document describes the intended scope of PIAF and, just as importantly, what is out of scope for early versions.

## Current scope

PIAF reads and interprets EDID data as an input to display capability discovery.

The library covers:

- parsing and validation of raw byte slices — header verification, checksum, and block structure,
- full decoding of the EDID base block into typed fields,
- extension block dispatch via a pluggable handler system,
- full CEA-861 extension decoding covering all major data block types,
- conversion into a stable `DisplayCapabilities` consumer model,
- structured diagnostics: hard errors for structurally invalid input; warnings for malformed, unknown, or suspicious content.

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

## Constraints

All development must maintain `no_std` compatibility for core modules. Optional features such as `serde` support and diagnostic pretty-printing can be enabled via crate features.
