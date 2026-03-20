# DisplayID 1.x Block Tags

Source: VESA DisplayID Standard Version 1.3.

Block tags are 8-bit values in the first byte of each data block header within a DisplayID
section. The section payload starts at byte 4 of the 128-byte EDID extension block; each
data block has a 3-byte header (tag, revision, payload length) followed by its payload.

> **Note:** Tag assignments below are sourced from the DisplayID 1.3 specification structure.
> They have not been confirmed against a spec PDF. Verify against the VESA document before
> implementing additional block types, and validate with a real DisplayID fixture.

## Tag assignments

| Tag           | Block                                                                      | Status        |
|---------------|----------------------------------------------------------------------------|---------------|
| `0x00`        | Product Identification Block (name, manufacturer ID, product code, serial) | ✓ implemented |
| `0x01`        | Display Parameters Block (physical size, color bit depths, aspect ratio)   | ✓ implemented |
| `0x02`        | Color Characteristics Block (primaries, white point)                       | ✓ implemented |
| `0x03`        | Detailed Timings Block (Type I — 20-byte descriptors)                      | ✓ implemented |
| `0x04`        | Video Timing Modes Type II — Detailed Timings Block                        | ✓ implemented |
| `0x05`        | Type III Short Descriptor Video Timing Block                               | ✓ implemented |
| `0x06`        | Type IV Short Descriptor Video Timing Block (DMT/VIC codes)                | ✓ implemented |
| `0x07`        | VESA Video Timing Block                                                    | ✓ implemented |
| `0x08`        | CTA Video Timing Block                                                     | ✓ implemented |
| `0x09`        | Video Timing Range Descriptor Block                                        | ✓ implemented |
| `0x0A`        | Product Serial Number Block                                                | ✓ implemented |
| `0x0B`        | General Purpose ASCII String Block                                         | ✓ implemented |
| `0x0C`        | Display Device Data Block                                                  | ✓ implemented |
| `0x0D`        | Interface Power Sequencing Block                                           | ✓ implemented |
| `0x0E`        | Transfer Characteristics Block                                             | — deferred    |
| `0x0F`        | Display Interface Block                                                    | — deferred    |
| `0x10`        | Stereo Display Interface Block                                             | — deferred    |
| `0x11`        | Type V Short Timings Block                                                 | ✓ implemented |
| `0x12`        | Tiled Display Topology Block                                               | — deferred    |
| `0x13`        | Type VI Detailed Timings Block                                             | ✓ implemented |
| `0x14`–`0x7E` | Reserved                                                                   | — reserved    |
| `0x7F`        | Vendor-Specific                                                            | — reserved    |
| `0x80`–`0xFF` | Undefined (outside DisplayID 1.x tag space)                                | — reserved    |

## Block structures

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

### Color Characteristics Block (`0x02`)

Provides the display's CIE xy color primaries and white point at higher precision than the
EDID base block.

```
Byte  0:     Block tag (0x02)
Byte  1:     Revision
Byte  2:     Payload length (minimum 16 bytes)

Per payload (16 bytes for an RGB display):
  Bytes  0–1:  Red primary x   (LE uint16; value × 1/1024 = CIE x; lower 10 bits significant)
  Bytes  2–3:  Red primary y
  Bytes  4–5:  Green primary x
  Bytes  6–7:  Green primary y
  Bytes  8–9:  Blue primary x
  Bytes 10–11: Blue primary y
  Bytes 12–13: White point x
  Bytes 14–15: White point y
```

Decoded into `caps.chromaticity`, overwriting any value from the EDID base block. Payloads
shorter than 16 bytes are silently ignored. Only available from the dynamic pipeline.

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

### Detailed Timings Block — Type II (`0x04`)

Each block contains one or more 11-byte timing descriptors. Both the dynamic and static
pipelines decode Type II blocks.

```
Byte  0:     Block tag (0x04)
Byte  1:     Revision
Byte  2:     Payload length (multiple of 11)
Bytes 3+:    11-byte timing descriptors

Per descriptor:
  Bytes 0–2:  Pixel clock (LE 24-bit; actual = (raw + 1) × 10 kHz)
  Byte  3:    Flags: [7]=preferred, [6:5]=stereo, [4]=interlaced,
                     [3]=HS polarity (+), [2]=VS polarity (+), [1:0]=reserved
  Byte  4:    H-active mantissa bits 7:0  (8-pixel granule; h_active = 8 + 8 × 9-bit-mantissa)
  Byte  5:    Bit 0 = H-active mantissa bit 8; bits 7:1 = H-blank mantissa (7-bit, same granule)
  Byte  6:    Bits 7:4 = H-front-porch mantissa (4-bit, 8-pixel granule)
              Bits 3:0 = H-sync-width mantissa  (4-bit, 8-pixel granule)
  Byte  7:    V-active mantissa bits 7:0  (1-line granule; v_active = 1 + 12-bit-mantissa)
  Byte  8:    Bits 3:0 = V-active mantissa bits 11:8; bits 7:4 = reserved
  Byte  9:    Full byte: V-blank mantissa (v_blank = 1 + byte)
              Bits 7:4:  V-front-porch mantissa (v_fp = 1 + nibble)
              Bits 3:0:  V-sync-width mantissa  (v_sw = 1 + nibble)
  Byte  10:   Reserved
```

Refresh rate derived as `pixel_clock_hz / (h_total × v_total)` where
`pixel_clock_hz = (raw + 1) × 10 000` and `h_total = h_active + h_blank`.

Note: byte 9 is dual-role — the full 8-bit value gives the total vertical blanking interval
while the upper/lower nibbles give the front-porch and sync-width sub-intervals. The implied
back porch is `v_blank − v_front_porch − v_sync_width`.

### Detailed Timings Block — Type III (`0x05`)

Each block contains one or more 3-byte short timing descriptors. Both the dynamic and static
pipelines decode Type III blocks. Vertical active is derived from horizontal active using the
aspect ratio; no blanking detail is stored.

```
Byte  0:     Block tag (0x05)
Byte  1:     Revision
Byte  2:     Payload length (multiple of 3)
Bytes 3+:    3-byte timing descriptors

Per descriptor:
  Byte 0:  Bit 7 = preferred timing
           Bits 6:4 = CVT algorithm (0 = standard blanking, 1 = reduced blanking)
           Bits 3:0 = Aspect ratio code:
             0=1:1, 1=5:4, 2=4:3, 3=15:9, 4=16:9, 5=16:10, 6=64:27, 7=256:135, 8=undefined
  Byte 1:  Horizontal active pixels = (byte + 1) × 8  (max 2048 px; 8-pixel granule)
  Byte 2:  Bit 7 = interlaced
           Bits 6:0 = Vertical refresh rate = bits + 1  Hz (range 1–128 Hz)
```

Vertical active is computed as `h_active × height / width` using the aspect ratio fraction.
Descriptors with aspect code 8 (undefined) or codes 9–15 (reserved) are silently skipped,
as are descriptors where the height calculation does not yield a whole number of lines.

### Timing Code Block — Type IV (`0x06`)

Each block carries a list of 1-byte timing identifiers. The code space is encoded in the
data block revision byte's upper 2 bits:

```
Byte  0:     Block tag (0x06)
Byte  1:     Revision — bits 7:6 = code type:
               0 = VESA DMT IDs
               1 = CTA-861 VIC codes
               2 = HDMI VIC codes (4 defined: 1–4)
               3 = reserved
             bits 5:0 = revision/reserved
Byte  2:     Payload length (number of 1-byte codes)
Bytes 3+:    One byte per timing code
```

DMT codes are resolved via the VESA DMT v1.13 table (IDs 0x01–0x58).
VIC codes are resolved via the CTA-861 table.
HDMI VIC codes 1–4 map to 3840×2160@30/25/24 Hz and 4096×2160@24 Hz respectively.
Unrecognised codes and reserved code types are silently skipped.

### VESA Video Timing Block (`0x07`)

A compact DMT presence bitmap. Each bit indicates whether the display supports the
corresponding VESA DMT mode. The payload covers DMT IDs 0x01–0x50 (80 modes, 10 bytes);
bit `i` (0-indexed, LSB-first within each byte) maps to DMT ID `i + 1`.

```
Byte  0:     Block tag (0x07)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (0–10; bytes beyond 10 are ignored)
Bytes 3+:    Presence bitmap, LSB-first
               Byte 0 bits 7:0 = DMT IDs 0x08–0x01
               Byte 1 bits 7:0 = DMT IDs 0x10–0x09
               ...
               Byte 9 bits 7:0 = DMT IDs 0x50–0x49
```

Set bits are resolved via the VESA DMT v1.13 table, including full timing detail
(front porch, sync width, sync polarity). DMT IDs 0x51–0x58 are not representable
in this block. Unset bits and payload bytes beyond 10 are silently skipped.

### CTA Video Timing Block (`0x08`)

A compact CTA-861 VIC presence bitmap, structurally identical to `0x07` but indexed
over VIC codes instead of DMT IDs. The payload covers VICs 1–64 (8 bytes maximum);
bit `i` (0-indexed, LSB-first within each byte) maps to VIC `i + 1`.

```
Byte  0:     Block tag (0x08)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (0–8; bytes beyond 8 are ignored)
Bytes 3+:    Presence bitmap, LSB-first
               Byte 0 bits 7:0 = VICs 8–1
               Byte 1 bits 7:0 = VICs 16–9
               ...
               Byte 7 bits 7:0 = VICs 64–57
```

Set bits are resolved via the CTA-861 VIC table with full timing detail. VICs 65 and
above are not representable in this block. Unset bits and payload bytes beyond 8 are
silently skipped.

### Video Timing Range Limits Block (`0x09`)

Describes the range of timings a display can accept: minimum/maximum pixel clock,
horizontal scan rate, and vertical refresh rate.

```
Byte  0:     Block tag (0x09)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (15 bytes)

Per payload:
  Bytes  0–2:  Minimum pixel clock, 10 kHz steps (LE 24-bit; not stored)
  Bytes  3–5:  Maximum pixel clock, 10 kHz steps (LE 24-bit; stored ÷ 100 → MHz)
  Byte   6:    Minimum horizontal scan frequency, kHz
  Byte   7:    Maximum horizontal scan frequency, kHz
  Bytes  8–9:  Minimum horizontal blanking pixels (LE uint16; not stored)
  Byte  10:    Minimum vertical refresh rate, Hz
  Byte  11:    Maximum vertical refresh rate, Hz
  Bytes 12–13: Minimum vertical blanking lines (LE uint16; not stored)
  Byte  14:    Video timing support flags (not stored)
```

Note: the specification document lists payload length as `9`, but the field table spans
15 payload bytes. The 15-byte interpretation is used here as it is self-consistent.

Decoded into `caps`: `max_pixel_clock_mhz`, `min_h_rate_khz`, `max_h_rate_khz`,
`min_v_rate`, `max_v_rate`. Fields are only written when the payload is long enough.
Only available from the dynamic pipeline.

### Product Serial Number Block (`0x0A`)

Carries the display's serial number as a plain ASCII string.

```
Byte  0:     Block tag (0x0A)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (number of ASCII bytes)
Bytes 3+:    Serial number string (ASCII, `0x0A`-terminated, space-padded)
```

Decoded into `caps.serial_number_string`, using the same `MonitorString` format as the
EDID base-block serial number descriptor (`0xFF`): up to 13 bytes, `0x0A`-terminated.
Longer strings are truncated. Empty payloads are silently ignored.
Only available from the dynamic pipeline.

### General Purpose ASCII String Block (`0x0B`)

Carries an application-defined text string. Multiple `0x0B` blocks may appear in one
section; each is stored in a successive `unspecified_text` slot (up to 4).

```
Byte  0:     Block tag (0x0B)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (number of ASCII bytes)
Bytes 3+:    String (ASCII, `0x0A`-terminated, space-padded)
```

Each string is stored using the same `MonitorString` format as EDID base-block
unspecified-text descriptors: up to 13 bytes, `0x0A`-terminated. Longer strings are
truncated. Empty payloads and blocks beyond the fourth are silently dropped.
Only available from the dynamic pipeline.

### Display Device Data Block (`0x0C`)

Describes panel characteristics for embedded display applications: display technology,
operating mode, native pixel format, sub-pixel layout, pixel pitch, and color bit depth.

```
Byte  0:     Block tag (0x0C)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (13 bytes)

Per payload:
  Byte  0:    Bits 7:4 = display technology type:
                0=TFT(unspecified), 1=DSTN/STN, 2=TFT-IPS, 3=TFT-MVA/PVA, 4=CRT,
                5=PDP, 6=OLED, 7=EL, 8=FED/SED, 9=LCoS, 10–15=reserved
              Bits 3:0 = technology-specific sub-type code (raw)
  Byte  1:    Bits 3:0 = operating mode (0=continuous, 1=non-continuous)
              Bits 5:4 = backlight type (0=none, 1=AC/CCFL, 2=DC/LED, 3=reserved)
              Bit  6   = Data Enable (DE) signal is used (1=yes)
              Bit  7   = DE signal polarity (1=positive, 0=negative)
  Bytes 2–3:  Horizontal native pixel count (LE uint16; 0 = not defined)
  Bytes 4–5:  Vertical native pixel count (LE uint16; 0 = not defined)
  Byte  6:    Aspect ratio encoded as (AR − 1) × 100 (raw; 0 = not defined)
  Byte  7:    Bits 1:0 = physical orientation (0=landscape, 1=portrait, 2=n/a, 3=undefined)
              Bits 3:2 = rotation capability (0=none, 1=90°CW, 2=180°, 3=270°CW)
              Bits 5:4 = zero pixel location (0=upper-left, 1=upper-right,
                                              2=lower-left, 3=lower-right)
              Bits 7:6 = scan direction (0=not defined, 1=normal, 2=reversed, 3=reserved)
  Byte  8:    Sub-pixel layout:
                0x00=not defined, 0x01=RGB-vertical, 0x02=BGR-vertical,
                0x03=RGB-horizontal, 0x04=BGR-horizontal, 0x05=quad-RGBG,
                0x06=quad-BGRG, 0x07=delta-RGB, 0x08=delta-BGR, ≥0x09=reserved
  Byte  9:    Horizontal pixel pitch in 0.01 mm steps (0 = not defined)
  Byte 10:    Vertical pixel pitch in 0.01 mm steps (0 = not defined)
  Byte 11:    Bits 3:0 = color bit depth: bpc − 1 (i.e. 5=6bpc, 7=8bpc, 9=10bpc…)
  Byte 12:    Pixel response time in ms (0 = not defined)
```

All payload fields are decoded into `DisplayCapabilities`. Zero values for native pixels,
pixel pitch, and response time are treated as "not defined" and not stored. Only available
from the dynamic pipeline.

### Interface Power Sequencing Block (`0x0D`)

Specifies the minimum timing delays required when powering the panel interface on and off.
Follows the standard T1–T6 power sequencing model used by LVDS and eDP panels.

```
Byte  0:     Block tag (0x0D)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (8 bytes)

Per payload:
  Byte 0:  T1 minimum — power supply enable → interface signal valid (2 ms per count)
  Byte 1:  T2 minimum — interface signal enable → backlight enable (2 ms per count)
  Byte 2:  T3 minimum — backlight disable → interface signal disable (2 ms per count)
  Byte 3:  T4 minimum — interface signal disable → power supply disable (2 ms per count)
  Byte 4:  T5 minimum — power supply off time before re-applying (2 ms per count)
  Byte 5:  T6 minimum — backlight off time (2 ms per count)
  Bytes 6–7: Reserved (ignored)
```

Power-on sequence: [VCC on] →T1→ [Signal on] →T2→ [Backlight on]
Power-off sequence: [Backlight off] →T3→ [Signal off] →T4→ [VCC off]
Minimum off times: T5 (VCC), T6 (backlight)

Decoded into `caps.power_sequencing` as a `PowerSequencing` struct. Raw counts are stored
as-is; multiply by 2 to obtain milliseconds. Payloads shorter than 6 bytes are silently
skipped. Only available from the dynamic pipeline.

### Type V Short Timings Block (`0x11`)

Each block contains one or more 7-byte short timing descriptors. Width and height are
stored directly (no aspect-ratio derivation). Only progressive timings are defined
(CVT-RB or CVT-RB2). No blanking detail is stored.

```
Byte  0:     Block tag (0x11)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (N × 7, 1 ≤ N ≤ 35)
Bytes 3+:    7-byte timing descriptors

Per descriptor:
  Byte  0:   Options: bits 1:0 = CVT algorithm (0=CVT-RB2, 1=CVT-RB);
             bit 4 = NTSC optimized; bits 6:5 = stereo; bit 7 = preferred
  Bytes 1–2: Horizontal active pixels (exact, LE uint16)
  Bytes 3–4: Vertical active lines (exact, LE uint16)
  Byte  5:   Vertical refresh rate = byte + 1 Hz (1–256 Hz; clamped to 255)
  Byte  6:   Reserved
```

Descriptors with zero width or height are silently skipped.

### Type VI Detailed Timings Block (`0x13`)

Each block contains one or more variable-length detailed timing descriptors (14 or
17 bytes each). Based on Type I but uses 1 kHz pixel clock steps for higher precision.

```
Byte  0:     Block tag (0x13)
Byte  1:     Revision (0x00)
Byte  2:     Payload length (N × 17 + M × 14)
Bytes 3+:    14- or 17-byte timing descriptors

Per descriptor:
  Bytes 0–2:  Pixel clock in 1 kHz steps (LE 22-bit, bits 21:0; max ~4194 MHz);
              bit 22 = aspect/size info present (17-byte form); bit 23 = preferred
  Bytes 3–4:  H-active pixels (15-bit, bits 14:0); bit 15 = H-sync polarity (1=+)
  Bytes 5–6:  V-active lines  (15-bit, bits 14:0); bit 15 = V-sync polarity (1=+)
  Bytes 7–9:  H-blank (12-bit) and H-front-porch (12-bit) packed:
                byte7 = H-blank[7:0], byte8 = H-fp[7:0],
                byte9[3:0] = H-blank[11:8], byte9[7:4] = H-fp[11:8]
  Byte  10:   H-sync width
  Byte  11:   V-blank lines
  Byte  12:   V-front-porch lines
  Byte  13:   bits 3:0 = V-sync width; bit 7 = interlaced (1=interlaced)
  Bytes 14–16 (optional): aspect/size info — present iff bit 22 of bytes 0–2 is set
```

Null descriptors (pixel clock = 0) advance the cursor without emitting a mode.
The descriptor size (14 or 17) is determined by bit 22 of the first field.
