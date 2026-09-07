use super::*;
use aidoku::imports::html::Html;
use aidoku_test::aidoku_test;

#[aidoku_test]
fn fixture_search_parses_and_skips_invalid_cards() {
    // Synthetic metadata, no real comic content or images fetched.
    let html = Html::parse_with_url(r#"
      <article><section><a href="/series/fixture/Sample">Official Sample</a><img src="/cover.png"></section></article>
      <article><section><a>No URL</a><img src="/cover.png"></section></article>
      <article><section><a href="https://example.invalid/foreign">Foreign</a><img src="/cover.png"></section></article>
    "#, BASE_URL).unwrap();
    let result = parse_search(&html);
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].key, "/series/fixture/Sample");
    assert_eq!(result.entries[0].title, "Sample");
    assert_eq!(result.entries[0].cover.as_deref(), Some("https://weebcentral.com/cover.png"));
    assert!(result.has_next_page);
}

#[aidoku_test]
fn fixture_empty_search_stops_pagination() {
    let html = Html::parse("<main>No results</main>").unwrap();
    let result = parse_search(&html);
    assert!(result.entries.is_empty());
    assert!(!result.has_next_page);
}

#[aidoku_test]
fn search_query_does_not_inject_parameters() {
    let query = filter::get_filters(Some("Sample&offset=999".into()), vec![]);
    assert!(query.contains("%26"));
    assert!(!query.contains("&offset=999"));
}
