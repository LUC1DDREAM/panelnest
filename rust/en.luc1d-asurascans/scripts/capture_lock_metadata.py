"""Capture public chapter-list metadata only (no chapter pages/images/auth).
Run manually from the source root. Fixtures retain genuine Astro field encoding.
"""
import datetime
import json
import pathlib
import re
import subprocess
from html.parser import HTMLParser

BASE = "https://asurascans.com"

class Islands(HTMLParser):
    def __init__(self):
        super().__init__()
        self.items = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "astro-island" and "ChapterListReact" in str(attrs):
            self.items.append(attrs)


def get(url):
    return subprocess.check_output(["curl", "-fsSL", "--max-time", "30", url]).decode()


def main():
    home = get(BASE)
    links = list(dict.fromkeys(re.findall(r'href="(/comics/[^"?#/]+)"', home)))[:8]
    captured = []
    for link in links:
        parser = Islands()
        parser.feed(get(BASE + link))
        for island in parser.items:
            props = json.loads(island["props"])
            chapters = props["chapters"][1][:2]
            captured.append({"url": BASE + link, "chapters": chapters})
            print(link, [(c[1].get("is_premium"), c[1].get("early_access_until")) for c in chapters])
    output = pathlib.Path("tests/fixtures/public-chapter-metadata.json")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({"captured_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "kind": "genuine public ChapterListReact props; first two rows of each series", "sources": captured}, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
