# Roadmap

## 0.2

### DisplayID support

DisplayID uses variable-length data blocks that may span multiple 128-byte EDID extension
blocks. Correct handling requires:

1. collecting all DisplayID fragments from the extension block list,
2. reassembling them into a contiguous byte stream,
3. parsing logical DisplayID blocks from that stream.

Naive per-block dispatch is not sufficient. This is the primary feature planned for 0.2.

### Broader fixture corpus

The current fixture corpus covers two hardware captures. Expanding it — particularly with
edge cases, malformed inputs, and displays from a wider range of manufacturers — will
increase confidence in the normalization layer and make refactoring safer.

## Post-0.2

### Derived-value helpers

Helpers for computed results (preferred mode selection, bandwidth checks, DPI, HDR and VRR
detection, mode filtering) belong in a separate module as free functions that accept
`&DisplayCapabilities`. They should not be methods on `DisplayCapabilities` itself, which
is a decoded data representation rather than a decision layer.

### Consistency validation

Helpers to detect internally inconsistent EDIDs: modes whose pixel clock exceeds the
declared maximum, refresh rates outside the declared range, conflicting identity fields.
These surface as warnings rather than errors, since the underlying data may still be useful.
