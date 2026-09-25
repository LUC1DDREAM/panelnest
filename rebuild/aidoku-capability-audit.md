# Aidoku source capability audit

Audited 2026-09-25 against the pinned Aidoku Rust SDK revision `e1320b0a2e11afb59e4dee374883a2212d325699` in `work/aidoku-rs-inspect/crates/lib/src/structs/source.rs`. The SDK lists the following source traits and the six sources implement them where the site's verified behavior benefits from them.

## Implemented across all six sources

| Aidoku capability | Asura Scans | WeebCentral | nhentai | WEBTOON | IMHentai | HentaiFox |
| --- | --- | --- | --- | --- | --- | --- |
| Required source operations: search, details, chapters, pages | yes | yes | yes | yes | yes | yes |
| Home and fixed listings | yes | yes | yes | yes | yes | yes |
| Dynamic listings | yes | yes | yes | yes | yes | yes |
| Dynamic search filters | yes | yes | yes | yes | yes | yes |
| Deep links | yes | yes | yes | yes | yes | yes |
| Image request headers | yes | yes | yes | yes | yes | yes |
| Page descriptions | yes | yes | yes | yes | yes | yes |

The feature list and per-source exceptions are recorded in `rebuild/sources.json`. Each implementation registers its applicable handlers in `register_source!`; source fixture tests exercise provider behavior, and the publication checks keep the source manifest aligned with the built package. All six sites need custom request headers or referers for at least some cover or page images, so the image request handler is registered everywhere.

## Applicable to selected sources

| Aidoku capability | Sources | Reason |
| --- | --- | --- |
| Alternate covers | nhentai | The official API provides distinct cover variants; the source validates and exposes those variants. The other reviewed sites expose one verified cover per title. |
| Web login | Asura Scans, HentaiFox | Both provide account-only bookmark listings and a verified browser login flow. Other sources have no verified account feature used by these sources. |
| Migration | Asura Scans | A historical source-key change is declared in the source config and maps legacy manga and chapter keys. The other source IDs and stored keys have not changed. |
| Notifications | Asura Scans, HentaiFox | Login setting changes need to refresh account listings and clear saved session data on logout. Other sources have no setting notification work. |
| Chapter deep links | Asura Scans, WeebCentral, WEBTOON | These sites expose distinct chapter or episode identifiers. Single-gallery sources identify a title and its reader pages through the same gallery key. |

## Reviewed, with no useful current site behavior

| Aidoku capability | Decision |
| --- | --- |
| Dynamic settings | Not registered. Current user controls are static settings or dynamic search filters; none of the reviewed sites requires fetching setting definitions from its API. |
| Page image processing | Not registered. The sources receive complete page images from the official CDNs and do not need source-side pixel transformations. |
| Cover image processing | Not registered. Covers are valid image resources from the official sites; no verified source needs custom decoding or transformation. |
| Basic username/password login | Not registered. The two account-enabled sources already use their verified browser login flow; no other site's account flow is needed or verified. |
| Programmatic base URL | Not registered. Each site has a fixed canonical HTTPS base URL. The pinned SDK explicitly discourages this handler when a static source URL is available. |

## Remaining verification limits

Feature support does not certify successful playback on an Aidoku device. Every source manifest currently records `runtime_tested` and `device_tested` as false. In particular, WeebCentral requests may receive Cloudflare 403 responses, nhentai may rate-limit requests, and HentaiFox authenticated bookmarks have not been verified with an account session. IMHentai v26 has fixture coverage for its live reader manifest and direct HTTP checks, but an Aidoku device run still needs the app's error output and installed source version to diagnose the user's chapter-opening failure.

The practical next step for future site changes is to recheck the applicable SDK traits, source manifest, registration macro, fixtures, and live site behavior before deciding whether a new handler is useful.
