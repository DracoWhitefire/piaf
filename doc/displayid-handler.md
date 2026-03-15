# DisplayID Extension Handler

DisplayID (tag `0x70`) is a VESA standard for display identification that carries richer
timing and capability data than the base EDID block. It is most common on DisplayPort
Alt Mode devices, docks, and professional monitors.

## Multi-block sections

Unlike CEA-861, where each 128-byte extension block is self-contained, a single logical
DisplayID section may span several consecutive 128-byte blocks all tagged `0x70`. The
dispatch layer collects all `0x70` blocks in stream order and passes them to
`DisplayIdHandler` as a slice — the handler owns reassembly. No other part of the pipeline
needs to know about DisplayID's multi-block structure.

In `alloc`/`std` builds, full multi-block reassembly is supported. In bare `no_std` builds
(no allocator), extension blocks cannot be stored after parsing, so the static pipeline
receives only base-block data; DisplayID content is unavailable at that tier.

## Dynamic pipeline

`DisplayIdHandler` is registered automatically by `ExtensionLibrary::with_standard_handlers()`.
After calling `capabilities_from_edid`, retrieve the parsed DisplayID section via
`get_extension_data`:

```rust
use piaf::{DisplayIdCapabilities, capabilities_from_edid, parse_edid};

let library = ExtensionLibrary::with_standard_handlers();
let parsed = parse_edid(&bytes, &library)?;
let caps = capabilities_from_edid(&parsed, &library);

if let Some(did) = caps.get_extension_data::<DisplayIdCapabilities>(0x70) {
    println!("DisplayID version: 0x{:02X}", did.version);
    println!("Product type: {}", did.product_type);
}
```

Video modes decoded from DisplayID timing blocks are added to `caps.supported_modes`
alongside modes from the base block and CEA-861.

## Static pipeline

`DisplayIdHandler` is included in `STANDARD_HANDLERS`. It decodes video modes from Type I
timing blocks and pushes them into the static output:

```rust
use piaf::{STANDARD_HANDLERS, StaticDisplayCapabilities, capabilities_from_edid_static, parse_edid};

let parsed = parse_edid(&bytes, STANDARD_HANDLERS)?;
let caps: StaticDisplayCapabilities<64> = capabilities_from_edid_static(&parsed, STANDARD_HANDLERS);

for mode in caps.iter_modes() {
    println!("{}×{}@{}Hz", mode.width, mode.height, mode.refresh_rate);
}
```

`DisplayIdCapabilities` is not available from the static pipeline — rich metadata requires
the dynamic pipeline.

## `DisplayIdCapabilities`

Stored under tag `0x70` in `DisplayCapabilities::extension_data`:

| Field | Type | Description |
|---|---|---|
| `version` | `u8` | Version byte from the section header (0x10–0x1F = v1.x, 0x20 = v2.x) |
| `product_type` | `u8` | Display product primary use case, bits 2:0 of header byte 3 |

## Extracted timing data

The initial implementation decodes **Type I Video Timing** blocks (tag `0x01`). Each
20-byte descriptor maps to a `VideoMode` with full timing detail:

| `VideoMode` field | Source |
|---|---|
| `width`, `height` | Horizontal/Vertical Active (exact pixel/line counts) |
| `refresh_rate` | Derived: `pixel_clock_hz / (h_total × v_total)` |
| `interlaced` | Byte 19 bit 0 |
| `h_front_porch`, `h_sync_width` | Bytes 7–10 |
| `v_front_porch`, `v_sync_width` | Bytes 15–18 |
| `sync` | `DigitalSeparate`; polarities from byte 19 bits 3–4 |

Null descriptors (pixel clock = 0) are silently skipped.

## Warnings

| Variant | Meaning |
|---|---|
| `DisplayIdVersionUnknown(u8)` | Version byte is outside the known ranges (0x10–0x1F, 0x20). The block is skipped. |
| `DisplayIdExtensionCountMismatch { declared, found }` | The extension count in the first fragment's header does not match the number of continuation blocks actually present. Processing continues with whatever fragments are available. |

## Fragment layout reference

Each 128-byte EDID extension block carrying DisplayID has the following structure:

```
Byte 0:      0x70 (EDID extension tag)
Byte 1:      DisplayID version/revision
Byte 2:      Section byte count (data block payload bytes in this fragment)
Byte 3:      Bits [7:3] = continuation block count
             Bits [2:0] = display product primary use case
Bytes 4–126: DisplayID data blocks
Byte 127:    Checksum
```

Data blocks within the payload each begin with a 3-byte header:

```
Byte 0: Block tag
Byte 1: Revision
Byte 2: Payload length (bytes following this header)
```

Iteration stops at an end-of-section sentinel (tag `0x00`, length `0`) or when a block's
declared length would extend past the available payload.
