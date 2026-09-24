use super::*;
use aidoku::FilterKind;

#[aidoku_test]
fn completed_series_skip_library_refresh_and_other_statuses_keep_refreshing() {
	assert_eq!(library_update_strategy(MangaStatus::Completed), UpdateStrategy::Never);
	for status in [MangaStatus::Ongoing, MangaStatus::Hiatus, MangaStatus::Cancelled, MangaStatus::Unknown] {
		assert_eq!(library_update_strategy(status), UpdateStrategy::Always);
	}
}
#[aidoku_test]
fn reader_pages_carry_ordered_descriptions_without_network_requests() {
	use aidoku::PageDescriptionProvider;
	let doc = Html::parse(
		r#"<div id="_imageList"><img data-url="https://webtoon-phinf.pstatic.net/episode/first.jpg"><img data-url="https://webtoon-phinf.pstatic.net/episode/second.webp"><img data-url="http://webtoon-phinf.pstatic.net/episode/rejected.jpg"></div>"#,
	)
	.unwrap();
	let pages = parse_pages(&doc).unwrap();
	assert_eq!(pages.len(), 2);
	assert!(pages.iter().all(|page| page.has_description));
	let source = Webtoon;
	let mut pages = pages.into_iter();
	assert_eq!(source.get_page_description(pages.next().unwrap()).unwrap(), "Page 1");
	assert_eq!(source.get_page_description(pages.next().unwrap()).unwrap(), "Page 2");
	assert!(source.get_page_description(Page::default()).is_err());
}

#[aidoku_test]
fn captured_discovery_retains_site_order_and_full_list() {
	let html = Html::parse(include_str!("../tests/fixtures/discovery.html")).unwrap();
	let result = parse_search(&html);
	assert_eq!(result.entries.len(), 595);
	assert!(result.entries[0].key.contains("title_no=8135"));
	assert!(!result.has_next_page);
}

#[aidoku_test]
fn discovery_options_follow_current_official_genre_and_sort_links() {
	let html = Html::parse_with_url(
		include_str!("../tests/fixtures/discovery.html"),
		"https://www.webtoons.com/en/genres/drama?sortOrder=MANA",
	)
	.unwrap();
	let (genres, sorts) = parse_discovery_options("en", &html).unwrap();
	assert_eq!(genres.len(), 17);
	assert_eq!(genres[0], ("drama".into(), "DRAMA".into()));
	assert!(genres.contains(&("tiptoon".into(), "INFORMATIVE".into())));
	assert_eq!(sorts.len(), 3);
	assert_eq!(sorts[0].0, "MANA");
	assert_eq!(sorts[1].0, "LIKEIT");
	assert_eq!(sorts[2].0, "UPDATE");
	assert_eq!(parse_discovery_options("zh-hant", &html), None);
}

#[aidoku_test]
fn dynamic_genre_routes_accept_new_official_genres_but_reject_bad_sorts() {
	assert_eq!(
		discovery_path_for_language("genre-new_site_genre", "en").as_deref(),
		Some("/en/genres/new_site_genre?sortOrder=MANA")
	);
	assert_eq!(
		discovery_path_for_language("genre-sort-new_site_genreLIKEIT", "en").as_deref(),
		Some("/en/genres/new_site_genre?sortOrder=LIKEIT")
	);
	assert_eq!(
		discovery_path_for_language("genre-sort-new_site_genreBAD", "en"),
		None
	);
	assert_eq!(discovery_path_for_language("genre-../evil", "en"), None);
	assert_eq!(
		discovery_path_for_language("genre-sort-../evilLIKEIT", "en"),
		None
	);
	assert_eq!(discovery_path_for_language("genre-drama", "../evil"), None);
}

#[aidoku_test]
fn newly_discovered_genres_are_searchable_and_get_dynamic_listings() {
	let path = filtered_discovery_path_for_language(
		&[
			FilterValue::Select {
				id: "genre".into(),
				value: "new_site_genre".into(),
			},
			FilterValue::Sort {
				id: "sort".into(),
				index: 1,
				ascending: false,
			},
		],
		"en",
	)
	.unwrap();
	assert_eq!(path, "/en/genres/new_site_genre?sortOrder=LIKEIT");
	assert!(
		filtered_discovery_path_for_language(
			&[FilterValue::Select {
				id: "genre".into(),
				value: "../evil".into(),
			}],
			"en",
		)
		.is_err()
	);

	let listings = dynamic_genre_listings(
		vec![
			("drama".into(), "Drama".into()),
			("new_site_genre".into(), "New Genre".into()),
		],
		vec![
			("MANA".into(), "Popularity".into()),
			("LIKEIT".into(), "Likes".into()),
			("UPDATE".into(), "Date".into()),
		],
	);
	assert_eq!(listings.len(), 3);
	assert_eq!(listings[0].id, "genre-sort-new_site_genreMANA");
	assert_eq!(listings[0].name, "New Genre: By Popularity");
	assert_eq!(listings[1].id, "genre-sort-new_site_genreLIKEIT");
	assert_eq!(listings[2].id, "genre-sort-new_site_genreUPDATE");
}

#[aidoku_test]
fn dynamic_listings_keep_existing_genres_in_the_static_catalog() {
	let listings = dynamic_genre_listings(
		vec![("drama".into(), "Drama".into())],
		vec![("MANA".into(), "Popularity".into())],
	);
	assert!(listings.is_empty());
}

#[aidoku_test]
fn static_discovery_manifest_has_unique_canonical_genre_sort_listings() {
	let manifest: serde_json::Value = serde_json::from_str(include_str!("../res/source.json")).unwrap();
	let listings = manifest["listings"].as_array().unwrap();
	let mut ids = Vec::new();
	let mut names = Vec::new();
	for listing in listings {
		ids.push(listing["id"].as_str().unwrap());
		names.push(listing["name"].as_str().unwrap());
	}
	ids.sort_unstable();
	ids.dedup();
	names.sort_unstable();
	names.dedup();
	assert_eq!(listings.len(), 51);
	assert_eq!(ids.len(), listings.len());
	assert_eq!(names.len(), listings.len());
	assert!(listings.iter().all(|listing| listing["id"].as_str().unwrap().starts_with("genre-sort-")));
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
	assert_eq!(
		component.subtitle.as_deref(),
		Some("Drama genre; site order (not a daily chart)")
	);
	match component.value {
		aidoku::HomeComponentValue::Scroller { entries, listing } => {
			assert_eq!(entries.len(), 1);
			assert_eq!(listing.unwrap().id, "popular");
		}
		_ => panic!("expected browsable scroller"),
	}
}

#[aidoku_test]
fn progressive_home_starts_with_stable_sections_and_dynamic_genre_links() {
	let layout = empty_home_layout();
	assert_eq!(layout.components.len(), 4);
	assert_eq!(
		layout.components[0].title.as_deref(),
		Some("Drama: By Popularity")
	);
	assert_eq!(layout.components[3].title.as_deref(), Some("Browse Genres"));
	let genres = vec![("new_site_genre".into(), "New Genre".into())];
	let component = browse_genres_component(&genres);
	match component.value {
		aidoku::HomeComponentValue::Links(links) => {
			assert_eq!(links.len(), 1);
			assert_eq!(links[0].title, "New Genre");
			match links[0].value.as_ref().unwrap() {
				aidoku::LinkValue::Listing(listing) => {
					assert_eq!(listing.id, "genre-new_site_genre");
				}
				_ => panic!("genre link must open its listing"),
			}
		}
		_ => panic!("expected genre links"),
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
		discovery_path("popular").as_deref(),
		Some("/en/genres/drama?sortOrder=MANA")
	);
	assert_eq!(
		discovery_path("likes").as_deref(),
		Some("/en/genres/drama?sortOrder=LIKEIT")
	);
	assert_eq!(
		discovery_path("date").as_deref(),
		Some("/en/genres/drama?sortOrder=UPDATE")
	);
	assert_eq!(
		discovery_path("genre-fantasy").as_deref(),
		Some("/en/genres/fantasy?sortOrder=MANA")
	);
	for (slug, _) in GENRES {
		assert_eq!(
			discovery_path(&format!("genre-{slug}")),
			Some(format!("/en/genres/{slug}?sortOrder=MANA"))
		);
	}
	assert_eq!(discovery_path("popular-today"), None);
	assert_eq!(discovery_path("https://example.invalid"), None);
}

#[aidoku_test]
fn selected_language_routes_cover_search_and_discovery() {
	for (code, site) in [
		("en", "en"),
		("zh", "zh-hant"),
		("th", "th"),
		("id", "id"),
		("es", "es"),
		("fr", "fr"),
		("de", "de"),
	] {
		assert_eq!(locale_for_language_code(code), site);
		assert_eq!(
			discovery_path_for_language("genre-drama", site).as_deref(),
			Some(format!("/{site}/genres/drama?sortOrder=MANA").as_str())
		);
		assert_eq!(
			search_path_for_language("space boy", "canvas", 2, site).unwrap(),
			format!("/{site}/search/canvas?keyword=space%20boy&page=2")
		);
		assert_eq!(
			search_path_for_language("space boy", "all", 2, site).unwrap(),
			format!("/{site}/search?keyword=space%20boy&page=2")
		);
	}
	assert_eq!(locale_for_language_code("xx"), "en");
}

#[aidoku_test]
fn dynamic_filters_expose_every_official_genre_and_sort() {
	let filters = Webtoon.get_dynamic_filters().unwrap();
	assert_eq!(filters.len(), 3);
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
	match &filters[2].kind {
		FilterKind::Select { options, ids, .. } => {
			assert_eq!(options.len(), 3);
			assert_eq!(ids.as_ref().unwrap()[0].as_ref(), "all");
			assert_eq!(ids.as_ref().unwrap()[1].as_ref(), "originals");
			assert_eq!(ids.as_ref().unwrap()[2].as_ref(), "canvas");
		}
		_ => panic!("expected catalog scope selector"),
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

#[aidoku_test]
fn official_search_scopes_build_paginated_routes() {
	assert_eq!(
		search_path("space boy", "all", 1).unwrap(),
		"/en/search?keyword=space%20boy"
	);
	assert_eq!(
		search_path("space boy", "all", 2).unwrap(),
		"/en/search?keyword=space%20boy&page=2"
	);
	assert_eq!(
		search_path("space boy", "originals", 1).unwrap(),
		"/en/search/originals?keyword=space%20boy&page=1"
	);
	assert_eq!(
		search_path("space boy", "canvas", 2).unwrap(),
		"/en/search/canvas?keyword=space%20boy&page=2"
	);
	assert!(search_path("x", "invalid", 1).is_err());
	assert_eq!(
		search_scope(&[FilterValue::Select {
			id: "scope".into(),
			value: "canvas".into(),
		}])
		.unwrap(),
		"canvas"
	);
	assert!(
		search_scope(&[FilterValue::Select {
			id: "scope".into(),
			value: "external".into(),
		}])
		.is_err()
	);
}

#[aidoku_test]
fn combined_search_uses_nonempty_page_as_next_page_signal() {
	let html = Html::parse_with_url(
		"<a href='/en/sf/space-boy/list?title_no=400'><strong class='title'>Space Boy</strong></a>",
		"https://www.webtoons.com",
	)
	.unwrap();
	let populated = parse_search(&html);
	assert!(has_search_next_page_for_scope("all", &populated, &html, 1));

	let empty_html = Html::parse("<div class='empty'>No results</div>").unwrap();
	let empty = parse_search(&empty_html);
	assert!(!has_search_next_page_for_scope(
		"all",
		&empty,
		&empty_html,
		2
	));

	let paginated = Html::parse(
		"<div class='list_pagination'><a class='pagination' href='/en/search/canvas?page=2'>2</a></div>",
	)
	.unwrap();
	assert!(has_search_next_page_for_scope(
		"canvas", &empty, &paginated, 1
	));
}

#[aidoku_test]
fn search_pagination_uses_server_page_links() {
	let page_one = Html::parse(
		"<div class='list_pagination'><a class='pagination' href='/en/search/canvas?keyword=x&amp;page=2'>2</a></div>",
	)
	.unwrap();
	assert!(has_next_search_page(&page_one, 1));
	assert!(!has_next_search_page(&page_one, 2));
	let final_page =
		Html::parse("<div class='list_pagination'><a class='pagination' href='#'>1</a></div>")
			.unwrap();
	assert!(!has_next_search_page(&final_page, 1));
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
	let (chapters, cursor) = parse_episodes(value, "en").unwrap();
	assert_eq!(chapters.len(), 100);
	assert_eq!(cursor, Some(102));
	assert_eq!(chapters[0].title.as_deref(), Some("Ep. 1"));
	assert_eq!(chapters[0].date_uploaded, Some(1425564010));
	assert_eq!(chapters[0].language.as_deref(), Some("en"));
	assert_eq!(chapters[0].thumbnail.as_deref(), Some("https://webtoon-phinf.pstatic.net/20150305_16/14255619312545UkI5_JPEG/142556193121740014.jpg"));
	let (translated, _) = parse_episodes(
		serde_json::from_str(include_str!("../tests/fixtures/episodes.json")).unwrap(),
		"zh",
	)
	.unwrap();
	assert_eq!(translated[0].language.as_deref(), Some("zh"));
	let (unsafe_thumb, _) = parse_episodes(
		serde_json::json!({"result":{"episodeList":[{"viewerLink":"/en/x/ep-1/viewer?title_no=1","thumbnail":"//attacker.example/image.jpg"}]}}),
		"en",
	)
	.unwrap();
	assert!(unsafe_thumb[0].thumbnail.is_none());
	assert!(parse_episodes(serde_json::json!({"error":"denied"}), "en").is_err());
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

#[aidoku_test]
fn official_episode_deep_links_open_the_matching_chapter() {
	use aidoku::{DeepLinkHandler, DeepLinkResult};
	let source = Webtoon;
	let mobile = "https://m.webtoons.com/en/sf/space-boy/ep-1/viewer?title_no=400&episode_no=1";
	assert_eq!(
		source.handle_deep_link(mobile.into()).unwrap(),
		Some(DeepLinkResult::Chapter {
			manga_key: "/en/sf/space-boy/list?title_no=400".into(),
			key: "/en/sf/space-boy/ep-1/viewer?title_no=400&episode_no=1".into(),
		})
	);
	let desktop = "https://www.webtoons.com/en/sf/space-boy/ep-1/viewer?title_no=400&episode_no=1";
	assert_eq!(
		source.handle_deep_link(desktop.into()).unwrap(),
		source.handle_deep_link(mobile.into()).unwrap()
	);
	for invalid in [
		"https://m.webtoons.com/en/sf/space-boy/ep-x/viewer?title_no=400&episode_no=1",
		"https://m.webtoons.com/en/sf/space-boy/ep-1/viewer?title_no=0&episode_no=1",
		"https://m.webtoons.com/en/sf/space-boy/ep-1/viewer?title_no=400&episode_no=nope",
	] {
		assert_eq!(source.handle_deep_link(invalid.into()).unwrap(), None, "{invalid}");
	}
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
