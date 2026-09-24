#![no_std]
use aidoku::{
	Chapter, ContentRating, DeepLinkHandler, DeepLinkResult, DynamicFilters, DynamicListings, Filter, FilterValue,
	HomeComponent, HomeComponentValue, HomeLayout, HomePartialResult, ImageRequestProvider, Manga,
	MangaPageResult, MangaStatus, MultiSelectFilter, Page, PageContent, PageDescriptionProvider,
	Result, SortFilter, Source, TextFilter, UpdateStrategy, Viewer,
	alloc::{String, Vec, string::ToString, vec},
	imports::{
		html::{Document, Element},
		net::{Request, RequestError, Response, TimeUnit, set_rate_limit},
	},
	imports::std::send_partial_result,
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
const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1";
const IS_IM: bool = true;
fn site_request(url: String, referer: &str) -> Result<Request> {
	Ok(Request::get(url)?
		.header("Referer", referer)
		.header("User-Agent", USER_AGENT))
}
fn key_from_url(url: &str) -> Option<String> {
	let path = url.strip_prefix(BASE_URL).unwrap_or(url);
	let path = path.split(['?', '#']).next()?;
	let path = path.strip_prefix("/gallery/").or_else(|| path.strip_prefix("/view/"))?;
	let key = path.split('/').next()?;
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
					update_strategy: UpdateStrategy::Never,
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
fn deep_link_key(url: &str) -> Option<String> {
	let rest = url.strip_prefix("https://")?;
	let (host, path) = rest.split_once('/')?;
	if host != BASE_URL.trim_start_matches("https://")
		|| !(path.starts_with("gallery/") || path.starts_with("view/"))
	{
		return None;
	}
	key_from_url(&format!("{BASE_URL}/{path}"))
}
fn search_url_with_filters(
	query: Option<&str>,
	page: i32,
	filters: &[FilterValue],
) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	let query = query.unwrap_or("").trim();
	let mut sort = 1usize; // Latest matches the source's unfiltered search order.
	let mut categories: Option<Vec<String>> = None;
	let mut languages: Option<Vec<String>> = None;
	let mut advanced_terms: Vec<(bool, &str, String)> = Vec::new();
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
			FilterValue::Text { id, value } => {
				let kind = match id.as_str() {
					"tags" => "tag",
					"artists" => "artist",
					"groups" => "group",
					"parodies" => "parody",
					"characters" => "character",
					_ => continue,
				};
				for raw_term in value.split(',') {
					let raw_term = raw_term.trim();
					let (excluded, raw_term) = raw_term
						.strip_prefix('-')
						.map_or((false, raw_term), |term| (true, term.trim()));
					let term = raw_term
						.chars()
						.filter(|character| !matches!(character, '"' | '\\') && !character.is_control())
						.collect::<String>();
					let term = term.split_whitespace().collect::<Vec<_>>().join("+");
					if !term.is_empty() {
						advanced_terms.push((excluded, kind, term));
					}
				}
			}
			_ => {}
		}
	}
	ensure!(
		query.is_empty() || advanced_terms.is_empty(),
		"Use title search or advanced tag filters, not both"
	);
	if query.is_empty()
		&& sort == 1
		&& categories.is_none()
		&& languages.is_none()
		&& advanced_terms.is_empty()
	{
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
		for (id, default) in [
			("m", true),
			("d", true),
			("w", true),
			("i", true),
			("a", true),
			("g", true),
		] {
			let enabled = categories.as_ref().map_or(default, |values| {
				values.iter().any(|value| value.as_str() == id)
			});
			flags.push('&');
			flags.push_str(id);
			flags.push('=');
			flags.push(if enabled { '1' } else { '0' });
		}
		for id in ["en", "jp", "es", "fr", "kr", "de", "ru"] {
			let enabled = languages.as_ref().map_or(true, |values| {
				values.iter().any(|value| value.as_str() == id)
			});
			flags.push('&');
			flags.push_str(id);
			flags.push('=');
			flags.push(if enabled { '1' } else { '0' });
		}
		let path = if advanced_terms.is_empty() {
			"search"
		} else {
			"advsearch"
		};
		let search_key = if advanced_terms.is_empty() {
			escaped
		} else {
			let mut key = String::new();
			for (index, (excluded, kind, term)) in advanced_terms.iter().enumerate() {
				if index > 0 {
					key.push('+');
				}
				if *excluded {
					key.push('-');
				} else {
					key.push_str("%2B");
				}
				key.push_str(kind);
				key.push_str("%3A%22");
				for b in term.bytes() {
					if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
						key.push(b as char);
					} else {
						key.push_str(&format!("%{b:02X}"));
					}
				}
				key.push_str("%22");
			}
			key
		};
		format!("{BASE_URL}/{path}/?{flags}&key={search_key}&page={page}")
	} else {
		format!("{BASE_URL}/search/?q={escaped}&page={page}")
	})
}

fn discovery_filters() -> Vec<Filter> {
	let mut sort = SortFilter::default();
	sort.id = "sort".into();
	sort.title = Some("Sort".into());
	sort.can_ascend = false;
	sort.options = vec![
		"Popular".into(),
		"Latest".into(),
		"Downloads".into(),
		"Top Rated".into(),
	];
	sort.default = Some(aidoku::SortFilterDefault {
		index: 1,
		ascending: false,
	});
	let mut categories = MultiSelectFilter::default();
	categories.id = "categories".into();
	categories.title = Some("Categories".into());
	categories.options = vec![
		"Manga".into(),
		"Doujinshi".into(),
		"Western".into(),
		"Image Set".into(),
		"Artist CG".into(),
		"Game CG".into(),
	];
	categories.ids = Some(vec![
		"m".into(),
		"d".into(),
		"w".into(),
		"i".into(),
		"a".into(),
		"g".into(),
	]);
	categories.default_included = Some(vec![
		"m".into(),
		"d".into(),
		"w".into(),
		"i".into(),
		"a".into(),
		"g".into(),
	]);
	let mut languages = MultiSelectFilter::default();
	languages.id = "languages".into();
	languages.title = Some("Languages".into());
	languages.options = vec![
		"English".into(),
		"Japanese".into(),
		"Spanish".into(),
		"French".into(),
		"Korean".into(),
		"German".into(),
		"Russian".into(),
	];
	languages.ids = Some(vec![
		"en".into(),
		"jp".into(),
		"es".into(),
		"fr".into(),
		"kr".into(),
		"de".into(),
		"ru".into(),
	]);
	languages.default_included = Some(vec![
		"en".into(),
		"jp".into(),
		"es".into(),
		"fr".into(),
		"kr".into(),
		"de".into(),
		"ru".into(),
	]);
	let mut advanced = Vec::new();
	for (id, title) in [
		("tags", "Tags"),
		("artists", "Artists"),
		("groups", "Groups"),
		("parodies", "Parodies"),
		("characters", "Characters"),
	] {
		let mut filter = TextFilter::default();
		filter.id = id.into();
		filter.title = Some(title.into());
		filter.placeholder = Some(format!("Comma-separated; prefix - to exclude {title}").into());
		advanced.push(filter.into());
	}
	let mut filters = vec![sort.into(), categories.into(), languages.into()];
	filters.extend(advanced);
	filters
}
fn discovery_listings() -> Vec<aidoku::Listing> {
	let mut listings = Vec::new();
	for (id, name) in [
		("m", "Manga"),
		("d", "Doujinshi"),
		("w", "Western"),
		("i", "Image Set"),
		("a", "Artist CG"),
		("g", "Game CG"),
	] {
		listings.push(aidoku::Listing {
			id: format!("category-{id}"),
			name: format!("Category: {name}"),
			..Default::default()
		});
	}
	for (id, name) in [
		("en", "English"),
		("jp", "Japanese"),
		("es", "Spanish"),
		("fr", "French"),
		("kr", "Korean"),
		("de", "German"),
		("ru", "Russian"),
	] {
		listings.push(aidoku::Listing {
			id: format!("language-{id}"),
			name: format!("Language: {name}"),
			..Default::default()
		});
	}
	listings
}
fn update(doc: &Document, mut manga: Manga, details: bool, chapters: bool) -> Result<Manga> {
	let chapter_thumbnail = doc
		.select_first(if IS_IM { ".left_cover img" } else { ".cover img" })
		.and_then(|e| image(&e));
	manga.update_strategy = UpdateStrategy::Never;
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
		manga.update_strategy = UpdateStrategy::Never;
		manga.content_rating = ContentRating::NSFW;
		manga.viewer = Viewer::RightToLeft;
		manga.url = Some(format!("{BASE_URL}/gallery/{}/", manga.key));
	}
	if chapters {
		let reader_url = reader_url_for_gallery(doc, &manga.key)?;
		manga.chapters = Some(vec![Chapter {
			key: manga.key.clone(),
			title: Some("Gallery".into()),
			chapter_number: Some(1.0),
			language: gallery_language(doc),
			thumbnail: chapter_thumbnail,
			url: Some(reader_url),
			..Default::default()
		}]);
	}
	Ok(manga)
}
fn gallery_language(doc: &Document) -> Option<String> {
	let language_tags = doc
		.select("li")?
		.filter(|item| {
			item.select_first(".tags_text")
				.and_then(|label| label.text())
				.is_some_and(|label| label.trim() == "Languages:")
		})
		.flat_map(|item| {
			item.select("a.tag")
				.map(|tags| tags.filter_map(|tag| tag.own_text()).collect::<Vec<_>>())
				.unwrap_or_default()
		})
		.filter_map(|tag| match tag.trim().to_ascii_lowercase().as_str() {
			"english" => Some("en"),
			"japanese" => Some("ja"),
			"spanish" => Some("es"),
			"french" => Some("fr"),
			"korean" => Some("ko"),
			"german" => Some("de"),
			"russian" => Some("ru"),
			_ => None,
		})
		.collect::<Vec<_>>();
	if language_tags.len() == 1 {
		Some(language_tags[0].into())
	} else {
		None
	}
}
fn input(doc: &Document, id: &str) -> Result<String> {
	doc.select_first(&format!("input#{id}"))
		.and_then(|e| e.attr("value"))
		.filter(|s| !s.is_empty())
		.ok_or(error!("Missing reader metadata"))
}
fn reader_url_for_gallery(doc: &Document, gallery_id: &str) -> Result<String> {
	let fallback = format!("{BASE_URL}/view/{gallery_id}/1/");
	let Some(link) = doc.select_first("a[href*='/view/']") else { return Ok(fallback); };
	let Some(href) = link.attr("href") else { return Ok(fallback); };
	if !href.contains("/view/") { return Ok(fallback); }
	let url = if href.starts_with("https://") { href } else if href.starts_with("http://") { bail!("Reader URL must use HTTPS") } else if href.starts_with('/') { format!("{BASE_URL}{href}") } else { return Ok(fallback); };
	let rest = url.strip_prefix("https://").ok_or(error!("Invalid reader URL"))?;
	let (host, path) = rest.split_once('/').ok_or(error!("Invalid reader URL"))?;
	ensure!(host == BASE_URL.trim_start_matches("https://"), "Unexpected reader host");
	let path = path.split(['?', '#']).next().unwrap_or_default();
	let path = path.strip_prefix("view/").ok_or(error!("Invalid reader URL"))?;
	let mut parts = path.trim_end_matches('/').split('/');
	let id = parts.next().unwrap_or_default();
	let page = parts.next().unwrap_or_default();
	ensure!(id == gallery_id && !page.is_empty() && page.bytes().all(|b| b.is_ascii_digit()) && parts.next().is_none(), "Invalid gallery reader URL");
	Ok(format!("{BASE_URL}/view/{id}/{page}/"))
}
fn reader_url_for_chapter(gallery_id: &str, chapter: &Chapter) -> Result<String> {
	ensure!(chapter.key == gallery_id, "Invalid gallery chapter key");
	let prefix = format!("{BASE_URL}/view/{gallery_id}/");
	let Some(url) = chapter.url.as_deref() else { return Ok(format!("{prefix}1/")); };
	let Some(page) = url.strip_prefix(&prefix) else {
		ensure!(!url.contains("/view/"), "Invalid gallery reader URL");
		return Ok(format!("{prefix}1/"));
	};
	let page = page.strip_suffix('/').ok_or(error!("Invalid gallery reader URL"))?;
	ensure!(!page.is_empty() && page.bytes().all(|byte| byte.is_ascii_digit()), "Invalid gallery reader URL");
	Ok(format!("{prefix}{page}/"))
}
fn gallery_referer(gallery_id: &str) -> Result<String> {
	ensure!(
		!gallery_id.is_empty() && gallery_id.bytes().all(|byte| byte.is_ascii_digit()),
		"Invalid gallery key"
	);
	Ok(format!("{BASE_URL}/gallery/{gallery_id}/"))
}
fn parse_pages(doc: &Document) -> Result<Vec<Page>> {
	parse_pages_with_referer(doc, &format!("{BASE_URL}/view/1/1/"))
}
fn parse_pages_with_referer(doc: &Document, reader_url: &str) -> Result<Vec<Page>> {
	if doc.select_first("#gimg").is_some() {
		let current = doc
			.select_first("#gimg")
			.and_then(|el| el.attr("src"))
			.filter(|src| src.starts_with("https://"))
			.ok_or(error!("Missing current reader image"))?;
		let current_host = current
			.strip_prefix("https://")
			.and_then(|value| value.split('/').next())
			.ok_or(error!("Invalid current reader image"))?;
		let domain = BASE_URL.trim_start_matches("https://");
		ensure!(
			current_host == domain || current_host.ends_with(&format!(".{domain}")),
			"Unexpected reader image host"
		);
		let current_path = current
			.strip_prefix("https://")
			.and_then(|value| value.split_once('/').map(|(_, path)| path))
			.ok_or(error!("Invalid current reader image path"))?;
		let filename = current_path.rsplit('/').next().unwrap_or_default();
		let (current_page, _) = filename
			.split_once('.')
			.ok_or(error!("Invalid current reader image filename"))?;
		let current_page = current_page
			.parse::<usize>()
			.map_err(|_| error!("Invalid current reader page number"))?;
		ensure!(current_page > 0, "Invalid current reader image filename");
		let manifest = doc
			.select("script")
			.and_then(|scripts| {
				scripts.filter_map(|element| element.html()).find_map(|script| {
					let raw = script
						.split("$.parseJSON('")
						.nth(1)?
						.split("');")
						.next()?;
					serde_json::from_str::<serde_json::Value>(raw).ok()
				})
			})
			.ok_or(error!("Reader page manifest missing"))?;
		let manifest = manifest
			.as_object()
			.ok_or(error!("Invalid reader page manifest"))?;
		let count = manifest.len();
		ensure!(
			count > 0 && count <= 10000 && current_page <= count,
			"Invalid reader page count"
		);
		let extension = |value: &serde_json::Value| -> Result<&'static str> {
			let format = value
				.as_str()
				.and_then(|value| value.split(',').next())
				.ok_or(error!("Invalid reader page format"))?;
			match format {
				"j" => Ok("jpg"),
				"p" => Ok("png"),
				"w" => Ok("webp"),
				"g" => Ok("gif"),
				"b" => Ok("bmp"),
				_ => bail!("Unknown reader page format"),
			}
		};
		for number in 1..=count {
			extension(
				manifest
					.get(&number.to_string())
					.ok_or(error!("Incomplete reader page manifest"))?,
			)?;
		}
		let prefix = current_path
			.strip_suffix(filename)
			.ok_or(error!("Invalid current reader image path"))?;
		let mut pages = Vec::with_capacity(count);
		for number in 1..=count {
			let ext = extension(
				manifest
					.get(&number.to_string())
					.ok_or(error!("Incomplete reader page manifest"))?,
			)?;
			pages.push(Page {
				content: reader_page_content(
					format!("https://{current_host}/{prefix}{number}.{ext}"),
					reader_url,
				)?,
				has_description: true,
				..Default::default()
			});
		}
		return Ok(pages);
	}
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
			content: reader_page_content(
				format!("https://{host}/{dir}/{id}/{n}.{ext}"),
				reader_url,
			)?,
			has_description: true,
			..Default::default()
		});
	}
	Ok(pages)
}
fn reader_page_content(image_url: String, reader_url: &str) -> Result<PageContent> {
	let rest = reader_url
		.strip_prefix("https://")
		.ok_or(error!("Reader URL must use HTTPS"))?;
	let (host, path) = rest
		.split_once('/')
		.ok_or(error!("Invalid reader URL"))?;
	ensure!(host == BASE_URL.trim_start_matches("https://"), "Unexpected reader host");
	let path = path.split(['?', '#']).next().unwrap_or_default();
	let path = path.strip_prefix("view/").ok_or(error!("Invalid reader URL"))?;
	let mut parts = path.split('/');
	let gallery_id = parts.next().unwrap_or_default();
	let page = parts.next().unwrap_or_default();
	ensure!(
		!gallery_id.is_empty()
			&& gallery_id.bytes().all(|byte| byte.is_ascii_digit())
			&& !page.is_empty()
			&& page.bytes().all(|byte| byte.is_ascii_digit()),
		"Invalid reader URL"
	);
	let mut context = aidoku::PageContext::new();
	context.insert("url".into(), format!("{BASE_URL}/view/{gallery_id}/{page}/"));
	Ok(PageContent::url_context(image_url, context))
}
fn image_request_referer(url: &str, context: Option<&aidoku::PageContext>) -> Result<String> {
	let rest = url
		.strip_prefix("https://")
		.ok_or(error!("Image URL must use HTTPS"))?;
	let (host, path) = rest
		.split_once('/')
		.ok_or(error!("Invalid image URL"))?;
	let domain = BASE_URL.trim_start_matches("https://");
	ensure!(
		host == domain
			|| (host
				.strip_suffix(&format!(".{domain}"))
				.and_then(|prefix| prefix.strip_prefix('m'))
				.is_some_and(|server| !server.is_empty() && server.bytes().all(|byte| byte.is_ascii_digit()))),
		"Unexpected image host"
	);
	let path = path.split(['?', '#']).next().unwrap_or_default();
	let filename = path.rsplit('/').next().unwrap_or_default();
	let (stem, extension) = filename
		.split_once('.')
		.ok_or(error!("Invalid image filename"))?;
	ensure!(
		!stem.is_empty()
			&& stem.bytes().all(|byte| byte.is_ascii_alphanumeric())
			&& matches!(extension, "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"),
		"Invalid image filename"
	);
	let is_reader_image = path
		.rsplit('/')
		.nth(1)
		.is_some_and(|segment| segment.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
		&& !filename.starts_with("cover.");
	if is_reader_image {
		if let Some(context) = context {
			for key in ["url", "chapter_url", "page_url", "referer"] {
				if let Some(referer) = context.get(key)
					&& let Some(rest) = referer.strip_prefix("https://")
					&& let Some((referer_host, path)) = rest.split_once('/')
					&& referer_host == domain
				{
					let path = path.split(['?', '#']).next().unwrap_or_default();
					let path = path
						.strip_prefix("view/")
						.unwrap_or_default()
						.trim_end_matches('/');
					let mut parts = path.split('/');
					let gallery = parts.next().unwrap_or_default();
					let page = parts.next().unwrap_or_default();
					if !gallery.is_empty()
						&& gallery.bytes().all(|byte| byte.is_ascii_digit())
						&& !page.is_empty()
						&& page.bytes().all(|byte| byte.is_ascii_digit())
						&& parts.next().is_none()
					{
						return Ok(format!("{BASE_URL}/view/{gallery}/{page}/"));
					}
				}
			}
		}
	}
	Ok(format!("{BASE_URL}/"))
}
impl PageDescriptionProvider for GallerySource {
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
			|| !matches!(extension, "jpg" | "png" | "webp" | "gif" | "bmp")
		{
			return Err(error!("Invalid page image URL"));
		}
		Ok(format!("Page {number}"))
	}
}
fn listing_url(id: &str, page: i32) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	match id {
		"latest" => search_url(None, page),
		"popular" => Ok(if page == 1 {
			format!("{BASE_URL}/popular/")
		} else {
			format!("{BASE_URL}/popular/{page}/")
		}),
		"top-rated" => Ok(if page == 1 {
			format!("{BASE_URL}/top-rated/")
		} else {
			format!("{BASE_URL}/top-rated/{page}/")
		}),
		"downloaded" => Ok(if page == 1 {
			format!("{BASE_URL}/downloaded/")
		} else {
			format!("{BASE_URL}/downloaded/{page}/")
		}),
		_ => dynamic_listing_url(id, page),
	}
}
fn dynamic_listing_url(id: &str, page: i32) -> Result<String> {
	ensure!(page > 0, "Invalid page");
	let filters = if let Some(category) = id.strip_prefix("category-") {
		ensure!(
			["m", "d", "w", "i", "a", "g"].contains(&category),
			"Unsupported listing"
		);
		vec![FilterValue::MultiSelect {
			id: "categories".into(),
			included: vec![category.into()],
			excluded: Vec::new(),
		}]
	} else if let Some(language) = id.strip_prefix("language-") {
		ensure!(
			["en", "jp", "es", "fr", "kr", "de", "ru"].contains(&language),
			"Unsupported listing"
		);
		vec![FilterValue::MultiSelect {
			id: "languages".into(),
			included: vec![language.into()],
			excluded: Vec::new(),
		}]
	} else {
		bail!("Unsupported listing")
	};
	search_url_with_filters(None, page, &filters)
}
fn listing_component_from_result(
	id: &str,
	title: &str,
	result: MangaPageResult,
) -> HomeComponent {
	HomeComponent {
		title: Some(title.into()),
		value: HomeComponentValue::Scroller {
			entries: result.entries.into_iter().map(Into::into).collect(),
			listing: Some(aidoku::Listing {
				id: id.into(),
				name: title.into(),
				..Default::default()
			}),
		},
		..Default::default()
	}
}
fn home_layout() -> HomeLayout {
	HomeLayout {
		components: vec![
			HomeComponent {
				title: Some("Latest".into()),
				value: HomeComponentValue::Scroller {
					entries: Vec::new(),
					listing: Some(aidoku::Listing {
						id: "latest".into(),
						name: "Latest".into(),
						..Default::default()
					}),
				},
				..Default::default()
			},
			HomeComponent {
				title: Some("Popular".into()),
				value: HomeComponentValue::Scroller {
					entries: Vec::new(),
					listing: Some(aidoku::Listing {
						id: "popular".into(),
						name: "Popular".into(),
						..Default::default()
					}),
				},
				..Default::default()
			},
			HomeComponent {
				title: Some("Top Rated".into()),
				value: HomeComponentValue::Scroller {
					entries: Vec::new(),
					listing: Some(aidoku::Listing {
						id: "top-rated".into(),
						name: "Top Rated".into(),
						..Default::default()
					}),
				},
				..Default::default()
			},
			HomeComponent {
				title: Some("Downloaded".into()),
				value: HomeComponentValue::Scroller {
					entries: Vec::new(),
					listing: Some(aidoku::Listing {
						id: "downloaded".into(),
						name: "Downloaded".into(),
						..Default::default()
					}),
				},
				..Default::default()
			},
		],
	}
}
fn update_home_component(home: &mut HomeLayout, component: HomeComponent) {
	if let HomeComponentValue::Scroller { listing, .. } = &component.value
		&& let Some(listing) = listing
		&& let Some(item) = home.components.iter_mut().find(|item| {
			matches!(&item.value, HomeComponentValue::Scroller { listing: Some(existing), .. } if existing.id == listing.id)
		})
	{
		*item = component;
	}
}
impl aidoku::Home for GallerySource {
	fn get_home(&self) -> Result<HomeLayout> {
		let mut home = home_layout();
		send_partial_result(&HomePartialResult::Layout(home.clone()));
		let requests = ["latest", "popular", "top-rated", "downloaded"]
			.map(|id| {
				let url = listing_url(id, 1)?;
				site_request(url, &format!("{BASE_URL}/"))
			})
			.into_iter()
			.collect::<Result<Vec<_>>>()?;
		let responses: [core::result::Result<Response, RequestError>; 4] = Request::send_all(requests)
			.try_into()
			.expect("request count matches home feeds");
		let results = responses.map(|response| {
			response.and_then(|response| response.get_html().map(|doc| parse_search(&doc)))
		});
		let [latest, popular, top_rated, downloaded] = results;
		let latest = latest.map_err(|_| error!("Latest unavailable or site layout changed"))?;
		ensure!(
			!latest.entries.is_empty(),
			"Latest unavailable or site layout changed"
		);
		let component = listing_component_from_result("latest", "Latest", latest);
		update_home_component(&mut home, component.clone());
		send_partial_result(&HomePartialResult::Component(component));

		for (id, title, result) in [
			("popular", "Popular", popular),
			("top-rated", "Top Rated", top_rated),
			("downloaded", "Downloaded", downloaded),
		]
		{
			if let Ok(result) = result {
				if !result.entries.is_empty() {
					let component = listing_component_from_result(id, title, result);
					update_home_component(&mut home, component.clone());
					send_partial_result(&HomePartialResult::Component(component));
				}
			}
		}
		Ok(home)
	}
}
impl DynamicFilters for GallerySource {
	fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
		Ok(discovery_filters())
	}
}
impl DeepLinkHandler for GallerySource {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		Ok(deep_link_key(&url).map(|key| DeepLinkResult::Manga { key }))
	}
}
impl aidoku::ListingProvider for GallerySource {
	fn get_manga_list(&self, listing: aidoku::Listing, page: i32) -> Result<MangaPageResult> {
		let url = listing_url(&listing.id, page)?;

		let doc = site_request(url, &format!("{BASE_URL}/"))?.html()?;
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
		let doc = site_request(
			search_url_with_filters(query.as_deref(), page, &filters)?,
			&format!("{BASE_URL}/"),
		)?
		.html()?;
		ensure!(
			doc.select_first("div.thumb, .pagination, .content, .container")
				.is_some(),
			"Search unavailable or layout changed"
		);
		Ok(parse_search(&doc))
	}
	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		manga.update_strategy = UpdateStrategy::Never;
		if !needs_details && !needs_chapters {
			return Ok(manga);
		}
		ensure!(
			!manga.key.is_empty() && manga.key.bytes().all(|b| b.is_ascii_digit()),
			"Invalid gallery key"
		);
		let doc = site_request(
			format!("{BASE_URL}/gallery/{}/", manga.key),
			&format!("{BASE_URL}/"),
		)?
		.html()?;
		update(&doc, manga, needs_details, needs_chapters)
	}
	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		ensure!(!manga.key.is_empty() && manga.key.bytes().all(|b| b.is_ascii_digit()), "Invalid gallery key");
		let reader_url = reader_url_for_chapter(&manga.key, &chapter)?;
		let referer = gallery_referer(&manga.key)?;
		let reader_doc = site_request(reader_url.clone(), referer.as_str())?.html()?;
		ensure!(reader_doc.select_first("#gimg, input#load_id").is_some(), "Reader unavailable or site layout changed");
		parse_pages_with_referer(&reader_doc, &reader_url)
	}
}
impl DynamicListings for GallerySource {
	fn get_dynamic_listings(&self) -> Result<Vec<aidoku::Listing>> {
		Ok(discovery_listings())
	}
}
impl ImageRequestProvider for GallerySource {
	fn get_image_request(
		&self,
		url: String,
		context: Option<aidoku::PageContext>,
	) -> Result<Request> {
		let referer = image_request_referer(&url, context.as_ref())?;
		Ok(Request::get(url)?
			.header("Referer", referer.as_str())
			.header("User-Agent", USER_AGENT))
	}
}
aidoku::register_source!(
	GallerySource,
	ImageRequestProvider,
	Home,
	ListingProvider,
	DynamicFilters,
	DynamicListings,
	DeepLinkHandler,
	PageDescriptionProvider
);

#[cfg(test)]
mod tests;
