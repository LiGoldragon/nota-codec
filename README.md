# nota-codec

Typed `Decoder` + `Encoder` for the
[nota](https://github.com/LiGoldragon/nota) and
[nexus](https://github.com/LiGoldragon/nexus) text dialects.
Serde-free codec built around closed-`Token`-enum dispatch
and the `NotaEncode` / `NotaDecode` traits.

## What this crate exposes

- **`Decoder<'input>`** — reads nota or nexus text, dispatched
  via the Token enum. `Decoder::next_request()` reads a top-
  level nexus request (any sigil + any verb).
- **`Encoder`** — writes nota or nexus text. Dialect-aware:
  refuses to emit nexus-only forms (sigils, pattern delimiters)
  when in nota mode.
- **`Lexer`**, **`Token`**, **`Dialect`** — the tokenizer
  layer. Public for callers that want raw token streams.
- **Traits `NotaEncode` + `NotaDecode`** — what every typed
  value implements (usually via the derives below).
- **Re-exports from
  [`nota-derive`](https://github.com/LiGoldragon/nota-derive)**
  — `NotaRecord`, `NotaEnum`, `NotaTransparent`,
  `NexusPattern`, `NexusVerb`. Users only depend on this crate.
- **`Error`** — typed enum; no `Custom(String)` arm.

## Why this exists

The typed text codec for the nota and nexus dialects. Closed-enum
dispatch (one variant per verb / record kind) at the boundary;
the type system carries the meaning, not stringly-typed metadata.
Replaces the previous serde-based path.

## Dialect knob

```rust
let mut decoder = Decoder::nexus(text);   // accepts nexus features
let mut decoder = Decoder::nota(text);    // strict nota subset

let mut encoder = Encoder::nexus();
let mut encoder = Encoder::nota();
```

The lexer carries the dialect; types deriving only
`NotaRecord` / `NotaEnum` / `NotaTransparent` round-trip in
either mode. Types deriving `NexusPattern` or `NexusVerb` are
nexus-only and the encoder errors when asked to emit them in
nota mode.

## License

[License of Non-Authority](LICENSE.md).
