# imhentai (LUC1D)
Independent Aidoku API implementation. SDK remains pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Discovery (source version 7)
Latest home scroller and paginated Latest listing reuse the existing IMHentai `/?page=N` route. This is an offline semantic adaptation, NOT live-verified: the homepage returned HTTP 403 and no bypass was attempted.

Search now exposes dynamic sort, category and language filters. Popular, Latest, Downloads and Top Rated plus the site's six category flags and seven language flags map to the documented GalleryAdults intermediate-search parameters. Filtered searches use `/search/`; the no-filter browse route remains the existing latest route. Endpoint access is still blocked, so parameter construction is verified against the public provider implementation and synthetic fixtures, not a live IMHentai response.

Home links and manifest listings use the registered `Home` / `ListingProvider` implementations. Unknown listing IDs and nonpositive pages fail rather than silently opening Latest. Challenge/empty home documents fail instead of producing a misleading empty success.

Canonical `https://imhentai.xxx/gallery/<numeric-id>/` links resolve to their manga entries. Foreign hosts and nonnumeric IDs are rejected.

Existing query search, title/cover/artist/tag metadata, one gallery chapter, numeric pages from the complete JSON reader manifest, Referer headers and one request/second remain intact.

## Verification
From this crate directory with Cargo and Aidoku tools on PATH:
```
cargo fmt --check
cargo test --locked
cargo build --release --locked
aidoku package .
aidoku verify package.aix
```
Neutral synthetic WASM fixtures cover home/listing routing, schema identity/language/rating, pagination, search escaping, metadata flags and page manifests. Tests do not fetch content or images. Package verification is not device testing.

## Limits
No Popular Today/Week/Month listings, account/favorites or new dates. Advanced tag/artist filters are not exposed. IMHentai access remains blocked. Keep runtime_tested=false and publish=false pending authorized runtime checks. No thumbnail guessing fallback. IDs, `languages: ["multi"]`, content rating 2, icons, Cargo lockfile and SDK pin are preserved.

## Provenance
Public technical route/schema reference: Keiyoushi extensions-source `GalleryAdults.kt`, `IMHentai.kt`, `HentaiFox.kt` (Apache-2.0; LICENSE retained). HentaiFox Top Rated additionally grounded in its public homepage technical markup. Existing icon retained unchanged. No source package copied. Full discovery feature matrix and check results: `C:/Users/LUC1D/aidoku-research/DISCOVERY-IM-HF.md`.
