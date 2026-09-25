# hentaifox (LUC1D)
Independent Aidoku API implementation. SDK remains pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Discovery (source version 34)
Version 34 refreshes the Groups: metadata idempotently, replacing stale values and removing the field when no groups are present.
Version 33 maps the separate official ul.groups taxonomy into Manga.description as Groups: ..., preserving any existing description and keeping groups separate from tags. A live gallery fixture checks group and artist metadata.
Version 32 maps official gallery artist links from ul.artists into Aidoku Manga.artists, trims whitespace, and clears Manga.authors because the site exposes no separate Authors field. A sanitized live gallery fixture verifies the mapping.

Version 24 fills Aidoku's chapter language when the gallery has exactly one recognized language tag. Unknown, translated-only and mixed-language galleries leave the field unset rather than guessing.
Version 23 carries each gallery page¡¯s /gallery/<id>/ URL through Aidoku page context so CDN image requests use the matching Referer. The image provider validates the CDN host and gallery path; fixtures cover valid and malformed contexts. Completed one-gallery entries also use Aidoku¡¯s UpdateStrategy::Never, so routine library refreshes skip their immutable chapter lists.
Version 21 adds a separate Popular-sorted dynamic listing for each of the 50 popular tags. The existing Tag: listings keep their IDs and Latest ordering; Popular: listings use the official paginated /tag/<slug>/popular/ route.
Version 20 adds the official Daily Top Rated Today and Yesterday picks to Home as direct gallery spotlights. Both are already present in the homepage HTML and toggle locally; no extra request is needed.
Version 19 exposes Aidoku page descriptions using the already validated reader image filenames, so each page shows its page number without another network request.
Version 18 streams the initial Latest/Top Rated home layout immediately, then sends each available sidebar ranking as soon as it returns. Users can browse the core sections while the remaining rankings load.
Version 17 expands the Home page with browsable Latest, Top Rated, Most Faved, Most Fapped and Most Downloaded sections. The three sidebar rankings are requested independently and omitted individually when unavailable, so a ranking failure does not hide the rest of Home. The ranking HTML uses the same CSRF-protected official sidebar route as its existing dynamic catalog listings.
Version 16 adds a Language filter from the site's official `/languages/popular/` directory. All 26 listed categories are available, including the site's translated, rewrite and text-cleaning classifications. Selected values open the corresponding `/language/<slug>/` routes and preserve Latest/Popular sorting and pagination.
Version 15 adds freeform name/slug filters for tags, artists, characters, parodies and groups. They open official category routes beyond the quick-select lists and preserve Latest/Popular sorting and pagination. Names are converted to lowercase hyphenated route slugs; exact ASCII site slugs can also be entered.
Version 14 fixes Popular sorting for selected tags: the official tag page has a paginated /popular/ route just like other taxonomy pages. Earlier builds always returned the latest tag route even when Popular was selected. Both sort modes now retain pagination.
Latest home scroller and paginated Latest listing use `/` then `/page/N/`. Top Rated reads the server-rendered default `#middle_sidebar div.item` only when `#top_rated_btn.sidebar_btn_active` is present. Most Faved, Most Fapped and Most Downloaded are dynamic listings loaded from the documented `includes/sidebar.php` endpoint with the site's CSRF token and XHR header. Those rankings are finite sidebar responses, not paginated archives: page 2 returns empty without a request. Dynamic listings include the 50 most popular tags from `/tags/popular/`; each opens the official `/tag/<slug>/` gallery list. Search filters load up to 50 popular artists, characters, parodies and groups and all 26 official language-directory entries. Selecting a taxonomy opens its canonical latest or popular route with pagination under `/pag/N/`. If a directory is unavailable, that filter is omitted while the others remain. Live taxonomy directory and representative paginated gallery routes were checked; no live Aidoku/device playback is claimed.

Quick-select menus show the 50 most popular tags and up to 50 popular entries for each taxonomy other than languages, whose official directory currently contains 26 entries. The version 15 text filters can open additional categories when their ASCII site name or slug is known. Names with non-ASCII characters are not converted automatically; enter their official ASCII slug instead.

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
No paginated Popular Today/Week/Month listings or generic Popular listing. Faplist is now exposed separately from Favorites after WebLogin; authenticated gallery contents could not be tested here. HentaiFox login opens the official /login/ page through Aidoku WebLogin. After validating the PHP session through /profile/, Bookmarks loads account favorites from /includes/user_favs.php with page-number pagination. The authenticated response was not live-tested because no account session was available. Home includes the current and previous Daily Top Rated spotlight. Keep runtime_tested=false; no Aidoku/device playback is claimed. No thumbnail guessing fallback. IDs, `languages: ["multi"]`, content rating 2, icons, Cargo lockfile and SDK pin are preserved.

## Provenance
Public technical route/schema reference: Keiyoushi extensions-source `GalleryAdults.kt`, `IMHentai.kt`, `HentaiFox.kt` (Apache-2.0; LICENSE retained). HentaiFox Top Rated additionally grounded in its public homepage technical markup. Existing icon retained unchanged. No source package copied. Full discovery feature matrix and check results: `C:/Users/LUC1D/aidoku-research/DISCOVERY-IM-HF.md`.

Version 25 adds the validated gallery cover URL to the single chapter thumbnail.

Version 26 exposes all 50 entries from the official popular-tag directory as quick-select tags, with separate Latest and Popular listings.


Version 27 fills Aidoku Chapter.date_uploaded from the official relative Posted age. The site does not expose an exact date, so this is an approximation anchored to detail-fetch time; unrecognized age formats remain unset.

Version 28 adds Aidoku WebLogin and account Bookmarks. Version 29 adds the settings control for logging in and clears the saved session on logout. Login validates the PHP session through /profile/; the official site JavaScript documents paginated favorites at /includes/user_favs.php. An authenticated account response was not available for live testing.

Version 29 exposes the Aidoku login control and handles logout notifications so Bookmarks can be accessed and sessions can be cleared from source settings.

Version 31 adds the account Faplist as a separate login-only paginated listing. It reads the official /faplist/ and /faplist/pag/N/ pages and recognizes the explicit empty-list state.
