# Scope

This document describes the intended scope of PIAF and, just as importantly, what is out of scope for early versions.

## Decoding philosophy

PIAF decodes everything a specification defines. No field is omitted because it appears
obscure or unlikely to be needed. Consumers decide which fields matter for their use case;
the library makes no judgement about importance.

`Option` fields communicate *presence or absence* — whether the source data contained a
value — not whether that value is considered significant. A field that is `None` was absent
or undecodable in the source; a field that is `Some` reflects what the display reported,
regardless of whether any particular consumer cares about it.

This principle applies to new block types as they are implemented: once a block's wire
format is specified, all of its defined fields are decoded into the model.

## Current scope

PIAF reads and interprets EDID data as an input to display capability discovery.

The library covers:

- parsing and validation of raw byte slices — header verification, checksum, and block structure,
- full decoding of the EDID base block into typed fields,
- extension block dispatch via a pluggable handler system,
- full CEA-861 extension decoding covering all major data block types,
- full DisplayID 1.x extension decoding covering all 20 defined block types,
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

In bare `no_std` builds (no `alloc`, no `std`) the extension handler pipeline is
unavailable, but base block decoding runs in full. All fixed-length fields in
`DisplayCapabilities` are available, including identity, input type, color, timing
range limits, and the fixed-count string and white-point fields. Variable-length
fields (`supported_modes`, `extension_data`) require `alloc` or `std`.

When a field has a known fixed maximum — such as a three-character PNP ID or a
13-byte monitor descriptor string — it is represented as a fixed-size array newtype
(`ManufacturerId`, `MonitorString`) rather than a heap-allocated string. This keeps
the field available in all build configurations without sacrificing API ergonomics.
New fields should follow this pattern where the bound is derivable from the spec.
