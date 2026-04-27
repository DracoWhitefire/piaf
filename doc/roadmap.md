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

### 0.4 — DisplayID 2.x support

Full coverage of the DisplayID 2.x tag space: Product Identification (`0x20`), Display
Parameters (`0x21`), Type VII/VIII/IX timings (`0x22`–`0x24`), Dynamic Video Timing
Range (`0x25`), Display Interface Features (`0x26`), Stereo Display Interface (`0x27`),
Tiled Display Topology (`0x28`), ContainerID (`0x29`), Vendor-Specific (`0x7E`), and
the CTA DisplayID block (`0x81`) which merges its CTA-861 payload into the existing
`Cea861Capabilities` regardless of processing order. Timing blocks (`0x22`–`0x24`,
`0x81`) decode in both the dynamic and static pipelines; metadata blocks are
dynamic-only.

## Planned

### DisplayID 2.x follow-ups

Several 2.x fields are intentionally not surfaced in the 0.4 release:

- **Type IX (`0x24`) byte 0 options** — partial. CVT algorithm selector and Y420 flag
  are reified onto `VideoMode` as `cvt_algorithm` and `y420`; CVT-RB v1 (VESA CVT 1.1
  §3.4) and CVT-RB v2 (VESA CVT 1.2 §4) are fully evaluated to populate
  `pixel_clock_khz` and blanking parameters via `display_types::compute_type_ix_timing`.
  Still pending: CVT-RB v3 and the "reduced blanking with CVT-RB1/RB2" variants
  (encodings 2–4) — descriptors using these algorithms currently get only the
  metadata, not the derived timing. Stereo bits (6:5) also still dropped — need to
  confirm Type IX's stereo encoding (likely distinct from the DTD `StereoMode` codes)
  before mapping.
- **Stereo Display Interface (`0x27`) inline timing-code list** — when the revision
  byte's timing scope indicates per-method codes, the descriptor is followed by a list
  of DMT/VIC/HDMI-VIC code records that scope the stereo configuration to specific
  timings. Currently ignored; needs a new field on `DisplayIdStereoInterfaceV2`.
- **Display Interface Features (`0x26`) bytes 7–8** — custom color space + EOTF
  combinations and the additional-bytes count are dropped. Needed once consumers care
  about HDR10+/non-standard EOTFs beyond the defined-combinations bitmask.
- **ContainerID (`0x29`) typed UUID wrapper** — the 16 bytes are exposed as `[u8; 16]`
  so callers can interpret as either Microsoft mixed-endian GUID or RFC 4122 UUID. A
  typed wrapper (likely behind a feature flag depending on `uuid` crate) would be more
  ergonomic.

### Consistency validation

Helpers to detect internally inconsistent EDIDs: modes whose pixel clock exceeds the
declared maximum, refresh rates outside the declared range, conflicting identity fields.
These surface as warnings rather than errors, since the underlying data may still be useful.

### Broader fixture corpus

Expanding the fixture corpus — particularly with edge cases, malformed inputs, and displays
from a wider range of manufacturers — will increase confidence in the normalization layer
and make refactoring safer.
