# Contributing to PIAF

Thanks for your interest in contributing. This document covers the basics.

## Getting started

Relevant docs for contributors:

- [`doc/setup.md`](doc/setup.md) — build, test, and fuzzing commands
- [`doc/testing.md`](doc/testing.md) — testing strategy, fixture corpus, and CI expectations
- [`doc/architecture.md`](doc/architecture.md) — pipeline structure, scope, and design principles
- [`doc/model.md`](doc/model.md) — data types, field conventions, and the fixed-capacity pattern
- [`doc/roadmap.md`](doc/roadmap.md) — planned features and known gaps

## Issues and pull requests

**Open an issue first** if you're unsure whether something is a bug or if you want to
discuss a change before implementing it. For small, self-contained fixes a PR on its own
is fine.

- Bug reports: include the EDID bytes (as a hex dump or attached binary) if you have them.
- Feature requests: a brief description of what you need and why is enough to start a
  conversation.
- PRs: keep them focused. One logical change per PR makes review faster and keeps history
  readable.

## Coding standards

- Run `cargo fmt` and `cargo clippy -- -D warnings` before pushing.
- Public items need rustdoc comments (`cargo rustdoc -- -D missing_docs` must pass).
- Follow the existing patterns in the codebase — see [`doc/architecture.md`](doc/architecture.md)
  and [`doc/model.md`](doc/model.md) for the design principles behind them.
- `#![deny(unsafe_code)]` is enforced; no unsafe code.
- Keep `no_std` compatibility. The static pipeline and all scalar decoding must compile
  without `alloc` or `std`.

## Commit and PR expectations

- Write commit messages in the imperative mood ("Add support for …", not "Added …").
- Keep commits logically atomic. A PR that touches three unrelated things should be three
  commits (or three PRs).
- Tests are expected for new decoding logic. A unit test with a handcrafted byte slice is
  usually sufficient; a real fixture capture is a bonus.
- CI must be green before a PR can merge: fmt, clippy, docs, all test and build targets,
  and coverage must not drop more than 0.1% below the baseline (stored in
  `.coverage-baseline`). New decoding logic without tests will likely trip this.

## Review process

PRs are reviewed on a best-effort basis. Expect feedback within a few days; if you haven't
heard back in a week feel free to ping the thread. Reviews aim to be constructive — if
something needs to change, the reviewer will explain why. Approval from the maintainer is
required to merge.

## Code of Conduct

This project follows the [Contributor Covenant 3.0](CODE_OF_CONDUCT.md). Please read it
before participating.
