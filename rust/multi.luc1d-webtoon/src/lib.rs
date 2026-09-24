#![no_std]
use aidoku::{
	Chapter, DeepLinkHandler, DeepLinkResult, DynamicFilters, Filter, FilterValue,
	ImageRequestProvider, Manga, MangaPageResult, MangaStatus, Page, PageContent, PageContext,
	Result, SelectFilter, SortFilter, SortFilterDefault, Source, Viewer,
	alloc::{String, Vec, borrow::Cow, vec},
	imports::{
		defaults::defaults_get,
		html::Document,
		net::{Request, TimeUnit, set_rate_limit},
	},
	prelude::*,
};
use serde_json::Value;
const BASE: &str = "https://m.webtoons.com";
struct Webtoon;
fn locale_for_language_code(code: &str) -> &'static str {
	match code {
		"zh" => "zh-hant",
		"th" => "th",
		"id" => "id",
		"es" => "es",
		"fr" => "fr",
		"de" => "de",
		_ => "en",
	}
}
fn selected_language() -> &'static str {
	defaults_get::<String>("language")
		.map(|language| locale_for_language_code(&language))
		.unwrap_or("en")
}
const GENRES: &[(&str, &str)] = &[
	("drama", "Drama"),
	("fantasy", "Fantasy"),
	("comedy", "Comedy"),
	("action", "Action"),
	("slice_of_life", "Slice of Life"),
	("romance", "Romance"),
	("super_hero", "Superhero"),
	("sf", "Sci-Fi"),
	("thriller", "Thriller"),
	("supernatural", "Supernatural"),
	("mystery", "Mystery"),
	("sports", "Sports"),
	("historical", "Historical"),
	("heartwarming", "Heartwarming"),
	("horror", "Horror"),
	("graphic_novel", "Graphic Novel"),
	("tiptoon", "Informative"),
];
const SORTS: &[(&str, &str)] = &[
	("MANA", "Popularity"),
	("LIKEIT", "Likes"),
	("UPDATE", "Date"),
];
const SEARCH_SCOPES: &[(&str, &str)] = &[
	("all", "All results"),
	("originals", "WEBTOON Originals"),
	("canvas", "CANVAS"),
];
fn discovery_path(id: &str) -> Option<String> {
	discovery_path_for_language(id, selected_language())
}
fn discovery_path_for_language(id: &str, language: &str) -> Option<String> {
	let (genre, sort) = match id {
		"popular" => ("drama", "MANA"),
		"likes" => ("drama", "LIKEIT"),
		"date" => ("drama", "UPDATE"),
		_ => {
			let slug = id.strip_prefix("genre-")?;
			if !GENRES.iter().any(|(known, _)| *known == slug) {
				return None;
			}
			(slug, "MANA")
		}
	};
	Some(format!("/{language}/genres/{genre}?sortOrder={sort}"))
}
fn parameter<'a>(path: &'a str, name: &str) -> Option<&'a str> {
	path.split_once('?')?
		.1
		.split('&')
		.filter_map(|p| p.split_once('='))
		.find(|(k, _)| *k == name)
		.map(|(_, v)| v)
}
fn official_path(url: &str) -> Option<String> {
	let path = url
		.strip_prefix("https://www.webtoons.com")
		.or_else(|| url.strip_prefix(BASE))
		.unwrap_or(url);
	if !path.starts_with('/')
		|| path.starts_with("//")
		|| path.contains('\\')
		|| path.contains("..")
	{
		return None;
	}
	let id = parameter(path, "title_no")?;
	if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
		return None;
	}
	Some(path.into())
}
fn encode_query(query: &str) -> String {
	query
		.bytes()
		.map(|b| {
			if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
				format!("{}", b as char)
			} else {
				format!("%{b:02X}")
			}
		})
		.collect()
}
fn request(path: &str) -> Result<Request> {
	let url = if path.starts_with("/api/") {
		format!("{BASE}{path}")
	} else {
		format!(
			"https://www.webtoons.com{path}{}webtoon-platform-redirect=true",
			if path.contains('?') { "&" } else { "?" }
		)
	};
	Ok(Request::get(url)?.header("Referer", "https://m.webtoons.com/"))
}
fn parse_search(html: &Document) -> MangaPageResult {
	let mut entries: Vec<Manga> = Vec::new();
	if let Some(links) = html.select("a[href*=\"/list?title_no=\"]") {
		for el in links {
			let Some(key) = el.attr("href").and_then(|u| official_path(&u)) else {
				continue;
			};
			let Some(title) = el.select_first(".title, .subj").and_then(|e| e.text()) else {
				continue;
			};
			if title.trim().is_empty() || entries.iter().any(|m| m.key == key) {
				continue;
			}
			entries.push(Manga {
				url: Some(format!("{BASE}{key}")),
				key,
				title,
				cover: el
					.select_first("img")
					.and_then(|e| e.attr("data-src").or_else(|| e.attr("src"))),
				viewer: Viewer::Webtoon,
				..Default::default()
			});
		}
	}
	MangaPageResult {
		entries,
		has_next_page: false,
	}
}
fn parse_details(mut m: Manga, h: &Document) -> Result<Manga> {
	let text = |s: &str| h.select_first(s).and_then(|e| e.text());
	m.title = text("h1.subj, .detail_header .subj")
		.ok_or_else(|| error!("WEBTOON details unavailable or restricted"))?;
	m.cover = h
		.select_first("meta[property='og:image']")
		.and_then(|e| e.attr("content"));
	m.description = text("#_asideDetail .summary, .summary");
	m.authors = text(".detail_header .author_area").map(|s| {
		s.replace("author info", "")
			.split(',')
			.map(|s| String::from(s.trim()))
			.filter(|s| !s.is_empty())
			.collect()
	});
	m.tags = text(".detail_header .genre").map(|s| vec![s]);
	let status = text(".day_info").unwrap_or_default().to_lowercase();
	m.status = if status.contains("completed") {
		MangaStatus::Completed
	} else if status.contains("every") {
		MangaStatus::Ongoing
	} else {
		MangaStatus::Unknown
	};
	m.viewer = Viewer::Webtoon;
	m.url = Some(format!("{BASE}{}", m.key));
	Ok(m)
}
fn parse_episodes(v: Value) -> Result<(Vec<Chapter>, Option<u64>)> {
	let result = v
		.get("result")
		.ok_or_else(|| error!("WEBTOON episode response missing result"))?;
	let list = result
		.get("episodeList")
		.and_then(Value::as_array)
		.ok_or_else(|| error!("WEBTOON episode list unavailable"))?;
	let mut chapters = Vec::new();
	for e in list {
		let path = e
			.get("viewerLink")
			.and_then(Value::as_str)
			.and_then(official_path)
			.ok_or_else(|| error!("Invalid episode URL"))?;
		chapters.push(Chapter {
			key: path.clone(),
			url: Some(format!("{BASE}{path}")),
			title: e
				.get("episodeTitle")
				.and_then(Value::as_str)
				.map(String::from),
			chapter_number: e.get("episodeNo").and_then(Value::as_u64).map(|n| n as f32),
			date_uploaded: e
				.get("exposureDateMillis")
				.and_then(Value::as_i64)
				.map(|n| n / 1000),
			..Default::default()
		});
	}
	let next = result
		.get("nextCursor")
		.and_then(Value::as_u64)
		.filter(|n| *n > 0);
	Ok((chapters, next))
}
fn parse_pages(h: &Document) -> Result<Vec<Page>> {
	let pages: Vec<Page> = h
		.select("#_imageList img")
		.map(|els| {
			els.filter_map(|el| {
				let url = el.attr("data-url")?;
				if !url.starts_with("https://") {
					return None;
				}
				Some(Page {
					content: PageContent::url(url),
					..Default::default()
				})
			})
			.collect()
		})
		.unwrap_or_default();
	if pages.is_empty() {
		return Err(error!(
			"No public WEBTOON pages: episode may require the app, login, or payment"
		));
	}
	Ok(pages)
}
impl Source for Webtoon {
	fn new() -> Self {
		set_rate_limit(2, 1, TimeUnit::Seconds);
		Self
	}
	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		if page < 1 {
			return Err(error!("Invalid page"));
		}
		let scope = search_scope(&filters)?;
		if let Some(query) = query.filter(|q| !q.trim().is_empty()) {
			if scope == "all" && page > 1 {
				return Ok(MangaPageResult::default());
			}
			let path = search_path_for_language(&query, scope, page, selected_language())?;
			let html = request(&path)?.html()?;
			let mut result = parse_search(&html);
			if scope != "all" {
				result.has_next_page = has_next_search_page(&html, page);
			}
			return Ok(result);
		}
		if page > 1 {
			return Ok(MangaPageResult::default());
		}
		if scope != "all" {
			return Err(error!(
				"Enter a text query to search a specific WEBTOON catalog"
			));
		}
		Ok(parse_search(
			&request(&filtered_discovery_path(&filters)?)?.html()?,
		))
	}
	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let path = official_path(&manga.key).ok_or_else(|| error!("Invalid WEBTOON key"))?;
		if needs_details {
			manga = parse_details(manga, &request(&path)?.html()?)?;
		}
		if needs_chapters {
			let id = parameter(&path, "title_no").ok_or_else(|| error!("Missing title number"))?;
			let kind = if path.contains("/canvas/") {
				"canvas"
			} else {
				"webtoon"
			};
			let mut chapters: Vec<Chapter> = Vec::new();
			let mut cursor = 0;
			for batch in 0..100 {
				let value: Value = request(&format!(
					"/api/v1/{kind}/{id}/episodes?pageSize=100&cursor={cursor}"
				))?
				.json_owned()?;
				let (entries, next) = parse_episodes(value)?;
				for chapter in entries {
					if !chapters.iter().any(|c| c.key == chapter.key) {
						chapters.push(chapter);
					}
				}
				match next {
					None => break,
					Some(n) if n > cursor => cursor = n,
					_ => return Err(error!("WEBTOON repeated pagination cursor")),
				}
				if batch == 99 {
					return Err(error!("WEBTOON episode pagination limit exceeded"));
				}
			}
			chapters.reverse();
			manga.chapters = Some(chapters);
		}
		Ok(manga)
	}
	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let path =
			official_path(&chapter.key).ok_or_else(|| error!("Invalid WEBTOON chapter URL"))?;
		if parameter(&path, "title_no") != parameter(&manga.key, "title_no") {
			return Err(error!("Chapter does not belong to this series"));
		}
		parse_pages(&request(&path)?.html()?)
	}
}
impl ImageRequestProvider for Webtoon {
	fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
		Ok(Request::get(url)?.header("Referer", "https://m.webtoons.com/"))
	}
}
fn discovery_component(id: &str, title: &str, entries: Vec<Manga>) -> aidoku::HomeComponent {
	aidoku::HomeComponent {
		title: Some(title.into()),
		subtitle: Some("English Drama genre; site order (not a daily chart)".into()),
		value: aidoku::HomeComponentValue::Scroller {
			entries: entries.into_iter().take(20).map(Into::into).collect(),
			listing: Some(aidoku::Listing {
				id: id.into(),
				name: title.into(),
				..Default::default()
			}),
		},
	}
}
impl DynamicFilters for Webtoon {
	fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
		let genres = GENRES
			.iter()
			.map(|(_, name)| Cow::Borrowed(*name))
			.collect();
		let genre_ids = GENRES
			.iter()
			.map(|(slug, _)| Cow::Borrowed(*slug))
			.collect();
		let sorts = SORTS
			.iter()
			.map(|(_, label)| Cow::Borrowed(*label))
			.collect();
		let scopes = SEARCH_SCOPES
			.iter()
			.map(|(_, label)| Cow::Borrowed(*label))
			.collect();
		let scope_ids = SEARCH_SCOPES
			.iter()
			.map(|(id, _)| Cow::Borrowed(*id))
			.collect();
		Ok(vec![
			SelectFilter {
				id: Cow::Borrowed("genre"),
				title: Some(Cow::Borrowed("Genre")),
				is_genre: true,
				options: genres,
				ids: Some(genre_ids),
				..Default::default()
			}
			.into(),
			SortFilter {
				id: Cow::Borrowed("sort"),
				title: Some(Cow::Borrowed("Sort by")),
				can_ascend: false,
				options: sorts,
				default: Some(SortFilterDefault {
					index: 0,
					ascending: false,
				}),
				..Default::default()
			}
			.into(),
			SelectFilter {
				id: Cow::Borrowed("scope"),
				title: Some(Cow::Borrowed("Search in")),
				options: scopes,
				ids: Some(scope_ids),
				..Default::default()
			}
			.into(),
		])
	}
}
impl aidoku::ListingProvider for Webtoon {
	fn get_manga_list(&self, listing: aidoku::Listing, page: i32) -> Result<MangaPageResult> {
		let path = discovery_path(&listing.id).ok_or_else(|| error!("Unknown WEBTOON listing"))?;
		if page < 1 {
			return Err(error!("Invalid page"));
		}
		if page > 1 {
			return Ok(MangaPageResult::default());
		}
		let result = parse_search(&request(&path)?.html()?);
		if result.entries.is_empty() {
			return Err(error!("WEBTOON discovery unavailable"));
		}
		Ok(result)
	}
}
impl aidoku::Home for Webtoon {
	fn get_home(&self) -> Result<aidoku::HomeLayout> {
		use aidoku::ListingProvider;
		let mut components = Vec::new();
		for (id, title) in [
			("popular", "Drama: By Popularity"),
			("likes", "Drama: By Likes"),
			("date", "Drama: By Date"),
		] {
			let result = self.get_manga_list(
				aidoku::Listing {
					id: id.into(),
					..Default::default()
				},
				1,
			)?;
			components.push(discovery_component(id, title, result.entries));
		}
		components.push(aidoku::HomeComponent {
			title: Some("Browse Genres".into()),
			subtitle: Some("By Popularity within each genre".into()),
			value: aidoku::HomeComponentValue::Links(
				GENRES
					.iter()
					.map(|(slug, name)| {
						let id = if *slug == "drama" {
							"popular".into()
						} else {
							format!("genre-{slug}")
						};
						aidoku::Link {
							title: (*name).into(),
							value: Some(aidoku::LinkValue::Listing(aidoku::Listing {
								id,
								name: (*name).into(),
								..Default::default()
							})),
							..Default::default()
						}
					})
					.collect(),
			),
		});
		Ok(aidoku::HomeLayout { components })
	}
}
fn filtered_discovery_path(filters: &[FilterValue]) -> Result<String> {
	filtered_discovery_path_for_language(filters, selected_language())
}
fn filtered_discovery_path_for_language(filters: &[FilterValue], language: &str) -> Result<String> {
	let mut genre = "drama";
	let mut sort = "MANA";
	for filter in filters {
		match filter {
			FilterValue::Select { id, value } if id == "genre" => {
				if !GENRES.iter().any(|(slug, _)| *slug == value.as_str()) {
					return Err(error!("Unknown WEBTOON genre"));
				}
				genre = value.as_str();
			}
			FilterValue::Sort { id, index, .. } if id == "sort" => {
				if *index < 0 {
					return Err(error!("Unknown WEBTOON sort order"));
				}
				sort = SORTS
					.get(*index as usize)
					.map(|(key, _)| *key)
					.ok_or_else(|| error!("Unknown WEBTOON sort order"))?;
			}
			_ => {}
		}
	}
	Ok(format!("/{language}/genres/{genre}?sortOrder={sort}"))
}
fn search_scope(filters: &[FilterValue]) -> Result<&'static str> {
	let Some(value) = filters.iter().find_map(|filter| match filter {
		FilterValue::Select { id, value } if id == "scope" => Some(value.as_str()),
		_ => None,
	}) else {
		return Ok("all");
	};
	SEARCH_SCOPES
		.iter()
		.find(|(id, _)| *id == value)
		.map(|(id, _)| *id)
		.ok_or_else(|| error!("Unknown WEBTOON search scope"))
}
#[cfg(test)]
fn search_path(query: &str, scope: &str, page: i32) -> Result<String> {
	search_path_for_language(query, scope, page, "en")
}
fn search_path_for_language(query: &str, scope: &str, page: i32, language: &str) -> Result<String> {
	if page < 1 {
		return Err(error!("Invalid page"));
	}
	let query = encode_query(query);
	match scope {
		"all" if page == 1 => Ok(format!("/{language}/search?keyword={query}")),
		"all" => Err(error!("All-results preview is not paginated")),
		"originals" | "canvas" => Ok(format!(
			"/{language}/search/{scope}?keyword={query}&page={page}"
		)),
		_ => Err(error!("Unknown WEBTOON search scope")),
	}
}
fn has_next_search_page(html: &Document, page: i32) -> bool {
	html.select(".list_pagination a.pagination[href*='page=']")
		.map(|links| {
			links.into_iter().any(|link| {
				link.attr("href")
					.and_then(|href| {
						parameter(&href, "page").and_then(|value| value.parse::<i32>().ok())
					})
					.is_some_and(|next| next > page)
			})
		})
		.unwrap_or(false)
}
impl DeepLinkHandler for Webtoon {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		Ok(official_path(&url).map(|key| DeepLinkResult::Manga { key }))
	}
}
aidoku::register_source!(
	Webtoon,
	ImageRequestProvider,
	ListingProvider,
	Home,
	DeepLinkHandler,
	DynamicFilters
);
#[cfg(test)]
mod tests;
