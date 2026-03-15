# Data model

PIAF keeps a clear separation between parsed source data and normalized capability output.

## Parsed representation

`ParsedEdid` stays close to the source structure. The base block is preserved as a raw
byte array; extension blocks are stored the same way and dispatched by tag.

```rust
pub struct ParsedEdid {
    pub base_block: [u8; 128],
    // alloc/std only:
    pub extensions: Vec<[u8; 128]>,
    pub warnings: Vec<ParseWarning>,
}
```

In bare `no_std` builds (no `alloc`), `extensions` and `warnings` are absent. The base block
is always present.

This structure is useful for:

- debugging,
- inspecting exact decoded content,
- preserving information that may not fit neatly into a simplified model,
- supporting future extensions.

## Capability representation

PIAF provides two output types depending on the pipeline used.

### `DisplayCapabilities` (dynamic pipeline, `alloc`/`std`)

The consumer-facing output for the dynamic pipeline. Fields are `Option` where the source
data may be absent or undecodable. The `extension_data` field allows handlers to attach
typed custom data without modifying the struct.

```rust
pub struct DisplayCapabilities {
    // Identity
    pub manufacturer: Option<ManufacturerId>,
    pub manufacture_date: Option<ManufactureDate>,
    pub edid_version: Option<EdidVersion>,
    pub product_code: Option<u16>,
    pub serial_number: Option<u32>,
    pub serial_number_string: Option<String>,
    pub display_name: Option<String>,
    pub unspecified_text: Vec<String>,
    // Input
    pub digital: bool,
    pub color_bit_depth: Option<ColorBitDepth>,
    pub video_interface: Option<VideoInterface>,
    pub analog_sync_level: Option<AnalogSyncLevel>,
    // Color
    pub chromaticity: Chromaticity,
    pub gamma: Option<DisplayGamma>,
    pub display_features: Option<DisplayFeatureFlags>,
    pub digital_color_encoding: Option<DigitalColorEncoding>,
    pub analog_color_type: Option<AnalogColorType>,
    pub color_management: Option<ColorManagementData>,
    pub white_points: [Option<WhitePoint>; 2],
    // Physical
    pub screen_size: Option<ScreenSize>,
    pub preferred_image_size_mm: Option<(u16, u16)>,
    // Timing
    pub min_v_rate: Option<u16>,
    pub max_v_rate: Option<u16>,
    pub min_h_rate_khz: Option<u16>,
    pub max_h_rate_khz: Option<u16>,
    pub max_pixel_clock_mhz: Option<u16>,
    pub timing_formula: Option<TimingFormula>,
    pub supported_modes: Vec<VideoMode>,
    // Extensions
    // alloc/std: pub warnings: Vec<ParseWarning>,
    // no_std:    pub warnings: [Option<EdidWarning>; 8],
    pub extension_data: Vec<(u8, Arc<dyn ExtensionData>)>,
}
```

Rate fields (`min_v_rate`, `max_v_rate`, `min_h_rate_khz`, `max_h_rate_khz`) are `u16`
rather than `u8` because the `0xFD` range limits descriptor can add a 255-unit offset to
extend beyond the 8-bit range.

### `StaticDisplayCapabilities<const MAX_MODES: usize>` (static pipeline, all tiers)

The output type for `capabilities_from_edid_static`. Contains all the same scalar fields as
`DisplayCapabilities` (same names, same types), plus fixed-capacity arrays for modes and
warnings:

```rust
pub struct StaticDisplayCapabilities<const MAX_MODES: usize> {
    // All scalar fields identical to DisplayCapabilities
    pub manufacturer: Option<ManufacturerId>,
    // ...

    // Mode and warning storage
    pub supported_modes: [Option<VideoMode>; MAX_MODES],
    pub num_modes: usize,
    pub warnings: [Option<EdidWarning>; 8],
    pub num_warnings: usize,
}
```

Access modes and warnings through iterators rather than indexing directly:

```rust
for mode in caps.iter_modes() { /* &VideoMode */ }
for warn in caps.iter_warnings() { /* &EdidWarning */ }
```

Modes and warnings beyond capacity are silently dropped — matching the existing 8-warning cap
philosophy. 64 is a reasonable default for `MAX_MODES`.

`StaticDisplayCapabilities` does not have an `extension_data` field. Rich extension metadata
(audio, VSDB, colorimetry, HDR) is only available through the dynamic pipeline.

## Why separate them

A parser-oriented structure and a consumer-oriented structure serve different purposes.

`ParsedEdid` prioritizes fidelity to the source data.

`DisplayCapabilities` prioritizes:

- ease of use,
- semantic clarity,
- stability across parser improvements.

Trying to use one structure for both usually produces an API that is awkward for everyone.

## Error and warning model

Errors and warnings are distinct.

```rust
pub enum EdidError {
    InvalidLength,
    InvalidHeader,
    ChecksumMismatch,
}

pub enum EdidWarning {
    /// Extension block tag not in the registered set.
    UnknownExtension(u8),
    /// An 18-byte descriptor slot could not be decoded.
    DescriptorParseFailed,
    /// Manufacturer ID bytes outside the valid PNP range (1–26 per 5-bit field).
    /// `DisplayCapabilities::manufacturer` is left as `None`.
    InvalidManufacturerId,
    /// Byte slice length differs from `(1 + extension_count) × 128`.
    /// Extra bytes are ignored; too few is a hard `EdidError::InvalidLength`.
    SizeMismatch { expected: usize, actual: usize },
    /// A data block inside an extension block declared a length that extends past the
    /// end of the data block collection. Remaining data blocks are skipped.
    MalformedDataBlock,
    /// A DTD slot was skipped because the pixel clock value would overflow during
    /// refresh rate calculation. Indicates a malformed or corrupted EDID.
    DtdPixelClockOverflow,
}
```

This separation allows callers to decide how strict they want to be without losing useful
diagnostic detail. Warnings from the parser (including `UnknownExtension` and `SizeMismatch`)
are propagated into `DisplayCapabilities::warnings` alongside handler warnings, so consumers
have a single place to inspect all diagnostics.

### Extensible warnings (`alloc`/`std` builds)

In `alloc`/`std` builds, warnings are type-erased behind a `ParseWarning` alias:

```rust
pub type ParseWarning = Arc<dyn core::error::Error + Send + Sync + 'static>;
```

This means custom extension handlers can push their own error types into the warning list
without wrapping them in `EdidWarning`. The built-in library always emits `EdidWarning`
variants, but a third-party handler that detects a protocol-specific anomaly can emit its
own type directly.

Using `Arc` (rather than `Box`) keeps `ParseWarning` cloneable, which lets warnings be
copied from `ParsedEdid` into `DisplayCapabilities` without consuming the parsed result.

To inspect a specific variant, use `downcast_ref` on the inner error:

```rust
for w in caps.iter_warnings() {
    if let Some(ew) = (**w).downcast_ref::<EdidWarning>() {
        // handle known library warning
    }
}
```

In bare `no_std` builds (without `alloc`) the warning list holds `EdidWarning` values
directly (no type erasure), capped at 8 entries. The `iter_warnings()` method provides
uniform access across both configurations.

