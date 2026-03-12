# Testing strategy

Binary parsers benefit from a testing approach that combines small deterministic tests with larger corpus-based validation.

## Test categories

### Unit tests

Unit tests should cover narrow pieces of logic such as:

- block size checks,
- header validation,
- checksum handling,
- identifier decoding,
- descriptor extraction.

These tests should be small and explicit.

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
### Normalization tests

Normalization tests should verify that parsed structures are translated into stable capability data correctly.

These tests are especially important because normalization introduces policy decisions that go beyond raw parsing.

### Fuzzing

Fuzzing is strongly recommended for the parser.

Important expectations:

- no panics,
- no uncontrolled memory growth,
- invalid input results in controlled errors or warnings,
- unknown structures do not break parsing invariants.

## Test philosophy

PIAF should be strict about structural integrity, but practical about diagnostics.

The test suite should reflect that balance by checking both:

- outright rejection of invalid core structure,
- graceful handling of unusual or partially malformed optional content.

## Long-term goal

As the fixture corpus grows, it should become a source of confidence for refactoring, extension support, and improvements to the normalization layer.
