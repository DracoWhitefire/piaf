# PIAF

A Rust library for decoding EDID display capability data into a clean, typed model.

PIAF reads raw EDID bytes — from a file, a kernel sysfs node, or a direct I²C read —
and produces a `DisplayCapabilities` value with all the information a display or
HDMI-adjacent application typically needs: identity, input type, supported modes,
color characteristics, HDR metadata, audio capabilities, and more.

Decoding happens in two steps. `parse_edid` validates the raw bytes and produces a
`ParsedEdid`, which holds the block structure close to the wire format. `capabilities_from_edid`
then runs the registered extension handlers over that structure and produces a
`DisplayCapabilities` — the typed, stable model your application works with. Keeping
these steps separate means you can inspect the raw parse result for debugging, or run
multiple handler configurations over the same parsed data without re-parsing.

```rust
use piaf::{parse_edid, capabilities_from_edid, ExtensionLibrary, ScreenSize};

let bytes = std::fs::read("/sys/class/drm/card0-HDMI-A-1/edid")?;
let library = ExtensionLibrary::with_standard_handlers();
let parsed = parse_edid(&bytes, &library)?;
let caps = capabilities_from_edid(&parsed, &library);

println!("Display: {}", caps.display_name.as_deref().unwrap_or("unknown"));
if let Some(ScreenSize::Physical { width_cm, height_cm }) = caps.screen_size {
    println!("{}×{} cm", width_cm, height_cm);
}
for mode in &caps.supported_modes {
    println!("  {}×{}@{}", mode.width, mode.height, mode.refresh_rate);
}
```

See [`examples/inspect_displays.rs`](examples/inspect_displays.rs) for a more complete example.

## Why PIAF

**Deep CEA-861 coverage.** Most EDID libraries decode the base block and stop.
PIAF decodes 20+ CEA-861 data block types, including HDR static and dynamic metadata,
HDMI 1.x and HDMI Forum VSDBs, colorimetry, speaker allocation, video timing blocks,
and the HDMI Forum Sink Capability block. If the information is in the EDID, PIAF
exposes it as typed fields rather than raw bytes.

**Pluggable handlers.** The extension handler system lets you register your own handler
for any extension block tag — override the built-in CEA-861 handler, add support for
a proprietary block, or attach typed custom data to `DisplayCapabilities` for downstream
consumers. Base block parsing is pluggable too.

**Honest diagnostics.** PIAF distinguishes between hard parse errors (invalid header,
checksum failure, truncated input) and non-fatal warnings (unknown extension block,
malformed data block, out-of-range manufacturer ID). You decide how strict to be;
nothing is silently discarded.

**`no_std` support.** The library runs on bare metal. The static extension handler
pipeline — `capabilities_from_edid_static` with `STANDARD_HANDLERS` — works at all
build tiers, including bare `no_std` without an allocator. Custom handlers implement
`StaticExtensionHandler` using `static` references instead of `Box` — see
[`no_std` builds](#no_std-builds) below.

**Stable consumer model.** `ParsedEdid` preserves raw bytes; `DisplayCapabilities`
is the typed, stable output. Parser improvements don't change the consumer-facing API.

## Extension system

`Cea861Handler` covers the common case. Write your own handler to support a proprietary
extension block, augment CEA-861 decoding with application-specific logic, or attach typed
custom data to `DisplayCapabilities` for downstream consumers.

### Dynamic handlers (`std`/`alloc`)

Register via `ExtensionLibrary`. Uses `Box<dyn ExtensionHandler>` internally, so requires
heap allocation:

```rust
use piaf::{ExtensionHandler, DisplayCapabilities, ParseWarning, ExtensionLibrary};

#[derive(Debug)]
struct MyHandler;

impl ExtensionHandler for MyHandler {
    fn process(&self, block: &[u8; 128], caps: &mut DisplayCapabilities, warnings: &mut Vec<ParseWarning>) {
        // inspect block, set fields on caps
    }
}

let mut library = ExtensionLibrary::new();
library.register(ExtensionMetadata {
    tag: 0xAB,
    display_name: String::from("My Extension"),
    handler: Some(Box::new(MyHandler)),
});
```

Typed data can be attached to `DisplayCapabilities` and retrieved by tag:

```rust
caps.set_extension_data(0xAB, MyCeaData { version: block[1] });

if let Some(data) = caps.get_extension_data::<MyCeaData>(0xAB) {
    println!("version: {}", data.version);
}
```

### Static handlers (no-alloc)

Use `StaticExtensionHandler` when heap allocation is unavailable. Pass a `static` slice
to `capabilities_from_edid_static`:

```rust
use piaf::{StaticExtensionHandler, ModeSink, StaticDisplayCapabilities, STANDARD_HANDLERS};

struct MyHandler;

impl StaticExtensionHandler for MyHandler {
    fn tag(&self) -> u8 { 0xAB }
    fn process(&self, block: &[u8; 128], sink: &mut dyn ModeSink) {
        // push modes via sink.push_mode(...)
    }
}

static MY_HANDLER: MyHandler = MyHandler;
static HANDLERS: &[&dyn StaticExtensionHandler] = &[STANDARD_HANDLERS[0], &MY_HANDLER];

let caps: StaticDisplayCapabilities<64> =
    piaf::capabilities_from_edid_static(&parsed, HANDLERS);
```

Static handlers extract modes only — audio, VSDB, colorimetry, and similar rich metadata
require the dynamic pipeline.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Enables `std`-backed types and the full extension system |
| `alloc` | no      | Enables dynamic allocation without `std` |
| `serde` | no      | Derives `Serialize`/`Deserialize` on public types |

## `no_std` builds

Bare `no_std` (neither `std` nor `alloc`) is supported. The dynamic extension handler
pipeline (`ExtensionLibrary`, `capabilities_from_edid`) requires `alloc` or `std`.
The static pipeline (`capabilities_from_edid_static`) is available unconditionally.

In bare `no_std`, `ParsedEdid` does not store extension blocks (that field is
alloc-gated), so `capabilities_from_edid_static` returns base-block data only —
`StaticDisplayCapabilities` with all scalar fields populated and modes from established
timings, standard timings, and detailed timing descriptors.

To also get modes from CEA-861 extension blocks, enable the `alloc` or `std` feature.

### Fields in `DisplayCapabilities` available in all build configurations

| Field | Type |
|-------|------|
| `manufacturer` | `Option<ManufacturerId>` |
| `manufacture_date` | `Option<ManufactureDate>` |
| `edid_version` | `Option<EdidVersion>` |
| `product_code` | `Option<u16>` |
| `serial_number` | `Option<u32>` |
| `serial_number_string` | `Option<MonitorString>` |
| `display_name` | `Option<MonitorString>` |
| `unspecified_text` | `[Option<MonitorString>; 4]` |
| `white_points` | `[Option<WhitePoint>; 2]` |
| `digital` | `bool` |
| `color_bit_depth` | `Option<ColorBitDepth>` |
| `video_interface` | `Option<VideoInterface>` |
| `analog_sync_level` | `Option<AnalogSyncLevel>` |
| `chromaticity` | `Chromaticity` |
| `gamma` | `Option<DisplayGamma>` |
| `display_features` | `Option<DisplayFeatureFlags>` |
| `digital_color_encoding` | `Option<DigitalColorEncoding>` |
| `analog_color_type` | `Option<AnalogColorType>` |
| `screen_size` | `Option<ScreenSize>` |
| `preferred_image_size_mm` | `Option<(u16, u16)>` |
| `min_v_rate` / `max_v_rate` | `Option<u16>` |
| `min_h_rate_khz` / `max_h_rate_khz` | `Option<u16>` |
| `max_pixel_clock_mhz` | `Option<u16>` |
| `timing_formula` | `Option<TimingFormula>` |
| `color_management` | `Option<ColorManagementData>` |
| `warnings` | `[Option<EdidWarning>; 8]` (first 8; use `iter_warnings()`) |

These fields are absent from `DisplayCapabilities` without `alloc` or `std`:

| Field | Reason |
|-------|--------|
| `supported_modes` | Variable-length list of video modes |
| `extension_data` | Type-erased handler data via `Arc<dyn ExtensionData>` |
| `warnings` (full) | `Vec<ParseWarning>` in alloc builds; use `iter_warnings()` for portable access |

**For supported modes without heap allocation**, use `capabilities_from_edid_static` instead.
It returns `StaticDisplayCapabilities<N>`, which holds all the scalar fields above plus a
fixed-capacity `[Option<VideoMode>; N]` array accessible via `iter_modes()`.

Fixed-length string fields (`MonitorString`, `ManufacturerId`) use fixed-size byte
array newtypes with `Display` and `Deref<Target = str>` impls, so they behave like
strings in all build configurations without requiring heap allocation.

## Base block decoding

Fields decoded by `BaseBlockHandler`:

| Field | Source | Notes |
|-------|--------|-------|
| Manufacturer ID | bytes `0x08`–`0x09` | Three-character PNP code; `InvalidManufacturerId` warning if out of range |
| Product code | bytes `0x0A`–`0x0B` | 16-bit little-endian |
| Serial number | bytes `0x0C`–`0x0F` | 32-bit little-endian |
| Manufacture date | bytes `0x10`–`0x11` | Week + year, model year, or unspecified |
| EDID version | bytes `0x12`–`0x13` | Version and revision |
| Input type | byte `0x14` | Digital/analog flag; interface type, color bit depth (digital); sync level (analog) |
| Screen size | bytes `0x15`–`0x16` | Physical dimensions in cm, or landscape/portrait aspect ratio |
| Chromaticity | bytes `0x19`–`0x22` | 10-bit CIE xy coordinates for R, G, B, and white point |
| Display gamma | byte `0x17` | Encoded as `(value + 100) / 100`; absent if byte is `0xFF` |
| Display features | byte `0x18` | DPMS states, preferred timing mode, sRGB default, continuous timings |
| Color encoding | byte `0x18` bits 4–3 | RGB/YCbCr variants for EDID 1.4+ digital; analog color type otherwise |
| Established timings I/II | bytes `0x23`–`0x25` | Bitmap of 17 legacy modes decoded as `VideoMode` entries |
| Established timings III | descriptor `0xF7` | Extended bitmap of 44 additional VESA modes |
| Standard timings | bytes `0x26`–`0x35` | Up to 8 resolution + refresh rate pairs decoded as `VideoMode` entries |
| Detailed timing descriptors | slots `0x36`, `0x48`, `0x5A`, `0x6C` | Full DTD parameters decoded as `VideoMode`; first non-zero image size sets `preferred_image_size_mm` |
| Monitor name | descriptor `0xFC` | Display name string |
| Serial number string | descriptor `0xFF` | Serial number as text |
| Unspecified text | descriptor `0xFE` | Manufacturer-defined ASCII string |
| Display range limits | descriptor `0xFD` | Min/max H and V rates, max pixel clock, GTF/CVT timing formula |
| Additional white points | descriptor `0xFB` | Up to two additional white point entries with optional gamma |
| Color management data | descriptor `0xF9` | DCM polynomial coefficients for R, G, and B channels |

## CEA-861 coverage

Data blocks decoded by `Cea861Handler`:

| Tag | Block | Notes |
|-----|-------|-------|
| `0x01` | Audio Data Block | Short Audio Descriptors (SADs) |
| `0x02` | Video Data Block | VICs 1–127 (standard SVDs) and 128–255 (extended SVDs) |
| `0x03` | Vendor-Specific Data Block | HDMI 1.x VSDB (OUI `0x000C03`); HDMI Forum VSDB (OUI `0xC45DD8`) |
| `0x04` | Speaker Allocation Data Block | Three-byte channel presence bitmask |
| `0x05` | VESA Display Transfer Characteristic | 8/10/12-bit packed luminance points |
| `0x07` ext `0x00` | Video Capability Data Block | Quantization range and overscan flags |
| `0x07` ext `0x01` | Vendor-Specific Video Data Block | IEEE OUI + opaque vendor payload (e.g. Dolby Vision) |
| `0x07` ext `0x02` | VESA Display Device Data Block | Interface type, clock range, native resolution, audio, color depth |
| `0x07` ext `0x03` | VESA Video Timing Block Extension | DTBs, CVT, and Standard Timing entries as `VideoMode` |
| `0x07` ext `0x05` | Colorimetry Data Block | xvYCC, sYCC, opRGB, BT.2020 variants |
| `0x07` ext `0x06` | HDR Static Metadata Data Block | EOTFs and luminance levels |
| `0x07` ext `0x07` | HDR Dynamic Metadata Data Block | HDR10+, Dolby Vision application types |
| `0x07` ext `0x0D` | Video Format Preference Data Block | Short Video References (SVRs) |
| `0x07` ext `0x0E` | YCbCr 4:2:0 Video Data Block | 4:2:0-only VICs |
| `0x07` ext `0x0F` | YCbCr 4:2:0 Capability Map | Per-VIC 4:2:0 capability bitmap |
| `0x07` ext `0x12` | HDMI Audio Data Block | Multi-stream audio flag and embedded SADs |
| `0x07` ext `0x13` | Room Configuration Data Block | Speaker count and location availability |
| `0x07` ext `0x14` | Speaker Location Data Block | Per-channel assignment and distance |
| `0x07` ext `0x11` | Vendor-Specific Audio Data Block | IEEE OUI + opaque vendor payload |
| `0x07` ext `0x22` | DisplayID Type VII Video Timing Data Block | Single 20-byte DisplayID timing descriptor decoded to `VideoMode` |
| `0x07` ext `0x23` | DisplayID Type VIII Video Timing Data Block | VESA DMT ID codes decoded via built-in 0x01–0x58 lookup table |
| `0x07` ext `0x2A` | DisplayID Type X Video Timing Data Block | CVT formula-based timings; 6/7/8-byte descriptors; refresh up to 1024 Hz |
| `0x07` ext `0x78` | HDMI Forum EDID Extension Override Data Block | 1-byte extension count override for HDMI 2.1 sinks |
| `0x07` ext `0x79` | HDMI Forum Sink Capability Data Block | FRL rate, SCDC, Deep Color 4:2:0, ALLM, VRR range, DSC capabilities |
| `0x07` ext `0x20` | InfoFrame Data Block | Short InfoFrame Descriptors with OUI for VSI |

## Documentation

Design and architecture notes live under [`doc/`](doc/):

- [`doc/architecture.md`](doc/architecture.md) — pipeline and layer overview
- [`doc/model.md`](doc/model.md) — data model and type design
- [`doc/extensibility.md`](doc/extensibility.md) — extension system guide
- [`doc/static-pipeline.md`](doc/static-pipeline.md) — static pipeline design and API reference
- [`doc/scope.md`](doc/scope.md) — scope and evolution strategy
- [`doc/testing.md`](doc/testing.md) — testing strategy and fuzzing
- [`doc/cea861-vsdb.md`](doc/cea861-vsdb.md) — VSDB wire formats (HDMI 1.x and HDMI Forum)
- [`doc/cea861-extended-tags.md`](doc/cea861-extended-tags.md) — extended tag block wire formats
- [`doc/roadmap.md`](doc/roadmap.md) — planned features and future work
