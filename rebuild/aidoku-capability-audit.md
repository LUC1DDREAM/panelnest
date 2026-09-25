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
| Web login | Asura Scans, HentaiFox | Both provide account-only bookmark listings and a verified browser login flow. IMHentai has an account-only Favorite action, but no verified read/list endpoint. WEBTOON's Help Center places My Series in the mobile app; no supported web subscription-list endpoint was verified. WeebCentral series pages expose Subscribe, but Cloudflare blocks source inspection in this environment, so its login and list endpoints remain unverified. |
| Migration | Asura Scans | A historical source-key change is declared in the source config and maps legacy manga and chapter keys. The other source IDs and stored keys have not changed. |
| Notifications | Asura Scans, HentaiFox | Login setting changes need to refresh account listings and clear saved session data on logout. Other sources have no setting notification work. |
| Chapter deep links | Asura Scans, WeebCentral, WEBTOON | These sites expose distinct chapter or episode identifiers. Single-gallery sources identify a title and its reader pages through the same gallery key. |
| Listing deep links | nhentai, HentaiFox | Official tag routes map to the existing Popular tag Listing IDs. nhentai resolves the official slug through `/api/v2/tags/tag/<slug>` and validates its canonical route; HentaiFox supports its verified latest and popular tag routes. |

## Reviewed, with no useful current site behavior

| Aidoku capability | Decision |
| --- | --- |
| Dynamic settings | Not registered. Current user controls are static settings or dynamic search filters; none of the reviewed sites requires fetching setting definitions from its API. |
| Page image processing | Not registered. The sources receive complete page images from the official CDNs and do not need source-side pixel transformations. |
| Cover image processing | Not registered. Covers are valid image resources from the official sites; no verified source needs custom decoding or transformation. |
| Basic username/password login | Not registered. The two account-enabled sources already use their verified browser login flow; no other site's account flow is verified. IMHentai's account action alone does not make a source login useful without a supported favorites read endpoint. |
| Programmatic base URL | Not registered. Each site has a fixed canonical HTTPS base URL. The pinned SDK explicitly discourages this handler when a static source URL is available. |

## Remaining verification limits

Feature support does not certify successful playback on an Aidoku device. Every source manifest currently records `runtime_tested` and `device_tested` as false. In particular, WeebCentral requests may receive Cloudflare 403 responses, nhentai may rate-limit requests, and HentaiFox authenticated bookmarks have not been verified with an account session. IMHentai v27 now falls back to the complete reader document when the inline manifest is absent from the script-element query; fixtures cover the captured 50-page reader response and the fallback path. Native playback still needs an Aidoku device run.

## Aidoku model-field coverage

The SDK's manga fields (metadata, status, content rating, viewer, update strategy and URLs) and chapter fields (numbering, date, language, scanlator, thumbnail and lock state) were compared against each source parser. WEBTOON Originals already populate episode dates from the official API. The public CANVAS list also displays date-only episode labels, so source version 22 now parses verified English month-name and ISO formats into UTC-midnight `Chapter.date_uploaded`; unknown or invalid labels remain unset. IMHentai v28 and HentaiFox v32 map their explicit gallery artist metadata to `Manga.artists` rather than misclassifying it as `Manga.authors`; neither site exposes a separate Authors field. Current official Originals episode API responses expose no lock indicator, so WEBTOON `Chapter.locked` cannot be set reliably from that endpoint. Other chapter fields are populated only where the site exposes verified values. No reviewed site exposes a dependable next-update timestamp.

The practical next step for future site changes is to recheck the applicable SDK traits, source manifest, registration macro, fixtures, and live site behavior before deciding whether a new handler is useful.

## Live discovery spot-checks

- IMHentai gallery pages expose an account-only Favourite control, and the official `user.14235631.js` posts additions to `/user/add_fav.php`. Logged-out requests to `/profile/` redirect to `/login/`. No official read endpoint or authenticated favorites response was verifiable from the public page and scripts, so an Aidoku WebLogin/Bookmarks feature needs further endpoint evidence before it can be implemented reliably.
- [Asura Scans Browse](https://asurascans.com/browse) currently exposes search, genre, creator, minimum-chapter, status and type controls. Its public client bundle builds `/api/series` requests from those values. “Hide Bookmarked” is applied in the browser after the result list is fetched from the user's bookmark state; it is not sent as a series-search parameter. The Aidoku source already provides the account Bookmarks listing. A site-equivalent hide filter would need extra account lookups and client-side pagination logic.
- [Weeb Central Advanced Search](https://weebcentral.com/search) currently exposes six sorts, order, official translation, anime adaptation, adult-content, status, type and all listed tags. These map to the source's dynamic filters and stable per-tag Popular listings. The source's logged-out series page advertises Subscribe, but direct requests from this environment receive Cloudflare HTTP 403, so the account workflow and subscription listing cannot be grounded here.
- [WEBTOON series pages](https://www.webtoons.com/en/drama/miyeon/list?title_no=8473) expose RSS and Subscribe. The official [Help Center](https://webtoon.zendesk.com/hc/en-us/articles/360051601031-How-do-I-manage-my-WEBTOON-activity) describes My Series under the mobile app. The source already supports its public seven language regions, discovery genres and sorts. An Aidoku web login and subscribed-series listing would need a verified supported web endpoint and authenticated response before it could be safely added.
