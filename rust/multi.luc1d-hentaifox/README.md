# hentaifox (LUC1D)
Independent Aidoku API implementation. SDK remains pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Discovery (source version 10)
Latest home scroller and paginated Latest listing use `/` then `/page/N/`. Top Rated reads the server-rendered default `#middle_sidebar div.item` only when `#top_rated_btn.sidebar_btn_active` is present. Most Faved, Most Fapped and Most Downloaded are dynamic listings loaded from the documented `includes/sidebar.php` endpoint with the site's CSRF token and XHR header. Those rankings are finite sidebar responses, not paginated archives: page 2 returns empty without a request. Dynamic listings also include the 25 most popular tags from `/tags/popular/`; each opens the official `/tag/<slug>/` gallery list, with next-page links under `/pag/N/`. Missing/changed tag-directory data falls back to the established sidebar lists. Homepage, popular-tag index and representative tag-page routes were checked against the public site; no live Aidoku/device playback is claimed.

Home links and manifest listings use the registered `Home` / `ListingProvider` implementations. Unknown listing IDs and nonpositive pages fail rather than silently opening Latest. Challenge/empty home documents fail instead of producing a misleading empty success.

Canonical `https://hentaifox.com/gallery/<numeric-id>/` links resolve to their manga entries. Foreign hosts and nonnumeric IDs are rejected.

Search adds a Sort filter with Latest as the unchanged default and Popular mapped to the site's `sort=popular` parameter. This sort applies to text searches; an empty-query browse still uses the existing Latest route.

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
No Popular Today/Week/Month listings, generic Popular listing, language selector, advanced tag filters, account favorites or new dates. Keep runtime_tested=false and publish=false pending authorized runtime checks. No thumbnail guessing fallback. IDs, `languages: ["multi"]`, content rating 2, icons, Cargo lockfile and SDK pin are preserved.

## Provenance
Public technical route/schema reference: Keiyoushi extensions-source `GalleryAdults.kt`, `IMHentai.kt`, `HentaiFox.kt` (Apache-2.0; LICENSE retained). HentaiFox Top Rated additionally grounded in its public homepage technical markup. Existing icon retained unchanged. No source package copied. Full discovery feature matrix and check results: `C:/Users/LUC1D/aidoku-research/DISCOVERY-IM-HF.md`.
