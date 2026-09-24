#![no_std]
use aidoku::{
	Chapter, ContentRating, DynamicFilters, Filter, FilterValue, ImageRequestProvider, Manga,
	MangaPageResult, MangaStatus, MultiSelectFilter, Page, PageContent, Result, SortFilter, Source,
	Viewer,
	alloc::{String, Vec, string::ToString, vec},
	imports::{
		html::{Document, Element},
		net::{Request, TimeUnit, set_rate_limit},
	},
	prelude::*,
};
macro_rules! ensure {
	($condition:expr, $message:expr) => {
		if !$condition {
			bail!($message)
		}
	};
}
const BASE_URL: &str = "https://imhentai.xxx";
const IS_IM: bool = true;
fn key_from_url(url: &str) -> Option<String> {
	let path = url.strip_prefix(BASE_URL).unwrap_or(url);
	let key = path.strip_prefix("/gallery/")?.trim_end_matches('/');
	if !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit()) {
		Some(key.into())
	} else {
		None
	}
}
fn image(el: &Element) -> Option<String> {
	["abs:data-src", "abs:src"]
		.iter()
		.find_map(|a| el.attr(a).filter(|v| v.starts_with("https://")))
}
fn parse_search(doc: &Document) -> MangaPageResult {
	let entries = doc
		.select("div.thumb")
		.map(|els| {
			els.filter_map(|el| {
				let key = key_from_url(&el.select_first(".inner_thumb a")?.attr("href")?)?;
				let title = el.select_first(".caption")?.text()?;
				if title.trim().is_empty() {
					return None;
				}
				Some(Manga {
					key,
					title,
					cover: el.select_first(".inner_thumb img").and_then(|e| image(&e)),
					content_rating: ContentRating::NSFW,
					status: MangaStatus::Completed,
					viewer: Viewer::RightToLeft,
					..Default::default()
				})
			})
			.collect()
		})
		.unwrap_or_default();
	MangaPageResult {
		entries,
		has_next_page: doc
			.select_first(".pagination li.active + li:not(.disabled) a, .pagination a[rel=next]")
			.is_some(),
	}
}
fn search_url(query: Option<&str>, page: i32) -> Result<String> {
	search_url_with_filters(query, page, &[])
}
fn search_url_with_filters(query: Option<&str>, page: i32, filters: &[FilterValue]) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	let query = query.unwrap_or("").trim();
	let mut sort = 1usize; // Latest matches the source's unfiltered search order.
	let mut categories: Option<Vec<String>> = None;
	let mut languages: Option<Vec<String>> = None;
	for filter in filters {
		match filter {
			FilterValue::Sort { id, index, .. } if id.as_str() == "sort" => {
				sort = usize::try_from(*index).unwrap_or(1).min(3);
			}
			FilterValue::MultiSelect { id, included, .. } if id.as_str() == "categories" => {
				categories = Some(included.clone());
			}
			FilterValue::MultiSelect { id, included, .. } if id.as_str() == "languages" => {
				languages = Some(included.clone());
			}
			_ => {}
		}
	}
	if query.is_empty() && filters.is_empty() {
		return Ok(if IS_IM {
			format!("{BASE_URL}/?page={page}")
		} else if page == 1 {
			format!("{BASE_URL}/")
		} else {
			format!("{BASE_URL}/page/{page}/")
		});
	}
	let mut escaped = String::new();
	for b in query.bytes() {
		if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
			escaped.push(b as char)
		} else {
			escaped.push_str(&format!("%{b:02X}"));
		}
	}
	Ok(if IS_IM {
		let mut flags = String::new();
		for (index, name) in ["pp", "lt", "dl", "tr"].iter().enumerate() {
			if !flags.is_empty() {
				flags.push('&');
			}
			flags.push_str(name);
			flags.push('=');
			flags.push(if index == sort { '1' } else { '0' });
		}
		for (id, default) in [("m", true), ("d", true), ("w", true), ("i", true), ("a", true), ("g", true)] {
			let enabled = categories
				.as_ref()
				.map_or(default, |values| values.iter().any(|value| value.as_str() == id));
			flags.push('&');
			flags.push_str(id);
			flags.push('=');
			flags.push(if enabled { '1' } else { '0' });
		}
		for id in ["en", "jp", "es", "fr", "kr", "de", "ru"] {
			let enabled = languages
				.as_ref()
				.map_or(true, |values| values.iter().any(|value| value.as_str() == id));
			flags.push('&');
			flags.push_str(id);
			flags.push('=');
			flags.push(if enabled { '1' } else { '0' });
		}
		format!("{BASE_URL}/search/?{flags}&key={escaped}&page={page}")
	} else {
		format!("{BASE_URL}/search/?q={escaped}&page={page}")
	})
}

fn discovery_filters() -> Vec<Filter> {
	let mut sort = SortFilter::default();
	sort.id = "sort".into();
	sort.title = Some("Sort".into());
	sort.can_ascend = false;
	sort.options = vec!["Popular".into(), "Latest".into(), "Downloads".into(), "Top Rated".into()];
	sort.default = Some(aidoku::SortFilterDefault { index: 1, ascending: false });
	let mut categories = MultiSelectFilter::default();
	categories.id = "categories".into();
	categories.title = Some("Categories".into());
	categories.options = vec!["Manga".into(), "Doujinshi".into(), "Western".into(), "Image Set".into(), "Artist CG".into(), "Game CG".into()];
	categories.ids = Some(vec!["m".into(), "d".into(), "w".into(), "i".into(), "a".into(), "g".into()]);
	categories.default_included = Some(vec!["m".into(), "d".into(), "w".into(), "i".into(), "a".into(), "g".into()]);
	let mut languages = MultiSelectFilter::default();
	languages.id = "languages".into();
	languages.title = Some("Languages".into());
	languages.options = vec!["English".into(), "Japanese".into(), "Spanish".into(), "French".into(), "Korean".into(), "German".into(), "Russian".into()];
	languages.ids = Some(vec!["en".into(), "jp".into(), "es".into(), "fr".into(), "kr".into(), "de".into(), "ru".into()]);
	languages.default_included = Some(vec!["en".into(), "jp".into(), "es".into(), "fr".into(), "kr".into(), "de".into(), "ru".into()]);
	vec![sort.into(), categories.into(), languages.into()]
}
fn update(doc: &Document, mut manga: Manga, details: bool, chapters: bool) -> Result<Manga> {
	ensure!(
		!manga.key.is_empty() && manga.key.bytes().all(|b| b.is_ascii_digit()),
		"Invalid gallery key"
	);
	if details || chapters {
		ensure!(
			doc.select_first(".gallery_top h1, .gallery_first h1")
				.is_some(),
			"Gallery unavailable or site layout changed"
		);
	}
	if details {
		manga.title = doc
			.select_first(".gallery_top h1, .gallery_first h1")
			.and_then(|e| e.text())
			.filter(|s| !s.trim().is_empty())
			.ok_or(error!("Missing title"))?;
		manga.cover = doc
			.select_first(if IS_IM {
				".left_cover img"
			} else {
				".cover img"
			})
			.and_then(|e| image(&e));
		let names = |selector: &str| -> Vec<String> {
			doc.select(selector)
				.map(|els| {
					els.filter_map(|e| e.own_text())
						.filter(|s| !s.trim().is_empty())
						.collect()
				})
				.unwrap_or_default()
		};
		manga.authors = Some(if IS_IM {
			doc.select("li")
				.map(|els| {
					els.filter(|e| {
						e.select_first(".tags_text")
							.and_then(|v| v.text())
							.is_some_and(|v| v.trim() == "Artists:")
					})
					.flat_map(|e| {
						e.select("a.tag")
							.map(|els| els.filter_map(|a| a.own_text()).collect::<Vec<_>>())
							.unwrap_or_default()
					})
					.collect()
				})
				.unwrap_or_default()
		} else {
			names("ul.artists a")
		});
		manga.tags = Some(if IS_IM {
			doc.select("li")
				.map(|els| {
					els.filter(|e| {
						e.select_first(".tags_text")
							.and_then(|v| v.text())
							.is_some_and(|v| v.trim() == "Tags:")
					})
					.flat_map(|e| {
						e.select("a.tag")
							.map(|els| els.filter_map(|a| a.own_text()).collect::<Vec<_>>())
							.unwrap_or_default()
					})
					.collect()
				})
				.unwrap_or_default()
		} else {
			names("ul.tags a")
		});
		manga.status = MangaStatus::Completed;
		manga.content_rating = ContentRating::NSFW;
		manga.viewer = Viewer::RightToLeft;
		manga.url = Some(format!("{BASE_URL}/gallery/{}/", manga.key));
	}
	if chapters {
		manga.chapters = Some(vec![Chapter {
			key: manga.key.clone(),
			title: Some("Gallery".into()),
			chapter_number: Some(1.0),
			url: Some(format!("{BASE_URL}/gallery/{}/", manga.key)),
			..Default::default()
		}]);
	}
	Ok(manga)
}
fn input(doc: &Document, id: &str) -> Result<String> {
	doc.select_first(&format!("input#{id}"))
		.and_then(|e| e.attr("value"))
		.filter(|s| !s.is_empty())
		.ok_or(error!("Missing reader metadata"))
}
fn parse_pages(doc: &Document) -> Result<Vec<Page>> {
	let id = input(doc, "load_id")?;
	let dir = input(doc, "load_dir")?;
	ensure!(
		id.bytes().all(|b| b.is_ascii_digit())
			&& dir.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'),
		"Invalid image path"
	);
	let host = if let Ok(server) = input(doc, "load_server") {
		ensure!(server.bytes().all(|b| b.is_ascii_digit()), "Invalid server");
		format!("m{server}.{}", BASE_URL.trim_start_matches("https://"))
	} else {
		let cover = doc
			.select_first(".cover img, .left_cover img")
			.and_then(|e| image(&e))
			.ok_or(error!("Missing image server"))?;
		let host = cover
			.strip_prefix("https://")
			.and_then(|v| v.split('/').next())
			.ok_or(error!("Invalid image server"))?;
		let domain = BASE_URL.trim_start_matches("https://");
		ensure!(
			host == domain || host.ends_with(&format!(".{domain}")),
			"Unexpected image server"
		);
		host.to_string()
	};
	let script = doc
		.select("script")
		.and_then(|els| {
			els.filter_map(|e| e.html())
				.find(|s| s.contains("$.parseJSON('"))
		})
		.ok_or(error!(
			"Reader manifest missing; thumbnail guessing is unsupported"
		))?;
	let raw = script
		.split("$.parseJSON('")
		.nth(1)
		.and_then(|s| s.split("');").next())
		.ok_or(error!("Invalid reader manifest"))?;
	let map: serde_json::Value =
		serde_json::from_str(raw).map_err(|_| error!("Invalid reader JSON"))?;
	let map = map.as_object().ok_or(error!("Invalid reader map"))?;
	let count = input(doc, "load_pages")?
		.parse::<usize>()
		.map_err(|_| error!("Invalid page count"))?;
	ensure!(
		count > 0 && count <= 10000 && map.len() == count,
		"Incomplete reader manifest"
	);
	let mut pages = Vec::with_capacity(count);
	for n in 1..=count {
		let value = map
			.get(&n.to_string())
			.and_then(|v| v.as_str())
			.ok_or(error!("Noncontiguous page manifest"))?;
		let ext = match value.split(',').next().unwrap_or("") {
			"j" => "jpg",
			"p" => "png",
			"w" => "webp",
			"g" => "gif",
			"b" => "bmp",
			_ => bail!("Unknown page format"),
		};
		pages.push(Page {
			content: PageContent::url(format!("https://{host}/{dir}/{id}/{n}.{ext}")),
			..Default::default()
		});
	}
	Ok(pages)
}
fn listing_url(id: &str, page: i32) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	match id {
		"latest" => search_url(None, page),

		_ => bail!("Unsupported listing"),
	}
}
fn latest_component(doc: &Document) -> Result<aidoku::HomeComponent> {
	let result = parse_search(doc);
	ensure!(
		!result.entries.is_empty(),
		"Latest unavailable or site layout changed"
	);
	Ok(aidoku::HomeComponent {
		title: Some("Latest".into()),
		value: aidoku::HomeComponentValue::Scroller {
			entries: result.entries.into_iter().map(Into::into).collect(),
			listing: Some(aidoku::Listing {
				id: "latest".into(),
				name: "Latest".into(),
				..Default::default()
			}),
		},
		..Default::default()
	})
}
fn parse_home(doc: &Document) -> Result<aidoku::HomeLayout> {
	let components = vec![latest_component(doc)?];

	Ok(aidoku::HomeLayout { components })
}
impl aidoku::Home for GallerySource {
	fn get_home(&self) -> Result<aidoku::HomeLayout> {
		parse_home(&Request::get(listing_url("latest", 1)?)?.html()?)
	}
}
impl DynamicFilters for GallerySource {
	fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
		Ok(discovery_filters())
	}
}
impl aidoku::ListingProvider for GallerySource {
	fn get_manga_list(&self, listing: aidoku::Listing, page: i32) -> Result<MangaPageResult> {
		let url = listing_url(&listing.id, page)?;

		let doc = Request::get(url)?.html()?;
		let result = parse_search(&doc);
		ensure!(
			!result.entries.is_empty(),
			"Listing unavailable or site layout changed"
		);
		Ok(result)
	}
}

struct GallerySource;
impl Source for GallerySource {
	fn new() -> Self {
		set_rate_limit(1, 1, TimeUnit::Seconds);
		Self
	}
	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let doc = Request::get(search_url_with_filters(query.as_deref(), page, &filters)?)?.html()?;
		ensure!(
			doc.select_first("div.thumb, .pagination, .content, .container")
				.is_some(),
			"Search unavailable or layout changed"
		);
		Ok(parse_search(&doc))
	}
	fn get_manga_update(
		&self,
		manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if !needs_details && !needs_chapters {
			return Ok(manga);
		}
		ensure!(
			!manga.key.is_empty() && manga.key.bytes().all(|b| b.is_ascii_digit()),
			"Invalid gallery key"
		);
		let doc = Request::get(format!("{BASE_URL}/gallery/{}/", manga.key))?.html()?;
		update(&doc, manga, needs_details, needs_chapters)
	}
	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		ensure!(
			chapter.key == manga.key
				&& !chapter.key.is_empty()
				&& chapter.key.bytes().all(|b| b.is_ascii_digit()),
			"Invalid gallery key"
		);
		parse_pages(&Request::get(format!("{BASE_URL}/gallery/{}/", chapter.key))?.html()?)
	}
}
impl ImageRequestProvider for GallerySource {
	fn get_image_request(
		&self,
		url: String,
		_context: Option<aidoku::PageContext>,
	) -> Result<Request> {
		Ok(Request::get(url)?.header("Referer", &format!("{BASE_URL}/")))
	}
}
aidoku::register_source!(GallerySource, ImageRequestProvider, Home, ListingProvider, DynamicFilters);

#[cfg(test)]
mod tests;
