use super::models::*;
#[aidoku_test]
fn today_section_can_browse_real_today_listing() {
	let component = super::home::today_component(aidoku::alloc::vec![aidoku::Manga {
		key: "1".into(),
		title: "Neutral fixture".into(),
		..Default::default()
	}]);
	assert_eq!(component.title.as_deref(), Some("Popular Today"));
	match component.value {
		aidoku::HomeComponentValue::MangaList {
			entries,
			listing,
			ranking,
			..
		} => {
			assert!(ranking);
			assert_eq!(entries.len(), 1);
			assert_eq!(listing.unwrap().id, "popular-today");
		}
		_ => panic!("Today must provide listing navigation"),
	}
}
#[aidoku_test]
fn discovery_rejects_invalid_page_and_listing() {
	use aidoku::{Listing, ListingProvider, Source};
	let source = super::NHentai::new();
	assert!(
		source
			.get_manga_list(
				Listing {
					id: "latest".into(),
					..Default::default()
				},
				0
			)
			.is_err()
	);
	assert!(
		source
			.get_manga_list(
				Listing {
					id: "unsupported".into(),
					..Default::default()
				},
				1
			)
			.is_err()
	);
}
#[cfg(feature = "live-tests")]
#[aidoku_test]
fn live_discovery_metadata_only() {
	use aidoku::{Home, Listing, ListingProvider, Source};
	let source = super::NHentai::new();
	// No covers or pages are fetched or logged.
	for id in ["popular-today", "popular-week", "popular", "latest"] {
		let listing = Listing {
			id: id.into(),
			..Default::default()
		};
		let first = source.get_manga_list(listing.clone(), 1).unwrap();
		assert!(!first.entries.is_empty());
		assert!(first.has_next_page);
		let second = source.get_manga_list(listing, 2).unwrap();
		assert!(!second.entries.is_empty());
		assert!(first.entries[0].key != second.entries[0].key);
	}
	source.get_home().unwrap();
}
use aidoku_test::aidoku_test;

#[aidoku_test]
fn fixture_search_metadata_deserializes() {
	// Synthetic API metadata only; never download gallery content in tests.
	let response: NHentaiSearchResponse = serde_json::from_str(r#"{
        "result":[{"id":1,"media_id":"fixture","thumbnail":"/fixture/cover.png","thumbnail_width":128,"thumbnail_height":128,"english_title":"Fixture","japanese_title":null,"tag_ids":[]}],
        "num_pages":2,"per_page":25,"total":26
    }"#).unwrap();
	assert_eq!(response.result.len(), 1);
	assert_eq!(response.result[0].id, 1);
	assert_eq!(response.result[0].english_title, "Fixture");
	assert_eq!(response.num_pages, 2);
}

#[aidoku_test]
fn malformed_metadata_is_not_silently_accepted() {
	assert!(serde_json::from_str::<NHentaiSearchResponse>(r#"{"result":[{}]}"#).is_err());
}

#[aidoku_test]
fn fixture_image_paths_preserve_host_and_slashes() {
	assert_eq!(
		make_image_url("fixture/page.png", false),
		"https://i.nhentai.net/fixture/page.png"
	);
	assert_eq!(
		make_image_url("/fixture/cover.png", true),
		"https://t.nhentai.net/fixture/cover.png"
	);
	assert_eq!(
		make_image_url("https://example.invalid/image.png", false),
		"https://example.invalid/image.png"
	);
}

#[aidoku_test]
fn alternate_cover_variants_are_safe_and_use_official_image_hosts() {
	let covers = super::models::cover_variants("12345", "fixturemedia123", "cover.jpg");
	assert_eq!(covers.len(), 2);
	assert_eq!(
		covers[0],
		"https://i.nhentai.net/galleries/fixturemedia123/cover.jpg"
	);
	assert_eq!(
		covers[1],
		"https://t.nhentai.net/galleries/fixturemedia123/cover.jpg"
	);
	assert!(super::models::cover_variants("../123", "fixture", "cover.jpg").is_empty());
	assert!(super::models::cover_variants("123", "fixture/../../host", "cover.jpg").is_empty());
	assert!(super::models::cover_variants("123", "fixture", "cover.unknown").is_empty());
}

#[aidoku_test]
fn details_include_language_metadata() {
	let gallery: NHentaiGallery = serde_json::from_str(r#"{
		"id":123,"media_id":"fixture123","title":{"english":"Fixture","japanese":null,"pretty":"Fixture"},
		"cover":{"path":"galleries/fixture123/cover.jpg","width":1,"height":1},
		"thumbnail":{"path":"galleries/fixture123/thumb.jpg","width":1,"height":1},"scanlator":"","upload_date":0,
		"tags":[{"id":1,"name":"english","count":1,"type":"language","url":"/language/english"}],
		"num_pages":1,"num_favorites":0,"pages":[]
	}"#).unwrap();
	let manga: aidoku::Manga = gallery.into();
	assert!(manga.description.unwrap().contains("Languages: english"));
}

#[aidoku_test]
fn dynamic_language_filter_uses_only_unique_language_tags() {
	let filter = super::language_filter(aidoku::alloc::vec![
		NHentaiTag {
			id: 1,
			name: "japanese".into(),
			count: 340_913,
			r#type: "language".into(),
			url: "/language/japanese/".into(),
			slug: Some("japanese".into()),
		},
		NHentaiTag {
			id: 2,
			name: "japanese".into(),
			count: 340_913,
			r#type: "language".into(),
			url: "/language/japanese/".into(),
			slug: Some("japanese".into()),
		},
		NHentaiTag {
			id: 3,
			name: "action".into(),
			count: 10,
			r#type: "tag".into(),
			url: "/tag/action/".into(),
			slug: Some("action".into()),
		},
		NHentaiTag {
			id: 4,
			name: "textless narrative".into(),
			count: 1,
			r#type: "language".into(),
			url: "/language/textless-narrative/".into(),
			slug: Some("textless-narrative".into()),
		},
	])
	.unwrap();
	match filter.kind {
		aidoku::FilterKind::MultiSelect { options, ids, .. } => {
			assert_eq!(options.len(), 2);
			assert_eq!(options[0].as_ref(), "japanese");
			assert_eq!(options[1].as_ref(), "textless narrative");
			let ids = ids.unwrap();
			assert_eq!(ids[0].as_ref(), "japanese");
			assert_eq!(ids[1].as_ref(), "textless narrative");
		}
		_ => panic!("expected multi-select language filter"),
	}
}

#[aidoku_test]
fn blocklist_setting_starts_empty_without_excluding_a_placeholder_tag() {
	let settings: serde_json::Value =
		serde_json::from_str(include_str!("../res/settings.json")).unwrap();
	let blocklist = settings
		.as_array()
		.unwrap()
		.iter()
		.flat_map(|group| group["items"].as_array().unwrap())
		.find(|item| item["key"] == "blocklist")
		.unwrap();
	assert_eq!(blocklist["default"], serde_json::json!([]));
}

#[aidoku_test]
fn deep_links_accept_only_numeric_gallery_paths_on_canonical_host() {
	use aidoku::{DeepLinkHandler, DeepLinkResult, Source};
	let source = super::NHentai::new();
	let valid = source
		.handle_deep_link("https://nhentai.net/g/12345/title/".into())
		.unwrap();
	assert!(matches!(valid, Some(DeepLinkResult::Manga { key }) if key == "12345"));
	for url in [
		"https://evil.example/nhentai.net/g/12345/",
		"https://nhentai.net.evil.example/g/12345/",
		"http://nhentai.net/g/12345/",
		"https://nhentai.net/archive/g/12345/",
		"https://nhentai.net/g/nope/",
		"https://nhentai.net/g//",
	] {
		assert!(
			source.handle_deep_link(url.into()).unwrap().is_none(),
			"{url}"
		);
	}
}
