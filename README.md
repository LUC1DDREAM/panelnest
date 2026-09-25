# PanelNest — independently maintained Aidoku sources

Six local Rust adapters, built against the pinned Aidoku SDK. Existing upstream attribution and licenses remain in `licenses/` and source directories.

## Install or refresh

[Open PanelNest](https://luc1ddream.github.io/panelnest/) or [add the list to Aidoku](https://aidoku.app/add-source-list/?url=https%3A%2F%2Fluc1ddream.github.io%2Fpanelnest%2Findex.min.json).

Manual URL: `https://luc1ddream.github.io/panelnest/index.min.json`. Requires Aidoku >=0.7.1.

All six sources are published in the root catalog. **Already installed? In Aidoku’s Browse tab, pull down to refresh the source list. Then install each entry shown in the Updates section.** Refreshing the list makes updates available; it does not replace locally installed source packages. The canonical and old `my-aidoku-sources` `/experimental/index.json` and `/experimental/index.min.json` subscriptions remain maintained aliases. IDs and installed version asset URLs are unchanged. The legacy directory name is a compatibility path, not a publication classification. The compatibility mirror runs every six hours and supports immediate manual dispatch.
- Discovery menu reads the genre and sort links from the selected locale current page, falling back to bundled defaults if fetching or parsing fails. Filters and home links then track WEBTOON catalog changes. Named listings cover each known genre in all three official sort orders and keep existing IDs.
| Display name | Stable ID | Version |
|---|---|---:|
| Asura Scans [PN] | `en.luc1d-asurascans` | 17 |
| Weeb Central [PN] | `en.luc1d-weebcentral` | 21 |
| nhentai [PN] | `multi.luc1d-nhentai` | 25 |
| WEBTOON [PN] | `multi.luc1d-webtoon` | 23 |
| IMHentai [PN] | `multi.luc1d-imhentai` | 31 |
| HentaiFox [PN] | `multi.luc1d-hentaifox` | 36 |

## Verification and approval

The [Aidoku capability audit](rebuild/aidoku-capability-audit.md) compares every pinned SDK source trait with the six implementations and records why the remaining handlers do not fit current site behavior.

The user explicitly authorized publication and removal of public test branding. Automated end-to-end and independent iPhone certification are not claimed. Runtime and device evidence is recorded per source; IMHentai v31 has a user-confirmed Aidoku device check for opening a chapter and displaying its page images. `release_authorized=true` records publication authorization separately from testing. Historical live results, including Cloudflare/HTTP 429 failures, remain in `rebuild/sources.json`.

Publication requires complete source functionality, package verification, publication approval, and either recorded runtime evidence or explicit release authorization. Every release rebuilds and executes all six locked WASM fixture suites, verifies packages and exact source/package/catalog metadata, opaque 128×128 icons, languages, ratings, IDs and hashes. Same-version ZIP/member immutability fails closed; prior versioned packages remain available.

Asura lock countdowns update on refresh. A passed deadline never grants access; current chapter metadata is checked before both reader paths. Unknown/premium chapters stay locked. WEBTOON Fast Pass, Daily Pass, app-only and authentication restrictions are not bypassed. No credentials, adult imagery or paid access tests were collected for this release.

## Website defaults

A fresh root visit is **English regardless of browser language**. Explicit `/en/`, `/de/`, `/es/`, `/fr/`, `/pt/` routes work without JavaScript; an explicitly chosen language is remembered for later root visits. English canonical/hreflang points to `/`; alias pages canonicalize rather than duplicate sitemap entries.

The website is **dark by default regardless of OS theme**. The accessible light/dark button stores an explicit preference. A small blocking head script applies the saved theme before CSS loads, preventing a wrong-theme flash. Storage is optional; no analytics or remote fonts are used.

## Build

Use Rust 1.98.1, `wasm32-unknown-unknown`, Python 3.11 and `aidoku-cli` / `aidoku-test-runner` from SDK `e1320b0a2e11afb59e4dee374883a2212d325699`, installed with `--locked`.

```sh
python -m unittest discover -s rebuild -p 'test_*.py'
python rebuild/pipeline.py --release
python rebuild/site_output.py
python rebuild/browser_smoke.py https://luc1ddream.github.io/panelnest/
```

Native Swift diagnostics exercise pinned Aidoku Codable models, root/legacy JSON URLs, resolved assets and malformed controls on macOS. They are not physical iPhone proof. Report problems using the [PanelNest issue form](https://github.com/LUC1DDREAM/panelnest/issues/new?template=bug-report.yml); never include tokens, cookies or private library data.
