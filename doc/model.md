# Data model

PIAF keeps a clear separation between parsed source data and normalized capability output.

## Parsed representation

`ParsedEdid` stays close to the source structure. The base block is preserved as a raw
byte array; extension blocks are stored the same way and dispatched by tag.

```rust
pub struct ParsedEdid {
    pub base_block: [u8; 128],
    pub extensions: Vec<[u8; 128]>,
    pub warnings: Vec<EdidWarning>,
}
```

This structure is useful for:

- debugging,
- inspecting exact decoded content,
- preserving information that may not fit neatly into a simplified model,
- supporting future extensions.

## Capability representation

`DisplayCapabilities` is the consumer-facing output. Fields are `Option` where the source
data may be absent or undecodable. The `extension_data` map allows handlers to attach
typed custom data without modifying the struct.

```rust
pub struct DisplayCapabilities {
    // Identity
    pub manufacturer: Option<String>,
    pub manufacture_date: Option<ManufactureDate>,
    pub edid_version: Option<EdidVersion>,
    pub product_code: Option<u16>,
    pub serial_number: Option<u32>,
    pub serial_number_string: Option<String>,
    pub display_name: Option<String>,
    // Input
    pub digital: bool,
    pub color_bit_depth: Option<ColorBitDepth>,
    pub video_interface: Option<VideoInterface>,
    // Color
    pub gamma: Option<DisplayGamma>,
    pub display_features: Option<DisplayFeatureFlags>,
    pub digital_color_encoding: Option<DigitalColorEncoding>,
    pub analog_color_type: Option<AnalogColorType>,
    // Physical
    pub width_cm: Option<u16>,
    pub height_cm: Option<u16>,
    // Timing
    pub min_v_rate: Option<u8>,
    pub max_v_rate: Option<u8>,
    pub min_h_rate_khz: Option<u8>,
    pub max_h_rate_khz: Option<u8>,
    pub max_pixel_clock_mhz: Option<u16>,
    pub supported_modes: Vec<VideoMode>,
    // Extensions
    pub has_audio: bool,
    pub warnings: Vec<EdidWarning>,
    pub extension_data: HashMap<u8, Arc<dyn ExtensionData>>,
}
```

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
    UnknownExtension(u8),
    DescriptorParseFailed,
}
```

This separation allows callers to decide how strict they want to be without losing useful
diagnostic detail. Warnings are collected from both the parser (into `ParsedEdid::warnings`)
and from extension handlers (into `DisplayCapabilities::warnings`).

## Planned additions

- Structured audio capabilities beyond the `has_audio` flag (CEA-861 Short Audio Descriptors)
- YCbCr color encoding details from CEA-861 (Short Video Descriptors)
