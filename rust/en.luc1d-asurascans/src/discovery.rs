use aidoku::{
	Manga, MangaPageResult, Result,
	alloc::{String, Vec},
};
use serde::Deserialize;

// Only periods offered by the comic PopularSidebar; no daily alias.
pub fn popularity_period(id: &str) -> Option<&'static str> {
	match id {
		"popular-week" => Some("week"),
		"popular-month" => Some("month"),
		"popular-all" => Some("all"),
		_ => None,
	}
}

#[derive(Deserialize)]
struct PopularResponse {
	data: Vec<PopularSeries>,
}
#[derive(Deserialize)]
struct PopularSeries {
	slug: String,
	title: String,
	cover_url: Option<String>,
}

pub fn parse_popularity(json: &str) -> Result<MangaPageResult> {
	let response: PopularResponse = serde_json::from_str(json)?;
	Ok(MangaPageResult {
		entries: response
			.data
			.into_iter()
			.filter(|m| !m.slug.is_empty() && !m.title.is_empty())
			.map(|m| Manga {
				key: m.slug,
				title: m.title,
				cover: m.cover_url,
				..Default::default()
			})
			.collect(),
		has_next_page: false,
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use aidoku_test::aidoku_test;
	#[aidoku_test]
	fn periods_are_explicit_not_invented() {
		assert_eq!(popularity_period("popular-week"), Some("week"));
		assert_eq!(popularity_period("popular-month"), Some("month"));
		assert_eq!(popularity_period("popular-all"), Some("all"));
		assert_eq!(popularity_period("popular-day"), None);
	}
	#[aidoku_test]
	fn captured_popularity_keeps_server_order_and_keys() {
		let page = parse_popularity(include_str!("../fixtures/popular-week.json")).unwrap();
		assert_eq!(page.entries.len(), 2);
		assert_eq!(page.entries[0].key, "surviving-the-game-as-a-barbarian");
		assert_eq!(page.entries[1].key, "limitless-predation");
		assert!(!page.has_next_page);
		assert!(parse_popularity(r#"{"error":"unavailable"}"#).is_err());
		assert!(
			parse_popularity(r#"{"data":[]}"#)
				.unwrap()
				.entries
				.is_empty()
		);
	}
	#[aidoku_test]
	fn single_page_listing_does_not_repeat_or_accept_invalid_pages() {
		use aidoku::{Listing, ListingProvider};
		let source = crate::AsuraScans;
		assert!(
			source
				.get_manga_list(
					Listing {
						id: "popular-week".into(),
						..Default::default()
					},
					0
				)
				.is_err()
		);
		let result = source
			.get_manga_list(
				Listing {
					id: "popular-week".into(),
					..Default::default()
				},
				2,
			)
			.unwrap();
		assert!(result.entries.is_empty());
		assert!(!result.has_next_page);
	}
	#[aidoku_test]
	#[ignore]
	fn live_popularity_metadata_only() {
		use aidoku::{Home, HomeComponentValue};
		let home = crate::AsuraScans.get_home().unwrap();
		for expected in ["popular-week", "popular-month", "popular-all"] {
			assert!(home.components.iter().any(|c| match &c.value {
				HomeComponentValue::Scroller {
					entries,
					listing: Some(listing),
				} => listing.id == expected && !entries.is_empty(),
				_ => false,
			}));
		}
		use aidoku::{Listing, ListingProvider, Source};
		let source = crate::AsuraScans::new();
		for id in ["popular-week", "popular-month", "popular-all"] {
			let page = source
				.get_manga_list(
					Listing {
						id: id.into(),
						..Default::default()
					},
					1,
				)
				.unwrap();
			assert!(!page.entries.is_empty());
			assert!(
				page.entries
					.iter()
					.all(|m| !m.key.is_empty() && !m.title.is_empty())
			);
			assert!(!page.has_next_page);
		}
	}
}
