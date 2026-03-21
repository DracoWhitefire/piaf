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

Two fuzz targets live in `fuzz/fuzz_targets/`:

- `parse_edid` — exercises the full dynamic pipeline: raw bytes → `parse_edid` → `capabilities_from_edid`.
- `capabilities_static` — exercises the static (no-alloc) pipeline: raw bytes → `parse_edid` → `capabilities_from_edid_static`.

Both are set up using `cargo-fuzz` with libFuzzer. `cargo-fuzz` requires nightly; the library itself stays on stable.

#### Quick smoke run

Runs until Ctrl+C. Useful for interactive testing:

```
cargo +nightly fuzz run parse_edid
cargo +nightly fuzz run capabilities_static
```

Any crashes are written to `fuzz/artifacts/<target>/`.

#### Long campaign

Run for a fixed duration (e.g. one hour), then minimise and commit the resulting corpus:

```
cargo +nightly fuzz run parse_edid -- -max_total_time=3600
cargo +nightly fuzz run capabilities_static -- -max_total_time=3600
```

After the run completes, deduplicate the corpus down to a minimal covering set:

```
cargo +nightly fuzz cmin parse_edid
cargo +nightly fuzz cmin capabilities_static
```

The minimised corpus lives in `fuzz/corpus/<target>/` and should be committed so that subsequent runs — locally and in CI — start from a richer base.

#### CI

The fuzz workflow (`.github/workflows/fuzz.yml`) runs a 60-second smoke test on every push and pull request, and a 1-hour deep run on a weekly schedule. The corpus is cached between runs using GitHub Actions cache. Crash artifacts are uploaded automatically on failure.

## Test philosophy

PIAF should be strict about structural integrity, but practical about diagnostics.

The test suite should reflect that balance by checking both:

- outright rejection of invalid core structure,
- graceful handling of unusual or partially malformed optional content.

## Long-term goal

As the fixture corpus grows, it should become a source of confidence for refactoring, extension support, and improvements to the normalization layer.
