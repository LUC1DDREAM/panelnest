# imhentai (LUC1D)
Independent Aidoku API implementation. SDK remains pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Discovery and reader requests (source version 22)
Version 22 fills Aidoku's chapter language when the gallery has exactly one recognized language tag. Unknown, translated-only and mixed-language galleries leave the field unset rather than guessing.
Version 21 applies a mobile Safari User-Agent and same-site Referer consistently to Home, search, listing, gallery detail and reader HTML requests. Reader and CDN requests retain the actual gallery-page Referer.
Version 19 uses Aidoku’s UpdateStrategy::Never for completed single-gallery titles so routine library refreshes skip their immutable one-chapter list.
Version 18 adds 13 dynamic browse feeds for six content categories and seven languages.
Version 18 adds 13 dynamic browse feeds: one for each of the six supported content categories and seven supported languages. They use the existing search flags and keep page navigation; existing Latest, Popular, Top Rated and Downloaded listings are unchanged.
Version 15 handles protected CDN images with a validated per-page Referer from Aidoku PageContext. Cover images use the source root; unsupported hosts and formats are rejected. Reader manifests are validated and each page uses its own file format. Device rendering remains untested.

Version 13 exposes Aidoku page descriptions using the already validated reader image filenames, so each page shows its page number without another network request.

Version 11 adds the site's advanced search filters for tags, artists, groups, parodies and characters. Each accepts comma-separated terms and a leading minus sign excludes a term. The source maps these to the public advanced-search key syntax while retaining the selected sort, category and language flags. Advanced terms are sanitized and URL-encoded; combining advanced filters with the separate title query is rejected instead of silently dropping either input. The route and encoding follow the current public GalleryAdults provider implementation and are covered by offline WASM tests.
Home presents Latest, Popular, Top Rated and Downloaded scrollers; each links to its full paginated listing. The public site exposes `/popular/`, `/top-rated/` and `/downloaded/` browse pages. Listing and filter routes remain fixture-verified; Aidoku runtime checks are unrun.

Search exposes dynamic sort, category and language filters. Popular, Latest, Downloads and Top Rated plus the site's six category flags and seven language flags map to the documented GalleryAdults intermediate-search parameters. Filtered searches use `/search/`; the no-filter browse route remains the existing latest route. Route construction is verified against the public provider implementation and synthetic fixtures; filtered responses are not live-verified.

Home links and manifest listings use the registered `Home` / `ListingProvider` implementations. Unknown listing IDs and nonpositive pages fail rather than silently opening Latest. Challenge/empty home documents fail instead of producing a misleading empty success.

Canonical `https://imhentai.xxx/gallery/<numeric-id>/` links resolve to their manga entries. Host matching now requires the exact HTTPS hostname, so lookalike suffix hosts, HTTP links and nonnumeric IDs are rejected.

Existing query search, title/cover/artist/tag metadata, one gallery chapter, reader pages from the current `/view/<id>/<page>/` route using its `g_th` page manifest and authoritative `#gimg` host/path, and legacy numeric pages from the complete JSON reader manifest remain intact. Each current reader page uses its own format from `g_th`; the source does not override Aidoku's image Referer with a generic site-root header, allowing the app's originating chapter/page context to reach the CDN. Reader deep links resolve to their gallery entry.

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
No Popular Today/Week/Month listings, account/favorites or exact upload dates. On 2026-09-25, gallery, reader and CDN image URLs returned HTTP 200 using the source's mobile Safari headers. This does not certify the Aidoku runtime or device, so runtime_tested and device_tested remain false. No thumbnail guessing fallback. IDs, `languages: ["multi"]`, content rating 2, icons, Cargo lockfile and SDK pin are preserved.

## Provenance
Public technical route/schema reference: Keiyoushi extensions-source `GalleryAdults.kt`, `IMHentai.kt`, `HentaiFox.kt` (Apache-2.0; LICENSE retained). HentaiFox Top Rated additionally grounded in its public homepage technical markup. Existing icon retained unchanged. No source package copied. Full discovery feature matrix and check results: `C:/Users/LUC1D/aidoku-research/DISCOVERY-IM-HF.md`.
