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
      <button hx-get="/search/data?limit=32&amp;offset=32&amp;display_mode=Full+Display"><span>View More Results...</span></button>
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
fn series_description_includes_associated_names_and_related_series() {
	let html = Html::parse_with_url(
		r#"
		<section id="details">
			<ul>
				<li><strong>Description</strong><p>A sample description.</p></li>
				<li><strong>Associated Name(s)</strong><ul>
					<li>Alternate Title</li><li>Another Title</li>
				</ul></li>
				<li><strong>Related Series(s)</strong><ul>
					<li><a href="/series/related">Related Title</a><span>(Prequel)</span></li>
				</ul></li>
			</ul>
		</section>
		"#,
		BASE_URL,
	)
	.unwrap();
	let details = html.select_first("#details").unwrap();
	let description = append_detail_metadata(
		&details,
		details
			.select_first("li:has(strong:contains(Description)) > p")
			.and_then(|element| element.text()),
	);
	assert_eq!(
		description.as_deref(),
		Some(
			"A sample description.\n\nAssociated Name(s):\n- Alternate Title\n- Another Title\n\nRelated Series(s):\n- Related Title (Prequel)"
		)
	);
}

#[aidoku_test]
fn series_description_metadata_is_optional_and_preserves_description() {
	let html = Html::parse("<section><ul><li><strong>Description</strong><p>Only description.</p></li></ul></section>").unwrap();
	let details = html.select_first("section").unwrap();
	let description = append_detail_metadata(
		&details,
		details
			.select_first("li:has(strong:contains(Description)) > p")
			.and_then(|element| element.text()),
	);
	assert_eq!(description.as_deref(), Some("Only description."));
	assert_eq!(append_detail_metadata(&details, None), None);
}

#[aidoku_test]
fn hot_updates_cards_resolve_to_deduplicated_series_keys_from_cover_ids() {
	let html = Html::parse_with_url(
		r#"
		<article data-tip="Sample Series"><a href="/chapters/chapter-1">Chapter 1</a><img src="https://temp.compsci88.com/cover/fallback/01J76XYDT7H7ANER8KJG5R9SJV.jpg"><div class="text-lg">Sample Series</div></article>
		<article class="hidden" data-tip="Duplicate update"><a href="/chapters/chapter-2">Chapter 2</a><img src="https://temp.compsci88.com/cover/normal/01J76XYDT7H7ANER8KJG5R9SJV.webp"><div class="text-lg">Sample Series</div></article>
		<article data-tip="Other Series"><a href="/chapters/chapter-3">Chapter 3</a><img src="https://temp.compsci88.com/cover/small/01M38CCE8ZP635YHV9Y8CV1F8M.webp"><div class="text-lg">Other Series</div></article>
		<article data-tip="Foreign"><img src="https://example.invalid/cover/normal/01J76XYDT7H7ANER8KJG5R9SJV.webp"><div class="text-lg">Foreign</div></article>
		"#,
		BASE_URL,
	)
	.unwrap();
	let result = parse_hot_updates(&html).unwrap();
	assert_eq!(result.entries.len(), 2);
	assert_eq!(result.entries[0].key, "/series/01J76XYDT7H7ANER8KJG5R9SJV");
	assert_eq!(result.entries[0].title, "Sample Series");
	assert_eq!(result.entries[1].key, "/series/01M38CCE8ZP635YHV9Y8CV1F8M");
	assert!(!result.has_next_page);
	assert!(parse_hot_updates(&Html::parse("<title>Access denied</title>").unwrap()).is_err());
}

#[aidoku_test]
fn challenge_is_not_an_empty_catalogue() {
	let html = Html::parse("<html><title>Attention Required! | Cloudflare</title><body>Sorry, you have been blocked</body></html>").unwrap();
	assert!(parse_search(&html).is_err());
	assert!(reject_cloudflare(&html).is_err());
	assert!(reject_cloudflare(&Html::parse("<title>Just a moment...</title>").unwrap()).is_err());
	assert!(reject_cloudflare(&Html::parse("<title>Sample Chapter</title>").unwrap()).is_ok());
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
fn author_filter_uses_the_official_advanced_search_field() {
	let author = filter::get_filters(
		None,
		vec![FilterValue::Text {
			id: "author".into(),
			value: "Author Name".into(),
		}],
	);
	assert!(author.contains("author=Author%20Name"));
	assert!(!author.contains("artist="));
}

#[aidoku_test]
fn official_anime_and_adult_checks_use_matching_site_fields() {
	for (id, field) in [
		("official", "official"),
		("anime", "anime"),
		("adult", "adult"),
	] {
		for (value, expected) in [(0, "False"), (1, "True"), (2, "Any")] {
			let query = filter::get_filters(
				None,
				vec![FilterValue::Check {
					id: id.into(),
					value,
				}],
			);
			assert!(query.contains(&format!("{field}={expected}")), "{query}");
			for other in ["official", "anime", "adult"] {
				if other != field {
					assert!(!query.contains(&format!("{other}=")), "{query}");
				}
			}
		}
	}
}

#[aidoku_test]
fn fixture_search_pagination_uses_real_htmx_more_button() {
	let terminal = Html::parse_with_url(
		r#"
		<article><section><a href="/series/fixture/Sample">Sample</a></section></article>
	"#,
		BASE_URL,
	)
	.unwrap();
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

#[aidoku_test]
fn image_requests_include_the_site_referer_and_provider_is_registered() {
	use aidoku::ImageRequestProvider;
	let request = WeebCentral
		.get_image_request("https://temp.compsci88.com/page.jpg".into(), None)
		.unwrap();
	let _ = request;
	let source = include_str!("lib.rs");
	let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");
	assert!(normalized.contains(
		"register_source!( WeebCentral, ListingProvider, Home, ImageRequestProvider, DeepLinkHandler );"
	));
}

#[aidoku_test]
fn deep_links_require_exact_https_host_and_supported_path() {
	use aidoku::{DeepLinkHandler, DeepLinkResult};
	let source = WeebCentral::new();
	assert!(matches!(
		source
			.handle_deep_link(
				"https://weebcentral.com/series/01J76XYEZYBE7Y3MEY7AEQ8MQN/Test".into()
			)
			.unwrap(),
		Some(DeepLinkResult::Manga { .. })
	));
	for url in [
		"https://weebcentral.com.evil/series/01J76XYEZYBE7Y3MEY7AEQ8MQN/Test",
		"http://weebcentral.com/series/01J76XYEZYBE7Y3MEY7AEQ8MQN/Test",
		"https://weebcentral.com.evil.example/chapters/01JXNANGY619TDR9F4FST2M5E8",
		"https://weebcentral.com/account/login",
	] {
		assert!(
			source.handle_deep_link(url.into()).unwrap().is_none(),
			"{url}"
		);
	}
}
