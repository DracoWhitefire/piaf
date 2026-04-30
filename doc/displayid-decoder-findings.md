# DisplayID decoder findings (cross-check vs `edid-decode`)

Source: `edid-decode` (freedesktop, swick repo), file `parse-displayid-block.cpp`,
last commit before the v4l-utils move (`cd4bba8~1`). Cross-referenced against
`doc/displayid-explained.md` (everything-explained.today excerpt of the DisplayID
spec).

Local clone for re-verification: `../../ref/edid-decode/parse-displayid-block.cpp`
(checkout via `git checkout cd4bba8~1 -- parse-displayid-block.cpp`).

These are real bugs in our decoders, not roadmap items. Tests pass because they
synthesize descriptors using our (incorrect) interpretation and round-trip through
the decoder — they verify self-consistency, not spec-conformance. Real-world
fixtures may already produce wrong output.

## Severity legend

- **CRIT** — wrong width/height/pixel_clock; visible in `caps.supported_modes`.
- **HIGH** — wrong byte-0 semantics (Y420 vs NTSC, missing stereo, phantom enum variants).
- **MED**  — missing field that real EDIDs may carry but isn't structurally wrong.

---

## 1a. Type I is structurally wrong (needs full rewrite) — **DONE**

Fixed: `decode_type_i_descriptor` rewritten to match spec; extracted shared
`decode_type_1_7_descriptor_body(d, pixel_clock_unit_khz)` helper (Type VII can
delegate once §1b lands). Three test helper copies (`detailed.rs`, `displayid/mod.rs`,
`timing/mod.rs`) updated to emit the correct wire format with `value − 1` baked in;
helpers now take `pixel_clock_10khz: u32` and `options_byte3: u8`. Interlaced test
moved from `flags = 0x01` (byte 19 bit 0) to `options_byte3 = 0x10` (byte 3 bit 4).
Two new tests cover sync-polarity decoding from the FP-word bit 15 (a feature
absent from the prior buggy decoder). 444 tests pass; no_std builds; `-D warnings` clean.

The original bug detail (kept for reference):


`decode_type_i_descriptor` at `src/capabilities/displayid/timing/detailed.rs:19`
has a completely wrong descriptor layout. Per spec (`displayid-explained.md` §0x03,
which says Type I is "superseded by Type VII" and shares its byte layout) and
edid-decode `parse_displayid_type_1_7_timing(x, false, ...)`:

| Spec offset | Field                                              | Our code |
|-------------|----------------------------------------------------|----------|
| Bytes 0–2   | Pixel clock, 24-bit LE, in 10 kHz steps, **+1**    | Reads bytes 1–2 as 16-bit (wrong width, no +1, wrong offset) |
| Byte 3      | Options: aspect ratio bits 3:0, interlaced bit 4, stereo bits 6:5, preferred bit 7 | Mistakenly read as part of `h_active` low byte |
| Bytes 4–5   | H active (+1)                                      | Reads bytes 3–4 (no +1) |
| Bytes 6–7   | H blank (+1)                                       | Reads bytes 5–6 (no +1) |
| Bytes 8–9   | H front porch, bits 14:0 (+1); bit 15 = HSP        | Reads bytes 7–8 (no +1, no polarity bit) |
| Bytes 10–11 | H sync (+1)                                        | Reads bytes 9–10 (no +1) |
| Bytes 12–13 | V active (+1)                                      | Reads bytes 11–12 (no +1) |
| Bytes 14–15 | V blank (+1)                                       | Reads bytes 13–14 (no +1) |
| Bytes 16–17 | V front porch, bits 14:0 (+1); bit 15 = VSP        | Reads bytes 15–16 (no +1, no polarity bit) |
| Bytes 18–19 | V sync (+1)                                        | Reads bytes 17–18 (no +1) |
| (no byte 19 flags field) | —                                     | Misreads `v_sync_width` high byte as flags (interlaced + sync polarity bits in wrong positions) |

**Severity:** CRIT — every field is wrong. Tests pass because they synthesize
descriptors using the same wrong layout, so values round-trip.

**Fix:** rewrite `decode_type_i_descriptor` to match Type VII's layout exactly,
parameterised on pixel-clock units (10 kHz for Type I, 1 kHz for Type VII).
Cleanest path: extract a shared `decode_type_1_7_descriptor_body(d, units_khz)`
helper, then both Type I and Type VII delegate.

## 1b. Off-by-one in remaining detailed-timing decoders — **DONE**

Fixed:
- **Type V** (`decode_type_v_descriptor`): h_active and v_active now decode `raw + 1`
  via `checked_add(1)?`. The "zero width/height skipped" tests were repurposed as
  "raw 0xFFFF overflow skipped" tests since the wire format can't represent value 0;
  added a "minimum representable (1×1)" test for completeness.
- **Type VII** (`decode_type_vii_descriptor_to_mode`): now a thin delegator to
  `decode_type_1_7_descriptor_body(d, 1)` introduced in §1a. All multi-byte fields
  inherit the correct `+1` decoding. Test helper `make_type_vii_descriptor` updated
  to apply `value − 1` internally; `t7_1080p60` and `test_t7vtdb_720p60` raw-byte
  fixtures recomputed; `test_type_vii_zero_active_skipped` repurposed as
  `test_type_vii_h_active_overflow_skipped`; `test_type_vii_rational_rate_preserved`
  updated to pass `1` for porch/sync values (the minimum representable on the wire).
- **Type IX** (`decode_type_ix_descriptor`): same `checked_add(1)?` treatment.
  Test helper `make_type_ix_descriptor` updated; `test_type_ix_zero_*_skipped`
  repurposed as overflow tests; `test_type_ix_v2_dispatch`,
  `test_type_ix_v2_static_pipeline`, `test_type_ix_not_decoded_on_v1_section`
  updated to write `value − 1` raw bytes via `replace_all` on the inline pattern.
- **`make_type_vii_descriptor_1080p60`** and the inline 720p60 builder in
  `timing/mod.rs` updated similarly.
- **Type V** (`test_type_v_static_pipeline`) raw-byte payload updated to `value − 1`.

445 tests pass; no_std builds; `-D warnings` clean.

The original bug detail (kept for reference):


**edid-decode pattern** (every Type V / VII / IX field):
```c
t.hact = 1 + (x[N] | (x[N+1] << 8));
```

The wire encoding is `value − 1` to allow representing 0 (which never appears as
an active dimension). We currently decode `value` raw.

| Decoder                              | File                                                | Affected fields                                                                                            | Severity |
|--------------------------------------|-----------------------------------------------------|------------------------------------------------------------------------------------------------------------|----------|
| `decode_type_v_descriptor`           | `src/capabilities/displayid/timing/short.rs:60`     | `h_active`, `v_active`                                                                                     | CRIT     |
| `decode_type_vii_descriptor_to_mode` | `src/capabilities/displayid/timing/detailed.rs:163` | `pixel_clock_khz`, `h_active`, `h_blank`, `h_front_porch`, `h_sync_width`, `v_active`, `v_blank`, `v_front_porch`, `v_sync_width` | CRIT     |
| `decode_type_ix_descriptor`          | `src/capabilities/displayid/timing/short.rs:96`     | `h_active`, `v_active`                                                                                     | CRIT     |

**Type II (`decode_type_ii_descriptor`) is already correct** — uses `(raw + 1) × 8`
for H fields and `1 + raw` for V fields per edid-decode `parse_displayid_type_2_timing`.
No change needed.

**Refresh-rate field** (byte 5 in Type V/IX): we already do `(d[5] as u16) + 1` —
matches edid-decode's `1 + x[5]`. ✓

**T7VTDB CTA path:** `parse_t7vtdb` calls `decode_type_vii_descriptor_to_mode` on
`block_data[2..22]`. Same fix applies via the shared helper.

### Test impact

Every existing test in `detailed.rs` and `short.rs` that constructs a descriptor
with literal bytes and asserts the decoded width/height was using values that
matched our (wrong) interpretation. After the fix, **all of those test bytes need
to drop by 1** (e.g. `make_type_i_descriptor(..., 1920, 280, ...)` becomes
`make_type_i_descriptor(..., 1919, 279, ...)` in the wire-byte sense — or, more
ergonomically, helpers should accept the actual active dimension and emit `dim − 1`
internally).

Pixel clock arithmetic (`pixel_clock_hz / (h_total × v_total)` for refresh
derivation) needs re-verification — `h_total`/`v_total` change because all four
fields now `+1`.

---

## 2. Type IX byte 0 — three semantic bugs

### 2a. Algorithm field (bits 2:0) — phantom variants

Per `displayid-explained.md` (DisplayID 2.0 §0x24) and edid-decode
(`parse_displayid_type_9_timing` switch on `x[0] & 0x07`):

| Bits 2:0 | Spec name | edid-decode mapping |
|----------|-----------|---------------------|
| `0`      | CVT       | (no RB; standard CVT) |
| `1`      | CVT-RB    | `RB_CVT_V1` |
| `2`      | CVT-R2    | `RB_CVT_V2` |
| `3`–`7`  | (not defined) | default no-op |

Our `display-types::CvtAlgorithm` enum:

| Variant | Discriminant per our `from_bits` |
|---------|---------------------------------|
| `CvtRb1`                | 0 (should be: standard CVT) |
| `CvtRb2`                | 1 (should be: CVT-RB v1) |
| `CvtRb3`                | 2 (should be: CVT-RB v2 / "CVT-R2") |
| `ReducedBlankingCvtRb1` | 3 (does not exist in spec) |
| `ReducedBlankingCvtRb2` | 4 (does not exist in spec) |
| `Reserved(b)`           | 5–7 (correct as "reserved") |

**Implication:** `CvtRb3`, `ReducedBlankingCvtRb1`, `ReducedBlankingCvtRb2`
must be removed. The remaining variants need re-mapping. Suggested final shape:

```rust
pub enum CvtAlgorithm {
    Cvt,        // 0 — standard CVT (not reduced blanking)
    CvtRb,      // 1 — CVT-RB (v1)
    CvtR2,      // 2 — CVT-R2 (RB v2)
    Reserved(u8),
}
```

`CvtAlgorithm::from_bits` needs the new mapping. `compute_type_ix_timing` arms
need updating (`CvtRb` → v1 evaluator, `CvtR2` → v2 evaluator). Drop `cvt_rb_v3`
delegation entirely — there is no v3 algorithm code at byte 0 bits 2:0.

The roadmap entries mentioning "RB-with-CVT-RB1/RB2 variants" and "CVT-RB v3 VRR
extensions" should be deleted from `doc/roadmap.md` — they were chasing phantom
values.

**Severity:** HIGH for the enum shape (public API churn); CRIT for the formula
dispatch (currently routes `bits = 0` → CVT-RB v1 evaluator, but bits=0 means
*standard CVT, not RB*). For real EDIDs encoding standard CVT (algorithm 0), we
emit RB-derived blanking — wrong.

### 2b. Bit 4 — NTSC fractional refresh, not Y420

Per spec table and edid-decode (`if (x[0] & 0x10) s += ", refresh rate * (1000/1001) supported"`):

> Bit 3 (in spec text — 0-indexed bit 3 = mask `0x08`?) — NTSC video optimized
> refresh rate × (1000/1001): `0` = not supported, `1` = supported

Wait — local spec text says "Bit 3" but mask `0x10` is bit 4. edid-decode tests
`x[0] & 0x10` (bit 4). Spec text in `displayid-explained.md` may use 1-indexed
bits or contain a typo. **edid-decode is authoritative** → bit 4 is the NTSC
fractional refresh flag.

Our code reads `(d[0] >> 4) & 1` and stores it in `VideoMode::y420`. **Wrong
semantics.** `VideoMode::y420` is a colorspace flag (set by CTA Y420 VDB / Y420
capability map and by Type VII byte 3 bit 7); Type IX bit 4 is a refresh-rate
flag.

**Fix path:**
- Stop writing `y420` from Type IX bit 4.
- Add a new `VideoMode` field for the NTSC-fractional flag — proposed name
  `supports_ntsc_fractional_refresh: bool`. Default `false`.
- `with_ntsc_fractional_refresh(b)` builder.
- Type IX decoder reads bit 4 → this new field.
- Type V byte 0 bit 4 has the same semantics (per edid-decode); plumb the same
  way.

### 2c. Bits 6:5 — stereo flag (tri-state, not viewing methods)

Per edid-decode:

| Bits 6:5 | Meaning |
|----------|---------|
| `0`      | Mono timing (no 3D stereo) |
| `1`      | 3D stereo timing |
| `2`      | Mono or 3D stereo depending on user action |
| `3`      | Reserved (warn) |

This is a **per-mode tri-state**, not the rich method enum used by the 0x27
Stereo Display Interface block. Don't re-use `StereoViewingMethodV2`.

**Fix path:**
- New enum in `display-types`, e.g. `TypeIxStereoMode { Mono, Stereo, ModeDependsOnUserAction, Reserved(u8) }`.
- Or simpler: a `Option<bool>` where `None` = mono, `Some(false)` = stereo, `Some(true)` = mode-depends. Less expressive but smaller.
- Add `VideoMode::type_ix_stereo: Option<TypeIxStereoMode>` (or similar). Default `None`.
- `with_type_ix_stereo(s)` builder.
- Type IX decoder reads bits 6:5 → this field.
- Type V has the same tri-state encoding (per edid-decode); plumb the same way.

**Severity:** MED — Type IX descriptors that actually use stereo are rare in
consumer EDIDs; missing this isn't an active correctness bug, just unsurfaced
data.

---

## 3. Type VII byte 3 bit 7 — Y420 flag (block rev ≥ 2)

Per edid-decode:
```c
if (block_rev >= 2 && (x[3] & 0x80)) {
    s += ", YCbCr 4:2:0";
    dispid.has_ycbcr_420 = true;
}
```

We currently don't decode this. The `VideoMode::y420` field exists and is the
right home for it. Decoder needs the data block's revision byte to gate the
read.

**Severity:** MED. Type VII is the right place to populate `y420`; this is the
flag we should be using (the Type IX bit 4 misuse in §2b is the wrong one).

---

## 4. Type V doc-comment errors (separate from §1)

`src/capabilities/displayid/timing/short.rs:60` rustdoc says:

> Byte 0: Options: bits 1:0 = CVT algorithm (0=CVT-RB2, 1=CVT-RB); bit 4 = NTSC;
> bits 6:5 = stereo; bit 7 = preferred

Per spec and edid-decode:
- Algorithm field is **bits 2:0** (3-bit), not bits 1:0.
- Algorithm encoding: `0` = CVT, `1` = CVT-RB, `2` = CVT-R2 (matches Type IX, §2a).
- Bit 4 / bits 6:5 / bit 7 doc lines are correct in *intent* but the corresponding
  fields aren't actually decoded.

**Fix path:** correct the rustdoc and either decode the fields or note them as
deferred. Tied into the §1 + §2 fixes since Type V and Type IX share the byte 0
layout.

---

## Suggested execution order

Each step is a separate increment, can land independently:

1. **Type I structural rewrite** (§1a) — biggest single change. Rewrite to match
   Type VII layout, optionally extracting a shared body decoder. All Type I
   tests need updating. No real-world fixture impact (corpus has no DisplayID
   blocks).

2. **Off-by-one sweep** (§1b) — Type V, Type VII, Type IX. Mechanical `+1` on
   all multi-byte fields. Test helpers should accept human-readable values
   (e.g. `1920` for h_active) and emit `value − 1` internally so assertions
   stay readable.

3. **Type IX algorithm enum re-shape** (§2a) — `display-types` API churn but
   path-deferred so no external impact. Drop `cvt_rb_v3()` delegation; remove
   `CvtRb3`, `ReducedBlankingCvtRb1`, `ReducedBlankingCvtRb2`. Update
   `compute_type_ix_timing` dispatch to new variant names. Update CHANGELOG to
   acknowledge prior CVT-RB v3 entry was based on a misreading.

4. **Type IX bit 4 NTSC field** (§2b) — new `VideoMode` field, drop the wrong
   `y420` write. Same change for Type V.

5. **Type IX stereo enum** (§2c) — new `display-types` enum + `VideoMode` field.

6. **Type VII Y420 flag** (§3) — new field read from byte 3 bit 7 (rev-gated).

7. **Type V doc-comment + field decoding** (§4) — bring in line with §1b/§2/§3
   fixes.

8. **Roadmap cleanup** — remove the phantom "RB-with-CVT-RB1/RB2" entries from
   `doc/roadmap.md`; remove the CVT-RB v3 VRR-extensions entry (no v3
   discriminant exists).

## Open questions before fixing

- ~~Do any of our fixtures carry Type I/V/VII/IX descriptors?~~ **Answered: no.**
  All 4 EDIDs in `testdata/valid/` carry only base + a single CEA-861 (`0x02`)
  extension; none has a DisplayID (`0x70`) extension. The off-by-one fix
  therefore changes only synthetic test outputs — no real-world regression risk
  in our corpus. Treat that as the floor; future fixtures with DisplayID blocks
  would catch any residual issues.
- For step 2, is the user comfortable with the public API churn on
  `CvtAlgorithm`? Path-deferred display-types means no downstream-breakage today
  but the prior CHANGELOG entries become misleading.
