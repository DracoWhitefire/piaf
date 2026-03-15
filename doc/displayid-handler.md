# DisplayID Handler — Implementation Plan

DisplayID is the primary feature planned for version 0.2. This document records the agreed
design before implementation begins.

## The multi-block problem

CEA-861 extension blocks are self-contained: each 128-byte block at tag `0x02` is an
independent unit, and the current per-block dispatch in `capabilities_from_edid` and
`capabilities_from_edid_static` handles it correctly.

DisplayID is different. A single logical DisplayID section may span several consecutive
128-byte extension blocks, all tagged `0x70`. Naive per-block dispatch would call the
handler once per fragment with no awareness of the others, making reassembly impossible
without introducing statefulness into the handler interface.

The fix is to change dispatch so that a handler receives **all blocks with its tag at once**,
as an ordered slice. This applies to both pipelines and requires updating both handler traits.

## Handler trait changes

Both traits change their `process` signature from a single block reference to a slice:

```rust
// Dynamic pipeline (alloc/std)
pub trait ExtensionHandler: Debug {
    fn process(
        &self,
        blocks: &[&[u8; 128]],
        caps: &mut DisplayCapabilities,
        warnings: &mut Vec<ParseWarning>,
    );
}

// Static pipeline (all tiers)
pub trait StaticExtensionHandler: Sync {
    fn tag(&self) -> u8;
    fn process(&self, blocks: &[&[u8; 128]], ctx: &mut StaticContext<'_>);
}
```

These are breaking changes. They are made now, before 0.1 has external consumers, to avoid
a second breaking release.

**CEA-861** is unaffected in practice: each CEA-861 block is independent, so its handler
iterates the slice and processes each element exactly as before. The slice will always
contain a single element for the standard CEA-861 case, and multiple elements are handled
gracefully since each is self-contained.

**DisplayID** receives all its fragments in stream order and owns the reassembly logic.
No other handler or dispatch layer needs to know about DisplayID's internal structure.

## Dispatch changes

**Agnostic by design.** Dispatch collects blocks by tag and passes the slice. It has no
knowledge of whether a handler is single-block or multi-block — that is the handler's
concern. If a future extension has a different multi-block structure, its handler handles it.

### Dynamic pipeline (`capabilities_from_edid`)

Before calling handlers, group extension blocks by tag into a `HashMap<u8, Vec<&[u8;128]>>`.
Then dispatch each registered handler once with its group:

```rust
let mut groups: HashMap<u8, Vec<&[u8; 128]>> = HashMap::new();
for ext in edid.extension_blocks() {
    groups.entry(ext[0]).or_default().push(ext);
}
for metadata in &library.extensions {
    if let Some(handler) = &metadata.handler {
        if let Some(blocks) = groups.get(&metadata.tag) {
            handler.process(blocks, &mut caps, &mut warnings);
        }
    }
}
```

### Static pipeline (`capabilities_from_edid_static`)

No allocator is available, so grouping uses a fixed-size stack array. For each handler,
scan the extension blocks and collect matching references:

```rust
for handler in handlers {
    let mut group: [MaybeUninit<&[u8; 128]>; MAX_EXTENSION_BLOCKS] =
        MaybeUninit::uninit_array();
    let mut count = 0;
    for ext in parsed.extension_blocks() {
        if ext[0] == handler.tag() {
            group[count].write(ext);
            count += 1;
        }
    }
    if count > 0 {
        let slice = unsafe { MaybeUninit::slice_assume_init_ref(&group[..count]) };
        let mut ctx = StaticContext::new(&mut caps);
        handler.process(slice, &mut ctx);
    }
}
```

Stack cost: `MAX_EXTENSION_BLOCKS × size_of::<&[u8; 128]>()` — 512 bytes on 64-bit, less
on 32-bit embedded targets. This is a call-frame-local cost, not a persistent allocation,
and is acceptable for all firmware targets that handle DisplayPort enumeration.
`MaybeUninit::uninit_array` and `MaybeUninit::slice_assume_init_ref` are stable since
Rust 1.82.

## `StaticContext` — the extensible sink bag

The static handler previously received `&mut dyn ModeSink` directly. Replacing it with
`StaticContext<'_>` makes the output side extensible without future breaking changes.

`StaticContext` lives in `capabilities.rs`, alongside `ModeSink` and
`StaticDisplayCapabilities`. It is an output-side type — the thing handlers write *to* —
and belongs with its peers, not in `extension.rs` which is concerned with handler
registration and dispatch infrastructure.

```rust
pub struct StaticContext<'a> {
    modes: &'a mut dyn ModeSink,
    // Future fields, e.g.:
    // identity: Option<&'a mut dyn IdentitySink>,
}

impl<'a> StaticContext<'a> {
    pub fn new(modes: &'a mut dyn ModeSink) -> Self {
        Self { modes }
    }

    pub fn push_mode(&mut self, mode: VideoMode) {
        self.modes.push_mode(mode);
    }

    pub fn push_warning(&mut self, w: EdidWarning) {
        self.modes.push_warning(w);
    }
}
```

Handlers call output through methods, not through direct field access. When a new sink
type is added to `StaticContext` as `Option<&'a mut dyn XxxSink>`, existing handler
implementations are unaffected — the new field defaults to `None` and handlers that do
not need it simply ignore it.

The dynamic pipeline's output type, `DisplayCapabilities`, already plays this role for
`ExtensionHandler`: it is the rich context that handlers write into. No equivalent change
is needed there.

## Block ordering and block map interaction

`EdidSource::extension_blocks()` yields all extension blocks in stream order — the physical
order they appear in the EDID byte stream. Block Map blocks (`0xF0`) are present in this
stream but are filtered out naturally when collecting for tag `0x70`, since their tags
differ.

Dispatch collects DisplayID fragments in stream order, which is the correct reassembly
order. The DisplayID specification requires fragments to be contiguous and ordered; trusting
the stream is both correct and simpler than consulting the block map.

Block-map validation — verifying the count and positions of `0x70` blocks against the map
— is a future additive check. It does not change what the handler receives; it would
surface as a diagnostic warning if the block map is inconsistent with what was found. This
design does not block that addition.

## DisplayID handler structure

The handler will live in `src/capabilities/displayid/`. The initial implementation covers
DisplayID 1.x embedded in EDID extension blocks.

**Fragment layout** (DisplayID 1.x):

```
Byte 0:     Tag (0x70)
Byte 1:     DisplayID version
Byte 2:     Payload length in this section
Byte 3:     Display product type / extension count
Bytes 4..N: DisplayID data blocks (variable-length)
Last byte:  Checksum
```

The handler receives the raw 128-byte EDID blocks. It is responsible for:

1. Validating the tag and version on the first block.
2. Reading the extension count to know how many fragments to expect; warning if the slice
   length does not match.
3. Iterating the logical DisplayID data blocks across the reassembled payload.
4. Dispatching each logical block by its DisplayID block tag to an internal decoder.

For the static pipeline, the handler pushes decoded video modes via `ctx.push_mode()`.
For the dynamic pipeline, it additionally writes rich data (product identity, interface
features, colorimetry) into `DisplayCapabilities` via `set_extension_data`.

The data type stored via `set_extension_data` will be `DisplayIdCapabilities` — a new
struct in `src/capabilities/displayid/` analogous to `Cea861Capabilities`.

## Files affected

| File | Change |
|---|---|
| `src/model/extension.rs` | `ExtensionHandler::process` signature |
| `src/model/capabilities.rs` | Add `StaticContext`; `StaticExtensionHandler::process` signature |
| `src/capabilities/mod.rs` | Dispatch logic in both pipeline functions |
| `src/capabilities/cea861/mod.rs` | Update to new slice signature |
| `src/capabilities/base/mod.rs` | Update if affected |
| `src/capabilities/displayid/` | New module: handler, capabilities struct, block decoders |
| `doc/extensibility.md` | Update code examples for new signatures |
| `doc/static-pipeline.md` | Update signatures and remove stale claim in Limitations section |

## Implementation checklist

Steps are ordered so each builds on the last. Remove this section once all items are done.

### Infrastructure

- [ ] Add `StaticContext<'a>` to `src/model/capabilities.rs`
- [ ] Change `ExtensionHandler::process` in `src/model/extension.rs` to take `&[&[u8; 128]]`
- [ ] Change `StaticExtensionHandler::process` in `src/model/extension.rs` to take `(&[&[u8; 128]], &mut StaticContext<'_>)`

### Dispatch

- [ ] Replace per-block dispatch in `capabilities_from_edid` with group-by-tag pre-pass (`HashMap`)
- [ ] Update base handler call site — wrap `edid.base_block()` in a one-element slice
- [ ] Replace per-block dispatch in `capabilities_from_edid_static` with per-handler scan into `[MaybeUninit; MAX_EXTENSION_BLOCKS]`

### Existing handler updates

- [ ] Update `BaseBlockHandler` (`src/capabilities/base/mod.rs`) — extract `blocks[0]`, rest unchanged
- [ ] Update `Cea861Handler` dynamic impl — iterate slice, process each block as before
- [ ] Update `Cea861Handler` static impl — iterate slice with `StaticContext`

### DisplayID handler

- [ ] Create `src/capabilities/displayid/mod.rs`; add `mod displayid` to `src/capabilities/mod.rs`
- [ ] Implement fragment validation: check tag, version, extension count against slice length; emit warnings on mismatch
- [ ] Implement logical block iteration across the reassembled payload
- [ ] Define `DisplayIdCapabilities` struct
- [ ] Implement data block decoders (timing blocks first: Type I, Type VII; identity and interface blocks after)
- [ ] Wire `DisplayIdHandler` into `with_standard_handlers()` (dynamic pipeline)
- [ ] Wire `DisplayIdHandler` into `STANDARD_HANDLERS` (static pipeline)

### Docs and tests

- [ ] Update code examples in `doc/extensibility.md` for new signatures
- [ ] Update `doc/static-pipeline.md`: new signatures, remove stale Limitations claim
- [ ] Unit tests for fragment reassembly and each data block decoder
- [ ] Integration test with a real DisplayID EDID fixture in `testdata/valid/`

---

## Note on `static-pipeline.md`

The Limitations section of `static-pipeline.md` currently states:

> Adding DisplayID to the static pipeline requires only implementing `StaticExtensionHandler`
> for a `DisplayIdHandler` struct and adding it to `STANDARD_HANDLERS`. No structural
> changes to the pipeline are needed.

This is now superseded. Structural changes are required: the `StaticExtensionHandler` trait
signature changes, `StaticContext` is introduced, and the dispatch loop in
`capabilities_from_edid_static` is replaced. That section should be removed when the doc
updates are made.
