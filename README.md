# wm-semcache — she asked an hour ago; answer for free

Embedding-keyed semantic response cache for the wintermute companion stack.

When an utterance is a near-duplicate (cosine ≥ threshold) of one already answered,
`wm-semcache` returns the stored response with zero API cost — no Sonnet turn, no local model.

## Safety property

Utterances classified cache-unsafe (time, weather, calendar-today, reminders) are
**never stored and never served**. A cache hit there would speak a stale lie.

## Design

- **Store + lookup** keyed by L2-normalised embedding vectors (dot-product == cosine similarity).
- **Embedder trait** — plug in recall's `embed` RPC or any stub; degrades to `None` on error.
- **TTL-based expiry** with a pluggable frozen clock for deterministic tests.
- **LRU eviction** enforces a max-entry capacity bound.
- **Deflection metrics** — `hits / total_lookups` so cost savings are measurable.

## Requirements

- Rust 1.85+ (MSRV)
- No `unsafe` code

## License

MIT OR Apache-2.0 — Joe Yen
