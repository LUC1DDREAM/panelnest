#[aidoku_test]
fn synthetic_details_and_chapter_flags() {
	let doc=Html::parse_with_url(r#"<div class="gallery_top gallery_first"><h1>Sample 2</h1><div class="cover left_cover"><img src="/cover.png"></div><ul class="artists"><li><a>Artist 7</a></li></ul><ul><li><span class="tags_text">Artists:</span><a class="tag">Artist 7</a></li></ul></div>"#,BASE_URL).unwrap();
	let m = update(
		&doc,
		Manga {
			key: "42".into(),
			..Default::default()
		},
		true,
		true,
	)
	.unwrap();
	assert_eq!(m.title, "Sample 2");
	assert_eq!(m.authors.unwrap()[0], "Artist 7");
	assert_eq!(m.chapters.unwrap()[0].key, "42");
	assert!(
		update(
			&doc,
			Manga {
				key: "42".into(),
				..Default::default()
			},
			false,
			false
		)
		.unwrap()
		.chapters
		.is_none()
	);
	assert!(
		update(
			&Html::parse("<html></html>").unwrap(),
			Manga {
				key: "42".into(),
				..Default::default()
			},
			true,
			false
		)
		.is_err()
	);
}
#[aidoku_test]
fn search_urls_escape_query_and_validate_page() {
	let url = search_url(Some("Sample &page=999"), 2).unwrap();
	assert!(url.contains("%26page%3D999"));
	assert!(url.ends_with("page=2"));
	assert!(search_url(None, 0).is_err());
	assert_eq!(
		search_url(None, 2).unwrap(),
		if IS_IM {
			format!("{BASE_URL}/?page=2")
		} else {
			format!("{BASE_URL}/page/2/")
		}
	);
}
#[aidoku_test]
fn synthetic_pages_order_formats_and_validation() {
	let doc=Html::parse_with_url(r#"<input id="load_id" value="81"><input id="load_dir" value="001"><input id="load_server" value="2"><input id="load_pages" value="2"><script>var images=$.parseJSON('{"2":"w,10,20","1":"p,30,40"}');</script>"#,BASE_URL).unwrap();
	let pages = parse_pages(&doc).unwrap();
	assert_eq!(pages.len(), 2);
	if let PageContent::Url(url, _) = &pages[0].content {
		assert!(url.ends_with("/001/81/1.png"));
	} else {
		panic!("not a URL")
	}
	if let PageContent::Url(url, _) = &pages[1].content {
		assert!(url.ends_with("/001/81/2.webp"));
	} else {
		panic!("not a URL")
	}
	assert!(parse_pages(&Html::parse("<html></html>").unwrap()).is_err());
}

#[aidoku_test]
fn discovery_home_has_working_latest_listing() {
	let doc = Html::parse_with_url(r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"></a></div><div class="caption">Sample</div></div>"#, BASE_URL).unwrap();
	let home = parse_home(&doc).unwrap();
	assert_eq!(home.components[0].title.as_deref(), Some("Latest"));
	if let aidoku::HomeComponentValue::Scroller { entries, listing } = &home.components[0].value {
		assert_eq!(entries.len(), 1);
		let listing = listing.as_ref().unwrap();
		assert_eq!(listing.id, "latest");
		assert_eq!(
			listing_url(&listing.id, 2).unwrap(),
			search_url(None, 2).unwrap()
		);
	} else {
		panic!("expected scroller");
	}
	assert!(listing_url("popular-today", 1).is_err());
	assert!(listing_url("latest", 0).is_err());
	assert!(
		parse_home(&Html::parse("<div class='container'>Access denied</div>").unwrap()).is_err()
	);
}

#[aidoku_test]
fn top_rated_is_scoped_finite_and_not_today() {
	let doc = Html::parse_with_url(r#"<button id="top_rated_btn" class="sidebar_btn_active">Top Rated</button><div id="middle_sidebar"><div class="item"><a href="/gallery/77/"><img alt="Sample Rated" src="/cover.png"></a></div><div class="item"><a href="https://invalid.example/gallery/78/"><img alt="Foreign"></a></div></div><div class="item"><a href="/gallery/99/"><img alt="Unrelated"></a></div>"#, BASE_URL).unwrap();
	let result = parse_top_rated(&doc).unwrap();
	assert_eq!(result.entries.len(), 1);
	assert_eq!(result.entries[0].key, "77");
	assert_eq!(result.entries[0].title, "Sample Rated");
	assert!(!result.has_next_page);
	assert_eq!(listing_url("top-rated", 1).unwrap(), format!("{BASE_URL}/"));
	assert!(parse_top_rated(&Html::parse("<div id='middle_sidebar'></div>").unwrap()).is_err());
}

#[aidoku_test]
fn dynamic_sidebar_rankings_are_registered_and_parse_scoped_entries() {
	let listings = SIDEBAR_LISTINGS
		.iter()
		.map(|(id, name, _)| (*id, *name))
		.collect::<Vec<_>>();
	assert_eq!(listings[0], ("most-faved", "Most Faved"));
	assert_eq!(listings[1], ("most-fapped", "Most Fapped"));
	assert_eq!(listings[2], ("most-downloaded", "Most Downloaded"));
	assert_eq!(sidebar_type("most-downloaded"), Some("top_downloaded"));
	assert_eq!(sidebar_type("unknown"), None);
	let doc = Html::parse_with_url(
		r#"<div class="item"><a href="/gallery/77/"><img alt="Sample Ranked" src="/cover.png"></a></div><div class="item"><a href="https://invalid.example/gallery/78/"><img alt="Foreign"></a></div>"#,
		BASE_URL,
	)
	.unwrap();
	let result = parse_sidebar_items(&doc).unwrap();
	assert_eq!(result.entries.len(), 1);
	assert_eq!(result.entries[0].key, "77");
	assert_eq!(result.entries[0].title, "Sample Ranked");
	assert!(!result.has_next_page);
	assert!(parse_sidebar_items(&Html::parse("<html></html>").unwrap()).is_err());
}

#[aidoku_test]
fn sidebar_rankings_become_browsable_home_scrollers() {
	let doc = Html::parse_with_url(
		r#"<div class="item"><a href="/gallery/77/"><img alt="Sample Ranked" src="/cover.png"></a></div>"#,
		BASE_URL,
	)
	.unwrap();
	for (id, title, _) in SIDEBAR_LISTINGS {
		let result = parse_sidebar_items(&doc).unwrap();
		let component = sidebar_home_component(id, title, result);
		assert_eq!(component.title.as_deref(), Some(title));
		if let aidoku::HomeComponentValue::Scroller { entries, listing } = component.value {
			assert_eq!(entries.len(), 1);
			assert_eq!(listing.unwrap().id, id);
		} else {
			panic!("expected browsable ranking scroller");
		}
	}
	assert!(parse_sidebar_items(&Html::parse("<html></html>").unwrap()).is_err());
}

#[aidoku_test]
fn sidebar_home_reuses_only_a_nonempty_homepage_csrf_token() {
	let doc = Html::parse(r#"<meta name="csrf-token" content="valid-token">"#).unwrap();
	assert_eq!(sidebar_csrf_token(&doc).as_deref(), Some("valid-token"));
	assert_eq!(
		sidebar_csrf_token(&Html::parse(r#"<meta name="csrf-token" content=" ">"#).unwrap()),
		None
	);
	assert_eq!(
		sidebar_csrf_token(&Html::parse("<html></html>").unwrap()),
		None
	);
}

#[aidoku_test]
fn home_sidebar_components_fail_independently_from_latest() {
	let latest = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"></a></div><div class="caption">Latest Sample</div></div>"#,
		BASE_URL,
	)
	.unwrap();
	let mut home = parse_home(&latest).unwrap();
	let failed_sidebar = Html::parse("<html></html>").unwrap();
	if let Ok(result) = parse_sidebar_items(&failed_sidebar) {
		home.components.push(sidebar_home_component(
			SIDEBAR_LISTINGS[0].0,
			SIDEBAR_LISTINGS[0].1,
			result,
		));
	}
	assert_eq!(home.components[0].title.as_deref(), Some("Latest"));
	assert_eq!(home.components.len(), 1);
}

#[aidoku_test]
fn popular_tag_directory_builds_safe_paginated_gallery_routes() {
	assert_eq!(POPULAR_TAGS_PATH, "/tags/popular/");
	assert_eq!(
		popular_tag_url("popular-tag-big-breasts", 1, false).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/")
	);
	assert_eq!(
		popular_tag_url("popular-tag-big-breasts", 2, false).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/pag/2/")
	);
	assert_eq!(
		popular_tag_url("popular-tag-big-breasts", 1, true).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/popular/")
	);
	assert_eq!(
		popular_tag_url("popular-tag-big-breasts", 2, true).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/popular/pag/2/")
	);
	assert!(popular_tag_url("popular-tag-../evil", 1, false).is_err());
	assert!(popular_tag_url("popular-tag-big-breasts", 0, false).is_err());
	assert!(popular_tag_url("latest", 1, false).is_err());
}

#[aidoku_test]
fn popular_tag_directory_exposes_valid_deduplicated_tags() {
	let doc = Html::parse_with_url(
		r#"<div class="tags_overview">
		<div class="tag_item"><a class="tag_btn" href="/tag/big-breasts/"><h3 class="list_tag">Big Breasts</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="/tag/sole-female/"><h3 class="list_tag">Sole Female</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="https://evil.example/tag/attack/"><h3 class="list_tag">Foreign</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="/tag/big-breasts/"><h3 class="list_tag">Duplicate</h3></a></div>
	</div>"#,
		BASE_URL,
	)
	.unwrap();
	let listings = parse_popular_tag_listings(&doc).unwrap();
	assert_eq!(listings.len(), 2);
	assert_eq!(listings[0].id, "popular-tag-big-breasts");
	assert_eq!(listings[0].name, "Tag: Big Breasts");
	assert_eq!(listings[1].id, "popular-tag-sole-female");
	assert!(parse_popular_tag_listings(&Html::parse("<html></html>").unwrap()).is_err());
}

#[aidoku_test]
fn popular_taxonomy_filters_parse_safe_live_categories() {
	let doc = Html::parse_with_url(
		r#"<div class="tags_overview">
		<div class="tag_item"><a class="tag_btn" href="/artist/ankoman/"><h3 class="list_tag">ankoman</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="/artist/.exe/"><h3 class="list_tag">.exe</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="/character/ignore/"><h3 class="list_tag">Wrong category</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="https://evil.example/artist/attack/"><h3 class="list_tag">Foreign</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="/artist/bad_slug/"><h3 class="list_tag">Invalid slug</h3></a></div>
	</div>"#,
		BASE_URL,
	)
	.unwrap();
	let values = parse_popular_taxonomy(&doc, "artist").unwrap();
	assert_eq!(
		values,
		vec![
			("ankoman".into(), "ankoman".into()),
			(".exe".into(), ".exe".into())
		]
	);
	assert!(parse_popular_taxonomy(&doc, "unknown").is_err());
	assert!(parse_popular_taxonomy(&Html::parse("<html></html>").unwrap(), "artist").is_err());
}

#[aidoku_test]
fn language_directory_values_and_routes_are_supported() {
	let doc = Html::parse_with_url(
		r#"<div class="tags_overview">
		<div class="tag_item"><a class="tag_btn" href="/language/english/"><h3 class="list_tag">english</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="/language/text-cleaned/"><h3 class="list_tag">text cleaned</h3></a></div>
		<div class="tag_item"><a class="tag_btn" href="/language/bad_slug/"><h3 class="list_tag">Invalid</h3></a></div>
	</div>"#,
		BASE_URL,
	)
	.unwrap();
	assert_eq!(
		parse_popular_taxonomy(&doc, "language").unwrap(),
		vec![
			("english".into(), "english".into()),
			("text cleaned".into(), "text-cleaned".into())
		]
	);
	assert_eq!(
		taxonomy_url("language", "english", 1, false).unwrap(),
		format!("{BASE_URL}/language/english/")
	);
	assert_eq!(
		taxonomy_url("language", "english", 2, true).unwrap(),
		format!("{BASE_URL}/language/english/popular/pag/2/")
	);
}

#[aidoku_test]
fn freeform_taxonomy_filters_reach_categories_outside_the_popular_top_fifty() {
	assert_eq!(
		taxonomy_text_slug("Naruto Uzumaki").unwrap(),
		"naruto-uzumaki"
	);
	assert_eq!(taxonomy_text_slug(".EXE").unwrap(), ".exe");
	assert!(taxonomy_text_slug("../escape").is_err());
	assert!(taxonomy_text_slug("日本語").is_err());

	let filters = search_filters();
	assert_eq!(filters.len(), 6);
	for (index, id) in [
		"tag-name",
		"artist-name",
		"character-name",
		"parody-name",
		"group-name",
	]
	.iter()
	.enumerate()
	{
		assert_eq!(filters[index + 1].id.as_ref(), *id);
	}

	let tag = vec![
		FilterValue::Text {
			id: "tag-name".into(),
			value: "Big Breasts".into(),
		},
		FilterValue::Sort {
			id: "sort".into(),
			index: 1,
			ascending: false,
		},
	];
	assert_eq!(
		search_url_with_filters(None, 2, &tag).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/popular/pag/2/")
	);

	let artist = vec![FilterValue::Text {
		id: "artist-name".into(),
		value: "Daum".into(),
	}];
	assert_eq!(
		search_url_with_filters(None, 1, &artist).unwrap(),
		format!("{BASE_URL}/artist/daum/")
	);
	assert!(search_url_with_filters(Some("gallery"), 1, &artist).is_err());

	let conflicting = vec![
		FilterValue::Select {
			id: "artist".into(),
			value: "daum".into(),
		},
		FilterValue::Text {
			id: "group-name".into(),
			value: "circle".into(),
		},
	];
	assert!(search_url_with_filters(None, 1, &conflicting).is_err());
}

#[aidoku_test]
fn popular_taxonomy_filter_exposes_names_and_validated_slugs() {
	let filter = taxonomy_filter(
		"artist",
		"Artist",
		vec![
			("ankoman".into(), "ankoman".into()),
			(".exe".into(), ".exe".into()),
		],
	);
	assert_eq!(filter.id, "artist");
	match filter.kind {
		aidoku::FilterKind::Select {
			options,
			ids,
			default,
			..
		} => {
			assert_eq!(options[0].as_ref(), "Any");
			assert_eq!(options[1].as_ref(), "ankoman");
			assert_eq!(options[2].as_ref(), ".exe");
			assert_eq!(ids.as_ref().unwrap()[0].as_ref(), "");
			assert_eq!(ids.as_ref().unwrap()[2].as_ref(), ".exe");
			assert_eq!(default.as_deref(), Some(""));
		}
		_ => panic!("Popular categories should be a single-select filter"),
	}
}

#[aidoku_test]
fn popular_tag_filter_exposes_directory_entries() {
	let filter = taxonomy_filter(
		"tag",
		"Tag",
		vec![("Big Breasts".into(), "big-breasts".into())],
	);
	assert_eq!(filter.id, "tag");
	match filter.kind {
		aidoku::FilterKind::Select { options, ids, .. } => {
			assert_eq!(options[0].as_ref(), "Any");
			assert_eq!(options[1].as_ref(), "Big Breasts");
			assert_eq!(ids.as_ref().unwrap()[1].as_ref(), "big-breasts");
		}
		_ => panic!("Popular tags should be a single-select filter"),
	}
}

#[aidoku_test]
fn popular_taxonomy_filters_map_to_official_latest_and_popular_routes() {
	assert_eq!(
		popular_tag_url("popular-tag-big-breasts", 2, false).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/pag/2/")
	);
	assert_eq!(
		taxonomy_url("artist", "ankoman", 1, false).unwrap(),
		format!("{BASE_URL}/artist/ankoman/")
	);
	assert_eq!(
		taxonomy_url("artist", "ankoman", 2, false).unwrap(),
		format!("{BASE_URL}/artist/ankoman/pag/2/")
	);
	assert_eq!(
		taxonomy_url("character", "2b", 1, true).unwrap(),
		format!("{BASE_URL}/character/2b/popular/")
	);
	assert_eq!(
		taxonomy_url("parody", "hack", 3, true).unwrap(),
		format!("{BASE_URL}/parody/hack/popular/pag/3/")
	);
	assert!(taxonomy_url("group", "../evil", 1, false).is_err());
	assert!(taxonomy_url("unknown", "valid", 1, false).is_err());
	assert!(taxonomy_url("artist", "ankoman", 0, false).is_err());

	let filters = vec![
		FilterValue::Select {
			id: "character".into(),
			value: "2b".into(),
		},
		FilterValue::Sort {
			id: "sort".into(),
			index: 1,
			ascending: false,
		},
	];
	assert_eq!(
		search_url_with_filters(None, 2, &filters).unwrap(),
		format!("{BASE_URL}/character/2b/popular/pag/2/")
	);
	let tag_filter = vec![FilterValue::Select {
		id: "tag".into(),
		value: "big-breasts".into(),
	}];
	assert_eq!(
		search_url_with_filters(None, 3, &tag_filter).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/pag/3/")
	);
	let popular_tag_filter = vec![
		tag_filter[0].clone(),
		FilterValue::Sort {
			id: "sort".into(),
			index: 1,
			ascending: false,
		},
	];
	assert_eq!(
		search_url_with_filters(None, 1, &popular_tag_filter).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/popular/")
	);
	assert_eq!(
		search_url_with_filters(None, 2, &popular_tag_filter).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/popular/pag/2/")
	);
	assert!(search_url_with_filters(Some("query"), 1, &filters).is_err());
	let conflicting = vec![
		FilterValue::Select {
			id: "artist".into(),
			value: "ankoman".into(),
		},
		FilterValue::Select {
			id: "group".into(),
			value: "circle".into(),
		},
	];
	assert!(search_url_with_filters(None, 1, &conflicting).is_err());
}

#[aidoku_test]
fn listing_dispatch_rejects_unknown_and_invalid_pages_without_network() {
	use aidoku::ListingProvider;
	let source = GallerySource;
	assert_eq!(sidebar_type("most-faved"), Some("top_faved"));
	assert!(
		source
			.get_manga_list(
				aidoku::Listing {
					id: "most-faved".into(),
					..Default::default()
				},
				2
			)
			.unwrap()
			.entries
			.is_empty()
	);
	assert!(
		source
			.get_manga_list(
				aidoku::Listing {
					id: "popular-today".into(),
					..Default::default()
				},
				1
			)
			.is_err()
	);
	assert!(
		source
			.get_manga_list(
				aidoku::Listing {
					id: "latest".into(),
					..Default::default()
				},
				0
			)
			.is_err()
	);
	let result = source
		.get_manga_list(
			aidoku::Listing {
				id: "top-rated".into(),
				..Default::default()
			},
			2,
		)
		.unwrap();
	assert!(result.entries.is_empty());
	assert!(!result.has_next_page);
}
#[aidoku_test]
fn manifest_preserves_identity_and_matches_discovery() {
	let manifest: serde_json::Value =
		serde_json::from_str(include_str!("../res/source.json")).unwrap();
	assert_eq!(manifest["info"]["contentRating"], 2);
	assert_eq!(manifest["info"]["languages"][0], "multi");
	assert_eq!(manifest["info"]["version"], 17);
	assert!(
		manifest["info"]["name"]
			.as_str()
			.unwrap()
			.ends_with(" [PN]")
	);
	for listing in manifest["listings"].as_array().unwrap() {
		assert!(listing_url(listing["id"].as_str().unwrap(), 1).is_ok());
	}
}

use super::*;
use aidoku::imports::html::Html;
use aidoku_test::aidoku_test;
#[aidoku_test]
fn synthetic_search() {
	let doc=Html::parse_with_url(r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"><img data-src="/cover.png"></a></div><div class="caption">Sample</div></div><div class="thumb"><div class="inner_thumb"><a href="https://invalid.example/gallery/43/"></a></div><div class="caption">Foreign</div></div><ul class="pagination"><li class="active">1</li><li><a href="?page=2">2</a></li></ul>"#,BASE_URL).unwrap();
	let result = parse_search(&doc);
	assert_eq!(result.entries.len(), 1);
	assert_eq!(result.entries[0].key, "42");
	assert_eq!(result.entries[0].title, "Sample");
	assert_eq!(
		result.entries[0].cover,
		Some(format!("{BASE_URL}/cover.png"))
	);
	assert!(result.has_next_page);
}

#[aidoku_test]
fn deep_links_resolve_only_numeric_gallery_ids_on_the_source_domain() {
	use aidoku::DeepLinkHandler;
	let source = GallerySource;
	assert_eq!(
		deep_link_key("https://hentaifox.com/gallery/123/"),
		Some("123".into())
	);
	assert_eq!(
		deep_link_key("https://hentaifox.com/gallery/123?from=share#reader"),
		Some("123".into())
	);
	assert!(deep_link_key("https://hentaifox.com.evil/gallery/123/").is_none());
	assert!(deep_link_key("https://hentaifox.com.evil.example/gallery/123/").is_none());
	assert!(deep_link_key("http://hentaifox.com/gallery/123/").is_none());
	assert!(deep_link_key("https://example.org/gallery/123/").is_none());
	assert!(deep_link_key("https://hentaifox.com/gallery/nope/").is_none());
	assert_eq!(
		source
			.handle_deep_link("https://hentaifox.com/gallery/123/".into())
			.unwrap(),
		Some(DeepLinkResult::Manga { key: "123".into() })
	);
}

#[aidoku_test]
fn search_sort_filter_preserves_latest_and_maps_popular_to_site_parameter() {
	let popular = vec![FilterValue::Sort {
		id: "sort".into(),
		index: 1,
		ascending: false,
	}];
	let latest = vec![FilterValue::Sort {
		id: "sort".into(),
		index: 0,
		ascending: false,
	}];
	let popular_url = search_url_with_filters(Some("two words"), 3, &popular).unwrap();
	assert!(popular_url.ends_with("q=two%20words&page=3&sort=popular"));
	assert_eq!(
		search_url_with_filters(Some("two words"), 3, &latest).unwrap(),
		search_url(Some("two words"), 3).unwrap()
	);
	assert_eq!(search_filters().len(), 6);
	assert_eq!(
		search_url_with_filters(None, 1, &popular).unwrap(),
		format!("{BASE_URL}/search/?q=&page=1&sort=popular")
	);
	assert_eq!(
		search_url_with_filters(None, 1, &latest).unwrap(),
		search_url(None, 1).unwrap()
	);
}
