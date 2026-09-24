Version 15 exposes all 38 official search tags as separate paginated Popular listings. Each listing uses a stable genre ID and the same included_tag search path and offset pagination as the site Advanced Search. Cloudflare may still block native requests in some environments.

# Weeb Central (LUC1D)

## Discovery (package version 14)

Version 14 exposes Aidoku search filters for the site's six sorts, author name,
tri-state Official Translation / Anime Adaptation / Adult Content, and the
official include/exclude tag list. These controls map to the site's advanced
search fields.

Version 13 adds tri-state filters for Official Translation, Anime Adaptation, and Adult
Content, mapped to the site's corresponding advanced-search parameters. Adds Popular, Most Subscribed, Recently Added and Latest Updates listings using
the existing Advanced Search endpoint, all six website sort values and explicit
Descending order. Best Match and Alphabetical are also available directly as
browse listings. Search pages use the live endpoint's 32-entry HTMX offsets;
the View More Results button accurately indicates whether another page exists.
Home Latest Updates now links to its full listing. Home, details, chapter-list
and reader requests now report Cloudflare challenge pages as access errors
instead of returning successful empty data. Hot Updates cards resolve
their series from the trusted cover-host ULID rather than treating chapter
links as series; responsive duplicate cards are collapsed. The standalone Hot
Updates listing is finite and does not repeat on page two. Recommendations are
retained on Home.
No daily/weekly/monthly popularity is exposed: the website Advanced Search
verified via public text extraction only offers unqualified Popularity.

Series details include the site's associated names and related series alongside
the description when those fields are present. This preserves alternate title
search terms and relationship labels in Aidoku's detail view.

Evidence: https://weebcentral.com/search exposes Best Match, Alphabet,
Popularity, Subscribers, Recently Added, Latest Updates and Ascending/Descending.
A public text extraction on 2026-09-08 succeeded, but direct HTTP and the native
WASM live smoke encountered Cloudflare. The browser harness additionally needed
user remote-debug approval, so it was not retried. No access controls were
bypassed. Sort mappings are deterministic-tested; live discovery is NOT proven
working on this network. Challenge pages now raise an error rather than looking
like an empty successful search.

```sh
cargo test --locked
# Opt-in, public Kobato metadata search, never downloads images:
cargo test --locked live_ -- --ignored --nocapture
cargo build --release --locked
aidoku package
aidoku verify package.aix
```

Default tests: 10 passed, 1 ignored. Live smoke: failed with explicit website
access-blocked error. Release build and package/schema/icon/WASM verification
passed. Tests preserve the existing synthetic nonexplicit HTML fixtures and
add sort mapping and challenge rejection. No iOS/device proof.

Search rejects page numbers below 1 before calculating the offset, matching the
listing handler and avoiding invalid negative-offset requests. Deep links require
the exact HTTPS hostname and a series or chapter path, rejecting lookalike hosts
and unrelated account URLs. The public capability index also records the
existing image request provider, which sets the site Referer for chapter page
requests.
