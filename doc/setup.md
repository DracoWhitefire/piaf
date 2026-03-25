# Development Setup

**Requirements:** Rust 1.85+ (stable). Install via [rustup](https://rustup.rs/).

## Clone and build

```sh
git clone https://github.com/DracoWhitefire/piaf.git
cd piaf
cargo build
```

## Running tests

```sh
cargo test                            # std (default)
cargo test --features serde           # std + serde
cargo build --no-default-features --features alloc  # alloc-only build check
cargo build --no-default-features                   # bare no_std build check
```

## Fuzzing

Fuzzing requires nightly:

```sh
cargo +nightly fuzz run parse_edid
cargo +nightly fuzz run capabilities_static
```

See [`doc/testing.md`](testing.md) for long-campaign workflows and corpus management.

## Capturing EDID fixtures

To capture EDID binaries from connected displays:

```sh
cargo run --example capture_fixture
```

Fixtures live in `testdata/`. Adding real-world captures from unusual hardware is welcome.
