use crate::{API_URL, auth, models::BookmarkResponse};
use aidoku::{
	Result,
	alloc::{String, Vec},
	imports::{
		defaults::{DefaultValue, defaults_get, defaults_set, defaults_set_data},
		net::Request,
		std::current_date,
	},
	prelude::*,
};
use serde::{Deserialize, Serialize};

const CACHE_KEY: &str = "browse_bookmarks_cache";
const CACHE_TTL_SECONDS: i64 = 300;
const PAGE_SIZE: i32 = 100;
const MAX_BOOKMARKS: i32 = 100_000;

#[derive(Serialize, Deserialize)]
struct BookmarkCache {
	expires_at: i64,
	slugs: Vec<String>,
}

fn cache_is_fresh(cache: &BookmarkCache, now: i64) -> bool {
	cache.expires_at > now
}

fn next_bookmark_offset(offset: i32, page_len: i32, total: i32) -> Result<Option<i32>> {
	if offset < 0 || total < 0 || total > MAX_BOOKMARKS {
		return Err(error!("Bookmark list size is invalid or too large"));
	}
	if page_len < 0 || page_len > PAGE_SIZE || offset.saturating_add(page_len) > total {
		return Err(error!("Bookmark list page is inconsistent"));
	}
	if offset >= total {
		return Ok(None);
	}
	if page_len == 0 {
		return Err(error!("Bookmark list ended before its reported total"));
	}
	let next = offset.saturating_add(page_len);
	Ok((next < total).then_some(next))
}

pub fn clear_cache() {
	defaults_set(CACHE_KEY, DefaultValue::Null);
}

pub fn get_bookmarked_slugs() -> Result<Vec<String>> {
	let now = current_date();
	if let Some(cache) = defaults_get::<BookmarkCache>(CACHE_KEY)
		&& cache_is_fresh(&cache, now)
	{
		return Ok(cache.slugs);
	}

	let token = auth::get_access_token()?;
	let mut offset = 0i32;
	let mut total = None;
	let mut slugs = Vec::new();
	loop {
		let url = format!(
			"{API_URL}/me/bookmarks?sort=updated&order=desc&limit={PAGE_SIZE}&offset={offset}"
		);
		let response: BookmarkResponse = Request::get(url)?
			.header("Authorization", &format!("Bearer {token}"))
			.json_owned()?;
		let page_len = i32::try_from(response.data.len())
			.map_err(|_| error!("Invalid bookmark page length"))?;
		let page_total = response.meta.total;
		let next_offset = next_bookmark_offset(offset, page_len, page_total)?;
		if let Some(previous_total) = total {
			if page_total != previous_total {
				return Err(error!("Bookmark list changed while loading"));
			}
		} else {
			total = Some(page_total);
		}
		slugs.extend(response.into_slugs());
		match next_offset {
			Some(next) => offset = next,
			None => break,
		}
	}

	let cache = BookmarkCache {
		expires_at: now.saturating_add(CACHE_TTL_SECONDS),
		slugs: slugs.clone(),
	};
	defaults_set_data(CACHE_KEY, cache);
	Ok(slugs)
}

#[cfg(test)]
mod tests {
	use super::*;
	use aidoku::alloc::vec;
	use aidoku_test::aidoku_test;

	#[aidoku_test]
	fn bookmark_cache_expires_at_its_deadline() {
		let cache = BookmarkCache {
			expires_at: 110,
			slugs: vec!["fixture".into()],
		};
		assert!(cache_is_fresh(&cache, 109));
		assert!(!cache_is_fresh(&cache, 110));
		assert!(!cache_is_fresh(&cache, 111));
	}

	#[aidoku_test]
	fn bookmark_pagination_advances_by_actual_page_length_and_rejects_stalls() {
		assert_eq!(next_bookmark_offset(0, 100, 250).unwrap(), Some(100));
		assert_eq!(next_bookmark_offset(100, 80, 250).unwrap(), Some(180));
		assert_eq!(next_bookmark_offset(180, 70, 250).unwrap(), None);
		assert_eq!(next_bookmark_offset(0, 0, 0).unwrap(), None);
		assert!(next_bookmark_offset(0, 0, 4).is_err());
		assert!(next_bookmark_offset(200, 100, 250).is_err());
		assert!(next_bookmark_offset(0, 1, MAX_BOOKMARKS + 1).is_err());
	}
}
