use super::*;
use aidoku::imports::html::Html;
use aidoku_test::aidoku_test;

#[aidoku_test]
fn discovery_listings_use_real_search_sorts() {
	for (id, sort) in [
		("popular", "Popularity"),
		("subscribers", "Subscribers"),
		("new", "Recently Added"),
		("latest", "Latest Updates"),
	] {
		let index = filter::listing_sort(id).unwrap();
		let query = filter::get_filters(
			None,
			vec![FilterValue::Sort {
				id: "sort".into(),
				index,
				ascending: false,
			}],
		);
		assert!(query.contains(&format!("sort={}", sort.replace(' ', "%20"))));
		assert!(query.contains("order=Descending"));
	}
	assert!(filter::listing_sort("popular-day").is_none());
	assert!(filter::listing_sort("hot").is_none());
}

#[aidoku_test]
#[ignore]
fn live_discovery_safe_metadata_only() {
	let page = WeebCentral::new()
		.get_search_manga_list(
			Some("Kobato".into()),
			1,
			vec![FilterValue::Sort {
				id: "sort".into(),
				index: filter::listing_sort("popular").unwrap(),
				ascending: false,
			}],
		)
		.unwrap();
	assert!(!page.entries.is_empty());
	assert!(
		page.entries
			.iter()
			.all(|m| m.key.starts_with("/series/") && !m.title.is_empty())
	);
}

#[aidoku_test]
fn fixture_search_parses_and_skips_invalid_cards() {
	// Synthetic metadata, no real comic content or images fetched.
	let html = Html::parse_with_url(r#"
      <article><section><a href="/series/fixture/Sample">Official Sample</a><img src="/cover.png"></section></article>
      <article><section><a>No URL</a><img src="/cover.png"></section></article>
      <article><section><a href="https://example.invalid/foreign">Foreign</a><img src="/cover.png"></section></article>
    "#, BASE_URL).unwrap();
	let result = parse_search(&html).unwrap();
	assert_eq!(result.entries.len(), 1);
	assert_eq!(result.entries[0].key, "/series/fixture/Sample");
	assert_eq!(result.entries[0].title, "Sample");
	assert_eq!(
		result.entries[0].cover.as_deref(),
		Some("https://weebcentral.com/cover.png")
	);
	assert!(result.has_next_page);
}

#[aidoku_test]
fn challenge_is_not_an_empty_catalogue() {
	let html = Html::parse("<html><title>Attention Required! | Cloudflare</title><body>Sorry, you have been blocked</body></html>").unwrap();
	assert!(parse_search(&html).is_err());
}

#[aidoku_test]
fn fixture_empty_search_stops_pagination() {
	let html = Html::parse("<main>No results</main>").unwrap();
	let result = parse_search(&html).unwrap();
	assert!(result.entries.is_empty());
	assert!(!result.has_next_page);
}

#[aidoku_test]
fn search_query_does_not_inject_parameters() {
	let query = filter::get_filters(Some("Sample&offset=999".into()), vec![]);
	assert!(query.contains("%26"));
	assert!(!query.contains("&offset=999"));
}

#[aidoku_test]
fn fixture_search_pagination_uses_real_htmx_more_button() {
	let terminal = Html::parse_with_url(r#"
		<article><section><a href="/series/fixture/Sample">Sample</a></section></article>
	"#, BASE_URL).unwrap();
	assert!(!parse_search(&terminal).unwrap().has_next_page);
	let more = Html::parse_with_url(r#"
		<article><section><a href="/series/fixture/Sample">Sample</a></section></article>
		<button hx-get="/search/data?limit=32&amp;offset=32&amp;display_mode=Full+Display"><span>View More Results...</span></button>
	"#, BASE_URL).unwrap();
	assert!(parse_search(&more).unwrap().has_next_page);
}

#[aidoku_test]
fn search_rejects_nonpositive_page_before_request() {
	use aidoku::Source;
	let source = WeebCentral::new();
	assert!(source.get_search_manga_list(None, 0, vec![]).is_err());
	assert!(source.get_search_manga_list(None, -1, vec![]).is_err());
}
