use aidoku::alloc::String;
use aidoku::{Page, PageContent, PageContext, alloc::Vec};
use core::cell::RefCell;

#[derive(Clone)]
pub struct CachedPage {
	url: String,
	context: Option<PageContext>,
	thumbnail: Option<String>,
	has_description: bool,
	description: Option<String>,
}

impl Cache<Vec<CachedPage>> {
	pub fn remember_pages(&self, key: String, now: i64, pages: &[Page]) {
		let snapshots: Option<Vec<_>> = pages
			.iter()
			.map(|page| {
				if let PageContent::Url(url, context) = &page.content {
					Some(CachedPage {
						url: url.clone(),
						context: context.clone(),
						thumbnail: page.thumbnail.clone(),
						has_description: page.has_description,
						description: page.description.clone(),
					})
				} else {
					None
				}
			})
			.collect();
		if let Some(snapshots) = snapshots.filter(|pages| !pages.is_empty()) {
			self.put(key, now, snapshots);
		}
	}

	pub fn pages(&self, key: &str, now: i64) -> Option<Vec<Page>> {
		Some(
			self.get(key, now, 300)?
				.into_iter()
				.map(|page| Page {
					content: PageContent::Url(page.url, page.context),
					thumbnail: page.thumbnail,
					has_description: page.has_description,
					description: page.description,
				})
				.collect(),
		)
	}
}

/// One bounded, source-instance-local entry. Times are Unix seconds.
pub struct Cache<T> {
	entry: RefCell<Option<(String, i64, T)>>,
}

impl<T> Default for Cache<T> {
	fn default() -> Self {
		Self {
			entry: RefCell::new(None),
		}
	}
}

impl<T: Clone> Cache<T> {
	pub fn get(&self, key: &str, now: i64, ttl: i64) -> Option<T> {
		let entry = self.entry.borrow();
		let (stored_key, stored_at, value) = entry.as_ref()?;
		if stored_key == key && now >= *stored_at && now - stored_at < ttl {
			Some(value.clone())
		} else {
			None
		}
	}

	pub fn put(&self, key: String, now: i64, value: T) {
		self.entry.replace(Some((key, now, value)));
	}
}

#[cfg(test)]
#[aidoku_test::aidoku_test]
fn cache_is_scoped_expires_and_replaces_its_only_entry() {
	let cache = Cache::default();
	cache.put("gallery/1".into(), 100, "first");
	assert_eq!(cache.get("gallery/1", 159, 60), Some("first"));
	assert_eq!(cache.get("gallery/2", 100, 60), None);
	assert_eq!(cache.get("gallery/1", 160, 60), None);
	assert_eq!(cache.get("gallery/1", 99, 60), None);
	cache.put("gallery/2".into(), 161, "second");
	assert_eq!(cache.get("gallery/1", 161, 60), None);
	assert_eq!(cache.get("gallery/2", 161, 60), Some("second"));
}
