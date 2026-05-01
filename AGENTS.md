# Agent instructions — nota-codec

You **MUST** read AGENTS.md at `github:ligoldragon/lore` — the workspace contract.

## Repo role

The **runtime half** of the nota-codec / nota-derive pair. `Decoder` + `Encoder` over nota and nexus dialects.

---

## Carve-outs worth knowing

- Every public method on `Decoder` and `Encoder` is a *protocol method* — the derives in `nota-derive` will call into it. Protocol stability matters; renaming a protocol method is a cross-crate change.
- Errors carry typed context. **No** `Error::Custom(String)` arm. Add structured variants when a new failure mode appears.
- Blanket impls for primitives + std containers live in `traits.rs`. New blanket impls land here, not scattered.
- Tests live in `tests/`, one file per derive, each round-tripping a representative type through `Decoder` + `Encoder`.
