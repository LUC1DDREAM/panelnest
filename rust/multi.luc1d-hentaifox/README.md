# hentaifox (LUC1D)
Independent Aidoku API implementation. SDK remains pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Discovery (source version 19)
Version 19 exposes Aidoku page descriptions using the already validated reader image filenames, so each page shows its page number without another network request.
Version 18 streams the initial Latest/Top Rated home layout immediately, then sends each available sidebar ranking as soon as it returns. Users can browse the core sections while the remaining rankings load.
Version 17 expands the Home page with browsable Latest, Top Rated, Most Faved, Most Fapped and Most Downloaded sections. The three sidebar rankings are requested independently and omitted individually when unavailable, so a ranking failure does not hide the rest of Home. The ranking HTML uses the same CSRF-protected official sidebar route as its existing dynamic catalog listings.
Version 16 adds a Language filter from the site's official `/languages/popular/` directory. All 26 listed categories are available, including the site's translated, rewrite and text-cleaning classifications. Selected values open the corresponding `/language/<slug>/` routes and preserve Latest/Popular sorting and pagination.
Version 15 adds freeform name/slug filters for tags, artists, characters, parodies and groups. They open official category routes beyond the quick-select lists and preserve Latest/Popular sorting and pagination. Names are converted to lowercase hyphenated route slugs; exact ASCII site slugs can also be entered.
Version 14 fixes Popular sorting for selected tags: the official tag page has a paginated /popular/ route just like other taxonomy pages. Earlier builds always returned the latest tag route even when Popular was selected. Both sort modes now retain pagination.
Latest home scroller and paginated Latest listing use `/` then `/page/N/`. Top Rated reads the server-rendered default `#middle_sidebar div.item` only when `#top_rated_btn.sidebar_btn_active` is present. Most Faved, Most Fapped and Most Downloaded are dynamic listings loaded from the documented `includes/sidebar.php` endpoint with the site's CSRF token and XHR header. Those rankings are finite sidebar responses, not paginated archives: page 2 returns empty without a request. Dynamic listings include the 25 most popular tags from `/tags/popular/`; each opens the official `/tag/<slug>/` gallery list. Search filters load up to 50 popular artists, characters, parodies and groups and all 26 official language-directory entries. Selecting a taxonomy opens its canonical latest or popular route with pagination under `/pag/N/`. If a directory is unavailable, that filter is omitted while the others remain. Live taxonomy directory and representative paginated gallery routes were checked; no live Aidoku/device playback is claimed.

Quick-select menus show the 25 most popular tags and up to 50 popular entries for each taxonomy other than languages, whose official directory currently contains 26 entries. The version 15 text filters can open additional categories when their ASCII site name or slug is known. Names with non-ASCII characters are not converted automatically; enter their official ASCII slug instead.

Home links and manifest listings use the registered `Home` / `ListingProvider` implementations. Unknown listing IDs and nonpositive pages fail rather than silently opening Latest. Challenge/empty latest-home documents fail instead of producing a misleading empty success.

Canonical `https://hentaifox.com/gallery/<numeric-id>/` links resolve to their manga entries. Host matching requires the exact HTTPS hostname, rejecting lookalike suffix hosts, HTTP links and nonnumeric IDs.

Search adds a Sort filter with Latest as the unchanged default and Popular mapped to the site's `sort=popular` parameter. For a selected artist, character, parody, group or language, the same sort maps to the official category route; category selection cannot be combined with free-text search. An empty-query browse still uses the existing Latest route.

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
No Popular Today/Week/Month listings, generic Popular listing, account favorites or new dates. Keep runtime_tested=false and publish=false pending authorized runtime checks. No thumbnail guessing fallback. IDs, `languages: ["multi"]`, content rating 2, icons, Cargo lockfile and SDK pin are preserved.

## Provenance
Public technical route/schema reference: Keiyoushi extensions-source `GalleryAdults.kt`, `IMHentai.kt`, `HentaiFox.kt` (Apache-2.0; LICENSE retained). HentaiFox Top Rated additionally grounded in its public homepage technical markup. Existing icon retained unchanged. No source package copied. Full discovery feature matrix and check results: `C:/Users/LUC1D/aidoku-research/DISCOVERY-IM-HF.md`.
