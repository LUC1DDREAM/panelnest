# PanelNest — independently maintained Aidoku sources

Six local Rust adapters, built from source against Aidoku SDK commit `e1320b0a2e11afb59e4dee374883a2212d325699`. Not a forwarding catalog. Existing upstream-derived code retains attribution and licenses in `licenses/` and source directories. Old catalogs and scheduled mirror updates are retired; history is preserved.

**Experimental, not release-ready. No iOS/device reading tests have been completed.** The supported root catalog is intentionally empty. Opt-in test packages are separate under `experimental/` on Pages and in CI artifacts. Do not interpret a successful compile, parser test, or package verification as proof a website works in Aidoku.

## Verification

All six implement search, details, chapters and page URL retrieval; actual site compatibility remains subject to the following limitations.

| Source / independent ID | Local WASM fixture tests | Live evidence | Device test / supported publication |
|---|---:|---|---|
| AsuraScans `en.luc1d-asurascans` | 9 passed | Public metadata fixture; refresh-based lock countdown and fail-closed reader guard; no live paid/device proof | Pending / blocked |
| WeebCentral `en.luc1d-weebcentral` | 3 passed | Not independently verified in integration | Pending / blocked |
| nhentai `multi.luc1d-nhentai` | 3 passed | Not independently verified in integration | Pending / blocked |
| Official WEBTOON `multi.luc1d-webtoon` | 5 passed | Worker WASM-host live smoke passed: public search, details, cursor chapters, page URLs, popular | Pending / blocked |
| IMHentai `multi.luc1d-imhentai` | 4 passed | Public HTTP 403; no bypass. Synthetic fixtures only | Pending / blocked |
| HentaiFox `multi.luc1d-hentaifox` | 4 passed | Worker metadata HTTP 200; full Aidoku live integration unverified | Pending / blocked |

Independent integration rebuilt and package-verified all six. Generated build-report.json records package SHA256 and test/publication distinctions. `runtime_tested=false` is conservatively retained for every source: WASM fixture execution is not end-to-end Aidoku reading evidence. `publish=false` remains in the manifest. Strict `--release` mode still rejects incomplete approval.

WEBTOON: English advertised, first search page only; Canvas end-to-end and multilingual behavior unverified. No Fast Pass, Daily Pass, app-only or authentication support. Adult adapters: no login, advanced filters, home/listings or deep links; missing reader metadata fails explicitly. No explicit image bytes were downloaded for verification. Access controls are not bypassed.

## Install the experimental list

[Open PanelNest](https://luc1ddream.github.io/panelnest/) in English, Deutsch, Español, Français or Português. [Add experimental list to Aidoku](https://aidoku.app/add-source-list/?url=https%3A%2F%2Fluc1ddream.github.io%2Fpanelnest%2Fexperimental%2Findex.min.json) on a device with Aidoku installed. Or add `https://luc1ddream.github.io/panelnest/experimental/index.min.json` in Aidoku Settings > Source Lists. The root list intentionally contains zero supported sources.

Already installed the `my-aidoku-sources/experimental/index.min.json` URL? Keep it: the [compatibility publisher](https://github.com/LUC1DDREAM/my-aidoku-sources) maintains complete JSON, package and icon copies at the old Pages path. No source IDs change for this rename. Compatibility checks run every six hours (GitHub schedules may be delayed) and can be dispatched manually. Git/repository links must use `LUC1DDREAM/panelnest`; recreating the legacy name intentionally replaces GitHub rename redirects.

Asura locks show the site-provided release time in UTC and a countdown updated **at refresh**, not a ticking timer. Unknown/premium chapters stay locked; a passed deadline alone never grants access. Current chapter metadata is checked before either reader path. No access-control bypass or paid-account/device test is claimed.

## Build and experimental distribution

Use Rust 1.98.1, `wasm32-unknown-unknown`, Python 3.11, and `aidoku-cli` / `aidoku-test-runner` installed from the pinned SDK revision with `--locked`.

```sh
python -m unittest discover -s rebuild -p test_pipeline.py
python rebuild/pipeline.py
python rebuild/site_output.py
```

The pipeline runs every crate's locked WASM tests, release build, `aidoku package`, `aidoku verify`, exact six-ID checks and package hashing. The site step separates experimental downloads and emits an empty supported catalog. CI deployment requires a successful build; it does not assert device validation.

Requires Aidoku >=0.7.1. Back up your library before migration: independent IDs are distinct from historical sources and will not silently update them. Test installation, search, details, chapter order, image rendering and pagination on-device before considering supported promotion. Record source version, device/Aidoku version, date and non-sensitive results; never toggle evidence gates just to make CI green.
