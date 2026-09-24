use crate::BASE_URL;
use aidoku::{alloc::string::String, prelude::*};

/// Returns the ID of a manga from a URL.
pub fn get_manga_key(url: &str) -> Option<String> {
	// Asura Scans appends a random string at the end of each series slug
	// The random string is not necessary, along with the trailing '-'

	// remove query parameters
	let path = url.split('?').next().unwrap_or("");

	// find the segment after "series"
	let manga_segment = path
		.split('/')
		.skip_while(|segment| *segment != "comics")
		.nth(1)?;

	// find the last '-' and keep it in the id
	let pos = manga_segment.rfind('-')?;
	Some(manga_segment[..pos].into())
}

/// Returns the ID of a chapter from a URL.
pub fn get_chapter_key(url: &str) -> Option<String> {
	// remove query parameters
	let path = url.split('?').next().unwrap_or("");

	// find the segment after "chapter"
	let chapter_segment = path
		.split('/')
		.skip_while(|segment| *segment != "chapter")
		.nth(1)?;

	// extract only the numeric (and '.') prefix
	let end_pos = chapter_segment
		.find(|c: char| !c.is_numeric() && c != '.')
		.unwrap_or(chapter_segment.len());

	let chapter_key = &chapter_segment[..end_pos];
	if chapter_key.is_empty() || !chapter_key.chars().any(|c| c.is_numeric()) {
		return None;
	}
	Some(chapter_key.into())
}

/// Returns whether a URL belongs to the Asura Scans website.
pub fn is_asura_url(url: &str) -> bool {
	let Some((scheme, remainder)) = url.split_once("://") else {
		return false;
	};
	if scheme != "https" && scheme != "http" {
		return false;
	}
	let host = remainder
		.split(['/', '?', '#'])
		.next()
		.unwrap_or("")
		.split('@')
		.next_back()
		.unwrap_or("")
		.split(':')
		.next()
		.unwrap_or("")
		.to_ascii_lowercase();
	matches!(host.as_str(), "asurascans.com" | "www.asurascans.com")
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: &str) -> String {
	format!("{BASE_URL}/comics/{manga_id}")
}

/// Returns full URL of a chapter from a chapter ID and manga ID.
pub fn get_chapter_url(chapter_id: &str, manga_id: &str) -> String {
	format!("{BASE_URL}/comics/{manga_id}/chapter/{chapter_id}")
}

#[cfg(test)]
mod tests {
	use super::*;
	use aidoku_test::aidoku_test;

	#[aidoku_test]
	fn test_manga_keys() {
		assert_eq!(
			get_manga_key("https://asurascans.com/comics/swordmasters-youngest-son-cb22671f")
				.as_deref(),
			Some("swordmasters-youngest-son")
		);
		assert_eq!(
			get_manga_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f?blahblah"
			)
			.as_deref(),
			Some("swordmasters-youngest-son")
		);
		assert_eq!(
			get_manga_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f/chapter/1"
			)
			.as_deref(),
			Some("swordmasters-youngest-son")
		);
	}

	#[aidoku_test]
	fn test_chapter_keys() {
		assert_eq!(
			get_chapter_key("https://asurascans.com/comics/swordmasters-youngest-son-cb22671f"),
			None
		);
		assert_eq!(
			get_chapter_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f?blahblah"
			),
			None
		);
		assert_eq!(
			get_chapter_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f/chapter/1"
			)
			.as_deref(),
			Some("1")
		);
		assert_eq!(
			get_chapter_key("https://asurascans.com/comics/swordmasters-youngest-son/chapter/1")
				.as_deref(),
			Some("1")
		);
		assert_eq!(
			get_chapter_key("https://asurascans.com/comics/series/chapter/abc"),
			None
		);
		assert_eq!(
			get_chapter_key("https://asurascans.com/comics/series/chapter/."),
			None
		);
	}

	#[aidoku_test]
	fn test_asura_url_hosts() {
		assert!(is_asura_url("https://asurascans.com/comics/series-123"));
		assert!(is_asura_url("https://www.asurascans.com/comics/series-123"));
		assert!(!is_asura_url("https://evil.example/comics/series-123"));
		assert!(!is_asura_url(
			"https://asurascans.com.evil.example/comics/series-123"
		));
		assert!(!is_asura_url(
			"javascript://asurascans.com/comics/series-123"
		));
	}
}
