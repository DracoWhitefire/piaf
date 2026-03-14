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
| `0x2A` | DisplayID Type X Video Timing Data Block (T10VTDB) | — not yet |
| `0x2B`–`0x77` | Reserved | — reserved |
| `0x78` | HDMI Forum EDID Extension Override Data Block | — not yet |
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

### DisplayID Type VII / VIII / X (`0x22`, `0x23`, `0x2A`)

Source: CTA-861-H Tables 104, 107, 109–110. These embed DisplayID timing descriptors
inside a CTA data block. Deferred to the 0.2 DisplayID milestone.

### HDMI Forum blocks (`0x78`, `0x79`)

Source: HDMI Forum EDID Extension Specification (not publicly available without HDMI Forum
membership). Deferred until the spec can be sourced.
