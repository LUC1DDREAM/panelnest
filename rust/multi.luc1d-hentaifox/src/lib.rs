#![no_std]
use aidoku::{
	Chapter, ContentRating, DeepLinkHandler, DeepLinkResult, DynamicFilters, DynamicListings,
	Filter, FilterValue, HomePartialResult, ImageRequestProvider, Listing, Manga, MangaPageResult,
	MangaStatus, Page, PageContent, Result, SelectFilter, SortFilter, Source, TextFilter, Viewer,
	alloc::{String, Vec, string::ToString, vec},
	imports::{
		html::{Document, Element},
		net::{Request, TimeUnit, set_rate_limit},
		std::send_partial_result,
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
const BASE_URL: &str = "https://hentaifox.com";
const IS_IM: bool = false;
const SIDEBAR_URL: &str = "https://hentaifox.com/includes/sidebar.php";
const SIDEBAR_LISTINGS: [(&str, &str, &str); 3] = [
	("most-faved", "Most Faved", "top_faved"),
	("most-fapped", "Most Fapped", "top_fapped"),
	("most-downloaded", "Most Downloaded", "top_downloaded"),
];
const POPULAR_TAGS_PATH: &str = "/tags/popular/";
const POPULAR_TAXONOMIES: [(&str, &str, &str); 5] = [
	("artists", "artist", "Artist"),
	("characters", "character", "Character"),
	("parodies", "parody", "Parody"),
	("groups", "group", "Group"),
	("languages", "language", "Language"),
];
const TEXT_TAXONOMIES: [(&str, &str, &str); 5] = [
	("tag-name", "tag", "Tag"),
	("artist-name", "artist", "Artist"),
	("character-name", "character", "Character"),
	("parody-name", "parody", "Parody"),
	("group-name", "group", "Group"),
];
fn key_from_url(url: &str) -> Option<String> {
	let path = url.strip_prefix(BASE_URL).unwrap_or(url);
	let path = path.split(['?', '#']).next()?;
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
fn taxonomy_url(kind: &str, slug: &str, page: i32, popular: bool) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	ensure!(
		POPULAR_TAXONOMIES
			.iter()
			.any(|(_, value, _)| *value == kind),
		"Unsupported gallery category"
	);
	ensure!(
		!slug.is_empty()
			&& slug.bytes().all(|byte| {
				byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
			}),
		"Invalid category"
	);
	Ok(if page == 1 {
		if popular {
			format!("{BASE_URL}/{kind}/{slug}/popular/")
		} else {
			format!("{BASE_URL}/{kind}/{slug}/")
		}
	} else if popular {
		format!("{BASE_URL}/{kind}/{slug}/popular/pag/{page}/")
	} else {
		format!("{BASE_URL}/{kind}/{slug}/pag/{page}/")
	})
}
fn taxonomy_text_slug(value: &str) -> Result<String> {
	let mut slug = String::new();
	for character in value.trim().chars() {
		if !character.is_ascii() {
			bail!("Enter an ASCII site name or slug");
		}
		let character = character.to_ascii_lowercase();
		if character.is_ascii_alphanumeric() || character == '.' {
			slug.push(character);
		} else if !slug.is_empty() && !slug.ends_with('-') {
			slug.push('-');
		}
	}
	while slug.ends_with('-') {
		slug.pop();
	}
	ensure!(
		!slug.is_empty() && slug != "." && slug != ".." && !slug.contains(".."),
		"Invalid taxonomy name or slug"
	);
	Ok(slug)
}
fn tag_url(slug: &str, page: i32, popular: bool) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	ensure!(
		!slug.is_empty()
			&& slug != "."
			&& slug != ".."
			&& !slug.contains("..")
			&& slug.bytes().all(|byte| {
				byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
			}),
		"Invalid tag"
	);
	let popular_path = if popular { "popular/" } else { "" };
	Ok(if page == 1 {
		format!("{BASE_URL}/tag/{slug}/{popular_path}")
	} else {
		format!("{BASE_URL}/tag/{slug}/{popular_path}pag/{page}/")
	})
}
fn search_url_with_filters(
	query: Option<&str>,
	page: i32,
	filters: &[FilterValue],
) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	let query = query.unwrap_or("").trim();
	let popular = filters.iter().any(
		|filter| matches!(filter, FilterValue::Sort { id, index: 1, .. } if id.as_str() == "sort"),
	);
	let mut selected_taxonomies = Vec::new();
	for filter in filters {
		match filter {
			FilterValue::Select { id, value } if !value.is_empty() => {
				let kind = if id == "tag" {
					Some("tag")
				} else {
					POPULAR_TAXONOMIES
						.iter()
						.find(|(_, filter_id, _)| *filter_id == id)
						.map(|(_, kind, _)| *kind)
				};
				if let Some(kind) = kind {
					selected_taxonomies.push((kind, value.as_str().to_string()));
				}
			}
			FilterValue::Text { id, value } if !value.trim().is_empty() => {
				if let Some((_, kind, _)) = TEXT_TAXONOMIES
					.iter()
					.find(|(filter_id, _, _)| *filter_id == id)
				{
					selected_taxonomies.push((*kind, taxonomy_text_slug(value)?));
				}
			}
			_ => {}
		}
	}
	ensure!(
		selected_taxonomies.len() <= 1,
		"Choose only one gallery category"
	);
	if !selected_taxonomies.is_empty() {
		let (kind, slug) = &selected_taxonomies[0];
		ensure!(query.is_empty(), "Clear text search to browse a category");
		return if *kind == "tag" {
			tag_url(slug, page, popular)
		} else {
			taxonomy_url(kind, slug, page, popular)
		};
	}
	if query.is_empty() && !popular {
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
	let url = if IS_IM {
		format!(
			"{BASE_URL}/search/?lt=1&pp=0&dl=0&tr=0&m=1&d=1&w=1&i=1&a=1&g=1&en=1&jp=1&es=1&fr=1&kr=1&de=1&ru=1&key={escaped}&page={page}"
		)
	} else {
		format!("{BASE_URL}/search/?q={escaped}&page={page}")
	};
	Ok(if popular && !IS_IM {
		format!("{url}&sort=popular")
	} else {
		url
	})
}

fn search_filters() -> Vec<Filter> {
	let mut sort = SortFilter::default();
	sort.id = "sort".into();
	sort.title = Some("Sort".into());
	sort.can_ascend = false;
	sort.options = vec!["Latest".into(), "Popular".into()];
	sort.default = Some(aidoku::SortFilterDefault {
		index: 0,
		ascending: false,
	});
	let mut filters = vec![sort.into()];
	for (id, _, title) in TEXT_TAXONOMIES {
		let mut filter = TextFilter::default();
		filter.id = id.into();
		filter.title = Some(format!("Browse {title}").into());
		filter.placeholder = Some(format!("Enter a {title} name or slug").into());
		filters.push(filter.into());
	}
	filters
}
fn taxonomy_filter(id: &'static str, title: &'static str, values: Vec<(String, String)>) -> Filter {
	let mut filter = SelectFilter::default();
	filter.id = id.into();
	filter.title = Some(title.into());
	filter.options = vec!["Any".into()];
	filter.ids = Some(vec!["".into()]);
	for (name, slug) in values {
		filter.options.push(name.into());
		filter.ids.as_mut().unwrap().push(slug.into());
	}
	filter.default = Some("".into());
	filter.into()
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
		"top-rated" => Ok(format!("{BASE_URL}/")),
		_ => bail!("Unsupported listing"),
	}
}
fn deep_link_key(url: &str) -> Option<String> {
	let rest = url.strip_prefix("https://")?;
	let (host, path) = rest.split_once('/')?;
	if host != BASE_URL.trim_start_matches("https://") || !path.starts_with("gallery/") {
		return None;
	}
	key_from_url(&format!("{BASE_URL}/{path}"))
}
fn sidebar_type(id: &str) -> Option<&'static str> {
	SIDEBAR_LISTINGS
		.iter()
		.find(|(listing_id, _, _)| *listing_id == id)
		.map(|(_, _, category)| *category)
}
fn popular_tag_slug(id: &str) -> Option<&str> {
	let slug = id.strip_prefix("popular-tag-")?;
	if !slug.is_empty()
		&& slug
			.bytes()
			.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
	{
		Some(slug)
	} else {
		None
	}
}
fn popular_tag_url(id: &str, page: i32, popular: bool) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	let slug = popular_tag_slug(id).ok_or(error!("Unsupported listing"))?;
	let popular_path = if popular { "popular/" } else { "" };
	Ok(if page == 1 {
		format!("{BASE_URL}/tag/{slug}/{popular_path}")
	} else {
		format!("{BASE_URL}/tag/{slug}/{popular_path}pag/{page}/")
	})
}
fn parse_popular_tag_listings(doc: &Document) -> Result<Vec<Listing>> {
	let mut listings = Vec::new();
	if let Some(tags) = doc.select(".tags_overview .tag_item a.tag_btn") {
		for tag in tags.into_iter().take(25) {
			let Some(href) = tag.attr("href") else {
				continue;
			};
			let Some(slug) = href
				.strip_prefix("/tag/")
				.and_then(|value| value.strip_suffix('/'))
			else {
				continue;
			};
			if slug.is_empty()
				|| !slug
					.bytes()
					.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
				|| listings.iter().any(|listing: &Listing| {
					popular_tag_slug(&listing.id).is_some_and(|existing| existing == slug)
				}) {
				continue;
			}
			let Some(name) = tag
				.select_first("h3.list_tag")
				.and_then(|el| el.text())
				.filter(|name| !name.trim().is_empty())
			else {
				continue;
			};
			listings.push(Listing {
				id: format!("popular-tag-{slug}"),
				name: format!("Tag: {name}"),
				..Default::default()
			});
		}
	}
	ensure!(
		!listings.is_empty(),
		"Popular tags unavailable or site layout changed"
	);
	Ok(listings)
}
fn parse_popular_taxonomy(doc: &Document, kind: &str) -> Result<Vec<(String, String)>> {
	ensure!(
		POPULAR_TAXONOMIES
			.iter()
			.any(|(_, value, _)| *value == kind),
		"Unsupported gallery category"
	);
	let prefix = format!("/{kind}/");
	let mut values = Vec::new();
	if let Some(tags) = doc.select(".tags_overview .tag_item a.tag_btn") {
		for tag in tags.into_iter().take(50) {
			let Some(href) = tag.attr("href") else {
				continue;
			};
			let Some(slug) = href
				.strip_prefix(&prefix)
				.and_then(|value| value.strip_suffix('/'))
			else {
				continue;
			};
			if slug.is_empty()
				|| !slug.bytes().all(|byte| {
					byte.is_ascii_lowercase()
						|| byte.is_ascii_digit()
						|| byte == b'-' || byte == b'.'
				}) {
				continue;
			}
			if values
				.iter()
				.any(|(_, existing): &(String, String)| existing == slug)
			{
				continue;
			}
			let Some(name) = tag
				.select_first("h3.list_tag")
				.and_then(|element| element.text())
				.filter(|name| !name.trim().is_empty())
			else {
				continue;
			};
			values.push((name, slug.into()));
		}
	}
	ensure!(!values.is_empty(), "Popular gallery categories unavailable");
	Ok(values)
}
fn parse_sidebar_items(doc: &Document) -> Result<MangaPageResult> {
	let entries = doc
		.select("div.item")
		.map(|els| {
			els.filter_map(|el| {
				let key = key_from_url(&el.select_first("a")?.attr("href")?)?;
				let img = el.select_first("img")?;
				let title = img.attr("alt").filter(|v| !v.trim().is_empty())?;
				Some(Manga {
					key,
					title,
					cover: image(&img),
					content_rating: ContentRating::NSFW,
					status: MangaStatus::Completed,
					viewer: Viewer::RightToLeft,
					..Default::default()
				})
			})
			.collect::<Vec<_>>()
		})
		.unwrap_or_default();
	ensure!(
		!entries.is_empty(),
		"Sidebar ranking unavailable or layout changed"
	);
	Ok(MangaPageResult {
		entries,
		has_next_page: false,
	})
}
fn fetch_sidebar_listing(category: &str) -> Result<MangaPageResult> {
	ensure!(
		SIDEBAR_LISTINGS
			.iter()
			.any(|(_, _, value)| *value == category),
		"Unsupported sidebar ranking"
	);
	let page = Request::get(format!("{BASE_URL}/"))?.html()?;
	let token = sidebar_csrf_token(&page).ok_or(error!("Missing homepage CSRF token"))?;
	post_sidebar_listing(category, &token)
}
fn sidebar_csrf_token(doc: &Document) -> Option<String> {
	doc.select_first("[name=csrf-token]")
		.and_then(|el| el.attr("content"))
		.filter(|value| !value.trim().is_empty())
}
fn post_sidebar_listing(category: &str, token: &str) -> Result<MangaPageResult> {
	ensure!(
		SIDEBAR_LISTINGS
			.iter()
			.any(|(_, _, value)| *value == category),
		"Unsupported sidebar ranking"
	);
	let response = Request::post(SIDEBAR_URL)?
		.header("X-Csrf-Token", token)
		.header("X-Requested-With", "XMLHttpRequest")
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(format!("type={category}"))
		.html()?;
	parse_sidebar_items(&response)
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
fn sidebar_home_component(id: &str, title: &str, result: MangaPageResult) -> aidoku::HomeComponent {
	aidoku::HomeComponent {
		title: Some(title.into()),
		value: aidoku::HomeComponentValue::Scroller {
			entries: result.entries.into_iter().map(Into::into).collect(),
			listing: Some(Listing {
				id: id.into(),
				name: title.into(),
				..Default::default()
			}),
		},
		..Default::default()
	}
}
fn parse_home(doc: &Document) -> Result<aidoku::HomeLayout> {
	let components = vec![latest_component(doc)?];
	let mut components = components;
	if let Ok(result) = parse_top_rated(doc) {
		components.push(aidoku::HomeComponent {
			title: Some("Top Rated".into()),
			value: aidoku::HomeComponentValue::MangaList {
				ranking: true,
				page_size: None,
				entries: result.entries.into_iter().map(Into::into).collect(),
				listing: Some(aidoku::Listing {
					id: "top-rated".into(),
					name: "Top Rated".into(),
					..Default::default()
				}),
			},
			..Default::default()
		});
	}
	Ok(aidoku::HomeLayout { components })
}
impl aidoku::Home for GallerySource {
	fn get_home(&self) -> Result<aidoku::HomeLayout> {
		let homepage = Request::get(listing_url("latest", 1)?)?.html()?;
		let mut home = parse_home(&homepage)?;
		send_partial_result(&HomePartialResult::Layout(home.clone()));
		if let Some(token) = sidebar_csrf_token(&homepage) {
			for (id, title, category) in SIDEBAR_LISTINGS {
				if let Ok(result) = post_sidebar_listing(category, &token) {
					let component = sidebar_home_component(id, title, result);
					send_partial_result(&HomePartialResult::Component(component.clone()));
					home.components.push(component);
				}
			}
		}
		Ok(home)
	}
}
impl aidoku::ListingProvider for GallerySource {
	fn get_manga_list(&self, listing: aidoku::Listing, page: i32) -> Result<MangaPageResult> {
		if popular_tag_slug(&listing.id).is_some() {
			let doc = Request::get(popular_tag_url(&listing.id, page, false)?)?.html()?;
			let result = parse_search(&doc);
			ensure!(
				!result.entries.is_empty(),
				"Tag listing unavailable or empty"
			);
			return Ok(result);
		}
		if let Some(category) = sidebar_type(&listing.id) {
			ensure!(page > 0, "Invalid page");
			return if page == 1 {
				fetch_sidebar_listing(category)
			} else {
				Ok(MangaPageResult::default())
			};
		}
		let url = listing_url(&listing.id, page)?;
		if listing.id == "top-rated" {
			if page > 1 {
				return Ok(MangaPageResult::default());
			}
			return parse_top_rated(&Request::get(url)?.html()?);
		}
		let doc = Request::get(url)?.html()?;
		let result = parse_search(&doc);
		ensure!(
			!result.entries.is_empty(),
			"Listing unavailable or site layout changed"
		);
		Ok(result)
	}
}
impl DynamicListings for GallerySource {
	fn get_dynamic_listings(&self) -> Result<Vec<Listing>> {
		let mut listings = SIDEBAR_LISTINGS
			.iter()
			.map(|(id, name, _)| Listing {
				id: (*id).into(),
				name: (*name).into(),
				..Default::default()
			})
			.collect::<Vec<_>>();
		if let Ok(response) = Request::get(format!("{BASE_URL}{POPULAR_TAGS_PATH}"))
			.and_then(|request| request.html())
		{
			if let Ok(popular_tags) = parse_popular_tag_listings(&response) {
				listings.extend(popular_tags);
			}
		}
		Ok(listings)
	}
}
impl DeepLinkHandler for GallerySource {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		Ok(deep_link_key(&url).map(|key| DeepLinkResult::Manga { key }))
	}
}
impl DynamicFilters for GallerySource {
	fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
		let mut filters = search_filters();
		for (directory, kind, title) in POPULAR_TAXONOMIES {
			let Ok(doc) = Request::get(format!("{BASE_URL}/{directory}/popular/"))
				.and_then(|request| request.html())
			else {
				continue;
			};
			let Ok(values) = parse_popular_taxonomy(&doc, kind) else {
				continue;
			};
			filters.push(taxonomy_filter(kind, title, values));
		}
		if let Ok(doc) = Request::get(format!("{BASE_URL}{POPULAR_TAGS_PATH}"))
			.and_then(|request| request.html())
		{
			if let Ok(listings) = parse_popular_tag_listings(&doc) {
				let values = listings
					.into_iter()
					.filter_map(|listing| {
						let slug = popular_tag_slug(&listing.id)?;
						let name = listing.name.strip_prefix("Tag: ")?;
						Some((name.into(), slug.into()))
					})
					.collect();
				filters.push(taxonomy_filter("tag", "Tag", values));
			}
		}
		Ok(filters)
	}
}

// The public homepage renders its default Top Rated sidebar server-side.
// Do not relabel it as daily popularity or request account-dependent rankings.
fn parse_top_rated(doc: &Document) -> Result<MangaPageResult> {
	ensure!(
		doc.select_first("#top_rated_btn.sidebar_btn_active")
			.is_some(),
		"Top Rated unavailable"
	);
	let entries = doc
		.select("#middle_sidebar div.item")
		.map(|els| {
			els.filter_map(|el| {
				let key = key_from_url(&el.select_first("a")?.attr("href")?)?;
				let img = el.select_first("img")?;
				let title = img.attr("alt").filter(|v| !v.trim().is_empty())?;
				Some(Manga {
					key,
					title,
					cover: image(&img),
					content_rating: ContentRating::NSFW,
					status: MangaStatus::Completed,
					viewer: Viewer::RightToLeft,
					..Default::default()
				})
			})
			.collect::<Vec<_>>()
		})
		.unwrap_or_default();
	ensure!(
		!entries.is_empty(),
		"Top Rated unavailable or site layout changed"
	);
	Ok(MangaPageResult {
		entries,
		has_next_page: false,
	})
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
		let doc =
			Request::get(search_url_with_filters(query.as_deref(), page, &filters)?)?.html()?;
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
aidoku::register_source!(
	GallerySource,
	ImageRequestProvider,
	Home,
	ListingProvider,
	DynamicListings,
	DeepLinkHandler,
	DynamicFilters
);

#[cfg(test)]
mod tests;
