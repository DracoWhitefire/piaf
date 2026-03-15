# Static Extension Handler Pipeline

The static pipeline is a zero-allocation alternative to the dynamic
[`capabilities_from_edid`][crate::capabilities_from_edid] pipeline. It produces
[`StaticDisplayCapabilities`][crate::StaticDisplayCapabilities] — a fixed-capacity output type
— and works at all build tiers, including bare `no_std` without an allocator.

## Motivation

The dynamic pipeline relies on `Vec<Box<dyn ExtensionHandler>>` for handler registration and
`Vec<VideoMode>` for output. Neither is available in bare `no_std` builds. Firmware targets with
an RTOS allocator (the `alloc` feature tier) can allocate, but still cannot register handlers
without `Box`.

The static pipeline solves both problems at once.

## Usage

```rust
let parsed = piaf::parse_edid(&bytes, piaf::STANDARD_HANDLERS)?;
let caps: piaf::StaticDisplayCapabilities<64> =
    piaf::capabilities_from_edid_static(&parsed, piaf::STANDARD_HANDLERS);

for mode in caps.iter_modes() {
    // ...
}
```

`STANDARD_HANDLERS` is a `&[&dyn StaticExtensionHandler]` covering CEA-861. It also implements
[`KnownExtensions`][crate::KnownExtensions], so it can be passed directly to `parse_edid` — no
separate `ExtensionTagRegistry` needed.

## Public API

### `capabilities_from_edid_static`

```rust
pub fn capabilities_from_edid_static<const N: usize>(
    parsed: &ParsedEdid,
    handlers: &[&dyn StaticExtensionHandler],
) -> StaticDisplayCapabilities<N>
```

Available unconditionally. Decodes the base block through the same logic as
`capabilities_from_edid`, then dispatches each extension block to the first handler whose
`tag()` matches. In bare `no_std` builds, extension blocks are not stored in `ParsedEdid`, so
the result contains base-block data only.

`capabilities_from_edid_static` is a **replacement** for `capabilities_from_edid`, not a
complement. Calling both on the same EDID will process extension blocks twice.

### `StaticDisplayCapabilities<const MAX_MODES: usize>`

Contains all scalar fields from `DisplayCapabilities` (manufacturer, timings, chromaticity,
etc.) plus fixed-capacity arrays for modes and warnings:

```rust
pub supported_modes: [Option<VideoMode>; MAX_MODES],
pub num_modes: usize,
pub warnings: [Option<EdidWarning>; 8],
pub num_warnings: usize,
```

Convenience iterators:

```rust
pub fn iter_modes(&self) -> impl Iterator<Item = &VideoMode>
pub fn iter_warnings(&self) -> impl Iterator<Item = &EdidWarning>
```

Modes and warnings beyond capacity are silently dropped. 64 is a reasonable default for
`MAX_MODES`; real displays rarely declare more than ~40 modes.

### `StaticExtensionHandler`

The no-alloc counterpart to `ExtensionHandler`. Object-safe; `&dyn StaticExtensionHandler`
works in a static slice.

```rust
pub trait StaticExtensionHandler: Sync {
    fn tag(&self) -> u8;
    fn process(&self, block: &[u8; 128], sink: &mut dyn ModeSink);
}
```

`tag()` makes each handler self-describing. A `&[&dyn StaticExtensionHandler]` slice implements
[`KnownExtensions`][crate::KnownExtensions] via the same method, so the same slice serves both
as a handler list and an extension tag filter for `parse_edid`.

### `ModeSink`

Internal trait abstracting mode and warning writes. Implemented by both `DisplayCapabilities`
(alloc) and `StaticDisplayCapabilities<N>` (no-alloc). Exposed publicly so custom handler
implementations can accept it.

```rust
pub trait ModeSink {
    fn push_mode(&mut self, mode: VideoMode);
    fn push_warning(&mut self, w: EdidWarning);
}
```

Both impls deduplicate modes by (width, height, refresh_rate, interlaced): the first entry for
a given resolution and rate wins. A DTD-derived mode with full timing detail therefore takes
precedence over a later SVD-derived mode with sparse timing detail.

### Pre-built statics

```rust
pub static CEA861_HANDLER: &dyn StaticExtensionHandler = &Cea861Handler;
pub static STANDARD_HANDLERS: &[&dyn StaticExtensionHandler] = &[&Cea861Handler];
```

These are `static`, not `const` — fat pointer coercions to trait objects are not const-stable
in current Rust.

## Custom handlers

Implement `StaticExtensionHandler` for a unit struct, then pass it alongside the pre-built
handlers:

```rust
struct MyHandler;

impl piaf::StaticExtensionHandler for MyHandler {
    fn tag(&self) -> u8 { 0xAB }
    fn process(&self, block: &[u8; 128], sink: &mut dyn piaf::ModeSink) {
        // extract modes and push via sink.push_mode(...)
    }
}

static MY_HANDLER: MyHandler = MyHandler;
static HANDLERS: &[&dyn piaf::StaticExtensionHandler] = &[
    piaf::CEA861_HANDLER,
    &MY_HANDLER,
];
```

## What the static pipeline extracts from CEA-861

The static CEA-861 path (`Cea861Handler` via `StaticExtensionHandler::process`) extracts
**modes only**:

- Short Video Descriptors (SVDs) from the Video Data Block, including extended VICs (128–255)
- Y420 Video Data Block VIC lookups
- DTDs from the CEA extension block's detailed timing section
- Mode-producing extended blocks: VTB-EXT, T7VTDB, T8VTDB, T10VTDB

It does not extract audio descriptors, VSDB/HF-VSDB fields, colorimetry, HDR metadata, or
speaker allocation — those require the alloc pipeline and are available through
`Cea861Capabilities` via `capabilities_from_edid`.

## Limitations and sizing notes

**Stack size.** At `MAX_MODES = 64`, `StaticDisplayCapabilities` is roughly 3 KB on the stack.
On Cortex-M0 targets with small stacks, place the result in a `static mut` if stack pressure is
a concern.

**`preferred_image_size_mm`.** Populated correctly from the base block. Not populated from CEA
DTDs — that field is a base-block concern and does not belong to the extension pipeline.

**Extension blocks in bare `no_std`.** `ParsedEdid` does not store extension blocks at the bare
`no_std` tier (no `alloc` feature). `capabilities_from_edid_static` returns base-block data
only in that configuration; passing handlers has no effect.

**Future DisplayID support.** Adding DisplayID to the static pipeline requires only
implementing `StaticExtensionHandler` for a `DisplayIdHandler` struct and adding it to
`STANDARD_HANDLERS`. No structural changes to the pipeline are needed.
