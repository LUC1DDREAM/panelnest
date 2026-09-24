# nhentai (LUC1D) discovery

Independent vendored current-API source, `multi.luc1d-nhentai`; pinned Aidoku SDK `e1320b0a2e11afb59e4dee374883a2212d325699`. Existing source/license provenance is retained in the repository. This change does not add any downloaded adult imagery or explicit fixture text.

## Website-backed capabilities

Existing `/api/v2/search` sorts are `popular-today`, `popular-week`, `popular` (all time), and `date`. Home, four listings, static tag/artist/group filters and language/blocklist settings already existed. They are not newly invented features. Home queries apply configured language and blocklist constraints, so ranks are within that selected result set. Gallery details now include the API's language tags alongside tags, artists, groups, parodies, and characters; they were previously omitted from the description.

Version 3 makes the Popular Today home section a ranked list with a working Today listing link (previous BigScroller had no browse-all action), preserving the actual today endpoint. Search/listings reject page numbers below 1. Existing identities, languages, content rating, icon, filters and reader behavior are unchanged.

Alternate covers are now fetched from the public gallery metadata on demand. Both original and thumbnail variants use the canonical nhentai image hosts; gallery keys, media IDs, and extensions are validated before constructing the URLs. Deep links now accept only numeric gallery paths on the exact canonical host; malformed and lookalike-host URLs are ignored.

## Verification and limits

- `cargo test --locked`: five fixture tests pass in the real WASM host; adult fixtures are synthetic neutral technical metadata.
- `cargo test --locked --features live-tests`: metadata-only integration probes four sorts and second pages, then home. In the recorded run all four two-page listing checks passed before home hit the site's HTTP 429 rate limit (error JSON missing the success `result` field). The complete live suite did NOT pass. Ordinary website HTML returned 403. No bypass or repeated retries were attempted.
- `cargo build --release --locked`, `aidoku package .`, `aidoku verify package.aix` passed. ZIP manifest/icon/WASM were compared to source/build artifacts.
- No gallery image bytes fetched, no device installation/rendering verified. `runtime_tested=false`. Hosted access policy, pagination stability during uploads, and full home execution remain live caveats. A rate-limited home fails rather than substituting another sort or manufactured data.

The opt-in live test makes multiple metadata requests; run only after server rate limits reset, not in an automatic retry loop.
