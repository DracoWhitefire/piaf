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
| `0x00` | Product Identification Block (name, manufacturer ID, product code, serial) | ✓ implemented |
| `0x01` | Display Parameters Block (physical size, color bit depths, aspect ratio) | ✓ implemented |
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

### Display Parameters Block (`0x01`)

Describes the physical display size and color bit depth.

```
Byte  0:     Block tag (0x01)
Byte  1:     Revision
Byte  2:     Payload length (minimum 6 bytes)

Per payload:
  Bytes 0–1: Horizontal image size in mm (LE uint16; 0 = not defined)
  Bytes 2–3: Vertical image size in mm (LE uint16; 0 = not defined)
  Byte  4:   Display technology (bits 7:4) and feature support flags (bits 3:0)
             Display technology: 0=monochrome, 1=RGB, 2=non-RGB multicolor
             Feature flags: bit 3=audio input, bit 2=separate default color char,
                            bit 1=power management, bit 0=fixed timing
  Byte  5:   Color bit depth:
             Bits 4:0 = interface data bit depth per primary:
               001=6 bpc, 010=8 bpc, 011=10 bpc, 100=12 bpc, 101=14 bpc, 110=16 bpc
               000=undefined, 111=reserved
             Bits 7:5 = overall display color bit depth (same encoding)
```

When both image size fields are non-zero they are written to `preferred_image_size_mm`.
Color bit depth is decoded from bits 4:0 of byte 5 into `color_bit_depth`. Both fields
are only available from the dynamic pipeline.

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

```
Bytes 0:     Block tag (0x00)
Byte  1:     Revision
Byte  2:     Payload length

Per payload:
  Bytes 0–1: Manufacturer ID (2-byte PNP-encoded, same as EDID base block bytes 0x08–0x09)
  Bytes 2–3: Product code (LE uint16)
  Bytes 4–7: Serial number (LE uint32; 0x00000000 = not specified)
  Byte  8:   Week of manufacture (0 = unspecified, 0xFF = model year)
  Byte  9:   Year (byte value + 1990; when week = 0xFF this is the model year)
  Bytes 10+: Product name (ASCII, 0x0A-terminated, space-padded; up to 13 bytes stored)
```

Decoded fields are written to `DisplayCapabilities`: `manufacturer`, `product_code`,
`serial_number`, `manufacture_date`, and `display_name`. A zero serial number is treated
as unspecified and not stored.
