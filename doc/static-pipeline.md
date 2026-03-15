# Static Extension Handler Pipeline

This document describes the design and implementation plan for adding a no_alloc-compatible
extension handler pipeline to PIAF.

## Motivation

The dynamic extension handler pipeline — which extracts display capabilities from CEA-861
extension blocks — is currently unavailable in bare `no_std` builds (no allocator) because it
relies on `Vec<Box<dyn ExtensionHandler>>` for handler registration and `Vec<VideoMode>` in
`DisplayCapabilities` for output.

Firmware targets with an RTOS allocator (the `alloc` feature tier) also lose the pipeline today,
since `ExtensionLibrary::with_standard_handlers` builds a `Vec<Box<dyn ExtensionHandler>>` but
the `capabilities_from_edid` function is still available — the gap is that users cannot register
handlers without `Box`.

The goal is a parallel pipeline that:

- Requires **zero heap allocation** at any tier
- Shares all internal parsing logic with the existing alloc path (no duplicate business logic)
- Lets users register their own handlers in no_alloc contexts using static instances
- Ships pre-built statics for CEA-861 (and future DisplayID) so the common case is one line

## Design

### The core unlock: static trait objects

`Box<dyn Trait>` requires allocation. `&'static dyn Trait` does not. A slice of static
references is allocation-free and works in bare no_std:

```rust
static MY_HANDLER: MyCustomHandler = MyCustomHandler;
static HANDLERS: &[&dyn piaf::StaticExtensionHandler] = &[
    &piaf::Cea861Handler,
    &MY_HANDLER,
];
```

This is the idiomatic embedded-Rust pattern for vtable dispatch without heap allocation.

### Three new types

#### `ModeSink` trait

Abstracts the write target for mode and warning output. Implemented by both
`DisplayCapabilities` (alloc, writes to `Vec`) and `StaticDisplayCapabilities<N>` (no_alloc,
writes to a fixed array). Object-safe, so it can be passed as `&mut dyn ModeSink`.

```rust
pub trait ModeSink {
    fn push_mode(&mut self, mode: VideoMode);
    fn push_warning(&mut self, w: EdidWarning);
}
```

#### `StaticExtensionHandler` trait

The no_alloc counterpart to `ExtensionHandler`. Available unconditionally (no cfg gate).
Object-safe so `&dyn StaticExtensionHandler` works in a static slice.

```rust
pub trait StaticExtensionHandler {
    fn tag(&self) -> u8;
    fn process(&self, block: &[u8; 128], sink: &mut dyn ModeSink);
}
```

The `tag()` method makes each handler self-describing, which doubles as the `KnownExtensions`
implementation for slices — no separate `ExtensionTagRegistry` needed:

```rust
impl KnownExtensions for [&dyn StaticExtensionHandler] {
    fn is_known(&self, tag: u8) -> bool {
        self.iter().any(|h| h.tag() == tag)
    }
}
```

#### `StaticDisplayCapabilities<const MAX_MODES: usize>`

A fixed-capacity output type. Contains all scalar fields from `DisplayCapabilities` verbatim,
plus:

```rust
pub supported_modes: [Option<VideoMode>; MAX_MODES],
pub num_modes: usize,
pub warnings: [Option<EdidWarning>; 8],
pub num_warnings: usize,
// No extension_data — the static pipeline is for mode extraction
```

`[Option<VideoMode>; MAX_MODES]` rather than `[VideoMode; MAX_MODES]` because `VideoMode` is
not `Copy`, and `None`-initialized arrays are the only const-constructible option. `Default` is
derived: `Option<T>: Default` always, so `[Option<VideoMode>; N]: Default` is satisfied.

Modes beyond `MAX_MODES` are silently dropped — the same philosophy as the existing 8-warning
cap. A typical display declares 20–40 modes; 64 is a safe default.

### New entry point

```rust
pub fn capabilities_from_edid_static<const N: usize>(
    parsed: &ParsedEdid,
    handlers: &[&dyn StaticExtensionHandler],
) -> StaticDisplayCapabilities<N>
```

No feature gate. Available at all tiers. In bare no_std, extension blocks do not exist in
`ParsedEdid` (that field is alloc-gated), so the result contains base-block data only — still
a significant improvement over the current bare no_std experience.

### Pre-built statics

```rust
pub static CEA861_HANDLER: &dyn StaticExtensionHandler = &Cea861Handler;
pub static STANDARD_HANDLERS: &[&dyn StaticExtensionHandler] = &[&Cea861Handler];
```

`Cea861Handler` is already a unit struct. Making it unconditionally available (removing its
alloc cfg gate) is the only structural change to an existing type.

### Resulting usage in bare no_std

```rust
let parsed = parse_edid(&bytes, piaf::STANDARD_HANDLERS)?;
let caps: piaf::StaticDisplayCapabilities<64> =
    piaf::capabilities_from_edid_static(&parsed, piaf::STANDARD_HANDLERS);

for mode in caps.iter_modes() {
    // ...
}
```

---

## Implementation Steps

Steps are ordered for minimum regression risk. Steps 1–3 are pure additions; step 4 is the
highest-risk step and should be validated with a full test run before proceeding.

### Step 1 — `ModeSink` trait

**File:** `src/model/capabilities.rs`

Add the trait unconditionally before `DisplayCapabilities`. Implement for `DisplayCapabilities`
under the existing alloc cfg gate — the impl calls `self.supported_modes.push(mode)` with the
same dedup logic currently in the alloc handler call sites, and wraps the `EdidWarning` in
`Arc::new` when pushing to `self.warnings`.

Export from `src/model/mod.rs` and `src/lib.rs` unconditionally.

**Verify:** `cargo build --no-default-features` and `cargo build` succeed.

---

### Step 2 — `StaticDisplayCapabilities<const MAX_MODES: usize>`

**File:** `src/model/capabilities.rs`

Add the struct unconditionally. Copy all fixed-size fields from `DisplayCapabilities` verbatim.
Add mode and warning arrays with cursor fields.

Implement `ModeSink for StaticDisplayCapabilities<N>`: dedup in `push_mode`, silent drop on
overflow for both modes and warnings.

Add convenience methods:

```rust
pub fn iter_modes(&self) -> impl Iterator<Item = &VideoMode> {
    self.supported_modes[..self.num_modes].iter().flatten()
}
pub fn iter_warnings(&self) -> impl Iterator<Item = &EdidWarning> {
    self.warnings.iter().flatten()
}
```

Export from `src/model/mod.rs` and `src/lib.rs` unconditionally.

**Verify:** `cargo build --no-default-features` succeeds.

---

### Step 3 — `StaticExtensionHandler` trait and `KnownExtensions` for slices

**File:** `src/model/extension.rs`

Add the trait and the `KnownExtensions` impl for `[&dyn StaticExtensionHandler]`
unconditionally.

Export `StaticExtensionHandler` from `src/model/mod.rs` and `src/lib.rs` unconditionally.

**Verify:** `cargo build --no-default-features` succeeds.

---

### Step 4 — Extract sink-based parsing logic

**Files:** `src/capabilities/cea861/mod.rs`, `src/capabilities/base/timings.rs`

This is the most delicate step. Add a `pub(crate) fn cea861_process_into_sink` that contains
all mode-producing CEA-861 logic, ungated:

- SVD Video Data Block → VIC lookup → `sink.push_mode`
- Y420 VDB VIC lookups
- VTB-EXT, T7VTDB, T8VTDB, T10VTDB modes
- DTDs from the end of the CEA extension block
- `sink.push_warning` for malformed data blocks

Excludes everything that builds `Cea861Capabilities` fields (audio, VSDB, HF-VSDB,
colorimetry, HDR) — those remain alloc-only.

Refactor the alloc `ExtensionHandler::process` impl to delegate its mode-extraction half to
`cea861_process_into_sink`. No behavioral change — it calls the same code via the sink
abstraction, since `DisplayCapabilities: ModeSink`.

Similarly add `pub(crate) fn decode_dtd_slot_into_sink(dtd: &[u8], sink: &mut dyn ModeSink)`
alongside the existing `decode_dtd_slot`. The only omission is writing
`preferred_image_size_mm` — that is a base-block concern handled separately in step 6.

**Verify:** `cargo test --all-features` passes before proceeding to step 5.

---

### Step 5 — Ungated timing functions for the base block

**Files:** `src/capabilities/base/timings.rs`, `src/capabilities/base/descriptors.rs`

Change `decode_established_timings`, `decode_standard_timings`, and
`decode_detailed_timings` from `(base, &mut DisplayCapabilities)` to
`(base, &mut dyn ModeSink)`, removing their alloc cfg gates. The single call site
(`BaseBlockHandler::process`) passes `caps as &mut dyn ModeSink` — no test changes needed.

In `descriptors.rs`, split `decode_descriptors` into:

- `decode_descriptors_meta(base, caps: &mut DisplayCapabilities)` — non-mode fields (serial
  string, display name, range limits, white points, CMData). Remains alloc-gated.
- `decode_descriptors_modes(base, sink: &mut dyn ModeSink)` — mode-producing descriptor types
  `0xF7`, `0xF8`, `0xFA`. Ungated.

**Verify:** `cargo test --all-features` passes.

---

### Step 6 — `capabilities_from_edid_static`

**File:** `src/capabilities/mod.rs`

Add the function unconditionally. Implementation:

1. Create a temporary `DisplayCapabilities::default()` on the stack. In bare no_std this is
   entirely stack-allocated — no `Vec` fields exist at that tier.
2. Call `decode_base_block` / `BaseBlockHandler::process` (cfg-gated) to populate it.
3. Copy all scalar fields from the temporary into `StaticDisplayCapabilities`.
4. Populate modes:
   - **alloc:** copy `base_caps.supported_modes` via `sink.push_mode`
   - **no_std:** call the ungated `decode_established_timings_into_sink`,
     `decode_standard_timings_into_sink`, `decode_detailed_timings_into_sink` directly
5. Copy base-block warnings.
6. Iterate extension blocks (alloc-gated — no extension blocks in bare no_std `ParsedEdid`),
   dispatching each block to the first handler with a matching tag.

The temporary `DisplayCapabilities` intermediary avoids duplicating all base-block decode
logic. The "copy scalar fields" step is a fixed, explicit list of assignments — new fields
added to `DisplayCapabilities` will be visibly absent from the copy if forgotten.

Export `capabilities_from_edid_static` from `src/lib.rs` unconditionally.

**Verify:** `cargo build --no-default-features` (base-block-only result), `cargo build`
(full result with extension support).

---

### Step 7 — `StaticExtensionHandler` impl and pre-built statics

**File:** `src/capabilities/cea861/mod.rs`

Remove the alloc cfg gate from the `Cea861Handler` struct definition (it is already a unit
struct). Add:

```rust
impl StaticExtensionHandler for Cea861Handler {
    fn tag(&self) -> u8 { 0x02 }
    fn process(&self, block: &[u8; 128], sink: &mut dyn ModeSink) {
        cea861_process_into_sink(block, sink);
    }
}

pub static CEA861_HANDLER: &dyn StaticExtensionHandler = &Cea861Handler;
pub static STANDARD_HANDLERS: &[&dyn StaticExtensionHandler] = &[&Cea861Handler];
```

`CEA861_HANDLER` and `STANDARD_HANDLERS` must be `static`, not `const` — fat pointer
coercions to trait objects are not const-stable in current Rust.

The existing `ExtensionHandler` (alloc) impl on `Cea861Handler` is unchanged.

Update `src/capabilities/mod.rs` to export `CEA861_HANDLER`, `STANDARD_HANDLERS`, and
`Cea861Handler` (removing the alloc gate from the last one). Update `src/lib.rs` to match.

---

### Step 8 — Public API surface

**File:** `src/lib.rs`

Final additions, all unconditional unless noted:

```rust
pub use model::StaticDisplayCapabilities;
pub use model::StaticExtensionHandler;
pub use model::ModeSink;
pub use capabilities::capabilities_from_edid_static;
pub use capabilities::{CEA861_HANDLER, STANDARD_HANDLERS};
pub use capabilities::Cea861Handler;  // remove alloc gate
```

---

### Step 9 — Tests

New tests to add in `src/capabilities/mod.rs` (or a new `tests/static_pipeline.rs`):

| Test | What it checks |
|------|----------------|
| `test_static_base_block_modes` | Established + standard timings appear in static caps |
| `test_static_cea861_svds` | VIC 16 (1080p60) from an SVD appears after CEA extension |
| `test_static_mode_cap_exceeded` | `num_modes == MAX_MODES`, no panic on overflow |
| `test_static_warning_cap` | Malformed CEA block produces `EdidWarning` in `iter_warnings()` |
| `test_static_known_extensions` | `STANDARD_HANDLERS.is_known(0x02)` true, `0x70` false |
| `test_static_same_modes_as_alloc` | Same CEA block, both pipelines produce identical mode lists |

In `src/capabilities/cea861/mod.rs`:

| Test | What it checks |
|------|----------------|
| `test_static_handler_tag` | `Cea861Handler.tag() == 0x02` |

---

## Risks and Edge Cases

**Stack size of `StaticDisplayCapabilities`.** At `MAX_MODES = 64`, the struct is roughly
3 KB. Very deep call stacks on Cortex-M0 targets with 1 KB stacks will overflow. Document the
sizing tradeoff and recommend placing the result in a `static mut` if stack pressure is a
concern.

**`preferred_image_size_mm` not populated from CEA DTDs.** The sink-based CEA DTD path omits
this field — it is a base-block concern and is populated correctly through the temporary
`DisplayCapabilities` intermediary. This is an acceptable and documented limitation.

**Dual-pipeline confusion.** A caller using both `capabilities_from_edid` and
`capabilities_from_edid_static` on the same EDID will process extension blocks twice. Document
clearly: the static function is a replacement for the dynamic one, not a complement.

**`[Option<VideoMode>; MAX_MODES]: Default` bounds.** At `MAX_MODES = 0` the struct is valid
but useless. At very large values (e.g. 1024) the struct is large enough to overflow small
stacks. These are caller responsibilities, not library bugs.

**Future DisplayID support.** Adding DisplayID to the static pipeline requires only:
1. Implementing `StaticExtensionHandler` for a new `DisplayIdHandler` struct
2. Adding it to `STANDARD_HANDLERS`

No structural changes to the pipeline are needed.
