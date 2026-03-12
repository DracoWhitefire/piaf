# Data model

PIAF should keep a clear separation between parsed source data and normalized capability output.

## Parsed representation

The parsed representation should stay close to the source structure.

Example direction:

```rust
pub struct ParsedEdid {
    pub base_block: BaseEdidBlock,
    pub extensions: Vec,
    pub warnings: Vec,
}
```

This structure is useful for:

- debugging,
- inspecting exact decoded content,
- preserving information that may not fit neatly into a simplified model,
- supporting future extensions.

## Capability representation

The capability model should be stable and ergonomic for downstream code.

Example direction:

```rust
pub struct DisplayCapabilities {
    pub manufacturer: Option,
    pub product_code: Option,
    pub serial_number: Option,
    pub display_name: Option,
    pub digital: bool,
    pub width_cm: Option,
    pub height_cm: Option,
    pub supported_modes: Vec,
    pub color_formats: Vec,
    pub audio: Option,
    pub warnings: Vec,
}
```

## Why separate them

A parser-oriented structure and a consumer-oriented structure serve different purposes.

`ParsedEdid` should prioritize fidelity to the source data.

`DisplayCapabilities` should prioritize:

- ease of use,
- semantic clarity,
- stability across parser improvements.

Trying to use one structure for both usually produces an API that is awkward for everyone.

## Error and warning model

Errors and warnings should be distinct.

Example direction:

```rust
pub enum EdidError {
    InvalidLength,
    InvalidHeader,
    ChecksumMismatch,
    TruncatedExtensionBlock,
    UnsupportedEncoding,
}

pub enum EdidWarning {
    UnknownExtension(u8), 
    DescriptorParseFailed,
    InconsistentDimensions,
    ReservedBitsSet,
}
```

This separation allows callers to decide how strict they want to be without losing useful diagnostic detail.
