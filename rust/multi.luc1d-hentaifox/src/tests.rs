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
fn listing_dispatch_rejects_unknown_and_invalid_pages_without_network() {
	use aidoku::ListingProvider;
	let source = GallerySource;
	assert_eq!(sidebar_type("most-faved"), Some("top_faved"));
	assert!(source
		.get_manga_list(
			aidoku::Listing { id: "most-faved".into(), ..Default::default() },
			2
		)
		.unwrap()
		.entries
		.is_empty());
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
	assert_eq!(manifest["info"]["version"], 5);
	assert!(manifest["info"]["name"].as_str().unwrap().ends_with(" [PN]"));
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
