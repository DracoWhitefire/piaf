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

`ExtensionHandler` is the primary extension point. Implement it to process any EDID block —
base or extension — and write results into `DisplayCapabilities`.

```rust
#[derive(Debug)]
struct MyHandler;

impl ExtensionHandler for MyHandler {
    fn process(&self, block: &[u8; 128], caps: &mut DisplayCapabilities, warnings: &mut Vec<EdidWarning>) {
        // Read from block, write to caps or warnings
    }
}
```

### Registering a base block handler

Use `add_base_handler` to register one or more handlers for the base block. All registered
handlers are called in order.

```rust
let mut library = ExtensionLibrary::with_standard_handlers();
library.add_base_handler(MyBaseHandler);
```

### Registering an extension block handler

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

### Replacing a built-in handler

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

Handlers receive a `&mut Vec<EdidWarning>` and can push diagnostics that will be collected
into `caps.warnings` after the pipeline completes.

```rust
warnings.push(EdidWarning::DescriptorParseFailed);
```

## `no_std` support

`ExtensionHandler` and dynamic dispatch require `alloc` or `std`. In `no_std` without an
allocator, handlers are unavailable.

Tag registration is available in all configurations via `ExtensionTagRegistry`:

```rust
let mut registry = ExtensionTagRegistry::new();
registry.register(0xAB);
let parsed = parse_edid(&bytes, &registry)?;
```

`ExtensionTagRegistry` holds up to 16 tags in a fixed-size array in `no_std` builds.
