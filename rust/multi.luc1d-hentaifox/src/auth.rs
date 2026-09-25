use aidoku::{
	HashMap, Result,
	alloc::{String, format},
	imports::{
		defaults::{DefaultValue, defaults_get, defaults_set},
		net::Request,
	},
	prelude::*,
};

const SESSION_KEY: &str = "web_session";

pub fn session_cookie(cookies: &HashMap<String, String>) -> Option<String> {
	let session_id = cookies.get("PHPSESSID")?;
	valid_session_id(session_id).then(|| format!("PHPSESSID={session_id}"))
}

fn valid_session_id(value: &str) -> bool {
	(16..=128).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

pub fn is_profile_url(url: &str) -> bool {
	let Some(path) = url.strip_prefix(crate::BASE_URL) else {
		return false;
	};
	let path = path.split(['?', '#']).next().unwrap_or(path);
	path == "/profile" || path == "/profile/"
}

fn validate_session(cookie: &str) -> Result<bool> {
	let response = Request::get(format!("{}/profile/", crate::BASE_URL))?
		.header("User-Agent", crate::USER_AGENT)
		.header("Accept-Encoding", "identity")
		.header("Cookie", cookie)
		.send()?;
	Ok(response.get_url().is_some_and(|url| is_profile_url(&url)))
}

pub fn handle_web_login(cookies: HashMap<String, String>) -> Result<bool> {
	let Some(cookie) = session_cookie(&cookies) else {
		return Ok(false);
	};
	if !validate_session(&cookie)? {
		return Ok(false);
	}
	defaults_set(SESSION_KEY, DefaultValue::String(cookie));
	Ok(true)
}

pub fn valid_session() -> Result<String> {
	let Some(cookie) = defaults_get::<String>(SESSION_KEY) else {
		bail!("Log in to HentaiFox to use account listings");
	};
	let Some(session_id) = cookie.strip_prefix("PHPSESSID=") else {
		bail!("Invalid HentaiFox session");
	};
	if !valid_session_id(session_id) {
		bail!("Invalid HentaiFox session");
	}
	if !validate_session(&cookie)? {
		bail!("HentaiFox login expired; log in again");
	}
	Ok(cookie)
}

pub fn logout() {
	defaults_set(SESSION_KEY, DefaultValue::Null);
}

pub fn is_login_setting_cleared() -> bool {
	defaults_get::<String>("login").is_none()
}

pub fn is_logged_in() -> bool {
	valid_session().is_ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use aidoku_test::aidoku_test;

	#[aidoku_test]
	fn login_cookie_requires_a_well_formed_php_session() {
		let mut cookies = HashMap::new();
		cookies.insert("PHPSESSID".into(), "l6rpbnd1noi3deb519mgbsambm".into());
		assert_eq!(
			session_cookie(&cookies).as_deref(),
			Some("PHPSESSID=l6rpbnd1noi3deb519mgbsambm")
		);
		for invalid in ["", "short", "has;delimiter;", "has space"] {
			cookies.insert("PHPSESSID".into(), invalid.into());
			assert!(session_cookie(&cookies).is_none());
		}
		cookies.insert("PHPSESSID".into(), "x".repeat(129));
		assert!(session_cookie(&cookies).is_none());
		cookies.remove("PHPSESSID");
		cookies.insert("analytics".into(), "12345678901234567890".into());
		assert!(session_cookie(&cookies).is_none());
	}

	#[aidoku_test]
	fn login_accepts_only_the_canonical_hentaifox_profile_route() {
		assert!(is_profile_url("https://hentaifox.com/profile/"));
		assert!(is_profile_url(
			"https://hentaifox.com/profile?tab=favorites"
		));
		for url in [
			"https://hentaifox.com/login/",
			"https://hentaifox.com.evil/profile/",
			"http://hentaifox.com/profile/",
			"https://example.org/profile/",
		] {
			assert!(!is_profile_url(url), "{url}");
		}
	}
}
