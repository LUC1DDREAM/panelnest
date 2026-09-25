use aidoku::{
	Chapter,
	alloc::{String, string::ToString},
};
use serde_json::Value;

/// Convert one public Astro chapter row without changing chapter identity/order.
pub fn chapter_from_astro(
	row: &Value,
	manga_key: &str,
	subscribed: bool,
	now: i64,
) -> Option<Chapter> {
	let obj = row[1].as_object()?;
	let premium = obj.get("is_premium").and_then(|v| v[1].as_bool());
	// Only the site's explicit early-access deadline is a release promise.
	// Never derive an unlock time from publication dates or a premium/coin flag.
	let release = obj
		.get("early_access_until")
		.and_then(|v| v[1].as_str())
		.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
	let remaining = release
		.map(|date| {
			date.timestamp()
				.saturating_add(i64::from(date.timestamp_subsec_nanos() > 0))
				.saturating_sub(now)
		})
		.unwrap_or(0);
	let locked = premium.is_none() || (!subscribed && (premium != Some(false) || remaining > 0));
	let status = if !locked {
		None
	} else if let Some(date) = release.filter(|_| remaining > 0) {
		let minutes = remaining.saturating_add(59) / 60;
		Some(aidoku::alloc::format!(
			"Unlocks in {}h {}m (at refresh); release {} UTC",
			minutes / 60,
			minutes % 60,
			date.with_timezone(&chrono::Utc).format("%Y-%m-%d %H:%M:%S")
		))
	} else if premium == Some(true) {
		Some(if let Some(date) = release {
			aidoku::alloc::format!(
				"Premium locked; scheduled release passed ({} UTC); refresh to check",
				date.with_timezone(&chrono::Utc).format("%Y-%m-%d %H:%M:%S")
			)
		} else {
			"Premium locked; release time unknown".into()
		})
	} else {
		Some("Locked; availability unknown".into())
	};
	let original_title = obj
		.get("title")
		.and_then(|v| v[1].as_str())
		.filter(|s| !s.trim().is_empty());
	let title = match (original_title, status) {
		(Some(original), Some(status)) => Some(aidoku::alloc::format!("{original} — {status}")),
		(Some(original), None) => Some(original.into()),
		(None, status) => status,
	};
	let chapter_number = obj.get("number")?[1].as_f64().map(|f| f as f32)?;
	let key = chapter_number.to_string();
	// Parse RFC3339 directly: the WASM runner does not implement Swift's quoted
	// date-format literals. Keep publication time separate from unlock time.
	let date_uploaded = obj.get("published_at")?[1]
		.as_str()
		.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
		.map(|date| date.timestamp());
	let url = crate::helpers::get_chapter_url(&key, manga_key);
	Some(Chapter {
		key,
		title,
		chapter_number: Some(chapter_number),
		date_uploaded,
		language: Some("en".into()),
		url: Some(url),
		locked,
		..Default::default()
	})
}

pub fn with_series_thumbnail(mut chapter: Chapter, cover: &Option<String>) -> Chapter {
	chapter.thumbnail = cover.clone();
	chapter
}

/// Gate page extraction using freshly fetched source metadata, never a cached lock bit.
pub fn require_readable(chapter: Option<&Chapter>) -> aidoku::Result<()> {
	match chapter {
		Some(chapter) if !chapter.locked => Ok(()),
		Some(_) => Err(aidoku::error!(
			"Chapter is locked. Refresh for its release status or sign in with authorized access."
		)),
		None => Err(aidoku::error!(
			"Cannot verify chapter availability. Refresh and try again."
		)),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use aidoku_test::aidoku_test;

	#[aidoku_test]
	fn synthetic_lock_classification_preserves_identity_and_title() {
		let fixture: Value =
			serde_json::from_str(include_str!("../tests/fixtures/synthetic-lock-cases.json"))
				.unwrap();
		for case in fixture["cases"].as_array().unwrap() {
			let row = serde_json::json!([0, {
				"number": [0, 12.5], "title": [0, "Original"],
				"is_premium": [0, case["premium"]],
				"early_access_until": [0, case["release"]],
				"published_at": [0, "2026-09-07T14:48:07Z"]
			}]);
			let chapter = chapter_from_astro(&row, "series", false, 0).unwrap();
			assert_eq!(
				chapter.locked,
				case["locked"].as_bool().unwrap(),
				"{}",
				case["name"]
			);
			let expected = case["status"]
				.as_str()
				.map(|s| aidoku::alloc::format!("Original — {s}"))
				.unwrap_or_else(|| "Original".into());
			assert_eq!(
				chapter.title.as_deref(),
				Some(expected.as_str()),
				"{}",
				case["name"]
			);
			assert_eq!(chapter.key, "12.5");
			assert_eq!(chapter.chapter_number, Some(12.5));
			// Preserve a valid publication timestamp; never weaken this to None.
			assert_eq!(chapter.date_uploaded, Some(1788792487));
			assert_eq!(chapter.language.as_deref(), Some("en"));
			assert_eq!(
				chapter.url.as_deref(),
				Some("https://asurascans.com/comics/series/chapter/12.5")
			);
			if case["premium"].is_boolean() {
				let subscriber = chapter_from_astro(&row, "series", true, 0).unwrap();
				assert!(!subscriber.locked);
				assert_eq!(subscriber.title.as_deref(), Some("Original"));
			} else {
				assert!(chapter_from_astro(&row, "series", true, 0).unwrap().locked);
			}
		}
	}

	#[aidoku_test]
	fn page_guard_denies_current_locks_and_missing_metadata() {
		let locked = Chapter {
			locked: true,
			..Default::default()
		};
		assert!(require_readable(Some(&locked)).is_err());
		assert!(require_readable(None).is_err());
		let free = Chapter {
			locked: false,
			..Default::default()
		};
		assert!(require_readable(Some(&free)).is_ok());
	}

	#[aidoku_test]
	fn chapters_can_reuse_the_series_cover_as_their_thumbnail() {
		let chapter = with_series_thumbnail(
			Chapter::default(),
			&Some("https://cdn.asurascans.com/asura-images/covers/series.webp".into()),
		);
		assert_eq!(
			chapter.thumbnail.as_deref(),
			Some("https://cdn.asurascans.com/asura-images/covers/series.webp")
		);
		assert!(
			with_series_thumbnail(Chapter::default(), &None)
				.thumbnail
				.is_none()
		);
	}

	#[aidoku_test]
	fn genuine_publication_dates_and_order_are_preserved() {
		let fixture: Value = serde_json::from_str(include_str!(
			"../tests/fixtures/public-chapter-metadata.json"
		))
		.unwrap();
		for source in fixture["sources"].as_array().unwrap() {
			let rows = source["chapters"].as_array().unwrap();
			let chapters: aidoku::alloc::Vec<_> = rows
				.iter()
				.filter_map(|row| chapter_from_astro(row, "series", false, 1788792487))
				.collect();
			assert_eq!(chapters.len(), rows.len());
			for (row, chapter) in rows.iter().zip(chapters.iter()) {
				let expected = chrono::DateTime::parse_from_rfc3339(
					row[1]["published_at"][1].as_str().unwrap(),
				)
				.unwrap()
				.timestamp();
				assert_eq!(chapter.date_uploaded, Some(expected));
				assert_eq!(
					chapter.chapter_number,
					row[1]["number"][1].as_f64().map(|n| n as f32)
				);
			}
		}
	}

	#[aidoku_test]
	fn publication_dates_handle_epoch_offsets_and_invalid_values() {
		for (date, expected) in [
			("1970-01-01T00:00:00Z", Some(0)),
			("2026-09-07T16:48:07.123456+02:00", Some(1788792487)),
			("invalid", None),
		] {
			let row = serde_json::json!([0, {"number": [0, 1], "published_at": [0, date], "is_premium": [0, false]}]);
			assert_eq!(
				chapter_from_astro(&row, "series", false, 0)
					.unwrap()
					.date_uploaded,
				expected
			);
		}
	}

	#[aidoku_test]
	fn genuine_timed_lock_has_refresh_countdown() {
		let fixture: Value = serde_json::from_str(include_str!(
			"../tests/fixtures/public-chapter-metadata.json"
		))
		.unwrap();
		let row = &fixture["sources"][1]["chapters"][0];
		// Fixed time, exactly six hours before the genuine published unlock time.
		let chapter = chapter_from_astro(row, "limitless-predation", false, 1788792487).unwrap();
		assert!(chapter.locked);
		assert_eq!(
			chapter.title.as_deref(),
			Some("Unlocks in 6h 1m (at refresh); release 2026-09-07 20:48:07 UTC")
		);
	}
}
