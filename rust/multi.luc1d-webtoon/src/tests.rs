use super::*;
#[aidoku_test]
fn captured_discovery_retains_site_order_and_full_list() {
	let html = Html::parse(include_str!("../tests/fixtures/discovery.html")).unwrap();
	let result = parse_search(&html);
	assert_eq!(result.entries.len(), 595);
	assert!(result.entries[0].key.contains("title_no=8135"));
	assert!(!result.has_next_page);
}
#[aidoku_test]
fn discovery_components_keep_listings_and_site_order() {
	let component = discovery_component(
		"popular",
		"Popular",
		vec![Manga {
			key: "fixture".into(),
			..Default::default()
		}],
	);
	assert_eq!(component.title.as_deref(), Some("Popular"));
	match component.value {
		aidoku::HomeComponentValue::Scroller { entries, listing } => {
			assert_eq!(entries.len(), 1);
			assert_eq!(listing.unwrap().id, "popular");
		}
		_ => panic!("expected browsable scroller"),
	}
}
#[cfg(feature = "live-tests")]
#[aidoku_test]
fn live_discovery_home_and_listings() {
	use aidoku::{Home, Listing, ListingProvider};
	let source = Webtoon::new();
	let home = source.get_home().unwrap();
	assert_eq!(home.components.len(), 4);
	for id in [
		"popular",
		"likes",
		"date",
		"genre-fantasy",
		"genre-romance",
		"genre-action",
		"genre-comedy",
	] {
		let result = source
			.get_manga_list(
				Listing {
					id: id.into(),
					..Default::default()
				},
				1,
			)
			.unwrap();
		assert!(!result.entries.is_empty());
		assert!(!result.has_next_page);
		assert!(
			source
				.get_manga_list(
					Listing {
						id: id.into(),
						..Default::default()
					},
					2
				)
				.unwrap()
				.entries
				.is_empty()
		);
	}
	assert!(
		source
			.get_manga_list(
				Listing {
					id: "popular-today".into(),
					..Default::default()
				},
				1
			)
			.is_err()
	);
}
#[aidoku_test]
fn discovery_listing_routes_are_site_specific() {
	assert_eq!(
		discovery_path("popular"),
		Some("/en/genres/drama?sortOrder=MANA")
	);
	assert_eq!(
		discovery_path("likes"),
		Some("/en/genres/drama?sortOrder=LIKEIT")
	);
	assert_eq!(
		discovery_path("date"),
		Some("/en/genres/drama?sortOrder=UPDATE")
	);
	assert_eq!(
		discovery_path("genre-fantasy"),
		Some("/en/genres/fantasy?sortOrder=MANA")
	);
	assert_eq!(discovery_path("popular-today"), None);
	assert_eq!(discovery_path("https://example.invalid"), None);
}

#[aidoku_test]
fn dynamic_filters_expose_every_official_genre_and_sort() {
	let filters = Webtoon.get_dynamic_filters().unwrap();
	assert_eq!(filters.len(), 2);
	match &filters[0].kind {
		FilterKind::Select {
			is_genre,
			options,
			ids,
			..
		} => {
			assert!(*is_genre);
			assert_eq!(options.len(), 17);
			assert_eq!(ids.as_ref().unwrap().len(), 17);
			assert_eq!(ids.as_ref().unwrap()[6].as_ref(), "super_hero");
			assert_eq!(ids.as_ref().unwrap()[7].as_ref(), "sf");
		}
		_ => panic!("expected genre selector"),
	}
	match &filters[1].kind {
		FilterKind::Sort {
			options,
			can_ascend,
			..
		} => {
			assert!(!can_ascend);
			assert_eq!(options.len(), 3);
		}
		_ => panic!("expected sort selector"),
	}
}

#[aidoku_test]
fn dynamic_genre_and_sort_route_to_official_catalogs() {
	assert_eq!(
		filtered_discovery_path(&[
			FilterValue::Select {
				id: "genre".into(),
				value: "sf".into()
			},
			FilterValue::Sort {
				id: "sort".into(),
				index: 2,
				ascending: false
			},
		])
		.unwrap(),
		"/en/genres/sf?sortOrder=UPDATE"
	);
	assert_eq!(
		filtered_discovery_path(&[FilterValue::Select {
			id: "genre".into(),
			value: "drama".into(),
		}])
		.unwrap(),
		"/en/genres/drama?sortOrder=MANA"
	);
	assert_eq!(
		filtered_discovery_path(&[FilterValue::Sort {
			id: "sort".into(),
			index: 1,
			ascending: false,
		}])
		.unwrap(),
		"/en/genres/drama?sortOrder=LIKEIT"
	);
	assert!(
		filtered_discovery_path(&[FilterValue::Select {
			id: "genre".into(),
			value: "untrusted/path".into(),
		}])
		.is_err()
	);
	assert!(
		filtered_discovery_path(&[FilterValue::Sort {
			id: "sort".into(),
			index: 99,
			ascending: false,
		}])
		.is_err()
	);
}

#[aidoku_test]
fn search_rejects_nonpositive_pages() {
	assert!(Webtoon.get_search_manga_list(None, 0, vec![]).is_err());
}
use aidoku::imports::html::Html;
use aidoku_test::aidoku_test;
#[aidoku_test]
fn live_search_fixture() {
	let html = Html::parse_with_url(
		include_str!("../tests/fixtures/search.html"),
		"https://www.webtoons.com",
	)
	.unwrap();
	let result = parse_search(&html);
	assert!(
		result
			.entries
			.iter()
			.any(|m| m.title == "Space Boy" && m.key == "/en/sf/space-boy/list?title_no=400")
	);
	assert!(!result.has_next_page);
}

#[aidoku_test]
fn details_fixture() {
	let h = Html::parse(include_str!("../tests/fixtures/details.html")).unwrap();
	let m = parse_details(Manga::default(), &h).unwrap();
	assert_eq!(m.title, "Space Boy");
	assert!(m.description.unwrap().len() > 30);
	assert!(m.authors.unwrap().join(",").contains("Stephen"));
}
#[aidoku_test]
fn episodes_fixture_and_cursor() {
	let value = serde_json::from_str(include_str!("../tests/fixtures/episodes.json")).unwrap();
	let (chapters, cursor) = parse_episodes(value).unwrap();
	assert_eq!(chapters.len(), 100);
	assert_eq!(cursor, Some(102));
	assert_eq!(chapters[0].title.as_deref(), Some("Ep. 1"));
	assert_eq!(chapters[0].date_uploaded, Some(1425564010));
	assert!(parse_episodes(serde_json::json!({"error":"denied"})).is_err());
}
#[aidoku_test]
fn pages_fixture_and_restricted_response() {
	let h = Html::parse(include_str!("../tests/fixtures/viewer.html")).unwrap();
	assert!(!parse_pages(&h).unwrap().is_empty());
	assert!(parse_pages(&Html::parse("<main>Log in or purchase</main>").unwrap()).is_err());
}
#[aidoku_test]
fn reject_foreign_links_and_query_injection() {
	assert!(official_path("https://webtoons.com.evil/en/x/list?title_no=1").is_none());
	assert!(official_path("//evil/list?title_no=1").is_none());
	assert!(official_path("/en/x/list?title_no=notnumeric").is_none());
	assert_eq!(encode_query("a&b é"), "a%26b%20%C3%A9");
}

#[aidoku_test]
fn series_deep_links_resolve_supported_hosts_and_reject_foreign_hosts() {
	use aidoku::{DeepLinkHandler, DeepLinkResult};
	let source = Webtoon;
	let mobile = "https://m.webtoons.com/en/sf/space-boy/list?title_no=400";
	let desktop = "https://www.webtoons.com/en/sf/space-boy/list?title_no=400";
	assert_eq!(
		source.handle_deep_link(mobile.into()).unwrap(),
		Some(DeepLinkResult::Manga {
			key: "/en/sf/space-boy/list?title_no=400".into()
		})
	);
	assert_eq!(official_path(desktop), official_path(mobile));
	assert!(
		source
			.handle_deep_link(
				"https://www.webtoons.com.evil/en/sf/space-boy/list?title_no=400".into()
			)
			.unwrap()
			.is_none()
	);
	assert!(
		source
			.handle_deep_link("https://example.org/en/sf/space-boy/list?title_no=400".into())
			.unwrap()
			.is_none()
	);
	assert!(
		source
			.handle_deep_link(
				"https://m.webtoons.com/en/sf/space-boy/list?title_no=notnumeric".into()
			)
			.unwrap()
			.is_none()
	);
}

#[cfg(feature = "live-tests")]
#[aidoku_test]
fn live_public_end_to_end() {
	let source = Webtoon::new();
	let result = source
		.get_search_manga_list(Some("space boy".into()), 1, vec![])
		.unwrap();
	let manga = result
		.entries
		.into_iter()
		.find(|m| m.key == "/en/sf/space-boy/list?title_no=400")
		.unwrap();
	let manga = source.get_manga_update(manga, true, true).unwrap();
	assert_eq!(manga.title, "Space Boy");
	let chapters = manga.chapters.as_ref().unwrap();
	assert!(chapters.len() > 100);
	let chapter = chapters
		.iter()
		.find(|c| parameter(&c.key, "episode_no") == Some("1"))
		.unwrap()
		.clone();
	let pages = source.get_page_list(manga, chapter).unwrap();
	assert!(!pages.is_empty());
	let browse = source.get_search_manga_list(None, 1, vec![]).unwrap();
	assert!(!browse.entries.is_empty());
}
