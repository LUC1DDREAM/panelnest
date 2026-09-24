# Weeb Central (LUC1D)

## Discovery (package version 9)

Adds separate Author and Artist search fields that route to Weeb Central's
matching advanced-search parameters. Adds Popular, Most Subscribed, Recently Added and Latest Updates listings using
the existing Advanced Search endpoint, its website sort values and explicit
Descending order. Search pages use the live endpoint's 32-entry HTMX offsets;
the View More Results button accurately indicates whether another page exists.
Home Latest Updates now links to its full listing. Hot Updates cards resolve
their series from the trusted cover-host ULID rather than treating chapter
links as series; responsive duplicate cards are collapsed. The standalone Hot
Updates listing is finite and does not repeat on page two. Recommendations are
retained on Home.
No daily/weekly/monthly popularity is exposed: the website Advanced Search
verified via public text extraction only offers unqualified Popularity.

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
