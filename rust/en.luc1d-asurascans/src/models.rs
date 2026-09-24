use aidoku::{
	Manga,
	alloc::{string::String, vec::Vec},
	imports::std::{current_date, parse_date},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginStatus {
	pub access_token: String,
	pub refresh_token: String,
	pub is_subscribed: bool,
}

impl From<RefreshResponse> for LoginStatus {
	fn from(value: RefreshResponse) -> Self {
		let is_subscribed = value
			.data
			.subscription_status
			.as_ref()
			.is_some_and(|status| status.has_subscription.unwrap_or_default())
			|| value
				.data
				.user
				.as_ref()
				.is_some_and(|user| user.has_active_subscription());
		Self {
			access_token: value.data.access_token,
			refresh_token: value.data.refresh_token.unwrap_or_default(),
			is_subscribed,
		}
	}
}

#[derive(Deserialize)]
pub struct RefreshResponse {
	pub data: RefreshResponseData,
}

#[derive(Deserialize)]
pub struct RefreshResponseData {
	pub access_token: String,
	pub refresh_token: Option<String>,
	pub subscription_status: Option<RefreshSubscriptionStatus>,
	pub user: Option<AuthUser>,
}

#[derive(Deserialize)]
pub struct RefreshSubscriptionStatus {
	pub has_subscription: Option<bool>,
}

#[derive(Deserialize)]
pub struct AuthUser {
	role: Option<String>,
	premium_until: Option<String>,
}

impl AuthUser {
	fn has_active_subscription(&self) -> bool {
		let role = self.role.as_deref().unwrap_or("user");
		matches!(role, "staff" | "moderator" | "uploader" | "admin")
			|| matches!(role, "basic" | "premium")
				&& self
					.premium_until
					.as_deref()
					.and_then(|date| parse_date(date, "yyyy-MM-dd'T'HH:mm:ss'Z'"))
					.is_some_and(|date| date > current_date())
	}
}

#[derive(Deserialize)]
pub struct BookmarkResponse {
	pub data: Vec<BookmarkItem>,
	pub meta: BookmarkResponseMeta,
}

#[derive(Deserialize)]
pub struct BookmarkResponseMeta {
	pub total: i32,
}

impl BookmarkResponse {
	pub fn has_next_page(&self, offset: i32) -> bool {
		offset.saturating_add(self.data.len() as i32) < self.meta.total
	}
}

#[cfg(test)]
mod bookmark_pagination_tests {
	use super::*;
	use aidoku_test::aidoku_test;

	#[aidoku_test]
	fn bookmark_pagination_uses_offset_and_page_length() {
		let response: BookmarkResponse = serde_json::from_str(r#"{
			"data":[{"series":{"cover_url":"https://example.invalid/cover.jpg","slug":"fixture","title":"Fixture"}}],
			"meta":{"total":21}
		}"#).unwrap();
		assert!(response.has_next_page(0));
		assert!(!response.has_next_page(20));
		let empty = BookmarkResponse { data: aidoku::alloc::Vec::new(), meta: BookmarkResponseMeta { total: 0 } };
		assert!(!empty.has_next_page(0));
	}
}

#[derive(Deserialize)]
pub struct BookmarkItem {
	// pub id: i32,
	series: BookmarkSeries,
}

impl From<BookmarkItem> for Manga {
	fn from(value: BookmarkItem) -> Self {
		Manga {
			key: value.series.slug,
			title: value.series.title,
			cover: Some(value.series.cover_url),
			..Default::default()
		}
	}
}

#[derive(Deserialize)]
pub struct BookmarkSeries {
	cover_url: String,
	slug: String,
	title: String,
}
