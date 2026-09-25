#[aidoku_test]
fn synthetic_details_and_chapter_flags() {
	let doc=Html::parse_with_url(r#"<div class="gallery_top gallery_first"><h1>Sample 2</h1><div class="cover left_cover"><img src="/cover.png"></div><ul class="artists"><li><a>Artist 7</a></li></ul><ul><li><span class="tags_text">Artists:</span><a class="tag">Artist 7</a></li><li><span class="tags_text">Languages:</span><a class="tag">french</a><a class="tag">translated</a></li></ul><li style="display:none" class="posted">Posted: 2 days ago</li></div>"#,BASE_URL).unwrap();
	let before_update = aidoku::imports::std::current_date();
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
	assert_eq!(m.artists.as_deref(), Some(&[String::from("Artist 7")][..]));
	assert_eq!(m.authors.as_deref(), Some(&[][..]));
	let chapters = m.chapters.unwrap();
	assert_eq!(chapters[0].key, "42");
	assert_eq!(chapters[0].language.as_deref(), Some("fr"));
	assert_eq!(chapters[0].thumbnail.as_deref(), Some("https://imhentai.xxx/cover.png"));
	let uploaded_at = chapters[0].date_uploaded.unwrap();
	assert!(uploaded_at >= before_update - 172_800);
	assert!(uploaded_at <= aidoku::imports::std::current_date() - 172_800);
	assert_eq!(m.update_strategy, UpdateStrategy::Never);
	let with_reader = Html::parse_with_url(
		r#"<div class="gallery_top"><h1>Reader sample</h1><a href="/view/42/3/">Read</a></div>"#,
		BASE_URL,
	)
	.unwrap();
	let chapter = update(
		&with_reader,
		Manga { key: "42".into(), ..Default::default() },
		false,
		true,
	)
	.unwrap()
	.chapters
	.unwrap()
	.remove(0);
	assert_eq!(chapter.key, "42");
	assert_eq!(chapter.url.as_deref(), Some("https://imhentai.xxx/view/42/1/"));
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
fn chapter_language_uses_only_one_recognized_gallery_language() {
	let french = Html::parse_with_url(
		r#"<ul><li><span class="tags_text">Languages:</span><a class="tag">french</a><a class="tag">translated</a></li></ul>"#,
		BASE_URL,
	)
	.unwrap();
	assert_eq!(gallery_language(&french).as_deref(), Some("fr"));

	let multiple = Html::parse_with_url(
		r#"<ul><li><span class="tags_text">Languages:</span><a class="tag">english</a><a class="tag">french</a></li></ul>"#,
		BASE_URL,
	)
	.unwrap();
	assert_eq!(gallery_language(&multiple), None);

	let unknown = Html::parse_with_url(
		r#"<ul><li><span class="tags_text">Languages:</span><a class="tag">translated</a></li></ul>"#,
		BASE_URL,
	)
	.unwrap();
	assert_eq!(gallery_language(&unknown), None);
}

#[aidoku_test]
fn live_gallery_artist_metadata_maps_to_aidoku_artists() {
	let doc = Html::parse_with_url(
		include_str!("../fixtures/live-gallery-artists.html"),
		"https://imhentai.xxx/gallery/1743990/",
	)
	.unwrap();
	assert_eq!(gallery_artists(&doc), vec!["anabuki bouhatei"]);
}

#[aidoku_test]
fn posted_age_is_converted_to_a_bounded_approximate_timestamp() {
	assert_eq!(posted_age_seconds("Posted: 8 hours ago"), Some(28_800));
	assert_eq!(posted_age_seconds("Posted: 2 days ago"), Some(172_800));
	assert_eq!(posted_age_seconds("Posted: yesterday"), Some(86_400));
	assert_eq!(posted_age_seconds("Posted: just now"), Some(0));
	assert_eq!(posted_age_seconds("Pages: 50"), None);
	assert_eq!(posted_age_seconds("Posted: someday ago"), None);
	assert_eq!(posted_age_seconds("Posted: -2 days ago"), None);
}

#[aidoku_test]
fn chapter_reader_starts_at_page_one_instead_of_a_thumbnail_page() {
	let doc = Html::parse_with_url(
		r#"<a href="/view/42/10/"><img src="/preview.jpg"></a><a href="https://imhentai.xxx/view/42/7/">Read</a>"#,
		BASE_URL,
	)
	.unwrap();
	assert_eq!(reader_url_for_gallery(&doc, "42").unwrap(), "https://imhentai.xxx/view/42/1/");
	assert!(reader_url_for_gallery(&doc, "not-a-gallery").is_err());
}

#[aidoku_test]
fn browser_requests_keep_the_requested_url_and_accept_a_gallery_referer() {
	let referer = gallery_referer("42").unwrap();
	let request = site_request("https://imhentai.xxx/gallery/42/".into(), &referer).unwrap();
	assert_eq!(request.url().map(|url| url.as_str()), Some("https://imhentai.xxx/gallery/42/"));
	assert!(gallery_referer("not-a-gallery").is_err());
}

#[aidoku_test]
fn stored_reader_url_cannot_start_a_chapter_on_a_thumbnail_page() {
	let chapter = Chapter {
		key: "42".into(),
		url: Some("https://imhentai.xxx/view/42/7/".into()),
		..Default::default()
	};
	assert_eq!(reader_url_for_chapter("42", &chapter).unwrap(), "https://imhentai.xxx/view/42/1/");
	assert_eq!(reader_url_for_chapter("42", &Chapter { key: "42".into(), ..Default::default() }).unwrap(), "https://imhentai.xxx/view/42/1/");
	assert!(reader_url_for_chapter("43", &chapter).is_err());
}

#[aidoku_test]
fn reader_navigation_uses_validated_gallery_referer() {
	assert_eq!(gallery_referer("1744017").unwrap(), "https://imhentai.xxx/gallery/1744017/");
	for invalid in ["", "1/2", "-1", "1?x=1"] {
		assert!(gallery_referer(invalid).is_err(), "{invalid}");
	}
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
	let filters = discovery_filters();
	assert_eq!(filters.len(), 8);
	for (index, id) in ["tags", "artists", "groups", "parodies", "characters"].iter().enumerate() {
		assert_eq!(filters[index + 3].id.as_ref(), *id);
	}
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
	let advanced = search_url_with_filters(
		None,
		2,
		&[
			FilterValue::Sort {
				id: "sort".into(),
				index: 2,
				ascending: false,
			},
			FilterValue::MultiSelect {
				id: "categories".into(),
				included: vec!["w".into()],
				excluded: vec![],
			},
			FilterValue::MultiSelect {
				id: "languages".into(),
				included: vec!["en".into()],
				excluded: vec![],
			},
			FilterValue::Text {
				id: "tags".into(),
				value: "maid,school life,-mind control".into(),
			},
			FilterValue::Text {
				id: "artists".into(),
				value: "artist name".into(),
			},
			FilterValue::Text {
				id: "groups".into(),
				value: "group".into(),
			},
			FilterValue::Text {
				id: "parodies".into(),
				value: "series".into(),
			},
			FilterValue::Text {
				id: "characters".into(),
				value: "hero".into(),
			},
		],
	)
	.unwrap();
	assert!(advanced.starts_with(&format!("{BASE_URL}/advsearch/?")));
	assert!(advanced.contains("pp=0&lt=0&dl=1&tr=0"));
	assert!(advanced.contains("m=0&d=0&w=1&i=0&a=0&g=0"));
	assert!(advanced.contains("en=1&jp=0&es=0&fr=0&kr=0&de=0&ru=0"));
	assert!(advanced.contains(
		"key=%2Btag%3A%22maid%22+%2Btag%3A%22school%2Blife%22+-tag%3A%22mind%2Bcontrol%22"
	));
	assert!(advanced.contains("+%2Bartist%3A%22artist%2Bname%22"));
	assert!(advanced.contains("+%2Bgroup%3A%22group%22"));
	assert!(advanced.contains("+%2Bparody%3A%22series%22"));
	assert!(advanced.contains("+%2Bcharacter%3A%22hero%22"));
	assert!(search_url_with_filters(
		Some("title"),
		1,
		&[FilterValue::Text {
			id: "tags".into(),
			value: "maid".into(),
		}]
	)
	.is_err());
}

#[aidoku_test]
fn dynamic_listings_expose_all_supported_categories_and_languages() {
	use aidoku::DynamicListings;
	let listings = GallerySource.get_dynamic_listings().unwrap();
	assert_eq!(listings.len(), 13);
	assert_eq!(listings[0].id, "category-m");
	assert_eq!(listings[0].name, "Category: Manga");
	assert_eq!(listings[5].id, "category-g");
	assert_eq!(listings[6].id, "language-en");
	assert_eq!(listings[6].name, "Language: English");
	assert_eq!(listings[12].id, "language-ru");
}

#[aidoku_test]
fn dynamic_category_and_language_listings_use_filtered_paginated_routes() {
	let category = dynamic_listing_url("category-w", 2).unwrap();
	assert!(category.starts_with(&format!("{BASE_URL}/search/?pp=0&lt=1&dl=0&tr=0")));
	assert!(category.contains("m=0&d=0&w=1&i=0&a=0&g=0"));
	assert!(category.contains("en=1&jp=1&es=1&fr=1&kr=1&de=1&ru=1"));
	assert!(category.ends_with("key=&page=2"));
	let language = dynamic_listing_url("language-jp", 3).unwrap();
	assert!(language.contains("m=1&d=1&w=1&i=1&a=1&g=1"));
	assert!(language.contains("en=0&jp=1&es=0&fr=0&kr=0&de=0&ru=0"));
	assert!(language.ends_with("key=&page=3"));
	for id in ["category-unknown", "language-unknown", "category-m/../latest", "latest"] {
		assert!(dynamic_listing_url(id, 1).is_err(), "{id}");
	}
	assert!(dynamic_listing_url("category-m", 0).is_err());
}
#[aidoku_test]
fn synthetic_pages_order_formats_and_validation() {
	let doc=Html::parse_with_url(r#"<input id="load_id" value="81"><input id="load_dir" value="001"><input id="load_server" value="2"><input id="load_pages" value="2"><script>var images=$.parseJSON('{"2":"w,10,20","1":"p,30,40"}');</script>"#,BASE_URL).unwrap();
	let reader_url = "https://imhentai.xxx/view/1743990/1/";
	let pages = parse_pages_with_referer(&doc, reader_url).unwrap();
	assert_eq!(pages.len(), 2);
	if let PageContent::Url(url, _) = &pages[0].content {
		assert!(url.ends_with("/001/81/1.png"));
	} else {
		panic!("not a URL")
	}
	if let PageContent::Url(_, Some(context)) = &pages[0].content {
		assert_eq!(context.get("url").map(String::as_str), Some(reader_url));
	} else {
		panic!("reader page must carry its referer context");
	}
	if let PageContent::Url(url, _) = &pages[1].content {
		assert!(url.ends_with("/001/81/2.webp"));
	} else {
		panic!("not a URL")
	}
	assert!(pages.iter().all(|page| page.has_description));
	assert!(parse_pages(&Html::parse("<html></html>").unwrap()).is_err());
}

#[aidoku_test]
fn reader_manifest_falls_back_to_full_document_markup() {
	let doc = Html::parse_with_url(
		r#"<html><body><!-- g_th = $.parseJSON('{"1":"j,1280,960"}'); --></body></html>"#,
		BASE_URL,
	)
	.unwrap();
	assert!(doc.select_first("script").is_none());
	assert_eq!(reader_manifest(&doc).unwrap()["1"], "j,1280,960");
}

#[aidoku_test]
fn current_imhentai_reader_uses_view_image_path_and_manifest() {
	let doc = Html::parse_with_url(
		r#"<input id="pages" value=""><input id="image_dir" value=""><input id="gallery_id" value=""><img id="gimg" src="https://m11.imhentai.xxx/033/readerkey/1.webp"><script>var g_th = $.parseJSON('{"1":"w,792,1224","2":"w,792,1224","3":"w,792,1224"}');</script>"#,
		BASE_URL,
	)
	.unwrap();
	let pages = parse_pages(&doc).unwrap();
	assert_eq!(pages.len(), 3);
	for (index, page) in pages.iter().enumerate() {
		if let PageContent::Url(url, _) = &page.content {
			assert_eq!(
				url,
				&format!(
					"https://m11.imhentai.xxx/033/readerkey/{}.webp",
					index + 1
				)
			);
		} else {
			panic!("not a URL");
		}
	}
	let incomplete = Html::parse_with_url(
		r#"<img id="gimg" src="https://m11.imhentai.xxx/033/readerkey/1.webp"><script>var g_th = $.parseJSON('{"1":"w,792,1224","3":"w,792,1224"}');</script>"#,
		BASE_URL,
	)
	.unwrap();
	assert!(parse_pages(&incomplete).is_err());
	let foreign = Html::parse_with_url(
		r#"<img id="gimg" src="https://m11.imhentai.xxx.evil/033/readerkey/1.webp"><script>var g_th = $.parseJSON('{"1":"w,792,1224"}');</script>"#,
		BASE_URL,
	)
	.unwrap();
	assert!(parse_pages(&foreign).is_err());
}

#[aidoku_test]
fn current_live_imhentai_reader_fixture_builds_all_pages() {
	let doc = Html::parse_with_url(
		include_str!("../fixtures/live-reader-page-one.html"),
		"https://imhentai.xxx/view/1743990/1/",
	)
	.unwrap();
	let reader_url = "https://imhentai.xxx/view/1743990/1/";
	let pages = parse_pages_with_referer(&doc, reader_url).unwrap();
	assert_eq!(pages.len(), 50);
	if let PageContent::Url(url, _) = &pages[0].content {
		assert_eq!(url, "https://m11.imhentai.xxx/033/nja31r49bw/1.jpg");
	} else {
		panic!("page must be a URL");
	}
	if let PageContent::Url(_, Some(context)) = &pages[0].content {
		assert_eq!(context.get("url").map(String::as_str), Some(reader_url));
	} else {
		panic!("reader page must carry its referer context");
	}
	if let PageContent::Url(url, _) = &pages[1].content {
		assert_eq!(url, "https://m11.imhentai.xxx/033/nja31r49bw/2.jpg");
	} else {
		panic!("page must be a URL");
	}
	if let PageContent::Url(url, _) = &pages[49].content {
		assert_eq!(url, "https://m11.imhentai.xxx/033/nja31r49bw/50.jpg");
	} else {
		panic!("page must be a URL");
	}
}

#[aidoku_test]
fn image_requests_use_reader_context_and_validate_image_hosts() {
	use aidoku::ImageRequestProvider;
	let source = GallerySource;
	let mut context = aidoku::PageContext::new();
	context.insert(
		"url".into(),
		"https://imhentai.xxx/view/1743990/12/".into(),
	);
	assert_eq!(
		image_request_referer(
			"https://m11.imhentai.xxx/033/nja31r49bw/12.jpg",
			Some(&context)
		)
		.unwrap(),
		"https://imhentai.xxx/view/1743990/12/"
	);
	assert_eq!(
		image_request_referer("https://m11.imhentai.xxx/033/nja31r49bw/cover.jpg", None)
			.unwrap(),
		"https://imhentai.xxx/"
	);
	for url in [
		"http://m11.imhentai.xxx/033/nja31r49bw/12.jpg",
		"https://m11.imhentai.xxx.evil/033/nja31r49bw/12.jpg",
		"https://example.org/033/nja31r49bw/12.jpg",
		"https://m11.imhentai.xxx/033/nja31r49bw/12.exe",
	] {
		assert!(image_request_referer(url, Some(&context)).is_err(), "{url}");
	}
	assert!(source
		.get_image_request(
			"https://m11.imhentai.xxx/033/nja31r49bw/12.jpg".into(),
			Some(context),
		)
		.is_ok());
}

#[aidoku_test]
fn page_descriptions_read_validated_reader_filenames() {
	use aidoku::PageDescriptionProvider;
	let source = GallerySource;
	let page = Page {
		content: PageContent::url("https://m2.imhentai.xxx/001/81/17.webp"),
		has_description: true,
		..Default::default()
	};
	assert_eq!(source.get_page_description(page).unwrap(), "Page 17");
	for url in [
		"https://m2.imhentai.xxx/001/81/x.webp",
		"https://m2.imhentai.xxx/001/81/1.exe",
		"https://m2.imhentai.xxx/001/81/1.webp/extra",
	] {
		assert!(
			source
				.get_page_description(Page {
					content: PageContent::url(url),
					..Default::default()
				})
				.is_err(),
			"{url}"
		);
	}
}

#[aidoku_test]
fn discovery_home_has_working_latest_listing() {
	let home = home_layout();
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
			assert!(entries.is_empty());
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
	let doc = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"></a></div><div class="caption">Sample</div></div>"#,
		BASE_URL,
	)
	.unwrap();
	let latest = parse_search(&doc);
	let component = listing_component_from_result("latest", "Latest", latest);
	let mut home = home_layout();
	update_home_component(&mut home, component);
	if let aidoku::HomeComponentValue::Scroller { entries, listing } = &home.components[0].value {
		assert_eq!(entries.len(), 1);
		assert_eq!(listing.as_ref().unwrap().id, "latest");
	} else {
		panic!("expected latest scroller");
	}
	let popular_doc = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/43/"></a></div><div class="caption">Popular</div></div>"#,
		BASE_URL,
	)
	.unwrap();
	let component = listing_component_from_result("popular", "Popular", parse_search(&popular_doc));
	update_home_component(&mut home, component);
	if let aidoku::HomeComponentValue::Scroller { entries, listing } = &home.components[1].value {
		assert_eq!(entries.len(), 1);
		assert_eq!(listing.as_ref().unwrap().id, "popular");
		assert_eq!(entries[0].title, "Popular");
	} else {
		panic!("expected popular scroller");
	}
	assert!(parse_search(&Html::parse("<div class='container'>Access denied</div>").unwrap()).entries.is_empty());
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
	assert_eq!(manifest["info"]["version"], 27);
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

#[aidoku_test]
fn live_reader_page_one_fixture_builds_all_manifest_pages() {
	let reader_url = "https://imhentai.xxx/view/1743990/1/";
	let html = include_str!("../fixtures/live-reader-page-one.html");
	let doc = Html::parse_with_url(html, reader_url).unwrap();
	let pages = parse_pages_with_referer(&doc, reader_url).unwrap();
	assert_eq!(pages.len(), 50);
	if let PageContent::Url(url, Some(context)) = &pages[0].content {
		assert_eq!(url, "https://m11.imhentai.xxx/033/nja31r49bw/1.jpg");
		assert_eq!(context.get("url").map(String::as_str), Some(reader_url));
	} else {
		panic!("live reader page must retain its reader context");
	}
	if let PageContent::Url(url, _) = &pages[49].content {
		assert_eq!(url, "https://m11.imhentai.xxx/033/nja31r49bw/50.jpg");
	} else {
		panic!("last live reader page must be a URL");
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
	assert_eq!(result.entries[0].update_strategy, UpdateStrategy::Never);
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
	assert_eq!(
		deep_link_key("https://imhentai.xxx/view/123/4/"),
		Some("123".into())
	);
	assert!(deep_link_key("https://imhentai.xxx.evil/gallery/123/").is_none());
	assert!(deep_link_key("https://imhentai.xxx.evil.example/gallery/123/").is_none());
	assert!(deep_link_key("http://imhentai.xxx/gallery/123/").is_none());
	assert!(deep_link_key("https://example.org/gallery/123/").is_none());
	assert!(deep_link_key("https://imhentai.xxx/gallery/nope/").is_none());
	assert_eq!(
		source
			.handle_deep_link("https://imhentai.xxx/gallery/123/".into())
			.unwrap(),
		Some(DeepLinkResult::Manga { key: "123".into() })
	);
}
