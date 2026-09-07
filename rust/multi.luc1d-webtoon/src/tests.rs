use super::*;
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
