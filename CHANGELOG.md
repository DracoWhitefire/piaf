# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-21

### Added

**Core EDID parsing**
- `ParsedEdid` struct with `DisplayCapabilities`, `EdidWarnings`, and extension data
- `parse_edid` entry point with extensible handler and warning infrastructure
- Manufacturer ID, product code, serial number, manufacture week and year
- Video input definition with bit flags, color bit depth, color type (analog/digital)
- Physical display dimensions and gamma
- Established timings bitmap, standard timings, and 18-byte descriptor block parsing
- Chromaticity coordinates, white point descriptors, and color management data
- Preferred image size, interlacing, CVT, and range limits in `VideoMode`
- Horizontal/vertical front porch and sync width in `VideoMode`
- DTD parsing with out-of-range refresh rate handling

**CEA-861 / CTA-861 extension**
- Full VIC lookup table covering VICs 1–255 (including extended VICs 128–255)
- CEA Audio Descriptors, HDMI SPA, Max TMDS, Deep Color
- Video capability, colorimetry standards, HDR EOTFs/luminance metadata
- HDMI 2.0 Vendor-Specific Data Block (VSDB) and HDMI Forum Sink Capability Data Block
- HDMI Forum EDID Extension Override
- VESA Display Data Block (DDDB) and VTB-EXT
- Type VII (T7VTDB), Type VIII (T8VTDB with full VESA DMT 1.13 table), and Type X (T10VTDB) Video Timing Data Blocks
- VESA Vendor-Specific Audio/Video Data Blocks (VSADB, VSVDB)
- VESA Transfer Characteristic Data Block
- Spatial audio and info frame data decoding
- CEA-861 flags exposed as `bitflags` with `serde` support

**Static / zero-copy pipeline**
- `StaticDisplayCapabilities<const MAX_MODES: usize>` for fixed-size environments
- `ModeSink` trait and `EdidSource` for zero-copy implementation
- `StaticExtensionHandler` trait with `KnownExtensions` for slice-backed dispatch
- `StaticContext` and `capabilities_from_edid_static` entry point

**`no_std` support**
- Full `no_std` + `alloc` build via `std` and `alloc` feature flags
- Serial number string, display name, and unspecified text in `no_std` builds via newtype pattern
- Warnings emitted in `no_std` builds
- `Vec` replaced with slice-compatible patterns for `no_std` compatibility

**Robustness and safety**
- `EdidError` enum via `thiserror`; extensible warnings infrastructure
- Handlers can emit warnings
- Hardened manufacturer ID parsing and early return on malformed DTDs
- Overflow-safe extension block counting with `checked_mul`
- `#![deny(unsafe_code)]`
- Fuzz testing corpus and `cargo fuzz` integration

**Developer experience**
- Example script demonstrating full field extraction
- Test fixtures for real-world EDID binaries with fixture capture script
- Full rustdoc coverage
- CI pipeline with `cargo test`, `cargo rustdoc`, and `cargo fmt`
- Published to crates.io with docs.rs integration
