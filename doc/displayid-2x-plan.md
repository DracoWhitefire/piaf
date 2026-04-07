# DisplayID 2.x Implementation Plan

## Architecture decisions

**Dispatch strategy**: Thread a `version: u8` parameter through `process_data_blocks` and
`scan_all_metadata_blocks`. Inside both, dispatch on `(is_v2, tag)` with a simple
`bool is_v2 = version == DISPLAYID_V2`. No function pointer tables, no boxing — just match arms.

**Type placement**: Extend `DisplayIdCapabilities` in `display-types` with new `Option` fields
(it's already `#[non_exhaustive]`, so additions are additive). No second struct keyed under a
separate tag — everything lives under `0x70`. Requires a `display-types 0.3.0` publish before
piaf can land the full feature.

**f16 luminance**: Add a `pub(crate) fn f16_to_f32(bits: u16) -> f32` inline in `metadata.rs`.
No external crate. A raw value of `0x8000` (negative zero, spec-defined as "not specified") maps
to `None`. Store as `Option<f32>`.

**OUI vs. PNP**: Block 0x20 uses a 3-byte IEEE OUI, not a 2-byte PNP ID. Do **not** populate
`caps.manufacturer` from it. Store raw `[u8; 3]` in
`DisplayIdCapabilities.manufacturer_oui: Option<[u8; 3]>`.

**Shared Type VII decoder**: Extract `decode_type_vii_descriptor(d: &[u8; 20], sink: &mut dyn ModeSink)`
into `timing/detailed.rs`. The CEA-861 ext-tag 0x22 handler already decodes one Type VII
descriptor — have it delegate to this shared function. The 2.x block 0x22 loops over N×20 strides
calling the same function.

---

## `display-types 0.3.0` changes (prerequisite)

New fields on `DisplayIdCapabilities`:

| Field | Type | Source block |
|-------|------|-------------|
| `manufacturer_oui` | `Option<[u8; 3]>` | 0x20 — IEEE OUI |
| `display_params_v2` | `Option<DisplayParamsV2>` | 0x21 — chromaticity, luminance, gamma |
| `dynamic_timing_range` | `Option<DynamicTimingRange>` | 0x25 — kHz-precision clocks, VRR flag |
| `interface_features` | `Option<DisplayInterfaceFeatures>` | 0x26 — per-encoding color depth bitmasks |
| `container_id` | `Option<[u8; 16]>` | 0x29 — UUID |

New tag constants in `tag.rs`: `V2_PRODUCT_ID = 0x20` through `V2_CONTAINER_ID = 0x29`,
`V2_VENDOR_SPECIFIC = 0x7E`, `V2_CTA_DISPLAYID = 0x81`.

All new types are `no_std`-compatible scalar structs. All existing code compiles without change.

**Not covered by this prerequisite**: accurate fractional refresh rates. `VideoMode::refresh_rate`
is `u16`, which cannot represent 23.976, 29.97, or 59.94 Hz. This affects `T7VtdbBlock`,
`T10VtdbEntry`, and any mode derived from a Type VII descriptor. Fixing it requires a
breaking change to `display-types` (separate semver bump) and must be coordinated before
the AVI InfoFrame layer is built — VIC selection is sensitive to the distinction.

---

## Phases

### Phase 1 — Version threading (no new decoders)

- Add `version: u8` parameter to `process_data_blocks` and `scan_all_metadata_blocks`
- Pass `version` from the fragment loop in both pipeline entry points in `mod.rs`
- Add `IMPLEMENTED_V2_BLOCK_TAGS` and `DEFERRED_V2_TAG_RANGES` test constants (initially empty / full)
- Add `test_all_v2_block_tags_accounted_for` alongside the existing 1.x coverage test
- All existing tests must pass unchanged

### Phase 2 — Timing blocks (both pipelines): 0x22, 0x23, 0x24

- **0x22 Type VII**: `decode_type_vii_descriptor` in `timing/detailed.rs`. Pixel clock is
  3-byte LE in **kHz** (not 10 kHz steps). H/V sync polarity encoded as bit 15 of the
  front-porch fields. Update CEA-861 ext-0x22 handler to call this shared function.
  **Fractional refresh rate caveat**: the exact rate is derivable as
  `pixel_clock_hz / (h_total × v_total)`, but `VideoMode::refresh_rate` is `u16` and
  cannot represent 23.976, 29.97, 59.94 Hz — they truncate to 23, 29, 59. The same
  truncation affects `T7VtdbBlock` (which wraps `VideoMode`) and `T10VtdbEntry::refresh_hz`.
  This matters downstream: the AVI InfoFrame layer selects VICs by refresh rate, and
  fractional-rate modes use distinct VICs (e.g. VIC 96 vs VIC 97 for 3840×2160@24 vs
  @23.976). A `display-types` type change — e.g. millihertz as `u32`, or a rational — is
  required before Type VII modes can be surfaced accurately. Until then, decoded modes with
  fractional rates will carry truncated values and the shared decoder must document this
  limitation clearly.
- **0x23 Type VIII**: `decode_type_viii_block(payload, revision)` in `timing/coded.rs`.
  Revision byte bit 3 = two-byte code flag; bits 7:6 = code type (DMT/VIC/HDMI VIC).
  Reuses existing DMT and VIC lookup tables.
- **0x24 Type IX**: `decode_type_ix_descriptor(d: &[u8; 6])` in `timing/short.rs`. Mirrors
  Type V but 6-byte descriptors (no trailing reserved byte). CVT algorithm code differs from
  Type V but doesn't affect `VideoMode` output fields.

All three work in the static pipeline exactly like their 1.x counterparts.

### Phase 3 — Product identity and display parameters: 0x20, 0x21 (dynamic only)

- **0x20**: `decode_v2_product_id_block` — OUI stored in `DisplayIdCapabilities.manufacturer_oui`;
  product name → `caps.display_name`; year = `payload[10] + 2000` (not `+1990`).
- **0x21**: `decode_v2_display_params_block` — decode 12-bit chromaticity, f16 luminance (3 values:
  max full/10%, min), color depth, display technology, gamma. Store in
  `DisplayIdCapabilities.display_params_v2`. Also populate `caps.preferred_image_size_mm` and
  `caps.native_pixels` from the pixel count fields.

### Phase 4 — Range and interface blocks: 0x25, 0x26 (dynamic only)

- **0x25**: Populate `caps.max_pixel_clock_mhz`, `min/max_v_rate` from the 3-byte kHz fields
  (consistent with 1.x range block). Store full-precision kHz values and VRR flag in
  `DisplayIdCapabilities.dynamic_timing_range`.
- **0x26**: Bitmask decoder for per-encoding color depth support (RGB, YCbCr 4:4:4, 4:2:2, 4:2:0)
  and EOTF flags. Store in `DisplayIdCapabilities.interface_features`.

### Phase 5 — Remaining blocks: 0x28, 0x29 (defer 0x27, 0x7E, 0x81)

- **0x28 Tiled Topology**: Wire format is identical to 1.x 0x12 — reuse `decode_tiled_topology_block`
  directly, dispatching on the 2.x tag.
- **0x29 ContainerID**: `payload[0..16].try_into()` → `caps.container_id`.
- **0x27 Stereo**: Defer — low demand, wire format differences need spec verification.
- **0x7E Vendor Specific**: Defer — opaque payload, no generic decoding possible.
- **0x81 CTA DisplayID**: Defer — requires extracting CEA-861 data-block dispatch into a shared
  function callable from the DisplayID handler. Significant refactor scope.

---

## Block summary

| Tag | Block | Phase | Pipeline |
|-----|-------|-------|----------|
| 0x20 | Product Identification | 3 | dynamic |
| 0x21 | Display Parameters | 3 | dynamic |
| 0x22 | Type VII Detailed Timing | 2 | both |
| 0x23 | Type VIII Enumerated Timing Code | 2 | both |
| 0x24 | Type IX Formula-Based Timing | 2 | both |
| 0x25 | Dynamic Video Timing Range | 4 | dynamic |
| 0x26 | Display Interface Features | 4 | dynamic |
| 0x27 | Stereo Display Interface | deferred | — |
| 0x28 | Tiled Display Topology | 5 | dynamic |
| 0x29 | ContainerID | 5 | dynamic |
| 0x7E | Vendor Specific | deferred | — |
| 0x81 | CTA DisplayID | deferred | — |

---

## Tag accounting

The existing `DEFERRED_OR_RESERVED_TAG_RANGES` includes `(0x14, 0x7E)` which incorrectly lumps
1.x reserved space with 2.x defined tags. As each 2.x tag is implemented, remove it from that
range (splitting it as needed) and add it to `IMPLEMENTED_V2_BLOCK_TAGS`. Two test functions must
both stay green after each phase:

- `test_all_block_tags_accounted_for` — covers the unified 0x00–0xFF space (existing)
- `test_all_v2_block_tags_accounted_for` — covers 0x20–0x81 (new, added in Phase 1)

---

## Testing strategy

- **Per-decoder unit tests**: builder helpers (like the existing `make_type_i_descriptor`) for each
  new format; assert decoded `VideoMode` fields or struct fields; test null-clock skip and
  short-payload graceful handling.
- **Version dispatch regression**: `test_v2_tag_not_decoded_as_v1` — version byte 0x10, tag 0x22
  → zero modes. `test_v2_timing_decoded` — version byte 0x20, same payload → one mode.
- **Handler-level integration tests**: extend `make_displayid_block` to accept `version: u8`; add
  roundtrip tests through the full `ExtensionHandler::process` path for each implemented block.
- **Fixture**: add a real DisplayID 2.x EDID binary to `testdata/` if obtainable; wire it into
  `tests/fixtures.rs`.

---

## What stays untouched

The 1.x decoders are not modified. The static pipeline interface (`StaticContext`, `ModeSink`) is
unchanged. The `for_each_data_block` iterator is unchanged. The `parse_section_header` and
`check_displayid_section` functions are unchanged — the section layout is identical between 1.x
and 2.x.
