# Independently maintained Aidoku sources

Six local Rust adapters, built from source against Aidoku SDK commit `e1320b0a2e11afb59e4dee374883a2212d325699`. Not a forwarding catalog. Existing upstream-derived code retains attribution and licenses in `licenses/` and source directories. Old catalogs and scheduled mirror updates are retired; history is preserved.

**Experimental, not release-ready. No iOS/device reading tests have been completed.** The supported root catalog is intentionally empty. Opt-in test packages are separate under `experimental/` on Pages and in CI artifacts. Do not interpret a successful compile, parser test, or package verification as proof a website works in Aidoku.

## Verification

All six implement search, details, chapters and page URL retrieval; actual site compatibility remains subject to the following limitations.

| Source / independent ID | Local WASM fixture tests | Live evidence | Device test / supported publication |
|---|---:|---|---|
| AsuraScans `en.luc1d-asurascans` | 2 passed | Not independently verified in integration | Pending / blocked |
| WeebCentral `en.luc1d-weebcentral` | 3 passed | Not independently verified in integration | Pending / blocked |
| nhentai `multi.luc1d-nhentai` | 3 passed | Not independently verified in integration | Pending / blocked |
| Official WEBTOON `multi.luc1d-webtoon` | 5 passed | Worker WASM-host live smoke passed: public search, details, cursor chapters, page URLs, popular | Pending / blocked |
| IMHentai `multi.luc1d-imhentai` | 4 passed | Public HTTP 403; no bypass. Synthetic fixtures only | Pending / blocked |
| HentaiFox `multi.luc1d-hentaifox` | 4 passed | Worker metadata HTTP 200; full Aidoku live integration unverified | Pending / blocked |

Independent integration rebuilt and package-verified all six. Generated build-report.json records package SHA256 and test/publication distinctions. `runtime_tested=false` is conservatively retained for every source: WASM fixture execution is not end-to-end Aidoku reading evidence. `publish=false` remains in the manifest. Strict `--release` mode still rejects incomplete approval.

WEBTOON: English advertised, first search page only; Canvas end-to-end and multilingual behavior unverified. No Fast Pass, Daily Pass, app-only or authentication support. Adult adapters: no login, advanced filters, home/listings or deep links; missing reader metadata fails explicitly. No explicit image bytes were downloaded for verification. Access controls are not bypassed.

## Build and experimental distribution

Use Rust 1.98.1, `wasm32-unknown-unknown`, Python 3.11, and `aidoku-cli` / `aidoku-test-runner` installed from the pinned SDK revision with `--locked`.

```sh
python -m unittest discover -s rebuild -p test_pipeline.py
python rebuild/pipeline.py
python rebuild/site_output.py
```

The pipeline runs every crate's locked WASM tests, release build, `aidoku package`, `aidoku verify`, exact six-ID checks and package hashing. The site step separates experimental downloads and emits an empty supported catalog. CI deployment requires a successful build; it does not assert device validation.

Requires Aidoku >=0.7.1. Back up your library before migration: independent IDs are distinct from historical sources and will not silently update them. Test installation, search, details, chapter order, image rendering and pagination on-device before considering supported promotion. Record source version, device/Aidoku version, date and non-sensitive results; never toggle evidence gates just to make CI green.
