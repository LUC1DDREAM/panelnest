#![no_std]
use aidoku::{
	AlternateCoverProvider, Chapter, DeepLinkHandler, DeepLinkResult, DynamicFilters, Filter,
	FilterValue, Listing, ListingProvider, Manga, MangaPageResult, MultiSelectFilter, Page,
	PageContent, PageDescriptionProvider, Result, Source,
	alloc::{String, Vec, borrow::Cow, string::ToString, vec},
	helpers::uri::encode_uri_component,
	imports::{
		error::AidokuError,
		net::{Request, TimeUnit, set_rate_limit},
	},
	prelude::*,
};

mod home;
mod models;
mod settings;
#[cfg(test)]
mod tests;

use core::cell::RefCell;

use models::*;

const BASE_URL: &str = "https://nhentai.net";
const API_URL: &str = "https://nhentai.net/api/v2";
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) \
						  AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 \
						  Mobile/15E148 Safari/604";

struct NHentai {
	cache: RefCell<Option<(String, NHentaiGallery)>>,
}

fn taxonomy_type(filter_id: &str) -> Option<&'static str> {
	match filter_id {
		"tags" | "genre" => Some("tag"),
		"artists" | "artist-tags" => Some("artist"),
		"groups" | "group-tags" => Some("group"),
		"languages" | "language-tags" => Some("language"),
		"parodies" | "parody-tags" => Some("parody"),
		"characters" | "character-tags" => Some("character"),
		_ => None,
	}
}

fn text_filter_query(id: &str, value: String) -> Option<String> {
	match id {
		"author" => Some(value),
		"tag" => Some(format!("tag:\"{value}\"")),
		"artist" => Some(format!("artist:{value}")),
		"groups" => Some(format!("group:{value}")),
		"parody" => Some(format!("parody:{value}")),
		"character" => Some(format!("character:{value}")),
		_ => None,
	}
}

impl Source for NHentai {
	fn new() -> Self {
		set_rate_limit(1, 1, TimeUnit::Seconds);
		Self {
			cache: RefCell::new(None),
		}
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
		// If the query is a numeric ID, return the manga directly
		if let Some(q) = &query
			&& let Ok(id) = q.parse::<i32>()
		{
			let url = format!("{API_URL}/galleries/{id}");
			let gallery: NHentaiGallery = Request::get(&url)?
				.header("User-Agent", USER_AGENT)
				.json_owned()?;
			return Ok(MangaPageResult {
				entries: vec![gallery.into()],
				has_next_page: false,
			});
		}

		let mut query_parts = Vec::new();

		if let Some(q) = query {
			query_parts.push(q);
		}

		let mut sort = "date";

		// parse filters
		for filter in filters {
			match filter {
				FilterValue::Text { id, value } => {
					if let Some(part) = text_filter_query(&id, value) {
						query_parts.push(part);
					}
				}
				FilterValue::Sort { index, .. } => {
					sort = match index {
						0 => "date",          // Latest
						1 => "popular-today", // Popular Today
						2 => "popular-week",  // Popular Week
						3 => "popular",       // Popular All
						_ => "date",
					};
				}
				FilterValue::MultiSelect {
					id,
					included,
					excluded,
					..
				} => {
					let tag_type = taxonomy_type(&id);
					if let Some(tag_type) = tag_type {
						for tag in included {
							query_parts.push(format!("{tag_type}:\"{tag}\""));
						}
						for tag in excluded {
							query_parts.push(format!("-{tag_type}:\"{tag}\""));
						}
					}
				}
				_ => continue,
			}
		}

		if let Some(language) = settings::get_language() {
			query_parts.push(format!("language:{language}"));
		}

		for blocked in settings::get_blocklist() {
			if !blocked.is_empty() {
				query_parts.push(format!("-tag:\"{blocked}\""));
			}
		}

		let combined_query = if query_parts.is_empty() {
			" ".into()
		} else {
			query_parts.join(" ")
		};

		let url = format!(
			"{API_URL}/search?query={}&page={page}&sort={sort}",
			encode_uri_component(combined_query),
		);
		let response: NHentaiSearchResponse = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.json_owned()?;

		let entries = response
			.result
			.into_iter()
			.map(|item| item.into())
			.collect::<Vec<Manga>>();
		let has_next_page = page < response.num_pages;

		Ok(MangaPageResult {
			entries,
			has_next_page,
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if needs_details || needs_chapters {
			let url = format!("{API_URL}/galleries/{}", manga.key);
			let gallery: NHentaiGallery = Request::get(&url)?
				.header("User-Agent", USER_AGENT)
				.json_owned()?;

			if needs_details {
				manga.copy_from(gallery.clone().into());
			}

			if needs_chapters {
				let mut languages = Vec::new();
				for tag in &gallery.tags {
					if tag.r#type == "language" && tag.name != "translated" && tag.name != "rewrite"
					{
						languages.push(tag.name.clone());
					}
				}

				let chapter = Chapter {
					key: manga.key.clone(),
					chapter_number: Some(1.0),
					date_uploaded: Some(gallery.upload_date),
					url: Some(format!("{}/g/{}", BASE_URL, manga.key)),
					scanlators: if !languages.is_empty() {
						Some(vec![languages.join(", ")])
					} else {
						None
					},
					..Default::default()
				};
				manga.chapters = Some(vec![chapter]);
			}

			// Cache the fetched gallery for potential reuse in get_page_list
			self.cache
				.borrow_mut()
				.replace((manga.key.clone(), gallery));
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Try to reuse cached gallery fetched by get_manga_update
		let maybe_cached = self.cache.borrow();
		let gallery: NHentaiGallery = match &*maybe_cached {
			Some((cached_key, cached_gallery)) if cached_key == &chapter.key => {
				cached_gallery.clone()
			}
			_ => {
				let api_url = format!("{API_URL}/galleries/{}", chapter.key);
				Request::get(&api_url)?
					.header("User-Agent", USER_AGENT)
					.json_owned()?
			}
		};

		let pages = gallery
			.pages
			.iter()
			.map(|page| {
				let path = make_image_url(&page.path, false);

				Page {
					content: PageContent::url(path),
					has_description: true,
					..Default::default()
				}
			})
			.collect::<Vec<Page>>();

		Ok(pages)
	}
}

impl PageDescriptionProvider for NHentai {
	fn get_page_description(&self, page: Page) -> Result<String> {
		let PageContent::Url(url, _) = page.content else {
			return Err(error!("Page does not contain an image URL"));
		};

		let filename = url.rsplit('/').next().unwrap_or_default();
		let (number, extension) = filename
			.split_once('.')
			.ok_or_else(|| error!("Invalid page image URL"))?;
		if number.is_empty()
			|| !number.bytes().all(|byte| byte.is_ascii_digit())
			|| !matches!(extension, "jpg" | "png" | "gif" | "webp")
		{
			return Err(error!("Invalid page image URL"));
		}

		Ok(format!("Page {number}"))
	}
}

impl ListingProvider for NHentai {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"popular-today" => self.get_search_manga_list(
				None,
				page,
				vec![FilterValue::Sort {
					id: "sort".to_string(),
					index: 1,
					ascending: false,
				}],
			),
			"popular-week" => self.get_search_manga_list(
				None,
				page,
				vec![FilterValue::Sort {
					id: "sort".to_string(),
					index: 2,
					ascending: false,
				}],
			),
			"popular" => self.get_search_manga_list(
				None,
				page,
				vec![FilterValue::Sort {
					id: "sort".to_string(),
					index: 3,
					ascending: false,
				}],
			),
			"latest" => self.get_search_manga_list(
				None,
				page,
				vec![FilterValue::Sort {
					id: "sort".to_string(),
					index: 0,
					ascending: false,
				}],
			),
			_ => Err(AidokuError::Unimplemented),
		}
	}
}

impl AlternateCoverProvider for NHentai {
	fn get_alternate_covers(&self, manga: Manga) -> Result<Vec<String>> {
		if manga.key.is_empty() || !manga.key.bytes().all(|byte| byte.is_ascii_digit()) {
			return Err(error!("Invalid gallery key"));
		}
		let url = format!("{API_URL}/galleries/{}", manga.key);
		let gallery: NHentaiGallery = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.json_owned()?;
		Ok(models::cover_variants(
			&manga.key,
			&gallery.media_id,
			&gallery.cover.path,
		))
	}
}

fn taxonomy_filter(
	tags: Vec<NHentaiTag>,
	tag_type: &str,
	id: &'static str,
	title: &'static str,
) -> Result<Filter> {
	let mut options: Vec<Cow<'static, str>> = Vec::new();
	let mut ids: Vec<Cow<'static, str>> = Vec::new();
	for tag in tags {
		let name = tag.name.trim();
		if tag.r#type != tag_type
			|| name.is_empty()
			|| options.iter().any(|existing| existing.as_ref() == name)
		{
			continue;
		}
		options.push(Cow::Owned(name.into()));
		ids.push(Cow::Owned(name.into()));
	}
	if options.is_empty() {
		return Err(error!("nhentai taxonomy directory unavailable"));
	}
	let mut filter = MultiSelectFilter::default();
	filter.id = Cow::Borrowed(id);
	filter.title = Some(Cow::Borrowed(title));
	filter.options = options;
	filter.ids = Some(ids);
	Ok(filter.into())
}

impl DynamicFilters for NHentai {
	fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
		let first: NHentaiTagsResponse = Request::get(format!(
			"{API_URL}/tags/language?sort=popular&page=1&per_page=100"
		))?
		.header("User-Agent", USER_AGENT)
		.json_owned()?;
		if !(1..=20).contains(&first.num_pages) {
			return Err(error!("Invalid nhentai language directory page count"));
		}
		let mut tags = first.result;
		for page in 2..=first.num_pages {
			let response: NHentaiTagsResponse = Request::get(format!(
				"{API_URL}/tags/language?sort=popular&page={page}&per_page=100"
			))?
			.header("User-Agent", USER_AGENT)
			.json_owned()?;
			if response.num_pages != first.num_pages || response.result.is_empty() {
				return Err(error!("Incomplete nhentai language directory"));
			}
			tags.extend(response.result);
		}
		let mut filters = vec![taxonomy_filter(tags, "language", "languages", "Language")?];
		for (tag_type, id, title) in [
			("artist", "artists", "Popular Artists"),
			("group", "group-tags", "Popular Groups"),
			("parody", "parodies", "Popular Parodies"),
			("character", "characters", "Popular Characters"),
		] {
			let response: Option<NHentaiTagsResponse> = Request::get(format!(
				"{API_URL}/tags/{tag_type}?sort=popular&page=1&per_page=100"
			))
			.ok()
			.map(|request| request.header("User-Agent", USER_AGENT))
			.and_then(|request| request.json_owned().ok());
			if let Some(response) = response
				&& response.num_pages > 0
				&& let Ok(filter) = taxonomy_filter(response.result, tag_type, id, title)
			{
				filters.push(filter);
			}
		}
		for (id, title, placeholder) in [
			("parody", "Parody", "Parody name"),
			("character", "Character", "Character name"),
		] {
			let mut filter = aidoku::TextFilter::default();
			filter.id = Cow::Borrowed(id);
			filter.title = Some(Cow::Borrowed(title));
			filter.placeholder = Some(Cow::Borrowed(placeholder));
			filters.push(filter.into());
		}
		Ok(filters)
	}
}

impl DeepLinkHandler for NHentai {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let Some(rest) = url.strip_prefix("https://") else {
			return Ok(None);
		};
		let Some((host, path)) = rest.split_once('/') else {
			return Ok(None);
		};
		if host != BASE_URL.trim_start_matches("https://") {
			return Ok(None);
		}

		const GALLERY_PATH: &str = "/g/";
		let Some(id_part) = path.strip_prefix(GALLERY_PATH.trim_start_matches('/')) else {
			return Ok(None);
		};
		let end = id_part.find('/').unwrap_or(id_part.len());
		let manga_id = &id_part[..end];
		if manga_id.is_empty() || !manga_id.bytes().all(|byte| byte.is_ascii_digit()) {
			return Ok(None);
		}

		Ok(Some(DeepLinkResult::Manga {
			key: manga_id.into(),
		}))
	}
}

register_source!(
	NHentai,
	Home,
	ListingProvider,
	DeepLinkHandler,
	AlternateCoverProvider,
	PageDescriptionProvider,
	DynamicFilters
);
