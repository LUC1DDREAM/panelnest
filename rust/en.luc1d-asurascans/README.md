# Asura Scans

## Timed locks and page access

The pinned Aidoku SDK exposes `Chapter.locked` and a title, not an unlock date or
live countdown field. A chapter with the site's explicit `early_access_until`
shows a refresh-based title, for example:

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


## Updating Genres

On https://asurascans.com/browse, run:

```js
(() => {
  const island = document.querySelector('astro-island[component-url*="BrowseFilters"]');
  if (!island) {
    console.error('BrowseFilters astro-island not found');
    return;
  }

  const rawProps = island.getAttribute('props');
  if (!rawProps) {
    console.error('No props attribute found on astro-island');
    return;
  }

  // decode html entities
  const textarea = document.createElement('textarea');
  textarea.innerHTML = rawProps;
  const decodedProps = textarea.value;
  const props = JSON.parse(decodedProps);

  const genreEntries = props.availableGenres?.[1] || [];
  const genres = genreEntries.map((entry) => {
    const g = entry?.[1] || {};
    return {
      id: g.id?.[1],
      name: g.name?.[1],
      slug: g.slug?.[1],
    };
  }).filter(g => g.id != null && g.name && g.slug);

  const options = genres.map(g => g.name);
  const ids = genres.map(g => g.slug);

  console.log('options:', JSON.stringify(options));
  console.log('ids:', JSON.stringify(ids));
})();
```
