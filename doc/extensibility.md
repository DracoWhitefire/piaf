# Extensibility

## Philosophy

PIAF is a building block, not a complete solution. Different consumers will encounter
different hardware, proprietary extensions, and application-specific requirements that the
library cannot anticipate. Rather than trying to cover every case internally, PIAF provides
the parsing infrastructure and exposes the right seams for consumers to plug into.

The goal is that a consumer should never need to fork the library to get their work done.
They should be able to register custom handlers, store custom data, and receive diagnostics
— all without modifying library code.

This also means PIAF stays small. Features that belong in a specific application or a
higher-level library should not accumulate here.

## Extension points

PIAF is designed so that consumers can add to its behaviour without forking the library.
This document describes the extension points and how to use them.

## Extension handlers

PIAF provides two handler traits that cover different build tiers. Use the one that matches
your allocation constraints.

### `ExtensionHandler` (dynamic, `alloc`/`std`)

`ExtensionHandler` is the primary extension point for std builds. Implement it to process
any EDID block — base or extension — and write results into `DisplayCapabilities`. Handlers
are registered via `ExtensionLibrary` using `Box<dyn ExtensionHandler>`.

```rust
#[derive(Debug)]
struct MyHandler;

impl ExtensionHandler for MyHandler {
    fn process(&self, block: &[u8; 128], caps: &mut DisplayCapabilities, warnings: &mut Vec<ParseWarning>) {
        // Read from block, write to caps or warnings
    }
}
```

### `StaticExtensionHandler` (static, all tiers)

`StaticExtensionHandler` is the no-alloc counterpart. Handlers are `'static` references in
a slice rather than boxed trait objects. Output goes to a `dyn ModeSink` rather than
`DisplayCapabilities` directly — this means mode extraction only; rich metadata requires
`ExtensionHandler`.

```rust
struct MyHandler;

impl StaticExtensionHandler for MyHandler {
    fn tag(&self) -> u8 { 0xAB }
    fn process(&self, block: &[u8; 128], sink: &mut dyn ModeSink) {
        // push modes via sink.push_mode(...)
    }
}

static MY_HANDLER: MyHandler = MyHandler;
```

Pass a slice of static references to `capabilities_from_edid_static`:

```rust
static HANDLERS: &[&dyn StaticExtensionHandler] = &[piaf::CEA861_HANDLER, &MY_HANDLER];

let caps: StaticDisplayCapabilities<64> =
    capabilities_from_edid_static(&parsed, HANDLERS);
```

Because `tag()` makes each handler self-describing, the same slice can be passed directly to
`parse_edid` as a `KnownExtensions` implementation — no separate `ExtensionTagRegistry`
needed.

### Registering a base block handler (dynamic pipeline)

Use `add_base_handler` to register one or more handlers for the base block. All registered
handlers are called in order.

```rust
let mut library = ExtensionLibrary::with_standard_handlers();
library.add_base_handler(MyBaseHandler);
```

### Registering an extension block handler (dynamic pipeline)

Extension block handlers are registered per tag via `ExtensionMetadata`.

```rust
library.register(ExtensionMetadata {
    tag: 0xAB,
    display_name: String::from("My Extension"),
    handler: Some(Box::new(MyHandler)),
});
```

Registering an extension with the library is sufficient — passing the library to
`parse_edid` automatically covers tag recognition:

```rust
let parsed = parse_edid(&bytes, &library)?;
let caps = capabilities_from_edid(&parsed, &library);
```

### Replacing a built-in handler (dynamic pipeline)

The handlers registered by `with_standard_handlers()` can be replaced by mutating the
library before use:

```rust
let mut library = ExtensionLibrary::with_standard_handlers();
if let Some(cea) = library.extensions.iter_mut().find(|e| e.tag == 0x02) {
    cea.handler = Some(Box::new(MyCeaHandler));
}
```

## Custom extension data

Handlers can store typed data in `DisplayCapabilities` for consumers to retrieve later.
Any type that is `Debug + Send + Sync` qualifies.

```rust
#[derive(Debug)]
struct MyData { version: u8 }

impl ExtensionHandler for MyHandler {
    fn process(&self, block: &[u8; 128], caps: &mut DisplayCapabilities, _warnings: &mut Vec<EdidWarning>) {
        caps.set_extension_data(MY_TAG, MyData { version: block[1] });
    }
}
```

Retrieve the data after `capabilities_from_edid`:

```rust
if let Some(data) = caps.get_extension_data::<MyData>(MY_TAG) {
    // data: &MyData
}
```

The stored value is wrapped in `Arc`, so cloning `DisplayCapabilities` shares the data
rather than deep-copying it.

See `examples/inspect_displays.rs` for a working demonstration using CEA-861.

## Emitting warnings

Dynamic handlers receive `&mut Vec<ParseWarning>` and can push any type implementing
`core::error::Error + Send + Sync + 'static`. Built-in library code pushes `EdidWarning`
variants; custom handlers may push their own types.

```rust
warnings.push(Arc::new(EdidWarning::MalformedDataBlock));
```

Static handlers emit warnings via `ModeSink::push_warning`, which accepts `EdidWarning`
directly (no boxing required):

```rust
sink.push_warning(EdidWarning::MalformedDataBlock);
```

## `no_std` support

`ExtensionHandler` and `ExtensionLibrary` require `alloc` or `std`. `StaticExtensionHandler`
and `capabilities_from_edid_static` are available unconditionally.

In bare `no_std` (no `alloc`), `parse_edid` returns a `ParsedEdidRef<'_>` that borrows
extension blocks directly from the input slice — no allocator needed. Static handlers receive
extension block calls normally; `capabilities_from_edid_static` processes both base-block and
extension-block data at all build tiers.

For tag-only registration without handlers (e.g., to suppress `UnknownExtension` warnings),
`ExtensionTagRegistry` is available at all tiers and holds up to 16 tags in a fixed-size
array. A `&[&dyn StaticExtensionHandler]` slice also implements `KnownExtensions` and is
generally the better choice when you already have handlers registered.

```rust
// Either works for parse_edid:
let parsed = parse_edid(&bytes, &registry)?;
let parsed = parse_edid(&bytes, piaf::STANDARD_HANDLERS)?;
```
