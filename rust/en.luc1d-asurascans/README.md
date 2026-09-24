# Asura Scans

## Discovery (package version 8)

The comic website PopularSidebar offers Weekly, Monthly and All Time, using
`https://api.asurascans.com/api/trending/{week|month|all}?limit=10`.
These are now Home scrollers and named listings. Each is the website's top ten,
in server order, with no pagination or all-time fallback for a failed period.
No Popular Today listing is exposed: no daily option was evidenced in that UI.
Existing Trending Comics, Latest Updates, Ranking and authenticated Bookmarks
remain. Package version 8 exposes the live browse facets in Aidoku: sort,
status (including Axed), series type, the site's current genre list, creator,
artist and minimum chapter count. Creator and artist filters now submit their
values as the site's `author` and `artist` query parameters; previously those
text filters appeared in Aidoku but were dropped when building the browse URL.
Genre names and slugs are read from the public
BrowseFilters payload, so new and renamed genres do not rely on a stale copy.
Search emits the same query parameter names and values as the site's browse
page. If that payload is unavailable, the other browse filters remain usable.
Deep links and chapter/authentication guards are unchanged.

Bookmark pagination now stops using the API's total item count and the current
offset plus returned page length. This prevents nearly every page from being
marked as having another page when the total exceeds the page number.

`fixtures/popular-week.json` is a reduced capture of the first two public entries
from the week endpoint on 2026-09-08; only slug, title, cover URL and public URL
are retained. No images or reader pages were downloaded.

```sh
cargo test --locked
cargo test --locked live_ -- --ignored --nocapture
cargo build --release --locked
aidoku package
aidoku verify package.aix
```

The opt-in test exercises actual Home and ListingProvider metadata calls. Default
fixture tests require no network. Both modes passed in the WASM runner. This is
not iOS rendering or paid-account access proof.


## Timed locks and page access

The Aidoku SDK exposes a lock flag and title, but no dedicated unlock date or
live countdown field. Asura supplies an explicit early-access deadline, shown
as a countdown in the chapter title when chapters are refreshed:

`Unlocks in 6h 1m (at refresh); release 2026-09-07 20:48:07 UTC`

Here `h` means hours and `m` minutes. Remaining minutes round upward; fractional
seconds do not unlock early. Refresh the chapter list to update the displayed
countdown. There is no background ticking UI. Existing titles are retained before
the status; chapter keys, numbers, list order and publication dates are preserved.
The absolute release time is UTC, including conversion from explicit offsets.

- A future explicit deadline marks early access as locked for non-subscribers.
- Premium chapters with no valid deadline say release time unknown. Publication
  dates and coin/premium flags are never used to invent release deadlines.
- An elapsed deadline does not override a still-premium flag. Refresh checks the
  site's current state; explicitly free chapters with elapsed/no deadlines open.
- Missing/invalid premium state fails closed, including for subscribers.
- A cached subscription alone is insufficient: it must refresh successfully.
  Existing authenticated reader requests remain subject to server authorization.
- `get_page_list` refetches current series metadata and rejects locked or missing
  chapters before either API page extraction or the HTML reader fallback. Neither
  cached chapter lock bits nor expired timers grant access. Metadata/network
  errors propagate; they do not fall through to reader requests.
- The existing Show Locked Chapters setting still controls list visibility.

## Verification

From this source directory, with the Aidoku WASM test runner on PATH:

```sh
cargo test
cargo fmt --check
aidoku package
aidoku verify package.aix
```

`tests/fixtures/public-chapter-metadata.json` contains previously captured public
chapter metadata and provenance, not reader pages. `synthetic-lock-cases.json` is
explicitly synthetic boundary coverage. Tests cover lock/title mapping, authorized
subscriber behavior, unknown metadata denial, elapsed/fractional/offset deadlines,
page gating, publication timestamps and chapter identity/order.

Publication timestamps now use RFC3339 parsing directly. The pinned WASM test
runner's Swift-date-format converter does not handle quoted literals such as
`'T'` and `'Z'`; valid dates returned None, not merely the epoch fixture. Regression
assertions retain actual timestamps, including epoch zero, modern dates,
fractional seconds and timezone offsets. Invalid dates remain None.

Package verification is structural, not device proof. No iOS/device rendering,
live paid-account access, reader images, or download flow was exercised.


## Search filters

Sort, status and type stay in `res/filters.json`. The source reads current genre
names and URL slugs from the public `BrowseFilters` payload each time Aidoku
loads dynamic filters; author, artist and minimum chapter count are exposed as
additional dynamic fields. That keeps the visible genre list aligned with the
site without manual tag-file updates. If the browse payload is unavailable,
Aidoku still receives the creator and chapter-count filters plus the static
sort/status/type filters.
