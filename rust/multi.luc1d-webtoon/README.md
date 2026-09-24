# Official WEBTOON (LUC1D)

Independent current-API source, ID `multi.luc1d-webtoon`, not WebtoonXYZ (package version 16).
SDK pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Support and limits

- English public search with All-results, WEBTOON Originals and CANVAS scopes. All-results accepts the official `page` query parameter even though its HTML omits next-page links; the source continues while a page contains series and stops after the first empty page. Originals and CANVAS use their official page links for pagination.
- Default popular genre browse, metadata, cursor-paginated episodes, public reader image URLs, vertical viewer, image Referer.
- Official mobile episode API; HTML requests use the official desktop site with its documented-in-URLs platform redirect switch because mobile search is JS-only. This is not an authentication or payment workaround.
- Full live WASM-host smoke: Space Boy search, details, all API episode batches, public episode 1 image URL parsing, default browse.
- Discovery reads genres and sort links from each selected locale¡¯s current genre page, with bundled fallbacks on request or parse failure. Filters and home links stay current as WEBTOON changes its genre catalog. Named listings cover each known genre in all three official sort orders while preserving existing IDs.
- Newly introduced official genres are now accepted by search filters and added as dynamic Aidoku listings in the current locale, without duplicating the existing bundled listing IDs.
- Home sends its stable section layout first and streams each discovery section as it loads; a failed section does not hide successful sections or the genre browser.
- Important: the site's bare `/en/genre` redirects to a Drama-selected page, NOT a global popularity chart. New discovery uses explicit `/en/genres/drama` routes and labels. No WEBTOON Popular Today or invented ranking period.
- The Aidoku language selector supports English (`en`), Traditional Chinese (`zh-hant`), Thai (`th`), Indonesian (`id`), Spanish (`es`), French (`fr`), and German (`de`). Search and genre routes use the selected locale; unknown values fall back to English. All three search scopes paginate. Optional-page filtering, season-title normalization, and Canvas end-to-end reader verification remain unavailable or unverified.
- Canonical mobile and desktop WEBTOON series links resolve to manga entries; official episode viewer links resolve directly to chapters after validating the host, series path, `title_no`, and `episode_no`.
- No login, app-only chapters, Fast Pass, Daily Pass, purchases, or paywall bypass. Missing reader images returns an explicit error. No image bytes downloaded in tests. Actual iOS installation/image rendering not verified.

## Reproduce

```
cargo test --locked
cargo test --locked --features live-tests
cargo build --release --locked --target wasm32-unknown-unknown
aidoku package .
aidoku verify package.aix
```

## Provenance

New trait-based implementation informed by the legacy Aidoku official `multi.webtoon` parser and helper at the supplied `legacy-upstream` checkout. Icon copied from that source. Repository MIT/Apache licenses retained under `../../licenses`; WEBTOON branding remains its owner's trademark. Public HTML/JSON fixtures were fetched without credentials from official WEBTOON URLs; they are parser regression inputs, not comic image files. See `tests/fixtures/provenance.json`.
