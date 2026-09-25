# nhentai (LUC1D) discovery

Independent vendored current-API source, `multi.luc1d-nhentai`; pinned Aidoku SDK `e1320b0a2e11afb59e4dee374883a2212d325699`. Existing source/license provenance is retained in the repository. This change does not add any downloaded adult imagery or explicit fixture text.

## Website-backed capabilities (source version 24)

Version 24 opens official `/tag/<slug>/` links as the matching Aidoku Popular tag listing. It resolves the canonical name with nhentai's per-slug tag API and validates the returned type, slug and route before creating the existing name-based listing ID.

Version 23 adds Aidoku's `multi` language-filter sentinel while retaining the existing English, Japanese, and Chinese language codes. Aidoku 0.9 build 3 filters source-language metadata by exact codes and does not treat `All` as `multi`; the extra code makes nhentai discoverable when users choose multilingual sources.

Version 21 uses the API's gallery thumbnail on the Aidoku chapter entry. Version 20 stores the API's actual scanlator in Aidoku's scanlator field and maps a single recognized language tag to the chapter language. Translated and rewrite tags are ignored; mixed and unknown languages remain unset.

Version 19 adds dynamic Popular tag feeds from the official tag ranking. Version 22 exposes all 120 tag entries returned by the API first ranking page. Each feed uses the exact tag name and popular search order; fixtures cover stable listing IDs and the 120-entry boundary. A live request confirmed that the first page currently returns 120 entries; remaining tags are reachable through freeform tag search.

Existing `/api/v2/search` sorts are `popular-today`, `popular-week`, `popular` (all time), and `date`. Home, four listings, static tag/artist/group filters and language/blocklist settings already existed. They are not newly invented features. Home queries apply configured language and blocklist constraints, so ranks are within that selected result set. Gallery details now include the API's language tags alongside tags, artists, groups, parodies, and characters; they were previously omitted from the description. Version 16 adds a freeform Tag field that uses the official `tag:"..."` query operator, so tags outside the 807 common-tag picker can be searched without waiting for a filter refresh. Version 17 fixes the freeform Artist field to use the official typed `artist:"..."` search operator while retaining its published `author` filter ID, so non-popular artists can be targeted instead of searched as generic text.

Version 10 adds a dynamic Language filter from the official `/api/v2/tags/language` directory. It validates and follows API pagination, then exposes every unique tag whose type is `language`, including classifications with fewer than ten galleries. Those language selections now reach the existing `language:"..."` search query support. Version 11 adds a Popular Artists multi-select from the first page of the official artist directory. Version 12 adds popular artist, group, parody and character selectors from their official type-specific endpoints (120 items per taxonomy). Version 13 adds freeform Parody and Character fields for values outside those quick-selects. Freeform artist, group, parody, character and tag fields map to their matching typed search operators. Source requests are paced at one per second to reduce the documented HTTP 429 failure rate.

The editable blocklist now starts empty; the former `example` placeholder was an active exclusion in all browse queries. The bundled tag filter now includes 807 current tags with at least ten galleries, refreshed from every page of the official tag API. `scripts/update_tags.py` respects pagination and leaves the previous filter untouched if any page fails or is incomplete; HTTP 429 responses receive a bounded, `Retry-After`-aware retry.

Version 3 makes the Popular Today home section a ranked list with a working Today listing link (previous BigScroller had no browse-all action), preserving the actual today endpoint. Search/listings reject page numbers below 1. Existing identities, languages, content rating, icon, filters and reader behavior are unchanged.

Alternate covers are now fetched from the public gallery metadata on demand. Both original and thumbnail variants use the canonical nhentai image hosts; gallery keys, media IDs, and extensions are validated before constructing the URLs. Deep links now require the exact HTTPS host and canonical `/g/<numeric-id>` path; malformed, noncanonical and lookalike-host URLs are ignored.

Version 15 adds on-demand page descriptions from the page number encoded in the official image filename. Aidoku marks each page as having a description; the provider validates the filename and returns a concise page label without another network request.

Version 18 routes chapter pages and cover downloads through `ImageRequestProvider`, applying the official site Referer and source User-Agent to the two canonical image CDNs. Other hosts and non-HTTPS URLs are rejected before a request is created.

## Verification and limits

- `cargo test --locked`: five fixture tests pass in the real WASM host; adult fixtures are synthetic neutral technical metadata.
- `cargo test --locked --features live-tests`: metadata-only integration probes four sorts and second pages, then home. In the recorded run all four two-page listing checks passed before home hit the site's HTTP 429 rate limit (error JSON missing the success `result` field). The complete live suite did NOT pass. Ordinary website HTML returned 403. No bypass or repeated retries were attempted.
- `cargo build --release --locked`, `aidoku package .`, `aidoku verify package.aix` passed. ZIP manifest/icon/WASM were compared to source/build artifacts.
- No gallery image bytes fetched, no device installation/rendering verified. `runtime_tested=false`. Hosted access policy, pagination stability during uploads, and full home execution remain live caveats. A rate-limited home fails rather than substituting another sort or manufactured data.

The opt-in live test makes multiple metadata requests; run only after server rate limits reset, not in an automatic retry loop.
