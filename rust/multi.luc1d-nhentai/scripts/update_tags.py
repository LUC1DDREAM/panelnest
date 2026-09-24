import json
import os
import time
from urllib.error import HTTPError
from urllib.request import urlopen, Request

# nhentai requires User-Agent
user_agent = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604"


def fetch_tags_from_api() -> list[tuple[str, int]]:
	"""Fetch popular tags from nhentai API v2."""
	tags: list[tuple[str, int]] = []
	page = 1
	num_pages: int | None = None
	while True:
		url = f"https://nhentai.net/api/v2/tags/tag?sort=popular&page={page}&per_page=100"
		req = Request(url, headers={"User-Agent": user_agent})
		for attempt in range(3):
			try:
				with urlopen(req, timeout=20) as response:
					data = json.load(response)
				break
			except HTTPError as exc:
				if exc.code != 429 or attempt == 2:
					raise RuntimeError(f"Failed to fetch tag page {page}: {exc}") from exc
				delay = exc.headers.get("Retry-After")
				time.sleep(min(30, max(1, int(delay) if delay and delay.isdigit() else 2**attempt)))
			except Exception as exc:
				raise RuntimeError(f"Failed to fetch tag page {page}: {exc}") from exc

		if page == 1:
			num_pages = data.get("num_pages")
			if not isinstance(num_pages, int) or num_pages < 1:
				raise RuntimeError("Tag API returned an invalid page count")

		result = data.get("result", [])
		if not result:
			raise RuntimeError(f"Tag API returned an empty page before page {num_pages}")

		for item in result:
			name = item.get("name", "").strip()
			count = item.get("count", 0)
			if not name:
				continue
			if count >= 10:
				tags.append((name, count))

		if page >= num_pages:
			break
		page += 1
		time.sleep(0.5)

	return tags


if __name__ == "__main__":
	try:
		tags = fetch_tags_from_api()
	except RuntimeError as exc:
		raise SystemExit(str(exc)) from exc

	tags.sort(key=lambda x: x[0].lower())
	popular_tags = [name for name, _ in tags]

	filters_json = os.path.join(
		os.path.dirname(os.path.realpath(__file__)), "..", "res", "filters.json"
	)
	with open(filters_json, "r", encoding="utf-8") as f:
		filters = json.load(f)
		for filter in filters:
			if filter.get("id") == "tags":
				filter["options"] = popular_tags

	with open(filters_json, "w", encoding="utf-8") as f:
		json.dump(filters, f, indent="\t", ensure_ascii=False)
		_ = f.write("\n")
