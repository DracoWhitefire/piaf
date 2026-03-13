# Testing strategy

Binary parsers benefit from a testing approach that combines small deterministic tests with larger corpus-based validation.

## Test categories

### Unit tests

Unit tests cover narrow pieces of logic and live next to the code they test. Handler tests
call `process` directly on a handcrafted `[u8; 128]`, without going through the parser or
building an `ExtensionLibrary`. Parser tests construct minimal valid EDID byte arrays and
assert on specific error and warning conditions.

This keeps failures localized: a failing test in `base.rs` can only mean `BaseBlockHandler`
is broken.

### Integration tests

A single integration test in `capabilities/mod.rs` verifies that the full pipeline wires
together correctly — that `with_standard_handlers()` registers the handlers and that
`capabilities_from_edid` invokes them. It does not duplicate the field-level assertions
that belong in handler unit tests.

### Fixture tests

PIAF should maintain a fixture corpus containing:

- valid EDID captures,
- malformed inputs,
- edge cases,
- truncated or corrupted data.

These fixtures make it easier to improve the parser without unintentionally changing behavior.

A suggested layout:
```text
testdata/
 ├── valid/
 ├── invalid/
 └── edge/
```

### Fuzzing

Fuzzing is strongly recommended for the parser.

Important expectations:

- no panics,
- no uncontrolled memory growth,
- invalid input results in controlled errors or warnings,
- unknown structures do not break parsing invariants.

The fuzz target is in `fuzz/fuzz_targets/parse_edid.rs` and exercises the full pipeline: raw bytes → `parse_edid` → `capabilities_from_edid`. It is set up using `cargo-fuzz` with libFuzzer.

`cargo-fuzz` requires nightly. The library itself stays on stable; nightly is only needed to build and run the fuzz targets.

```
cargo +nightly fuzz build parse_edid
cargo +nightly fuzz run parse_edid
```

The fuzzer runs indefinitely. Stop it with Ctrl+C. Any crashes are written to `fuzz/artifacts/parse_edid/`.

## Test philosophy

PIAF should be strict about structural integrity, but practical about diagnostics.

The test suite should reflect that balance by checking both:

- outright rejection of invalid core structure,
- graceful handling of unusual or partially malformed optional content.

## Long-term goal

As the fixture corpus grows, it should become a source of confidence for refactoring, extension support, and improvements to the normalization layer.
