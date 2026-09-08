# PanelNest website

Static Python renderer: `site_output.py`; translated content: `site-locales.json`; no runtime framework or tracking dependencies. `pipeline.py` builds packages and `site_output.py` isolates the experimental catalog and generates all website routes. Do not hand-edit dist.

Run `python -m unittest discover -s rebuild -p "test_*.py"`. Full release gate: `python rebuild/pipeline.py && python rebuild/site_output.py`. Existing fail-closed gates and all source IDs/versions/icons remain unchanged.

English static root plus `/en/`, `/de/`, `/es/`, `/fr/`, `/pt/`. Root and `/experimental/` are English aliases canonicalized to `/en/`; only aliases auto-select a matching browser language. Explicit locale links always win and persist to localStorage when available. All text is rendered without JavaScript; JS only enhances selection and copying. Catalog/package language metadata is shown as stored, not changed by UI locale.

`PANELNEST_SITE_URL` sets the absolute HTTPS canonical base, including its trailing project path; the default is https://luc1ddream.github.io/panelnest/. `GITHUB_REPOSITORY` selects issue links. The old installed my-aidoku-sources endpoint is maintained by a separate compatibility publisher. No rename is performed by this renderer. Old installed JSON URLs need complete compatibility artifacts (indexes + packages + icons), not HTML redirects.

## Original identity artwork
`site-assets/mark.svg` is an original hand-authored vector: divided comic panel above three nested V-shaped page folds, on a blue rounded square. The wordmark uses system text; no third-party wordmark or font. `create_social.py` creates the matching 1200×630 PNG locally with Pillow. No image-generation API or stock assets were used. The six source icons are the existing independently generated catalog artwork with recorded provenance in icon-provenance.json. No claim of trademark clearance.

## Honesty and privacy
All six sources were reported working by the user on Aidoku 0.9 build 3. This does not replace formal device verification; root supported catalog remains empty and manifest runtime/publish flags remain false. GitHub reports are public and explicitly warn against secrets and adult imagery. Issue forms avoid label assumptions. No analytics, external fonts, tracking cookies or adult preview images.
