# Roadmap

## Shipped

### 0.2 — DisplayID 1.x support

Full coverage of all 20 DisplayID 1.x block types, including multi-block section
reassembly, checksum verification, and `DisplayIdCapabilities` for the dynamic pipeline.
All timing block formats (Types I–VI, VESA bitmap, CTA bitmap) are supported in both
the dynamic and static pipelines. Panel-specific fields (device data, power sequencing,
stereo interface, tiled topology, transfer characteristics, display interface) are exposed
directly on `DisplayCapabilities`.

### 0.3 — Shared type library (`display-types`)

All output types extracted into the
[`display-types`](https://crates.io/crates/display-types) crate. Types continue to be
re-exported from `piaf`; downstream crates can depend on `display-types` directly to
share types without depending on the parser.

## Planned

### Derived-value helpers

Helpers for computed results (preferred mode selection, bandwidth checks, DPI, HDR and VRR
detection, mode filtering) belong in a separate module as free functions that accept
`&DisplayCapabilities`. They should not be methods on `DisplayCapabilities` itself, which
is a decoded data representation rather than a decision layer.

### Consistency validation

Helpers to detect internally inconsistent EDIDs: modes whose pixel clock exceeds the
declared maximum, refresh rates outside the declared range, conflicting identity fields.
These surface as warnings rather than errors, since the underlying data may still be useful.

### Broader fixture corpus

Expanding the fixture corpus — particularly with edge cases, malformed inputs, and displays
from a wider range of manufacturers — will increase confidence in the normalization layer
and make refactoring safer.
