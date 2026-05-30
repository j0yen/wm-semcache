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

## Acceptance tests

1. `store` + `lookup` of a paraphrase (stubbed embedder) returns the cached response when cosine ≥ threshold.
2. Unrelated utterance (cosine < threshold) returns `None`.
3. Cache-unsafe utterance returns `None` from `lookup` and is rejected by `store` even when a matching entry exists — the cardinal safety property.
4. Expired entry (injected frozen clock) is not served.
5. LRU eviction: exceeding capacity evicts the least-recently-hit entry.
6. Embedder degrades safely when unreachable — `lookup` returns `None`, never panics; works with 256-dim and 384-dim vectors.
7. Deflection metric (`hits / total_lookups`) is exposed and correct.

## Install

This is a library crate — add it to your `Cargo.toml`:

```toml
[dependencies]
wm-semcache = { git = "https://github.com/j0yen/wm-semcache" }
```

## Requirements

- Rust 1.85+ (MSRV)
- No `unsafe` code

## License

MIT OR Apache-2.0 — Joe Yen
