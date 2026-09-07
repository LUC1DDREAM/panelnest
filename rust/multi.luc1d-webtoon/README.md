# Official WEBTOON (LUC1D)

Independent current-API source, ID `multi.luc1d-webtoon`, not WebtoonXYZ.
SDK pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

## Support and limits

- English public search (Originals and Canvas cards), first search result page only.
- Default popular genre browse, metadata, cursor-paginated episodes, public reader image URLs, vertical viewer, image Referer.
- Official mobile episode API; HTML requests use the official desktop site with its documented-in-URLs platform redirect switch because mobile search is JS-only. This is not an authentication or payment workaround.
- Full live WASM-host smoke: Space Boy search, details, all API episode batches, public episode 1 image URL parsing, default browse.
- English advertised only: multilingual settings, advanced filters/listings, deep links, exhaustive search pagination, optional-page filtering, season-title normalization, Canvas end-to-end verification deferred.
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
