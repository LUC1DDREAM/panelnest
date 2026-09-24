#![no_std]
use aidoku::{
	Chapter, ContentRating, DeepLinkHandler, DeepLinkResult, DynamicFilters, DynamicListings,
	Filter, FilterValue, HashMap, Home, HomeComponent, HomeComponentValue, HomeLayout, Link,
	ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, MangaStatus, MangaWithChapter,
	MigrationHandler, MultiSelectFilter, NotificationHandler, Page, PageContent, RangeFilter,
	PageContext, PageDescriptionProvider, Result, Source, TextFilter, UpdateStrategy, Viewer, WebLoginHandler,
	alloc::{String, Vec, string::ToString, vec},
	helpers::uri::QueryParameters,
	imports::{
		defaults::defaults_get,
		net::{Request, TimeUnit, set_rate_limit},
	},
	prelude::*,
};

mod auth;
mod chapters;
mod discovery;
mod helpers;
mod models;

use models::*;

const BASE_URL: &str = "https://asurascans.com";
const API_URL: &str = "https://api.asurascans.com/api";

fn is_trusted_image_url(url: &str) -> bool {
	let Some(authority) = url.strip_prefix("https://") else {
		return false;
	};
	authority
		.split(['/', '?', '#'])
		.next()
		.is_some_and(|host| host == "cdn.asurascans.com")
}

fn numbered_reader_page(url: String, number: usize) -> Page {
	let mut context = PageContext::new();
	context.insert("page_number".into(), number.to_string());
	Page {
		content: PageContent::url_context(url, context),
		has_description: true,
		..Default::default()
	}
}

struct AsuraScans;

fn library_update_strategy(status: MangaStatus) -> UpdateStrategy {
	if status == MangaStatus::Completed {
		UpdateStrategy::Never
	} else {
		UpdateStrategy::Always
	}
}

fn browse_url(query: Option<&str>, page: i32, filters: &[FilterValue]) -> Result<String> {
	if page < 1 {
		return Err(error!("Invalid page"));
	}
	let mut qs = QueryParameters::new();
	qs.push("page", Some(&page.to_string()));
	if query.is_some() {
		qs.push("q", query);
	}

	for filter in filters {
		match filter {
			FilterValue::Sort {
				id,
				index,
				ascending,
			} => {
				qs.push(
					id,
					Some(match index {
						0 => "update",
						1 => "popular",
						2 => "rating",
						3 => "name",
						4 => "newest",
						_ => "update",
					}),
				);
				if *ascending {
					qs.push("order", Some("asc"));
				}
			}
			FilterValue::Select { id, value } => {
				if !value.is_empty() && !((id == "status" || id == "type") && value == "all") {
					qs.push(id, Some(value));
				}
			}
			FilterValue::MultiSelect { id, included, .. } => {
				qs.push(id, Some(&included.join(",")));
			}
			FilterValue::Range { id, from, .. } => {
				if let Some(value) = from.filter(|value| *value >= 0.0) {
					qs.push(id, Some(&value.to_string()));
				}
			}
			FilterValue::Text { id, value } => {
				if !value.trim().is_empty() && (id == "author" || id == "artist") {
					qs.push(id, Some(value.trim()));
				}
			}
			_ => continue,
		}
	}

	Ok(format!("{BASE_URL}/browse?{qs}"))
}

impl Source for AsuraScans {
	fn new() -> Self {
		set_rate_limit(2, 2, TimeUnit::Seconds);
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let url = browse_url(query.as_deref(), page, &filters)?;
		let html = Request::get(url)?.html()?;

		let entries = html
			.select("#series-grid > .series-card")
			.map(|els| {
				els.filter_map(|el| {
					Some(Manga {
						key: el
							.select_first("a")?
							.attr("abs:href")
							.and_then(|url| helpers::get_manga_key(&url))?,
						title: el.select_first("h3")?.own_text()?,
						cover: el.select_first("img").and_then(|el| el.attr("abs:src")),
						..Default::default()
					})
				})
				.collect()
			})
			.unwrap_or_default();

		let has_next_page = html
			.select_first("button[aria-label=\"Next page\"].cursor-pointer")
			.is_some();

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
		let url = helpers::get_manga_url(&manga.key);
		let html = Request::get(&url)?.html()?;

		if needs_details {
			manga.title = html
				.select_first("h1.text-xl.font-semibold")
				.and_then(|el| el.own_text())
				.unwrap_or(manga.title);
			manga.cover = html
				.select_first("div#desktop-cover-container img")
				.and_then(|el| el.attr("abs:src"));
			manga.artists = html.select("a[href^=/browse?artist]").map(|els| {
				els.filter_map(|el| el.text())
					.filter(|s| s != "_")
					.collect()
			});
			manga.authors = html.select("a[href^=/browse?author]").map(|els| {
				els.filter_map(|el| el.text())
					.filter(|s| s != "_")
					.collect()
			});
			manga.description = html
				.select_first("div#description-text")
				.and_then(|el| el.text());
			manga.url = Some(url);
			manga.tags = html
				.select("a[href^=/browse?genres=]")
				.map(|els| els.filter_map(|el| el.text()).collect());
			manga.status = html
				.select_first(
					"div.flex.gap-3.pt-4.border-t > div:nth-child(1) > div > span.text-base",
				)
				.and_then(|el| el.text())
				.map(|s| match s.as_str() {
					"ongoing" => MangaStatus::Ongoing,
					"hiatus" => MangaStatus::Hiatus,
					"completed" => MangaStatus::Completed,
					"dropped" => MangaStatus::Cancelled,
					_ => MangaStatus::Unknown,
				})
				.unwrap_or_default();
			let tags = manga.tags.as_deref().unwrap_or_default();
			manga.content_rating = if tags
				.as_ref()
				.iter()
				.any(|e| matches!(e.as_str(), "Adult" | "Ecchi"))
			{
				ContentRating::Suggestive
			} else {
				ContentRating::Safe
			};
			manga.viewer = html
				.select_first(
					"div.flex.gap-3.pt-4.border-t > div:nth-child(2) > div > span.text-base",
				)
				.and_then(|el| el.text())
				.map(|s| match s.as_str() {
					"manhwa" => Viewer::Webtoon,
					"manhua" => Viewer::Webtoon,
					"mangatoon" => Viewer::RightToLeft,
					_ => Viewer::Webtoon,
				})
				.unwrap_or(Viewer::Webtoon);
		}

		if needs_chapters {
			let island_props = html
				.select_first(
					"astro-island[component-url*=ChapterListReact], astro-island[opts*=ChapterListReact]",
				)
				.and_then(|el| el.attr("props"))
				.ok_or_else(|| error!("Missing astro-island"))?;

			let json = serde_json::from_str::<serde_json::Value>(&island_props)?;
			let chapters_arr = json["chapters"][1]
				.as_array()
				.ok_or_else(|| error!("Missing chapters"))?;

			let skip_locked = !defaults_get::<bool>("showLocked").unwrap_or(true);
			// Do not grant premium access from an expired cached subscription.
			let is_subscribed = auth::is_subscribed();

			let now = aidoku::imports::std::current_date();
			manga.chapters = Some(
				chapters_arr
					.iter()
					.filter_map(|row| {
						chapters::chapter_from_astro(row, &manga.key, is_subscribed, now)
					})
					.filter(|chapter| !skip_locked || !chapter.locked)
					.map(|chapter| chapters::with_series_thumbnail(chapter, &manga.cover))
					.collect(),
			);
		}
		manga.update_strategy = library_update_strategy(manga.status);

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Recheck the source before either reader endpoint: cached Chapter.locked
		// may be stale, and elapsed timers alone never grant premium access.
		let current = self.get_manga_update(manga.clone(), false, true)?;
		chapters::require_readable(
			current
				.chapters
				.as_ref()
				.and_then(|chapters| chapters.iter().find(|current| current.key == chapter.key)),
		)?;
		let api_url = format!("{API_URL}/series/{}/chapters/{}", manga.key, chapter.key);
		let mut api_req = Request::get(api_url)?;
		if let Ok(status) = auth::get_login_status() {
			api_req.set_header("Authorization", &format!("Bearer {}", status.access_token));
			api_req.set_header(
				"Cookie",
				&format!(
					"access_token={}; refresh_token={}",
					status.access_token, status.refresh_token
				),
			);
		}
		if let Ok(json) = api_req.json_owned::<serde_json::Value>()
			&& let Some(page_arr) = json["data"]["chapter"]["pages"].as_array()
		{
			let mut pages = Vec::new();
			for obj in page_arr {
				let Some(url) = obj
					.as_str()
					.or_else(|| obj["url"].as_str())
					.or_else(|| obj["url"][1].as_str())
				else {
					continue;
				};
				pages.push(numbered_reader_page(url.into(), pages.len() + 1));
			}
			if !pages.is_empty() {
				return Ok(pages);
			}
		}

		let url = helpers::get_chapter_url(&chapter.key, &manga.key);
		let mut req = Request::get(url)?;
		if let Ok(status) = auth::get_login_status() {
			req.set_header("Authorization", &format!("Bearer {}", status.access_token));
			req.set_header(
				"Cookie",
				&format!(
					"access_token={}; refresh_token={}",
					status.access_token, status.refresh_token
				),
			);
		}
		let html = req.html()?;

		let island_props = html
			.select_first(
				"astro-island[component-url*=ChapterReader], astro-island[opts*=ChapterReader]",
			)
			.and_then(|el| el.attr("props"))
			.ok_or_else(|| error!("Missing astro-island"))?;

		let json = serde_json::from_str::<serde_json::Value>(&island_props)?;

		let page_arr = json["pages"][1]
			.as_array()
			.ok_or_else(|| error!("Missing pages"))?;

		let mut pages = Vec::new();
		for obj in page_arr {
			let Some(url) = obj[1]["url"][1].as_str() else {
				continue;
			};
			pages.push(numbered_reader_page(url.into(), pages.len() + 1));
		}
		Ok(pages)
	}
}

impl PageDescriptionProvider for AsuraScans {
	fn get_page_description(&self, page: Page) -> Result<String> {
		let PageContent::Url(_, Some(context)) = page.content else {
			return Err(error!("Page number context missing"));
		};
		let number = context
			.get("page_number")
			.and_then(|value| value.parse::<usize>().ok())
			.filter(|number| *number > 0)
			.ok_or_else(|| error!("Invalid page number context"))?;
		Ok(format!("Page {number}"))
	}
}

fn parse_browse_genres(props: &str) -> Result<Vec<(String, String)>> {
	let props = serde_json::from_str::<serde_json::Value>(&props)?;
	let rows = props["availableGenres"][1]
		.as_array()
		.ok_or_else(|| error!("Browse genres unavailable"))?;
	let genres = rows
		.iter()
		.filter_map(|row| {
			let row = &row[1];
			Some((
				row["name"][1].as_str()?.into(),
				row["slug"][1].as_str()?.into(),
			))
		})
		.collect::<Vec<_>>();
	if genres.is_empty() {
		return Err(error!("Browse genres unavailable"));
	}
	Ok(genres)
}

fn browse_genres() -> Result<Vec<(String, String)>> {
	let html = Request::get(format!("{BASE_URL}/browse"))?.html()?;
	let props = html
		.select_first("astro-island[component-url*=BrowseFilters]")
		.and_then(|el| el.attr("props"))
		.ok_or_else(|| error!("Browse filters unavailable"))?;
	parse_browse_genres(&props)
}

fn dynamic_search_filters(genres: Vec<(String, String)>) -> Vec<Filter> {
	let mut genre = MultiSelectFilter::default();
	genre.id = "genres".into();
	genre.title = Some("Genres".into());
	genre.is_genre = true;
	genre.uses_tag_style = true;
	genre.can_exclude = false;
	genre.options = genres.iter().map(|(name, _)| name.clone().into()).collect();
	genre.ids = Some(genres.into_iter().map(|(_, slug)| slug.into()).collect());

	let mut author = TextFilter::default();
	author.id = "author".into();
	author.title = Some("Creator".into());
	author.placeholder = Some("Search creator".into());

	let mut artist = TextFilter::default();
	artist.id = "artist".into();
	artist.title = Some("Artist".into());
	artist.placeholder = Some("Search artist".into());

	let mut chapters = RangeFilter::default();
	chapters.id = "min_chapters".into();
	chapters.title = Some("Minimum Chapters".into());
	chapters.min = Some(0.0);

	let mut filters = vec![author.into(), artist.into(), chapters.into()];
	if !genre.options.is_empty() {
		filters.insert(0, genre.into());
	}
	filters
}

impl Home for AsuraScans {
	fn get_home(&self) -> Result<HomeLayout> {
		let html = Request::get(BASE_URL)?.html()?;

		let mut components = Vec::new();

		if let Some(trending_today) =
			html.select_first("astro-island[opts*=TrendingSection] > section")
		{
			let title = trending_today
				.select_first("h2")
				.and_then(|el| el.text())
				.unwrap_or("Trending Comics".into());
			let entries: Vec<Link> = trending_today
				.select("div.embla-trending > div > div > a")
				.map(|els| {
					els.filter_map(|el| {
						let key = helpers::get_manga_key(&el.attr("abs:href")?)?;
						Some(
							Manga {
								key,
								title: el.select_first("span.block")?.text()?,
								cover: el.select_first("img").and_then(|img| img.attr("abs:src")),
								..Default::default()
							}
							.into(),
						)
					})
					.collect()
				})
				.unwrap_or_default();
			if !entries.is_empty() {
				components.push(HomeComponent {
					title: Some(title),
					subtitle: None,
					value: HomeComponentValue::Scroller {
						entries,
						listing: None,
					},
				});
			}
		}

		if let Some(latest_updates) =
			html.select_first("astro-island[opts*=LatestUpdates] > section")
		{
			let title = latest_updates
				.select_first("h2")
				.and_then(|el| el.text())
				.unwrap_or("Latest Updates".into());
			let entries: Vec<MangaWithChapter> = latest_updates
				.select("div.grid > div.grid")
				.map(|els| {
					els.filter_map(|el| {
						let link = el.select_first("a.font-bold")?;
						let chapter_link = el.select_first("div.flex > div > a.group.grid")?;
						let manga_key = helpers::get_manga_key(&link.attr("abs:href")?)?;
						let chapter_key =
							helpers::get_chapter_key(&chapter_link.attr("abs:href")?)?;
						let chapter_number = chapter_link
							.select_first("span.font-medium")?
							.text()?
							.strip_prefix("Chapter")?
							.trim()
							.parse()
							.ok();
						Some(MangaWithChapter {
							manga: Manga {
								key: manga_key,
								title: link.text()?,
								cover: el.select_first("img").and_then(|img| img.attr("abs:src")),
								..Default::default()
							},
							chapter: Chapter {
								key: chapter_key,
								chapter_number,
								..Default::default()
							},
						})
					})
					.collect()
				})
				.unwrap_or_default();
			if !entries.is_empty() {
				components.push(HomeComponent {
					title: Some(title),
					subtitle: None,
					value: HomeComponentValue::MangaChapterList {
						page_size: None,
						entries,
						listing: None,
					},
				});
			}
		}

		for (id, title) in [
			("popular-today", "Popular Today"),
			("popular-week", "Popular This Week"),
			("popular-month", "Popular This Month"),
			("popular-all", "Popular All Time"),
		] {
			let listing = Listing {
				id: id.into(),
				name: title.into(),
				..Default::default()
			};
			// A failed period must never be replaced with another period's ranking.
			if let Ok(result) = self.get_manga_list(listing.clone(), 1) {
				if !result.entries.is_empty() {
					components.push(HomeComponent {
						title: Some(title.into()),
						subtitle: None,
						value: HomeComponentValue::Scroller {
							entries: result.entries.into_iter().map(Into::into).collect(),
							listing: Some(listing),
						},
					});
				}
			}
		}
		Ok(HomeLayout { components })
	}
}

impl DeepLinkHandler for AsuraScans {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		if !helpers::is_asura_url(&url) {
			return Ok(None);
		}
		let Some(manga_key) = helpers::get_manga_key(&url) else {
			return Ok(None);
		};

		if let Some(chapter_key) = helpers::get_chapter_key(&url) {
			Ok(Some(DeepLinkResult::Chapter {
				manga_key,
				key: chapter_key,
			}))
		} else {
			Ok(Some(DeepLinkResult::Manga { key: manga_key }))
		}
	}
}

impl MigrationHandler for AsuraScans {
	fn handle_manga_migration(&self, key: String) -> Result<String> {
		// v12: asuracomic.net -> asurascans.com, trailing '-' removed from ids
		Ok(key.strip_suffix("-").map(Into::into).unwrap_or(key))
	}

	fn handle_chapter_migration(&self, _manga_key: String, chapter_key: String) -> Result<String> {
		Ok(chapter_key) // no change
	}
}

impl ListingProvider for AsuraScans {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		if page < 1 {
			bail!("Invalid page");
		}
		if let Some(period) = discovery::popularity_period(&listing.id) {
			if page > 1 {
				return Ok(MangaPageResult {
					entries: Vec::new(),
					has_next_page: false,
				});
			}
			let json = Request::get(format!("{API_URL}/trending/{period}?limit=10"))?.string()?;
			return discovery::parse_popularity(&json);
		}
		match listing.id.as_str() {
			"Ranking" => {
				let html = Request::get(format!("{BASE_URL}/series-ranking"))?.html()?;
				let entries = html
					.select(".comics-ranking-list > a")
					.map(|els| {
						els.filter_map(|el| {
							// the ranking page doesn't have extra ids appended to the slug
							let key = el.attr("abs:href").and_then(|url| {
								url.split('/')
									.skip_while(|segment| *segment != "comics")
									.nth(1)
									.map(Into::into)
							})?;
							Some(Manga {
								key,
								title: el.select_first(".flex-1 > .text-sm")?.own_text()?,
								cover: el.select_first("img").and_then(|el| el.attr("abs:src")),
								..Default::default()
							})
						})
						.collect()
					})
					.unwrap_or_default();
				Ok(MangaPageResult {
					entries,
					has_next_page: false,
				})
			}
			"Bookmarks" => {
				let offset = 20 * (page - 1);
				let token = auth::get_access_token()?;
				let url = format!(
					"{API_URL}/me/bookmarks?sort=updated&order=desc&limit=20&offset={offset}",
				);
				let json: BookmarkResponse = Request::get(url)?
					.header("Authorization", &format!("Bearer {token}"))
					.json_owned()?;
				let has_next_page = json.has_next_page(offset);
				let entries = json.data.into_iter().map(Into::into).collect();
				Ok(MangaPageResult {
					entries,
					has_next_page,
				})
			}
			_ => bail!("Invalid listing"),
		}
	}
}

impl DynamicFilters for AsuraScans {
	fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
		Ok(dynamic_search_filters(browse_genres().unwrap_or_default()))
	}
}

impl DynamicListings for AsuraScans {
	fn get_dynamic_listings(&self) -> Result<Vec<Listing>> {
		if !auth::is_logged_in() {
			return Ok(Vec::new());
		}
		Ok(vec![Listing {
			id: "Bookmarks".into(),
			name: "Bookmarks".into(),
			..Default::default()
		}])
	}
}

impl WebLoginHandler for AsuraScans {
	fn handle_web_login(&self, _key: String, cookies: HashMap<String, String>) -> Result<bool> {
		auth::handle_login(cookies)
	}
}

impl NotificationHandler for AsuraScans {
	fn handle_notification(&self, notification: String) {
		if notification != "login" {
			return;
		}
		let is_logged_in = defaults_get::<String>("login").is_some();
		if !is_logged_in {
			auth::logout();
		}
	}
}

impl ImageRequestProvider for AsuraScans {
	fn get_image_request(
		&self,
		url: String,
		_context: Option<aidoku::PageContext>,
	) -> Result<Request> {
		if !is_trusted_image_url(&url) {
			return Err(error!("Unsupported Asura Scans image host"));
		}
		let referer = format!("{BASE_URL}/");
		Ok(Request::get(url)?.header("Referer", referer.as_str()))
	}
}

register_source!(
	AsuraScans,
	Home,
	DeepLinkHandler,
	MigrationHandler,
	ListingProvider,
	DynamicFilters,
	DynamicListings,
	WebLoginHandler,
	NotificationHandler,
	ImageRequestProvider,
	PageDescriptionProvider
);

#[cfg(test)]
	mod filter_tests {
	use super::*;
	use aidoku::alloc::vec;
	use aidoku::{FilterKind, FilterValue};
	use aidoku_test::aidoku_test;

	#[aidoku_test]
	fn reader_page_descriptions_follow_both_supported_page_contexts() {
		use aidoku::PageDescriptionProvider;
		let source = AsuraScans;
		let first = numbered_reader_page("https://cdn.asurascans.com/chapter/1.jpg".into(), 1);
		let ninth = numbered_reader_page("https://cdn.asurascans.com/chapter/9.jpg".into(), 9);
		assert!(first.has_description && ninth.has_description);
		assert_eq!(source.get_page_description(first).unwrap(), "Page 1");
		assert_eq!(source.get_page_description(ninth).unwrap(), "Page 9");
		assert!(source.get_page_description(Page::default()).is_err());
	}

	#[aidoku_test]
	fn browse_props_decode_current_genre_names_and_slugs() {
		let props = r#"{"availableGenres":[1,[[0,{"id":[0,1],"name":[0,"Action"],"slug":[0,"action"]}],[0,{"id":[0,4],"name":[0,"Adventure"],"slug":[0,"adventure"]}]]]}"#;
		assert_eq!(
			parse_browse_genres(props).unwrap(),
			vec![
				("Action".into(), "action".into()),
				("Adventure".into(), "adventure".into())
			]
		);
		assert!(parse_browse_genres("{}").is_err());
	}

	#[aidoku_test]
	fn browse_filters_cover_site_facets_and_map_values_to_query_parameters() {
		let filters = dynamic_search_filters(vec![("Action".into(), "action".into())]);
		assert_eq!(filters.len(), 4);
		assert_eq!(filters[0].id.as_ref(), "genres");
		assert_eq!(filters[1].id.as_ref(), "author");
		assert_eq!(filters[2].id.as_ref(), "artist");
		assert_eq!(filters[3].id.as_ref(), "min_chapters");
		match &filters[0].kind {
			FilterKind::MultiSelect { options, ids, .. } => {
				assert_eq!(options[0].as_ref(), "Action");
				assert_eq!(ids.as_ref().unwrap()[0].as_ref(), "action");
			}
			_ => panic!("Genres should be a multi-select filter"),
		}
		assert_eq!(dynamic_search_filters(vec![]).len(), 3);

		let values = vec![
			FilterValue::Sort {
				id: "sort".into(),
				index: 2,
				ascending: true,
			},
			FilterValue::Select {
				id: "status".into(),
				value: "ongoing".into(),
			},
			FilterValue::Select {
				id: "type".into(),
				value: "all".into(),
			},
			FilterValue::MultiSelect {
				id: "genres".into(),
				included: vec!["action".into(), "fantasy".into()],
				excluded: vec![],
			},
			FilterValue::Range {
				id: "min_chapters".into(),
				from: Some(10.0),
				to: None,
			},
			FilterValue::Text {
				id: "author".into(),
				value: "Daum".into(),
			},
			FilterValue::Text {
				id: "artist".into(),
				value: "Studio".into(),
			},
		];
		let url = browse_url(Some("dragon"), 2, &values).unwrap();
		assert!(url.starts_with("https://asurascans.com/browse?page=2&q=dragon"));
		assert!(url.contains("sort=rating&order=asc"));
		assert!(url.contains("status=ongoing"));
		assert!(url.contains("genres=action%2Cfantasy"));
		assert!(url.contains("min_chapters=10"));
		assert!(url.contains("author=Daum"));
		assert!(url.contains("artist=Studio"));
		assert!(!url.contains("type=all"));
		assert!(browse_url(None, 0, &[]).is_err());
	}

	#[aidoku_test]
	fn completed_manga_skip_library_refresh_but_active_statuses_keep_refreshing() {
		assert_eq!(library_update_strategy(MangaStatus::Completed), UpdateStrategy::Never);
		for status in [
			MangaStatus::Ongoing,
			MangaStatus::Hiatus,
			MangaStatus::Cancelled,
			MangaStatus::Unknown,
		] {
			assert_eq!(library_update_strategy(status), UpdateStrategy::Always);
		}
	}

	#[aidoku_test]
	fn image_requests_allow_only_the_official_https_cdn_host() {
		assert!(is_trusted_image_url(
			"https://cdn.asurascans.com/asura-images/covers/series.webp"
		));
		for url in [
			"http://cdn.asurascans.com/asura-images/covers/series.webp",
			"https://cdn.asurascans.com.evil.example/asura-images/covers/series.webp",
			"https://evil.example/asura-images/covers/series.webp",
			"https://api.asurascans.com/asura-images/covers/series.webp",
		] {
			assert!(!is_trusted_image_url(url), "{url}");
		}
	}
}
