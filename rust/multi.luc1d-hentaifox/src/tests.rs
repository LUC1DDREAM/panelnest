#[aidoku_test]
fn synthetic_details_and_chapter_flags() {
	let doc=Html::parse_with_url(r#"<div class="gallery_top gallery_first"><h1>Sample 2</h1><div class="cover left_cover"><img src="/cover.png"></div><ul class="artists"><li><a>Artist 7</a></li></ul><ul class="languages"><span class="i_text">Languages:</span><li><a class="tag_btn">english <span class="t_badge">1</span></a></li><li><a class="tag_btn">translated <span class="t_badge">2</span></a></li></ul><span class="i_text pages">Pages: 193</span><span class="i_text pages">Posted: 2 days ago</span><ul><li><span class="tags_text">Artists:</span><a class="tag">Artist 7</a></li></ul></div>"#,BASE_URL).unwrap();
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
	assert_eq!(chapters[0].language.as_deref(), Some("en"));
	assert_eq!(chapters[0].thumbnail.as_deref(), Some("https://hentaifox.com/cover.png"));
	let uploaded_at = chapters[0].date_uploaded.unwrap();
	assert!(uploaded_at >= before_update - 172_800);
	assert!(uploaded_at <= aidoku::imports::std::current_date() - 172_800);
	assert_eq!(m.update_strategy, UpdateStrategy::Never);
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
fn chapter_language_maps_all_official_language_tags_and_ignores_classifications() {
	for (label, expected) in [
		("chinese", "zh"),
		("english", "en"),
		("french", "fr"),
		("german", "de"),
		("hindi", "hi"),
		("indonesian", "id"),
		("italian", "it"),
		("japanese", "ja"),
		("javanese", "jv"),
		("korean", "ko"),
		("norwegian", "no"),
		("portuguese", "pt"),
		("romanian", "ro"),
		("russian", "ru"),
		("sanskrit", "sa"),
		("spanish", "es"),
		("tagalog", "tl"),
		("thai", "th"),
		("turkish", "tr"),
		("ukrainian", "uk"),
		("vietnamese", "vi"),
	] {
		let html = format!(
			"<ul class='languages'><span class='i_text'>Languages:</span><li><a class='tag_btn'>{label}</a></li></ul>"
		);
		let doc = Html::parse_with_url(&html, BASE_URL).unwrap();
		assert_eq!(gallery_language(&doc).as_deref(), Some(expected), "{label}");
	}
	for classification in ["translated", "rewrite", "speechless", "text cleaned", "textless narrative"] {
		let html = format!(
			"<ul class='languages'><span class='i_text'>Languages:</span><li><a class='tag_btn'>{classification}</a></li></ul>"
		);
		let doc = Html::parse_with_url(&html, BASE_URL).unwrap();
		assert_eq!(gallery_language(&doc), None, "{classification}");
	}
	let mixed = Html::parse_with_url(
		"<ul class='languages'><span class='i_text'>Languages:</span><li><a class='tag_btn'>english</a></li><li><a class='tag_btn'>japanese</a></li></ul>",
		BASE_URL,
	)
	.unwrap();
	assert_eq!(gallery_language(&mixed), None);
}

#[aidoku_test]
fn live_gallery_artist_and_group_metadata_maps_to_aidoku_fields() {
	let doc = Html::parse_with_url(
		&format!(
			"<div class='gallery_top'><h1>Sample gallery</h1><div class='cover'><img src='/cover.jpg'></div></div>{}",
			include_str!("../fixtures/live-gallery-artists.html")
		),
		"https://hentaifox.com/gallery/173753/",
	)
	.unwrap();
	assert_eq!(gallery_artists(&doc), vec!["kumatora"]);
	assert_eq!(gallery_groups(&doc), vec!["studio mizuyokan"]);
	let manga = update(
		&doc,
		Manga {
			key: "173753".into(),
			..Default::default()
		},
		true,
		false,
	)
	.unwrap();
	assert_eq!(manga.description.as_deref(), Some("Groups: studio mizuyokan"));
	let existing = update(
		&doc,
		Manga {
			key: "173753".into(),
			description: Some("Existing summary".into()),
			..Default::default()
		},
		true,
		false,
	)
	.unwrap();
	assert_eq!(
		existing.description.as_deref(),
		Some("Existing summary\n\nGroups: studio mizuyokan")
	);
	assert_eq!(
		description_with_groups(
			Some("Existing summary\n\nGroups: previous group".into()),
			&gallery_groups(&doc),
		)
		.as_deref(),
		Some("Existing summary\n\nGroups: studio mizuyokan")
	);
	assert_eq!(
		description_with_groups(
			Some("Existing summary\n\nGroups: previous group".into()),
			&[],
		)
		.as_deref(),
		Some("Existing summary")
	);
}

#[aidoku_test]
fn posted_age_is_converted_to_a_bounded_approximate_timestamp() {
	assert_eq!(posted_age_seconds("Posted: 7 days ago"), Some(604_800));
	assert_eq!(posted_age_seconds("Posted: 2 months ago"), Some(5_184_000));
	assert_eq!(posted_age_seconds("Posted: yesterday"), Some(86_400));
	assert_eq!(posted_age_seconds("Posted: just now"), Some(0));
	assert_eq!(posted_age_seconds("Pages: 193"), None);
	assert_eq!(posted_age_seconds("Posted: someday ago"), None);
	assert_eq!(posted_age_seconds("Posted: -2 days ago"), None);
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
	let pages = parse_pages(&doc, "81").unwrap();
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
	assert!(pages.iter().all(|page| page.has_description));
	if let PageContent::Url(_, Some(context)) = &pages[0].content {
		assert_eq!(context.get("url").map(|url| url.as_str()), Some("https://hentaifox.com/gallery/81/"));
	} else {
		panic!("page must retain its gallery Referer context");
	}
	assert!(parse_pages(&Html::parse("<html></html>").unwrap(), "81").is_err());
	assert!(parse_pages(&doc, "bad").is_err());
}

#[aidoku_test]
fn page_descriptions_read_validated_reader_filenames() {
	use aidoku::PageDescriptionProvider;
	let source = GallerySource;
	let page = Page {
		content: PageContent::url("https://m2.hentaifox.com/001/81/17.webp"),
		has_description: true,
		..Default::default()
	};
	assert_eq!(source.get_page_description(page).unwrap(), "Page 17");
	for url in [
		"https://m2.hentaifox.com/001/81/x.webp",
		"https://m2.hentaifox.com/001/81/1.exe",
		"https://m2.hentaifox.com/001/81/1.webp/extra",
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
fn image_requests_validate_cdn_hosts_and_use_the_gallery_context() {
	use aidoku::ImageRequestProvider;
	let source = GallerySource;
	let mut context = aidoku::PageContext::new();
	context.insert("url".into(), "https://hentaifox.com/gallery/81/".into());
	assert!(source
		.get_image_request("https://m2.hentaifox.com/001/81/1.webp".into(), Some(context))
		.is_ok());
	assert!(source
		.get_image_request("https://evil.example/001/81/1.webp".into(), None)
		.is_err());
	let mut invalid_context = aidoku::PageContext::new();
	invalid_context.insert("url".into(), "https://hentaifox.com/gallery/nope/".into());
	assert!(source
		.get_image_request(
			"https://m2.hentaifox.com/001/81/1.webp".into(),
			Some(invalid_context),
		)
		.is_err());
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
fn initial_home_layout_contains_all_server_rendered_sections() {
	let doc = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"></a></div><div class="caption">Latest</div></div><button id="top_rated_btn" class="sidebar_btn_active">Top Rated</button><div id="middle_sidebar"><div class="item"><a href="/gallery/77/"><img alt="Rated" src="/rated.jpg"></a></div></div>"#,
		BASE_URL,
	)
	.unwrap();
	let home = parse_home(&doc).unwrap();
	assert_eq!(home.components.len(), 2);
	assert_eq!(home.components[0].title.as_deref(), Some("Latest"));
	assert_eq!(home.components[1].title.as_deref(), Some("Top Rated"));
}

#[aidoku_test]
fn daily_top_rated_today_and_yesterday_are_home_spotlights() {
	let doc = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"></a></div><div class="caption">Latest</div></div>
		<div id="main_top_daily">
			<a id="today_content" href="/gallery/173623/"><img src="//i3.hentaifox.com/today.jpg" alt="Today Pick"></a>
			<a id="yesterday_content" href="/gallery/173529/"><img src="//i3.hentaifox.com/yesterday.jpg" alt="Yesterday Pick"></a>
		</div>"#,
		BASE_URL,
	)
	.unwrap();
	let daily = parse_daily_top_rated(&doc);
	assert_eq!(daily.len(), 2);
	assert_eq!(daily[0].0, "Daily Top Rated Today");
	assert_eq!(daily[0].1.key, "173623");
	assert_eq!(daily[0].1.title, "Today Pick");
	assert_eq!(daily[0].1.update_strategy, UpdateStrategy::Never);
	assert_eq!(daily[0].1.cover.as_deref(), Some("https://i3.hentaifox.com/today.jpg"));
	assert_eq!(daily[1].0, "Daily Top Rated Yesterday");
	assert_eq!(daily[1].1.key, "173529");

	let home = parse_home(&doc).unwrap();
	assert_eq!(home.components[1].title.as_deref(), Some("Daily Top Rated Today"));
	assert_eq!(home.components[2].title.as_deref(), Some("Daily Top Rated Yesterday"));
	for component in &home.components[1..=2] {
		if let aidoku::HomeComponentValue::Scroller { entries, listing } = &component.value {
			assert_eq!(entries.len(), 1);
			assert!(listing.is_none());
		} else {
			panic!("expected a daily spotlight scroller");
		}
	}
	assert!(parse_daily_top_rated(&Html::parse("<div class='top_daily'></div>").unwrap()).is_empty());
}

#[aidoku_test]
fn top_rated_is_scoped_finite_and_not_today() {
	let doc = Html::parse_with_url(r#"<button id="top_rated_btn" class="sidebar_btn_active">Top Rated</button><div id="middle_sidebar"><div class="item"><a href="/gallery/77/"><img alt="Sample Rated" src="/cover.png"></a></div><div class="item"><a href="https://invalid.example/gallery/78/"><img alt="Foreign"></a></div></div><div class="item"><a href="/gallery/99/"><img alt="Unrelated"></a></div>"#, BASE_URL).unwrap();
	let result = parse_top_rated(&doc).unwrap();
	assert_eq!(result.entries.len(), 1);
	assert_eq!(result.entries[0].key, "77");
	assert_eq!(result.entries[0].title, "Sample Rated");
	assert_eq!(result.entries[0].update_strategy, UpdateStrategy::Never);
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
	assert_eq!(result.entries[0].update_strategy, UpdateStrategy::Never);
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
		popular_tag_url("tag-popular-big-breasts", 1, true).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/popular/")
	);
	assert_eq!(
		popular_tag_url("tag-popular-big-breasts", 2, true).unwrap(),
		format!("{BASE_URL}/tag/big-breasts/popular/pag/2/")
	);
	assert!(popular_tag_url("popular-tag-../evil", 1, false).is_err());
	assert!(popular_tag_url("tag-popular-../evil", 1, true).is_err());
	assert!(popular_tag_url("popular-tag-big-breasts", 1, true).is_err());
	assert!(popular_tag_url("tag-popular-big-breasts", 1, false).is_err());
	assert!(popular_tag_url("popular-tag-big-breasts", 0, false).is_err());
	assert!(popular_tag_url("latest", 1, false).is_err());
}

#[aidoku_test]
fn popular_tag_listing_ids_select_distinct_latest_and_popular_routes() {
	assert_eq!(popular_tag_slug("popular-tag-big-breasts"), Some("big-breasts"));
	assert_eq!(popular_sorted_tag_slug("tag-popular-big-breasts"), Some("big-breasts"));
	assert!(popular_tag_slug("tag-popular-big-breasts").is_none());
	assert!(popular_sorted_tag_slug("popular-tag-big-breasts").is_none());
	assert!(popular_sorted_tag_slug("tag-popular-Big-Breasts").is_none());
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
	assert_eq!(listings.len(), 4);
	assert_eq!(listings[0].id, "popular-tag-big-breasts");
	assert_eq!(listings[0].name, "Tag: Big Breasts");
	assert_eq!(listings[1].id, "tag-popular-big-breasts");
	assert_eq!(listings[1].name, "Popular: Big Breasts");
	assert_eq!(listings[2].id, "popular-tag-sole-female");
	assert_eq!(listings[3].id, "tag-popular-sole-female");
	assert!(parse_popular_tag_listings(&Html::parse("<html></html>").unwrap()).is_err());
}

#[aidoku_test]
fn popular_tag_directory_exposes_all_fifty_official_tags() {
	let mut html = String::from("<div class=\"tags_overview\">");
	for number in 1..=51 {
		html.push_str(&format!(
			"<div class=\"tag_item\"><a class=\"tag_btn\" href=\"/tag/tag-{number}/\"><h3 class=\"list_tag\">Tag {number}</h3></a></div>"
		));
	}
	html.push_str("</div>");
	let doc = Html::parse_with_url(&html, BASE_URL).unwrap();
	let listings = parse_popular_tag_listings(&doc).unwrap();
	assert_eq!(listings.len(), 100);
	assert!(listings.iter().any(|listing| listing.id == "popular-tag-tag-50"));
	assert!(listings.iter().any(|listing| listing.id == "tag-popular-tag-50"));
	assert!(!listings.iter().any(|listing| listing.id == "popular-tag-tag-51"));
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
	assert_eq!(manifest["info"]["version"], 35);
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
fn web_login_has_an_account_setting_and_logout_notification() {
	let settings: serde_json::Value =
		serde_json::from_str(include_str!("../res/settings.json")).unwrap();
	let login = &settings[0]["items"][0];
	assert_eq!(login["type"], "login");
	assert_eq!(login["key"], "login");
	assert_eq!(login["method"], "web");
	assert_eq!(login["notification"], "login");
	assert_eq!(login["url"], "https://hentaifox.com/login/");
	assert_eq!(login["refreshes"][0], "listings");
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
fn authenticated_bookmarks_listing_is_hidden_without_a_valid_login() {
	assert!(bookmarks_listing(false).is_none());
	let listing = bookmarks_listing(true).unwrap();
	assert_eq!(listing.id, "bookmarks");
	assert_eq!(listing.name, "Bookmarks");
}

#[aidoku_test]
fn favorites_response_parses_gallery_cards_and_button_pagination() {
	let first_page = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"><img src="/cover.jpg"></a></div><div class="caption">Saved title</div></div><div class="pagination"><button data-page="1">1</button><button data-page="2">Next</button></div>"#,
		BASE_URL,
	)
	.unwrap();
	let result = parse_favorites(&first_page, 1).unwrap();
	assert_eq!(result.entries.len(), 1);
	assert_eq!(result.entries[0].key, "42");
	assert!(result.has_next_page);

	let last_page = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/43/"><img src="/cover.jpg"></a></div><div class="caption">Last title</div></div><div class="pagination"><button data-page="2">2</button></div>"#,
		BASE_URL,
	)
	.unwrap();
	assert!(!parse_favorites(&last_page, 2).unwrap().has_next_page);
	assert!(parse_favorites(&last_page, 0).is_err());
}

#[aidoku_test]
fn faplist_is_a_login_only_account_listing() {
	assert!(faplist_listing(false).is_none());
	let listing = faplist_listing(true).unwrap();
	assert_eq!(listing.id, "faplist");
	assert_eq!(listing.name, "Faplist");
}

#[aidoku_test]
fn faplist_pages_use_the_official_paginated_route_and_validate_the_response_url() {
	assert_eq!(
		faplist_url(1).unwrap(),
		format!("{BASE_URL}/faplist/")
	);
	assert_eq!(
		faplist_url(2).unwrap(),
		format!("{BASE_URL}/faplist/pag/2/")
	);
	assert!(faplist_url(0).is_err());
	assert!(is_faplist_url("https://hentaifox.com/faplist/", 1));
	assert!(!is_faplist_url("https://hentaifox.com.evil/faplist/", 1));
	assert!(!is_faplist_url("http://hentaifox.com/faplist/", 1));
	assert!(!is_faplist_url("https://hentaifox.com/faplist/pag/3/", 2));
}

#[aidoku_test]
fn faplist_parses_gallery_cards_and_recognizes_the_official_empty_state() {
	let populated = Html::parse_with_url(
		r#"<div class="thumb"><div class="inner_thumb"><a href="/gallery/42/"><img src="/cover.jpg"></a></div><div class="caption">Saved title</div></div><ul class="pagination"><li class="active"><a href="/faplist/">1</a></li><li><a href="/faplist/pag/2/">Next</a></li></ul>"#,
		BASE_URL,
	)
	.unwrap();
	let result = parse_faplist(&populated).unwrap();
	assert_eq!(result.entries.len(), 1);
	assert_eq!(result.entries[0].key, "42");
	assert!(result.has_next_page);

	let empty = Html::parse_with_url(
		r#"<div class="galleries_overview"><div class="alert alert-info">Your faplist is empty. You can add some galleries.</div></div>"#,
		BASE_URL,
	)
	.unwrap();
	let result = parse_faplist(&empty).unwrap();
	assert!(result.entries.is_empty());
	assert!(!result.has_next_page);
	assert!(parse_faplist(&Html::parse("<html>Unexpected response</html>").unwrap()).is_err());
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
fn deep_links_open_tag_latest_and_popular_listings() {
	use aidoku::DeepLinkHandler;
	let source = GallerySource;
	for (url, expected_id) in [
		("https://hentaifox.com/tag/english/", "popular-tag-english"),
		(
			"https://hentaifox.com/tag/english/popular/",
			"tag-popular-english",
		),
		(
			"https://hentaifox.com/tag/english/?from=share#top",
			"popular-tag-english",
		),
	] {
		let Some(DeepLinkResult::Listing(listing)) =
			source.handle_deep_link(url.into()).unwrap()
		else {
			panic!("Expected listing deep link for {url}");
		};
		assert_eq!(listing.id, expected_id);
		let is_popular = expected_id.starts_with("tag-popular-");
		assert_eq!(
			popular_tag_url(&listing.id, 1, is_popular).unwrap(),
			if is_popular {
				"https://hentaifox.com/tag/english/popular/"
			} else {
				"https://hentaifox.com/tag/english/"
			}
		);
	}
	for url in [
		"https://hentaifox.com.evil/tag/english/",
		"http://hentaifox.com/tag/english/",
		"https://hentaifox.com/tag/English/",
		"https://hentaifox.com/tag/english/pag/2/",
		"https://hentaifox.com/language/english/",
	] {
		assert!(deep_link_listing(url).is_none(), "Unexpected listing for {url}");
	}
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
