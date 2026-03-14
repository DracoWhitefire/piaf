# CEA-861 / CTA-861 Extended Tag Codes

Source: CTA-861-H Table 62 (confirmed against actual spec PDF).

Extended tag data blocks use outer CTA tag `0x07`, with the first payload byte as the extended tag code. Max 30 payload bytes (after the outer length byte).

## Tag assignments

| Extended Tag | Block | Status |
|---|---|---|
| `0x00` | Video Capability Data Block (VCDB) | ✓ implemented |
| `0x01` | Vendor-Specific Video Data Block (VSVDB) | ✓ implemented (`VendorSpecificBlock`) |
| `0x02` | VESA Display Device Data Block (DDDB) | ✓ implemented |
| `0x03` | Reserved for VESA Video Data Block (VTB-EXT) | ✓ implemented |
| `0x04` | Reserved for HDMI Video Data Block (HDMI Forum spec) | — reserved, no public structure |
| `0x05` | Colorimetry Data Block (CDB) | ✓ implemented |
| `0x06` | HDR Static Metadata Data Block | ✓ implemented |
| `0x07` | HDR Dynamic Metadata Data Block | ✓ implemented |
| `0x08`–`0x0C` | Reserved for video-related blocks | — reserved |
| `0x0D` | Video Format Preference Data Block (VFPDB) | ✓ implemented |
| `0x0E` | YCbCr 4:2:0 Video Data Block (Y420VDB) | ✓ implemented |
| `0x0F` | YCbCr 4:2:0 Capability Map Data Block (Y420CMDB) | ✓ implemented |
| `0x10` | Reserved for CTA Miscellaneous Audio Fields (MAF) | — reserved |
| `0x11` | Vendor-Specific Audio Data Block (VSADB) | ✓ implemented |
| `0x12` | HDMI Audio Data Block | ✓ implemented |
| `0x13` | Room Configuration Data Block (RCDB) | ✓ implemented |
| `0x14` | Speaker Location Data Block (SLDB) | ✓ implemented |
| `0x15`–`0x1F` | Reserved for audio-related blocks | — reserved |
| `0x20` | InfoFrame Data Block (IFDB) | ✓ implemented |
| `0x21` | Reserved | — reserved |
| `0x22` | DisplayID Type VII Video Timing Data Block (T7VTDB) | ✓ implemented (`T7VtdbBlock`) |
| `0x23` | DisplayID Type VIII Video Timing Data Block (T8VTDB) | ✓ implemented (`T8VtdbBlock`) |
| `0x24`–`0x29` | Reserved | — reserved |
| `0x2A` | DisplayID Type X Video Timing Data Block (T10VTDB) | ✓ implemented (`T10VtdbBlock`) |
| `0x2B`–`0x77` | Reserved | — reserved |
| `0x78` | HDMI Forum EDID Extension Override Data Block | ✓ implemented (`hf_eeodb_extension_count`) |
| `0x79` | HDMI Forum Sink Capability Data Block | — not yet |
| `0x7A`–`0x7F` | Reserved for HDMI | — reserved |
| `0x80`–`0xFF` | Reserved | — reserved |

## Block structures

### VSVDB — Vendor-Specific Video Data Block (`0x01`)

Source: CTA-861-H / CEA-861-E Table 56, Section 7.5.7.

```
Byte 1:  Tag Code (0x07) | Length L
Byte 2:  Extended Tag Code (0x01)
Byte 3:  OUI byte 0 (LSB)
Byte 4:  OUI byte 1
Byte 5:  OUI byte 2 (MSB)
Byte 6…L+1: Vendor-specific payload (L-4 bytes)
```

- Minimum length: 4 bytes after extended tag (3 OUI + at least 0 payload).
- A sink may include multiple VSVDBs (different OUIs).
- Payload interpretation is vendor-defined. Well-known OUIs include Dolby Vision (`0x00D046`).

### VSADB — Vendor-Specific Audio Data Block (`0x11`)

Source: CTA-861-H / CEA-861-E Table 57, Section 7.5.8.

```
Byte 1:  Tag Code (0x07) | Length L
Byte 2:  Extended Tag Code (0x11)
Byte 3:  OUI byte 0 (LSB)
Byte 4:  OUI byte 1
Byte 5:  OUI byte 2 (MSB)
Byte 6…L+1: Vendor-specific payload (L-4 bytes)
```

Identical structure to VSVDB; semantics are audio-capability-related rather than video.

### DisplayID Type VII (`0x22`)

Source: CTA-861-H Table 104. One 20-byte DisplayID-style timing descriptor per block.
Pixel clock in kHz (24-bit LE); 16-bit H/V fields; T7Y420 flag. Implemented as `T7VtdbBlock`.

### DisplayID Type VIII (`0x23`)

Source: CTA-861-H Table 107. List of VESA DMT ID codes (1-byte when TCS=0, 2-byte LE
when TCS=1). Only `Code_Type = 0x00` (DMT) is defined for CTA-861. Implemented as
`T8VtdbBlock` with a built-in lookup table for DMT IDs 0x01–0x58.

### DisplayID Type X (`0x2A`)

Source: CTA-861-H Tables 109–110. CVT formula-based timing descriptors of 6–8 bytes
each (size = 6 + M, where M = bits[6:4] of the `rev` byte). Per descriptor:

```
[0]     flags: YCC420[7] | Stereo[6:5] | VR_HB[4] | EVS[3] | Formula[2:0]
[1..2]  Horizontal active − 1 (LE u16)
[3..4]  Vertical active − 1 (LE u16)
[5]     Refresh rate LSB (stored − 1)
[6]     (M≥1) bits[1:0] = refresh rate high 2 bits; bits[7:2] = CVT controls
[7]     (M=2) Alt_Min_VBlank and reserved
```

Full refresh Hz = `(x[5] | ((x[6] & 0x03) << 8)) + 1`. Implemented as `T10VtdbBlock`
containing a `Vec<T10VtdbEntry>`. M > 2 is invalid and returns `None`.

### HDMI Forum EDID Extension Override Data Block (`0x78`)

Source: HDMI 2.1 section 10.3.6 (structure reconstructed from Linux kernel `drm_edid.c`
and edid-decode; the full HDMI Forum spec requires membership).

```
Byte 1:  Tag Code (0x07) | Length = 2
Byte 2:  Extended Tag Code (0x78)
Byte 3:  EDID Extension Block Count (override value; 0 = invalid)
```

Must be the **first data block** in Block 1 (the first CTA-861 extension). Overrides the
1-byte extension count in the base EDID header for HDMI 2.1 sinks whose full E-EDID
exceeds what the base EDID byte can represent. Exposed as
`Cea861Capabilities::hf_eeodb_extension_count: Option<u8>`.

### HDMI Forum Sink Capability Data Block (`0x79`)

Source: HDMI Forum EDID Extension Specification (not publicly available without HDMI Forum
membership). Deferred until the spec can be sourced.
