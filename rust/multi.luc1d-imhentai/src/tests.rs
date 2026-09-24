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
fn discovery_filters_map_to_official_intermediate_search_parameters() {
	let filters = vec![
		FilterValue::Sort {
			id: "sort".into(),
			index: 3,
			ascending: false,
		},
		FilterValue::MultiSelect {
			id: "categories".into(),
			included: vec!["m".into(), "g".into()],
			excluded: vec![],
		},
		FilterValue::MultiSelect {
			id: "languages".into(),
			included: vec!["en".into(), "jp".into()],
			excluded: vec![],
		},
	];
	let url = search_url_with_filters(Some("two words"), 4, &filters).unwrap();
	assert!(url.contains("pp=0&lt=0&dl=0&tr=1"));
	assert!(url.contains("m=1&d=0&w=0&i=0&a=0&g=1"));
	assert!(url.contains("en=1&jp=1&es=0&fr=0&kr=0&de=0&ru=0"));
	assert!(url.ends_with("key=two%20words&page=4"));
	let browse = search_url_with_filters(None, 1, &filters).unwrap();
	assert!(browse.starts_with(&format!("{BASE_URL}/search/?")));
	assert_eq!(discovery_filters().len(), 3);
	let popular_only = search_url_with_filters(
		None,
		1,
		&[FilterValue::Sort {
			id: "sort".into(),
			index: 0,
			ascending: false,
		}],
	)
	.unwrap();
	assert!(popular_only.starts_with(&format!("{BASE_URL}/search/?pp=1&lt=0&dl=0&tr=0")));
	let downloads_only = search_url_with_filters(
		None,
		1,
		&[FilterValue::Sort {
			id: "sort".into(),
			index: 2,
			ascending: false,
		}],
	)
	.unwrap();
	assert!(downloads_only.starts_with(&format!("{BASE_URL}/search/?pp=0&lt=0&dl=1&tr=0")));
	let latest_only = search_url_with_filters(
		None,
		1,
		&[FilterValue::Sort {
			id: "sort".into(),
			index: 1,
			ascending: false,
		}],
	)
	.unwrap();
	assert_eq!(latest_only, format!("{BASE_URL}/?page=1"));
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
	assert_eq!(home.components.len(), 4);
	for (index, (id, title)) in [
		("latest", "Latest"),
		("popular", "Popular"),
		("top-rated", "Top Rated"),
		("downloaded", "Downloaded"),
	]
	.iter()
	.enumerate()
	{
		assert_eq!(home.components[index].title.as_deref(), Some(*title));
		if let aidoku::HomeComponentValue::Scroller { entries, listing } =
			&home.components[index].value
		{
			assert_eq!(entries.len(), 1);
			let listing = listing.as_ref().unwrap();
			assert_eq!(listing.id, *id);
		} else {
			panic!("expected scroller");
		}
	}
	assert_eq!(
		listing_url("latest", 2).unwrap(),
		search_url(None, 2).unwrap()
	);
	assert_eq!(
		listing_url("popular", 1).unwrap(),
		format!("{BASE_URL}/popular/")
	);
	assert_eq!(
		listing_url("popular", 2).unwrap(),
		format!("{BASE_URL}/popular/2/")
	);
	assert_eq!(
		listing_url("top-rated", 2).unwrap(),
		format!("{BASE_URL}/top-rated/2/")
	);
	assert_eq!(
		listing_url("downloaded", 3).unwrap(),
		format!("{BASE_URL}/downloaded/3/")
	);
	assert!(listing_url("popular-today", 1).is_err());
	assert!(listing_url("latest", 0).is_err());
	assert!(
		parse_home(&Html::parse("<div class='container'>Access denied</div>").unwrap()).is_err()
	);
}

#[aidoku_test]
fn listing_dispatch_rejects_unknown_and_invalid_pages_without_network() {
	use aidoku::ListingProvider;
	let source = GallerySource;
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
}
#[aidoku_test]
fn manifest_preserves_identity_and_matches_discovery() {
	let manifest: serde_json::Value =
		serde_json::from_str(include_str!("../res/source.json")).unwrap();
	assert_eq!(manifest["info"]["contentRating"], 2);
	assert_eq!(manifest["info"]["languages"][0], "multi");
	assert_eq!(manifest["info"]["version"], 8);
	assert!(
		manifest["info"]["name"]
			.as_str()
			.unwrap()
			.ends_with(" [PN]")
	);
	for listing in manifest["listings"].as_array().unwrap() {
		assert!(listing_url(listing["id"].as_str().unwrap(), 1).is_ok());
	}
	assert_eq!(manifest["info"]["version"], 9);
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
		deep_link_key("https://imhentai.xxx/gallery/123/"),
		Some("123".into())
	);
	assert_eq!(
		deep_link_key("https://imhentai.xxx/gallery/123?from=share#reader"),
		Some("123".into())
	);
	assert!(deep_link_key("https://imhentai.xxx.evil/gallery/123/").is_none());
	assert!(deep_link_key("https://example.org/gallery/123/").is_none());
	assert!(deep_link_key("https://imhentai.xxx/gallery/nope/").is_none());
	assert_eq!(
		source
			.handle_deep_link("https://imhentai.xxx/gallery/123/".into())
			.unwrap(),
		Some(DeepLinkResult::Manga { key: "123".into() })
	);
}
