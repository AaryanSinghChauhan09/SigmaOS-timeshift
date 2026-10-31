# SigmaOS-timeshift

System snapshot and restore component for SigmaOS, inspired by Linux Mint
Timeshift. The Rust crate models rsync-style and Btrfs snapshot modes, retention
levels, schedules, and snapshot metadata. Kernel and filesystem integrations are
declared as sibling path dependencies in `Cargo.toml`.

The current manager models snapshots in memory: creation uses a fixed
100 MiB size, restore returns a formatted status string, and listing reports
the stored model entries. It does not invoke rsync, create Btrfs snapshots,
write data to the configured snapshot path, or restore files. The tests cover
the model and state transitions, not filesystem backup or recovery.

## License

Apache-2.0. See [`LICENSE`](LICENSE) and [`Cargo.toml`](Cargo.toml).
