# DisplayID 1.x Block Tags

Source: VESA DisplayID Standard Version 1.3.

Block tags are 8-bit values in the first byte of each data block header within a DisplayID
section. The section payload starts at byte 4 of the 128-byte EDID extension block; each
data block has a 3-byte header (tag, revision, payload length) followed by its payload.

> **Note:** Tag assignments below are sourced from the DisplayID 1.3 specification structure.
> They have not been confirmed against a spec PDF. Verify against the VESA document before
> implementing additional block types, and validate with a real DisplayID fixture.

## Tag assignments

| Tag | Block | Status |
|---|---|---|
| `0x00` | Product Identification Block (name, manufacturer ID, product code, serial) | — deferred |
| `0x01` | Display Parameters Block (physical size, color bit depths, aspect ratio) | — deferred |
| `0x02` | Color Characteristics Block (primaries, white point) | — deferred |
| `0x03` | Detailed Timings Block (Type I — 20-byte descriptors) | ✓ implemented |
| `0x04` | Video Format Block (Type II — formula-based) | — deferred |
| `0x05` | Type III Short Descriptor Video Timing Block | — deferred |
| `0x06` | Type IV Short Descriptor Video Timing Block (DMT codes) | — deferred |
| `0x07` | VESA Video Timing Block | — deferred |
| `0x08` | CTA Video Timing Block | — deferred |
| `0x09` | Video Timing Range Descriptor Block | — deferred |
| `0x0A` | Product Serial Number Block | — deferred |
| `0x0B` | General Purpose ASCII String Block | — deferred |
| `0x0C` | Display Device Data Block | — deferred |
| `0x0D` | Interface Power Sequencing Block | — deferred |
| `0x0E` | Transfer Characteristics Block | — deferred |
| `0x0F` | Display Interface Block | — deferred |
| `0x10` | Stereo Display Interface Block | — deferred |
| `0x11` | Type V Short Descriptor Video Timing Block | — deferred |
| `0x12` | Tiled Display Topology Block | — deferred |
| `0x13` | Type VI Short Descriptor Video Timing Block (added in 1.3) | — deferred |
| `0x14`–`0x7E` | Reserved | — reserved |
| `0x7F` | Vendor-Specific | — reserved |
| `0x80`–`0xFF` | Undefined (outside DisplayID 1.x tag space) | — reserved |

## Block structures

### Detailed Timings Block — Type I (`0x03`)

Each block contains one or more 20-byte timing descriptors. Null descriptors (pixel
clock = 0) are skipped.

```
Bytes 0:     Block tag (0x03)
Byte  1:     Revision
Byte  2:     Payload length (multiple of 20)
Bytes 3+:    20-byte timing descriptors

Per descriptor:
  Byte  0:     Options (bit 7 = preferred timing)
  Bytes 1–2:   Pixel clock in 10 kHz units (LE uint16; 0 = null, skip)
  Bytes 3–4:   Horizontal Active in pixels (exact, LE uint16)
  Bytes 5–6:   Horizontal Blank in pixels (exact, LE uint16)
  Bytes 7–8:   Horizontal Front Porch in pixels (exact, LE uint16)
  Bytes 9–10:  Horizontal Sync Width in pixels (exact, LE uint16)
  Bytes 11–12: Vertical Active in lines (exact, LE uint16)
  Bytes 13–14: Vertical Blank in lines (exact, LE uint16)
  Bytes 15–16: Vertical Front Porch in lines (exact, LE uint16)
  Bytes 17–18: Vertical Sync Width in lines (exact, LE uint16)
  Byte  19:    Flags: [0]=interlaced, [3]=HS polarity (+), [4]=VS polarity (+)
```

Refresh rate derived as `pixel_clock_hz / (h_total × v_total)` where
`pixel_clock_hz = field × 10 000` and `h_total = h_active + h_blank`.

### Product Identification Block (`0x00`)

Contains the display name string, manufacturer PNP ID, product code, and serial number.
The end-of-section sentinel (tag `0x00`, length `0`) shares the tag byte; a Product
Identification Block is distinguished by having a non-zero length field.
