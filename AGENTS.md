# Agent instructions

Workspace-wide rules + tools-documentation pointers live in
[`mentci/AGENTS.md`](https://github.com/LiGoldragon/mentci/blob/main/AGENTS.md).

This crate is the runtime half of the nota-codec / nota-derive
pair. Read [`ARCHITECTURE.md`](ARCHITECTURE.md) before editing.

## Specific rules for this crate

- Every public method on `Decoder` and `Encoder` is a *protocol
  method* — the derives in `nota-derive` will call into it.
  Protocol stability matters; renaming a protocol method is a
  cross-crate change.
- Errors carry typed context. **No** `Error::Custom(String)`
  arm. Add structured variants when a new failure mode appears.
- Blanket impls for primitives + std containers live in
  `traits.rs`. New blanket impls land here, not scattered.
- Tests live in `tests/`, one file per derive, each round-
  tripping a representative type through `Decoder` + `Encoder`.
