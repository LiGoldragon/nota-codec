# nota-codec — architecture

## Role

Runtime half of the `nota-codec` / `nota-derive` pair. This
crate hosts the `Lexer`, the `Decoder` and `Encoder` types,
the `NotaEncode` + `NotaDecode` traits, the `Error` type, and
blanket impls for primitives + standard containers. It re-
exports the derives from
[`nota-derive`](https://github.com/LiGoldragon/nota-derive) so
users depend on a single crate.

## Boundaries

**Owns:**
- The `Lexer` (token stream over nota/nexus source text). The
  same lexer the previous `nota-serde-core` crate carried.
- The `Token` and `Dialect` enums.
- The `Decoder<'input>` type — typed reading from a token
  stream.
- The `Encoder` type — typed writing to a `String` buffer.
- The `NotaEncode` + `NotaDecode` traits.
- Blanket impls for `u64`, `i64`, `f64`, `bool`, `String`,
  `Vec<T>`, `Option<T>`.
- The crate-wide `Error` enum.
- Top-level sigil-and-delimiter dispatch in
  `Decoder::next_request`.

**Does not own:**
- The proc-macro logic — that's
  [`nota-derive`](https://github.com/LiGoldragon/nota-derive).
- Any record kinds or wire-IR types — those live in
  [`signal`](https://github.com/LiGoldragon/signal) (Node /
  Edge / Graph / KindDecl / AssertOperation / …).
- `PatternField<T>` itself — defined in `signal`; this crate
  exposes the protocol method `decode_pattern_field` that
  the type's hand-impl of `NotaDecode` calls into.

## Code map

```
src/
├── lib.rs        # re-exports + crate-level doc + Result<T> alias
├── lexer.rs      # Lexer + Token + Dialect (copied from nota-serde-core)
├── decoder.rs    # Decoder<'input> + protocol methods
├── encoder.rs    # Encoder + protocol methods
├── traits.rs     # NotaEncode + NotaDecode + blanket impls
└── error.rs      # Error enum + Result<T> type alias

tests/
├── nota_record_round_trip.rs
├── nota_enum_round_trip.rs
├── nota_transparent_round_trip.rs
├── nexus_pattern_round_trip.rs
└── nexus_verb_round_trip.rs
```

(Tests added as each derive's codegen lands.)

## Cross-cutting context

- The other half of the pair lives at
  [`nota-derive`](https://github.com/LiGoldragon/nota-derive/blob/main/ARCHITECTURE.md).
- Both crates exist as the serde replacement at the
  nota-and-nexus text layer — see
  [mentci reports/098](https://github.com/LiGoldragon/mentci/blob/main/reports/098-serde-replacement-decision-2026-04-27.md)
  and
  [mentci reports/099](https://github.com/LiGoldragon/mentci/blob/main/reports/099-custom-derive-design-2026-04-27.md).
- The codec's design respects criome's perfect-specificity
  invariant — closed `Token` enum drives request dispatch,
  closed-variant enums drive verb dispatch — see
  [criome ARCHITECTURE.md §2 Invariant D](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md#invariant-d).

## Status

**v0.1** — skeleton in place: lexer copied from
nota-serde-core; trait + decoder + encoder + error stubs.
Codegen and protocol methods land incrementally as the
implementation proceeds.
