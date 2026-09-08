# Discovery and generated-icon update

All six packages remain opt-in EXPERIMENTAL. Supported catalog remains empty. No physical iOS/reader image verification claimed.

| Source | Version | Discovery | Offline tests |
|---|---:|---|---:|
| Asura | 3 | Popular week/month/all-time; original Ranking/bookmarks retained | 12 |
| WeebCentral | 2 | Popularity, Subscribers, Recently Added, Latest Updates | 5 |
| nhentai | 3 | Existing today/week/all/latest; today now has full-list navigation | 5 |
| WEBTOON | 3 | Explicit English Drama popularity/likes/date; genre links, no global daily ranking | 8 |
| IMHentai | 4 | Latest Home and listing; offline adaptation, site HTTP 403 | 7 |
| HentaiFox | 4 | Latest and finite server-rendered Top Rated | 8 |

Integration: 45 locked WASM fixture tests passed; 2 optional live tests ignored. All six locked release builds, aidoku package and aidoku verify passed. 11 Python publication contract tests passed. Exact source/package manifests and WASM/icon bytes checked; IDs, languages and ratings unchanged, IMHentai/HentaiFox retain ["multi"]. Asura countdown and fresh subscription reader guard files unchanged.

Review: integration self-review of production diffs, dispatch, finite pagination, labels, manifest identity, tests and provenance; not an independent reviewer. No invented day rankings or fallback from failed periods. No added secrets or authentication bypass. Live evidence belongs to source worker runs, not a repeated integration live run; nhentai suite Home hit 429 and WeebCentral/IMHentai 403 remain known blockers. User will test on device.

Icons: six original assistant-authored vector interpretations, not official extracted/downloaded logos and not generative image-service output. Each res/icon.svg is editable original geometry; res/icon.png is opaque RGB 128x128. Current hashes/attribution: rebuild/icon-provenance.json. Previous downloaded-branding evidence retained verbatim in rebuild/icon-provenance-historical.json; contact sheet replaced with current designs.

Native Swift: decoder model and language metadata unchanged; existing exact language regression tests passed. No new native Swift/iOS claim made.
