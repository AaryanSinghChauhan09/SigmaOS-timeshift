# SigmaOS-timeshift

System snapshot and restore component for SigmaOS, inspired by Linux Mint
Timeshift. The Rust crate models rsync-style and Btrfs snapshot modes, retention
levels, schedules, and snapshot metadata. Kernel and filesystem integrations are
declared as sibling path dependencies in `Cargo.toml`.

This repository's current source and tests determine implementation status;
the declared modes do not imply that every backend is production-ready.

## Principles
- MODULARITY: Single responsibility per repository
- LOOSE COUPLING: Minimal dependencies between repos
- INTERFACE SEGREGATION: Clear, focused public APIs
- DETERMINISTIC TESTING: Comprehensive test coverage

## License
SPDX-License-Identifier: GPL-3.0-or-later

## Copyright
Copyright (C) 2026 SigmaOS Project
