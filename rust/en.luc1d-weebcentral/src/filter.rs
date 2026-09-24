use crate::helper;
use crate::model::SortOptions;
use aidoku::{
	FilterValue,
	alloc::{String, Vec, borrow::ToOwned},
	helpers::uri::QueryParameters,
	prelude::*,
};

// Website Advanced Search sorts, deliberately without time-window aliases.
pub fn listing_sort(id: &str) -> Option<i32> {
	match id {
		"best-match" => Some(0),
		"alphabet" => Some(1),
		"popular" => Some(2),
		"subscribers" => Some(3),
		"new" => Some(4),
		"latest" => Some(5),
		_ => None,
	}
}

pub fn get_filters(query: Option<String>, filters: Vec<FilterValue>) -> String {
	let mut qs = QueryParameters::new();

	if let Some(query) = query {
		let cleaned = helper::remove_special_chars(query).trim().to_owned();
		if !cleaned.is_empty() {
			qs.push("text", Some(&cleaned));
		}
	}

	let mut extra_params = String::new();

	for filter in filters {
		match filter {
			FilterValue::Text { ref id, ref value } if id == "author" => {
				if !value.is_empty() {
					qs.push(id, Some(value));
				}
			}
			FilterValue::Sort {
				index, ascending, ..
			} => {
				let option: &str = SortOptions::from(index).into();
				qs.push("sort", Some(option));
				qs.push(
					"order",
					Some(if ascending { "Ascending" } else { "Descending" }),
				);
			}
			FilterValue::MultiSelect {
				ref id,
				ref included,
				ref excluded,
			} => {
				if id == "genre" {
					for tag in included {
						qs.push("included_tag", Some(tag));
					}
					for tag in excluded {
						qs.push("excluded_tag", Some(tag));
					}
				} else {
					for val in included {
						extra_params.push_str(val);
					}
				}
			}
			FilterValue::Check { ref id, value }
				if id == "official" || id == "anime" || id == "adult" =>
			{
				qs.push(
					id,
					Some(match value {
						0 => "False",
						1 => "True",
						_ => "Any",
					}),
				)
			}
			_ => {}
		}
	}

	format!("{qs}{extra_params}")
}
