# Official WEBTOON (LUC1D)

Independent current-API source, ID `multi.luc1d-webtoon`, not WebtoonXYZ.
SDK pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Support and limits

- English public search (Originals and Canvas cards), first search result page only.
- Default popular genre browse, metadata, cursor-paginated episodes, public reader image URLs, vertical viewer, image Referer.
- Official mobile episode API; HTML requests use the official desktop site with its documented-in-URLs platform redirect switch because mobile search is JS-only. This is not an authentication or payment workaround.
- Full live WASM-host smoke: Space Boy search, details, all API episode batches, public episode 1 image URL parsing, default browse.
- Discovery home: Drama by Popularity / Likes / Date, plus genre navigation. Search filters expose all 17 English site genres and the official Popularity / Likes / Date sort orders (`MANA`, `LIKEIT`, `UPDATE`); clearing the text query browses the selected genre and sort. Seven named listings retain their existing IDs and site order. Shelf previews show 20 entries, and opening a listing returns the complete server-rendered genre result. These pages are not paginated: page > 1 is empty.
- Important: the site's bare `/en/genre` redirects to a Drama-selected page, NOT a global popularity chart. New discovery uses explicit `/en/genres/drama` routes and labels. No WEBTOON Popular Today or invented ranking period.
- English advertised only. Text search exposes its first result page only. Optional-page filtering, season-title normalization, Canvas end-to-end verification remain unavailable or unverified.
- Canonical mobile and desktop WEBTOON series links resolve to manga entries after validating the host, path and numeric `title_no`.
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
